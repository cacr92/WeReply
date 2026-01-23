# 项目概览 - WeReply 微信回复建议助手

## 项目简介

**WeReply** 是一款基于 Tauri + Rust + React TypeScript 技术栈构建的跨平台微信回复建议助手。系统通过 Platform Agents 监听前台微信聊天消息，调用 DeepSeek API 生成多风格回复建议，用户可选择建议并一键写入微信输入框（不自动发送），实现智能辅助回复。

## 核心技术栈

### 后端 (Rust 2021)
- **框架**: Tauri 2.x + Tokio 1.37 (完整异步运行时)
- **HTTP客户端**: reqwest 0.11 (DeepSeek API 调用)
- **日志**: tracing + tracing-subscriber (结构化日志)
- **类型绑定**: specta 2.0 + tauri-specta 2.0 (自动类型生成)
- **错误处理**: anyhow (错误传播)
- **序列化**: serde + serde_json (JSON 序列化)

### 前端 (React 18 + TypeScript 5.x)
- **UI框架**: Ant Design 5.x
- **状态管理**: React Context + Hooks
- **构建工具**: Vite
- **HTTP客户端**: 内置 Tauri fetch API

### Platform Agents
- **Windows**: Python 3.9+ + wxauto v4 (微信 4.1 自动化)
- **macOS**: Swift + Accessibility API + AppleScript (系统权限)

### AI 集成
- **DeepSeek**: 仅支持 DeepSeek API
- **通信方式**: HTTP/HTTPS (reqwest)
- **响应处理**: 流式/非流式

## 项目结构

```
wereply/
├── src/                              # Rust 后端源码
│   ├── wechat/                       # 微信监听与自动化模块
│   │   ├── mod.rs
│   │   ├── monitor.rs                # 前台窗口监听
│   │   └── window_tracker.rs         # 窗口跟踪器
│   ├── ai/                           # AI 服务模块
│   │   ├── mod.rs
│   │   ├── deepseek_service.rs       # DeepSeek API 服务
│   │   └── prompt_builder.rs         # 提示词构建器
│   ├── orchestrator/                 # 协调器模块
│   │   ├── mod.rs
│   │   ├── state_machine.rs          # 状态机
│   │   ├── context_manager.rs        # 上下文管理（去重、裁剪）
│   │   └── error_isolator.rs         # 错误隔离器
│   ├── ipc/                          # IPC 通信模块
│   │   ├── mod.rs
│   │   ├── agent_protocol.rs         # Agent 协议定义
│   │   └── message_handler.rs        # 消息处理器
│   ├── automation/                   # 自动化模块
│   │   ├── mod.rs
│   │   ├── keyboard_simulator.rs     # 键盘模拟
│   │   └── clipboard_manager.rs      # 剪贴板管理
│   └── commands.rs                   # Tauri 命令
├── platform_agents/                  # Platform Agent 源码
│   ├── windows/
│   │   ├── wxauto_agent.py           # Windows Agent (wxauto)
│   │   └── requirements.txt
│   └── macos/
│       ├── wechat_agent.swift        # macOS Agent (Accessibility)
│       └── scripts.applescript       # AppleScript 辅助脚本
├── frontend/                         # React 前端源码
│   └── src/
│       ├── components/
│       │   ├── WeChatAssistant/
│       │   │   ├── AssistantPanel.tsx       # 助手主面板
│       │   │   ├── SuggestionList.tsx       # 建议列表
│       │   │   ├── SuggestionItem.tsx       # 单个建议项
│       │   │   └── ConfigDialog.tsx         # 配置对话框
│       │   └── common/                      # 通用组件
│       ├── hooks/
│       │   ├── useWeChatMonitor.ts          # 微信监听 Hook
│       │   └── useDeepSeekSuggestions.ts    # DeepSeek 建议 Hook
│       └── bindings.ts                      # 自动生成的类型绑定
├── docs/                             # 文档目录
│   ├── architecture.md               # 架构设计
│   ├── ipc.md                        # IPC 协议规范
│   ├── ai-deepseek.md                # DeepSeek API 规范
│   ├── platforms.md                  # Platform Agents 实现
│   ├── ui-flow.md                    # UI 状态机与交互流程
│   ├── tauri-api.md                  # 前后端命令与事件
│   ├── configuration.md              # 配置项与优先级
│   ├── logging.md                    # 日志与诊断策略
│   ├── testing.md                    # 测试与验收清单
│   ├── security.md                   # 安全与隐私
│   └── roadmap.md                    # 里程碑
└── .claude/                          # Claude Code 规则
```

## 核心功能模块

### 1. 微信监听与自动化
- 前台聊天窗口消息监听
- 消息去重与上下文管理
- 文本写入微信输入框（不自动发送）
- 窗口跟踪与定位

### 2. DeepSeek 回复建议
- DeepSeek API 调用
- 多风格回复生成（正式、亲切、幽默等）
- 上下文感知的建议生成
- 流式/非流式响应处理

