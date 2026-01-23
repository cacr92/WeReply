# Rust 后端开发规范 - WeReply

## 命名规范

| 类型 | 规范 | 示例 |
|------|------|------|
| 结构体 | PascalCase | `WeChatMonitor`, `DeepSeekService`, `AgentMessage` |
| 枚举 | PascalCase | `AgentStatus`, `MessageType`, `SuggestionStyle` |
| 特征 (Trait) | PascalCase | `AgentProtocol`, `MessageHandler` |
| 函数 | snake_case | `listen_messages()`, `generate_suggestions()` |
| 变量 | snake_case | `message_content`, `api_key` |
| 常量 | SCREAMING_SNAKE_CASE | `MAX_CONTEXT_SIZE`, `DEFAULT_TIMEOUT` |
| 模块 | snake_case | `wechat`, `orchestrator`, `ipc` |

## 类型系统规范

### Tauri 命令类型定义
所有 Tauri 命令必须使用 specta 导出类型：

**✓ 正确示例**:
```rust
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Type, Clone)]
#[specta(inline)]
pub struct SuggestionRequest {
    pub context_messages: Vec<String>,
    pub style: SuggestionStyle,
}

#[derive(Serialize, Deserialize, Type, Clone)]
#[specta(inline)]
pub struct Suggestion {
    pub id: String,
    pub content: String,
    pub style: SuggestionStyle,
    pub confidence: f32,
}

#[tauri::command]
#[specta::specta]  // 必须添加此宏
pub async fn generate_suggestions(
    request: SuggestionRequest,
    state: State<'_, AppState>,
) -> ApiResponse<Vec<Suggestion>> {
    // 实现...
}
```

### 错误类型统一处理
使用项目统一的 `ApiResponse` 类型：

**✓ 正确示例**:
```rust
use crate::utils::error::{api_err, api_ok, ApiResponse};

pub async fn call_deepseek_api(prompt: String) -> ApiResponse<String> {
    match reqwest::Client::new()
        .post("https://api.deepseek.com/v1/chat/completions")
        .json(&request_body)
        .send()
        .await
    {
        Ok(response) => api_ok(response.text().await.unwrap()),
        Err(e) => api_err(format!("DeepSeek API 调用失败: {}", e)),
    }
}
```

## 异步编程规范

### IPC 通信使用异步
- Agent 通信必须使用异步 IO
- 使用 `tokio::process::Command` 启动 Agent
- 使用 `tokio::io::{AsyncBufReadExt, AsyncWriteExt}` 读写 stdin/stdout

**✓ 正确示例**:
```rust
use tokio::process::{Command, ChildStdin, ChildStdout};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

pub struct AgentConnection {
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl AgentConnection {
    pub async fn send_message(&mut self, msg: &AgentMessage) -> Result<()> {
        let json = serde_json::to_string(msg)?;
        self.stdin.write_all(json.as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;
        self.stdin.flush().await?;
        Ok(())
    }

    pub async fn receive_message(&mut self) -> Result<AgentMessage> {
        let mut line = String::new();
        self.stdout.read_line(&mut line).await?;
        let msg: AgentMessage = serde_json::from_str(&line)?;
        Ok(msg)
    }
}
```

### 超时处理
使用 `tokio::time::timeout` 处理超时：

```rust
use tokio::time::{timeout, Duration};

pub async fn call_deepseek_with_timeout(
    prompt: String,
    timeout_secs: u64,
) -> Result<String> {
    timeout(
        Duration::from_secs(timeout_secs),
        call_deepseek_api(prompt)
    )
    .await
    .map_err(|_| anyhow!("DeepSeek API 调用超时"))??
}
```

## IPC 通信规范

### JSON 消息协议
Agent 与 Rust 使用 JSON 协议通信：

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AgentMessage {
    #[serde(rename = "message.new")]
    MessageNew {
        content: String,
        sender: String,
        timestamp: u64,
    },

    #[serde(rename = "input.write")]
    InputWrite {
        text: String,
    },

    #[serde(rename = "input.result")]
    InputResult {
        success: bool,
        error: Option<String>,
    },
}
```

### Agent 崩溃处理
优雅处理 Agent 异常退出：

```rust
pub async fn monitor_agent(mut agent: AgentConnection) {
    loop {
        match agent.receive_message().await {
            Ok(msg) => handle_message(msg).await,
            Err(e) => {
                error!("Agent 连接断开: {}", e);
                // 尝试重启 Agent
                if let Err(restart_err) = restart_agent().await {
                    error!("Agent 重启失败: {}", restart_err);
                }
                break;
            }
        }
    }
}
```

## DeepSeek API 集成

### HTTP 客户端使用
使用 `reqwest` 调用 DeepSeek API：

```rust
use reqwest::Client;
use serde_json::json;

pub struct DeepSeekService {
    client: Client,
    api_key: String,
    api_endpoint: String,
}

