# Exchange Architecture & Business Logic

This document explains the Exchange trait abstraction and related components, focusing on business logic and how they work together for arbitrage trading.

## Overview

The Exchange module provides a unified interface for interacting with cryptocurrency exchanges. It enables the arbitrage bot to work with multiple exchanges (Binance, Coinbase, etc.) through a common abstraction.

---

## Core Components

### 1. Exchange Trait - Main Abstraction

**File:** `src/exchanges/mod.rs`

**Business Purpose:** Defines the interface that all exchanges must implement. This allows swapping exchange implementations without changing the arbitrage detection logic.

**Key Methods (Business Logic):**

#### `connect() -> Result<()>`

- **Purpose:** Establishes WebSocket connection to the exchange
- **Business:** Must connect before getting prices or placing orders
- **Returns:** Success if connection established

#### `subscribe_ticker(pair: &str) -> Result<()>`

- **Purpose:** Subscribes to real-time price updates for a trading pair
- **Business:** Enables receiving live price feed updates (e.g., SOL/USDC price changes)
- **Note:** Some exchanges require explicit subscription before price updates

#### `get_latest_price(pair: &str) -> Result<Price>`

- **Purpose:** Returns current bid/ask/last price for a trading pair
- **Business:** Used by arbitrage detection to compare prices across exchanges
- **Returns:** `Price` struct containing bid, ask, last, volume, timestamp
- **Critical for arbitrage:** Compare prices between exchanges to find opportunities

#### `place_order(order: Order) -> Result<OrderResult>`

- **Purpose:** Executes a trade (buy or sell)
- **Business:** The actual execution of arbitrage opportunities
- **Input:** `Order` specifying what to buy/sell and quantity
- **Output:** `OrderResult` with order ID, execution status, fees, and average price
- **Profit calculation:** Use `OrderResult::total_cost()` to calculate net profit

#### `get_balance(asset: &str) -> Result<Decimal>`

- **Purpose:** Checks available funds for a specific asset
- **Business:** Ensure sufficient balance before attempting to trade
- **Example:** Check USDC balance before buying SOL
- **Returns:** Available balance as `Decimal` for precision

#### `name() -> &str`

- **Purpose:** Returns exchange identifier
- **Business:** Used for logging, tracking which exchange executed trades
- **Example:** Returns "binance", "coinbase", "mock" (for testing)

#### `is_connected() -> bool` / `disconnect() -> Result<()>`

- **Purpose:** Connection state management
- **Business:** Monitor connection health, reconnect if dropped
- **Critical:** Arbitrage requires reliable connections to both exchanges

---

### 2. Price Struct - Market Data Representation

**File:** `src/exchanges/types.rs`

**Business Purpose:** Represents current market price data from an exchange.

**Fields:**

- `bid: Decimal` - Best price buyers are willing to pay (you can sell at this price)
- `ask: Decimal` - Best price sellers are asking (you can buy at this price)
- `last: Decimal` - Most recent trade price
- `volume_24h: Decimal` - Trading volume over 24 hours (liquidity indicator)
- `timestamp: DateTime<Utc>` - When the price was captured

**Business Methods:**

#### `mid_price() -> Decimal`

- **Purpose:** Average of bid and ask prices
- **Business:** Represents fair market price estimate
- **Formula:** `(bid + ask) / 2`
- **Use case:** Used in spread calculations for arbitrage detection

#### `spread() -> Decimal`

- **Purpose:** Difference between ask and bid prices
- **Business:** The bid-ask spread is a trading cost - larger spread = less profit
- **Formula:** `ask - bid`
- **Example:** bid=100, ask=101 → spread=1.00

#### `spread_percentage() -> Decimal`

- **Purpose:** Spread as a percentage of mid price
- **Business:** Used by arbitrage detection - need spread > threshold to be profitable
- **Formula:** `(spread / mid_price) * 100`
- **Arbitrage threshold:** Typically need >0.2% spread to cover fees

**Example Price Data:**

```rust
Price {
    pair: "SOL/USDC",
    bid: 100.00,    // You can sell at this price
    ask: 101.00,    // You can buy at this price
    last: 100.50,   // Last trade price
    spread: 1.00,   // Trading cost
    spread_percentage: ~1%  // Need >0.2% for arbitrage
}
```

---

### 3. Order Struct - Trade Instructions

