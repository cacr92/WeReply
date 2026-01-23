# 多 Agent 并行工作规范（优雅方案）

> **核心理念**：智能任务分解 + 最小化隔离，避免物理目录混乱

---

## 一、方案选择决策树

```
用户请求并行任务
    ↓
是否真的需要并行？
    ├─ 否 → 串行执行（更简单）
    └─ 是 → 继续
         ↓
    任务是否修改相同文件？
         ├─ 是 → 串行执行或合并任务
         └─ 否 → 继续
              ↓
         任务是否有依赖关系？
              ├─ 是 → 串行执行
              └─ 否 → 可以并行
                   ↓
              选择并行策略：
              ├─ 方案 A：Sub-agents（推荐）
              ├─ 方案 B：Background Tasks
              └─ 方案 C：Git Worktree（仅必要时）
```

---

## 二、推荐方案 A：Sub-agents（无物理隔离）

### 核心思想
- **主 agent** 负责协调和任务分解
- **Sub-agents** 在同一代码库中独立执行任务
- **通过任务分解避免冲突**，而非物理隔离

### 适用场景
✅ 修改不同的模块（如 `material/` vs `species/`）
✅ 修改不同的层次（如前端 vs 后端）
✅ 独立的功能开发
✅ 可以在同一分支上工作的任务

### 工作流程

```markdown
## 步骤 1：主 Agent 分析任务

用户：优化配方系统的性能

主 Agent 分析：
- 任务 1：优化 material_service.rs（后端）
- 任务 2：优化 species_service.rs（后端）
- 任务 3：优化 MaterialList.tsx（前端）

检查冲突：
- ✅ 三个任务修改不同文件
- ✅ 无依赖关系
- ✅ 可以并行

## 步骤 2：启动 Sub-agents

使用 Task 工具启动 3 个 sub-agents：

Task(
  subagent_type: "rust-backend-specialist",
  description: "优化原料服务",
  prompt: "优化 src/material/material_service.rs 的性能，使用缓存和并行计算"
)

Task(
  subagent_type: "rust-backend-specialist",
  description: "优化品种服务",
  prompt: "优化 src/species/species_service.rs 的性能"
)

Task(
  subagent_type: "react-frontend-specialist",
  description: "优化前端列表",
  prompt: "优化 frontend/src/components/MaterialList.tsx 的渲染性能"
)

## 步骤 3：Sub-agents 独立工作

每个 sub-agent：
1. 在同一代码库中工作（无需 worktree）
2. 修改各自负责的文件
3. 运行各自的测试
4. 返回结果给主 agent

## 步骤 4：主 Agent 整合结果

主 Agent：
1. 收集所有 sub-agents 的结果
2. 检查是否有冲突（通常没有，因为修改不同文件）
3. 运行整体测试
4. 提交到同一个 feature 分支
```

### 优势
✅ **无物理目录混乱** - 所有工作在同一代码库
✅ **简单高效** - 无需管理多个 worktree
✅ **自动协调** - Claude Code 自动处理任务分配
✅ **易于合并** - 所有修改在同一分支

### 实现示例

```rust
// Sub-agent 1 修改 src/material/material_service.rs
impl MaterialService {
    pub async fn get_all_cached(&self) -> Result<Vec<Material>> {
        // 添加缓存逻辑
    }
}

// Sub-agent 2 修改 src/species/species_service.rs
impl SpeciesService {
    pub async fn get_all_cached(&self) -> Result<Vec<Species>> {
        // 添加缓存逻辑
    }
}

// Sub-agent 3 修改 frontend/src/components/MaterialList.tsx
export const MaterialList = memo(() => {
    // 添加虚拟滚动
});
```

**关键**：三个文件互不冲突，可以安全地并行修改。

---

## 三、方案 B：Background Tasks（异步执行）

### 核心思想
- 主 agent 继续与用户交互
- 后台任务异步执行
- 适合耗时的独立任务

### 适用场景
✅ 耗时的测试运行
✅ 大规模代码分析
✅ 文档生成
✅ 不需要立即结果的任务

### 工作流程

```markdown
## 启动后台任务

主 Agent：
"我将在后台运行完整的测试套件，同时继续优化代码"

Task(
  subagent_type: "test-automation-specialist",
  description: "运行完整测试",
  prompt: "运行 cargo test 和 npm test，生成覆盖率报告",
  run_in_background: true
)

## 主 Agent 继续工作

主 Agent 继续：
- 优化代码
- 回答用户问题
- 进行其他任务

## 检查后台任务

稍后检查：
TaskOutput(task_id: "test-task-123")
```

