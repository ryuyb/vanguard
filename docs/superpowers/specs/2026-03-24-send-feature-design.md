# Send Feature Design

**Date**: 2026-03-24
**Author**: Vanguard Team
**Status**: Design Phase

## Overview

本文档描述了 Vanguard 集成 Vaultwarden Send 功能的完整设计方案，支持文本和文件两种类型的 Send，包含完整的 CRUD 操作、加密机制、访问权限控制以及前端 UI 集成。

## Goals

- 支持完整的 Send 功能：创建、查看、编辑、删除
- 支持文本和文件两种 Send 类型
- 支持完整的访问权限控制（密码、访问次数、过期时间、删除时间）
- 遵循现有 DDD 架构模式
- 提供用户友好的 UI 体验

## Non-Goals

- Send 的公开访问页面（这是 Vaultwarden Web 的功能）
- Send 的批量操作
- Send 的导入导出功能

## Architecture

### 整体架构

遵循现有的 DDD 四层架构：

```
Domain Layer (领域层)
    ↓
Application Layer (应用层)
    ↓
Infrastructure Layer (基础设施层)
    ↓
Interfaces Layer (接口层)
```

### 模块划分

**后端：**

```
src-tauri/src/
├── domain/send/                    # Send 领域模型
│   ├── mod.rs
│   ├── send.rs                     # Send 聚合根
│   ├── types.rs                    # SendType、SendAccess 等
│   └── state.rs                    # Encrypted/Decrypted 状态
│
├── application/
│   ├── dto/send.rs                 # Send DTO
│   ├── use_cases/                  # Send Use Cases
│   │   ├── create_send_use_case.rs
│   │   ├── update_send_use_case.rs
│   │   ├── delete_send_use_case.rs
│   │   ├── list_sends_use_case.rs
│   │   └── get_send_detail_use_case.rs
│   ├── ports/
│   │   └── send_repository_port.rs # Send 存储接口
│   └── policy/
│       └── send_policy.rs          # Send 策略（文件大小限制等）
│
├── infrastructure/
│   ├── vaultwarden/
│   │   └── send_adapter.rs         # Vaultwarden Send API
│   └── persistence/
│       └── sqlite_send_repository.rs # Send 本地缓存
│
└── interfaces/tauri/
    ├── commands/
    │   └── send.rs                 # Tauri 命令
    └── dto/
        └── send.rs                 # 前端 DTO
```

**前端：**

```
src/features/send/
├── components/
│   ├── send-list.tsx               # Send 列表
│   ├── send-detail-panel.tsx       # Send 详情面板
│   ├── send-form-dialog.tsx        # 创建/编辑表单
│   └── send-access-config.tsx      # 访问权限配置
├── hooks/
│   ├── use-send-list.ts            # 列表数据
│   ├── use-send-detail.ts          # 详情数据
│   ├── use-send-mutations.ts       # CRUD 操作
│   └── use-send-file-upload.ts     # 文件上传
├── schema.ts                       # 表单验证 Schema
├── types.ts                        # TypeScript 类型
├── utils.ts                        # 工具函数
└── index.ts
```

## Domain Layer Design

### Send 聚合根

采用类型状态模式支持加密/解密状态：

```rust
pub struct Send<S: SendState> {
    pub id: String,
    pub r#type: SendType,

    // 基本信息（需要加密）
    pub name: EncryptedField<S, String>,
    pub notes: EncryptedField<S, String>,

    // 类型特定数据
    pub text: Option<SendText<S>>,
    pub file: Option<SendFile<S>>,

    // 访问控制
    pub key: Option<String>,              // Send 加密密钥
    pub password: Option<String>,         // 访问密码
    pub max_access_count: Option<i32>,    // 最大访问次数
    pub access_count: i32,                // 当前访问次数
    pub expiration_date: Option<String>,  // 过期时间
    pub deletion_date: String,            // 删除时间

    // 元数据
    pub hide_email: bool,
    pub disabled: bool,
    pub revision_date: String,
}

pub enum SendType {
    Text = 0,
    File = 1,
}

pub struct SendText<S: SendState> {
    pub text: EncryptedField<S, String>,
    pub hidden: bool,
}

pub struct SendFile<S: SendState> {
    pub id: String,
    pub file_name: EncryptedField<S, String>,
    pub key: Option<String>,
    pub size: Option<String>,
    pub size_name: Option<String>,
}
```

