# Data Model: Vault 文件夹管理功能

**Feature**: 001-folder-management
**Date**: 2026-03-11
**Source**: Extracted from spec.md requirements

## Entities

### 1. Folder (文件夹)

**Description**: 用于组织 cipher 的容器实体

**Fields**:

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| id | String (UUID) | PRIMARY KEY, NOT NULL | 唯一标识符 |
| name | String | NOT NULL, UNIQUE, 1-255 chars | 文件夹名称 |
| created_at | DateTime | NOT NULL | 创建时间 (UTC) |
| updated_at | DateTime | NOT NULL | 最后修改时间 (UTC) |
| user_id | String (UUID) | NOT NULL, FOREIGN KEY | 所属用户 ID |

**Validation Rules** (from FR-002, FR-010, FR-011, FR-012):
- 名称不能为空或仅包含空格
- 名称长度 1-255 个字符
- 名称在同一用户下必须唯一
- 名称支持中文、英文、数字和常见标点符号
- 名称必须过滤或转义危险字符 (如 `/`, `\`, `<`, `>`)

**Business Rules**:
- 一个用户最多创建 100 个文件夹 (from Assumptions)
- 删除文件夹时,其中的 cipher 移至未分类区域 (from FR-006)
- 文件夹是扁平结构,不支持嵌套 (from Assumptions)

**State Transitions**:
```
[Not Exists] --create--> [Active]
[Active] --rename--> [Active] (name updated)
[Active] --delete--> [Deleted]
```

---

### 2. Cipher (密码项)

**Description**: 存储在文件夹中的密码条目 (已存在实体,此处仅列出扩展字段)

**Extended Fields**:

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| folder_id | String (UUID) | NULLABLE, FOREIGN KEY | 所属文件夹 ID (NULL 表示未分类) |

**Relationships**:
- Many-to-One with Folder: 一个 cipher 属于一个文件夹 (或未分类)
- ON DELETE SET NULL: 删除文件夹时,cipher 的 folder_id 设为 NULL

---

## Relationships

```
User (1) ----< (N) Folder
Folder (1) ----< (N) Cipher
```

**Cardinality**:
- 一个用户可以有多个文件夹 (0..100)
- 一个文件夹属于一个用户
- 一个文件夹可以包含多个 cipher (0..N)
- 一个 cipher 属于零个或一个文件夹 (NULL = 未分类)

---

## Domain Model (Rust)

### Folder Entity

```rust
// src-tauri/src/domain/vault/folder.rs

use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Folder {
    id: FolderId,
    name: FolderName,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    user_id: UserId,
}

impl Folder {
    /// 创建新文件夹 (FR-001)
    pub fn create(name: FolderName, user_id: UserId) -> Result<Self, DomainError> {
        Ok(Self {
            id: FolderId::new(),
            name,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            user_id,
        })
    }

    /// 重命名文件夹 (FR-003)
    pub fn rename(&mut self, new_name: FolderName) -> Result<(), DomainError> {
        self.name = new_name;
        self.updated_at = Utc::now();
        Ok(())
    }

    // Getters
    pub fn id(&self) -> &FolderId { &self.id }
    pub fn name(&self) -> &FolderName { &self.name }
    pub fn created_at(&self) -> DateTime<Utc> { self.created_at }
    pub fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
    pub fn user_id(&self) -> &UserId { &self.user_id }
}
```

### Value Objects

```rust
// FolderId
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FolderId(Uuid);

impl FolderId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_string(s: &str) -> Result<Self, DomainError> {
        Uuid::parse_str(s)
            .map(Self)
            .map_err(|_| DomainError::InvalidFolderId)
    }
}

// FolderName (with validation)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FolderName(String);

impl FolderName {
    pub fn new(name: String) -> Result<Self, DomainError> {
        let trimmed = name.trim();

        // FR-002: 名称不能为空
        if trimmed.is_empty() {
            return Err(DomainError::InvalidFolderName("文件夹名称不能为空".into()));
        }

        // FR-011: 长度限制 1-255 字符
        if trimmed.len() > 255 {
            return Err(DomainError::InvalidFolderName("文件夹名称过长(最多255字符)".into()));
        }

        // FR-012: 过滤危险字符
        let sanitized = Self::sanitize(trimmed);

        Ok(Self(sanitized))
    }

