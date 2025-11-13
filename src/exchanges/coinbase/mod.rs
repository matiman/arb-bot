//! Coinbase Exchange Integration
//!
//! Implements the Exchange trait for Coinbase, providing WebSocket price feeds
//! and REST API for trading operations.

pub mod exchange;
pub mod parser;
pub mod rest;
pub mod types;

pub use exchange::CoinbaseExchange;
pub use parser::CoinbaseParser;

