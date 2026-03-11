---
name: create-issue
description: Automatically create GitHub issues from feasibility reports or user requirements. Use after feasibility analysis or when user wants to track work in GitHub.
compatibility: Requires GitHub repository with gh CLI configured
metadata:
  author: vanguard-workflow
  version: 1.0.0
---

# Create Issue Skill

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Outline

### 1. 确定 Issue 来源

检查是否存在可行性报告：

```bash
# 检查是否存在 feasibility-report.md
if [ -f "feasibility-report.md" ]; then
  echo "Found feasibility report"
else
  echo "No feasibility report found"
fi
```

**两种模式**:
- **模式 A**: 基于 `feasibility-report.md` 创建 Issue（推荐）
- **模式 B**: 直接基于用户输入创建 Issue（快速模式）

### 2. 提取 Issue 内容

#### 2.1 从可行性报告提取（模式 A）

如果存在 `feasibility-report.md`，读取并提取：
- **标题**: 从"需求描述"生成简洁标题（不超过 60 字符）
- **描述**: 整合报告的关键部分
- **标签**: 根据复杂度和风险自动生成
- **里程碑**: 根据预估工时建议里程碑

#### 2.2 从用户输入提取（模式 B）

如果没有可行性报告，直接从 `$ARGUMENTS` 提取：
- **标题**: 从用户输入生成简洁标题
- **描述**: 使用用户原始输入
- **标签**: 默认使用 `enhancement`

### 3. 生成 Issue 模板

使用以下结构生成 Issue 内容：

```markdown
## 📋 需求描述

[从可行性报告或用户输入提取的需求描述]

## 🎯 目标

[明确的、可衡量的目标]

## 📊 复杂度评估

**总体复杂度**: [简单/中等/复杂]
**预估工时**: [X-Y 天]

| 维度 | 评分 | 说明 |
|------|------|------|
| 代码量 | X/5 | ... |
| 技术难度 | X/5 | ... |
| 测试复杂度 | X/5 | ... |
| 风险等级 | X/5 | ... |

## 💡 实现方案

### 推荐方案: [方案名称]

**核心思路**:
[简要描述]

**关键步骤**:
1. [步骤 1]
2. [步骤 2]
3. [步骤 3]

**技术栈**:
- [技术 1]
- [技术 2]

## ⚠️ 风险与注意事项

### 高风险 🔴
- [风险项 1]

### 中风险 🟡
- [风险项 2]

### 低风险 🟢
- [风险项 3]

## ✅ 验收标准

- [ ] [标准 1]
- [ ] [标准 2]
- [ ] [标准 3]
- [ ] 代码通过所有测试
- [ ] 代码符合 SOLID、KISS、YAGNI、DRY 原则
- [ ] 文档已更新（README、CHANGELOG 等）

## 📚 相关资源

- 可行性报告: [链接到 feasibility-report.md]
- 相关 Issue: #[编号]
- 相关 PR: #[编号]

---

**生成方式**: 使用 `/create-issue` 自动生成
**下一步**: 使用 `/spec-driven-dev --issue [编号]` 开始开发
```

### 4. 确定 Issue 标签

根据以下规则自动生成标签：

#### 4.1 类型标签（必选一个）
- `enhancement`: 新功能
- `bug`: 错误修复
- `refactor`: 代码重构
- `documentation`: 文档更新
- `performance`: 性能优化
- `security`: 安全相关

#### 4.2 复杂度标签（必选一个）
- `complexity: simple`: 总体复杂度 1.0-2.0
- `complexity: medium`: 总体复杂度 2.1-3.5
- `complexity: complex`: 总体复杂度 3.6-5.0

#### 4.3 优先级标签（可选）
- `priority: high`: 高优先级
- `priority: medium`: 中优先级
- `priority: low`: 低优先级

#### 4.4 风险标签（可选）
- `risk: high`: 存在高风险项
- `risk: medium`: 存在中风险项

### 5. 验证 GitHub 环境

在创建 Issue 前，执行以下检查：

```bash
# 1. 检查是否在 Git 仓库中
if ! git rev-parse --git-dir > /dev/null 2>&1; then
  echo "ERROR: Not in a git repository"
  exit 1
fi

# 2. 检查是否有 GitHub remote
REMOTE_URL=$(git config --get remote.origin.url)
if [[ ! "$REMOTE_URL" =~ github\.com ]]; then
  echo "ERROR: Remote is not a GitHub repository"
  echo "Remote URL: $REMOTE_URL"
  exit 1
fi

# 3. 检查 gh CLI 是否已安装和认证
if ! command -v gh &> /dev/null; then
  echo "ERROR: gh CLI not installed"
  echo "Install: brew install gh (macOS) or see https://cli.github.com"
  exit 1
fi

if ! gh auth status &> /dev/null; then
  echo "ERROR: gh CLI not authenticated"
  echo "Run: gh auth login"
  exit 1
fi

echo "✅ GitHub environment validated"
```

> [!CAUTION]
> **ONLY PROCEED IF ALL CHECKS PASS**

### 6. 创建 GitHub Issue

使用 `gh` CLI 创建 Issue：

