# 安全开发规范 - WeReply

## 桌面应用安全特点

本项目是 Tauri 桌面应用，安全重点与 Web 应用不同：
- ✅ 重点：API 密钥管理、IPC 通信安全、隐私保护、Tauri 命令安全
- ❌ 不适用：CSRF、CSP（这些是 Web 应用的安全措施）

---

## 强制安全检查清单

### 代码提交前必查
- [ ] 无硬编码密钥（API keys、密码、tokens）
- [ ] DeepSeek API 密钥使用系统密钥链存储
- [ ] 所有 Tauri 命令参数已验证
- [ ] IPC 消息已验证（防止恶意 Agent 消息）
- [ ] 用户输入已验证（前端 + 后端双重验证）
- [ ] 日志中无敏感信息（聊天内容、API 密钥）
- [ ] 错误消息不暴露敏感信息
- [ ] 避免使用 `unsafe` 代码（除非绝对必要）

---

## 1. API 密钥管理（系统密钥链）

### 禁止使用环境变量或文件存储
**WeReply 特定要求**：DeepSeek API 密钥必须使用系统密钥链存储。

**✓ 正确示例**：
```rust
use keyring::Entry;
use anyhow::{Context, Result};

pub struct ApiKeyManager;

impl ApiKeyManager {
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

    pub fn delete_deepseek_api_key() -> Result<()> {
        let entry = Entry::new("wereply", "deepseek_api_key")?;
        entry.delete_password()?;
        Ok(())
    }
}
```

**✗ 错误示例**：
```rust
// ✗ 禁止硬编码
const API_KEY: &str = "sk-1234567890abcdef";

// ✗ 禁止使用环境变量（不安全）
use std::env;
pub fn get_api_key() -> Result<String> {
    env::var("DEEPSEEK_API_KEY")  // ✗ 不安全
}

// ✗ 禁止存储在配置文件
pub struct AppConfig {
    pub api_key: String,  // ✗ 明文存储
}
```

### Tauri 命令接口
```rust
#[tauri::command]
#[specta::specta]
pub async fn save_api_key(api_key: String) -> ApiResponse<()> {
    // 验证密钥格式
    if !api_key.starts_with("sk-") {
        return api_err("DeepSeek API 密钥格式错误".to_string());
    }

    match ApiKeyManager::set_deepseek_api_key(&api_key) {
        Ok(()) => api_ok(()),
        Err(e) => api_err(format!("保存 API 密钥失败: {}", e)),
    }
}

#[tauri::command]
#[specta::specta]
pub async fn get_api_key_status() -> ApiResponse<bool> {
    match ApiKeyManager::get_deepseek_api_key() {
        Ok(_) => api_ok(true),
        Err(_) => api_ok(false),
    }
}

#[tauri::command]
#[specta::specta]
pub async fn delete_api_key() -> ApiResponse<()> {
    match ApiKeyManager::delete_deepseek_api_key() {
        Ok(()) => api_ok(()),
        Err(e) => api_err(format!("删除 API 密钥失败: {}", e)),
    }
}
```

---

## 2. IPC 通信安全

### Agent 消息验证
**强制要求**：所有从 Platform Agent 接收的消息必须验证。

**✓ 正确示例**：
```rust
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Validate)]
#[serde(tag = "type")]
pub enum AgentMessage {
    #[serde(rename = "message.new")]
    MessageNew {
        #[validate(length(max = 10000))]  // 限制消息长度，防止内存攻击
        content: String,

        #[validate(length(max = 100))]
        sender: String,

        timestamp: u64,
    },

    #[serde(rename = "input.result")]
    InputResult {
        success: bool,

        #[validate(length(max = 1000))]
        error: Option<String>,
    },
}

pub async fn handle_agent_message(raw_message: &str) -> Result<()> {
    // 1. 验证消息长度
    if raw_message.len() > 100000 {
        return Err(anyhow!("Agent 消息过大"));
    }

    // 2. 解析 JSON
    let message: AgentMessage = serde_json::from_str(raw_message)
        .context("Agent 消息格式错误")?;

    // 3. 验证消息内容
    match &message {
        AgentMessage::MessageNew { content, sender, .. } => {
            if content.is_empty() {
                return Err(anyhow!("消息内容为空"));
            }
            if sender.is_empty() {
                return Err(anyhow!("发送者为空"));
            }
        }
        AgentMessage::InputResult { .. } => {
            // 验证结果消息
        }
    }

    // 4. 处理消息
    process_message(message).await
}
```

