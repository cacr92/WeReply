pub mod types;
pub mod windows;
pub mod macos;

use crate::types::{api_err, api_ok, ApiResponse};
use anyhow::Result;
use std::sync::Arc;
use tokio::task::spawn_blocking;
pub use types::{ChatSummary, IncomingMessage, ListenTarget, Platform};

pub trait WeChatAutomation {
    fn platform(&self) -> Platform;
    fn list_recent_chats(&self) -> Result<Vec<ChatSummary>>;
    fn start_listening(&self, targets: Vec<ListenTarget>) -> Result<()>;
    fn stop_listening(&self) -> Result<()>;
    fn write_input(&self, chat_id: &str, text: &str) -> Result<()>;
    fn poll_latest_message(&self) -> Result<Option<IncomingMessage>>;
}

pub fn build_platform_automation() -> Option<Arc<dyn WeChatAutomation + Send + Sync>> {
    #[cfg(target_os = "windows")]
    {
        return windows::WindowsAutomation::new()
            .ok()
            .map(|automation| Arc::new(automation) as Arc<dyn WeChatAutomation + Send + Sync>);
    }
    #[cfg(target_os = "macos")]
    {
        return macos::MacosAutomation::new()
            .ok()
            .map(|automation| Arc::new(automation) as Arc<dyn WeChatAutomation + Send + Sync>);
    }
    None
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
        let automation = Arc::clone(automation);
        match spawn_blocking(move || automation.list_recent_chats()).await {
            Ok(Ok(chats)) => api_ok(chats),
            Ok(Err(err)) => api_err(err.to_string()),
            Err(err) => api_err(format!("Automation task failed: {}", err)),
        }
    }

    pub async fn start_listening(&self, targets: Vec<ListenTarget>) -> ApiResponse<()> {
        let Some(automation) = self.inner.as_ref() else {
            return api_err("Automation not ready");
        };
        let automation = Arc::clone(automation);
        match spawn_blocking(move || automation.start_listening(targets)).await {
            Ok(Ok(())) => api_ok(()),
            Ok(Err(err)) => api_err(err.to_string()),
            Err(err) => api_err(format!("Automation task failed: {}", err)),
        }
    }

    pub async fn stop_listening(&self) -> ApiResponse<()> {
        let Some(automation) = self.inner.as_ref() else {
            return api_err("Automation not ready");
        };
        let automation = Arc::clone(automation);
        match spawn_blocking(move || automation.stop_listening()).await {
            Ok(Ok(())) => api_ok(()),
            Ok(Err(err)) => api_err(err.to_string()),
            Err(err) => api_err(format!("Automation task failed: {}", err)),
        }
    }

    pub async fn write_input(&self, chat_id: String, text: String) -> ApiResponse<()> {
        let Some(automation) = self.inner.as_ref() else {
            return api_err("Automation not ready");
        };
        let automation = Arc::clone(automation);
        match spawn_blocking(move || automation.write_input(&chat_id, &text)).await {
            Ok(Ok(())) => api_ok(()),
            Ok(Err(err)) => api_err(err.to_string()),
            Err(err) => api_err(format!("Automation task failed: {}", err)),
        }
    }

    pub async fn poll_latest_message(&self) -> ApiResponse<Option<IncomingMessage>> {
        let Some(automation) = self.inner.as_ref() else {
            return api_err("Automation not ready");
        };
        let automation = Arc::clone(automation);
        match spawn_blocking(move || automation.poll_latest_message()).await {
            Ok(Ok(message)) => api_ok(message),
            Ok(Err(err)) => api_err(err.to_string()),
            Err(err) => api_err(format!("Automation task failed: {}", err)),
        }
    }
}

#[cfg(test)]
mod tests;
