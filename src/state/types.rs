//! Common types for price state management

use crate::exchanges::Price;
use std::time::{Duration, Instant};

/// Identifies an exchange for price tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExchangeId {
    Binance,
    Coinbase,
    // Future exchanges can be added here
}

impl ExchangeId {
    /// Returns the exchange name as a string
    pub fn name(&self) -> &'static str {
        match self {
            ExchangeId::Binance => "Binance",
            ExchangeId::Coinbase => "Coinbase",
        }
    }
}

/// Stores price data with metadata for staleness detection
#[derive(Debug, Clone)]
pub struct PriceData {
    /// The price information (bid, ask, last, etc.)
    pub price: Price,
    /// When this price was received/captured
    pub timestamp: Instant,
    /// Sequence number from the exchange (for deduplication/ordering)
    pub sequence: u64,
}

impl PriceData {
    /// Creates a new PriceData with the current timestamp
    pub fn new(price: Price, sequence: u64) -> Self {
        Self {
            price,
            timestamp: Instant::now(),
            sequence,
        }
    }

    /// Returns the age of this price data
    pub fn age(&self) -> Duration {
        self.timestamp.elapsed()
    }

    /// Checks if this price data is stale (older than max_age)
    pub fn is_stale(&self, max_age: Duration) -> bool {
        self.age() > max_age
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exchanges::Price;
    use chrono::Utc;
    use rust_decimal::Decimal;
    use std::thread;
    use std::time::Duration as StdDuration;

    #[test]
    fn test_exchange_id_name() {
        assert_eq!(ExchangeId::Binance.name(), "Binance");
        assert_eq!(ExchangeId::Coinbase.name(), "Coinbase");
    }

    #[test]
    fn test_exchange_id_hash_eq() {
        let id1 = ExchangeId::Binance;
        let id2 = ExchangeId::Binance;
        let id3 = ExchangeId::Coinbase;

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);

        // Verify it can be used in HashMap
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(id1, "value1");
        assert_eq!(map.get(&id2), Some(&"value1"));
    }

    #[test]
    fn test_price_data_new() {
        let price = Price {
            pair: "SOL/USDC".to_string(),
            bid: Decimal::from(100),
            ask: Decimal::from(101),
            last: Decimal::from(100),
            volume_24h: Decimal::ZERO,
            timestamp: Utc::now(),
        };

        let price_data = PriceData::new(price.clone(), 1);
        assert_eq!(price_data.price.pair, "SOL/USDC");
        assert_eq!(price_data.sequence, 1);
        assert!(price_data.age() < Duration::from_millis(100)); // Should be very recent
    }

    #[test]
    fn test_price_data_age() {
        let price = Price {
            pair: "SOL/USDC".to_string(),
            bid: Decimal::from(100),
            ask: Decimal::from(101),
            last: Decimal::from(100),
            volume_24h: Decimal::ZERO,
            timestamp: Utc::now(),
        };

        let price_data = PriceData::new(price, 1);
        let initial_age = price_data.age();

        thread::sleep(StdDuration::from_millis(100));
        let new_age = price_data.age();

        assert!(new_age > initial_age);
        assert!(new_age >= StdDuration::from_millis(100));
    }

    #[test]
    fn test_price_data_is_stale() {
        let price = Price {
            pair: "SOL/USDC".to_string(),
            bid: Decimal::from(100),
            ask: Decimal::from(101),
            last: Decimal::from(100),
            volume_24h: Decimal::ZERO,
            timestamp: Utc::now(),
        };

        let price_data = PriceData::new(price, 1);
        assert!(!price_data.is_stale(Duration::from_secs(5))); // Fresh

        // Note: This test relies on time, so we test the logic rather than exact timing
        // In integration tests, we'll verify staleness with controlled timing
    }
}
