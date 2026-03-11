# Tasks: Vault 文件夹管理功能

**Input**: Design documents from `/specs/001-folder-management/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/tauri-commands.md

**Tests**: 根据 research.md,前端测试使用 Vitest + React Testing Library。后端测试使用 cargo test。每个 user story 包含必要的测试任务以验证功能正确性。

**Organization**: 任务按 user story 组织,以支持独立实现和测试。

## Format: `[ID] [P?] [Story] Description`

- **[P]**: 可并行执行(不同文件,无依赖)
- **[Story]**: 任务所属的 user story (US1, US2, US3)
- 描述中包含精确的文件路径

## Path Conventions

本项目采用 Tauri 桌面应用架构:
- **后端 (Rust)**: `src-tauri/src/`
- **前端 (React)**: `src/`
- **测试**: `tests/` (集成测试), 单元测试在各模块内

---

## Phase 1: Setup (共享基础设施)

**Purpose**: 项目初始化和数据库迁移

- [ ] T001 创建数据库迁移脚本,新增 folders 表和扩展 ciphers 表 (src-tauri/src/infrastructure/persistence/migrations/)
- [ ] T002 [P] 在 domain/vault/mod.rs 中导出 folder 模块
- [ ] T003 [P] 在 application/dto/vault.rs 中添加 FolderDto 和请求 DTO 定义

---

## Phase 2: Foundational (阻塞性前置条件)

**Purpose**: 所有 user story 依赖的核心基础设施

**⚠️ CRITICAL**: 此阶段完成前,任何 user story 都无法开始

- [ ] T004 创建 Folder 实体和 FolderName 值对象 (src-tauri/src/domain/vault/folder.rs)
- [ ] T005 在 Folder 实体中实现单元测试,覆盖所有验证规则 (FR-002, FR-011, FR-012)
- [ ] T006 扩展 VaultRepositoryPort trait,添加文件夹操作接口 (src-tauri/src/application/ports/vault_repository_port.rs)
- [ ] T007 在 SQLite repository 中实现文件夹 CRUD 方法 (src-tauri/src/infrastructure/persistence/sqlite_vault_repository.rs)
- [ ] T008 为 SQLite repository 的文件夹操作添加单元测试

**Checkpoint**: 基础设施就绪 - user story 实现现在可以并行开始

---

## Phase 3: User Story 1 - 创建新文件夹组织密码 (Priority: P1) 🎯 MVP

**Goal**: 用户可以创建新文件夹来分类管理不同类型的密码,使密码库更有条理

**Independent Test**: 用户可以在 vault 页面点击"新建文件夹"按钮,输入文件夹名称后保存,新文件夹立即出现在文件夹列表中,可以用于组织 cipher

### 后端实现 (User Story 1)

- [ ] T009 [P] [US1] 创建 CreateFolderUseCase (src-tauri/src/application/use_cases/create_folder_use_case.rs)
- [ ] T010 [P] [US1] 为 CreateFolderUseCase 添加单元测试,覆盖验证逻辑和错误场景
- [ ] T011 [US1] 创建 get_folders Tauri 命令 (src-tauri/src/interfaces/tauri/commands/folder.rs)
- [ ] T012 [US1] 创建 create_folder Tauri 命令 (src-tauri/src/interfaces/tauri/commands/folder.rs)
- [ ] T013 [US1] 在 lib.rs 中注册 get_folders 和 create_folder 命令
- [ ] T014 [US1] 运行 specta 生成 TypeScript 类型绑定 (src/bindings.ts)

### 前端实现 (User Story 1)

- [ ] T015 [P] [US1] 创建 useFolders hook (src/features/vault/hooks/use-folders.ts)
- [ ] T016 [P] [US1] 创建 useCreateFolder hook (src/features/vault/hooks/use-folder-create.ts)
- [ ] T017 [US1] 创建 FolderList 组件 (src/features/vault/components/folder-list.tsx)
- [ ] T018 [US1] 创建 FolderCreateDialog 组件 (src/features/vault/components/folder-create-dialog.tsx)
- [ ] T019 [US1] 在 vault 路由中集成文件夹列表和创建对话框 (src/routes/vault.tsx)

### 测试验证 (User Story 1)

- [ ] T020 [US1] 创建集成测试验证创建文件夹完整流程 (tests/integration/folder_operations_test.rs)
- [ ] T021 [US1] 手动测试 Acceptance Scenarios 1-4 (spec.md)
- [ ] T022 [US1] 验证性能目标: 创建文件夹操作 <100ms (SC-001)

**Checkpoint**: User Story 1 应完全功能化且可独立测试

---

## Phase 4: User Story 2 - 重命名文件夹 (Priority: P2)

**Goal**: 用户可以修改文件夹名称以更好地反映其内容或适应组织结构变化

**Independent Test**: 用户可以选择现有文件夹,点击"重命名"选项,输入新名称后保存,文件夹名称立即更新,其中的 cipher 保持不变

### 后端实现 (User Story 2)

- [ ] T023 [P] [US2] 创建 RenameFolderUseCase (src-tauri/src/application/use_cases/rename_folder_use_case.rs)
- [ ] T024 [P] [US2] 为 RenameFolderUseCase 添加单元测试
- [ ] T025 [US2] 创建 rename_folder Tauri 命令 (src-tauri/src/interfaces/tauri/commands/folder.rs)
- [ ] T026 [US2] 在 lib.rs 中注册 rename_folder 命令
- [ ] T027 [US2] 重新生成 TypeScript 类型绑定

### 前端实现 (User Story 2)

- [ ] T028 [P] [US2] 创建 useRenameFolder hook (src/features/vault/hooks/use-folder-rename.ts)
- [ ] T029 [US2] 创建 FolderRenameDialog 组件 (src/features/vault/components/folder-rename-dialog.tsx)
- [ ] T030 [US2] 在 FolderList 组件中添加重命名操作入口 (右键菜单或按钮)

### 测试验证 (User Story 2)

- [ ] T031 [US2] 扩展集成测试验证重命名文件夹流程 (tests/integration/folder_operations_test.rs)
- [ ] T032 [US2] 手动测试 Acceptance Scenarios 1-4 (spec.md)
- [ ] T033 [US2] 验证性能目标: 重命名文件夹操作 <100ms (SC-002)

**Checkpoint**: User Stories 1 和 2 应都能独立工作

---

## Phase 5: User Story 3 - 删除不需要的文件夹 (Priority: P3)

**Goal**: 用户可以删除不再使用的文件夹以保持 vault 整洁

**Independent Test**: 用户可以选择文件夹,点击"删除"选项,确认后文件夹从列表中移除。如果文件夹包含 cipher,系统会提示用户处理这些 cipher

### 后端实现 (User Story 3)

- [ ] T034 [P] [US3] 创建 DeleteFolderUseCase (src-tauri/src/application/use_cases/delete_folder_use_case.rs)
- [ ] T035 [P] [US3] 为 DeleteFolderUseCase 添加单元测试,包含级联更新 cipher 的场景
- [ ] T036 [US3] 创建 delete_folder Tauri 命令 (src-tauri/src/interfaces/tauri/commands/folder.rs)
- [ ] T037 [US3] 在 lib.rs 中注册 delete_folder 命令
- [ ] T038 [US3] 重新生成 TypeScript 类型绑定

### 前端实现 (User Story 3)

- [ ] T039 [P] [US3] 创建 useDeleteFolder hook (src/features/vault/hooks/use-folder-delete.ts)
- [ ] T040 [US3] 创建 FolderDeleteDialog 组件,包含警告信息 (src/features/vault/components/folder-delete-dialog.tsx)
- [ ] T041 [US3] 在 FolderList 组件中添加删除操作入口 (右键菜单或按钮)

### 测试验证 (User Story 3)

- [ ] T042 [US3] 扩展集成测试验证删除文件夹流程和 cipher 级联处理 (tests/integration/folder_operations_test.rs)
- [ ] T043 [US3] 手动测试 Acceptance Scenarios 1-4 (spec.md)
- [ ] T044 [US3] 验证性能目标: 删除文件夹操作 <150ms (SC-003)

**Checkpoint**: 所有 user stories 应现在都能独立功能化

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: 影响多个 user stories 的改进

- [ ] T045 [P] 运行 quickstart.md 中的所有验证场景
- [ ] T046 [P] 验证 Edge Cases (spec.md): 100 个文件夹上限、特殊字符处理、长名称处理
- [ ] T047 [P] 运行 cargo clippy 检查 Rust 代码质量
- [ ] T048 [P] 运行 pnpm biome:check 检查前端代码风格
- [ ] T049 验证所有 Success Criteria (SC-001 ~ SC-007)
- [ ] T050 更新 CHANGELOG.md 记录新功能

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: 无依赖 - 可立即开始
- **Foundational (Phase 2)**: 依赖 Setup 完成 - 阻塞所有 user stories
- **User Stories (Phase 3-5)**: 都依赖 Foundational phase 完成
  - User stories 可以并行进行(如有人力)
  - 或按优先级顺序执行 (P1 → P2 → P3)
- **Polish (Phase 6)**: 依赖所有期望的 user stories 完成

### User Story Dependencies

- **User Story 1 (P1)**: Foundational 完成后可开始 - 无其他 story 依赖
- **User Story 2 (P2)**: Foundational 完成后可开始 - 依赖 US1 的 get_folders 命令,但应独立可测
- **User Story 3 (P3)**: Foundational 完成后可开始 - 依赖 US1 的 get_folders 命令,但应独立可测

### Within Each User Story

- 后端实现 → 前端实现 → 测试验证
- Use cases 在 Tauri commands 之前
- Hooks 在 UI 组件之前
- 核心实现在集成之前
- Story 完成后再进入下一优先级

### Parallel Opportunities

- Phase 1: T002 和 T003 可并行
- Phase 2: T004-T005 可与 T006-T008 并行(不同文件)
- Phase 3 (US1): T009-T010 可并行, T015-T016 可并行
- Phase 4 (US2): T023-T024 可并行, T028 独立
- Phase 5 (US3): T034-T035 可并行, T039 独立
- Phase 6: T045-T048 可并行
- 一旦 Foundational 完成,所有 user stories 可由不同团队成员并行开发

---

## Parallel Example: User Story 1

```bash
# 并行启动后端 use case 和测试:
Task T009: "创建 CreateFolderUseCase"
Task T010: "为 CreateFolderUseCase 添加单元测试"

