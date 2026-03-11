---
name: spec-driven-dev
description: Execute spec-driven development workflow based on GitHub Issue. Orchestrates sub-agents to execute speckit skills (specify → plan → tasks → implement) in isolated contexts.
compatibility: Requires spec-kit project structure with .specify/ directory and GitHub issue
metadata:
  author: vanguard-workflow
  version: 1.0.0
  architecture: orchestrator
---

# Spec-Driven Development Skill (Orchestrator)

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Architecture

此 skill 采用 **Orchestrator Pattern**，主 session 仅负责调度，实际工作由 sub-agents 执行：

```
Main Session (Orchestrator)
    ├─> Agent 1: Issue Reader (读取 Issue 内容)
    ├─> Agent 2: Spec Generator (调用 speckit-specify)
    ├─> Agent 3: Plan Generator (调用 speckit-plan)
    ├─> Agent 4: Tasks Generator (调用 speckit-tasks)
    └─> Agent 5: Implementation Executor (调用 speckit-implement)
```

## Outline

### 1. 解析 Issue 编号

从用户输入中提取 GitHub Issue 编号：

```bash
# 支持多种输入格式
ISSUE_NUMBER=$(echo "$ARGUMENTS" | grep -oE '[0-9]+' | head -1)

if [ -z "$ISSUE_NUMBER" ]; then
  echo "ERROR: No issue number provided"
  echo "Usage: /spec-driven-dev --issue <number>"
  exit 1
fi
```

### 2. 启动 Agent 1: Issue Reader

**任务**: 读取 GitHub Issue 内容并验证环境

使用 `Agent` 工具启动 sub-agent：

```markdown
Launch Agent with:
- description: "Read GitHub Issue"
- prompt: "
  Read GitHub Issue #$ISSUE_NUMBER and extract the following information:

  1. Verify GitHub environment:
     - Check if gh CLI is installed and authenticated
     - Verify Issue #$ISSUE_NUMBER exists

  2. Extract Issue details:
     - Title
     - Body (full description)
     - Labels
     - Current status

  3. Add a comment to the Issue:
     '🚀 **开始开发**

     使用 Spec-Driven Development 工作流:
     1. ⏳ 生成功能规格 (spec.md)
     2. ⏳ 生成实现计划 (plan.md)
     3. ⏳ 生成任务分解 (tasks.md)
     4. ⏳ 执行实现

     **执行时间**: [current timestamp]'

  4. Return a JSON summary:
     {
       \"issue_number\": $ISSUE_NUMBER,
       \"title\": \"...\",
       \"body\": \"...\",
       \"labels\": [\"...\"],
       \"status\": \"open\"
     }

  If any step fails, return an error with details.
"
```

**等待 Agent 1 完成**，获取返回的 JSON 数据。

**错误处理**:
- 如果 Agent 1 失败，停止流程并向用户报告错误
- 如果 Issue 不存在，提示用户先创建 Issue

### 3. 启动 Agent 2: Spec Generator

**任务**: 生成功能规格 (spec.md)

**输入**: Agent 1 返回的 Issue 标题和描述

```markdown
Launch Agent with:
- description: "Generate feature specification"
- prompt: "
  Execute the speckit-specify skill to generate a feature specification.

  **Feature Description**: [Issue Title from Agent 1]

  **Detailed Requirements**: [Issue Body from Agent 1]

  Steps:
  1. Call the speckit-specify skill with the feature description
  2. Ensure spec.md is generated successfully
  3. Verify the spec contains:
     - User stories
     - Functional requirements
     - Acceptance criteria
  4. If there are [NEEDS CLARIFICATION] markers, list them

  After completion, add a comment to Issue #$ISSUE_NUMBER:
  '✅ **阶段 1/4 完成**: 功能规格已生成

  **输出文件**: \`specs/[feature]/spec.md\`
  **分支**: \`[branch-name]\`
  **需要澄清**: [list clarifications or \"无\"]'

  Return:
  {
    \"status\": \"success\" | \"needs_clarification\" | \"error\",
    \"branch_name\": \"...\",
    \"spec_file\": \"...\",
    \"clarifications\": [\"...\"] or []
  }
"
```

**等待 Agent 2 完成**。

**错误处理**:
- 如果返回 `needs_clarification`，暂停流程并向用户展示需要澄清的问题
- 用户澄清后，重新启动 Agent 2 更新 spec.md
- 如果返回 `error`，停止流程并报告错误

### 4. 启动 Agent 3: Plan Generator

**任务**: 生成实现计划 (plan.md)

**输入**: Agent 2 返回的分支名称和 spec 文件路径

