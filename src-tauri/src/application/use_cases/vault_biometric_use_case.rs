use std::sync::Arc;

use async_trait::async_trait;
use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine;

use crate::application::dto::vault::{
    UnlockVaultResult, VaultBiometricBundle, VaultBiometricStatus, VaultUserKeyMaterial,
};
use crate::application::ports::biometric_unlock_port::BiometricUnlockPort;
use crate::application::ports::master_password_unlock_data_port::MasterPasswordUnlockDataPort;
use crate::application::ports::vault_runtime_port::VaultRuntimePort;
use crate::application::use_cases::unlock_vault_use_case::BiometricUnlockExecutor;
use crate::application::vault_crypto;
use crate::support::error::AppError;
use crate::support::result::AppResult;

#[derive(Clone)]
pub struct VaultBiometricUseCase {
    master_password_unlock_data_port: Arc<dyn MasterPasswordUnlockDataPort>,
    biometric_unlock_port: Arc<dyn BiometricUnlockPort>,
}

impl VaultBiometricUseCase {
    pub fn new(
        master_password_unlock_data_port: Arc<dyn MasterPasswordUnlockDataPort>,
        biometric_unlock_port: Arc<dyn BiometricUnlockPort>,
    ) -> Self {
        Self {
            master_password_unlock_data_port,
            biometric_unlock_port,
        }
    }

    pub async fn biometric_status(
        &self,
        runtime: &dyn VaultRuntimePort,
    ) -> AppResult<VaultBiometricStatus> {
        if !self.biometric_unlock_port.is_supported() {
            return Ok(VaultBiometricStatus {
                supported: false,
                enabled: false,
            });
        }

        let account_id = match runtime.active_account_id() {
            Ok(value) => value,
            Err(
                AppError::ValidationFieldError { .. }
                | AppError::ValidationFormatError { .. }
                | AppError::ValidationRequired { .. },
            ) => {
                return Ok(VaultBiometricStatus {
                    supported: true,
                    enabled: false,
                });
            }
            Err(error) => return Err(error),
        };

        let enabled = self.biometric_unlock_port.has_unlock_bundle(&account_id)?;
        Ok(VaultBiometricStatus {
            supported: true,
            enabled,
        })
    }

    pub async fn can_unlock_with_biometric(
        &self,
        runtime: &dyn VaultRuntimePort,
    ) -> AppResult<bool> {
        if !self.biometric_unlock_port.is_supported() {
            return Ok(false);
        }

        let account_id = match runtime.active_account_id() {
            Ok(value) => value,
            Err(
                AppError::ValidationFieldError { .. }
                | AppError::ValidationFormatError { .. }
                | AppError::ValidationRequired { .. },
            ) => return Ok(false),
            Err(error) => return Err(error),
        };

        if self
            .master_password_unlock_data_port
            .load_master_password_unlock_data(&account_id)
            .await?
            .is_none()
        {
            return Ok(false);
        }

        self.biometric_unlock_port.has_unlock_bundle(&account_id)
    }

    pub fn enable_biometric_unlock(&self, runtime: &dyn VaultRuntimePort) -> AppResult<()> {
        if !self.biometric_unlock_port.is_supported() {
            return Err(AppError::ValidationFieldError {
                field: "unknown".to_string(),
                message: "biometric unlock is only supported on macOS".to_string(),
            });
        }

        let account_id = runtime.active_account_id()?;
        let user_key = runtime
            .get_vault_user_key_material(&account_id)?
            .ok_or_else(|| AppError::ValidationFieldError {
                field: "unknown".to_string(),
                message: "vault is locked, please unlock with password before enabling touch id"
                    .to_string(),
            })?;

        // Get refresh_token from runtime for session restoration
        let refresh_token = runtime.get_refresh_token()?;

        let bundle = vault_user_key_to_biometric_bundle(&account_id, &user_key, refresh_token)?;
        self.biometric_unlock_port
            .save_unlock_bundle(&account_id, &bundle)?;
        let verified_bundle = self.biometric_unlock_port.load_unlock_bundle(&account_id)?;
        if verified_bundle.account_id != account_id {
            return Err(AppError::InternalUnexpected {
                message: "biometric verification returned mismatched account id".into(),
            });
        }

        log::info!(
            target: "vanguard::application::vault_biometric",
            "biometric unlock enabled account_id={}",
            account_id
        );
        Ok(())
    }

