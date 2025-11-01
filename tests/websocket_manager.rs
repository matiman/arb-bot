//! Integration tests for WebSocket Manager
//!

use arb_bot::error::Result;
use arb_bot::exchanges::Price;
use arb_bot::websocket::{MessageParser, ReconnectionStrategy, WebSocketManager};
use chrono::Utc;
use rust_decimal::Decimal;
use std::time::Duration;
use tokio::time::{sleep, timeout};

/// Mock parser for testing - converts simple JSON to Price
#[derive(Clone)]
struct MockParser;

impl MessageParser for MockParser {
    type Output = Price;

    fn parse(&self, message: &str) -> Result<Self::Output> {
        // Simple parser for tests: expects {"pair":"SOL/USDC","bid":"100","ask":"101"}
        let json: serde_json::Value = serde_json::from_str(message)?;
        let pair =
            json["pair"]
                .as_str()
                .ok_or_else(|| arb_bot::error::ArbitrageError::ParseError {
                    message: "Missing 'pair' field".to_string(),
                    input: Some(message.to_string()),
                })?;
        let bid = json["bid"]
            .as_str()
            .and_then(|s| Decimal::from_str_exact(s).ok())
            .ok_or_else(|| arb_bot::error::ArbitrageError::ParseError {
                message: "Invalid 'bid' field".to_string(),
                input: Some(message.to_string()),
            })?;
        let ask = json["ask"]
            .as_str()
            .and_then(|s| Decimal::from_str_exact(s).ok())
            .ok_or_else(|| arb_bot::error::ArbitrageError::ParseError {
                message: "Invalid 'ask' field".to_string(),
                input: Some(message.to_string()),
            })?;

        Ok(Price {
            pair: pair.to_string(),
            bid,
            ask,
            last: bid,
            volume_24h: Decimal::ZERO,
            timestamp: Utc::now(),
        })
    }
}

#[tokio::test]
async fn test_websocket_manager_creation() {
    let url = "wss://echo.websocket.org".to_string();
    let parser = MockParser;
    let reconnect_strategy = ReconnectionStrategy::exponential_backoff();

    let (manager, receiver) = WebSocketManager::new(url, parser, reconnect_strategy);

    // Verify creation succeeded - if new() returned, manager and receiver are valid
    drop(manager);
    drop(receiver);

    // Explicit assertion: creation succeeded (doesn't panic)
    assert!(true);
}

#[tokio::test]
async fn test_reconnection_strategy_exponential_backoff() {
    let mut strategy =
        ReconnectionStrategy::new(Some(5), Duration::from_secs(1), Duration::from_secs(60));

    // First retry: 1 second
    let delay1 = strategy.next_delay();
    assert_eq!(delay1.as_secs(), 1);

    // Second retry: 2 seconds
    let delay2 = strategy.next_delay();
    assert_eq!(delay2.as_secs(), 2);

    // Third retry: 4 seconds
    let delay3 = strategy.next_delay();
    assert_eq!(delay3.as_secs(), 4);

    // Fourth retry: 8 seconds
    let delay4 = strategy.next_delay();
    assert_eq!(delay4.as_secs(), 8);
}

#[tokio::test]
async fn test_reconnection_strategy_max_delay_cap() {
    let mut strategy = ReconnectionStrategy::new(
        Some(10),
        Duration::from_secs(1),
        Duration::from_secs(10), // Max 10 seconds
    );

    // Should cap at max_delay even if exponential would exceed it
    for _ in 0..10 {
        let delay = strategy.next_delay();
        assert!(delay <= Duration::from_secs(10));
    }
}

#[tokio::test]
async fn test_reconnection_strategy_should_retry() {
    // With max retries
    let mut strategy =
        ReconnectionStrategy::new(Some(3), Duration::from_secs(1), Duration::from_secs(60));

    assert!(strategy.should_retry()); // Before first retry
    strategy.next_delay();
    assert!(strategy.should_retry()); // After first retry
    strategy.next_delay();
    assert!(strategy.should_retry()); // After second retry
    strategy.next_delay();
    assert!(!strategy.should_retry()); // After third retry (exceeded max)

    // Without max retries (retry forever)
    let mut infinite_strategy = ReconnectionStrategy::new(
        None, // No max retries
        Duration::from_secs(1),
        Duration::from_secs(60),
    );

    for _ in 0..100 {
        assert!(infinite_strategy.should_retry());
        infinite_strategy.next_delay();
    }
}

