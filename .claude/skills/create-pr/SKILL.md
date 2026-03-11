---
name: create-pr
description: Automatically create GitHub Pull Request from feature branch. Links to Issue, generates PR description from commits and spec artifacts, and sets up reviewers.
compatibility: Requires GitHub repository with gh CLI configured
metadata:
  author: vanguard-workflow
  version: 1.0.0
  architecture: orchestrator
---

# Create Pull Request Skill (Orchestrator)

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Architecture

此 skill 采用 **Orchestrator Pattern**，主 session 负责调度，实际工作由 sub-agents 执行：

```
Main Session (Orchestrator)
    ├─> Agent 1: Branch Validator (验证分支状态)
    ├─> Agent 2: PR Content Generator (生成 PR 描述)
    └─> Agent 3: PR Creator (创建 PR 并设置)
```

## Outline

### 1. 解析输入参数

从用户输入中提取 Issue 编号和其他参数：

```bash
# 支持多种输入格式
# /create-pr --issue 123
# /create-pr #123
# /create-pr 123
# /create-pr (自动从当前分支推断)

ISSUE_NUMBER=$(echo "$ARGUMENTS" | grep -oE '[0-9]+' | head -1)

# 如果没有提供 Issue 编号，尝试从分支名推断
if [ -z "$ISSUE_NUMBER" ]; then
  CURRENT_BRANCH=$(git branch --show-current)
  ISSUE_NUMBER=$(echo "$CURRENT_BRANCH" | grep -oE '^[0-9]+' | head -1)
fi

if [ -z "$ISSUE_NUMBER" ]; then
  echo "ERROR: Cannot determine Issue number"
  echo "Usage: /create-pr --issue <number>"
  exit 1
fi
```

### 2. 启动 Agent 1: Branch Validator

**任务**: 验证分支状态和代码变更

```markdown
Launch Agent with:
- description: "Validate branch and changes"
- prompt: "
  Validate the current branch and prepare for PR creation.

  **Issue Number**: $ISSUE_NUMBER

  Steps:
  1. Verify GitHub environment:
     - Check gh CLI is installed and authenticated
     - Verify Issue #$ISSUE_NUMBER exists
     - Check current branch is not main/master

  2. Get branch information:
     - Current branch name
     - Base branch (usually main or master)
     - Commits ahead of base branch

  3. Verify code changes:
     - Check there are uncommitted changes (warn if yes)
     - Get list of changed files
     - Get diff statistics (lines added/deleted)
     - Verify all tests pass (run test suite)

  4. Check for conflicts:
     - Fetch latest from remote
     - Check if branch can be merged cleanly

  5. Commit any uncommitted changes (if user confirms):
     - Use zcf:git-commit skill to generate conventional commit message
     - Commit with appropriate message

  Return JSON:
  {
    \"status\": \"success\" | \"error\",
    \"branch_name\": \"001-user-auth\",
    \"base_branch\": \"main\",
    \"commits_ahead\": 8,
    \"files_changed\": 12,
    \"lines_added\": 450,
    \"lines_deleted\": 120,
    \"tests_passed\": true,
    \"has_conflicts\": false,
    \"uncommitted_changes\": false,
    \"error\": \"error message if status is error\"
  }

  If there are uncommitted changes, ask user:
  'Found uncommitted changes. Commit them before creating PR? (yes/no)'
"
```

**等待 Agent 1 完成**。

**错误处理**:
- 如果测试失败，停止流程并提示用户修复
- 如果有冲突，提示用户先解决冲突
- 如果有未提交的变更，询问用户是否提交

### 3. 启动 Agent 2: PR Content Generator

**任务**: 生成 PR 标题和描述

**输入**: Agent 1 返回的分支信息和变更统计

