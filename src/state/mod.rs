//! Price State Management Module
//!
//! Provides thread-safe shared state for storing and accessing latest prices
//! from multiple exchanges, with staleness detection and spread calculation.

pub mod price;
pub mod types;

pub use price::PriceState;
pub use types::{ExchangeId, PriceData};

