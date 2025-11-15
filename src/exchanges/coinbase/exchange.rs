//! Coinbase Exchange WebSocket Implementation
//!
//! Connects to Coinbase Advanced Trade WebSocket to receive real-time price updates.

use crate::config::CoinbaseConfig;
use crate::error::{ArbitrageError, Result};
use crate::exchanges::{Exchange, Price};
use crate::logger::{debug, error, info, warn};
use crate::websocket::MessageParser;
use futures_util::{SinkExt, StreamExt};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use super::parser::CoinbaseParser;
use super::rest::CoinbaseRestClient;

/// Coinbase exchange implementation using WebSocket for price feeds
///
/// # Business Logic
///
/// Connects to Coinbase Advanced Trade WebSocket stream to receive real-time ticker updates.
/// Prices are stored in-memory and can be queried via `get_latest_price()`.
///
/// REST API client is available for order placement and balance queries.
pub struct CoinbaseExchange {
    name: String,
    #[allow(dead_code)] // Kept for future use (sandbox flag)
    config: CoinbaseConfig,
    /// WebSocket manager (moved into spawned task on connect)
    ws_manager_handle: Option<tokio::task::JoinHandle<()>>,
    /// Receiver for price updates from WebSocket
    price_rx: Option<broadcast::Receiver<Price>>,
    /// In-memory store of latest prices by trading pair
    latest_prices: Arc<RwLock<HashMap<String, Price>>>,
    /// Base WebSocket URL
    base_url: String,
    /// REST API client for trading operations (optional, only if API credentials provided)
    rest_client: Option<CoinbaseRestClient>,
}

impl CoinbaseExchange {
    /// Create a new Coinbase exchange instance
    pub fn new(config: CoinbaseConfig) -> Result<Self> {
        // Coinbase Exchange WebSocket endpoint (public, no auth required for ticker)
        // See: https://docs.cdp.coinbase.com/exchange/docs/websocket-feed
        // This is the classic Coinbase Exchange WebSocket, not Advanced Trade
        // Format: wss://ws-feed.exchange.coinbase.com
        let base_url = crate::constants::websocket::COINBASE_EXCHANGE.to_string();

        // Initialize REST client if API credentials are provided
        // First try config, then fall back to environment variables
        let (api_key, api_secret) = if !config.api_key.is_empty() && !config.api_secret.is_empty() {
            (config.api_key.clone(), config.api_secret.clone())
        } else {
            // Try to load from environment variables
            let _ = dotenvy::dotenv();
            let env_key = std::env::var("COINBASE_API_KEY")
                .or_else(|_| std::env::var("COINBASE_API_KEY_ID"))
                .unwrap_or_default();
            let env_secret = std::env::var("COINBASE_API_SECRET").unwrap_or_default();
            (env_key, env_secret)
        };

        let rest_client = if !api_key.is_empty() && !api_secret.is_empty() {
            Some(CoinbaseRestClient::new(
                api_key,
                api_secret,
                config.sandbox,
            )?)
        } else {
            None
        };

        Ok(Self {
            name: crate::constants::exchange::COINBASE.to_string(),
            config,
            ws_manager_handle: None,
            price_rx: None,
            latest_prices: Arc::new(RwLock::new(HashMap::new())),
            base_url,
            rest_client,
        })
    }

    /// Connect to WebSocket with a specific ticker subscription
    ///
    /// Coinbase requires sending a subscription message after connection:
    /// {"type":"subscribe","product_ids":["SOL-USDC"],"channels":["ticker"]}
    #[tracing::instrument(name = "connect_with_subscription", skip(self), fields(exchange = %self.name, pair = %pair))]
    async fn connect_with_subscription(&mut self, pair: &str) -> Result<()> {
        let product_id = CoinbaseParser::pair_to_product_id(pair);

        // Connect to base WebSocket URL
        let url = self.base_url.clone();
        info!(url = %url, "Connecting to Coinbase WebSocket");

        let (ws_stream, response) = connect_async(&url).await.map_err(|e| {
            error!(url = %url, error = %e, "Connection failed");
            ArbitrageError::NetworkError {
                message: format!("Failed to connect to {}: {}", url, e),
                retry_after: None,
            }
        })?;

        info!(status = %response.status(), "Connected to Coinbase WebSocket");

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

        debug!(subscription = %subscribe_text, "Sending subscription message");
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
                                            warn!(error = %e, "Parse error");
                                        }
                                    }
                                }
                            }
                            Some(Ok(Message::Ping(data))) => {
                                // Respond to server ping with pong
                                if let Err(e) = write.send(Message::Pong(data)).await {
                                    error!(error = %e, "Failed to send pong");
                                    break;
                                }
                            }
                            Some(Ok(Message::Close(_))) => {
                                info!("Server closed connection");
                                break;
                            }
                            Some(Err(e)) => {
                                error!(error = %e, "WebSocket error");
                                break;
                            }
                            None => {
                                info!("Stream ended");
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
                            error!(error = %e, "Failed to send ping");
                            break;
                        }
                    }
                }
            }
        });

        self.ws_manager_handle = Some(handle);

        Ok(())
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

    #[tracing::instrument(name = "subscribe_ticker", skip(self), fields(exchange = %self.name, pair = %pair))]
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

    #[tracing::instrument(name = "get_latest_price", skip(self), fields(exchange = %self.name, pair = %pair))]
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

    #[tracing::instrument(name = "place_order", skip(self, order), fields(
        exchange = %self.name,
        pair = %order.pair,
        side = ?order.side,
        order_type = ?order.order_type,
        quantity = %order.quantity
    ))]
    async fn place_order(
        &mut self,
        order: crate::exchanges::Order,
    ) -> Result<crate::exchanges::OrderResult> {
        match &self.rest_client {
            Some(client) => client.place_market_order(order).await,
            None => Err(ArbitrageError::ExchangeError {
                exchange: self.name.clone(),
                message: "REST API not available - API credentials required".to_string(),
                code: None,
            }),
        }
    }

    #[tracing::instrument(name = "get_balance", skip(self), fields(exchange = %self.name, asset = %asset))]
    async fn get_balance(&self, asset: &str) -> Result<rust_decimal::Decimal> {
        match &self.rest_client {
            Some(client) => client.get_balance(asset).await,
            None => Err(ArbitrageError::ExchangeError {
                exchange: self.name.clone(),
                message: "REST API not available - API credentials required".to_string(),
                code: None,
            }),
        }
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
