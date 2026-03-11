# Research: Vault 文件夹管理功能

**Feature**: 001-folder-management
**Date**: 2026-03-11
**Status**: Complete

## Research Tasks

### 1. 前端测试框架选择

**Decision**: Vitest + React Testing Library

**Rationale**:
- Vitest 是 Vite 生态的原生测试框架,与项目现有的 Vite 构建工具无缝集成
- React Testing Library 是 React 社区标准,专注于用户行为测试而非实现细节
- 性能优异: Vitest 基于 Vite 的 HMR,测试执行速度快
- TypeScript 原生支持,无需额外配置
- 与现有工具链兼容: 支持 ESM, TypeScript 5.8

**Alternatives Considered**:
- Jest: 传统选择,但需要额外配置 ESM 和 TypeScript,与 Vite 集成不如 Vitest 流畅
- Playwright Component Testing: 适合 E2E 测试,但对单元测试过重

**Implementation**:
```json
// package.json 新增依赖
{
  "devDependencies": {
    "vitest": "^2.0.0",
    "@testing-library/react": "^16.0.0",
    "@testing-library/user-event": "^14.5.0",
    "@testing-library/jest-dom": "^6.5.0"
  },
  "scripts": {
    "test": "vitest",
    "test:ui": "vitest --ui",
    "test:coverage": "vitest --coverage"
  }
}
```

---

### 2. SQLite 数据库 Schema 设计

**Decision**: 新增 `folders` 表,扩展 `ciphers` 表添加 `folder_id` 外键

**Rationale**:
- 遵循关系型数据库设计原则,文件夹和 cipher 是一对多关系
- 使用外键约束保证数据完整性
- 支持级联操作: 删除文件夹时自动处理关联的 cipher
- 索引优化: 在 `folder_id` 上建立索引以加速查询

**Schema**:
```sql
-- 新增 folders 表
CREATE TABLE IF NOT EXISTS folders (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    user_id TEXT NOT NULL
);

-- 扩展 ciphers 表 (假设已存在)
ALTER TABLE ciphers ADD COLUMN folder_id TEXT REFERENCES folders(id) ON DELETE SET NULL;

-- 索引优化
CREATE INDEX idx_ciphers_folder_id ON ciphers(folder_id);
CREATE INDEX idx_folders_user_id ON folders(user_id);
```

**Migration Strategy**:
- 使用 rusqlite 的 migration 机制
- 向后兼容: 现有 cipher 的 `folder_id` 默认为 NULL (表示未分类)
- 迁移脚本位置: `src-tauri/src/infrastructure/persistence/migrations/`

---

### 3. Rust Domain 模型设计最佳实践

**Decision**: 使用 Rich Domain Model + Value Objects 模式

**Rationale**:
- Rich Domain Model: Folder 实体包含业务逻辑验证(名称长度、唯一性检查)
- Value Objects: FolderName 作为值对象封装验证规则
- 不可变性: 使用 Rust 的所有权系统保证数据一致性
- 错误处理: 使用 Result<T, DomainError> 显式处理业务规则违规

**Pattern**:
```rust
// domain/vault/folder.rs
pub struct Folder {
    id: FolderId,
    name: FolderName,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    user_id: UserId,
}

impl Folder {
    pub fn create(name: FolderName, user_id: UserId) -> Result<Self, DomainError> {
        // 业务规则验证
        Ok(Self {
            id: FolderId::new(),
            name,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            user_id,
        })
    }

    pub fn rename(&mut self, new_name: FolderName) -> Result<(), DomainError> {
        self.name = new_name;
        self.updated_at = Utc::now();
        Ok(())
    }
}

// Value Object
pub struct FolderName(String);

impl FolderName {
    pub fn new(name: String) -> Result<Self, DomainError> {
        if name.trim().is_empty() {
            return Err(DomainError::InvalidFolderName("名称不能为空".into()));
        }
        if name.len() > 255 {
            return Err(DomainError::InvalidFolderName("名称过长".into()));
        }
        Ok(Self(name))
    }
}
```

**Alternatives Considered**:
- Anemic Domain Model: 将验证逻辑放在 use case 层,但违反 DDD 原则
- Active Record: 将数据库操作混入实体,但违反关注点分离

---

### 4. React 状态管理策略

**Decision**: 使用 TanStack Query (React Query) 管理服务端状态

**Rationale**:
- 服务端状态管理: 文件夹数据来自 Tauri 后端,属于服务端状态
- 自动缓存和同步: 减少不必要的后端调用
- 乐观更新: 提升用户体验,操作立即反馈
- 错误处理和重试: 内置机制处理网络错误
- 与 Tauri 集成良好: 通过 invoke 调用后端命令

**Implementation**:
```typescript
// features/vault/hooks/use-folders.ts
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';

export function useFolders() {
  return useQuery({
    queryKey: ['folders'],
    queryFn: () => invoke<Folder[]>('get_folders'),
  });
}

export function useCreateFolder() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (name: string) => invoke('create_folder', { name }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['folders'] });
    },
  });
}
```

**Alternatives Considered**:
- Zustand/Jotai: 适合客户端状态,但对服务端状态管理不如 React Query 专业
- Redux: 过于复杂,不适合小型桌面应用

---

### 5. 错误处理和用户反馈

**Decision**: 使用 Sonner (Toast 通知库) + 错误边界

**Rationale**:
- Sonner 已在项目依赖中,保持一致性
- Toast 通知适合非阻塞式反馈(成功、警告、错误)
- 对话框确认适合破坏性操作(删除文件夹)
- 错误边界捕获 React 组件错误,防止应用崩溃

**Pattern**:
```typescript
// 成功反馈
toast.success('文件夹创建成功');

// 错误反馈
toast.error('文件夹名称已存在');

// 确认对话框 (删除操作)
<AlertDialog>
  <AlertDialogContent>
    <AlertDialogTitle>确认删除</AlertDialogTitle>
    <AlertDialogDescription>
      此文件夹包含 {cipherCount} 个密码项,删除后这些项将移至未分类
    </AlertDialogDescription>
    <AlertDialogAction onClick={handleDelete}>删除</AlertDialogAction>
    <AlertDialogCancel>取消</AlertDialogCancel>
  </AlertDialogContent>
</AlertDialog>
```

---

## Summary

所有 "NEEDS CLARIFICATION" 项已解决:

1. **前端测试框架**: Vitest + React Testing Library
2. **数据库设计**: 新增 `folders` 表,扩展 `ciphers` 表
3. **Domain 模型**: Rich Domain Model + Value Objects
4. **状态管理**: TanStack Query
5. **错误处理**: Sonner Toast + 对话框确认

这些技术选择与项目现有技术栈一致,无需引入新的依赖或学习曲线,符合 Constitution 的简洁性原则。
