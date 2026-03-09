# 错误响应格式规范

**Feature**: 001-error-handling-refactor
**Date**: 2026-03-09
**Version**: 1.0.0

## 概述

本文档定义 Tauri 后端返回给前端的标准化错误响应格式,确保前后端错误处理的一致性。

## 响应格式

### JSON Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["code", "message", "timestamp", "severity"],
  "properties": {
    "code": {
      "type": "string",
      "description": "唯一错误代码,格式为 CATEGORY_SPECIFIC_ERROR",
      "pattern": "^[A-Z_]+$",
      "examples": ["AUTH_INVALID_CREDENTIALS", "VAULT_CIPHER_NOT_FOUND"]
    },
    "message": {
      "type": "string",
      "description": "人类可读的错误消息,用于日志和调试",
      "minLength": 1
    },
    "details": {
      "type": "object",
      "description": "可选的详细信息,如字段验证错误的具体字段",
      "additionalProperties": true
    },
    "timestamp": {
      "type": "integer",
      "description": "错误发生的 Unix 时间戳 (秒)",
      "minimum": 0
    },
    "severity": {
      "type": "string",
      "enum": ["info", "warning", "error", "fatal"],
      "description": "错误严重程度"
    }
  }
}
```

### TypeScript 类型定义

```typescript
export interface ErrorResponse {
  /** 唯一错误代码 */
  code: string;

  /** 错误消息 (用于日志) */
  message: string;

  /** 可选的详细信息 */
  details?: Record<string, unknown>;

  /** Unix 时间戳 (秒) */
  timestamp: number;

  /** 严重程度 */
  severity: 'info' | 'warning' | 'error' | 'fatal';
}
```

### Rust 类型定义

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorPayload {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    pub timestamp: i64,
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

## 响应示例

### 基本错误

```json
{
  "code": "AUTH_INVALID_CREDENTIALS",
  "message": "Invalid username or password",
  "timestamp": 1709971200,
  "severity": "error"
}
```

### 带详细信息的错误

```json
{
  "code": "VALIDATION_FIELD_ERROR",
  "message": "Field validation failed",
  "details": {
    "field": "email",
    "message": "Invalid email format"
  },
  "timestamp": 1709971200,
  "severity": "warning"
}
```

### 保险库错误

```json
{
  "code": "VAULT_CIPHER_NOT_FOUND",
  "message": "Cipher not found",
  "details": {
    "cipherId": "abc-123-def-456"
  },
  "timestamp": 1709971200,
  "severity": "error"
}
```

### 网络错误

```json
{
  "code": "NETWORK_REMOTE_ERROR",
  "message": "Remote server returned error",
  "details": {
    "status": 503,
    "endpoint": "/api/sync"
  },
  "timestamp": 1709971200,
  "severity": "error"
}
```

## Tauri Command 错误处理

### 后端实现

```rust
// src-tauri/src/interfaces/tauri/commands/auth.rs

use crate::support::error::AppError;
use crate::support::result::AppResult;

#[tauri::command]
pub async fn login(
    username: String,
    password: String,
) -> AppResult<LoginResponse> {
    // 业务逻辑
    if !validate_credentials(&username, &password) {
        return Err(AppError::AuthInvalidCredentials);
    }

    // 成功返回
    Ok(LoginResponse { token: "..." })
}
```

### Tauri 错误序列化

Tauri 会自动将 `Err(AppError)` 序列化为 JSON 字符串传递给前端:

```rust
impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_payload().serialize(serializer)
    }
}
```

### 前端接收

```typescript
import { invoke } from '@tauri-apps/api/core';

