use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IpcMessage {
    pub version: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub id: String,
    pub timestamp: u64,
    pub payload: Value,
}

impl IpcMessage {
    pub fn new(message_type: &str, payload: Value) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            version: "1.0".to_string(),
            r#type: message_type.to_string(),
            id: format!("msg-{}", timestamp),
            timestamp,
            payload,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_message_new() {
        let msg = IpcMessage::new(
            "message.new",
            serde_json::json!({"chat_id": "c1", "text": "hi"})
        );
        let line = serde_json::to_string(&msg).unwrap();
        assert!(line.contains("\"type\":\"message.new\""));
    }
}
