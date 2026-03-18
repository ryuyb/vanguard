## 1. 项目准备

- [x] 1.1 添加 Rust 密码学依赖 (ring, rsa, base64)
- [x] 1.2 创建注册功能模块目录结构
- [x] 1.3 更新 i18n 资源文件,添加注册相关文案

## 2. 后端基础设施 (阶段 1: MVP)

- [x] 2.1 在 `endpoints.rs` 添加注册 API endpoints
- [x] 2.2 创建 `registration_adapter.rs` 实现注册 API 调用
- [x] 2.3 创建注册相关的 DTO 类型定义
- [x] 2.4 实现 `send_verification_email` API 调用
- [x] 2.5 添加错误处理和日志记录

## 3. 前端注册页面 (阶段 1: MVP)

- [x] 3.1 创建 `/register` 路由
- [x] 3.2 创建 `register-page.tsx` 注册页面组件
- [x] 3.3 创建 `register-form.tsx` 注册表单组件
- [x] 3.4 实现表单验证 schema (邮箱、服务器 URL、名字)
- [x] 3.5 创建 `registration-feedback.tsx` 反馈提示组件
- [x] 3.6 在登录页添加"创建新账户"导航链接

## 4. 注册流程逻辑 (阶段 1: MVP)

- [x] 4.1 创建 `use-registration-flow.ts` hook
- [x] 4.2 实现场景 1: 处理 400 错误响应,显示错误提示
- [x] 4.3 实现场景 2: 处理 204 响应,显示邮件已发送提示
- [x] 4.4 实现场景 3 占位: 处理 200 响应,显示"开发中"提示
- [x] 4.5 实现返回登录页面功能
- [x] 4.6 添加加载状态和进度提示

## 5. 测试和优化 (阶段 1: MVP)

- [x] 5.1 测试场景 1: 服务器不允许注册
- [x] 5.2 测试场景 2: 需要邮箱验证
- [x] 5.3 测试表单验证和错误提示
- [x] 5.4 测试导航和路由
- [x] 5.5 优化 UI 样式和用户体验
- [x] 5.6 添加错误日志和监控

## 6. 密码学模块 (阶段 2: 完整功能)

- [x] 6.1 创建 `key_derivation.rs` 实现 PBKDF2 密钥派生
- [x] 6.2 实现 HKDF-Expand 对称密钥派生
- [x] 6.3 实现 Master Password Hash 生成
- [x] 6.4 创建 `encryption.rs` 实现 AES-256-CBC 加密/解密
- [x] 6.5 实现 HMAC-SHA256 消息认证
- [x] 6.6 创建 `cipher_string.rs` 实现 CipherString 格式处理
- [x] 6.7 创建 `rsa_keys.rs` 实现 RSA-2048 密钥对生成
- [x] 6.8 实现 RSA 私钥加密
- [x] 6.9 实现对称密钥自加密

## 7. 密码学模块测试 (阶段 2: 完整功能)

- [x] 7.1 编写 PBKDF2 单元测试
- [x] 7.2 编写 HKDF 单元测试
- [x] 7.3 编写 AES-256-CBC 加密/解密测试
- [x] 7.4 编写 HMAC-SHA256 测试
- [x] 7.5 编写 CipherString 格式解析测试
- [x] 7.6 编写 RSA 密钥对生成测试
- [x] 7.7 验证与 Bitwarden 官方实现的兼容性

## 8. 注册完成逻辑 (阶段 2: 完整功能)

- [x] 8.1 创建 `registration_service.rs` 注册业务逻辑
- [x] 8.2 实现 `register_finish` Tauri command
- [x] 8.3 集成密码学模块,生成注册所需的所有密钥
- [x] 8.4 实现 `register/finish` API 调用
- [x] 8.5 实现注册成功后的自动登录流程
- [x] 8.6 添加详细的错误处理和回滚逻辑

## 9. 密码设置 UI (阶段 2: 完整功能)

- [x] 9.1 创建 `password-setup-form.tsx` 密码设置表单
- [x] 9.2 创建 `password-strength.tsx` 密码强度指示器
- [x] 9.3 实现 `use-password-strength.ts` 密码强度检查 hook
- [x] 9.4 实现密码泄露检查 (Have I Been Pwned API)
- [x] 9.5 实现密码确认验证
- [x] 9.6 添加密码提示输入 (可选)

## 10. 场景 3 完整流程 (阶段 2: 完整功能)

- [x] 10.1 更新 `use-registration-flow.ts` 处理 200 响应
- [x] 10.2 实现 JWT token 存储和传递
- [x] 10.3 实现密码设置表单显示逻辑
- [x] 10.4 实现"完成注册"按钮点击处理
- [x] 10.5 实现注册进度提示 (创建账户、登录中等)
- [x] 10.6 实现注册成功后跳转到 Vault 页面

## 11. 集成测试 (阶段 2: 完整功能)

- [ ] 11.1 测试完整的场景 3 注册流程
- [ ] 11.2 测试密码强度检查和泄露检查
- [ ] 11.3 测试注册后自动登录
- [ ] 11.4 测试错误场景和边界情况
- [ ] 11.5 与真实 Vaultwarden 服务器进行端到端测试
- [ ] 11.6 测试跨客户端互操作性 (在其他 Bitwarden 客户端登录)

## 12. 文档和发布

- [ ] 12.1 更新 README 添加注册功能说明
- [ ] 12.2 更新 CHANGELOG 记录新功能
- [ ] 12.3 编写用户使用文档
- [ ] 12.4 准备发布说明
- [ ] 12.5 创建 GitHub Release
