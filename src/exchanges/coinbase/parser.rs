//! Coinbase WebSocket message parser
//!
//! Converts Coinbase Advanced Trade WebSocket ticker messages into our common `Price` type.

use crate::error::{ArbitrageError, Result};
use crate::exchanges::Price;
use crate::websocket::MessageParser;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

/// Parser for Coinbase WebSocket ticker messages
///
/// Converts Coinbase's ticker format into our common `Price` type.
#[derive(Debug, Clone)]
pub struct CoinbaseParser;

impl CoinbaseParser {
    /// Create a new Coinbase parser
    pub fn new() -> Self {
        Self
    }

    /// Convert Coinbase product_id format to trading pair
    ///
    /// Example: "SOL-USDC" -> "SOL/USDC"
    pub fn product_id_to_pair(product_id: &str) -> String {
        product_id.replace("-", "/")
    }

    /// Convert trading pair to Coinbase product_id format
    ///
    /// Example: "SOL/USDC" -> "SOL-USDC"
    pub fn pair_to_product_id(pair: &str) -> String {
        pair.replace("/", "-")
    }
}

impl MessageParser for CoinbaseParser {
    type Output = Price;

    fn parse(&self, message: &str) -> Result<Self::Output> {
        let value: serde_json::Value = serde_json::from_str(message).map_err(|e| {
            ArbitrageError::ParseError {
                message: format!("Invalid JSON: {}", e),
                input: Some(message.to_string()),
            }
        })?;

        // Handle error messages
        if value["type"].as_str() == Some("error") {
            let error_msg = value["message"].as_str().unwrap_or("Unknown error");
            return Err(ArbitrageError::ExchangeError {
                exchange: "coinbase".to_string(),
                message: format!("Coinbase WebSocket error: {}", error_msg),
                code: None,
            });
        }

        // Handle subscription confirmation
        if value["type"].as_str() == Some("subscriptions") {
            return Err(ArbitrageError::ParseError {
                message: "Subscription confirmation message (not a ticker)".to_string(),
                input: Some(message.to_string()),
            });
        }

        // Classic Coinbase Exchange WebSocket format (simpler):
        // {
        //   "type": "ticker",
        //   "product_id": "SOL-USD",
        //   "price": "152.31",
        //   "best_bid": "152.28",
        //   "best_ask": "152.32",
        //   "volume_24h": "1124763.89",
        //   "time": "2025-10-30T12:00:00.000000Z"
        // }
        
        // Advanced Trade WebSocket format (nested):
        // {
        //   "channel": "ticker",
        //   "events": [{"type": "snapshot", "tickers": [...]}]
        // }

        let ticker = if value["type"].as_str() == Some("ticker") {
            // Classic Exchange format - message IS the ticker
            &value
        } else if value["channel"].as_str() == Some("ticker") {
            // Advanced Trade format - extract from events
            let events = value["events"]
                .as_array()
                .ok_or_else(|| ArbitrageError::ParseError {
                    message: "Missing or invalid events array".to_string(),
                    input: Some(message.to_string()),
                })?;

            if events.is_empty() {
                return Err(ArbitrageError::ParseError {
                    message: "Events array is empty".to_string(),
                    input: Some(message.to_string()),
                });
            }

            let tickers = events[0]["tickers"]
                .as_array()
                .ok_or_else(|| ArbitrageError::ParseError {
                    message: "Missing or invalid tickers array".to_string(),
                    input: Some(message.to_string()),
                })?;

            if tickers.is_empty() {
                return Err(ArbitrageError::ParseError {
                    message: "Tickers array is empty".to_string(),
                    input: Some(message.to_string()),
                });
            }

            &tickers[0]
        } else {
            return Err(ArbitrageError::ParseError {
                message: format!("Not a ticker message, got type: {}", value["type"].as_str().unwrap_or("unknown")),
                input: Some(message.to_string()),
            });
        };

        // Extract product_id
        let product_id = ticker["product_id"]
            .as_str()
            .ok_or_else(|| ArbitrageError::ParseError {
                message: "Missing product_id".to_string(),
                input: Some(message.to_string()),
            })?;

        // Convert product_id to pair format
        let pair = Self::product_id_to_pair(product_id);

        // Parse prices (Coinbase uses strings for decimal values)
        // Note: field name is "volume_24_h" (with underscore) not "volume_24h"
        let last_str = ticker["price"]
            .as_str()
            .ok_or_else(|| ArbitrageError::ParseError {
                message: "Missing price".to_string(),
                input: Some(message.to_string()),
            })?;

        let bid_str = ticker["best_bid"]
            .as_str()
            .ok_or_else(|| ArbitrageError::ParseError {
                message: "Missing best_bid".to_string(),
                input: Some(message.to_string()),
            })?;

        let ask_str = ticker["best_ask"]
            .as_str()
            .ok_or_else(|| ArbitrageError::ParseError {
                message: "Missing best_ask".to_string(),
                input: Some(message.to_string()),
            })?;

        // Classic Exchange uses "volume_24h", Advanced Trade uses "volume_24_h"
        let volume_str = ticker["volume_24h"]
            .as_str()
            .or_else(|| ticker["volume_24_h"].as_str())
            .unwrap_or("0");

        // Parse decimals
        let last = Decimal::from_str_exact(last_str).map_err(|e| ArbitrageError::ParseError {
            message: format!("Invalid price: {}", e),
            input: Some(message.to_string()),
        })?;

        let bid = Decimal::from_str_exact(bid_str).map_err(|e| ArbitrageError::ParseError {
            message: format!("Invalid best_bid: {}", e),
            input: Some(message.to_string()),
        })?;

        let ask = Decimal::from_str_exact(ask_str).map_err(|e| ArbitrageError::ParseError {
            message: format!("Invalid best_ask: {}", e),
            input: Some(message.to_string()),
        })?;

        let volume =
            Decimal::from_str_exact(volume_str).map_err(|e| ArbitrageError::ParseError {
                message: format!("Invalid volume_24h: {}", e),
                input: Some(message.to_string()),
            })?;

        // Parse timestamp - Classic Exchange uses "time", Advanced Trade uses top-level "timestamp"
        let timestamp = ticker["time"]
            .as_str()
            .or_else(|| value["timestamp"].as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        Ok(Price {
            pair,
            bid,
            ask,
            last,
            volume_24h: volume,
            timestamp,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_id_to_pair() {
        assert_eq!(CoinbaseParser::product_id_to_pair("SOL-USDC"), "SOL/USDC");
        assert_eq!(CoinbaseParser::product_id_to_pair("BTC-USD"), "BTC/USD");
    }

    #[test]
    fn test_pair_to_product_id() {
        assert_eq!(CoinbaseParser::pair_to_product_id("SOL/USDC"), "SOL-USDC");
        assert_eq!(CoinbaseParser::pair_to_product_id("BTC/USD"), "BTC-USD");
    }

    #[test]
    fn test_parse_valid_ticker() {
        let parser = CoinbaseParser::new();

        let ticker_json = r#"{
            "type": "ticker",
            "product_id": "SOL-USDC",
            "price": "143.50",
            "best_bid": "143.48",
            "best_ask": "143.52",
            "volume_24h": "1234567.89",
            "time": "2025-10-30T12:00:00.000000Z"
        }"#;

        let price = parser.parse(ticker_json).unwrap();

        assert_eq!(price.pair, "SOL/USDC");
        assert_eq!(price.last, Decimal::from_str_exact("143.50").unwrap());
        assert_eq!(price.bid, Decimal::from_str_exact("143.48").unwrap());
        assert_eq!(price.ask, Decimal::from_str_exact("143.52").unwrap());
        assert_eq!(
            price.volume_24h,
            Decimal::from_str_exact("1234567.89").unwrap()
        );
    }

    #[test]
    fn test_parse_invalid_message_type() {
        let parser = CoinbaseParser::new();

        let invalid_json = r#"{
            "type": "subscriptions",
            "channels": ["ticker"]
        }"#;

        let result = parser.parse(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_fields() {
        let parser = CoinbaseParser::new();

        let incomplete_json = r#"{
            "type": "ticker"
        }"#;

        let result = parser.parse(incomplete_json);
        assert!(result.is_err());
    }
}

