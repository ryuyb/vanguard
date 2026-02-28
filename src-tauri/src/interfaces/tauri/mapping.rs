use crate::application::dto::auth::{
    PasswordLoginCommand, PasswordLoginOutcome, PreloginInfo, PreloginQuery, RefreshTokenCommand,
    SendEmailLoginCommand, SessionInfo, TwoFactorChallenge, VerifyEmailTokenCommand,
};
use crate::interfaces::tauri::dto::auth::{
    PasswordLoginRequestDto, PasswordLoginResponseDto, PreloginRequestDto, PreloginResponseDto,
    RefreshTokenRequestDto, SendEmailLoginRequestDto, SessionResponseDto, TwoFactorChallengeDto,
    VerifyEmailTokenRequestDto,
};

pub fn to_prelogin_query(dto: PreloginRequestDto) -> PreloginQuery {
    PreloginQuery {
        base_url: dto.base_url,
        email: dto.email,
    }
}

pub fn to_prelogin_response_dto(info: PreloginInfo) -> PreloginResponseDto {
    PreloginResponseDto {
        kdf: info.kdf,
        kdf_iterations: info.kdf_iterations,
        kdf_memory: info.kdf_memory,
        kdf_parallelism: info.kdf_parallelism,
    }
}

pub fn to_password_login_command(dto: PasswordLoginRequestDto) -> PasswordLoginCommand {
    PasswordLoginCommand {
        base_url: dto.base_url,
        username: dto.username,
        password: dto.password,
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

pub fn to_refresh_token_command(dto: RefreshTokenRequestDto) -> RefreshTokenCommand {
    RefreshTokenCommand {
        base_url: dto.base_url,
        refresh_token: dto.refresh_token,
    }
}

pub fn to_send_email_login_command(dto: SendEmailLoginRequestDto) -> SendEmailLoginCommand {
    SendEmailLoginCommand {
        base_url: dto.base_url,
        email: dto.email,
        plaintext_password: dto.master_password_hash,
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
        providers2: challenge.providers2,
        master_password_policy: challenge.master_password_policy,
    }
}
