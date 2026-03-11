# 可行性分析报告

**需求描述**: 在 vault 页面实现文件夹的新增、修改和删除功能
**分析日期**: 2026-03-11
**分析人**: Claude AI

---

## 1. 需求概述

用户希望在 vault 页面增加文件夹管理功能,具体包括:
- **新增文件夹**: 创建新的文件夹用于组织 cipher
- **修改文件夹**: 重命名现有文件夹
- **删除文件夹**: 删除不需要的文件夹

### 业务价值
- 提升用户体验,允许用户直接在应用内管理文件夹结构
- 减少对 Web 端的依赖,提高应用的独立性
- 符合密码管理器的标准功能预期

### 用户场景
- 用户在 vault 页面左侧文件夹树中右键点击文件夹,选择"重命名"或"删除"
- 用户点击"新建文件夹"按钮,输入文件夹名称后创建

---

## 2. 技术可行性

### 2.1 代码库兼容性

#### ✅ 已有基础设施

1. **后端架构完善**
   - 采用 Clean Architecture (DDD 分层架构)
   - 已有 `VaultwardenEndpoints::folder()` API 端点定义
   - 已有 `delete_folder_live()` 方法用于删除文件夹
   - 数据库表 `live_folders` 和 `staging_folders` 已存在

2. **前端组件基础**
   - 已有 `FolderTreeMenuItem` 组件展示文件夹树
   - 已有 `VaultFolderItemDto` 数据结构
   - 使用 Tauri 2.x + React + TypeScript 技术栈

3. **数据同步机制**
   - 已有 `sync_vault_use_case.rs` 处理文件夹增量更新
   - 已有 WebSocket 实时同步支持

#### ⚠️ 需要补充的部分

1. **后端缺失的 API 命令**
   - 缺少 `vault_create_folder` Tauri 命令
   - 缺少 `vault_update_folder` Tauri 命令
   - 缺少 `vault_delete_folder` Tauri 命令

2. **前端缺失的 UI 组件**
   - 缺少文件夹操作的上下文菜单 (右键菜单)
   - 缺少新建/编辑文件夹的对话框
   - 缺少删除确认对话框

3. **远程 API 调用**
   - 需要实现 `RemoteVaultPort` 中的文件夹 CRUD 方法
   - 需要调用 Vaultwarden API (`POST /api/folders`, `PUT /api/folders/{id}`, `DELETE /api/folders/{id}`)

### 2.2 架构影响评估

- **模块耦合度**: 中等 (需修改 3-4 个模块)
  - `src-tauri/src/interfaces/tauri/commands/vault.rs` (新增命令)
  - `src-tauri/src/application/use_cases/` (新增 use case)
  - `src-tauri/src/infrastructure/vaultwarden/` (实现 API 调用)
  - `src/features/vault/components/` (新增 UI 组件)

- **API 变更**: 新增 3 个 Tauri 命令,不影响现有接口
- **数据迁移**: 无需数据库 schema 变更
- **向后兼容性**: 完全兼容,不破坏现有功能

---

## 3. 实现复杂度

| 维度 | 评分 | 说明 |
|------|------|------|
| 代码量 | 3/5 | 预估 500-800 行代码 (后端 300 行 + 前端 400 行) |
| 技术难度 | 2/5 | 遵循现有架构模式,无复杂算法 |
| 测试复杂度 | 3/5 | 需要测试 API 调用、UI 交互、数据同步 |
| 风险等级 | 2/5 | 主要风险在于 API 调用失败处理 |
| 依赖复杂度 | 1/5 | 无需新增外部依赖 |

**总体复杂度**: 2.2/5.0 (中等偏简单)
**预估工时**: 3-5 天

---

## 4. 风险识别

### 中风险 🟡

1. **API 调用失败处理**
   - 网络异常时的错误提示
   - 文件夹名称冲突处理
   - 删除非空文件夹的策略 (是否级联删除 cipher)

2. **数据同步一致性**
   - 本地操作后需立即同步到远程
   - WebSocket 实时同步可能导致的竞态条件

3. **UI 交互体验**
   - 右键菜单的触发方式 (桌面端 vs 移动端)
   - 文件夹树的刷新时机

### 低风险 🟢

1. **代码质量**
   - 遵循现有 Clean Architecture 模式
   - TypeScript 类型安全保障

2. **性能影响**
   - 文件夹操作频率低,不影响性能

---

## 5. 实现方案

### 方案 A: 完整实现 (推荐) ⭐⭐⭐⭐⭐

#### 核心思路
按照 Clean Architecture 分层实现完整的文件夹 CRUD 功能,包括:
1. 后端实现 3 个 use case (CreateFolder, UpdateFolder, DeleteFolder)
2. 前端实现上下文菜单和对话框组件
3. 集成到现有的 vault 页面

