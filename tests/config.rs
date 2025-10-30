mod helpers;
use helpers::{ConfigSource, TestSource, load_via_helper, load_with_sources, try_parse_inline};

// These tests define the contract; they will fail until implementation exists.

#[test]
fn load_config_from_toml_file() {
    // Expect: successful parse from example file later
    let cfg = load_via_helper("config.example.toml").expect("config should load");
    assert_eq!(cfg.trading_pair(), "SOL/USDC");
}

#[test]
fn overrides_take_precedence_via_source_injection() {
    // Do not mutate env; inject a test source that overrides values
    let overrides =
        TestSource::new([("trading.spread_threshold".to_string(), "0.003".to_string())]);
    let cfg = load_with_sources(
        "config.example.toml",
        &[Box::new(overrides) as Box<dyn ConfigSource>],
    )
    .expect("config should load with overrides");

    assert!(cfg.spread_threshold() >= rust_decimal::Decimal::from_f64_retain(0.003).unwrap());
}

#[test]
fn reject_invalid_spread_threshold() {
    let raw = r#"
        [trading]
        pair = "SOL/USDC"
        spread_threshold = 1.5
        order_size = 10.0
        cooldown_ms = 5000
    "#;
    let err = try_parse_inline(raw).unwrap_err();
    assert!(format!("{}", err).to_lowercase().contains("spread"));
}

#[test]
fn reject_invalid_order_size() {
    let raw = r#"
        [trading]
        pair = "SOL/USDC"
        spread_threshold = 0.002
        order_size = 0.0
        cooldown_ms = 5000
    "#;
    let err = try_parse_inline(raw).unwrap_err();
    assert!(format!("{}", err).to_lowercase().contains("order_size"));
}

#[test]
fn reject_missing_required_fields() {
    let raw = r#"
        [trading]
        spread_threshold = 0.002
        order_size = 10.0
        cooldown_ms = 5000
    "#;
    let err = try_parse_inline(raw).unwrap_err();
    assert!(format!("{}", err).to_lowercase().contains("pair"));
}
