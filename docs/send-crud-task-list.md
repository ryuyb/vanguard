# Send CRUD — Task List

> 基于 [send-crud-implementation-plan.md](./send-crud-implementation-plan.md) 的分步实现任务清单。
> 每个 task 有明确的输入/输出文件、验收标准和前置依赖。

## 总览

| # | 模块 | Task | 依赖 | 验收命令 |
|---|------|------|------|----------|
| 1 | 数据模型 | 扩展 SyncSend 完整字段 | — | `cargo test && cargo clippy` |
| 2 | 持久化 | VaultRepositoryPort list/get sends | 1 | `cargo test` |
| 3 | 远程 API | RemoteVaultPort + Client CRUD | 1 | `cargo test && cargo clippy` |
| 4 | 加密 | send_encryption.rs | 1 | `cargo test`（单元测试） |
| 5 | 用例 | Send CRUD Use Cases + Events | 2, 3, 4 | `cargo test` |
| 6 | 接口 | Tauri Commands + AppState 布线 | 5 | `cargo build && cargo clippy` |
| 7 | 前端 | Send 列表 + 侧边栏集成 | 6 | `pnpm build` |
| 8 | 前端 | Send 详情面板 | 7 | `pnpm build` |
| 9 | 前端 | Send 创建/编辑表单 | 7 | `pnpm build` |
| 10 | 前端 | Send 删除 + 完整集成 | 8, 9 | 全栈 pre-commit |

---

## Task 1: 扩展 SyncSend 数据模型为完整字段

**依赖：** 无
**模块：** 数据模型层（application/dto + infrastructure/vaultwarden）

### 1.1 扩展 application DTO

**文件：** `src-tauri/src/application/dto/sync.rs`

- [x] 新增 `SyncSendText` 结构体
  ```rust
  pub struct SyncSendText {
      pub text: Option<String>,
      pub hidden: Option<bool>,
  }
  ```
- [x] 新增 `SyncSendFile` 结构体
  ```rust
  pub struct SyncSendFile {
      pub id: Option<String>,
      pub file_name: Option<String>,
      pub size: Option<String>,
      pub size_name: Option<String>,
  }
  ```
- [x] 扩展 `SyncSend`，在现有 6 字段基础上添加：
  - `access_id: Option<String>`
  - `notes: Option<String>`
  - `key: Option<String>`
  - `password: Option<String>`
  - `text: Option<SyncSendText>`
  - `file: Option<SyncSendFile>`
  - `max_access_count: Option<i32>`
  - `access_count: Option<i32>`
  - `disabled: Option<bool>`
  - `hide_email: Option<bool>`
  - `expiration_date: Option<String>`
  - `emails: Option<String>`
  - `auth_type: Option<i32>`
- [x] 新增 Send 变更命令 DTO：
  - `CreateSendCommand`（account_id, base_url, access_token, send: SyncSend）
  - `UpdateSendCommand`（account_id, base_url, access_token, send_id, send: SyncSend）
  - `DeleteSendCommand`（account_id, base_url, access_token, send_id）
  - `SendMutationResult`（send_id, revision_date）
  - `CreateFileSendResult`（send_id, file_id, url, revision_date）

### 1.2 扩展 Vaultwarden 远程模型

**文件：** `src-tauri/src/infrastructure/vaultwarden/models.rs`

- [x] 扩展远程 `SyncSend` 结构体，添加与 1.1 对应的 camelCase 字段
- [x] 新增 `SyncSendText`（远程）和 `SyncSendFile`（远程）
- [x] 新增 `SendRequestModel`（用于 POST/PUT 请求体序列化）：
  ```rust
  pub struct SendRequestModel {
      pub r#type: Option<i32>,
      pub auth_type: Option<i32>,
      pub file_length: Option<i64>,
      pub name: Option<String>,
      pub notes: Option<String>,
      pub key: String,
      pub max_access_count: Option<i32>,
      pub expiration_date: Option<String>,
      pub deletion_date: String,
      pub file: Option<SyncSendFile>,
      pub text: Option<SyncSendText>,
      pub password: Option<String>,
      pub emails: Option<String>,
      pub disabled: bool,
      pub hide_email: Option<bool>,
  }
  ```
