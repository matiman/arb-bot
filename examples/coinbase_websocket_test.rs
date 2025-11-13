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
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔌 Connecting to Coinbase Advanced Trade WebSocket...");

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
    let mut exchange = CoinbaseExchange::new(config)?;

    // Subscribe to ticker (this connects and sends subscription message)
    // Coinbase uses product_id format: SOL-USD (not SOL-USDC)
    let pair = "SOL/USD";

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
