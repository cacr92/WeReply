mod agent;
pub mod bindings;
mod config;
mod deepseek;
mod ipc;
mod listen_targets;
mod logging;
mod secret;
mod state;
mod types;
mod ui_automation;

use crate::agent::start_agent;
use crate::config::load_config;
use crate::config::save_config;
use crate::secret::ApiKeyManager;
use crate::state::AppState;
use crate::ipc::{
    ChatsListPayload, InputWritePayload, IpcEnvelope, ListenControlPayload, ListenTargetsPayload,
};
use crate::listen_targets::{normalize_listen_targets, MAX_LISTEN_TARGETS};
use crate::types::{
    api_err, api_ok, ApiResponse, ChatSummary, Config, DeepseekDiagnostics, ListenTarget, Platform,
    RuntimeState, Status,
};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, LogicalSize, Manager, Size, State};
use tokio::sync::{Mutex, oneshot};
use tokio::time::{timeout, Duration};
use uuid::Uuid;
use tracing::{info, warn};

type SharedState = Arc<Mutex<AppState>>;

#[tauri::command]
#[specta::specta]
async fn get_config(state: State<'_, SharedState>) -> Result<ApiResponse<Config>, String> {
    let guard = state.lock().await;
    Ok(api_ok(guard.config.clone()))
}

#[tauri::command]
#[specta::specta]
async fn set_config(
    _app: AppHandle,
    _state: State<'_, SharedState>,
    _config: Config,
) -> Result<ApiResponse<()>, String> {
    Ok(api_err("配置已固定为默认值"))
}

#[tauri::command]
#[specta::specta]
async fn list_models(state: State<'_, SharedState>) -> Result<ApiResponse<Vec<String>>, String> {
    let config = {
        let guard = state.lock().await;
        guard.config.clone()
    };
    let api_key = match ApiKeyManager::get_deepseek_api_key() {
        Ok(key) => key,
        Err(err) => return Ok(api_err(err.to_string())),
    };
    match deepseek::list_models(&config, &api_key).await {
        Ok(models) => Ok(api_ok(models)),
        Err(err) => Ok(api_err(err.to_string())),
    }
}

#[tauri::command]
#[specta::specta]
async fn get_status(state: State<'_, SharedState>) -> Result<ApiResponse<Status>, String> {
    let guard = state.lock().await;
    Ok(api_ok(guard.status.clone()))
}

#[tauri::command]
#[specta::specta]
async fn start_listening(
    app: AppHandle,
    state: State<'_, SharedState>,
) -> Result<ApiResponse<()>, String> {
    info!("收到开始监听请求");
    {
        let guard = state.lock().await;
        if guard.status.state == RuntimeState::Listening {
            info!("已在监听中，忽略重复请求");
            return Ok(api_ok(()));
        }
        if guard.listen_targets.is_empty() {
            warn!("未设置监听对象，拒绝开始监听");
            return Ok(api_err("请先设置监听对象"));
        }
    }

    if let Err(err) = ensure_agent_running(app.clone(), state.inner().clone()).await {
        warn!("启动 Agent 失败: {}", err);
        return Ok(api_err(err.to_string()));
    }
    if let Err(err) =
        send_listen_control(state.inner().clone(), "listen.start", true, true).await
    {
        warn!("发送监听指令失败: {}", err);
        return Ok(api_err(err));
    }
    set_runtime_state(&app, state.inner().clone(), RuntimeState::Listening, "").await;
    info!("监听已启动");
    Ok(api_ok(()))
}

#[tauri::command]
#[specta::specta]
async fn stop_listening(
    app: AppHandle,
    state: State<'_, SharedState>,
) -> Result<ApiResponse<()>, String> {
    info!("收到停止监听请求");
    if let Err(err) =
        send_listen_control(state.inner().clone(), "listen.stop", false, false).await
    {
        warn!("发送停止监听指令失败: {}", err);
        return Ok(api_err(err));
    }
    set_runtime_state(&app, state.inner().clone(), RuntimeState::Idle, "").await;
    info!("监听已停止");
    Ok(api_ok(()))
}

#[tauri::command]
#[specta::specta]
async fn pause_listening(
    app: AppHandle,
    state: State<'_, SharedState>,
) -> Result<ApiResponse<()>, String> {
    info!("收到暂停监听请求");
    if let Err(err) =
        send_listen_control(state.inner().clone(), "listen.pause", false, false).await
    {
        warn!("发送暂停监听指令失败: {}", err);
        return Ok(api_err(err));
    }
    set_runtime_state(&app, state.inner().clone(), RuntimeState::Paused, "").await;
    info!("监听已暂停");
    Ok(api_ok(()))
}

