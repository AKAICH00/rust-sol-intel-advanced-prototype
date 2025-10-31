//! PumpPortal Sell Example
//!
//! This example demonstrates how to execute a sell order using the PumpPortal SDK.
//!
//! # Setup
//!
//! 1. Add your API key to the .env file:
//!    ```
//!    PUMP_PORTAL_API_KEY=your-api-key-here
//!    ```
//!
//! 2. Run the example:
//!    ```
//!    cargo run --bin pump-portal-sell
//!    ```

use pump_portal_sdk::{PumpPortalClient, Pool, TradeRequest};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Get API key from environment
    let api_key = env::var("PUMPPORTAL_API_KEY")
        .expect("PUMPPORTAL_API_KEY must be set in .env file");

    // Create client
    let client = PumpPortalClient::new(api_key);

    println!("🚀 PumpPortal Sell Example\n");

    // Example token mint address (replace with actual token)
    let token_mint = env::var("TOKEN_MINT")
        .unwrap_or_else(|_| "REPLACE_WITH_TOKEN_MINT".to_string());

    if token_mint == "REPLACE_WITH_TOKEN_MINT" {
        println!("⚠️  Please set TOKEN_MINT in your .env file");
        println!("   Example: TOKEN_MINT=YourTokenMintAddressHere");
        return Ok(());
    }

    println!("📋 Example 1: Simple Sell (sell all tokens)");
    println!("============================================\n");

    // Sell parameters
    let token_amount = "100%".to_string(); // Sell 100% of tokens
    let slippage = 10;      // 10% slippage
    let priority_fee = 0.0001; // Priority fee

    println!("📊 Trade Parameters:");
    println!("   Token: {}", token_mint);
    println!("   Amount: {}", token_amount);
    println!("   Slippage: {}%", slippage);
    println!("   Priority Fee: {} SOL", priority_fee);
    println!();

    // Execute sell order
    println!("⏳ Executing sell order...\n");

    match client.sell(token_mint.clone(), token_amount, slippage, priority_fee).await {
        Ok(response) => {
            if let Some(signature) = response.signature {
                println!("✅ Trade successful!");
                println!("   Signature: {}", signature);
                println!("   Explorer: https://solscan.io/tx/{}", signature);
            } else {
                println!("⚠️  Trade completed but no signature returned");
                println!("   Response: {:?}", response);
            }
        }
        Err(e) => {
            println!("❌ Trade failed: {}", e);
        }
    }

    println!("\n");
    println!("📋 Example 2: Advanced Sell with Custom Settings");
    println!("=================================================\n");

    // Advanced sell with custom pool and settings
    let advanced_request = TradeRequest::sell(
        token_mint.clone(),
        "50%".to_string(), // Sell 50% of tokens
        15, // 15% slippage
        0.0005, // Higher priority fee
    )
    .with_pool(Pool::Raydium) // Use Raydium instead of default
    .with_skip_preflight(false) // Enable preflight simulation
    .with_jito_only(true); // Route through Jito only

    println!("📊 Advanced Trade Parameters:");
    println!("   Token: {}", token_mint);
    println!("   Amount: 50%");
    println!("   Slippage: 15%");
    println!("   Priority Fee: 0.0005 SOL");
    println!("   Pool: Raydium");
    println!("   Skip Preflight: false (simulation enabled)");
    println!("   Jito Only: true");
    println!();

    println!("⏳ Executing advanced sell order...\n");

    match client.trade(advanced_request).await {
        Ok(response) => {
            if let Some(signature) = response.signature {
                println!("✅ Trade successful!");
                println!("   Signature: {}", signature);
                println!("   Explorer: https://solscan.io/tx/{}", signature);
            } else {
                println!("⚠️  Trade completed but no signature returned");
                println!("   Response: {:?}", response);
            }
        }
        Err(e) => {
            println!("❌ Trade failed: {}", e);
        }
    }

    println!();
    println!("💡 Tips:");
    println!("   - Use percentage amounts (\"50%\", \"100%\") to sell portions of your holdings");
    println!("   - Or specify exact token amounts as strings");
    println!("   - Adjust slippage based on market conditions (higher for volatile markets)");
    println!("   - Use higher priority fees during network congestion");
    println!("   - Enable preflight simulation to catch errors before sending");

    Ok(())
}
