use crate::application::dto::auth::{
    MasterPasswordPolicy, PasswordLoginCommand, PasswordLoginOutcome, SendEmailLoginCommand,
    SessionInfo, TwoFactorChallenge, TwoFactorProviderHint, VerifyEmailTokenCommand,
    WebauthnAllowCredential, WebauthnRequestExtensions,
};
use crate::application::dto::sync::{SyncMetricsSummary, SyncOutcome};
use crate::application::dto::vault::VaultCipherDetail;
use crate::domain::sync::{SyncContext, SyncState, WsStatus};
use crate::interfaces::tauri::dto::auth::{
    MasterPasswordPolicyDto, PasswordLoginRequestDto, PasswordLoginResponseDto,
    SendEmailLoginRequestDto, SessionResponseDto, TwoFactorChallengeDto, TwoFactorProviderHintDto,
    VerifyEmailTokenRequestDto, WebauthnAllowCredentialDto, WebauthnRequestExtensionsDto,
};
use crate::interfaces::tauri::dto::sync::{
    SyncCountsDto, SyncMetricsDto, SyncStateDto, SyncStatusResponseDto, WsStatusDto,
};
use crate::interfaces::tauri::dto::vault::VaultCipherDetailDto;
use crate::support::error::AppError;
use std::collections::HashMap;

pub fn to_password_login_command(dto: PasswordLoginRequestDto) -> PasswordLoginCommand {
    PasswordLoginCommand {
        base_url: dto.base_url,
        username: dto.email,
        password: dto.master_password,
        two_factor_provider: dto.two_factor_provider,
        two_factor_token: dto.two_factor_token,
        two_factor_remember: dto.two_factor_remember,
        authrequest: dto.authrequest,
    }
}

pub fn to_password_login_response_dto(outcome: PasswordLoginOutcome) -> PasswordLoginResponseDto {
    match outcome {
        PasswordLoginOutcome::Authenticated(session) => {
            PasswordLoginResponseDto::Authenticated(to_session_response_dto(session))
        }
        PasswordLoginOutcome::TwoFactorRequired(challenge) => {
            PasswordLoginResponseDto::TwoFactorRequired(to_two_factor_challenge_dto(challenge))
        }
    }
}

pub fn to_send_email_login_command(dto: SendEmailLoginRequestDto) -> SendEmailLoginCommand {
    SendEmailLoginCommand {
        base_url: dto.base_url,
        email: dto.email,
        plaintext_password: dto.master_password,
        auth_request_id: dto.auth_request_id,
        auth_request_access_code: dto.auth_request_access_code,
    }
}

pub fn to_verify_email_token_command(dto: VerifyEmailTokenRequestDto) -> VerifyEmailTokenCommand {
    VerifyEmailTokenCommand {
        base_url: dto.base_url,
        user_id: dto.user_id,
        token: dto.token,
    }
}

pub fn to_session_response_dto(session: SessionInfo) -> SessionResponseDto {
    SessionResponseDto {
        access_token: session.access_token,
        refresh_token: session.refresh_token,
        expires_in: session.expires_in.clamp(i32::MIN as i64, i32::MAX as i64) as i32,
        token_type: session.token_type,
        scope: session.scope,
        key: session.key,
        private_key: session.private_key,
        kdf: session.kdf,
        kdf_iterations: session.kdf_iterations,
        kdf_memory: session.kdf_memory,
        kdf_parallelism: session.kdf_parallelism,
        two_factor_token: session.two_factor_token,
    }
}

pub fn to_two_factor_challenge_dto(challenge: TwoFactorChallenge) -> TwoFactorChallengeDto {
    TwoFactorChallengeDto {
        error: challenge.error,
        error_description: challenge.error_description,
        providers: challenge.providers,
        providers2: challenge.providers2.map(to_two_factor_provider_map_dto),
        master_password_policy: challenge
            .master_password_policy
            .map(to_master_password_policy_dto),
    }
}

fn to_master_password_policy_dto(policy: MasterPasswordPolicy) -> MasterPasswordPolicyDto {
    MasterPasswordPolicyDto {
        min_complexity: policy.min_complexity,
        min_length: policy.min_length,
        require_lower: policy.require_lower,
        require_upper: policy.require_upper,
        require_numbers: policy.require_numbers,
        require_special: policy.require_special,
        enforce_on_login: policy.enforce_on_login,
        object: policy.object,
    }
}

fn to_two_factor_provider_map_dto(
    providers: HashMap<String, Option<TwoFactorProviderHint>>,
) -> HashMap<String, Option<TwoFactorProviderHintDto>> {
    providers
        .into_iter()
        .map(|(key, value)| (key, value.map(to_two_factor_provider_hint_dto)))
        .collect()
}