- [x] 新增 `SendFileUploadDataResponse`（file_id, url 等）

### 1.3 更新 mapper

**文件：** `src-tauri/src/infrastructure/vaultwarden/mapper.rs`

- [x] 更新 `map_sync_send` 函数，映射所有新增字段
- [x] 新增 `map_send_to_request_model`（SyncSend → SendRequestModel）

### 验收标准

- [x] `cargo test` — 现有测试全部通过（同步功能不受影响）
- [x] `cargo clippy --all-targets --all-features -- -D warnings` — 无警告
- [ ] 全量同步后 `live_sends` 表中的 JSON payload 包含完整字段

---

## Task 2: 扩展 VaultRepositoryPort — Send 的 list 和 get

**依赖：** Task 1
**模块：** 持久化层（application/ports + infrastructure/persistence）

### 2.1 扩展 Port 接口

**文件：** `src-tauri/src/application/ports/vault_repository_port.rs`

- [x] 添加 `list_live_sends(&self, account_id: &str) -> AppResult<Vec<SyncSend>>`
- [x] 添加 `get_live_send(&self, account_id: &str, send_id: &str) -> AppResult<Option<SyncSend>>`
- [x] 添加 `upsert_send(&self, account_id: &str, send: &SyncSend) -> AppResult<()>`（用于 CRUD 后本地持久化）
- [x] 添加 `delete_send(&self, account_id: &str, send_id: &str) -> AppResult<()>`

### 2.2 实现 SQLite 适配器

**文件：** `src-tauri/src/infrastructure/persistence/sqlite_vault_repository.rs`

- [x] 实现 `list_live_sends` — 参考 `list_live_ciphers`，查询 `live_sends` 表，反序列化 JSON payload
- [x] 实现 `get_live_send` — 参考 `get_live_cipher`，按 id 查询单条
- [x] 实现 `upsert_send` — INSERT OR REPLACE 到 `live_sends`
- [x] 实现 `delete_send` — DELETE FROM `live_sends` WHERE id = ?

### 验收标准

- [x] `cargo test` — 通过
- [x] 新增方法签名与现有 cipher 方法模式一致

---

## Task 3: 扩展 RemoteVaultPort 和 VaultwardenClient — Send 远程 CRUD API

**依赖：** Task 1
**模块：** 远程 API 层（application/ports + infrastructure/vaultwarden）

### 3.1 新增 Endpoints

**文件：** `src-tauri/src/infrastructure/vaultwarden/endpoints.rs`

- [x] `sends(base_url) -> "{base}/api/sends"` — 创建 Text Send / 获取所有
- [x] `sends_file_v2(base_url) -> "{base}/api/sends/file/v2"` — 创建 File Send
- [x] `send_file_upload(base_url, send_id, file_id) -> "{base}/api/sends/{send_id}/file/{file_id}"` — 上传文件

### 3.2 扩展 VaultwardenClient

**文件：** `src-tauri/src/infrastructure/vaultwarden/client.rs`

- [x] `create_send(base_url, access_token, body: &SendRequestModel) -> SyncSend` — POST /api/sends
- [x] `create_file_send(base_url, access_token, body: &SendRequestModel) -> SendFileUploadDataResponse` — POST /api/sends/file/v2
- [x] `upload_send_file(base_url, access_token, send_id, file_id, file_data: Vec<u8>) -> ()` — POST multipart
- [x] `update_send(base_url, access_token, send_id, body: &SendRequestModel) -> SyncSend` — PUT /api/sends/{id}
- [x] `delete_send(base_url, access_token, send_id) -> ()` — DELETE /api/sends/{id}

### 3.3 扩展 RemoteVaultPort trait

**文件：** `src-tauri/src/application/ports/remote_vault_port.rs`

- [x] `create_send(command: CreateSendCommand) -> AppResult<SendMutationResult>`
- [x] `create_file_send(command: CreateSendCommand) -> AppResult<CreateFileSendResult>`
- [x] `upload_send_file(base_url, access_token, send_id, file_id, file_data: Vec<u8>) -> AppResult<()>`
- [x] `update_send(command: UpdateSendCommand) -> AppResult<SendMutationResult>`
- [x] `delete_send(command: DeleteSendCommand) -> AppResult<()>`