### 加密机制

1. **Send Key 生成**：每个 Send 创建时生成独立的随机密钥
2. **内容加密**：name、notes、text、file_name 等敏感字段用 Send Key 加密
3. **密钥保护**：Send Key 用用户的 User Key 加密后存储在 `key` 字段
4. **文件加密**：文件内容也用 Send Key 加密

## Application Layer Design

### Use Cases

#### CreateSendUseCase

**职责**：创建新的 Send

**流程**：
1. 验证数据（文件大小、必填字段等）
2. 生成 Send Key
3. 加密 Send 内容
4. 调用 API 创建 Send
5. 保存到本地缓存

**文件上传**：
- 两步流程：先创建 Send 记录，再上传文件到服务器返回的 URL
- 文件大小限制：前端和后端双重验证（默认 100MB）

#### UpdateSendUseCase

**职责**：更新现有 Send

**流程**：
1. 验证权限和状态
2. 加密更新的内容
3. 调用 API 更新
4. 更新本地缓存

**限制**：
- 文件类型不支持更换文件（只能修改元数据）
- Send 类型创建后不可更改

#### DeleteSendUseCase

**职责**：删除 Send

**流程**：
1. 验证权限
2. 调用 API 删除（服务器会自动删除关联文件）
3. 删除本地缓存

#### ListSendsUseCase

**职责**：获取所有 Send 列表

**流程**：
1. 从本地缓存获取
2. 如果缓存为空或过期，触发同步

#### GetSendDetailUseCase

**职责**：获取单个 Send 详情

**流程**：
1. 从服务器获取最新数据（包括 access_count）
2. 更新本地缓存的 access_count
3. 解密 Send 内容
4. 返回解密后的数据

### DTO

```rust
pub struct CreateSendCommand {
    pub r#type: SendType,
    pub name: String,
    pub notes: Option<String>,
    pub text: Option<CreateSendText>,
    pub file: Option<CreateSendFile>,
    pub password: Option<String>,
    pub max_access_count: Option<i32>,
    pub expiration_date: Option<String>,
    pub deletion_date: String,
    pub hide_email: bool,
    pub disabled: bool,
}

pub struct CreateSendText {
    pub text: String,
    pub hidden: bool,
}

pub struct CreateSendFile {
    pub file_name: String,
    pub file_size: u64,
    pub file_data: Vec<u8>,
}
```

### Policy

```rust
pub struct SendPolicy;

impl SendPolicy {
    pub const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100MB

    pub fn validate_file_size(size: u64) -> Result<(), AppError> {
        if size > Self::MAX_FILE_SIZE {
            return Err(AppError::ValidationError {
                message: format!("File size exceeds maximum allowed ({} MB)",
                    Self::MAX_FILE_SIZE / 1024 / 1024),
            });
        }
        Ok(())
    }
}
```

## Infrastructure Layer Design

### Vaultwarden API Integration

**Endpoints**：

```rust
impl VaultwardenEndpoints {
    pub fn sends(base_url: &str) -> String {
        format!("{}/api/sends", normalize_base(base_url))
    }

    pub fn send(base_url: &str, send_id: &str) -> String {
        format!("{}/api/sends/{}", normalize_base(base_url), send_id)
    }

    pub fn send_file(base_url: &str, send_id: &str) -> String {
        format!("{}/api/sends/{}/file", normalize_base(base_url), send_id)
    }
}
```

