//! Integration tests for Price State Manager
//!
//! Following TDD: These tests will fail initially until implementation exists.

use arb_bot::exchanges::Price;
use arb_bot::state::{ExchangeId, PriceState};
use chrono::Utc;
use rust_decimal::Decimal;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_price_state_creation() {
    let state = PriceState::new(Duration::from_secs(5));
    let all_prices = state.get_all_prices();
    assert!(all_prices.is_empty());
}

#[tokio::test]
async fn test_update_and_get_price() {
    let state = PriceState::new(Duration::from_secs(5));

    let price = Price {
        pair: "SOL/USDC".to_string(),
        bid: Decimal::from(100),
        ask: Decimal::from(101),
        last: Decimal::from(100),
        volume_24h: Decimal::from(1000000),
        timestamp: Utc::now(),
    };

    state.update_price(ExchangeId::Binance, "SOL/USDC", price.clone(), 1);

    let retrieved = state.get_price(ExchangeId::Binance, "SOL/USDC");
    assert!(retrieved.is_some());
    let price_data = retrieved.unwrap();
    assert_eq!(price_data.price.pair, "SOL/USDC");
    assert_eq!(price_data.price.bid, Decimal::from(100));
    assert_eq!(price_data.sequence, 1);
}

#[tokio::test]
async fn test_multiple_exchanges_same_pair() {
    let state = PriceState::new(Duration::from_secs(5));

    let binance_price = Price {
        pair: "SOL/USDC".to_string(),
        bid: Decimal::from(100),
        ask: Decimal::from(101),
        last: Decimal::from(100),
        volume_24h: Decimal::ZERO,
        timestamp: Utc::now(),
    };

    let coinbase_price = Price {
        pair: "SOL/USDC".to_string(),
        bid: Decimal::from(102),
        ask: Decimal::from(103),
        last: Decimal::from(102),
        volume_24h: Decimal::ZERO,
        timestamp: Utc::now(),
    };

    state.update_price(ExchangeId::Binance, "SOL/USDC", binance_price, 1);
    state.update_price(ExchangeId::Coinbase, "SOL/USDC", coinbase_price, 1);

    let binance = state.get_price(ExchangeId::Binance, "SOL/USDC").unwrap();
    let coinbase = state.get_price(ExchangeId::Coinbase, "SOL/USDC").unwrap();

    assert_eq!(binance.price.bid, Decimal::from(100));
    assert_eq!(coinbase.price.bid, Decimal::from(102));
}

#[tokio::test]
async fn test_concurrent_writes() {
    let state = Arc::new(PriceState::new(Duration::from_secs(5)));

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let state = Arc::clone(&state);
            tokio::spawn(async move {
                let pair = format!("PAIR{}", i);
                let price = Price {
                    pair: pair.clone(),
                    bid: Decimal::from(100 + i),
                    ask: Decimal::from(101 + i),
                    last: Decimal::from(100 + i),
                    volume_24h: Decimal::ZERO,
                    timestamp: Utc::now(),
                };
                state.update_price(ExchangeId::Binance, &pair, price, i);
            })
        })
        .collect();

    for handle in handles {
        handle.await.unwrap();
    }

    // Verify all prices were written
    let all_prices = state.get_all_prices();
    assert_eq!(all_prices.len(), 10);
}

#[tokio::test]
async fn test_concurrent_reads_and_writes() {
    let state = Arc::new(PriceState::new(Duration::from_secs(5)));

    // Writer task
    let writer_state = Arc::clone(&state);
    let writer = tokio::spawn(async move {
        for i in 0..100 {
            let price = Price {
                pair: "SOL/USDC".to_string(),
                bid: Decimal::from(100 + i),
                ask: Decimal::from(101 + i),
                last: Decimal::from(100 + i),
                volume_24h: Decimal::ZERO,
                timestamp: Utc::now(),
            };
            writer_state.update_price(ExchangeId::Binance, "SOL/USDC", price, i);
            sleep(Duration::from_millis(10)).await;
        }
    });

    // Reader tasks (multiple concurrent readers)
    let reader_handles: Vec<_> = (0..5)
        .map(|_| {
            let state = Arc::clone(&state);
            tokio::spawn(async move {
                for _ in 0..50 {
                    let _price = state.get_price(ExchangeId::Binance, "SOL/USDC");
                    sleep(Duration::from_millis(20)).await;
                }
            })
        })
        .collect();

    writer.await.unwrap();
    for handle in reader_handles {
        handle.await.unwrap();
    }

    // Verify final price
    let final_price = state.get_price(ExchangeId::Binance, "SOL/USDC");
    assert!(final_price.is_some());
}

