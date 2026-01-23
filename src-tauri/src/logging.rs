use crate::types::Config;
use anyhow::{Context, Result};
use tauri::AppHandle;
use tauri::Manager;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, EnvFilter};

#[allow(dead_code)]
pub struct LogGuard(pub WorkerGuard);

pub fn init_logging(app: &AppHandle, config: &Config) -> Result<()> {
    let filter = EnvFilter::try_new(config.log_level.clone())
        .unwrap_or_else(|_| EnvFilter::new("info"));

    if config.log_to_file {
        let log_dir = app.path().app_log_dir().context("无法获取日志目录")?;
        std::fs::create_dir_all(&log_dir).context("创建日志目录失败")?;
        let file_appender = tracing_appender::rolling::never(log_dir, "wereply.log");
        let (writer, guard) = tracing_appender::non_blocking(file_appender);
        fmt().with_env_filter(filter).with_writer(writer).init();
        app.manage(LogGuard(guard));
    } else {
        fmt().with_env_filter(filter).init();
    }
    Ok(())
}
