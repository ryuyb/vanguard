/**
 * 错误代码到用户消息的映射表
 *
 * 提供统一的错误消息管理,支持未来国际化扩展
 */

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
 * 错误消息映射表类型
 */
export type ErrorMessageMap = Record<string, ErrorMessage>;

/**
 * 错误代码到用户消息的映射
 */
const ERROR_MESSAGES: ErrorMessageMap = {
  // === 认证错误 (AUTH_*) ===
  AUTH_INVALID_CREDENTIALS: {
    title: "登录失败",
    description: "用户名或密码错误,请检查后重试",
  },
  AUTH_TOKEN_EXPIRED: {
    title: "会话已过期",
    description: "请重新登录",
    requiresAction: true,
    actionLabel: "重新登录",
  },
  AUTH_TOKEN_INVALID: {
    title: "认证失败",
    description: "认证信息无效,请重新登录",
    requiresAction: true,
    actionLabel: "重新登录",
  },
  AUTH_PERMISSION_DENIED: {
    title: "权限不足",
    description: "您没有权限执行此操作",
  },
  AUTH_ACCOUNT_LOCKED: {
    title: "账户已锁定",
    description: "您的账户已被锁定,请联系管理员",
  },
  AUTH_TWO_FACTOR_REQUIRED: {
    title: "需要两步验证",
    description: "请输入两步验证码",
  },
  AUTH_INVALID_PIN: {
    title: "PIN 码错误",
    description: "PIN 码不正确,请重新输入",
  },

  // === 保险库错误 (VAULT_*) ===
  VAULT_CIPHER_NOT_FOUND: {
    title: "密码项不存在",
    description: "未找到该密码项,可能已被删除",
  },
  VAULT_DECRYPTION_FAILED: {
    title: "解密失败",
    description: "无法解密数据,请检查主密码是否正确",
  },
  VAULT_SYNC_CONFLICT: {
    title: "同步冲突",
    description: "检测到数据冲突,请手动解决",
  },
  VAULT_LOCKED: {
    title: "保险库已锁定",
    description: "请先解锁保险库",
    requiresAction: true,
    actionLabel: "解锁",
  },
  VAULT_CORRUPTED: {
    title: "数据损坏",
    description: "保险库数据已损坏,请联系技术支持",
  },

  // === 验证错误 (VALIDATION_*) ===
  VALIDATION_FIELD_ERROR: {
    title: "输入有误",
    description: "请检查输入的数据是否正确",
  },
  VALIDATION_FORMAT_ERROR: {
    title: "格式错误",
    description: "数据格式不正确,请重新输入",
  },
  VALIDATION_REQUIRED: {
    title: "缺少必填项",
    description: "请填写所有必填字段",
  },

  // === 网络错误 (NETWORK_*) ===
  NETWORK_CONNECTION_FAILED: {
    title: "网络连接失败",
    description: "无法连接到服务器,请检查网络连接",
  },
  NETWORK_TIMEOUT: {
    title: "请求超时",
    description: "服务器响应超时,请稍后重试",
  },
  NETWORK_REMOTE_ERROR: {
    title: "服务器错误",
    description: "服务器返回错误,请稍后重试",
  },
  NETWORK_DNS_RESOLUTION_FAILED: {
    title: "无法连接",
    description: "无法解析服务器地址,请检查网络设置",
  },

  // === 存储错误 (STORAGE_*) ===
  STORAGE_DATABASE_ERROR: {
    title: "数据保存失败",
    description: "无法保存数据,请重试",
  },
  STORAGE_FILE_NOT_FOUND: {
    title: "文件未找到",
    description: "请求的文件不存在",
  },
  STORAGE_PERMISSION_DENIED: {
    title: "权限不足",
    description: "无权限访问该文件",
  },

  // === 加密错误 (CRYPTO_*) ===
  CRYPTO_KEY_DERIVATION_FAILED: {
    title: "密钥生成失败",
    description: "无法生成加密密钥",
  },
  CRYPTO_ENCRYPTION_FAILED: {
    title: "加密失败",
    description: "数据加密失败,请重试",
  },
  CRYPTO_DECRYPTION_FAILED: {
    title: "解密失败",
    description: "无法解密数据,请检查密码",
  },
  CRYPTO_INVALID_KEY: {
    title: "密钥无效",
    description: "加密密钥无效,无法继续操作",
  },

  // === 内部错误 (INTERNAL_*) ===
  INTERNAL_UNEXPECTED: {
    title: "意外错误",
    description: "发生意外错误,请联系技术支持",
  },
  INTERNAL_NOT_IMPLEMENTED: {
    title: "功能未实现",
    description: "该功能暂未实现",
  },

  // === 降级处理 ===
  UNKNOWN_ERROR: {
    title: "未知错误",
    description: "发生未知错误,请重试",
  },
};

/**
 * 根据错误代码获取用户消息
 * @param code 错误代码
 * @returns 错误消息对象
 */
export function getErrorMessage(code: string): ErrorMessage {
  return (
    ERROR_MESSAGES[code] ||
    ERROR_MESSAGES.UNKNOWN_ERROR || {
      title: "未知错误",
      description: "发生未知错误,请重试",
    }
  );
}
