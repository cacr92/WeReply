#[derive(Clone, Debug)]
#[cfg(test)]
pub struct WindowInfo {
    pub hwnd: i64,
    pub process_name: String,
    #[allow(dead_code)]
    pub title: String,
}

#[cfg(test)]
impl WindowInfo {
    pub fn new(hwnd: i64, process_name: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            hwnd,
            process_name: process_name.into(),
            title: title.into(),
        }
    }
}
