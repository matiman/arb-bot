use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

/// Represents current market price data from an exchange.
///
/// Used for arbitrage detection by comparing prices across exchanges.
/// Contains bid (sell price), ask (buy price), and spread information.
///
/// # Business Logic
///
/// - **Bid**: Best price you can sell at (buyers' highest offer)
/// - **Ask**: Best price you can buy at (sellers' lowest ask)
/// - **Spread**: Difference between ask and bid (trading cost)
/// - **Spread Percentage**: Used to determine if arbitrage is profitable
///
/// # Example
///
/// ```
/// use arb_bot::exchanges::Price;
/// use rust_decimal::Decimal;
/// use chrono::Utc;
///
/// let price = Price {
///     pair: "SOL/USDC".to_string(),
///     bid: Decimal::from(100),
///     ask: Decimal::from(101),
///     last: Decimal::from(100),
///     volume_24h: Decimal::from(1000000),
///     timestamp: Utc::now(),
/// };
///
/// let spread_pct = price.spread_percentage(); // ~1%
/// if spread_pct > rust_decimal::Decimal::from(20) / rust_decimal::Decimal::from(100) {
///     // Profitable arbitrage opportunity
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Price {
    pub pair: String,
    //Use Decimal to avoid floating point precision issues and rounding errors
    pub bid: Decimal,
    pub ask: Decimal,
    pub last: Decimal,
    pub volume_24h: Decimal,
    pub timestamp: DateTime<Utc>,
}

impl Price {
    /// Calculate mid price (average of bid and ask).
    ///
    /// Represents fair market price estimate for spread calculations.
    pub fn mid_price(&self) -> Decimal {
        (self.bid + self.ask) / Decimal::from(2)
    }

    /// Calculate bid-ask spread (difference between ask and bid).
    ///
    /// This represents the trading cost - larger spread means less profit potential.
    pub fn spread(&self) -> Decimal {
        self.ask - self.bid
    }