**File:** `src/exchanges/types.rs`

**Business Purpose:** Represents a trade instruction sent to an exchange.

**Fields:**

- `pair: String` - Trading pair (e.g., "SOL/USDC")
- `side: OrderSide` - Buy or Sell
- `order_type: OrderType` - Market (immediate) or Limit (at specific price)
- `quantity: Decimal` - Amount to trade

**Order Types:**

- `Market` - Execute immediately at current market price
- `Limit { price: Decimal }` - Execute only at specified price (not implemented yet)

**Factory Methods:**

#### `Order::market_buy(pair, quantity) -> Order`

- **Purpose:** Create a buy order for immediate execution
- **Business:** Used when executing arbitrage - buy low on one exchange
- **Example:** `Order::market_buy("SOL/USDC", Decimal::from(10))`

#### `Order::market_sell(pair, quantity) -> Order`

- **Purpose:** Create a sell order for immediate execution
- **Business:** Used when executing arbitrage - sell high on another exchange
- **Example:** `Order::market_sell("SOL/USDC", Decimal::from(10))`

**Business Logic:**

- Orders are immutable once created
- Used in `Exchange::place_order()` to execute trades
- Market orders execute immediately but price may vary slightly

---

### 4. OrderResult Struct - Trade Execution Results

**File:** `src/exchanges/types.rs`

**Business Purpose:** Represents the outcome of a placed order - what actually happened.

**Fields:**

- `order_id: String` - Unique identifier for tracking the order
- `status: OrderStatus` - Current state: Pending, Filled, PartiallyFilled, Cancelled, Failed
- `filled_quantity: Decimal` - How much was actually traded
- `average_price: Option<Decimal>` - Average execution price (if filled)
- `fee: Decimal` - Trading fee paid
- `fee_asset: String` - Currency of fee (usually USDC or base asset)
- `timestamp: DateTime<Utc>` - When order was executed

**Business Methods:**

#### `is_complete() -> bool`

