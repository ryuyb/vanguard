use async_trait::async_trait;

use crate::application::dto::auth::{
    PasswordLoginCommand, PasswordLoginOutcome, PreloginInfo, PreloginQuery, RefreshTokenCommand,
    SendEmailLoginCommand, SessionInfo, VerifyEmailTokenCommand,
};
use crate::application::dto::sync::{
    CipherMutationResult, CreateCipherCommand, DeleteCipherCommand, RestoreCipherCommand,
    RevisionDateQuery, SoftDeleteCipherCommand, SyncCipher, SyncFolder, SyncSend,
    SyncVaultCommand, SyncVaultPayload, UpdateCipherCommand,
};
use crate::support::result::AppResult;

#[async_trait]
pub trait RemoteVaultPort: Send + Sync {
    async fn prelogin(&self, query: PreloginQuery) -> AppResult<PreloginInfo>;

    async fn login_with_password(
        &self,
        command: PasswordLoginCommand,
    ) -> AppResult<PasswordLoginOutcome>;

    async fn refresh_token(&self, command: RefreshTokenCommand) -> AppResult<SessionInfo>;

    async fn send_email_login(&self, command: SendEmailLoginCommand) -> AppResult<()>;

    async fn verify_email_token(&self, command: VerifyEmailTokenCommand) -> AppResult<()>;

    async fn sync_vault(&self, command: SyncVaultCommand) -> AppResult<SyncVaultPayload>;

    async fn get_cipher(
        &self,
        command: SyncVaultCommand,
        cipher_id: String,
    ) -> AppResult<SyncCipher>;

    async fn get_folder(
        &self,
        command: SyncVaultCommand,
        folder_id: String,
    ) -> AppResult<SyncFolder>;

    async fn get_folders(&self, command: SyncVaultCommand) -> AppResult<Vec<SyncFolder>>;

    async fn get_send(&self, command: SyncVaultCommand, send_id: String) -> AppResult<SyncSend>;

    async fn get_revision_date(&self, query: RevisionDateQuery) -> AppResult<i64>;

    async fn create_cipher(&self, command: CreateCipherCommand) -> AppResult<CipherMutationResult>;

    async fn update_cipher(&self, command: UpdateCipherCommand) -> AppResult<CipherMutationResult>;

    async fn delete_cipher(&self, command: DeleteCipherCommand) -> AppResult<()>;

    async fn soft_delete_cipher(
        &self,
        command: SoftDeleteCipherCommand,
    ) -> AppResult<CipherMutationResult>;

    async fn restore_cipher(&self, command: RestoreCipherCommand) -> AppResult<()>;
}
