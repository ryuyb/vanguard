# Send Feature Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现 Vanguard 的 Send 功能，支持创建、查看、编辑、删除文本和文件类型的 Send，集成 Vaultwarden API，提供完整的 UI 界面。

**Architecture:** 采用 DDD 四层架构（Domain → Application → Infrastructure → Interfaces），使用类型状态模式处理加密/解密状态，通过 WebSocket 实现实时同步，使用 SQLite 作为本地缓存。

**Tech Stack:** Rust + Tauri (后端), React + TypeScript + TanStack Query (前端), SQLite (缓存), tauri-specta (类型安全)

---

## File Structure Overview

此功能涉及以下文件的创建或修改：

### Backend (Rust)

**Domain Layer:**
- Create: `src-tauri/src/domain/send/mod.rs` - 模块导出
- Create: `src-tauri/src/domain/send/send.rs` - Send 聚合根
- Create: `src-tauri/src/domain/send/types.rs` - SendType、SendAccess 等类型
- Create: `src-tauri/src/domain/send/state.rs` - Encrypted/Decrypted 状态

**Application Layer:**
- Create: `src-tauri/src/application/dto/send.rs` - Send DTO 定义
- Create: `src-tauri/src/application/use_cases/create_send_use_case.rs` - 创建 Send
- Create: `src-tauri/src/application/use_cases/update_send_use_case.rs` - 更新 Send
- Create: `src-tauri/src/application/use_cases/delete_send_use_case.rs` - 删除 Send
- Create: `src-tauri/src/application/use_cases/list_sends_use_case.rs` - 列出 Send
- Create: `src-tauri/src/application/use_cases/get_send_detail_use_case.rs` - 获取详情
- Create: `src-tauri/src/application/ports/send_repository_port.rs` - Repository 接口
- Create: `src-tauri/src/application/policy/send_policy.rs` - 业务策略

**Infrastructure Layer:**
- Create: `src-tauri/src/infrastructure/vaultwarden/send_adapter.rs` - API 适配器
- Create: `src-tauri/src/infrastructure/persistence/sqlite_send_repository.rs` - SQLite 实现
- Modify: `src-tauri/src/infrastructure/database/migrations/` - 数据库迁移

**Interfaces Layer:**
- Create: `src-tauri/src/interfaces/tauri/commands/send.rs` - Tauri 命令
- Create: `src-tauri/src/interfaces/tauri/dto/send.rs` - 前端 DTO
- Create: `src-tauri/src/interfaces/tauri/events/send.rs` - Send Events

**Module Registration:**
- Modify: `src-tauri/src/domain/mod.rs` - 添加 send 模块
- Modify: `src-tauri/src/application/mod.rs` - 注册 use cases
- Modify: `src-tauri/src/infrastructure/mod.rs` - 注册 adapters
- Modify: `src-tauri/src/interfaces/tauri/commands/mod.rs` - 注册 commands
- Modify: `src-tauri/src/lib.rs` - 注册 events

### Frontend (TypeScript/React)

**Feature Module:**
- Create: `src/features/send/components/send-list.tsx` - Send 列表组件
- Create: `src/features/send/components/send-detail-panel.tsx` - Send 详情面板
- Create: `src/features/send/components/send-form-dialog.tsx` - 创建/编辑表单
- Create: `src/features/send/components/send-access-config.tsx` - 访问权限配置
- Create: `src/features/send/hooks/use-send-list.ts` - 列表数据 Hook
- Create: `src/features/send/hooks/use-send-detail.ts` - 详情数据 Hook
- Create: `src/features/send/hooks/use-send-mutations.ts` - CRUD 操作 Hook
- Create: `src/features/send/hooks/use-send-file-upload.ts` - 文件上传 Hook
- Create: `src/features/send/hooks/use-send-events.ts` - Event 监听 Hook
- Create: `src/features/send/schema.ts` - 表单验证 Schema
- Create: `src/features/send/types.ts` - TypeScript 类型定义
- Create: `src/features/send/utils.ts` - 工具函数
- Create: `src/features/send/index.ts` - 模块导出

**UI Integration:**
- Modify: `src/components/app-sidebar.tsx` - 添加 Vault/Send 标签切换
- Modify: `src/app.tsx` - 集成 Send Events 监听
- Modify: `src/routes/` - 添加 Send 路由（如需要）

---

## Phase 1: Domain Layer - Core Models

