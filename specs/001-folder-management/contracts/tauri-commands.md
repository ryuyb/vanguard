# Tauri Commands Contract: Folder Management

**Feature**: 001-folder-management
**Date**: 2026-03-11
**Protocol**: Tauri IPC (Inter-Process Communication)

## Overview

本文档定义了前端与后端之间的 Tauri 命令接口契约。所有命令通过 `@tauri-apps/api/core` 的 `invoke` 函数调用。

---

## Commands

### 1. get_folders

**Description**: 获取当前用户的所有文件夹列表

**Command Name**: `get_folders`

**Request**:
```typescript
// 无参数
invoke<FolderDto[]>('get_folders')
```

**Response**:
```typescript
interface FolderDto {
  id: string;
  name: string;
  created_at: number;  // Unix timestamp (ms)
  updated_at: number;
  cipher_count: number;  // 包含的 cipher 数量
}

// 返回值
FolderDto[]
```

**Errors**:
- `Unauthorized`: 用户未登录
- `DatabaseError`: 数据库查询失败

**Example**:
```typescript
import { invoke } from '@tauri-apps/api/core';

const folders = await invoke<FolderDto[]>('get_folders');
console.log(folders); // [{ id: "...", name: "工作账号", ... }]
```

---

### 2. create_folder

**Description**: 创建新文件夹 (FR-001)

**Command Name**: `create_folder`

**Request**:
```typescript
interface CreateFolderRequest {
  name: string;
}

invoke<FolderDto>('create_folder', { name: "工作账号" })
```

**Response**:
```typescript
FolderDto  // 新创建的文件夹
```

**Errors**:
- `InvalidFolderName`: 名称为空或包含非法字符
- `DuplicateFolderName`: 名称已存在 (FR-002)
- `FolderLimitExceeded`: 已达到 100 个文件夹上限
- `Unauthorized`: 用户未登录
- `DatabaseError`: 数据库操作失败

**Validation**:
- 名称不能为空或仅包含空格 (FR-002)
- 名称长度 1-255 字符 (FR-011)
- 名称在当前用户下唯一 (FR-002)
- 危险字符自动过滤 (FR-012)

**Example**:
```typescript
try {
  const folder = await invoke<FolderDto>('create_folder', {
    name: "工作账号"
  });
  toast.success('文件夹创建成功');
} catch (error) {
  if (error === 'DuplicateFolderName') {
    toast.error('文件夹名称已存在');
  }
}
```

---

### 3. rename_folder

**Description**: 重命名文件夹 (FR-003)

**Command Name**: `rename_folder`

**Request**:
```typescript
interface RenameFolderRequest {
  folder_id: string;
  new_name: string;
}

invoke<FolderDto>('rename_folder', {
  folder_id: "uuid-here",
  new_name: "个人账号"
})
```

**Response**:
```typescript
FolderDto  // 更新后的文件夹
```

**Errors**:
- `FolderNotFound`: 文件夹不存在
- `InvalidFolderName`: 新名称为空或包含非法字符
- `DuplicateFolderName`: 新名称已存在
- `Unauthorized`: 用户未登录或无权限
- `DatabaseError`: 数据库操作失败

**Validation**:
- 与 `create_folder` 相同的名称验证规则
- 文件夹必须存在且属于当前用户

**Example**:
```typescript
try {
  const folder = await invoke<FolderDto>('rename_folder', {
    folder_id: selectedFolder.id,
    new_name: "个人账号"
  });
  toast.success('文件夹重命名成功');
} catch (error) {
  if (error === 'FolderNotFound') {
    toast.error('文件夹不存在');
  }
}
```

---

### 4. delete_folder

**Description**: 删除文件夹 (FR-004)

**Command Name**: `delete_folder`

**Request**:
```typescript
interface DeleteFolderRequest {
  folder_id: string;
}

invoke<DeleteFolderResponse>('delete_folder', {
  folder_id: "uuid-here"
})
```

**Response**:
```typescript
interface DeleteFolderResponse {
  deleted_folder_id: string;
  affected_cipher_count: number;  // 移至未分类的 cipher 数量
}
```

**Errors**:
- `FolderNotFound`: 文件夹不存在
- `Unauthorized`: 用户未登录或无权限
- `DatabaseError`: 数据库操作失败

