use crate::types::Config;
use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;

const CONFIG_FILE: &str = "config.json";

pub fn load_config(app: &AppHandle) -> Result<Config> {
    let path = config_path(app)?;
    let mut config = if path.exists() {
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("读取配置失败: {}", path.display()))?;
        serde_json::from_str(&contents).context("配置文件格式错误")?
    } else {
        Config::default()
    };
    apply_env_overrides(&mut config);
    Ok(config)
}

pub fn save_config(app: &AppHandle, config: &Config) -> Result<()> {
    let path = config_path(app)?;
    let contents = serde_json::to_string_pretty(config).context("序列化配置失败")?;
    fs::write(&path, contents).with_context(|| format!("写入配置失败: {}", path.display()))
}

pub fn validate_config(config: &Config) -> Result<()> {
    if config.suggestion_count == 0 {
        anyhow::bail!("建议数量必须大于 0");
    }
    if config.context_max_messages == 0 || config.context_max_chars == 0 {
        anyhow::bail!("上下文限制必须大于 0");
    }
    if config.poll_interval_ms < 200 {
        anyhow::bail!("监听间隔不能小于 200ms");
    }
    if !(0.0..=2.0).contains(&config.temperature) {
        anyhow::bail!("temperature 必须在 0.0 到 2.0 之间");
    }
    if !(0.0..=1.0).contains(&config.top_p) {
        anyhow::bail!("top_p 必须在 0.0 到 1.0 之间");
    }
    Ok(())
}

fn config_path(app: &AppHandle) -> Result<PathBuf> {
    let dir = app
        .path()
        .app_config_dir()
        .context("无法获取配置目录")?;
    fs::create_dir_all(&dir).context("创建配置目录失败")?;
    Ok(dir.join(CONFIG_FILE))
}

fn apply_env_overrides(config: &mut Config) {
    if let Ok(value) = env::var("DEEPSEEK_BASE_URL") {
        if !value.trim().is_empty() {
            config.base_url = value;
        }
    }
    if let Ok(value) = env::var("DEEPSEEK_MODEL") {
        if !value.trim().is_empty() {
            config.deepseek_model = value;
        }
    }
    if let Ok(value) = env::var("DEEPSEEK_TIMEOUT_MS") {
        if let Ok(parsed) = value.parse::<u64>() {
            config.timeout_ms = parsed;
        }
    }
    if let Ok(value) = env::var("DEEPSEEK_MAX_RETRIES") {
        if let Ok(parsed) = value.parse::<u32>() {
            config.max_retries = parsed;
        }
    }
    if let Ok(value) = env::var("DEEPSEEK_TEMPERATURE") {
        if let Ok(parsed) = value.parse::<f32>() {
            config.temperature = parsed;
        }
    }
    if let Ok(value) = env::var("DEEPSEEK_TOP_P") {
        if let Ok(parsed) = value.parse::<f32>() {
            config.top_p = parsed;
        }
    }
    if let Ok(value) = env::var("SUGGESTION_COUNT") {
        if let Ok(parsed) = value.parse::<u32>() {
            config.suggestion_count = parsed;
        }
    }
    if let Ok(value) = env::var("CONTEXT_MAX_MESSAGES") {
        if let Ok(parsed) = value.parse::<u32>() {
            config.context_max_messages = parsed;
        }
    }
    if let Ok(value) = env::var("CONTEXT_MAX_CHARS") {
        if let Ok(parsed) = value.parse::<u32>() {
            config.context_max_chars = parsed;
        }
    }
    if let Ok(value) = env::var("POLL_INTERVAL_MS") {
        if let Ok(parsed) = value.parse::<u64>() {
            config.poll_interval_ms = parsed;
        }
    }
    if let Ok(value) = env::var("LOG_LEVEL") {
        if !value.trim().is_empty() {
            config.log_level = value;
        }
    }
    if let Ok(value) = env::var("LOG_TO_FILE") {
        config.log_to_file = value == "true" || value == "1";
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_config_rejects_invalid_values() {
        let mut config = Config::default();
        config.suggestion_count = 0;
        assert!(validate_config(&config).is_err());
    }
}
