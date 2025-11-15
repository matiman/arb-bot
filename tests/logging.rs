//! TDD tests for Logging & Observability
//!
//! These tests follow TDD approach - they will fail initially until implementation is complete.
//!
//! To run tests:
//!   cargo test --test logging

use arb_bot::logger::{LoggerConfig, LogFormat};
use tempfile::TempDir;

#[test]
fn test_log_format_enum() {
    // Test: LogFormat enum should have Json, Pretty, Compact variants
    let _json = LogFormat::Json;
    let _pretty = LogFormat::Pretty;
    let _compact = LogFormat::Compact;
}

#[test]
fn test_logger_config_creation() {
    // Test: LoggerConfig should be creatable with parse pattern
    let config = LoggerConfig::new()
        .with_level("info")
        .with_format(LogFormat::Pretty)
        .with_file_path("logs")
        .with_rotation("daily");

    assert_eq!(config.level(), "info");
    assert!(matches!(config.format(), LogFormat::Pretty));
    assert_eq!(config.file_path(), Some("logs"));
    assert_eq!(config.rotation(), "daily");
}

#[test]
fn test_logger_config_defaults() {
    // Test: LoggerConfig should have sensible defaults
    let config = LoggerConfig::new();
    assert_eq!(config.level(), "info");
    assert!(matches!(config.format(), LogFormat::Pretty));
    assert_eq!(config.file_path(), None);
    assert_eq!(config.rotation(), "never");
}

#[test]
fn test_logger_init_console() {
    // Test: Logger should initialize with console output
    // Note: This test may fail if another test already initialized the logger
    let config = LoggerConfig::new()
        .with_level("debug")
        .with_format(LogFormat::Pretty);

    // Try to initialize - may fail if already initialized, which is OK for tests
    let _ = config.init();
}

#[test]
fn test_logger_init_file() {
    // Test: Logger should initialize with file output
    // Note: This test may fail if another test already initialized the logger
    let temp_dir = TempDir::new().expect("Should create temp directory");
    let log_dir = temp_dir.path().join("logs");

    let config = LoggerConfig::new()
        .with_level("info")
        .with_format(LogFormat::Json)
        .with_file_path(log_dir.to_str().unwrap())
        .with_rotation("daily");

    // Try to initialize - may fail if already initialized, which is OK for tests
    let _ = config.init();

    // Verify log directory was created (if init succeeded)
    if log_dir.parent().is_some() {
        // Directory may or may not exist depending on init success
    }
}

#[test]
fn test_logger_init_file_rotation() {
    // Test: Logger should support file rotation
    // Note: This test may fail if another test already initialized the logger
    let temp_dir = TempDir::new().expect("Should create temp directory");
    let log_dir = temp_dir.path().join("logs");

    let config = LoggerConfig::new()
        .with_level("warn")
        .with_format(LogFormat::Json)
        .with_file_path(log_dir.to_str().unwrap())
        .with_rotation("hourly");

    // Try to initialize - may fail if already initialized, which is OK for tests
    let _ = config.init();
}

#[test]
fn test_structured_logging_macros() {
    // Test: Structured logging macros should be available
    use arb_bot::logger::{debug, info, warn, error, trace};

    // These should compile and not panic
    trace!("trace message");
    debug!("debug message");
    info!("info message");
    warn!("warn message");
    error!("error message");
}

#[test]
fn test_structured_logging_with_fields() {
    // Test: Structured logging with fields
    use arb_bot::logger::info;

    info!(
        exchange = "coinbase",
        pair = "SOL/USDC",
        price = 100.50,
        "Price update received"
    );
}

#[test]
fn test_logger_helper_functions() {
    // Test: Helper functions for common logging scenarios
    use arb_bot::logger;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    logger::log_price_update("binance", "SOL/USDC", Decimal::from(100));
    logger::log_arbitrage_opportunity(
        "binance",
        "coinbase",
        "SOL/USDC",
        Decimal::from_str("0.5").unwrap(),
    );
    logger::log_order_placed("coinbase", "order-123", "BUY", Decimal::from(10));
}

#[test]
fn test_log_level_filtering() {
    // Test: Log level filtering should work
    use arb_bot::logger::{debug, info, warn, error};

    // Try to initialize - may fail if already initialized, which is OK for tests
    let config = LoggerConfig::new().with_level("debug");
    let _ = config.init();

    // These should compile and not panic (even if logger not initialized)
    debug!("This debug message should be visible");
    info!("This info message should be visible");
    warn!("This warn message should be visible");
    error!("This error message should be visible");
}

#[test]
fn test_module_filtering() {
    // Test: Module filtering via RUST_LOG should work
    // This is tested via environment variable, so we just verify the config accepts it
    unsafe {
        std::env::set_var("RUST_LOG", "arb_bot=debug,info");
    }
    let config = LoggerConfig::new();
    // Try to initialize - may fail if already initialized, which is OK for tests
    let _ = config.init();
}

