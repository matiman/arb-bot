//! Common test utilities and helpers
//!
//! This module provides shared testing utilities that can be used
//! across different integration tests.

use arb_bot::arb_bot;
use arb_bot::exchanges::Exchange;
use arb_bot::monitoring;

/// Test configuration for integration tests
pub struct TestConfig {
    pub test_exchange_url: String,
    pub test_symbol: String,
    pub test_timeout_ms: u64,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            test_exchange_url: "https://api.test-exchange.com".to_string(),
            test_symbol: "BTC/USDT".to_string(),
            test_timeout_ms: 5000,
        }
    }
}

/// Mock exchange for testing purposes
pub struct MockExchange {
    pub name: String,
    pub base_url: String,
}

impl MockExchange {
    pub fn new(name: &str, base_url: &str) -> Self {
        Self {
            name: name.to_string(),
            base_url: base_url.to_string(),
        }
    }
}

impl Exchange for MockExchange {
    fn get_price(&self, symbol: &str) -> Result<f64, String> {
        // Mock price data for testing
        match symbol {
            "BTC/USDT" => Ok(45000.0),
            "ETH/USDT" => Ok(3000.0),
            _ => Err(format!("Symbol {} not supported", symbol)),
        }
    }

    fn place_order(&self, symbol: &str, amount: f64) -> Result<String, String> {
        Ok(format!(
            "Order placed on {}: {} {} at {}",
            self.name,
            amount,
            symbol,
            self.get_price(symbol)?
        ))
    }
}

/// Helper function to initialize test environment
pub fn setup_test_env() -> TestConfig {
    println!("Setting up test environment...");
    TestConfig::default()
}

/// Helper function to cleanup test environment
pub fn cleanup_test_env() {
    println!("Cleaning up test environment...");
}

/// Helper function to create mock exchanges for testing
pub fn create_mock_exchanges() -> Vec<MockExchange> {
    vec![
        MockExchange::new("TestExchange1", "https://api.test1.com"),
        MockExchange::new("TestExchange2", "https://api.test2.com"),
    ]
}