### Task 1: Create Send Domain Module Structure

**Files:**
- Create: `src-tauri/src/domain/send/mod.rs`
- Create: `src-tauri/src/domain/send/state.rs`
- Create: `src-tauri/src/domain/send/types.rs`
- Modify: `src-tauri/src/domain/mod.rs`

- [ ] **Step 1: Create domain/send directory and mod.rs**

```rust
// src-tauri/src/domain/send/mod.rs
pub mod send;
pub mod state;
pub mod types;

pub use send::*;
pub use state::*;
pub use types::*;
```

- [ ] **Step 2: Implement state types for encryption status**

```rust
// src-tauri/src/domain/send/state.rs
use serde::{Deserialize, Serialize};

/// Marker trait for Send state (encrypted or decrypted)
pub trait SendState: Send + Sync + 'static {}

/// Encrypted state - data is encrypted with Send Key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Encrypted;

impl SendState for Encrypted {}

/// Decrypted state - data is decrypted and readable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decrypted;

impl SendState for Decrypted {}
```

- [ ] **Step 3: Implement Send types (SendType, SendText, SendFile)**

```rust
// src-tauri/src/domain/send/types.rs
use serde::{Deserialize, Serialize};
use specta::Type;

use super::state::SendState;

/// Send type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum SendType {
    Text = 0,
    File = 1,
}

impl SendType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SendType::Text => "text",
            SendType::File => "file",
        }
    }
}

/// Text content for Send
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SendText<S: SendState> {
    pub text: String,
    pub hidden: bool,
    #[serde(skip)]
    _state: std::marker::PhantomData<S>,
}

impl<S: SendState> SendText<S> {
    pub fn new(text: String, hidden: bool) -> Self {
        Self {
            text,
            hidden,
            _state: std::marker::PhantomData,
        }
    }
}

/// File information for Send
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SendFile<S: SendState> {
    pub id: String,
    pub file_name: String,
    pub key: Option<String>,
    pub size: Option<String>,
    pub size_name: Option<String>,
    #[serde(skip)]
    _state: std::marker::PhantomData<S>,
}

impl<S: SendState> SendFile<S> {
    pub fn new(
        id: String,
        file_name: String,
        key: Option<String>,
        size: Option<String>,
        size_name: Option<String>,
    ) -> Self {
        Self {
            id,
            file_name,
            key,
            size,
            size_name,
            _state: std::marker::PhantomData,
        }
    }
}
```

- [ ] **Step 4: Register send module in domain/mod.rs**

```rust
// src-tauri/src/domain/mod.rs (add this line)
pub mod send;
```

- [ ] **Step 5: Run tests to verify compilation**