**Side Effects** (FR-006):
- 文件夹中的所有 cipher 的 `folder_id` 设为 NULL (移至未分类)
- 文件夹记录从数据库删除

**Example**:
```typescript
try {
  const result = await invoke<DeleteFolderResponse>('delete_folder', {
    folder_id: selectedFolder.id
  });

  if (result.affected_cipher_count > 0) {
    toast.success(`文件夹已删除,${result.affected_cipher_count} 个密码项已移至未分类`);
  } else {
    toast.success('文件夹已删除');
  }
} catch (error) {
  toast.error('删除失败');
}
```

---

## Error Handling

### Error Format

所有错误通过 Tauri 的标准错误机制返回:

```typescript
try {
  await invoke('create_folder', { name: "" });
} catch (error) {
  // error 是字符串类型,对应 Rust 的 Error variant
  console.error(error); // "InvalidFolderName"
}
```

### Error Codes

| Error Code | HTTP Equivalent | Description |
|------------|-----------------|-------------|
| `InvalidFolderName` | 400 Bad Request | 名称验证失败 |
| `DuplicateFolderName` | 409 Conflict | 名称已存在 |
| `FolderNotFound` | 404 Not Found | 文件夹不存在 |
| `FolderLimitExceeded` | 429 Too Many Requests | 超过数量限制 |
| `Unauthorized` | 401 Unauthorized | 未登录或无权限 |
| `DatabaseError` | 500 Internal Server Error | 数据库错误 |

---

## Type Safety

### Rust Side (Backend)

```rust
// src-tauri/src/interfaces/tauri/commands/folder.rs

use tauri::State;
use crate::application::dto::vault::*;
use crate::support::result::AppResult;

#[tauri::command]
#[specta::specta]
pub async fn get_folders(
    state: State<'_, AppState>,
) -> AppResult<Vec<FolderDto>> {
    // Implementation
}

#[tauri::command]
#[specta::specta]
pub async fn create_folder(
    name: String,
    state: State<'_, AppState>,
) -> AppResult<FolderDto> {
    // Implementation
}

#[tauri::command]
#[specta::specta]
pub async fn rename_folder(
    folder_id: String,
    new_name: String,
    state: State<'_, AppState>,
) -> AppResult<FolderDto> {
    // Implementation
}

#[tauri::command]
#[specta::specta]
pub async fn delete_folder(
    folder_id: String,
    state: State<'_, AppState>,
) -> AppResult<DeleteFolderResponse> {
    // Implementation
}
```

### TypeScript Side (Frontend)

```typescript
// src/bindings.ts (auto-generated by specta)

export interface FolderDto {
  id: string;
  name: string;
  created_at: number;
  updated_at: number;
  cipher_count: number;
}

export interface DeleteFolderResponse {
  deleted_folder_id: string;
  affected_cipher_count: number;
}

// Type-safe invoke wrappers
export const commands = {
  getFolders: () => invoke<FolderDto[]>('get_folders'),

  createFolder: (name: string) =>
    invoke<FolderDto>('create_folder', { name }),

  renameFolder: (folderId: string, newName: string) =>
    invoke<FolderDto>('rename_folder', {
      folder_id: folderId,
      new_name: newName
    }),

  deleteFolder: (folderId: string) =>
    invoke<DeleteFolderResponse>('delete_folder', {
      folder_id: folderId
    }),
};
```

---

## Performance Expectations

| Command | Expected Latency | Notes |
|---------|------------------|-------|
| `get_folders` | <50ms | 本地 SQLite 查询 |
| `create_folder` | <100ms | 包含唯一性检查 |
| `rename_folder` | <100ms | 包含唯一性检查 |
| `delete_folder` | <150ms | 包含级联更新 cipher |

---

## Security Considerations

1. **Authentication**: 所有命令需验证用户已登录
2. **Authorization**: 用户只能操作自己的文件夹
3. **Input Sanitization**: 名称中的危险字符自动过滤 (FR-012)
4. **SQL Injection**: 使用参数化查询防止注入攻击
5. **Rate Limiting**: 前端限制创建频率,防止滥用

---

## Versioning

- **Version**: 1.0.0
- **Compatibility**: 向后兼容,新增命令不影响现有功能
- **Breaking Changes**: 如需修改接口,必须更新此文档并通知前端团队
