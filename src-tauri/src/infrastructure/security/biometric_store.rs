use serde::{Deserialize, Serialize};

use crate::support::result::AppResult;

const BIOMETRIC_BUNDLE_VERSION: u8 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BiometricUnlockBundle {
    pub version: u8,
    pub account_id: String,
    pub enc_key_b64: String,
    pub mac_key_b64: Option<String>,
}

impl BiometricUnlockBundle {
    pub fn new(account_id: String, enc_key_b64: String, mac_key_b64: Option<String>) -> Self {
        Self {
            version: BIOMETRIC_BUNDLE_VERSION,
            account_id,
            enc_key_b64,
            mac_key_b64,
        }
    }
}

pub fn is_supported() -> bool {
    imp::is_supported()
}

pub fn save_unlock_bundle(account_id: &str, bundle: &BiometricUnlockBundle) -> AppResult<()> {
    imp::save_unlock_bundle(account_id, bundle)
}

pub fn load_unlock_bundle(account_id: &str) -> AppResult<BiometricUnlockBundle> {
    imp::load_unlock_bundle(account_id)
}

pub fn has_unlock_bundle(account_id: &str) -> AppResult<bool> {
    imp::has_unlock_bundle(account_id)
}

pub fn delete_unlock_bundle(account_id: &str) -> AppResult<()> {
    imp::delete_unlock_bundle(account_id)
}

#[cfg(target_os = "macos")]
mod imp {
    use core_foundation::base::TCFType;
    use core_foundation::string::CFString;
    use objc2::{extern_class, extern_methods, rc::Retained, runtime::NSObject};
    use objc2_foundation::{NSError, NSInteger};
    use security_framework::base::Error;
    use security_framework::passwords::{
        delete_generic_password_options, generic_password, set_generic_password_options,
        AccessControlOptions, PasswordOptions,
    };
    use security_framework_sys::base::{errSecAuthFailed, errSecItemNotFound};
    use security_framework_sys::item::{kSecUseAuthenticationUI, kSecUseAuthenticationUISkip};

    use crate::infrastructure::security::biometric_store::BiometricUnlockBundle;
    use crate::support::error::AppError;
    use crate::support::result::AppResult;

    const KEYCHAIN_SERVICE: &str = "com.ryuyb.vanguard.biometric.unlock";
    const KEYCHAIN_LABEL: &str = "Vanguard Biometric Unlock";
    const KEYCHAIN_DESCRIPTION: &str = "Biometric vault unlock bundle";

    const ERR_SEC_USER_CANCELED: i32 = -128;
    const ERR_SEC_INTERACTION_NOT_ALLOWED: i32 = -25_308;
    const ERR_SEC_MISSING_ENTITLEMENT: i32 = -34_018;

    const LA_POLICY_DEVICE_OWNER_AUTHENTICATION_WITH_BIOMETRICS: NSInteger = 1;
    #[link(name = "LocalAuthentication", kind = "framework")]
    unsafe extern "C" {}

    extern_class!(
        #[unsafe(super(NSObject))]
        #[derive(PartialEq, Eq, Hash)]
        struct LAContext;
    );

    impl LAContext {
        extern_methods!(
            #[unsafe(method(new))]
            fn new() -> Retained<Self>;

            #[unsafe(method(canEvaluatePolicy:error:))]
            fn can_evaluate_policy(
                &self,
                policy: NSInteger,
                error: Option<&mut Option<Retained<NSError>>>,
            ) -> bool;
        );
    }

    pub fn is_supported() -> bool {
        let context = LAContext::new();
        let mut error = None;

        let supported = context.can_evaluate_policy(
            LA_POLICY_DEVICE_OWNER_AUTHENTICATION_WITH_BIOMETRICS,
            Some(&mut error),
        );

        if !supported {
            if let Some(error) = error {
                log::debug!(
                    target: "vanguard::infrastructure::security::biometric_store",
                    "biometric support unavailable code={} description={}",
                    error.code(),
                    error.localizedDescription()
                );
            } else {
                log::debug!(
                    target: "vanguard::infrastructure::security::biometric_store",
                    "biometric support unavailable without LocalAuthentication error"
                );
            }
        }

        supported
    }

    pub fn save_unlock_bundle(account_id: &str, bundle: &BiometricUnlockBundle) -> AppResult<()> {
        let account_key = normalize_account_id(account_id)?;
        if bundle.account_id.trim() != account_key {
            return Err(AppError::ValidationFieldError {
                field: "unknown".to_string(),
                message: "biometric bundle account_id mismatch".to_string(),
            });
        }
        let serialized =
            serde_json::to_string(bundle).map_err(|error| AppError::InternalUnexpected {
                message: format!("failed to serialize biometric bundle: {error}"),
            })?;

        let mut delete_options = options_for_account(&account_key);
        delete_options.set_access_synchronized(Some(false));
        if let Err(error) = delete_generic_password_options(delete_options) {
            if error.code() != errSecItemNotFound {
                return Err(map_keychain_error(
                    "failed to clear old biometric keychain entry",
                    error,
                ));
            }
        }

        let mut options = options_for_account(&account_key);
        options.set_access_synchronized(Some(false));
        options.set_access_control_options(AccessControlOptions::BIOMETRY_CURRENT_SET);
        options.set_label(KEYCHAIN_LABEL);
        options.set_description(KEYCHAIN_DESCRIPTION);

        set_generic_password_options(serialized.as_bytes(), options)
            .map_err(|error| map_keychain_error("failed to save biometric keychain entry", error))
    }

