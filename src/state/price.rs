//! Price State Manager
//!
//! Thread-safe shared state for storing latest prices from multiple exchanges.
//! Provides staleness detection and spread calculation between exchanges.

use super::types::{ExchangeId, PriceData};
use crate::exchanges::Price;
use parking_lot::RwLock;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Thread-safe price state manager for tracking prices across exchanges
///
/// # Business Logic
///
/// This manager stores the latest price for each (exchange, trading pair) combination.
/// When multiple exchanges update prices concurrently (e.g., Binance and Coinbase),
/// this manager ensures thread-safe access without blocking reads unnecessarily.
///
/// **Staleness Detection**: Prices older than `max_age` are considered stale and
/// rejected from spread calculations. This prevents trading on outdated data.
///
/// **Max Time Difference**: When comparing prices between exchanges, prices captured
/// more than `max_age / 2` apart are rejected. This ensures we only compare prices
/// from similar time windows, avoiding false arbitrage opportunities.
///
/// # Example
///
/// ```rust
/// use arb_bot::state::{PriceState, ExchangeId};
/// use arb_bot::exchanges::Price;
/// use chrono::Utc;
/// use rust_decimal::Decimal;
/// use std::time::Duration;
///
/// let state = PriceState::new(Duration::from_secs(5));
///
/// // Binance WebSocket updates price
/// let binance_price = Price {
///     pair: "SOL/USDC".to_string(),
///     bid: Decimal::from(100),
///     ask: Decimal::from(101),
///     last: Decimal::from(100),
///     volume_24h: Decimal::ZERO,
///     timestamp: Utc::now(),
/// };
/// state.update_price(ExchangeId::Binance, "SOL/USDC", binance_price, 1);
///
/// // Coinbase WebSocket updates price
/// let coinbase_price = Price {
///     pair: "SOL/USDC".to_string(),
///     bid: Decimal::from(102),
///     ask: Decimal::from(103),
///     last: Decimal::from(102),
///     volume_24h: Decimal::ZERO,
///     timestamp: Utc::now(),
/// };
/// state.update_price(ExchangeId::Coinbase, "SOL/USDC", coinbase_price, 1);
///
/// // Arbitrage detector reads spread
/// let spread = state.get_spread(ExchangeId::Binance, ExchangeId::Coinbase, "SOL/USDC");
/// assert!(spread.is_some());
/// ```
#[derive(Clone)]
pub struct PriceState {
    /// Thread-safe HashMap storing (ExchangeId, pair) -> PriceData
    prices: Arc<RwLock<HashMap<(ExchangeId, String), PriceData>>>,
    /// Maximum age before a price is considered stale
    max_age: Duration,
}

impl PriceState {
    /// Creates a new PriceState with the specified maximum age for staleness detection
    ///
    /// `max_age` determines:
    /// - How old a price can be before it's considered stale
    /// - Max time difference between prices for comparison = `max_age / 2`
    pub fn new(max_age: Duration) -> Self {
        Self {
            prices: Arc::new(RwLock::new(HashMap::new())),
            max_age,
        }
    }

    /// Updates the price for a given exchange and trading pair
    ///
    /// This is called by WebSocket managers when new price data arrives.
    /// Overwrites any existing price for the same (exchange, pair) key.
    pub fn update_price(&self, exchange: ExchangeId, pair: &str, price: Price, sequence: u64) {
        let key = (exchange, pair.to_string());
        let price_data = PriceData::new(price, sequence);
        self.prices.write().insert(key, price_data);
    }

    /// Retrieves the latest price for a given exchange and trading pair
    ///
    /// Returns `None` if no price exists for the given (exchange, pair) combination.
    pub fn get_price(&self, exchange: ExchangeId, pair: &str) -> Option<PriceData> {
        let key = (exchange, pair.to_string());
        self.prices.read().get(&key).cloned()
    }

    /// Calculates the absolute spread between two exchanges for a trading pair
    ///
    /// Returns `None` if:
    /// - Either price is missing
    /// - Either price is stale (> max_age)
    /// - Prices were captured too far apart (> max_age / 2)
    ///
    /// Spread = |mid_price2 - mid_price1|
    pub fn get_spread(&self, ex1: ExchangeId, ex2: ExchangeId, pair: &str) -> Option<Decimal> {
        let price1 = self.get_price(ex1, pair)?;
        let price2 = self.get_price(ex2, pair)?;

        // Check staleness - reject if either price is too old
        if price1.is_stale(self.max_age) || price2.is_stale(self.max_age) {
            return None;
        }

        // Check max time difference - reject if prices captured too far apart
        // This ensures we compare prices from similar time windows
        let time_diff = if price1.timestamp > price2.timestamp {
            price1.timestamp.duration_since(price2.timestamp)
        } else {
            price2.timestamp.duration_since(price1.timestamp)
        };

        // Max time difference: half of max_age (e.g., 2.5s if max_age is 5s)
        // This ensures prices are from similar time windows
        let max_time_diff = self.max_age / 2;
        if time_diff > max_time_diff {
            return None;
        }

        let mid1 = price1.price.mid_price();
        let mid2 = price2.price.mid_price();

        Some((mid2 - mid1).abs())
    }