#[tokio::test]
async fn test_staleness_detection() {
    let state = PriceState::new(Duration::from_secs(1)); // Max age: 1 second

    let price = Price {
        pair: "SOL/USDC".to_string(),
        bid: Decimal::from(100),
        ask: Decimal::from(101),
        last: Decimal::from(100),
        volume_24h: Decimal::ZERO,
        timestamp: Utc::now(),
    };

    state.update_price(ExchangeId::Binance, "SOL/USDC", price, 1);

    // Immediately: not stale
    assert!(!state.is_stale(ExchangeId::Binance, "SOL/USDC"));

    // Wait for price to become stale
    sleep(Duration::from_millis(1100)).await;

    // Now: stale
    assert!(state.is_stale(ExchangeId::Binance, "SOL/USDC"));
}

#[tokio::test]
async fn test_spread_calculation() {
    let state = PriceState::new(Duration::from_secs(5));

    let binance_price = Price {
        pair: "SOL/USDC".to_string(),
        bid: Decimal::from(100),
        ask: Decimal::from(101),
        last: Decimal::from(100),
        volume_24h: Decimal::ZERO,
        timestamp: Utc::now(),
    };

    let coinbase_price = Price {
        pair: "SOL/USDC".to_string(),
        bid: Decimal::from(102),
        ask: Decimal::from(103),
        last: Decimal::from(102),
        volume_24h: Decimal::ZERO,
        timestamp: Utc::now(),
    };

    state.update_price(ExchangeId::Binance, "SOL/USDC", binance_price, 1);
    state.update_price(ExchangeId::Coinbase, "SOL/USDC", coinbase_price, 1);

    let spread = state.get_spread(ExchangeId::Binance, ExchangeId::Coinbase, "SOL/USDC");
    assert!(spread.is_some());
    // Binance mid: 100.5, Coinbase mid: 102.5, spread: 2.0
    assert_eq!(spread.unwrap(), Decimal::from(2));
}

#[tokio::test]
async fn test_spread_percentage_calculation() {
    let state = PriceState::new(Duration::from_secs(5));

    let binance_price = Price {
        pair: "SOL/USDC".to_string(),
        bid: Decimal::from(100),
        ask: Decimal::from(101),
        last: Decimal::from(100),
        volume_24h: Decimal::ZERO,
        timestamp: Utc::now(),
    };

    let coinbase_price = Price {
        pair: "SOL/USDC".to_string(),
        bid: Decimal::from(102),
        ask: Decimal::from(103),
        last: Decimal::from(102),
        volume_24h: Decimal::ZERO,
        timestamp: Utc::now(),
    };

    state.update_price(ExchangeId::Binance, "SOL/USDC", binance_price, 1);
    state.update_price(ExchangeId::Coinbase, "SOL/USDC", coinbase_price, 1);

    let spread_pct = state.get_spread_percentage(
        ExchangeId::Binance,
        ExchangeId::Coinbase,
        "SOL/USDC",
    );
    assert!(spread_pct.is_some());
    // Spread: 2.0, Binance mid: 100.5, Percentage: (2.0 / 100.5) * 100 ≈ 1.99%
    let expected = (Decimal::from(2) / Decimal::from_str_exact("100.5").unwrap())
        * Decimal::from(100);
    assert!((spread_pct.unwrap() - expected).abs() < Decimal::from_str_exact("0.01").unwrap());
}

#[tokio::test]
async fn test_spread_with_stale_price() {
    let state = PriceState::new(Duration::from_secs(1)); // Max age: 1 second

    let fresh_price = Price {
        pair: "SOL/USDC".to_string(),
        bid: Decimal::from(100),
        ask: Decimal::from(101),
        last: Decimal::from(100),
        volume_24h: Decimal::ZERO,
        timestamp: Utc::now(),
    };

    let stale_price = Price {
        pair: "SOL/USDC".to_string(),
        bid: Decimal::from(102),
        ask: Decimal::from(103),
        last: Decimal::from(102),
        volume_24h: Decimal::ZERO,
        timestamp: Utc::now(),
    };

    state.update_price(ExchangeId::Binance, "SOL/USDC", fresh_price, 1);
    state.update_price(ExchangeId::Coinbase, "SOL/USDC", stale_price, 1);

    // Wait for coinbase price to become stale
    sleep(Duration::from_millis(1100)).await;

    // Spread calculation should reject stale price
    let spread = state.get_spread(ExchangeId::Binance, ExchangeId::Coinbase, "SOL/USDC");
    assert!(spread.is_none()); // Stale price rejected
}

#[tokio::test]
async fn test_spread_with_max_time_difference() {
    let state = PriceState::new(Duration::from_secs(5)); // Max age: 5s, max time diff: 2.5s

    let price1 = Price {
        pair: "SOL/USDC".to_string(),
        bid: Decimal::from(100),
        ask: Decimal::from(101),
        last: Decimal::from(100),
        volume_24h: Decimal::ZERO,
        timestamp: Utc::now(),
    };

    state.update_price(ExchangeId::Binance, "SOL/USDC", price1, 1);

    // Wait 3 seconds (exceeds max_time_diff of 2.5s)
    sleep(Duration::from_millis(3100)).await;

    let price2 = Price {
        pair: "SOL/USDC".to_string(),
        bid: Decimal::from(102),
        ask: Decimal::from(103),
        last: Decimal::from(102),
        volume_24h: Decimal::ZERO,
        timestamp: Utc::now(),
    };

    state.update_price(ExchangeId::Coinbase, "SOL/USDC", price2, 1);

    // Both prices are fresh (< 5s), but captured 3s apart (> 2.5s max diff)
    // Should reject because time difference too large
    let spread = state.get_spread(ExchangeId::Binance, ExchangeId::Coinbase, "SOL/USDC");
    assert!(spread.is_none()); // Rejected due to max time difference
}

