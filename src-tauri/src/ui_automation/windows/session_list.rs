use crate::types::{ChatKind, ChatSummary};
use anyhow::Result;
use std::collections::HashSet;

pub trait SessionListProvider {
    fn session_titles(&self) -> Vec<String>;
}

#[derive(Default)]
pub struct MockSessionList {
    sessions: Vec<String>,
}

impl MockSessionList {
    pub fn with_sessions(sessions: Vec<&str>) -> Self {
        Self {
            sessions: sessions.into_iter().map(|item| item.to_string()).collect(),
        }
    }
}

impl SessionListProvider for MockSessionList {
    fn session_titles(&self) -> Vec<String> {
        self.sessions.clone()
    }
}

pub fn collect_recent_chats(provider: &dyn SessionListProvider) -> Result<Vec<ChatSummary>> {
    let mut seen = HashSet::new();
    let mut chats = Vec::new();
    for title in provider.session_titles() {
        let title = title.trim().to_string();
        if title.is_empty() {
            continue;
        }
        if !seen.insert(title.clone()) {
            continue;
        }
        chats.push(ChatSummary {
            chat_id: title.clone(),
            chat_title: title,
            kind: ChatKind::Unknown,
        });
    }
    Ok(chats)
}
