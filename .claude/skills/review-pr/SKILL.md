---
name: review-pr
description: Perform comprehensive code review on GitHub Pull Request. Analyzes code quality, security, performance, and adherence to SOLID/KISS/YAGNI/DRY principles. Provides actionable feedback.
compatibility: Requires GitHub repository with gh CLI configured
metadata:
  author: vanguard-workflow
  version: 1.0.0
  architecture: orchestrator
---

# Review Pull Request Skill (Orchestrator)

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Architecture

此 skill 采用 **Orchestrator Pattern**，主 session 负责调度，实际工作由 sub-agents 执行：

```
Main Session (Orchestrator)
    ├─> Agent 1: PR Fetcher (获取 PR 信息和 diff)
    ├─> Agent 2: Code Quality Analyzer (代码质量分析)
    ├─> Agent 3: Security Scanner (安全漏洞扫描)
    ├─> Agent 4: Performance Checker (性能影响分析)
    └─> Agent 5: Review Reporter (生成审查报告)
```

## Outline

### 1. 解析 PR 编号

从用户输入中提取 PR 编号：

```bash
# 支持多种输入格式
# /review-pr --pr 456
# /review-pr #456
# /review-pr 456
# /review-pr https://github.com/owner/repo/pull/456

PR_NUMBER=$(echo "$ARGUMENTS" | grep -oE '[0-9]+' | head -1)

if [ -z "$PR_NUMBER" ]; then
  echo "ERROR: No PR number provided"
  echo "Usage: /review-pr --pr <number>"
  exit 1
fi
```

### 2. 启动 Agent 1: PR Fetcher

**任务**: 获取 PR 信息和代码变更

```markdown
Launch Agent with:
- description: "Fetch PR information"
- prompt: "
  Fetch Pull Request information and code changes.

  **PR Number**: $PR_NUMBER

  Steps:
  1. Verify GitHub environment:
     - Check gh CLI is installed and authenticated
     - Verify PR #$PR_NUMBER exists

  2. Get PR metadata:
     ```bash
     gh pr view $PR_NUMBER --json number,title,body,state,author,labels,reviewDecision,commits,additions,deletions,changedFiles
     ```

  3. Get PR diff:
     ```bash
     gh pr diff $PR_NUMBER
     ```

  4. Get list of changed files:
     ```bash
     gh pr view $PR_NUMBER --json files --jq '.files[].path'
     ```

  5. Get commit messages:
     ```bash
     gh pr view $PR_NUMBER --json commits --jq '.commits[].messageHeadline'
     ```

  6. Check CI/CD status:
     ```bash
     gh pr checks $PR_NUMBER
     ```

  Return JSON:
  {
    \"status\": \"success\" | \"error\",
    \"pr_number\": $PR_NUMBER,
    \"title\": \"feat: Add user authentication\",
    \"author\": \"username\",
    \"state\": \"open\",
    \"labels\": [\"enhancement\", \"complexity: medium\"],
    \"commits\": 8,
    \"additions\": 450,
    \"deletions\": 120,
    \"changed_files\": 12,
    \"files\": [\"src/auth.rs\", \"src/user.rs\", ...],
    \"diff\": \"[full diff content]\",
    \"ci_status\": \"passing\" | \"failing\" | \"pending\",
    \"error\": \"error message if status is error\"
  }
"
```

**等待 Agent 1 完成**。

**错误处理**:
- 如果 PR 不存在，停止流程并提示用户
- 如果无法获取 diff，尝试使用 git 命令获取

### 3. 启动 Agent 2: Code Quality Analyzer

**任务**: 分析代码质量和设计原则

**输入**: Agent 1 返回的 diff 和文件列表

