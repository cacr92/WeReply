use super::{find_wechat_hwnd, MockUia};

#[test]
fn uia_finds_wechat_main_window_by_process_name() {
    let mock = MockUia::with_window("Weixin.exe", "WeChat");
    let hwnd = find_wechat_hwnd(&mock).unwrap();
    assert_eq!(hwnd, 1001);
}
