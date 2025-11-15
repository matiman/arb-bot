//! Logger Configuration
//!
//! Provides LoggerConfig with parse pattern for type-safe configuration.

use crate::error::{ArbitrageError, Result};
use std::path::Path;
use tracing_subscriber::{
    fmt, layer::SubscriberExt, EnvFilter, Registry,
};

/// Log format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    /// JSON format for production (machine-readable)
    Json,
    /// Pretty format for development (human-readable)
    Pretty,
    /// Compact format (minimal)
    Compact,
}

/// Logger configuration with parse pattern
#[derive(Debug, Clone)]
pub struct LoggerConfig {
    level: String,
    format: LogFormat,
    file_path: Option<String>,
    rotation: String,
}

impl LoggerConfig {
    /// Create a new LoggerConfig with defaults
    pub fn new() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Pretty,
            file_path: None,
            rotation: "never".to_string(),
        }
    }

    /// Set log level
    pub fn with_level(mut self, level: &str) -> Self {
        self.level = level.to_string();
        self
    }

    /// Set log format
    pub fn with_format(mut self, format: LogFormat) -> Self {
        self.format = format;
        self
    }

    /// Set file path for logging
    pub fn with_file_path(mut self, path: &str) -> Self {
        self.file_path = Some(path.to_string());
        self
    }

    /// Set rotation strategy
    pub fn with_rotation(mut self, rotation: &str) -> Self {
        self.rotation = rotation.to_string();
        self
    }

    /// Get log level
    pub fn level(&self) -> &str {
        &self.level
    }

    /// Get log format
    pub fn format(&self) -> LogFormat {
        self.format
    }

    /// Get file path
    pub fn file_path(&self) -> Option<&str> {
        self.file_path.as_deref()
    }

    /// Get rotation strategy
    pub fn rotation(&self) -> &str {
        &self.rotation
    }

    /// Initialize the logger with this configuration
    ///
    /// This sets up the tracing subscriber with the configured format,
    /// file output (if specified), and log level filtering.
    pub fn init(self) -> Result<()> {
        // Use RUST_LOG environment variable if set, otherwise use configured level
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(&self.level));

        // If file path is specified, create file layer
        if let Some(file_path) = &self.file_path {
            let log_dir = Path::new(file_path);
            if let Some(parent) = log_dir.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| ArbitrageError::ConfigError {
                        field: "logging.file_path".to_string(),
                        reason: format!("Failed to create log directory: {}", e),
                    })?;
            }

            let file_appender = match self.rotation.as_str() {
                "daily" => tracing_appender::rolling::daily(file_path, "app.log"),
                "hourly" => tracing_appender::rolling::hourly(file_path, "app.log"),
                _ => tracing_appender::rolling::never(file_path, "app.log"),
            };

            let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
            // Keep guard alive - in production, store this in a static or struct
            std::mem::forget(_guard);

            match self.format {
                LogFormat::Json => {
                    let subscriber = Registry::default()
                        .with(env_filter)
                        .with(fmt::layer().json().with_writer(non_blocking))
                        .with(fmt::layer().json().with_writer(std::io::stdout));
                    // Try to set as global default - may fail if already set (OK in tests)
                    let _ = tracing::subscriber::set_global_default(subscriber);
                }
                LogFormat::Pretty => {
                    let subscriber = Registry::default()
                        .with(env_filter)
                        .with(fmt::layer().pretty().with_writer(non_blocking))
                        .with(fmt::layer().pretty().with_writer(std::io::stdout));
                    let _ = tracing::subscriber::set_global_default(subscriber);
                }
                LogFormat::Compact => {
                    let subscriber = Registry::default()
                        .with(env_filter)
                        .with(fmt::layer().compact().with_writer(non_blocking))
                        .with(fmt::layer().compact().with_writer(std::io::stdout));
                    let _ = tracing::subscriber::set_global_default(subscriber);
                }
            }
        } else {
            // Console only
            match self.format {
                LogFormat::Json => {
                    let subscriber = Registry::default()
                        .with(env_filter)
                        .with(fmt::layer().json().with_writer(std::io::stdout));
                    // Try to set as global default - may fail if already set (OK in tests)
                    let _ = tracing::subscriber::set_global_default(subscriber);
                }
                LogFormat::Pretty => {
                    let subscriber = Registry::default()
                        .with(env_filter)
                        .with(fmt::layer().pretty().with_writer(std::io::stdout));
                    let _ = tracing::subscriber::set_global_default(subscriber);
                }
                LogFormat::Compact => {
                    let subscriber = Registry::default()
                        .with(env_filter)
                        .with(fmt::layer().compact().with_writer(std::io::stdout));
                    let _ = tracing::subscriber::set_global_default(subscriber);
                }
            }
        }

        Ok(())
    }
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_format_variants() {
        let _json = LogFormat::Json;
        let _pretty = LogFormat::Pretty;
        let _compact = LogFormat::Compact;
    }

    #[test]
    fn test_logger_config_defaults() {
        let config = LoggerConfig::new();
        assert_eq!(config.level(), "info");
        assert!(matches!(config.format(), LogFormat::Pretty));
        assert_eq!(config.file_path(), None);
        assert_eq!(config.rotation(), "never");
    }

    #[test]
    fn test_logger_config_builder() {
        let config = LoggerConfig::new()
            .with_level("debug")
            .with_format(LogFormat::Json)
            .with_file_path("logs")
            .with_rotation("daily");

        assert_eq!(config.level(), "debug");
        assert!(matches!(config.format(), LogFormat::Json));
        assert_eq!(config.file_path(), Some("logs"));
        assert_eq!(config.rotation(), "daily");
    }
}

