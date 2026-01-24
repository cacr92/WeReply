# Auto Merge and Cleanup Skill

## 概述

**auto-merge-and-cleanup** 是 WeReply 项目 14 步开发流程的最后一步（步骤 14），在 `finishing-a-development-branch` 完成后**强制自动执行**。

### 核心功能

本 skill 自动执行以下 6 个关键操作：

1. **检查并提交未提交的更改** - 确保所有本地修改已提交
2. **合并分支到 main** - 将当前 feature 分支合并到 main 分支
3. **推送到远程仓库** - 同步本地 main 分支到远程
4. **删除远程分支** - 删除远程的 feature 分支
5. **删除本地 worktree** - 清理本地 worktree 目录
6. **清理临时文件** - 删除任务跟踪文件和临时文件

### 设计原则

- **自动化优先** - 无需用户手动操作，减少人为错误
- **安全第一** - 合并前检查冲突，失败时保留现场
- **精确清理** - 只删除当前任务创建的分支，不影响其他并行任务
- **完整闭环** - 从开发到合并到清理，形成完整的开发闭环

---

## 执行流程

### 步骤 1：检查未提交的更改

```bash
# 检查是否有未提交的更改
git status --porcelain

# 如果有未提交的更改，自动提交
if [ -n "$(git status --porcelain)" ]; then
  git add .
  git commit -m "chore: auto-commit before merge

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
fi
```

**目的**：确保所有工作都已保存，避免合并时丢失修改。

**错误处理**：
- 如果提交失败（如 pre-commit hook 失败），停止执行并报告错误
- 用户需要手动修复问题后重新执行

### 步骤 2：读取当前分支名称

```bash
# 从临时文件读取分支名称
BRANCH_NAME=$(cat .claude/temp/current-branch.txt)

# 验证分支名称
if [ -z "$BRANCH_NAME" ]; then
  echo "错误：未找到分支跟踪文件"
  exit 1
fi
```

**目的**：获取当前任务创建的分支名称，确保只删除正确的分支。

**安全机制**：
- 使用临时文件跟踪分支，避免误删其他任务的分支
- 验证分支名称非空

### 步骤 3：合并到 main 分支

```bash
# 切换到 main 分支
git checkout main

# 拉取最新的 main 分支
git pull origin main --rebase

# 合并 feature 分支
git merge $BRANCH_NAME --no-ff -m "Merge branch '$BRANCH_NAME'

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

**目的**：将 feature 分支的修改合并到 main 分支。

**参数说明**：
- `--no-ff`：强制创建合并提交，保留分支历史
- `--rebase`：使用 rebase 模式拉取，保持提交历史整洁

**错误处理**：
- 如果合并冲突，停止执行并报告冲突文件
- 用户需要手动解决冲突后重新执行

### 步骤 4：推送到远程仓库

```bash
# 推送 main 分支到远程
git push origin main
```

**目的**：同步本地 main 分支到远程仓库。

**错误处理**：
- 如果推送失败（如网络问题、权限问题），停止执行并报告错误
- 用户需要手动推送或修复问题后重新执行

### 步骤 5：删除远程分支

```bash
# 删除远程 feature 分支
git push origin --delete $BRANCH_NAME
```

**目的**：清理远程仓库中的 feature 分支。

**错误处理**：
- 如果远程分支不存在，忽略错误（可能已被删除）
- 如果删除失败（如权限问题），记录警告但继续执行

### 步骤 6：删除本地 worktree

```bash
# 检测 worktree 路径
if [ -d ".worktrees/$BRANCH_NAME" ]; then
  WORKTREE_PATH=".worktrees/$BRANCH_NAME"
elif [ -d "worktrees/$BRANCH_NAME" ]; then
  WORKTREE_PATH="worktrees/$BRANCH_NAME"
else
  echo "警告：未找到 worktree 目录，跳过删除"
  WORKTREE_PATH=""
fi

# 删除 worktree
if [ -n "$WORKTREE_PATH" ]; then
  git worktree remove $WORKTREE_PATH --force
fi

# 删除本地分支
git branch -D $BRANCH_NAME
```

**目的**：清理本地 worktree 目录和分支。

**参数说明**：
- `--force`：强制删除 worktree，即使有未提交的修改（因为已在步骤 1 提交）
- `-D`：强制删除本地分支

**错误处理**：
- 如果 worktree 不存在，记录警告但继续执行
- 如果删除失败，记录错误但继续执行清理

### 步骤 7：清理临时文件

```bash
# 删除分支跟踪文件
rm -f .claude/temp/current-branch.txt

# 删除任务文件（如果存在）
rm -f .claude/temp/tasks.json

