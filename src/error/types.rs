use thiserror::Error;

pub type Result<T> = std::result::Result<T, ArbitrageError>;

#[derive(Debug, Error)]
pub enum ArbitrageError {
    #[error("Exchange error on {exchange}: {message}{}", code.map(|c| format!(" (code {c})")).unwrap_or_default())]
    ExchangeError {
        exchange: String,
        message: String,
        code: Option<i32>,
    },

    #[error("WebSocket error from {endpoint} (reconnect_possible={reconnect_possible})")]
    WebSocketError {
        endpoint: String,
        reconnect_possible: bool,
    },

    #[error("Network error: {message}{}", retry_after.map(|ms| format!("; retry after {ms}ms")).unwrap_or_default())]
    NetworkError {
        message: String,
        retry_after: Option<u64>,
    },

    #[error("Parse error: {message}{}", input.as_ref().map(|s| format!("; input: {s}")).unwrap_or_default())]
    ParseError {
        message: String,
        input: Option<String>,
    },

    #[error("Config error: field '{field}' - {reason}")]
    ConfigError { field: String, reason: String },

    #[error("Rate limit exceeded on {exchange}, retry after {retry_after}ms")]
    RateLimitExceeded { exchange: String, retry_after: u64 },

    #[error("Authentication error on {exchange}: {reason}")]
    AuthenticationError { exchange: String, reason: String },

    #[error(
        "Insufficient balance on {exchange} for {asset}: required {required}, available {available}"
    )]
    InsufficientBalance {
        exchange: String,
        asset: String,
        required: String,
        available: String,
    },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("WebSocket library error: {0}")]
    WebSocketLib(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error(transparent)]
    ConfigParse(Box<crate::config::parse::ConfigError>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_exchange_with_code() {
        let e = ArbitrageError::ExchangeError {
            exchange: "X".into(),
            message: "m".into(),
            code: Some(1),
        };
        let s = e.to_string();
        assert!(s.contains("code 1"));
    }

    #[test]
    fn from_io() {
        let e: ArbitrageError = std::io::Error::other("x").into();
        assert!(e.to_string().to_lowercase().contains("io"));
    }
}
