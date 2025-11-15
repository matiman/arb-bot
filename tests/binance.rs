//! Integration tests for Binance Exchange implementation
//!
//! Tests marked with `#[ignore]` require live connection to Binance.US WebSocket.
//! Run them with: `cargo test --test binance -- --ignored`
//!
//! Note: Binance testnet is geo-restricted (HTTP 451) in US, so tests use
//! Binance.US production which works without API keys for public ticker streams.

use arb_bot::config::BinanceConfig;
use arb_bot::exchanges::Exchange;
use arb_bot::exchanges::binance::{BinanceExchange, BinanceParser};
use arb_bot::websocket::MessageParser;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::timeout;

/// Helper to create a test Binance config
/// For testnet, API keys are optional (some endpoints work without auth)
/// For WebSocket ticker streams, no API keys are needed
fn create_testnet_config() -> BinanceConfig {
    // For testnet, we can use dummy keys or read from env
    // WebSocket ticker streams don't require authentication
    let api_key = std::env::var("BINANCE_TESTNET_API_KEY").unwrap_or_else(|_| String::new());
    let api_secret = std::env::var("BINANCE_TESTNET_API_SECRET").unwrap_or_else(|_| String::new());
    BinanceConfig {
        api_key,
        api_secret,
        testnet: true, // Use testnet
    }
}

/// Helper to create a production Binance.US config
/// Binance.US WebSocket works without API keys for public ticker streams
fn create_production_config() -> BinanceConfig {
    BinanceConfig {
        api_key: String::new(),
        api_secret: String::new(),
        testnet: false, // Use Binance.US production
    }
}

#[tokio::test]
#[ignore] // Ignored by default - requires live connection
async fn test_binance_connect_testnet() {
    // Test connecting to Binance.US (testnet is geo-restricted in US)
    // Use production Binance.US which works without API keys for public ticker streams
    let config = create_production_config();
    let mut exchange = BinanceExchange::new(config).unwrap();

    // subscribe_ticker() handles the connection and waits for first price
    exchange.subscribe_ticker("SOL/USDC").await.unwrap();

    // Give it a moment for price data to arrive (subscribe_ticker already waits, but be safe)
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Check if connected (has price data)
    assert!(exchange.is_connected());

    // Verify we can actually get a price
    let price = exchange.get_latest_price("SOL/USDC").await.unwrap();
    assert_eq!(price.pair, "SOL/USDC");

    exchange.disconnect().await.unwrap();
}

#[tokio::test]
#[ignore] // Ignored by default - requires live connection
async fn test_binance_subscribe_ticker() {
    // Test subscribing to ticker and receiving price updates
    // Use production Binance.US (more reliable than testnet)
    let config = create_production_config();
    let mut exchange = BinanceExchange::new(config).unwrap();

    exchange.subscribe_ticker("SOL/USDC").await.unwrap();

    // Wait for price update (subscribe_ticker already waits, but double-check)
    let price = timeout(Duration::from_secs(15), async {
        loop {
            match exchange.get_latest_price("SOL/USDC").await {
                Ok(p) => return p,
                Err(_) => tokio::time::sleep(Duration::from_millis(100)).await,
            }
        }
    })
    .await
    .unwrap();

    assert_eq!(price.pair, "SOL/USDC");
    assert!(price.bid > Decimal::ZERO);
    assert!(price.ask > price.bid);
    assert!(price.last > Decimal::ZERO);

    exchange.disconnect().await.unwrap();
}

#[tokio::test]
async fn test_binance_parser_valid_ticker() {
    // Test BinanceParser with valid ticker message
    let ticker_json = r#"{
        "e": "24hrTicker",
        "s": "SOLUSDC",
        "c": "143.50",
        "b": "143.48",
        "a": "143.52",
        "v": "1234567.89"
    }"#;

    let parser = BinanceParser::new();
    let price = parser.parse(ticker_json).unwrap();

    assert_eq!(price.pair, "SOL/USDC");
    assert_eq!(price.last, Decimal::from_str("143.50").unwrap());
    assert_eq!(price.bid, Decimal::from_str("143.48").unwrap());
    assert_eq!(price.ask, Decimal::from_str("143.52").unwrap());
}

#[tokio::test]
async fn test_binance_parser_invalid_message() {
    // Test parser rejects invalid messages
    let invalid_json = r#"{"type": "unknown"}"#;

    let parser = BinanceParser::new();
    let result = parser.parse(invalid_json);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_binance_parser_missing_fields() {
    // Test parser handles missing required fields
    let incomplete_json = r#"{"e": "24hrTicker"}"#;

    let parser = BinanceParser::new();
    let result = parser.parse(incomplete_json);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_binance_rest_sign_request() {
    // Test HMAC SHA256 signing
    // REST API deferred - test will be implemented in arbitrage logic phase
    // This test is intentionally empty until REST API is implemented
}

#[tokio::test]
#[ignore] // REST API deferred - requires testnet API keys
async fn test_binance_rest_get_balance() {
    // Test balance query (testnet)
    // REST API deferred - test will be implemented in arbitrage logic phase
    let _config = create_testnet_config();
    // This will fail until BinanceRestClient is implemented
    // let client = BinanceRestClient::new(config.api_key, config.api_secret, config.testnet);
    // let balance = client.get_balance("USDC").await.unwrap();
    // assert!(balance >= Decimal::ZERO);
}

#[tokio::test]
#[ignore] // REST API deferred - requires testnet API keys
async fn test_binance_rest_place_order() {
    // Test market order placement (testnet)
    // REST API deferred - test will be implemented in arbitrage logic phase
    let _config = create_testnet_config();
    // This will fail until BinanceRestClient and BinanceExchange are implemented
    // let mut exchange = BinanceExchange::new(config).unwrap();
    //
    // let order = Order::market_buy("SOL/USDC", Decimal::from(10));
    // let result = exchange.place_order(order).await.unwrap();
    //
    // assert!(result.is_complete() || !result.is_complete()); // Either is valid
    // assert!(!result.order_id.is_empty());
}

#[tokio::test]
async fn test_binance_symbol_conversion() {
    // Test pair format conversion
    // BinanceParser::symbol_to_pair converts symbol to pair format
    assert_eq!(BinanceParser::symbol_to_pair("SOLUSDC"), "SOL/USDC");

    // BinanceParser::pair_to_symbol returns uppercase (Binance convention)
    assert_eq!(BinanceParser::pair_to_symbol("SOL/USDC"), "SOLUSDC");
}

#[tokio::test]
async fn test_binance_error_handling() {
    // Test handling of errors for invalid pairs
    // This doesn't require a live connection - just test error handling
    let config = create_production_config();
    let exchange = BinanceExchange::new(config).unwrap();

    // Try to get price for pair we haven't subscribed to
    // Should return error (no price data available)
    let result = exchange.get_latest_price("INVALID/PAIR").await;
    assert!(result.is_err());

    // Error should indicate no price data
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("No price data") || error_msg.contains("INVALID"));
}
