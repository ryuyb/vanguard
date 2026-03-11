# Implementation Plan: Vault 文件夹管理功能

**Branch**: `001-folder-management` | **Date**: 2026-03-11 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-folder-management/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

实现 vault 页面的文件夹管理功能,包括创建、重命名和删除文件夹。用户可以通过文件夹组织和分类密码项(cipher),提升密码库的可管理性。技术方案采用 Tauri 桌面应用架构,后端使用 Rust 实现领域逻辑和数据持久化,前端使用 React + TypeScript 实现用户界面。

## Technical Context

**Language/Version**: Rust 2021 Edition (后端), TypeScript 5.8 (前端)
**Primary Dependencies**: Tauri 2.x, React 19.x, TanStack Router, rusqlite 0.38, specta/tauri-specta (类型安全的前后端通信)
**Storage**: SQLite (通过 rusqlite,已有 vault 数据库)
**Testing**: cargo test (后端), Vitest + React Testing Library (前端)
**Target Platform**: macOS desktop (主要), Windows/Linux (次要支持)
**Project Type**: desktop-app (Tauri 跨平台桌面应用)
**Performance Goals**: 文件夹操作响应时间 <100ms, UI 更新 <50ms, 支持至少 100 个文件夹
**Constraints**: 文件夹名称 1-255 字符, 扁平结构(不支持嵌套), 需要用户认证, 操作需持久化到 SQLite
**Scale/Scope**: 单用户应用, 预计 100 个文件夹上限, 每个文件夹可包含数百个 cipher

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Code Quality Gate**: ✅ PASS
  - 最小可行设计: 复用现有 vault 架构模式(domain/application/infrastructure/interfaces 四层)
  - 具体文件变更:
    - 后端: `domain/vault/folder.rs` (新增), `application/use_cases/folder_*.rs` (新增 3 个), `infrastructure/persistence/sqlite_vault_repository.rs` (扩展), `interfaces/tauri/commands/folder.rs` (新增)
    - 前端: `features/vault/components/folder-*.tsx` (新增 3 个), `features/vault/hooks/use-folder-*.ts` (新增), `routes/vault.tsx` (扩展)
  - 无新依赖: 复用现有 rusqlite, tauri-specta, React 生态
  - 无新抽象层: 遵循现有 Clean Architecture 模式

- **Testing Gate**: ✅ PASS
  - 后端单元测试: `cargo test` 覆盖 folder domain 逻辑、use cases、repository 操作
  - 后端集成测试: SQLite 数据库操作的完整流程测试
  - 前端测试: NEEDS CLARIFICATION (待 Phase 0 研究确定测试框架)
  - 手动验证: 每个 user story 的 acceptance scenarios 需手动测试

- **UX Consistency Gate**: ✅ PASS
  - 复用现有 vault 页面的设计语言和交互模式
  - 文件夹操作使用与 cipher 操作一致的对话框和按钮样式
  - 错误提示使用现有 toast/notification 组件
  - 确认对话框遵循现有的 destructive action 模式
  - 可访问性: 键盘导航、ARIA 标签、屏幕阅读器支持(遵循现有标准)

- **Performance Gate**: ✅ PASS
  - 文件夹 CRUD 操作延迟 <100ms (SQLite 本地操作)
  - UI 渲染更新 <50ms (React 状态更新)
  - 支持 100 个文件夹无性能下降(列表虚拟化如需要)
  - 验证方式: 性能测试用例 + 手动计时验证

## Project Structure

### Documentation (this feature)

```text
specs/001-folder-management/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
# Tauri 桌面应用架构 (前后端分离)

# 后端 (Rust)
src-tauri/src/
├── domain/
│   └── vault/
│       ├── folder.rs           # 新增: Folder 实体和业务规则
│       └── mod.rs              # 扩展: 导出 folder 模块
├── application/
│   ├── use_cases/
│   │   ├── create_folder_use_case.rs      # 新增
│   │   ├── rename_folder_use_case.rs      # 新增
│   │   ├── delete_folder_use_case.rs      # 新增
│   │   └── get_vault_view_data_use_case.rs # 扩展: 包含文件夹数据
│   ├── ports/
│   │   └── vault_repository_port.rs       # 扩展: 添加文件夹操作接口
│   └── dto/
│       └── vault.rs                        # 扩展: 添加 FolderDto
├── infrastructure/
│   └── persistence/
│       └── sqlite_vault_repository.rs      # 扩展: 实现文件夹 CRUD
└── interfaces/
    └── tauri/
        ├── commands/
        │   └── folder.rs                   # 新增: Tauri 命令
        └── dto/
            └── vault.rs                    # 扩展: 前端 DTO

# 前端 (React + TypeScript)
src/
├── features/
│   └── vault/
│       ├── components/
│       │   ├── folder-create-dialog.tsx    # 新增
│       │   ├── folder-rename-dialog.tsx    # 新增
│       │   ├── folder-delete-dialog.tsx    # 新增
│       │   └── folder-list.tsx             # 新增
│       └── hooks/
│           ├── use-folder-create.ts        # 新增
│           ├── use-folder-rename.ts        # 新增
│           └── use-folder-delete.ts        # 新增
├── routes/
│   └── vault.tsx                           # 扩展: 集成文件夹 UI
└── bindings.ts                             # 自动生成: TypeScript 类型

# 测试
src-tauri/src/
├── domain/vault/folder.rs                  # 包含单元测试
└── application/use_cases/*.rs              # 包含单元测试

tests/
└── integration/
    └── folder_operations_test.rs           # 新增: 集成测试
```

**Structure Decision**: 采用 Tauri 标准架构,后端遵循 Clean Architecture 四层模式(domain/application/infrastructure/interfaces),前端采用 feature-based 组织。文件夹功能作为 vault 领域的扩展,复用现有架构模式,无需引入新的抽象层。

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

无违规项。所有设计决策符合 Constitution 原则。
