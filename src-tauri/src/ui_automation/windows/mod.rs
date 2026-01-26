pub mod element;
pub mod uia;

pub use uia::{find_wechat_hwnd, MockUia};
pub use uia::UiaProvider;

#[cfg(test)]
mod tests;
