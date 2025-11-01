//! WebSocket module for exchange connections
//!
//! Provides generic WebSocket manager with reconnection logic and message parsing.

pub mod manager;
pub mod parser;
pub mod reconnect;

pub use manager::WebSocketManager;
pub use parser::MessageParser;
pub use reconnect::ReconnectionStrategy;

