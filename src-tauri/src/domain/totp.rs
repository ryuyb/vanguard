use crate::support::error::AppError;
use crate::support::result::AppResult;

/// TOTP 配置值对象
///
/// 封装 TOTP 的业务验证规则：
/// - step (period) 必须在 1-300 秒范围内
/// - digits 必须在 6-9 位范围内
/// - secret 不能为空
///
/// 这些规则是 TOTP 协议的领域固有约束，不应分散在 Application 层。
#[derive(Debug, Clone)]
pub struct TotpConfiguration {
    /// TOTP 更新周期（秒），必须在 1-300 范围内
    pub step: u64,
    /// TOTP 码位数，必须在 6-9 范围内
    pub digits: usize,
    /// TOTP 密钥，不能为空
    pub secret: Vec<u8>,
}

impl TotpConfiguration {
    /// 验证 TOTP 配置是否符合领域规则
    ///
    /// # Errors
    /// - 配置违反领域规则时返回 ValidationFieldError
    pub fn validate(&self) -> AppResult<()> {
        if self.step == 0 || self.step > 300 {
            return Err(AppError::ValidationFieldError {
                field: "totp.step".to_string(),
                message: "totp period must be between 1 and 300 seconds".to_string(),
            });
        }

        if self.digits < 6 || self.digits > 9 {
            return Err(AppError::ValidationFieldError {
                field: "totp.digits".to_string(),
                message: "totp digits must be between 6 and 9".to_string(),
            });
        }

        if self.secret.is_empty() {
            return Err(AppError::ValidationFieldError {
                field: "totp.secret".to_string(),
                message: "totp secret cannot be empty".to_string(),
            });
        }

        Ok(())
    }

    /// 从 totp_rs::TOTP 结构创建配置并验证
    ///
    /// # Errors
    /// - TOTP 配置不符合领域规则时返回错误
    pub fn from_totp(totp: &totp_rs::TOTP) -> AppResult<Self> {
        let config = Self {
            step: totp.step,
            digits: totp.digits,
            secret: totp.secret.clone(),
        };
        config.validate()?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_totp_configuration_passes_validation() {
        let config = TotpConfiguration {
            step: 30,
            digits: 6,
            secret: vec![1, 2, 3, 4],
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn step_zero_fails_validation() {
        let config = TotpConfiguration {
            step: 0,
            digits: 6,
            secret: vec![1, 2, 3, 4],
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn step_exceeds_300_fails_validation() {
        let config = TotpConfiguration {
            step: 301,
            digits: 6,
            secret: vec![1, 2, 3, 4],
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn digits_below_6_fails_validation() {
        let config = TotpConfiguration {
            step: 30,
            digits: 5,
            secret: vec![1, 2, 3, 4],
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn digits_above_9_fails_validation() {
        let config = TotpConfiguration {
            step: 30,
            digits: 10,
            secret: vec![1, 2, 3, 4],
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn empty_secret_fails_validation() {
        let config = TotpConfiguration {
            step: 30,
            digits: 6,
            secret: vec![],
        };
        assert!(config.validate().is_err());
    }
}
