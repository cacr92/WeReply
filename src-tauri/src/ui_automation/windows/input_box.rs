#[cfg(test)]
pub struct MockInputWriter {
    uia_ok: bool,
    used_clipboard: bool,
}

#[cfg(test)]
impl MockInputWriter {
    pub fn uia_fail() -> Self {
        Self {
            uia_ok: false,
            used_clipboard: false,
        }
    }

    #[allow(dead_code)]
    pub fn uia_ok() -> Self {
        Self {
            uia_ok: true,
            used_clipboard: false,
        }
    }

    pub fn write(&mut self, _chat_id: &str, _text: &str) -> bool {
        if self.uia_ok {
            return true;
        }
        self.used_clipboard = true;
        true
    }

    pub fn used_clipboard(&self) -> bool {
        self.used_clipboard
    }
}

#[cfg(target_os = "windows")]
pub mod uia {
    use anyhow::{anyhow, Result};
    use uiautomation::clipboards::Clipboard;
    use uiautomation::inputs::Keyboard;
    use uiautomation::patterns::UIValuePattern;
    use uiautomation::types::ControlType;
    use uiautomation::{UIAutomation, UIElement};

    pub struct UiaInputWriter {
        automation: UIAutomation,
        window: UIElement,
    }

    impl UiaInputWriter {
        pub fn new(automation: &UIAutomation, window: &UIElement) -> Self {
            Self {
                automation: automation.clone(),
                window: window.clone(),
            }
        }

        pub fn write(&self, text: &str) -> Result<()> {
            let input = find_input_box(&self.automation, &self.window)?;
            input.set_focus().ok();
            if write_via_value_pattern(&input, text).is_ok() {
                return Ok(());
            }
            if write_via_keyboard(text).is_ok() {
                return Ok(());
            }
            write_via_clipboard(&input, text)
        }
    }

    fn find_input_box(automation: &UIAutomation, window: &UIElement) -> Result<UIElement> {
        let window_rect = window.get_bounding_rectangle()?;
        let mid_x = window_rect.get_left() + (window_rect.get_width() / 2);
        let min_y = window_rect.get_top() + (window_rect.get_height() * 2 / 3);
        let candidates = automation
            .create_matcher()
            .from_ref(window)
            .filter_fn(Box::new(move |element| {
                let rect = element.get_bounding_rectangle().ok();
                if let Some(rect) = rect {
                    if rect.get_left() < mid_x || rect.get_top() < min_y {
                        return Ok(false);
                    }
                }
                let control_type = element.get_control_type().ok();
                Ok(matches!(
                    control_type,
                    Some(ControlType::Edit | ControlType::Document | ControlType::Pane)
                ))
            }))
            .depth(14)
            .timeout(0)
            .find_all()
            .unwrap_or_default();
        for element in candidates {
            if let Ok(pattern) = element.get_pattern::<UIValuePattern>() {
                if pattern.is_readonly().unwrap_or(true) {
                    continue;
                }
                return Ok(element);
            }
            if element.get_control_type().ok() == Some(ControlType::Edit) {
                return Ok(element);
            }
        }
        Err(anyhow!("Input box not found"))
    }

    fn write_via_value_pattern(input: &UIElement, text: &str) -> Result<()> {
        let value = input.get_pattern::<UIValuePattern>()?;
        value.set_value("")?;
        value.set_value(text)?;
        Ok(())
    }

    fn write_via_keyboard(text: &str) -> Result<()> {
        let keyboard = Keyboard::default();
        keyboard.send_keys("{ctrl}(a)")?;
        keyboard.send_keys("{backspace}")?;
        keyboard.send_text(text)?;
        Ok(())
    }

    fn write_via_clipboard(input: &UIElement, text: &str) -> Result<()> {
        let clipboard = Clipboard::open()?;
        let original = clipboard.get_text().ok();
        clipboard.set_text(text)?;
        input.set_focus().ok();
        let keyboard = Keyboard::default();
        let _ = keyboard.send_keys("{ctrl}(v)");
        if let Some(original) = original {
            let _ = clipboard.set_text(&original);
        }
        Ok(())
    }
}
