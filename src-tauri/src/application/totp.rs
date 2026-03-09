use std::time::{SystemTime, UNIX_EPOCH};

use totp_rs::{Algorithm, Secret, TOTP};

use crate::support::error::AppError;
use crate::support::result::AppResult;

#[derive(Debug, Clone)]
pub struct TotpCodeSnapshot {
    pub code: String,
    pub period_seconds: u64,
    pub remaining_seconds: u64,
    pub expires_at_ms: i64,
}

pub fn current_unix_seconds() -> AppResult<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| AppError::InternalUnexpected {
            message: "failed to read system time for totp generation".into(),
        })
}

pub fn generate_current_totp(raw_totp: &str, unix_seconds: u64) -> AppResult<TotpCodeSnapshot> {
    let totp = parse_totp(raw_totp)?;
    let code = totp.generate(unix_seconds);
    let remaining_seconds = totp.step - (unix_seconds % totp.step);
    let expires_at_seconds = unix_seconds.checked_add(remaining_seconds).ok_or_else(|| {
        AppError::InternalUnexpected {
            message: "totp expiration timestamp overflow".into(),
        }
    })?;
    let expires_at_ms = i64::try_from(expires_at_seconds.checked_mul(1000).ok_or_else(|| {
        AppError::InternalUnexpected {
            message: "totp expiration timestamp overflow".into(),
        }
    })?)
    .map_err(|_| AppError::InternalUnexpected {
        message: "totp expiration timestamp overflow".into(),
    })?;

    Ok(TotpCodeSnapshot {
        code,
        period_seconds: totp.step,
        remaining_seconds,
        expires_at_ms,
    })
}

fn parse_totp(raw_totp: &str) -> AppResult<TOTP> {
    let raw = raw_totp.trim();
    if raw.is_empty() {
        return Err(AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: "invalid totp configuration".to_string(),
        });
    }

    let totp = if raw.to_ascii_lowercase().starts_with("otpauth://") {
        TOTP::from_url_unchecked(raw).map_err(|_| AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: "invalid totp configuration".to_string(),
        })?
    } else {
        let secret = Secret::Encoded(String::from(raw)).to_bytes().map_err(|_| {
            AppError::ValidationFieldError {
                field: "unknown".to_string(),
                message: "invalid totp configuration".to_string(),
            }
        })?;
        TOTP::new_unchecked(Algorithm::SHA1, 6, 1, 30, secret, None, String::new())
    };

    if totp.step == 0 || totp.step > 300 || totp.secret.is_empty() {
        return Err(AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: "invalid totp configuration".to_string(),
        });
    }
    if totp.digits < 6 || totp.digits > 9 {
        return Err(AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: "invalid totp configuration".to_string(),
        });
    }

    Ok(totp)
}

#[cfg(test)]
mod tests {
    use super::generate_current_totp;

    #[test]
    fn generate_totp_from_base32_secret() {
        let snapshot =
            generate_current_totp("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ", 59).expect("generate");
        assert_eq!(snapshot.code, "287082");
        assert_eq!(snapshot.period_seconds, 30);
        assert_eq!(snapshot.remaining_seconds, 1);
        assert_eq!(snapshot.expires_at_ms, 60_000);
    }

    #[test]
    fn generate_totp_from_otpauth_uri() {
        let snapshot = generate_current_totp(
            "otpauth://totp/Vanguard?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&algorithm=SHA1&digits=8&period=30",
            59,
        )
        .expect("generate");
        assert_eq!(snapshot.code, "94287082");
        assert_eq!(snapshot.period_seconds, 30);
        assert_eq!(snapshot.remaining_seconds, 1);
    }

    #[test]
    fn invalid_totp_algorithm_returns_error() {
        let result = generate_current_totp(
            "otpauth://totp/Vanguard?secret=GEZDGNBVGY3TQOJQ&algorithm=MD5",
            59,
        );
        assert!(result.is_err());
    }

    #[test]
    fn short_secret_totp_is_accepted() {
        let result = generate_current_totp(
            "otpauth://totp/Vanguard?secret=JBSWY3DPEHPK3PXP&period=30",
            59,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn unsupported_digits_totp_returns_error() {
        let result = generate_current_totp(
            "otpauth://totp/Vanguard?secret=JBSWY3DPEHPK3PXP&digits=10&period=30",
            59,
        );
        assert!(result.is_err());
    }
}
