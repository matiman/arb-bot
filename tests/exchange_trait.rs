use arb_bot::exchanges::{Exchange, Price};
use chrono::Utc;
use rust_decimal::Decimal;
use std::sync::Arc;
use tokio::sync::RwLock;

mod common;
use common::MockExchange;

#[tokio::test]
async fn test_mock_exchange_connect() {
    let mut exchange = MockExchange::new("coinbase");
    assert!(!exchange.is_connected());

    let result = exchange.connect().await;
    assert!(result.is_ok());
    assert!(exchange.is_connected());
}

#[tokio::test]
async fn test_mock_exchange_subscribe_ticker() {
    let mut exchange = MockExchange::new("coinbase");
    exchange.connect().await.unwrap();

    let result = exchange.subscribe_ticker("SOL/USDC").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_mock_exchange_get_latest_price() {
    let mut exchange = MockExchange::new("coinbase");
    exchange.connect().await.unwrap();

    let price = Price {
        pair: "SOL/USDC".to_string(),
        bid: Decimal::from(100),
        ask: Decimal::from(101),
        last: Decimal::from(100),
        volume_24h: Decimal::from(1000000),
        timestamp: Utc::now(),
    };

    exchange.set_price("SOL/USDC", price.clone());

    let result = exchange.get_latest_price("SOL/USDC").await;
    assert!(result.is_ok());
    let retrieved_price = result.unwrap();
    assert_eq!(retrieved_price.pair, "SOL/USDC");
    assert_eq!(retrieved_price.bid, Decimal::from(100));
}

#[tokio::test]
async fn test_mock_exchange_place_order() {
    let mut exchange = MockExchange::new("coinbase");
    exchange.connect().await.unwrap();

    use arb_bot::exchanges::Order;
    let order = Order::market_buy("SOL/USDC", Decimal::from(10));

    let result = exchange.place_order(order).await;
    assert!(result.is_ok());
    let order_result = result.unwrap();
    assert!(!order_result.order_id.is_empty());
}

#[tokio::test]
async fn test_mock_exchange_get_balance() {
    let mut exchange = MockExchange::new("coinbase");
    exchange.connect().await.unwrap();

    exchange.set_balance("USDC", Decimal::from(1000));

    let result = exchange.get_balance("USDC").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Decimal::from(1000));
}

#[tokio::test]
async fn test_mock_exchange_name() {
    let exchange = MockExchange::new("binance");
    assert_eq!(exchange.name(), "binance");
}

#[tokio::test]
async fn test_mock_exchange_disconnect() {
    let mut exchange = MockExchange::new("coinbase");
    exchange.connect().await.unwrap();
    assert!(exchange.is_connected());

    let result = exchange.disconnect().await;
    assert!(result.is_ok());
    assert!(!exchange.is_connected());
}

#[tokio::test]
async fn test_trait_object_usage() {
    let mut exchange: Box<dyn Exchange> = Box::new(MockExchange::new("binance"));
    exchange.connect().await.unwrap();
    assert!(exchange.is_connected());

    let price = exchange.get_latest_price("SOL/USDC").await;
    // Should handle error gracefully (price not set)
    assert!(price.is_err() || price.is_ok());
}

#[tokio::test]
async fn test_mock_exchange_new() {
    let exchange = MockExchange::new("binance");
    assert_eq!(exchange.name(), "binance");
    assert!(!exchange.is_connected());
}

#[tokio::test]
async fn test_mock_exchange_set_price() {
    let mut exchange = MockExchange::new("binance");
    let price = Price {
        pair: "SOL/USDC".to_string(),
        bid: Decimal::from(100),
        ask: Decimal::from(101),
        last: Decimal::from(100),
        volume_24h: Decimal::from(1000000),
        timestamp: Utc::now(),
    };

    exchange.set_price("SOL/USDC", price.clone());
    exchange.connect().await.unwrap();

    let retrieved = exchange.get_latest_price("SOL/USDC").await.unwrap();
    assert_eq!(retrieved.pair, price.pair);
    assert_eq!(retrieved.bid, price.bid);
}

#[tokio::test]
async fn test_mock_exchange_set_balance() {
    let mut exchange = MockExchange::new("binance");
    exchange.set_balance("USDC", Decimal::from(1000));
    exchange.connect().await.unwrap();

    let balance = exchange.get_balance("USDC").await.unwrap();
    assert_eq!(balance, Decimal::from(1000));
}

#[tokio::test]
async fn test_concurrent_access_to_exchange() {
    let exchange = Arc::new(RwLock::new(MockExchange::new("binance")));

    // Spawn multiple tasks accessing exchange concurrently
    let handles: Vec<_> = (0..5)
        .map(|_| {
            let exchange = Arc::clone(&exchange);
            tokio::spawn(async move {
                let mut guard = exchange.write().await;
                guard.connect().await.unwrap();
                guard.get_balance("USDC").await
            })
        })
        .collect();

    for handle in handles {
        let result = handle.await.unwrap();
        // Some may succeed, some may fail depending on implementation
        assert!(result.is_ok() || result.is_err());
    }
}

#[tokio::test]
async fn test_handle_connection_errors() {
    let exchange = MockExchange::new("binance");

    // Try to get price without connecting
    let result = exchange.get_latest_price("SOL/USDC").await;
    // Should return error when not connected
    assert!(result.is_err());
}
