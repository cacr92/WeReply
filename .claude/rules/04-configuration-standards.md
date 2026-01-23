# 配置管理与环境变量规范 - WeReply

## 配置文件结构

WeReply 使用多层配置系统，优先级从高到低：
1. **用户配置**（UI 设置页面）
2. **环境变量**（`.env` 文件）
3. **默认配置**（代码中的默认值）

---

## 一、配置文件格式

### 1. 环境变量文件 (`.env`)

**位置**：项目根目录

**格式**：
```bash
# DeepSeek API 配置
DEEPSEEK_API_ENDPOINT=https://api.deepseek.com/v1/chat/completions
DEEPSEEK_MODEL=deepseek-chat

# 微信监听配置
WECHAT_MONITOR_INTERVAL_MS=500
MAX_CONTEXT_MESSAGES=10
MAX_SUGGESTIONS=3

# UI 配置
ASSISTANT_PANEL_OPACITY=0.95
WINDOW_FOLLOW_INTERVAL_MS=500

# 日志配置
LOG_LEVEL=info
LOG_FILE_PATH=logs/wereply.log
```

**注意事项**：
- ❌ **禁止在 `.env` 文件中存储 API 密钥**
- ❌ **禁止将 `.env` 文件提交到 Git**（已在 `.gitignore` 中排除）
- ✅ 提供 `.env.example` 作为模板

---

## 二、API 密钥管理

### 使用系统密钥链存储

**禁止硬编码或存储在文件中**，必须使用系统密钥链：

**Rust 实现**：
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

**Tauri 命令**：
```rust
#[tauri::command]
#[specta::specta]
pub async fn save_api_key(api_key: String) -> ApiResponse<()> {
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
```

---

## 三、用户配置管理

### 配置结构

```rust
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Type, Clone)]
#[specta(inline)]
pub struct UserConfig {
    pub deepseek_endpoint: String,
    pub deepseek_model: String,
    pub default_style: SuggestionStyle,
    pub max_suggestions: u8,
    pub monitor_interval_ms: u64,
    pub max_context_messages: usize,
    pub window_opacity: f32,
    pub window_follow_enabled: bool,
    pub window_follow_interval_ms: u64,
    pub theme: Theme,
    pub enable_logging: bool,
    pub log_level: LogLevel,
}

#[derive(Serialize, Deserialize, Type, Clone)]
pub enum SuggestionStyle {
    Formal,
    Friendly,
    Humorous,
}

#[derive(Serialize, Deserialize, Type, Clone)]
pub enum Theme {
    Light,
    Dark,
    Auto,
}

#[derive(Serialize, Deserialize, Type, Clone)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}
```

### 配置持久化

```rust
use tauri::Manager;
use std::sync::Arc;
use parking_lot::RwLock;

pub struct ConfigManager {
    config: Arc<RwLock<UserConfig>>,
}

impl ConfigManager {
    pub fn new() -> Self {
        let default_config = UserConfig::default();
        Self {
            config: Arc::new(RwLock::new(default_config)),
        }
    }

    pub fn load(&self, app_handle: &tauri::AppHandle) -> Result<()> {
        let config_path = app_handle
            .path_resolver()
            .app_config_dir()
            .context("无法获取配置目录")?
            .join("config.json");

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: UserConfig = serde_json::from_str(&content)?;
            *self.config.write() = config;
        }

        Ok(())
    }

    pub fn save(&self, app_handle: &tauri::AppHandle) -> Result<()> {
        let config_path = app_handle
            .path_resolver()
            .app_config_dir()
            .context("无法获取配置目录")?
            .join("config.json");

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let config = self.config.read().clone();
        let content = serde_json::to_string_pretty(&config)?;
        std::fs::write(&config_path, content)?;

        Ok(())
    }

    pub fn get(&self) -> UserConfig {
        self.config.read().clone()
    }

    pub fn update<F>(&self, f: F) -> UserConfig
    where
        F: FnOnce(&mut UserConfig),
    {
        let mut config = self.config.write();
        f(&mut config);
        config.clone()
    }
}
```

### Tauri 命令

