# Agents 配置说明

本目录包含 CaCrFeedFormula 项目的专业 Agents（子代理）配置。

## 什么是 Agents？

Agents 是专门化的 AI 助手，每个 Agent 专注于特定的技术领域或任务类型。它们具有：
- **隔离的上下文窗口**：每个 Agent 维护独立的记忆
- **领域特定智能**：针对特定任务优化的指令
- **精细的工具权限**：基于角色的工具访问控制

## 可用的 Agents

### 1. rust-backend-specialist
**专长**：Rust 后端开发
**工具**：Read, Write, Edit, Bash, Glob, Grep
**适用场景**：
- Tauri 命令开发
- SQLx 数据库访问
- 异步编程
- 性能优化

**何时使用**：
- 开发新的 Tauri 命令
- 优化数据库查询
- 实现异步业务逻辑
- 性能调优

### 2. react-frontend-specialist
**专长**：React TypeScript 前端开发
**工具**：Read, Write, Edit, Bash, Glob, Grep
**适用场景**：
- React 组件开发
- 状态管理（TanStack Query）
- Tauri 集成
- 性能优化

**何时使用**：
- 创建新组件
- 实现复杂交互
- 优化渲染性能
- 集成后端 API

### 3. formula-optimization-specialist
**专长**：饲料配方优化
**工具**：Read, Write, Edit, Bash, Glob, Grep
**适用场景**：
- 线性规划优化
- HiGHS 求解器
- 营养计算
- 预混料设计

**何时使用**：
- 实现配方优化算法
- 处理营养计算
- 设计预混料方案
- 优化求解性能

### 4. test-automation-specialist
**专长**：测试自动化
**工具**：Read, Write, Edit, Bash, Glob, Grep
**适用场景**：
- 单元测试
- 集成测试
- TDD 工作流
- 测试覆盖率

**何时使用**：
- 编写测试用例
- 实施 TDD
- 提高测试覆盖率
- Mock 和 Stub

### 5. security-auditor
**专长**：安全审计
**工具**：Read, Grep, Glob（只读）
**适用场景**：
- 代码安全审查
- 漏洞检测
- 依赖安全
- 合规性检查

**何时使用**：
- 代码审查
- 安全审计
- 漏洞扫描
- 依赖检查

## 使用方式

### 方法 1：通过 Task 工具调用
```typescript
// 在对话中使用
"请使用 rust-backend-specialist agent 帮我优化这个数据库查询"
```

### 方法 2：项目级配置
将 agent 文件放在 `.claude/agents/` 目录中，Claude Code 会自动识别。

### 方法 3：全局配置
将 agent 文件放在 `~/.claude/agents/` 目录中，所有项目都可使用。

## Agent 配置结构

每个 Agent 文件遵循以下结构：

```markdown
---
name: agent-name
description: Agent 的用途和适用场景
tools: Read, Write, Edit, Bash, Glob, Grep
---

# Agent 标题

## 核心职责
- 职责 1
- 职责 2

## 技术规范
- 代码模板
- 最佳实践

## 开发检查清单
- [ ] 检查项 1
- [ ] 检查项 2

## 通信协议
- 与其他角色的协作方式

## 开发工作流
- 阶段 1
- 阶段 2

## 相关规范
- 相关的规范文件

## 相关 Skills
- 相关的 Skills
```

## 工具权限说明

### Read, Grep, Glob（只读）
- **适用**：审查类 Agent（security-auditor, code-reviewer）
- **权限**：只能读取和搜索代码，不能修改

### Read, Write, Edit, Bash, Glob, Grep（完整权限）
- **适用**：开发类 Agent（rust-backend-specialist, react-frontend-specialist）
- **权限**：可以读取、修改代码和执行命令

### Read, Write, Edit, Glob, Grep（无 Bash）
- **适用**：文档类 Agent
- **权限**：可以读写文件，但不能执行命令

## 最佳实践

### 1. 选择合适的 Agent
- 根据任务类型选择专业 Agent
- 复杂任务可以组合多个 Agent
- 审查任务使用只读 Agent

### 2. 明确任务范围
- 清晰描述任务需求
- 提供必要的上下文
- 指定预期输出

### 3. 利用 Agent 专长
- 让 Agent 专注于其擅长的领域
- 不要让 Agent 做超出其专长的事
- 组合多个 Agent 处理复杂任务

### 4. 审查 Agent 输出
- 验证代码质量
- 检查安全性
- 确认符合规范

## 与 Skills 的关系

### Agents vs Skills
- **Agents**：专业化的 AI 助手，有独立上下文和工具权限
- **Skills**：知识库和工作流指南，提供参考和模板

### 协同工作
- Agents 可以引用 Skills 中的知识
- Skills 可以推荐使用特定 Agent
- 两者互补，提供完整的开发支持

## 示例场景

### 场景 1：开发新的 Tauri 命令
```
1. 使用 rust-backend-specialist 实现后端逻辑
2. 使用 test-automation-specialist 编写测试
3. 使用 security-auditor 进行安全审查
4. 使用 react-frontend-specialist 集成前端
```

### 场景 2：优化配方计算性能
```
1. 使用 formula-optimization-specialist 分析算法
2. 使用 rust-backend-specialist 优化代码
3. 使用 test-automation-specialist 验证正确性
```

### 场景 3：安全审计
```
1. 使用 security-auditor 扫描代码
2. 使用 rust-backend-specialist 修复后端问题
3. 使用 react-frontend-specialist 修复前端问题
4. 使用 security-auditor 验证修复
```

## 相关文档

- `.claude/CLAUDE.md` - 核心规则
- `.claude/rules/` - 详细规范
- `.claude/skills/` - Skills 目录
- `.claude/INTEGRATION_SUMMARY.md` - 配置整合说明
- `.claude/SECOND_ROUND_SUMMARY.md` - 第二轮完善说明

## 贡献指南

### 创建新 Agent
1. 确定 Agent 的专长领域
2. 定义工具权限
3. 编写详细的职责和规范
4. 提供代码模板和示例
5. 添加检查清单
6. 更新本 README

### Agent 命名规范
- 使用 kebab-case
- 名称应描述专长
- 添加 -specialist 或 -auditor 后缀

### 文档要求
- 清晰的职责说明
- 详细的技术规范
- 实用的代码模板
- 完整的检查清单
- 相关规范引用
