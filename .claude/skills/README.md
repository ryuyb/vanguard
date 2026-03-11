# Vanguard Workflow Skills

基于 GitHub Issue 的 Spec-Driven Development 工作流自动化工具集。

## 📋 概述

这套 skills 实现了从需求分析到代码审查的完整开发工作流，通过 GitHub Issues 和 Pull Requests 进行协作，使用 Orchestrator Pattern 确保上下文隔离和高效执行。

## 🔄 完整工作流

```mermaid
graph LR
    A[用户需求] --> B[/analyze-feasibility]
    B --> C[/create-issue]
    C --> D[/spec-driven-dev]
    D --> E[/create-pr]
    E --> F[/review-pr]
    F --> G[合并 PR]
    G --> H[关闭 Issue]
```

## 🛠️ Skills 列表

### 1. `/analyze-feasibility` - 可行性分析

**功能**: 分析技术可行性和实现复杂度

**使用场景**: 用户提出新功能需求时，首先评估可行性

**输入**: 自然语言描述的功能需求

**输出**:
- `feasibility-report.md` - 结构化的可行性分析报告
- 复杂度评分（1-5 分制）
- 风险识别
- 2-3 个实现方案建议

**示例**:
```bash
/analyze-feasibility 添加用户登录功能，支持邮箱和密码登录
```

**输出示例**:
```
✅ 可行性分析完成
- 总体复杂度: 3.2/5.0 (中等)
- 预估工时: 3-5 天
- 推荐方案: 使用 JWT + bcrypt 实现用户认证
- 下一步: 使用 /create-issue 创建 GitHub Issue
```

---

### 2. `/create-issue` - 创建 GitHub Issue

**功能**: 自动创建 GitHub Issue，关联可行性报告

**使用场景**: 可行性分析完成后，创建 Issue 跟踪开发进度

**输入**:
- 可行性报告（自动读取 `feasibility-report.md`）
- 或直接输入需求描述（快速模式）

**输出**:
- GitHub Issue（包含详细描述、标签、验收标准）
- Issue 编号和链接

**示例**:
```bash
# 模式 A: 基于可行性报告
/create-issue

# 模式 B: 快速创建（无可行性报告）
/create-issue 添加用户登录功能
```

**自动生成的标签**:
- 类型: `enhancement`, `bug`, `refactor`, `documentation`, `performance`, `security`
- 复杂度: `complexity: simple`, `complexity: medium`, `complexity: complex`
- 优先级: `priority: high`, `priority: medium`, `priority: low`
- 风险: `risk: high`, `risk: medium`

---

### 3. `/spec-driven-dev` - Spec 驱动开发 (Orchestrator)

**功能**: 执行完整的 Spec-Driven Development 工作流

**架构**: Orchestrator Pattern，主 session 仅调度，实际工作由 5 个 sub-agents 执行

**使用场景**: Issue 创建后，开始功能开发

**输入**: GitHub Issue 编号

**执行流程**:
1. **Agent 1: Issue Reader** - 读取 Issue 内容并验证环境
2. **Agent 2: Spec Generator** - 调用 `speckit-specify` 生成 `spec.md`
3. **Agent 3: Plan Generator** - 调用 `speckit-plan` 生成 `plan.md`
4. **Agent 4: Tasks Generator** - 调用 `speckit-tasks` 生成 `tasks.md`
5. **Agent 5: Implementation Executor** - 调用 `speckit-implement` 执行实现（后台运行）

**输出**:
- 新分支（如 `001-user-auth`）
- `specs/001-user-auth/spec.md` - 功能规格
- `specs/001-user-auth/plan.md` - 实现计划
- `specs/001-user-auth/tasks.md` - 任务分解
- 代码实现和测试

**示例**:
```bash
/spec-driven-dev --issue 123
```

**进度跟踪**:
- 每个阶段完成后，在 Issue 中自动添加评论
- 支持断点续传（如果中断，可以从上次停止的地方继续）

**错误处理**:
- 如果某个阶段失败，在 Issue 中记录错误
- 支持从失败的阶段重试: `/spec-driven-dev --issue 123 --retry plan`

---

### 4. `/create-pr` - 创建 Pull Request (Orchestrator)

**功能**: 自动创建 GitHub Pull Request

**架构**: Orchestrator Pattern，3 个 sub-agents 执行

**使用场景**: 功能实现完成后，创建 PR 进行代码审查

**输入**: GitHub Issue 编号（或自动从分支名推断）

**执行流程**:
1. **Agent 1: Branch Validator** - 验证分支状态、运行测试、检查冲突
2. **Agent 2: PR Content Generator** - 生成 PR 标题和描述
3. **Agent 3: PR Creator** - 推送分支、创建 PR、设置标签和审查人

**输出**:
- GitHub Pull Request
- PR 编号和链接
- 自动关联 Issue（使用 `Closes #123`）

**示例**:
```bash
# 基于 Issue 创建 PR
/create-pr --issue 123

# 自动推断 Issue（从分支名 001-user-auth）
/create-pr

# 创建草稿 PR
/create-pr --issue 123 --draft
```