```rust
#[tauri::command]
#[specta::specta]
pub async fn get_config(
    state: State<'_, AppState>,
) -> ApiResponse<UserConfig> {
    api_ok(state.config_manager.get())
}

#[tauri::command]
#[specta::specta]
pub async fn update_config(
    config: UserConfig,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> ApiResponse<()> {
    state.config_manager.update(|c| *c = config);

    match state.config_manager.save(&app_handle) {
        Ok(()) => api_ok(()),
        Err(e) => api_err(format!("保存配置失败: {}", e)),
    }
}
```

---

## 四、环境变量读取

```rust
use dotenv::dotenv;
use std::env;

pub struct EnvConfig;

impl EnvConfig {
    pub fn load() -> Result<()> {
        dotenv().ok();
        Ok(())
    }

    pub fn get_deepseek_endpoint() -> String {
        env::var("DEEPSEEK_API_ENDPOINT")
            .unwrap_or_else(|_| "https://api.deepseek.com/v1/chat/completions".to_string())
    }

    pub fn get_deepseek_model() -> String {
        env::var("DEEPSEEK_MODEL")
            .unwrap_or_else(|_| "deepseek-chat".to_string())
    }

    pub fn get_monitor_interval_ms() -> u64 {
        env::var("WECHAT_MONITOR_INTERVAL_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(500)
    }

    pub fn get_max_context_messages() -> usize {
        env::var("MAX_CONTEXT_MESSAGES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10)
    }
}
```

---

## 五、前端配置界面

```typescript
import { Form, Input, Select, Switch, Slider, Button } from 'antd';
import { commands } from '../bindings';

export const ConfigDialog: React.FC = () => {
  const [form] = Form.useForm();
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    const result = await commands.getConfig();
    if (result.success) {
      form.setFieldsValue(result.data);
    }
  };

  const handleSave = async (values: UserConfig) => {
    setLoading(true);
    try {
      const result = await commands.updateConfig(values);
      if (result.success) {
        message.success('配置已保存');
      } else {
        message.error(result.message);
      }
    } finally {
      setLoading(false);
    }
  };

  return (
    <Form form={form} onFinish={handleSave} layout="vertical">
      <Form.Item name="deepseek_model" label="DeepSeek 模型">
        <Select>
          <Select.Option value="deepseek-chat">deepseek-chat</Select.Option>
        </Select>
      </Form.Item>

      <Form.Item name="default_style" label="默认回复风格">
        <Select>
          <Select.Option value="formal">正式</Select.Option>
          <Select.Option value="friendly">亲切</Select.Option>
          <Select.Option value="humorous">幽默</Select.Option>
        </Select>
      </Form.Item>

      <Form.Item name="max_suggestions" label="建议数量">
        <Slider min={1} max={5} marks={{ 1: '1', 3: '3', 5: '5' }} />
      </Form.Item>

      <Form.Item name="window_opacity" label="窗口透明度">
        <Slider min={0.5} max={1.0} step={0.05} />
      </Form.Item>

      <Form.Item name="window_follow_enabled" label="启用窗口跟随" valuePropName="checked">
        <Switch />
      </Form.Item>

      <Button type="primary" htmlType="submit" loading={loading}>
        保存配置
      </Button>
    </Form>
  );
};
```

---

## 六、配置验证

```rust
impl UserConfig {
    pub fn validate(&self) -> Result<()> {
        if self.max_suggestions < 1 || self.max_suggestions > 5 {
            return Err(anyhow!("建议数量必须在 1-5 之间"));
        }

        if self.monitor_interval_ms < 100 || self.monitor_interval_ms > 5000 {
            return Err(anyhow!("监听间隔必须在 100-5000ms 之间"));
        }

        if self.window_opacity < 0.5 || self.window_opacity > 1.0 {
            return Err(anyhow!("窗口透明度必须在 0.5-1.0 之间"));
        }

        if !self.deepseek_endpoint.starts_with("http") {
            return Err(anyhow!("API 端点必须以 http:// 或 https:// 开头"));
        }

        Ok(())
    }
}
```
