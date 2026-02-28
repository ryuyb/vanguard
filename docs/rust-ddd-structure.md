# Tauri + Vaultwarden 密码管理器（Rust 端 DDD 目录结构）

```text
src-tauri/
├── Cargo.toml                                # Rust 依赖与 feature 管理
├── build.rs                                  # 构建阶段处理（按需）
└── src/
    ├── main.rs                               # 二进制入口，仅调用 lib::run()
    ├── lib.rs                                # Tauri 启动与路由注册入口
    │
    ├── bootstrap/                            # 启动编排、依赖注入、配置装配
    │   ├── mod.rs
    │   ├── app_state.rs                      # 全局共享状态（Arc/trait object）
    │   ├── wiring.rs                         # 接口到实现绑定 + 命令/类型导出装配
    │   └── config.rs                         # 配置加载与校验
    │
    ├── interfaces/                           # 接口层（入站适配器）
    │   ├── mod.rs
    │   └── tauri/
    │       ├── mod.rs
    │       ├── registry.rs                   # 命令注册清单（tauri-specta 单一真源）
    │       ├── commands/                     # tauri::command 按业务分组
    │       │   ├── mod.rs
    │       │   ├── auth.rs                   # 登录/解锁/会话相关命令
    │       │   ├── vault.rs                  # 密码条目 CRUD 与查询命令
    │       │   └── sync.rs                   # 与 Vaultwarden 同步命令
    │       ├── dto/                          # 请求/响应模型，隔离领域对象
    │       │   ├── mod.rs
    │       │   ├── request.rs
    │       │   └── response.rs
    │       ├── mapping.rs                    # DTO 与应用层对象转换
    │       └── specta/
    │           ├── mod.rs
    │           ├── builder.rs                # tauri_specta::Builder 构建与命令挂载
    │           ├── types.rs                  # 统一导出给前端的类型集合
    │           └── export.rs                 # 绑定文件导出（dev/CI）
    │
    ├── application/                          # 应用层（用例编排、事务边界）
    │   ├── mod.rs
    │   ├── services/
    │   │   ├── mod.rs
    │   │   ├── auth_service.rs               # 鉴权用例
    │   │   ├── vault_service.rs              # 密码条目用例
    │   │   └── sync_service.rs               # 同步用例
    │   ├── ports/                            # 出站端口（trait 抽象）
    │   │   ├── mod.rs
    │   │   ├── vault_repository.rs
    │   │   ├── remote_vault_port.rs
    │   │   ├── crypto_port.rs
    │   │   ├── key_store_port.rs
    │   │   └── clock_port.rs
    │   └── dto/                              # 应用层 command/query 模型
    │       ├── mod.rs
    │       ├── command.rs
    │       └── query.rs
    │
    ├── domain/                               # 领域层（纯业务规则，无框架依赖）
    │   ├── mod.rs
    │   ├── shared/
    │   │   ├── mod.rs
    │   │   ├── error.rs                      # 领域错误定义
    │   │   ├── value_objects.rs              # 通用值对象
    │   │   └── events.rs                     # 领域事件
    │   ├── auth/
    │   │   ├── mod.rs
    │   │   ├── aggregate.rs
    │   │   ├── entity.rs
    │   │   ├── policy.rs
    │   │   └── service.rs                    # 纯领域服务
    │   ├── vault_item/
    │   │   ├── mod.rs
    │   │   ├── aggregate.rs
    │   │   ├── entity.rs
    │   │   ├── policy.rs
    │   │   └── repository.rs                 # 仓储接口（可与 ports 对齐）
    │   └── organization/
    │       ├── mod.rs
    │       ├── aggregate.rs
    │       ├── entity.rs
    │       └── policy.rs
    │
    ├── infrastructure/                       # 基础设施层（出站适配器实现）
    │   ├── mod.rs
    │   ├── persistence/
    │   │   ├── mod.rs
    │   │   ├── sqlite/
    │   │   │   ├── mod.rs
    │   │   │   ├── schema.rs
    │   │   │   ├── migrations.rs
    │   │   │   └── vault_repository_impl.rs
    │   │   └── cache.rs
    │   ├── vaultwarden/
    │   │   ├── mod.rs
    │   │   ├── client.rs                     # HTTP 客户端封装
    │   │   ├── endpoints.rs                  # API 路由定义
    │   │   ├── models.rs                     # 第三方协议模型
    │   │   └── mapper.rs                     # 协议模型到领域模型映射
    │   ├── security/
    │   │   ├── mod.rs
    │   │   ├── crypto.rs                     # 加解密实现
    │   │   ├── kdf.rs                        # 密钥派生实现
    │   │   └── keyring.rs                    # 系统安全存储实现
    │   ├── telemetry/
    │   │   ├── mod.rs
    │   │   └── tracing.rs                    # 日志/追踪初始化
    │   └── time/
    │       ├── mod.rs
    │       └── system_clock.rs               # 时钟实现
    │
    ├── support/                              # 横切支撑层（非业务）
    │   ├── mod.rs
    │   ├── error.rs                          # 应用级统一错误
    │   ├── result.rs
    │   ├── id.rs
    │   └── serde_ext.rs
    │
    └── tests/                                # 集成测试与测试夹具
        ├── mod.rs
        ├── fixtures/
        │   ├── mod.rs
        │   └── builders.rs                   # 测试对象构造器
        ├── application/
        │   ├── auth_flow_test.rs
        │   └── vault_flow_test.rs
        └── interfaces/
            ├── tauri_command_test.rs
            └── specta_contract_test.rs       # 命令与类型导出契约测试
```

## 简要说明（维护性与可测试性）

- `interfaces -> application -> domain` 单向依赖，避免 Tauri/HTTP 细节污染核心业务。
- `application/ports` 先定义 trait，再由 `infrastructure` 提供实现，便于 mock 与替换。
- `domain` 保持纯净，不依赖数据库、网络、Tauri，便于高密度单元测试。
- `bootstrap` 统一装配依赖与配置，降低 `lib.rs` 复杂度，便于按环境切换实现。
- `interfaces/tauri/dto` 与 `mapping` 隔离外部协议，减少前后端接口变化对领域模型的冲击。
- `interfaces/tauri/registry.rs` 作为命令注册单一真源，同时服务 `invoke_handler` 与 `tauri-specta`，避免重复维护。
- `interfaces/tauri/specta/export.rs` 负责绑定导出流程，建议在开发与 CI 中执行，输出前端可直接消费的类型绑定文件。
- `tests` 以“用例流”组织集成测试，并通过 `fixtures/builders` 提升测试可读性与复用性。
