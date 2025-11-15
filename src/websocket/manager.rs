//! Generic WebSocket manager for exchange connections
//!
//! Handles connection lifecycle, message parsing, broadcasting, and reconnection logic.

use crate::error::{ArbitrageError, Result};
use crate::websocket::{MessageParser, ReconnectionStrategy};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// Generic WebSocket manager for exchange price feeds
///
/// # Business Logic
///
/// This manager:
/// 1. Maintains persistent WebSocket connection to exchange
/// 2. Receives messages → parses via `MessageParser` → broadcasts to subscribers
/// 3. Automatically reconnects on failure using `ReconnectionStrategy`
/// 4. Sends periodic ping messages to keep connection alive
//TODO Change Ping Pong to Heartbeat to keep connection alive if exchange supports it
/// # Example Usage
///
/// ```rust,no_run
/// use arb_bot::websocket::{WebSocketManager, ReconnectionStrategy};
/// use arb_bot::websocket::MessageParser;
/// use arb_bot::exchanges::Price;
/// use arb_bot::error::Result;
///
/// #[derive(Clone)]
/// struct MyParser;
///
/// impl MessageParser for MyParser {
///     type Output = Price;
///     fn parse(&self, msg: &str) -> Result<Self::Output> { todo!() }
/// }
// TODO: Move to Main.rs
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let url = "wss://exchange.com/ws".to_string();
///     let parser = MyParser;
///     let strategy = ReconnectionStrategy::exponential_backoff();
///
///     let (mut manager, mut receiver) = WebSocketManager::new(url, parser, strategy);
///
///     // Spawn manager in background
///     tokio::spawn(async move {
///         manager.run().await
///     });
///
///     // Receive parsed messages
///     while let Ok(price) = receiver.recv().await {
///         println!("Price update: {:?}", price);
///     }
///     Ok(())
/// }
/// ```
pub struct WebSocketManager<P: MessageParser> {
    /// WebSocket URL to connect to
    url: String,
    /// Parser for converting messages to common types
    parser: P,
    /// Reconnection strategy for handling failures
    reconnect_strategy: ReconnectionStrategy,
    /// Broadcast channel for sending parsed messages to subscribers
    message_tx: broadcast::Sender<P::Output>,
    /// Interval for sending ping messages (default: 30 seconds)
    health_check_interval: std::time::Duration,
}

impl<P: MessageParser> WebSocketManager<P> {
    /// Create a new WebSocket manager
    ///
    /// # Returns
    ///
    /// Tuple of `(WebSocketManager, Receiver)` where:
    /// - `WebSocketManager`: The manager instance to run
    /// - `Receiver`: Broadcast receiver for subscribing to parsed messages
    ///
    /// Multiple receivers can be created by calling `receiver.resubscribe()`.
    pub fn new(
        url: String,
        parser: P,
        reconnect_strategy: ReconnectionStrategy,
    ) -> (Self, broadcast::Receiver<P::Output>) {
        let (message_tx, message_rx) = broadcast::channel(100);

        let manager = Self {
            url,
            parser,
            reconnect_strategy,
            message_tx,
            health_check_interval: std::time::Duration::from_secs(30),
        };

        (manager, message_rx)
    }

    /// Run the WebSocket manager (blocks until connection closes or error)
    ///
    /// # Behavior
    ///
    /// 1. Attempts to connect to WebSocket URL
    /// 2. On success: runs message loop (receive, parse, broadcast)
    /// 3. On failure: uses `ReconnectionStrategy` to retry with exponential backoff
    /// 4. Returns when connection closes normally or retries exhausted
    pub async fn run(&mut self) -> Result<()> {
        loop {
            match self.connect_and_run().await {
                Ok(_) => {
                    // Connection closed normally
                    return Ok(());
                }
                Err(e) => {
                    // Connection failed
                    if !self.reconnect_strategy.should_retry() {
                        return Err(e);
                    }

                    let delay = self.reconnect_strategy.next_delay();
                    tokio::time::sleep(delay).await;
                    // Loop continues to retry
                }
            }
        }
    }

