# Research: 统一错误处理与展示重构

**Date**: 2026-03-09
**Feature**: 001-error-handling-refactor

## 研究目标

解决以下技术决策问题:
1. Rust 后端错误代码命名和分类策略
2. 前端 Sonner Toast 集成方式
3. 错误代码到用户消息的映射机制
4. 现有错误处理代码的迁移路径

## 决策 1: Rust 错误代码分类策略

**选择**: 使用分层错误代码体系,按错误来源和严重程度分类

**理由**:
- 现有 `AppError` 已有 4 个基础变体 (Validation, Remote, RemoteStatus, Internal)
- 需要扩展为更细粒度的错误类型,同时保持向后兼容
- 采用 `CATEGORY_SPECIFIC_ERROR` 命名模式 (如 `AUTH_INVALID_CREDENTIALS`, `VAULT_CIPHER_NOT_FOUND`)

**实现方案**:
```rust
pub enum AppError {
    // 认证相关
    AuthInvalidCredentials,
    AuthTokenExpired,
    AuthPermissionDenied,

    // 保险库相关
    VaultCipherNotFound,
    VaultDecryptionFailed,
    VaultSyncConflict,

    // 验证相关
    ValidationFieldError { field: String, message: String },
    ValidationFormatError(String),

    // 网络相关
    NetworkConnectionFailed,
    NetworkTimeout,
    NetworkRemoteError { status: u16, message: String },

    // 内部错误
    InternalDatabaseError(String),
    InternalCryptoError(String),
    InternalUnexpected(String),
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::AuthInvalidCredentials => "AUTH_INVALID_CREDENTIALS",
            Self::AuthTokenExpired => "AUTH_TOKEN_EXPIRED",
            // ... 其他映射
        }
    }
}
```

**替代方案考虑**:
- 使用字符串常量而非枚举: 放弃,因为失去编译时类型安全
- 使用错误码数字: 放弃,因为可读性差且不符合 Web API 最佳实践

## 决策 2: Sonner Toast 集成方式

**选择**: 使用 shadcn/ui 的 Sonner 组件,通过全局 Toaster 和 toast() 函数调用

**理由**:
- Sonner 是 shadcn/ui 生态推荐的 Toast 库,与项目现有 UI 组件一致
- 提供声明式 API,支持 Promise 状态跟踪
- 内置无障碍支持和键盘导航
- 轻量级 (~3KB gzipped)

**安装步骤**:
```bash
pnpm add sonner
```

**集成代码**:
```tsx
// src/lib/toast.tsx
import { toast as sonnerToast } from 'sonner';

export const toast = {
  error: (title: string, description?: string) => {
    sonnerToast.error(title, { description });
  },
  warning: (title: string, description?: string) => {
    sonnerToast.warning(title, { description });
  },
  success: (title: string, description?: string) => {
    sonnerToast.success(title, { description });
  },
  info: (title: string, description?: string) => {
    sonnerToast.info(title, { description });
  },
};

// src/main.tsx
import { Toaster } from 'sonner';

<Toaster position="top-right" richColors />
```

**替代方案考虑**:
- react-hot-toast: 放弃,因为 shadcn/ui 不推荐
- 自建 Toast 组件: 放弃,因为重复造轮子且需要额外维护

## 决策 3: 错误代码到用户消息映射

**选择**: 使用 TypeScript 对象映射 + i18n 预留

**理由**:
- 集中管理所有用户可见错误消息
- 支持未来国际化扩展
- 提供类型安全的错误代码引用

**实现方案**:
```typescript
// src/lib/error-messages.ts
export const ERROR_MESSAGES: Record<string, { title: string; description?: string }> = {
  // 认证错误
  AUTH_INVALID_CREDENTIALS: {
    title: '登录失败',
    description: '用户名或密码错误,请重试',
  },
  AUTH_TOKEN_EXPIRED: {
    title: '会话已过期',
    description: '请重新登录',
  },
  AUTH_PERMISSION_DENIED: {
    title: '权限不足',
    description: '您没有权限执行此操作',
  },

  // 保险库错误
  VAULT_CIPHER_NOT_FOUND: {
    title: '项目不存在',
    description: '请求的密码项未找到',
  },
  VAULT_DECRYPTION_FAILED: {
    title: '解密失败',
    description: '无法解密此项目,请检查主密码',
  },

  // 网络错误
  NETWORK_CONNECTION_FAILED: {
    title: '网络连接失败',
    description: '请检查网络连接后重试',
  },
  NETWORK_TIMEOUT: {
    title: '请求超时',
    description: '服务器响应超时,请稍后重试',
  },

  // 降级处理
  UNKNOWN_ERROR: {
    title: '未知错误',
    description: '发生了意外错误,请联系支持',
  },
};

export function getErrorMessage(code: string) {
  return ERROR_MESSAGES[code] || ERROR_MESSAGES.UNKNOWN_ERROR;
}
```