    /// Calculates the spread percentage between two exchanges
    ///
    /// Returns `None` if spread calculation fails (same conditions as `get_spread`).
    ///
    /// Spread % = (spread / mid_price1) * 100
    pub fn get_spread_percentage(
        &self,
        ex1: ExchangeId,
        ex2: ExchangeId,
        pair: &str,
    ) -> Option<Decimal> {
        let spread = self.get_spread(ex1, ex2, pair)?;
        let price1 = self.get_price(ex1, pair)?;
        let mid1 = price1.price.mid_price();

        if mid1.is_zero() {
            return None;
        }

        Some((spread / mid1) * Decimal::from(100))
    }

    /// Checks if a price for the given exchange and pair is stale
    ///
    /// Returns `false` if the price doesn't exist.
    pub fn is_stale(&self, exchange: ExchangeId, pair: &str) -> bool {
        if let Some(price_data) = self.get_price(exchange, pair) {
            price_data.is_stale(self.max_age)
        } else {
            false // Missing price is not considered stale (it doesn't exist)
        }
    }

    /// Removes all stale prices from the state
    ///
    /// Returns the number of prices removed.
    pub fn remove_stale_prices(&self) -> usize {
        let mut prices = self.prices.write();
        let initial_count = prices.len();

        prices.retain(|_, data| !data.is_stale(self.max_age));

        initial_count - prices.len()
    }

    /// Returns a clone of all current prices
    ///
    /// Useful for debugging and monitoring. Consider using iterators for
    /// large datasets to avoid cloning.
    pub fn get_all_prices(&self) -> HashMap<(ExchangeId, String), PriceData> {
        self.prices.read().clone()
    }

    /// Clears all prices from the state
    pub fn clear(&self) {
        self.prices.write().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exchanges::Price;
    use chrono::Utc;
    use rust_decimal::Decimal;

    #[test]
    fn test_price_state_new() {
        let state = PriceState::new(Duration::from_secs(5));
        let all_prices = state.get_all_prices();
        assert!(all_prices.is_empty());
    }

    #[test]
    fn test_update_and_get_price() {
        let state = PriceState::new(Duration::from_secs(5));

        let price = Price {
            pair: "SOL/USDC".to_string(),
            bid: Decimal::from(100),
            ask: Decimal::from(101),
            last: Decimal::from(100),
            volume_24h: Decimal::from(1000000),
            timestamp: Utc::now(),
        };

        state.update_price(ExchangeId::Binance, "SOL/USDC", price.clone(), 1);

        let retrieved = state.get_price(ExchangeId::Binance, "SOL/USDC");
        assert!(retrieved.is_some());
        let price_data = retrieved.unwrap();
        assert_eq!(price_data.price.pair, "SOL/USDC");
        assert_eq!(price_data.price.bid, Decimal::from(100));
        assert_eq!(price_data.sequence, 1);
    }

    #[test]
    fn test_spread_calculation() {
        let state = PriceState::new(Duration::from_secs(5));

        let binance_price = Price {
            pair: "SOL/USDC".to_string(),
            bid: Decimal::from(100),
            ask: Decimal::from(101),
            last: Decimal::from(100),
            volume_24h: Decimal::ZERO,
            timestamp: Utc::now(),
        };

        let coinbase_price = Price {
            pair: "SOL/USDC".to_string(),
            bid: Decimal::from(102),
            ask: Decimal::from(103),
            last: Decimal::from(102),
            volume_24h: Decimal::ZERO,
            timestamp: Utc::now(),
        };

        state.update_price(ExchangeId::Binance, "SOL/USDC", binance_price, 1);
        state.update_price(ExchangeId::Coinbase, "SOL/USDC", coinbase_price, 1);

        let spread = state.get_spread(ExchangeId::Binance, ExchangeId::Coinbase, "SOL/USDC");
        assert!(spread.is_some());
        // Binance mid: 100.5, Coinbase mid: 102.5, spread: 2.0
        assert_eq!(spread.unwrap(), Decimal::from(2));
    }

    #[test]
    fn test_spread_missing_price() {
        let state = PriceState::new(Duration::from_secs(5));

        state.update_price(
            ExchangeId::Binance,
            "SOL/USDC",
            Price {
                pair: "SOL/USDC".to_string(),
                bid: Decimal::from(100),
                ask: Decimal::from(101),
                last: Decimal::from(100),
                volume_24h: Decimal::ZERO,
                timestamp: Utc::now(),
            },
            1,
        );

        let spread = state.get_spread(ExchangeId::Binance, ExchangeId::Coinbase, "SOL/USDC");
        assert!(spread.is_none());
    }

    #[test]
    fn test_clear() {
        let state = PriceState::new(Duration::from_secs(5));

        state.update_price(
            ExchangeId::Binance,
            "SOL/USDC",
            Price {
                pair: "SOL/USDC".to_string(),
                bid: Decimal::from(100),
                ask: Decimal::from(101),
                last: Decimal::from(100),
                volume_24h: Decimal::ZERO,
                timestamp: Utc::now(),
            },
            1,
        );

        state.clear();
        assert!(state.get_all_prices().is_empty());
    }
}