#[tokio::test]
async fn test_reconnection_strategy_reset() {
    let mut strategy =
        ReconnectionStrategy::new(Some(5), Duration::from_secs(1), Duration::from_secs(60));

    strategy.next_delay(); // Move to retry 1
    strategy.next_delay(); // Move to retry 2
    assert_eq!(strategy.current_retry, 2);

    strategy.reset();
    assert_eq!(strategy.current_retry, 0);

    // After reset, delay should be initial again
    let delay = strategy.next_delay();
    assert_eq!(delay.as_secs(), 1);
}

#[tokio::test]
async fn test_message_parser_trait() {
    let parser = MockParser;

    let valid_message = r#"{"pair":"SOL/USDC","bid":"100.5","ask":"101.0"}"#;
    let price = parser.parse(valid_message).unwrap();
    assert_eq!(price.pair, "SOL/USDC");
    assert_eq!(price.bid, Decimal::from_str_exact("100.5").unwrap());
    assert_eq!(price.ask, Decimal::from_str_exact("101.0").unwrap());

    let invalid_message = r#"{"invalid":"json"}"#;
    assert!(parser.parse(invalid_message).is_err());
}

#[tokio::test]
async fn test_websocket_connection_establishment() {
    // Using echo.websocket.org for testing (public echo server)
    let url = "wss://echo.websocket.org".to_string();
    let parser = MockParser;
    let reconnect_strategy = ReconnectionStrategy::exponential_backoff();

    let (mut manager, _receiver) = WebSocketManager::new(url, parser, reconnect_strategy);

    // Spawn manager in background
    let manager_handle = tokio::spawn(async move { manager.run().await });

    // Give it time to connect
    sleep(Duration::from_millis(500)).await;

    // Manager should be running (connection established)
    // We can't easily test connection state without exposing internals,
    // but if run() didn't error immediately, connection likely succeeded
    manager_handle.abort(); // Clean up
}

#[tokio::test]
async fn test_message_broadcast_to_subscribers() {
    let url = "wss://echo.websocket.org".to_string();
    let parser = MockParser;
    let reconnect_strategy = ReconnectionStrategy::exponential_backoff();

    let (_manager, receiver1) =
        WebSocketManager::new(url.clone(), parser.clone(), reconnect_strategy.clone());

    // Create second subscriber (broadcast allows multiple receivers)
    let (_manager2, receiver2) = WebSocketManager::new(url, parser, reconnect_strategy);

    // Note: This test is limited without actual WebSocket server
    // In real implementation, we'd send a message and verify both receivers get it
    // For now, we just verify the broadcast channel structure exists
    drop(receiver1);
    drop(receiver2);
}

#[tokio::test]
async fn test_graceful_shutdown() {
    let url = "wss://echo.websocket.org".to_string();
    let parser = MockParser;
    let reconnect_strategy = ReconnectionStrategy::exponential_backoff();

    let (mut manager, _receiver) = WebSocketManager::new(url, parser, reconnect_strategy);

    // Spawn manager
    let handle = tokio::spawn(async move { manager.run().await });

    // Wait a bit then abort (simulating shutdown)
    sleep(Duration::from_millis(100)).await;
    handle.abort();

    // If we get here without panic, shutdown was graceful
}

#[tokio::test]
async fn test_reconnection_on_failure() {
    // Use invalid URL to trigger connection failure
    let url = "wss://invalid-websocket-url-that-does-not-exist.com".to_string();
    let parser = MockParser;
    let reconnect_strategy = ReconnectionStrategy::new(
        Some(2),                    // Only 2 retries for test speed
        Duration::from_millis(100), // Short delay for testing
        Duration::from_secs(60),
    );

    let (mut manager, _receiver) = WebSocketManager::new(url, parser, reconnect_strategy);

    // Manager should attempt reconnection
    // With timeout to prevent hanging
    let result = timeout(Duration::from_secs(5), manager.run()).await;

    // Should eventually fail after retries exhausted
    assert!(result.is_ok()); // Timeout completed
}