### 3.4 实现 VaultwardenRemotePort 适配器

**文件：** `src-tauri/src/infrastructure/vaultwarden/port_adapter.rs`

- [x] 实现上述 5 个方法，调用 client 方法 + mapper 转换

### 验收标准

- [x] `cargo test && cargo clippy` — 通过
- [x] 所有新增 trait 方法有对应的 adapter 实现

---

## Task 4: 实现 Send 加密模块

**依赖：** Task 1
**模块：** 应用层加密（application/send_encryption）

### 4.1 新建加密模块

**文件：** `src-tauri/src/application/send_encryption.rs`（新建）

- [x] `generate_send_key() -> Vec<u8>` — 生成 16 字节随机 key
- [x] `derive_send_shareable_key(send_key: &[u8]) -> (Vec<u8>, Option<Vec<u8>>)` — HKDF 派生（info="send", salt="send"）
- [x] `encrypt_send_key(send_key: &[u8], user_key: &VaultUserKeyMaterial) -> AppResult<String>` — 用 user key 加密 send key
- [x] `decrypt_send_key(encrypted_key: &str, user_key: &VaultUserKeyMaterial) -> AppResult<Vec<u8>>` — 解密 send key
- [x] `encrypt_send(send: &SyncSend, user_key: &VaultUserKeyMaterial) -> AppResult<SyncSend>` — 加密 name, notes, text.text, file.file_name
- [x] `decrypt_send(send: &SyncSend, user_key: &VaultUserKeyMaterial) -> AppResult<SyncSend>` — 解密上述字段
- [x] `hash_send_password(password: &str, send_key: &[u8]) -> String` — PBKDF2(password, send_key, 100000) → base64

### 4.2 注册模块

**文件：** `src-tauri/src/application/mod.rs`

- [x] 添加 `pub mod send_encryption;`

### 验收标准

- [x] 单元测试：encrypt → decrypt 往返一致性
- [x] 单元测试：password hash 输出格式正确（base64）
- [x] `cargo test` — 通过
- [x] 参考 `cipher_encryption.rs` 的模式，使用相同的底层 crypto 工具

---

## Task 5: 实现 Send CRUD Use Cases + Events

**依赖：** Task 2, 3, 4
**模块：** 应用层用例 + 事件

### 5.1 CreateSendUseCase

**文件：** `src-tauri/src/application/use_cases/create_send_use_case.rs`（新建）

- [x] 注入 `RemoteVaultPort`, `VaultRepositoryPort`, `SyncEventPort`
- [x] `execute(account_id, base_url, access_token, send: SyncSend, user_key, file_data: Option<Vec<u8>>) -> AppResult<SendMutationResult>`
- [x] 流程：验证 name 非空 → encrypt_send → 判断类型：
  - Text: `remote_vault.create_send` → 本地 `upsert_send` → emit
  - File: `remote_vault.create_file_send` → `upload_send_file(file_data)` → 本地 `upsert_send` → emit

### 5.2 UpdateSendUseCase

**文件：** `src-tauri/src/application/use_cases/update_send_use_case.rs`（新建）

- [x] 注入 `RemoteVaultPort`, `VaultRepositoryPort`, `SyncEventPort`
- [x] `execute(account_id, base_url, access_token, send_id, send: SyncSend, user_key) -> AppResult<SendMutationResult>`
- [x] 流程：验证 → encrypt_send → `remote_vault.update_send` → 本地 `upsert_send` → emit

### 5.3 DeleteSendUseCase

**文件：** `src-tauri/src/application/use_cases/delete_send_use_case.rs`（新建）

- [x] 注入 `RemoteVaultPort`, `VaultRepositoryPort`, `SyncEventPort`
- [x] `execute(account_id, base_url, access_token, send_id) -> AppResult<()>`
- [x] 流程：验证 → `remote_vault.delete_send` → 本地 `delete_send` → emit

### 5.4 ListSendsUseCase

**文件：** `src-tauri/src/application/use_cases/list_sends_use_case.rs`（新建）

