# Changelog

## [Unreleased]
- Windows Agent 内置 wxauto 源码并通过 PYTHONPATH 引用，避免运行时安装该依赖。
- Windows 打包内置嵌入式 Python 3.12，并自动安装 wxauto 等依赖，运行时优先使用内置 Python。
- Windows Agent 启动前自动安装 Python 依赖（wxauto/pyautogui/pyperclip/comtypes），缺失时自动尝试安装并复检。
- 设置默认运行二进制为 `wereply`，避免 `cargo run` 需要手动指定 `--bin`。
- 修复保存与诊断 API Key 时参数名不匹配导致的调用失败。
- 新增 bindings 自动生成器，确保前后端参数命名一致。
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
- 精简主界面为“监听控制 + 回复建议 + 回复输入”，API Key 入口移入设置弹窗。
- 点击建议或手动回复可直接写入微信输入框（不发送），并加强输入校验。
- API Key 保存流程增加异常兜底与超时防护，补齐关键链路日志。
- 修复 DeepSeek 请求 URL 与参数设置，使用官方默认配置完成验证与建议生成。
- 对齐 DeepSeek 官方文档，请求显式关闭 stream 并放宽超时上限。
- 使用系统根证书进行 TLS 校验，降低企业代理/自签证书导致的连接失败。
- 新增连接诊断按钮，展示聊天与模型接口的详细状态。
- API Key 支持显示/隐藏切换，并在设置中展示失败原因。
- 设置弹窗在保存 API Key 后自动拉取模型列表，并支持模型选择与持久化。
