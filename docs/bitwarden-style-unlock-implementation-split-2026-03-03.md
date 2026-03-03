# Bitwarden 风格解锁重构实施拆分（无兼容迁移）

日期：2026-03-03
关联方案：`docs/bitwarden-style-unlock-redesign-no-migration-2026-03-03.md`

## 说明
本拆分文档将重构工作拆为可独立评审、可逐步合入的 PR 序列。
约束如下：
1. 不做兼容迁移，不保留旧解锁数据结构回退。
2. 每个 PR 必须可编译、可测试、可回滚。
3. 严格遵守依赖方向：`interfaces -> application -> domain`。

## PR 序列总览
| PR | 目标 | 主要层 | 依赖 |
| --- | --- | --- | --- |
| PR-1 | 建立统一解锁领域模型与应用主用例骨架 | domain/application | 无 |
| PR-2 | 硬切换 Master Password 解锁为单路径 | application/infrastructure/interfaces | PR-1 |
| PR-3 | 收敛 Biometric 到统一解锁主链路 | application/interfaces/infrastructure | PR-1 |
| PR-4 | 引入 PIN domain + port + store（先 backend） | domain/application/infrastructure | PR-1 |
| PR-5 | 暴露 PIN Tauri commands + DTO + bindings | interfaces/frontend bindings | PR-4 |
| PR-6 | 前端 Unlock/Settings 接入 PIN 与统一 unlock 命令 | frontend | PR-5 |
| PR-7 | 清理旧密码解锁用例与死代码，补全测试与文档 | 全层清理 | PR-2/3/6 |

## PR 详细拆分
## PR-1 统一解锁骨架
### 范围
1. 新增 `UnlockMethod`、`UnlockVaultCommand`、`UnlockVaultResult`。
2. 新增 `UnlockVaultUseCase`（只保留骨架与依赖注入，不改现有 command 行为）。
3. 新增解锁相关 port 抽象：
   1. `MasterPasswordUnlockDataPort`
   2. `PinUnlockPort`（空实现可后续补齐）
4. 增加最小单元测试，验证 method 分发与错误路径。

### 文件建议
1. `src-tauri/src/domain/...`（新增 unlock 相关类型）
2. `src-tauri/src/application/use_cases/unlock_vault_use_case.rs`（新增）
3. `src-tauri/src/application/ports/master_password_unlock_data_port.rs`（新增）
4. `src-tauri/src/application/ports/pin_unlock_port.rs`（新增）

### 验收标准
1. `cargo check` 通过。
2. `UnlockVaultUseCase` 单元测试通过。
3. 现有 `vault_unlock_with_password` 行为不变。

### 建议提交信息
`refactor(vault): add unified unlock use case skeleton`

## PR-2 Master Password 硬切换
### 范围
1. 删除多候选 KDF/多候选 key 尝试逻辑。
2. 引入 canonical `MasterPasswordUnlockData` 的读取与单路径解封装。
3. `vault_unlock_with_password` 改为调用统一 `UnlockVaultUseCase::MasterPassword`。
4. 若 canonical 数据不存在，直接失败（不回退旧逻辑）。

### 文件建议
1. `src-tauri/src/application/use_cases/unlock_vault_with_password_use_case.rs`（删除或并入）
2. `src-tauri/src/application/use_cases/unlock_vault_use_case.rs`（落地主流程）
3. `src-tauri/src/infrastructure/persistence/sqlite_vault_repository.rs`（canonical 数据读写）
4. `src-tauri/src/interfaces/tauri/commands/vault.rs`（命令改路由）

### 验收标准
1. 密码解锁路径不再含候选循环。
2. 解锁路径不触发网络请求。
3. 错误信息保留 redaction 基线。

### 建议提交信息
`refactor(vault): switch master password unlock to canonical single-path`

## PR-3 Biometric 收敛
### 范围
1. `vault_unlock_with_biometric` 改为统一走 `UnlockVaultUseCase::Biometric`。
2. biometric 仅负责获取 user key，不直接承担完整业务编排。
3. 启用/禁用 biometric 的 use case 与日志语义对齐。

### 文件建议
1. `src-tauri/src/application/use_cases/vault_biometric_use_case.rs`
2. `src-tauri/src/application/use_cases/unlock_vault_use_case.rs`
3. `src-tauri/src/interfaces/tauri/commands/vault.rs`

### 验收标准
1. biometric 解锁成功后 runtime user key 可读。
2. biometric 启用/禁用行为无回归。
3. `vault_get_biometric_status` 返回语义不变。

### 建议提交信息
`refactor(vault): route biometric unlock through unified unlock pipeline`

