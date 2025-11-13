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
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔌 Connecting to Binance.US WebSocket...");

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
    let mut exchange = BinanceExchange::new(config)?;

    // Subscribe to ticker (this connects with subscription URL)
    // Binance.US supports SOL/USDC (verified working)
    let pair = "SOL/USDC";

    exchange.subscribe_ticker(pair).await?;
    println!("✅ Connected and subscribed to {} ticker!", pair);

    // Poll for price updates
    println!("\n📈 Waiting for price updates (Ctrl+C to stop)...\n");

    for i in 0..10 {
        sleep(Duration::from_secs(2)).await;

        match exchange.get_latest_price(pair).await {
            Ok(price) => {
                println!(
                    "[{}] {}: bid={}, ask={}, last={}, spread={:.4}%",
                    i + 1,
                    pair,
                    price.bid,
                    price.ask,
                    price.last,
                    price.spread_percentage()
                );
            }
            Err(e) => {
                println!("[{}] No price data yet: {}", i + 1, e);
            }
        }
    }

    println!("\n🔌 Disconnecting...");
    exchange.disconnect().await?;
    println!("✅ Disconnected");

    Ok(())
}
