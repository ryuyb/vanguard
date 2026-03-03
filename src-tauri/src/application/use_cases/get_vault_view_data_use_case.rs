use std::sync::Arc;

use crate::application::dto::vault::{
    GetVaultViewDataQuery, GetVaultViewDataResult, VaultCipherItem, VaultFolderItem,
};
use crate::application::ports::vault_runtime_port::VaultRuntimePort;
use crate::application::services::sync_service::SyncService;
use crate::application::vault_crypto;
use crate::support::error::AppError;
use crate::support::result::AppResult;

const DEFAULT_PAGE: u32 = 1;
const DEFAULT_PAGE_SIZE: u32 = 50;
const MAX_PAGE_SIZE: u32 = 200;

#[derive(Clone)]
pub struct GetVaultViewDataUseCase {
    sync_service: Arc<SyncService>,
}

impl GetVaultViewDataUseCase {
    pub fn new(sync_service: Arc<SyncService>) -> Self {
        Self { sync_service }
    }

    pub async fn execute(
        &self,
        runtime: &dyn VaultRuntimePort,
        query: GetVaultViewDataQuery,
    ) -> AppResult<GetVaultViewDataResult> {
        let account_id = runtime.active_account_id()?;
        let page = normalize_page(query.page);
        let page_size = normalize_page_size(query.page_size);
        let offset = (u64::from(page.saturating_sub(1)) * u64::from(page_size))
            .min(u64::from(u32::MAX)) as u32;

        let user_key = runtime
            .get_vault_user_key_material(&account_id)?
            .ok_or_else(|| {
                AppError::validation("vault is locked, please unlock with master password first")
            })?;

        let sync_context = self.sync_service.sync_status(account_id.clone()).await?;
        let sync_metrics = self.sync_service.sync_metrics(account_id.clone())?;
        let folders = self
            .sync_service
            .list_live_folders(account_id.clone())
            .await?;
        let ciphers = self
            .sync_service
            .list_live_ciphers(account_id.clone(), offset, page_size)
            .await?;
        let total_ciphers = self
            .sync_service
            .count_live_ciphers(account_id.clone())
            .await?;

        let folders = folders
            .into_iter()
            .map(|folder| {
                Ok(VaultFolderItem {
                    id: folder.id,
                    name: vault_crypto::decrypt_optional_field(
                        folder.name,
                        &user_key,
                        "folder.name",
                    )?,
                })
            })
            .collect::<Result<Vec<_>, AppError>>()?;

        let ciphers = ciphers
            .into_iter()
            .map(|cipher| {
                let login_username = vault_crypto::decrypt_optional_field(
                    cipher
                        .login
                        .as_ref()
                        .and_then(|login| login.username.clone()),
                    &user_key,
                    "cipher.login.username",
                )?;
                let data_username = vault_crypto::decrypt_optional_field(
                    cipher.data.as_ref().and_then(|data| data.username.clone()),
                    &user_key,
                    "cipher.data.username",
                )?;

                Ok(VaultCipherItem {
                    id: cipher.id,
                    folder_id: cipher.folder_id,
                    organization_id: cipher.organization_id,
                    r#type: cipher.r#type,
                    name: vault_crypto::decrypt_optional_field(
                        cipher.name,
                        &user_key,
                        "cipher.name",
                    )?,
                    username: first_non_empty(login_username, data_username),
                    creation_date: cipher.creation_date,
                    revision_date: cipher.revision_date,
                    deleted_date: cipher.deleted_date,
                    attachment_count: cipher.attachments.len().min(u32::MAX as usize) as u32,
                })
            })
            .collect::<Result<Vec<_>, AppError>>()?;

        Ok(GetVaultViewDataResult {
            account_id,
            sync_context,
            sync_metrics,
            folders,
            ciphers,
            total_ciphers,
            page,
            page_size,
        })
    }
}

fn normalize_page(page: Option<u32>) -> u32 {
    page.unwrap_or(DEFAULT_PAGE).max(1)
}

fn normalize_page_size(page_size: Option<u32>) -> u32 {
    page_size
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE)
}

fn first_non_empty(left: Option<String>, right: Option<String>) -> Option<String> {
    if let Some(value) = left {
        if !value.trim().is_empty() {
            return Some(value);
        }
    }
    if let Some(value) = right {
        if !value.trim().is_empty() {
            return Some(value);
        }
    }
    None
}