#### 优点
- 功能完整,用户体验最佳
- 遵循现有架构模式,代码质量高
- 易于维护和扩展

#### 缺点
- 开发工时较长 (3-5 天)
- 需要处理较多边界情况

#### 预估工时
3-5 天

#### 实现步骤

**第一阶段: 后端实现 (1.5-2 天)**

1. 定义 DTO 和命令 (`src-tauri/src/interfaces/tauri/dto/vault.rs`)
   ```rust
   pub struct VaultCreateFolderRequestDto {
       pub name: String,
   }

   pub struct VaultUpdateFolderRequestDto {
       pub folder_id: String,
       pub name: String,
   }

   pub struct VaultDeleteFolderRequestDto {
       pub folder_id: String,
   }
   ```

2. 实现 Use Cases (`src-tauri/src/application/use_cases/`)
   - `create_folder_use_case.rs`
   - `update_folder_use_case.rs`
   - `delete_folder_use_case.rs`

3. 实现 RemoteVaultPort 方法 (`src-tauri/src/infrastructure/vaultwarden/port_adapter.rs`)
   - `create_folder()`
   - `update_folder()`
   - `delete_folder()`

4. 添加 Tauri 命令 (`src-tauri/src/interfaces/tauri/commands/vault.rs`)
   - `vault_create_folder()`
   - `vault_update_folder()`
   - `vault_delete_folder()`

**第二阶段: 前端实现 (1.5-2 天)**

1. 创建对话框组件 (`src/features/vault/components/`)
   - `folder-create-dialog.tsx` (新建文件夹对话框)
   - `folder-edit-dialog.tsx` (编辑文件夹对话框)
   - `folder-delete-dialog.tsx` (删除确认对话框)

2. 修改 `FolderTreeMenuItem` 组件
   - 添加右键菜单支持
   - 集成对话框触发逻辑

3. 更新 `useVaultPageModel` hook
   - 添加文件夹操作的状态管理
   - 实现 API 调用逻辑

4. 更新 TypeScript 类型定义 (`src/bindings.ts`)
   - 运行 `cargo build` 自动生成新的类型定义

**第三阶段: 测试与优化 (0.5-1 天)**

1. 单元测试
   - 后端 use case 测试
   - 前端组件测试

2. 集成测试
   - 端到端测试文件夹 CRUD 流程
   - 测试数据同步一致性

3. UI/UX 优化
   - 加载状态提示
   - 错误提示优化

---

### 方案 B: 最小可行实现 ⭐⭐⭐

#### 核心思路
仅实现文件夹的重命名和删除功能,暂不实现新建功能 (因为新建功能需要额外的 UI 入口)

#### 优点
- 开发工时短 (2-3 天)
- 快速验证技术方案

#### 缺点
- 功能不完整,用户体验受限
- 后续仍需补充新建功能

#### 预估工时
2-3 天

---

## 6. 建议

### 是否建议实施
**是** - 该功能技术可行,风险可控,能显著提升用户体验

### 优先级建议
**中** - 非核心功能,但对用户体验有明显提升

### 前置条件
- 确认 Vaultwarden API 支持文件夹 CRUD 操作
- 确认删除文件夹时的业务逻辑 (是否级联删除 cipher)

### 后续步骤
1. **使用 `/create-issue` 创建 GitHub Issue** 跟踪此功能开发
2. 与产品团队确认删除文件夹的业务规则
3. 开始实施方案 A (推荐)

---

## 7. 技术细节补充

### API 端点设计

根据 Vaultwarden API 规范,需要调用以下端点:

```
POST   /api/folders           # 创建文件夹
PUT    /api/folders/{id}      # 更新文件夹
DELETE /api/folders/{id}      # 删除文件夹
```

### 数据流设计

```
用户操作 (前端)
  ↓
Tauri 命令 (IPC)
  ↓
Use Case (应用层)
  ↓
RemoteVaultPort (基础设施层)
  ↓
Vaultwarden API (远程服务)
  ↓
本地数据库同步 (SQLite)
  ↓
UI 刷新 (React 状态更新)
```

### 错误处理策略

1. **网络错误**: 显示"网络连接失败,请稍后重试"
2. **名称冲突**: 显示"文件夹名称已存在,请使用其他名称"
3. **权限错误**: 显示"您没有权限执行此操作"
4. **非空文件夹删除**: 显示"此文件夹包含 X 个项目,确认删除吗?"

---

**生成命令**: 使用 `/create-issue` 自动创建 GitHub Issue

---

**GitHub Issue**: https://github.com/ryuyb/vanguard/issues/2
**Issue Number**: #2
**创建时间**: 2026-03-11 13:24:00