    fn sanitize(name: &str) -> String {
        // 移除或替换路径分隔符和危险字符
        name.replace(['/', '\\', '<', '>', ':', '"', '|', '?', '*'], "_")
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

---

## Database Schema (SQLite)

```sql
-- 新增 folders 表
CREATE TABLE IF NOT EXISTS folders (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    created_at INTEGER NOT NULL,  -- Unix timestamp (milliseconds)
    updated_at INTEGER NOT NULL,
    user_id TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- 扩展 ciphers 表 (假设已存在)
ALTER TABLE ciphers ADD COLUMN folder_id TEXT;

-- 外键约束 (如果 SQLite 支持)
-- ALTER TABLE ciphers ADD CONSTRAINT fk_folder
--   FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE SET NULL;

-- 索引优化
CREATE INDEX IF NOT EXISTS idx_folders_user_id ON folders(user_id);
CREATE INDEX IF NOT EXISTS idx_folders_name ON folders(name);
CREATE INDEX IF NOT EXISTS idx_ciphers_folder_id ON ciphers(folder_id);
```

**Migration Notes**:
- 使用 `INTEGER` 存储时间戳 (Unix milliseconds) 以兼容 rusqlite
- `folder_id` 为 NULL 表示 cipher 未分类
- 删除文件夹时,相关 cipher 的 `folder_id` 设为 NULL (通过应用层逻辑实现)

---

## DTOs (Data Transfer Objects)

### Application Layer DTO

```rust
// src-tauri/src/application/dto/vault.rs

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FolderDto {
    pub id: String,
    pub name: String,
    pub created_at: i64,  // Unix timestamp (ms)
    pub updated_at: i64,
    pub cipher_count: usize,  // 包含的 cipher 数量
}

#[derive(Debug, Clone, Deserialize, Type)]
pub struct CreateFolderRequest {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Type)]
pub struct RenameFolderRequest {
    pub folder_id: String,
    pub new_name: String,
}

#[derive(Debug, Clone, Deserialize, Type)]
pub struct DeleteFolderRequest {
    pub folder_id: String,
}
```

### Frontend TypeScript Types (auto-generated by specta)

```typescript
// src/bindings.ts (auto-generated)

export interface FolderDto {
  id: string;
  name: string;
  created_at: number;
  updated_at: number;
  cipher_count: number;
}

export interface CreateFolderRequest {
  name: string;
}

export interface RenameFolderRequest {
  folder_id: string;
  new_name: string;
}

export interface DeleteFolderRequest {
  folder_id: string;
}
```

---

## Validation Summary

| Requirement | Validation Location | Implementation |
|-------------|---------------------|----------------|
| FR-002: 名称非空 | Domain (FolderName) | `trimmed.is_empty()` check |
| FR-002: 名称唯一 | Repository | Database UNIQUE constraint + query check |
| FR-011: 长度 1-255 | Domain (FolderName) | `trimmed.len() > 255` check |
| FR-012: 危险字符 | Domain (FolderName) | `sanitize()` method |
| Assumption: 最多 100 个 | Use Case | Count query before create |

---

## Error Handling

```rust
// src-tauri/src/domain/vault/error.rs

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("文件夹名称不能为空")]
    EmptyFolderName,

    #[error("文件夹名称过长(最多255字符)")]
    FolderNameTooLong,

    #[error("文件夹名称已存在")]
    DuplicateFolderName,

    #[error("文件夹不存在")]
    FolderNotFound,

    #[error("已达到文件夹数量上限(100个)")]
    FolderLimitExceeded,

    #[error("无效的文件夹ID")]
    InvalidFolderId,

    #[error("文件夹名称无效: {0}")]
    InvalidFolderName(String),
}
```

---

## Summary

- **核心实体**: Folder (新增), Cipher (扩展)
- **关系**: User 1:N Folder, Folder 1:N Cipher
- **验证规则**: 名称非空、长度限制、唯一性、字符过滤
- **数据库**: SQLite 新增 `folders` 表,扩展 `ciphers` 表
- **类型安全**: 使用 specta 自动生成 TypeScript 类型
