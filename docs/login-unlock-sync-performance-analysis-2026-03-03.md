# 登录-解锁-同步性能分析（对照 Bitwarden）

日期：2026-03-03

## 背景与范围
本文基于当前项目代码路径（前端 React 路由 + Tauri Rust 应用层/基础设施层）与 Bitwarden 客户端公开实现，对“登录 -> 解锁 -> 同步”链路的慢点进行定位，并给出可落地的优化优先级。

## 结论
1. 当前慢的核心不是单点，而是多段串行阻塞叠加：登录页、Vault 页初始化、解锁页都存在多个 `await` 串行调用，导致用户感知延迟被线性放大。
2. “解锁”路径混入了远程 `prelogin` 请求，破坏了“解锁应尽量本地完成”的预期；网络抖动会直接放大解锁耗时。
3. 同步链路存在“先拉全量 `/api/sync`，再查 revision 决定是否持久化”的顺序问题，导致在 revision 未变化时仍然消耗主要网络与解析成本。
4. 本地持久化采用 live/staging 双表拷贝事务，完整同步时 I/O 与 JSON 反序列化成本较高；再叠加启动时的全量视图读取/解密，首屏延迟明显。

## 当前项目慢点定位
### 1) 前端链路串行等待
- 登录页在 `authLoginWithPassword` 后继续串行 `vaultCanUnlock`、`vaultUnlockWithPassword`、`vaultSync`、跳转，任何一段慢都会阻塞下一段。
- Vault 页启动串行请求 `authRestoreState` -> `vaultGetBiometricStatus` -> `vaultIsUnlocked` -> `vaultGetViewData`，首屏需等待全链路完成。
- Unlock 页启动同样串行恢复状态/生物能力判断。

影响：端到端等待时间近似为各步骤耗时之和，而不是取最大值。

### 2) 解锁路径发生远程 prelogin
- `unlock_vault_with_password_use_case` 在本地候选密钥不足时会调用 `auth_service.prelogin(...)` 获取 KDF 参数并继续派生。
- 网络慢或失败时会直接增加解锁路径时延，并引入额外不确定性。

影响：解锁从“本地快路径”退化为“本地 + 网络”混合路径。

### 3) 登录后初始化包含背景同步启动
- `auth_login_with_password` 成功后进入 `initialize_authenticated_session`，其中会 `await session::start_background_sync(...)`。
- `start_background_sync` 内部包含 `realtime_sync_service.start_for_account(...).await`。

影响：登录成功响应前引入额外初始化耗时，用户感知为“点了登录但页面迟迟不动”。

### 4) 同步顺序与持久化成本
- `sync_vault_use_case.execute_once` 先请求 `remote_vault.sync_vault(...)`，之后才请求 revision-date 并判断是否跳过 payload 落库。
- 即使最终 `skip_payload_persist = true`，全量同步请求与解析成本已经发生。
- 持久化层 `sqlite_vault_repository` 在事务中存在 live -> staging、staging -> live、清理 staging 的多轮拷贝与写入。

影响：在“无变化”或“小变化”场景下，仍会付出接近全量同步成本。

### 5) 视图读取热路径偏重
- `get_vault_view_data_use_case` 在取视图数据时会进行数据读取与字段解密。
- `sqlite_vault_repository` 列表读取路径存在频繁 JSON 反序列化热点。

影响：解锁后进入 Vault 首屏时，CPU 与 I/O 抖动会直接反映为渲染等待。

### 6) 超时与串行叠加
- HTTP 请求超时默认为 `15000ms`，在多段串行场景下，最坏时间会被放大。

影响：弱网下极端尾延迟（P95/P99）较高。

## 与 Bitwarden 流程对照
1. Bitwarden 对“Log in（服务器鉴权）”与“Unlock（本地解密）”有明确区分，产品语义上强调解锁应更接近本地操作。
2. Bitwarden 登录组件存在 prelogin 预取能力（密码登录策略中使用 KDF 预取），倾向把网络依赖前置到登录阶段，而不是放到解锁热路径。
3. Bitwarden 的默认同步服务是后台调度思路；其 lock 相关实现里也有“某些同步/再生成流程不应阻塞”的注释，方向上强调降低 UI 阻塞。