- [x] 注入 `VaultRepositoryPort`
- [x] `execute(account_id, user_key) -> AppResult<Vec<SyncSend>>`
- [x] 流程：`vault_repository.list_live_sends` → 逐条 `decrypt_send` → 返回

### 5.5 注册 use cases 模块

**文件：** `src-tauri/src/application/use_cases/mod.rs`

- [x] 添加 `pub mod create_send_use_case;`
- [x] 添加 `pub mod update_send_use_case;`
- [x] 添加 `pub mod delete_send_use_case;`
- [x] 添加 `pub mod list_sends_use_case;`

### 5.6 扩展 SyncEventPort

**文件：** `src-tauri/src/application/ports/sync_event_port.rs`

- [x] 添加 `fn emit_send_created(&self, account_id: &str, send_id: &str);`
- [x] 添加 `fn emit_send_updated(&self, account_id: &str, send_id: &str);`
- [x] 添加 `fn emit_send_deleted(&self, account_id: &str, send_id: &str);`

### 5.7 实现 Tauri 事件

**文件：** `src-tauri/src/interfaces/tauri/events/send.rs`（新建）

- [x] 定义 `SendCreated`、`SendUpdated`、`SendDeleted` 事件结构体（参考 `events/cipher.rs`）

**文件：** `src-tauri/src/interfaces/tauri/events/mod.rs`

- [x] 添加 `pub mod send;`

**文件：** `src-tauri/src/interfaces/tauri/events/sync_event_adapter.rs`

- [x] 实现 `emit_send_created`、`emit_send_updated`、`emit_send_deleted`（参考 cipher 事件实现）

**文件：** `src-tauri/src/lib.rs`

- [x] 在 `collect_events!` 中注册 `SendCreated`、`SendUpdated`、`SendDeleted`

### 验收标准

- [x] `cargo test` — 通过
- [x] 4 个 use case 文件结构与现有 cipher use cases 一致
- [x] SyncEventPort 的 3 个新方法在 adapter 中有实现

---

## Task 6: Tauri Commands + AppState 布线

**依赖：** Task 5
**模块：** 接口层（interfaces/tauri）+ 启动布线（bootstrap）

### 6.1 Send DTOs

**文件：** `src-tauri/src/interfaces/tauri/dto/send.rs`（新建）

- [x] `SendItemDto` — 列表项（id, type, name, disabled, expiration_date, deletion_date, access_count, max_access_count, has_password, revision_date）
- [x] `SendDetailDto` — 详情（SendItemDto 全部字段 + notes, text, file, hide_email, access_id, key, emails, auth_type）
- [x] `CreateSendRequestDto` — 创建请求（send: SyncSend, file_data: Option<Vec<u8>>）
- [x] `UpdateSendRequestDto` — 更新请求（send_id, send: SyncSend）
- [x] `DeleteSendRequestDto` — 删除请求（send_id）
- [x] `SendMutationResponseDto` — 变更响应（send_id, revision_date）

**文件：** `src-tauri/src/interfaces/tauri/dto/mod.rs`

- [x] 添加 `pub mod send;`

### 6.2 Tauri Commands

**文件：** `src-tauri/src/interfaces/tauri/commands/send.rs`（新建）

- [x] `list_sends(state) -> Result<Vec<SendItemDto>, ErrorPayload>`
  - 获取 account_id + user_key → 调用 ListSendsUseCase → 映射为 SendItemDto
- [x] `create_send(state, request: CreateSendRequestDto) -> Result<SendMutationResponseDto, ErrorPayload>`
  - 获取 session 信息 + user_key → 调用 CreateSendUseCase
- [x] `update_send(state, request: UpdateSendRequestDto) -> Result<SendMutationResponseDto, ErrorPayload>`
  - 获取 session 信息 + user_key → 调用 UpdateSendUseCase
- [x] `delete_send(state, request: DeleteSendRequestDto) -> Result<(), ErrorPayload>`
  - 获取 session 信息 → 调用 DeleteSendUseCase

**文件：** `src-tauri/src/interfaces/tauri/commands/mod.rs`

- [x] 添加 `pub mod send;`

### 6.3 AppState 布线

