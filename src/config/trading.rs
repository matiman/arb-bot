use crate::config::parse::ConfigError;
use rust_decimal::Decimal;
use serde::Deserialize;

/// Wrapper for TOML deserialization with [trading] section
#[derive(Debug, Deserialize)]
pub struct TradingConfigToml {
    pub trading: RawTradingConfig,
}

/// Raw trading configuration for deserialization (loose validation)
#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct RawTradingConfig {
    pub pair: Option<String>,
    pub spread_threshold: Option<f64>,
    pub order_size: Option<f64>,
    pub cooldown_ms: Option<u64>,
}

/// Validated trading configuration (guaranteed valid after parse)
#[derive(Debug, Clone)]
pub struct TradingConfig {
    pair: String,
    spread_threshold: Decimal,
    order_size: Decimal,
    cooldown_ms: u64,
}

impl TryFrom<RawTradingConfig> for TradingConfig {
    type Error = ConfigError;

    fn try_from(raw: RawTradingConfig) -> std::result::Result<Self, Self::Error> {
        let pair = raw.pair.ok_or_else(|| ConfigError::MissingField {
            field: "pair".to_string(),
        })?;

        let spread_threshold_raw =
            raw.spread_threshold
                .ok_or_else(|| ConfigError::MissingField {
                    field: "spread_threshold".to_string(),
                })?;

        let order_size_raw = raw.order_size.ok_or_else(|| ConfigError::MissingField {
            field: "order_size".to_string(),
        })?;

        let cooldown_ms = raw.cooldown_ms.ok_or_else(|| ConfigError::MissingField {
            field: "cooldown_ms".to_string(),
        })?;

        // Validate spread_threshold: must be in [0.0, 1.0]
        if !(0.0..=1.0).contains(&spread_threshold_raw) {
            return Err(ConfigError::InvalidSpreadThreshold {
                value: spread_threshold_raw,
                reason: "must be between 0.0 and 1.0".to_string(),
            });
        }

        // Validate order_size: must be > 0
        if order_size_raw <= 0.0 {
            return Err(ConfigError::InvalidOrderSize {
                value: order_size_raw,
                reason: "must be greater than 0".to_string(),
            });
        }

        // Validate cooldown: must be >= 1000ms
        if cooldown_ms < 1000 {
            return Err(ConfigError::InvalidCooldown {
                value: cooldown_ms,
                reason: "must be at least 1000ms".to_string(),
            });
        }

        // Convert to validated types
        Ok(TradingConfig {
            pair,
            spread_threshold: Decimal::from_f64_retain(spread_threshold_raw)
                .ok_or(ConfigError::InvalidDecimal)?,
            order_size: Decimal::from_f64_retain(order_size_raw)
                .ok_or(ConfigError::InvalidDecimal)?,
            cooldown_ms,
        })
    }
}

impl TradingConfig {
    pub fn pair(&self) -> &str {
        &self.pair
    }

    pub fn trading_pair(&self) -> &str {
        &self.pair
    }

    pub fn spread_threshold(&self) -> Decimal {
        self.spread_threshold
    }

    pub fn order_size(&self) -> Decimal {
        self.order_size
    }

    pub fn cooldown_ms(&self) -> u64 {
        self.cooldown_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_config_parses() {
        let raw = RawTradingConfig {
            pair: Some("SOL/USDC".to_string()),
            spread_threshold: Some(0.002),
            order_size: Some(10.0),
            cooldown_ms: Some(5000),
        };

        let cfg = TradingConfig::try_from(raw).unwrap();
        assert_eq!(cfg.pair(), "SOL/USDC");
        assert_eq!(
            cfg.spread_threshold(),
            Decimal::from_f64_retain(0.002).unwrap()
        );
    }

    #[test]
    fn reject_invalid_spread_threshold() {
        let raw = RawTradingConfig {
            pair: Some("SOL/USDC".to_string()),
            spread_threshold: Some(1.5),
            order_size: Some(10.0),
            cooldown_ms: Some(5000),
        };

        let err = TradingConfig::try_from(raw).unwrap_err();
        assert!(format!("{}", err).to_lowercase().contains("spread"));
    }

    #[test]
    fn reject_invalid_order_size() {
        let raw = RawTradingConfig {
            pair: Some("SOL/USDC".to_string()),
            spread_threshold: Some(0.002),
            order_size: Some(0.0),
            cooldown_ms: Some(5000),
        };

        let err = TradingConfig::try_from(raw).unwrap_err();
        assert!(format!("{}", err).to_lowercase().contains("order_size"));
    }

    #[test]
    fn reject_missing_pair() {
        let raw = RawTradingConfig {
            pair: None,
            spread_threshold: Some(0.002),
            order_size: Some(10.0),
            cooldown_ms: Some(5000),
        };

        let err = TradingConfig::try_from(raw).unwrap_err();
        assert!(format!("{}", err).to_lowercase().contains("pair"));
    }
}
