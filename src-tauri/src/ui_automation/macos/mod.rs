pub mod ax;
pub mod element;
pub mod message_watch;
pub mod input_box;
pub mod session_list;

pub use ax::{find_wechat_app, AxProvider, MockAx};
pub use input_box::MockAxInputWriter;
pub use message_watch::{MockAxWatcher, WatchMode};
pub use session_list::{collect_recent_chats, AxSessionListProvider, MockAxSessionList};

#[cfg(test)]
mod tests;
