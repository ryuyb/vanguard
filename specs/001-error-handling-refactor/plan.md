# Implementation Plan: 统一错误处理与展示重构

**Branch**: `001-error-handling-refactor` | **Date**: 2026-03-09 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-error-handling-refactor/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

重构 Rust 后端和 React 前端的错误处理机制,建立基于唯一错误代码的标准化错误响应体系,替代现有的基于消息文本的错误识别方式。后端通过扩展现有的 AppError 类型实现细粒度的错误分类,前端建立统一的错误拦截层并使用 Sonner Toast 组件提供非侵入式的错误通知。

## Technical Context

**Language/Version**: Rust 2021 Edition (后端), TypeScript 5.8 (前端)
**Primary Dependencies**:
- 后端: Tauri 2, serde/serde_json, tauri-specta 2.0.0-rc.21
- 前端: React 19, TanStack Router 1.163, shadcn/ui (需安装 Sonner)
**Storage**: SQLite (rusqlite 0.38) - 不涉及此次重构
**Testing**: cargo test (后端), 前端测试框架待确认
**Target Platform**: macOS desktop (Tauri 应用)
**Project Type**: Desktop application (Tauri + React)
**Performance Goals**: 错误处理延迟 <10ms, Toast 渲染 <100ms
**Constraints**:
- 必须保持与现有 tauri-specta 类型生成的兼容性
- 错误响应必须可序列化为 JSON
- Toast 通知不能阻塞 UI 主线程
**Scale/Scope**:
- 后端: ~50 个错误类型需要迁移/定义
- 前端: ~20 个组件/hooks 需要移除内联错误处理

## Constitution Check (Phase 1 后重新评估)

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### 初始评估 (Phase 0 前)

- **Code Quality Gate**: ✅ PASS
- **Testing Gate**: ✅ PASS
- **UX Consistency Gate**: ✅ PASS
- **Performance Gate**: ✅ PASS

### Phase 1 后重新评估

- **Code Quality Gate**: ✅ PASS - 设计确认
  - 数据模型已定义: `AppError` 枚举扩展为 30+ 个细粒度错误类型
  - 接口契约已明确: `ErrorPayload` 结构和 `ErrorResponse` 接口
  - 无新架构层: 复用现有 Tauri command 层和 React hooks 层
  - 新依赖合理: 仅添加 `sonner` (3KB, shadcn/ui 推荐)

- **Testing Gate**: ✅ PASS - 测试策略已定义
  - 后端: 单元测试验证错误代码映射,集成测试验证 Tauri commands
  - 前端: 单元测试验证 ErrorHandler 解析,集成测试验证 Toast 显示
  - 契约测试: 验证 ErrorPayload 序列化/反序列化

- **UX Consistency Gate**: ✅ PASS - UX 规范已确立
  - Toast 位置: 右上角 (Sonner 默认)
  - Toast 样式: 根据 severity 使用不同颜色 (info/warning/error/fatal)
  - Toast 持续时间: 3-5 秒自动消失
  - 错误消息: 统一在 `error-messages.ts` 中定义,支持未来国际化

- **Performance Gate**: ✅ PASS - 性能预算已验证
  - 后端错误构造: <1ms (纯内存操作,无 I/O)
  - 前端错误解析: <5ms (简单 JSON 解析和对象查找)
  - Toast 渲染: <100ms (Sonner 轻量级实现)
  - 去重机制: O(1) 查找,使用 Map 数据结构

**结论**: 所有 Constitution Check 项在 Phase 1 后仍然通过,设计方案可行,可进入实现阶段。

## Project Structure

### Documentation (this feature)

```text
specs/001-error-handling-refactor/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   ├── error-codes.md   # 错误代码清单
│   └── error-response.md # 错误响应格式规范
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
# Tauri 桌面应用结构
src-tauri/
├── src/
│   ├── support/
│   │   ├── error.rs           # [修改] 扩展 AppError 枚举
│   │   └── result.rs          # [保持] AppResult 类型别名
│   ├── interfaces/tauri/
│   │   ├── commands/
│   │   │   ├── auth.rs        # [修改] 更新错误处理
│   │   │   ├── vault.rs       # [修改] 更新错误处理
│   │   │   └── sync.rs        # [修改] 更新错误处理
│   │   └── dto/
│   │       └── error.rs       # [新建] 前端错误 DTO
│   └── application/
│       └── use_cases/         # [修改] 各 use case 返回细粒度错误
└── tests/
    └── error_handling_test.rs # [新建] 错误处理集成测试

src/
├── lib/
│   ├── error-handler.ts       # [新建] 统一错误处理器
│   ├── error-messages.ts      # [新建] 错误代码到消息映射
│   └── toast.tsx              # [新建] Toast 通知封装
├── features/
│   ├── auth/
│   │   ├── login/
│   │   │   └── hooks/         # [修改] 移除内联错误处理
│   │   └── unlock/
│   │       └── hooks/         # [修改] 移除内联错误处理
│   ├── vault/
│   │   └── hooks/             # [修改] 移除内联错误处理
│   └── spotlight/
│       ├── error-utils.ts     # [删除] 不再需要
│       └── hooks/             # [修改] 移除内联错误处理
└── main.tsx                   # [修改] 添加 Toaster 组件
```

**Structure Decision**: 采用 Tauri 桌面应用的标准结构,后端使用 DDD 分层架构(已存在),前端使用 feature-based 模块化结构(已存在)。错误处理作为横切关注点,在后端通过 `support` 模块统一管理,在前端通过 `lib` 模块提供全局服务。

## Complexity Tracking

无需填写 - 所有 Constitution Check 项均通过,无违规需要辩护。
