use base64::engine::general_purpose::{URL_SAFE, URL_SAFE_NO_PAD};
use base64::Engine;
use serde_json::Value;

use crate::support::error::AppError;
use crate::support::result::AppResult;

pub fn derive_account_id_from_access_token(
    base_url: &str,
    access_token: &str,
) -> AppResult<String> {
    let normalized_base_url = normalize_base_url(base_url)?;
    let subject = decode_token_subject(access_token)?;
    Ok(format!("{normalized_base_url}::{subject}"))
}

fn normalize_base_url(base_url: &str) -> AppResult<String> {
    let normalized = base_url.trim().trim_end_matches('/').to_lowercase();
    if normalized.is_empty() {
        return Err(AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: "base_url cannot be empty".to_string(),
        });
    }
    Ok(normalized)
}

fn decode_token_subject(access_token: &str) -> AppResult<String> {
    let payload_segment =
        access_token
            .trim()
            .split('.')
            .nth(1)
            .ok_or_else(|| AppError::ValidationFieldError {
                field: "unknown".to_string(),
                message: "access_token is not a valid JWT".to_string(),
            })?;

    let decoded = URL_SAFE_NO_PAD
        .decode(payload_segment)
        .or_else(|_| URL_SAFE.decode(payload_segment))
        .map_err(|_| AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: "access_token payload is not valid base64url".to_string(),
        })?;

    let claims: Value =
        serde_json::from_slice(&decoded).map_err(|_| AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: "access_token payload is not valid JSON".to_string(),
        })?;

    let subject = claims
        .get("sub")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: "access_token payload missing subject claim".to_string(),
        })?;

    Ok(subject.to_lowercase())
}