- **Purpose:** Check if order is finished (won't change status)
- **Business:** Determines if arbitrage leg completed successfully
- **Returns:** `true` if Filled, Cancelled, or Failed

#### `total_cost() -> Option<Decimal>`

- **Purpose:** Calculate total cost including fees
- **Business:** Essential for profit calculation
- **Formula:** `average_price * filled_quantity + fee`
- **Returns:** `None` if order not filled (no average_price)

**Profit Calculation Example:**

```rust
// Buy order result
let buy_cost = buy_result.total_cost().unwrap(); // e.g., 1000 USDC

// Sell order result
let sell_revenue = sell_result.total_cost().unwrap(); // e.g., 1020 USDC

// Net profit
let profit = sell_revenue - buy_cost; // 20 USDC
```

---

### 5. MockExchange - Test Implementation

**File:** `src/exchanges/mock.rs`

**Business Purpose:** Simulates an exchange for testing without real connections or money.

**How it works:**

- Uses in-memory `HashMap` to store prices and balances
- All methods behave like real exchange but use stored data
- Thread-safe using `Arc<RwLock<T>>` for concurrent access

**Key Methods:**

#### `set_price(pair, price)`

- **Purpose:** Set mock price for testing
- **Business:** Simulate different market conditions
- **Example:** Test arbitrage detection with specific price differences

#### `set_balance(asset, amount)`

- **Purpose:** Set mock account balance
- **Business:** Test scenarios with different account balances
- **Example:** Test insufficient balance error handling

**Business Value:**

- Test arbitrage logic without real money or API calls
- Simulate error conditions (connection failures, insufficient balance)
- Fast, deterministic tests
- No rate limits or API costs

---

### 6. ExchangeFactory - Exchange Creation Pattern

**File:** `src/exchanges/factory.rs`

**Business Purpose:** Creates exchange instances by name - centralizes exchange creation logic.

**Method:**

```rust
fn create_exchange(name: &str, config: Option<&()>) -> Result<Box<dyn Exchange>>
```

**Business Logic:**

```rust
match name {
    "binance" => create BinanceExchange,
    "coinbase" => create CoinbaseExchange,
    "mock" => create MockExchange (for testing),
    _ => error
}
```

**Why it's useful:**

- Single point for creating exchanges
- Easy to add new exchanges later
- Swappable implementations (test vs real)
- Consistent initialization across codebase

---

## How Components Work Together (Arbitrage Flow)

### Step 1: Connect to Exchanges

```rust
let factory = DefaultExchangeFactory;
let mut binance = factory.create_exchange("binance", None)?;
let mut coinbase = factory.create_exchange("coinbase", None)?;

binance.connect().await?;
coinbase.connect().await?;
```

### Step 2: Get Prices from Both Exchanges

```rust
let binance_price = binance.get_latest_price("SOL/USDC").await?;
let coinbase_price = coinbase.get_latest_price("SOL/USDC").await?;
```

### Step 3: Detect Arbitrage Opportunity

```rust
// Calculate potential profit
// Buy on Binance (lower ask price), sell on Coinbase (higher bid price)
let buy_cost = binance_price.ask;  // e.g., 100.00
let sell_price = coinbase_price.bid; // e.g., 101.00
let spread = sell_price - buy_cost;  // 1.00

// Check if profitable after fees
let spread_pct = (spread / buy_cost) * Decimal::from(100);
let threshold = Decimal::from_str("0.2").unwrap(); // 0.2% minimum

if spread_pct > threshold {
    // Opportunity found!
}
```

### Step 4: Check Balances

```rust
let usdc_balance = binance.get_balance("USDC").await?;
let required = quantity * binance_price.ask;

if usdc_balance < required {
    return Err("Insufficient balance".into());
}
```

### Step 5: Execute Trades

```rust
let quantity = Decimal::from(10);

// Buy on Binance (lower price)
let buy_order = Order::market_buy("SOL/USDC", quantity);
let buy_result = binance.place_order(buy_order).await?;

// Sell on Coinbase (higher price)
let sell_order = Order::market_sell("SOL/USDC", quantity);
let sell_result = coinbase.place_order(sell_order).await?;
```

### Step 6: Calculate Profit

```rust
if buy_result.is_complete() && sell_result.is_complete() {
    let buy_cost = buy_result.total_cost().unwrap();
    let sell_revenue = sell_result.total_cost().unwrap();
    let profit = sell_revenue - buy_cost;

    println!("Profit: {} USDC", profit);
}
```

---

## Design Patterns Used

### 1. **Trait Objects (`Box<dyn Exchange>`)**

- Enables polymorphism - can swap exchange implementations
- Runtime dispatch allows different exchange types
- Factory returns trait objects for flexibility

### 2. **Dependency Injection (MockExchange)**

- Tests can inject mock implementations
- No real API calls during testing
- Deterministic test behavior

### 3. **Thread-Safe State (`Arc<RwLock<T>>`)**

- MockExchange uses shared state for concurrent access
- Multiple tasks can access exchange concurrently
- Safe for async/await context

### 4. **Builder Pattern (Order factory methods)**

- `Order::market_buy()` / `Order::market_sell()` create orders easily
- Reduces boilerplate
- Type-safe order creation

---

## Future Enhancements

### Limit Orders

Currently only `Market` orders are supported. Future work:

- `OrderType::Limit { price: Decimal }` - Execute at specific price
- Order cancellation
- Order status polling

### Additional Exchange Methods

- Cancel order
- Get order status
- Get order history
- Get trade history

### Error Recovery

- Automatic reconnection on connection loss
- Retry failed orders
- Exponential backoff for rate limits

---

## Testing Strategy

### Unit Tests

- Price calculations (mid_price, spread, spread_percentage)
- Order creation (market_buy, market_sell)
- OrderResult calculations (is_complete, total_cost)

### Integration Tests

- MockExchange full workflow
- Trait object usage
- Concurrent access patterns
- Error handling (connection failures, insufficient balance)

### Real Exchange Tests (Future)

- Integration tests with real exchanges (testnet/sandbox)
- Validate against actual API behavior
- Test rate limiting and error responses

---

## Summary

**Exchange Trait:** Core abstraction for any exchange - enables swapping implementations

**Price:** Market data with spread calculations - critical for arbitrage detection

**Order:** Trade instructions - represents what to buy/sell

**OrderResult:** Execution results - enables profit calculation

**MockExchange:** Testing without real APIs - fast, safe, deterministic

**Factory:** Creates exchanges by name - centralizes initialization

Together these components enable: price monitoring, arbitrage detection, trade execution, and profit calculation - all the core functionality needed for a cryptocurrency arbitrage bot.