#[tauri::command]
#[specta::specta]
async fn resume_listening(
    app: AppHandle,
    state: State<'_, SharedState>,
) -> Result<ApiResponse<()>, String> {
    info!("收到恢复监听请求");
    {
        let guard = state.lock().await;
        if guard.listen_targets.is_empty() {
            warn!("未设置监听对象，拒绝恢复监听");
            return Ok(api_err("请先设置监听对象"));
        }
    }
    if let Err(err) =
        send_listen_control(state.inner().clone(), "listen.resume", true, true).await
    {
        warn!("发送恢复监听指令失败: {}", err);
        return Ok(api_err(err));
    }
    set_runtime_state(&app, state.inner().clone(), RuntimeState::Listening, "").await;
    info!("监听已恢复");
    Ok(api_ok(()))
}

#[tauri::command]
#[specta::specta]
async fn get_listen_targets(
    state: State<'_, SharedState>,
) -> Result<ApiResponse<Vec<ListenTarget>>, String> {
    let guard = state.lock().await;
    Ok(api_ok(guard.listen_targets.clone()))
}

#[tauri::command]
#[specta::specta]
async fn set_listen_targets(
    app: AppHandle,
    state: State<'_, SharedState>,
    targets: Vec<ListenTarget>,
) -> Result<ApiResponse<()>, String> {
    let normalized = match normalize_listen_targets(targets, MAX_LISTEN_TARGETS) {
        Ok(targets) => targets,
        Err(err) => return Ok(api_err(err.to_string())),
    };

    let sender = {
        let mut guard = state.lock().await;
        let mut next_config = guard.config.clone();
        next_config.listen_targets = normalized.clone();
        if let Err(err) = save_config(&app, &next_config) {
            warn!("保存监听对象失败: {}", err);
            return Ok(api_err(err.to_string()));
        }
        guard.config = next_config;
        guard.listen_targets = normalized.clone();
        guard.agent.as_ref().map(|agent| agent.clone_sender())
    };

    if let Some(sender) = sender {
        let payload = ListenTargetsPayload {
            targets: normalized,
        };
        let payload_value = serde_json::to_value(payload).map_err(|err| err.to_string())?;
        if let Err(err) = sender.send(IpcEnvelope::new("listen.targets", payload_value)).await {
            warn!("发送监听对象失败: {}", err);
            return Ok(api_err(err.to_string()));
        }
    }

    Ok(api_ok(()))
}

#[tauri::command]
#[specta::specta]
async fn list_recent_chats(
    state: State<'_, SharedState>,
) -> Result<ApiResponse<Vec<ChatSummary>>, String> {
    list_recent_chats_inner(state.inner().clone()).await
}

async fn list_recent_chats_inner(
    state: SharedState,
) -> Result<ApiResponse<Vec<ChatSummary>>, String> {
    let automation = {
        let guard = state.lock().await;
        guard.automation.clone()
    };
    if automation.is_ready() {
        return Ok(automation.list_recent_chats().await);
    }

    let request_id = Uuid::new_v4().to_string();
    let (sender, receiver) = {
        let mut guard = state.lock().await;
        if guard.pending_chats_list.is_some() {
            return Ok(api_err("已有会话列表请求进行中"));
        }
        let sender = match guard.agent.as_ref() {
            Some(agent) => agent.clone_sender(),
            None => return Ok(api_err("Agent 未连接")),
        };
        let (tx, rx) = oneshot::channel();
        guard.pending_chats_list = Some((request_id.clone(), tx));
        (sender, rx)
    };

    let payload_value =
        serde_json::to_value(ChatsListPayload { request_id: request_id.clone() })
            .map_err(|err| err.to_string())?;
    if let Err(err) = sender.send(IpcEnvelope::new("chats.list", payload_value)).await {
        let mut guard = state.lock().await;
        guard.pending_chats_list = None;
        warn!("发送会话列表请求失败: {}", err);
        return Ok(api_err(err.to_string()));
    }

    match timeout(Duration::from_secs(3), receiver).await {
        Ok(Ok(chats)) => Ok(api_ok(chats)),
        Ok(Err(_)) => {
            let mut guard = state.lock().await;
            if matches!(guard.pending_chats_list.as_ref(), Some((pending_id, _)) if pending_id == &request_id) {
                guard.pending_chats_list = None;
            }
            Ok(api_err("会话列表获取失败"))
        }
        Err(_) => {
            let mut guard = state.lock().await;
            if matches!(guard.pending_chats_list.as_ref(), Some((pending_id, _)) if pending_id == &request_id) {
                guard.pending_chats_list = None;
            }
            Ok(api_err("会话列表请求超时"))
        }
    }
}