## 改进建议（按优先级）
### P0（优先做，直接影响体感）
1. 登录成功后“先返回，再启动后台同步”
- 将 `initialize_authenticated_session` 中的 `start_background_sync(...).await` 改为非阻塞派发（保持错误日志，但不阻塞登录响应）。

2. 解锁路径去网络依赖
- 登录成功、同步成功时落盘稳定 KDF 参数与派生上下文。
- 解锁默认只走本地；仅在本地材料缺失时触发“可观测的降级路径”（并做超时兜底）。

3. 启动阶段并行化
- 前端对相互独立的状态查询改为 `Promise.all`。
- `vaultGetViewData` 延后到已确认 unlock 后触发，必要时做 skeleton/分段加载。

4. 自动同步前置 revision 预检
- 非手动触发下先请求 revision-date，未变化则跳过 `/api/sync` 拉取。
- 手动同步保留强制全量语义。

### P1（中期，减少资源开销）
1. 简化 SQLite 快照事务
- 评估从“双表复制提交”改为“单表版本化 + 原子替换/增量 upsert”。
- 优先减少全表复制与重复 JSON 序列化/反序列化。

2. 视图数据按需解密与分页
- 首屏只解密必要字段与可见区数据。
- 非关键字段改为惰性解密。

3. 增加链路埋点
- 对登录、解锁、同步、落库、首屏渲染增加阶段耗时埋点，输出 P50/P95/P99，避免“凭感觉优化”。

### P2（策略调优）
1. 按场景区分超时策略
- 登录/解锁依赖请求与后台轮询请求使用不同超时阈值。

2. 优化同步触发治理
- 在 `running slot + debounce` 机制下增加“挂起一次后续触发”能力，避免高频事件全部被抑制后用户感知“不同步/很慢”。

## 参考来源
### 项目内代码（本仓库）
- `src/routes/index.tsx`（登录后串行链路）：266, 285, 297, 336
- `src/routes/vault.tsx`（Vault 启动串行链路）：164, 173, 189, 195
- `src/routes/unlock.tsx`（Unlock 启动链路）：70, 87, 97
- `src-tauri/src/interfaces/tauri/commands/auth.rs`（登录后会话初始化与后台同步触发）：28-54, 59-95
- `src-tauri/src/interfaces/tauri/session.rs`（后台同步启动）：242-274
- `src-tauri/src/application/use_cases/unlock_vault_with_password_use_case.rs`（解锁时 prelogin 回退）：131-163
- `src-tauri/src/application/use_cases/sync_vault_use_case.rs`（全量同步 + revision 逻辑 + 持久化流程）：536-744, 1126-1144
- `src-tauri/src/application/services/sync_service.rs`（running slot 与 debounce）：60-85, 476-511
- `src-tauri/src/infrastructure/persistence/sqlite_vault_repository.rs`（staging/live 拷贝与事务提交）：386-645, 876-905, 1272-1313
- `src-tauri/src/application/use_cases/get_vault_view_data_use_case.rs`（视图读取/解密）：43-90
- `src-tauri/src/infrastructure/vaultwarden/config.rs`（HTTP 超时默认值）：24-25

### Bitwarden 公开资料与源码
- 登录与解锁语义说明：
  - https://bitwarden.com/help/understand-log-in-vs-unlock/
- Vault 超时与锁定行为：
  - https://bitwarden.com/help/vault-timeout/
- Bitwarden 客户端源码（登录策略/预取与同步实现）：
  - https://github.com/bitwarden/clients/blob/master/libs/auth/src/common/services/login-strategies/login-strategy.service.ts
  - https://github.com/bitwarden/clients/blob/master/libs/auth/src/common/login-strategies/password-login.strategy.ts
  - https://github.com/bitwarden/clients/blob/master/libs/auth/src/angular/login/login.component.ts
  - https://github.com/bitwarden/clients/blob/master/libs/common/src/platform/sync/default-sync.service.ts
  - https://github.com/bitwarden/clients/blob/master/libs/key-management-ui/src/lock/components/lock.component.ts

## 备注
- 本文聚焦架构与代码路径分析，未引入实测 benchmark。建议先完成 P0 后做一次 A/B 量化（登录耗时、解锁耗时、首屏可交互时间、同步完成时间）。
