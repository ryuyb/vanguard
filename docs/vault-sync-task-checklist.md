# Vault 同步从 0 到 V3 任务清单

本文档基于 `vault_sync_flow.md` 与当前项目分层（Tauri + DDD）整理，目标是按阶段稳定落地 Vaultwarden 同步能力。

## 里程碑总览

- V0：同步基础设施搭建（可发请求、可入库、可观测）
- V1：全量同步 + revision 轮询兜底（最小可用）
- V2：WebSocket 通知驱动同步（实时性增强）
- V3：精细增量同步 + 性能与稳定性优化（生产可用）

## V0 基础搭建

### Domain

- [x] 定义同步上下文实体 `SyncContext`（`account_id/base_url/last_revision/sync_status/last_error/ws_status`）
- [ ] 定义快照元数据 `VaultSnapshotMeta`（`snapshot_revision/snapshot_synced_at/source`）
- [x] 定义统一同步触发源枚举 `SyncTrigger`（`Startup/Manual/Poll/WebSocket/Foreground`）
- [x] 定义同步结果值对象 `SyncResult`（`duration_ms/item_counts/revision_changed`）

### Application

- [ ] 新增 `SyncVaultUseCase`（执行一次 `GET /api/sync` + 本地落库）（部分完成：已抽象 SyncVaultUseCase 并接入调用链，尚未实现实体级本地落库）
- [x] 新增 `PollRevisionUseCase`（执行 `GET /api/accounts/revision-date`）
- [x] 新增 `SyncOrchestrator`（串行化同一 account 的同步任务，防重入）
- [x] 约定 `SyncPolicy`（失败重试策略、触发去抖策略、超时）

### Ports

- [x] 定义 `RemoteVaultSyncPort`（`sync_vault`、`get_revision_date`）（以 RemoteVaultPort 形式承载）
- [x] 定义 `VaultRepositoryPort` 同步相关接口（upsert/transaction/save_sync_context/load_sync_context）
- [x] 定义 `SyncEventPort`（向前端发同步状态事件）

### Infrastructure

- [x] 在 `vaultwarden/client.rs` 增加 `/api/sync` 与 `/api/accounts/revision-date` 调用
- [x] 在 `vaultwarden/models.rs` 增加 sync/revision response 模型
- [x] 在 `vaultwarden/mapper.rs` 增加远端模型到领域对象映射
- [ ] 实现本地仓储（建议 SQLite）并支持事务写入（部分完成：已实现 in-memory 仓储与事务语义，尚未替换为 SQLite）
- [x] 接入日志打点（每次同步的开始/结束/失败）

### Interfaces (Tauri)

- [x] 新增命令 `vault_sync_now(account_id)`
- [x] 新增命令 `vault_sync_status(account_id)`
- [x] 新增事件 `vault-sync:started/vault-sync:succeeded/vault-sync:failed`
- [x] 使用 tauri-specta 导出新增 DTO 类型

### V0 验收标准

- [x] 登录成功后可手动触发一次全量同步并入库
- [x] 同步失败时有明确错误日志（包含 account_id、endpoint、status）
- [x] 同一 account 并发触发同步只执行一次（其余合并或丢弃）

### V0 收尾 TODO（基于部分完成项）

- [x] 扩展 `SyncContext` 字段：补充 `base_url` 与 `ws_status`
- [ ] 新增 `VaultSnapshotMeta` 领域对象（`snapshot_revision/snapshot_synced_at/source`）
- [x] 新增 `SyncTrigger` 枚举并在同步入口传入触发源
- [x] 新增 `SyncResult` 值对象（`duration_ms/item_counts/revision_changed`）
- [x] 将 `sync_now` 重命名/抽象为 `SyncVaultUseCase`，并返回 `SyncResult`
- [x] 将 revision 查询逻辑从 `sync_now` 拆分为独立 `PollRevisionUseCase`
- [x] 定义 `SyncPolicy`（`max_retries/backoff_ms/debounce_ms/timeout_ms`）并在应用层消费
- [x] 扩展 `VaultRepositoryPort`：增加 `upsert_*` 与事务接口（例如 `transaction`）
- [x] 将同步状态事件下沉为 `SyncEventPort`，Tauri 层通过适配器实现
- [x] 将 revision-date response 模型迁移到 `vaultwarden/models.rs` 并统一复用
- [x] 将 sync 映射逻辑从 `port_adapter.rs` 下沉到 `vaultwarden/mapper.rs`
- [ ] 实现 SQLite 仓储并替换当前 in-memory 实现
- [x] 在同步流程增加“开始日志”，并统一输出 `account_id/endpoint/status/error_code`
- [x] 统一事件命名为 `vault-sync:started/vault-sync:succeeded/vault-sync:failed`
- [ ] 在联调流程中补齐“实体级入库成功”的可见验证（例如同步后可查询本地 ciphers 数量）
- [x] 增强失败日志：明确区分网络错误/HTTP 状态错误/解析错误

