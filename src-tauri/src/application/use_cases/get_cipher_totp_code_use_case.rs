use std::sync::Arc;

use crate::application::dto::vault::{
    GetCipherDetailQuery, GetCipherTotpCodeCommand, GetCipherTotpCodeResult,
};
use crate::application::ports::vault_runtime_port::VaultRuntimePort;
use crate::application::totp::{current_unix_seconds, generate_current_totp};
use crate::application::use_cases::get_cipher_detail_use_case::GetCipherDetailUseCase;
use crate::support::error::AppError;
use crate::support::result::AppResult;

#[derive(Clone)]
pub struct GetCipherTotpCodeUseCase {
    get_cipher_detail_use_case: Arc<GetCipherDetailUseCase>,
}

impl GetCipherTotpCodeUseCase {
    pub fn new(get_cipher_detail_use_case: Arc<GetCipherDetailUseCase>) -> Self {
        Self {
            get_cipher_detail_use_case,
        }
    }

    pub async fn execute(
        &self,
        runtime: &dyn VaultRuntimePort,
        command: GetCipherTotpCodeCommand,
    ) -> AppResult<GetCipherTotpCodeResult> {
        let cipher_id = command.cipher_id.trim();
        if cipher_id.is_empty() {
            return Err(AppError::validation("cipher_id cannot be empty"));
        }

        let account_id = runtime.active_account_id()?;
        let user_key = runtime
            .get_vault_user_key_material(&account_id)?
            .ok_or_else(|| {
                AppError::validation("vault is locked, please unlock with master password first")
            })?;

        let cipher = self
            .get_cipher_detail_use_case
            .execute(GetCipherDetailQuery {
                account_id,
                cipher_id: String::from(cipher_id),
                user_key,
            })
            .await?;

        let raw_totp = pick_first_non_empty(&[
            cipher.login.as_ref().and_then(|entry| entry.totp.clone()),
            cipher.data.as_ref().and_then(|entry| entry.totp.clone()),
        ])
        .ok_or_else(|| AppError::validation("requested field is empty: totp"))?;

        let snapshot = generate_current_totp(&raw_totp, current_unix_seconds()?)?;
        Ok(GetCipherTotpCodeResult {
            code: snapshot.code,
            period_seconds: snapshot.period_seconds,
            remaining_seconds: snapshot.remaining_seconds,
            expires_at_ms: snapshot.expires_at_ms,
        })
    }
}

fn pick_first_non_empty(values: &[Option<String>]) -> Option<String> {
    values
        .iter()
        .filter_map(|value| value.as_ref())
        .find(|value| !value.trim().is_empty())
        .cloned()
}
