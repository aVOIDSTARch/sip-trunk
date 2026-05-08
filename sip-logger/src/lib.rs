use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, EnvFilter};

pub struct LogConfig {
    pub level: String,
    pub file_path: Option<String>,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file_path: None,
        }
    }
}

pub struct LoggerGuard {
    _guard: WorkerGuard,
}

pub fn init_logger(config: LogConfig) -> LoggerGuard {
    let filter = EnvFilter::try_new(&config.level).unwrap_or_else(|_| {
        eprintln!(
            "sip-logger: invalid log level '{}', falling back to INFO",
            config.level
        );
        EnvFilter::new("info")
    });

    match config.file_path {
        Some(path) => {
            let p = std::path::Path::new(&path);
            let dir = p.parent().unwrap_or_else(|| std::path::Path::new("."));
            let prefix = p.file_name().unwrap_or_else(|| std::ffi::OsStr::new("sip-trunk"));
            let appender = tracing_appender::rolling::daily(dir, prefix);
            let (non_blocking, guard) = tracing_appender::non_blocking(appender);
            fmt()
                .with_env_filter(filter)
                .with_writer(non_blocking)
                .init();
            LoggerGuard { _guard: guard }
        }
        None => {
            let (non_blocking, guard) = tracing_appender::non_blocking(std::io::stdout());
            fmt()
                .with_env_filter(filter)
                .with_writer(non_blocking)
                .init();
            LoggerGuard { _guard: guard }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_info_level() {
        let config = LogConfig::default();
        assert_eq!(config.level, "info");
        assert!(config.file_path.is_none());
    }
}
