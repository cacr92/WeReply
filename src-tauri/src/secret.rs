use anyhow::{Context, Result};
use keyring::Entry;

const SERVICE_NAME: &str = "wereply";
const API_KEY_NAME: &str = "deepseek_api_key";

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reject_invalid_key_format() {
        let result = ApiKeyManager::set_deepseek_api_key("invalid-key");
        assert!(result.is_err());
    }
}
