# 快速入门: 统一错误处理与展示重构

**Feature**: 001-error-handling-refactor
**Date**: 2026-03-09

## 概述

本指南帮助开发者快速理解和使用新的统一错误处理机制。

## 5 分钟快速上手

### 后端 (Rust)

#### 1. 返回标准化错误

```rust
use crate::support::error::AppError;
use crate::support::result::AppResult;

#[tauri::command]
pub async fn my_command() -> AppResult<MyResponse> {
    // ✅ 使用具体的错误类型
    if !is_valid() {
        return Err(AppError::ValidationFieldError {
            field: "email".to_string(),
            message: "Invalid email format".to_string(),
        });
    }

    // ❌ 不要使用字符串错误
    // return Err("Invalid email".into());

    Ok(MyResponse { /* ... */ })
}
```

#### 2. 常用错误类型

```rust
// 认证错误
Err(AppError::AuthInvalidCredentials)
Err(AppError::AuthTokenExpired)

// 保险库错误
Err(AppError::VaultCipherNotFound { cipher_id: id.to_string() })
Err(AppError::VaultDecryptionFailed { reason: "Invalid key".to_string() })

// 验证错误
Err(AppError::ValidationRequired { field: "password".to_string() })

// 网络错误
Err(AppError::NetworkConnectionFailed)
Err(AppError::NetworkTimeout)
```

### 前端 (TypeScript)

#### 1. 使用统一错误处理器

```typescript
import { errorHandler } from '@/lib/error-handler';
import { invoke } from '@tauri-apps/api/core';

// ✅ 推荐: 使用 errorHandler
try {
  const result = await invoke('my_command', { /* args */ });
} catch (error) {
  errorHandler.handle(error); // 自动显示 Toast
}

// ❌ 不推荐: 手动处理错误
try {
  const result = await invoke('my_command', { /* args */ });
} catch (error) {
  const message = typeof error === 'string' ? error : 'Unknown error';
  // 手动显示错误...
}
```

#### 2. 在 React Hook 中使用

```typescript
import { errorHandler } from '@/lib/error-handler';

export function useMyFeature() {
  const [loading, setLoading] = useState(false);

  const doSomething = async () => {
    setLoading(true);
    try {
      await invoke('my_command');
      // 成功处理
    } catch (error) {
      errorHandler.handle(error);
    } finally {
      setLoading(false);
    }
  };

  return { doSomething, loading };
}
```

## 添加新错误类型

### 步骤 1: 定义后端错误

编辑 `src-tauri/src/support/error.rs`:

```rust
pub enum AppError {
    // ... 现有错误

    // 添加新错误
    MyNewError { details: String },
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            // ... 现有映射
            Self::MyNewError { .. } => "MY_NEW_ERROR",
        }
    }
}
```

### 步骤 2: 添加前端消息映射

编辑 `src/lib/error-messages.ts`:

```typescript
export const ERROR_MESSAGES: Record<string, ErrorMessage> = {
  // ... 现有消息

  MY_NEW_ERROR: {
    title: '我的新错误',
    description: '这是新错误的描述',
  },
};
```

### 步骤 3: 使用新错误

```rust
// 后端
return Err(AppError::MyNewError {
    details: "Something went wrong".to_string(),
});
```

前端会自动显示对应的 Toast 通知。

## 常见场景

### 场景 1: 表单验证错误

```rust
// 后端
if email.is_empty() {
    return Err(AppError::ValidationRequired {
        field: "email".to_string(),
    });
}

if !is_valid_email(&email) {
    return Err(AppError::ValidationFormatError {
        format: "email".to_string(),
        value: email.clone(),
    });
}
```

前端会自动显示 "请填写必填字段" 或 "数据格式不正确" 的 Toast。

### 场景 2: 网络请求失败

```rust
// 后端
match reqwest::get(url).await {
    Ok(response) => { /* ... */ },
    Err(e) if e.is_timeout() => {
        return Err(AppError::NetworkTimeout);
    },
    Err(e) if e.is_connect() => {
        return Err(AppError::NetworkConnectionFailed);
    },
    Err(e) => {
        return Err(AppError::NetworkRemoteError {
            status: 0,
            message: e.to_string(),
        });
    },
}
```

### 场景 3: 资源不存在

```rust
// 后端
let cipher = repository.find_by_id(&cipher_id)
    .ok_or_else(|| AppError::VaultCipherNotFound {
        cipher_id: cipher_id.to_string(),
    })?;
```

### 场景 4: 认证失败自动跳转

```typescript
// 前端 - 无需手动处理
try {
  await invoke('protected_command');
} catch (error) {
  errorHandler.handle(error);
  // 如果是 AUTH_TOKEN_EXPIRED,会自动跳转到登录页
}
```

