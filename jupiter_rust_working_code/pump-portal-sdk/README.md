# PumpPortal Trading API SDK for Rust

A Rust SDK for interacting with the [PumpPortal Lightning Transaction API](https://pumpportal.fun/trading-api/). Execute fast buy and sell trades on Solana with minimal latency.

## Features

- ✅ Simple async/await API
- ✅ Type-safe request/response handling
- ✅ Built-in error handling
- ✅ Support for all PumpPortal pools (Pump, Raydium, etc.)
- ✅ Flexible trade parameters
- ✅ Comprehensive examples

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
pump-portal-sdk = { path = "../pump-portal-sdk" }
tokio = { version = "1.0", features = ["full"] }
```

## Quick Start

### Basic Buy Order

```rust
use pump_portal_sdk::PumpPortalClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with API key
    let client = PumpPortalClient::new("your-api-key".to_string());

    // Execute buy order
    let response = client.buy(
        "TokenMintAddress".to_string(),
        0.1,      // 0.1 SOL
        10,       // 10% slippage
        0.0001,   // priority fee
    ).await?;

    if let Some(signature) = response.signature {
        println!("Trade successful: {}", signature);
    }

    Ok(())
}
```

### Basic Sell Order

```rust
use pump_portal_sdk::PumpPortalClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = PumpPortalClient::new("your-api-key".to_string());

    // Sell 100% of tokens
    let response = client.sell(
        "TokenMintAddress".to_string(),
        "100%".to_string(), // Sell all tokens
        10,       // 10% slippage
        0.0001,   // priority fee
    ).await?;

    if let Some(signature) = response.signature {
        println!("Trade successful: {}", signature);
    }

    Ok(())
}
```

## Advanced Usage

### Custom Pool Selection

```rust
use pump_portal_sdk::{PumpPortalClient, Pool, TradeRequest};

let client = PumpPortalClient::new("your-api-key".to_string());

let request = TradeRequest::buy(
    "TokenMintAddress".to_string(),
    0.1,
    10,
    0.0001,
)
.with_pool(Pool::Raydium)           // Use Raydium instead of default
.with_skip_preflight(false)         // Enable preflight simulation
.with_jito_only(true);              // Route through Jito only

let response = client.trade(request).await?;
```

### Available Pools

```rust
use pump_portal_sdk::Pool;

Pool::Pump          // Pump.fun (default)
Pool::Raydium       // Raydium
Pool::PumpAmm       // Pump AMM
Pool::Launchlab     // LaunchLab
Pool::RaydiumCpmm   // Raydium CPMM
Pool::Bonk          // Bonk
Pool::Auto          // Auto-select best pool
```

## API Reference

### `PumpPortalClient`

#### Methods

- `new(api_key: String) -> Self`
  - Create a new client instance

- `buy(mint: String, sol_amount: f64, slippage: u32, priority_fee: f64) -> Result<TradeResponse>`
  - Execute a buy order

- `sell(mint: String, token_amount: String, slippage: u32, priority_fee: f64) -> Result<TradeResponse>`
  - Execute a sell order

- `trade(request: TradeRequest) -> Result<TradeResponse>`
  - Execute a custom trade request

### `TradeRequest`

#### Builders

- `buy(mint, sol_amount, slippage, priority_fee) -> Self`
  - Create a buy request

- `sell(mint, token_amount, slippage, priority_fee) -> Self`
  - Create a sell request

#### Configuration

- `with_pool(pool: Pool) -> Self`
  - Set the pool/exchange

- `with_skip_preflight(skip: bool) -> Self`
  - Enable/disable preflight simulation

- `with_jito_only(jito: bool) -> Self`
  - Enable/disable Jito-only routing

### `TradeResponse`

```rust
pub struct TradeResponse {
    pub signature: Option<String>,  // Transaction signature if successful
    pub error: Option<String>,       // Error message if failed
    pub extra: serde_json::Value,    // Additional response data
}
```

## Error Handling

The SDK uses the `PumpPortalError` enum for all errors:

```rust
use pump_portal_sdk::{PumpPortalClient, PumpPortalError};

match client.buy(mint, 0.1, 10, 0.0001).await {
    Ok(response) => {
        println!("Success: {:?}", response.signature);
    }
    Err(PumpPortalError::ApiError(msg)) => {
        eprintln!("API error: {}", msg);
    }
    Err(PumpPortalError::RequestFailed(e)) => {
        eprintln!("Request failed: {}", e);
    }
    Err(e) => {
        eprintln!("Other error: {}", e);
    }
}
```

## Examples

Run the included examples:

```bash
# Setup environment
cp examples/.env.example examples/.env
# Edit .env and add your PUMP_PORTAL_API_KEY and TOKEN_MINT

# Run buy example
cargo run --bin pump-portal-buy

# Run sell example
cargo run --bin pump-portal-sell
```

## Parameters Guide

### Slippage

Acceptable slippage percentage. Higher values allow more price movement:
- Low volatility: `5-10%`
- Medium volatility: `10-20%`
- High volatility: `20-50%`

### Priority Fee

Transaction speed enhancement fee in SOL:
- Normal: `0.0001-0.001 SOL`
- Fast: `0.001-0.01 SOL`
- Urgent: `0.01+ SOL`

### Token Amount (Sell)

Can be specified as:
- Percentage: `"100%"`, `"50%"`, `"25%"`
- Absolute: `"1000000"` (token amount)

## API Documentation

For full API details, visit: https://pumpportal.fun/trading-api/

## License

See LICENSE file in repository root.
