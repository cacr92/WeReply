use super::{
    collect_recent_chats, find_wechat_app, MockAx, MockAxSessionList, MockAxWatcher, WatchMode,
};

#[test]
fn ax_finds_wechat_app() {
    let mock = MockAx::with_bundle("com.tencent.xinWeChat");
    let app = find_wechat_app(&mock);
    assert_eq!(app, Some("com.tencent.xinWeChat".to_string()));
}

#[test]
fn macos_session_list_dedupes() {
    let mock = MockAxSessionList::with_sessions(vec!["A", "A", "B"]);
    let chats = collect_recent_chats(&mock).unwrap();
    assert_eq!(chats.len(), 2);
}

#[test]
fn macos_watcher_falls_back_to_polling_on_subscribe_failure() {
    let mock = MockAxWatcher::subscribe_fail();
    let mode = mock.start();
    assert_eq!(mode, WatchMode::Polling);
}
