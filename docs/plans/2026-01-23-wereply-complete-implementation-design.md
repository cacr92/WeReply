# WeReply 完整实现设计

**目标**
- UI 简化，仅保留状态区、监听控制、建议列表、编辑区、API Key 输入与主题切换。
- 使用 DeepSeek 默认配置，不提供 UI 配置项。
- API Key 保存后立即测试连接，失败则删除密钥并提示。
- Windows/macOS Agent 完整实现监听与输入写入，配合 IPC 可靠通信。

## 架构与数据流
- 前端仅做控制与展示：启动/暂停/恢复/停止监听，显示建议与状态；API Key 输入与连接验证；主题切换。
- Rust 作为中枢：管理状态、上下文、调用 DeepSeek、维护 Agent 子进程与 IPC。
- Agent 轮询前台微信，发送 `message.new` 触发建议生成；写入请求通过 `input.write` 完成。
- IPC 采用 `event.ack` 确认，Agent 侧对关键消息做超时重发，Rust 侧收到消息即发送 ack。

## API Key 验证
- `save_api_key`：保存密钥后立即调用 DeepSeek 最小请求验证。
- 验证失败则删除密钥并返回错误；验证成功返回 ok。
- UI 在保存成功后显示“已连接”，失败显示错误。

## 监听控制
- 新增 IPC 控制消息：`listen.start`、`listen.pause`、`listen.resume`、`listen.stop`。
- Rust 在 start/pause/resume/stop 中向 Agent 发送控制消息。
- Agent 按控制消息切换监听状态，并发送 `agent.status`。

## Agent 端实现
### Windows (wxauto)
- 启动后发送 `agent.ready`。
- 轮询前台聊天，抽取最新消息文本与时间戳，去重后发送 `message.new`。
- 输入写入：切换目标聊天 → 聚焦输入框 → 通过剪贴板粘贴（不自动发送）。

### macOS (Accessibility)
- 检测 Accessibility 权限，不通过则发送 `PERMISSION_DENIED`。
- 轮询前台 WeChat 窗口，提取消息列表末尾文本作为最新消息。
- 输入写入：激活 WeChat → 写入剪贴板 → AppleScript 触发 Cmd+V → 恢复剪贴板。

## 错误处理与降级
- Agent 断开：Rust 进入 error 状态并提示。
- DeepSeek 调用失败：使用本地 fallback 建议。
- Agent 权限缺失或监听失败：发送 error 事件并提示用户。

## 测试策略
- Rust：新增 API Key 验证与监听控制的单元测试。
- 前端：新增 API Key 流程与 UI 简化回归测试。
- 端到端：在本地通过真实 WeChat 测试监听与写入。