### 3. IPC 通信
- Rust Orchestrator ↔ Platform Agent 通信
- JSON 协议（`message.new`, `input.write`, `input.result` 等）
- Agent 崩溃检测与自动重启
- 超时处理与错误隔离

### 4. 状态机与协调
- 消息接收 → 去重 → 上下文裁剪 → DeepSeek 调用
- 建议展示 → 用户选择 → 输入写入 → 结果上报
- 错误降级策略

### 5. UI 与配置
- 助手主面板（建议列表、编辑功能）
- 配置对话框（API 密钥、回复风格、窗口设置）
- 跨平台 UI 一致性
- 实时建议展示

## 项目特点

1. **跨平台桌面应用**：使用 Tauri 构建 Windows/macOS 桌面应用
2. **分平台自动化**：Windows (wxauto) / macOS (Accessibility) Agent
3. **IPC 通信架构**：Rust Orchestrator 与 Python/Swift Agent 解耦
4. **DeepSeek 专用**：仅支持 DeepSeek API（不支持其他 LLM）
5. **前台聊天限定**：只对前台微信窗口生效
6. **非自动发送**：只写入输入框，不自动发送消息
7. **上下文感知**：去重、裁剪、历史消息管理
8. **错误隔离**：Agent 异常不影响主程序运行

## 开发环境

- **Rust**: 2021 Edition
- **Node.js**: 推荐 18+ 版本
- **Python**: 3.9+ (Windows Agent)
- **Swift**: macOS Xcode (macOS Agent)
- **操作系统**: Windows 10+ (微信 4.1), macOS 10.15+
- **内存**: 推荐 8GB+ RAM
- **磁盘**: 推荐 2GB+ 可用空间

## 数据流设计

```
微信窗口（前台）
    ↓ (Platform Agent 监听)
Platform Agent (Python/Swift)
    ↓ (IPC: message.new)
Rust Orchestrator
    ├─ 去重与上下文裁剪
    ├─ DeepSeek API 调用
    └─ 建议生成
    ↓ (Tauri 事件)
React 前端
    ├─ 建议列表展示
    ├─ 用户选择/编辑
    └─ 发送写入命令
    ↓ (Tauri 命令)
Rust Orchestrator
    ↓ (IPC: input.write)
Platform Agent
    ↓ (键盘模拟/剪贴板)
微信输入框（写入，不发送）
    ↓ (IPC: input.result)
Rust Orchestrator → 前端确认
```

## 关键技术难点

### 1. Platform Agent 实现
- **Windows**: wxauto v4 监听前台窗口、获取最新消息
- **macOS**: Accessibility API + AppleScript 获取消息列表

### 2. IPC 通信协议
- Rust 与 Python/Swift Agent 使用 stdin/stdout JSON 通信
- 处理 Agent 崩溃、超时、异常退出
- 消息队列与异步处理

### 3. 上下文管理
- 消息去重（避免重复生成建议）
- 上下文裁剪（控制 DeepSeek API 请求大小）
- 历史消息管理

### 4. 文本输入
- 键盘模拟或剪贴板粘贴
- 处理特殊字符和表情
- 确保输入准确性

### 5. 错误隔离
- Agent 异常不影响主程序
- DeepSeek API 调用失败降级
- 网络异常处理

## 安全考虑

### 1. 数据隐私
- 所有消息处理在本地进行
- 不上传聊天记录到第三方服务器
- DeepSeek API 调用仅发送必要信息（不包含敏感信息）

### 2. API 密钥管理
- 使用系统密钥链存储 API 密钥
- 不在代码中硬编码密钥
- 不在日志中记录 API 密钥

### 3. 风控风险
- 基于 UI 自动化，模拟人工操作
- 避免频繁操作触发安全限制
- 遵守微信软件许可协议

## 配置项

### DeepSeek 配置
- API 密钥（存储在系统密钥链）
- API 端点（默认 DeepSeek 官方）
- 回复风格（正式、亲切、幽默等）
- 模型选择（deepseek-chat 等）

### 微信配置
- 微信窗口自动检测
- 消息监听频率（默认 500ms）
- 建议生成数量（默认 3 条）
- 上下文消息数量（默认 10 条）

### UI 配置
- 助手窗口位置
- 透明度设置
- 主题颜色
- 字体大小

## 性能指标

- 消息监听延迟：< 500ms
- DeepSeek 建议生成时间：< 3s
- 建议展示响应：< 100ms
- 文本输入延迟：< 200ms
- Agent 重启时间：< 2s

## 开发阶段

### Phase 1: 基础架构
- IPC 协议与 Orchestrator
- UI 骨架与配置页
- DeepSeek API 适配层

### Phase 2: Platform Agents
- Windows Agent（wxauto v4）
- macOS Agent（Accessibility）

### Phase 3: 核心功能
- 建议面板与写入流程
- 错误处理与降级策略

### Phase 4: 完善与优化
- 稳定性与权限引导
- 日志与诊断工具
- 性能优化
