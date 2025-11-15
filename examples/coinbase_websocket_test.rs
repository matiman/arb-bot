//! Example: Connect to Coinbase WebSocket and receive real-time price updates
//!
//! This example demonstrates connecting to Coinbase Advanced Trade WebSocket
//! and receiving live price updates for SOL/USDC.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example coinbase_websocket_test
//! ```
//!
//! # Requirements
//!
//! - Coinbase Advanced Trade WebSocket (public market data endpoint)
//! - No API keys required for public ticker streams
//! - Internet connection

use arb_bot::config::CoinbaseConfig;
use arb_bot::exchanges::Exchange;
use arb_bot::exchanges::coinbase::CoinbaseExchange;
use arb_bot::logger::{LogFormat, LoggerConfig, info, warn};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    // Initialize logger
    LoggerConfig::new()
        .with_level("info")
        .with_format(LogFormat::Pretty)
        .init()
        .map_err(|e| color_eyre::eyre::eyre!("Failed to initialize logger: {}", e))?;

    info!("Connecting to Coinbase Advanced Trade WebSocket...");

    // Create Coinbase config
    // Using production Coinbase Advanced Trade Market Data endpoint
    // Public ticker streams don't require API keys (same as Binance.US)
    // See: https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/guides/websocket
    let config = CoinbaseConfig {
        api_key: String::new(),
        api_secret: String::new(),
        sandbox: false, // Use production Coinbase
    };

    // Create exchange instance
    let mut exchange =
        CoinbaseExchange::new(config).map_err(|e| color_eyre::eyre::eyre!("{}", e))?;

    // Subscribe to ticker (this connects and sends subscription message)
    // Coinbase uses product_id format: SOL-USD (not SOL-USDC)
    let pair = "SOL/USD";

    exchange
        .subscribe_ticker(pair)
        .await
        .map_err(|e| color_eyre::eyre::eyre!("{}", e))?;
    info!(pair = %pair, "Connected and subscribed to ticker");

    // Poll for price updates
    info!("Waiting for price updates (Ctrl+C to stop)...");

    for i in 0..10 {
        sleep(Duration::from_secs(2)).await;

        match exchange.get_latest_price(pair).await {
            Ok(price) => {
                info!(
                    iteration = i + 1,
                    pair = %pair,
                    bid = %price.bid,
                    ask = %price.ask,
                    last = %price.last,
                    spread_pct = %price.spread_percentage(),
                    "Price update"
                );
            }
            Err(e) => {
                warn!(iteration = i + 1, error = %e, "No price data yet");
            }
        }
    }

    info!("Disconnecting...");
    exchange
        .disconnect()
        .await
        .map_err(|e| color_eyre::eyre::eyre!("{}", e))?;
    info!("Disconnected");

    Ok(())
}
