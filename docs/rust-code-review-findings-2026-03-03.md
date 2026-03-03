# Rust/Tauri 代码审查问题清单

- 审查日期: 2026-03-03
- 审查范围: `src-tauri/src/**`
- 审查方式: 静态阅读 + `cargo check` + `cargo test` + `cargo clippy --all-targets --all-features -D warnings`

## 1. 执行结果摘要

- `cargo check`: 通过
- `cargo test`: 通过（37 passed, 0 failed）
- `cargo clippy -D warnings`: 失败（22 个错误）

## 2. 高优先级问题（行为级 bug / 一致性风险）

### P1-1 登录成功但后端会话初始化失败时仍返回 Authenticated

- 文件: `src-tauri/src/interfaces/tauri/commands/auth.rs`
- 位置: 69-102, 105
- 描述:
  - 登录成功后，若 `derive_account_id_from_access_token` 或 `set_auth_session` 失败，当前实现仅 `warn` 日志，不返回错误，仍返回 `PasswordLoginResponseDto::Authenticated`。
- 风险:
  - 前端拿到认证成功响应，但后端状态未建立，后续命令会出现“未登录/无会话”错误，形成前后端状态分裂。
- 建议:
  - 会话初始化失败时应直接 fail-closed，返回错误并阻止成功响应。

### P1-2 `sync_now` 防抖与运行槽顺序不合理，导致误伤合法触发

- 文件: `src-tauri/src/application/services/sync_service.rs`
- 位置: 60-84, 491-517
- 描述:
  - 当前先 `enforce_debounce` 再 `acquire_running_slot`。
  - 当“已有同步在跑”导致 acquire 失败时，防抖时间戳已经被写入。
- 风险:
  - 用户下一个合法触发会被错误地判为 debounce，造成体验和行为异常。
- 建议:
  - 先 acquire，再写 debounce；或在 acquire 失败时回滚 debounce 记录。

## 3. 中优先级问题（并发、边界、可维护性）

### P2-1 刷新 token 缺少 singleflight 保护，存在并发刷新竞态

- 文件: `src-tauri/src/interfaces/tauri/session.rs`
- 位置: 108-171
- 描述:
  - 并发请求可能同时触发 refresh，同一账号不存在 refresh 互斥。
- 风险:
  - 若服务端 refresh token 有轮换策略，并发刷新可能让后续请求 401，触发误清理会话。
- 建议:
  - 以 `account_id` 维度增加异步互斥（singleflight）。

### P2-2 详情解密使用启发式匹配，可能误解密普通字符串

- 文件: `src-tauri/src/interfaces/tauri/commands/vault.rs`
- 位置: 519-557, 1173-1182
- 描述:
  - 递归遍历 JSON 所有字符串，只要形态像 `N.xxx|yyy` 就尝试解密。
- 风险:
  - 明文字段若恰好符合形态会导致接口失败（误判为密文）。
- 建议:
  - 按协议字段白名单解密，不对任意字符串做启发式处理。

### P2-3 Async 接口内直接执行阻塞 SQLite + 全局 Mutex 持锁执行

- 文件: `src-tauri/src/infrastructure/persistence/sqlite_vault_repository.rs`
- 位置: 21-66, 640+（全体仓储方法）
- 描述:
  - 使用 `rusqlite`（同步阻塞）在 async trait 方法中直接执行。
  - `connections: Mutex<HashMap<...>>` 在 DB 操作期间一直持锁。
- 风险:
  - 阻塞 tokio runtime 工作线程，跨账号请求串行化，吞吐和响应都受限。
- 建议:
  - 用 `tokio::task::spawn_blocking` 封装 DB 操作，或迁移 async 数据库驱动。
  - 缩小连接表锁的持有范围。

### P2-4 接口层承担大量业务/密码学逻辑，DDD 分层被打穿

- 文件: `src-tauri/src/interfaces/tauri/commands/vault.rs`
- 位置: 413+（解锁流程、KDF 组合、密钥候选、解密策略）
- 描述:
  - `tauri::command` 文件同时承担编排、领域规则、密码学细节。
- 风险:
  - 难以测试和复用，接口层过重，变更风险高。
- 建议:
  - 下沉到 `application/use_cases` 与 `domain/service`，command 仅做 DTO 和错误映射。

### P2-5 托盘菜单行为未完成（Lock/Settings）

- 文件: `src-tauri/src/interfaces/tauri/desktop/tray.rs`
- 位置: 104-111
- 描述:
  - `Lock` 与 `Settings` 仅日志提示 “not wired yet”。
- 风险:
  - UI 显示了可操作菜单但无实际行为，用户感知为功能缺失。
- 建议:
  - 至少连接到已有 `vault_lock` 或设置页入口，或先临时隐藏菜单项。

## 4. 安全与鲁棒性问题

### P3-1 认证状态写盘存在并发竞态窗口

