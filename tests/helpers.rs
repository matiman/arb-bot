use arb_bot::config::trading::TradingConfig;
use arb_bot::error::Result;

// Contract expected from config module ---------------------------------------------------------
pub trait TradingConfigContract {
    fn trading_pair(&self) -> &str;
    fn spread_threshold(&self) -> rust_decimal::Decimal;
}

impl TradingConfigContract for TradingConfig {
    fn trading_pair(&self) -> &str {
        self.trading_pair()
    }

    fn spread_threshold(&self) -> rust_decimal::Decimal {
        self.spread_threshold()
    }
}

#[allow(clippy::result_large_err)]
pub trait ConfigSource {
    fn load_raw(&self) -> Result<String>;
}

pub struct TestSource {
    entries: Vec<(String, String)>,
}

impl TestSource {
    pub fn new<const N: usize>(entries: [(String, String); N]) -> Self {
        Self {
            entries: entries.to_vec(),
        }
    }
}

impl ConfigSource for TestSource {
    fn load_raw(&self) -> Result<String> {
        let s = self
            .entries
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("\n");
        Ok(s)
    }
}

// Helper functions using actual config loading ------------------------------------------------
#[allow(clippy::result_large_err)]
pub fn load_via_helper(path: &str) -> Result<Box<dyn TradingConfigContract>> {
    use arb_bot::config::trading::TradingConfigToml;
    use arb_bot::error::ArbitrageError;
    let content = std::fs::read_to_string(path).map_err(ArbitrageError::from)?;
    let wrapper: TradingConfigToml = toml::from_str(&content).map_err(ArbitrageError::from)?;
    let cfg = TradingConfig::try_from(wrapper.trading).map_err(ArbitrageError::from)?;
    Ok(Box::new(cfg))
}

#[allow(clippy::result_large_err)]
pub fn load_with_sources(
    path: &str,
    sources: &[Box<dyn ConfigSource>],
) -> Result<Box<dyn TradingConfigContract>> {
    use arb_bot::config::trading::TradingConfigToml;
    use arb_bot::error::ArbitrageError;
    let content = std::fs::read_to_string(path).map_err(ArbitrageError::from)?;
    let mut wrapper: TradingConfigToml = toml::from_str(&content).map_err(ArbitrageError::from)?;

    // Apply overrides from sources
    for source in sources {
        let raw = source.load_raw()?;
        // Simple key=value parser for test overrides (e.g., "trading.spread_threshold=0.003")
        for line in raw.lines() {
            if let Some((key, value)) = line.split_once('=')
                && key.trim() == "trading.spread_threshold"
                && let Ok(v) = value.trim().parse::<f64>()
            {
                wrapper.trading.spread_threshold = Some(v);
            }
        }
    }

    let cfg = TradingConfig::try_from(wrapper.trading).map_err(ArbitrageError::from)?;
    Ok(Box::new(cfg))
}

#[allow(clippy::result_large_err)]
pub fn try_parse_inline(toml_str: &str) -> Result<()> {
    use arb_bot::config::trading::TradingConfigToml;
    use arb_bot::error::ArbitrageError;
    let wrapper: TradingConfigToml = toml::from_str(toml_str).map_err(ArbitrageError::from)?;
    TradingConfig::try_from(wrapper.trading).map_err(ArbitrageError::from)?;
    Ok(())
}
