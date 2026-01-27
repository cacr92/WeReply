#[cfg(test)]
use super::element::WindowInfo;

#[cfg(test)]
pub trait UiaProvider {
    fn list_windows(&self) -> Vec<WindowInfo>;
}

#[cfg(test)]
#[derive(Default)]
pub struct MockUia {
    windows: Vec<WindowInfo>,
}

#[cfg(test)]
impl MockUia {
    pub fn with_window(process_name: &str, title: &str) -> Self {
        Self {
            windows: vec![WindowInfo::new(1001, process_name, title)],
        }
    }

    #[allow(dead_code)]
    pub fn add_window(&mut self, hwnd: i64, process_name: &str, title: &str) {
        self.windows
            .push(WindowInfo::new(hwnd, process_name, title));
    }
}

#[cfg(test)]
impl UiaProvider for MockUia {
    fn list_windows(&self) -> Vec<WindowInfo> {
        self.windows.clone()
    }
}

#[cfg(test)]
pub fn find_wechat_hwnd(provider: &dyn UiaProvider) -> Option<i64> {
    provider
        .list_windows()
        .into_iter()
        .find(|window| is_wechat_process(&window.process_name))
        .map(|window| window.hwnd)
}

#[cfg(test)]
fn is_wechat_process(name: &str) -> bool {
    let normalized = name.trim().to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "weixin.exe" | "wechat.exe" | "wechatappex.exe"
    )
}

#[cfg(target_os = "windows")]
pub mod uia {
    use anyhow::{anyhow, Result};
    use uiautomation::types::ControlType;
    use uiautomation::{UIAutomation, UIElement};

    const WECHAT_MAIN_CLASS: &str = "WeChatMainWndForPC";

    pub struct UiaClient {
        automation: UIAutomation,
    }

    impl UiaClient {
        pub fn new() -> Result<Self> {
            Ok(Self {
                automation: UIAutomation::new()?,
            })
        }

        pub fn automation(&self) -> &UIAutomation {
            &self.automation
        }

        pub fn find_wechat_windows(&self) -> Result<Vec<UIElement>> {
            let by_class = self
                .automation
                .create_matcher()
                .classname(WECHAT_MAIN_CLASS)
                .control_type(ControlType::Window)
                .depth(4)
                .timeout(0)
                .find_all();
            if let Ok(windows) = by_class {
                if !windows.is_empty() {
                    return Ok(windows);
                }
            }
            let all_windows = self
                .automation
                .create_matcher()
                .control_type(ControlType::Window)
                .depth(3)
                .timeout(0)
                .find_all()
                .map_err(|_| anyhow!("No windows found"))?;
            Ok(all_windows)
        }

        pub fn pick_wechat_window(&self) -> Result<UIElement> {
            let mut windows = self.find_wechat_windows()?;
            if let Some(found) = windows
                .iter()
                .find(|window| window.get_classname().map(|name| name == WECHAT_MAIN_CLASS).unwrap_or(false))
                .cloned()
            {
                return Ok(found);
            }
            windows
                .drain(..)
                .max_by_key(|window| window.get_name().map(|name| name.len()).unwrap_or(0))
                .ok_or_else(|| anyhow!("WeChat window not found"))
        }
    }
}