```markdown
Launch Agent with:
- description: "Analyze code quality"
- prompt: "
  Analyze code quality and adherence to design principles.

  **Context**:
  - PR: #$PR_NUMBER
  - Changed files: [files from Agent 1]
  - Diff: [diff from Agent 1]

  Steps:
  1. Read each changed file using Read tool

  2. Analyze code against SOLID principles:
     - **S (Single Responsibility)**: 每个类/函数是否只有一个职责？
     - **O (Open/Closed)**: 是否易于扩展但不需修改现有代码？
     - **L (Liskov Substitution)**: 子类是否可以替换父类？
     - **I (Interface Segregation)**: 接口是否精简专注？
     - **D (Dependency Inversion)**: 是否依赖抽象而非具体实现？

  3. Analyze code against other principles:
     - **KISS (Keep It Simple)**: 代码是否简洁易懂？
     - **YAGNI (You Aren't Gonna Need It)**: 是否有过度设计？
     - **DRY (Don't Repeat Yourself)**: 是否有重复代码？

  4. Check code quality:
     - 命名规范（变量、函数、类名是否清晰）
     - 函数长度（是否过长，建议 < 50 行）
     - 代码复杂度（是否有过深的嵌套）
     - 注释质量（是否有必要的注释，是否过多无用注释）
     - 错误处理（是否完善）

  5. Identify code smells:
     - 长函数 (Long Method)
     - 大类 (Large Class)
     - 重复代码 (Duplicated Code)
     - 过长参数列表 (Long Parameter List)
     - 发散式变化 (Divergent Change)
     - 霰弹式修改 (Shotgun Surgery)
     - 依恋情结 (Feature Envy)
     - 数据泥团 (Data Clumps)

  6. Use simplify skill to get additional insights:
     Call /simplify on the changed code

  Return JSON:
  {
    \"status\": \"success\" | \"error\",
    \"solid_violations\": [
      {\"principle\": \"SRP\", \"file\": \"src/auth.rs\", \"line\": 45, \"issue\": \"AuthService handles both authentication and logging\", \"severity\": \"medium\"}
    ],
    \"principle_violations\": [
      {\"principle\": \"DRY\", \"file\": \"src/user.rs\", \"line\": 120, \"issue\": \"Duplicate validation logic\", \"severity\": \"high\"}
    ],
    \"code_smells\": [
      {\"smell\": \"Long Method\", \"file\": \"src/auth.rs\", \"function\": \"authenticate\", \"lines\": 80, \"severity\": \"medium\"}
    ],
    \"quality_issues\": [
      {\"category\": \"naming\", \"file\": \"src/user.rs\", \"line\": 30, \"issue\": \"Variable 'x' is not descriptive\", \"severity\": \"low\"}
    ],
    \"suggestions\": [
      \"Split AuthService into AuthService and LoggingService\",
      \"Extract duplicate validation logic into a shared function\",
      \"Rename variable 'x' to 'userId'\"
    ],
    \"overall_score\": 7.5,
    \"error\": \"error message if status is error\"
  }
"
```

**等待 Agent 2 完成**。

### 4. 启动 Agent 3: Security Scanner

**任务**: 扫描安全漏洞

**输入**: Agent 1 返回的 diff 和文件列表

```markdown
Launch Agent with:
- description: "Scan for security issues"
- prompt: "
  Scan code for security vulnerabilities.

  **Context**:
  - PR: #$PR_NUMBER
  - Changed files: [files from Agent 1]
  - Diff: [diff from Agent 1]

  Steps:
  1. Check for common security vulnerabilities:
     - **SQL Injection**: 是否有未参数化的 SQL 查询？
     - **XSS (Cross-Site Scripting)**: 是否有未转义的用户输入？
     - **CSRF (Cross-Site Request Forgery)**: 是否缺少 CSRF 保护？
     - **Command Injection**: 是否有未验证的命令执行？
     - **Path Traversal**: 是否有未验证的文件路径？
     - **Insecure Deserialization**: 是否有不安全的反序列化？
     - **Hardcoded Secrets**: 是否有硬编码的密码、API key？
     - **Weak Cryptography**: 是否使用弱加密算法？

  2. Check authentication and authorization:
     - 是否有未授权的访问？
     - 是否正确验证用户权限？
     - 是否有会话管理问题？

  3. Check data handling:
     - 敏感数据是否加密存储？
     - 是否有数据泄露风险？
     - 是否正确处理用户输入验证？

  4. Check dependencies:
     - 是否引入了有已知漏洞的依赖？
     - 使用 Grep 搜索 Cargo.toml 或 package.json 中的依赖版本

  5. Check for OWASP Top 10 vulnerabilities

  Return JSON:
  {
    \"status\": \"success\" | \"error\",
    \"vulnerabilities\": [
      {\"type\": \"SQL Injection\", \"file\": \"src/db.rs\", \"line\": 45, \"severity\": \"critical\", \"description\": \"Unparameterized SQL query\", \"recommendation\": \"Use parameterized queries\"}
    ],
    \"secrets_found\": [
      {\"file\": \".env.example\", \"line\": 10, \"type\": \"API Key\", \"severity\": \"high\"}
    ],
    \"dependency_issues\": [
      {\"package\": \"old-crate\", \"version\": \"0.1.0\", \"vulnerability\": \"CVE-2023-1234\", \"severity\": \"high\"}
    ],
    \"overall_risk\": \"low\" | \"medium\" | \"high\" | \"critical\",
    \"error\": \"error message if status is error\"
  }
"
```

