#[cfg(test)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WatchMode {
    Event,
    Polling,
}

#[cfg(test)]
pub struct MockAxWatcher {
    subscribe_ok: bool,
}

#[cfg(test)]
impl MockAxWatcher {
    pub fn subscribe_fail() -> Self {
        Self { subscribe_ok: false }
    }

    #[allow(dead_code)]
    pub fn subscribe_ok() -> Self {
        Self { subscribe_ok: true }
    }

    pub fn start(&self) -> WatchMode {
        if self.subscribe_ok {
            WatchMode::Event
        } else {
            WatchMode::Polling
        }
    }
}

#[cfg(target_os = "macos")]
pub mod ax {
    use crate::ui_automation::macos::ax::{self, AxElement};
    use anyhow::{anyhow, Result};
    #[cfg(test)]
    use super::WatchMode;

    pub struct AxMessageWatcher {
        window: AxElement,
        list: AxElement,
    }

    impl AxMessageWatcher {
        pub fn new(window: &AxElement) -> Result<Self> {
            let list = find_message_list(window)?;
            Ok(Self {
                window: window.clone(),
                list,
            })
        }

        #[cfg(test)]
        #[allow(dead_code)]
        pub fn start(&self) -> WatchMode {
            WatchMode::Polling
        }

        pub fn latest_message_text(&self) -> Option<String> {
            let mut candidates = Vec::new();
            for row in ax::children(&self.list) {
                if let Some(text) = ax::first_static_text(&row, 4) {
                    candidates.push(text);
                }
            }
            candidates.into_iter().last()
        }

        pub fn window(&self) -> &AxElement {
            &self.window
        }
    }

    fn find_message_list(window: &AxElement) -> Result<AxElement> {
        let candidates = ax::find_lists_with_titles(window, 6);
        if let Some(best) = candidates.into_iter().max_by_key(|item| item.1.len()) {
            return Ok(best.0);
        }
        Err(anyhow!("Message list not found"))
    }
}