**✗ 错误示例**：
```rust
// ✗ 直接解析，不验证
pub async fn handle_agent_message(raw_message: &str) -> Result<()> {
    let message: AgentMessage = serde_json::from_str(raw_message)?;
    process_message(message).await  // ✗ 没有验证消息合法性
}
```

### Agent 进程安全
```rust
use tokio::process::Command;

pub async fn spawn_agent_safely() -> Result<Child> {
    let agent_path = validate_agent_path()?;

    // 使用受限权限启动 Agent
    let child = Command::new(&agent_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("启动 Agent 失败")?;

    Ok(child)
}

fn validate_agent_path() -> Result<PathBuf> {
    let agent_path = get_agent_executable_path()?;

    // 验证路径安全性
    if !agent_path.exists() {
        return Err(anyhow!("Agent 可执行文件不存在"));
    }

    // 检查文件权限（可选）
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(&agent_path)?;
        let permissions = metadata.permissions();

        // 确保只有所有者可以写入
        if permissions.mode() & 0o022 != 0 {
            return Err(anyhow!("Agent 文件权限不安全"));
        }
    }

    Ok(agent_path)
}
```

### Agent 通信超时
```rust
use tokio::time::{timeout, Duration};

pub async fn send_message_with_timeout(
    agent: &mut AgentConnection,
    message: &AgentMessage,
) -> Result<()> {
    timeout(
        Duration::from_secs(5),
        agent.send_message(message)
    )
    .await
    .map_err(|_| anyhow!("发送消息超时"))?
}

pub async fn receive_message_with_timeout(
    agent: &mut AgentConnection,
) -> Result<AgentMessage> {
    timeout(
        Duration::from_secs(10),
        agent.receive_message()
    )
    .await
    .map_err(|_| anyhow!("接收消息超时"))?
}
```

---

## 3. Tauri 命令安全

### 参数验证
所有 Tauri 命令必须验证参数：

**✓ 正确示例**：
```rust
use validator::Validate;

#[derive(Deserialize, Validate, Type)]
pub struct GenerateSuggestionsRequest {
    #[validate(length(min = 1, max = 10))]
    pub context_messages: Vec<String>,

    #[validate(custom = "validate_suggestion_style")]
    pub style: SuggestionStyle,
}

fn validate_suggestion_style(style: &SuggestionStyle) -> Result<(), ValidationError> {
    match style {
        SuggestionStyle::Formal | SuggestionStyle::Friendly | SuggestionStyle::Humorous => Ok(()),
        _ => Err(ValidationError::new("invalid_style")),
    }
}

#[tauri::command]
#[specta::specta]
pub async fn generate_suggestions(
    request: GenerateSuggestionsRequest,
    state: State<'_, AppState>,
) -> ApiResponse<Vec<Suggestion>> {
    // 验证输入
    if let Err(e) = request.validate() {
        return api_err(format!("输入验证失败: {}", e));
    }

    // 处理逻辑...
}
```

### 权限控制
敏感操作应添加权限检查：

```rust
#[tauri::command]
#[specta::specta]
pub async fn clear_all_suggestions(
    state: State<'_, AppState>,
) -> ApiResponse<()> {
    // 可以添加二次确认机制
    // 或者要求用户确认密码

    // 执行清除...
}
```

---

## 4. 隐私保护

### 日志中不记录敏感信息
**强制要求**：禁止在日志中记录微信聊天内容、API 密钥等敏感信息。

