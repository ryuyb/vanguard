use std::sync::Arc;

use async_trait::async_trait;

use crate::application::dto::vault::{UnlockVaultCommand, UnlockVaultResult};
use crate::application::ports::vault_runtime_port::VaultRuntimePort;
use crate::domain::unlock::UnlockMethod;
use crate::support::error::AppError;
use crate::support::result::AppResult;

#[async_trait]
pub trait MasterPasswordUnlockExecutor: Send + Sync {
    async fn execute_master_password_unlock(
        &self,
        runtime: &dyn VaultRuntimePort,
        master_password: String,
    ) -> AppResult<UnlockVaultResult>;
}

#[async_trait]
pub trait PinUnlockExecutor: Send + Sync {
    async fn execute_pin_unlock(
        &self,
        runtime: &dyn VaultRuntimePort,
        pin: String,
    ) -> AppResult<UnlockVaultResult>;
}

#[async_trait]
pub trait BiometricUnlockExecutor: Send + Sync {
    async fn execute_biometric_unlock(
        &self,
        runtime: &dyn VaultRuntimePort,
    ) -> AppResult<UnlockVaultResult>;
}

#[derive(Clone)]
pub struct UnlockVaultUseCase {
    master_password_executor: Arc<dyn MasterPasswordUnlockExecutor>,
    pin_executor: Arc<dyn PinUnlockExecutor>,
    biometric_executor: Arc<dyn BiometricUnlockExecutor>,
}

