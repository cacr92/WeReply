pub use crate::types::{ChatSummary, ListenTarget, Platform};

#[derive(Clone, Debug)]
pub struct IncomingMessage {
    pub chat_id: String,
    pub text: String,
    pub timestamp: u64,
    pub msg_id: Option<String>,
}