```markdown
Launch Agent with:
- description: "Generate implementation plan"
- prompt: "
  Execute the speckit-plan skill to generate an implementation plan.

  **Context**:
  - Branch: [branch_name from Agent 2]
  - Spec file: [spec_file from Agent 2]

  Steps:
  1. Ensure you are on the correct branch
  2. Call the speckit-plan skill
  3. Verify plan.md is generated with:
     - Technical stack
     - Architecture design
     - Implementation phases

  After completion, add a comment to Issue #$ISSUE_NUMBER:
  '✅ **阶段 2/4 完成**: 实现计划已生成

  **输出文件**: \`specs/[feature]/plan.md\`
  **技术栈**: [list tech stack]
  **实现阶段**: [number] 个阶段'

  Return:
  {
    \"status\": \"success\" | \"error\",
    \"plan_file\": \"...\",
    \"tech_stack\": [\"...\"],
    \"phases\": 3
  }
"
```

**等待 Agent 3 完成**。

### 5. 启动 Agent 4: Tasks Generator

**任务**: 生成任务分解 (tasks.md)

**输入**: Agent 3 返回的 plan 文件路径

```markdown
Launch Agent with:
- description: "Generate task breakdown"
- prompt: "
  Execute the speckit-tasks skill to generate a task breakdown.

  **Context**:
  - Branch: [branch_name from Agent 2]
  - Plan file: [plan_file from Agent 3]

  Steps:
  1. Call the speckit-tasks skill
  2. Verify tasks.md is generated with:
     - Ordered task list
     - Clear dependencies
     - Acceptance criteria for each task

  After completion, add a comment to Issue #$ISSUE_NUMBER:
  '✅ **阶段 3/4 完成**: 任务分解已生成

  **输出文件**: \`specs/[feature]/tasks.md\`
  **任务数量**: [number] 个任务
  **预估工时**: [estimate] 天'

  Return:
  {
    \"status\": \"success\" | \"error\",
    \"tasks_file\": \"...\",
    \"task_count\": 12,
    \"estimated_days\": 5
  }
"
```

**等待 Agent 4 完成**。

### 6. 启动 Agent 5: Implementation Executor

**任务**: 执行功能实现

**输入**: Agent 4 返回的 tasks 文件路径

```markdown
Launch Agent with:
- description: "Execute implementation"
- run_in_background: true  # 实现阶段可能耗时较长，在后台运行
- prompt: "
  Execute the speckit-implement skill to implement the feature.

  **Context**:
  - Branch: [branch_name from Agent 2]
  - Tasks file: [tasks_file from Agent 4]

  Steps:
  1. Call the speckit-implement skill
  2. Follow the tasks in tasks.md sequentially
  3. For each task:
     - Implement the code
     - Write tests
     - Run tests to verify
     - Commit changes with conventional commit message
  4. After all tasks complete:
     - Run full test suite
     - Check code quality (linting, formatting)
     - Verify SOLID, KISS, YAGNI, DRY principles

  After completion, add a comment to Issue #$ISSUE_NUMBER:
  '✅ **阶段 4/4 完成**: 功能实现完成

  **提交数量**: [number] 次提交
  **测试状态**: ✅ 所有测试通过
  **代码变更**:
  - 修改文件: [number] 个
  - 新增代码: +[lines] 行
  - 删除代码: -[lines] 行

  **下一步**: 使用 \`/create-pr --issue $ISSUE_NUMBER\` 创建 Pull Request'

  Return:
  {
    \"status\": \"success\" | \"error\",
    \"commits\": 8,
    \"files_changed\": 12,
    \"lines_added\": 450,
    \"lines_deleted\": 120,
    \"tests_passed\": true,
    \"test_summary\": \"15/15 tests passed\"
  }
"
```

**注意**: Agent 5 使用 `run_in_background: true`，因为实现阶段可能耗时较长。

主 session 不需要等待 Agent 5 完成，可以：
- 向用户报告 "实现阶段已启动，正在后台运行"
- 提供查看进度的命令
- 当 Agent 5 完成时，系统会自动通知

### 7. 监控和进度报告

主 session 提供进度查询功能：

```bash
# 用户可以随时查询进度
/spec-driven-dev --status

# 主 session 返回当前状态
✅ Agent 1: Issue Reader - Completed
✅ Agent 2: Spec Generator - Completed
✅ Agent 3: Plan Generator - Completed
✅ Agent 4: Tasks Generator - Completed
⏳ Agent 5: Implementation Executor - Running (60% complete)
```

### 8. 错误处理和重试

如果任何 Agent 失败：

