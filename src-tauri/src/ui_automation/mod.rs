pub mod types;

use crate::types::{api_err, api_ok, ApiResponse};
use anyhow::Result;
use std::sync::Arc;
pub use types::{ChatSummary, ListenTarget, Platform};

pub trait WeChatAutomation {
    fn platform(&self) -> Platform;
    fn list_recent_chats(&self) -> Result<Vec<ChatSummary>>;
    fn start_listening(&self, targets: Vec<ListenTarget>) -> Result<()>;
    fn stop_listening(&self) -> Result<()>;
    fn write_input(&self, chat_id: &str, text: &str) -> Result<()>;
}

#[derive(Clone)]
pub struct AutomationManager {
    inner: Option<Arc<dyn WeChatAutomation + Send + Sync>>,
}

impl AutomationManager {
    pub fn new(inner: Option<Arc<dyn WeChatAutomation + Send + Sync>>) -> Self {
        Self { inner }
    }

    pub fn is_ready(&self) -> bool {
        self.inner.is_some()
    }

    pub async fn list_recent_chats(&self) -> ApiResponse<Vec<ChatSummary>> {
        let Some(automation) = self.inner.as_ref() else {
            return api_err("Automation not ready");
        };
        match automation.list_recent_chats() {
            Ok(chats) => api_ok(chats),
            Err(err) => api_err(err.to_string()),
        }
    }
}

#[cfg(test)]
mod tests;