**✓ 正确示例**：
```rust
use tracing::info;

info!(
    message_count = context.len(),
    suggestion_style = ?style,
    "生成建议请求"
);
// ✓ 不记录聊天内容、API 密钥
```

**✗ 错误示例**：
```rust
// ✗ 禁止记录敏感信息
info!("聊天内容: {}", message_content);  // ✗ 泄露隐私
info!("DeepSeek API Key: {}", api_key);  // ✗ 泄露密钥
info!("用户微信号: {}", wechat_id);  // ✗ 泄露用户信息
```

### DeepSeek API 调用日志
```rust
pub async fn call_deepseek_api(
    prompt: String,
    api_key: &str,
) -> Result<String> {
    info!(
        prompt_length = prompt.len(),
        "调用 DeepSeek API"
    );
    // ✓ 只记录长度，不记录内容

    let response = reqwest::Client::new()
        .post("https://api.deepseek.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_body)
        .send()
        .await?;

    info!(
        status = response.status().as_u16(),
        "DeepSeek API 响应"
    );
    // ✓ 只记录状态码

    Ok(response.text().await?)
}
```

### 错误消息不暴露内部信息
**✓ 正确示例**：
```rust
pub async fn generate_suggestions(
    request: GenerateSuggestionsRequest,
) -> ApiResponse<Vec<Suggestion>> {
    match deepseek_service.generate_suggestions(request).await {
        Ok(suggestions) => api_ok(suggestions),
        Err(e) => {
            error!(error = %e, "建议生成失败");
            // 返回通用错误消息，不暴露内部细节
            api_err("建议生成失败，请检查 API 密钥配置".to_string())
        }
    }
}
```

### Agent 通信日志
```rust
pub async fn monitor_agent_messages(
    agent: &mut AgentConnection,
) -> Result<()> {
    loop {
        match agent.receive_message().await {
            Ok(message) => {
                info!(
                    message_type = message.message_type(),
                    "收到 Agent 消息"
                );
                // ✓ 只记录消息类型，不记录内容

                handle_message(message).await?;
            }
            Err(e) => {
                error!(error = %e, "Agent 消息接收失败");
                break;
            }
        }
    }

    Ok(())
}
```

---

## 5. 输入验证

### 前端验证（第一道防线）
使用 Ant Design Form 验证：

```typescript
<Form.Item
  name="apiKey"
  label="DeepSeek API 密钥"
  rules={[
    { required: true, message: '请输入 API 密钥' },
    { pattern: /^sk-/, message: 'API 密钥格式错误（应以 sk- 开头）' },
    { min: 20, message: 'API 密钥长度不足' }
  ]}
>
  <Input.Password />
</Form.Item>

<Form.Item
  name="maxSuggestions"
  label="建议数量"
  rules={[
    { required: true, message: '请选择建议数量' },
    { type: 'number', min: 1, max: 5, message: '建议数量必须在 1-5 之间' }
  ]}
>
  <InputNumber />
</Form.Item>
```

### 后端验证（必须有）
**永远不要信任前端验证**，后端必须再次验证：

```rust
use validator::{Validate, ValidationError};

#[derive(Deserialize, Validate, Type)]
pub struct UserConfig {
    #[validate(length(min = 1, max = 100))]
    pub deepseek_endpoint: String,

    #[validate(range(min = 1, max = 5))]
    pub max_suggestions: u8,

    #[validate(range(min = 100, max = 5000))]
    pub monitor_interval_ms: u64,

    #[validate(range(min = 0.5, max = 1.0))]
    pub window_opacity: f32,
}

#[tauri::command]
#[specta::specta]
pub async fn update_config(
    config: UserConfig,
    state: State<'_, AppState>,
) -> ApiResponse<()> {
    // 验证配置
    if let Err(e) = config.validate() {
        return api_err(format!("配置验证失败: {}", e));
    }

    // 额外验证
    if !config.deepseek_endpoint.starts_with("http") {
        return api_err("API 端点必须以 http:// 或 https:// 开头".to_string());
    }

    // 保存配置...
    api_ok(())
}
```

