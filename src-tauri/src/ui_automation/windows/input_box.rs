pub struct MockInputWriter {
    uia_ok: bool,
    used_clipboard: bool,
}

impl MockInputWriter {
    pub fn uia_fail() -> Self {
        Self {
            uia_ok: false,
            used_clipboard: false,
        }
    }

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
