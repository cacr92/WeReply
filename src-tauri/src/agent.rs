use crate::deepseek;
use crate::ipc::{
    parse_envelope, AgentErrorPayload, AgentReadyPayload, AgentStatusPayload, IpcEnvelope,
    InputResultPayload, MessageNewPayload, validate_message_new,
};
use crate::secret::ApiKeyManager;
use crate::state::{AppState, ChatMessage};
use crate::types::{ErrorPayload, Platform, RuntimeState, SuggestionsUpdated};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use tauri::AppHandle;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;
use tokio::time::{timeout, Duration};
use tauri::{Emitter, Manager};
use tracing::{info, warn};

pub struct AgentHandle {
    sender: mpsc::Sender<IpcEnvelope>,
    _child: tokio::process::Child,
    _read_handle: JoinHandle<()>,
    _write_handle: JoinHandle<()>,
    _stderr_handle: JoinHandle<()>,
}

struct AgentCommand {
    command: String,
    args: Vec<String>,
    workdir: PathBuf,
    env: Vec<(String, String)>,
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
    if cfg!(target_os = "windows") {
        ensure_windows_agent_dependencies(&app).await?;
    }
    let agent = resolve_agent_command(&app)?;
    let mut cmd = Command::new(&agent.command);
    cmd.args(&agent.args).current_dir(&agent.workdir);
    for (key, value) in &agent.env {
        cmd.env(key, value);
    }
    let mut child = cmd
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
                info!("Agent 就绪: {}", payload.platform);
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
                info!("Agent 状态更新: {}", payload.state);
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
                warn!("Agent 错误: {}", payload.message);
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

fn resolve_agent_command(app: &AppHandle) -> Result<AgentCommand> {
    let base = find_agent_root(app)?;
    let platform_agents = base.join("platform_agents");

    if cfg!(target_os = "windows") {
        let script = platform_agents.join("windows").join("wxauto_agent.py");
        let (python, env) = resolve_windows_python(app, &base)?;
        Ok(AgentCommand {
            command: python,
            args: vec![script.to_string_lossy().to_string()],
            workdir: base,
            env,
        })
    } else if cfg!(target_os = "macos") {
        let script = platform_agents.join("macos").join("wechat_agent.swift");
        Ok(AgentCommand {
            command: "swift".to_string(),
            args: vec![script.to_string_lossy().to_string()],
            workdir: base,
            env: Vec::new(),
        })
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

const WINDOWS_AGENT_MODULES: &[&str] = &["wxauto", "pyautogui", "pyperclip"];
const WINDOWS_DEP_INSTALL_TIMEOUT_SECONDS: u64 = 60;

static WINDOWS_DEP_READY: AtomicBool = AtomicBool::new(false);
static WINDOWS_DEP_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn windows_dep_lock() -> &'static Mutex<()> {
    WINDOWS_DEP_LOCK.get_or_init(|| Mutex::new(()))
}

fn python_check_args(modules: &[&str]) -> Vec<String> {
    let mut script = String::new();
    for module in modules {
        script.push_str("import ");
        script.push_str(module);
        script.push('\n');
    }
    vec!["-c".to_string(), script]
}

fn pip_install_args(requirements: &str) -> Vec<String> {
    vec![
        "-m".to_string(),
        "pip".to_string(),
        "install".to_string(),
        "--disable-pip-version-check".to_string(),
        "--no-input".to_string(),
        "-r".to_string(),
        requirements.to_string(),
    ]
}

fn windows_requirements_path(base: &Path) -> PathBuf {
    base.join("platform_agents")
        .join("windows")
        .join("requirements.txt")
}

fn embedded_python_paths(resource_root: &Path) -> (PathBuf, PathBuf) {
    (
        resource_root.join("python").join("python.exe"),
        resource_root
            .join("python")
            .join("Lib")
            .join("site-packages"),
    )
}

fn embedded_python_exists(resource_root: &Path) -> bool {
    let (python, _) = embedded_python_paths(resource_root);
    python.exists()
}

fn embedded_python_env(resource_root: &Path) -> Vec<(String, String)> {
    let (python, site) = embedded_python_paths(resource_root);
    let python_home = python
        .parent()
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_default();
    vec![
        ("PYTHONHOME".to_string(), python_home),
        ("PYTHONPATH".to_string(), site.to_string_lossy().to_string()),
        ("PYTHONNOUSERSITE".to_string(), "1".to_string()),
    ]
}

fn resolve_windows_python(app: &AppHandle, base: &Path) -> Result<(String, Vec<(String, String)>)> {
    if let Ok(resource_dir) = app.path().resource_dir() {
        if embedded_python_exists(&resource_dir) {
            let (python, _) = embedded_python_paths(&resource_dir);
            return Ok((
                python.to_string_lossy().to_string(),
                embedded_python_env(&resource_dir),
            ));
        }
    }

    let repo_resources = base.join("src-tauri").join("resources");
    if embedded_python_exists(&repo_resources) {
        let (python, _) = embedded_python_paths(&repo_resources);
        return Ok((
            python.to_string_lossy().to_string(),
            embedded_python_env(&repo_resources),
        ));
    }

    Ok(("python".to_string(), Vec::new()))
}

async fn run_python_command(
    python: &str,
    args: Vec<String>,
    workdir: &Path,
    env: &[(String, String)],
) -> Result<()> {
    let mut cmd = Command::new(python);
    cmd.args(args).current_dir(workdir);
    for (key, value) in env {
        cmd.env(key, value);
    }
    let output = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("调用 Python 失败")?;

    if output.status.success() {
        return Ok(());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    warn!("Python 执行失败 stdout: {}", stdout.trim());
    warn!("Python 执行失败 stderr: {}", stderr.trim());
    anyhow::bail!("Python 命令执行失败");
}

async fn ensure_windows_agent_dependencies(app: &AppHandle) -> Result<()> {
    if WINDOWS_DEP_READY.load(Ordering::SeqCst) {
        return Ok(());
    }

    let _guard = windows_dep_lock().lock().await;
    if WINDOWS_DEP_READY.load(Ordering::SeqCst) {
        return Ok(());
    }

    let base = find_agent_root(app)?;
    let (python, env) = resolve_windows_python(app, &base)?;
    let requirements = windows_requirements_path(&base);
    if !requirements.exists() {
        anyhow::bail!("未找到 Windows Agent 依赖列表");
    }

    info!("检测 Windows Agent Python 依赖");
    if run_python_command(
        &python,
        python_check_args(WINDOWS_AGENT_MODULES),
        &base,
        &env,
    )
    .await
    .is_ok()
    {
        WINDOWS_DEP_READY.store(true, Ordering::SeqCst);
        return Ok(());
    }

    info!("依赖缺失，开始自动安装");
    let install = timeout(
        Duration::from_secs(WINDOWS_DEP_INSTALL_TIMEOUT_SECONDS),
        run_python_command(
            &python,
            pip_install_args(&requirements.to_string_lossy()),
            &base,
            &env,
        ),
    )
    .await
    .context("安装依赖超时")?;

    install.context("自动安装依赖失败")?;

    info!("依赖安装完成，进行复检");
    run_python_command(
        &python,
        python_check_args(WINDOWS_AGENT_MODULES),
        &base,
        &env,
    )
        .await
        .context("依赖复检失败")?;

    WINDOWS_DEP_READY.store(true, Ordering::SeqCst);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn python_check_args_include_required_modules() {
        let args = python_check_args(&["wxauto", "pyautogui", "pyperclip"]);
        assert_eq!(args[0], "-c");
        assert!(args[1].contains("import wxauto"));
        assert!(args[1].contains("import pyautogui"));
        assert!(args[1].contains("import pyperclip"));
    }

    #[test]
    fn pip_install_args_include_requirements_flag() {
        let args = pip_install_args("C:/path/requirements.txt");
        assert_eq!(args[0], "-m");
        assert_eq!(args[1], "pip");
        assert!(args.iter().any(|arg| arg == "-r"));
    }

    #[test]
    fn windows_requirements_path_is_under_platform_agents() {
        let base = std::path::Path::new("C:/app");
        let path = windows_requirements_path(base);
        assert!(path.ends_with("platform_agents/windows/requirements.txt"));
    }

    #[test]
    fn python_check_args_are_stable_for_three_modules() {
        let args = python_check_args(&["wxauto", "pyautogui", "pyperclip"]);
        assert_eq!(args.len(), 2);
    }

    #[test]
    fn embedded_python_paths_use_resource_layout() {
        let base = std::path::Path::new("C:/app/resources");
        let (python, site) = embedded_python_paths(base);
        assert!(python.ends_with("python/python.exe"));
        assert!(site.ends_with("python/Lib/site-packages"));
    }

    #[test]
    fn embedded_python_env_sets_pythonhome_and_pythonpath() {
        let base = std::path::Path::new("C:/app/resources");
        let env = embedded_python_env(base);
        assert!(env.iter().any(|(k, _)| k == "PYTHONHOME"));
        assert!(env.iter().any(|(k, _)| k == "PYTHONPATH"));
    }

    #[test]
    fn embedded_python_exists_flag_checks_exe_path() {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path();
        std::fs::create_dir_all(base.join("python")).unwrap();
        std::fs::write(base.join("python").join("python.exe"), "").unwrap();
        assert!(embedded_python_exists(base));
    }
}