**文件：** `src-tauri/src/bootstrap/app_state.rs`

- [x] 添加字段：`create_send_use_case`, `update_send_use_case`, `delete_send_use_case`, `list_sends_use_case`（均为 `Arc<...>`）
- [x] 添加对应 getter 方法
- [x] 更新 `new()` 构造函数参数

**文件：** `src-tauri/src/bootstrap/wiring.rs`

- [x] 构建 4 个 send use case 实例
- [x] 传入 `AppState::new()`

### 6.4 注册 Commands

**文件：** `src-tauri/src/lib.rs`

- [x] 在 `collect_commands!` 中添加 `list_sends`, `create_send`, `update_send`, `delete_send`

### 验收标准

- [x] `cargo build` — 成功编译
- [x] `cargo clippy --all-targets --all-features -- -D warnings` — 无警告
- [x] `src/bindings.ts` 自动生成，包含 `listSends`, `createSend`, `updateSend`, `deleteSend` 命令类型
- [x] `src/bindings.ts` 包含 `SendCreated`, `SendUpdated`, `SendDeleted` 事件类型

---

## Task 7: 前端 Send 列表 + 侧边栏集成

**依赖：** Task 6
**模块：** 前端（src/features/send + vault-page 集成）

### 7.1 创建 feature 目录结构

- [ ] `src/features/send/constants.ts` — `SEND_ID = "__send__"`
- [ ] `src/features/send/types.ts` — `SendTypeFilter = "all" | "text" | "file"`
- [ ] `src/features/send/utils.ts` — `isSendExpired(send)`, `formatSendSubtitle(send, t)` 工具函数
- [ ] `src/features/send/index.ts` — 导出

### 7.2 Hooks

**文件：** `src/features/send/hooks/use-send-list.ts`（新建）

- [ ] 调用 `commands.listSends()` 获取列表
- [ ] 管理 `sendTypeFilter` 状态（all/text/file）
- [ ] 管理 `searchQuery` 状态
- [ ] 返回 `filteredSends`, `sendCount`, `isLoading`, `reload`

**文件：** `src/features/send/hooks/use-send-events.ts`（新建）

- [ ] 监听 `events.sendCreated` / `sendUpdated` / `sendDeleted`
- [ ] 触发列表刷新回调

**文件：** `src/features/send/hooks/index.ts`

- [ ] 导出 hooks

### 7.3 列表组件

**文件：** `src/features/send/components/send-row.tsx`（新建）

- [ ] Props：`send: SendItemDto`, `selected: boolean`, `onClick`
- [ ] 左侧图标：type=0 → `FileText`，type=1 → `Paperclip`（9x9 圆角方块，参考 CipherIcon 样式）
- [ ] 名称行：`text-sm font-semibold`
- [ ] 副标题行：过期时间 / 访问次数（`text-xs text-slate-500`）
- [ ] 状态 Badge：disabled → 灰色，expired → 红色
- [ ] 选中样式：`bg-blue-50 border-blue-200 text-blue-900 shadow-sm`

**文件：** `src/features/send/components/send-list-panel.tsx`（新建）

- [ ] 顶部工具栏：类型筛选（DropdownMenuRadioGroup）+ 搜索按钮 + 新建按钮（Plus 图标）
- [ ] ScrollArea 包裹列表
- [ ] 空状态：Send 图标 + 提示文字 + 创建按钮
- [ ] Props：`onSelectSend`, `onCreateSend`, `selectedSendId`

**文件：** `src/features/send/components/index.ts`

- [ ] 导出组件

### 7.4 侧边栏集成

**文件：** `src/features/vault/constants.ts`

- [ ] 添加 `SEND_ID = "__send__"`（或从 send/constants 导入）

**文件：** `src/features/vault/components/vault-page.tsx`

- [ ] 侧边栏：在 Trash 按钮下方添加 Send 菜单按钮
  - `Send` 图标（lucide-react）
  - 显示 sendCount
  - 选中样式与其他菜单项一致
- [ ] 中间面板：条件渲染 `selectedMenuId === SEND_ID ? <SendListPanel /> : 现有列表`
- [ ] 添加 `selectedSendId` 状态
- [ ] 切换到非 SEND_ID 菜单时清空 `selectedSendId`

