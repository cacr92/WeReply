#[cfg(test)]
#[allow(dead_code)]
pub struct MockAxInputWriter {
    ax_ok: bool,
    used_clipboard: bool,
}

#[cfg(test)]
impl MockAxInputWriter {
    #[allow(dead_code)]
    pub fn ax_fail() -> Self {
        Self {
            ax_ok: false,
            used_clipboard: false,
        }
    }

    #[allow(dead_code)]
    pub fn ax_ok() -> Self {
        Self {
            ax_ok: true,
            used_clipboard: false,
        }
    }

    #[allow(dead_code)]
    pub fn write(&mut self, _chat_id: &str, _text: &str) -> bool {
        if self.ax_ok {
            return true;
        }
        self.used_clipboard = true;
        true
    }

    #[allow(dead_code)]
    pub fn used_clipboard(&self) -> bool {
        self.used_clipboard
    }
}

#[cfg(target_os = "macos")]
pub mod ax {
    use crate::ui_automation::macos::ax::{self, AxElement};
    use anyhow::{anyhow, Result};

    pub struct AxInputWriter {
        window: AxElement,
    }

    impl AxInputWriter {
        pub fn new(window: &AxElement) -> Self {
            Self {
                window: window.clone(),
            }
        }

        pub fn write(&self, text: &str) -> Result<()> {
            let input = ax::find_input_element(&self.window, 8)
                .ok_or_else(|| anyhow!("Input box not found"))?;
            if ax::set_input_value(&input, text).is_ok() {
                return Ok(());
            }
            ax::focus_element(&input).ok();
            ax::paste_text(text)
        }
    }
}
