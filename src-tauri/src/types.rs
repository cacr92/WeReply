use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeState {
    Idle,
    Listening,
    Generating,
    Paused,
    Error,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Windows,
    Macos,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChatKind {
    Direct,
    Group,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq, Eq)]
#[specta(inline)]
pub struct ListenTarget {
    pub name: String,
    pub kind: ChatKind,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq, Eq)]
#[specta(inline)]
pub struct ChatSummary {
    pub chat_id: String,
    pub chat_title: String,
    pub kind: ChatKind,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SuggestionStyle {
    Formal,
    Neutral,
    Casual,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
#[specta(inline)]
pub struct Suggestion {
    pub id: String,
    pub style: SuggestionStyle,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
#[specta(inline)]
pub struct Status {
    pub state: RuntimeState,
    pub platform: Platform,
    pub agent_connected: bool,
    pub last_error: String,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
#[specta(inline)]
pub struct Config {
    pub deepseek_model: String,
    pub suggestion_count: u32,
    pub context_max_messages: u32,
    pub context_max_chars: u32,
    pub poll_interval_ms: u64,
    pub temperature: f32,
    pub top_p: f32,
    pub base_url: String,
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub log_level: String,
    pub log_to_file: bool,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
#[specta(inline)]
pub struct SuggestionsUpdated {
    pub chat_id: String,
    pub suggestions: Vec<Suggestion>,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
#[specta(inline)]
pub struct ErrorPayload {
    pub code: String,
    pub message: String,
    pub recoverable: bool,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
#[specta(inline)]
pub struct DeepseekEndpointStatus {
    pub ok: bool,
    pub status: Option<u16>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
#[specta(inline)]
pub struct DeepseekDiagnostics {
    pub base_url: String,
    pub model: String,
    pub chat: DeepseekEndpointStatus,
    pub models: DeepseekEndpointStatus,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
#[specta(inline)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

pub fn api_ok<T>(data: T) -> ApiResponse<T> {
    ApiResponse {
        success: true,
        message: String::new(),
        data: Some(data),
    }
}

pub fn api_err<T>(message: impl Into<String>) -> ApiResponse<T> {
    ApiResponse {
        success: false,
        message: message.into(),
        data: None,
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            deepseek_model: "deepseek-chat".to_string(),
            suggestion_count: 3,
            context_max_messages: 10,
            context_max_chars: 2000,
            poll_interval_ms: 800,
            temperature: 0.7,
            top_p: 1.0,
            base_url: "https://api.deepseek.com".to_string(),
            timeout_ms: 12_000,
            max_retries: 2,
            log_level: "info".to_string(),
            log_to_file: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = Config::default();
        assert_eq!(cfg.deepseek_model, "deepseek-chat");
        assert_eq!(cfg.suggestion_count, 3);
        assert_eq!(cfg.context_max_messages, 10);
        assert_eq!(cfg.context_max_chars, 2000);
        assert_eq!(cfg.poll_interval_ms, 800);
        assert_eq!(cfg.temperature, 0.7);
        assert_eq!(cfg.top_p, 1.0);
        assert_eq!(cfg.base_url, "https://api.deepseek.com");
        assert_eq!(cfg.timeout_ms, 12_000);
        assert_eq!(cfg.max_retries, 2);
        assert_eq!(cfg.log_level, "info");
        assert!(!cfg.log_to_file);
    }
}
