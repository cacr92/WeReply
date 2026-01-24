# 饲料配方系统 - Claude Code Skills

本项目包含专门为饲料配方优化系统定制的 Claude Code Skills，涵盖 Tauri + Rust + React 技术栈的各个开发方面。

## 📁 可用 Skills

### 0. [project-overview](./project-overview/SKILL.md) **v1.0** 🌟
**项目简介（自动加载）**

触发词：无（自动加载，不可手动调用）

适用于：
- 为 AI 提供项目背景知识
- 自动加载项目架构、技术栈、功能模块信息
- 作为 AI 理解项目的基础上下文
- 在会话启动时自动激活

---

### 1. [tauri-development](./tauri-development/SKILL.md) **v4.0**
**Tauri 桌面应用开发**

触发词："创建 Tauri 命令"、"添加后端命令"、"Tauri"、"桌面应用"、"前后端类型不一致"、"更新 bindings.ts"

适用于：
- 创建或修改 Tauri 命令
- 处理 Rust 后端与 React 前端集成
- specta 类型绑定
- 前后端通信优化
- 桌面应用特定模式

---

### 2. [rust-optimization](./rust-optimization/SKILL.md) **v3.0**
**Rust 性能优化**

触发词："优化 Rust 代码"、"性能优化"、"Rust优化"、"并行计算"、"缓存策略"、"避免克隆"、"SIMD 优化"

适用于：
- 优化 Rust 代码性能
- 实现缓存策略（Moka）
- 使用并行处理（Rayon）
- 数值计算和线性规划优化
- 内存管理和性能调优

---

### 3. [formula-calculation](./formula-calculation/SKILL.md) **v2.1**
**饲料配方计算**

触发词："配方优化"、"线性规划"、"营养计算"、"成本优化"、"约束处理"、"数值稳定性"、"浮点精度"

适用于：
- 实现配方优化逻辑
- 线性规划计算
- 营养约束处理
- 成本优化
- 影子价格分析

---

### 4. [sqlite-optimization](./sqlite-optimization/SKILL.md) **v2.1**
**SQLite 数据库优化**

触发词："优化数据库"、"优化查询"、"创建索引"、"事务处理"、"迁移管理"、"WAL 模式"、"PRAGMA 优化"

适用于：
- 编写 SQLx 查询
- 优化数据库性能
- 设计数据库架构
- 创建索引
- 处理事务和迁移

---

### 5. [react-typescript-development](./react-typescript-development/SKILL.md) **v3.0**
**React TypeScript 开发**

触发词："开发 React 组件"、"编写前端代码"、"创建界面"、"实现状态管理"、"前端开发"、"Hooks 优化"、"类型安全"、"避免 as any"

适用于：
- 开发 React 19 + TypeScript 5 组件
- 使用 Ant Design 组件库
- 实现自定义 Hooks
- TanStack Query 状态管理
- Tauri 前端集成

---

### 6. [error-handling-debugging](./error-handling-debugging/SKILL.md) **v2.0**
**错误处理与调试**

触发词："处理错误"、"错误处理"、"捕获异常"、"错误恢复"

适用于：
- 实现 Rust/TypeScript 错误处理
- 调试应用问题
- 排查 Tauri 命令失败
- 数据库错误处理
- 错误恢复策略

---

### 7. [testing-strategy](./testing-strategy/SKILL.md) **v2.0** ⭐
**测试策略**

触发词："编写测试"、"创建测试用例"、"添加单元测试"、"添加集成测试"、"测试覆盖率"、"TDD"

适用于：
- 编写 Rust 单元测试和集成测试
- 编写 React 组件和 Hook 测试
- Tauri 命令模拟（Mocking）
- 数据库测试夹具
- TDD（测试驱动开发）工作流
- 测试覆盖率分析

---

### 8. [api-design](./api-design/SKILL.md) **v2.0** ⭐
**API 设计与文档**

触发词："设计API"、"创建DTO"、"定义数据结构"、"API文档"、"请求响应格式"

适用于：
- 设计 Tauri 命令 API
- 创建类型安全的 DTO
- API 文档和契约
- 请求/响应模式
- API 错误处理
- API 版本控制

