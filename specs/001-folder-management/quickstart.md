# Quickstart: Vault 文件夹管理功能

**Feature**: 001-folder-management
**Date**: 2026-03-11
**Target Audience**: 开发者

## 概述

本文档提供文件夹管理功能的快速开发指南,包括环境设置、开发流程和测试验证。

---

## 前置条件

- Rust 1.75+ (项目使用 Rust 2021 Edition)
- Node.js 18+ 和 pnpm
- SQLite 3
- Tauri CLI 2.x
- 已完成用户登录和 vault 基础功能

---

## 开发流程

### Phase 1: 后端开发 (Rust)

#### 1.1 创建 Domain 层

```bash
# 创建文件夹实体
touch src-tauri/src/domain/vault/folder.rs
```

**实现要点**:
- `Folder` 实体包含 id, name, created_at, updated_at, user_id
- `FolderName` 值对象封装验证逻辑 (非空、长度、字符过滤)
- 业务方法: `create()`, `rename()`
- 单元测试覆盖所有验证规则

**参考**: `specs/001-folder-management/data-model.md`

#### 1.2 扩展 Repository Port

```bash
# 编辑 vault repository 接口
vim src-tauri/src/application/ports/vault_repository_port.rs
```

**新增方法**:
```rust
#[async_trait]
pub trait VaultRepositoryPort: Send + Sync {
    // 现有方法...

    // 新增文件夹方法
    async fn create_folder(&self, folder: &Folder) -> Result<(), RepositoryError>;
    async fn get_folder_by_id(&self, id: &FolderId) -> Result<Option<Folder>, RepositoryError>;
    async fn get_folders_by_user(&self, user_id: &UserId) -> Result<Vec<Folder>, RepositoryError>;
    async fn update_folder(&self, folder: &Folder) -> Result<(), RepositoryError>;
    async fn delete_folder(&self, id: &FolderId) -> Result<usize, RepositoryError>; // 返回受影响的 cipher 数量
    async fn check_folder_name_exists(&self, user_id: &UserId, name: &str) -> Result<bool, RepositoryError>;
    async fn count_folders_by_user(&self, user_id: &UserId) -> Result<usize, RepositoryError>;
}
```

#### 1.3 实现 SQLite Repository

```bash
# 编辑 SQLite 实现
vim src-tauri/src/infrastructure/persistence/sqlite_vault_repository.rs
```

**实现要点**:
- 数据库迁移: 创建 `folders` 表,扩展 `ciphers` 表
- 实现所有 port 方法
- 使用参数化查询防止 SQL 注入
- 事务处理: 删除文件夹时同时更新 cipher

**Migration SQL**:
```sql
CREATE TABLE IF NOT EXISTS folders (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    user_id TEXT NOT NULL
);

ALTER TABLE ciphers ADD COLUMN folder_id TEXT;
CREATE INDEX idx_ciphers_folder_id ON ciphers(folder_id);
```

#### 1.4 创建 Use Cases

```bash
# 创建 3 个 use case 文件
touch src-tauri/src/application/use_cases/create_folder_use_case.rs
touch src-tauri/src/application/use_cases/rename_folder_use_case.rs
touch src-tauri/src/application/use_cases/delete_folder_use_case.rs
```

**Use Case 结构**:
```rust
pub struct CreateFolderUseCase {
    vault_repository: Arc<dyn VaultRepositoryPort>,
}

impl CreateFolderUseCase {
    pub async fn execute(&self, name: String, user_id: UserId) -> Result<Folder, UseCaseError> {
        // 1. 验证文件夹数量限制 (100 个)
        // 2. 创建 FolderName 值对象 (触发验证)
        // 3. 检查名称唯一性
        // 4. 创建 Folder 实体
        // 5. 持久化到数据库
        // 6. 返回结果
    }
}
```

**测试**:
```bash
cargo test --package vanguard --lib domain::vault::folder
cargo test --package vanguard --lib application::use_cases::create_folder_use_case
```

#### 1.5 创建 Tauri Commands

```bash
# 创建命令文件
touch src-tauri/src/interfaces/tauri/commands/folder.rs
```

**实现要点**:
- 使用 `#[tauri::command]` 和 `#[specta::specta]` 宏
- 从 `AppState` 获取 use case 实例
- 错误转换: `DomainError` -> `AppError` -> Tauri Error
- DTO 映射: `Folder` -> `FolderDto`

**注册命令**:
```rust
// src-tauri/src/lib.rs
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // 现有命令...
            commands::folder::get_folders,
            commands::folder::create_folder,
            commands::folder::rename_folder,
            commands::folder::delete_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

#### 1.6 生成 TypeScript 绑定

```bash
# 运行 specta 生成类型
cargo run --bin generate-bindings
# 或者在开发模式下自动生成
pnpm tauri dev
```

**验证**: 检查 `src/bindings.ts` 是否包含新的类型定义

---

### Phase 2: 前端开发 (React + TypeScript)

#### 2.1 创建 Hooks

```bash
# 创建文件夹管理 hooks
mkdir -p src/features/vault/hooks
touch src/features/vault/hooks/use-folders.ts
touch src/features/vault/hooks/use-folder-create.ts
touch src/features/vault/hooks/use-folder-rename.ts
touch src/features/vault/hooks/use-folder-delete.ts
```

**use-folders.ts** (使用 TanStack Query):
```typescript
import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import type { FolderDto } from '@/bindings';

export function useFolders() {
  return useQuery({
    queryKey: ['folders'],
    queryFn: () => invoke<FolderDto[]>('get_folders'),
    staleTime: 5 * 60 * 1000, // 5 分钟
  });
}
```

**use-folder-create.ts**:
```typescript
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { toast } from 'sonner';