---

## 6. DeepSeek API 调用安全

### HTTPS 强制使用
```rust
pub fn validate_api_endpoint(endpoint: &str) -> Result<()> {
    if !endpoint.starts_with("https://") {
        return Err(anyhow!("API 端点必须使用 HTTPS"));
    }
    Ok(())
}
```

### 请求超时
```rust
use tokio::time::{timeout, Duration};

pub async fn call_deepseek_with_timeout(
    prompt: String,
    api_key: &str,
    timeout_secs: u64,
) -> Result<String> {
    timeout(
        Duration::from_secs(timeout_secs),
        call_deepseek_api(prompt, api_key)
    )
    .await
    .map_err(|_| anyhow!("DeepSeek API 调用超时"))?
}
```

### 错误处理
```rust
pub async fn call_deepseek_api(
    prompt: String,
    api_key: &str,
) -> Result<String> {
    let response = reqwest::Client::new()
        .post("https://api.deepseek.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_body)
        .send()
        .await
        .context("DeepSeek API 请求失败")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();

        error!(
            status = status.as_u16(),
            "DeepSeek API 错误"
        );
        // ✓ 不记录错误详情（可能包含敏感信息）

        return Err(anyhow!("DeepSeek API 调用失败"));
    }

    Ok(response.text().await?)
}
```

---

## 7. 依赖安全

### 定期更新依赖
```bash
# 检查过时的依赖
cargo outdated

# 检查安全漏洞
cargo audit

# 更新依赖
cargo update
```

### 审查新依赖
添加新依赖前检查：
- [ ] 依赖是否活跃维护
- [ ] 是否有已知安全漏洞
- [ ] 下载量和社区评价
- [ ] 许可证是否兼容

---

## 8. Rust 特定安全

### 避免 unsafe 代码
**原则**：除非绝对必要，否则避免使用 `unsafe`。

**✓ 可接受的 unsafe 使用场景**：
- FFI 调用（与 Python/Swift Agent 交互）
- 性能关键路径（经过充分测试）
- 与 C 库交互

**✗ 不可接受的 unsafe 使用**：
- 为了绕过编译器检查
- 未经充分测试的代码
- 有安全替代方案的情况

### 使用 Rust 安全特性
```rust
// ✓ 使用 Option 而非空指针
pub fn find_suggestion(id: &str) -> Option<Suggestion> {
    // ...
}

// ✓ 使用 Result 进行错误处理
pub fn parse_agent_message(json: &str) -> Result<AgentMessage> {
    // ...
}

// ✓ 使用借用检查器防止数据竞争
pub fn process_messages(messages: &[WeChatMessage]) -> Vec<Suggestion> {
    // 编译器保证内存安全
}
```

---

## 9. 前端安全（React）

### 避免 XSS
**✓ 正确示例**：
```typescript
// ✓ React 默认转义内容
<div>{suggestionContent}</div>

// ✓ 如需渲染富文本，使用 DOMPurify
import DOMPurify from 'dompurify';
const clean = DOMPurify.sanitize(userInput);
<div dangerouslySetInnerHTML={{ __html: clean }} />
```

**✗ 错误示例**：
```typescript
// ✗ 直接渲染未清理的 HTML
<div dangerouslySetInnerHTML={{ __html: userInput }} />
```

### 验证 Tauri 命令响应
```typescript
const result = await commands.generateSuggestions(request);
if (!result.success) {
  message.error(result.message);
  return;
}

// 验证数据结构
if (!result.data || !Array.isArray(result.data)) {
  message.error('建议数据格式错误');
  return;
}

// 使用数据
setSuggestions(result.data);
```

---

## 10. 文件操作安全

### 路径验证
处理用户提供的文件路径时，验证路径安全：

```rust
use std::path::{Path, PathBuf};

pub fn validate_file_path(path: &str) -> Result<PathBuf> {
    let path = Path::new(path);

    // 检查路径遍历攻击
    if path.components().any(|c| c == std::path::Component::ParentDir) {
        return Err(anyhow!("不允许使用 .. 路径"));
    }

    // 检查绝对路径
    if path.is_absolute() {
        return Err(anyhow!("不允许使用绝对路径"));
    }

    Ok(path.to_path_buf())
}
```