### 7.5 i18n

**文件：** `src/i18n/resources/en.ts` + `src/i18n/resources/zh.ts`

- [ ] `vault.page.menus.send` — "Send" / "Send"
- [ ] `send.list.empty.title` — "No sends yet" / "暂无 Send"
- [ ] `send.list.empty.description` — "Create a send to share text or files securely" / "创建 Send 安全分享文本或文件"
- [ ] `send.list.empty.action` — "Create Send" / "创建 Send"
- [ ] `send.types.all` — "All types" / "所有类型"
- [ ] `send.types.text` — "Text" / "文本"
- [ ] `send.types.file` — "File" / "文件"

### 验收标准

- [ ] `pnpm build` — 通过
- [ ] 侧边栏显示 Send 菜单项，点击后中间面板切换为 Send 列表
- [ ] 列表正确显示已同步的 Send 数据
- [ ] 类型筛选和搜索功能正常
- [ ] 空状态正确显示

---

## Task 8: 前端 Send 详情面板

**依赖：** Task 7
**模块：** 前端（src/features/send/components）

### 8.1 详情面板组件

**文件：** `src/features/send/components/send-detail-panel.tsx`（新建）

- [ ] Props：`sendId: string | null`, `sends: SendItemDto[]`, `onEdit`, `onDelete`, `baseUrl`
- [ ] 未选中状态：居中提示 "Select a send to view details"
- [ ] 标题区域：Send 名称（`text-lg font-bold`）+ 类型 Badge（Text/File）+ 编辑按钮（Edit2 图标）+ 删除按钮（Trash2 图标）
- [ ] 内容区域（参考 CipherDetailPanel 的 label + value 行样式）：
  - Text Send：文本内容区域（hidden 时默认 `••••••••`，点击 Eye 图标切换显示）
  - File Send：文件名 + 文件大小
  - 备注
- [ ] Send 链接区域：链接文本 + 复制按钮（Copy 图标）
- [ ] 详情区域：密码保护状态、最大访问次数、当前访问次数、隐藏邮箱、禁用状态
- [ ] 时间区域：过期时间、删除时间、最后更新时间

### 8.2 集成到 vault-page

**文件：** `src/features/vault/components/vault-page.tsx`

- [ ] 右侧面板条件渲染：`selectedMenuId === SEND_ID ? <SendDetailPanel /> : <CipherDetailPanel />`

### 8.3 工具函数

**文件：** `src/features/send/utils.ts`

- [ ] `generateSendLink(baseUrl, accessId, key)` — 生成 Send 分享链接
- [ ] `isSendExpired(send)` — 判断是否已过期
- [ ] `formatSendDate(dateStr, t)` — 格式化日期显示

### 8.4 i18n

**文件：** `src/i18n/resources/en.ts` + `src/i18n/resources/zh.ts`

- [ ] `send.detail.selectPrompt` — "Select a send to view details"
- [ ] `send.detail.textContent` — "Text Content"
- [ ] `send.detail.fileInfo` — "File"
- [ ] `send.detail.notes` — "Notes"
- [ ] `send.detail.sendLink` — "Send Link"
- [ ] `send.detail.details` — "Details"
- [ ] `send.detail.password` — "Password"
- [ ] `send.detail.passwordProtected` — "Protected"
- [ ] `send.detail.passwordNone` — "None"
- [ ] `send.detail.maxViews` — "Max views"
- [ ] `send.detail.currentViews` — "Current views"
- [ ] `send.detail.hideEmail` — "Hide email"
- [ ] `send.detail.disabled` — "Disabled"
- [ ] `send.detail.dates` — "Dates"
- [ ] `send.detail.expiration` — "Expiration"
- [ ] `send.detail.deletion` — "Deletion"
- [ ] `send.detail.lastUpdated` — "Last updated"
- [ ] `send.detail.noExpiration` — "No expiration"
- [ ] `send.detail.linkCopied` — "Send link copied"

### 验收标准

