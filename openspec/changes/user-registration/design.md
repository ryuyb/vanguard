## Context

当前 Vanguard 只支持登录已有账户,缺少注册功能。Vaultwarden 服务器支持三种注册模式:
1. 禁止注册 (返回 400)
2. 需要邮箱验证 (返回 204)
3. 直接注册 (返回 200 + JWT token)

现有架构:
- 前端: React + TypeScript + shadcn/ui
- 后端: Rust + Tauri
- 认证流程: 已有登录和解锁功能
- 密码学: 目前只有解密功能,缺少完整的密钥派生和加密实现

技术约束:
- 必须完全兼容 Bitwarden 密码学规范
- 需要支持跨客户端互操作性
- 密钥材料不能以明文形式存储

## Goals / Non-Goals

**Goals:**
- 实现三种注册场景的完整支持
- 提供清晰的用户体验和错误提示
- 实现 Bitwarden 兼容的密码学处理
- 注册成功后自动登录
- 支持密码强度检查和泄露检查

**Non-Goals:**
- 不实现服务器端注册逻辑 (使用现有 Vaultwarden API)
- 不支持社交账号注册 (OAuth)
- 不实现邮箱验证的深度链接 (deep link) 处理
- 不修改现有登录流程

## Decisions

### 决策 1: 分阶段实现

**选择**: 先实现场景 1 和 2 (MVP),后实现场景 3 (增强功能)

**理由**:
- 场景 1 和 2 实现简单,可快速上线
- 大多数 Vaultwarden 服务器使用邮箱验证模式
- 场景 3 需要复杂的密码学实现,风险较高
- 分阶段可以更早获得用户反馈

**替代方案**:
- 一次性实现所有场景: 开发周期长,风险高
- 只实现场景 3: 不支持邮箱验证模式的服务器

### 决策 2: 密码学模块架构

**选择**: 在 Rust 后端实现密码学模块,前端通过 Tauri commands 调用

**理由**:
- Rust 有成熟的密码学库 (ring, rsa)
- 密钥材料不会暴露给前端 JavaScript 环境
- 更好的性能和安全性
- 便于单元测试

**替代方案**:
- 在前端实现密码学: 安全性较低,性能较差
- 使用 WebAssembly: 增加复杂度,收益不明显

### 决策 3: 密码泄露检查

**选择**: 使用 Have I Been Pwned API 的 k-anonymity 模型

**理由**:
- 行业标准做法
- 保护用户隐私 (只发送密码哈希的前 5 个字符)
- Bitwarden 官方客户端也使用此方案
- 免费且可靠

**替代方案**:
- 不检查密码泄露: 安全性降低
- 本地密码字典: 维护成本高,覆盖不全

### 决策 4: 邮箱验证场景的处理

**选择**: 显示提示信息,让用户在浏览器中完成注册,然后手动登录

**理由**:
- 实现简单,无需处理 deep link
- 不需要修改服务器邮件模板
- 用户体验可接受

