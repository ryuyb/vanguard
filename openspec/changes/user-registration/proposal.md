## Why

当前 Vanguard 只支持登录已有账户,用户无法通过客户端注册新账户。这导致新用户必须先通过 Vaultwarden 网页端或其他客户端注册,然后才能使用 Vanguard,增加了使用门槛。添加注册功能可以提供完整的用户体验,让用户在首次使用时就能直接创建账户。

## What Changes

- 添加用户注册页面和导航入口
- 支持三种注册场景:
  - 服务器不允许注册时显示错误提示
  - 需要邮箱验证时引导用户通过邮件完成注册
  - 支持直接注册时在客户端完成密码设置和自动登录
- 实现 Bitwarden 兼容的密码学处理(PBKDF2, HKDF, AES-256-CBC, RSA-2048)
- 集成密码强度检查和密码泄露检查(Have I Been Pwned API)

## Capabilities

### New Capabilities
- `user-registration`: 用户注册流程,包括表单输入、服务器通信、密码学处理和自动登录
- `password-crypto`: Bitwarden 兼容的密码学处理,包括密钥派生、加密和 CipherString 格式

### Modified Capabilities

## Impact

**前端影响**:
- 新增 `/register` 路由和注册页面组件
- 登录页面添加"创建新账户"导航链接
- 新增密码强度指示器和密码泄露检查组件

**后端影响**:
- 新增密码学模块(PBKDF2, HKDF, AES, RSA, HMAC)
- 新增注册相关的 Tauri commands
- 新增 Vaultwarden 注册 API 端点集成

**依赖影响**:
- Rust: 需要添加密码学相关 crates (ring, rsa, base64)
- 前端: 可能需要添加密码强度检查库

**用户体验影响**:
- 新用户可以直接在客户端完成注册
- 支持多种服务器配置的注册流程
- 注册后自动登录,无需手动输入凭证