```markdown
Launch Agent with:
- description: "Generate PR content"
- prompt: "
  Generate Pull Request title and description.

  **Context**:
  - Issue: #$ISSUE_NUMBER
  - Branch: [branch_name from Agent 1]
  - Commits: [commits_ahead from Agent 1]
  - Files changed: [files_changed from Agent 1]

  Steps:
  1. Read Issue #$ISSUE_NUMBER to get:
     - Issue title
     - Issue description
     - Labels

  2. Read spec artifacts (if exist):
     - specs/[feature]/spec.md
     - specs/[feature]/plan.md
     - specs/[feature]/tasks.md

  3. Get commit history:
     ```bash
     git log [base_branch]..[branch_name] --oneline
     ```

  4. Generate PR title:
     - Format: '[Type] Brief description (closes #$ISSUE_NUMBER)'
     - Type: feat/fix/refactor/docs/perf/test
     - Example: 'feat: Add user authentication (closes #123)'

  5. Generate PR description using this template:

     ```markdown
     ## 📋 概述

     [从 Issue 描述提取的简要说明]

     Closes #$ISSUE_NUMBER

     ## 🎯 变更内容

     ### 主要功能
     - [功能点 1]
     - [功能点 2]
     - [功能点 3]

     ### 技术实现
     - [实现细节 1]
     - [实现细节 2]

     ## 📊 变更统计

     - **提交数量**: [commits_ahead]
     - **修改文件**: [files_changed] 个
     - **新增代码**: +[lines_added] 行
     - **删除代码**: -[lines_deleted] 行

     ## ✅ 测试

     - [x] 所有单元测试通过
     - [x] 所有集成测试通过
     - [x] 手动测试完成
     - [x] 代码覆盖率: [coverage]%

     ## 📸 截图/演示

     [如果是 UI 变更，添加截图或 GIF]

     ## 🔍 审查要点

     - [ ] 代码符合 SOLID 原则
     - [ ] 遵循 KISS 和 YAGNI 原则
     - [ ] 无重复代码 (DRY)
     - [ ] 错误处理完善
     - [ ] 安全性检查通过
     - [ ] 性能影响可接受

     ## 📚 相关资源

     - Issue: #$ISSUE_NUMBER
     - Spec: [link to spec.md]
     - Plan: [link to plan.md]
     - Tasks: [link to tasks.md]

     ## 📝 部署说明

     [如果需要特殊的部署步骤，在此说明]

     ---

     **生成方式**: 使用 `/create-pr` 自动生成
     ```

  Return JSON:
  {
    \"status\": \"success\" | \"error\",
    \"pr_title\": \"feat: Add user authentication (closes #123)\",
    \"pr_body\": \"[full PR description]\",
    \"pr_type\": \"feat\",
    \"error\": \"error message if status is error\"
  }
"
```

**等待 Agent 2 完成**。

### 4. 启动 Agent 3: PR Creator

**任务**: 推送分支并创建 PR

**输入**: Agent 1 的分支信息 + Agent 2 的 PR 内容

```markdown
Launch Agent with:
- description: "Create Pull Request"
- prompt: "
  Push branch and create Pull Request on GitHub.

  **Context**:
  - Branch: [branch_name from Agent 1]
  - Base: [base_branch from Agent 1]
  - Issue: #$ISSUE_NUMBER
  - PR Title: [pr_title from Agent 2]
  - PR Body: [pr_body from Agent 2]

  Steps:
  1. Push branch to remote:
     ```bash
     git push origin [branch_name]
     ```

     If push fails (e.g., branch already exists with different commits):
     - Ask user: 'Branch exists on remote with different commits. Force push? (yes/no)'
     - If yes: `git push --force-with-lease origin [branch_name]`
     - If no: Stop and report error

  2. Create Pull Request using gh CLI:
     ```bash
     gh pr create \\
       --title \"[pr_title]\" \\
       --body \"[pr_body]\" \\
       --base [base_branch] \\
       --head [branch_name] \\
       --assignee \"@me\"
     ```

  3. Extract PR number and URL from output

  4. Set PR labels based on Issue labels:
     ```bash
     gh pr edit [pr_number] --add-label \"[label1],[label2]\"
     ```

  5. Link PR to Issue (if not auto-linked by 'Closes #'):
     - Add comment to Issue: 'Pull Request created: #[pr_number]'

  6. Optional: Request reviewers (if configured):
     ```bash
     gh pr edit [pr_number] --add-reviewer \"[reviewer1],[reviewer2]\"
     ```

  7. Optional: Add to project board (if configured):
     ```bash
     gh pr edit [pr_number] --add-project \"[project_name]\"
     ```

  Return JSON:
  {
    \"status\": \"success\" | \"error\",
    \"pr_number\": 456,
    \"pr_url\": \"https://github.com/owner/repo/pull/456\",
    \"branch_pushed\": true,
    \"labels_added\": [\"enhancement\", \"complexity: medium\"],
    \"reviewers_added\": [\"reviewer1\"],
    \"error\": \"error message if status is error\"
  }
"
```

