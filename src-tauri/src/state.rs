use crate::agent::AgentHandle;
use crate::listen_targets::{normalize_listen_targets, MAX_LISTEN_TARGETS};
use crate::types::{ChatSummary, Config, ListenTarget, Status};
use crate::ui_automation::AutomationManager;
use std::collections::HashMap;
use tokio::sync::oneshot;

#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub text: String,
    pub timestamp: u64,
    pub msg_id: Option<String>,
}

pub struct AppState {
    pub config: Config,
    pub status: Status,
    pub agent: Option<AgentHandle>,
    pub automation: AutomationManager,
    pub listen_targets: Vec<ListenTarget>,
    pub recent_chats: Vec<ChatSummary>,
    pub pending_chats_list: Option<(String, oneshot::Sender<Vec<ChatSummary>>)>,
    conversations: HashMap<String, Vec<ChatMessage>>,
    last_message_keys: HashMap<String, String>,
}

impl AppState {
    pub fn new(mut config: Config, status: Status) -> Self {
        let listen_targets = normalize_listen_targets(
            config.listen_targets.clone(),
            MAX_LISTEN_TARGETS,
        )
        .unwrap_or_default();
        config.listen_targets = listen_targets.clone();
        Self {
            config,
            status,
            agent: None,
            automation: AutomationManager::new(None),
            listen_targets,
            recent_chats: Vec::new(),
            pending_chats_list: None,
            conversations: HashMap::new(),
            last_message_keys: HashMap::new(),
        }
    }

    pub fn is_duplicate(
        &self,
        chat_id: &str,
        msg_id: &Option<String>,
        text: &str,
        timestamp: u64,
    ) -> bool {
        let key = dedupe_key(msg_id, text, timestamp);
        self.last_message_keys
            .get(chat_id)
            .map(|last| last == &key)
            .unwrap_or(false)
    }

    pub fn record_message(&mut self, chat_id: &str, message: ChatMessage) {
        let key = dedupe_key(&message.msg_id, &message.text, message.timestamp);
        self.last_message_keys.insert(chat_id.to_string(), key);

        let messages = self.conversations.entry(chat_id.to_string()).or_default();
        messages.push(message);
        trim_messages(messages, &self.config);
    }

    pub fn context_for_chat(&self, chat_id: &str) -> Vec<String> {
        self.conversations
            .get(chat_id)
            .map(|messages| messages.iter().map(|m| m.text.clone()).collect())
            .unwrap_or_default()
    }
}

fn dedupe_key(msg_id: &Option<String>, text: &str, timestamp: u64) -> String {
    msg_id
        .as_ref()
        .cloned()
        .unwrap_or_else(|| format!("{}:{}", text, timestamp))
}

fn trim_messages(messages: &mut Vec<ChatMessage>, config: &Config) {
    let max_messages = config.context_max_messages as usize;
    while messages.len() > max_messages {
        messages.remove(0);
    }

    let max_chars = config.context_max_chars as usize;
    let mut total_chars = 0;
    let mut keep_start = messages.len();
    for (index, message) in messages.iter().enumerate().rev() {
        total_chars += message.text.chars().count();
        if total_chars > max_chars {
            keep_start = index + 1;
            break;
        }
    }
    if keep_start > 0 && keep_start < messages.len() {
        messages.drain(0..keep_start);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RuntimeState;
    use crate::types::{Platform, Status};

    #[test]
    fn trims_by_message_count() {
        let config = Config {
            context_max_messages: 2,
            ..Config::default()
        };
        let status = Status {
            state: RuntimeState::Idle,
            platform: Platform::Unknown,
            agent_connected: false,
            last_error: String::new(),
        };
        let mut state = AppState::new(config, status);
        for i in 0..3 {
            state.record_message(
                "c1",
                ChatMessage {
                    text: format!("msg{}", i),
                    timestamp: i,
                    msg_id: None,
                },
            );
        }
        let context = state.context_for_chat("c1");
        assert_eq!(context.len(), 2);
        assert_eq!(context[0], "msg1");
    }
}