---

### 9. [code-review](./code-review/SKILL.md) **v2.0** ⭐⭐
**代码审查**

触发词："审查代码"、"代码检查"、"检查bug"、"代码质量检查"、"发现问题"、"重构建议"、"code review"

适用于：
- 系统性代码审查框架
- 分类检查清单（正确性、性能、安全性、可维护性）
- Rust/React/集成专项审查
- PR 审查流程
- 代码审查评论模板

---

### 10. [refactoring](./refactoring/SKILL.md) **v2.0** ⭐⭐
**代码重构**

触发词："重构代码"、"改进代码结构"、"清理代码"、"降低复杂度"、"提取方法"、"代码异味"

适用于：
- 识别和消除代码异味
- 提取方法、函数、组件
- 简化复杂逻辑
- SOLID 原则应用
- 重构工作流程和检查清单

---

### 11. [debugging](./debugging/SKILL.md) **v2.1** ⭐⭐
**问题诊断**

触发词："调试这个"、"修复bug"、"排查问题"、"不工作"、"出错了"、"调查问题"、"断点调试"、"日志分析"、"性能分析"

适用于：
- 科学调试方法论
- Rust/React/Tauri/数据库常见问题
- 问题诊断流程图
- 性能分析工具
- 系统化调试步骤

---

### 12. [tdd-workflow](./tdd-workflow/SKILL.md) **v1.0** ⭐
**TDD 工作流**

触发词："TDD"、"测试驱动开发"、"先写测试"、"红绿重构"、"test-driven"

适用于：
- 测试驱动开发工作流
- 红-绿-重构循环
- 先写测试后写代码
- 测试优先的开发方式
- TDD 最佳实践

---

### 13. [security-review](./security-review/SKILL.md) **v1.0** ⭐⭐
**安全审查**

触发词："安全审查"、"安全检查"、"漏洞扫描"、"安全审计"、"security review"

适用于：
- 代码安全审查
- 漏洞检测和修复
- SQL 注入防护
- XSS 防护
- 密钥管理检查
- 依赖安全审计

---

### 14. [auto-merge-and-cleanup](./auto-merge-and-cleanup/skill.md) **v1.0** ⭐⭐⭐
**自动合并和清理（步骤14）**

触发词："自动合并"、"清理分支"、"完成任务"、"合并到main"、"auto merge"、"cleanup"、"finish task"、"步骤14"、"step 14"

适用于：
- 在14步开发流程完成后，自动合并分支到 main
- 同步远程代码
- 删除远程和本地分支
- 清理 worktree 目录
- 清理临时文件
- **强制执行，不可跳过**

---

## 🚀 如何使用

### 自动激活（推荐）

Skills 会根据描述中的触发条件自动激活。当你开始相关任务时，Claude 会自动使用对应的 skill。

**中文触发示例：**
- "请审查这段代码" → 自动使用 `code-review`
- "帮我调试这个问题" → 自动使用 `debugging`
- "优化一下这个 Rust 函数" → 自动使用 `rust-optimization`
- "重构这段代码" → 自动使用 `refactoring`
- "编写单元测试" → 自动使用 `testing-strategy`
- "设计一个新的 API" → 自动使用 `api-design`

**英文触发示例：**
- "Review this code" → `code-review`
- "Debug this issue" → `debugging`
- "Optimize performance" → `rust-optimization`
- "Refactor this function" → `refactoring`

### 手动指定

明确要求使用特定 skill：

```
请使用 code-review skill 帮我审查这个 PR
```

## 📖 Skill 结构

每个 skill 都遵循标准结构：

```
skill-name/
└── SKILL.md
```

**SKILL.md 包含：**
- **Frontmatter 元数据** (中文 + 英文)
  - `name`: Skill 标识符
  - `description`: 中文触发条件 + 英文关键词
  - `version`: 版本号

- **Markdown 内容** (英文内容，便于 AI 理解)
  - 概述和核心概念
  - 代码示例和模式
  - 最佳实践
  - 常见陷阱
  - 快速参考

## 🎯 使用场景

