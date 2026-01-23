# 架构设计

## 目标
- 跨平台 UI + 分平台自动化 Agent
- 仅支持 DeepSeek 生成回复建议
- 只对“前台聊天窗口”生效

## 分层
1) UI（Tauri/React）
2) Orchestrator（Rust）
3) Platform Agent（Windows/macOS）
4) DeepSeek 服务

## 组件职责
- UI：配置与状态展示、建议列表、人工选择与编辑
- Orchestrator：状态机、去重、上下文裁剪、DeepSeek 调度、错误隔离
- Agent：平台内微信自动化（监听与输入）
- DeepSeek：API 调用与建议生成

## 关键流程
1) Agent 监听前台聊天消息 → 上报 `message.new`
2) Orchestrator 进行去重/上下文裁剪 → 调用 DeepSeek
3) UI 展示建议 → 用户点击 → Orchestrator 下发 `input.write`
4) Agent 写入输入框（不发送）→ 上报 `input.result`

## 关联文档
- `docs/ipc.md`
- `docs/ai-deepseek.md`
- `docs/platforms.md`
- `docs/ui-flow.md`
- `docs/tauri-api.md`
- `docs/configuration.md`
- `docs/testing.md`