# 并行启动前端 hooks:
Task T015: "创建 useFolders hook"
Task T016: "创建 useCreateFolder hook"
```

---

## Implementation Strategy

### MVP First (仅 User Story 1)

1. 完成 Phase 1: Setup
2. 完成 Phase 2: Foundational (关键 - 阻塞所有 stories)
3. 完成 Phase 3: User Story 1
4. **停止并验证**: 独立测试 User Story 1
5. 如果就绪则部署/演示

### Incremental Delivery

1. 完成 Setup + Foundational → 基础就绪
2. 添加 User Story 1 → 独立测试 → 部署/演示 (MVP!)
3. 添加 User Story 2 → 独立测试 → 部署/演示
4. 添加 User Story 3 → 独立测试 → 部署/演示
5. 每个 story 增加价值而不破坏之前的 stories

### Parallel Team Strategy

多开发者情况:

1. 团队一起完成 Setup + Foundational
2. Foundational 完成后:
   - 开发者 A: User Story 1
   - 开发者 B: User Story 2
   - 开发者 C: User Story 3
3. Stories 独立完成和集成

---

## Notes

- [P] 任务 = 不同文件,无依赖
- [Story] 标签将任务映射到特定 user story 以便追溯
- 每个 user story 应可独立完成和测试
- 在实现前验证测试失败
- 每个任务或逻辑组后提交
- 在任何 checkpoint 停止以独立验证 story
- 避免: 模糊任务、同文件冲突、破坏独立性的跨 story 依赖
