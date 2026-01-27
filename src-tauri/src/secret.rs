use anyhow::{Context, Result};
use keyring::Entry;

const SERVICE_NAME: &str = "wereply";
const API_KEY_NAME: &str = "deepseek_api_key";
const WECHAT_DB_KEY_NAME: &str = "wechat_db_key";

pub struct ApiKeyManager;

impl ApiKeyManager {
    pub fn get_deepseek_api_key() -> Result<String> {
        let entry = Entry::new(SERVICE_NAME, API_KEY_NAME)
            .context("初始化系统密钥链失败")?;
        entry
            .get_password()
            .context("未找到 DeepSeek API 密钥，请在设置中配置")
    }

    pub fn set_deepseek_api_key(api_key: &str) -> Result<()> {
        if !api_key.starts_with("sk-") {
            anyhow::bail!("DeepSeek API 密钥格式错误");
        }
        let entry = Entry::new(SERVICE_NAME, API_KEY_NAME)
            .context("初始化系统密钥链失败")?;
        entry
            .set_password(api_key)
            .context("保存 API 密钥失败")?;
        Ok(())
    }

    pub fn delete_deepseek_api_key() -> Result<()> {
        let entry = Entry::new(SERVICE_NAME, API_KEY_NAME)
            .context("初始化系统密钥链失败")?;
        entry
            .delete_password()
            .context("删除 API 密钥失败")?;
        Ok(())
    }

    pub fn get_wechat_db_key() -> Result<String> {
        let entry = Entry::new(SERVICE_NAME, WECHAT_DB_KEY_NAME)
            .context("初始化系统密钥链失败")?;
        entry
            .get_password()
            .context("未找到 WeChat 数据库密钥")
    }

    pub fn set_wechat_db_key(key_hex: &str) -> Result<()> {
        if !is_hex_string(key_hex) {
            anyhow::bail!("WeChat 数据库密钥格式错误");
        }
        let entry = Entry::new(SERVICE_NAME, WECHAT_DB_KEY_NAME)
            .context("初始化系统密钥链失败")?;
        entry
            .set_password(key_hex)
            .context("保存 WeChat 数据库密钥失败")?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn delete_wechat_db_key() -> Result<()> {
        let entry = Entry::new(SERVICE_NAME, WECHAT_DB_KEY_NAME)
            .context("初始化系统密钥链失败")?;
        entry
            .delete_password()
            .context("删除 WeChat 数据库密钥失败")?;
        Ok(())
    }
}

fn is_hex_string(input: &str) -> bool {
    if input.is_empty() || !input.len().is_multiple_of(2) {
        return false;
    }
    input.chars().all(|ch| ch.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reject_invalid_key_format() {
        let result = ApiKeyManager::set_deepseek_api_key("invalid-key");
        assert!(result.is_err());
    }

    #[test]
    fn reject_invalid_wechat_key_format() {
        assert!(!is_hex_string("zz11"));
        assert!(!is_hex_string("123"));
        assert!(is_hex_string("aabbcc"));
    }
}
