//! Exchange configuration types

use serde::Deserialize;

/// Binance exchange configuration
#[derive(Debug, Clone, Deserialize)]
pub struct BinanceConfig {
    /// Binance API key
    pub api_key: String,
    /// Binance API secret
    pub api_secret: String,
    /// Use testnet (true) or production (false)
    // TODO Change to use environment variables
    pub testnet: bool,
}

/// Coinbase exchange configuration
#[derive(Debug, Clone, Deserialize)]
pub struct CoinbaseConfig {
    /// Coinbase API key
    pub api_key: String,
    /// Coinbase API secret (for JWT signing)
    pub api_secret: String,
    /// Use sandbox (true) or production (false)
    // TODO Change to use environment variables
    pub sandbox: bool,
}
