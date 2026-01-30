use crate::types::{UiPathStep, UiPathsStatus};
use crate::ui_automation::macos::ax_learn::{LearnedPaths, PathStepSpec};
use crate::ui_automation::macos::ax_path::OwnedAxPathStep;
use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};

const UI_PATHS_FILE: &str = "wechat_ui_paths.json";
const UI_TREE_FILE: &str = "wechat_ui_tree.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredUiPaths {
    pub version: u32,
    pub saved_at: u64,
    pub session_list_path: Vec<UiPathStep>,
    pub message_list_path: Vec<UiPathStep>,
    pub input_path: Vec<UiPathStep>,
}

#[derive(Debug, Clone)]
pub struct UiPaths {
    pub session_list: Vec<OwnedAxPathStep>,
    pub message_list: Vec<OwnedAxPathStep>,
    pub input: Vec<OwnedAxPathStep>,
}

static UI_PATHS_STATE: OnceLock<RwLock<Option<UiPaths>>> = OnceLock::new();

fn store() -> &'static RwLock<Option<UiPaths>> {
    UI_PATHS_STATE.get_or_init(|| RwLock::new(None))
}

pub fn get_paths() -> Option<UiPaths> {
    store().read().ok().and_then(|guard| guard.clone())
}

pub fn set_paths(paths: UiPaths) {
    if let Ok(mut guard) = store().write() {
        *guard = Some(paths);
    }
}

pub fn load_from_disk(app: &AppHandle) -> Result<Option<UiPaths>, String> {
    let paths_file = ui_paths_file(app)?;
    let contents = match std::fs::read_to_string(&paths_file) {
        Ok(contents) => contents,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(format!("读取 UI 路径失败: {err}")),
    };
    let stored: StoredUiPaths =
        serde_json::from_str(&contents).map_err(|err| format!("解析 UI 路径失败: {err}"))?;
    let paths = UiPaths::from(&stored);
    set_paths(paths.clone());
    Ok(Some(paths))
}

pub fn read_status(app: &AppHandle) -> Result<UiPathsStatus, String> {
    let paths_file = ui_paths_file(app)?;
    let tree_file = ui_tree_file(app)?;
    let contents = match std::fs::read_to_string(&paths_file) {
        Ok(contents) => contents,
        Err(err) if err.kind() == ErrorKind::NotFound => {
            return Ok(UiPathsStatus {
                saved: false,
                saved_at: None,
                version: None,
                paths_file: None,
                tree_file: None,
            });
        }
        Err(err) => return Err(format!("读取 UI 路径失败: {err}")),
    };
    let stored: StoredUiPaths =
        serde_json::from_str(&contents).map_err(|err| format!("解析 UI 路径失败: {err}"))?;
    Ok(status_from_stored(
        &stored,
        &paths_file,
        &tree_file,
    ))
}

pub fn save_learned_paths(
    app: &AppHandle,
    learned: &LearnedPaths,
    tree_json: &str,
) -> Result<Vec<String>, String> {
    let stored = stored_from_learned(learned)?;
    let paths_file = ui_paths_file(app)?;
    let tree_file = ui_tree_file(app)?;
    let contents = serde_json::to_string_pretty(&stored)
        .map_err(|err| format!("序列化 UI 路径失败: {err}"))?;
    std::fs::write(&paths_file, contents)
        .map_err(|err| format!("写入 UI 路径失败: {err}"))?;
    std::fs::write(&tree_file, tree_json)
        .map_err(|err| format!("写入 UI 树失败: {err}"))?;
    let paths = UiPaths::from(&stored);
    set_paths(paths);
    Ok(vec![
        paths_file.to_string_lossy().to_string(),
        tree_file.to_string_lossy().to_string(),
    ])
}

