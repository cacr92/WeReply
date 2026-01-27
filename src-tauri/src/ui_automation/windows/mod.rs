pub mod element;
pub mod input_box;
pub mod message_watch;
pub mod session_list;
pub mod uia;


#[cfg(target_os = "windows")]
pub use input_box::uia::UiaInputWriter;
#[cfg(target_os = "windows")]
pub use message_watch::uia::UiaMessageWatcher;
#[cfg(target_os = "windows")]
pub use session_list::uia::UiaSessionList;
#[cfg(target_os = "windows")]
pub use uia::uia::UiaClient;

#[cfg(test)]
mod tests;

#[cfg(target_os = "windows")]
mod automation {
    use super::message_watch::WatchMode;
    use super::session_list::collect_recent_chats;
    use super::{UiaClient, UiaInputWriter, UiaMessageWatcher, UiaSessionList};
    use crate::types::{ChatSummary, ListenTarget, Platform};
    use crate::ui_automation::{IncomingMessage, WeChatAutomation};
    use anyhow::{anyhow, Result};
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    pub struct WindowsAutomation {
        client: UiaClient,
        watcher: Mutex<Option<UiaMessageWatcher>>,
    }

    impl WindowsAutomation {
        pub fn new() -> Result<Self> {
            Ok(Self {
                client: UiaClient::new()?,
                watcher: Mutex::new(None),
            })
        }

        fn list_chats(&self) -> Result<Vec<ChatSummary>> {
            let window = self.client.pick_wechat_window()?;
            let mut list = UiaSessionList::from_window(self.client.automation(), &window)?;
            collect_recent_chats(&mut list)
        }
    }

    impl WeChatAutomation for WindowsAutomation {
        fn platform(&self) -> Platform {
            Platform::Windows
        }

        fn list_recent_chats(&self) -> Result<Vec<ChatSummary>> {
            self.list_chats()
        }

        fn start_listening(&self, _targets: Vec<ListenTarget>) -> Result<()> {
            let window = self.client.pick_wechat_window()?;
            let mut watcher = UiaMessageWatcher::new(self.client.automation(), &window)?;
            let mode = watcher.start();
            if matches!(mode, WatchMode::Polling | WatchMode::Event) {
                let mut guard = self.watcher.lock().map_err(|_| anyhow!("Watcher lock poisoned"))?;
                *guard = Some(watcher);
                return Ok(());
            }
            Err(anyhow!("Failed to start watcher"))
        }

        fn stop_listening(&self) -> Result<()> {
            let mut guard = self.watcher.lock().map_err(|_| anyhow!("Watcher lock poisoned"))?;
            *guard = None;
            Ok(())
        }

        fn write_input(&self, _chat_id: &str, text: &str) -> Result<()> {
            let window = self.client.pick_wechat_window()?;
            let writer = UiaInputWriter::new(self.client.automation(), &window);
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
            let window = self.client.pick_wechat_window()?;
            let mut list = UiaSessionList::from_window(self.client.automation(), &window).ok();
            let chat_id = list
                .as_ref()
                .and_then(|list| list.active_title())
                .or_else(|| window.get_name().ok())
                .unwrap_or_else(|| "WeChat".to_string());
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            Ok(Some(IncomingMessage {
                chat_id,
                text,
                timestamp,
                msg_id: None,
            }))
        }
    }

    pub use WindowsAutomation;
}

#[cfg(target_os = "windows")]
pub use automation::WindowsAutomation;