## PR-4 PIN backend 基础能力
### 范围
1. 新增 `PinLockType` 与 PIN 相关 DTO（应用层）。
2. 新增 `EnablePinUnlockUseCase`、`DisablePinUnlockUseCase`、`GetPinUnlockStatusUseCase`。
3. 新增 `pin_store`：
   1. persistent：安全存储（macOS keychain）
   2. ephemeral：`AppState` 内存态
4. 新增 `UnlockVaultUseCase::Pin` 执行分支。

### 文件建议
1. `src-tauri/src/application/use_cases/vault_pin_use_case.rs`（新增）
2. `src-tauri/src/infrastructure/security/pin_store.rs`（新增）
3. `src-tauri/src/bootstrap/app_state.rs`（ephemeral pin 内存存储）
4. `src-tauri/src/application/ports/pin_unlock_port.rs`（实现）

### 验收标准
1. backend 单测覆盖：
   1. 启用 PIN 需 vault 已解锁
   2. Ephemeral 重启失效语义（单测可模拟清空）
   3. Persistent 可读取
2. PIN 明文不落盘，不打印日志。

### 建议提交信息
`feat(vault): add pin unlock backend with persistent and ephemeral modes`

## PR-5 PIN Tauri 命令与 bindings
### 范围
1. 新增命令：
   1. `vault_get_pin_status`
   2. `vault_enable_pin_unlock`
   3. `vault_disable_pin_unlock`
   4. `vault_unlock_with_pin`
2. 新增接口 DTO 与 specta 导出。
3. 更新 `src/bindings.ts`。

### 文件建议
1. `src-tauri/src/interfaces/tauri/commands/vault.rs`
2. `src-tauri/src/interfaces/tauri/dto/vault.rs`
3. `src-tauri/src/lib.rs`
4. `src/bindings.ts`（生成）

### 验收标准
1. 前端可调用新命令（类型可用）。
2. 命令层仅做映射，不含业务规则。
3. 旧命令未破坏。

### 建议提交信息
`feat(desktop): expose pin unlock tauri commands and bindings`

## PR-6 前端接入与交互改造
### 范围
1. Unlock 页面接入 PIN 解锁入口。
2. Vault 设置页面新增 PIN 启用/关闭与 lock type 切换。
3. 解锁按钮统一走后端 unlock 命令族，避免页面侧分散业务逻辑。

### 文件建议
1. `src/routes/unlock.tsx`
2. `src/routes/vault.tsx`
3. `src/lib/route-session.ts`（如需能力判断）

### 验收标准
1. 页面状态切换正确：
   1. `needsLogin` 不显示 PIN 解锁。
   2. `locked` 且已启用 PIN 时显示 PIN 解锁。
2. 错误提示不泄露敏感信息。
3. 不引入额外启动串行阻塞。

### 建议提交信息
`feat(vault): add pin unlock ui and settings integration`

## PR-7 清理与收口
### 范围
1. 删除旧密码解锁冗余实现与不可达分支。
2. 统一日志与错误码文案。
3. 更新开发文档与 QA 用例。
4. 补全端到端测试矩阵。

### 文件建议
1. `src-tauri/src/application/use_cases/unlock_vault_with_password_use_case.rs`（若已并入则删除）
2. `docs/*.md`（方案文档与测试说明）

### 验收标准
1. 无死代码/重复逻辑。
2. `cargo check`、`cargo test`、`cargo clippy --all-targets --all-features -- -D warnings` 全通过。
3. 手工回归通过（见下方测试矩阵）。

### 建议提交信息
`chore(vault): remove legacy unlock paths and finalize unlock redesign`

## 测试矩阵（回归最小集）
1. Master password 解锁成功/失败。
2. Biometric 解锁成功/取消/失败。
3. PIN persistent 启用 -> 锁定 -> 解锁成功。
4. PIN ephemeral 启用 -> 锁定 -> 解锁成功。
5. PIN ephemeral 在进程重启后不可用。
6. 登出后 PIN/biometric 状态清理符合预期。
7. `needsLogin/locked/unlocked` 路由跳转逻辑正确。
8. 解锁失败后 `vault_get_view_data` 仍返回 locked 错误。

## 开工顺序建议
1. 先做 PR-1 + PR-2（先解决性能核心问题）。
2. 再做 PR-3（收敛现有 biometric，降低复杂度）。
3. 最后做 PR-4/5/6（PIN 全链路）与 PR-7 收口。

## 风险与控制
1. 风险：硬切换后旧本地数据不可用。
控制：在 release note 明确“需重新登录并同步一次后使用新解锁机制”。

2. 风险：PIN keychain 行为跨平台不一致。
控制：首版明确 macOS fully supported，其它平台返回 `supported=false`。

3. 风险：命令层与用例层边界被打破。
控制：PR review 时按 DDD 约束逐条检查。