fn stored_from_learned(learned: &LearnedPaths) -> Result<StoredUiPaths, String> {
    Ok(StoredUiPaths {
        version: 1,
        saved_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| format!("获取时间失败: {err}"))?
            .as_secs(),
        session_list_path: learned
            .session_list
            .iter()
            .map(path_step_from_learned)
            .collect(),
        message_list_path: learned
            .message_list
            .iter()
            .map(path_step_from_learned)
            .collect(),
        input_path: learned.input.iter().map(path_step_from_learned).collect(),
    })
}

fn path_step_from_learned(step: &PathStepSpec) -> UiPathStep {
    UiPathStep {
        roles: step.roles.clone(),
        index: step.index as u32,
        title_contains: step.title_contains.clone(),
    }
}

fn ui_paths_file(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|err| format!("无法获取配置目录: {err}"))?;
    std::fs::create_dir_all(&dir).map_err(|err| format!("创建配置目录失败: {err}"))?;
    Ok(dir.join(UI_PATHS_FILE))
}

fn ui_tree_file(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|err| format!("无法获取配置目录: {err}"))?;
    std::fs::create_dir_all(&dir).map_err(|err| format!("创建配置目录失败: {err}"))?;
    Ok(dir.join(UI_TREE_FILE))
}

impl From<&StoredUiPaths> for UiPaths {
    fn from(value: &StoredUiPaths) -> Self {
        Self {
            session_list: value
                .session_list_path
                .iter()
                .map(path_step_to_owned)
                .collect(),
            message_list: value
                .message_list_path
                .iter()
                .map(path_step_to_owned)
                .collect(),
            input: value.input_path.iter().map(path_step_to_owned).collect(),
        }
    }
}

fn path_step_to_owned(step: &UiPathStep) -> OwnedAxPathStep {
    OwnedAxPathStep {
        roles: step.roles.clone(),
        title_contains: step.title_contains.clone(),
        index: step.index as usize,
    }
}

fn status_from_stored(
    stored: &StoredUiPaths,
    paths_file: &Path,
    tree_file: &Path,
) -> UiPathsStatus {
    let tree_path = if tree_file.exists() {
        Some(tree_file.to_string_lossy().to_string())
    } else {
        None
    };
    UiPathsStatus {
        saved: true,
        saved_at: Some(stored.saved_at),
        version: Some(stored.version),
        paths_file: Some(paths_file.to_string_lossy().to_string()),
        tree_file: tree_path,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn converts_stored_paths_to_owned_steps() {
        let stored = StoredUiPaths {
            version: 1,
            saved_at: 1,
            session_list_path: vec![UiPathStep {
                roles: vec!["AXGroup".to_string()],
                index: 2,
                title_contains: Some("Sessions".to_string()),
            }],
            message_list_path: vec![UiPathStep {
                roles: vec!["AXList".to_string()],
                index: 0,
                title_contains: None,
            }],
            input_path: vec![UiPathStep {
                roles: vec!["AXTextArea".to_string()],
                index: 1,
                title_contains: None,
            }],
        };
        let paths = UiPaths::from(&stored);
        assert_eq!(paths.session_list.len(), 1);
        assert_eq!(paths.session_list[0].index, 2);
        assert_eq!(
            paths.session_list[0].title_contains,
            Some("Sessions".to_string())
        );
        assert_eq!(paths.message_list[0].roles, vec!["AXList".to_string()]);
    }

    #[test]
    fn status_from_stored_includes_paths() {
        let stored = StoredUiPaths {
            version: 2,
            saved_at: 123,
            session_list_path: Vec::new(),
            message_list_path: Vec::new(),
            input_path: Vec::new(),
        };
        let dir = tempdir().expect("tempdir");
        let paths_file = dir.path().join("paths.json");
        let tree_file = dir.path().join("tree.json");
        std::fs::write(&tree_file, "{}").expect("write tree");
        let status = status_from_stored(&stored, &paths_file, &tree_file);
        assert!(status.saved);
        assert_eq!(status.saved_at, Some(123));
        assert_eq!(status.version, Some(2));
        assert_eq!(
            status.tree_file,
            Some(tree_file.to_string_lossy().to_string())
        );
    }
}
