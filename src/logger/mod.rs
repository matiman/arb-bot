//! Structured Logging Module
//!
//! Provides structured logging using the `tracing` crate with support for:
//! - Multiple formats (JSON, Pretty, Compact)
//! - File logging with rotation
//! - Log level filtering
//! - Module filtering
//! - Structured fields

mod config;

pub use config::{LogFormat, LoggerConfig};
pub use tracing::{debug, error, info, trace, warn};

use crate::error::ArbitrageError;
use rust_decimal::Decimal;

/// Log a price update with structured fields
pub fn log_price_update(exchange: &str, pair: &str, price: Decimal) {
    info!(
        exchange = %exchange,
        pair = %pair,
        price = %price,
        "Price update received"
    );
}

/// Log an arbitrage opportunity with structured fields
pub fn log_arbitrage_opportunity(
    buy_exchange: &str,
    sell_exchange: &str,
    pair: &str,
    spread_pct: Decimal,
) {
    info!(
        buy_exchange = %buy_exchange,
        sell_exchange = %sell_exchange,
        pair = %pair,
        spread_pct = %spread_pct,
        "Arbitrage opportunity detected"
    );
}

/// Log an order placement with structured fields
pub fn log_order_placed(exchange: &str, order_id: &str, side: &str, quantity: Decimal) {
    info!(
        exchange = %exchange,
        order_id = %order_id,
        side = %side,
        quantity = %quantity,
        "Order placed"
    );
}

/// Log an error with context
pub fn log_error(context: &str, error: &ArbitrageError) {
    error!(
        context = %context,
        error = %error,
        "Error occurred"
    );
}
