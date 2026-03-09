# Data Model: 统一错误处理与展示重构

**Date**: 2026-03-09
**Feature**: 001-error-handling-refactor

## 概述

本文档定义错误处理重构涉及的核心数据结构,包括后端错误类型、前端错误响应接口和错误消息映射。

## 后端数据模型 (Rust)

### AppError 枚举

扩展后的错误类型枚举,支持细粒度错误分类:

```rust
// src-tauri/src/support/error.rs

#[derive(Debug)]
pub enum AppError {
    // === 认证错误 ===
    AuthInvalidCredentials,
    AuthTokenExpired,
    AuthTokenInvalid,
    AuthPermissionDenied,
    AuthAccountLocked,
    AuthTwoFactorRequired,

    // === 保险库错误 ===
    VaultCipherNotFound { cipher_id: String },
    VaultDecryptionFailed { reason: String },
    VaultSyncConflict { cipher_id: String },
    VaultLocked,
    VaultCorrupted,

    // === 验证错误 ===
    ValidationFieldError { field: String, message: String },
    ValidationFormatError { format: String, value: String },
    ValidationRequired { field: String },

    // === 网络错误 ===
    NetworkConnectionFailed,
    NetworkTimeout,
    NetworkRemoteError { status: u16, message: String },
    NetworkDnsResolutionFailed,

    // === 存储错误 ===
    StorageDatabaseError { operation: String, details: String },
    StorageFileNotFound { path: String },
    StoragePermissionDenied { path: String },

    // === 加密错误 ===
    CryptoKeyDerivationFailed,
    CryptoEncryptionFailed,
    CryptoDecryptionFailed,
    CryptoInvalidKey,

    // === 内部错误 ===
    InternalUnexpected { message: String },
    InternalNotImplemented { feature: String },
}
```

### ErrorPayload 结构

前端接收的标准化错误响应:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorPayload {
    /// 唯一错误代码 (如 "AUTH_INVALID_CREDENTIALS")
    pub code: String,

    /// 人类可读的错误消息 (用于日志和调试)
    pub message: String,

    /// 可选的详细信息 (如字段验证错误的具体字段)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,

    /// 错误发生时间戳
    pub timestamp: i64,

    /// 错误严重程度
    pub severity: ErrorSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Fatal,
}
```

### 错误代码映射

```rust
impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            // 认证
            Self::AuthInvalidCredentials => "AUTH_INVALID_CREDENTIALS",
            Self::AuthTokenExpired => "AUTH_TOKEN_EXPIRED",
            Self::AuthTokenInvalid => "AUTH_TOKEN_INVALID",
            Self::AuthPermissionDenied => "AUTH_PERMISSION_DENIED",
            Self::AuthAccountLocked => "AUTH_ACCOUNT_LOCKED",
            Self::AuthTwoFactorRequired => "AUTH_TWO_FACTOR_REQUIRED",

            // 保险库
            Self::VaultCipherNotFound { .. } => "VAULT_CIPHER_NOT_FOUND",
            Self::VaultDecryptionFailed { .. } => "VAULT_DECRYPTION_FAILED",
            Self::VaultSyncConflict { .. } => "VAULT_SYNC_CONFLICT",
            Self::VaultLocked => "VAULT_LOCKED",
            Self::VaultCorrupted => "VAULT_CORRUPTED",

            // 验证
            Self::ValidationFieldError { .. } => "VALIDATION_FIELD_ERROR",
            Self::ValidationFormatError { .. } => "VALIDATION_FORMAT_ERROR",
            Self::ValidationRequired { .. } => "VALIDATION_REQUIRED",

            // 网络
            Self::NetworkConnectionFailed => "NETWORK_CONNECTION_FAILED",
            Self::NetworkTimeout => "NETWORK_TIMEOUT",
            Self::NetworkRemoteError { .. } => "NETWORK_REMOTE_ERROR",
            Self::NetworkDnsResolutionFailed => "NETWORK_DNS_RESOLUTION_FAILED",

            // 存储
            Self::StorageDatabaseError { .. } => "STORAGE_DATABASE_ERROR",
            Self::StorageFileNotFound { .. } => "STORAGE_FILE_NOT_FOUND",
            Self::StoragePermissionDenied { .. } => "STORAGE_PERMISSION_DENIED",

            // 加密
            Self::CryptoKeyDerivationFailed => "CRYPTO_KEY_DERIVATION_FAILED",
            Self::CryptoEncryptionFailed => "CRYPTO_ENCRYPTION_FAILED",
            Self::CryptoDecryptionFailed => "CRYPTO_DECRYPTION_FAILED",
            Self::CryptoInvalidKey => "CRYPTO_INVALID_KEY",

            // 内部
            Self::InternalUnexpected { .. } => "INTERNAL_UNEXPECTED",
            Self::InternalNotImplemented { .. } => "INTERNAL_NOT_IMPLEMENTED",
        }
    }

    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::AuthTwoFactorRequired => ErrorSeverity::Info,
            Self::ValidationFieldError { .. }
            | Self::ValidationFormatError { .. }
            | Self::ValidationRequired { .. } => ErrorSeverity::Warning,
            Self::InternalUnexpected { .. }
            | Self::VaultCorrupted
            | Self::CryptoInvalidKey => ErrorSeverity::Fatal,
            _ => ErrorSeverity::Error,
        }
    }

    pub fn to_payload(&self) -> ErrorPayload {
        ErrorPayload {
            code: self.code().to_string(),
            message: self.message(),
            details: self.details(),
            timestamp: chrono::Utc::now().timestamp(),
            severity: self.severity(),
        }
    }

    fn details(&self) -> Option<serde_json::Value> {
        match self {
            Self::ValidationFieldError { field, message } => Some(serde_json::json!({
                "field": field,
                "message": message,
            })),
            Self::VaultCipherNotFound { cipher_id } => Some(serde_json::json!({
                "cipherId": cipher_id,
            })),
            Self::NetworkRemoteError { status, .. } => Some(serde_json::json!({
                "status": status,
            })),
            _ => None,
        }
    }
}
```

## 前端数据模型 (TypeScript)

### ErrorResponse 接口

```typescript
// src/lib/error-handler.ts

