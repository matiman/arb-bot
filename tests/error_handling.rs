// tests/error_handling.rs

use arb_bot::error::{ArbitrageError, Result};

fn mk_io_error() -> std::io::Error {
    std::io::Error::other("disk gone")
}

#[test]
fn exchange_error_basic() {
    let err = ArbitrageError::ExchangeError {
        exchange: "Binance".to_string(),
        message: "Order failed".to_string(),
        code: Some(1010),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("Binance"));
    assert!(msg.contains("Order failed"));
    assert!(format!("{:?}", err).contains("ExchangeError"));
}

#[test]
fn websocket_error_flags() {
    let err = ArbitrageError::WebSocketError {
        endpoint: "Coinbase".to_string(),
        reconnect_possible: true,
    };
    let msg = err.to_string();
    assert!(msg.to_lowercase().contains("websocket"));
    assert!(msg.to_lowercase().contains("reconnect"));
}

#[test]
fn network_timeout_with_retry_after() {
    let err = ArbitrageError::NetworkError {
        message: "timeout".to_string(),
        retry_after: Some(500),
    };
    let msg = err.to_string();
    assert!(msg.contains("timeout"));
    assert!(msg.contains("500"));
}

#[test]
fn parse_error_with_input_snippet() {
    let err = ArbitrageError::ParseError {
        message: "invalid json".to_string(),
        input: Some("{bad json}".to_string()),
    };
    let msg = err.to_string();
    assert!(msg.to_lowercase().contains("parse"));
    assert!(msg.contains("bad json"));
}

#[test]
fn rate_limit_error() {
    let err = ArbitrageError::RateLimitExceeded {
        exchange: "Coinbase".to_string(),
        retry_after: 120,
    };
    let msg = err.to_string();
    assert!(msg.to_lowercase().contains("rate limit"));
    assert!(msg.contains("120"));
}

#[test]
fn auth_error() {
    let err = ArbitrageError::AuthenticationError {
        exchange: "Binance".to_string(),
        reason: "bad signature".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.to_lowercase().contains("auth"));
    assert!(msg.to_lowercase().contains("binance"));
}

#[test]
fn insufficient_balance() {
    let err = ArbitrageError::InsufficientBalance {
        exchange: "Coinbase".to_string(),
        asset: "USDC".to_string(),
        required: "100.0".to_string(),
        available: "25.0".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.to_lowercase().contains("insufficient"));
    assert!(msg.contains("USDC"));
    assert!(msg.contains("100.0"));
    assert!(msg.contains("25.0"));
}

#[test]
fn from_std_io_error() {
    let io_err = mk_io_error();
    let err: ArbitrageError = io_err.into();
    assert!(format!("{}", err).to_lowercase().contains("io"));
}

#[test]
fn from_serde_json_error() {
    let serde_err: serde_json::Error = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
    let err: ArbitrageError = serde_err.into();
    assert!(format!("{}", err).to_lowercase().contains("json"));
}

#[allow(clippy::result_large_err)]
#[test]
fn result_alias_smoke() {
    fn demo() -> Result<()> {
        Err(ArbitrageError::ConfigError {
            field: "spread_threshold".to_string(),
            reason: "must be > 0".to_string(),
        })
    }
    let e = demo().unwrap_err();
    assert!(format!("{}", e).to_lowercase().contains("config"));
}