impl DeepSeekService {
    pub async fn generate_suggestions(
        &self,
        context: Vec<String>,
        style: SuggestionStyle,
    ) -> Result<Vec<String>> {
        let request_body = json!({
            "model": "deepseek-chat",
            "messages": [
                {
                    "role": "system",
                    "content": format!("生成{}风格的回复建议", style.to_string())
                },
                {
                    "role": "user",
                    "content": context.join("\n")
                }
            ],
            "n": 3,
            "temperature": 0.7,
        });

        let response = self.client
            .post(&self.api_endpoint)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request_body)
            .send()
            .await?;

        // 解析响应
        let result: serde_json::Value = response.json().await?;
        let suggestions = result["choices"]
            .as_array()
            .unwrap()
            .iter()
            .map(|choice| choice["message"]["content"].as_str().unwrap().to_string())
            .collect();

        Ok(suggestions)
    }
}
```

## 错误处理规范

### 使用 anyhow 进行错误传播
```rust
use anyhow::{Context, Result};

pub async fn process_message(msg: String) -> Result<Suggestion> {
    let parsed = parse_message(&msg)
        .context("解析消息失败")?;

    let suggestion = generate_suggestion(parsed)
        .await
        .context("生成建议失败")?;

    Ok(suggestion)
}
```

### 顶层统一转换
```rust
#[tauri::command]
#[specta::specta]
pub async fn handle_new_message(
    message: String,
    state: State<'_, AppState>,
) -> ApiResponse<Vec<Suggestion>> {
    with_service(state, |ctx| async move {
        ctx.orchestrator.process_message(message).await
    })
    .await
}
```

## 日志记录规范

### 使用结构化日志
使用 `tracing` 而不是 `log`：

```rust
use tracing::{info, warn, error, debug};

pub async fn process_wechat_message(msg: String) -> Result<()> {
    info!(message_length = msg.len(), "收到微信消息");

    match generate_suggestions(&msg).await {
        Ok(suggestions) => {
            info!(
                suggestion_count = suggestions.len(),
                "建议生成成功"
            );
            Ok(())
        }
        Err(e) => {
            error!(
                error = %e,
                "建议生成失败"
            );
            Err(e)
        }
    }
}
```

### 日志级别使用
- `error!`: 错误，需要立即关注（Agent 崩溃、API 调用失败）
- `warn!`: 警告，可能存在问题（超时、重试）
- `info!`: 重要信息，业务流程关键点（消息接收、建议生成）
- `debug!`: 调试信息，开发时使用（IPC 消息内容）
- `trace!`: 详细追踪，性能分析时使用

## 代码组织规范

### 模块结构
```rust
// mod.rs
pub mod monitor;
pub mod automation;

pub use monitor::WeChatMonitor;
pub use automation::InputWriter;
```

### 依赖注入
使用构造函数注入依赖：

```rust
pub struct Orchestrator {
    deepseek_service: Arc<DeepSeekService>,
    context_manager: Arc<ContextManager>,
    agent_connection: Arc<RwLock<AgentConnection>>,
}

impl Orchestrator {
    pub fn new(
        deepseek_service: Arc<DeepSeekService>,
        context_manager: Arc<ContextManager>,
        agent_connection: Arc<RwLock<AgentConnection>>,
    ) -> Self {
        Self {
            deepseek_service,
            context_manager,
            agent_connection,
        }
    }
}
```

## 常见陷阱

### 避免阻塞异步运行时
**✗ 错误示例**:
```rust
pub async fn process_data() {
    std::thread::sleep(Duration::from_secs(1));  // ✗ 阻塞整个运行时
}
```

**✓ 正确示例**:
```rust
pub async fn process_data() {
    tokio::time::sleep(Duration::from_secs(1)).await;  // ✓ 异步睡眠
}
```

### 处理 Agent 断开连接
**✓ 正确示例**:
```rust
match agent.receive_message().await {
    Ok(msg) => handle_message(msg).await,
    Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => {
        warn!("Agent 断开连接，尝试重启");
        restart_agent().await?;
    }
    Err(e) => return Err(e.into()),
}
```

## API 密钥管理

### 使用系统密钥链
```rust
use keyring::Entry;

pub fn get_deepseek_api_key() -> Result<String> {
    let entry = Entry::new("wereply", "deepseek_api_key")?;
    entry.get_password()
        .context("未找到 DeepSeek API 密钥，请在设置中配置")
}

pub fn set_deepseek_api_key(api_key: &str) -> Result<()> {
    let entry = Entry::new("wereply", "deepseek_api_key")?;
    entry.set_password(api_key)?;
    Ok(())
}
```

**禁止硬编码**:
```rust
// ✗ 禁止
const API_KEY: &str = "sk-1234567890abcdef";

// ✓ 正确
let api_key = get_deepseek_api_key()?;
```
