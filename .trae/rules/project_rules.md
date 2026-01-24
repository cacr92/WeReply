---
alwaysApply: true
---

# WeReply - 微信回复建议助手 - Trae 规则

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

- **问候语**: 对话开始时，以"你好，主人！我是+模型名字"作为开场白
- **沟通语言**: 必须使用中文进行交流
- **专业性**: 保持专业、客观、技术导向的沟通风格

---

## 任务处理流程

### 非开发任务（咨询、解释、分析）

1. **需求分析** - 列出需求点
2. **任务分解** - 分解为具体子任务
3. **执行说明** - 说明正在执行的操作
4. **标记进度** - 实时标记完成状态

### 开发任务（代码修改、新功能、Bug修复）

**必须执行开发流程（6步核心 + 可选步骤）**，详见下方。

---

## 开发流程

### 核心原则

- ✅ **Plan-First Workflow** - 先计划后编码（复杂任务）
- ✅ **使用 feature 分支开发** - 永不直接修改 main 分支
- ✅ **灵活执行** - 简单任务可跳过部分步骤
- ✅ **TDD 优先** - 先写测试后写代码
- ✅ **质量保障** - 完成前必须验证

---

### 阶段 1：计划与设计（复杂任务时）

#### 步骤 1：需求分析

**触发时机**: 新功能、复杂任务、不清楚的需求

**作用**:
- 探索用户意图和需求
- 澄清技术实现细节（WeReply 特定：微信监听、DeepSeek API、IPC 通信等）
- 讨论设计方案和权衡
- 确定功能范围

**何时跳过**:
- 简单的 bug 修复
- 文档更新
- 明确的小改动
- 单文件样式调整

---

#### 步骤 2：编写实现计划

**触发时机**: 复杂功能、重构任务、跨模块修改

**作用**:
- 生成详细的实施计划
- 识别关键文件和依赖（参考 `.claude/rules/01-project-overview.md`）
- 考虑架构权衡（Rust Orchestrator ↔ Platform Agent ↔ DeepSeek API）
- 用户审批后执行

**何时跳过**:
- 单文件修改
- 明确的 API 调整
- 简单的样式更新
- UI 文本修改

**WeReply 特定考虑**:
- Agent 通信影响
- DeepSeek API 调用变更
- 微信监听逻辑修改

---

### 阶段 2：开发实施

#### 步骤 3：创建功能分支（强制执行）

**所有涉及代码修改的任务都必须执行**

```bash
git checkout -b feat/your-feature-name
# 或
git checkout -b fix/bug-description
```

**分支命名规范**:
- `feat/` - 新功能（如：`feat/add-voice-reply-support`）
- `fix/` - Bug 修复（如：`fix/agent-timeout-issue`）
- `refactor/` - 重构（如：`refactor/ipc-protocol`）
- `perf/` - 性能优化（如：`perf/optimize-deepseek-cache`）
- `test/` - 测试相关（如：`test/agent-communication`）

**绝对禁止**: 直接在 main 分支上修改代码

---

#### 步骤 4：TDD 实现（强制执行）

**所有新功能和 bug 修复都必须执行**

**TDD 流程**:
1. **编写测试** - 先写失败的测试
2. **实现代码** - 编写最小可用代码
3. **运行测试** - 确保测试通过
4. **重构** - 优化代码质量
5. **重复** - 循环直到功能完成

**覆盖率要求**:
- 整体覆盖率：≥ 80%
- 核心业务逻辑：≥ 90%
- 工具函数：≥ 95%

**WeReply 特定测试重点**:
- Agent 通信异常测试（超时、崩溃、格式错误）
- DeepSeek API 调用失败测试（网络错误、超时、限流）
- IPC 消息验证测试（恶意消息、格式错误）
- 微信监听去重测试

**何时简化**:
- 纯展示组件（UI only）
- 配置文件修改
- 文档更新

---

### 阶段 3：质量保障

