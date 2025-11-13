//! Binance WebSocket message parser

use crate::error::{ArbitrageError, Result};
use crate::exchanges::Price;
use crate::websocket::MessageParser;
use chrono::Utc;
use rust_decimal::Decimal;

/// Parser for Binance WebSocket ticker messages
///
/// Converts Binance's 24hrTicker format into our common `Price` type.
#[derive(Debug, Clone)]
pub struct BinanceParser;

impl BinanceParser {
    /// Create a new Binance parser
    pub fn new() -> Self {
        Self
    }

    /// Convert Binance symbol format to trading pair
    ///
    /// Example: "SOLUSDC" -> "SOL/USDC"
    /// Note: This is a heuristic - Binance symbols vary in length
    pub fn symbol_to_pair(symbol: &str) -> String {
        // Binance symbols are typically 6-12 chars (e.g., BTCUSDT, SOLUSDC)
        // For simplicity, assume format: BASEQUOTE where BASE is first 3-4 chars
        // This is a heuristic - real implementation might need a symbol mapping

        // Try common patterns: SOLUSDC (6 chars = 3+3), BTCUSDT (8 chars = 3+5)
        if symbol.len() >= 6 {
            // Simple split: assume first half is base, second half is quote
            // For SOLUSDC: SOL = 3, USDC = 3
            // For BTCUSDT: BTC = 3, USDT = 4
            // This is approximate - real code should use a symbol table
            let mid = symbol.len() / 2;
            format!("{}/{}", &symbol[..mid], &symbol[mid..])
        } else {
            // Fallback: can't determine split
            format!("UNKNOWN/{}", symbol)
        }
    }

    /// Convert trading pair to Binance symbol format
    ///
    /// Example: "SOL/USDC" -> "SOLUSDC" (Binance uses UPPERCASE)
    pub fn pair_to_symbol(pair: &str) -> String {
        pair.replace("/", "").to_uppercase()
    }
}

impl MessageParser for BinanceParser {
    type Output = Price;

    fn parse(&self, message: &str) -> Result<Self::Output> {
        // Debug: print first 200 chars of message
        let preview = if message.len() > 200 {
            format!("{}...", &message[..200])
        } else {
            message.to_string()
        };
        println!(
            "🔍 BinanceParser: Received message ({} chars): {}",
            message.len(),
            preview
        );

        let value: serde_json::Value = serde_json::from_str(message).map_err(|e| {
            eprintln!(
                "❌ BinanceParser: JSON parse error: {} (message: {})",
                e, preview
            );
            ArbitrageError::ParseError {
                message: format!("Invalid JSON: {}", e),
                input: Some(message.to_string()),
            }
        })?;

        // Binance ticker format:
        // {
        //   "e": "24hrTicker",
        //   "s": "SOLUSDC",
        //   "c": "143.50",  // Close price (last)
        //   "b": "143.48",  // Best bid
        //   "a": "143.52",  // Best ask
        //   "v": "1234567.89"  // Volume
        // }

        // Check event type
        let event_type = value["e"]
            .as_str()
            .ok_or_else(|| ArbitrageError::ParseError {
                message: "Missing or invalid event type 'e'".to_string(),
                input: Some(message.to_string()),
            })?;

        if event_type != "24hrTicker" {
            return Err(ArbitrageError::ParseError {
                message: format!("Not a ticker message, got: {}", event_type),
                input: Some(message.to_string()),
            });
        }

        // Extract symbol
        let symbol = value["s"]
            .as_str()
            .ok_or_else(|| ArbitrageError::ParseError {
                message: "Missing symbol 's'".to_string(),
                input: Some(message.to_string()),
            })?;

        // Convert symbol to pair format
        let pair = Self::symbol_to_pair(symbol);

        // Parse prices (Binance uses strings for decimal values)
        let last_str = value["c"]
            .as_str()
            .ok_or_else(|| ArbitrageError::ParseError {
                message: "Missing close price 'c'".to_string(),
                input: Some(message.to_string()),
            })?;

        let bid_str = value["b"]
            .as_str()
            .ok_or_else(|| ArbitrageError::ParseError {
                message: "Missing bid price 'b'".to_string(),
                input: Some(message.to_string()),
            })?;

        let ask_str = value["a"]
            .as_str()
            .ok_or_else(|| ArbitrageError::ParseError {
                message: "Missing ask price 'a'".to_string(),
                input: Some(message.to_string()),
            })?;

        let volume_str = value["v"].as_str().unwrap_or("0");

        // Parse decimals
        let last = Decimal::from_str_exact(last_str).map_err(|e| ArbitrageError::ParseError {
            message: format!("Invalid close price: {}", e),
            input: Some(message.to_string()),
        })?;

        let bid = Decimal::from_str_exact(bid_str).map_err(|e| ArbitrageError::ParseError {
            message: format!("Invalid bid price: {}", e),
            input: Some(message.to_string()),
        })?;

        let ask = Decimal::from_str_exact(ask_str).map_err(|e| ArbitrageError::ParseError {
            message: format!("Invalid ask price: {}", e),
            input: Some(message.to_string()),
        })?;

        let volume =
            Decimal::from_str_exact(volume_str).map_err(|e| ArbitrageError::ParseError {
                message: format!("Invalid volume: {}", e),
                input: Some(message.to_string()),
            })?;

        Ok(Price {
            pair,
            bid,
            ask,
            last,
            volume_24h: volume,
            timestamp: Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_to_pair() {
        assert_eq!(BinanceParser::symbol_to_pair("SOLUSDC"), "SOL/USDC");
        assert_eq!(BinanceParser::symbol_to_pair("BTCUSDT"), "BTC/USDT");
    }

    #[test]
    fn test_pair_to_symbol() {
        assert_eq!(BinanceParser::pair_to_symbol("SOL/USDC"), "SOLUSDC");
        assert_eq!(BinanceParser::pair_to_symbol("BTC/USDT"), "BTCUSDT");
    }

    #[test]
    fn test_parse_valid_ticker() {
        let parser = BinanceParser::new();

        let ticker_json = r#"{
            "e": "24hrTicker",
            "s": "SOLUSDC",
            "c": "143.50",
            "b": "143.48",
            "a": "143.52",
            "v": "1234567.89"
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
    fn test_parse_invalid_event_type() {
        let parser = BinanceParser::new();

        let invalid_json = r#"{
            "e": "trade",
            "s": "SOLUSDC",
            "c": "143.50"
        }"#;

        let result = parser.parse(invalid_json);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Not a ticker message")
        );
    }

    #[test]
    fn test_parse_missing_fields() {
        let parser = BinanceParser::new();

        let incomplete_json = r#"{
            "e": "24hrTicker"
        }"#;

        let result = parser.parse(incomplete_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_json() {
        let parser = BinanceParser::new();

        let invalid_json = "not json";
        let result = parser.parse(invalid_json);
        assert!(result.is_err());
    }
}
