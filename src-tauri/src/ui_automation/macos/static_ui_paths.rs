use super::ax_path::{step, AxPathStep};

pub const SESSION_LIST_PATH_A: &[AxPathStep] = &[
    step(&["AXSplitGroup"], 0, None),
    step(&["AXGroup"], 0, None),
    step(&["AXScrollArea"], 0, None),
    step(&["AXOutline", "AXTable", "AXList"], 0, None),
];

pub const SESSION_LIST_PATH_B: &[AxPathStep] = &[
    step(&["AXSplitGroup"], 0, None),
    step(&["AXGroup"], 0, None),
    step(&["AXOutline", "AXTable", "AXList"], 0, None),
];

pub const MESSAGE_LIST_PATH_A: &[AxPathStep] = &[
    step(&["AXSplitGroup"], 0, None),
    step(&["AXGroup"], 1, None),
    step(&["AXScrollArea"], 0, None),
    step(&["AXList", "AXTable", "AXOutline"], 0, None),
];

pub const MESSAGE_LIST_PATH_B: &[AxPathStep] = &[
    step(&["AXSplitGroup"], 0, None),
    step(&["AXGroup"], 1, None),
    step(&["AXList", "AXTable", "AXOutline"], 0, None),
];

pub const INPUT_PATH_A: &[AxPathStep] = &[
    step(&["AXSplitGroup"], 0, None),
    step(&["AXGroup"], 1, None),
    step(&["AXTextArea", "AXTextField"], 0, None),
];

pub const INPUT_PATH_B: &[AxPathStep] = &[
    step(&["AXSplitGroup"], 0, None),
    step(&["AXGroup"], 1, None),
    step(&["AXGroup"], 0, None),
    step(&["AXTextArea", "AXTextField"], 0, None),
];

pub const SESSION_LIST_PATHS: &[&[AxPathStep]] = &[SESSION_LIST_PATH_A, SESSION_LIST_PATH_B];
pub const MESSAGE_LIST_PATHS: &[&[AxPathStep]] = &[MESSAGE_LIST_PATH_A, MESSAGE_LIST_PATH_B];
pub const INPUT_PATHS: &[&[AxPathStep]] = &[INPUT_PATH_A, INPUT_PATH_B];

pub fn allow_dynamic_scan() -> bool {
    std::env::var("WEREPLY_ALLOW_DYNAMIC_AX_SCAN")
        .ok()
        .as_deref()
        .map(|value| value != "0")
        .unwrap_or(true)
}

#[cfg(test)]
mod tests {
    use super::allow_dynamic_scan;
    use std::sync::Mutex;

    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    fn with_env<F: FnOnce() -> T, T>(value: Option<&str>, f: F) -> T {
        let _guard = ENV_MUTEX.lock().unwrap();
        let key = "WEREPLY_ALLOW_DYNAMIC_AX_SCAN";
        let old = std::env::var(key).ok();
        match value {
            Some(val) => std::env::set_var(key, val),
            None => std::env::remove_var(key),
        }
        let result = f();
        match old {
            Some(val) => std::env::set_var(key, val),
            None => std::env::remove_var(key),
        }
        result
    }

    #[test]
    fn dynamic_scan_enabled_by_default() {
        let enabled = with_env(None, allow_dynamic_scan);
        assert!(enabled);
    }

    #[test]
    fn dynamic_scan_respects_allow_flag() {
        let enabled = with_env(Some("1"), allow_dynamic_scan);
        assert!(enabled);
    }

    #[test]
    fn dynamic_scan_can_be_disabled() {
        let enabled = with_env(Some("0"), allow_dynamic_scan);
        assert!(!enabled);
    }
}
