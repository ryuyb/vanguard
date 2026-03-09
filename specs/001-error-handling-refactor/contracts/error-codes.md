# 错误代码清单

**Feature**: 001-error-handling-refactor
**Date**: 2026-03-09
**Version**: 1.0.0

## 概述

本文档列出所有标准化的错误代码,用于前后端错误处理的统一标识。

## 错误代码格式

- 格式: `CATEGORY_SPECIFIC_ERROR`
- 示例: `AUTH_INVALID_CREDENTIALS`, `VAULT_CIPHER_NOT_FOUND`
- 规则:
  - 全大写字母
  - 使用下划线分隔
  - 前缀表示错误类别
  - 后缀描述具体错误

## 认证错误 (AUTH_*)

| 错误代码 | 严重程度 | 描述 | 用户消息 |
|---------|---------|------|---------|
| `AUTH_INVALID_CREDENTIALS` | Error | 用户名或密码错误 | 登录失败,请检查用户名和密码 |
| `AUTH_TOKEN_EXPIRED` | Error | 认证令牌已过期 | 会话已过期,请重新登录 |
| `AUTH_TOKEN_INVALID` | Error | 认证令牌无效 | 认证失败,请重新登录 |
| `AUTH_PERMISSION_DENIED` | Error | 用户无权限执行操作 | 权限不足,无法执行此操作 |
| `AUTH_ACCOUNT_LOCKED` | Error | 账户已被锁定 | 账户已锁定,请联系管理员 |
| `AUTH_TWO_FACTOR_REQUIRED` | Info | 需要两步验证 | 请输入两步验证码 |

## 保险库错误 (VAULT_*)

| 错误代码 | 严重程度 | 描述 | 用户消息 |
|---------|---------|------|---------|
| `VAULT_CIPHER_NOT_FOUND` | Error | 密码项不存在 | 未找到该密码项 |
| `VAULT_DECRYPTION_FAILED` | Error | 解密失败 | 无法解密,请检查主密码 |
| `VAULT_SYNC_CONFLICT` | Warning | 同步冲突 | 检测到同步冲突,请手动解决 |
| `VAULT_LOCKED` | Error | 保险库已锁定 | 保险库已锁定,请先解锁 |
| `VAULT_CORRUPTED` | Fatal | 保险库数据损坏 | 保险库数据损坏,请联系支持 |

## 验证错误 (VALIDATION_*)

| 错误代码 | 严重程度 | 描述 | 用户消息 |
|---------|---------|------|---------|
| `VALIDATION_FIELD_ERROR` | Warning | 字段验证失败 | 输入数据有误,请检查 |
| `VALIDATION_FORMAT_ERROR` | Warning | 格式错误 | 数据格式不正确 |
| `VALIDATION_REQUIRED` | Warning | 必填字段缺失 | 请填写必填字段 |

## 网络错误 (NETWORK_*)

| 错误代码 | 严重程度 | 描述 | 用户消息 |
|---------|---------|------|---------|
| `NETWORK_CONNECTION_FAILED` | Error | 网络连接失败 | 网络连接失败,请检查网络 |
| `NETWORK_TIMEOUT` | Error | 请求超时 | 请求超时,请稍后重试 |
| `NETWORK_REMOTE_ERROR` | Error | 远程服务器错误 | 服务器错误,请稍后重试 |
| `NETWORK_DNS_RESOLUTION_FAILED` | Error | DNS 解析失败 | 无法连接到服务器 |

## 存储错误 (STORAGE_*)

| 错误代码 | 严重程度 | 描述 | 用户消息 |
|---------|---------|------|---------|
| `STORAGE_DATABASE_ERROR` | Error | 数据库操作失败 | 数据保存失败,请重试 |
| `STORAGE_FILE_NOT_FOUND` | Error | 文件不存在 | 文件未找到 |
| `STORAGE_PERMISSION_DENIED` | Error | 存储权限不足 | 无权限访问文件 |

## 加密错误 (CRYPTO_*)

| 错误代码 | 严重程度 | 描述 | 用户消息 |
|---------|---------|------|---------|
| `CRYPTO_KEY_DERIVATION_FAILED` | Error | 密钥派生失败 | 密钥生成失败 |
| `CRYPTO_ENCRYPTION_FAILED` | Error | 加密失败 | 加密失败,请重试 |
| `CRYPTO_DECRYPTION_FAILED` | Error | 解密失败 | 解密失败,请检查密码 |
| `CRYPTO_INVALID_KEY` | Fatal | 密钥无效 | 密钥无效,无法继续 |

## 内部错误 (INTERNAL_*)

| 错误代码 | 严重程度 | 描述 | 用户消息 |
|---------|---------|------|---------|
| `INTERNAL_UNEXPECTED` | Fatal | 未预期的内部错误 | 发生意外错误,请联系支持 |
| `INTERNAL_NOT_IMPLEMENTED` | Error | 功能未实现 | 该功能暂未实现 |

## 降级错误

| 错误代码 | 严重程度 | 描述 | 用户消息 |
|---------|---------|------|---------|
| `UNKNOWN_ERROR` | Error | 未知错误 (降级处理) | 发生未知错误,请重试 |

## 使用指南

### 后端使用

```rust
// 返回特定错误
return Err(AppError::AuthInvalidCredentials);

// 带详细信息的错误
return Err(AppError::VaultCipherNotFound {
    cipher_id: cipher_id.to_string(),
});
```

### 前端使用

```typescript
// 自动处理错误
try {
  await someCommand();
} catch (error) {
  errorHandler.handle(error); // 自动显示对应的用户消息
}
```

## 添加新错误代码

1. 在本文档添加新错误代码条目
2. 在 `src-tauri/src/support/error.rs` 添加 `AppError` 变体
3. 在 `src/lib/error-messages.ts` 添加用户消息映射
4. 添加单元测试验证
5. 更新此文档的版本号

## 版本历史

- **1.0.0** (2026-03-09): 初始版本,定义 30+ 个标准错误代码
