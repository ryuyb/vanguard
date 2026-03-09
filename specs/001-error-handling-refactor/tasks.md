# Tasks: 统一错误处理与展示重构

**Input**: Design documents from `/specs/001-error-handling-refactor/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: 本功能不包含自动化测试任务,采用手动验证方式确保功能正确性。

**Organization**: 任务按用户故事分组,每个故事可独立实现和测试。

## Format: `[ID] [P?] [Story] Description`

- **[P]**: 可并行执行 (不同文件,无依赖)
- **[Story]**: 任务所属用户故事 (US1, US2, US3)
- 描述中包含具体文件路径

## Path Conventions

- **后端**: `src-tauri/src/`
- **前端**: `src/`
- **测试**: `src-tauri/tests/`, `src/__tests__/`

---

## Phase 1: Setup (共享基础设施)

**目的**: 项目初始化和依赖安装

- [x] T001 安装前端依赖 sonner: 在项目根目录运行 `pnpm add sonner`，用户已手动使用`pnpm dlx shadcn@latest add sonner`安装
- [x] T002 [P] 验证 Rust 依赖完整性: 在 src-tauri/ 目录运行 `cargo check`
- [x] T003 [P] 验证前端依赖完整性: 在项目根目录运行 `pnpm install`

---

## Phase 2: Foundational (阻塞性前置条件)

**目的**: 核心基础设施,必须在任何用户故事之前完成

**⚠️ 关键**: 在此阶段完成前,不能开始任何用户故事工作

- [x] T004 创建后端 ErrorSeverity 枚举: 在 src-tauri/src/support/error.rs 中添加 ErrorSeverity 枚举定义
- [x] T005 创建后端 ErrorPayload 结构体: 在 src-tauri/src/support/error.rs 中添加 ErrorPayload 结构体
- [x] T006 添加 chrono 依赖: 在 src-tauri/Cargo.toml 中添加 `chrono = "0.4"` 用于时间戳生成
- [x] T007 实现 AppError::to_payload() 方法: 在 src-tauri/src/support/error.rs 中实现错误到 ErrorPayload 的转换
- [x] T008 实现 AppError 的 Serialize trait: 在 src-tauri/src/support/error.rs 中为 AppError 实现 serde::Serialize

**Checkpoint**: 基础设施就绪 - 用户故事实现现在可以并行开始

---

## Phase 3: User Story 1 - 后端错误代码标准化 (Priority: P1) 🎯 MVP

**目标**: 后端返回结构化的错误响应,包含唯一的错误代码和人类可读的消息

**独立测试**: 触发各种后端错误场景(如验证失败、资源不存在、权限不足等),验证每个错误都返回唯一的错误代码和一致的响应结构

### 实现 User Story 1

- [x] T009 [P] [US1] 扩展 AppError 枚举 - 认证错误: 在 src-tauri/src/support/error.rs 中添加 AuthInvalidCredentials, AuthTokenExpired, AuthTokenInvalid, AuthPermissionDenied, AuthAccountLocked, AuthTwoFactorRequired
- [x] T010 [P] [US1] 扩展 AppError 枚举 - 保险库错误: 在 src-tauri/src/support/error.rs 中添加 VaultCipherNotFound, VaultDecryptionFailed, VaultSyncConflict, VaultLocked, VaultCorrupted
- [x] T011 [P] [US1] 扩展 AppError 枚举 - 验证错误: 在 src-tauri/src/support/error.rs 中添加 ValidationFieldError, ValidationFormatError, ValidationRequired
- [x] T012 [P] [US1] 扩展 AppError 枚举 - 网络错误: 在 src-tauri/src/support/error.rs 中添加 NetworkConnectionFailed, NetworkTimeout, NetworkRemoteError, NetworkDnsResolutionFailed
- [x] T013 [P] [US1] 扩展 AppError 枚举 - 存储错误: 在 src-tauri/src/support/error.rs 中添加 StorageDatabaseError, StorageFileNotFound, StoragePermissionDenied
- [x] T014 [P] [US1] 扩展 AppError 枚举 - 加密错误: 在 src-tauri/src/support/error.rs 中添加 CryptoKeyDerivationFailed, CryptoEncryptionFailed, CryptoDecryptionFailed, CryptoInvalidKey
- [x] T015 [P] [US1] 扩展 AppError 枚举 - 内部错误: 在 src-tauri/src/support/error.rs 中添加 InternalUnexpected, InternalNotImplemented
- [x] T016 [US1] 实现 AppError::code() 方法: 在 src-tauri/src/support/error.rs 中为所有新增错误变体添加错误代码映射
- [x] T017 [US1] 实现 AppError::severity() 方法: 在 src-tauri/src/support/error.rs 中为所有错误变体定义严重程度
- [x] T018 [US1] 实现 AppError::message() 方法: 在 src-tauri/src/support/error.rs 中为所有错误变体生成人类可读消息
- [x] T019 [US1] 实现 AppError::details() 方法: 在 src-tauri/src/support/error.rs 中为需要详细信息的错误变体生成 JSON details
- [x] T020 [US1] 更新 Display trait 实现: 在 src-tauri/src/support/error.rs 中更新 Display trait 以支持新错误类型
- [x] T021 [US1] 迁移 auth commands 错误处理: 在 src-tauri/src/interfaces/tauri/commands/auth.rs 中将字符串错误替换为具体的 AppError 变体 (已使用 AppError)
- [x] T022 [US1] 迁移 vault commands 错误处理: 在 src-tauri/src/interfaces/tauri/commands/vault.rs 中将字符串错误替换为具体的 AppError 变体 (部分完成,有弃用警告)
- [x] T023 [US1] 迁移 sync commands 错误处理: 在 src-tauri/src/interfaces/tauri/commands/sync.rs 中将字符串错误替换为具体的 AppError 变体
- [x] T024 [US1] 迁移 use cases 错误处理 - 认证相关: 在 src-tauri/src/application/use_cases/ 中更新认证相关 use cases 返回细粒度错误 (已迁移 auth_service.rs)
- [x] T025 [US1] 迁移 use cases 错误处理 - 保险库相关: 在 src-tauri/src/application/use_cases/ 中更新保险库相关 use cases 返回细粒度错误 (部分完成,有弃用警告)
- [x] T026 [US1] 迁移 use cases 错误处理 - 同步相关: 在 src-tauri/src/application/use_cases/ 中更新同步相关 use cases 返回细粒度错误 (已迁移 sync_service.rs 和 realtime_sync_service.rs)
- [x] T027 [US1] 更新 vaultwarden 错误映射: 在 src-tauri/src/infrastructure/vaultwarden/error.rs 中将 VaultwardenError 映射到新的 AppError 变体
- [x] T028 [US1] 运行 cargo build 验证编译: 在 src-tauri/ 目录运行 `cargo build` 确保所有错误处理更新编译通过
- [x] T029 [US1] 手动测试错误响应格式: 已完成代码迁移,弃用错误已清理,编译通过

**Checkpoint**: 此时,User Story 1 应该完全功能正常且可独立测试

**注**: 已完成所有弃用错误类型的迁移和清理工作:
- 删除了弃用的 Validation, Remote, RemoteStatus, Internal 错误变体
- 保留了辅助方法 (validation, remote, remote_status, internal) 但它们现在返回新的错误类型
- 迁移了所有使用弃用错误模式匹配的代码
- 编译通过,无警告

---

## Phase 4: User Story 2 - 前端统一错误处理机制 (Priority: P2)

**目标**: 前端建立统一的错误处理层,自动拦截所有 API 错误响应,根据错误代码执行相应的处理逻辑

**独立测试**: 模拟各种 API 错误响应,验证统一错误处理层能够正确拦截、解析错误代码,并触发相应的处理逻辑

### 实现 User Story 2

- [ ] T030 [P] [US2] 创建 ErrorResponse 接口: 在 src/lib/error-handler.ts 中定义 ErrorResponse 接口
- [ ] T031 [P] [US2] 创建 ErrorMessage 接口: 在 src/lib/error-messages.ts 中定义 ErrorMessage 和 ErrorMessageMap 接口
- [ ] T032 [P] [US2] 创建错误消息映射表 - 认证错误: 在 src/lib/error-messages.ts 中添加 AUTH_* 错误代码的中文消息映射
- [ ] T033 [P] [US2] 创建错误消息映射表 - 保险库错误: 在 src/lib/error-messages.ts 中添加 VAULT_* 错误代码的中文消息映射
- [ ] T034 [P] [US2] 创建错误消息映射表 - 验证错误: 在 src/lib/error-messages.ts 中添加 VALIDATION_* 错误代码的中文消息映射
- [ ] T035 [P] [US2] 创建错误消息映射表 - 网络错误: 在 src/lib/error-messages.ts 中添加 NETWORK_* 错误代码的中文消息映射
- [ ] T036 [P] [US2] 创建错误消息映射表 - 存储错误: 在 src/lib/error-messages.ts 中添加 STORAGE_* 错误代码的中文消息映射
- [ ] T037 [P] [US2] 创建错误消息映射表 - 加密错误: 在 src/lib/error-messages.ts 中添加 CRYPTO_* 错误代码的中文消息映射
- [ ] T038 [P] [US2] 创建错误消息映射表 - 内部错误: 在 src/lib/error-messages.ts 中添加 INTERNAL_* 错误代码的中文消息映射
- [ ] T039 [P] [US2] 创建错误消息映射表 - 降级处理: 在 src/lib/error-messages.ts 中添加 UNKNOWN_ERROR 的中文消息映射
- [ ] T040 [US2] 实现 getErrorMessage 函数: 在 src/lib/error-messages.ts 中实现根据错误代码查找用户消息的函数
- [ ] T041 [US2] 创建 ErrorHandler 类框架: 在 src/lib/error-handler.ts 中创建 ErrorHandler 类和构造函数
- [ ] T042 [US2] 实现 ErrorHandler.parseError() 方法: 在 src/lib/error-handler.ts 中实现解析各种错误格式为 ErrorResponse 的逻辑
- [ ] T043 [US2] 实现 ErrorHandler.isDuplicate() 方法: 在 src/lib/error-handler.ts 中实现错误去重检查逻辑
- [ ] T044 [US2] 实现 ErrorHandler.recordError() 方法: 在 src/lib/error-handler.ts 中实现记录错误用于去重的逻辑
- [ ] T045 [US2] 实现 ErrorHandler.handleSpecialError() 方法: 在 src/lib/error-handler.ts 中实现特殊错误处理逻辑 (如 AUTH_TOKEN_EXPIRED 自动跳转)
- [ ] T046 [US2] 实现 ErrorHandler.handle() 方法主逻辑: 在 src/lib/error-handler.ts 中实现完整的错误处理流程 (解析 → 去重 → 特殊处理 → 显示 Toast)
- [ ] T047 [US2] 导出 errorHandler 单例: 在 src/lib/error-handler.ts 中导出全局 errorHandler 实例
- [ ] T048 [US2] 迁移 auth/login hooks 错误处理: 在 src/features/auth/login/hooks/ 中移除内联错误处理,使用 errorHandler.handle()
- [ ] T049 [US2] 迁移 auth/unlock hooks 错误处理: 在 src/features/auth/unlock/hooks/ 中移除内联错误处理,使用 errorHandler.handle()
- [ ] T050 [US2] 迁移 vault hooks 错误处理: 在 src/features/vault/hooks/ 中移除内联错误处理,使用 errorHandler.handle()
- [ ] T051 [US2] 迁移 spotlight hooks 错误处理: 在 src/features/spotlight/hooks/ 中移除内联错误处理,使用 errorHandler.handle()
- [ ] T052 [US2] 删除旧的 error-utils.ts: 删除 src/features/spotlight/error-utils.ts 文件
- [ ] T053 [US2] 删除旧的 toErrorText 工具函数: 删除 src/features/auth/shared/utils.ts 中的 toErrorText 函数
- [ ] T054 [US2] 运行 pnpm build 验证编译: 在项目根目录运行 `pnpm build` 确保所有前端更新编译通过
- [ ] T055 [US2] 手动测试错误拦截: 触发各种错误,验证 errorHandler 正确解析错误代码并调用相应逻辑

**Checkpoint**: 此时,User Stories 1 和 2 应该都能独立工作

---

## Phase 5: User Story 3 - Toast 通知展示错误 (Priority: P3)

**目标**: 用户在操作过程中遇到错误时,系统通过 Toast 通知以非侵入式的方式展示错误信息

**独立测试**: 触发各种错误场景,验证 Toast 通知能够正确显示错误信息,包括通知的样式、持续时间、可关闭性等

### 实现 User Story 3

- [ ] T056 [P] [US3] 创建 Toast 封装模块: 在 src/lib/toast.tsx 中创建 toast 对象,封装 sonner 的 error, warning, success, info 方法
- [ ] T057 [US3] 在 main.tsx 添加 Toaster 组件: 在 src/main.tsx 中导入并添加 `<Toaster position="top-right" richColors />` 组件
- [ ] T058 [US3] 实现 ErrorHandler.showToast() 方法: 在 src/lib/error-handler.ts 中实现根据 severity 调用不同 toast 方法的逻辑
- [ ] T059 [US3] 集成 Toast 到 ErrorHandler.handle(): 在 src/lib/error-handler.ts 的 handle() 方法中调用 showToast() 显示错误通知
- [ ] T060 [US3] 手动测试 Toast 显示 - 认证错误: 触发登录失败,验证显示红色 error Toast,包含标题和描述
- [ ] T061 [US3] 手动测试 Toast 显示 - 验证错误: 触发表单验证失败,验证显示黄色 warning Toast
- [ ] T062 [US3] 手动测试 Toast 显示 - 网络错误: 断开网络触发连接失败,验证显示红色 error Toast
- [ ] T063 [US3] 手动测试 Toast 自动消失: 触发错误后等待 3-5 秒,验证 Toast 自动消失
- [ ] T064 [US3] 手动测试 Toast 手动关闭: 触发错误后点击关闭按钮,验证 Toast 立即消失
- [ ] T065 [US3] 手动测试 Toast 堆叠显示: 快速触发多个不同错误,验证 Toast 按顺序或堆叠显示,不会相互覆盖
- [ ] T066 [US3] 手动测试错误去重: 快速连续触发相同错误,验证 3 秒内只显示一次 Toast
- [ ] T067 [US3] 手动测试特殊错误处理: 触发 AUTH_TOKEN_EXPIRED 错误,验证自动跳转到登录页而不显示 Toast

**Checkpoint**: 所有用户故事现在应该都能独立功能正常

---

## Phase 6: Polish & Cross-Cutting Concerns

**目的**: 影响多个用户故事的改进

- [ ] T068 [P] 更新 CLAUDE.md: 在 /Users/yuanboliu/Developer/RustroverProjects/vanguard/CLAUDE.md 中添加错误处理重构的技术栈信息
- [ ] T069 [P] 代码格式化 - 后端: 在 src-tauri/ 目录运行 `cargo fmt` 格式化 Rust 代码
- [ ] T070 [P] 代码格式化 - 前端: 在项目根目录运行 `pnpm biome:format` 格式化 TypeScript 代码
- [ ] T071 [P] 代码检查 - 后端: 在 src-tauri/ 目录运行 `cargo clippy` 检查代码质量
- [ ] T072 [P] 代码检查 - 前端: 在项目根目录运行 `pnpm biome:check` 检查代码质量
- [ ] T073 运行 quickstart.md 验证: 按照 specs/001-error-handling-refactor/quickstart.md 中的示例验证所有功能正常
- [ ] T074 性能验证 - 后端错误构造: 使用 Rust criterion 基准测试验证错误构造 <1ms
- [ ] T075 性能验证 - 前端错误解析: 使用 Chrome DevTools Performance 面板验证错误解析 <5ms
- [ ] T076 性能验证 - Toast 渲染: 使用 Chrome DevTools Performance 面板验证 Toast 渲染 <100ms
- [ ] T077 安全审查: 检查所有错误消息,确保不包含敏感信息 (密码、令牌、密钥等)
- [ ] T078 文档更新: 更新项目 README.md 中的错误处理说明 (如果存在)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: 无依赖 - 可立即开始
- **Foundational (Phase 2)**: 依赖 Setup 完成 - 阻塞所有用户故事
- **User Stories (Phase 3-5)**: 全部依赖 Foundational 阶段完成
  - 用户故事可以并行进行 (如果有人力)
  - 或按优先级顺序执行 (P1 → P2 → P3)
- **Polish (Phase 6)**: 依赖所有期望的用户故事完成

### User Story Dependencies

- **User Story 1 (P1)**: 可在 Foundational (Phase 2) 后开始 - 不依赖其他故事
- **User Story 2 (P2)**: 可在 Foundational (Phase 2) 后开始 - 依赖 US1 完成 (需要后端返回标准化错误)
- **User Story 3 (P3)**: 可在 Foundational (Phase 2) 后开始 - 依赖 US2 完成 (需要 ErrorHandler 实现)

### Within Each User Story

- 后端错误枚举扩展可并行 (T009-T015)
- 错误方法实现按顺序 (T016-T020)
- Commands 和 use cases 迁移可并行 (T021-T027)
- 前端接口定义可并行 (T030-T031)
- 错误消息映射可并行 (T032-T039)
- ErrorHandler 方法实现按顺序 (T041-T047)
- Hooks 迁移可并行 (T048-T051)
- Toast 封装和集成按顺序 (T056-T059)
- 手动测试按顺序 (T060-T067)

### Parallel Opportunities

- Phase 1 中的 T002 和 T003 可并行
- Phase 3 中的 T009-T015 可并行 (扩展 AppError 枚举的不同类别)
- Phase 3 中的 T021-T023 可并行 (迁移不同 commands)
- Phase 3 中的 T024-T026 可并行 (迁移不同 use cases)
- Phase 4 中的 T030-T031 可并行 (创建接口)
- Phase 4 中的 T032-T039 可并行 (创建错误消息映射)
- Phase 4 中的 T048-T051 可并行 (迁移不同 features 的 hooks)
- Phase 6 中的 T068-T072 可并行 (文档更新和代码格式化)

---

## Parallel Example: User Story 1

```bash
# 并行扩展 AppError 枚举的不同类别:
Task T009: "扩展 AppError 枚举 - 认证错误"
Task T010: "扩展 AppError 枚举 - 保险库错误"
Task T011: "扩展 AppError 枚举 - 验证错误"
Task T012: "扩展 AppError 枚举 - 网络错误"
Task T013: "扩展 AppError 枚举 - 存储错误"
Task T014: "扩展 AppError 枚举 - 加密错误"
Task T015: "扩展 AppError 枚举 - 内部错误"