try {
  const response = await invoke('login', {
    username: 'user@example.com',
    password: 'password123',
  });
} catch (error) {
  // error 是 JSON 字符串,需要解析
  const errorResponse: ErrorResponse = JSON.parse(error as string);
  console.log(errorResponse.code); // "AUTH_INVALID_CREDENTIALS"
}
```

## 错误严重程度定义

| 严重程度 | 描述 | Toast 样式 | 用户操作 |
|---------|------|-----------|---------|
| `info` | 信息提示,不影响功能 | 蓝色 | 无需操作 |
| `warning` | 警告,可能影响部分功能 | 黄色 | 建议修正 |
| `error` | 错误,功能无法完成 | 红色 | 需要重试或修正 |
| `fatal` | 致命错误,应用无法继续 | 深红色 | 需要重启或联系支持 |

## 错误响应约束

### 必须遵守的规则

1. **唯一性**: 每个错误代码必须唯一,不得重复
2. **一致性**: 相同错误场景必须返回相同的错误代码
3. **安全性**: 错误消息不得包含敏感信息 (密码、令牌、密钥等)
4. **可序列化**: 所有字段必须可序列化为 JSON
5. **时间戳**: 必须使用 UTC 时间戳

### 推荐实践

1. **详细信息**: 使用 `details` 字段提供结构化的额外信息
2. **日志友好**: `message` 字段应包含足够的上下文用于调试
3. **用户友好**: 前端根据 `code` 显示本地化的用户消息,而非直接显示 `message`
4. **可追踪**: 考虑在 `details` 中添加 `requestId` 或 `traceId` 用于日志关联

## 向后兼容性

### 迁移策略

现有代码可能直接返回字符串错误:

```rust
// 旧代码
return Err("Invalid credentials".into());
```

迁移为:

```rust
// 新代码
return Err(AppError::AuthInvalidCredentials);
```

### 降级处理

前端必须处理未知错误代码:

```typescript
const message = getErrorMessage(errorResponse.code);
// 如果 code 未定义,返回 UNKNOWN_ERROR 的消息
```

## 测试用例

### 后端测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_payload_serialization() {
        let error = AppError::AuthInvalidCredentials;
        let payload = error.to_payload();

        assert_eq!(payload.code, "AUTH_INVALID_CREDENTIALS");
        assert_eq!(payload.severity, ErrorSeverity::Error);

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"code\":\"AUTH_INVALID_CREDENTIALS\""));
    }

    #[test]
    fn test_error_with_details() {
        let error = AppError::VaultCipherNotFound {
            cipher_id: "test-123".to_string(),
        };
        let payload = error.to_payload();

        assert!(payload.details.is_some());
        let details = payload.details.unwrap();
        assert_eq!(details["cipherId"], "test-123");
    }
}
```

### 前端测试

```typescript
import { describe, it, expect } from 'vitest';
import { errorHandler } from './error-handler';

describe('ErrorHandler', () => {
  it('should parse error response correctly', () => {
    const errorJson = JSON.stringify({
      code: 'AUTH_INVALID_CREDENTIALS',
      message: 'Invalid credentials',
      timestamp: 1709971200,
      severity: 'error',
    });

    const parsed = errorHandler['parseError'](errorJson);
    expect(parsed.code).toBe('AUTH_INVALID_CREDENTIALS');
    expect(parsed.severity).toBe('error');
  });

  it('should handle unknown error codes', () => {
    const errorJson = JSON.stringify({
      code: 'UNKNOWN_CODE_12345',
      message: 'Some error',
      timestamp: 1709971200,
      severity: 'error',
    });

    const message = getErrorMessage('UNKNOWN_CODE_12345');
    expect(message.title).toBe('未知错误');
  });
});
```

## 版本控制

### 版本策略

- 错误响应格式遵循语义化版本控制
- 添加新字段: MINOR 版本升级
- 修改现有字段: MAJOR 版本升级
- 添加新错误代码: PATCH 版本升级

### 当前版本

- **Version**: 1.0.0
- **Date**: 2026-03-09
- **Changes**: 初始版本

## 参考资料

- [RFC 7807 - Problem Details for HTTP APIs](https://tools.ietf.org/html/rfc7807)
- [Google API Design Guide - Errors](https://cloud.google.com/apis/design/errors)
- [Tauri Error Handling](https://tauri.app/v1/guides/features/command#error-handling)
