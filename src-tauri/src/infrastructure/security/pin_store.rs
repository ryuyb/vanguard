use serde::{Deserialize, Serialize};

use crate::domain::unlock::PinProtectedUserKeyEnvelope;
use crate::support::result::AppResult;

const PERSISTENT_PIN_ENVELOPE_VERSION: u8 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistentPinEnvelope {
    pub version: u8,
    pub account_id: String,
    pub envelope: PinProtectedUserKeyEnvelope,
}

impl PersistentPinEnvelope {
    pub fn new(account_id: String, envelope: PinProtectedUserKeyEnvelope) -> Self {
        Self {
            version: PERSISTENT_PIN_ENVELOPE_VERSION,
            account_id,
            envelope,
        }
    }
}

pub fn is_supported() -> bool {
    cfg!(target_os = "macos")
}

pub fn save_persistent_pin_envelope(
    account_id: &str,
    envelope: &PersistentPinEnvelope,
) -> AppResult<()> {
    imp::save_persistent_pin_envelope(account_id, envelope)
}

pub fn load_persistent_pin_envelope(account_id: &str) -> AppResult<PersistentPinEnvelope> {
    imp::load_persistent_pin_envelope(account_id)
}

pub fn has_persistent_pin_envelope(account_id: &str) -> AppResult<bool> {
    imp::has_persistent_pin_envelope(account_id)
}

pub fn delete_persistent_pin_envelope(account_id: &str) -> AppResult<()> {
    imp::delete_persistent_pin_envelope(account_id)
}

#[cfg(target_os = "macos")]
mod imp {
    use security_framework::base::Error;
    use security_framework::passwords::{
        delete_generic_password_options, generic_password, set_generic_password_options,
        PasswordOptions,
    };
    use security_framework_sys::base::errSecItemNotFound;

    use crate::infrastructure::security::pin_store::PersistentPinEnvelope;
    use crate::support::error::AppError;
    use crate::support::result::AppResult;

    const KEYCHAIN_SERVICE: &str = "com.ryuyb.vanguard.pin.unlock";
    const KEYCHAIN_LABEL: &str = "Vanguard PIN Unlock";
    const KEYCHAIN_DESCRIPTION: &str = "PIN-protected vault unlock envelope";

    const ERR_SEC_MISSING_ENTITLEMENT: i32 = -34_018;

    pub fn save_persistent_pin_envelope(
        account_id: &str,
        envelope: &PersistentPinEnvelope,
    ) -> AppResult<()> {
        let account_key = normalize_account_id(account_id)?;
        if envelope.account_id.trim() != account_key {
            return Err(AppError::ValidationFieldError {
                field: "unknown".to_string(),
                message: "pin envelope account_id mismatch".to_string(),
            });
        }
        let serialized =
            serde_json::to_string(envelope).map_err(|error| AppError::InternalUnexpected {
                message: format!("failed to serialize pin envelope: {error}"),
            })?;

        let mut delete_options = options_for_account(&account_key);
        delete_options.set_access_synchronized(Some(false));
        if let Err(error) = delete_generic_password_options(delete_options) {
            if error.code() != errSecItemNotFound {
                return Err(map_keychain_error(
                    "failed to clear old pin keychain entry",
                    error,
                ));
            }
        }

        let mut options = options_for_account(&account_key);
        options.set_access_synchronized(Some(false));
        options.set_label(KEYCHAIN_LABEL);
        options.set_description(KEYCHAIN_DESCRIPTION);

        set_generic_password_options(serialized.as_bytes(), options)
            .map_err(|error| map_keychain_error("failed to save pin keychain entry", error))
    }

    pub fn load_persistent_pin_envelope(account_id: &str) -> AppResult<PersistentPinEnvelope> {
        let account_key = normalize_account_id(account_id)?;
        let mut options = options_for_account(&account_key);
        options.set_access_synchronized(Some(false));

        match generic_password(options) {
            Ok(value) => {
                let text =
                    String::from_utf8(value).map_err(|error| AppError::InternalUnexpected {
                        message: format!("pin keychain entry contains non-utf8 data: {error}"),
                    })?;
                let payload = serde_json::from_str::<PersistentPinEnvelope>(&text).map_err(|error| {
                    AppError::ValidationFieldError {
                        field: "unknown".to_string(),
                        message: format!(
                            "pin keychain entry is invalid or legacy format, please disable and re-enable pin unlock: {error}"
                        ),
                    }
                })?;
                Ok(payload)
            }
            Err(error) if error.code() == errSecItemNotFound => {
                Err(AppError::ValidationFieldError {
                    field: "unknown".to_string(),
                    message: "persistent pin unlock is not configured for this account".to_string(),
                })
            }
            Err(error) => Err(map_keychain_error(
                "failed to read pin keychain entry",
                error,
            )),
        }
    }

    pub fn has_persistent_pin_envelope(account_id: &str) -> AppResult<bool> {
        let account_key = normalize_account_id(account_id)?;
        let mut options = options_for_account(&account_key);
        options.set_access_synchronized(Some(false));

        match generic_password(options) {
            Ok(_) => Ok(true),
            Err(error) if error.code() == errSecItemNotFound => Ok(false),
            Err(error) => Err(map_keychain_error(
                "failed to inspect pin keychain entry",
                error,
            )),
        }
    }

    pub fn delete_persistent_pin_envelope(account_id: &str) -> AppResult<()> {
        let account_key = normalize_account_id(account_id)?;
        let mut options = options_for_account(&account_key);
        options.set_access_synchronized(Some(false));

        match delete_generic_password_options(options) {
            Ok(()) => Ok(()),
            Err(error) if error.code() == errSecItemNotFound => Ok(()),
            Err(error) => Err(map_keychain_error(
                "failed to delete pin keychain entry",
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
                message: "account_id is empty, cannot use pin unlock".to_string(),
            });
        }
        Ok(String::from(trimmed))
    }

    fn map_keychain_error(context: &str, error: Error) -> AppError {
        match error.code() {
            ERR_SEC_MISSING_ENTITLEMENT => AppError::ValidationFieldError {
                field: "unknown".to_string(),
                message: "pin unlock requires a signed macOS app with keychain entitlements (errSecMissingEntitlement, -34018). tauri dev's unsigned runtime cannot use persistent pin items; run a signed .app build instead".to_string(),
            },
            _ => AppError::InternalUnexpected {
                message: format!("{context}: status={} error={error}", error.code()),
            },
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    use crate::infrastructure::security::pin_store::PersistentPinEnvelope;
    use crate::support::error::AppError;
    use crate::support::result::AppResult;

    pub fn save_persistent_pin_envelope(
        _account_id: &str,
        _envelope: &PersistentPinEnvelope,
    ) -> AppResult<()> {
        Err(AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: "pin unlock is only supported on macOS".to_string(),
        })
    }

    pub fn load_persistent_pin_envelope(_account_id: &str) -> AppResult<PersistentPinEnvelope> {
        Err(AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: "pin unlock is only supported on macOS".to_string(),
        })
    }

    pub fn has_persistent_pin_envelope(_account_id: &str) -> AppResult<bool> {
        Ok(false)
    }

    pub fn delete_persistent_pin_envelope(_account_id: &str) -> AppResult<()> {
        Ok(())
    }
}