#### 步骤 5：验证完成（强制执行）

**所有代码任务都必须执行**

**验证清单**:

##### 代码质量
- [ ] `cargo clippy` 无警告
- [ ] `npm run lint` 无错误
- [ ] 所有 Tauri 命令有 `#[specta::specta]` 宏
- [ ] 前端无 `console.log` 和 `as any`
- [ ] 错误处理使用 `message` 组件（前端）
- [ ] 使用 tracing 日志（后端）
- [ ] React Hooks 遵循规则（参考 `.claude/rules/03-react-frontend-standards.md`）

##### 安全检查（WeReply 桌面应用专项）
- [ ] 无硬编码 API 密钥、密码、tokens
- [ ] DeepSeek API 密钥使用系统密钥链存储（参考 `.claude/rules/06-security-standards.md`）
- [ ] IPC 消息已验证（防止恶意 Agent）
- [ ] Tauri 命令参数已验证
- [ ] 日志中无敏感信息（聊天内容、API 密钥）

##### 测试覆盖
- [ ] 所有测试通过（`cargo test` 和 `npm test`）
- [ ] 测试覆盖率 ≥ 80%
- [ ] 边界情况已测试
- [ ] 错误路径已测试
- [ ] Agent 通信异常已测试（WeReply 特定）
- [ ] DeepSeek API 调用失败已测试（WeReply 特定）

##### WeReply 特定验证
- [ ] Agent 崩溃不影响主程序运行
- [ ] 微信消息去重正常工作
- [ ] DeepSeek API 超时处理正确
- [ ] IPC 通信延迟 < 100ms（目标）
- [ ] 建议生成时间 < 3s（目标）

---

#### 步骤 6：完成分支（强制执行）

**所有代码任务都必须执行**

**提供选项**:
1. **创建 Pull Request** - 团队协作、需要审查
2. **直接合并到 main** - 个人项目、紧急修复
3. **继续开发** - 功能未完成、需要更多工作

**Commit 规范** (Conventional Commits):
```
feat: 添加语音回复支持
fix: 修复 Agent 超时问题
refactor: 重构 IPC 消息协议
perf: 优化 DeepSeek API 缓存
test: 添加 Agent 通信测试
docs: 更新 IPC 协议文档
chore: 更新依赖版本
```

---

### 可选步骤

#### 可选 A：系统化调试（Bug 修复时替代步骤 1-2）

**触发时机**: Bug 修复、异常排查

**调试流程**:
1. **重现问题** - 创建可复现的测试用例
2. **诊断原因** - 使用日志、断点分析
3. **修复问题** - 实施最小化修复
4. **验证修复** - 确保问题解决且无副作用
5. **防止复发** - 添加回归测试

**WeReply 特定调试技巧**:
- 检查 Agent 日志（`platform_agents/logs/`）
- 查看 tracing 日志（`logs/wereply.log`）
- 监控 DeepSeek API 响应时间

---

#### 可选 B：代码审查（重大功能时）

**触发时机**: 重大功能、重构、安全关键代码

**审查重点**:
- 架构设计是否合理（Orchestrator ↔ Agent ↔ DeepSeek）
- 代码质量和可维护性
- 安全漏洞和性能问题
- 测试覆盖率和质量

**WeReply 特定审查点**:
- IPC 消息格式变更影响（向后兼容性）
- DeepSeek API 调用频率（避免限流）
- Agent 崩溃恢复机制
- 系统密钥链使用正确性

---

## 核心编码准则

### 严禁事项（WeReply 桌面应用专项）

1. **禁止 `console.log/error/warn`** → 使用 `message.error/success/warning`（前端）或 `tracing`（后端）
2. **禁止 `as any` 类型转换** → 使用精确类型断言或类型守卫
3. **禁止原始 `invoke`** → 使用生成的 `commands`
4. **禁止在循环中使用 Hooks**
5. **禁止硬编码 API 密钥** → 使用系统密钥链存储（DeepSeek API 密钥）
6. **禁止阻塞异步运行时** → 使用 `tokio::spawn` 或 `tokio::time::sleep`

