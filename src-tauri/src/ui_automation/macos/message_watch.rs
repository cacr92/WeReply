#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WatchMode {
    Event,
    Polling,
}

pub struct MockAxWatcher {
    subscribe_ok: bool,
}

impl MockAxWatcher {
    pub fn subscribe_fail() -> Self {
        Self { subscribe_ok: false }
    }

    pub fn subscribe_ok() -> Self {
        Self { subscribe_ok: true }
    }

    pub fn start(&self) -> WatchMode {
        if self.subscribe_ok {
            WatchMode::Event
        } else {
            WatchMode::Polling
        }
    }
}
