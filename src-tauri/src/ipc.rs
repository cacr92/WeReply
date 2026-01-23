use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const MAX_RAW_MESSAGE_LEN: usize = 100_000;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IpcEnvelope {
    pub version: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub id: String,
    pub timestamp: u64,
    pub payload: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentReadyPayload {
    pub platform: String,
    pub agent_version: String,
    pub capabilities: Vec<String>,
    pub supports_clipboard_restore: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentStatusPayload {
    pub state: String,
    #[serde(default)]
    pub detail: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentErrorPayload {
    pub code: String,
    pub message: String,
    pub recoverable: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageNewPayload {
    pub chat_id: String,
    pub chat_title: String,
    pub is_group: bool,
    pub sender_name: String,
    pub text: String,
    pub timestamp: u64,
    #[serde(default)]
    pub msg_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputWritePayload {
    pub chat_id: String,
    pub text: String,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub restore_clipboard: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputResultPayload {
    pub ok: bool,
    #[serde(default)]
    pub error: String,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventAckPayload {
    pub ack_id: String,
    pub ok: bool,
    #[serde(default)]
    pub error: String,
}

impl IpcEnvelope {
    pub fn new(message_type: &str, payload: Value) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            version: "1.0".to_string(),
            r#type: message_type.to_string(),
            id: Uuid::new_v4().to_string(),
            timestamp,
            payload,
        }
    }

    pub fn ack_for(message_id: &str, ok: bool, error: impl Into<String>) -> Self {
        let payload = serde_json::json!({
            "ack_id": message_id,
            "ok": ok,
            "error": error.into()
        });
        Self::new("event.ack", payload)
    }
}

pub fn parse_envelope(line: &str) -> Result<IpcEnvelope> {
    if line.len() > MAX_RAW_MESSAGE_LEN {
        anyhow::bail!("Agent 消息过大");
    }
    let envelope: IpcEnvelope =
        serde_json::from_str(line).context("Agent 消息格式错误")?;
    validate_envelope(&envelope)?;
    Ok(envelope)
}

fn validate_envelope(envelope: &IpcEnvelope) -> Result<()> {
    if envelope.version != "1.0" {
        anyhow::bail!("IPC 协议版本不匹配");
    }
    if envelope.id.trim().is_empty() || envelope.r#type.trim().is_empty() {
        anyhow::bail!("IPC 消息缺少必要字段");
    }
    Ok(())
}

pub fn validate_message_new(payload: &MessageNewPayload) -> Result<()> {
    if payload.chat_id.trim().is_empty() {
        anyhow::bail!("chat_id 不能为空");
    }
    if payload.text.trim().is_empty() {
        anyhow::bail!("消息内容为空");
    }
    if payload.text.len() > 10_000 {
        anyhow::bail!("消息内容过长");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_message_new() {
        let msg = IpcEnvelope::new(
            "message.new",
            serde_json::json!({"chat_id": "c1", "text": "hi"})
        );
        let line = serde_json::to_string(&msg).unwrap();
        assert!(line.contains("\"type\":\"message.new\""));
    }

    #[test]
    fn reject_empty_message() {
        let payload = MessageNewPayload {
            chat_id: "c1".to_string(),
            chat_title: "t".to_string(),
            is_group: false,
            sender_name: "s".to_string(),
            text: "".to_string(),
            timestamp: 1,
            msg_id: None,
        };
        assert!(validate_message_new(&payload).is_err());
    }
}
