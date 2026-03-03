# Bitwarden 风格解锁重构方案（无兼容迁移）

日期：2026-03-03

## 背景
当前项目解锁能力已具备 `master password` 和 `biometric`，但存在以下问题：
1. `master password` 解锁采用多候选 KDF 与多候选 key 尝试，CPU 成本高、长尾慢。
2. 尚未提供 `PIN` 解锁。
3. 三种解锁方式没有统一的解锁主流程，导致维护成本高、行为不一致。

目标是对齐 Bitwarden 的核心思路：
1. 登录（认证）与解锁（本地密钥恢复）严格分离。
2. 解锁仅恢复 `user key` 到内存，不走网络。
3. 统一解锁流水线，`master password / PIN / biometric` 只是不同 unlock method。

本文明确采用**硬切换**策略：**不做兼容迁移，不保留旧 unlock 结构回退逻辑**。

## 范围与非目标
### 范围
1. Rust backend（application/domain/interfaces/infrastructure）解锁链路重构。
2. Tauri command 与前端 bindings 更新。
3. Unlock 页面交互新增 PIN，并调整 master password 与 biometric 流程。

### 非目标
1. 不修改同步协议与远程 API。
2. 不保留旧版本地 unlock 数据格式兼容。
3. 不在本次引入跨设备 PIN 同步能力（仅设备本地）。

## 总体设计
### 统一解锁用例
新增 `UnlockVaultUseCase`，以统一输入驱动解锁：
1. `MasterPassword { password }`
2. `Pin { pin }`
3. `Biometric`

统一输出：
1. `VaultUserKeyMaterial`（`enc_key`, `mac_key`）

统一收尾动作：
1. `runtime.set_vault_user_key_material(account_id, user_key)`
2. 记录解锁成功审计日志（不含敏感内容）

### 状态语义保持
保持现有三态语义不变：
1. `needsLogin`：没有可用认证上下文。
2. `locked`：有认证上下文，但未装载 user key 到 runtime。
3. `unlocked`：runtime 内存在 user key，可访问 vault 明文视图。

## 分层落点（DDD 约束）
### Domain
新增纯领域类型，不引入平台 API：
1. `UnlockMethod`
2. `PinLockType`（`Disabled | Ephemeral | Persistent`）
3. `MasterPasswordUnlockData`
4. `PinProtectedUserKeyEnvelope`

### Application
新增/重构 use case：
1. `UnlockVaultUseCase`（统一主入口）
2. `EnablePinUnlockUseCase`
3. `DisablePinUnlockUseCase`
4. `GetPinUnlockStatusUseCase`
5. `EnableBiometricUnlockUseCase`（可复用现有逻辑，但收敛到统一结构）
6. `DisableBiometricUnlockUseCase`

新增 ports：
1. `MasterPasswordUnlockDataPort`
2. `PinUnlockPort`
3. `BiometricUnlockPort`（保留，语义收敛）

### Infrastructure
仅实现 ports：
1. `sqlite` 持久化 `master_password_unlock_data`
2. `keychain`/secure storage 持久化 PIN persistent envelope
3. `AppState` 内存态持有 PIN ephemeral envelope
4. `keychain` 持久化 biometric unlock bundle（沿用现有）

### Interfaces（Tauri）
新增/调整 commands（只做参数校验与 DTO 映射）：
1. `vault_unlock`（统一接口，method 区分）
2. `vault_get_unlock_capabilities`
3. `vault_get_pin_status`
4. `vault_enable_pin_unlock`
5. `vault_disable_pin_unlock`
6. `vault_enable_biometric_unlock`
7. `vault_disable_biometric_unlock`

说明：可保留旧 command 名称作为 UI 过渡，但业务逻辑统一落到 `UnlockVaultUseCase`。

## 三种解锁方式详细流程
### 1) Master Password
1. 读取 `MasterPasswordUnlockData`（本地 canonical 数据）。
2. 使用其中 KDF 参数 + salt 进行单次 master key 派生。
3. 解封装 `master_key_wrapped_user_key` 得到 `user key`。
4. 设置 runtime user key 并返回成功。

约束：
1. 严禁多候选尝试与“猜测式”解密。
2. 严禁在解锁热路径进行 `prelogin`/网络请求。

### 2) PIN
#### 启用 PIN
1. 前提：vault 已解锁（runtime 内有 user key）。
2. 用户提交 `pin` 与 `pin_lock_type`。
3. 生成 `pin_protected_user_key_envelope`。
4. 根据 `pin_lock_type` 存储：
   1. `Persistent`：写入安全存储。
   2. `Ephemeral`：仅写入内存（进程重启失效）。

#### PIN 解锁
1. 按锁型读取 envelope。
2. 使用输入 `pin` 解封装得到 `user key`。
3. 设置 runtime user key。

#### 关闭 PIN
1. 清除 persistent envelope（安全存储）。
2. 清除 ephemeral envelope（内存）。