**替代方案考虑**:
- 后端返回完整用户消息: 放弃,因为违反前后端职责分离原则
- 使用 i18n 库: 暂缓,当前仅支持中文,未来可迁移

## 决策 4: 统一错误处理器设计

**选择**: 创建全局 ErrorHandler 类,拦截所有 Tauri command 错误

**理由**:
- 集中处理所有错误逻辑,避免重复代码
- 支持错误去重(防止相同错误重复显示)
- 支持特殊错误的自动处理(如 token 过期自动跳转登录)

**实现方案**:
```typescript
// src/lib/error-handler.ts
import { toast } from './toast';
import { getErrorMessage } from './error-messages';

interface ErrorResponse {
  code: string;
  message: string;
}

class ErrorHandler {
  private recentErrors = new Set<string>();
  private readonly DEDUPE_WINDOW = 3000; // 3秒内相同错误去重

  handle(error: unknown) {
    const errorResponse = this.parseError(error);

    // 去重检查
    const errorKey = `${errorResponse.code}:${Date.now()}`;
    if (this.recentErrors.has(errorResponse.code)) {
      return;
    }

    this.recentErrors.add(errorResponse.code);
    setTimeout(() => {
      this.recentErrors.delete(errorResponse.code);
    }, this.DEDUPE_WINDOW);

    // 特殊错误处理
    if (errorResponse.code === 'AUTH_TOKEN_EXPIRED') {
      this.handleTokenExpired();
      return;
    }

    // 显示 Toast
    const message = getErrorMessage(errorResponse.code);
    toast.error(message.title, message.description);
  }

  private parseError(error: unknown): ErrorResponse {
    if (typeof error === 'string') {
      try {
        const parsed = JSON.parse(error);
        if (parsed.code && parsed.message) {
          return parsed;
        }
      } catch {}
      return { code: 'UNKNOWN_ERROR', message: error };
    }

    if (error && typeof error === 'object' && 'code' in error) {
      return error as ErrorResponse;
    }

    return { code: 'UNKNOWN_ERROR', message: String(error) };
  }

  private handleTokenExpired() {
    // 清除本地认证状态
    // 跳转到登录页
    window.location.href = '/login';
  }
}

export const errorHandler = new ErrorHandler();
```

**使用方式**:
```typescript
// 在各个 hook 中
try {
  await someCommand();
} catch (error) {
  errorHandler.handle(error);
}
```

**替代方案考虑**:
- 使用 React Error Boundary: 放弃,因为无法捕获异步错误
- 使用 TanStack Query 的 onError: 考虑,但需要确认项目是否使用 Query

## 决策 5: 迁移策略

**选择**: 渐进式迁移,优先级 P1 → P2 → P3

**迁移步骤**:
1. **Phase 1 (P1)**: 后端错误代码标准化
   - 扩展 `AppError` 枚举
   - 更新所有 use cases 返回细粒度错误
   - 确保 `ErrorPayload` 序列化正确
   - 运行后端测试验证

2. **Phase 2 (P2)**: 前端统一错误处理
   - 安装 Sonner
   - 创建 `error-handler.ts`, `error-messages.ts`, `toast.tsx`
   - 在 `main.tsx` 添加 `<Toaster />`
   - 更新一个 feature (如 auth) 作为试点
   - 验证 Toast 显示正确

3. **Phase 3 (P3)**: 全面迁移
   - 迁移所有 features 的错误处理
   - 删除旧的 `error-utils.ts` 文件
   - 移除组件内的内联错误状态
   - 运行前端测试验证

**向后兼容性**:
- 后端: 保留 `AppError::code()` 方法,确保现有调用不中断
- 前端: 保留 `toErrorText()` 工具函数,直到所有调用点迁移完成

## 最佳实践参考

### Rust 错误处理
- [The Rust Programming Language - Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [thiserror crate](https://docs.rs/thiserror/) - 考虑未来使用以简化错误定义

### Toast 通知
- [Sonner Documentation](https://sonner.emilkowal.ski/)
- [shadcn/ui Sonner](https://ui.shadcn.com/docs/components/sonner)

### 错误代码设计
- [RFC 7807 - Problem Details for HTTP APIs](https://tools.ietf.org/html/rfc7807)
- [Google API Design Guide - Errors](https://cloud.google.com/apis/design/errors)

## 未解决问题

无 - 所有技术决策已明确。

## 下一步

进入 Phase 1: 设计数据模型和接口契约。