impl UnlockVaultUseCase {
    pub fn new(
        master_password_executor: Arc<dyn MasterPasswordUnlockExecutor>,
        pin_executor: Arc<dyn PinUnlockExecutor>,
        biometric_executor: Arc<dyn BiometricUnlockExecutor>,
    ) -> Self {
        Self {
            master_password_executor,
            pin_executor,
            biometric_executor,
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
                self.master_password_executor
                    .execute_master_password_unlock(runtime, trimmed)
                    .await
            }
            UnlockMethod::Pin { pin } => {
                let trimmed = pin.trim().to_string();
                if trimmed.is_empty() {
                    return Err(AppError::ValidationFieldError {
                        field: "unknown".to_string(),
                        message: "pin cannot be empty".to_string(),
                    });
                }
                self.pin_executor.execute_pin_unlock(runtime, trimmed).await
            }
            UnlockMethod::Biometric => {
                self.biometric_executor
                    .execute_biometric_unlock(runtime)
                    .await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;

    use super::{
        BiometricUnlockExecutor, MasterPasswordUnlockExecutor, PinUnlockExecutor,
        UnlockVaultUseCase,
    };
    use crate::application::dto::vault::{
        UnlockVaultCommand, UnlockVaultResult, VaultUnlockContext, VaultUserKeyMaterial,
    };
    use crate::application::ports::vault_runtime_port::VaultRuntimePort;
    use crate::domain::unlock::UnlockMethod;
    use crate::support::error::AppError;
    use crate::support::result::AppResult;

    struct FakeRuntime;

    impl VaultRuntimePort for FakeRuntime {
        fn active_account_id(&self) -> AppResult<String> {
            Ok(String::from("account-1"))
        }

        fn auth_session_context(&self) -> AppResult<Option<VaultUnlockContext>> {
            Ok(None)
        }

        fn persisted_auth_context(&self) -> AppResult<Option<VaultUnlockContext>> {
            Ok(None)
        }

        fn get_vault_user_key_material(
            &self,
            _account_id: &str,
        ) -> AppResult<Option<VaultUserKeyMaterial>> {
            Ok(None)
        }

        fn set_vault_user_key_material(
            &self,
            _account_id: String,
            _key: VaultUserKeyMaterial,
        ) -> AppResult<()> {
            Ok(())
        }

        fn remove_vault_user_key_material(&self, _account_id: &str) -> AppResult<()> {
            Ok(())
        }
    }

    struct RecordingMasterExecutor {
        calls: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl MasterPasswordUnlockExecutor for RecordingMasterExecutor {
        async fn execute_master_password_unlock(
            &self,
            runtime: &dyn VaultRuntimePort,
            master_password: String,
        ) -> AppResult<UnlockVaultResult> {
            self.calls
                .lock()
                .expect("master calls lock")
                .push(master_password);
            Ok(UnlockVaultResult {
                account_id: runtime.active_account_id()?,
            })
        }
    }

    struct RecordingPinExecutor {
        calls: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl PinUnlockExecutor for RecordingPinExecutor {
        async fn execute_pin_unlock(
            &self,
            runtime: &dyn VaultRuntimePort,
            pin: String,
        ) -> AppResult<UnlockVaultResult> {
            self.calls.lock().expect("pin calls lock").push(pin);
            Ok(UnlockVaultResult {
                account_id: runtime.active_account_id()?,
            })
        }
    }

    struct RecordingBiometricExecutor {
        calls: Arc<Mutex<u32>>,
    }

    #[async_trait]
    impl BiometricUnlockExecutor for RecordingBiometricExecutor {
        async fn execute_biometric_unlock(
            &self,
            runtime: &dyn VaultRuntimePort,
        ) -> AppResult<UnlockVaultResult> {
            let mut guard = self.calls.lock().expect("biometric calls lock");
            *guard += 1;
            Ok(UnlockVaultResult {
                account_id: runtime.active_account_id()?,
            })
        }
    }

    fn build_use_case(
        master_calls: Arc<Mutex<Vec<String>>>,
        pin_calls: Arc<Mutex<Vec<String>>>,
        biometric_calls: Arc<Mutex<u32>>,
    ) -> UnlockVaultUseCase {
        UnlockVaultUseCase::new(
            Arc::new(RecordingMasterExecutor {
                calls: master_calls,
            }),
            Arc::new(RecordingPinExecutor { calls: pin_calls }),
            Arc::new(RecordingBiometricExecutor {
                calls: biometric_calls,
            }),
        )
    }

    #[tokio::test]
    async fn dispatches_master_password_unlock() {
        let master_calls = Arc::new(Mutex::new(Vec::new()));
        let pin_calls = Arc::new(Mutex::new(Vec::new()));
        let biometric_calls = Arc::new(Mutex::new(0u32));
        let use_case = build_use_case(
            Arc::clone(&master_calls),
            Arc::clone(&pin_calls),
            Arc::clone(&biometric_calls),
        );

        let result = use_case
            .execute(
                &FakeRuntime,
                UnlockVaultCommand {
                    method: UnlockMethod::MasterPassword {
                        password: String::from("  secret  "),
                    },
                },
            )
            .await
            .expect("master unlock should dispatch");

        assert_eq!(result.account_id, "account-1");
        assert_eq!(
            *master_calls.lock().expect("master calls lock"),
            vec!["secret"]
        );
        assert!(pin_calls.lock().expect("pin calls lock").is_empty());
        assert_eq!(*biometric_calls.lock().expect("biometric calls lock"), 0);
    }

    #[tokio::test]
    async fn dispatches_pin_unlock() {
        let master_calls = Arc::new(Mutex::new(Vec::new()));
        let pin_calls = Arc::new(Mutex::new(Vec::new()));
        let biometric_calls = Arc::new(Mutex::new(0u32));
        let use_case = build_use_case(
            Arc::clone(&master_calls),
            Arc::clone(&pin_calls),
            Arc::clone(&biometric_calls),
        );

        let result = use_case
            .execute(
                &FakeRuntime,
                UnlockVaultCommand {
                    method: UnlockMethod::Pin {
                        pin: String::from("  123456  "),
                    },
                },
            )
            .await
            .expect("pin unlock should dispatch");

        assert_eq!(result.account_id, "account-1");
        assert!(master_calls.lock().expect("master calls lock").is_empty());
        assert_eq!(*pin_calls.lock().expect("pin calls lock"), vec!["123456"]);
        assert_eq!(*biometric_calls.lock().expect("biometric calls lock"), 0);
    }

    #[tokio::test]
    async fn dispatches_biometric_unlock() {
        let master_calls = Arc::new(Mutex::new(Vec::new()));
        let pin_calls = Arc::new(Mutex::new(Vec::new()));
        let biometric_calls = Arc::new(Mutex::new(0u32));
        let use_case = build_use_case(
            Arc::clone(&master_calls),
            Arc::clone(&pin_calls),
            Arc::clone(&biometric_calls),
        );

        let result = use_case
            .execute(
                &FakeRuntime,
                UnlockVaultCommand {
                    method: UnlockMethod::Biometric,
                },
            )
            .await
            .expect("biometric unlock should dispatch");

        assert_eq!(result.account_id, "account-1");
        assert!(master_calls.lock().expect("master calls lock").is_empty());
        assert!(pin_calls.lock().expect("pin calls lock").is_empty());
        assert_eq!(*biometric_calls.lock().expect("biometric calls lock"), 1);
    }

    #[tokio::test]
    async fn rejects_empty_master_password_before_dispatch() {
        let master_calls = Arc::new(Mutex::new(Vec::new()));
        let pin_calls = Arc::new(Mutex::new(Vec::new()));
        let biometric_calls = Arc::new(Mutex::new(0u32));
        let use_case = build_use_case(
            Arc::clone(&master_calls),
            Arc::clone(&pin_calls),
            Arc::clone(&biometric_calls),
        );

        let error = use_case
            .execute(
                &FakeRuntime,
                UnlockVaultCommand {
                    method: UnlockMethod::MasterPassword {
                        password: String::from("   "),
                    },
                },
            )
            .await
            .expect_err("empty password should fail");

        match error {
            AppError::ValidationRequired { field } => assert_eq!(field, "master_password"),
            other => panic!("unexpected error variant: {other:?}"),
        }
        assert!(master_calls.lock().expect("master calls lock").is_empty());
        assert!(pin_calls.lock().expect("pin calls lock").is_empty());
        assert_eq!(*biometric_calls.lock().expect("biometric calls lock"), 0);
    }

    #[tokio::test]
    async fn rejects_empty_pin_before_dispatch() {
        let master_calls = Arc::new(Mutex::new(Vec::new()));
        let pin_calls = Arc::new(Mutex::new(Vec::new()));
        let biometric_calls = Arc::new(Mutex::new(0u32));
        let use_case = build_use_case(
            Arc::clone(&master_calls),
            Arc::clone(&pin_calls),
            Arc::clone(&biometric_calls),
        );

        let error = use_case
            .execute(
                &FakeRuntime,
                UnlockVaultCommand {
                    method: UnlockMethod::Pin {
                        pin: String::from("   "),
                    },
                },
            )
            .await
            .expect_err("empty pin should fail");

        match error {
            AppError::ValidationRequired { field } => assert_eq!(field, "pin"),
            other => panic!("unexpected error variant: {other:?}"),
        }
        assert!(master_calls.lock().expect("master calls lock").is_empty());
        assert!(pin_calls.lock().expect("pin calls lock").is_empty());
        assert_eq!(*biometric_calls.lock().expect("biometric calls lock"), 0);
    }
}
