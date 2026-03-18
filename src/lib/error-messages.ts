/**
 * 错误代码到翻译键的映射表
 *
 * 提供统一的错误消息管理,支持国际化
 */

import { appI18n } from "@/i18n";

/**
 * 错误消息接口
 */
export interface ErrorMessage {
  /** Toast 标题 */
  title: string;

  /** Toast 描述 (可选) */
  description?: string;

  /** 是否需要用户操作 */
  requiresAction?: boolean;

  /** 操作按钮文本 (如 "重新登录") */
  actionLabel?: string;

  /** 操作回调 */
  action?: () => void;
}

/**
 * 错误代码到翻译键的映射
 */
const ERROR_CODE_TO_I18N_KEY: Record<string, string> = {
  // === 认证错误 (AUTH_*) ===
  AUTH_INVALID_CREDENTIALS: "errors.auth.invalidCredentials",
  AUTH_TOKEN_EXPIRED: "errors.auth.tokenExpired",
  AUTH_TOKEN_INVALID: "errors.auth.tokenInvalid",
  AUTH_PERMISSION_DENIED: "errors.auth.permissionDenied",
  AUTH_ACCOUNT_LOCKED: "errors.auth.accountLocked",
  AUTH_TWO_FACTOR_REQUIRED: "errors.auth.twoFactorRequired",
  AUTH_INVALID_PIN: "errors.auth.invalidPin",

  // === 保险库错误 (VAULT_*) ===
  VAULT_CIPHER_NOT_FOUND: "errors.vault.cipherNotFound",
  VAULT_DECRYPTION_FAILED: "errors.vault.decryptionFailed",
  VAULT_SYNC_CONFLICT: "errors.vault.syncConflict",
  VAULT_LOCKED: "errors.vault.locked",
  VAULT_CORRUPTED: "errors.vault.corrupted",

  // === 验证错误 (VALIDATION_*) ===
  VALIDATION_FIELD_ERROR: "errors.validation.fieldError",
  VALIDATION_FORMAT_ERROR: "errors.validation.formatError",
  VALIDATION_REQUIRED: "errors.validation.required",

  // === 网络错误 (NETWORK_*) ===
  NETWORK_CONNECTION_FAILED: "errors.network.connectionFailed",
  NETWORK_TIMEOUT: "errors.network.timeout",
  NETWORK_REMOTE_ERROR: "errors.network.remoteError",
  NETWORK_DNS_RESOLUTION_FAILED: "errors.network.dnsResolutionFailed",

  // === 存储错误 (STORAGE_*) ===
  STORAGE_DATABASE_ERROR: "errors.storage.databaseError",
  STORAGE_FILE_NOT_FOUND: "errors.storage.fileNotFound",
  STORAGE_PERMISSION_DENIED: "errors.storage.permissionDenied",

  // === 加密错误 (CRYPTO_*) ===
  CRYPTO_KEY_DERIVATION_FAILED: "errors.crypto.keyDerivationFailed",
  CRYPTO_ENCRYPTION_FAILED: "errors.crypto.encryptionFailed",
  CRYPTO_DECRYPTION_FAILED: "errors.crypto.decryptionFailed",
  CRYPTO_INVALID_KEY: "errors.crypto.invalidKey",

  // === 内部错误 (INTERNAL_*) ===
  INTERNAL_UNEXPECTED: "errors.internal.unexpected",
  INTERNAL_NOT_IMPLEMENTED: "errors.internal.notImplemented",

  // === 降级处理 ===
  UNKNOWN_ERROR: "errors.common.unknown",
};

/**
 * 需要用户操作的错误代码
 */
const ERRORS_REQUIRING_ACTION = new Set([
  "AUTH_TOKEN_EXPIRED",
  "AUTH_TOKEN_INVALID",
  "VAULT_LOCKED",
]);

/**
 * 根据错误代码获取用户消息
 * @param code 错误代码
 * @returns 错误消息对象
 */
export function getErrorMessage(code: string): ErrorMessage {
  const i18nKey =
    ERROR_CODE_TO_I18N_KEY[code] || ERROR_CODE_TO_I18N_KEY.UNKNOWN_ERROR;

  const title = appI18n.t(`${i18nKey}.title`);
  const description = appI18n.t(`${i18nKey}.description`);
  const requiresAction = ERRORS_REQUIRING_ACTION.has(code);
  const actionLabel = requiresAction
    ? appI18n.t(`${i18nKey}.action`, {
        defaultValue: appI18n.t("errors.common.action"),
      })
    : undefined;

  return {
    title,
    description,
    requiresAction,
    actionLabel,
  };
}
