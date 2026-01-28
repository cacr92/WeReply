pub mod ax;
pub mod ax_path;
pub mod ax_learn;
pub mod ax_snapshot;
pub mod message_watch;
pub mod input_box;
pub mod session_list;
pub mod static_ui_paths;


#[cfg(target_os = "macos")]
pub use ax::AxClient;
#[cfg(target_os = "macos")]
pub use input_box::ax::AxInputWriter;
#[cfg(target_os = "macos")]
pub use message_watch::ax::AxMessageWatcher;
#[cfg(target_os = "macos")]
pub use session_list::ax::AxSessionList;

#[cfg(test)]
mod tests;

#[cfg(target_os = "macos")]
mod automation {
    use super::session_list::collect_recent_chats;
    use super::{AxClient, AxInputWriter, AxMessageWatcher, AxSessionList};
    use crate::types::{ChatSummary, ListenTarget, Platform};
    use crate::ui_automation::{IncomingMessage, WeChatAutomation};
    use anyhow::{anyhow, Result};
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tracing::{info, warn};

    pub struct MacosAutomation {
        client: Option<AxClient>,
        watcher: Mutex<Option<AxMessageWatcher>>,
    }

    impl MacosAutomation {
        pub fn new() -> Result<Self> {
            let client = if super::ax::check_accessibility() {
                AxClient::new().ok()
            } else {
                None
            };
            if client.is_none() {
                return Err(anyhow!("WeChat automation unavailable"));
            }
            Ok(Self {
                client,
                watcher: Mutex::new(None),
            })
        }

        fn list_chats(&self) -> Result<Vec<ChatSummary>> {
            let client = self
                .client
                .as_ref()
                .ok_or_else(|| anyhow!("WeChat window not found"))?;
            let window = client
                .front_window()
                .ok_or_else(|| anyhow!("WeChat window not found"))?;
            let mut list = AxSessionList::from_window(&window)?;
            collect_recent_chats(&mut list)
        }
    }

    impl WeChatAutomation for MacosAutomation {
        fn platform(&self) -> Platform {
            Platform::Macos
        }

        fn list_recent_chats(&self) -> Result<Vec<ChatSummary>> {
            self.list_chats()
        }

        fn start_listening(&self, _targets: Vec<ListenTarget>) -> Result<()> {
            info!("macOS 自动化开始监听");
            let client = self
                .client
                .as_ref()
                .ok_or_else(|| anyhow!("WeChat window not found"))?;
            let window = client
                .front_window()
                .ok_or_else(|| anyhow!("WeChat window not found"))?;
            info!("WeChat 窗口已找到，初始化消息监听器");
            let watcher = AxMessageWatcher::new(&window).map_err(|err| {
                warn!("创建消息监听器失败: {}", err);
                err
            })?;
            let mut guard = self
                .watcher
                .lock()
                .map_err(|_| anyhow!("Watcher lock poisoned"))?;
            *guard = Some(watcher);
            info!("macOS 消息监听器已就绪");
            Ok(())
        }

        fn stop_listening(&self) -> Result<()> {
            info!("macOS 自动化停止监听");
            let mut guard = self.watcher.lock().map_err(|_| anyhow!("Watcher lock poisoned"))?;
            *guard = None;
            Ok(())
        }

        fn write_input(&self, _chat_id: &str, text: &str) -> Result<()> {
            let client = self
                .client
                .as_ref()
                .ok_or_else(|| anyhow!("WeChat window not found"))?;
            let window = client
                .front_window()
                .ok_or_else(|| anyhow!("WeChat window not found"))?;
            let writer = AxInputWriter::new(&window);
            writer.write(text)
        }

        fn poll_latest_message(&self) -> Result<Option<IncomingMessage>> {
            let guard = self.watcher.lock().map_err(|_| anyhow!("Watcher lock poisoned"))?;
            let Some(watcher) = guard.as_ref() else {
                return Ok(None);
            };
            let text = match watcher.latest_message_text() {
                Some(text) => text,
                None => return Ok(None),
            };
            let title = super::ax::title(watcher.window())
                .unwrap_or_else(|| "WeChat".to_string());
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            Ok(Some(IncomingMessage {
                chat_id: title,
                text,
                timestamp,
                msg_id: None,
            }))
        }
    }

}

#[cfg(target_os = "macos")]
pub use automation::MacosAutomation;