    pub fn load_unlock_bundle(account_id: &str) -> AppResult<BiometricUnlockBundle> {
        let account_key = normalize_account_id(account_id)?;
        let mut options = options_for_account(&account_key);
        options.set_access_synchronized(Some(false));

        match generic_password(options) {
            Ok(value) => {
                let text =
                    String::from_utf8(value).map_err(|error| AppError::InternalUnexpected {
                        message: format!(
                            "biometric keychain entry contains non-utf8 data: {error}"
                        ),
                    })?;
                let bundle = serde_json::from_str::<BiometricUnlockBundle>(&text).map_err(|error| {
                    AppError::ValidationFieldError {
                        field: "unknown".to_string(),
                        message: format!(
                            "biometric keychain entry is invalid or legacy format, please disable and re-enable touch id: {error}"
                        ),
                    }
                })?;
                Ok(bundle)
            }
            Err(error) if error.code() == errSecItemNotFound => {
                Err(AppError::ValidationFieldError {
                    field: "unknown".to_string(),
                    message: "biometric unlock is not configured for this account".to_string(),
                })
            }
            Err(error) if error.code() == ERR_SEC_USER_CANCELED => {
                Err(AppError::ValidationFieldError {
                    field: "unknown".to_string(),
                    message: "biometric authentication was cancelled".to_string(),
                })
            }
            Err(error) if error.code() == errSecAuthFailed => Err(AppError::ValidationFieldError {
                field: "unknown".to_string(),
                message: "biometric authentication failed".to_string(),
            }),
            Err(error) => Err(map_keychain_error(
                "failed to read biometric keychain entry",
                error,
            )),
        }
    }

    pub fn has_unlock_bundle(account_id: &str) -> AppResult<bool> {
        let account_key = normalize_account_id(account_id)?;
        let mut options = options_for_account(&account_key);
        options.set_access_synchronized(Some(false));
        #[allow(deprecated)]
        options.query.push((
            unsafe { CFString::wrap_under_get_rule(kSecUseAuthenticationUI) },
            unsafe { CFString::wrap_under_get_rule(kSecUseAuthenticationUISkip) }.into_CFType(),
        ));

        match generic_password(options) {
            Ok(_) => Ok(true),
            Err(error) if error.code() == errSecItemNotFound => Ok(false),
            Err(error) if error.code() == ERR_SEC_INTERACTION_NOT_ALLOWED => Ok(true),
            Err(error) => Err(map_keychain_error(
                "failed to inspect biometric keychain entry",
                error,
            )),
        }
    }

    pub fn delete_unlock_bundle(account_id: &str) -> AppResult<()> {
        let account_key = normalize_account_id(account_id)?;
        let mut options = options_for_account(&account_key);
        options.set_access_synchronized(Some(false));

        match delete_generic_password_options(options) {
            Ok(()) => Ok(()),
            Err(error) if error.code() == errSecItemNotFound => Ok(()),
            Err(error) => Err(map_keychain_error(
                "failed to delete biometric keychain entry",
                error,
            )),
        }
    }

    fn options_for_account(account_id: &str) -> PasswordOptions {
        PasswordOptions::new_generic_password(KEYCHAIN_SERVICE, account_id)
    }

    fn normalize_account_id(value: &str) -> AppResult<String> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(AppError::ValidationFieldError {
                field: "unknown".to_string(),
                message: "account_id is empty, cannot use biometric unlock".to_string(),
            });
        }
        Ok(String::from(trimmed))
    }

    fn map_keychain_error(context: &str, error: Error) -> AppError {
        match error.code() {
            ERR_SEC_MISSING_ENTITLEMENT => AppError::ValidationFieldError {
                field: "unknown".to_string(),
                message: "touch id requires a signed macOS app with keychain entitlements (errSecMissingEntitlement, -34018). tauri dev's unsigned runtime cannot use biometric keychain items; run a signed .app build instead".to_string(),
            },
            _ => AppError::InternalUnexpected {
                message: format!("{context}: status={} error={error}", error.code()),
            },
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    use crate::infrastructure::security::biometric_store::BiometricUnlockBundle;
    use crate::support::error::AppError;
    use crate::support::result::AppResult;

    pub fn is_supported() -> bool {
        false
    }

    pub fn save_unlock_bundle(_account_id: &str, _bundle: &BiometricUnlockBundle) -> AppResult<()> {
        Err(AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: "biometric unlock is only supported on macOS".to_string(),
        })
    }

    pub fn load_unlock_bundle(_account_id: &str) -> AppResult<BiometricUnlockBundle> {
        Err(AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: "biometric unlock is only supported on macOS".to_string(),
        })
    }

    pub fn has_unlock_bundle(_account_id: &str) -> AppResult<bool> {
        Ok(false)
    }

    pub fn delete_unlock_bundle(_account_id: &str) -> AppResult<()> {
        Ok(())
    }
}