#[tauri::command]
#[specta::specta]
async fn write_suggestion(
    state: State<'_, SharedState>,
    chat_id: String,
    text: String,
) -> Result<ApiResponse<()>, String> {
    if chat_id.trim().is_empty() {
        warn!("写入建议失败: chat_id 为空");
        return Ok(api_err("chat_id 不能为空"));
    }
    if text.trim().is_empty() {
        warn!("写入建议失败: 回复内容为空");
        return Ok(api_err("回复内容不能为空"));
    }
    if text.len() > 2000 {
        warn!("写入建议失败: 回复内容过长");
        return Ok(api_err("回复内容过长"));
    }

    let guard = state.lock().await;
    let Some(agent) = guard.agent.as_ref() else {
        warn!("写入建议失败: Agent 未连接");
        return Ok(api_err("Agent 未连接"));
    };

    let payload = InputWritePayload {
        chat_id,
        text,
        mode: Some("paste".to_string()),
        restore_clipboard: Some(true),
    };
    let payload_value = match serde_json::to_value(payload) {
        Ok(value) => value,
        Err(err) => return Ok(api_err(err.to_string())),
    };
    if let Err(err) =
        agent
            .send(crate::ipc::IpcEnvelope::new("input.write", payload_value))
            .await
    {
        warn!("写入建议失败: {}", err);
        return Ok(api_err(err.to_string()));
    }
    info!("写入建议完成");
    Ok(api_ok(()))
}

#[tauri::command]
#[specta::specta]
async fn save_api_key(
    state: State<'_, SharedState>,
    api_key: String,
) -> Result<ApiResponse<()>, String> {
    info!("保存 API 密钥");
    if let Err(err) = ApiKeyManager::set_deepseek_api_key(&api_key) {
        warn!("保存 API 密钥失败: {}", err);
        return Ok(api_err(err.to_string()));
    }

    let config = {
        let guard = state.lock().await;
        guard.config.clone()
    };
    match deepseek::validate_api_key(&config, &api_key).await {
        Ok(()) => {
            info!("API 密钥验证成功");
            Ok(api_ok(()))
        }
        Err(err) => {
            warn!("API 密钥验证失败: {}", err);
            let _ = ApiKeyManager::delete_deepseek_api_key();
            Ok(api_err(err.to_string()))
        }
    }
}

#[tauri::command]
#[specta::specta]
async fn set_deepseek_model(
    app: AppHandle,
    state: State<'_, SharedState>,
    model: String,
) -> Result<ApiResponse<()>, String> {
    if !deepseek::is_supported_model(&model) {
        return Ok(api_err("不支持的模型"));
    }
    let mut guard = state.lock().await;
    guard.config.deepseek_model = model;
    if let Err(err) = save_config(&app, &guard.config) {
        warn!("保存模型失败: {}", err);
        return Ok(api_err(err.to_string()));
    }
    Ok(api_ok(()))
}

#[tauri::command]
#[specta::specta]
async fn get_api_key_status() -> Result<ApiResponse<bool>, String> {
    Ok(match ApiKeyManager::get_deepseek_api_key() {
        Ok(_) => api_ok(true),
        Err(_) => api_ok(false),
    })
}

#[tauri::command]
#[specta::specta]
async fn delete_api_key() -> Result<ApiResponse<()>, String> {
    info!("删除 API 密钥");
    Ok(match ApiKeyManager::delete_deepseek_api_key() {
        Ok(()) => {
            info!("API 密钥已删除");
            api_ok(())
        }
        Err(err) => api_err(err.to_string()),
    })
}

#[tauri::command]
#[specta::specta]
async fn diagnose_deepseek(
    state: State<'_, SharedState>,
    api_key: Option<String>,
) -> Result<ApiResponse<DeepseekDiagnostics>, String> {
    let key = match api_key {
        Some(key) if !key.trim().is_empty() => key,
        _ => match ApiKeyManager::get_deepseek_api_key() {
            Ok(key) => key,
            Err(err) => return Ok(api_err(err.to_string())),
        },
    };
    let config = {
        let guard = state.lock().await;
        guard.config.clone()
    };
    match deepseek::diagnose(&config, &key).await {
        Ok(result) => Ok(api_ok(result)),
        Err(err) => Ok(api_err(err.to_string())),
    }
}