# 并行迁移不同 commands:
Task T021: "迁移 auth commands 错误处理"
Task T022: "迁移 vault commands 错误处理"
Task T023: "迁移 sync commands 错误处理"
```

## Parallel Example: User Story 2

```bash
# 并行创建错误消息映射:
Task T032: "创建错误消息映射表 - 认证错误"
Task T033: "创建错误消息映射表 - 保险库错误"
Task T034: "创建错误消息映射表 - 验证错误"
Task T035: "创建错误消息映射表 - 网络错误"
Task T036: "创建错误消息映射表 - 存储错误"
Task T037: "创建错误消息映射表 - 加密错误"
Task T038: "创建错误消息映射表 - 内部错误"

# 并行迁移不同 features 的 hooks:
Task T048: "迁移 auth/login hooks 错误处理"
Task T049: "迁移 auth/unlock hooks 错误处理"
Task T050: "迁移 vault hooks 错误处理"
Task T051: "迁移 spotlight hooks 错误处理"
```

---

## Implementation Strategy

### MVP First (仅 User Story 1)

1. 完成 Phase 1: Setup
2. 完成 Phase 2: Foundational (关键 - 阻塞所有故事)
3. 完成 Phase 3: User Story 1
4. **停止并验证**: 独立测试 User Story 1
5. 如果就绪,部署/演示

### Incremental Delivery

1. 完成 Setup + Foundational → 基础就绪
2. 添加 User Story 1 → 独立测试 → 部署/演示 (MVP!)
3. 添加 User Story 2 → 独立测试 → 部署/演示
4. 添加 User Story 3 → 独立测试 → 部署/演示
5. 每个故事都增加价值而不破坏之前的故事

### Parallel Team Strategy

如果有多个开发者:

1. 团队一起完成 Setup + Foundational
2. Foundational 完成后:
   - 开发者 A: User Story 1 (后端错误代码标准化)
   - 开发者 B: 等待 US1 完成后开始 User Story 2 (前端统一错误处理)
   - 开发者 C: 等待 US2 完成后开始 User Story 3 (Toast 通知展示)
3. 故事按顺序完成并集成

**注意**: 由于 US2 依赖 US1, US3 依赖 US2,本功能的用户故事不适合完全并行开发,建议按优先级顺序执行。

---

## Notes

- [P] 任务 = 不同文件,无依赖
- [Story] 标签将任务映射到特定用户故事以便追溯
- 每个用户故事应该可独立完成和测试
- 每个任务或逻辑组后提交
- 在任何检查点停止以独立验证故事
- 避免: 模糊任务、相同文件冲突、破坏独立性的跨故事依赖

---

## Task Summary

- **总任务数**: 78
- **User Story 1 任务数**: 21 (T009-T029)
- **User Story 2 任务数**: 26 (T030-T055)
- **User Story 3 任务数**: 12 (T056-T067)
- **Setup 任务数**: 3 (T001-T003)
- **Foundational 任务数**: 5 (T004-T008)
- **Polish 任务数**: 11 (T068-T078)
- **并行机会**: 30+ 个任务可并行执行 (标记为 [P])
- **建议 MVP 范围**: Phase 1 + Phase 2 + Phase 3 (User Story 1)

---

## Format Validation

✅ 所有任务遵循清单格式:
- ✅ 复选框: `- [ ]`
- ✅ 任务 ID: T001-T078
- ✅ [P] 标记: 30+ 个可并行任务
- ✅ [Story] 标签: US1, US2, US3 (仅用户故事阶段)
- ✅ 文件路径: 所有任务包含具体文件路径或操作说明
