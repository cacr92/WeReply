# IPC 协议

## 通用封装
所有消息均为 JSON 行（1 行 1 个消息）：
```
{
  "version": "1.0",
  "type": "message.new",
  "id": "uuid",
  "timestamp": 1737620000,
  "payload": {}
}
```

## 约定
- `id` 为发送方生成的消息 ID
- 接收方需返回 `event.ack`
- 3 秒无 ack 则重发，最多 3 次

## 事件类型
### 生命周期
- `agent.ready`
- `agent.status`
- `agent.error`
- `agent.ping`
- `agent.pong`

### 消息与输入
- `message.new`
- `input.write`
- `input.result`

### 协议确认
- `event.ack`

## 事件结构
### agent.ready
```
payload: {
  "platform": "windows" | "macos",
  "agent_version": "0.1.0",
  "capabilities": ["listen", "write"],
  "supports_clipboard_restore": true
}
```

### agent.status
```
payload: {
  "state": "idle" | "listening" | "error" | "paused",
  "detail": "..."
}
```

### agent.error
```
payload: {
  "code": "AGENT_NOT_READY" | "WX_NOT_FOUND" | "PERMISSION_DENIED" | "WRITE_FAILED",
  "message": "...",
  "recoverable": true
}
```

### message.new
```
payload: {
  "chat_id": "chat_001",
  "chat_title": "张三",
  "is_group": false,
  "sender_name": "张三",
  "text": "晚上一起吃饭吗？",
  "timestamp": 1737620000,
  "msg_id": "optional"
}
```

### input.write
```
payload: {
  "chat_id": "chat_001",
  "text": "好的，几点？",
  "mode": "paste",
  "restore_clipboard": true
}
```

### input.result
```
payload: {
  "ok": true,
  "error": ""
}
```

### event.ack
```
payload: {
  "ack_id": "<original id>",
  "ok": true,
  "error": ""
}
```

## 错误码建议
- AGENT_NOT_READY
- WX_NOT_FOUND
- PERMISSION_DENIED
- LISTEN_FAILED
- WRITE_FAILED
- PROTOCOL_ERROR
