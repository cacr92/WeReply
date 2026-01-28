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
    use super::{pick_row_text, score_message_list};
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
                let texts = ax::collect_static_texts(&row, 8);
                if let Some(text) = pick_row_text(&texts) {
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
        let candidates = ax::find_lists_with_titles(window, 8);
        if let Some(best) = select_message_list(window, candidates) {
            return Ok(best.0);
        }
        Err(anyhow!("Message list not found"))
    }

    fn select_message_list(
        window: &AxElement,
        candidates: Vec<(AxElement, Vec<String>)>,
    ) -> Option<(AxElement, Vec<String>)> {
        let window_frame = ax::frame(window);
        let mut scored = candidates
            .into_iter()
            .map(|(list, titles)| {
                let frame = ax::frame(&list);
                let score = if let (Some(window_frame), Some(frame)) = (window_frame, frame) {
                    score_message_list(window_frame, frame.center_x(), frame.width, titles.len())
                } else {
                    titles.len() as i64
                };
                (score, list, titles)
            })
            .collect::<Vec<_>>();
        scored.sort_by_key(|(score, _, _)| *score);
        scored.pop().map(|(_, list, titles)| (list, titles))
    }
}

fn pick_row_text(texts: &[String]) -> Option<String> {
    texts
        .iter()
        .map(|item| item.trim())
        .filter(|item| !item.is_empty())
        .max_by_key(|item| item.chars().count())
        .map(|item| item.to_string())
}

fn score_message_list(
    window: crate::ui_automation::macos::ax::AxRect,
    center_x: f64,
    width: f64,
    title_count: usize,
) -> i64 {
    let mut score = title_count as i64;
    if center_x >= window.center_x() {
        score += 10_000;
    } else {
        score -= 10_000;
    }
    if width >= window.width * 0.45 {
        score += 500;
    }
    score
}

#[cfg(test)]
mod tests {
    use super::{pick_row_text, score_message_list};
    use crate::ui_automation::macos::ax::AxRect;

    #[test]
    fn picks_longest_text_from_row() {
        let texts = vec![
            "09:11".to_string(),
            "Alice".to_string(),
            "See you tonight?".to_string(),
        ];
        assert_eq!(pick_row_text(&texts), Some("See you tonight?".to_string()));
    }

    #[test]
    fn message_list_prefers_right_side() {
        let window = AxRect {
            x: 0.0,
            y: 0.0,
            width: 1000.0,
            height: 800.0,
        };
        let left_score = score_message_list(
            window,
            200.0,
            240.0,
            20,
        );
        let right_score = score_message_list(
            window,
            720.0,
            560.0,
            8,
        );
        assert!(right_score > left_score);
    }
}
