mod config;
mod deepseek;
mod agent;
mod ipc;

use serde::Serialize;

#[derive(Serialize)]
struct Status {
    state: String,
    platform: String,
    agent_connected: bool,
    last_error: String,
}

#[tauri::command]
fn start_listening() -> bool {
    true
}

#[tauri::command]
fn stop_listening() -> bool {
    true
}

#[tauri::command]
fn get_status() -> Status {
    let platform = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "unknown"
    };
    Status {
        state: "idle".to_string(),
        platform: platform.to_string(),
        agent_connected: false,
        last_error: "".to_string(),
    }
}

#[tauri::command]
fn write_suggestion(_chat_id: String, _text: String) -> bool {
    true
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            start_listening,
            stop_listening,
            get_status,
            write_suggestion
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
