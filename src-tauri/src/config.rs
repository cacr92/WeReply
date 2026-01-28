use crate::deepseek::is_supported_model;
use crate::types::{Config, ListenTarget};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;
use tracing::warn;

const CONFIG_FILE: &str = "config.json";

#[derive(Debug, Serialize, Deserialize)]
struct StoredConfig {
    deepseek_model: Option<String>,
    listen_targets: Option<Vec<ListenTarget>>,
}

impl StoredConfig {
    fn from_config(config: &Config) -> Self {
        Self {
            deepseek_model: Some(config.deepseek_model.clone()),
            listen_targets: Some(config.listen_targets.clone()),
        }
    }

    fn apply(self, config: &mut Config) {
        if let Some(model) = self.deepseek_model {
            config.deepseek_model = model;
        }
        if let Some(listen_targets) = self.listen_targets {
            config.listen_targets = listen_targets;
        }
    }
}

pub fn load_config(app: &AppHandle) -> Result<Config> {
    let path = config_path(app)?;
    let mut config = Config::default();
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(config),
        Err(err) => {
            return Err(err).with_context(|| format!("读取配置失败: {}", path.display()));
        }
    };
    match serde_json::from_str::<StoredConfig>(&contents) {
        Ok(stored) => stored.apply(&mut config),
        Err(err) => {
            warn!("解析配置失败，使用默认配置: {}", err);
        }
    }
    if let Err(err) = validate_config(&config) {
        warn!("配置校验失败，使用默认配置: {}", err);
        return Ok(Config::default());
    }
    Ok(config)
}

#[allow(dead_code)]
pub fn save_config(app: &AppHandle, config: &Config) -> Result<()> {
    let path = config_path(app)?;
    let stored = StoredConfig::from_config(config);
    let contents = serde_json::to_string_pretty(&stored).context("序列化配置失败")?;
    fs::write(&path, contents).with_context(|| format!("写入配置失败: {}", path.display()))
}

#[allow(dead_code)]
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
    if !is_supported_model(&config.deepseek_model) {
        anyhow::bail!("不支持的模型");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_config_rejects_invalid_values() {
        let config = Config {
            suggestion_count: 0,
            ..Config::default()
        };
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn validate_config_rejects_unknown_model() {
        let config = Config {
            deepseek_model: "unknown".to_string(),
            ..Config::default()
        };
        assert!(validate_config(&config).is_err());
    }
}