### 配置文件验证
```rust
pub fn validate_config_file(path: &Path) -> Result<()> {
    let extension = path.extension()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("无效的文件扩展名"))?;

    match extension.to_lowercase().as_str() {
        "json" => Ok(()),
        _ => Err(anyhow!("不支持的配置文件类型: {}", extension)),
    }
}
```

---

## 11. 安全响应协议

### 发现漏洞时的处理流程
1. **立即停止工作**
2. **评估漏洞严重性**
   - 高危：可能导致 API 密钥泄露、聊天内容泄露
   - 中危：可能导致功能异常、Agent 崩溃
   - 低危：理论风险，实际影响小
3. **修复漏洞**
4. **轮换暴露的密钥**（如适用）
5. **审计整个代码库**查找类似问题
6. **更新安全检查清单**

---

## 安全审查触发条件

以下情况必须进行安全审查：
- [ ] 添加新的 Tauri 命令
- [ ] 修改 IPC 通信层
- [ ] 修改 DeepSeek API 调用逻辑
- [ ] 添加新的 Agent 消息类型
- [ ] 集成第三方 API
- [ ] 添加新的配置项
- [ ] 修改日志记录逻辑

---

## 提交前安全检查清单

- [ ] 运行 `cargo clippy` 无安全警告
- [ ] 运行 `cargo audit` 无已知漏洞
- [ ] 无硬编码 API 密钥
- [ ] DeepSeek API 密钥使用系统密钥链
- [ ] 所有 IPC 消息已验证
- [ ] 所有 Tauri 命令参数已验证
- [ ] 日志中无敏感信息（聊天内容、API 密钥）
- [ ] 错误消息不暴露内部细节
- [ ] 无不必要的 `unsafe` 代码
- [ ] 前端无 `dangerouslySetInnerHTML`（或已清理）

---

## 常见安全陷阱

### 1. 信任前端验证
**✗ 错误**：只在前端验证，后端不验证
**✓ 正确**：前后端都验证，后端验证是最后防线

### 2. 日志记录敏感信息
**✗ 错误**：`info!("聊天内容: {}", message)`
**✓ 正确**：`info!("收到新消息")`

### 3. 硬编码 API 密钥
**✗ 错误**：`const API_KEY: &str = "sk-xxx"`
**✓ 正确**：`ApiKeyManager::get_deepseek_api_key()`

### 4. 不验证 Agent 消息
**✗ 错误**：`let msg: AgentMessage = serde_json::from_str(raw)?;`
**✓ 正确**：`validate_agent_message(raw)?; let msg = parse(raw)?;`

### 5. 过于详细的错误消息
**✗ 错误**：`api_err(format!("DeepSeek API 错误: {}", detailed_error))`
**✓ 正确**：`api_err("建议生成失败，请检查 API 配置".to_string())`

---

## WeReply 特定安全要点

### 1. 微信聊天内容隐私
- ❌ 不记录聊天内容到日志
- ❌ 不上传聊天内容到任何服务器（除 DeepSeek API）
- ✅ 仅在内存中处理消息
- ✅ 及时清理历史消息

### 2. Agent 进程隔离
- ✅ Agent 异常不影响主程序
- ✅ Agent 崩溃自动重启
- ✅ Agent 消息超时处理

### 3. DeepSeek API 调用
- ✅ 使用 HTTPS
- ✅ 设置超时（防止长时间阻塞）
- ✅ 错误处理（不暴露 API 密钥）
- ✅ 请求频率限制（防止滥用）

---

## 参考资源

- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Tauri Security Best Practices](https://tauri.app/v1/references/architecture/security/)
- [keyring-rs Documentation](https://docs.rs/keyring/)
- [reqwest Security](https://docs.rs/reqwest/latest/reqwest/)
