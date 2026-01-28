use super::{AutomationManager, WeChatAutomation};
use crate::types::ChatSummary;
use crate::ui_automation::IncomingMessage;
use std::sync::Arc;
use std::time::Duration;

struct MockAutomation;

impl WeChatAutomation for MockAutomation {
    fn platform(&self) -> super::Platform {
        super::Platform::Unknown
    }

    fn list_recent_chats(&self) -> anyhow::Result<Vec<ChatSummary>> {
        Ok(vec![ChatSummary {
            chat_id: "c1".to_string(),
            chat_title: "Chat 1".to_string(),
            kind: crate::types::ChatKind::Direct,
        }])
    }

    fn start_listening(&self, _targets: Vec<super::ListenTarget>) -> anyhow::Result<()> {
        Ok(())
    }

    fn stop_listening(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn write_input(&self, _chat_id: &str, _text: &str) -> anyhow::Result<()> {
        Ok(())
    }

    fn poll_latest_message(&self) -> anyhow::Result<Option<IncomingMessage>> {
        Ok(None)
    }
}

struct SlowAutomation {
    delay: Duration,
}

impl WeChatAutomation for SlowAutomation {
    fn platform(&self) -> super::Platform {
        super::Platform::Unknown
    }

    fn list_recent_chats(&self) -> anyhow::Result<Vec<ChatSummary>> {
        Ok(Vec::new())
    }

    fn start_listening(&self, _targets: Vec<super::ListenTarget>) -> anyhow::Result<()> {
        std::thread::sleep(self.delay);
        Ok(())
    }

    fn stop_listening(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn write_input(&self, _chat_id: &str, _text: &str) -> anyhow::Result<()> {
        Ok(())
    }

    fn poll_latest_message(&self) -> anyhow::Result<Option<IncomingMessage>> {
        Ok(None)
    }
}

#[tokio::test]
async fn automation_manager_rejects_when_not_ready() {
    let mgr = AutomationManager::new(None);
    let res = mgr.list_recent_chats().await;
    assert!(!res.success);
}

#[tokio::test]
async fn automation_manager_accepts_when_ready() {
    let mgr = AutomationManager::new(Some(Arc::new(MockAutomation)));
    let res = mgr.list_recent_chats().await;
    assert!(res.success);
    let chats = res.data.unwrap_or_default();
    assert_eq!(chats.len(), 1);
}

#[tokio::test]
async fn automation_manager_times_out_on_slow_start() {
    std::env::set_var("WEREPLY_AUTOMATION_START_TIMEOUT_MS", "20");
    let mgr = AutomationManager::new(Some(Arc::new(SlowAutomation {
        delay: Duration::from_millis(80),
    })));
    let res = mgr.start_listening(Vec::new()).await;
    assert!(!res.success);
    assert!(res.message.contains("超时"));
    std::env::remove_var("WEREPLY_AUTOMATION_START_TIMEOUT_MS");
}