```markdown
1. 主 session 捕获错误
2. 在 Issue 中添加错误评论:
   '❌ **阶段 [N] 失败**: [Agent Name]

   **错误信息**:
   ```
   [error details]
   ```

   **建议操作**:
   - 检查错误日志
   - 修复问题后重新运行: `/spec-driven-dev --issue $ISSUE_NUMBER --retry [stage]`'

3. 向用户展示错误和重试选项
```

支持从失败的阶段重试：

```bash
# 从 Agent 3 (Plan Generator) 重新开始
/spec-driven-dev --issue 123 --retry plan

# 主 session 跳过 Agent 1 和 2，直接启动 Agent 3
```

### 9. 生成最终总结

当所有 Agents 完成后，主 session 生成总结：

```markdown
✅ **Spec-Driven Development 完成**

**Issue**: #$ISSUE_NUMBER - [Issue Title]
**分支**: [branch_name]
**执行时间**: [total duration]

**各阶段结果**:
1. ✅ 功能规格 - specs/[feature]/spec.md
2. ✅ 实现计划 - specs/[feature]/plan.md
3. ✅ 任务分解 - specs/[feature]/tasks.md
4. ✅ 功能实现 - [commits] 次提交

**代码变更统计**:
- 修改文件: [number] 个
- 新增代码: +[lines] 行
- 删除代码: -[lines] 行

**测试结果**:
- ✅ 单元测试: [passed]/[total]
- ✅ 集成测试: [passed]/[total]
- ✅ 代码覆盖率: [percentage]%

**下一步**:
```bash
# 创建 Pull Request
/create-pr --issue $ISSUE_NUMBER

# 或查看变更
git diff main...[branch_name]
```
```

## Agent 通信协议

各 Agent 之间通过 **返回值** 传递数据，主 session 负责协调：

```
Agent 1 Output → Agent 2 Input
Agent 2 Output → Agent 3 Input
Agent 3 Output → Agent 4 Input
Agent 4 Output → Agent 5 Input
```

**数据格式**: 统一使用 JSON

```json
{
  "status": "success" | "error" | "needs_clarification",
  "data": {
    // Agent-specific data
  },
  "error": "error message if status is error"
}
```

## 并行优化

某些阶段可以并行执行（如果不依赖前一阶段的输出）：

```markdown
# 示例：如果 spec.md 和 plan.md 已存在，可以并行执行质量检查
Launch Agent 6 (Quality Check - Spec) in parallel with Agent 7 (Quality Check - Plan)
```

## 断点续传

主 session 检查已完成的阶段，跳过已完成的 Agents：

```bash
# 检查文件是否存在
if [ -f "specs/[feature]/spec.md" ]; then
  echo "✅ Skipping Agent 2 (Spec Generator) - spec.md already exists"
fi

if [ -f "specs/[feature]/plan.md" ]; then
  echo "✅ Skipping Agent 3 (Plan Generator) - plan.md already exists"
fi

# 从第一个未完成的阶段开始
```

## 优势

1. **上下文隔离**: 每个 Agent 只关注自己的任务，避免 context 污染
2. **并行执行**: 独立的 Agents 可以并行运行，提高效率
3. **容错性**: 单个 Agent 失败不影响其他 Agents
4. **可观测性**: 主 session 可以实时监控各 Agent 的状态
5. **可扩展性**: 容易添加新的 Agent（如代码审查、性能测试）
6. **资源优化**: 避免主 session 的 token 消耗过大

## 注意事项

1. **Agent 启动开销**: 每个 Agent 启动需要时间，权衡并行度和开销
2. **数据传递**: 确保 Agent 之间的数据格式一致
3. **错误传播**: 一个 Agent 失败可能导致后续 Agents 无法执行
4. **状态管理**: 主 session 需要维护各 Agent 的状态
5. **日志记录**: 在 Issue 中记录每个阶段的进度和结果

## 示例用法

```bash
# 用户输入
/spec-driven-dev --issue 123

# 主 session 执行流程
1. 解析 Issue 编号: 123
2. 启动 Agent 1 (Issue Reader) → 等待完成 → 获取 Issue 数据
3. 启动 Agent 2 (Spec Generator) → 等待完成 → 获取 spec 路径
4. 启动 Agent 3 (Plan Generator) → 等待完成 → 获取 plan 路径
5. 启动 Agent 4 (Tasks Generator) → 等待完成 → 获取 tasks 路径
6. 启动 Agent 5 (Implementation Executor, 后台运行) → 不等待
7. 向用户报告: "实现阶段已启动，正在后台运行"
8. 当 Agent 5 完成时，生成最终总结
```

## 与其他 Skills 的集成

- **上游**: `/create-issue` → 创建 GitHub Issue
- **下游**: `/create-pr` → 创建 Pull Request
- **并行**: `/review-pr` → 代码审查（可在实现完成后自动触发）
