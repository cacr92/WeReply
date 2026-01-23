# 测试计划

## 单元测试
- Prompt 构建
- 上下文裁剪（按条数/字符数）
- 去重逻辑
- 重试与降级策略

## 集成测试
- IPC 协议兼容
- Agent 模拟（模拟 message.new 与 input.result）
- DeepSeek API Mock

## 手动验收
- Win/mac 前台聊天监听
- 建议生成与列表展示
- 写入输入框 + 剪贴板恢复
- DeepSeek 超时降级为模板
- macOS 权限引导
