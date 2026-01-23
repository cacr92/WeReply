# 平台实现概览

## Windows Agent（Python + wxauto v4）

### 启动
- Orchestrator 启动 Python 子进程
- Python 输出 `agent.ready`

### 监听策略
- 每 800ms 轮询前台聊天窗口
- 获取最近 N 条消息
- 通过 `msg_id` 或 `text+timestamp` 去重

### 输入写入
- 聚焦输入框 → 备份剪贴板 → 写入剪贴板 → Ctrl+V → 恢复剪贴板
- 失败则返回 `WRITE_FAILED`

### 交互接口（内部约定）
```
ready() -> bool
get_active_chat() -> {chat_id, title, is_group}
get_messages(chat_id, limit) -> [WxMessage]
write_input(chat_id, text) -> bool
```

## macOS Agent（Swift + Accessibility）

### 启动
- 检测 `AXIsProcessTrustedWithOptions`
- 不通过则返回 `PERMISSION_DENIED`

### 监听策略
- 仅支持前台 WeChat 窗口
- 通过 AXUIElement 遍历窗口 → 查找消息列表容器
- 轮询提取最新消息文本并去重

### 输入写入
- 查找输入框（AXTextArea）并聚焦
- 备份剪贴板 → 写入剪贴板 → Cmd+V → 恢复剪贴板

### 交互接口（内部约定）
```
ready() -> bool
get_active_chat() -> {chat_id, title, is_group}
get_messages(chat_id, limit) -> [WxMessage]
write_input(chat_id, text) -> bool
```
