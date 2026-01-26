use super::{find_wechat_app, MockAx};

#[test]
fn ax_finds_wechat_app() {
    let mock = MockAx::with_bundle("com.tencent.xinWeChat");
    let app = find_wechat_app(&mock);
    assert_eq!(app, Some("com.tencent.xinWeChat".to_string()));
}