async fn ensure_agent_running(app: AppHandle, state: SharedState) -> anyhow::Result<()> {
    let exists = {
        let guard = state.lock().await;
        guard.agent.is_some()
    };
    if exists {
        return Ok(());
    }
    match start_agent(app.clone(), state.clone()).await {
        Ok(agent) => {
            let mut guard = state.lock().await;
            guard.agent = Some(agent);
            Ok(())
        }
        Err(err) => {
            warn!("启动 Agent 失败: {}", err);
            Err(err)
        }
    }
}

async fn send_listen_control(
    state: SharedState,
    message_type: &str,
    include_poll_interval: bool,
    include_targets: bool,
) -> Result<(), String> {
    let (sender, poll_interval_ms, targets) = {
        let guard = state.lock().await;
        let Some(agent) = guard.agent.as_ref() else {
            return Err("Agent 未连接".to_string());
        };
        (
            agent.clone_sender(),
            if include_poll_interval {
                Some(guard.config.poll_interval_ms)
            } else {
                None
            },
            if include_targets {
                Some(guard.listen_targets.clone())
            } else {
                None
            },
        )
    };
    let payload = ListenControlPayload {
        poll_interval_ms,
        targets,
    };
    let payload_value = serde_json::to_value(payload).map_err(|err| err.to_string())?;
    sender
        .send(crate::ipc::IpcEnvelope::new(message_type, payload_value))
        .await
        .map_err(|err| err.to_string())
}

async fn set_runtime_state(
    app: &AppHandle,
    state: SharedState,
    runtime: RuntimeState,
    last_error: impl Into<String>,
) {
    let mut guard = state.lock().await;
    guard.status.state = runtime;
    guard.status.last_error = last_error.into();
    let _ = app.emit("status.changed", guard.status.clone());
}

fn initial_status() -> Status {
    let platform = if cfg!(target_os = "windows") {
        Platform::Windows
    } else if cfg!(target_os = "macos") {
        Platform::Macos
    } else {
        Platform::Unknown
    };
    Status {
        state: RuntimeState::Idle,
        platform,
        agent_connected: false,
        last_error: String::new(),
    }
}

fn adjust_window_size(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let monitor = window
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| window.primary_monitor().ok().flatten());
    let Some(monitor) = monitor else {
        warn!("获取显示器信息失败，跳过窗口尺寸调整");
        return;
    };
    let scale_factor = monitor.scale_factor();
    let logical_size: LogicalSize<f64> = monitor.size().to_logical(scale_factor);
    let target_width = (logical_size.width * 0.5).round();
    let target_height = (logical_size.height * 0.6).round();
    let width = target_width.clamp(600.0, 1100.0);
    let height = target_height.clamp(640.0, 900.0);
    if let Err(err) = window.set_size(Size::Logical(LogicalSize { width, height })) {
        warn!("窗口尺寸调整失败: {}", err);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let config = load_config(app.handle())?;
            logging::init_logging(app.handle(), &config)?;
            let state = Arc::new(Mutex::new(AppState::new(config, initial_status())));
            app.manage(state);
            adjust_window_size(app.handle());
            info!("WeReply 启动完成");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            set_config,
            start_listening,
            stop_listening,
            pause_listening,
            resume_listening,
            get_listen_targets,
            set_listen_targets,
            list_recent_chats,
            write_suggestion,
            get_status,
            save_api_key,
            get_api_key_status,
            delete_api_key,
            diagnose_deepseek,
            list_models,
            set_deepseek_model
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn list_recent_chats_requires_agent() {
        let state = Arc::new(Mutex::new(AppState::new(
            Config::default(),
            initial_status(),
        )));
        let result = list_recent_chats_inner(state).await.unwrap();
        assert!(!result.success);
    }

    #[tokio::test]
    async fn list_recent_chats_rejects_when_pending() {
        let state = Arc::new(Mutex::new(AppState::new(
            Config::default(),
            initial_status(),
        )));
        let (tx, _rx) = oneshot::channel();
        {
            let mut guard = state.lock().await;
            guard.pending_chats_list = Some(("req".to_string(), tx));
        }
        let result = list_recent_chats_inner(state).await.unwrap();
        assert!(!result.success);
    }
}