export function useCreateFolder() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (name: string) => invoke('create_folder', { name }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['folders'] });
      toast.success('文件夹创建成功');
    },
    onError: (error: string) => {
      if (error === 'DuplicateFolderName') {
        toast.error('文件夹名称已存在');
      } else if (error === 'InvalidFolderName') {
        toast.error('文件夹名称无效');
      } else {
        toast.error('创建失败,请重试');
      }
    },
  });
}
```

#### 2.2 创建 UI 组件

```bash
# 创建组件文件
mkdir -p src/features/vault/components
touch src/features/vault/components/folder-list.tsx
touch src/features/vault/components/folder-create-dialog.tsx
touch src/features/vault/components/folder-rename-dialog.tsx
touch src/features/vault/components/folder-delete-dialog.tsx
```

**folder-create-dialog.tsx** (示例):
```typescript
import { useState } from 'react';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { useCreateFolder } from '../hooks/use-folder-create';

export function FolderCreateDialog({ open, onOpenChange }) {
  const [name, setName] = useState('');
  const createFolder = useCreateFolder();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    await createFolder.mutateAsync(name);
    setName('');
    onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>新建文件夹</DialogTitle>
        </DialogHeader>
        <form onSubmit={handleSubmit}>
          <Input
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="输入文件夹名称"
            maxLength={255}
            autoFocus
          />
          <div className="flex justify-end gap-2 mt-4">
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              取消
            </Button>
            <Button type="submit" disabled={!name.trim() || createFolder.isPending}>
              创建
            </Button>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  );
}
```

#### 2.3 集成到 Vault 页面

```bash
# 编辑 vault 路由
vim src/routes/vault.tsx
```

**集成要点**:
- 在侧边栏显示文件夹列表
- 添加"新建文件夹"按钮
- 文件夹右键菜单: 重命名、删除
- 点击文件夹过滤 cipher 列表

---

### Phase 3: 测试

#### 3.1 后端测试

```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test domain::vault::folder
cargo test application::use_cases::create_folder

# 集成测试
cargo test --test folder_operations_test
```

**测试覆盖**:
- Domain 层: 验证规则、业务逻辑
- Use Case 层: 完整流程、错误处理
- Repository 层: 数据库操作、事务

#### 3.2 前端测试

```bash
# 安装测试依赖
pnpm add -D vitest @testing-library/react @testing-library/user-event

# 运行测试
pnpm test

# 覆盖率报告
pnpm test:coverage
```

**测试示例**:
```typescript
// src/features/vault/hooks/use-folder-create.test.ts
import { renderHook, waitFor } from '@testing-library/react';
import { useCreateFolder } from './use-folder-create';

test('should create folder successfully', async () => {
  const { result } = renderHook(() => useCreateFolder());

  result.current.mutate('工作账号');

  await waitFor(() => expect(result.current.isSuccess).toBe(true));
});
```

#### 3.3 手动测试

**测试场景** (from spec.md):

1. **创建文件夹**:
   - ✅ 输入有效名称,创建成功
   - ✅ 输入空名称,显示错误
   - ✅ 输入重复名称,显示错误
   - ✅ 点击取消,不创建

2. **重命名文件夹**:
   - ✅ 输入新名称,重命名成功
   - ✅ 输入空名称,显示错误
   - ✅ 输入重复名称,显示错误
   - ✅ 文件夹中的 cipher 保持不变

3. **删除文件夹**:
   - ✅ 删除空文件夹,成功
   - ✅ 删除包含 cipher 的文件夹,显示警告
   - ✅ 确认删除,cipher 移至未分类
   - ✅ 点击取消,不删除

---

## 性能验证

### 性能目标 (from spec.md SC-001 ~ SC-003)

- 创建文件夹: <3 秒 (目标 <100ms)
- 重命名文件夹: <3 秒 (目标 <100ms)
- 删除文件夹: <5 秒 (目标 <150ms)

### 验证方法

```typescript
// 前端性能测试
console.time('create_folder');
await invoke('create_folder', { name: '测试文件夹' });
console.timeEnd('create_folder'); // 应 <100ms
```

```rust
// 后端性能测试
#[tokio::test]
async fn test_create_folder_performance() {
    let start = Instant::now();
    let result = use_case.execute("测试文件夹".to_string(), user_id).await;
    let duration = start.elapsed();

    assert!(result.is_ok());
    assert!(duration.as_millis() < 100, "创建文件夹耗时过长: {:?}", duration);
}
```

---

## 常见问题

### Q1: 如何处理文件夹名称中的特殊字符?

**A**: `FolderName::sanitize()` 方法自动将 `/`, `\`, `<`, `>` 等危险字符替换为 `_`。

### Q2: 删除文件夹后,cipher 去哪了?

**A**: cipher 的 `folder_id` 设为 NULL,在 UI 中显示为"未分类"或根目录。

### Q3: 如何限制文件夹数量为 100 个?

**A**: `CreateFolderUseCase` 在创建前调用 `count_folders_by_user()`,超过 100 返回 `FolderLimitExceeded` 错误。

### Q4: 前端如何获取最新的文件夹列表?

**A**: TanStack Query 自动管理缓存,mutation 成功后调用 `invalidateQueries` 触发重新获取。

---

## 下一步

完成开发后:

1. 运行 `pnpm biome:check` 检查代码风格
2. 运行 `cargo clippy` 检查 Rust 代码
3. 提交代码前运行所有测试
4. 更新 CHANGELOG.md
5. 创建 Pull Request

**相关文档**:
- [Spec](./spec.md): 功能需求
- [Data Model](./data-model.md): 数据模型
- [Contracts](./contracts/tauri-commands.md): API 契约