### 优势
✅ **不阻塞主流程** - 用户可以继续交互
✅ **充分利用时间** - 耗时任务在后台运行
✅ **无物理隔离** - 在同一代码库工作

---

## 四、方案 C：Git Worktree（仅必要时）

### 何时使用
⚠️ **仅在以下情况使用**：
- 需要同时测试多个不兼容的方案
- 需要长期并行开发（数天）
- 任务之间有潜在的破坏性冲突

### 简化的 Worktree 管理

**使用 Worktrunk 工具**（推荐）：

```bash
# 安装 Worktrunk
npm install -g worktrunk

# 创建 worktree（像创建分支一样简单）
worktrunk create feature-a

# 切换 worktree
worktrunk switch feature-a

# 清理 worktree
worktrunk remove feature-a
```

**手动管理**（如果不想安装工具）：

```bash
# 创建 worktree（在项目内的 .worktrees/ 目录）
git worktree add .worktrees/feature-a -b feature/add-material-import

# 工作
cd .worktrees/feature-a
# 开发...

# 完成后删除
cd ../..
git worktree remove .worktrees/feature-a
```

**关键改进**：
- 使用 `.worktrees/` 子目录而非平级目录
- 保持项目目录整洁
- 在 `.gitignore` 中添加 `.worktrees/`

### 数据库隔离（仅 Worktree 方案需要）

```rust
// src/database/mod.rs
pub fn get_database_path() -> String {
    // 检测是否在 worktree 中
    let current_dir = std::env::current_dir().unwrap();
    let dir_name = current_dir.file_name().unwrap().to_str().unwrap();

    if current_dir.to_str().unwrap().contains(".worktrees/") {
        // 在 worktree 中，使用独立数据库
        format!("data/dev_{}.db", dir_name)
    } else {
        // 主目录，使用默认数据库
        "data/dev.db".to_string()
    }
}
```

---

## 五、任务分解最佳实践

### 1. 识别可并行任务

**✅ 可以并行**：
```markdown
任务：优化系统性能

分解：
- Task 1: 优化 Rust 后端（material_service.rs）
- Task 2: 优化 Rust 后端（species_service.rs）
- Task 3: 优化前端组件（MaterialList.tsx）

理由：修改不同文件，无冲突
```

**❌ 不能并行**：
```markdown
任务：重构 FormulaService

分解：
- Task 1: 重构 optimize_formula 方法
- Task 2: 重构 calculate_nutrition 方法

问题：两个任务都修改 formula_service.rs，会冲突

正确做法：合并为一个任务
```

### 2. 任务粒度控制

| 粒度 | 适合并行 | 示例 |
|------|---------|------|
| **模块级** | ✅ 是 | material/ vs species/ |
| **文件级** | ✅ 是 | service.rs vs repository.rs |
| **函数级** | ❌ 否 | 同一文件的不同函数 |
| **层次级** | ✅ 是 | 前端 vs 后端 |

### 3. 依赖关系检查

```markdown
## 示例：添加新功能

任务 A：创建数据库表（migration）
任务 B：创建 Rust 模型
任务 C：创建 API 端点
任务 D：创建前端组件

依赖关系：
A → B → C → D

结论：必须串行执行
```

---

## 六、冲突检测和解决

### 自动冲突检测

主 Agent 在分配任务前检查：

```markdown
## 冲突检测清单

- [ ] 任务是否修改相同文件？
- [ ] 任务是否有依赖关系？
- [ ] 任务是否需要相同的资源（数据库、端口）？
- [ ] 任务是否会产生不兼容的修改？

如果任何一项为"是"，则不能并行。
```

### 冲突解决策略

| 冲突类型 | 解决方案 |
|---------|---------|
| **文件冲突** | 串行执行或合并任务 |
| **依赖冲突** | 按依赖顺序串行执行 |
| **资源冲突** | 使用资源隔离（如独立数据库） |
| **逻辑冲突** | 重新设计任务分解 |

---

## 七、实战示例

### 示例 1：优化系统性能（推荐方案 A）