**替代方案**:
- 实现 deep link (vanguard://): 需要修改服务器,不现实
- 手动输入验证码: 用户体验较差,需要服务器支持

### 决策 5: UI 路由设计

**选择**: 创建独立的 `/register` 路由,从登录页导航

**理由**:
- 清晰的页面分离
- 便于维护和测试
- 符合用户心智模型

**替代方案**:
- 在登录页内嵌注册表单: 页面过于复杂
- 使用 modal 弹窗: 不适合多步骤流程

### 决策 6: 密码学库选择

**选择**:
- PBKDF2/HKDF/AES/HMAC: `ring` crate
- RSA: `rsa` crate
- Base64: `base64` crate

**理由**:
- `ring` 是 Rust 生态中最成熟的密码学库
- 经过广泛审计,安全性高
- 性能优秀
- `rsa` crate 是 RSA 操作的标准选择

**替代方案**:
- `openssl`: 依赖 C 库,编译复杂
- `rust-crypto`: 已废弃

## Risks / Trade-offs

### 风险 1: 密码学实现错误
**影响**: 导致数据无法解密或跨客户端不兼容

**缓解措施**:
- 严格遵循 Bitwarden 密码学白皮书
- 参考 Bitwarden 官方客户端源码
- 编写详细的单元测试
- 与真实 Vaultwarden 服务器进行集成测试
- 测试跨客户端互操作性

### 风险 2: 服务器配置检测不准确
**影响**: 用户看到错误的注册流程

**缓解措施**:
- 严格按照 HTTP 状态码判断场景
- 提供清晰的错误提示
- 记录详细的日志便于调试

### 风险 3: 密码泄露检查 API 不可用
**影响**: 注册流程被阻塞

**缓解措施**:
- 设置合理的超时时间 (5 秒)
- API 失败时允许用户继续注册
- 显示警告但不阻止流程

### 风险 4: 场景 3 实现复杂度高
**影响**: 开发周期长,可能延期

**缓解措施**:
- 采用分阶段实现策略
- 先上线场景 1 和 2
- 场景 3 作为独立迭代

### Trade-off 1: 邮箱验证场景的用户体验
**权衡**: 用户需要在浏览器中完成注册,然后手动登录

**理由**: 实现 deep link 需要修改服务器,成本过高

**影响**: 用户体验略有下降,但可接受

### Trade-off 2: 密码学在后端实现
**权衡**: 前端无法直接访问密钥材料

**理由**: 安全性优先

**影响**: 前端需要通过 Tauri commands 调用,增加一层抽象

## Architecture

### 模块划分

```
src-tauri/src/
├── application/
│   ├── services/
│   │   └── registration_service.rs    # 注册业务逻辑
│   └── crypto/
│       ├── key_derivation.rs          # 密钥派生 (PBKDF2, HKDF)
│       ├── encryption.rs              # 加密/解密 (AES-256-CBC)
│       ├── cipher_string.rs           # CipherString 格式处理
│       └── rsa_keys.rs                # RSA 密钥对生成
├── infrastructure/
│   └── vaultwarden/
│       ├── endpoints.rs               # 添加注册 endpoints
│       └── registration_adapter.rs    # 注册 API 适配器
└── interfaces/
    └── tauri/
        └── commands/
            └── registration.rs        # 注册相关 Tauri commands

src/
├── features/
│   └── auth/
│       └── register/
│           ├── components/
│           │   ├── register-page.tsx           # 注册页面
│           │   ├── register-form.tsx           # 注册表单
│           │   ├── password-setup-form.tsx     # 密码设置表单
│           │   ├── password-strength.tsx       # 密码强度指示器
│           │   └── registration-feedback.tsx   # 反馈提示
│           ├── hooks/
│           │   ├── use-registration-flow.ts    # 注册流程逻辑
│           │   └── use-password-strength.ts    # 密码强度检查
│           ├── schema.ts                       # 表单验证 schema
│           └── types.ts                        # 类型定义
└── routes/
    └── register.tsx                            # 注册路由
```

### 数据流

#### 场景 1 和 2: 基础注册流程
```
用户输入 → 前端验证 → 调用 send-verification-email API
                    ↓
            根据状态码显示提示
```

#### 场景 3: 完整注册流程
```
用户输入邮箱 → send-verification-email API → 返回 JWT token
                                          ↓
用户设置密码 → 前端验证 → 密码泄露检查 (可选)
                      ↓
            调用 Tauri command: register_finish
                      ↓
            Rust 后端密码学处理:
            - PBKDF2: 派生 Master Key
            - Hash: 生成 masterPasswordHash
            - HKDF: 派生 Symmetric Key
            - RSA: 生成密钥对
            - AES: 加密私钥和对称密钥
                      ↓
            调用 register/finish API
                      ↓
            自动登录流程:
            - prelogin
            - connect/token
            - sync
                      ↓
            跳转到 Vault 页面
```

## Migration Plan

### 阶段 1: MVP (场景 1 和 2)

**部署步骤**:
1. 添加前端注册页面和路由
2. 实现基础 API 调用
3. 添加错误处理和提示
4. 测试三种场景的响应处理
5. 发布 MVP 版本

**回滚策略**:
- 移除 `/register` 路由
- 隐藏登录页的"创建新账户"链接

### 阶段 2: 完整功能 (场景 3)

**部署步骤**:
1. 实现密码学模块并编写单元测试
2. 实现注册完成逻辑
3. 添加密码强度和泄露检查
4. 与真实服务器进行集成测试
5. 测试跨客户端互操作性
6. 发布完整版本

**回滚策略**:
- 场景 3 失败时回退到显示"开发中"提示
- 不影响场景 1 和 2 的功能

## Open Questions

1. **密码强度要求**: 是否需要强制最低密码长度和复杂度?
   - 建议: 参考 Bitwarden 官方要求 (最少 8 个字符)

2. **密码泄露检查**: 是否强制要求密码未泄露?
   - 建议: 显示警告但允许用户继续

3. **注册成功后的引导**: 是否需要显示欢迎页面或新手引导?
   - 建议: 直接跳转到 Vault 页面,保持简洁

4. **错误重试**: 注册失败时是否允许用户重试?
   - 建议: 允许重试,但记录失败次数防止滥用