Run: `cd src-tauri && cargo test --lib domain::send --no-run`
Expected: Compilation succeeds without errors

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/domain/send/ src-tauri/src/domain/mod.rs
git commit -m "feat(domain): add Send state and types"
```

---

### Task 2: Implement Send Aggregate Root

**Files:**
- Create: `src-tauri/src/domain/send/send.rs`
- Modify: `src-tauri/src/domain/send/mod.rs` (update exports)

- [ ] **Step 1: Write failing test for Send creation**

Create test file: `src-tauri/src/domain/send/tests.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_text_send() {
        let send = Send::<Encrypted>::new(
            "test-id".to_string(),
            SendType::Text,
            "Test Send".to_string(),
            None,
            "2026-03-31T00:00:00Z".to_string(),
        );

        assert_eq!(send.id, "test-id");
        assert_eq!(send.r#type, SendType::Text);
        assert_eq!(send.name, "Test Send");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test domain::send::tests::test_create_text_send`
Expected: FAIL - Send struct not implemented

- [ ] **Step 3: Implement Send aggregate root**

```rust
// src-tauri/src/domain/send/send.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;

use super::state::SendState;
use super::types::{SendFile, SendText, SendType};

/// Send aggregate root with type-state pattern for encryption status
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Send<S: SendState> {
    pub id: String,
    pub r#type: SendType,

    // Basic information (encrypted)
    pub name: String,
    pub notes: Option<String>,

    // Type-specific data
    pub text: Option<SendText<S>>,
    pub file: Option<SendFile<S>>,

    // Access control
    pub key: Option<String>,
    pub password: Option<String>,
    pub max_access_count: Option<i32>,
    pub access_count: i32,
    pub expiration_date: Option<String>,
    pub deletion_date: String,

    // Metadata
    pub hide_email: bool,
    pub disabled: bool,
    pub revision_date: String,

    #[serde(skip)]
    _state: std::marker::PhantomData<S>,
}

impl<S: SendState> Send<S> {
    /// Create a new Send
    pub fn new(
        id: String,
        r#type: SendType,
        name: String,
        notes: Option<String>,
        deletion_date: String,
    ) -> Self {
        Self {
            id,
            r#type,
            name,
            notes,
            text: None,
            file: None,
            key: None,
            password: None,
            max_access_count: None,
            access_count: 0,
            expiration_date: None,
            deletion_date,
            hide_email: false,
            disabled: false,
            revision_date: Utc::now().to_rfc3339(),
            _state: std::marker::PhantomData,
        }
    }

    /// Check if Send is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expiration) = &self.expiration_date {
            if let Ok(exp_time) = DateTime::parse_from_rfc3339(expiration) {
                return exp_time.with_timezone(&Utc) < Utc::now();
            }
        }
        false
    }

    /// Check if Send is deleted (deletion_date passed)
    pub fn is_deleted(&self) -> bool {
        if let Ok(del_time) = DateTime::parse_from_rfc3339(&self.deletion_date) {
            return del_time.with_timezone(&Utc) < Utc::now();
        }
        false
    }

    /// Check if Send has reached max access count
    pub fn is_access_limited(&self) -> bool {
        if let Some(max) = self.max_access_count {
            return self.access_count >= max;
        }
        false
    }

    /// Check if Send is accessible
    pub fn is_accessible(&self) -> bool {
        !self.disabled
            && !self.is_expired()
            && !self.is_deleted()
            && !self.is_access_limited()
    }
}
```

- [ ] **Step 4: Update mod.rs to export Send**

```rust
// src-tauri/src/domain/send/mod.rs (update)
pub mod send;
pub mod state;
pub mod types;

pub use send::*;
pub use state::*;
pub use types::*;
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cd src-tauri && cargo test domain::send::tests::test_create_text_send`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/domain/send/
git commit -m "feat(domain): implement Send aggregate root with state pattern"
```

---

## Phase 2: Infrastructure Layer - Database & API

### Task 3: Create Database Migration for Sends Table

**Files:**
- Create: `src-tauri/src/infrastructure/database/migrations/V012__create_sends_table.sql`

- [ ] **Step 1: Write migration SQL**

```sql
-- src-tauri/src/infrastructure/database/migrations/V012__create_sends_table.sql
CREATE TABLE sends (
    id TEXT PRIMARY KEY NOT NULL,
    account_id TEXT NOT NULL,
    type INTEGER NOT NULL,
    name TEXT NOT NULL,
    notes TEXT,
    key TEXT,
    password TEXT,
    max_access_count INTEGER,
    access_count INTEGER NOT NULL DEFAULT 0,
    expiration_date TEXT,
    deletion_date TEXT NOT NULL,
    hide_email INTEGER NOT NULL DEFAULT 0,
    disabled INTEGER NOT NULL DEFAULT 0,
    revision_date TEXT NOT NULL,
    text TEXT,  -- JSON: SendText
    file TEXT,  -- JSON: SendFile
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
);

CREATE INDEX idx_sends_account_id ON sends(account_id);
CREATE INDEX idx_sends_deletion_date ON sends(deletion_date);
CREATE INDEX idx_sends_type ON sends(type);
```

- [ ] **Step 2: Update migration version constant**

Modify: `src-tauri/src/infrastructure/database/migrations/mod.rs`

```rust
// Find the constant that defines the current migration version
// Increment it by 1
pub const CURRENT_VERSION: u32 = 12; // or whatever the next version is
```

- [ ] **Step 3: Run migration test**

Run: `cd src-tauri && cargo test --lib infrastructure::database::migrations`
Expected: All migration tests pass

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/infrastructure/database/migrations/
git commit -m "feat(db): add sends table migration"
```

---

### Task 4: Implement SendRepositoryPort

**Files:**
- Create: `src-tauri/src/application/ports/send_repository_port.rs`
- Modify: `src-tauri/src/application/ports/mod.rs`

- [ ] **Step 1: Write failing test for repository interface**

Create test stub in the port file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_repository_port_trait_bounds() {
        // This test just verifies the trait compiles correctly
        fn assert_send_sync<T: SendRepositoryPort>() {}
        // assert_send_sync::<MockSendRepository>();
    }
}
```

