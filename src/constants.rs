//! Application-wide string constants
//!
//! Centralized location for all string literals used throughout the codebase.
//! This improves maintainability and reduces typos.

/// Exchange identifiers
pub mod exchange {
    /// Binance exchange identifier
    pub const BINANCE: &str = "binance";

    /// Coinbase exchange identifier
    pub const COINBASE: &str = "coinbase";
}

/// WebSocket endpoints
pub mod websocket {
    /// Binance testnet WebSocket endpoint
    pub const BINANCE_TESTNET: &str = "wss://testnet.binance.vision/ws";

    /// Binance.US production WebSocket endpoint
    pub const BINANCE_US_PRODUCTION: &str = "wss://stream.binance.us:9443/ws";

    /// Coinbase Exchange WebSocket endpoint (public, no auth required)
    pub const COINBASE_EXCHANGE: &str = "wss://ws-feed.exchange.coinbase.com";
}

/// REST API endpoints
pub mod api {
    /// Coinbase sandbox API base URL
    pub const COINBASE_SANDBOX: &str = "https://api-public.sandbox.exchange.coinbase.com";

    /// Coinbase production API base URL
    pub const COINBASE_PRODUCTION: &str = "https://api.coinbase.com";

    /// Coinbase accounts endpoint path
    pub const COINBASE_ACCOUNTS_PATH: &str = "/api/v3/brokerage/accounts";

    /// Coinbase orders endpoint path
    pub const COINBASE_ORDERS_PATH: &str = "/api/v3/brokerage/orders";
}

/// Currency symbols
pub mod currency {
    /// USDC stablecoin
    pub const USDC: &str = "USDC";

    /// USDT stablecoin
    pub const USDT: &str = "USDT";

    /// SOL (Solana) cryptocurrency
    pub const SOL: &str = "SOL";

    /// BTC (Bitcoin) cryptocurrency
    pub const BTC: &str = "BTC";
}

/// Common trading pairs
pub mod pairs {
    /// SOL/USDC trading pair
    pub const SOL_USDC: &str = "SOL/USDC";

    /// SOL/USDT trading pair
    pub const SOL_USDT: &str = "SOL/USDT";

    /// SOL/USD trading pair
    pub const SOL_USD: &str = "SOL/USD";

    /// BTC/USDT trading pair
    pub const BTC_USDT: &str = "BTC/USDT";
}

/// HTTP methods
pub mod http {
    /// GET HTTP method
    pub const GET: &str = "GET";

    /// POST HTTP method
    pub const POST: &str = "POST";
}

/// JWT authentication constants
pub mod jwt {
    /// Coinbase JWT issuer claim
    pub const COINBASE_ISSUER: &str = "cdp";
}

/// Order-related constants
pub mod order {
    /// Market order type
    pub const MARKET: &str = "MARKET";

    /// Buy order side
    pub const BUY: &str = "BUY";

    /// Sell order side
    pub const SELL: &str = "SELL";

    /// Filled order status
    pub const FILLED: &str = "FILLED";

    /// Pending order status
    pub const PENDING: &str = "PENDING";

    /// Partially filled order status
    pub const PARTIALLY_FILLED: &str = "PARTIALLY_FILLED";

    /// Cancelled order status
    pub const CANCELLED: &str = "CANCELLED";
}