### 必须遵守

1. **使用 specta 导出类型** - 所有 Tauri 命令必须有 `#[specta::specta]` 宏
2. **使用 `message` 组件** - 所有用户反馈通过 Ant Design `message` 组件
3. **使用 tracing 日志** - 使用结构化日志记录
4. **使用 `anyhow::Result + ApiResponse`** - 底层 anyhow，顶层 ApiResponse
5. **使用 JSON 进行 IPC 通信** - Rust Orchestrator ↔ Platform Agent 使用 JSON 协议
6. **处理 Agent 异常** - 优雅处理 Agent 崩溃和超时

---

## WeReply 项目特定指南

### 技术栈概览

- **后端**: Rust 2021 + Tauri 2.x + Tokio 1.37
- **前端**: React 18 + TypeScript 5.x + Ant Design 5.x + Vite
- **Platform Agents**:
  - Windows: Python 3.9+ + wxauto v4
  - macOS: Swift + Accessibility API
- **AI 集成**: 仅支持 DeepSeek API
- **通信协议**: IPC (JSON via stdin/stdout)

详见：`.claude/rules/01-project-overview.md`

---

### 修改文件时的技术栈参考

| 文件路径 | 参考规范文档 | 关键点 |
|---------|------------|-------|
| `src/wechat/**/*.rs` | `.claude/rules/02-rust-backend-standards.md` | 微信监听、消息去重 |
| `src/ai/**/*.rs` | `.claude/rules/02-rust-backend-standards.md` + `06-security-standards.md` | DeepSeek API、密钥管理 |
| `src/orchestrator/**/*.rs` | `.claude/rules/02-rust-backend-standards.md` | IPC 通信、状态机 |
| `platform_agents/**/*.py` | `.claude/rules/` | wxauto v4、消息监听 |
| `platform_agents/**/*.swift` | `.claude/rules/` | Accessibility API |
| `frontend/src/**/*.tsx` | `.claude/rules/03-react-frontend-standards.md` | React Hooks、Ant Design |
| `src/**/commands.rs` | `.claude/rules/02-rust-backend-standards.md` | Tauri 命令、specta 类型 |

---

### 性能目标

- **消息监听延迟**: < 500ms
- **DeepSeek 建议生成时间**: < 3s
- **建议展示响应**: < 100ms
- **文本输入延迟**: < 200ms
- **Agent 重启时间**: < 2s
- **IPC 通信延迟**: < 100ms

---

### 安全重点

1. **DeepSeek API 密钥** - 必须使用系统密钥链存储
2. **IPC 消息验证** - 防止恶意 Agent 消息
3. **聊天内容隐私** - 不记录到日志、不上传（除 DeepSeek API）
4. **Agent 进程隔离** - Agent 崩溃不影响主程序

---

## 核心工作原则

### 1. Plan-First Workflow（复杂任务）

在任何复杂代码修改前，先进行计划和设计。

**推荐流程**:
1. 需求分析 - 探索需求
2. 编写计划 - 生成详细计划
3. 用户审批后再执行

### 2. 上下文驱动开发

在任何代码修改前，必须彻底理解业务需求、现有代码逻辑及其上下文。禁止在信息不完整或存在假设的情况下进行编码。

### 3. Test-Driven Development (TDD)

遵循 TDD 原则，先写测试后写代码。这确保：
- 代码可测试性
- 需求明确性
- 重构安全性
- 高覆盖率

### 4. 原子化提交

每次修改遵循最小化原则，每次提交仅解决一个明确定义的问题。

### 5. 完整功能交付

一次性完成所有相关功能，不留 `TODO` 或未完成的实现。

### 6. 文档生成限制

严禁主动生成任何文档（README、API文档等）。仅当用户明确要求时才生成。

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

---