- [ ] `pnpm build` — 通过
- [ ] 选中 Send 后右侧面板显示完整详情
- [ ] Text Send 的 hidden 文本可以切换显示/隐藏
- [ ] 复制链接功能正常
- [ ] 未选中时显示提示文字

---

## Task 9: 前端 Send 创建/编辑表单

**依赖：** Task 7
**模块：** 前端（src/features/send/components + hooks）

### 9.1 Mutation Hook

**文件：** `src/features/send/hooks/use-send-mutations.ts`（新建）

- [ ] `createSend` mutation — 调用 `commands.createSend()`
- [ ] `updateSend` mutation — 调用 `commands.updateSend()`
- [ ] `deleteSend` mutation — 调用 `commands.deleteSend()`
- [ ] 每个 mutation 返回 `{ mutate, mutateAsync, isLoading, error }`
- [ ] 参考 `use-cipher-mutations.ts` 的模式

### 9.2 表单 Dialog 组件

**文件：** `src/features/send/components/send-form-dialog.tsx`（新建）

- [ ] Props：`open`, `mode: "create" | "edit"`, `initialSend?`, `onOpenChange`, `onConfirm`, `isLoading`
- [ ] 使用 `@tanstack/react-form`
- [ ] 表单字段：
  - 类型选择（Select：Text=0 / File=1）— 创建时可选，编辑时 disabled
  - 名称（TextInput，必填）
  - Text 专属区域（type=0 时显示）：文本内容（Textarea）+ 隐藏文本（Switch）
  - File 专属区域（type=1 时显示）：文件选择（input[type=file]）+ 已选文件信息
  - 备注（Textarea）
  - 高级选项（Collapsible，默认折叠）：
    - 密码（TextInput type=password）
    - 最大访问次数（TextInput type=number）
    - 过期时间（TextInput type=datetime-local）
    - 删除时间（TextInput type=datetime-local，必填，默认 7 天后）
    - 隐藏邮箱（Switch）
    - 禁用（Switch）
- [ ] 底部按钮：Cancel + Create Send / Save（与 CipherFormDialog 一致）
- [ ] 编辑模式：从 initialSend 填充表单默认值

### 9.3 集成到 vault-page

**文件：** `src/features/vault/components/vault-page.tsx`

- [ ] 添加 Send 表单状态：`isSendFormOpen`, `sendFormMode`, `selectedSendForEdit`
- [ ] `handleCreateSend()` — 打开创建表单
- [ ] `handleEditSend(send)` — 打开编辑表单
- [ ] `handleSendFormConfirm(send)` — 调用 mutation + toast 反馈 + 关闭 dialog
- [ ] 渲染 `<SendFormDialog />`

### 9.4 i18n

**文件：** `src/i18n/resources/en.ts` + `src/i18n/resources/zh.ts`

- [ ] `send.form.createTitle` — "Create Send"
- [ ] `send.form.editTitle` — "Edit Send"
- [ ] `send.form.createDescription` — "Create a new send to share securely"
- [ ] `send.form.editDescription` — "Edit send details"
- [ ] `send.form.type` — "Type"
- [ ] `send.form.name` — "Name"
- [ ] `send.form.textContent` — "Text Content"
- [ ] `send.form.hideText` — "Hide text by default"
- [ ] `send.form.file` — "File"
- [ ] `send.form.chooseFile` — "Choose file..."
- [ ] `send.form.notes` — "Notes"
- [ ] `send.form.advanced` — "Advanced Options"
- [ ] `send.form.password` — "Password"
- [ ] `send.form.maxAccessCount` — "Max access count"
- [ ] `send.form.expirationDate` — "Expiration date"
- [ ] `send.form.deletionDate` — "Deletion date"
- [ ] `send.form.hideEmail` — "Hide my email"
- [ ] `send.form.disable` — "Disable this send"
- [ ] `send.form.submit.create` — "Create Send"
- [ ] `send.form.submit.save` — "Save"
- [ ] `send.feedback.createSuccess` — "Send created"
- [ ] `send.feedback.createError` — "Failed to create send"
- [ ] `send.feedback.saveSuccess` — "Send saved"
- [ ] `send.feedback.saveError` — "Failed to save send"

### 验收标准

