//! Coinbase Exchange WebSocket Implementation
//!
//! Connects to Coinbase Advanced Trade WebSocket to receive real-time price updates.

use crate::config::CoinbaseConfig;
use crate::error::{ArbitrageError, Result};
use crate::exchanges::{Exchange, Price};
use crate::websocket::MessageParser;
use futures_util::{SinkExt, StreamExt};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use super::parser::CoinbaseParser;

/// Coinbase exchange implementation using WebSocket for price feeds
///
/// # Business Logic
///
/// Connects to Coinbase Advanced Trade WebSocket stream to receive real-time ticker updates.
/// Prices are stored in-memory and can be queried via `get_latest_price()`.
///
/// **WebSocket-only**: This implementation focuses on price feeds only.
/// REST API for trading will be added later.
pub struct CoinbaseExchange {
    name: String,
    config: CoinbaseConfig,
    /// WebSocket manager (moved into spawned task on connect)
    ws_manager_handle: Option<tokio::task::JoinHandle<()>>,
    /// Receiver for price updates from WebSocket
    price_rx: Option<broadcast::Receiver<Price>>,
    /// In-memory store of latest prices by trading pair
    latest_prices: Arc<RwLock<HashMap<String, Price>>>,
    /// Base WebSocket URL
    base_url: String,
}

impl CoinbaseExchange {
    /// Create a new Coinbase exchange instance
    pub fn new(config: CoinbaseConfig) -> Result<Self> {
        // Coinbase Exchange WebSocket endpoint (public, no auth required for ticker)
        // See: https://docs.cdp.coinbase.com/exchange/docs/websocket-feed
        // This is the classic Coinbase Exchange WebSocket, not Advanced Trade
        // Format: wss://ws-feed.exchange.coinbase.com
        let base_url = "wss://ws-feed.exchange.coinbase.com".to_string();

        Ok(Self {
            name: "coinbase".to_string(),
            config,
            ws_manager_handle: None,
            price_rx: None,
            latest_prices: Arc::new(RwLock::new(HashMap::new())),
            base_url,
        })
    }

    /// Connect to WebSocket with a specific ticker subscription
    ///
    /// Coinbase requires sending a subscription message after connection:
    /// {"type":"subscribe","product_ids":["SOL-USDC"],"channels":["ticker"]}
    async fn connect_with_subscription(&mut self, pair: &str) -> Result<()> {
        let product_id = Self::pair_to_product_id(pair);

        // Connect to base WebSocket URL
        let url = self.base_url.clone();
        println!("🔌 CoinbaseExchange: Connecting to {}", url);

        let (ws_stream, response) = connect_async(&url).await.map_err(|e| {
            eprintln!("❌ CoinbaseExchange: Connection failed: {}", e);
            ArbitrageError::NetworkError {
                message: format!("Failed to connect to {}: {}", url, e),
                retry_after: None,
            }
        })?;

        println!(
            "✅ CoinbaseExchange: Connected! Status: {}",
            response.status()
        );

        // Split into read and write halves
        let (mut write, mut read) = ws_stream.split();

        // Send subscription message
        // Classic Coinbase Exchange WebSocket format (public, no auth required)
        // See: https://docs.cdp.coinbase.com/exchange/docs/websocket-feed
        // Format: {"type": "subscribe", "product_ids": ["BTC-USD"], "channels": ["ticker"]}
        let subscribe_msg = serde_json::json!({
            "type": "subscribe",
            "product_ids": [product_id],
            "channels": ["ticker"]
        });

        let subscribe_text =
            serde_json::to_string(&subscribe_msg).map_err(|e| ArbitrageError::ParseError {
                message: format!("Failed to serialize subscription message: {}", e),
                input: None,
            })?;

        println!(
            "📤 CoinbaseExchange: Sending subscription: {}",
            subscribe_text
        );
        write
            .send(Message::Text(subscribe_text))
            .await
            .map_err(|e| ArbitrageError::NetworkError {
                message: format!("Failed to send subscription message: {}", e),
                retry_after: None,
            })?;

        // Create parser
        let parser = CoinbaseParser::new();

        // Create broadcast channel for price updates
        let (message_tx, price_rx) = broadcast::channel(100);
        self.price_rx = Some(price_rx);

        // Spawn background task to handle WebSocket messages
        let prices = self.latest_prices.clone();
        let handle = tokio::spawn(async move {
            let mut ping_interval = tokio::time::interval(std::time::Duration::from_secs(30));

            loop {
                tokio::select! {
                    // Handle incoming messages
                    message_result = read.next() => {
                        match message_result {
                            Some(Ok(Message::Text(text))) => {
                                // Parse message using the parser
                                match parser.parse(&text) {
                                    Ok(parsed) => {
                                        // Broadcast to subscribers
                                        let _ = message_tx.send(parsed.clone());
                                        // Store in cache (silently - no verbose logging)
                                        prices.write().insert(parsed.pair.clone(), parsed);
                                    }
                                    Err(e) => {
                                        // Silently ignore subscription confirmations and other non-ticker messages
                                        let error_str = e.to_string();
                                        if !error_str.contains("Subscription confirmation") {
                                            // Only log actual errors
                                            eprintln!("⚠️ CoinbaseExchange: Parse error: {}", e);
                                        }
                                    }
                                }
                            }
                            Some(Ok(Message::Ping(data))) => {
                                // Respond to server ping with pong
                                if let Err(e) = write.send(Message::Pong(data)).await {
                                    eprintln!("❌ CoinbaseExchange: Failed to send pong: {}", e);
                                    break;
                                }
                            }
                            Some(Ok(Message::Close(_))) => {
                                println!("🔌 CoinbaseExchange: Server closed connection");
                                break;
                            }
                            Some(Err(e)) => {
                                eprintln!("❌ CoinbaseExchange: WebSocket error: {}", e);
                                break;
                            }
                            None => {
                                println!("🔌 CoinbaseExchange: Stream ended");
                                break;
                            }
                            _ => {
                                // Other message types - ignore
                            }
                        }
                    }
                    // Send periodic ping to keep connection alive
                    _ = ping_interval.tick() => {
                        if let Err(e) = write.send(Message::Ping(vec![])).await {
                            eprintln!("❌ CoinbaseExchange: Failed to send ping: {}", e);
                            break;
                        }
                    }
                }
            }
        });

        self.ws_manager_handle = Some(handle);

        Ok(())
    }