fn to_two_factor_provider_hint_dto(hint: TwoFactorProviderHint) -> TwoFactorProviderHintDto {
    TwoFactorProviderHintDto {
        host: hint.host,
        signature: hint.signature,
        auth_url: hint.auth_url,
        nfc: hint.nfc,
        email: hint.email,
        challenge: hint.challenge,
        timeout: hint.timeout,
        rp_id: hint.rp_id,
        allow_credentials: hint
            .allow_credentials
            .into_iter()
            .map(to_webauthn_allow_credential_dto)
            .collect(),
        user_verification: hint.user_verification,
        extensions: hint.extensions.map(to_webauthn_request_extensions_dto),
    }
}

fn to_webauthn_allow_credential_dto(
    credential: WebauthnAllowCredential,
) -> WebauthnAllowCredentialDto {
    WebauthnAllowCredentialDto {
        r#type: credential.r#type,
        id: credential.id,
        transports: credential.transports,
    }
}

fn to_webauthn_request_extensions_dto(
    extensions: WebauthnRequestExtensions,
) -> WebauthnRequestExtensionsDto {
    WebauthnRequestExtensionsDto {
        appid: extensions.appid,
    }
}

pub fn to_sync_status_response_dto(
    context: SyncContext,
    metrics: Option<SyncMetricsSummary>,
) -> SyncStatusResponseDto {
    SyncStatusResponseDto {
        account_id: context.account_id,
        base_url: context.base_url,
        state: to_sync_state_dto(context.state),
        ws_status: to_ws_status_dto(context.ws_status),
        last_revision_ms: context.last_revision_ms.map(|value| value.to_string()),
        last_sync_at_ms: context.last_sync_at_ms.map(|value| value.to_string()),
        last_error: context.last_error,
        counts: SyncCountsDto {
            folders: context.counts.folders,
            collections: context.counts.collections,
            policies: context.counts.policies,
            ciphers: context.counts.ciphers,
            sends: context.counts.sends,
        },
        metrics: metrics.map(to_sync_metrics_dto),
    }
}

pub fn to_sync_outcome_dto(outcome: SyncOutcome) -> SyncStatusResponseDto {
    to_sync_status_response_dto(outcome.context, None)
}

pub fn to_vault_cipher_detail_dto(
    detail: VaultCipherDetail,
) -> Result<VaultCipherDetailDto, AppError> {
    let has_totp = has_cipher_totp(&detail);
    let mut raw = serde_json::to_value(detail).map_err(|error| {
        AppError::internal(format!(
            "failed to serialize application vault cipher detail: {error}"
        ))
    })?;

    let Some(root) = raw.as_object_mut() else {
        return Err(AppError::internal(
            "failed to map vault cipher detail dto: expected object payload",
        ));
    };

    if let Some(serde_json::Value::Object(login)) = root.get_mut("login") {
        login.remove("totp");
    }
    if let Some(serde_json::Value::Object(data)) = root.get_mut("data") {
        data.remove("totp");
    }
    root.insert(String::from("hasTotp"), serde_json::Value::Bool(has_totp));

    serde_json::from_value::<VaultCipherDetailDto>(raw).map_err(|error| {
        AppError::internal(format!(
            "failed to deserialize interface vault cipher detail dto: {error}"
        ))
    })
}

fn has_cipher_totp(detail: &VaultCipherDetail) -> bool {
    has_non_empty_value(
        detail
            .login
            .as_ref()
            .and_then(|entry| entry.totp.as_deref()),
    ) || has_non_empty_value(detail.data.as_ref().and_then(|entry| entry.totp.as_deref()))
}

fn has_non_empty_value(value: Option<&str>) -> bool {
    value.map(|entry| !entry.trim().is_empty()).unwrap_or(false)
}

fn to_sync_metrics_dto(metrics: SyncMetricsSummary) -> SyncMetricsDto {
    SyncMetricsDto {
        window_size: metrics.window_size,
        sample_count: metrics.sample_count,
        success_count: metrics.success_count,
        failure_count: metrics.failure_count,
        failure_rate: metrics.failure_rate,
        last_duration_ms: metrics.last_duration_ms,
        average_duration_ms: metrics.average_duration_ms,
        last_counts: metrics.last_item_counts.map(to_counts_dto),
        average_counts: metrics.average_item_counts.map(to_counts_dto),
    }
}

fn to_counts_dto(counts: crate::domain::sync::SyncItemCounts) -> SyncCountsDto {
    SyncCountsDto {
        folders: counts.folders,
        collections: counts.collections,
        policies: counts.policies,
        ciphers: counts.ciphers,
        sends: counts.sends,
    }
}

fn to_sync_state_dto(state: SyncState) -> SyncStateDto {
    match state {
        SyncState::Idle => SyncStateDto::Idle,
        SyncState::Running => SyncStateDto::Running,
        SyncState::Succeeded => SyncStateDto::Succeeded,
        SyncState::Degraded => SyncStateDto::Degraded,
        SyncState::Failed => SyncStateDto::Failed,
    }
}

fn to_ws_status_dto(status: WsStatus) -> WsStatusDto {
    match status {
        WsStatus::Unknown => WsStatusDto::Unknown,
        WsStatus::Connected => WsStatusDto::Connected,
        WsStatus::Disconnected => WsStatusDto::Disconnected,
    }
}
