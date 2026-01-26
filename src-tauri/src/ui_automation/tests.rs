use super::AutomationManager;

#[tokio::test]
async fn automation_manager_rejects_when_not_ready() {
    let mgr = AutomationManager::new(None);
    let res = mgr.list_recent_chats().await;
    assert!(!res.success);
}