    /// Calculate spread as percentage of mid price.
    ///
    /// Used by arbitrage detection - typically need >0.2% to be profitable after fees.
    pub fn spread_percentage(&self) -> Decimal {
        let mid = self.mid_price();
        if mid.is_zero() {
            Decimal::ZERO
        } else {
            (self.spread() / mid) * Decimal::from(100)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrderType {
    Market,
    Limit { price: Decimal },
}

/// Represents a trade instruction sent to an exchange.
///
/// Contains trading pair, side (buy/sell), order type (market/limit), and quantity.
/// Used by `Exchange::place_order()` to execute trades.
///
/// # Example
///
/// ```
/// use arb_bot::exchanges::Order;
/// use rust_decimal::Decimal;
///
/// // Buy 10 SOL/USDC at market price
/// let buy_order = Order::market_buy("SOL/USDC", Decimal::from(10));
///
/// // Sell 10 SOL/USDC at market price
/// let sell_order = Order::market_sell("SOL/USDC", Decimal::from(10));
/// ```
#[derive(Debug, Clone)]
pub struct Order {
    pub pair: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: Decimal,
}

impl Order {
    pub fn market_buy(pair: impl Into<String>, quantity: Decimal) -> Self {
        Self {
            pair: pair.into(),
            side: OrderSide::Buy,
            order_type: OrderType::Market,
            quantity,
        }
    }

    pub fn market_sell(pair: impl Into<String>, quantity: Decimal) -> Self {
        Self {
            pair: pair.into(),
            side: OrderSide::Sell,
            order_type: OrderType::Market,
            quantity,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrderStatus {
    Pending,
    Filled,
    PartiallyFilled,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone)]
pub struct OrderResult {
    pub order_id: String,
    pub status: OrderStatus,
    pub filled_quantity: Decimal,
    pub average_price: Option<Decimal>,
    pub fee: Decimal,
    //the currency used to pay the fee
    pub fee_asset: String,
    pub timestamp: DateTime<Utc>,
}

impl OrderResult {
    pub fn is_complete(&self) -> bool {
        matches!(
            self.status,
            OrderStatus::Filled | OrderStatus::Cancelled | OrderStatus::Failed
        )
    }

    pub fn total_cost(&self) -> Option<Decimal> {
        self.average_price
            .map(|price| price * self.filled_quantity + self.fee)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn price_mid_price() {
        let price = Price {
            pair: "SOL/USDC".to_string(),
            bid: Decimal::from(100),
            ask: Decimal::from(102),
            last: Decimal::from(101),
            volume_24h: Decimal::from(1000000),
            timestamp: Utc::now(),
        };

        assert_eq!(price.mid_price(), Decimal::from(101));
    }

    #[test]
    fn price_spread() {
        let price = Price {
            pair: "SOL/USDC".to_string(),
            bid: Decimal::from(100),
            ask: Decimal::from(102),
            last: Decimal::from(101),
            volume_24h: Decimal::from(1000000),
            timestamp: Utc::now(),
        };

        assert_eq!(price.spread(), Decimal::from(2));
    }

    #[test]
    fn price_spread_percentage() {
        let price = Price {
            pair: "SOL/USDC".to_string(),
            bid: Decimal::from(100),
            ask: Decimal::from(102),
            last: Decimal::from(101),
            volume_24h: Decimal::from(1000000),
            timestamp: Utc::now(),
        };

        let spread_pct = price.spread_percentage();
        // Should be approximately 2% (2/101 * 100)
        assert!(spread_pct > Decimal::from(1) && spread_pct < Decimal::from(3));
    }

    #[test]
    fn price_spread_percentage_zero_mid() {
        let price = Price {
            pair: "SOL/USDC".to_string(),
            bid: Decimal::ZERO,
            ask: Decimal::ZERO,
            last: Decimal::ZERO,
            volume_24h: Decimal::ZERO,
            timestamp: Utc::now(),
        };

        assert_eq!(price.spread_percentage(), Decimal::ZERO);
    }

    #[test]
    fn order_market_buy() {
        let order = Order::market_buy("SOL/USDC", Decimal::from(10));
        assert_eq!(order.pair, "SOL/USDC");
        assert_eq!(order.side, OrderSide::Buy);
        assert_eq!(order.order_type, OrderType::Market);
        assert_eq!(order.quantity, Decimal::from(10));
    }

    #[test]
    fn order_market_sell() {
        let order = Order::market_sell("SOL/USDC", Decimal::from(5));
        assert_eq!(order.pair, "SOL/USDC");
        assert_eq!(order.side, OrderSide::Sell);
        assert_eq!(order.order_type, OrderType::Market);
        assert_eq!(order.quantity, Decimal::from(5));
    }

    #[test]
    fn order_result_is_complete() {
        let filled = OrderResult {
            order_id: "123".to_string(),
            status: OrderStatus::Filled,
            filled_quantity: Decimal::from(10),
            average_price: Some(Decimal::from(100)),
            fee: Decimal::from(1),
            fee_asset: "USDC".to_string(),
            timestamp: Utc::now(),
        };
        assert!(filled.is_complete());

        let pending = OrderResult {
            order_id: "124".to_string(),
            status: OrderStatus::Pending,
            filled_quantity: Decimal::from(10),
            average_price: Some(Decimal::from(100)),
            fee: Decimal::from(1),
            fee_asset: "USDC".to_string(),
            timestamp: Utc::now(),
        };
        assert!(!pending.is_complete());
    }

    #[test]
    fn order_result_total_cost() {
        let result = OrderResult {
            order_id: "123".to_string(),
            status: OrderStatus::Filled,
            filled_quantity: Decimal::from(10),
            average_price: Some(Decimal::from(100)),
            fee: Decimal::from(1),
            fee_asset: "USDC".to_string(),
            timestamp: Utc::now(),
        };

        let total = result.total_cost().unwrap();
        // Should be 10 * 100 + 1 = 1001
        assert_eq!(total, Decimal::from(1001));
    }
}