# 删除其他临时文件
rm -f .claude/temp/*.tmp
```

**目的**：清理所有任务相关的临时文件。

**安全机制**：
- 使用 `-f` 参数，即使文件不存在也不报错
- 只删除 `.claude/temp/` 目录下的文件，不影响其他目录

---

## 错误处理

### 合并冲突

**场景**：main 分支和 feature 分支有冲突的修改。

**处理流程**：
1. 停止自动执行
2. 报告冲突文件列表
3. 提示用户手动解决冲突：
   ```bash
   # 查看冲突文件
   git status

   # 手动编辑冲突文件
   # 解决冲突后：
   git add <冲突文件>
   git commit -m "Merge branch 'feature/xxx'"

   # 重新执行 auto-merge-and-cleanup
   ```

### 推送失败

**场景**：网络问题、权限问题、远程分支被保护等。

**处理流程**：
1. 停止自动执行
2. 报告推送失败原因
3. 提示用户手动推送：
   ```bash
   # 检查网络连接
   git remote -v

   # 手动推送
   git push origin main

   # 重新执行 auto-merge-and-cleanup
   ```

### 分支删除失败

**场景**：远程分支不存在、权限不足等。

**处理流程**：
1. 记录警告信息
2. 继续执行后续步骤（不阻塞流程）
3. 提示用户手动删除（如需要）：
   ```bash
   # 手动删除远程分支
   git push origin --delete feature/xxx

   # 手动删除本地分支
   git branch -D feature/xxx
   ```

---

## 使用示例

### 正常流程

```markdown
## 用户完成开发任务

步骤 1-13：正常开发流程
- 创建 worktree
- 需求分析
- 编写计划
- TDD 开发
- 测试
- 代码审查
- 生成 changelog
- 提交代码
- finishing-a-development-branch

## 步骤 14：自动执行（无需用户操作）

AI 自动调用 auto-merge-and-cleanup：

1. ✅ 检查未提交的更改 - 无未提交修改
2. ✅ 读取分支名称 - feature/add-deepseek-api
3. ✅ 合并到 main - 合并成功
4. ✅ 推送到远程 - 推送成功
5. ✅ 删除远程分支 - 删除成功
6. ✅ 删除本地 worktree - 删除成功
7. ✅ 清理临时文件 - 清理成功

## 结果

- main 分支已更新
- 远程仓库已同步
- feature 分支已删除（远程 + 本地）
- worktree 已清理
- 临时文件已删除

用户可以开始下一个任务。
```

### 有未提交修改的流程

```markdown
## 步骤 14：自动执行

AI 自动调用 auto-merge-and-cleanup：

1. ⚠️ 检查未提交的更改 - 发现 3 个文件未提交
   - 自动提交：chore: auto-commit before merge
2. ✅ 读取分支名称 - feature/fix-bug-123
3. ✅ 合并到 main - 合并成功
4. ✅ 推送到远程 - 推送成功
5. ✅ 删除远程分支 - 删除成功
6. ✅ 删除本地 worktree - 删除成功
7. ✅ 清理临时文件 - 清理成功

## 结果

- 未提交的修改已自动提交
- main 分支已更新
- 所有清理工作已完成
```

### 合并冲突的流程

```markdown
## 步骤 14：自动执行

AI 自动调用 auto-merge-and-cleanup：

1. ✅ 检查未提交的更改 - 无未提交修改
2. ✅ 读取分支名称 - feature/refactor-service
3. ❌ 合并到 main - 合并冲突

## 冲突文件

- src/services/deepseek_service.rs
- frontend/src/components/AssistantPanel.tsx

## 用户手动解决冲突

```bash
# 查看冲突
git status

# 编辑冲突文件
# 解决冲突后：
git add src/services/deepseek_service.rs
git add frontend/src/components/AssistantPanel.tsx
git commit -m "Merge branch 'feature/refactor-service'"

# 重新执行 auto-merge-and-cleanup
```

## 重新执行

AI 再次调用 auto-merge-and-cleanup：

1. ✅ 检查未提交的更改 - 无未提交修改
2. ✅ 读取分支名称 - feature/refactor-service
3. ✅ 合并到 main - 已合并（跳过）
4. ✅ 推送到远程 - 推送成功
5. ✅ 删除远程分支 - 删除成功
6. ✅ 删除本地 worktree - 删除成功
7. ✅ 清理临时文件 - 清理成功
```

---

## 安全机制

### 1. 分支跟踪

使用 `.claude/temp/current-branch.txt` 文件跟踪当前任务创建的分支：

```bash
# 在 using-git-worktrees 中创建
echo "feature/add-new-feature" > .claude/temp/current-branch.txt

# 在 auto-merge-and-cleanup 中读取
BRANCH_NAME=$(cat .claude/temp/current-branch.txt)

# 清理后删除
rm -f .claude/temp/current-branch.txt
```

**优势**：
- 精确跟踪当前任务的分支
- 避免误删其他并行任务的分支
- 支持多任务并行开发

### 2. 合并前检查

在合并前执行多项检查：

```bash
# 检查分支是否存在
git rev-parse --verify $BRANCH_NAME

# 检查 main 分支是否是最新的
git fetch origin main
git diff main origin/main

# 检查是否有未提交的修改
git status --porcelain
```

### 3. 失败回滚

如果合并失败，保留现场供用户手动处理：

```bash
# 合并失败时不自动回滚
# 保留冲突状态，让用户解决

# 用户可以选择：
# 1. 解决冲突后继续
# 2. 放弃合并：git merge --abort
```

### 4. 强制执行

本 skill 是 14 步流程的强制步骤，不可跳过：

- `executionOrder: "mandatory-after-step-13"`
- `autoLoad: true`
- `priority: "critical"`

**原因**：
- 确保分支及时合并，避免长期分支
- 自动清理临时文件，保持项目整洁
- 形成完整的开发闭环

---

## 与其他 Skills 的集成

### 前置 Skill

1. **using-git-worktrees** (步骤 1)
   - 创建 worktree 和 feature 分支
   - 写入分支跟踪文件：`.claude/temp/current-branch.txt`

2. **finishing-a-development-branch** (步骤 13)
   - 完成开发工作
   - 提供 PR/合并/继续开发选项
   - 如果用户选择"合并"，触发 auto-merge-and-cleanup

### 后续 Skill

无（本 skill 是最后一步）

### 相关 Skill

- **changelog-generator** (步骤 11)
  - 生成 CHANGELOG.md
  - 本 skill 会将 changelog 一起合并到 main

- **commit** (步骤 12)
  - 提交代码到 feature 分支
  - 本 skill 会将所有提交合并到 main

---

## 配置选项

### 合并策略

默认使用 `--no-ff` 合并策略，可在 `.claude/config.json` 中配置：

```json
{
  "autoMergeCleanup": {
    "mergeStrategy": "no-ff",  // "no-ff" | "ff-only" | "squash"
    "deleteBranch": true,       // 是否删除分支
    "deleteWorktree": true,     // 是否删除 worktree
    "cleanupTempFiles": true    // 是否清理临时文件
  }
}
```

### 推送选项

默认推送到 `origin main`，可配置：

```json
{
  "autoMergeCleanup": {
    "remote": "origin",         // 远程仓库名称
    "mainBranch": "main"        // 主分支名称
  }
}
```

---

## 常见问题

### Q1: 如果我不想自动合并怎么办？

**A**: 本 skill 是强制执行的，不可跳过。如果你想手动控制合并时机，可以：
1. 在步骤 13 (finishing-a-development-branch) 选择"继续开发"
2. 完成所有工作后，手动执行合并
3. 然后手动调用 auto-merge-and-cleanup 清理

### Q2: 如果合并冲突怎么办？

**A**:
1. 本 skill 会停止执行并报告冲突文件
2. 你需要手动解决冲突
3. 解决后重新执行 auto-merge-and-cleanup

### Q3: 如果我有多个并行任务怎么办？

**A**:
- 本 skill 使用 `.claude/temp/current-branch.txt` 跟踪当前任务的分支
- 只会删除当前任务创建的分支
- 不会影响其他并行任务的分支

### Q4: 如果推送失败怎么办？

**A**:
1. 检查网络连接
2. 检查远程仓库权限
3. 手动推送：`git push origin main`
4. 重新执行 auto-merge-and-cleanup

### Q5: 如果我想保留 feature 分支怎么办？

**A**:
- 本 skill 会自动删除 feature 分支
- 如果你想保留，可以在执行前手动备份：
  ```bash
  git branch feature/xxx-backup feature/xxx
  ```

---

## 测试

使用 `test-workflow.sh` 脚本测试完整流程：

```bash
# 运行测试
bash .claude/skills/auto-merge-and-cleanup/test-workflow.sh

# 测试内容：
# 1. 创建测试分支
# 2. 模拟开发工作
# 3. 执行 auto-merge-and-cleanup
# 4. 验证合并结果
# 5. 验证分支删除
# 6. 验证临时文件清理
```

---

## 总结

**auto-merge-and-cleanup** 是 WeReply 项目 14 步开发流程的最后一步，负责：

✅ 自动合并 feature 分支到 main
✅ 同步远程仓库
✅ 删除远程和本地分支
✅ 清理 worktree 目录
✅ 清理临时文件
✅ 形成完整的开发闭环

**核心优势**：
- 自动化：无需手动操作
- 安全：合并前检查，失败时保留现场
- 精确：只删除当前任务的分支
- 完整：从开发到合并到清理，一气呵成

**强制执行**：
- 不可跳过
- 自动触发
- 确保项目整洁
