//! Example: Connect to Binance.US WebSocket and receive real-time price updates
//!
//! This example demonstrates connecting to Binance.US WebSocket
//! and receiving live price updates for SOL/USDC.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example binance_websocket_test
//! ```
//!
//! # Requirements
//!
//! - Binance.US WebSocket (no API keys required for public ticker streams)
//! - Internet connection
//! - Must be in a US state where Binance.US operates

use arb_bot::config::BinanceConfig;
use arb_bot::exchanges::Exchange;
use arb_bot::exchanges::binance::BinanceExchange;
use arb_bot::logger::{info, warn, LoggerConfig, LogFormat};
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
    
    info!("Connecting to Binance.US WebSocket...");

    // Create Binance config
    // Using Binance.US (for US customers)
    // Public ticker streams don't require API keys
    // TODO: Remove API keys and use environment variables for WebSocket connections
    let config = BinanceConfig {
        api_key: String::new(),
        api_secret: String::new(),
        testnet: false, // Use production Binance.US
    };

    // Create exchange instance
    let mut exchange = BinanceExchange::new(config)
        .map_err(|e| color_eyre::eyre::eyre!("{}", e))?;

    // Subscribe to ticker (this connects with subscription URL)
    // Binance.US supports SOL/USDC (verified working)
    let pair = "SOL/USDC";

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
