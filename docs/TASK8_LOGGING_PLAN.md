# Task 8: Logging & Observability - Implementation Plan

## Overview
Set up structured logging with `tracing` for debugging and monitoring. This will replace `println!`/`eprintln!` statements with proper structured logging.

## Branch
- **Branch name**: `task8-logging-observability`
- **Base branch**: `task7-coinbase-integration`

## Objectives

1. **Structured Logging**: Use `tracing` crate for structured, contextual logging
2. **Multiple Formats**: Support JSON (production) and Pretty (development)
3. **Log Levels**: Filter by level (trace, debug, info, warn, error)
4. **File Logging**: Optional file output with rotation
5. **Async Logging**: Non-blocking file appender
6. **Module Filtering**: Filter logs by module/component
7. **Structured Fields**: Log with context (exchange, pair, price, etc.)

## Dependencies to Add

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json", "fmt"] }
tracing-appender = "0.2"  # For file rotation
```

## Files to Create/Modify

### New Files
1. **`src/logger/mod.rs`**
   - Module exports
   - Re-export tracing macros (`debug!`, `info!`, `warn!`, `error!`, `trace!`)
   - Helper functions for structured logging

2. **`src/logger/config.rs`**
   - `LogFormat` enum (Json, Pretty, Compact)
   - `LoggerConfig` struct with parse pattern
   - `init()` method to set up tracing subscriber
   - Unit tests

3. **`src/logger/format.rs`** (optional)
   - Format helpers if needed
   - Custom formatters

4. **`tests/logging.rs`**
   - Integration tests (TDD - write first, will fail)
   - Test log levels, formats, file rotation, async behavior

### Files to Modify
1. **`src/lib.rs`**
   - Add `pub mod logger;`

2. **`src/config/mod.rs`**
   - Add logging config module (if separate from logger)

3. **Replace `println!`/`eprintln!`** in:
   - `src/exchanges/binance/exchange.rs`
   - `src/exchanges/coinbase/exchange.rs`
   - `src/exchanges/binance/parser.rs`
   - `src/exchanges/coinbase/parser.rs`
   - `src/websocket/manager.rs`
   - Any other files with debug output

## Implementation Steps (TDD)

### Step 1: Write Failing Tests
- Create `tests/logging.rs` with tests for:
  - Log level filtering
  - JSON format output
  - Pretty format output
  - File logging
  - Log rotation
  - Structured fields
  - Module filtering
- Run tests - they should fail (TDD)

### Step 2: Add Dependencies
- Add `tracing`, `tracing-subscriber`, `tracing-appender` to `Cargo.toml`

### Step 3: Implement LoggerConfig
- Create `src/logger/config.rs`
- Implement `LogFormat` enum
- Implement `LoggerConfig` struct
- Implement `init()` method with:
  - EnvFilter for level/module filtering
  - File appender (if path specified) with rotation
  - Console appender
  - Format selection (JSON/Pretty/Compact)
  - Non-blocking file writer

### Step 4: Create Logger Module
- Create `src/logger/mod.rs`
- Re-export tracing macros
- Add helper functions:
  - `log_price_update(exchange, pair, price)`
  - `log_arbitrage_opportunity(buy_exchange, sell_exchange, pair, spread_pct)`
  - `log_order_placed(exchange, order_id, side, quantity)`
  - `log_error(context, error)`

### Step 5: Replace Print Statements
- Replace `println!` with `info!` or appropriate level
- Replace `eprintln!` with `error!` or `warn!`
- Add structured fields where appropriate
- Use `#[instrument]` attribute on key async functions

### Step 6: Integration
- Initialize logger in examples
- Add logger initialization helper
- Update documentation

### Step 7: Tests & Validation
- All tests should pass
- Verify log output in different formats
- Test file rotation
- Test async behavior

## Key Features

### Log Levels
- `trace!` - Very detailed debugging
- `debug!` - Debugging information
- `info!` - General information (price updates, connections)
- `warn!` - Warnings (reconnections, rate limits)
- `error!` - Errors (connection failures, API errors)

### Structured Fields
- Exchange name
- Trading pair
- Price values
- Order IDs
- Error context
- Timestamps (automatic)

### Formats
- **JSON**: Production format, machine-readable
- **Pretty**: Development format, human-readable
- **Compact**: Minimal format

### File Rotation
- Daily rotation (default)
- Hourly rotation (optional)
- Configurable log directory

## Configuration

### From Config File
```toml
[logging]
level = "info"
format = "json"  # or "pretty", "compact"
file_path = "logs"
rotation = "daily"  # or "hourly", "never"
```

### From Environment
```bash
RUST_LOG=arb_bot=debug,info
RUST_LOG_SPAN_EVENTS=full
```

## Integration Points

1. **Exchange connections**: Log connection attempts, successes, failures
2. **Price updates**: Log price updates with structured fields
3. **WebSocket events**: Log connection, disconnection, reconnection
4. **Order execution**: Log order placement, fills, errors
5. **Arbitrage detection**: Log opportunities with spread calculations
6. **Error handling**: Log all errors with context

## Testing Strategy

1. **Unit Tests**: Test `LoggerConfig` initialization, format selection
2. **Integration Tests**: Test actual log output, file rotation, async behavior
3. **Manual Testing**: Verify log output in examples

## Success Criteria

- ✅ All tests passing
- ✅ JSON format works for production
- ✅ Pretty format works for development
- ✅ File rotation works
- ✅ Async logging doesn't block
- ✅ Module filtering works
- ✅ All `println!`/`eprintln!` replaced
- ✅ Structured fields in logs
- ✅ Documentation complete

## Estimated Changes

- **New files**: 3-4 files (~300-400 lines)
- **Modified files**: ~10 files (replace print statements)
- **Tests**: ~100-150 lines
- **Total**: ~500-600 lines added

## Notes

- Keep existing error handling - logging doesn't replace it
- Use appropriate log levels (don't log everything as `info`)
- Structured fields make logs searchable/queryable
- File rotation prevents disk space issues
- Async logging ensures performance isn't impacted

