# 微信回复建议助手（WeReply）

跨平台桌面应用：监听当前聊天消息，生成 DeepSeek 回复建议，用户一键写入输入框（不自动发送）。

## 平台支持
- Windows：通过 wxauto v4 + Python Agent（微信 4.1 / Windows 10+）
- macOS：通过 Accessibility + AppleScript/Swift Agent（需系统权限）

wxauto 仅支持 Windows，且官方仓库已归档为只读，因此 Windows 端被封装为可替换 Agent。

## 主要功能（MVP）
- 前台聊天消息监听
- DeepSeek 回复建议生成（多风格）
- 点击写入输入框（不自动发送）
- 跨平台 UI 与统一配置

## 文档索引
- `docs/architecture.md`：总体架构与职责划分
- `docs/ipc.md`：Agent <-> Orchestrator 协议
- `docs/ai-deepseek.md`：DeepSeek API 规范与请求模板
- `docs/platforms.md`：Windows/macOS Agent 实现细节
- `docs/ui-flow.md`：UI 状态机与交互流程
- `docs/tauri-api.md`：前后端命令与事件
- `docs/configuration.md`：配置项与优先级
- `docs/logging.md`：日志与诊断策略
- `docs/testing.md`：测试与验收清单
- `docs/security.md`：安全与隐私
- `docs/roadmap.md`：里程碑

## 配置
环境变量示例见 `.env.example`。API Key 仅存系统安全存储，不落地明文。

## 架构概览
- Tauri/React：UI 与配置
- Rust Orchestrator：状态机、协议、DeepSeek 调度
- Platform Agent：Windows(wxauto) / macOS(Accessibility)

详见 `docs/architecture.md`。

## 许可证
MIT
