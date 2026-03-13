## Why

当前项目的配置持久化方式分散在前端 localStorage 与后端多个存储路径中，导致配置职责不清、行为不一致，并且现有单文件 auth-state 无法稳定支持多账户并存。现在需要统一配置入口并重构授权状态存储，以支撑可预测的多账户体验与后续维护。

## What Changes

- 将语言偏好从前端 localStorage 迁移为后端统一配置读写，前端不再直接持久化配置。
- 扩展后端统一配置存储（app-config）承载全局应用偏好，包括 locale、device_identifier、allow_invalid_certs、sync_poll_interval_seconds。
- 将 Vault Settings 中当前仅存在前端组件状态/临时状态的配置项纳入 app-config 统一持久化与读取（例如 launchOnLogin、showWebsiteIcon、快捷键、自动锁定与剪贴板清理相关选项等）。
- 将授权状态从单文件 `auth-state.json` 调整为按账户隔离的 `auth-states/{account_id}.json`。
- 新增 `auth-states/active.json` 记录当前活跃账户标识，用于会话恢复与账户切换。
- 明确本次变更不包含旧数据迁移逻辑：不读取旧 localStorage 语言值，不读取旧单文件 auth-state。
- 保持账户数据库与 Keychain（PIN/生物解锁）现有按账户隔离策略不变。

## Capabilities

### New Capabilities
- `backend-config-unification`: 后端提供统一的应用配置持久化与访问能力，前端通过后端接口读取与更新配置。
- `multi-account-auth-state-storage`: 授权状态按账户文件隔离存储，并维护活跃账户索引以支持多账户恢复。

### Modified Capabilities
- `language-preference`: 语言偏好持久化从前端 localStorage 调整为后端统一配置存储。

## Impact

- 前端：
  - 语言初始化与语言保存逻辑改为调用后端命令，不再依赖 localStorage。
  - Vault Settings 通用偏好项改为从后端 app-config 初始化并写回后端，不再仅存在组件内存状态。
- 后端：
  - 配置模块需要扩展 locale 与 Vault Settings 字段，并提供统一访问接口。
  - 认证状态读写模块需要从单文件改为目录化多文件结构，并维护 active account 文件。
  - 认证恢复与登出流程需要基于新 auth-state 存储路径工作。
- 数据与行为：
  - 由于不做迁移，升级后历史语言偏好与旧 auth-state 不会自动继承。
  - 多账户授权状态可并存，避免账户切换覆盖 refresh token。
