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
        == Some("1")
}
