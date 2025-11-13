//! Integration tests for Coinbase Exchange implementation
//!
//! Following TDD: These tests will fail initially until implementation exists.
//! Tests marked with `#[ignore]` require live connection to Coinbase WebSocket.
//! Run them with: `cargo test --test coinbase -- --ignored`
//!
//! Note: Coinbase WebSocket works without API keys for public ticker streams.

use arb_bot::config::CoinbaseConfig;
use arb_bot::exchanges::Exchange;
use arb_bot::exchanges::coinbase::{CoinbaseExchange, CoinbaseParser};
use arb_bot::websocket::MessageParser;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::timeout;

/// Helper to create a test Coinbase config
/// For WebSocket ticker streams, no API keys are needed
fn create_sandbox_config() -> CoinbaseConfig {
    // WebSocket ticker streams don't require authentication
    let api_key = std::env::var("COINBASE_SANDBOX_API_KEY").unwrap_or_else(|_| String::new());
    let api_secret = std::env::var("COINBASE_SANDBOX_API_SECRET").unwrap_or_else(|_| String::new());
    CoinbaseConfig {
        api_key,
        api_secret,
        sandbox: true, // Use sandbox
    }
}

/// Helper to create a production Coinbase config
/// Coinbase WebSocket works without API keys for public ticker streams
fn create_production_config() -> CoinbaseConfig {
    CoinbaseConfig {
        api_key: String::new(),
        api_secret: String::new(),
        sandbox: false, // Use production
    }
}

#[tokio::test]
#[ignore] // Ignored by default - requires live connection
async fn test_coinbase_connect() {
    // Test connecting to Coinbase WebSocket
    let config = create_production_config();
    let mut exchange = CoinbaseExchange::new(config).unwrap();

    // subscribe_ticker() handles the connection
    exchange.subscribe_ticker("SOL/USD").await.unwrap();

    // Give it a moment for price data to arrive
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Check if connected (has price data)
    assert!(exchange.is_connected());

    // Verify we can actually get a price
    let price = exchange.get_latest_price("SOL/USD").await.unwrap();
    assert_eq!(price.pair, "SOL/USD");

    exchange.disconnect().await.unwrap();
}

#[tokio::test]
#[ignore] // Ignored by default - requires live connection
async fn test_coinbase_subscribe_ticker() {
    // Test subscribing to ticker and receiving price updates
    let config = create_production_config();
    let mut exchange = CoinbaseExchange::new(config).unwrap();

    exchange.subscribe_ticker("SOL/USD").await.unwrap();

    // Wait for price update
    let price = timeout(Duration::from_secs(15), async {
        loop {
            match exchange.get_latest_price("SOL/USD").await {
                Ok(p) => return p,
                Err(_) => tokio::time::sleep(Duration::from_millis(100)).await,
            }
        }
    })
    .await
    .unwrap();

    assert_eq!(price.pair, "SOL/USD");
    assert!(price.bid > Decimal::ZERO);
    assert!(price.ask > price.bid);
    assert!(price.last > Decimal::ZERO);

    exchange.disconnect().await.unwrap();
}

#[tokio::test]
async fn test_coinbase_parser_valid_ticker() {
    // Test CoinbaseParser with valid ticker message (actual WebSocket format)
    let ticker_json = r#"{
        "channel": "ticker",
        "client_id": "",
        "timestamp": "2025-10-30T12:00:00.000000Z",
        "sequence_num": 0,
        "events": [
            {
                "type": "snapshot",
                "tickers": [
                    {
                        "type": "ticker",
                        "product_id": "SOL-USDC",
                        "price": "143.50",
                        "best_bid": "143.48",
                        "best_ask": "143.52",
                        "volume_24_h": "1234567.89"
                    }
                ]
            }
        ]
    }"#;

    let parser = CoinbaseParser::new();
    let price = parser.parse(ticker_json).unwrap();

    assert_eq!(price.pair, "SOL/USDC");
    assert_eq!(price.last, Decimal::from_str("143.50").unwrap());
    assert_eq!(price.bid, Decimal::from_str("143.48").unwrap());
    assert_eq!(price.ask, Decimal::from_str("143.52").unwrap());
}

#[tokio::test]
async fn test_coinbase_parser_invalid_message() {
    // Test parser rejects invalid messages
    let invalid_json = r#"{"channel": "unknown"}"#;

    let parser = CoinbaseParser::new();
    let result = parser.parse(invalid_json);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_coinbase_parser_missing_fields() {
    // Test parser handles missing required fields
    let incomplete_json = r#"{"channel": "ticker"}"#;

    let parser = CoinbaseParser::new();
    let result = parser.parse(incomplete_json);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_coinbase_product_id_conversion() {
    // Test pair format conversion
    // CoinbaseExchange::pair_to_product_id converts SOL/USDC to SOL-USDC
    assert_eq!(CoinbaseExchange::pair_to_product_id("SOL/USDC"), "SOL-USDC");

    // CoinbaseParser::product_id_to_pair converts SOL-USDC to SOL/USDC
    assert_eq!(CoinbaseParser::product_id_to_pair("SOL-USDC"), "SOL/USDC");
}

#[tokio::test]
async fn test_coinbase_error_handling() {
    // Test handling of errors for invalid pairs
    // This doesn't require a live connection - just test error handling
    let config = create_production_config();
    let exchange = CoinbaseExchange::new(config).unwrap();

    // Try to get price for pair we haven't subscribed to
    // Should return error (no price data available)
    let result = exchange.get_latest_price("INVALID/PAIR").await;
    assert!(result.is_err());

    // Error should indicate no price data
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("No price data") || error_msg.contains("INVALID"));
}

#[tokio::test]
async fn test_coinbase_rest_sign_request() {
    // Test JWT token generation
    // REST API deferred - test will be implemented in arbitrage logic phase
    // This test is intentionally empty until REST API is implemented
}

#[tokio::test]
#[ignore] // REST API deferred - requires sandbox API keys
async fn test_coinbase_rest_get_balance() {
    // Test balance query (sandbox)
    // REST API deferred - test will be implemented in arbitrage logic phase
    let _config = create_sandbox_config();
    // This will fail until CoinbaseRestClient is implemented
    // let client = CoinbaseRestClient::new(config.api_key, config.api_secret, config.sandbox);
    // let balance = client.get_balance("USDC").await.unwrap();
    // assert!(balance >= Decimal::ZERO);
}

#[tokio::test]
#[ignore] // REST API deferred - requires sandbox API keys
async fn test_coinbase_rest_place_order() {
    // Test market order placement (sandbox)
    // REST API deferred - test will be implemented in arbitrage logic phase
    let _config = create_sandbox_config();
    // This will fail until CoinbaseRestClient and CoinbaseExchange are implemented
    // let mut exchange = CoinbaseExchange::new(config).unwrap();
    //
    // let order = Order::market_buy("SOL/USDC", Decimal::from(10));
    // let result = exchange.place_order(order).await.unwrap();
    //
    // assert!(result.is_complete() || !result.is_complete()); // Either is valid
    // assert!(!result.order_id.is_empty());
}
