## ADDED Requirements

### Requirement: 系统派生 Master Key
系统 SHALL 使用 PBKDF2-SHA256 从用户主密码派生 Master Key。

#### Scenario: 使用正确的 KDF 参数派生 Master Key
- **WHEN** 用户输入主密码
- **THEN** 系统使用 PBKDF2-SHA256 算法,以邮箱为 salt,迭代 600000 次,派生 32 字节的 Master Key

### Requirement: 系统生成 Master Password Hash
系统 SHALL 从 Master Key 生成用于服务器认证的密码哈希。

#### Scenario: 生成密码哈希
- **WHEN** 系统获得 Master Key
- **THEN** 系统使用 PBKDF2-SHA256 算法,以主密码为 salt,迭代 1 次,生成 32 字节的密码哈希并 Base64 编码

### Requirement: 系统派生 Symmetric Key
系统 SHALL 使用 HKDF-Expand 从 Master Key 派生对称加密密钥。

#### Scenario: 派生对称密钥
- **WHEN** 系统获得 Master Key
- **THEN** 系统使用 HKDF-Expand 算法,以 "enc" 为 info,派生 32 字节的 Symmetric Key

### Requirement: 系统生成 RSA 密钥对
系统 SHALL 生成 RSA-2048 密钥对用于数据共享。

#### Scenario: 生成 RSA 密钥对
- **WHEN** 系统需要创建用户密钥对
- **THEN** 系统生成 2048 位 RSA 密钥对,包含公钥和私钥

### Requirement: 系统加密 RSA 私钥
系统 SHALL 使用 Symmetric Key 加密 RSA 私钥。

#### Scenario: 加密私钥
- **WHEN** 系统生成 RSA 密钥对
- **THEN** 系统使用 AES-256-CBC 和 HMAC-SHA256 加密私钥,生成 CipherString 格式的加密数据

### Requirement: 系统加密 Symmetric Key
系统 SHALL 使用 Symmetric Key 加密自身以生成 Protected Symmetric Key。

#### Scenario: 加密对称密钥
- **WHEN** 系统派生 Symmetric Key
- **THEN** 系统使用 AES-256-CBC 和 HMAC-SHA256 加密 Symmetric Key,生成 CipherString 格式的 userSymmetricKey

### Requirement: 系统使用 CipherString 格式
系统 SHALL 使用 Bitwarden CipherString 格式存储加密数据。

#### Scenario: 生成 CipherString
- **WHEN** 系统加密数据
- **THEN** 系统生成格式为 "2.iv|ciphertext|mac" 的 CipherString,其中 2 表示 AesCbc256_HmacSha256_B64 类型

#### Scenario: 解析 CipherString
- **WHEN** 系统接收 CipherString 格式的数据
- **THEN** 系统正确解析加密类型、IV、密文和 MAC

### Requirement: 系统使用 AES-256-CBC 加密
系统 SHALL 使用 AES-256-CBC 模式加密敏感数据。

#### Scenario: 加密数据
- **WHEN** 系统需要加密数据
- **THEN** 系统生成随机 16 字节 IV,使用 AES-256-CBC 加密数据

#### Scenario: 解密数据
- **WHEN** 系统需要解密数据
- **THEN** 系统从 CipherString 提取 IV 和密文,使用 AES-256-CBC 解密

### Requirement: 系统使用 HMAC-SHA256 验证数据完整性
系统 SHALL 使用 HMAC-SHA256 验证加密数据的完整性。

#### Scenario: 生成 MAC
- **WHEN** 系统加密数据
- **THEN** 系统使用 HMAC-SHA256 计算 IV 和密文的 MAC

#### Scenario: 验证 MAC
- **WHEN** 系统解密数据
- **THEN** 系统验证 MAC 是否匹配,不匹配则拒绝解密

### Requirement: 系统兼容 Bitwarden 密码学规范
系统 SHALL 完全兼容 Bitwarden 的密码学实现,确保跨客户端互操作性。

#### Scenario: 在其他 Bitwarden 客户端登录
- **WHEN** 用户使用 Vanguard 注册账户
- **THEN** 用户可以使用相同凭证在 Bitwarden 官方客户端或其他兼容客户端登录

#### Scenario: 解密其他客户端创建的数据
- **WHEN** 用户在其他客户端创建加密数据
- **THEN** Vanguard 可以正确解密该数据

### Requirement: 系统安全处理密钥材料
系统 SHALL 安全地处理和存储密钥材料,防止泄露。

#### Scenario: 密钥仅在内存中存在
- **WHEN** 系统派生或生成密钥
- **THEN** 密钥仅在内存中存在,不写入磁盘明文

#### Scenario: 使用后清除密钥
- **WHEN** 密钥使用完毕
- **THEN** 系统从内存中安全清除密钥数据