### 3) Biometric
#### 启用 Biometric
1. 前提：vault 已解锁。
2. 将当前 `user key` 封装后写入 keychain（受系统生物认证策略保护）。

#### Biometric 解锁
1. 调用系统生物认证读取 bundle。
2. 解析出 `user key`。
3. 设置 runtime user key。

#### 关闭 Biometric
1. 删除 keychain 中对应 bundle。

## 数据模型（硬切换后唯一格式）
### Master Password Unlock Data（持久化）
```json
{
  "version": 1,
  "accountId": "string",
  "kdf": {
    "type": "pbkdf2|argon2id",
    "iterations": 600000,
    "memory": 64,
    "parallelism": 4
  },
  "salt": "string",
  "masterKeyWrappedUserKey": "cipher_string"
}
```

### PIN Unlock Data（persistent）
```json
{
  "version": 1,
  "accountId": "string",
  "lockType": "persistent",
  "envelope": {
    "algorithm": "xchacha20poly1305",
    "kdf": "argon2id",
    "saltB64": "string",
    "nonceB64": "string",
    "ciphertextB64": "string"
  }
}
```

### PIN Unlock Data（ephemeral）
1. 结构与 persistent 相同。
2. 仅驻留 `AppState` 内存，不落盘。

### Biometric Bundle
沿用现有 `VaultBiometricBundle` 结构即可，作为硬切换后的唯一格式。

## 接口草案
### Rust DTO（应用层）
```rust
pub enum UnlockMethod {
    MasterPassword { password: String },
    Pin { pin: String },
    Biometric,
}

pub struct UnlockVaultCommand {
    pub method: UnlockMethod,
}

pub struct EnablePinUnlockCommand {
    pub pin: String,
    pub lock_type: PinLockType,
}
```

### Tauri DTO（接口层）
```ts
type VaultUnlockMethodDto =
  | { type: "masterPassword"; password: string }
  | { type: "pin"; pin: string }
  | { type: "biometric" };

type VaultUnlockRequestDto = {
  method: VaultUnlockMethodDto;
};
```

## 前端改造要点
1. Unlock 页面新增 PIN 表单与按钮。
2. 优先展示可用的快捷解锁（biometric > pin > master password）。
3. 设置页新增 PIN 开关与锁型选择（Persistent/Ephemeral）。
4. 所有解锁按钮最终调用统一 `vaultUnlock` 命令，仅 method 不同。
5. 异常信息统一走后端 redaction 后输出，不拼接敏感字段。

## 安全与性能要求
### 安全
1. 禁止日志输出 `password/pin/access_token/refresh_token/user key`。
2. `pin` 仅用于派生，不以明文持久化。
3. biometric 数据仅依赖系统能力，不保存生物特征原始数据。
4. 解锁失败默认 fail-closed，不写入 runtime key。

### 性能
1. Master Password 解锁路径必须是单次 KDF + 单次 unwrap。
2. PIN/Biometric 解锁避免额外 IO 与重复反序列化。
3. 解锁接口不得触发同步或网络请求。

## 实施步骤（无迁移版本）
1. 重构后端：引入 `UnlockVaultUseCase`，删除旧多候选解锁逻辑。
2. 引入 canonical `MasterPasswordUnlockData` 结构与存取 port。
3. 新增 PIN ports + use cases + keychain/in-memory adapter。
4. 收敛 biometric 到统一 unlock pipeline。
5. 调整 Tauri commands 与 TS bindings。
6. 更新前端 unlock/settings 交互。
7. 补充单元测试与集成测试。

## 验收标准
1. `master password` 解锁不再出现候选循环与网络依赖。
2. `pin` 支持启用/禁用/解锁，`ephemeral` 重启后失效，`persistent` 重启后可用。
3. `biometric` 能稳定解锁并与统一用例共用收尾逻辑。
4. `needsLogin/locked/unlocked` 状态语义保持一致。
5. 通过 Rust 质量门禁：
   1. `cargo check`
   2. `cargo test`
   3. `cargo clippy --all-targets --all-features -- -D warnings`

## 受影响文件（计划）
1. `src-tauri/src/application/use_cases/unlock_vault_use_case.rs`（新增）
2. `src-tauri/src/application/use_cases/unlock_vault_with_password_use_case.rs`（删除或并入）
3. `src-tauri/src/application/use_cases/vault_pin_use_case.rs`（新增）
4. `src-tauri/src/application/ports/pin_unlock_port.rs`（新增）
5. `src-tauri/src/infrastructure/security/pin_store.rs`（新增）
6. `src-tauri/src/interfaces/tauri/commands/vault.rs`（改造）
7. `src-tauri/src/interfaces/tauri/dto/vault.rs`（新增 unlock/pin DTO）
8. `src/routes/unlock.tsx`（改造）
9. `src/routes/vault.tsx`（设置区改造）
10. `src/bindings.ts`（specta 生成更新）