- [ ] **Step 2: Run test to verify compilation failure**

Run: `cd src-tauri && cargo test --lib application::ports::send_repository_port`
Expected: FAIL - trait not implemented

- [ ] **Step 3: Implement SendRepositoryPort trait**

```rust
// src-tauri/src/application/ports/send_repository_port.rs
use async_trait::async_trait;

use crate::domain::send::Send;
use crate::domain::send::state::Encrypted;
use crate::error::AppError;

#[async_trait]
pub trait SendRepositoryPort: Send + Sync {
    /// List all sends for an account
    async fn list_sends(&self, account_id: &str) -> Result<Vec<Send<Encrypted>>, AppError>;

    /// Save a send (create or update)
    async fn save_send(&self, account_id: &str, send: &Send<Encrypted>) -> Result<(), AppError>;

    /// Delete a send
    async fn delete_send(&self, account_id: &str, send_id: &str) -> Result<(), AppError>;

    /// Update access count for a send
    async fn update_access_count(
        &self,
        account_id: &str,
        send_id: &str,
        count: i32,
    ) -> Result<(), AppError>;

    /// Clear expired sends (deletion_date < now)
    async fn clear_expired_sends(&self, account_id: &str) -> Result<u64, AppError>;
}
```

- [ ] **Step 4: Register port in mod.rs**

```rust
// src-tauri/src/application/ports/mod.rs (add)
pub mod send_repository_port;

pub use send_repository_port::*;
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cd src-tauri && cargo test --lib application::ports::send_repository_port`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/application/ports/
git commit -m "feat(application): add SendRepositoryPort trait"
```

---

### Task 5: Implement SqliteSendRepository

**Files:**
- Create: `src-tauri/src/infrastructure/persistence/sqlite_send_repository.rs`
- Modify: `src-tauri/src/infrastructure/persistence/mod.rs`

- [ ] **Step 1: Write failing test for SQLite repository**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_save_and_list_sends() {
        let pool = create_test_pool().await;
        let repo = SqliteSendRepository::new(pool);

        let send = Send::<Encrypted>::new(
            "test-id".to_string(),
            SendType::Text,
            "Test Send".to_string(),
            None,
            "2026-04-01T00:00:00Z".to_string(),
        );

        repo.save_send("test-account", &send).await.unwrap();
        let sends = repo.list_sends("test-account").await.unwrap();

        assert_eq!(sends.len(), 1);
        assert_eq!(sends[0].id, "test-id");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test --lib infrastructure::persistence::sqlite_send_repository::tests::test_save_and_list_sends`
Expected: FAIL - SqliteSendRepository not implemented

- [ ] **Step 3: Implement SqliteSendRepository**

由于实现代码较长，请参考设计文档中的完整实现。关键点：
- 实现 SendRepositoryPort trait 的所有方法
- 使用 sqlx 进行数据库操作
- 实现 SendRow 到 Send<Encrypted> 的转换

- [ ] **Step 4: Register repository in mod.rs**

```rust
// src-tauri/src/infrastructure/persistence/mod.rs (add)
pub mod sqlite_send_repository;

pub use sqlite_send_repository::*;
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cd src-tauri && cargo test --lib infrastructure::persistence::sqlite_send_repository::tests::test_save_and_list_sends`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/infrastructure/persistence/
git commit -m "feat(infrastructure): implement SqliteSendRepository"
```

---

### Task 6: Implement Vaultwarden Send API Adapter

**Files:**
- Create: `src-tauri/src/infrastructure/vaultwarden/send_adapter.rs`
- Modify: `src-tauri/src/infrastructure/vaultwarden/mod.rs`

- [ ] **Step 1: Define API request/response models**

参考设计文档中的 SyncSend、CreateSendRequest 等模型定义。

- [ ] **Step 2: Implement API adapter methods**

实现 list_sends、create_send、upload_send_file、get_send、delete_send 等方法。

- [ ] **Step 3: Register adapter in mod.rs**

```rust
// src-tauri/src/infrastructure/vaultwarden/mod.rs (add)
pub mod send_adapter;

pub use send_adapter::*;
```

- [ ] **Step 4: Run tests to verify compilation**

Run: `cd src-tauri && cargo test --lib infrastructure::vaultwarden::send_adapter --no-run`
Expected: Compilation succeeds

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/infrastructure/vaultwarden/
git commit -m "feat(infrastructure): add Vaultwarden Send API adapter"
```

