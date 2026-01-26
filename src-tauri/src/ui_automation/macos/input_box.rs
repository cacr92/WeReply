pub struct MockAxInputWriter {
    ax_ok: bool,
    used_clipboard: bool,
}

impl MockAxInputWriter {
    pub fn ax_fail() -> Self {
        Self {
            ax_ok: false,
            used_clipboard: false,
        }
    }

    pub fn ax_ok() -> Self {
        Self {
            ax_ok: true,
            used_clipboard: false,
        }
    }

    pub fn write(&mut self, _chat_id: &str, _text: &str) -> bool {
        if self.ax_ok {
            return true;
        }
        self.used_clipboard = true;
        true
    }

    pub fn used_clipboard(&self) -> bool {
        self.used_clipboard
    }
}
