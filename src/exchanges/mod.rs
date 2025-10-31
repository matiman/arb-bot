pub mod factory;
pub mod types;

pub use factory::{DefaultExchangeFactory, ExchangeFactory};
pub use types::{Order, OrderResult, OrderSide, OrderStatus, OrderType, Price};

use crate::error::Result;
use async_trait::async_trait;

/// Trait abstraction for cryptocurrency exchange interactions.
///
/// This trait provides a unified interface for interacting with different exchanges
/// (Binance, Coinbase, etc.), enabling the arbitrage bot to work with any exchange
/// implementation without changing the core trading logic.
///
/// # Business Logic Overview
///
/// The Exchange trait represents the core operations needed for arbitrage trading:
///
/// 1. **Price Monitoring**: Subscribe to and retrieve latest prices for trading pairs
/// 2. **Order Execution**: Place buy/sell orders to execute arbitrage opportunities
/// 3. **Balance Management**: Check available funds before trading
/// 4. **Connection Management**: Establish and maintain WebSocket connections
///
/// # Example Usage
///
/// ```no_run
/// use arb_bot::exchanges::Exchange;
/// ```
#[async_trait]
#[allow(clippy::result_large_err)]
pub trait Exchange: Send + Sync {
    /// Connect to the exchange WebSocket
    async fn connect(&mut self) -> Result<()>;

    /// Subscribe to ticker updates for a trading pair
    async fn subscribe_ticker(&mut self, pair: &str) -> Result<()>;

    /// Get the latest price for a pair
    async fn get_latest_price(&self, pair: &str) -> Result<Price>;

    /// Place a market order
    async fn place_order(&mut self, order: Order) -> Result<OrderResult>;

    /// Get account balance for an asset
    async fn get_balance(&self, asset: &str) -> Result<rust_decimal::Decimal>;

    /// Get exchange name
    fn name(&self) -> &str;

    /// Check if connected
    fn is_connected(&self) -> bool;

    /// Disconnect from exchange
    async fn disconnect(&mut self) -> Result<()>;
}
