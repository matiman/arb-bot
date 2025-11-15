//! Binance Exchange Integration
//!
//! Implements the Exchange trait for Binance, providing WebSocket price feeds
//! and REST API for trading operations.

pub mod exchange;
pub mod parser;
pub mod types;

pub use exchange::BinanceExchange;
pub use parser::BinanceParser;
