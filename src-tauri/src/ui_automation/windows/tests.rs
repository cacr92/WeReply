use super::{find_wechat_hwnd, MockUia};
use super::{collect_recent_chats, MockSessionList, MockWatcher, WatchMode};

#[test]
fn uia_finds_wechat_main_window_by_process_name() {
    let mock = MockUia::with_window("Weixin.exe", "WeChat");
    let hwnd = find_wechat_hwnd(&mock).unwrap();
    assert_eq!(hwnd, 1001);
}

#[test]
fn session_list_scrolls_and_dedupes() {
    let mock = MockSessionList::with_sessions(vec!["A", "B", "C", "B"]);
    let chats = collect_recent_chats(&mock).unwrap();
    assert_eq!(chats.len(), 3);
}

#[test]
fn watcher_falls_back_to_polling_on_subscribe_failure() {
    let mock = MockWatcher::subscribe_fail();
    let mode = mock.start();
    assert_eq!(mode, WatchMode::Polling);
}
