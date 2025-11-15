//! Binance Exchange WebSocket Implementation
//!
//! Connects to Binance WebSocket stream to receive real-time price updates.

use crate::config::BinanceConfig;
use crate::error::{ArbitrageError, Result};
use crate::exchanges::{Exchange, Price};
use crate::websocket::{ReconnectionStrategy, WebSocketManager};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;

use super::parser::BinanceParser;

/// Binance exchange implementation using WebSocket for price feeds
///
/// # Business Logic
///
/// Connects to Binance WebSocket stream to receive real-time ticker updates.
/// Prices are stored in-memory and can be queried via `get_latest_price()`.
///
/// **WebSocket-only**: This implementation focuses on price feeds only.
/// REST API for trading will be added later.
pub struct BinanceExchange {
    name: String,
    #[allow(dead_code)] // Kept for future use (testnet flag, API credentials)
    config: BinanceConfig,
    /// WebSocket manager (moved into spawned task on connect)
    ws_manager_handle: Option<tokio::task::JoinHandle<()>>,
    /// Receiver for price updates from WebSocket
    price_rx: Option<broadcast::Receiver<Price>>,
    /// In-memory store of latest prices by trading pair
    latest_prices: Arc<RwLock<HashMap<String, Price>>>,
    /// Base WebSocket URL (without subscription)
    base_url: String,
}

impl BinanceExchange {
    /// Create a new Binance exchange instance
    pub fn new(config: BinanceConfig) -> Result<Self> {
        // Binance.US for US customers, Binance.com for international
        // Note: Binance.com is geo-restricted (HTTP 451) in US
        // TODO Change to use environment variables
        let base_url = if config.testnet {
            crate::constants::websocket::BINANCE_TESTNET.to_string()
        } else {
            // Binance.US WebSocket endpoint
            // Format: wss://stream.binance.us:9443/ws or wss://stream.binance.us/ws
            // Try with port 9443 first (matches Binance.com format)
            crate::constants::websocket::BINANCE_US_PRODUCTION.to_string()
        };

        Ok(Self {
            name: crate::constants::exchange::BINANCE.to_string(),
            config,
            ws_manager_handle: None,
            price_rx: None,
            latest_prices: Arc::new(RwLock::new(HashMap::new())),
            base_url,
        })
    }

    /// Connect to WebSocket with a specific ticker subscription
    ///
    /// Binance supports subscribing via URL parameter:
    /// Production: `wss://stream.binance.com:9443/ws/<symbol>@ticker` OR `wss://stream.binance.com:9443/stream?streams=<symbol>@ticker`
    /// Testnet: `wss://testnet.binance.vision/ws/<symbol>@ticker`
    async fn connect_with_subscription(&mut self, pair: &str) -> Result<()> {
        let symbol = BinanceParser::pair_to_symbol(pair);

        // Use the base_url configured (already set to Binance.US or Binance.com)
        // Format: wss://stream.binance.us/ws/<symbol>@ticker
        let url = format!("{}/{}@ticker", self.base_url, symbol);

        let parser = BinanceParser::new();
        let reconnect_strategy = ReconnectionStrategy::exponential_backoff();

        // Create WebSocket manager with subscription URL
        let (mut manager, price_rx) = WebSocketManager::new(url, parser, reconnect_strategy);

        // Store receiver
        self.price_rx = Some(price_rx);

        // Spawn background task to run WebSocket manager
        let handle = tokio::spawn(async move {
            if let Err(e) = manager.run().await {
                eprintln!("Binance WebSocket manager error: {}", e);
            }
        });

        self.ws_manager_handle = Some(handle);

        // Spawn background task to update latest prices from WebSocket stream
        if let Some(mut rx) = self.price_rx.take() {
            let prices = self.latest_prices.clone();
            tokio::spawn(async move {
                loop {
                    match rx.recv().await {
                        Ok(price) => {
                            // Silently cache price updates (no verbose logging)
                            prices.write().insert(price.pair.clone(), price);
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                            eprintln!("⚠️ Lagged {} messages", skipped);
                            continue;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            eprintln!("❌ Broadcast channel closed");
                            break;
                        }
                    }
                }
            });
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Exchange for BinanceExchange {
    async fn connect(&mut self) -> Result<()> {
        // Initial connection without subscription
        // Subscription will be done in subscribe_ticker()
        // For now, just initialize - actual connection happens on subscribe
        Ok(())
    }

    async fn subscribe_ticker(&mut self, pair: &str) -> Result<()> {
        // Disconnect existing connection if any
        self.disconnect().await.ok();

        // Connect with subscription URL
        // Binance format: wss://stream.binance.com:9443/ws/solusdc@ticker
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
