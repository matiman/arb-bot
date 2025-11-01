//! Message parser trait for converting exchange-specific messages to common types

use crate::error::Result;

/// Trait for parsing exchange-specific WebSocket messages into common types
///
/// Each exchange (Binance, Coinbase) will implement this trait to convert
/// their specific JSON format into our common `Price` type.
///
/// # Example
///
/// ```rust,no_run
/// use arb_bot::websocket::MessageParser;
/// use arb_bot::exchanges::Price;
/// use arb_bot::error::Result;
///
/// #[derive(Clone)]
/// struct MyParser;
///
/// impl MessageParser for MyParser {
///     type Output = Price;
///
///     fn parse(&self, message: &str) -> Result<Self::Output> {
///         // Parse message into Price struct
///         # todo!()
///     }
/// }
/// ```
pub trait MessageParser: Send + Sync + Clone {
    /// The type this parser produces (typically `Price`)
    type Output: Send + Clone;

    /// Parse a WebSocket message string into the output type
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if the message format is invalid or missing required fields
    fn parse(&self, message: &str) -> Result<Self::Output>;
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

    #[test]
    fn test_parser_trait_basic() {
        let parser = TestParser;
        let message = r#"{"pair":"SOL/USDC","bid":"100.5","ask":"101.0"}"#;
        let price = parser.parse(message).unwrap();
        assert_eq!(price.pair, "SOL/USDC");
    }

    #[test]
    fn test_parser_clone() {
        let parser = TestParser;
        let cloned = parser.clone();
        let message = r#"{"pair":"BTC/USD","bid":"50000","ask":"50001"}"#;
        let price1 = parser.parse(message).unwrap();
        let price2 = cloned.parse(message).unwrap();
        assert_eq!(price1.pair, price2.pair);
    }

    //TODO Parser should fail if the message is invalid
}
