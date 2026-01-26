<div align="center">
  <h1>WeReply - 微信回复建议助手</h1>
  <p>监听当前聊天，生成 DeepSeek 回复建议，一键写入输入框（不自动发送）</p>
  <p>
    <img alt="platform" src="https://img.shields.io/badge/platform-windows%20%7C%20macOS-blue" />
    <img alt="tauri" src="https://img.shields.io/badge/Tauri-2.x-0f6fff" />
    <img alt="react" src="https://img.shields.io/badge/React-19-149eca" />
    <img alt="license" src="https://img.shields.io/badge/license-MIT-black" />
  </p>
</div>

---

## 为什么是 WeReply
- 专注助手体验：只写入输入框，不自动发送，避免误发。
- 低侵入：当前聊天窗口监听，建议生成后即点即用。
- 安全可控：API Key 存系统密钥链，不落地明文。
- 跨平台一致：Windows(wxauto) + macOS(Accessibility) 统一体验。

## 核心流程
1. 启动监听当前聊天消息。
2. DeepSeek 生成三种风格建议（正式/中性/轻松）。
3. 点击建议一键写入输入框。
4. 用户自行确认并发送。

## 功能亮点
| 功能 | 说明 |
| --- | --- |
| 实时监听 | Agent 轮询当前聊天窗口消息，去重后触发建议生成。 |
| 回复建议 | 默认 3 条风格化建议，支持模型选择；无 Key 或失败时降级。 |
| 连接诊断 | 一键检测聊天与模型接口，定位网络或鉴权问题。 |
| 安全密钥 | DeepSeek API Key 存入系统密钥链，可随时删除。 |
| 轻量写入 | 写入输入框但不自动发送，支持恢复剪贴板。 |

## 平台支持与权限
| 平台 | 依赖/权限 | 备注 |
| --- | --- | --- |
| Windows | Python（系统或内置）、wxauto(vendor)、pyautogui/pyperclip/comtypes | 运行时自动检查并安装依赖；优先使用内置 Python（如打包） |
| macOS | Accessibility 权限、AppleScript | 需允许辅助功能权限；支持 com.tencent.xinWeChat / com.tencent.WeChat |

## 架构概览
```
UI (React)
  <-> Tauri commands/events
Rust Orchestrator
  <-> JSON IPC
Platform Agent (Python/Swift)
  <-> WeChat UI
```

- Orchestrator 维护状态机、去重、DeepSeek 调度与降级策略。
- Agent 负责监听与写入，消息通过 stdin/stdout JSON 传输。

## 快速开始（开发者）
```bash
npm install
npm run tauri:dev
```

常用命令：
```bash
npm run lint
npm test
npm run tauri:build
cargo test -p wereply
cargo run -p wereply --bin generate_bindings
```

## 目录结构
```
src/                     # React UI
src-tauri/src/            # Rust Orchestrator
platform_agents/windows/  # Windows Agent (Python + wxauto)
platform_agents/macos/    # macOS Agent (Swift + AppleScript)
```

## 配置与安全
- API Key 必须以 `sk-` 开头，存储在系统密钥链。
- 运行时配置以默认值为主，仅持久化 `deepseek_model` 到 `config.json`。
- `.env.example` 仅用于字段说明，当前运行不读取环境变量。

默认配置（节选）：
| 配置项 | 默认值 |
| --- | --- |
| suggestion_count | 3 |
| context_max_messages | 10 |
| context_max_chars | 2000 |
| poll_interval_ms | 800 |
| timeout_ms | 12000 |
| base_url | https://api.deepseek.com |

## 常见问题
- 无建议生成：确认已保存 API Key，或在设置中点击“连接诊断”。
- Agent 未连接：检查 WeChat 是否运行，Windows 需确保 Python 可用，macOS 需授权 Accessibility。
- 写入失败：确认当前聊天窗口在前台并可输入。

## 贡献
请阅读 `CONTRIBUTING.md`。

## 许可证
MIT
