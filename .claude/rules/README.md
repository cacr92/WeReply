# Claude Code 规则文件说明

本目录包含饲料配方优化系统（CaCrFeedFormula）的 Claude Code 详细开发规范。

## 📁 文件列表

### 01-project-overview.md
项目概览和技术栈说明，包含：
- 项目简介和核心功能
- 技术栈详解（Rust + React + Tauri）
- 项目结构说明
- 开发环境要求

### 02-rust-backend-standards.md
Rust 后端开发规范，包含：
- 类型系统规范（Tauri 命令、specta 类型导出）
- 异步编程规范（Tokio 使用）
- 数据库访问规范（SQLx 使用）
- 事务处理规范
- 错误处理规范（anyhow + ApiResponse）
- 性能优化规范（缓存、并行计算）
- 日志记录规范（tracing）
- 代码组织规范

### 03-react-frontend-standards.md
React TypeScript 前端开发规范，包含：
- 组件组织规范
- React Hooks 使用规范
- TypeScript 类型规范（禁止 as any）
- 状态管理规范（useState, Context, TanStack Query）
- Ant Design 使用规范（message 组件）
- Tauri 集成规范（使用生成的 commands）
- 性能优化规范（memo, useCallback, useMemo）
- 代码组织规范

### 04-database-standards.md
数据库与数据访问规范，包含：
- Migration 规范（文件命名、SQL 编写）
- SQLx 使用规范（查询宏、参数绑定）
- 事务处理规范
- 查询优化规范（索引、避免 N+1 查询、批量操作）
- 数据库连接池管理
- 数据类型映射
- 常见陷阱和测试规范

### 05-lsp-usage-standards.md
LSP (Language Server Protocol) 使用规范，包含：
- LSP 工具概述（hover, goToDefinition, findReferences 等）
- 必须使用 LSP 的场景（修改代码前、理解结构、重构、调试、添加功能）
- 最佳实践（优先使用LSP而非盲目搜索）
- 工作流程示例（Rust/TypeScript）
- 常见错误和解决方案
- 自我检查清单

### 06-security-standards.md
安全开发规范（针对桌面应用），包含：
- 桌面应用安全特点（与 Web 应用的区别）
- 密钥管理（环境变量、加密存储）
- Tauri 命令安全（参数验证、权限控制）
- SQL 注入防护（SQLx 参数化查询）
- 输入验证（前后端双重验证）
- 敏感数据保护（日志、错误消息���
- 文件操作安全（路径验证、类型验证）
- 依赖安全（cargo audit）
- Rust 特定安全（避免 unsafe）
- 安全响应协议

### 07-testing-standards.md
测试规范（80% 覆盖率要求），包含：
- 测试覆盖���要求（强制 80%）
- 单元测试（Rust + TypeScript）
- 集成测试（数据库、API）
- Mock 策略（Tauri 命令、数据库）
- 测试最佳实践（测试行为而非实现）
- 测试组织结构
- 覆盖率验证（tarpaulin, vitest）
- 常见测试陷阱

### 08-performance-standards.md
性能优化规范，包含：
- Rust 后端性能优化（并行计算、缓存、避免克隆）
- React 前端性能优化（memo, useCallback, useMemo, 虚拟滚动）
- 数据库性能优化（索引、避免 N+1、批量操作、SQLite 优化）
- 启动性能优化（延迟加载、预加载）
- 内存优化（及时释放、流式处理）
- 性能监控（tracing）
- 性能基准（目标指标）
- 常见性能陷阱

### 09-parallel-agents-standards.md
多 Agent 并行工作规范（优雅方案），包含：
- **方案选择决策树**（何时并行、何时串行）
- **方案 A：Sub-agents**（推荐，无物理隔离）
- **方案 B：Background Tasks**（耗时任务）
- **方案 C：Git Worktree**（仅必要时，使用 .worktrees/ 子目录）
- 任务分解最佳实践（模块级、文件级、层次级）
- 冲突检测和解决策略
- 实战示例（优化性能、运行测试、实验性重构）
- 性能和资源考虑
- 快速决策指南

## 📖 使用说明

这些规则文件是对 `.claude/CLAUDE.md` 主规则文件的详细补充。主规则文件提供概览和核心准则，而这些模块化规则文件提供具体的技术实现细节。

## 🔗 与主规则文件的关系

- **主规则文件** (`.claude/CLAUDE.md`): 包含 AI 行为准则、核心编码准则、执行协议和开发流程
- **详细规则文件** (本目录): 提供具体技术栈的详细实现规范和最佳实践

## ⚠️ 重要提醒

所有规则文件都应该被 Claude Code 在对话开始时完整阅读和理解。这些规则确保代码质量、一致性和项目标准的遵守。

## 📝 规范要点

### 严禁事项
1. 禁止使用 `console.log/error/warn`（桌面应用）
2. 禁止使用 `as any` 类型转换（TypeScript）
3. 禁止使用原始 `invoke`（使用生成的 commands）
4. 禁止在循环中使用 Hooks（React）
5. 禁止忘记提交事务（数据库）
6. 禁止 `SELECT *`（数据库查询）

### 必须遵守
1. 使用 specta 导出类型（Rust → TypeScript）
2. 使用 `message.error/success/warning`（前端提示）
3. 使用 tracing 日志（后端日志）
4. 使用 `anyhow::Result + ApiResponse`（错误处理）
5. 使用 SQLx 编译时检查（数据库查询）
6. 使用事务处理复杂操作（数据一致性）
7. **主动使用 LSP 工具**（修改代码前查看、评估影响、理解结构）

