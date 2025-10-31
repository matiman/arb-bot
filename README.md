# Arb Bot

A Rust-based arbitrage bot for cryptocurrency trading.

## Overview

This project implements an arbitrage bot that monitors price differences across multiple exchanges to identify and execute profitable trading opportunities.

## Features

- Real-time price monitoring
- Multi-exchange support (Binance, Coinbase, etc.)
- Risk management
- Automated trading execution
- Exchange trait abstraction for flexible implementations

## Architecture

The bot uses a trait-based architecture for exchange interactions:

- **Exchange Trait**: Unified interface for all exchanges
- **Price Monitoring**: Real-time price feeds with spread calculation
- **Order Execution**: Market order placement with fee tracking
- **Mock Exchange**: Testing without real API calls

For detailed architecture documentation, see [Exchange Architecture Guide](docs/EXCHANGE_ARCHITECTURE.md).

## Getting Started

### Prerequisites

- Rust 1.85.0 or later
- Cargo package manager

### Installation

1. Clone the repository:

```bash
git clone <repository-url>
cd arb-bot
```

2. Build the project:

```bash
cargo build
```

3. Run the application:

```bash
cargo run
```

## Development

### Project Structure

```
arb-bot/
├── Cargo.toml          # Package configuration
├── Cargo.lock          # Dependency lock file
├── .gitignore          # Git ignore rules
├── README.md           # Project documentation
├── src/
│   ├── main.rs         # Main binary entry point
│   ├── lib.rs          # Library entry point
│   ├── bin/            # Additional binaries
│   ├── examples/       # Example code
│   ├── benches/        # Benchmark code
│   ├── api/            # API modules
│   ├── config/         # Configuration modules
│   ├── database/       # Database modules
│   ├── error/          # Error handling modules
│   ├── exchanges/      # Exchange integrations
│   ├── logger/         # Logging modules
│   ├── risk/           # Risk management modules
│   ├── strategy/       # Trading strategy modules
│   ├── utils/          # Utility modules
│   └── websocket/      # WebSocket modules
├── tests/              # Integration tests
│   └── common/         # Shared test utilities
└── target/             # Build artifacts
```

### Running Tests

```bash
# Run all tests (unit + integration)
cargo test

# Run unit tests only
cargo test --lib

# Run integration tests only
cargo test --test '*'

# Run all tests with output
cargo test -- --nocapture
```

### Running Examples

```bash
# Run example code
cargo run --example <example-name>
```

### Running Benchmarks

```bash
# Run benchmarks
cargo bench
```

### Building for Release

```bash
cargo build --release
```

## License

[Add your license here]