**API Models**：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncSend {
    pub id: String,
    pub r#type: Option<i32>,
    pub name: Option<String>,
    pub notes: Option<String>,
    pub key: Option<String>,
    pub password: Option<String>,
    pub max_access_count: Option<i32>,
    pub access_count: Option<i32>,
    pub expiration_date: Option<String>,
    pub deletion_date: Option<String>,
    pub hide_email: Option<bool>,
    pub disabled: Option<bool>,
    pub revision_date: Option<String>,
    pub text: Option<SyncSendText>,
    pub file: Option<SyncSendFile>,
    pub object: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSendRequest {
    pub r#type: i32,
    pub name: String,
    pub notes: Option<String>,
    pub key: String,
    pub password: Option<String>,
    pub max_access_count: Option<i32>,
    pub expiration_date: Option<String>,
    pub deletion_date: String,
    pub hide_email: bool,
    pub disabled: bool,
    pub text: Option<CreateSendTextRequest>,
    pub file: Option<CreateSendFileRequest>,
}
```

### Local Cache (SQLite)

**Database Schema**：

```sql
CREATE TABLE sends (
    id TEXT PRIMARY KEY NOT NULL,
    account_id TEXT NOT NULL,
    type INTEGER NOT NULL,
    name TEXT NOT NULL,
    notes TEXT,
    key TEXT,
    password TEXT,
    max_access_count INTEGER,
    access_count INTEGER NOT NULL DEFAULT 0,
    expiration_date TEXT,
    deletion_date TEXT NOT NULL,
    hide_email INTEGER NOT NULL DEFAULT 0,
    disabled INTEGER NOT NULL DEFAULT 0,
    revision_date TEXT NOT NULL,
    text TEXT,  -- JSON: SendText
    file TEXT,  -- JSON: SendFile
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
);

CREATE INDEX idx_sends_account_id ON sends(account_id);
CREATE INDEX idx_sends_deletion_date ON sends(deletion_date);
CREATE INDEX idx_sends_type ON sends(type);
```

**Repository Implementation**：

```rust
pub struct SqliteSendRepository {
    pool: SqlitePool,
}

impl SendRepositoryPort for SqliteSendRepository {
    async fn list_sends(&self, account_id: &str) -> Result<Vec<Send<Encrypted>>, AppError>;
    async fn save_send(&self, account_id: &str, send: &Send<Encrypted>) -> Result<(), AppError>;
    async fn delete_send(&self, account_id: &str, send_id: &str) -> Result<(), AppError>;
    async fn update_access_count(&self, account_id: &str, send_id: &str, count: i32) -> Result<(), AppError>;
    async fn clear_expired_sends(&self, account_id: &str) -> Result<u64, AppError>;
}
```

### Sync Integration

将 Send 同步集成到现有的 `SyncVaultUseCase`：

```rust
impl SyncVaultUseCase {
    pub async fn execute(&self, account_id: &str) -> Result<SyncResult, AppError> {
        // ... 现有的 cipher、folder 同步 ...

        // 同步 Sends
        let sends = remote_vault.list_sends(&base_url, &access_token).await?;

        // 清理过期的 Send
        self.send_repo.clear_expired_sends(account_id).await?;

        // 保存新的 Send 数据
        for send in sends {
            self.send_repo.save_send(account_id, &send).await?;
        }

        Ok(SyncResult { /* ... */ })
    }
}
```

### Cache Strategy

**缓存内容**：
- Send 的所有加密数据
- 基本元数据（过期时间、删除时间等）
- 访问次数（需要实时更新）

**缓存更新时机**：
1. **主动同步** - 用户手动同步或自动定时同步
2. **创建/更新/删除操作后** - 操作成功后更新本地缓存
3. **查看详情时** - 更新 access_count

**缓存清理**：
- 定期清理已过期的 Send（deletion_date < now）
- 用户登出时清空该账户的所有 Send 缓存

**离线策略**：
- 离线时可以查看缓存列表和详情
- 创建/更新/删除操作需要在线
- 提示用户"离线模式，数据可能不是最新"

## Interfaces Layer Design

### Tauri Commands

```rust
#[tauri::command]
#[specta::specta]
pub async fn list_sends(state: State<'_, AppState>) -> Result<Vec<SendItemDto>, ErrorPayload>;

#[tauri::command]
#[specta::specta]
pub async fn get_send_detail(
    state: State<'_, AppState>,
    request: GetSendDetailRequestDto,
) -> Result<SendDetailResponseDto, ErrorPayload>;

#[tauri::command]
#[specta::specta]
pub async fn create_send(
    request: CreateSendRequestDto,
    state: State<'_, AppState>,
) -> Result<SendMutationResponseDto, ErrorPayload>;

#[tauri::command]
#[specta::specta]
pub async fn update_send(
    request: UpdateSendRequestDto,
    state: State<'_, AppState>,
) -> Result<SendMutationResponseDto, ErrorPayload>;

#[tauri::command]
#[specta::specta]
pub async fn delete_send(
    request: DeleteSendRequestDto,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload>;

#[tauri::command]
#[specta::specta]
pub async fn download_send_file(
    request: DownloadSendFileRequestDto,
    state: State<'_, AppState>,
) -> Result<Vec<u8>, ErrorPayload>;
```

### Frontend DTO

```typescript
interface SendItemDto {
  id: string;
  type: SendTypeDto;
  name: string;
  accessCount: number;
  maxAccessCount?: number;
  expirationDate?: string;
  deletionDate: string;
  disabled: boolean;
  revisionDate: string;
}

interface SendDetailDto {
  id: string;
  type: SendTypeDto;
  name: string;
  notes: string;
  password?: string;
  maxAccessCount?: number;
  accessCount: number;
  expirationDate?: string;
  deletionDate: string;
  hideEmail: boolean;
  disabled: boolean;
  text?: SendTextDto;
  file?: SendFileDto;
}

enum SendTypeDto {
  Text = 0,
  File = 1,
}
```

## Frontend UI Design

### Page Layout

**侧边栏底部标签页切换**：

```
┌─────────────────────────────────────────────┐
│  [Search Bar]                    [Settings] │
├──────────────┬──────────────────────────────┤
│              │                              │
│  📁 Folder 1 │                              │
│  📁 Folder 2 │      Main Content Area       │
│  📁 Folder 3 │   (Cipher/Send Details)      │
│              │                              │
│              │                              │
│ ─────────────│                              │
│ [Vault][Send]│                              │
└──────────────┴──────────────────────────────┘
```

**切换行为**：
- **Vault 模式**：显示文件夹列表，内容区显示 Cipher
- **Send 模式**：侧边栏显示 Send 列表（分组：文本/文件），顶部有"新建 Send"按钮，内容区显示 Send 详情/编辑面板

### Send List Component

```tsx
export function SendList() {
  const { sends, isLoading } = useSendList();

  return (
    <div className="p-2">
      {/* 创建按钮 */}
      <Button className="w-full mb-2">
        <Plus /> {t('send.create')}
      </Button>

      {/* 分组显示 */}
      <ScrollArea>
        {/* 文本类型 */}
        <div className="mb-4">
          <div className="text-xs font-semibold text-muted-foreground">
            {t('send.type.text')}
          </div>
          {sends.filter(s => s.type === 'text').map(send => (
            <SendListItem key={send.id} send={send} />
          ))}
        </div>

        {/* 文件类型 */}
        <div>
          <div className="text-xs font-semibold text-muted-foreground">
            {t('send.type.file')}
          </div>
          {sends.filter(s => s.type === 'file').map(send => (
            <SendListItem key={send.id} send={send} />
          ))}
        </div>
      </ScrollArea>
    </div>
  );
}
```

### Send Detail Panel

**功能**：
- 显示 Send 名称、备注
- 显示内容（文本或文件信息）
- 复制 Send 链接
- 复制文本内容
- 下载文件
- 编辑/删除操作
- 显示访问控制信息

### Send Form Dialog

**表单字段**：

1. **基本信息**：
   - 类型选择（文本/文件）
   - 名称
   - 备注

2. **内容**：
   - 文本类型：文本内容、是否隐藏
   - 文件类型：文件选择器

3. **访问权限配置**：
   - 访问密码（可选）
   - 访问次数限制（可选）
   - 过期时间（可选）
   - 删除时间（必填，默认7天）

4. **其他选项**：
   - 隐藏邮箱
   - 禁用 Send

**验证规则**：
- 名称：必填，1-100 字符
- 文本内容：必填，最大 10000 字符
- 文件大小：最大 100MB
- 访问密码：最小 4 字符
- 访问次数：1-100
- 删除时间：必填，必须大于当前时间

## Frontend Hooks Design

### Data Fetching

```tsx
export function useSendList() {
  return useQuery({
    queryKey: ['sends'],
    queryFn: async () => {
      const result = await commands.listSends();
      if (result.status === 'error') throw new Error(result.error);
      return result.data;
    },
    staleTime: 30000, // 30秒
  });
}

export function useSendDetail(sendId: string) {
  return useQuery({
    queryKey: ['send', sendId],
    queryFn: async () => {
      const result = await commands.getSendDetail({ sendId });
      if (result.status === 'error') throw new Error(result.error);
      return result.data;
    },
    enabled: !!sendId,
  });
}
```

### Mutations

```tsx
export function useSendMutations() {
  const queryClient = useQueryClient();

  const createSend = useMutation({
    mutationFn: async (data: CreateSendRequestDto) => {
      const result = await commands.createSend(data);
      if (result.status === 'error') throw new Error(result.error);
      return result.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['sends'] });
      toast.success(t('send.created'));
    },
  });

  const updateSend = useMutation({ /* ... */ });
  const deleteSend = useMutation({ /* ... */ });

  return { createSend, updateSend, deleteSend };
}
```

### File Upload

```tsx
export function useSendFileUpload() {
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [fileSize, setFileSize] = useState<number>(0);

  const selectFile = async (file: File) => {
    const maxSize = 100 * 1024 * 1024; // 100MB
    if (file.size > maxSize) {
      throw new Error(t('send.fileTooLarge', { max: '100MB' }));
    }
    setSelectedFile(file);
    setFileSize(file.size);
  };

  const readFileData = async (): Promise<number[]> => {
    if (!selectedFile) throw new Error('No file selected');
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => {
        const arrayBuffer = reader.result as ArrayBuffer;
        resolve(Array.from(new Uint8Array(arrayBuffer)));
      };
      reader.onerror = () => reject(new Error('Failed to read file'));
      reader.readAsArrayBuffer(selectedFile);
    });
  };

  return { selectedFile, fileSize, selectFile, readFileData };
}
```

## Security Considerations

1. **加密机制**：
   - 所有敏感数据在客户端加密
   - 每个 Send 有独立的加密密钥
   - 文件内容也经过加密

2. **访问控制**：
   - 密码使用哈希存储
   - 访问次数限制在服务器端验证
   - 过期时间和删除时间由服务器执行

3. **文件安全**：
   - 文件大小双重验证（前后端）
   - 文件内容加密后上传
   - 文件下载时需要解密

4. **权限验证**：
   - 所有修改操作需要 unlock 状态
   - 验证用户对 Send 的所有权

## Performance Considerations

1. **缓存策略**：
   - 本地 SQLite 缓存减少网络请求
   - 访问次数实时更新不触发完整同步

2. **文件处理**：
   - 大文件异步上传
   - 显示上传进度
   - 文件大小限制防止内存溢出

3. **UI 响应**：
   - 列表虚拟滚动（大量 Send）
   - 乐观更新（创建/删除）
   - 后台同步不阻塞 UI

## Testing Strategy

### Unit Tests

- Domain 层：Send 加密/解密逻辑
- Application 层：Use Case 业务逻辑
- Infrastructure 层：数据转换和存储

### Integration Tests

- API 集成：与 Vaultwarden 服务器交互
- 数据库集成：SQLite 存储和查询
- 加密流程：端到端加密验证

### E2E Tests

- 创建文本 Send
- 创建文件 Send
- 编辑 Send
- 删除 Send
- 访问权限验证

## Migration Plan

### Phase 1: 后端基础设施

1. 创建 Domain 层模型
2. 实现 Infrastructure 层（API + SQLite）
3. 实现 Application 层 Use Cases
4. 添加 Tauri Commands

### Phase 2: 前端 UI

1. 改造侧边栏支持标签切换
2. 实现 Send List 和 Detail Panel
3. 实现 Send Form Dialog
4. 添加 Hooks 和状态管理

### Phase 3: 集成测试

1. 端到端功能测试
2. 加密流程验证
3. 文件上传/下载测试
4. 性能测试

## Success Criteria

- ✅ 用户可以创建文本和文件类型的 Send
- ✅ 用户可以查看、编辑、删除 Send
- ✅ 访问权限控制正常工作
- ✅ 本地缓存提升加载速度
- ✅ 离线模式可用
- ✅ 与现有 Vault 功能无缝集成
- ✅ UI 简洁直观，符合 1Password 体验

## Future Enhancements

- Send 模板（快速创建常用 Send）
- Send 分享历史记录
- Send 批量操作
- Send 导入导出
- Send QR 码分享
- Send 使用统计可视化

## References

- [Vaultwarden Send API Documentation](https://github.com/dani-garcia/vaultwarden/wiki)
- [Bitwarden Send Specification](https://bitwarden.com/help/send/)
- Vanguard Cipher Implementation
- Vanguard Architecture Documentation