**等待 Agent 3 完成**。

**错误处理**:
- 如果推送失败，提示用户检查权限或网络
- 如果 PR 创建失败，提供手动创建的指导
- 如果添加 reviewer 失败，记录警告但不阻止流程

### 5. 生成最终总结

当所有 Agents 完成后，主 session 生成总结：

```markdown
✅ **Pull Request 创建成功**

**PR 信息**:
- 编号: #[pr_number]
- 标题: [pr_title]
- 链接: [pr_url]
- 分支: [branch_name] → [base_branch]

**变更统计**:
- 提交数量: [commits_ahead]
- 修改文件: [files_changed] 个
- 新增代码: +[lines_added] 行
- 删除代码: -[lines_deleted] 行

**关联信息**:
- 关闭 Issue: #$ISSUE_NUMBER
- 标签: [labels]
- 审查人: [reviewers]

**下一步操作**:
```bash
# 在浏览器中查看 PR
gh pr view [pr_number] --web

# 请求代码审查
/review-pr --pr [pr_number]

# 查看 PR 状态
gh pr status

# 查看 CI/CD 检查
gh pr checks [pr_number]
```

**快捷命令**:
```bash
# 添加评论
gh pr comment [pr_number] --body "评论内容"

# 更新 PR 描述
gh pr edit [pr_number] --body "新的描述"

# 合并 PR (需要审查通过)
gh pr merge [pr_number] --squash --delete-branch
```
```

## 高级功能

### 自动检测 PR 类型

根据变更内容自动确定 PR 类型：

```bash
# 分析 commit messages
if commits contain "feat:"; then type="feat"
elif commits contain "fix:"; then type="fix"
elif commits contain "refactor:"; then type="refactor"
elif commits contain "docs:"; then type="docs"
elif commits contain "test:"; then type="test"
elif commits contain "perf:"; then type="perf"
else type="chore"
```

### 自动添加标签

根据变更内容自动添加标签：

```bash
# 根据文件类型
if changed files include "*.rs"; then add "rust"
if changed files include "*.ts"; then add "typescript"
if changed files include "*.md"; then add "documentation"

# 根据目录
if changed files in "src/frontend/"; then add "frontend"
if changed files in "src/backend/"; then add "backend"
if changed files in "tests/"; then add "tests"

# 根据变更规模
if lines_changed > 500; then add "large-pr"
elif lines_changed > 200; then add "medium-pr"
else add "small-pr"
```

### 自动截图（UI 变更）

如果检测到前端代码变更，提示用户添加截图：

```markdown
Agent 2 检测到前端变更，询问用户:
'检测到 UI 变更。是否需要添加截图或演示 GIF？(yes/no)'

如果 yes:
- 提示用户上传截图到 Issue 或 PR
- 或使用 Playwright 自动截图（如果配置了）
```

### 自动运行 CI/CD 检查

PR 创建后，等待 CI/CD 检查完成：

```bash
# 等待 CI/CD 检查
gh pr checks [pr_number] --watch

# 如果检查失败，在 PR 中添加评论
if checks failed; then
  gh pr comment [pr_number] --body "⚠️ CI/CD 检查失败，请修复后重新推送"
fi
```

