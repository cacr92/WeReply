use crate::deepseek;
use crate::ipc::{validate_message_new, MessageNewPayload};
use crate::secret::ApiKeyManager;
use crate::state::{AppState, ChatMessage};
use crate::types::{ErrorPayload, RuntimeState, SuggestionsUpdated};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use tracing::{info, warn};

pub async fn handle_incoming_message(
    app: &AppHandle,
    state: &Arc<Mutex<AppState>>,
    payload: MessageNewPayload,
) {
    if let Err(err) = validate_message_new(&payload) {
        warn!("消息验证失败: {}", err);
        return;
    }
    if is_duplicate_message(state, &payload).await {
        return;
    }
    record_message(state, &payload).await;
    info!("收到新消息，生成回复建议");
    update_state(state, app, RuntimeState::Generating, "").await;
    let context = {
        let guard = state.lock().await;
        guard.context_for_chat(&payload.chat_id)
    };
    let config = {
        let guard = state.lock().await;
        guard.config.clone()
    };
    let app_handle = app.clone();
    let state_handle = state.clone();
    tokio::spawn(async move {
        let api_key = ApiKeyManager::get_deepseek_api_key().ok();
        let suggestions = deepseek::generate_suggestions(&config, api_key, &context)
            .await
            .unwrap_or_else(|_| Vec::new());
        if suggestions.is_empty() {
            warn!("生成建议为空");
            emit_error(
                &app_handle,
                ErrorPayload {
                    code: "SUGGESTION_EMPTY".to_string(),
                    message: "未生成回复建议".to_string(),
                    recoverable: true,
                },
            );
        } else {
            info!("生成建议完成: {} 条", suggestions.len());
            let payload = SuggestionsUpdated {
                chat_id: payload.chat_id.clone(),
                suggestions,
            };
            let _ = app_handle.emit("suggestions.updated", payload);
        }
        update_state(&state_handle, &app_handle, RuntimeState::Listening, "").await;
    });
}

async fn is_duplicate_message(state: &Arc<Mutex<AppState>>, payload: &MessageNewPayload) -> bool {
    let guard = state.lock().await;
    guard.is_duplicate(
        &payload.chat_id,
        &payload.msg_id,
        &payload.text,
        payload.timestamp,
    )
}

async fn record_message(state: &Arc<Mutex<AppState>>, payload: &MessageNewPayload) {
    let mut guard = state.lock().await;
    guard.record_message(
        &payload.chat_id,
        ChatMessage {
            text: payload.text.clone(),
            timestamp: payload.timestamp,
            msg_id: payload.msg_id.clone(),
        },
    );
}

async fn update_state(
    state: &Arc<Mutex<AppState>>,
    app: &AppHandle,
    runtime: RuntimeState,
    last_error: impl Into<String>,
) {
    let mut guard = state.lock().await;
    guard.status.state = runtime;
    guard.status.last_error = last_error.into();
    let _ = app.emit("status.changed", guard.status.clone());
}

fn emit_error(app: &AppHandle, payload: ErrorPayload) {
    let _ = app.emit("error.raised", payload);
}
