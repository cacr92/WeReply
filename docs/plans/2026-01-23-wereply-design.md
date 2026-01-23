# WeReply 设计文档（2026-01-23）

## 目标
- 监听前台微信聊天消息
- 生成 DeepSeek 回复建议
- 一键写入输入框（不自动发送）
- Windows/macOS 均可完整使用

## 非目标（MVP 不做）
- 后台静默监听或批量群发
- 自动发送消息
- 多账号/多开管理
- 云端存储聊天记录

## 约束与风险
- wxauto 仅支持 Windows，且仓库归档为只读，需隔离为可替换 Agent
- macOS 需 Accessibility 权限，UI 自动化易受微信 UI 更新影响
- 仅支持“当前前台聊天窗口”

## 架构
分层：UI（Tauri/React） → Orchestrator（Rust） → Platform Agent → DeepSeek 服务
- UI：配置、状态、建议展示与人工选择
- Orchestrator：状态机、去重、上下文裁剪、DeepSeek 调度、错误隔离
- Agent：平台内微信自动化与输入写入
- DeepSeek：API 调用与建议生成

## 平台实现
### Windows Agent
- Python 子进程 + wxauto v4
- 轮询前台聊天消息列表，增量检测
- 输入写入：剪贴板置换 + 粘贴

### macOS Agent
- Swift CLI + Accessibility + AppleScript
- AXUIElement 读取前台聊天窗口
- 输入写入：聚焦输入框 → 剪贴板置换 → Cmd+V → 恢复

## IPC 协议
- JSON 行格式，1 行 1 条消息
- 统一消息封装：`{version, type, id, timestamp, payload}`
- 事件：`agent.ready` / `agent.status` / `message.new` / `input.write` / `input.result`
- 3 秒无 ack 则重发，最多 3 次

## DeepSeek 生成策略
- 默认模型：deepseek-chat
- 3 条建议（正式/中性/轻松）
- 超时 12s，最多重试 2 次
- 上下文最多 10 条或 2000 字符

## 配置与存储
- 默认值 < 配置文件 < 环境变量
- API Key 存系统安全存储，不落地明文

## 安全与隐私
- 不自动发送
- 不落地存储聊天内容
- 频率节流降低风控风险

## 错误处理与降级
- Agent 不可用 → UI 切换手动模式
- DeepSeek 超时 → 模板回复
- 监听不稳定 → 手动刷新
- macOS 权限缺失 → 引导授权

## 测试计划
- 单元：去重、上下文裁剪、Prompt 构建
- 集成：IPC 协议、Agent 模拟
- 手动验收：Win/mac 前台聊天与写入

## 里程碑
1) UI/Orchestrator 骨架 + IPC
2) Windows Agent（wxauto v4）
3) macOS Agent（Accessibility）
4) DeepSeek 接入
5) 稳定性与权限引导
