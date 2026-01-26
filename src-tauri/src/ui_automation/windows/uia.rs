use super::element::WindowInfo;

pub trait UiaProvider {
    fn list_windows(&self) -> Vec<WindowInfo>;
}

#[derive(Default)]
pub struct MockUia {
    windows: Vec<WindowInfo>,
}

impl MockUia {
    pub fn with_window(process_name: &str, title: &str) -> Self {
        Self {
            windows: vec![WindowInfo::new(1001, process_name, title)],
        }
    }

    pub fn add_window(&mut self, hwnd: i64, process_name: &str, title: &str) {
        self.windows
            .push(WindowInfo::new(hwnd, process_name, title));
    }
}

impl UiaProvider for MockUia {
    fn list_windows(&self) -> Vec<WindowInfo> {
        self.windows.clone()
    }
}

pub fn find_wechat_hwnd(provider: &dyn UiaProvider) -> Option<i64> {
    provider
        .list_windows()
        .into_iter()
        .find(|window| is_wechat_process(&window.process_name))
        .map(|window| window.hwnd)
}

fn is_wechat_process(name: &str) -> bool {
    let normalized = name.trim().to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "weixin.exe" | "wechat.exe" | "wechatappex.exe"
    )
}