    pub fn disable_biometric_unlock(&self, runtime: &dyn VaultRuntimePort) -> AppResult<()> {
        if !self.biometric_unlock_port.is_supported() {
            return Ok(());
        }

        let account_id = match runtime.active_account_id() {
            Ok(value) => value,
            Err(
                AppError::ValidationFieldError { .. }
                | AppError::ValidationFormatError { .. }
                | AppError::ValidationRequired { .. },
            ) => return Ok(()),
            Err(error) => return Err(error),
        };

        self.biometric_unlock_port
            .delete_unlock_bundle(&account_id)?;
        log::info!(
            target: "vanguard::application::vault_biometric",
            "biometric unlock disabled account_id={}",
            account_id
        );
        Ok(())
    }

    pub fn unlock_with_biometric(
        &self,
        runtime: &dyn VaultRuntimePort,
    ) -> AppResult<UnlockVaultResult> {
        if !self.biometric_unlock_port.is_supported() {
            return Err(AppError::ValidationFieldError {
                field: "unknown".to_string(),
                message: "biometric unlock is only supported on macOS".to_string(),
            });
        }

        let account_id = runtime.active_account_id()?;
        let bundle = self.biometric_unlock_port.load_unlock_bundle(&account_id)?;
        if bundle.account_id != account_id {
            return Err(AppError::ValidationFieldError {
                field: "unknown".to_string(),
                message: "biometric unlock account does not match current account".to_string(),
            });
        }

        let user_key = biometric_bundle_to_vault_user_key(&bundle)?;
        runtime.set_vault_user_key_material(account_id.clone(), user_key.clone())?;

        log::info!(
            target: "vanguard::application::vault_biometric",
            "vault unlocked with biometric account_id={}",
            account_id
        );
        Ok(UnlockVaultResult {
            account_id,
            refresh_token: user_key.refresh_token,
        })
    }

    pub fn lock(&self, runtime: &dyn VaultRuntimePort) -> AppResult<()> {
        let account_id = runtime.active_account_id()?;
        runtime.remove_vault_user_key_material(&account_id)
    }
}

#[async_trait]
impl BiometricUnlockExecutor for VaultBiometricUseCase {
    async fn execute_biometric_unlock(
        &self,
        runtime: &dyn VaultRuntimePort,
    ) -> AppResult<UnlockVaultResult> {
        self.unlock_with_biometric(runtime)
    }
}

fn vault_user_key_to_biometric_bundle(
    account_id: &str,
    user_key: &VaultUserKeyMaterial,
    refresh_token: Option<String>,
) -> Result<VaultBiometricBundle, AppError> {
    vault_crypto::validate_key_lengths(&user_key.enc_key, user_key.mac_key.as_deref())?;
    Ok(VaultBiometricBundle {
        account_id: String::from(account_id),
        enc_key_b64: STANDARD_NO_PAD.encode(&user_key.enc_key),
        mac_key_b64: user_key
            .mac_key
            .as_ref()
            .map(|value| STANDARD_NO_PAD.encode(value)),
        refresh_token,
    })
}

fn biometric_bundle_to_vault_user_key(
    bundle: &VaultBiometricBundle,
) -> Result<VaultUserKeyMaterial, AppError> {
    if bundle.account_id.trim().is_empty() {
        return Err(AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: "biometric unlock bundle account_id is empty".to_string(),
        });
    }

    let enc_key =
        vault_crypto::decode_base64_flexible(&bundle.enc_key_b64, "biometric.enc_key_b64")?;
    let mac_key = bundle
        .mac_key_b64
        .as_ref()
        .map(|value| vault_crypto::decode_base64_flexible(value, "biometric.mac_key_b64"))
        .transpose()?;
    vault_crypto::validate_key_lengths(&enc_key, mac_key.as_deref())?;

    Ok(VaultUserKeyMaterial {
        enc_key,
        mac_key,
        refresh_token: bundle.refresh_token.clone(),
    })
}