---

## Phase 3: Application Layer - Business Logic

### Task 7: Create Send DTOs

**Files:**
- Create: `src-tauri/src/application/dto/send.rs`
- Modify: `src-tauri/src/application/dto/mod.rs`

- [ ] **Step 1: Implement Send DTOs**

定义 CreateSendCommand、UpdateSendCommand、DeleteSendCommand、GetSendDetailQuery 等 DTO。

- [ ] **Step 2: Register DTOs in mod.rs**

- [ ] **Step 3: Run tests to verify compilation**

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/application/dto/
git commit -m "feat(application): add Send DTOs"
```

---

### Task 8: Implement SendPolicy

**Files:**
- Create: `src-tauri/src/application/policy/send_policy.rs`
- Modify: `src-tauri/src/application/policy/mod.rs`

- [ ] **Step 1: Write failing test for policy**

- [ ] **Step 2: Run tests to verify they fail**

- [ ] **Step 3: Implement SendPolicy**

实现 validate_file_size、validate_name、validate_text_content、validate_password、validate_max_access_count 等方法。

- [ ] **Step 4: Register policy in mod.rs**

- [ ] **Step 5: Run tests to verify they pass**

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/application/policy/
git commit -m "feat(application): add SendPolicy for validation"
```

---

### Task 9: Implement CreateSendUseCase

**Files:**
- Create: `src-tauri/src/application/use_cases/create_send_use_case.rs`
- Modify: `src-tauri/src/application/use_cases/mod.rs`

- [ ] **Step 1: Write failing test for CreateSendUseCase**

- [ ] **Step 2: Run test to verify compilation failure**

- [ ] **Step 3: Implement CreateSendUseCase**

实现创建 Send 的完整流程：验证、生成密钥、加密、调用 API、保存缓存。

- [ ] **Step 4: Register use case in mod.rs**

