use thiserror::Error;

pub type Result<T> = std::result::Result<T, ArbitrageError>;

#[derive(Debug, Error)]
pub enum ArbitrageError {
    ExchangeError {
        exchange: String,
        message: String,
        code: Option<i32>,
    },

    WebSocketError {
        endpoint: String,
        reconnect_possible: bool,
    },

    NetworkError {
        message: String,
        retry_after: Option<u64>,
    },

    ParseError {
        message: String,
        input: Option<String>,
    },

    ConfigError {
        field: String,
        reason: String,
    },

    RateLimitExceeded {
        exchange: String,
        retry_after: u64,
    },

    AuthenticationError {
        exchange: String,
        reason: String,
    },

    InsufficientBalance {
        exchange: String,
        asset: String,
        required: String,
        available: String,
    },

    Io(#[from] std::io::Error),

    Json(#[from] serde_json::Error),

    WebSocketLib(#[from] tokio_tungstenite::tungstenite::Error),
}

impl ArbitrageError {
    fn format_optional_code(code: &Option<i32>) -> String {
        match code {
            Some(c) => format!(" (code {c})"),
            None => String::new(),
        }
    }

    fn format_optional_retry(retry_after: &Option<u64>) -> String {
        match retry_after {
            Some(ms) => format!("; retry after {ms}ms"),
            None => String::new(),
        }
    }

    fn format_optional_input(input: &Option<String>) -> String {
        match input {
            Some(s) => format!("; input: {s}"),
            None => String::new(),
        }
    }
}

impl std::fmt::Display for ArbitrageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArbitrageError::ExchangeError { exchange, message, code } => {
                let code_s = Self::format_optional_code(code);
                write!(f, "Exchange error on {exchange}: {message} - {code_s}")
            }
            ArbitrageError::WebSocketError { endpoint, reconnect_possible } => {
                write!(f, "WebSocket error from {endpoint} (reconnect_possible={reconnect_possible})")
            }
            ArbitrageError::NetworkError { message, retry_after } => {
                let retry = Self::format_optional_retry(retry_after);
                write!(f, "Network error: {message} - {retry}")
            }
            ArbitrageError::ParseError { message, input } => {
                let input_s = Self::format_optional_input(input);
                write!(f, "Parse error: {message}{input_s}")
            }
            ArbitrageError::ConfigError { field, reason } => {
                write!(f, "Config error: field '{field}' - {reason}")
            }
            ArbitrageError::RateLimitExceeded { exchange, retry_after } => {
                write!(f, "Rate limit exceeded on {exchange}, retry after {retry_after}ms")
            }
            ArbitrageError::AuthenticationError { exchange, reason } => {
                write!(f, "Authentication error on {exchange}: {reason}")
            }
            ArbitrageError::InsufficientBalance { exchange, asset, required, available } => {
                write!(f, "Insufficient balance on {exchange} for {asset}: required {required}, available {available}")
            }
            ArbitrageError::Io(e) => write!(f, "IO error: {e}"),
            ArbitrageError::Json(e) => write!(f, "JSON error: {e}"),
            ArbitrageError::WebSocketLib(e) => write!(f, "WebSocket library error: {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_exchange_with_code() {
        let e = ArbitrageError::ExchangeError { exchange: "X".into(), message: "m".into(), code: Some(1) };
        let s = e.to_string();
        assert!(s.contains("code 1"));
    }

    #[test]
    fn from_io() {
        let e: ArbitrageError = std::io::Error::other("x").into();
        assert!(e.to_string().to_lowercase().contains("io"));
    }
}