```bash
# 生成 Issue 内容到临时文件
ISSUE_BODY=$(cat <<'EOF'
[生成的 Issue 模板内容]
EOF
)

# 创建 Issue
ISSUE_URL=$(gh issue create \
  --title "[生成的标题]" \
  --body "$ISSUE_BODY" \
  --label "enhancement,complexity: medium" \
  --assignee "@me")

# 提取 Issue 编号
ISSUE_NUMBER=$(echo "$ISSUE_URL" | grep -oE '[0-9]+$')

echo "✅ Issue created: $ISSUE_URL"
echo "Issue number: #$ISSUE_NUMBER"
```

### 7. 更新可行性报告（如果存在）

如果基于可行性报告创建 Issue，在报告末尾添加链接：

```bash
if [ -f "feasibility-report.md" ]; then
  echo "" >> feasibility-report.md
  echo "---" >> feasibility-report.md
  echo "" >> feasibility-report.md
  echo "**GitHub Issue**: $ISSUE_URL" >> feasibility-report.md
  echo "**Issue Number**: #$ISSUE_NUMBER" >> feasibility-report.md
  echo "**创建时间**: $(date '+%Y-%m-%d %H:%M:%S')" >> feasibility-report.md
fi
```

### 8. 输出总结

向用户展示：

```markdown
✅ **GitHub Issue 创建成功**

**Issue 信息**:
- 标题: [标题]
- 编号: #[编号]
- 链接: [URL]
- 标签: [标签列表]

**下一步操作**:
1. 查看 Issue: `gh issue view [编号]`
2. 开始开发: `/spec-driven-dev --issue [编号]`
3. 查看所有 Issue: `gh issue list`

**快捷命令**:
```bash
# 在浏览器中打开 Issue
gh issue view [编号] --web

# 添加评论
gh issue comment [编号] --body "评论内容"

# 关闭 Issue
gh issue close [编号]
```
```

## 高级功能

### 自动关联相关 Issue

搜索相关的现有 Issue：

```bash
# 搜索关键词相关的 Issue
gh issue list --search "[关键词]" --state all --limit 5
```

如果找到相关 Issue，在新 Issue 中添加引用：

```markdown
## 相关 Issue

- Related to #123
- Depends on #456
- Blocks #789
```

### 自动分配给当前用户

默认将 Issue 分配给当前用户：

```bash
gh issue create --assignee "@me"
```

如果用户指定了其他负责人，使用：

```bash
gh issue create --assignee "username"
```

### 支持里程碑

如果项目有里程碑，根据预估工时自动分配：

```bash
# 列出可用里程碑
gh api repos/:owner/:repo/milestones --jq '.[].title'

# 创建 Issue 时指定里程碑
gh issue create --milestone "v1.0.0"
```

## 错误处理

### 常见错误及解决方案

| 错误 | 原因 | 解决方案 |
|------|------|----------|
| `gh: command not found` | gh CLI 未安装 | `brew install gh` (macOS) |
| `gh auth status` 失败 | 未认证 | `gh auth login` |
| `Not a GitHub repository` | Remote 不是 GitHub | 检查 `git remote -v` |
| `Permission denied` | 无权限创建 Issue | 检查仓库权限 |
| `Rate limit exceeded` | API 调用过多 | 等待或使用 Personal Access Token |

### 回退机制

如果 `gh` CLI 失败，提供手动创建 Issue 的指导：

```markdown
⚠️ **自动创建失败，请手动创建 Issue**

1. 访问: https://github.com/[owner]/[repo]/issues/new
2. 复制以下内容到 Issue 表单:

**标题**:
[生成的标题]

**描述**:
[生成的 Issue 内容]

**标签**:
[标签列表]
```

## 注意事项

1. **Issue 标题规范**: 使用动词开头，简洁明了（如："添加用户登录功能"）
2. **标签一致性**: 确保标签与项目现有标签体系一致
3. **避免重复**: 创建前搜索是否存在类似 Issue
4. **信息完整性**: 确保 Issue 包含足够的上下文信息
5. **可追溯性**: 保持可行性报告与 Issue 的双向链接

## 示例用法

### 示例 1: 基于可行性报告创建 Issue

```bash
# 前提：已运行 /analyze-feasibility
/create-issue

# AI 执行流程
1. 读取 feasibility-report.md
2. 提取关键信息（标题、描述、复杂度、风险）
3. 生成 Issue 模板
4. 验证 GitHub 环境
5. 使用 gh CLI 创建 Issue
6. 更新可行性报告，添加 Issue 链接
7. 输出 Issue 信息和下一步建议
```

### 示例 2: 快速创建 Issue（无可行性报告）

```bash
/create-issue 添加用户登录功能，支持邮箱和密码登录

# AI 执行流程
1. 检测无可行性报告，使用快速模式
2. 从用户输入生成简化的 Issue 模板
3. 使用默认标签（enhancement）
4. 创建 Issue
5. 输出 Issue 信息
```

### 示例 3: 指定标签和负责人

```bash
/create-issue --labels "bug,priority: high" --assignee "username" 修复支付超时问题

# AI 执行流程
1. 解析命令行参数
2. 使用指定的标签和负责人
3. 创建 Issue
```

## 与其他 Skills 的集成

- **上游**: `/analyze-feasibility` → 生成可行性报告
- **下游**: `/spec-driven-dev --issue [编号]` → 开始开发
- **并行**: `/review-pr` → 代码审查（未来实现）