- 文件: `src-tauri/src/bootstrap/app_state.rs`
- 位置: 352-359, 420-433
- 描述:
  - 先写磁盘再更新内存缓存，且临时文件名固定为 `auth-state.json.tmp`。
- 风险:
  - 并发持久化时可能出现覆盖/rename 竞争。
- 建议:
  - 引入专用持久化锁或唯一临时文件名（随机后缀）。

### P3-2 密钥材料未零化（zeroization）

- 文件:
  - `src-tauri/src/bootstrap/app_state.rs`（`VaultUserKey`）
  - `src-tauri/src/bootstrap/auth_persistence.rs`（`SessionWrapRuntime.key`）
- 描述:
  - 密钥缓冲未在 drop 时清零。
- 风险:
  - 增加内存取证风险。
- 建议:
  - 使用 `zeroize`（`Zeroize`/`ZeroizeOnDrop`）管理敏感字节。

### P3-3 允许跳过 TLS 证书校验开关缺少环境约束

- 文件:
  - `src-tauri/src/bootstrap/config.rs`（`allow_invalid_certs` 可配置）
  - `src-tauri/src/infrastructure/vaultwarden/client.rs`（`danger_accept_invalid_certs`）
- 位置: `config.rs` 33, `client.rs` 20
- 描述:
  - 运行时可打开跳过证书校验，未见明确仅限 dev 的硬约束。
- 风险:
  - 若在生产环境误开，存在中间人攻击风险。
- 建议:
  - `debug_assertions` 之外强制关闭，或在 UI 上增加高风险告警与二次确认。

### P3-4 Tauri CSP 为 `null`，默认安全基线偏弱

- 文件: `src-tauri/tauri.conf.json`
- 位置: `app.security.csp = null`
- 描述:
  - 未配置 CSP。
- 风险:
  - 在加载外部内容或插件注入场景下，XSS 防护基线较低。
- 建议:
  - 评估并配置最小可用 CSP 策略（结合现有前端资源清单）。

## 5. Clippy 明确问题（22 项）

以下为当前 `-D warnings` 直接报错项：

### C1 手动 range 模式匹配

- 文件: `src-tauri/src/application/services/realtime_sync_service.rs`
- 位置: 808, 816
- 问题:
  - `matches!(x, 0 | 1 | 2)`、`matches!(x, 12 | 13 | 14)` 可改 `0..=2`、`12..=14`。

### C2 构造函数参数过多

- 文件: `src-tauri/src/bootstrap/auth_persistence.rs`
- 位置: 49
- 问题:
  - `PersistedAuthState::new(...)` 参数 8 个，触发 `too_many_arguments`。

### C3 Result 的 Err 体积过大（多处）

- 文件:
  - `src-tauri/src/infrastructure/vaultwarden/client.rs`（多处 endpoint/new/validated_base_url）
  - `src-tauri/src/infrastructure/vaultwarden/config.rs`（validate_base_url）
- 位置:
  - `client.rs`: 18, 56, 61, 66, 71, 76, 81, 86, 91, 96, 480
  - `config.rs`: 29
- 问题:
  - `VaultwardenError::TokenRejected` 含较大结构体，触发 `result_large_err`。

### C4 不必要显式生命周期

- 文件: `src-tauri/src/infrastructure/vaultwarden/client.rs`
- 位置: 480
- 问题:
  - `fn validated_base_url<'a>(...) -> ...` 可生命周期省略。

### C5 多余的 `-> ()`

- 文件: `src-tauri/src/interfaces/tauri/desktop/spotlight.rs`
- 位置: 26, 27
- 问题:
  - `panel_event!` 方法返回类型 `-> ()` 可去掉。

### C6 `Result.ok()` 后匹配 `Some`（冗余）

- 文件: `src-tauri/src/interfaces/tauri/desktop/window_placement.rs`
- 位置: 114, 131
- 问题:
  - 建议直接 `if let Ok(x) = ...`。

## 6. 架构与最佳实践改进建议（非阻塞）

1. 把 `require_non_empty` 这类重复参数校验抽成统一 validator（`support` 或 `application`）减少重复代码。  
2. 统一 endpoint 拼接逻辑（`sync_service` / `sync_vault_use_case` 里有重复 endpoint helper）。  
3. `AppError` 现在只分四类，建议补充细分错误码（auth、network、decode、storage）以便前端精确处理。  
4. 可增加关键集成测试：
   - 登录成功但本地会话写入失败应返回错误。
   - 并发 refresh 仅发生一次远端刷新。
   - “同步进行中”不应污染 debounce。

## 7. 建议修复顺序

1. 先修 P1（登录状态分裂、debounce 顺序）。  
2. 再修并发和分层（P2-1 ~ P2-4）。  
3. 最后处理安全基线与 clippy 清理（P3 + C1~C6）。  

