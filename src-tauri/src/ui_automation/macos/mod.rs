pub mod ax;
pub mod message_watch;
pub mod input_box;
pub mod session_list;


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

    pub struct MacosAutomation {
        client: AxClient,
        watcher: Mutex<Option<AxMessageWatcher>>,
    }

    impl MacosAutomation {
        pub fn new() -> Result<Self> {
            if !super::ax::check_accessibility() {
                return Err(anyhow!("Accessibility permission required"));
            }
            Ok(Self {
                client: AxClient::new()?,
                watcher: Mutex::new(None),
            })
        }

        fn list_chats(&self) -> Result<Vec<ChatSummary>> {
            let window = self.client.front_window().ok_or_else(|| anyhow!("WeChat window not found"))?;
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
            let window = self.client.front_window().ok_or_else(|| anyhow!("WeChat window not found"))?;
            let watcher = AxMessageWatcher::new(&window)?;
            let mut guard = self.watcher.lock().map_err(|_| anyhow!("Watcher lock poisoned"))?;
            *guard = Some(watcher);
            Ok(())
        }

        fn stop_listening(&self) -> Result<()> {
            let mut guard = self.watcher.lock().map_err(|_| anyhow!("Watcher lock poisoned"))?;
            *guard = None;
            Ok(())
        }

        fn write_input(&self, _chat_id: &str, text: &str) -> Result<()> {
            let window = self.client.front_window().ok_or_else(|| anyhow!("WeChat window not found"))?;
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
