use crate::deepseek;
use crate::ipc::{
    parse_envelope, AgentErrorPayload, AgentReadyPayload, AgentStatusPayload, IpcEnvelope,
    InputResultPayload, MessageNewPayload, validate_message_new,
};
use crate::secret::ApiKeyManager;
use crate::state::{AppState, ChatMessage};
use crate::types::{ErrorPayload, Platform, RuntimeState, SuggestionsUpdated};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tauri::AppHandle;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;
use tauri::{Emitter, Manager};
use tracing::{info, warn};

pub struct AgentHandle {
    sender: mpsc::Sender<IpcEnvelope>,
    _child: tokio::process::Child,
    _read_handle: JoinHandle<()>,
    _write_handle: JoinHandle<()>,
    _stderr_handle: JoinHandle<()>,
}

impl AgentHandle {
    pub fn clone_sender(&self) -> mpsc::Sender<IpcEnvelope> {
        self.sender.clone()
    }

    pub async fn send(&self, message: IpcEnvelope) -> Result<()> {
        self.sender
            .send(message)
            .await
            .context("Agent 写入通道已关闭")
    }
}

pub async fn start_agent(app: AppHandle, state: Arc<Mutex<AppState>>) -> Result<AgentHandle> {
    let (command, args, workdir) = resolve_agent_command(&app)?;
    let mut child = Command::new(command)
        .args(args)
        .current_dir(workdir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("启动 Agent 失败")?;

    let stdin = child.stdin.take().context("Agent stdin 不可用")?;
    let stdout = child.stdout.take().context("Agent stdout 不可用")?;
    let stderr = child.stderr.take().context("Agent stderr 不可用")?;

    let (sender, mut receiver) = mpsc::channel::<IpcEnvelope>(32);

    let write_handle = tokio::spawn(async move {
        let mut stdin = stdin;
        while let Some(message) = receiver.recv().await {
            if let Ok(line) = serde_json::to_string(&message) {
                if stdin.write_all(line.as_bytes()).await.is_err() {
                    break;
                }
                if stdin.write_all(b"\n").await.is_err() {
                    break;
                }
                let _ = stdin.flush().await;
            }
        }
    });

    let read_app = app.clone();
    let read_state = state.clone();
    let read_sender = sender.clone();
    let read_handle = tokio::spawn(async move {
        let mut lines = BufReader::new(stdout).lines();
        loop {
            match lines.next_line().await {
                Ok(Some(line)) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    match parse_envelope(trimmed) {
                        Ok(envelope) => {
                            let ack = IpcEnvelope::ack_for(&envelope.id, true, "");
                            if let Err(err) = read_sender.send(ack).await {
                                warn!("发送 ack 失败: {}", err);
                            }
                            handle_envelope(&read_app, &read_state, envelope).await;
                        }
                        Err(err) => {
                            warn!("解析 Agent 消息失败: {}", err);
                            emit_error(
                                &read_app,
                                ErrorPayload {
                                    code: "PROTOCOL_ERROR".to_string(),
                                    message: "Agent 消息格式错误".to_string(),
                                    recoverable: true,
                                },
                            );
                        }
                    }
                }
                Ok(None) => {
                    emit_error(
                        &read_app,
                        ErrorPayload {
                            code: "AGENT_DISCONNECTED".to_string(),
                            message: "Agent 连接断开".to_string(),
                            recoverable: true,
                        },
                    );
                    update_agent_connected(&read_state, &read_app, false, "Agent 连接断开").await;
                    break;
                }
                Err(err) => {
                    warn!("读取 Agent 输出失败: {}", err);
                    break;
                }
            }
        }
    });

    let stderr_handle = tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if !line.trim().is_empty() {
                warn!("Agent stderr: {}", line);
            }
        }
    });

    info!("Agent 已启动");
    Ok(AgentHandle {
        sender,
        _child: child,
        _read_handle: read_handle,
        _write_handle: write_handle,
        _stderr_handle: stderr_handle,
    })
}