- [ ] **Step 5: Run tests to verify compilation**

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/application/use_cases/
git commit -m "feat(application): implement CreateSendUseCase"
```

---

### Task 10: Implement Other Use Cases

**Files:**
- Create: `src-tauri/src/application/use_cases/update_send_use_case.rs`
- Create: `src-tauri/src/application/use_cases/delete_send_use_case.rs`
- Create: `src-tauri/src/application/use_cases/list_sends_use_case.rs`
- Create: `src-tauri/src/application/use_cases/get_send_detail_use_case.rs`

- [ ] **Implement UpdateSendUseCase** - 遵循相同的 TDD 流程
- [ ] **Implement DeleteSendUseCase**
- [ ] **Implement ListSendsUseCase**
- [ ] **Implement GetSendDetailUseCase**
- [ ] **Register all use cases in mod.rs**
- [ ] **Run all tests**
- [ ] **Commit**

```bash
git add src-tauri/src/application/use_cases/
git commit -m "feat(application): implement remaining Send use cases"
```

---

## Phase 4: Interfaces Layer - Tauri Commands & Events

### Task 11: Implement Send Events

**Files:**
- Create: `src-tauri/src/interfaces/tauri/events/send.rs`
- Modify: `src-tauri/src/interfaces/tauri/events/mod.rs`
- Modify: `src-tauri/src/lib.rs` (register events)

- [ ] **Define SendCreated, SendUpdated, SendDeleted events**
- [ ] **Implement tauri_specta::Event trait**
- [ ] **Register events in lib.rs**
- [ ] **Test event emission**
- [ ] **Commit changes**

---

### Task 12: Implement Tauri Commands

**Files:**
- Create: `src-tauri/src/interfaces/tauri/commands/send.rs`
- Modify: `src-tauri/src/interfaces/tauri/commands/mod.rs`
- Create: `src-tauri/src/interfaces/tauri/dto/send.rs`
- Modify: `src-tauri/src/interfaces/tauri/dto/mod.rs`

- [ ] **Define frontend DTOs**
- [ ] **Implement list_sends command**
- [ ] **Implement get_send_detail command**
- [ ] **Implement create_send command**
- [ ] **Implement update_send command**
- [ ] **Implement delete_send command**
- [ ] **Register commands in mod.rs and lib.rs**
- [ ] **Test all commands**
- [ ] **Commit changes**

---

## Phase 5: Frontend Implementation

### Task 13: Create Send Feature Module Structure

**Files:**
- Create: `src/features/send/types.ts`
- Create: `src/features/send/schema.ts`
- Create: `src/features/send/utils.ts`
- Create: `src/features/send/index.ts`

- [ ] **Define TypeScript types** (based on Tauri DTOs)
- [ ] **Create form validation schema with Zod**
- [ ] **Implement utility functions** (copy link, format date, etc.)
- [ ] **Create module index file**
- [ ] **Commit changes**

---

### Task 14: Implement Send Hooks

**Files:**
- Create: `src/features/send/hooks/use-send-list.ts`
- Create: `src/features/send/hooks/use-send-detail.ts`
- Create: `src/features/send/hooks/use-send-mutations.ts`
- Create: `src/features/send/hooks/use-send-file-upload.ts`
- Create: `src/features/send/hooks/use-send-events.ts`

- [ ] **Implement useSendList hook**
- [ ] **Implement useSendDetail hook**
- [ ] **Implement useSendMutations hook**
- [ ] **Implement useSendFileUpload hook**
- [ ] **Implement useSendEvents hook**
- [ ] **Test all hooks**
- [ ] **Commit changes**

---

### Task 15: Implement Send UI Components

**Files:**
- Create: `src/features/send/components/send-list.tsx`
- Create: `src/features/send/components/send-detail-panel.tsx`
- Create: `src/features/send/components/send-form-dialog.tsx`
- Create: `src/features/send/components/send-access-config.tsx`

- [ ] **Implement SendList component**
- [ ] **Implement SendDetailPanel component**
- [ ] **Implement SendFormDialog component** (create/edit)
- [ ] **Implement SendAccessConfig component**
- [ ] **Add i18n translations**
- [ ] **Test all components**
- [ ] **Commit changes**

---

### Task 16: Integrate Send into Sidebar

**Files:**
- Modify: `src/components/app-sidebar.tsx`

- [ ] **Add Vault/Send tab switcher** at bottom of sidebar
- [ ] **Implement tab switching logic**
- [ ] **Update sidebar to show Send list when in Send mode**
- [ ] **Test tab switching**
- [ ] **Commit changes**

---

### Task 17: Enable Real-time Sync with WebSocket

**Files:**
- Modify: `src-tauri/src/infrastructure/sync/realtime_sync_service.rs`
- Modify: `src-tauri/src/application/use_cases/sync_vault_use_case.rs`

- [ ] **Implement trigger_websocket_incremental_send_sync**
- [ ] **Integrate Send sync into SyncVaultUseCase**
- [ ] **Test WebSocket sync for Send events**
- [ ] **Commit changes**

---

### Task 18: End-to-End Testing

- [ ] **Test text Send creation**
- [ ] **Test file Send creation** (with file upload)
- [ ] **Test Send editing**
- [ ] **Test Send deletion**
- [ ] **Test Send access control** (password, max access count, expiration)
- [ ] **Test WebSocket real-time sync**
- [ ] **Test offline cache**
- [ ] **Fix any bugs found**
- [ ] **Commit changes**

---

### Task 19: Update Documentation

**Files:**
- Update: `README.md` (add Send feature description)
- Create: `docs/features/send.md` (if needed)

- [ ] **Document Send feature usage**
- [ ] **Document API endpoints**
- [ ] **Document frontend components**
- [ ] **Add screenshots** (optional)
- [ ] **Commit documentation**

---

## Summary

此实施计划涵盖完整的 Send 功能开发：

**Backend (Rust/Tauri):**
- Domain Layer: Send 聚合根、类型系统、状态模式
- Application Layer: 5 个 Use Cases、DTO、Policy、Port
- Infrastructure Layer: SQLite Repository、Vaultwarden Adapter
- Interfaces Layer: Tauri Commands、Events、DTO

**Frontend (React/TypeScript):**
- Send Feature Module: Types、Hooks、Components
- Sidebar Integration: Vault/Send 标签切换
- Real-time Sync: WebSocket Events

**Estimated Tasks:** 19 major tasks, ~100 individual steps
**Testing Strategy:** TDD for all critical paths
**Commit Strategy:** Frequent commits after each working feature

---

**执行选项：**

**1. Subagent-Driven (推荐)** - 我为每个任务派遣独立的子代理，任务间进行审查，快速迭代

**2. Inline Execution** - 在当前会话中使用 executing-plans 技能执行，批量执行带检查点

请选择执行方式。