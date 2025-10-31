use super::Exchange;
use crate::error::{ArbitrageError, Result};

#[allow(clippy::result_large_err)]
pub trait ExchangeFactory {
    fn create_exchange(&self, name: &str, _config: Option<&()>) -> Result<Box<dyn Exchange>>;
}

pub struct DefaultExchangeFactory;

impl ExchangeFactory for DefaultExchangeFactory {
    fn create_exchange(&self, name: &str, _config: Option<&()>) -> Result<Box<dyn Exchange>> {
        match name {
            // Production exchanges will be added here in future tasks
            // "binance" => Ok(Box::new(BinanceExchange::new(config)?)),
            // "coinbase" => Ok(Box::new(CoinbaseExchange::new(config)?)),
            _ => Err(ArbitrageError::ConfigError {
                field: "exchange".to_string(),
                reason: format!("Unknown exchange: {}", name),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory_rejects_unknown() {
        let factory = DefaultExchangeFactory;
        let result = factory.create_exchange("binance", None);
        assert!(result.is_err());
    }
}