    /// Convert trading pair to Coinbase product_id format
    ///
    /// Example: "SOL/USDC" -> "SOL-USDC"
    pub fn pair_to_product_id(pair: &str) -> String {
        pair.replace("/", "-")
    }

    /// Convert Coinbase product_id format to trading pair
    ///
    /// Example: "SOL-USDC" -> "SOL/USDC"
    pub fn product_id_to_pair(product_id: &str) -> String {
        product_id.replace("-", "/")
    }
}

#[async_trait::async_trait]
impl Exchange for CoinbaseExchange {
    async fn connect(&mut self) -> Result<()> {
        // Initial connection without subscription
        // Subscription will be done in subscribe_ticker()
        // For now, just initialize - actual connection happens on subscribe
        Ok(())
    }

    async fn subscribe_ticker(&mut self, pair: &str) -> Result<()> {
        // Disconnect existing connection if any
        self.disconnect().await.ok();

        // Connect with subscription
        self.connect_with_subscription(pair).await?;

        // Wait for first price to arrive (max 10 seconds)
        // This ensures we have data before returning
        let mut attempts = 0;
        let max_attempts = 100; // 100 * 100ms = 10 seconds max wait

        while attempts < max_attempts {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            // Check if we have price data
            if self.latest_prices.read().contains_key(pair) {
                return Ok(());
            }

            attempts += 1;
        }

        // If we get here, connection might still be establishing
        // Return Ok anyway - caller can check get_latest_price() to verify
        Ok(())
    }

    async fn get_latest_price(&self, pair: &str) -> Result<Price> {
        let prices = self.latest_prices.read();
        prices
            .get(pair)
            .cloned()
            .ok_or_else(|| ArbitrageError::ExchangeError {
                exchange: self.name.clone(),
                message: format!("No price data available for {}", pair),
                code: None,
            })
    }

    async fn place_order(
        &mut self,
        _order: crate::exchanges::Order,
    ) -> Result<crate::exchanges::OrderResult> {
        // REST API not implemented yet - WebSocket only
        Err(ArbitrageError::ExchangeError {
            exchange: self.name.clone(),
            message: "Trading not implemented yet - WebSocket price feed only".to_string(),
            code: None,
        })
    }

    async fn get_balance(&self, _asset: &str) -> Result<rust_decimal::Decimal> {
        // REST API not implemented yet - WebSocket only
        Err(ArbitrageError::ExchangeError {
            exchange: self.name.clone(),
            message: "Balance queries not implemented yet - WebSocket price feed only".to_string(),
            code: None,
        })
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_connected(&self) -> bool {
        // Check if we have recent price data (indicates connection is working)
        !self.latest_prices.read().is_empty()
    }

    async fn disconnect(&mut self) -> Result<()> {
        // Cancel WebSocket manager task
        if let Some(handle) = self.ws_manager_handle.take() {
            handle.abort();
        }

        // Clear price data
        self.latest_prices.write().clear();

        Ok(())
    }
}
