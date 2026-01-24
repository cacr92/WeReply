# WeReply - 微信回复建议助手 - Claude Code 核心规则

> **快速参考**: 本文件包含核心工作流和编程规范。详细技术规范请参考 `.claude/rules/` 目录。

---

## AI 行为准则

### 对话启动要求

每次新对话开始时，AI 必须：

1. **重新完整阅读并理解本规则文件的所有内容**
2. **在回复用户之前，内化所有编码准则和执行协议**
3. **严格按照本规则文件中定义的开发流程执行任务**
4. **在整个对话过程中始终遵守这些规则，不得偏离**

### 沟通规范

- **问候语**：对话开始时，以"你好，主人！+你的模型名字"作为开场白
- **沟通语言**：必须使用中文进行交流

---

## 任务处理流程

### 非开发任务（咨询、解释、分析）

1. **需求分析** - 列出需求点
2. **任务分解** - 分解为具体子任务
3. **执行说明** - 说明正在执行的操作
4. **标记进度** - 实时标记完成状态

### 开发任务（代码修改、新功能、Bug修复）

**必须执行"完整开发闭环流程（14步）"**，详见下方。

---

## 完整开发闭环流程（14步强制执行）

**核心原则**：

- ✅ 使用 feature 分支开发，**永不直接修改 main 分支**
- ✅ 自动生成 changelog 和提交信息
- ✅ 所有代码提交到新分支，由用户本地验证后合并
- ✅ 根据修改的文件类型自动调用项目特定 skills

---

### 14步开发流程

#### 步骤 1：创建隔离分支（using-git-worktrees）

**触发时机**：所有涉及代码修改的任务开始时

```bash
调用 Skill(skill: "using-git-worktrees")
```

#### 步骤 2：需求分析与设计（brainstorming）

**触发条件**：新功能开发

```bash
调用 Skill(skill: "brainstorming")
```

#### 步骤 3：编写实现计划（writing-plans）

**触发条件**：复杂功能或重构任务

```bash
调用 Skill(skill: "writing-plans")
```

#### 步骤 4：项目特定技术栈检查（自动识别）

**自动识别规则**：

| 文件特征                       | 自动调用 Skill                   | 用途                  |
| ------------------------------ | -------------------------------- | --------------------- |
| `src/wechat/**/*.rs`         | `wechat-automation`            | 微信监听与自动化规范  |
| `src/ai/**/*.rs`             | `deepseek-integration`         | DeepSeek API 集成规范 |
| `src/orchestrator/**/*.rs`   | `ipc-communication`            | IPC 通信协议规范      |
| `platform_agents/**/*.py`    | `python-agent-development`     | Python Agent 开发规范 |
| `platform_agents/**/*.swift` | `macos-agent-development`      | macOS Agent 开发规范  |
| `frontend/src/**/*.tsx`      | `react-typescript-development` | React TypeScript 规范 |
| `src/**/commands.rs`         | `tauri-development`            | Tauri 命令规范        |
| `src/**/*.rs`（其他）        | `rust-optimization`            | Rust 性能优化         |
| 任何 API 设计                  | `api-design`                   | API 设计规范          |

#### 步骤 5：TDD - 先写测试（test-driven-development）

```bash
调用 Skill(skill: "test-driven-development")
```

**要求**：先写失败的测试，确保测试覆盖率 ≥ 80%

#### 步骤 6：实现代码

**强制要求**：

- 遵循所有编码准则
- 使用 LSP 工具理解现有代码结构
- 保持原子化提交原则

#### 步骤 7-10：质量保障流程

7. **security-review** - 检查 API 密钥管理、IPC 安全、输入验证
8. **check** - 运行 `cargo clippy` 和 `npm run lint`
9. **optimize** - 分析性能瓶颈（监听延迟、API 响应）
10. **verification-before-completion** - 运行所有测试，验证覆盖率

#### 步骤 11-14：提交和完成

11. **changelog-generator** - 自动生成 CHANGELOG.md 条目
12. **commit** - 生成规范的提交信息（Conventional Commits）
13. **finishing-a-development-branch** - 提供 PR/合并/继续开发选项
14. **auto-merge-and-cleanup** - 自动合并到 main，清理分支和临时文件（强制执行）

---

### 特殊流程

**Bug 修复流程（12步）**：跳过步骤2和3，使用 `systematic-debugging` 替代

**多任务并行**：智能任务分解 + 最小化隔离

- **方案 **：使用 Sub-agents - 无物理隔离，简单高效

**并行前提**：

- 任务修改不同文件
- 任务无依赖关系
- 系统资源充足

详见：`.claude/rules/09-parallel-agents-standards.md`

---

### 强制要求

1. **必须显式调用 Skill 工具** - ✅ `Skill(skill: "xxx")` / ✗ 仅在文字中说明
2. **不得跳过任何步骤** - 即使是简单任务也必须执行完整流程
3. **分支保护** - 绝对禁止直接在 main 分支上修改代码
4. **自动识别技术栈** - 分析文件路径，自动调用相应 skills

---

## 核心编码准则

### 严禁事项（桌面应用专项）

