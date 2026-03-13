## Context

当前项目存在三类配置与状态持久化路径：

1. 前端 localStorage 保存语言偏好。
2. 后端 `app-config.json`（tauri-plugin-store）保存部分全局配置（`device_identifier`、`allow_invalid_certs`、`sync_poll_interval_seconds`）。
3. 后端单文件 `auth-state.json` 保存加密后的授权状态。

同时，Vault Settings 中多项用户偏好当前仅保存在前端组件状态，未形成稳定持久化来源（例如 launchOnLogin、showWebsiteIcon、快捷键、自动锁定与剪贴板清理选项）。

该现状导致：
- 配置入口不统一，前后端配置来源不一致。
- Vault Settings 的通用偏好跨窗口/重启不可预测。
- 授权状态仅支持单账户覆盖写入，无法稳定支持多账户并存。
- 恢复逻辑依赖单一 auth-state 文件，账户切换时可预测性不足。

本变更目标是统一后端配置入口，并将授权状态改为按账户隔离文件存储，同时明确本次不做旧数据迁移。

## Goals / Non-Goals

**Goals:**
- 所有应用级偏好由后端统一持久化与读取，语言偏好与 Vault Settings 通用偏好纳入 `app-config.json`。
- 授权状态采用 `auth-states/{account_id}.json` 按账户隔离存储。
- 新增 `auth-states/active.json` 维护当前活跃账户索引，支撑恢复与切换路径。
- 保持现有数据库与 Keychain 的账户隔离策略不变。
- 在无迁移约束下完成一致行为：只识别新结构数据。

**Non-Goals:**
- 不实现旧 localStorage 语言偏好迁移。
- 不实现旧单文件 `auth-state.json` 迁移。
- 不引入账户级偏好配置层（Layer 3）。
- 不调整 PIN / 生物识别数据模型与存储机制。

## Decisions

### 决策 1：应用配置单一来源为后端 app-config
- 方案：在 `app-config.json` 中新增并维护 `locale` 与 Vault Settings 通用偏好字段，前端通过后端命令读写。
- 原因：
  - 消除前端 localStorage、前端内存状态与后端配置并存导致的一致性问题。
  - 维持配置职责单一，便于测试与排障。
- 备选：保留前端 localStorage/组件状态作为缓存来源。
- 放弃原因：会继续引入双写与来源竞争，不符合统一目标。

### 决策 2：授权状态按账户文件隔离 + 活跃账户索引
- 方案：
  - `auth-states/{account_id}.json`：每个账户一个加密授权状态文件。
  - `auth-states/active.json`：记录当前活跃 `account_id`。
- 原因：
  - 直接消除单文件覆盖问题。
  - 对多账户恢复与切换更直观。
- 备选：单文件中维护账户映射字典。
- 放弃原因：单文件并发与冲突处理更复杂，局部损坏影响面更大。

### 决策 3：无迁移发布策略
- 方案：新版本仅读取新结构，不读取旧结构。
- 原因：
  - 降低实现复杂度与风险，缩短交付路径。
  - 与当前范围约束一致（明确不考虑迁移）。
- 备选：启动时自动迁移旧 localStorage 与旧 auth-state。
- 放弃原因：增加兼容逻辑、回退成本和测试矩阵。

### 决策 4：保持全局配置作用域，不引入账户级偏好层
- 方案：`sync_poll_interval_seconds` 及 Vault Settings 通用偏好继续为全局配置。
- 原因：
  - 遵循 YAGNI，当前需求仅要求 auth-state 按账户隔离。
  - 避免提前引入 Layer 3 复杂度。
- 备选：将通用偏好改为账户级。
- 放弃原因：当前未被需求明确要求。

## Risks / Trade-offs

- [升级后旧配置不可用] → 通过发布说明明确：用户需重新登录且偏好回到默认后再手动设置。
- [active.json 与实际会话状态短暂不一致] → 所有登录/切换/登出路径统一封装写入顺序并做失败回滚（先持久化账户 state，再更新 active）。
- [多文件读写带来的 I/O 失败面增加] → 复用现有原子写策略（tmp + rename）并保持错误可观测日志。
- [前端初始化依赖后端配置读取时序] → 在应用启动与设置弹窗初始化阶段统一等待 app-config 读取完成后再渲染相关文案与控件状态。
