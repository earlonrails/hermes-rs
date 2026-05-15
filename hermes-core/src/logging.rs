use crate::paths::get_hermes_home;
use std::path::PathBuf;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
    Layer,
};
use tracing::Level;

/// Logging mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Cli,
    Gateway,
    Cron,
}

pub struct LoggingConfig {
    pub hermes_home: Option<PathBuf>,
    pub level: Level,
    pub mode: Option<Mode>,
    pub force: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            hermes_home: None,
            level: Level::INFO,
            mode: None,
            force: false,
        }
    }
}

/// Setup the centralized logging for Hermes Agent.
pub fn setup_logging(config: LoggingConfig) -> PathBuf {
    let home = config.hermes_home.unwrap_or_else(get_hermes_home);
    let log_dir = home.join("logs");

    if !log_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&log_dir) {
            eprintln!("Failed to create log directory: {}", e);
        }
    }

    // agent.log (INFO+)
    let agent_file_appender = RollingFileAppender::new(Rotation::DAILY, &log_dir, "agent.log");
    let agent_layer = fmt::layer()
        .with_writer(agent_file_appender)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_filter(tracing_subscriber::filter::LevelFilter::from_level(config.level));

    // errors.log (WARNING+)
    let errors_file_appender = RollingFileAppender::new(Rotation::DAILY, &log_dir, "errors.log");
    let errors_layer = fmt::layer()
        .with_writer(errors_file_appender)
        .with_filter(tracing_subscriber::filter::LevelFilter::from_level(Level::WARN));

    // Optional gateway.log
    let gateway_layer = if config.mode == Some(Mode::Gateway) {
        let gateway_file_appender = RollingFileAppender::new(Rotation::DAILY, &log_dir, "gateway.log");
        Some(
            fmt::layer()
                .with_writer(gateway_file_appender)
                .with_filter(tracing_subscriber::filter::LevelFilter::from_level(Level::INFO))
        )
    } else {
        None
    };

    // Console output for CLI mode (only show warnings/errors to console unless verbose mode later configures it)
    let console_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_filter(tracing_subscriber::filter::LevelFilter::WARN);

    // Initialize the subscriber
    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env()
            .add_directive(config.level.into()))
        .with(agent_layer)
        .with(errors_layer)
        .with(console_layer);

    if let Some(gw_layer) = gateway_layer {
        let _ = subscriber.with(gw_layer).try_init();
    } else {
        let _ = subscriber.try_init();
    }

    log_dir
}