### 草稿 PR 模式

支持创建草稿 PR（用于早期反馈）：

```bash
# 创建草稿 PR
/create-pr --issue 123 --draft

# Agent 3 使用 --draft 参数
gh pr create --draft ...
```

## 错误处理

| 错误类型 | 处理方式 |
|---------|---------|
| 测试失败 | 停止流程，提示用户修复测试 |
| 合并冲突 | 提示用户先解决冲突 |
| 推送失败 | 检查权限和网络，提供重试选项 |
| PR 已存在 | 提示用户更新现有 PR 或关闭后重新创建 |
| 权限不足 | 提示用户检查 GitHub 权限 |

### 回退机制

如果自动创建失败，提供手动创建指导：

```markdown
⚠️ **自动创建 PR 失败，请手动创建**

1. 推送分支:
   ```bash
   git push origin [branch_name]
   ```

2. 访问: https://github.com/[owner]/[repo]/compare/[base]...[branch]

3. 复制以下内容到 PR 表单:

   **标题**:
   [pr_title]

   **描述**:
   [pr_body]

4. 设置:
   - Base: [base_branch]
   - Compare: [branch_name]
   - Assignee: 自己
   - Labels: [labels]
   - Linked Issue: #$ISSUE_NUMBER
```

## 与其他 Skills 的集成

- **上游**: `/spec-driven-dev` → 完成功能实现
- **下游**: `/review-pr` → 代码审查
- **并行**: 可以在 PR 创建后自动触发 `/review-pr`

## 自动触发 Code Review

PR 创建成功后，可选自动触发代码审查：

```bash
# 询问用户
'PR 已创建。是否立即进行代码审查？(yes/no)'

if yes:
  /review-pr --pr [pr_number]
```

## 注意事项

1. **分支保护**: 确保不在 main/master 分支直接创建 PR
2. **测试通过**: 创建 PR 前确保所有测试通过
3. **提交规范**: 使用 Conventional Commits 格式
4. **描述完整**: PR 描述应包含足够的上下文信息
5. **关联 Issue**: 使用 `Closes #123` 自动关联 Issue
6. **审查人**: 根据团队规范添加合适的审查人

## 示例用法

### 示例 1: 基于 Issue 创建 PR

```bash
# 用户输入
/create-pr --issue 123

# 主 session 执行流程
1. 解析 Issue 编号: 123
2. 启动 Agent 1 (Branch Validator)
   - 验证分支状态
   - 检查测试通过
   - 返回分支信息和变更统计
3. 启动 Agent 2 (PR Content Generator)
   - 读取 Issue 和 spec 文件
   - 生成 PR 标题和描述
   - 返回 PR 内容
4. 启动 Agent 3 (PR Creator)
   - 推送分支到 remote
   - 创建 PR
   - 设置标签和审查人
   - 返回 PR 信息
5. 生成最终总结
6. 询问是否立即进行代码审查
```

### 示例 2: 自动推断 Issue（从分支名）

```bash
# 当前分支: 001-user-auth
/create-pr

# 主 session 自动推断 Issue #1
# 执行相同流程
```

### 示例 3: 创建草稿 PR

```bash
/create-pr --issue 123 --draft

# Agent 3 使用 --draft 参数创建草稿 PR
# 用于早期反馈，不会触发自动合并
```

## 性能优化

1. **并行检查**: Agent 1 可以并行执行多个验证（测试、linting、格式化）
2. **缓存 Issue 数据**: 如果之前读取过 Issue，可以复用数据
3. **增量推送**: 仅推送新的 commits，而非整个分支
4. **异步 CI/CD**: 不等待 CI/CD 完成，创建 PR 后立即返回

## 质量保证

在创建 PR 前，自动执行质量检查：

1. **代码质量**: Linting、格式化
2. **测试覆盖率**: 确保新代码有测试覆盖
3. **安全扫描**: 检查潜在的安全漏洞
4. **性能检查**: 检查是否引入性能回归
5. **文档更新**: 检查是否需要更新文档
