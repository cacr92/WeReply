use crate::types::Config;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;

const CONFIG_FILE: &str = "config.json";

pub fn load_config(app: &AppHandle) -> Result<Config> {
    let _ = config_path(app)?;
    Ok(Config::default())
}

#[allow(dead_code)]
pub fn save_config(app: &AppHandle, config: &Config) -> Result<()> {
    let path = config_path(app)?;
    let contents = serde_json::to_string_pretty(config).context("序列化配置失败")?;
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
        let mut config = Config::default();
        config.suggestion_count = 0;
        assert!(validate_config(&config).is_err());
    }
}