1. **禁止 `console.log/error/warn`** → 使用 `message.error/success/warning`
2. **禁止 `as any` 类型转换** → 使用精确类型断言或类型守卫
3. **禁止原始 `invoke`** → 使用生成的 `commands`
4. **禁止在循环中使用 Hooks**
5. **禁止硬编码 API 密钥** → 使用系统密钥链存储
6. **禁止阻塞异步运行时** → 使用 `tokio::spawn` 或 `tokio::time::sleep`

### 必须遵守

1. **使用 specta 导出类型** - 所有 Tauri 命令必须有 `#[specta::specta]` 宏
2. **使用 `message` 组件** - 所有用户反馈通过 Ant Design `message` 组件
3. **使用 tracing 日志** - 使用结构化日志记录
4. **使用 `anyhow::Result + ApiResponse`** - 底层 anyhow，顶层 ApiResponse
5. **使用 JSON 进行 IPC 通信** - Rust ↔ Agent 使用 JSON 协议
6. **处理 Agent 异常** - 优雅处理 Agent 崩溃和超时
7. **主动使用 LSP 工具** - 修改代码前查看、评估影响、理解结构

---

## 核心工作原则

### 1. 上下文驱动开发

在任何代码修改前，必须彻底理解业务需求、现有代码逻辑及其上下文。禁止在信息不完整或存在假设的情况下进行编码。

### 2. 原子化提交

每次修改遵循最小化原则，每次提交仅解决一个明确定义的问题。

### 3. 完整功能交付

一次性完成所有相关功能，不留 `TODO` 或未完成的实现。

### 4. 文档生成限制

严禁主动生成任何文档（README、API文档等）。仅当用户明确要求时才生成。

### 5. 命令执行规范

- ❌ 禁止编写脚本文件（.sh、.bat等）
- ✅ 直接使用工具逐条执行命令
- ✅ 按正确顺序自动执行所有必要命令
- ✅ 对每条命令的执行结果进行验证

---

## 提交前检查清单

### 代码质量

- [ ] `cargo clippy` 无警告
- [ ] `npm run lint` 无错误
- [ ] 所有 Tauri 命令有 `#[specta::specta]` 宏
- [ ] 前端无 `console.log` 和 `as any`
- [ ] 错误处理使用 `message` 组件
- [ ] IPC 通信使用 JSON 协议
- [ ] React Hooks 遵循规则

### 安全检查

- [ ] 无硬编码 API 密钥、密码、tokens
- [ ] DeepSeek API 密钥使用系统密钥链存储
- [ ] IPC 消息已验证（防止恶意 Agent）
- [ ] Tauri 命令参数已验证
- [ ] 日志中无敏感信息（聊天内容、API 密钥）

### 测试覆盖

- [ ] 所有测试通过（`cargo test` 和 `npm test`）
- [ ] 测试覆盖率 ≥ 80%
- [ ] Agent 通信异常已测试
- [ ] DeepSeek API 调用失败已测试

### LSP 使用

- [ ] 修改代码前使用 LSP 查看结构和类型
- [ ] 使用 `findReferences` 评估修改影响范围
- [ ] 已更新 CHANGELOG.md 文件

---

## 完整闭环验证

**开发任务完成前，AI 必须确认（14项）**：

- [ ] 步骤 1：已调用 using-git-worktrees 创建分支
- [ ] 步骤 2：已调用 brainstorming（新功能）或 systematic-debugging（Bug修复）
- [ ] 步骤 3：已调用 writing-plans（复杂任务）
- [ ] 步骤 4：已自动调用项目特定 skills（根据文件类型）
- [ ] 步骤 5：已调用 test-driven-development（先写测试）
- [ ] 步骤 6：已实现代码
- [ ] 步骤 7：已调用 security-review（安全审查）
- [ ] 步骤 8：已调用 check（质量检查）
- [ ] 步骤 9：已调用 optimize（性能优化）
- [ ] 步骤 10：已调用 verification-before-completion（完成验证）
- [ ] 步骤 11：已调用 changelog-generator（生成日志）
- [ ] 步骤 12：已调用 commit（提交到 feature 分支）
- [ ] 步骤 13：已调用 finishing-a-development-branch（完成分支）
- [ ] 步骤 14：已调用 auto-merge-and-cleanup（自动合并和清理）

**如果任何一项未完成，任务不算完成，必须继续执行直到完整闭环。**

---

## 重要提醒

- 处理问题时，如果一直无法修复，积极思考新方案
- 不要陷入死循环，及时调整思路和方法
- 遇到困难时，回到需求分析阶段重新审视问题
- 每一步都先输出在做什么，然后开始执行

---

## 详细规范文档

详细技术规范请参考 `.claude/rules/` 目录：

- `01-project-overview.md` - WeReply 项目概览和技术栈
- `02-rust-backend-standards.md` - Rust 后端开发规范（Orchestrator）
- `03-react-frontend-standards.md` - React TypeScript 前端规范（助手面板）
- `04-configuration-standards.md` - 配置管理与环境变量规范
- `05-lsp-usage-standards.md` - LSP 使用规范
- `06-security-standards.md` - 安全开发规范（API 密钥、隐私保护）
- `07-testing-standards.md` - 测试规范（80% 覆盖率）
- `08-performance-standards.md` - 性能优化规范（低延迟响应）
- `09-parallel-agents-standards.md` - 多 Agent 并行工作规范
- `10-ipc-protocol-standards.md` - IPC 通信协议规范
- `11-platform-agent-standards.md` - Platform Agent 开发规范