### 日常开发
```bash
开发功能 → tauri-development + react-typescript-development
设计 API → api-design
```

### 代码审查
```bash
PR 审查 → code-review
发现问题 → debugging
```

### 优化改进
```bash
性能优化 → rust-optimization + sqlite-optimization
代码重构 → refactoring
```

### 质量保证
```bash
编写测试 → testing-strategy
修复 bug → debugging + error-handling-debugging
```

## 🔧 维护指南

### 修改 Skill

1. 编辑对应的 `SKILL.md` 文件
2. 保持 frontmatter 格式正确
3. 更新 `version` 字段
4. 测试触发条件

### 添加新 Skill

1. 在 `.claude/skills/` 下创建新目录
2. 创建 `SKILL.md` 文件
3. 添加中英双语的 description：

```markdown
---
name: your-skill-name
description: 当用户要求"中文触发词1"、"中文触发词2"，或者提到"英文关键词1"、"英文关键词2"时使用此技能。用于...
version: 1.0.0
---
```

## 📝 更新日志

### v4.0 (2025-01-21) - 项目结构优化 ⭐⭐⭐⭐
**重大改进**：
- ✅ **新增 project-overview skill** - 自动加载项目背景知识（`user-invocable: false`）
- ✅ **优化 hooks 配置** - 为 `cargo clippy` 和 `npm run lint` 添加 timeout（30秒/20秒）
- ✅ **精简 CLAUDE.md** - 只保留核心工作流和编程规范，项目简介移至 `project-overview` skill
- ✅ **改进自动调用机制** - 基于官方最佳实践优化 skills description
- ✅ **更清晰的文档结构** - `.claude/rules/` 存放详细技术规范，CLAUDE.md 仅作快速参考

**总计**: 15 个专业 skills（包含 1 个自动加载的背景 skill），**超过 5,507 行**实战指导文档

### v3.0 (2025-12-25) - 中英双语版本 ⭐⭐⭐
**重大改进：**
- ✅ **所有 skills 支持中文触发** - description 改为中英双语
- ✅ 新增 **code-review** skill (代码审查框架)
- ✅ 新增 **refactoring** skill (系统性重构指导)
- ✅ 新增 **debugging** skill (问题诊断和修复)
- ✅ 更注重实际开发场景和工作流程
- ✅ 符合项目实际情况（AI对话、预混料、生产批次、盈亏测算等）

**总计**: 11 个专业 skills，**5,507 行**实战指导文档

### v2.0 (2025-12-25)
- 新增 testing-strategy, api-design skills
- 改进触发条件
- 添加快速参考部分

### v1.0 (2025-12-25)
- 初始版本，6 个基础 skills

## 🎯 Skills 完整性

### ✅ 符合项目实际情况

本项目有：
- **167 个 Tauri 命令** - tauri-development skill 支持
- **AI 对话功能** - 可添加 ai-integration skill
- **配方管理、优化、分析** - formula-calculation skill 专注
- **原料管理、库存** - material 可扩展
- **预混料设计** - premix 相关
- **生产批次管理** - production-batch 可扩展
- **盈亏测算** - profit-analysis 可扩展
- **营养预测** - nutrition-prediction 可扩展

### ✅ 支持中英双语触发

现在所有 skills 都支持：
- **中文触发词**：用户用中文提问即可触发
- **英文关键词**：保留英文兼容性
- **双语支持**：确保最大灵活性

## 💡 核心特色

1. **实战导向** - 基于 real-world 开发场景
2. **问题解决** - 常见问题和解决方案表格
3. **工作流集成** - 覆盖完整开发生命周期
4. **中英双语** - 支持中英文触发
5. **项目定制** - 专门为饲料配方系统优化

## 🔄 完整开发工作流

```
需求分析
  ↓
设计 API (api-design)
  ↓
开发实现 (tauri-development + react-typescript-development)
  ↓
代码审查 (code-review)
  ↓
重构优化 (refactoring + rust-optimization)
  ↓
测试验证 (testing-strategy)
  ↓
问题修复 (debugging + error-handling-debugging)
  ↓
部署上线
```

---

**祝开发愉快！** 🚀