## 特殊错误处理

### 自动处理的错误

以下错误会触发自动处理,无需手动干预:

- `AUTH_TOKEN_EXPIRED`: 自动清除认证状态并跳转登录页
- `AUTH_TOKEN_INVALID`: 自动清除认证状态并跳转登录页

### 自定义错误处理

如果需要自定义处理某些错误:

```typescript
import { errorHandler } from '@/lib/error-handler';

try {
  await invoke('my_command');
} catch (error) {
  const parsed = errorHandler['parseError'](error);

  if (parsed.code === 'MY_SPECIAL_ERROR') {
    // 自定义处理
    console.log('Handling special error');
  } else {
    // 使用默认处理
    errorHandler.handle(error);
  }
}
```

## 调试技巧

### 查看错误详情

```typescript
import { errorHandler } from '@/lib/error-handler';

try {
  await invoke('my_command');
} catch (error) {
  const parsed = errorHandler['parseError'](error);
  console.log('Error code:', parsed.code);
  console.log('Error message:', parsed.message);
  console.log('Error details:', parsed.details);
  console.log('Timestamp:', new Date(parsed.timestamp * 1000));

  errorHandler.handle(error);
}
```

### 测试错误场景

```rust
// 后端测试
#[cfg(test)]
mod tests {
    #[test]
    fn test_error_code() {
        let error = AppError::AuthInvalidCredentials;
        assert_eq!(error.code(), "AUTH_INVALID_CREDENTIALS");
    }
}
```

```typescript
// 前端测试
import { describe, it, expect } from 'vitest';
import { getErrorMessage } from '@/lib/error-messages';

describe('Error Messages', () => {
  it('should return correct message for error code', () => {
    const message = getErrorMessage('AUTH_INVALID_CREDENTIALS');
    expect(message.title).toBe('登录失败');
  });
});
```

## 迁移现有代码

### 迁移后端错误

**之前**:
```rust
return Err("Invalid credentials".into());
```

**之后**:
```rust
return Err(AppError::AuthInvalidCredentials);
```

### 迁移前端错误处理

**之前**:
```typescript
try {
  await invoke('login', { username, password });
} catch (error) {
  const message = typeof error === 'string' ? error : 'Unknown error';
  setErrorMessage(message); // 组件内状态
}

// JSX
{errorMessage && <div className="error">{errorMessage}</div>}
```

**之后**:
```typescript
try {
  await invoke('login', { username, password });
} catch (error) {
  errorHandler.handle(error); // 自动显示 Toast
}

// JSX - 移除错误显示代码
```

## 最佳实践

### ✅ 推荐做法

1. **使用具体的错误类型**
   ```rust
   Err(AppError::VaultCipherNotFound { cipher_id })
   ```

2. **让 errorHandler 处理所有错误**
   ```typescript
   errorHandler.handle(error);
   ```

3. **在错误消息中提供上下文**
   ```rust
   Err(AppError::ValidationFieldError {
       field: "email".to_string(),
       message: "Must be a valid email address".to_string(),
   })
   ```

### ❌ 避免做法

1. **不要使用字符串错误**
   ```rust
   // ❌ 不推荐
   Err("Something went wrong".into())
   ```

2. **不要在组件中内联错误处理**
   ```typescript
   // ❌ 不推荐
   const [error, setError] = useState<string | null>(null);
   ```

3. **不要直接显示后端错误消息**
   ```typescript
   // ❌ 不推荐
   toast.error(error.message); // 应该使用 error.code 查找用户消息
   ```

## 故障排查

### 问题: Toast 没有显示

**检查清单**:
1. 确认 `<Toaster />` 已添加到 `main.tsx`
2. 确认 `errorHandler.handle()` 被调用
3. 检查浏览器控制台是否有错误
4. 确认错误代码在 `ERROR_MESSAGES` 中有映射

### 问题: 显示 "未知错误"

**原因**: 错误代码未在 `ERROR_MESSAGES` 中定义

**解决**:
1. 检查后端返回的错误代码
2. 在 `src/lib/error-messages.ts` 中添加对应的消息映射

### 问题: 相同错误重复显示

**原因**: 错误去重机制未生效

**检查**:
1. 确认使用的是同一个 `errorHandler` 实例
2. 检查 `DEDUPE_WINDOW` 配置 (默认 3 秒)

## 参考资料

- [错误代码清单](./contracts/error-codes.md)
- [错误响应格式规范](./contracts/error-response.md)
- [数据模型文档](./data-model.md)
- [Sonner 文档](https://sonner.emilkowal.ski/)

## 获取帮助

如有问题,请:
1. 查看本文档的故障排查部分
2. 查看错误代码清单确认错误类型
3. 查看浏览器控制台和后端日志
4. 联系团队寻求支持