export interface ErrorResponse {
  /** 唯一错误代码 */
  code: string;

  /** 错误消息 (用于日志) */
  message: string;

  /** 可选的详细信息 */
  details?: Record<string, unknown>;

  /** 时间戳 */
  timestamp: number;

  /** 严重程度 */
  severity: 'info' | 'warning' | 'error' | 'fatal';
}
```

### ErrorMessage 接口

```typescript
// src/lib/error-messages.ts

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

export type ErrorMessageMap = Record<string, ErrorMessage>;
```

### ErrorHandler 类

```typescript
// src/lib/error-handler.ts

export class ErrorHandler {
  private recentErrors: Map<string, number>;
  private readonly DEDUPE_WINDOW = 3000; // 3秒

  constructor() {
    this.recentErrors = new Map();
  }

  /**
   * 处理错误并显示 Toast 通知
   */
  handle(error: unknown): void;

  /**
   * 解析错误对象为标准格式
   */
  private parseError(error: unknown): ErrorResponse;

  /**
   * 检查是否为重复错误
   */
  private isDuplicate(code: string): boolean;

  /**
   * 记录错误用于去重
   */
  private recordError(code: string): void;

  /**
   * 处理特殊错误 (如 token 过期)
   */
  private handleSpecialError(response: ErrorResponse): boolean;

  /**
   * 显示 Toast 通知
   */
  private showToast(response: ErrorResponse): void;
}
```

## 实体关系

```
┌─────────────────┐
│   AppError      │ (Rust 后端)
│   (枚举)        │
└────────┬────────┘
         │ .to_payload()
         ↓
┌─────────────────┐
│ ErrorPayload    │ (Rust → JSON)
│ (结构体)        │
└────────┬────────┘
         │ Tauri IPC
         ↓
┌─────────────────┐
│ ErrorResponse   │ (TypeScript 前端)
│ (接口)          │
└────────┬────────┘
         │ errorHandler.handle()
         ↓
┌─────────────────┐       ┌──────────────────┐
│ ErrorHandler    │──────→│ ErrorMessageMap  │
│ (类)            │       │ (映射表)         │
└────────┬────────┘       └──────────────────┘
         │
         ↓
┌─────────────────┐
│ Toast 通知      │ (Sonner)
│ (UI 组件)       │
└─────────────────┘
```

## 状态转换

### 错误处理流程

```
[错误发生] → [AppError 构造] → [转换为 ErrorPayload]
    ↓
[Tauri IPC 传输]
    ↓
[前端接收 ErrorResponse] → [ErrorHandler 解析]
    ↓
[去重检查] → [特殊错误处理?]
    ↓              ↓
  [否]          [是: 跳转/清理]
    ↓
[查找错误消息] → [显示 Toast]
    ↓
[3-5秒后自动消失]
```

## 验证规则

### 后端验证
- 每个 `AppError` 变体必须有唯一的错误代码
- 错误代码必须使用 `CATEGORY_SPECIFIC_ERROR` 格式
- `ErrorPayload` 必须可序列化为 JSON
- 敏感信息不得包含在错误消息中

### 前端验证
- 所有错误代码必须在 `ErrorMessageMap` 中有对应的用户消息
- Toast 通知必须在 5 秒内自动消失
- 相同错误在 3 秒内不得重复显示
- 特殊错误 (如 token 过期) 必须触发自动处理

## 扩展性考虑

### 添加新错误类型
1. 在 `AppError` 枚举中添加新变体
2. 在 `code()` 方法中添加映射
3. 在前端 `ErrorMessageMap` 中添加用户消息
4. 添加单元测试验证

### 国际化支持
- 预留 i18n 集成点: `ErrorMessageMap` 可替换为 i18n 函数调用
- 错误代码保持英文,仅翻译用户消息

### 错误追踪集成
- `ErrorPayload` 包含 `timestamp` 字段,便于日志关联
- 可扩展 `details` 字段添加 trace ID 或 request ID