**PR 描述模板**:
- 概述和关联 Issue
- 变更内容（主要功能、技术实现）
- 变更统计（提交数、文件数、代码行数）
- 测试结果
- 审查要点（SOLID、KISS、YAGNI、DRY）
- 相关资源（spec.md、plan.md、tasks.md）

---

### 5. `/review-pr` - 代码审查 (Orchestrator)

**功能**: 全面的代码审查，分析代码质量、安全性、性能

**架构**: Orchestrator Pattern，5 个 sub-agents 执行

**使用场景**: PR 创建后，进行代码审查

**输入**: GitHub PR 编号

**执行流程**:
1. **Agent 1: PR Fetcher** - 获取 PR 信息和 diff
2. **Agent 2: Code Quality Analyzer** - 分析 SOLID/KISS/YAGNI/DRY 原则
3. **Agent 3: Security Scanner** - 扫描 OWASP Top 10 漏洞
4. **Agent 4: Performance Checker** - 分析性能影响
5. **Agent 5: Review Reporter** - 生成审查报告并发布到 PR

**输出**:
- 代码质量评分（1-10 分）
- 问题列表（按严重程度分类）
- 改进建议
- 审查报告（作为 PR 评论发布）

**示例**:
```bash
/review-pr --pr 456
```

**审查维度**:

#### 代码质量
- SOLID 原则违反
- KISS/YAGNI/DRY 原则违反
- 代码异味（长函数、大类、重复代码等）
- 命名规范、注释质量

#### 安全性
- SQL 注入、XSS、CSRF
- 命令注入、路径遍历
- 硬编码的密钥
- 弱加密算法
- 依赖漏洞

#### 性能
- N+1 查询
- 不必要的克隆
- 阻塞操作
- 内存泄漏
- 低效算法

**问题严重程度**:
- 🔴 **Critical**: 必须修复才能合并
- 🟡 **High**: 强烈建议修复
- 🟢 **Medium/Low**: 可选修复

---

## 🚀 快速开始

### 前置条件

1. **安装 gh CLI**:
   ```bash
   # macOS
   brew install gh

   # Linux
   sudo apt install gh

   # Windows
   scoop install gh
   ```

2. **认证 GitHub**:
   ```bash
   gh auth login
   ```

3. **确保在 Git 仓库中**:
   ```bash
   git status
   ```

### 完整工作流示例

```bash
# 步骤 1: 用户提出需求
# "我想添加用户登录功能，支持邮箱和密码"

# 步骤 2: 分析可行性
/analyze-feasibility 添加用户登录功能，支持邮箱和密码登录

# 输出: feasibility-report.md
# 复杂度: 3.2/5.0 (中等)
# 预估: 3-5 天

# 步骤 3: 创建 GitHub Issue
/create-issue

# 输出: Issue #123 创建成功
# 链接: https://github.com/owner/repo/issues/123

# 步骤 4: 开始开发（Spec 驱动）
/spec-driven-dev --issue 123

# 输出:
# ✅ Agent 1: Issue Reader - 完成
# ✅ Agent 2: Spec Generator - spec.md 已生成
# ✅ Agent 3: Plan Generator - plan.md 已生成
# ✅ Agent 4: Tasks Generator - tasks.md 已生成
# ⏳ Agent 5: Implementation Executor - 后台运行中...

# 步骤 5: 创建 Pull Request
/create-pr --issue 123

# 输出: PR #456 创建成功
# 链接: https://github.com/owner/repo/pull/456

# 步骤 6: 代码审查
/review-pr --pr 456

# 输出:
# 代码质量评分: 8.5/10
# 🔴 Critical Issues: 0
# 🟡 High Priority: 2
# 🟢 Medium/Low: 5
# 推荐: ✅ 批准合并

# 步骤 7: 合并 PR
gh pr merge 456 --squash --delete-branch

# Issue #123 自动关闭 ✅
```

---

## 🎯 核心优势

### 1. **上下文隔离**
- 每个 skill 使用独立的 sub-agents
- 主 session 仅负责调度，不消耗大量 tokens
- 避免 context 污染

### 2. **并行执行**
- 独立的 agents 可以并行运行
- 例如：`/review-pr` 的代码质量、安全、性能分析可以同时执行

### 3. **容错性强**
- 单个 agent 失败不影响整体流程
- 支持从失败的阶段重试
- 错误信息记录在 GitHub Issue/PR 中

### 4. **可观测性**
- 主 session 实时监控各 agent 状态
- 在 GitHub Issue/PR 中记录进度
- 用户可以随时查看当前状态

### 5. **可扩展性**
- 容易添加新的 agents（如性能测试、文档生成）
- 可以自定义审查规则和质量标准
- 支持自定义工作流阶段

---

## 📚 高级用法

### 断点续传

如果工作流中断，可以从上次停止的地方继续：

```bash
# 假设 spec.md 和 plan.md 已存在
/spec-driven-dev --issue 123

# 自动检测已完成的阶段，跳过 Agent 2 和 3
# 直接从 Agent 4 (Tasks Generator) 开始
```