#[tokio::test]
async fn test_spread_with_acceptable_time_difference() {
    let state = PriceState::new(Duration::from_secs(5)); // Max age: 5s, max time diff: 2.5s

    let price1 = Price {
        pair: "SOL/USDC".to_string(),
        bid: Decimal::from(100),
        ask: Decimal::from(101),
        last: Decimal::from(100),
        volume_24h: Decimal::ZERO,
        timestamp: Utc::now(),
    };

    state.update_price(ExchangeId::Binance, "SOL/USDC", price1, 1);

    // Wait 1 second (within max_time_diff of 2.5s)
    sleep(Duration::from_millis(1000)).await;

    let price2 = Price {
        pair: "SOL/USDC".to_string(),
        bid: Decimal::from(102),
        ask: Decimal::from(103),
        last: Decimal::from(102),
        volume_24h: Decimal::ZERO,
        timestamp: Utc::now(),
    };

    state.update_price(ExchangeId::Coinbase, "SOL/USDC", price2, 1);

    // Both prices fresh and captured within acceptable time window
    let spread = state.get_spread(ExchangeId::Binance, ExchangeId::Coinbase, "SOL/USDC");
    assert!(spread.is_some()); // Should succeed
    assert_eq!(spread.unwrap(), Decimal::from(2));
}

#[tokio::test]
async fn test_remove_stale_prices() {
    let state = PriceState::new(Duration::from_secs(1)); // Max age: 1 second

    // Add fresh price
    let fresh_price = Price {
        pair: "SOL/USDC".to_string(),
        bid: Decimal::from(100),
        ask: Decimal::from(101),
        last: Decimal::from(100),
        volume_24h: Decimal::ZERO,
        timestamp: Utc::now(),
    };
    state.update_price(ExchangeId::Binance, "SOL/USDC", fresh_price, 1);

    // Add stale price (will become stale)
    let stale_price = Price {
        pair: "BTC/USD".to_string(),
        bid: Decimal::from(50000),
        ask: Decimal::from(50001),
        last: Decimal::from(50000),
        volume_24h: Decimal::ZERO,
        timestamp: Utc::now(),
    };
    state.update_price(ExchangeId::Coinbase, "BTC/USD", stale_price, 1);

    // Wait for BTC price to become stale (max_age is 1 second)
    sleep(Duration::from_millis(1100)).await;

    // Both prices are now stale. Update SOL to make it fresh again
    // so we can verify only BTC gets removed
    let fresh_sol_price = Price {
        pair: "SOL/USDC".to_string(),
        bid: Decimal::from(100),
        ask: Decimal::from(101),
        last: Decimal::from(100),
        volume_24h: Decimal::ZERO,
        timestamp: Utc::now(),
    };
    state.update_price(ExchangeId::Binance, "SOL/USDC", fresh_sol_price, 2);

    let removed_count = state.remove_stale_prices();
    assert_eq!(removed_count, 1); // BTC price should be removed

    // SOL price should still exist (we just updated it)
    assert!(state.get_price(ExchangeId::Binance, "SOL/USDC").is_some());
    // BTC price should be gone
    assert!(state.get_price(ExchangeId::Coinbase, "BTC/USD").is_none());
}

#[tokio::test]
async fn test_clear_all_prices() {
    let state = PriceState::new(Duration::from_secs(5));

    state.update_price(
        ExchangeId::Binance,
        "SOL/USDC",
        Price {
            pair: "SOL/USDC".to_string(),
            bid: Decimal::from(100),
            ask: Decimal::from(101),
            last: Decimal::from(100),
            volume_24h: Decimal::ZERO,
            timestamp: Utc::now(),
        },
        1,
    );

    state.clear();

    let all_prices = state.get_all_prices();
    assert!(all_prices.is_empty());
}

#[tokio::test]
async fn test_spread_missing_price() {
    let state = PriceState::new(Duration::from_secs(5));

    // Only update one exchange
    state.update_price(
        ExchangeId::Binance,
        "SOL/USDC",
        Price {
            pair: "SOL/USDC".to_string(),
            bid: Decimal::from(100),
            ask: Decimal::from(101),
            last: Decimal::from(100),
            volume_24h: Decimal::ZERO,
            timestamp: Utc::now(),
        },
        1,
    );

    // Try to get spread with missing Coinbase price
    let spread = state.get_spread(ExchangeId::Binance, ExchangeId::Coinbase, "SOL/USDC");
    assert!(spread.is_none()); // Should return None when price missing
}

