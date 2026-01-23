# Changelog

## [Unreleased]
- 将 Tauri + React 项目骨架从 worktree 移回仓库根目录。
- 添加应用源码与配置（`src/`、`src-tauri/`、`platform_agents/` 等）。
- 更新 `README.md` 补充本地开发命令。
- 完成前端主界面（状态面板、建议列表、编辑区、配置面板、主题切换）。
- 补齐后端命令与事件、Agent 进程管理、DeepSeek 调用与降级策略。
- 新增配置与密钥链管理、结构化日志与基础测试用例。
- 支持在项目根目录直接运行 `cargo tauri dev`。
- 简化 UI，仅保留主题切换、状态、建议、编辑与 API Key 连接流程。
- 保存 API Key 时进行 DeepSeek 连接验证，失败自动清理。
- 新增监听控制 IPC，完善 Windows/macOS Agent 的监听与输入写入实现。
- 修复 macOS Agent 依赖导入，确保可访问 Accessibility API。
- 增加规则：每次完成代码修改后同步到主目录供验证。
