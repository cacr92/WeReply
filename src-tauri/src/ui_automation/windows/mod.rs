pub mod element;
pub mod message_watch;
pub mod session_list;
pub mod uia;

pub use message_watch::{MockWatcher, WatchMode};
pub use session_list::{collect_recent_chats, MockSessionList, SessionListProvider};
pub use uia::{find_wechat_hwnd, MockUia};
pub use uia::UiaProvider;

#[cfg(test)]
mod tests;