## V1 最小可用（全量 + 轮询）

### 功能

- [ ] 登录成功自动触发一次 `sync_vault`
- [ ] 首次全量后立即刷新 `revision-date` 并保存
- [ ] 启动后台轮询任务（建议 30~120s，可配置）
- [ ] 轮询发现 revision 变化后自动全量同步
- [ ] 前台恢复时执行一次 revision 检查
- [ ] 提供手动刷新入口，触发全量同步

### 数据一致性

- [ ] 全量同步使用事务提交（profile/folders/collections/policies/ciphers/sends）
- [ ] 本地 upsert 以 `id` 为主键，支持重复同步幂等
- [ ] 处理删除（不存在对象标记删除或硬删，二选一并固定策略）

### 异常处理

- [ ] 401/403：清理会话并停止自动同步，通知前端重新登录
- [ ] 网络错误：指数退避重试
- [ ] 5xx：保留本地旧数据，状态标记 degraded

### V1 验收标准

- [ ] 不依赖 WebSocket 的情况下，同步可持续更新
- [ ] 连续 10 次重复全量同步后本地数据保持稳定（无重复、无异常膨胀）
- [ ] 杀进程重启后，`last_revision` 与同步状态可恢复

## V2 实时同步（WebSocket 通知）

### 连接与会话

- [ ] 实现 `/notifications/hub` 建连管理
- [ ] 实现 messagepack 握手（含 `0x1e` 分隔）
- [ ] 实现断线重连（指数退避 + 最大重试间隔）
- [ ] WS 在线时降低轮询频率，离线时提升轮询频率

### 事件路由（先“稳妥全量”策略）

- [ ] `Type=11 LogOut`：立即本地下线
- [ ] `Type=5 SyncVault`：触发全量 `sync_vault`
- [ ] `Type=4 SyncCiphers`：先走全量 `sync_vault`
- [ ] `Type=0/1/2`：先走全量 `sync_vault`
- [ ] `Type=3/7/8`：先走全量 `sync_vault`（后续可优化为 folders 局刷）
- [ ] `Type=10`：全量或设置刷新策略落地
- [ ] `Type=12/13/14`：支持 Send 时刷新 sends，不支持则忽略并打 debug 日志

### 可观测性

- [ ] 增加 WS 生命周期日志（connect/disconnect/reconnect/message_type）
- [ ] 增加触发链路日志（trigger -> enqueue -> run -> done）

### V2 验收标准

- [ ] 两端同时登录同账号时，一端变更可在另一端 5 秒内触发同步
- [ ] WS 断网后可自动恢复，恢复后会做一次兜底全量同步
- [ ] 无消息风暴导致的并发同步堆积（有去抖/队列上限）

## V3 精细增量与优化

### 精细增量

- [ ] 为 `Type=0/1/2` 增加按 `Payload.Id` 的条目级更新能力
- [ ] 为 `Type=3/7/8` 增加文件夹级局部刷新
- [ ] 为 `Type=12/13/14` 增加 Send 局部刷新
- [ ] 保留“失败回退全量同步”兜底路径

### 性能优化

- [ ] 支持批量写入与分批事务（大库场景）
- [ ] 减少不必要字段写入（revision 未变化跳过）
- [ ] 同步结果缓存统计（耗时、对象数量、失败率）

### 稳定性与安全

- [ ] 附件 URL 只短期使用，不长期缓存
- [ ] 日志脱敏（不输出 token、password、masterPasswordHash）
- [ ] 关键路径超时控制（HTTP、WS、DB）
- [ ] 加入异常注入测试（网络抖动、WS 闪断、DB 锁冲突）

### V3 验收标准

- [ ] 大于 1 万条 ciphers 场景下，同步耗时与内存占用可接受（定义阈值并记录）
- [ ] 常见更新事件可走局部刷新，无需每次全量
- [ ] 全链路具备可排障日志与错误码，线上问题可复现定位

## 测试清单（贯穿 V0~V3）

- [ ] Domain 单元测试：幂等合并、删除策略、状态流转
- [ ] Application 单元测试：触发路由、并发互斥、失败重试
- [ ] Infrastructure 集成测试：`/api/sync`、`revision-date`、WS 事件模拟
- [ ] Tauri 命令测试：参数校验、状态查询、错误映射
- [ ] 端到端验证：登录 -> 首次同步 -> 轮询更新 -> WS 更新 -> 登出失效

## 推荐实施顺序

- [ ] 第 1 周：V0 完成并可手动同步
- [ ] 第 2 周：V1 完成并稳定运行
- [ ] 第 3 周：V2 完成并接入 WS
- [ ] 第 4 周：V3 关键优化 + 压测 + 收口