    /// Connect and run the message loop
    async fn connect_and_run(&mut self) -> Result<()> {
        println!("🔌 WebSocketManager: Attempting to connect to {}", self.url);
        // Connect to WebSocket
        let (ws_stream, response) = connect_async(&self.url).await.map_err(|e| {
            eprintln!("❌ WebSocketManager: Connection failed: {}", e);
            ArbitrageError::NetworkError {
                message: format!("Failed to connect to {}: {}", self.url, e),
                retry_after: None,
            }
        })?;

        println!(
            "✅ WebSocketManager: Connected! Status: {}",
            response.status()
        );

        // Split into read and write halves since we need to send and receive messages concurrently
        let (mut write, mut read) = ws_stream.split();

        // Reset retry counter on successful connection
        self.reconnect_strategy.reset();

        // Set up ping interval for health checks
        let mut ping_interval = tokio::time::interval(self.health_check_interval);

        // Main message loop
        loop {
            tokio::select! {
                // Handle incoming messages
                //Creates a future that resolves to the next item in the stream.
                message_result = read.next() => {
                    match message_result {
                        Some(Ok(Message::Text(text))) => {
                            // Parse message using the parser
                            match self.parser.parse(&text) {
                                Ok(parsed) => {
                                    // Broadcast to all subscribers
                                    // Ignore error if no subscribers
                                    let _ = self.message_tx.send(parsed);
                                }
                                Err(e) => {
                                    // Log parse error but continue running
                                    // Not all messages may be price updates
                                    eprintln!("Failed to parse message: {}", e);
                                }
                            }
                        }
                        Some(Ok(Message::Ping(data))) => {
                            // Respond to server ping with pong
                            if let Err(e) = write.send(Message::Pong(data)).await {
                                return Err(ArbitrageError::NetworkError {
                                    message: format!("Failed to send pong: {}", e),
                                    retry_after: None,
                                });
                            }
                        }
                        Some(Ok(Message::Close(_))) => {
                            // Server closed connection
                            return Ok(());
                        }
                        Some(Err(e)) => {
                            // WebSocket error
                            return Err(ArbitrageError::NetworkError {
                                message: format!("WebSocket error: {}", e),
                                retry_after: None,
                            });
                        }
                        None => {
                            // Stream ended
                            return Ok(());
                        }
                        _ => {
                            // Other message types (binary, pong, etc.) - ignore
                        }
                    }
                }
                // Send periodic ping to keep connection alive
                _ = ping_interval.tick() => {
                    if let Err(e) = write.send(Message::Ping(vec![])).await {
                        return Err(ArbitrageError::NetworkError {
                            message: format!("Failed to send ping: {}", e),
                            retry_after: None,
                        });
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exchanges::Price;
    use chrono::Utc;
    use rust_decimal::Decimal;

    #[derive(Clone)]
    struct TestParser;

    impl MessageParser for TestParser {
        type Output = Price;

        fn parse(&self, message: &str) -> Result<Self::Output> {
            let json: serde_json::Value = serde_json::from_str(message)?;
            Ok(Price {
                pair: json["pair"].as_str().unwrap_or("UNKNOWN").to_string(),
                bid: Decimal::from_str_exact(json["bid"].as_str().unwrap_or("0"))
                    .unwrap_or(Decimal::ZERO),
                ask: Decimal::from_str_exact(json["ask"].as_str().unwrap_or("0"))
                    .unwrap_or(Decimal::ZERO),
                last: Decimal::ZERO,
                volume_24h: Decimal::ZERO,
                timestamp: Utc::now(),
            })
        }
    }

    #[tokio::test]
    async fn test_manager_creation() {
        let url = "wss://echo.websocket.org".to_string();
        let parser = TestParser;
        let strategy = ReconnectionStrategy::exponential_backoff();

        let (manager, receiver) = WebSocketManager::new(url.clone(), parser, strategy);

        // Verify manager was created successfully
        // (Can't easily test internals, but if new() returned, it succeeded)
        drop(manager);
        drop(receiver);
        // Test passes if creation doesn't panic
    }

    #[tokio::test]
    async fn test_manager_broadcast_channel() {
        let url = "wss://echo.websocket.org".to_string();
        let parser = TestParser;
        let strategy = ReconnectionStrategy::exponential_backoff();

        let (_manager, mut receiver1) =
            WebSocketManager::new(url.clone(), parser.clone(), strategy.clone());

        // Create second receiver (broadcast allows multiple)
        let mut receiver2 = receiver1.resubscribe();

        // Channel structure is valid
        assert!(receiver1.try_recv().is_err()); // No messages yet
        assert!(receiver2.try_recv().is_err()); // No messages yet
    }
}