### 自定义阶段

跳过某些阶段或仅执行特定阶段：

```bash
# 仅生成 spec 和 plan，不执行实现
/spec-driven-dev --issue 123 --stages "specify,plan"

# 从 tasks 阶段开始（假设 spec 和 plan 已存在）
/spec-driven-dev --issue 123 --from tasks
```

### 草稿 PR

创建草稿 PR 用于早期反馈：

```bash
/create-pr --issue 123 --draft
```

### 自定义审查规则

创建 `.claude/review-rules.yaml` 定义自定义审查规则：

```yaml
rules:
  - name: "No console.log in production"
    pattern: "console\\.log"
    severity: "high"
    message: "Remove console.log before merging"

  - name: "No TODO comments"
    pattern: "TODO|FIXME"
    severity: "medium"
    message: "Resolve TODO comments"

  - name: "Test coverage required"
    min_coverage: 80
    severity: "high"
```

---

## 🔧 故障排除

### 常见错误

| 错误 | 原因 | 解决方案 |
|------|------|----------|
| `gh: command not found` | gh CLI 未安装 | `brew install gh` (macOS) |
| `gh auth status` 失败 | 未认证 | `gh auth login` |
| `Not a GitHub repository` | Remote 不是 GitHub | 检查 `git remote -v` |
| `Issue not found` | Issue 不存在 | 先使用 `/create-issue` 创建 |
| `Permission denied` | 无权限创建 Issue/PR | 检查仓库权限 |
| `Tests failed` | 测试未通过 | 修复测试后重新运行 |

### 重试失败的阶段

```bash
# 从失败的阶段重新开始
/spec-driven-dev --issue 123 --retry plan
```

### 查看详细日志

```bash
# 查看 Issue 评论（包含各阶段的进度）
gh issue view 123 --comments

# 查看 PR 评论（包含审查报告）
gh pr view 456 --comments
```

---

## 📖 最佳实践

### 1. 需求描述要清晰

```bash
# ✅ 好的需求描述
/analyze-feasibility 添加用户登录功能，支持邮箱和密码登录，使用 JWT 进行会话管理

# ❌ 不好的需求描述
/analyze-feasibility 登录
```

### 2. 及时处理澄清问题

如果 `speckit-specify` 生成的 spec 包含 `[NEEDS CLARIFICATION]` 标记，及时回答问题：

```bash
# Agent 2 会询问澄清问题
Q1: 密码最小长度是多少？
A: 8 个字符

Q2: 是否需要支持第三方登录（如 Google、GitHub）？
A: 暂时不需要
```

### 3. 代码审查后及时修复

如果 `/review-pr` 发现 Critical 或 High Priority 问题，及时修复：

```bash
# 修复问题后，重新审查
/review-pr --pr 456
```

### 4. 保持分支整洁

合并 PR 后，删除特性分支：

```bash
gh pr merge 456 --squash --delete-branch
```

### 5. 定期清理已合并的分支

```bash
# 使用 zcf:git-cleanBranches skill
/zcf:git-cleanBranches
```

---

## 🤝 与现有 Skills 的集成

### Speckit Skills

这些 workflow skills 内部调用了 speckit skills：

- `speckit-specify` - 生成功能规格
- `speckit-plan` - 生成实现计划
- `speckit-tasks` - 生成任务分解
- `speckit-implement` - 执行实现
- `speckit-analyze` - 一致性分析
- `speckit-checklist` - 质量检查清单

### ZCF Skills

可以与 zcf skills 配合使用：

- `zcf:git-commit` - 生成规范的 commit message
- `zcf:git-cleanBranches` - 清理已合并的分支
- `zcf:git-rollback` - 回滚到历史版本

### Simplify Skill

`/review-pr` 内部调用 `simplify` skill 进行代码质量分析。

---

## 📝 贡献指南

### 添加新的 Agent

如果需要扩展功能，可以添加新的 agent：

```markdown
# 示例：添加文档生成 agent

Launch Agent with:
- description: "Generate documentation"
- prompt: "
  Generate documentation for the implemented feature.

  Steps:
  1. Read spec.md and plan.md
  2. Generate API documentation
  3. Update README.md
  4. Generate usage examples

  Return JSON with generated files.
"
```

### 自定义工作流

可以创建自定义的 workflow skill，组合现有的 skills：

```bash
# 示例：快速开发工作流（跳过可行性分析）
/quick-dev 添加用户登录功能
# → 直接创建 Issue
# → 执行 spec-driven-dev
# → 创建 PR
```

---

## 📄 许可证

MIT License

---

## 🙏 致谢

- [Speckit](https://github.com/spec-kit) - Spec-Driven Development 方法论
- [GitHub CLI](https://cli.github.com/) - GitHub 命令行工具
- [Claude Code](https://www.anthropic.com/) - AI 编程助手

---

## 📞 支持

如有问题或建议，请：
1. 查看 [故障排除](#-故障排除) 部分
2. 在 GitHub 创建 Issue
3. 查看 [最佳实践](#-最佳实践)

---

**最后更新**: 2026-03-10
