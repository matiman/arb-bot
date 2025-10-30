use crate::error::ArbitrageError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Invalid spread threshold: {value} - {reason}")]
    InvalidSpreadThreshold { value: f64, reason: String },

    #[error("Invalid order_size: {value} - {reason}")]
    InvalidOrderSize { value: f64, reason: String },

    #[error("Invalid cooldown: {value}ms - {reason}")]
    InvalidCooldown { value: u64, reason: String },

    #[error("Invalid decimal conversion")]
    InvalidDecimal,

    #[error("Missing required field: {field}")]
    MissingField { field: String },
}

impl From<ConfigError> for ArbitrageError {
    fn from(e: ConfigError) -> Self {
        ArbitrageError::ConfigParse(Box::new(e))
    }
}