- [ ] `pnpm build` — 通过
- [ ] 点击新建按钮打开创建表单，类型可切换
- [ ] Text Send：可输入文本内容，可切换隐藏
- [ ] File Send：可选择文件
- [ ] 高级选项折叠/展开正常
- [ ] 编辑模式正确填充已有数据
- [ ] 提交后 toast 反馈正确

---

## Task 10: 前端 Send 删除 + 完整集成 + Pre-commit

**依赖：** Task 8, 9
**模块：** 前端（完整集成）

### 10.1 删除确认对话框

**文件：** `src/features/send/components/delete-send-dialog.tsx`（新建）

- [ ] Props：`open`, `sendName`, `onOpenChange`, `onConfirm`, `isLoading`
- [ ] 完全参考 `delete-cipher-dialog.tsx` 的样式
- [ ] 红色警告图标 + 确认文案 + Cancel / Delete 按钮

### 10.2 集成删除到 vault-page

**文件：** `src/features/vault/components/vault-page.tsx`

- [ ] 添加删除状态：`isDeleteSendDialogOpen`, `selectedSendIdForDelete`, `selectedSendNameForDelete`
- [ ] `handleDeleteSend(sendId, sendName)` — 打开删除确认
- [ ] `handleDeleteSendConfirm()` — 调用 deleteSend mutation + toast + 关闭
- [ ] 渲染 `<DeleteSendDialog />`

### 10.3 列表右键菜单

**文件：** `src/features/send/components/send-list-panel.tsx`

- [ ] 为每个 SendRow 包裹 ContextMenu
- [ ] 菜单项：编辑（Edit2 图标）、复制链接（Copy 图标）、删除（Trash2 图标，destructive）

### 10.4 详情面板操作按钮联动

**文件：** `src/features/send/components/send-detail-panel.tsx`

- [ ] 编辑按钮 → 调用 `onEdit(send)`
- [ ] 删除按钮 → 调用 `onDelete(sendId, sendName)`

### 10.5 事件驱动刷新

**文件：** `src/features/vault/components/vault-page.tsx`

- [ ] 使用 `useSendEvents` 监听 Send 变更事件
- [ ] 事件触发时自动刷新 Send 列表
- [ ] 删除当前选中的 Send 后清空 `selectedSendId`

### 10.6 i18n 补全

**文件：** `src/i18n/resources/en.ts` + `src/i18n/resources/zh.ts`

- [ ] `send.dialogs.delete.title` — "Delete Send"
- [ ] `send.dialogs.delete.descriptionPrefix` — "Are you sure you want to delete"
- [ ] `send.dialogs.delete.descriptionSuffix` — "? This action cannot be undone."
- [ ] `send.dialogs.delete.deleting` — "Deleting..."
- [ ] `send.feedback.deleteSuccess` — "Send deleted"
- [ ] `send.feedback.deleteError` — "Failed to delete send"
- [ ] `send.contextMenu.edit` — "Edit"
- [ ] `send.contextMenu.copyLink` — "Copy link"
- [ ] `send.contextMenu.delete` — "Delete"

### 10.7 WebSocket 增量同步协调

- [ ] 确认现有 `SyncSendCreate/Update/Delete` push event 处理与新的 CRUD 操作不冲突
- [ ] 手动 CRUD 后本地数据已更新，WebSocket 增量同步到达时 upsert 幂等

### 10.8 Pre-commit 全量验证

**Rust：**
- [ ] `cargo test`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `cargo fmt`

**Frontend：**
- [ ] `pnpm run biome:write`
- [ ] `pnpm run biome:format`
- [ ] `pnpm build`

### 验收标准

- [ ] 全栈 pre-commit checks 全部通过
- [ ] 完整用户流程可用：
  1. 侧边栏点击 Send → 显示列表
  2. 点击 + 创建 Text Send → 列表刷新显示新 Send
  3. 选中 Send → 右侧显示详情
  4. 复制链接 → 剪贴板包含正确链接
  5. 编辑 Send → 修改保存成功
  6. 删除 Send → 确认后从列表移除
  7. 创建 File Send → 选择文件 → 上传成功
  8. WebSocket 推送 SyncSendUpdate → 列表自动刷新
