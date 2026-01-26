#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WatchMode {
    Event,
    Polling,
}

pub struct MockWatcher {
    subscribe_ok: bool,
}

impl MockWatcher {
    pub fn subscribe_fail() -> Self {
        Self { subscribe_ok: false }
    }

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

#[cfg(target_os = "windows")]
pub mod uia {
    use super::WatchMode;
    use anyhow::{anyhow, Result};
    use uiautomation::events::{CustomEventHandlerFn, UIEventHandler, UIEventType};
    use uiautomation::types::ControlType;
    use uiautomation::{TreeScope, UIAutomation, UIElement};

    const MESSAGE_LIST_NAMES: [&str; 2] = [
        "\u6d88\u606f",
        "\u804a\u5929\u8bb0\u5f55",
    ];

    pub struct UiaMessageWatcher {
        automation: UIAutomation,
        message_list: UIElement,
        handler: Option<UIEventHandler>,
    }

    impl UiaMessageWatcher {
        pub fn new(automation: &UIAutomation, window: &UIElement) -> Result<Self> {
            let message_list = find_message_list(automation, window)?;
            Ok(Self {
                automation: automation.clone(),
                message_list,
                handler: None,
            })
        }

        pub fn start(&mut self) -> WatchMode {
            if self.try_subscribe().is_ok() {
                WatchMode::Event
            } else {
                WatchMode::Polling
            }
        }

        fn try_subscribe(&mut self) -> Result<()> {
            let handle_fn: Box<CustomEventHandlerFn> = Box::new(|_sender, _event_type| Ok(()));
            let handler = UIEventHandler::from(handle_fn);
            self.automation.add_automation_event_handler(
                UIEventType::Text_TextChanged,
                &self.message_list,
                TreeScope::Subtree,
                None,
                &handler,
            )?;
            self.handler = Some(handler);
            Ok(())
        }

        pub fn latest_message_text(&self) -> Option<String> {
            let items = self
                .automation
                .create_matcher()
                .from_ref(&self.message_list)
                .control_type(ControlType::ListItem)
                .depth(8)
                .timeout(0)
                .find_all()
                .unwrap_or_default();
            items
                .into_iter()
                .filter_map(|item| item.get_name().ok())
                .map(|name| name.trim().to_string())
                .filter(|name| !name.is_empty())
                .last()
        }
    }

    fn find_message_list(automation: &UIAutomation, window: &UIElement) -> Result<UIElement> {
        let list_types = [
            ControlType::List,
            ControlType::DataGrid,
            ControlType::Table,
            ControlType::Tree,
        ];
        let window_rect = window.get_bounding_rectangle()?;
        let mid_x = window_rect.get_left() + (window_rect.get_width() / 2);
        let mut best: Option<UIElement> = None;
        for control_type in list_types {
            let candidates = automation
                .create_matcher()
                .from_ref(window)
                .control_type(control_type)
                .depth(12)
                .timeout(0)
                .find_all()
                .unwrap_or_default();
            for candidate in candidates {
                if let Ok(rect) = candidate.get_bounding_rectangle() {
                    if rect.get_left() < mid_x {
                        continue;
                    }
                }
                best = Some(candidate);
                break;
            }
            if best.is_some() {
                break;
            }
        }
        if let Some(list) = best {
            return Ok(list);
        }
        automation
            .create_matcher()
            .from_ref(window)
            .filter_fn(Box::new(|element| {
                let name = element.get_name().unwrap_or_default();
                Ok(MESSAGE_LIST_NAMES.iter().any(|label| label == &name))
            }))
            .depth(12)
            .timeout(0)
            .find_first()
            .map_err(|_| anyhow!("Message list not found"))
    }
}
