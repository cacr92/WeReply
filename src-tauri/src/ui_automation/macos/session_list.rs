use crate::types::{ChatKind, ChatSummary};
use anyhow::{anyhow, Result};
use std::collections::HashSet;
use std::thread::sleep;
use std::time::Duration;

#[cfg(any(test, target_os = "macos"))]
pub trait AxSessionListProvider {
    fn snapshot(&self) -> Vec<String>;
    fn scroll_down(&mut self) -> bool;
}

#[cfg(test)]
#[derive(Default)]
pub struct MockAxSessionList {
    pages: Vec<Vec<String>>,
    index: usize,
}

#[cfg(test)]
impl MockAxSessionList {
    #[allow(dead_code)]
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

#[cfg(test)]
impl AxSessionListProvider for MockAxSessionList {
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

#[cfg(any(test, target_os = "macos"))]
pub fn collect_recent_chats(provider: &mut dyn AxSessionListProvider) -> Result<Vec<ChatSummary>> {
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

#[cfg(target_os = "macos")]
pub mod ax {
    use super::AxSessionListProvider;
    use crate::ui_automation::macos::ax::{self, AxElement};
    use crate::ui_automation::macos::static_ui_paths;
    use anyhow::{anyhow, Result};

    pub struct AxSessionList {
        list: AxElement,
    }

    impl AxSessionList {
        pub fn from_window(window: &AxElement) -> Result<Self> {
            let list = find_session_list(window)?;
            Ok(Self { list })
        }
    }

    impl AxSessionListProvider for AxSessionList {
        fn snapshot(&self) -> Vec<String> {
            ax::collect_session_titles(&self.list)
        }

        fn scroll_down(&mut self) -> bool {
            ax::focus_element(&self.list).is_ok() && ax::send_page_down().is_ok()
        }
    }

    fn find_session_list(window: &AxElement) -> Result<AxElement> {
        if let Some(list) = ax::resolve_any_path(window, static_ui_paths::SESSION_LIST_PATHS) {
            return Ok(list);
        }
        if static_ui_paths::allow_dynamic_scan() {
            let candidates = ax::find_lists_with_titles(window, 8);
            if let Some(best) = select_session_list(window, candidates) {
                return Ok(best.0);
            }
        }
        Err(anyhow!(
            "Session list not found (static UI path)."
        ))
    }

    fn select_session_list(
        window: &AxElement,
        candidates: Vec<(AxElement, Vec<String>)>,
    ) -> Option<(AxElement, Vec<String>)> {
        let window_frame = ax::frame(window);
        let mut scored = candidates
            .into_iter()
            .map(|(list, titles)| {
                let frame = ax::frame(&list);
                let score = if let (Some(window_frame), Some(frame)) = (window_frame, frame) {
                    score_session_list(window_frame, frame.center_x(), frame.width, titles.len())
                } else {
                    titles.len() as i64
                };
                (score, list, titles)
            })
            .collect::<Vec<_>>();
        scored.sort_by_key(|(score, _, _)| *score);
        scored.pop().map(|(_, list, titles)| (list, titles))
    }

    pub(super) fn score_session_list(
        window: crate::ui_automation::macos::ax::AxRect,
        center_x: f64,
        width: f64,
        title_count: usize,
    ) -> i64 {
        let mut score = title_count as i64;
        if center_x < window.center_x() {
            score += 10_000;
        } else {
            score -= 10_000;
        }
        if width <= window.width * 0.45 {
            score += 500;
        }
        if width <= window.width * 0.35 {
            score += 200;
        }
        score
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::ax::score_session_list;
    use crate::ui_automation::macos::ax::AxRect;

    #[test]
    fn session_list_prefers_left_side() {
        let window = AxRect {
            x: 0.0,
            y: 0.0,
            width: 1000.0,
            height: 800.0,
        };
        let left_score = score_session_list(window, 200.0, 300.0, 6);
        let right_score = score_session_list(window, 720.0, 500.0, 12);
        assert!(left_score > right_score);
    }
}
