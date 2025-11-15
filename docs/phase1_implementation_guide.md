# 🦀 Phase 1 Implementation Guide - CEX-to-CEX Arbitrage Bot

**Version:** 1.0  
**Last Updated:** October 30, 2025  
**Project:** Rust-based cryptocurrency arbitrage bot (Coinbase ↔ Binance)

---

## 📋 Table of Contents

1. [AI Agent Identity & Context](#ai-agent-identity--context)
2. [Phase 1 Overview](#phase-1-overview)
3. [Project Structure](#project-structure)
4. [Dependencies](#dependencies)
5. [Task 1: Core Error Types](#task-1-core-error-types--result-aliases)
6. [Task 2: Configuration System](#task-2-configuration-system-with-parse-pattern)
7. [Task 3: Exchange Trait](#task-3-exchange-trait-abstraction)
8. [Task 4: WebSocket Manager](#task-4-websocket-price-feed-manager)
9. [Task 5: Price State](#task-5-shared-price-state-manager)
10. [Task 6: Binance Integration](#task-6-binance-websocket-integration)
11. [Task 7: Coinbase Integration](#task-7-coinbase-websocket-integration)
12. [Task 8: Logging](#task-8-logging--observability)
13. [Rust Best Practices](#rust-best-practices-checklist)
14. [Phase 1 Completion Criteria](#phase-1-completion-criteria)
15. [XML Template Structure](#xml-template-structure)
16. [Example Configurations](#example-configurations)

---

## 🎯 AI Agent Identity & Context

```markdown
You are a **seasoned Rust software architect** with 10+ years of experience in:

- High-performance financial systems and low-latency trading platforms
- Async Rust programming with Tokio and async/await patterns
- Trait-based abstractions, generics, and zero-cost abstractions
- Test-Driven Development (TDD) in Rust
- WebSocket implementations and real-time data processing
- Error handling with Result types and custom error hierarchies
- High Frequency and Algorithmic Trading system at cryptocurrency hedge funds

You follow Rust idioms as of October 2025, emphasizing:

- Trait objects and dynamic dispatch where appropriate
- Generic programming with trait bounds
- Async traits and futures
- Interior mutability patterns (Arc<Mutex<T>>, and preferably Arc<RwLock<T>>)
- Type-safe builder patterns
- Comprehensive error handling with thiserror/anyhow
- Zero-copy optimizations where possible
- Parse pattern for validation (validate at construction time)
- Code reuse and abstraction for future implemenatfions
```

### HIGH PRIORITY Core Principles

1. **TDD Always**: Write failing tests first, then implement to pass
2. **Small & Testable**: Each task should be completable in 30 minutes
3. **Incremental**: Build on previous tasks without breaking existing code
4. **Status Markers**: Use ✅ _DONE_, ❌ _FAIL_, ⚠️ _WARNING_ for visibility
5. **Manual Checkpoints**: Wait for human confirmation before proceeding
6. **Pause And Ask Me Questions**: If you are in doubt to make a decision, ask me and wait for my response
7. **Git Commit After Confirmation**: After I confirm task completion, immediately run `git add .`, `git commit -m "✅ Complete Task N: [name][Details]"`, and `git push origin main` before logging. In the [Details] section, add what you did. Do this before step 8 below. Dont ask me to run it for you.
8. **⚠️ MANDATORY Protocol**: Follow `docs/ai_prompts/task_logging_protocol.md` - check progress before starting, log conversation after git push

---

## 📚 Phase 1 Overview

**Objective:** Establish the foundational architecture for the arbitrage bot

**Deliverables:**

- Error handling system
- Configuration management with parse pattern
- Exchange trait abstraction
- WebSocket connectivity
- Shared price state management
- Binance integration
- Coinbase integration
- Structured logging

**Testing Strategy:**

- Write tests first (TDD approach - they will fail initially)
- Unit tests alongside implementation files
- Integration tests in `tests/` folder
- 80%+ code coverage target

---

## 🏗️ Project Structure

```
arb-bot/
├── Cargo.toml                    # Package configuration
├── Cargo.lock                    # Dependency lock file
├── .gitignore                    # Git ignore rules
├── .env.example                  # Example environment variables
├── config.example.toml           # Example configuration
├── README.md                     # Project documentation
│
├── docs/
│   ├── phase1_implementation_guide.md  # This file
│   └── Technical_Architecture.md       # System architecture
│   └── Project_Spec.md                 # Project Specification
│
├── src/
│   ├── main.rs                  # Main binary entry point
│   ├── lib.rs                   # Library entry point
│   │
│   ├── error/
│   │   ├── mod.rs               # Main error types
│   │   └── exchange.rs          # Exchange-specific errors
│   │
│   ├── config/
│   │   ├── mod.rs               # Config aggregation
│   │   ├── exchange.rs          # Exchange configs (with unit tests)
│   │   ├── trading.rs           # Trading configs (with unit tests)
│   │   ├── risk.rs              # Risk configs (with unit tests)
│   │   ├── logging.rs           # Logging configs (with unit tests)
│   │   └── parse.rs             # Parse utilities (with unit tests)
│   │
│   ├── exchanges/
│   │   ├── mod.rs               # Exchange trait definition
│   │   ├── types.rs             # Common types (with unit tests)
│   │   ├── mock.rs              # Mock implementation (with unit tests)
│   │   ├── factory.rs           # Factory pattern (with unit tests)
│   │   │
│   │   ├── binance/
│   │   │   ├── mod.rs           # Binance integration
│   │   │   ├── websocket.rs     # WebSocket client (with unit tests)
│   │   │   ├── rest.rs          # REST API client (with unit tests)
│   │   │   ├── types.rs         # Binance-specific types (with unit tests)
│   │   │   ├── parser.rs        # Message parser (with unit tests)
│   │   │   └── auth.rs          # JWT authentication (with unit tests)
│   │   │
│   │   └── coinbase/
│   │       ├── mod.rs           # Coinbase integration
│   │       ├── websocket.rs     # WebSocket client (with unit tests)
│   │       ├── rest.rs          # REST API client (with unit tests)
│   │       ├── types.rs         # Coinbase-specific types (with unit tests)
│   │       ├── parser.rs        # Message parser (with unit tests)
│   │       └── auth.rs          # JWT authentication (with unit tests)
│   │
│   ├── websocket/
│   │   ├── mod.rs               # WebSocket module
│   │   ├── manager.rs           # Generic manager (with unit tests)
│   │   ├── reconnect.rs         # Reconnection logic (with unit tests)
│   │   └── parser.rs            # Parser trait (with unit tests)
│   │
│   ├── state/
│   │   ├── mod.rs               # State module
│   │   ├── price.rs             # Price state manager (with unit tests)
│   │   └── types.rs             # State types (with unit tests)
│   │
│   └── logger/
│       ├── mod.rs               # Logger module (with unit tests)
│       ├── config.rs            # Logger config (with unit tests)
│       └── format.rs            # Log formatters (with unit tests)
│
├── tests/                       # Integration tests
│   ├── error_handling.rs        # Error system integration tests
│   ├── config.rs                # Config loading integration tests
│   ├── exchange_trait.rs        # Exchange trait integration tests
│   ├── websocket_manager.rs     # WebSocket integration tests
│   ├── price_state.rs           # Price state integration tests
│   ├── binance.rs               # Binance integration tests
│   ├── coinbase.rs              # Coinbase integration tests
│   └── logging.rs               # Logging integration tests
│
└── target/                      # Build artifacts (gitignored)
```

---

## 📦 Dependencies

### Cargo.toml

```toml
[package]
name = "arb-bot"
version = "0.1.0"
edition = "2021"
rust-version = "1.85.0"

[dependencies]
# Async runtime
tokio = { version = "1.40", features = ["full"] }
tokio-tungstenite = "0.24"
futures-util = "0.3"

# Error handling
thiserror = "2.0"
anyhow = "1.0"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# Configuration
config = "0.14"

# Async traits
async-trait = "0.1"

# Decimal precision (for prices)
rust_decimal = "1.35"
rust_decimal_macros = "1.35"

# Concurrency primitives
parking_lot = "0.12"
dashmap = "6.1"

# HTTP client
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }

# Cryptography (for API authentication)
hmac = "0.12"
sha2 = "0.10"
base64 = "0.22"
jsonwebtoken = "9.3"

# Logging and tracing
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-appender = "0.2"

# Time handling
chrono = { version = "0.4", features = ["serde"] }

# Environment variables
dotenvy = "0.15"

[dev-dependencies]
# Testing utilities
tokio-test = "0.4"
mockito = "1.5"
proptest = "1.5"
tempfile = "3.10"

# Code coverage
tarpaulin = "0.31"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

---

## 🎯 Task 1: Core Error Types & Result Aliases

### Objective

Establish the foundational error handling system using Rust's idiomatic error patterns.

### What to Build

1. Create a comprehensive error type hierarchy using `thiserror`
2. Implement `Result` type aliases for convenience
3. Set up tests for error propagation

### Implementation Specification

```xml
<task id="1" phase="1">
  <name>Core Error Types & Result Aliases</name>
  <priority>critical</priority>

  <test_first>
    <file>tests/error_handling.rs</file>
    <description>
      Create tests that verify (these WILL FAIL initially):
      - Exchange-specific errors (Binance, Coinbase)
      - WebSocket connection errors
      - API rate limit errors
      - Network timeout errors
      - JSON parsing errors
      - Error conversion between types
      - Error display formatting

      IMPORTANT: Write these tests FIRST. They will fail because the implementation
      doesn't exist yet. This is expected and correct (TDD approach).
    </description>
  </test_first>

  <implementation>
    <file>src/error/mod.rs</file>
    <requirements>
      - Use thiserror for derive macros
      - Create ArbitrageError enum with variants:
        * ExchangeError { exchange: String, message: String, code: Option<i32> }
        * WebSocketError { source: String, reconnect_possible: bool }
        * NetworkError { message: String, retry_after: Option<u64> }
        * ParseError { message: String, input: Option<String> }
        * ConfigError { field: String, reason: String }
        * RateLimitExceeded { exchange: String, retry_after: u64 }
        * AuthenticationError { exchange: String, reason: String }
        * InsufficientBalance { exchange: String, asset: String, required: String, available: String }

      - Implement From traits for:
        * std::io::Error
        * serde_json::Error
        * tokio_tungstenite::tungstenite::Error

      - Create type alias: type Result<T> = std::result::Result<T, ArbitrageError>

      - Add Display implementation with user-friendly messages

      - Add comprehensive documentation with examples

      - Add unit tests in the same file for:
        * Error creation
        * Error conversion (From implementations)
        * Display formatting
        * Debug formatting
    </requirements>

    <file>src/error/exchange.rs</file>
    <requirements>
      - Define ExchangeErrorKind enum:
        * ConnectionFailed
        * OrderFailed
        * InsufficientFunds
        * InvalidOrder
        * RateLimitExceeded
        * ApiError(i32)
        * Unknown

      - Add unit tests for each error kind
      - Add helper methods for common error checks
    </requirements>
  </implementation>

  <validation>
    - All tests pass (cargo test)
    - No clippy warnings (cargo clippy)
    - Documentation builds (cargo doc --no-deps)
    - Error messages are user-friendly and actionable
  </validation>

  <completion_criteria>
    ❌ Tests written and failing initially (expected - TDD)
    💻 Error types implemented
    ✅ All tests now passing
    ✅ Unit tests added for error conversions
    ✅ Documentation complete
    ✅ Human verification complete
  </completion_criteria>
</task>
```

### Files to Create

1. **src/error/mod.rs** - Main error types with unit tests
2. **src/error/exchange.rs** - Exchange-specific errors with unit tests
3. **tests/error_handling.rs** - Integration tests
4. **Update src/lib.rs** - Export error module

### Example Error Usage

```rust
use crate::error::{ArbitrageError, Result};

pub fn connect_to_exchange(url: &str) -> Result<Connection> {
    // This returns Result<Connection, ArbitrageError>
    let conn = try_connect(url)
        .map_err(|e| ArbitrageError::NetworkError {
            message: format!("Failed to connect to {}", url),
            retry_after: Some(5000),
        })?;

    Ok(conn)
}
```

### Note: Revisit HIGH PRIORITY Core Principles before imlementing

---

## 🎯 Task 2: Configuration System with Parse Pattern

### Note: Revisit HIGH PRIORITY Core Principles before imlementing next Task and make sure you complete them. Especially point 7 & 8.

### Objective

Create a type-safe, validated configuration system using the Parse pattern to ensure configs are always valid.

### What to Build

1. TOML-based configuration with environment variable overrides
2. Parse pattern for validation at construction time
3. Strongly-typed config structs that are guaranteed to be valid

### Implementation Specification

````xml
<task id="2" phase="1">
  <name>Configuration System with Parse Pattern</name>
  <priority>critical</priority>

  <test_first>
    <file>tests/config.rs</file>
    <description>
      Write tests FIRST (they will fail initially):
      - Parse valid config from TOML file
      - Parse config from environment variables
      - Reject invalid spread_threshold (not in 0.0-1.0 range)
      - Reject invalid order_size (must be > 0)
      - Reject missing required fields
      - Reject invalid numeric values
      - Test config builder pattern
      - Test config merge (file + env overrides)

      These tests establish the contract. They WILL FAIL until implementation is done.
    </description>
  </test_first>

  <implementation>
    <files>
      <file>src/config/mod.rs</file>
      <file>src/config/exchange.rs</file>
      <file>src/config/trading.rs</file>
      <file>src/config/risk.rs</file>
      <file>src/config/logging.rs</file>
      <file>src/config/parse.rs</file>
    </files>

    <requirements>
      - Use serde for serialization/deserialization
      - Use config crate for multi-source loading

      **CRITICAL: Use Parse Pattern (not validation)**

      Pattern Overview:
      1. Create "Raw" structs for deserialization (e.g., RawTradingConfig)
      2. These Raw structs have loose types and derive Deserialize
      3. Create validated structs (e.g., TradingConfig) with strong types
      4. Implement TryFrom<RawConfig> for Config to parse and validate
      5. Once created, Config is ALWAYS valid - no need for validation methods

      Example Implementation:
      ```rust
      use serde::Deserialize;
      use rust_decimal::Decimal;

      // Step 1: Raw type for deserialization
      #[derive(Debug, Deserialize)]
      struct RawTradingConfig {
          pair: String,
          spread_threshold: f64,
          order_size: f64,
          cooldown_ms: u64,
      }

      // Step 2: Validated type (guaranteed valid)
      #[derive(Debug, Clone)]
      pub struct TradingConfig {
          pair: String,
          spread_threshold: Decimal,  // Guaranteed 0.0-1.0
          order_size: Decimal,        // Guaranteed > 0
          cooldown_ms: u64,           // Guaranteed >= 1000
      }

      // Step 3: Parse with TryFrom
      impl TryFrom<RawTradingConfig> for TradingConfig {
          type Error = ConfigError;

          fn try_from(raw: RawTradingConfig) -> Result<Self, Self::Error> {
              // Validate spread_threshold
              if raw.spread_threshold < 0.0 || raw.spread_threshold > 1.0 {
                  return Err(ConfigError::InvalidSpreadThreshold {
                      value: raw.spread_threshold,
                      reason: "must be between 0.0 and 1.0".into(),
                  });
              }

              // Validate order_size
              if raw.order_size <= 0.0 {
                  return Err(ConfigError::InvalidOrderSize {
                      value: raw.order_size,
                      reason: "must be greater than 0".into(),
                  });
              }

              // Validate cooldown
              if raw.cooldown_ms < 1000 {
                  return Err(ConfigError::InvalidCooldown {
                      value: raw.cooldown_ms,
                      reason: "must be at least 1000ms".into(),
                  });
              }

              // Convert to validated types
              Ok(TradingConfig {
                  pair: raw.pair,
                  spread_threshold: Decimal::from_f64_retain(raw.spread_threshold)
                      .ok_or_else(|| ConfigError::InvalidDecimal)?,
                  order_size: Decimal::from_f64_retain(raw.order_size)
                      .ok_or_else(|| ConfigError::InvalidDecimal)?,
                  cooldown_ms: raw.cooldown_ms,
              })
          }
      }

      // Step 4: Only expose validated config
      impl TradingConfig {
          pub fn pair(&self) -> &str {
              &self.pair
          }

          pub fn spread_threshold(&self) -> Decimal {
              self.spread_threshold  // Always valid, no need to check
          }

          pub fn order_size(&self) -> Decimal {
              self.order_size  // Always valid, no need to check
          }

          pub fn cooldown_ms(&self) -> u64 {
              self.cooldown_ms  // Always valid, no need to check
          }
      }
      ```

      Apply this pattern to:
      - ExchangeConfig (validate API keys are non-empty)
      - TradingConfig (validate ranges as shown above)
      - RiskConfig (validate max_position_size > 0, max_daily_trades > 0)
      - LoggingConfig (validate log level, file paths)

      Main Config Loading:
      ```rust
      impl Config {
          pub fn load() -> Result<Self> {
              // Load from file and environment
              let raw: RawConfig = config::Config::builder()
                  .add_source(config::File::with_name("config"))
                  .add_source(config::Environment::with_prefix("ARB"))
                  .build()?
                  .try_deserialize()?;

              // Parse and validate
              Self::try_from(raw)
          }
      }
      ```

      - Add comprehensive documentation with examples
      - Add unit tests for:
        * Each validation rule
        * Parse success cases
        * Parse failure cases with specific errors
        * Environment variable overrides
    </requirements>
  </implementation>

  <validation>
    - Config loads from file successfully
    - Environment variables override file values
    - Invalid configs are rejected at parse time with clear errors
    - All tests pass
    - No unsafe unwrap() calls
    - Unit tests verify each validation rule
  </validation>

  <completion_criteria>
    ❌ Tests written and failing initially (TDD)
    💻 Parse pattern implemented for all config types
    ✅ All tests passing
    ✅ Unit tests cover all validation rules
    ✅ Documentation complete
    ✅ Human verification complete
  </completion_criteria>
</task>
````

### Files to Create

1. **src/config/mod.rs** - Main config aggregation with unit tests
2. **src/config/exchange.rs** - Exchange configs with parse pattern and unit tests
3. **src/config/trading.rs** - Trading configs with parse logic and unit tests
4. **src/config/risk.rs** - Risk configs with parse logic and unit tests
5. **src/config/logging.rs** - Logging configs with parse logic and unit tests
6. **src/config/parse.rs** - Parse utilities and custom errors with unit tests
7. **tests/config.rs** - Integration tests
8. **config.example.toml** - Example configuration file
9. **.env.example** - Example environment variables

### Example Configuration Files

**config.example.toml:**

```toml
[exchanges.coinbase]
api_key = "your_coinbase_api_key"
api_secret = "your_coinbase_api_secret"
sandbox = true

[exchanges.binance]
api_key = "your_binance_api_key"
api_secret = "your_binance_api_secret"
testnet = true

[trading]
pair = "SOL/USDC"
spread_threshold = 0.002  # 0.2%
order_size = 10.0
cooldown_ms = 5000

[risk]
max_position_size = 1000.0
max_daily_trades = 100
stop_loss_threshold = 0.05
emergency_stop = false

[logging]
level = "info"
format = "json"  # or "pretty"
file_path = "logs/arb-bot.log"
rotation = "daily"
```

**.env.example:**

```bash
# Override any config value with environment variables
# Format: ARB_<SECTION>_<KEY>=value

ARB_EXCHANGES_COINBASE_API_KEY=override_key
ARB_EXCHANGES_BINANCE_API_KEY=override_key
ARB_TRADING_SPREAD_THRESHOLD=0.003
ARB_RISK_EMERGENCY_STOP=true
```

### Note: Revisit HIGH PRIORITY Core Principles before imlementing next Task and make sure you complete them. Especially point 7 & 8.

---

## 🎯 Task 3: Exchange Trait Abstraction

### Note: Revisit HIGH PRIORITY Core Principles before imlementing next Task and make sure you complete them. Especially point 7 & 8.

### Objective

Create a trait-based abstraction for exchange interactions, enabling polymorphism and testability.

### What to Build

1. Exchange trait with async methods
2. Common types (Price, Order, OrderResult)
3. Mock implementation for testing
4. Factory pattern for creating exchanges

### Implementation Specification

````xml
<task id="3" phase="1">
  <name>Exchange Trait & Mock Implementation</name>
  <priority>critical</priority>

  <test_first>
    <file>tests/exchange_trait.rs</file>
    <description>
      Write tests FIRST using MockExchange (will fail initially):
      - Subscribe to price updates
      - Parse ticker messages
      - Handle connection errors gracefully
      - Test async trait methods
      - Verify trait object usage (Box<dyn Exchange>)
      - Test exchange factory pattern
      - Test concurrent access to exchange

      Start by writing these tests. They'll fail until MockExchange is implemented.
    </description>
  </test_first>

  <implementation>
    <requirements>
      **Define Exchange Trait:**
      ```rust
      use async_trait::async_trait;
      use rust_decimal::Decimal;

      #[async_trait]
      pub trait Exchange: Send + Sync {
          /// Connect to the exchange WebSocket
          async fn connect(&mut self) -> Result<()>;

          /// Subscribe to ticker updates for a trading pair
          async fn subscribe_ticker(&mut self, pair: &str) -> Result<()>;

          /// Get the latest price for a pair
          async fn get_latest_price(&self, pair: &str) -> Result<Price>;

          /// Place a market order
          async fn place_order(&mut self, order: Order) -> Result<OrderResult>;

          /// Get account balance for an asset
          async fn get_balance(&self, asset: &str) -> Result<Decimal>;

          /// Get exchange name
          fn name(&self) -> &str;

          /// Check if connected
          fn is_connected(&self) -> bool;

          /// Disconnect from exchange
          async fn disconnect(&mut self) -> Result<()>;
      }
      ```

      **Define Common Types:**

      Price struct:
      ```rust
      use rust_decimal::Decimal;
      use chrono::{DateTime, Utc};

      #[derive(Debug, Clone, PartialEq)]
      pub struct Price {
          pub pair: String,
          pub bid: Decimal,
          pub ask: Decimal,
          pub last: Decimal,
          pub volume_24h: Decimal,
          pub timestamp: DateTime<Utc>,
      }

      impl Price {
          pub fn mid_price(&self) -> Decimal {
              (self.bid + self.ask) / Decimal::from(2)
          }

          pub fn spread(&self) -> Decimal {
              self.ask - self.bid
          }

          pub fn spread_percentage(&self) -> Decimal {
              if self.mid_price().is_zero() {
                  Decimal::ZERO
              } else {
                  (self.spread() / self.mid_price()) * Decimal::from(100)
              }
          }
      }
      ```
      Add unit tests for Price methods (mid_price, spread, spread_percentage)

      Order struct:
      ```rust
      #[derive(Debug, Clone, PartialEq)]
      pub enum OrderSide {
          Buy,
          Sell,
      }

      #[derive(Debug, Clone, PartialEq)]
      pub enum OrderType {
          Market,
          Limit { price: Decimal },
      }

      #[derive(Debug, Clone)]
      pub struct Order {
          pub pair: String,
          pub side: OrderSide,
          pub order_type: OrderType,
          pub quantity: Decimal,
      }

      impl Order {
          pub fn market_buy(pair: impl Into<String>, quantity: Decimal) -> Self {
              Self {
                  pair: pair.into(),
                  side: OrderSide::Buy,
                  order_type: OrderType::Market,
                  quantity,
              }
          }

          pub fn market_sell(pair: impl Into<String>, quantity: Decimal) -> Self {
              Self {
                  pair: pair.into(),
                  side: OrderSide::Sell,
                  order_type: OrderType::Market,
                  quantity,
              }
          }
      }
      ```
      Add unit tests for Order factory methods

      OrderResult struct:
      ```rust
      #[derive(Debug, Clone, PartialEq)]
      pub enum OrderStatus {
          Pending,
          Filled,
          PartiallyFilled,
          Cancelled,
          Failed,
      }

      #[derive(Debug, Clone)]
      pub struct OrderResult {
          pub order_id: String,
          pub status: OrderStatus,
          pub filled_quantity: Decimal,
          pub average_price: Option<Decimal>,
          pub fee: Decimal,
          pub fee_asset: String,
          pub timestamp: DateTime<Utc>,
      }

      impl OrderResult {
          pub fn is_complete(&self) -> bool {
              matches!(self.status, OrderStatus::Filled | OrderStatus::Cancelled | OrderStatus::Failed)
          }

          pub fn total_cost(&self) -> Option<Decimal> {
              self.average_price.map(|price| price * self.filled_quantity + self.fee)
          }
      }
      ```
      Add unit tests for OrderResult methods

      **Implement MockExchange:**
      ```rust
      use std::collections::HashMap;
      use std::sync::Arc;
      use parking_lot::RwLock;

      pub struct MockExchange {
          name: String,
          connected: bool,
          prices: Arc<RwLock<HashMap<String, Price>>>,
          balances: Arc<RwLock<HashMap<String, Decimal>>>,
          subscriptions: Vec<String>,
      }

      impl MockExchange {
          pub fn new(name: impl Into<String>) -> Self {
              Self {
                  name: name.into(),
                  connected: false,
                  prices: Arc::new(RwLock::new(HashMap::new())),
                  balances: Arc::new(RwLock::new(HashMap::new())),
                  subscriptions: Vec::new(),
              }
          }

          pub fn set_price(&self, pair: &str, price: Price) {
              self.prices.write().insert(pair.to_string(), price);
          }

          pub fn set_balance(&self, asset: &str, amount: Decimal) {
              self.balances.write().insert(asset.to_string(), amount);
          }
      }

      #[async_trait]
      impl Exchange for MockExchange {
          // Implement all trait methods
          // Make them work with internal state
      }
      ```
      Add unit tests for MockExchange behavior

      **Exchange Factory:**
      ```rust
      pub trait ExchangeFactory {
          fn create_exchange(&self, name: &str, config: &ExchangeConfig) -> Result<Box<dyn Exchange>>;
      }

      pub struct DefaultExchangeFactory;

      impl ExchangeFactory for DefaultExchangeFactory {
          fn create_exchange(&self, name: &str, config: &ExchangeConfig) -> Result<Box<dyn Exchange>> {
              match name {
                  "binance" => Ok(Box::new(BinanceExchange::new(config)?)),
                  "coinbase" => Ok(Box::new(CoinbaseExchange::new(config)?)),
                  "mock" => Ok(Box::new(MockExchange::new(name))),
                  _ => Err(ArbitrageError::ConfigError {
                      field: "exchange".into(),
                      reason: format!("Unknown exchange: {}", name),
                  }),
              }
          }
      }
      ```
      Add unit tests for factory

      - Use Arc<RwLock<T>> for thread-safe state sharing in Mock
      - Add comprehensive documentation with examples
      - Add unit tests for all domain structs
    </requirements>

    <rust_patterns>
      - Trait objects: Box<dyn Exchange>
      - Generics with trait bounds where appropriate
      - Interior mutability for shared state (Arc<RwLock<T>>)
      - Zero-copy string handling with &str
      - Builder pattern for complex types
    </rust_patterns>
  </implementation>

  <validation>
    - All trait methods work with MockExchange
    - Trait objects work correctly (Box<dyn Exchange>)
    - All tests pass
    - No clippy warnings
    - Documentation complete
  </validation>

  <completion_criteria>
    ❌ Tests written and failing initially (TDD)
    💻 Exchange trait and types implemented
    ✅ MockExchange working correctly
    ✅ All tests passing
    ✅ Unit tests for all domain types
    ✅ Documentation complete
    ✅ Human verification complete
  </completion_criteria>
</task>
````

### Files to Create

1. **src/exchanges/mod.rs** - Exchange trait definition and re-exports
2. **src/exchanges/types.rs** - Common types (Price, Order, OrderResult) with unit tests
3. **src/exchanges/mock.rs** - Mock implementation with unit tests
4. **src/exchanges/factory.rs** - Factory pattern with unit tests
5. **tests/exchange_trait.rs** - Integration tests

### Note: Revisit HIGH PRIORITY Core Principles before imlementing next Task and make sure you complete them. Especially point 7 & 8.

---

## 🎯 Task 4: WebSocket Price Feed Manager

### Note: Revisit HIGH PRIORITY Core Principles before imlementing next Task and make sure you complete them. Especially point 7 & 8.

### Objective

Create a generic WebSocket manager that handles connections, reconnections, and message parsing.

### What to Build

1. Generic WebSocket connection manager
2. Reconnection strategy with exponential backoff
3. Message parsing trait
4. Health check mechanism (ping/pong)

### Implementation Specification

````xml
<task id="4" phase="1">
  <name>WebSocket Manager with Reconnection Logic</name>
  <priority>high</priority>

  <test_first>
    <file>tests/websocket_manager.rs</file>
    <description>
      Write tests FIRST using mock WebSocket (will fail initially):
      - Establish connection successfully
      - Parse incoming price messages
      - Handle connection drops gracefully
      - Automatic reconnection with exponential backoff
      - Message queue management (bounded capacity)
      - Concurrent message handling
      - Graceful shutdown
      - Health check (ping/pong) mechanism

      These tests define the expected behavior before implementation exists.
    </description>
  </test_first>

  <implementation>
    <requirements>
      **Generic WebSocket Manager:**
      ```rust
      use tokio_tungstenite::{connect_async, tungstenite::Message};
      use futures_util::{StreamExt, SinkExt};
      use tokio::sync::{mpsc, broadcast};

      pub struct WebSocketManager<P: MessageParser> {
          url: String,
          parser: P,
          reconnect_strategy: ReconnectionStrategy,
          message_tx: broadcast::Sender<P::Output>,
          health_check_interval: Duration,
      }

      impl<P: MessageParser> WebSocketManager<P> {
          pub fn new(
              url: String,
              parser: P,
              reconnect_strategy: ReconnectionStrategy,
          ) -> (Self, broadcast::Receiver<P::Output>) {
              let (message_tx, message_rx) = broadcast::channel(100);

              let manager = Self {
                  url,
                  parser,
                  reconnect_strategy,
                  message_tx,
                  health_check_interval: Duration::from_secs(30),
              };

              (manager, message_rx)
          }

          pub async fn run(&mut self) -> Result<()> {
              loop {
                  match self.connect_and_run().await {
                      Ok(_) => {
                          tracing::info!("WebSocket connection closed normally");
                          break Ok(());
                      }
                      Err(e) => {
                          tracing::error!("WebSocket error: {}", e);

                          if !self.reconnect_strategy.should_retry() {
                              break Err(e);
                          }

                          let delay = self.reconnect_strategy.next_delay();
                          tracing::info!("Reconnecting in {:?}", delay);
                          tokio::time::sleep(delay).await;
                      }
                  }
              }
          }

          async fn connect_and_run(&mut self) -> Result<()> {
              let (ws_stream, _) = connect_async(&self.url).await?;
              let (mut write, mut read) = ws_stream.split();

              let mut ping_interval = tokio::time::interval(self.health_check_interval);

              loop {
                  tokio::select! {
                      Some(message) = read.next() => {
                          match message? {
                              Message::Text(text) => {
                                  match self.parser.parse(&text) {
                                      Ok(parsed) => {
                                          let _ = self.message_tx.send(parsed);
                                      }
                                      Err(e) => {
                                          tracing::warn!("Failed to parse message: {}", e);
                                      }
                                  }
                              }
                              Message::Ping(data) => {
                                  write.send(Message::Pong(data)).await?;
                              }
                              Message::Close(_) => {
                                  tracing::info!("Received close frame");
                                  break;
                              }
                              _ => {}
                          }
                      }
                      _ = ping_interval.tick() => {
                          write.send(Message::Ping(vec![])).await?;
                      }
                  }
              }

              Ok(())
          }
      }
      ```

      **MessageParser Trait:**
      ```rust
      pub trait MessageParser: Send + Sync + Clone {
          type Output: Send + Clone;

          fn parse(&self, message: &str) -> Result<Self::Output>;
      }
      ```
      Add unit tests for different parser implementations

      **Reconnection Strategy:**
      ```rust
      #[derive(Debug, Clone)]
      pub struct ReconnectionStrategy {
          max_retries: Option<u32>,
          current_retry: u32,
          initial_delay: Duration,
          max_delay: Duration,
          multiplier: f64,
      }

      impl ReconnectionStrategy {
          pub fn new(
              max_retries: Option<u32>,
              initial_delay: Duration,
              max_delay: Duration,
          ) -> Self {
              Self {
                  max_retries,
                  current_retry: 0,
                  initial_delay,
                  max_delay,
                  multiplier: 2.0,
              }
          }

          pub fn exponential_backoff() -> Self {
              Self::new(
                  Some(10),
                  Duration::from_secs(1),
                  Duration::from_secs(60),
              )
          }

          pub fn should_retry(&self) -> bool {
              match self.max_retries {
                  Some(max) => self.current_retry < max,
                  None => true,
              }
          }

          pub fn next_delay(&mut self) -> Duration {
              let delay = self.initial_delay.mul_f64(
                  self.multiplier.powi(self.current_retry as i32)
              );

              self.current_retry += 1;
              delay.min(self.max_delay)
          }

          pub fn reset(&mut self) {
              self.current_retry = 0;
          }
      }
      ```
      Add unit tests for:
      - Exponential backoff calculation
      - Max retries enforcement
      - Delay capping at max_delay
      - Reset functionality

      **Message Queue:**
      - Use tokio::sync::broadcast for fan-out to multiple subscribers
      - Bounded capacity (default 100 messages)
      - Handle slow consumers gracefully (drop old messages)

      - Add comprehensive documentation
      - Add unit tests for all components
    </requirements>

    <async_patterns>
      - tokio::spawn for background tasks
      - mpsc/broadcast channels for message passing
      - tokio::select! for handling multiple futures
      - Graceful shutdown with cancellation tokens
    </async_patterns>
  </implementation>

  <validation>
    - Connection establishes successfully
    - Reconnection works with exponential backoff
    - Messages parse and broadcast correctly
    - Health checks work (ping/pong)
    - All tests pass
    - No clippy warnings
  </validation>

  <completion_criteria>
    ❌ Tests written and failing initially (TDD)
    💻 WebSocket manager implemented
    ✅ Reconnection strategy working
    ✅ All tests passing
    ✅ Unit tests for backoff and parsing
    ✅ Documentation complete
    ✅ Human verification complete
  </completion_criteria>
</task>
````

### Files to Create

1. **src/websocket/mod.rs** - Module exports
2. **src/websocket/manager.rs** - WebSocket manager with unit tests
3. **src/websocket/reconnect.rs** - Reconnection strategy with unit tests
4. **src/websocket/parser.rs** - MessageParser trait with example implementations and unit tests
5. **tests/websocket_manager.rs** - Integration tests

### Note: Revisit HIGH PRIORITY Core Principles before imlementing next Task and make sure you complete them. Especially point 7 & 8.

---

## 🎯 Task 5: Shared Price State Manager

### Note: Revisit HIGH PRIORITY Core Principles before imlementing next Task and make sure you complete them. Especially point 7 & 8.

### Objective

Create thread-safe shared state for storing latest prices from multiple exchanges.

### What to Build

1. Thread-safe price storage using RwLock
2. Price staleness detection
3. Spread calculation between exchanges
4. Price history tracking (optional)

### Implementation Specification

````xml
<task id="5" phase="1">
  <name>Thread-Safe Price State with RwLock</name>
  <priority>high</priority>

  <test_first>
    <file>tests/price_state.rs</file>
    <description>
      Write concurrent tests FIRST (will fail initially):
      - Multiple writers updating prices simultaneously
      - Multiple readers accessing prices concurrently
      - Timestamp tracking for staleness detection
      - Price history (last N prices) per exchange
      - Atomic read-modify-write operations
      - Spread calculation between two exchanges
      - No data races (use tokio::test for concurrent scenarios)

      Write these before implementing PriceState.
    </description>
  </test_first>

  <implementation>
    <requirements>
      **ExchangeId Enum:**
      ```rust
      #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
      pub enum ExchangeId {
          Binance,
          Coinbase,
      }

      impl std::fmt::Display for ExchangeId {
          fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
              match self {
                  ExchangeId::Binance => write!(f, "binance"),
                  ExchangeId::Coinbase => write!(f, "coinbase"),
              }
          }
      }
      ```
      Add unit tests for Display implementation

      **PriceData Struct:**
      ```rust
      use std::time::Instant;

      #[derive(Debug, Clone)]
      pub struct PriceData {
          pub price: Price,
          pub timestamp: Instant,
          pub sequence: u64,
      }

      impl PriceData {
          pub fn new(price: Price, sequence: u64) -> Self {
              Self {
                  price,
                  timestamp: Instant::now(),
                  sequence,
              }
          }

          pub fn age(&self) -> Duration {
              self.timestamp.elapsed()
          }

          pub fn is_stale(&self, max_age: Duration) -> bool {
              self.age() > max_age
          }
      }
      ```
      Add unit tests for:
      - PriceData creation
      - Age calculation
      - Staleness detection

      **PriceState Manager:**
      ```rust
      use std::collections::HashMap;
      use std::sync::Arc;
      use parking_lot::RwLock;

      #[derive(Clone)]
      pub struct PriceState {
          prices: Arc<RwLock<HashMap<(ExchangeId, String), PriceData>>>,
          max_age: Duration,  // Max age for staleness detection (e.g., 5 seconds)
          // Max time difference between prices for comparison = max_age / 2
      }

      impl PriceState {
          pub fn new(max_age: Duration) -> Self {
              Self {
                  prices: Arc::new(RwLock::new(HashMap::new())),
                  max_age,
              }
          }

          pub fn update_price(
              &self,
              exchange: ExchangeId,
              pair: impl Into<String>,
              price: Price,
              sequence: u64,
          ) {
              let key = (exchange, pair.into());
              let price_data = PriceData::new(price, sequence);

              self.prices.write().insert(key, price_data);
          }

          pub fn get_price(
              &self,
              exchange: ExchangeId,
              pair: &str,
          ) -> Option<PriceData> {
              let key = (exchange, pair.to_string());
              self.prices.read().get(&key).cloned()
          }

          pub fn get_spread(
              &self,
              ex1: ExchangeId,
              ex2: ExchangeId,
              pair: &str,
          ) -> Option<Decimal> {
              let price1 = self.get_price(ex1, pair)?;
              let price2 = self.get_price(ex2, pair)?;

              // Check staleness - reject if either price is too old
              if price1.is_stale(self.max_age) || price2.is_stale(self.max_age) {
                  return None;
              }

              // Check max time difference - reject if prices captured too far apart
              // This ensures we compare prices from similar time windows
              let time_diff = if price1.timestamp > price2.timestamp {
                  price1.timestamp.duration_since(price2.timestamp).unwrap()
              } else {
                  price2.timestamp.duration_since(price1.timestamp).unwrap()
              };

              // Max time difference: half of max_age (e.g., 2.5s if max_age is 5s)
              // This ensures prices are from similar time windows
              let max_time_diff = self.max_age / 2;
              if time_diff > max_time_diff {
                  return None;
              }

              let mid1 = price1.price.mid_price();
              let mid2 = price2.price.mid_price();

              Some((mid2 - mid1).abs())
          }

          pub fn get_spread_percentage(
              &self,
              ex1: ExchangeId,
              ex2: ExchangeId,
              pair: &str,
          ) -> Option<Decimal> {
              let spread = self.get_spread(ex1, ex2, pair)?;
              let price1 = self.get_price(ex1, pair)?;
              let mid1 = price1.price.mid_price();

              if mid1.is_zero() {
                  return None;
              }

              Some((spread / mid1) * Decimal::from(100))
          }

          pub fn is_stale(
              &self,
              exchange: ExchangeId,
              pair: &str,
          ) -> bool {
              match self.get_price(exchange, pair) {
                  Some(data) => data.is_stale(self.max_age),
                  None => true,
              }
          }

          pub fn remove_stale_prices(&self) -> usize {
              let mut prices = self.prices.write();
              let initial_count = prices.len();

              prices.retain(|_, data| !data.is_stale(self.max_age));

              initial_count - prices.len()
          }

          pub fn get_all_prices(&self) -> HashMap<(ExchangeId, String), PriceData> {
              self.prices.read().clone()
          }

          pub fn clear(&self) {
              self.prices.write().clear();
          }
      }
      ```
      Add unit tests for:
      - Individual PriceState methods
      - Edge cases (empty state, missing exchange)
      - Staleness detection logic
      - Max time difference rejection (prices captured too far apart)
      - Spread calculation correctness
      - Concurrent updates and reads

      - Use parking_lot::RwLock for better performance than std::sync::RwLock
      - Consider dashmap as alternative for high-concurrency scenarios
      - Add comprehensive documentation
    </requirements>
  </implementation>

  <validation>
    - Concurrent access works correctly
    - Staleness detection accurate
    - Spread calculations correct
    - No data races
    - All tests pass
    - No clippy warnings
  </validation>

  <completion_criteria>
    ❌ Tests written and failing initially (TDD)
    💻 PriceState implemented with thread safety
    ✅ All tests passing (including concurrent tests)
    ✅ Unit tests for all methods
    ✅ Documentation complete
    ✅ Human verification complete
  </completion_criteria>
</task>
````

### Files to Create

1. **src/state/mod.rs** - Module exports
2. **src/state/price.rs** - PriceState manager with unit tests
3. **src/state/types.rs** - ExchangeId, PriceData with unit tests
4. **tests/price_state.rs** - Integration tests (concurrent scenarios)

### Note: Revisit HIGH PRIORITY Core Principles before imlementing next Task and make sure you complete them. Especially point 7 & 8.

---

## 🎯 Task 6: Binance WebSocket Integration

### Note: Revisit HIGH PRIORITY Core Principles before imlementing next Task and make sure you complete them. Especially point 7 & 8.

### Objective

Implement concrete Binance WebSocket client using the Exchange trait.

### What to Build

1. Binance-specific Exchange implementation
2. WebSocket ticker stream subscription
3. Message parsing for Binance format
4. ~~REST API for order placement and balance queries~~ **DEFERRED** - Will be implemented in arbitrage logic phase when order execution is needed

### Implementation Specification

````xml
<task id="6" phase="1">
  <name>Binance Exchange Implementation</name>
  <priority>high</priority>

  <test_first>
    <file>tests/binance.rs</file>
    <description>
      Write tests FIRST (will fail initially):
      - Connect to Binance testnet WebSocket
      - Subscribe to SOLUSDC ticker stream
      - Parse price update messages correctly
      - Handle Binance-specific error codes
      - Respect rate limits
      - Use mock for unit tests, real testnet for integration

      **Note:** REST API tests (authentication, order placement, balance queries) are deferred until arbitrage logic phase when order execution is needed.

      Start with these tests before implementing BinanceExchange.
    </description>
  </test_first>

  <implementation>
    <requirements>
      **Binance Exchange Implementation:**
      ```rust
      pub struct BinanceExchange {
          name: String,
          config: BinanceConfig,
          ws_manager: Option<WebSocketManager<BinanceParser>>,
          // rest_client: BinanceRestClient,  // DEFERRED - Will be added in arbitrage logic phase
          price_rx: Option<broadcast::Receiver<Price>>,
          latest_prices: Arc<RwLock<HashMap<String, Price>>>,
      }

      impl BinanceExchange {
          pub fn new(config: BinanceConfig) -> Result<Self> {
              // REST client initialization deferred until arbitrage logic phase
              // let rest_client = BinanceRestClient::new(
              //     config.api_key.clone(),
              //     config.api_secret.clone(),
              //     config.testnet,
              // );

              Ok(Self {
                  name: "binance".to_string(),
                  config,
                  ws_manager: None,
                  // rest_client,  // DEFERRED
                  price_rx: None,
                  latest_prices: Arc::new(RwLock::new(HashMap::new())),
              })
          }
      }

      #[async_trait]
      impl Exchange for BinanceExchange {
          async fn connect(&mut self) -> Result<()> {
              let url = if self.config.testnet {
                  "wss://testnet.binance.vision/ws"
              } else {
                  "wss://stream.binance.com:9443/ws"
              };

              let parser = BinanceParser::new();
              let reconnect_strategy = ReconnectionStrategy::exponential_backoff();

              let (mut manager, price_rx) = WebSocketManager::new(
                  url.to_string(),
                  parser,
                  reconnect_strategy,
              );

              self.price_rx = Some(price_rx);

              // Spawn background task to run WebSocket
              let latest_prices = self.latest_prices.clone();
              tokio::spawn(async move {
                  if let Err(e) = manager.run().await {
                      tracing::error!("WebSocket manager error: {}", e);
                  }
              });

              // Spawn background task to update latest prices
              if let Some(mut rx) = self.price_rx.take() {
                  let prices = self.latest_prices.clone();
                  tokio::spawn(async move {
                      while let Ok(price) = rx.recv().await {
                          prices.write().insert(price.pair.clone(), price);
                      }
                  });
              }

              Ok(())
          }

          async fn subscribe_ticker(&mut self, pair: &str) -> Result<()> {
              // Convert pair format: SOL/USDC -> solusdc
              let symbol = pair.replace("/", "").to_lowercase();

              let subscribe_msg = json!({
                  "method": "SUBSCRIBE",
                  "params": [format!("{}@ticker", symbol)],
                  "id": 1
              });

              // Send subscription message through WebSocket
              // (Implementation depends on how you expose WS write channel)

              Ok(())
          }

          async fn get_latest_price(&self, pair: &str) -> Result<Price> {
              let prices = self.latest_prices.read();
              prices.get(pair)
                  .cloned()
                  .ok_or_else(|| ArbitrageError::ExchangeError {
                      exchange: self.name.clone(),
                      message: format!("No price data for {}", pair),
                      code: None,
                  })
          }

          async fn place_order(&mut self, order: Order) -> Result<OrderResult> {
              // REST API not implemented yet - WebSocket price feed only
              // Will be implemented in arbitrage logic phase when order execution is needed
              Err(ArbitrageError::ExchangeError {
                  exchange: self.name.clone(),
                  message: "Trading not implemented yet - WebSocket price feed only".to_string(),
                  code: None,
              })
          }

          async fn get_balance(&self, asset: &str) -> Result<Decimal> {
              // REST API not implemented yet - WebSocket price feed only
              // Will be implemented in arbitrage logic phase when order execution is needed
              Err(ArbitrageError::ExchangeError {
                  exchange: self.name.clone(),
                  message: "Balance queries not implemented yet - WebSocket price feed only".to_string(),
                  code: None,
              })
          }

          fn name(&self) -> &str {
              &self.name
          }

          fn is_connected(&self) -> bool {
              // Check if we have recent price data
              !self.latest_prices.read().is_empty()
          }

          async fn disconnect(&mut self) -> Result<()> {
              // Clean up WebSocket connection
              Ok(())
          }
      }
      ```

      **Binance Message Parser:**
      ```rust
      #[derive(Clone)]
      pub struct BinanceParser;

      impl BinanceParser {
          pub fn new() -> Self {
              Self
          }
      }

      impl MessageParser for BinanceParser {
          type Output = Price;

          fn parse(&self, message: &str) -> Result<Self::Output> {
              let value: serde_json::Value = serde_json::from_str(message)?;

              // Binance ticker format:
              // {
              //   "e": "24hrTicker",
              //   "s": "SOLUSDC",
              //   "c": "143.50",  // Close price (last)
              //   "b": "143.48",  // Best bid
              //   "a": "143.52",  // Best ask
              //   "v": "1234567.89"  // Volume
              // }

              if value["e"] != "24hrTicker" {
                  return Err(ArbitrageError::ParseError {
                      message: "Not a ticker message".into(),
                      input: Some(message.to_string()),
                  });
              }

              let symbol = value["s"].as_str()
                  .ok_or_else(|| ArbitrageError::ParseError {
                      message: "Missing symbol".into(),
                      input: None,
                  })?;

              // Convert symbol back to pair format: SOLUSDC -> SOL/USDC
              let pair = format!("{}/{}", &symbol[..3], &symbol[3..]);

              let last = Decimal::from_str(value["c"].as_str().unwrap_or("0"))?;
              let bid = Decimal::from_str(value["b"].as_str().unwrap_or("0"))?;
              let ask = Decimal::from_str(value["a"].as_str().unwrap_or("0"))?;
              let volume = Decimal::from_str(value["v"].as_str().unwrap_or("0"))?;

              Ok(Price {
                  pair,
                  bid,
                  ask,
                  last,
                  volume_24h: volume,
                  timestamp: Utc::now(),
              })
          }
      }
      ```
      Add unit tests for:
      - Valid ticker message parsing
      - Invalid message handling
      - Symbol format conversion
      - Edge cases (missing fields, invalid numbers)

      **Binance REST Client:**
      ```rust
      // DEFERRED - Will be implemented in arbitrage logic phase when order execution is needed
      // REST API client stub exists in src/exchanges/binance/rest.rs
      // Full implementation will include:
      // - BinanceRestClient struct
      // - place_market_order() method
      // - get_balance() method
      // - Request signing (HMAC SHA256)
      // - Rate limit handling
      // - Server time synchronization
      ```

      **Note:** REST API implementation is deferred until arbitrage logic phase. The Exchange trait methods `place_order()` and `get_balance()` currently return errors indicating WebSocket-only mode.

      **Binance-Specific Types:**
      ```rust
      // DEFERRED - Will be implemented in arbitrage logic phase
      // Types needed for REST API:
      // - BinanceOrderResponse
      // - BinanceAccountInfo
      // - BinanceBalance
      // - Type conversions to OrderResult
      ```

      - Handle Binance rate limits (weight-based) - **DEFERRED**
      - Server time synchronization for signatures - **DEFERRED**
      - Add comprehensive documentation
    </requirements>

    <binance_specifics>
      - Ticker stream format: solusdc@ticker
      - WebSocket URL: wss://stream.binance.com:9443/ws (production)
      - WebSocket URL: wss://testnet.binance.vision/ws (testnet)
      - REST API: https://api.binance.com (production)
      - REST API: https://testnet.binance.vision (testnet)
      - Authentication: HMAC SHA256 signatures
      - Rate limits: Weight-based system (1200 weight per minute)
      - Server time sync required (use /api/v3/time endpoint)
    </binance_specifics>
  </implementation>

  <validation>
    - Connection to testnet works
    - Ticker subscription works
    - Message parsing accurate
    - All tests pass
    - No clippy warnings
    - ~~Order placement works (testnet)~~ **DEFERRED**
    - ~~Balance queries work~~ **DEFERRED**
  </validation>

  <completion_criteria>
    ❌ Tests written and failing initially (TDD)
    💻 Binance WebSocket integration implemented
    ✅ WebSocket working (price feeds)
    ❌ REST API working - **DEFERRED until arbitrage logic phase**
    ✅ All tests passing (WebSocket tests)
    ✅ Unit tests for parser
    ❌ Unit tests for REST - **DEFERRED**
    ✅ Documentation complete
    ✅ Human verification with testnet (WebSocket verified)
  </completion_criteria>
</task>
````

### Files to Create

1. **src/exchanges/binance/mod.rs** - Module exports and BinanceExchange struct
2. **src/exchanges/binance/exchange.rs** - BinanceExchange implementation (WebSocket focus)
3. **src/exchanges/binance/parser.rs** - Message parser with unit tests
4. **src/exchanges/binance/types.rs** - Binance-specific types with unit tests
5. **src/exchanges/binance/rest.rs** - REST API client stub (implementation deferred)
6. **tests/binance.rs** - Integration tests (WebSocket focus)

**Note:** REST API implementation files will be completed in arbitrage logic phase when order execution is needed.

### Note: Revisit HIGH PRIORITY Core Principles before imlementing next Task and make sure you complete them. Especially point 7 & 8.

---

## 🎯 Task 7: Coinbase WebSocket Integration

### Objective

Implement Coinbase Advanced Trade API integration using the Exchange trait.

### What to Build

1. Coinbase-specific Exchange implementation
2. WebSocket ticker subscription
3. Message parsing for Coinbase format
4. ~~JWT authentication for REST API~~ **DEFERRED** - Will be implemented in arbitrage logic phase when order execution is needed

### Implementation Specification

````xml
<task id="7" phase="1">
  <name>Coinbase Exchange Implementation</name>
  <priority>high</priority>

  <test_first>
    <file>tests/coinbase.rs</file>
    <description>
      Write tests FIRST (will fail initially):
      - Connect to Coinbase WebSocket
      - Subscribe to SOL-USDC ticker
      - Parse Coinbase ticker messages
      - Handle product_id format conversion
      - Use mock for unit tests, real sandbox for integration

      **Note:** REST API tests (authentication, JWT token generation, order placement, balance queries) are deferred until arbitrage logic phase when order execution is needed.

      These tests define expected behavior before implementation.
    </description>
  </test_first>

  <implementation>
    <requirements>
      **Coinbase Exchange Implementation:**
      ```rust
      pub struct CoinbaseExchange {
          name: String,
          config: CoinbaseConfig,
          ws_manager: Option<WebSocketManager<CoinbaseParser>>,
          // rest_client: CoinbaseRestClient,  // DEFERRED - Will be added in arbitrage logic phase
          price_rx: Option<broadcast::Receiver<Price>>,
          latest_prices: Arc<RwLock<HashMap<String, Price>>>,
      }

      impl CoinbaseExchange {
          pub fn new(config: CoinbaseConfig) -> Result<Self> {
              // REST client initialization deferred until arbitrage logic phase
              // let rest_client = CoinbaseRestClient::new(
              //     config.api_key.clone(),
              //     config.api_secret.clone(),
              //     config.sandbox,
              // );

              Ok(Self {
                  name: "coinbase".to_string(),
                  config,
                  ws_manager: None,
                  // rest_client,  // DEFERRED
                  price_rx: None,
                  latest_prices: Arc::new(RwLock::new(HashMap::new())),
              })
          }

          fn pair_to_product_id(pair: &str) -> String {
              // Convert SOL/USDC to SOL-USDC
              pair.replace("/", "-")
          }

          fn product_id_to_pair(product_id: &str) -> String {
              // Convert SOL-USDC to SOL/USDC
              product_id.replace("-", "/")
          }
      }

      #[async_trait]
      impl Exchange for CoinbaseExchange {
          async fn connect(&mut self) -> Result<()> {
              let url = "wss://advanced-trade-ws.coinbase.com";

              let parser = CoinbaseParser::new();
              let reconnect_strategy = ReconnectionStrategy::exponential_backoff();

              let (mut manager, price_rx) = WebSocketManager::new(
                  url.to_string(),
                  parser,
                  reconnect_strategy,
              );

              self.price_rx = Some(price_rx);

              // Similar spawning as Binance
              // ...

              Ok(())
          }

          async fn subscribe_ticker(&mut self, pair: &str) -> Result<()> {
              let product_id = Self::pair_to_product_id(pair);

              let subscribe_msg = json!({
                  "type": "subscribe",
                  "product_ids": [product_id],
                  "channels": ["ticker"]
              });

              // Send subscription message
              // ...

              Ok(())
          }

          // Implement other Exchange trait methods...
      }
      ```

      **Coinbase Message Parser:**
      ```rust
      #[derive(Clone)]
      pub struct CoinbaseParser;

      impl CoinbaseParser {
          pub fn new() -> Self {
              Self
          }
      }

      impl MessageParser for CoinbaseParser {
          type Output = Price;

          fn parse(&self, message: &str) -> Result<Self::Output> {
              let value: serde_json::Value = serde_json::from_str(message)?;

              // Coinbase ticker format:
              // {
              //   "type": "ticker",
              //   "product_id": "SOL-USDC",
              //   "price": "143.50",
              //   "best_bid": "143.48",
              //   "best_ask": "143.52",
              //   "volume_24h": "1234567.89",
              //   "time": "2025-10-30T12:00:00.000000Z"
              // }

              if value["type"] != "ticker" {
                  // Could be subscriptions message or error
                  return Err(ArbitrageError::ParseError {
                      message: "Not a ticker message".into(),
                      input: Some(message.to_string()),
                  });
              }

              let product_id = value["product_id"].as_str()
                  .ok_or_else(|| ArbitrageError::ParseError {
                      message: "Missing product_id".into(),
                      input: None,
                  })?;

              let pair = product_id.replace("-", "/");

              let last = Decimal::from_str(value["price"].as_str().unwrap_or("0"))?;
              let bid = Decimal::from_str(value["best_bid"].as_str().unwrap_or("0"))?;
              let ask = Decimal::from_str(value["best_ask"].as_str().unwrap_or("0"))?;
              let volume = Decimal::from_str(value["volume_24h"].as_str().unwrap_or("0"))?;

              let timestamp = value["time"].as_str()
                  .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                  .map(|dt| dt.with_timezone(&Utc))
                  .unwrap_or_else(Utc::now);

              Ok(Price {
                  pair,
                  bid,
                  ask,
                  last,
                  volume_24h: volume,
                  timestamp,
              })
          }
      }
      ```
      Add unit tests for parser

      **Coinbase REST Client with JWT Auth:**
      ```rust
      // DEFERRED - Will be implemented in arbitrage logic phase when order execution is needed
      // REST API client stub exists in src/exchanges/coinbase/rest.rs
      // Full implementation will include:
      // - CoinbaseRestClient struct
      // - JWT token generation (ES256 algorithm)
      // - place_market_order() method
      // - get_balance() method
      // - Request signing with JWT
      // - Rate limit handling
      ```

      **Note:** REST API implementation is deferred until arbitrage logic phase. The Exchange trait methods `place_order()` and `get_balance()` currently return errors indicating WebSocket-only mode.

      **Coinbase-Specific Types:**
      ```rust
      // DEFERRED - Will be implemented in arbitrage logic phase
      // Types needed for REST API:
      // - CoinbaseOrderResponse
      // - CoinbaseAccountsResponse
      // - CoinbaseBalance
      // - Type conversions to OrderResult
      ```

      Add unit tests for (when implemented):
      - JWT token generation
      - Request formatting
      - URL construction

      - Product ID format handling (SOL-USDC vs SOL/USDC)
      - Rate limit: 10 requests per second
      - Add comprehensive documentation
    </requirements>

    <coinbase_specifics>
      - WebSocket URL: wss://advanced-trade-ws.coinbase.com
      - REST API: https://api.coinbase.com (production)
      - REST API: https://api-public.sandbox.exchange.coinbase.com (sandbox)
      - Authentication: JWT with ES256 (not HMAC like Binance)
      - Subscribe format: {"type":"subscribe","product_ids":["SOL-USDC"],"channels":["ticker"]}
      - Ticker format: {"type":"ticker","product_id":"SOL-USDC","price":"143.50",...}
      - Rate limit: 10 req/sec
    </coinbase_specifics>
  </implementation>

  <validation>
    - Connection works
    - Ticker subscription works
    - Message parsing accurate
    - All tests pass
    - No clippy warnings
    - ~~JWT generation correct~~ **DEFERRED**
    - ~~Order placement works (sandbox)~~ **DEFERRED**
    - ~~Balance queries work~~ **DEFERRED**
  </validation>

  <completion_criteria>
    ❌ Tests written and failing initially (TDD)
    💻 Coinbase WebSocket integration implemented
    ✅ WebSocket working (price feeds)
    ❌ REST API working - **DEFERRED until arbitrage logic phase**
    ❌ JWT auth working correctly - **DEFERRED**
    ✅ All tests passing (WebSocket tests)
    ✅ Unit tests for parser
    ❌ Unit tests for auth, REST - **DEFERRED**
    ✅ Documentation complete
    ✅ Human verification with sandbox (WebSocket verified)
  </completion_criteria>
</task>
````

### Files to Create

1. **src/exchanges/coinbase/mod.rs** - Module exports and CoinbaseExchange struct
2. **src/exchanges/coinbase/exchange.rs** - CoinbaseExchange implementation (WebSocket focus)
3. **src/exchanges/coinbase/parser.rs** - Message parser with unit tests
4. **src/exchanges/coinbase/types.rs** - Coinbase-specific types with unit tests
5. **src/exchanges/coinbase/rest.rs** - REST API client stub (implementation deferred)
6. **src/exchanges/coinbase/auth.rs** - JWT authentication stub (implementation deferred)
7. **tests/coinbase.rs** - Integration tests (WebSocket focus)

**Note:** REST API and JWT authentication implementation files will be completed in arbitrage logic phase when order execution is needed.

### Note: Revisit HIGH PRIORITY Core Principles before imlementing next Task and make sure you complete them. Especially point 7 & 8.

---

## 🎯 Task 7.5: Coinbase REST API Integration

### Objective

Implement Coinbase Advanced Trade REST API client for order execution and balance queries. This enables the arbitrage bot to execute trades on Coinbase.

### What to Build

1. JWT authentication with ES256 (ECDSA) algorithm
2. REST API client for Coinbase Advanced Trade API
3. Market order placement (`POST /api/v3/brokerage/orders`)
4. Account balance queries (`GET /api/v3/brokerage/accounts`)
5. Error handling for API responses
6. Rate limiting (10 requests per second)
7. Integration with existing `Exchange` trait methods

### Implementation Specification

````xml
<task id="7.5" phase="1">
  <name>Coinbase REST API Integration</name>
  <priority>high</priority>

  <test_first>
    <file>tests/coinbase_rest.rs</file>
    <description>
      Write tests FIRST (will fail initially):
      - JWT token generation with ES256
      - JWT token contains correct claims (iss, sub, exp, nbf)
      - REST API client initialization
      - Market order placement (mock HTTP responses)
      - Balance query (mock HTTP responses)
      - Error handling for API errors (400, 401, 403, 429, 500)
      - Rate limit handling (429 responses)
      - Product ID format conversion (SOL/USDC ↔ SOL-USDC)
      - OrderResult conversion from Coinbase response
      - Balance conversion from Coinbase response
      - Sandbox integration tests (with real API, marked #[ignore])
      - Production integration tests (with real API, marked #[ignore])

      These tests define expected behavior before implementation exists.
    </description>
  </test_first>

  <implementation>
    <dependencies>
      Add to Cargo.toml:
      ```toml
      # JWT signing (ES256 algorithm)
      jsonwebtoken = "9.3"
      p256 = "0.13"  # ECDSA P-256 curve support
      # OR use ring or other crypto library that supports ES256

      # For parsing EC private keys
      pkcs8 = "0.10"  # For parsing PEM-encoded EC private keys
      # OR use openssl crate if preferred
      ```

      Note: Coinbase uses ES256 (ECDSA P-256) for JWT signing, not HS256 (HMAC).
      The API secret is an EC private key in PEM format.
    </dependencies>

    <file>src/exchanges/coinbase/auth.rs</file>
    <requirements>
      **JWT Authentication:**
      ```rust
      use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
      use serde::{Deserialize, Serialize};
      use chrono::{Duration, Utc};

      /// JWT claims for Coinbase Advanced Trade API
      #[derive(Debug, Serialize, Deserialize)]
      struct CoinbaseClaims {
          sub: String,  // API key name (e.g., "organizations/org-id/apiKeys/key-id")
          iss: String,  // "coinbase-cloud"
          nbf: i64,     // Not before timestamp
          exp: i64,     // Expiration timestamp
      }

      pub struct CoinbaseAuth {
          api_key: String,      // Full API key path
          private_key: String,  // EC private key in PEM format
      }

      impl CoinbaseAuth {
          pub fn new(api_key: String, api_secret: String) -> Result<Self> {
              // Validate that api_secret is a valid PEM-encoded EC private key
              // Parse and validate the key format
              Ok(Self {
                  api_key,
                  private_key: api_secret,
              })
          }

          /// Generate JWT token for API authentication
          ///
          /// Token expires in 2 minutes (Coinbase requirement)
          /// Algorithm: ES256 (ECDSA P-256)
          pub fn generate_jwt(&self) -> Result<String> {
              let now = Utc::now();
              let claims = CoinbaseClaims {
                  sub: self.api_key.clone(),
                  iss: "coinbase-cloud".to_string(),
                  nbf: now.timestamp(),
                  exp: (now + Duration::minutes(2)).timestamp(),
              };

              let header = Header::new(Algorithm::ES256);

              // Parse PEM-encoded EC private key
              let encoding_key = EncodingKey::from_ec_pem(self.private_key.as_bytes())
                  .map_err(|e| ArbitrageError::AuthenticationError {
                      exchange: "coinbase".to_string(),
                      reason: format!("Failed to parse EC private key: {}", e),
                  })?;

              encode(&header, &claims, &encoding_key)
                  .map_err(|e| ArbitrageError::AuthenticationError {
                      exchange: "coinbase".to_string(),
                      reason: format!("Failed to generate JWT: {}", e),
                  })
          }

          /// Generate JWT with custom expiration
          pub fn generate_jwt_with_exp(&self, expires_in: Duration) -> Result<String> {
              // Similar to generate_jwt but with custom expiration
          }
      }

      Add unit tests:
      - JWT token generation
      - JWT token expiration handling
      - Invalid private key handling
      - Token claims verification
    </requirements>

    <file>src/exchanges/coinbase/types.rs</file>
    <requirements>
      **Coinbase API Response Types:**
      ```rust
      use serde::{Deserialize, Serialize};
      use rust_decimal::Decimal;
      use chrono::{DateTime, Utc};

      /// Coinbase order request
      #[derive(Debug, Serialize)]
      pub struct CoinbaseOrderRequest {
          pub product_id: String,  // e.g., "SOL-USDC"
          pub side: String,         // "BUY" or "SELL"
          pub order_configuration: OrderConfiguration,
      }

      #[derive(Debug, Serialize)]
      pub struct OrderConfiguration {
          #[serde(rename = "market_market_ioc")]
          pub market_ioc: MarketIocConfig,
      }

      #[derive(Debug, Serialize)]
      pub struct MarketIocConfig {
          pub quote_size: Option<String>,  // Quote currency amount (e.g., "1000" USDC)
          pub base_size: Option<String>,   // Base currency amount (e.g., "10" SOL)
      }

      /// Coinbase order response
      #[derive(Debug, Deserialize)]
      pub struct CoinbaseOrderResponse {
          pub order_id: String,
          pub product_id: String,
          pub side: String,
          pub client_order_id: Option<String>,
          pub status: String,  // "FILLED", "PENDING", "CANCELLED", etc.
          pub average_filled_price: Option<String>,
          pub filled_size: Option<String>,
          pub fees: Option<String>,
          pub number_of_fills: Option<u32>,
          pub created_time: Option<String>,
      }

      /// Coinbase account response
      #[derive(Debug, Deserialize)]
      pub struct CoinbaseAccountsResponse {
          pub accounts: Vec<CoinbaseAccount>,
      }

      #[derive(Debug, Deserialize)]
      pub struct CoinbaseAccount {
          pub uuid: String,
          pub name: String,
          pub currency: String,
          pub available_balance: CoinbaseBalance,
          pub hold: CoinbaseBalance,
      }

      #[derive(Debug, Deserialize)]
      pub struct CoinbaseBalance {
          pub value: String,
          pub currency: String,
      }

      /// Convert CoinbaseOrderResponse to OrderResult
      impl TryFrom<CoinbaseOrderResponse> for OrderResult {
          type Error = ArbitrageError;

          fn try_from(resp: CoinbaseOrderResponse) -> Result<Self> {
              let status = match resp.status.as_str() {
                  "FILLED" => OrderStatus::Filled,
                  "PENDING" => OrderStatus::Pending,
                  "PARTIALLY_FILLED" => OrderStatus::PartiallyFilled,
                  "CANCELLED" => OrderStatus::Cancelled,
                  "FAILED" => OrderStatus::Failed,
                  _ => OrderStatus::Failed,
              };

              let filled_quantity = resp.filled_size
                  .as_ref()
                  .and_then(|s| Decimal::from_str_exact(s).ok())
                  .unwrap_or(Decimal::ZERO);

              let average_price = resp.average_filled_price
                  .as_ref()
                  .and_then(|s| Decimal::from_str_exact(s).ok());

              let fee = resp.fees
                  .as_ref()
                  .and_then(|s| Decimal::from_str_exact(s).ok())
                  .unwrap_or(Decimal::ZERO);

              // Extract fee currency from product_id (e.g., "SOL-USDC" -> "USDC")
              let fee_asset = resp.product_id
                  .split('-')
                  .nth(1)
                  .unwrap_or("USDC")
                  .to_string();

              let timestamp = resp.created_time
                  .as_ref()
                  .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                  .map(|dt| dt.with_timezone(&Utc))
                  .unwrap_or_else(Utc::now);

              Ok(OrderResult {
                  order_id: resp.order_id,
                  status,
                  filled_quantity,
                  average_price,
                  fee,
                  fee_asset,
                  timestamp,
              })
          }
      }

      /// Convert CoinbaseAccount to Decimal balance
      impl CoinbaseAccount {
          pub fn available_balance_decimal(&self) -> Result<Decimal> {
              Decimal::from_str_exact(&self.available_balance.value)
                  .map_err(|e| ArbitrageError::ParseError {
                      message: format!("Failed to parse balance: {}", e),
                      input: Some(self.available_balance.value.clone()),
                  })
          }
      }

      Add unit tests:
      - OrderResult conversion from CoinbaseOrderResponse
      - Balance parsing from CoinbaseAccount
      - Error handling for invalid responses
    </requirements>

    <file>src/exchanges/coinbase/rest.rs</file>
    <requirements>
      **REST API Client:**
      ```rust
      use reqwest::Client;
      use crate::error::{ArbitrageError, Result};
      use crate::exchanges::{Order, OrderResult, OrderSide};
      use super::auth::CoinbaseAuth;
      use super::types::*;
      use rust_decimal::Decimal;
      use std::time::Duration;
      use tokio::time::sleep;

      pub struct CoinbaseRestClient {
          client: Client,
          auth: CoinbaseAuth,
          base_url: String,
          rate_limiter: RateLimiter,  // Simple token bucket or delay-based
      }

      impl CoinbaseRestClient {
          pub fn new(api_key: String, api_secret: String, sandbox: bool) -> Result<Self> {
              let auth = CoinbaseAuth::new(api_key, api_secret)?;

              let base_url = if sandbox {
                  "https://api-public.sandbox.exchange.coinbase.com".to_string()
              } else {
                  "https://api.coinbase.com".to_string()
              };

              let client = Client::builder()
                  .timeout(Duration::from_secs(30))
                  .build()
                  .map_err(|e| ArbitrageError::NetworkError {
                      message: format!("Failed to create HTTP client: {}", e),
                      retry_after: None,
                  })?;

              // Rate limiter: 10 requests per second
              let rate_limiter = RateLimiter::new(10, Duration::from_secs(1));

              Ok(Self {
                  client,
                  auth,
                  base_url,
                  rate_limiter,
              })
          }

          /// Place a market order
          ///
          /// # Arguments
          /// * `order` - Order to place (must be Market order type)
          ///
          /// # Returns
          /// OrderResult with execution details
          pub async fn place_market_order(&self, order: Order) -> Result<OrderResult> {
              // Rate limit check
              self.rate_limiter.wait_if_needed().await;

              // Validate order type
              if !matches!(order.order_type, OrderType::Market) {
                  return Err(ArbitrageError::ExchangeError {
                      exchange: "coinbase".to_string(),
                      message: "Only market orders are supported".to_string(),
                      code: None,
                  });
              }

              // Convert pair format (SOL/USDC -> SOL-USDC)
              let product_id = order.pair.replace("/", "-");

              // Convert OrderSide to Coinbase format
              let side = match order.side {
                  OrderSide::Buy => "BUY",
                  OrderSide::Sell => "SELL",
              };

              // Build order request
              // For market orders, use base_size (quantity in base currency)
              let order_config = OrderConfiguration {
                  market_ioc: MarketIocConfig {
                      quote_size: None,
                      base_size: Some(order.quantity.to_string()),
                  },
              };

              let order_request = CoinbaseOrderRequest {
                  product_id,
                  side: side.to_string(),
                  order_configuration: order_config,
              };

              // Generate JWT token
              let jwt = self.auth.generate_jwt()?;

              // Make API request
              let url = format!("{}/api/v3/brokerage/orders", self.base_url);

              let response = self.client
                  .post(&url)
                  .header("Authorization", format!("Bearer {}", jwt))
                  .header("Content-Type", "application/json")
                  .json(&order_request)
                  .send()
                  .await
                  .map_err(|e| ArbitrageError::NetworkError {
                      message: format!("Failed to send order request: {}", e),
                      retry_after: None,
                  })?;

              // Handle rate limiting
              if response.status() == 429 {
                  let retry_after = response
                      .headers()
                      .get("Retry-After")
                      .and_then(|h| h.to_str().ok())
                      .and_then(|s| s.parse::<u64>().ok())
                      .unwrap_or(1);

                  return Err(ArbitrageError::RateLimitExceeded {
                      exchange: "coinbase".to_string(),
                      retry_after,
                  });
              }

              // Handle authentication errors
              if response.status() == 401 || response.status() == 403 {
                  return Err(ArbitrageError::AuthenticationError {
                      exchange: "coinbase".to_string(),
                      reason: format!("Authentication failed: {}", response.status()),
                  });
              }

              // Parse response
              let status = response.status();
              let response_text = response.text().await.map_err(|e| ArbitrageError::NetworkError {
                  message: format!("Failed to read response: {}", e),
                  retry_after: None,
              })?;

              if !status.is_success() {
                  return Err(ArbitrageError::ExchangeError {
                      exchange: "coinbase".to_string(),
                      message: format!("Order placement failed: {} - {}", status, response_text),
                      code: Some(status.as_u16() as i32),
                  });
              }

              // Parse order response
              let order_response: CoinbaseOrderResponse = serde_json::from_str(&response_text)
                  .map_err(|e| ArbitrageError::ParseError {
                      message: format!("Failed to parse order response: {}", e),
                      input: Some(response_text),
                  })?;

              // Convert to OrderResult
              OrderResult::try_from(order_response)
          }

          /// Get account balance for an asset
          ///
          /// # Arguments
          /// * `asset` - Asset symbol (e.g., "SOL", "USDC")
          ///
          /// # Returns
          /// Available balance as Decimal
          pub async fn get_balance(&self, asset: &str) -> Result<Decimal> {
              // Rate limit check
              self.rate_limiter.wait_if_needed().await;

              // Generate JWT token
              let jwt = self.auth.generate_jwt()?;

              // Make API request
              let url = format!("{}/api/v3/brokerage/accounts", self.base_url);

              let response = self.client
                  .get(&url)
                  .header("Authorization", format!("Bearer {}", jwt))
                  .header("Content-Type", "application/json")
                  .send()
                  .await
                  .map_err(|e| ArbitrageError::NetworkError {
                      message: format!("Failed to get balance: {}", e),
                      retry_after: None,
                  })?;

              // Handle rate limiting
              if response.status() == 429 {
                  let retry_after = response
                      .headers()
                      .get("Retry-After")
                      .and_then(|h| h.to_str().ok())
                      .and_then(|s| s.parse::<u64>().ok())
                      .unwrap_or(1);

                  return Err(ArbitrageError::RateLimitExceeded {
                      exchange: "coinbase".to_string(),
                      retry_after,
                  });
              }

              // Handle authentication errors
              if response.status() == 401 || response.status() == 403 {
                  return Err(ArbitrageError::AuthenticationError {
                      exchange: "coinbase".to_string(),
                      reason: format!("Authentication failed: {}", response.status()),
                  });
              }

              // Parse response
              let status = response.status();
              let response_text = response.text().await.map_err(|e| ArbitrageError::NetworkError {
                  message: format!("Failed to read response: {}", e),
                  retry_after: None,
              })?;

              if !status.is_success() {
                  return Err(ArbitrageError::ExchangeError {
                      exchange: "coinbase".to_string(),
                      message: format!("Balance query failed: {} - {}", status, response_text),
                      code: Some(status.as_u16() as i32),
                  });
              }

              // Parse accounts response
              let accounts_response: CoinbaseAccountsResponse = serde_json::from_str(&response_text)
                  .map_err(|e| ArbitrageError::ParseError {
                      message: format!("Failed to parse accounts response: {}", e),
                      input: Some(response_text),
                  })?;

              // Find account for the requested asset
              let account = accounts_response.accounts
                  .iter()
                  .find(|acc| acc.currency == asset)
                  .ok_or_else(|| ArbitrageError::ExchangeError {
                      exchange: "coinbase".to_string(),
                      message: format!("Account not found for asset: {}", asset),
                      code: None,
                  })?;

              // Return available balance
              account.available_balance_decimal()
          }
      }

      /// Simple rate limiter using token bucket algorithm
      struct RateLimiter {
          max_requests: u32,
          window: Duration,
          last_request: Arc<Mutex<Option<Instant>>>,
          request_count: Arc<Mutex<u32>>,
      }

      impl RateLimiter {
          fn new(max_requests: u32, window: Duration) -> Self {
              Self {
                  max_requests,
                  window,
                  last_request: Arc::new(Mutex::new(None)),
                  request_count: Arc::new(Mutex::new(0)),
              }
          }

          async fn wait_if_needed(&self) {
              // Simple implementation: track requests in time window
              // Reset counter if window expired
              // Wait if at limit
              // TODO: Implement proper token bucket algorithm
          }
      }

      Add unit tests:
      - REST client initialization
      - Order placement with mock responses
      - Balance query with mock responses
      - Rate limit handling
      - Error handling for various HTTP status codes
    </requirements>

    <file>src/exchanges/coinbase/exchange.rs</file>
    <requirements>
      **Update CoinbaseExchange to use REST client:**
      ```rust
      // Add rest_client field to CoinbaseExchange struct
      pub struct CoinbaseExchange {
          // ... existing fields ...
          rest_client: Option<CoinbaseRestClient>,
      }

      impl CoinbaseExchange {
          pub fn new(config: CoinbaseConfig) -> Result<Self> {
              // Initialize REST client if API credentials are provided
              let rest_client = if !config.api_key.is_empty() && !config.api_secret.is_empty() {
                  Some(CoinbaseRestClient::new(
                      config.api_key.clone(),
                      config.api_secret.clone(),
                      config.sandbox,
                  )?)
              } else {
                  None
              };

              Ok(Self {
                  // ... existing fields ...
                  rest_client,
              })
          }
      }

      #[async_trait]
      impl Exchange for CoinbaseExchange {
          async fn place_order(&mut self, order: Order) -> Result<OrderResult> {
              let rest_client = self.rest_client.as_ref()
                  .ok_or_else(|| ArbitrageError::ExchangeError {
                      exchange: "coinbase".to_string(),
                      message: "REST API client not initialized. API credentials required.".to_string(),
                      code: None,
                  })?;

              rest_client.place_market_order(order).await
          }

          async fn get_balance(&self, asset: &str) -> Result<Decimal> {
              let rest_client = self.rest_client.as_ref()
                  .ok_or_else(|| ArbitrageError::ExchangeError {
                      exchange: "coinbase".to_string(),
                      message: "REST API client not initialized. API credentials required.".to_string(),
                      code: None,
                  })?;

              rest_client.get_balance(asset).await
          }
      }
      ```
    </requirements>

    <coinbase_rest_api_details>
      **Coinbase Advanced Trade API Endpoints:**

      1. **Place Order**: `POST /api/v3/brokerage/orders`
         - Requires: JWT Bearer token in Authorization header
         - Body: CoinbaseOrderRequest JSON
         - Response: CoinbaseOrderResponse JSON
         - Rate limit: 10 req/sec

      2. **Get Accounts**: `GET /api/v3/brokerage/accounts`
         - Requires: JWT Bearer token in Authorization header
         - Response: CoinbaseAccountsResponse JSON
         - Rate limit: 10 req/sec

      **Authentication:**
      - Algorithm: ES256 (ECDSA P-256)
      - Token expiration: 2 minutes
      - Header format: `Authorization: Bearer <JWT_TOKEN>`
      - JWT claims:
        - `sub`: API key path (e.g., "organizations/org-id/apiKeys/key-id")
        - `iss`: "coinbase-cloud"
        - `nbf`: Not before timestamp (current time)
        - `exp`: Expiration timestamp (current time + 2 minutes)

      **Product ID Format:**
      - Coinbase uses hyphen format: "SOL-USDC"
      - Internal format: "SOL/USDC"
      - Conversion: `pair.replace("/", "-")` and `product_id.replace("-", "/")`

      **Sandbox vs Production:**
      - Sandbox: `https://api-public.sandbox.exchange.coinbase.com`
      - Production: `https://api.coinbase.com`
      - Use `config.sandbox` flag to determine base URL
    </coinbase_rest_api_details>
  </implementation>

  <validation>
    - JWT token generation works correctly
    - JWT token expires after 2 minutes
    - Order placement works in sandbox
    - Balance queries work in sandbox
    - Rate limiting prevents exceeding 10 req/sec
    - Error handling for all HTTP status codes
    - Product ID format conversion correct
    - OrderResult conversion accurate
    - All unit tests pass
    - All integration tests pass (sandbox)
    - No clippy warnings
    - Documentation complete
  </validation>

  <completion_criteria>
    ❌ Tests written and failing initially (TDD)
    💻 JWT authentication implemented
    💻 REST API client implemented
    💻 Order placement working (sandbox)
    💻 Balance queries working (sandbox)
    ✅ All tests passing
    ✅ Unit tests for auth, REST client, types
    ✅ Integration tests for sandbox API
    ✅ Rate limiting working
    ✅ Error handling comprehensive
    ✅ Documentation complete
    ✅ Human verification with sandbox
  </completion_criteria>
</task>
````

### Files to Create/Update

1. **src/exchanges/coinbase/auth.rs** - JWT authentication with ES256 (NEW)
2. **src/exchanges/coinbase/types.rs** - Update with REST API types (UPDATE)
3. **src/exchanges/coinbase/rest.rs** - REST API client implementation (UPDATE)
4. **src/exchanges/coinbase/exchange.rs** - Integrate REST client (UPDATE)
5. **src/exchanges/coinbase/mod.rs** - Export auth module (UPDATE)
6. **tests/coinbase_rest.rs** - REST API integration tests (NEW)
7. **Cargo.toml** - Add JWT and crypto dependencies (UPDATE)
8. **examples/coinbase_rest_test.rs** - Example for testing REST API (NEW)

### Dependencies to Add

```toml
# JWT signing (ES256 algorithm)
jsonwebtoken = "9.3"
p256 = "0.13"  # ECDSA P-256 curve support
# OR use ring = "0.17" with ES256 support

# For parsing EC private keys (PEM format)
pkcs8 = "0.10"  # For parsing PEM-encoded EC private keys
```

### Testing Strategy

1. **Unit Tests**: Mock HTTP responses using `mockito` or similar
2. **Integration Tests**: Real sandbox API calls (marked `#[ignore]`)
3. **Manual Testing**: Example program to test order placement and balance queries

### Security Considerations

- API keys stored in config (not hardcoded)
- JWT tokens expire after 2 minutes
- Private keys never logged
- Rate limiting prevents API abuse
- Sandbox mode for testing (no real funds)

### Note: Revisit HIGH PRIORITY Core Principles before implementing this Task and make sure you complete them. Especially point 7 & 8.

---

## 🎯 Task 8: Logging & Observability

### Note: Revisit HIGH PRIORITY Core Principles before imlementing next Task and make sure you complete them. Especially point 7 & 8.

### Objective

Set up structured logging with tracing for debugging and monitoring.

### What to Build

1. Structured logging with tracing
2. Multiple output formats (JSON, pretty)
3. Log level filtering
4. File rotation
5. Async logging

### Implementation Specification

````xml
<task id="8" phase="1">
  <name>Structured Logging with Tracing</name>
  <priority>medium</priority>

  <test_first>
    <file>tests/logging.rs</file>
    <description>
      Write tests FIRST (will fail initially):
      - Log messages at different levels (debug, info, warn, error)
      - Structured fields in logs (exchange name, price, timestamp)
      - Log to file and stdout
      - Log rotation works
      - Async logging doesn't block
      - Filter logs by module/level
      - JSON format for production
      - Pretty format for development

      These tests verify logging behavior before implementation exists.
    </description>
  </test_first>

  <implementation>
    <requirements>
      **Logging Configuration:**
      ```rust
      use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
      use tracing_appender::rolling::{RollingFileAppender, Rotation};

      #[derive(Debug, Clone)]
      pub enum LogFormat {
          Json,
          Pretty,
          Compact,
      }

      #[derive(Debug, Clone)]
      pub struct LoggerConfig {
          pub level: String,
          pub format: LogFormat,
          pub file_path: Option<String>,
          pub rotation: Rotation,
      }

      impl LoggerConfig {
          pub fn init(&self) -> Result<()> {
              let env_filter = EnvFilter::try_from_default_env()
                  .or_else(|_| EnvFilter::try_new(&self.level))
                  .map_err(|e| ArbitrageError::ConfigError {
                      field: "logging.level".into(),
                      reason: format!("Invalid log level: {}", e),
                  })?;

              let registry = tracing_subscriber::registry()
                  .with(env_filter);

              // Add file appender if path is specified
              if let Some(file_path) = &self.file_path {
                  let file_appender = RollingFileAppender::new(
                      self.rotation,
                      file_path,
                      "arb-bot.log",
                  );

                  let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

                  let file_layer = match self.format {
                      LogFormat::Json => {
                          fmt::layer()
                              .json()
                              .with_writer(non_blocking)
                              .boxed()
                      }
                      LogFormat::Pretty => {
                          fmt::layer()
                              .pretty()
                              .with_writer(non_blocking)
                              .boxed()
                      }
                      LogFormat::Compact => {
                          fmt::layer()
                              .compact()
                              .with_writer(non_blocking)
                              .boxed()
                      }
                  };

                  registry.with(file_layer).init();
              } else {
                  // Console only
                  let console_layer = match self.format {
                      LogFormat::Json => fmt::layer().json().boxed(),
                      LogFormat::Pretty => fmt::layer().pretty().boxed(),
                      LogFormat::Compact => fmt::layer().compact().boxed(),
                  };

                  registry.with(console_layer).init();
              }

              Ok(())
          }
      }
      ```

      **Usage Macros and Helpers:**
      ```rust
      // Re-export tracing macros
      pub use tracing::{debug, info, warn, error, trace};

      // Structured logging helpers
      pub fn log_price_update(exchange: &str, pair: &str, price: Decimal) {
          info!(
              exchange = %exchange,
              pair = %pair,
              price = %price,
              "Price update received"
          );
      }

      pub fn log_arbitrage_opportunity(
          buy_exchange: &str,
          sell_exchange: &str,
          pair: &str,
          spread_pct: Decimal,
      ) {
          info!(
              buy_exchange = %buy_exchange,
              sell_exchange = %sell_exchange,
              pair = %pair,
              spread_pct = %spread_pct,
              "Arbitrage opportunity detected"
          );
      }

      pub fn log_order_placed(
          exchange: &str,
          order_id: &str,
          side: &str,
          quantity: Decimal,
      ) {
          info!(
              exchange = %exchange,
              order_id = %order_id,
              side = %side,
              quantity = %quantity,
              "Order placed"
          );
      }

      pub fn log_error(context: &str, error: &ArbitrageError) {
          error!(
              context = %context,
              error = %error,
              "Error occurred"
          );
      }
      ```

      **Span Instrumentation:**
      ```rust
      use tracing::instrument;

      // Example: Instrument async functions
      #[instrument(skip(exchange))]
      pub async fn connect_exchange(exchange: &mut dyn Exchange) -> Result<()> {
          info!("Connecting to exchange");
          exchange.connect().await?;
          info!("Connected successfully");
          Ok(())
      }

      #[instrument(skip(exchange), fields(pair = %pair))]
      pub async fn subscribe_to_pair(
          exchange: &mut dyn Exchange,
          pair: &str,
      ) -> Result<()> {
          info!("Subscribing to ticker");
          exchange.subscribe_ticker(pair).await?;
          info!("Subscribed successfully");
          Ok(())
      }
      ```

      - Use tracing + tracing-subscriber
      - JSON formatted logs for production
      - Pretty logs for development
      - Span tracking for request tracing
      - Log filtering by module/level
      - Async file appender (non-blocking)
      - Log rotation (daily, hourly, etc.)
      - Add unit tests for:
        * Log level filtering
        * Format selection (JSON vs pretty)
        * Structured field extraction
        * File rotation
    </requirements>
  </implementation>

  <validation>
    - Logs output correctly in different formats
    - Structured fields present
    - File rotation works
    - Async logging doesn't block
    - All tests pass
    - No clippy warnings
  </validation>

  <completion_criteria>
    ❌ Tests written and failing initially (TDD)
    💻 Logging system implemented
    ✅ Multiple formats working (JSON, pretty)
    ✅ File rotation working
    ✅ All tests passing
    ✅ Unit tests for configuration
    ✅ Documentation complete
    ✅ Human verification complete
  </completion_criteria>
</task>
````

### Files to Create

1. **src/logger/mod.rs** - Module exports and helper functions
2. **src/logger/config.rs** - LoggerConfig with parse pattern and unit tests
3. **src/logger/format.rs** - Format helpers with unit tests
4. **tests/logging.rs** - Integration tests

### Note: Revisit HIGH PRIORITY Core Principles before imlementing next Task and make sure you complete them. Especially point 7 & 8.

---

## 🦀 Rust Best Practices Checklist

For every task, the AI agent must ensure:

### Error Handling

- ✅ **No `unwrap()` in production code** - Use `?` operator or `expect()` with context
- ✅ **Proper error handling** - All `Result` types handled explicitly
- ✅ **Custom error types** - Use `thiserror` for domain errors
- ✅ **Context in errors** - Include relevant information for debugging

### Async Programming

- ✅ **Async/await** - Use `tokio::spawn` for background tasks
- ✅ **Select macro** - Use `tokio::select!` for concurrent operations
- ✅ **Channels** - Use mpsc/broadcast for message passing
- ✅ **Cancellation** - Implement graceful shutdown

### Type System

- ✅ **Traits over concrete types** - Enable polymorphism and testing
- ✅ **Generics with trait bounds** - Maximize code reuse
- ✅ **Parse pattern** - Validate at construction time, not later
- ✅ **Type-state pattern** - Use types to enforce valid states

### Concurrency

- ✅ **Interior mutability** - `Arc<RwLock<T>>` for shared state
- ✅ **Send + Sync** - Ensure types can be safely shared
- ✅ **No data races** - Use proper synchronization primitives
- ✅ **Deadlock prevention** - Always acquire locks in same order

### Performance

- ✅ **Zero-copy where possible** - Use `&str` over `String`, `Cow<'_, str>`
- ✅ **Avoid allocations** - Reuse buffers, use references
- ✅ **Decimal for money** - Never use f64 for prices
- ✅ **Efficient collections** - DashMap for concurrent access

### Code Quality

- ✅ **Documentation** - Every public item has rustdoc comments
- ✅ **Tests** - Unit tests alongside code, integration tests in `tests/`
- ✅ **Clippy clean** - No warnings from `cargo clippy`
- ✅ **Format check** - `cargo fmt` before committing
- ✅ **Examples** - Provide usage examples in doc comments

### Project Structure

- ✅ **Module organization** - Logical grouping of related code
- ✅ **Re-exports** - Clean public API via mod.rs files
- ✅ **Feature flags** - Optional dependencies behind features
- ✅ **Workspace** - If project grows, use cargo workspace

---

## ✅ Phase 1 Completion Criteria

Phase 1 is considered complete when ALL of the following are met:

### Tests

- ✅ All 8 tasks completed and confirmed by human
- ✅ Integration tests for all components passing
- ✅ Unit tests for all domain logic passing
- ✅ `cargo test` shows 100% passing tests
- ✅ Code coverage >80% (measure with `cargo tarpaulin`)

### Code Quality

- ✅ `cargo clippy` shows zero warnings
- ✅ `cargo fmt --check` passes
- ✅ Documentation builds successfully (`cargo doc --no-deps`)
- ✅ All public items have documentation

### Functionality

- ✅ Error handling system works end-to-end
- ✅ Configuration loads from file and environment variables
- ✅ Mock exchanges simulate price feeds correctly
- ✅ WebSocket connections handle reconnections
- ✅ Price state updates thread-safely
- ✅ Binance integration connects to testnet
- ✅ Coinbase integration connects to sandbox
- ✅ Logging captures all important events

### Manual Verification

- ✅ Connect to Binance testnet and receive price updates
- ✅ Connect to Coinbase sandbox and receive price updates
- ✅ Verify logs show structured data
- ✅ Verify configuration validation works
- ✅ Verify error messages are clear and actionable

### Documentation

- ✅ README updated with Phase 1 status
- ✅ Architecture diagram created (if needed)
- ✅ API documentation generated
- ✅ Usage examples provided

---

## 📝 XML Template Structure

When generating code, the AI agent must use this XML structure:

````xml
<code_changes>
  <changed_files>
    <file>
      <file_operation>CREATE|UPDATE|DELETE</file_operation>
      <file_path>relative/path/to/file.rs</file_path>
      <file_code><![CDATA[
/**
 * @file Brief description of file
 * @description
 * Detailed description of what this file does.
 * Explain its role in the system.
 *
 * Key features:
 * - Feature 1: Description
 * - Feature 2: Description
 *
 * @dependencies
 * - DependencyA: Used for X
 * - DependencyB: Used for Y
 *
 * @notes
 * - Important implementation detail 1
 * - Important implementation detail 2
 */

// COMPLETE FILE CONTENTS HERE
// Never use ... or TODO comments
// Include ALL necessary code

use crate::error::Result;

/// Brief description of struct/enum/function
///
/// More detailed explanation of what it does,
/// why it exists, and how it should be used.
///
/// # Examples
///
/// ```
/// use arb_bot::something::Thing;
///
/// let thing = Thing::new();
/// thing.do_something();
/// ```
pub struct Thing {
    // Fields with documentation
}

impl Thing {
    /// Create a new Thing
    pub fn new() -> Self {
        Self {
            // Implementation
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // Unit test
    }
}
]]></file_code>
    </file>

    <!-- Additional files here -->

  </changed_files>
</code_changes>
````

### Documentation Requirements

Every file must include:

- **File-level documentation** explaining purpose and scope
- **Type documentation** for all public structs/enums/traits
- **Function documentation** detailing inputs, outputs, and behavior
- **Inline comments** explaining complex logic
- **Examples** showing typical usage
- **Notes** about edge cases and error handling

---

## 📄 Example Configurations

### config.example.toml

```toml
# CEX-to-CEX Arbitrage Bot Configuration

[exchanges.coinbase]
api_key = "organizations/your-org-id/apiKeys/your-key-id"
api_secret = "-----BEGIN EC PRIVATE KEY-----\nYour private key here\n-----END EC PRIVATE KEY-----"
sandbox = true  # Use sandbox environment for testing

[exchanges.binance]
api_key = "your_binance_api_key"
api_secret = "your_binance_api_secret"
testnet = true  # Use testnet for testing

[trading]
pair = "SOL/USDC"
spread_threshold = 0.002  # 0.2% - minimum spread to trigger arbitrage
order_size = 10.0         # Order size in base currency (SOL)
cooldown_ms = 5000        # Minimum time between trades (5 seconds)

[risk]
max_position_size = 1000.0     # Maximum position size per exchange
max_daily_trades = 100         # Maximum number of trades per day
stop_loss_threshold = 0.05     # 5% - stop trading if loss exceeds this
emergency_stop = false         # Emergency stop switch

[logging]
level = "info"              # trace, debug, info, warn, error
format = "json"             # json, pretty, compact
file_path = "logs"          # Directory for log files
rotation = "daily"          # daily, hourly, never
```

### .env.example

```bash
# Environment Variables for CEX-to-CEX Arbitrage Bot
# Copy this file to .env and fill in your values

# Override config file values with environment variables
# Format: ARB_<SECTION>_<KEY>=value

# Coinbase Advanced Trade API
ARB_EXCHANGES_COINBASE_API_KEY="organizations/your-org-id/apiKeys/your-key-id"
ARB_EXCHANGES_COINBASE_API_SECRET="-----BEGIN EC PRIVATE KEY-----\nYour private key\n-----END EC PRIVATE KEY-----"
ARB_EXCHANGES_COINBASE_SANDBOX=true

# Binance API
ARB_EXCHANGES_BINANCE_API_KEY="your_binance_api_key"
ARB_EXCHANGES_BINANCE_API_SECRET="your_binance_api_secret"
ARB_EXCHANGES_BINANCE_TESTNET=true

# Trading Parameters
ARB_TRADING_PAIR="SOL/USDC"
ARB_TRADING_SPREAD_THRESHOLD=0.002
ARB_TRADING_ORDER_SIZE=10.0
ARB_TRADING_COOLDOWN_MS=5000

# Risk Management
ARB_RISK_MAX_POSITION_SIZE=1000.0
ARB_RISK_MAX_DAILY_TRADES=100
ARB_RISK_STOP_LOSS_THRESHOLD=0.05
ARB_RISK_EMERGENCY_STOP=false

# Logging
ARB_LOGGING_LEVEL="info"
ARB_LOGGING_FORMAT="json"
ARB_LOGGING_FILE_PATH="logs"
ARB_LOGGING_ROTATION="daily"

# Application
RUST_LOG="arb_bot=debug,info"  # Override log levels by module
RUST_BACKTRACE=1                # Enable backtraces for debugging
```

---

## 🎯 Summary

This guide provides everything needed for Phase 1 implementation:

1. **8 detailed tasks** with TDD approach
2. **Complete code specifications** with examples
3. **Test-first methodology** clearly explained
4. **Parse pattern** for configuration validation
5. **Rust best practices** checklist
6. **XML template** for code delivery
7. **Example configurations** for quick setup

The AI agent should:

- Read this guide before starting each task
- Follow the TDD workflow (❌ → 💻 → ✅)
- Use the parse pattern for validation
- Write comprehensive tests and documentation
- Wait for human confirmation between tasks

Good luck with Phase 1 implementation! 🚀