async fn handle_envelope(app: &AppHandle, state: &Arc<Mutex<AppState>>, envelope: IpcEnvelope) {
    match envelope.r#type.as_str() {
        "agent.ready" => {
            if let Ok(payload) = serde_json::from_value::<AgentReadyPayload>(envelope.payload) {
                let platform = match payload.platform.as_str() {
                    "windows" => Platform::Windows,
                    "macos" => Platform::Macos,
                    _ => Platform::Unknown,
                };
                update_platform(state, app, platform).await;
                update_agent_connected(state, app, true, "").await;
            }
        }
        "agent.status" => {
            if let Ok(payload) = serde_json::from_value::<AgentStatusPayload>(envelope.payload) {
                let runtime = match payload.state.as_str() {
                    "listening" => RuntimeState::Listening,
                    "paused" => RuntimeState::Paused,
                    "error" => RuntimeState::Error,
                    _ => RuntimeState::Idle,
                };
                update_state(state, app, runtime, payload.detail).await;
            }
        }
        "agent.error" => {
            if let Ok(payload) = serde_json::from_value::<AgentErrorPayload>(envelope.payload) {
                update_state(state, app, RuntimeState::Error, payload.message.clone()).await;
                emit_error(
                    app,
                    ErrorPayload {
                        code: payload.code,
                        message: payload.message,
                        recoverable: payload.recoverable,
                    },
                );
            }
        }
        "message.new" => {
            if let Ok(payload) = serde_json::from_value::<MessageNewPayload>(envelope.payload) {
                if let Err(err) = validate_message_new(&payload) {
                    warn!("消息验证失败: {}", err);
                    return;
                }
                if is_duplicate_message(state, &payload).await {
                    return;
                }
                record_message(state, &payload).await;
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
                        emit_error(
                            &app_handle,
                            ErrorPayload {
                                code: "SUGGESTION_EMPTY".to_string(),
                                message: "未生成回复建议".to_string(),
                                recoverable: true,
                            },
                        );
                    } else {
                        let payload = SuggestionsUpdated {
                            chat_id: payload.chat_id.clone(),
                            suggestions,
                        };
                        let _ = app_handle.emit("suggestions.updated", payload);
                    }
                    update_state(&state_handle, &app_handle, RuntimeState::Listening, "").await;
                });
            }
        }
        "input.result" => {
            if let Ok(payload) = serde_json::from_value::<InputResultPayload>(envelope.payload) {
                if !payload.ok {
                    emit_error(
                        app,
                        ErrorPayload {
                            code: "WRITE_FAILED".to_string(),
                            message: payload.error,
                            recoverable: true,
                        },
                    );
                }
            }
        }
        _ => {}
    }
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

async fn update_platform(
    state: &Arc<Mutex<AppState>>,
    app: &AppHandle,
    platform: Platform,
) {
    let mut guard = state.lock().await;
    guard.status.platform = platform;
    let _ = app.emit("status.changed", guard.status.clone());
}

async fn update_agent_connected(
    state: &Arc<Mutex<AppState>>,
    app: &AppHandle,
    connected: bool,
    last_error: impl Into<String>,
) {
    let mut guard = state.lock().await;
    guard.status.agent_connected = connected;
    if !connected {
        guard.status.state = RuntimeState::Error;
        guard.status.last_error = last_error.into();
    }
    let _ = app.emit("status.changed", guard.status.clone());
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

fn emit_error(app: &AppHandle, payload: ErrorPayload) {
    let _ = app.emit("error.raised", payload);
}

fn resolve_agent_command(app: &AppHandle) -> Result<(String, Vec<String>, PathBuf)> {
    let base = find_agent_root(app)?;
    let platform_agents = base.join("platform_agents");

    if cfg!(target_os = "windows") {
        let script = platform_agents.join("windows").join("wxauto_agent.py");
        Ok((
            "python".to_string(),
            vec![script.to_string_lossy().to_string()],
            base,
        ))
    } else if cfg!(target_os = "macos") {
        let script = platform_agents.join("macos").join("wechat_agent.swift");
        Ok((
            "swift".to_string(),
            vec![script.to_string_lossy().to_string()],
            base,
        ))
    } else {
        anyhow::bail!("当前系统不支持 Agent");
    }
}

fn find_agent_root(app: &AppHandle) -> Result<PathBuf> {
    if let Ok(resource_dir) = app.path().resource_dir() {
        if resource_dir.join("platform_agents").exists() {
            return Ok(resource_dir);
        }
    }
    let cwd = std::env::current_dir().context("无法获取当前目录")?;
    if cwd.join("platform_agents").exists() {
        return Ok(cwd);
    }
    if let Some(parent) = cwd.parent() {
        if parent.join("platform_agents").exists() {
            return Ok(parent.to_path_buf());
        }
    }
    anyhow::bail!("未找到 platform_agents 目录");
}
