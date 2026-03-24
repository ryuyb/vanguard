use crate::application::dto::vault::{UnlockVaultCommand, UnlockVaultResult};
use crate::application::ports::vault_runtime_port::VaultRuntimePort;
use crate::application::use_cases::master_password_unlock_use_case::MasterPasswordUnlockUseCase;
use crate::application::use_cases::vault_biometric_use_case::VaultBiometricUseCase;
use crate::application::use_cases::vault_pin_use_case::VaultPinUseCase;
use crate::domain::unlock::UnlockMethod;
use crate::support::error::AppError;
use crate::support::result::AppResult;

#[derive(Clone)]
pub struct UnlockVaultUseCase {
    master_password: MasterPasswordUnlockUseCase,
    pin: VaultPinUseCase,
    biometric: VaultBiometricUseCase,
}

impl UnlockVaultUseCase {
    pub fn new(
        master_password: MasterPasswordUnlockUseCase,
        pin: VaultPinUseCase,
        biometric: VaultBiometricUseCase,
    ) -> Self {
        Self {
            master_password,
            pin,
            biometric,
        }
    }

    pub async fn execute(
        &self,
        runtime: &dyn VaultRuntimePort,
        command: UnlockVaultCommand,
    ) -> AppResult<UnlockVaultResult> {
        match command.method {
            UnlockMethod::MasterPassword { password } => {
                let trimmed = password.trim().to_string();
                if trimmed.is_empty() {
                    return Err(AppError::ValidationFieldError {
                        field: "unknown".to_string(),
                        message: "master_password cannot be empty".to_string(),
                    });
                }
                self.master_password.execute(runtime, trimmed).await
            }
            UnlockMethod::Pin { pin } => {
                let trimmed = pin.trim().to_string();
                if trimmed.is_empty() {
                    return Err(AppError::ValidationFieldError {
                        field: "unknown".to_string(),
                        message: "pin cannot be empty".to_string(),
                    });
                }
                self.pin.execute_pin_unlock(runtime, trimmed).await
            }
            UnlockMethod::Biometric => self.biometric.execute_biometric_unlock(runtime).await,
        }
    }
}