```markdown
## 用户请求
"优化整个系统的性能"

## 主 Agent 分析
任务分解：
1. 优化 Rust 后端缓存策略
2. 优化数据库查询
3. 优化前端渲染

冲突检查：
- ✅ 修改不同文件
- ✅ 无依赖关系
- ✅ 可以并行

## 执行方案：Sub-agents（方案 A）

启动 3 个 sub-agents：
- rust-backend-specialist: 添加 Moka 缓存
- database-specialist: 添加索引和优化查询
- react-frontend-specialist: 添加 memo 和虚拟滚动

## 结果
- 所有修改在同一分支
- 无物理目录混乱
- 自动整合结果
```

### 示例 2：运行测试（推荐方案 B）

```markdown
## 用户请求
"运行所有测试并继续开发"

## 执行方案：Background Task（方案 B）

主 Agent：
"我将在后台运行测试，同时继续优化代码"

后台任务：
Task(
  subagent_type: "test-automation-specialist",
  run_in_background: true
)

主 Agent 继续：
- 优化代码
- 回答用户问题

稍后检查测试结果
```

### 示例 3：实验性重构（方案 C - 仅必要时）

```markdown
## 用户请求
"我想尝试两种不同的架构方案，看哪个更好"

## 执行方案：Git Worktree（方案 C）

创建两个 worktree：
1. .worktrees/arch-v1 - 方案 1：微服务架构
2. .worktrees/arch-v2 - 方案 2：模块化单体

每个 worktree 独立开发和测试

完成后比较结果，选择最佳方案

清理未选择的 worktree
```

---

## 八、性能和资源考虑

### 资源消耗对比

| 方案 | CPU | 内存 | 磁盘 | 复杂度 |
|------|-----|------|------|--------|
| **Sub-agents** | 中 | 中 | 低 | 低 |
| **Background Tasks** | 中 | 中 | 低 | 低 |
| **Git Worktree** | 高 | 高 | 高 | 高 |

### 推荐配置

| 并行任务数 | 推荐方案 | 最低配置 |
|-----------|---------|---------|
| 2-3 个 | Sub-agents | 8GB RAM, 4 核 CPU |
| 1 主 + 1 后台 | Background Tasks | 8GB RAM, 4 核 CPU |
| 2-3 个 Worktree | Git Worktree | 16GB RAM, 8 核 CPU, SSD |

---

## 九、决策指南

### 快速决策表

| 场景 | 推荐方案 | 理由 |
|------|---------|------|
| 优化不同模块 | Sub-agents | 简单高效，无冲突 |
| 运行耗时测试 | Background Tasks | 不阻塞主流程 |
| 实验性开发 | Git Worktree | 需要完全隔离 |
| 修改同一文件 | 串行执行 | 避免冲突 |
| 有依赖关系 | 串行执行 | 保证正确性 |

### 何时不使用并行

❌ **不要并行**：
- 任务修改相同文件
- 任务有依赖关系
- 任务需要协调（如 API 契约变更）
- 系统资源不足
- 任务本身很简单（并行开销大于收益）

---

## 十、更新规则文件

### 更新 CLAUDE.md

```markdown
### 特殊流程

**多任务并行**：
1. **优先使用 Sub-agents**（方案 A）- 无物理隔离，简单高效
2. **耗时任务使用 Background Tasks**（方案 B）- 不阻塞主流程
3. **仅必要时使用 Git Worktree**（方案 C）- 完全隔离但复杂

**并行前提**：
- 任务修改不同文件
- 任务无依赖关系
- 系统资源充足

详见：`.claude/rules/09-parallel-agents-standards.md`
```

---

## 十一、总结

### 核心原则
1. **智能分解优于物理隔离** - 通过任务分解避免冲突
2. **简单优于复杂** - 优先使用 Sub-agents
3. **按需隔离** - 仅必要时使用 Worktree
4. **保持整洁** - 避免物理目录混乱

### 推荐优先级
1. 🥇 **Sub-agents**（方案 A）- 90% 的场景
2. 🥈 **Background Tasks**（方案 B）- 耗时任务
3. 🥉 **Git Worktree**（方案 C）- 特殊需求

### 关键收益
✅ 无物理目录混乱
✅ 简单易用
✅ 高效协作
✅ 自动冲突检测
✅ 优雅的用户体验

---

## 参考资源

- [Cursor: Scaling Agents](https://cursor.com/blog/scaling-agents)
- [ClaudeFast: Task Distribution](https://claudefa.st/blog/guide/agents/task-distribution)
- [Worktrunk: Git Worktree Management](https://worktrunk.dev/)
- [Turion: Multi-Agent Orchestration](https://turion.ai/blog/claude-code-multi-agents-subagents-guide)
