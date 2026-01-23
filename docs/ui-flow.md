# UI 状态机与交互流程

## 状态
- idle: 未连接 Agent
- listening: 监听中
- generating: 正在生成建议
- paused: 用户暂停
- error: 发生错误

## 主要交互
- 启动监听 → 进入 listening
- 收到消息 → 进入 generating → 生成完成返回 listening
- 用户点击建议 → 写入输入框 → 返回 listening
- 发生错误 → error（显示重试/重连按钮）
- 用户暂停 → paused（停止监听）

## UI 组件
- 状态栏：显示平台、Agent 状态、最近错误
- 建议列表：展示 3 条建议，支持一键写入
- 编辑区：可对建议内容做轻量编辑
- 配置面板：DeepSeek 参数、监听频率