**等待 Agent 3 完成**。

### 5. 启动 Agent 4: Performance Checker

**任务**: 分析性能影响

**输入**: Agent 1 返回的 diff 和文件列表

```markdown
Launch Agent with:
- description: "Check performance impact"
- prompt: "
  Analyze potential performance impacts of the changes.

  **Context**:
  - PR: #$PR_NUMBER
  - Changed files: [files from Agent 1]
  - Diff: [diff from Agent 1]

  Steps:
  1. Identify performance-critical code:
     - 循环中的操作
     - 数据库查询
     - 网络请求
     - 文件 I/O
     - 内存分配

  2. Check for performance anti-patterns:
     - **N+1 查询**: 循环中的数据库查询
     - **不必要的克隆**: 过多的 .clone() 调用
     - **阻塞操作**: 同步操作阻塞异步代码
     - **内存泄漏**: 未释放的资源
     - **低效算法**: 时间复杂度过高

  3. Check resource usage:
     - 是否有大量内存分配？
     - 是否有未关闭的连接？
     - 是否有长时间运行的操作？

  4. Suggest optimizations:
     - 使用缓存
     - 批量操作
     - 异步处理
     - 索引优化
     - 算法优化

  Return JSON:
  {
    \"status\": \"success\" | \"error\",
    \"performance_issues\": [
      {\"type\": \"N+1 Query\", \"file\": \"src/user.rs\", \"line\": 120, \"severity\": \"high\", \"description\": \"Database query in loop\", \"recommendation\": \"Use batch query\"}
    ],
    \"optimizations\": [
      {\"file\": \"src/cache.rs\", \"line\": 45, \"suggestion\": \"Add caching for frequently accessed data\", \"estimated_improvement\": \"50% faster\"}
    ],
    \"overall_impact\": \"positive\" | \"neutral\" | \"negative\",
    \"error\": \"error message if status is error\"
  }
"
```

**等待 Agent 4 完成**。

### 6. 启动 Agent 5: Review Reporter

**任务**: 生成综合审查报告

**输入**: Agent 1-4 的所有结果

```markdown
Launch Agent with:
- description: "Generate review report"
- prompt: "
  Generate a comprehensive code review report.

  **Context**:
  - PR: #$PR_NUMBER
  - PR Info: [from Agent 1]
  - Code Quality: [from Agent 2]
  - Security: [from Agent 3]
  - Performance: [from Agent 4]

  Steps:
  1. Aggregate all findings

  2. Prioritize issues by severity:
     - Critical: 必须修复才能合并
     - High: 强烈建议修复
     - Medium: 建议修复
     - Low: 可选修复

  3. Generate review report using this template:

     ```markdown
     # Code Review Report: PR #$PR_NUMBER

     **PR 标题**: [title]
     **作者**: [author]
     **状态**: [state]
     **CI/CD**: [ci_status]

     ---

     ## 📊 变更概览

     - **提交数量**: [commits]
     - **修改文件**: [changed_files] 个
     - **新增代码**: +[additions] 行
     - **删除代码**: -[deletions] 行

     ---

     ## 🎯 总体评价

     **代码质量评分**: [overall_score]/10

     **总体风险等级**: [overall_risk]

     **推荐操作**: ✅ 批准合并 | ⚠️ 需要修改 | ❌ 拒绝合并

     ---

     ## 🔴 Critical Issues (必须修复)

     [列出所有 critical severity 的问题]

     ### 1. [Issue Title]
     - **文件**: `[file]:[line]`
     - **类型**: [type]
     - **描述**: [description]
     - **建议**: [recommendation]

     ---

     ## 🟡 High Priority Issues (强烈建议修复)

     [列出所有 high severity 的问题]

     ---

     ## 🟢 Medium/Low Priority Issues (可选修复)

     [列出所有 medium/low severity 的问题]

     ---

     ## ✅ 优点

     - [列出代码的优点]
     - [好的设计模式]
     - [清晰的命名]

     ---

     ## 💡 改进建议

     ### 代码质量
     [from Agent 2 suggestions]

     ### 安全性
     [from Agent 3 recommendations]

     ### 性能
     [from Agent 4 optimizations]

     ---

     ## 📋 Checklist

     - [ ] 代码符合 SOLID 原则
     - [ ] 遵循 KISS 和 YAGNI 原则
     - [ ] 无重复代码 (DRY)
     - [ ] 无安全漏洞
     - [ ] 性能影响可接受
     - [ ] 测试覆盖充分
     - [ ] 文档已更新
     - [ ] 无硬编码的敏感信息

     ---

     ## 🔗 相关资源

     - PR: #$PR_NUMBER
     - 关联 Issue: [linked issues]
     - CI/CD: [ci link]

     ---

     **审查时间**: [timestamp]
     **审查工具**: Claude Code - /review-pr
     ```

  4. Post review as PR comment:
     ```bash
     gh pr comment $PR_NUMBER --body \"[review report]\"
     ```

  5. If critical issues found, request changes:
     ```bash
     gh pr review $PR_NUMBER --request-changes --body \"发现 [N] 个 critical issues，请修复后重新提交\"
     ```

  6. If no critical issues, approve (if user confirms):
     ```bash
     gh pr review $PR_NUMBER --approve --body \"代码审查通过，建议合并\"
     ```

  Return JSON:
  {
    \"status\": \"success\" | \"error\",
    \"report\": \"[full review report]\",
    \"critical_count\": 2,
    \"high_count\": 5,
    \"medium_count\": 8,
    \"low_count\": 3,
    \"recommendation\": \"approve\" | \"request_changes\" | \"comment\",
    \"posted\": true,
    \"error\": \"error message if status is error\"
  }
"
```

