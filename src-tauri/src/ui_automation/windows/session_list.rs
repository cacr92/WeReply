use crate::types::{ChatKind, ChatSummary};
use anyhow::{anyhow, Result};
use std::collections::HashSet;
use std::thread::sleep;
use std::time::Duration;

pub trait SessionListProvider {
    fn snapshot(&self) -> Vec<String>;
    fn scroll_down(&mut self) -> bool;
}

#[derive(Default)]
pub struct MockSessionList {
    pages: Vec<Vec<String>>,
    index: usize,
}

impl MockSessionList {
    pub fn with_sessions(sessions: Vec<&str>) -> Self {
        Self {
            pages: vec![sessions.into_iter().map(|item| item.to_string()).collect()],
            index: 0,
        }
    }

    pub fn with_pages(pages: Vec<Vec<&str>>) -> Self {
        Self {
            pages: pages
                .into_iter()
                .map(|page| page.into_iter().map(|item| item.to_string()).collect())
                .collect(),
            index: 0,
        }
    }
}

impl SessionListProvider for MockSessionList {
    fn snapshot(&self) -> Vec<String> {
        self.pages
            .get(self.index)
            .cloned()
            .unwrap_or_default()
    }

    fn scroll_down(&mut self) -> bool {
        if self.index + 1 >= self.pages.len() {
            return false;
        }
        self.index += 1;
        true
    }
}

pub fn collect_recent_chats(provider: &mut dyn SessionListProvider) -> Result<Vec<ChatSummary>> {
    let mut seen = HashSet::new();
    let mut chats = Vec::new();
    let mut stagnant_rounds = 0;
    for _ in 0..64 {
        let mut new_count = 0;
        for title in provider.snapshot() {
            let title = title.trim().to_string();
            if title.is_empty() {
                continue;
            }
            if !seen.insert(title.clone()) {
                continue;
            }
            new_count += 1;
            chats.push(ChatSummary {
                chat_id: title.clone(),
                chat_title: title,
                kind: ChatKind::Unknown,
            });
        }
        if new_count == 0 {
            stagnant_rounds += 1;
        } else {
            stagnant_rounds = 0;
        }
        if stagnant_rounds >= 2 {
            break;
        }
        if !provider.scroll_down() {
            break;
        }
        sleep(Duration::from_millis(80));
    }
    if chats.is_empty() {
        return Err(anyhow!("Session list empty"));
    }
    Ok(chats)
}

#[cfg(target_os = "windows")]
pub mod uia {
    use super::SessionListProvider;
    use anyhow::{anyhow, Result};
    use uiautomation::patterns::{UISelectionItemPattern, UIScrollPattern};
    use uiautomation::types::{ControlType, ScrollAmount};
    use uiautomation::{UIAutomation, UIElement};
    use uiautomation::inputs::Keyboard;

    const SESSION_LIST_NAMES: [&str; 4] = [
        "\u4f1a\u8bdd",
        "\u804a\u5929",
        "\u804a\u5929\u8bb0\u5f55",
        "\u6298\u53e0\u7684\u7fa4\u804a",
    ];

    pub struct UiaSessionList {
        automation: UIAutomation,
        list: UIElement,
        scroll: Option<UIScrollPattern>,
    }

    impl UiaSessionList {
        pub fn from_window(automation: &UIAutomation, window: &UIElement) -> Result<Self> {
            let list = find_session_list(automation, window)?;
            let scroll = list.get_pattern::<UIScrollPattern>().ok();
            Ok(Self {
                automation: automation.clone(),
                list,
                scroll,
            })
        }

        fn list_item_names(&self) -> Vec<String> {
            let mut names = Vec::new();
            let items = self
                .automation
                .create_matcher()
                .from_ref(&self.list)
                .control_type(ControlType::ListItem)
                .depth(6)
                .timeout(0)
                .find_all()
                .or_else(|_| {
                    self.automation
                        .create_matcher()
                        .from_ref(&self.list)
                        .control_type(ControlType::DataItem)
                        .depth(6)
                        .timeout(0)
                        .find_all()
                })
                .unwrap_or_default();
            for item in items {
                if let Some(name) = extract_item_title(&self.automation, &item) {
                    names.push(name);
                }
            }
            names
        }

        pub fn active_title(&self) -> Option<String> {
            let items = self
                .automation
                .create_matcher()
                .from_ref(&self.list)
                .control_type(ControlType::ListItem)
                .depth(6)
                .timeout(0)
                .find_all()
                .unwrap_or_default();
            for item in items {
                if let Ok(selection) = item.get_pattern::<UISelectionItemPattern>() {
                    if selection.is_selected().unwrap_or(false) {
                        return extract_item_title(&self.automation, &item);
                    }
                }
            }
            None
        }
    }

    impl SessionListProvider for UiaSessionList {
        fn snapshot(&self) -> Vec<String> {
            self.list_item_names()
        }

        fn scroll_down(&mut self) -> bool {
            if let Some(pattern) = &self.scroll {
                if pattern
                    .scroll(ScrollAmount::NoAmount, ScrollAmount::LargeIncrement)
                    .is_ok()
                {
                    return true;
                }
            }
            if self.list.set_focus().is_ok() {
                let keyboard = Keyboard::default();
                return keyboard.send_keys("{PGDN}").is_ok();
            }
            false
        }
    }

    pub fn find_session_list(automation: &UIAutomation, window: &UIElement) -> Result<UIElement> {
        let list_types = [
            ControlType::List,
            ControlType::DataGrid,
            ControlType::Table,
            ControlType::Tree,
        ];
        let window_rect = window.get_bounding_rectangle()?;
        let mid_x = window_rect.get_left() + (window_rect.get_width() * 6 / 10);
        let mut best: Option<(UIElement, usize)> = None;
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
                    if rect.get_right() > mid_x {
                        continue;
                    }
                }
                let count = count_list_items(automation, &candidate);
                if count < 3 {
                    continue;
                }
                let score = count;
                match best {
                    Some((_, best_score)) if best_score >= score => {}
                    _ => best = Some((candidate, score)),
                }
            }
        }
        if let Some((element, _)) = best {
            return Ok(element);
        }
        let named = automation
            .create_matcher()
            .from_ref(window)
            .filter_fn(Box::new(|element| {
                let name = element.get_name().unwrap_or_default();
                Ok(SESSION_LIST_NAMES.iter().any(|label| label == &name))
            }))
            .depth(12)
            .timeout(0)
            .find_first();
        named.map_err(|_| anyhow!("Failed to locate session list"))
    }

    fn count_list_items(automation: &UIAutomation, list: &UIElement) -> usize {
        let list_items = automation
            .create_matcher()
            .from_ref(list)
            .control_type(ControlType::ListItem)
            .depth(6)
            .timeout(0)
            .find_all()
            .map(|items| items.len())
            .unwrap_or(0)
            ;
        let data_items = automation
            .create_matcher()
            .from_ref(list)
            .control_type(ControlType::DataItem)
            .depth(6)
            .timeout(0)
            .find_all()
            .map(|items| items.len())
            .unwrap_or(0);
        list_items.max(data_items)
    }

    fn extract_item_title(automation: &UIAutomation, item: &UIElement) -> Option<String> {
        if let Ok(name) = item.get_name() {
            let trimmed = name.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
        let text = automation
            .create_matcher()
            .from_ref(item)
            .control_type(ControlType::Text)
            .depth(4)
            .timeout(0)
            .find_first()
            .ok()?;
        let name = text.get_name().ok()?;
        let trimmed = name.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }
}
