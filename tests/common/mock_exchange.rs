//! Mock exchange implementation for testing only.
//!
//! This is NOT included in production builds - it lives in the tests/ directory.

use arb_bot::error::{ArbitrageError, Result};
use arb_bot::exchanges::{Exchange, Order, OrderResult, OrderStatus, Price};
use async_trait::async_trait;
use parking_lot::RwLock;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Mock exchange for testing - NOT available in production.
///
/// Simulates exchange behavior for integration tests without real API calls.
pub struct MockExchange {
    name: String,
    connected: Arc<RwLock<bool>>,
    prices: Arc<RwLock<HashMap<String, Price>>>,
    balances: Arc<RwLock<HashMap<String, Decimal>>>,
    subscriptions: Arc<RwLock<Vec<String>>>,
}

impl MockExchange {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            connected: Arc::new(RwLock::new(false)),
            prices: Arc::new(RwLock::new(HashMap::new())),
            balances: Arc::new(RwLock::new(HashMap::new())),
            subscriptions: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn set_price(&self, pair: &str, price: Price) {
        self.prices.write().insert(pair.to_string(), price);
    }

    pub fn set_balance(&self, asset: &str, amount: Decimal) {
        self.balances.write().insert(asset.to_string(), amount);
    }
}

#[async_trait]
#[allow(clippy::result_large_err)]
impl Exchange for MockExchange {
    async fn connect(&mut self) -> Result<()> {
        *self.connected.write() = true;
        Ok(())
    }

    async fn subscribe_ticker(&mut self, pair: &str) -> Result<()> {
        if !*self.connected.read() {
            return Err(ArbitrageError::NetworkError {
                message: "Not connected".to_string(),
                retry_after: None,
            });
        }

        self.subscriptions.write().push(pair.to_string());
        Ok(())
    }

    async fn get_latest_price(&self, pair: &str) -> Result<Price> {
        if !*self.connected.read() {
            return Err(ArbitrageError::NetworkError {
                message: "Not connected".to_string(),
                retry_after: None,
            });
        }

        self.prices
            .read()
            .get(pair)
            .cloned()
            .ok_or_else(|| ArbitrageError::ParseError {
                message: format!("Price not found for pair: {}", pair),
                input: None,
            })
    }

    async fn place_order(&mut self, order: Order) -> Result<OrderResult> {
        if !*self.connected.read() {
            return Err(ArbitrageError::NetworkError {
                message: "Not connected".to_string(),
                retry_after: None,
            });
        }

        // Generate a mock order ID
        let order_id = format!(
            "mock_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );

        Ok(OrderResult {
            order_id,
            status: OrderStatus::Filled,
            filled_quantity: order.quantity,
            average_price: self.prices.read().get(&order.pair).map(|p| p.mid_price()),
            fee: Decimal::from(1),
            fee_asset: "USDC".to_string(),
            timestamp: chrono::Utc::now(),
        })
    }

    async fn get_balance(&self, asset: &str) -> Result<Decimal> {
        if !*self.connected.read() {
            return Err(ArbitrageError::NetworkError {
                message: "Not connected".to_string(),
                retry_after: None,
            });
        }

        self.balances.read().get(asset).copied().ok_or_else(|| {
            ArbitrageError::InsufficientBalance {
                exchange: self.name.clone(),
                asset: asset.to_string(),
                required: "0".to_string(),
                available: "0".to_string(),
            }
        })
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_connected(&self) -> bool {
        *self.connected.read()
    }

    async fn disconnect(&mut self) -> Result<()> {
        *self.connected.write() = false;
        Ok(())
    }
}
