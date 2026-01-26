use super::{find_wechat_hwnd, MockUia};
use super::{collect_recent_chats, MockInputWriter, MockSessionList, MockWatcher, WatchMode};

#[test]
fn uia_finds_wechat_main_window_by_process_name() {
    let mock = MockUia::with_window("Weixin.exe", "WeChat");
    let hwnd = find_wechat_hwnd(&mock).unwrap();
    assert_eq!(hwnd, 1001);
}

#[test]
fn session_list_scrolls_and_dedupes() {
    let mut mock = MockSessionList::with_pages(vec![
        vec!["A", "B"],
        vec!["C", "B"],
    ]);
    let chats = collect_recent_chats(&mut mock).unwrap();
    assert_eq!(chats.len(), 3);
}

#[test]
fn watcher_falls_back_to_polling_on_subscribe_failure() {
    let mock = MockWatcher::subscribe_fail();
    let mode = mock.start();
    assert_eq!(mode, WatchMode::Polling);
}

#[test]
fn input_writer_uses_clipboard_on_uia_failure() {
    let mut mock = MockInputWriter::uia_fail();
    let ok = mock.write("chat", "hello");
    assert!(ok);
    assert!(mock.used_clipboard());
}