**等待 Agent 5 完成**。

### 7. 生成最终总结

当所有 Agents 完成后，主 session 生成总结：

```markdown
✅ **代码审查完成**

**PR 信息**:
- 编号: #$PR_NUMBER
- 标题: [title]
- 作者: [author]

**审查结果**:
- 🔴 Critical Issues: [count]
- 🟡 High Priority: [count]
- 🟢 Medium/Low: [count]

**代码质量评分**: [score]/10

**总体风险**: [risk level]

**推荐操作**: [recommendation]

**审查报告已发布**: [PR comment link]

**下一步**:
```bash
# 查看完整报告
gh pr view $PR_NUMBER --comments

# 如果需要修改
# 作者修复问题后，重新审查
/review-pr --pr $PR_NUMBER

# 如果通过审查，合并 PR
gh pr merge $PR_NUMBER --squash --delete-branch
```
```

## 高级功能

### 自动化审查规则

支持自定义审查规则：

```yaml
# .claude/review-rules.yaml
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

### 增量审查

仅审查新增或修改的代码，跳过未变更的部分：

```bash
# 仅审查 diff 中的变更
# 不审查整个文件
```

### 审查历史

记录审查历史，跟踪问题修复：

```bash
# 保存审查结果到 .claude/reviews/
# 文件名: pr-$PR_NUMBER-[timestamp].md
```

### 自动标签

根据审查结果自动添加标签：

```bash
if critical_count > 0; then
  gh pr edit $PR_NUMBER --add-label "needs-work"
elif high_count > 5; then
  gh pr edit $PR_NUMBER --add-label "needs-improvement"
else
  gh pr edit $PR_NUMBER --add-label "ready-to-merge"
fi
```

## 与其他 Skills 的集成

- **上游**: `/create-pr` → 创建 PR 后自动触发审查
- **下游**: 审查通过后，可以自动合并 PR
- **并行**: 可以与 CI/CD 检查并行运行

## 注意事项

1. **客观公正**: 基于代码事实进行审查，不带个人偏见
2. **建设性反馈**: 提供具体的改进建议，而非仅指出问题
3. **优先级明确**: 区分必须修复和可选修复的问题
4. **尊重作者**: 认可代码的优点，鼓励良好实践
5. **可操作性**: 每个问题都应提供具体的修复建议

## 示例用法

```bash
# 用户输入
/review-pr --pr 456

# 主 session 执行流程
1. 解析 PR 编号: 456
2. 启动 Agent 1 (PR Fetcher) - 获取 PR 信息和 diff
3. 启动 Agent 2 (Code Quality Analyzer) - 分析代码质量
4. 启动 Agent 3 (Security Scanner) - 扫描安全漏洞
5. 启动 Agent 4 (Performance Checker) - 检查性能影响
6. 启动 Agent 5 (Review Reporter) - 生成审查报告
7. 发布审查报告到 PR
8. 生成最终总结
```

## 性能优化

1. **并行分析**: Agent 2、3、4 可以并行执行
2. **缓存 diff**: 避免重复获取相同的 diff
3. **增量审查**: 仅审查变更的代码
4. **智能跳过**: 跳过自动生成的文件（如 lock files）
