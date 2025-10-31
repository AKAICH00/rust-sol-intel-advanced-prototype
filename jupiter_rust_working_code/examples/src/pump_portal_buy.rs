//! PumpPortal Buy Example
//!
//! This example demonstrates how to execute a buy order using the PumpPortal SDK.
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
//!    cargo run --bin pump-portal-buy
//!    ```

use pump_portal_sdk::PumpPortalClient;
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

    println!("🚀 PumpPortal Buy Example\n");

    // Example token mint address (replace with actual token)
    let token_mint = env::var("TOKEN_MINT")
        .unwrap_or_else(|_| "REPLACE_WITH_TOKEN_MINT".to_string());

    if token_mint == "REPLACE_WITH_TOKEN_MINT" {
        println!("⚠️  Please set TOKEN_MINT in your .env file");
        println!("   Example: TOKEN_MINT=YourTokenMintAddressHere");
        return Ok(());
    }

    // Buy parameters
    let sol_amount = 0.01;  // 0.01 SOL
    let slippage = 10;      // 10% slippage
    let priority_fee = 0.0001; // Priority fee

    println!("📊 Trade Parameters:");
    println!("   Token: {}", token_mint);
    println!("   Amount: {} SOL", sol_amount);
    println!("   Slippage: {}%", slippage);
    println!("   Priority Fee: {} SOL", priority_fee);
    println!();

    // Execute buy order
    println!("⏳ Executing buy order...\n");

    match client.buy(token_mint.clone(), sol_amount, slippage, priority_fee).await {
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
    println!("💡 Advanced Usage:");
    println!("   You can also use the trade() method for more control:");
    println!("   ");
    println!("   use pump_portal_sdk::{{TradeRequest, Pool}};");
    println!("   ");
    println!("   let request = TradeRequest::buy(token_mint, sol_amount, slippage, priority_fee)");
    println!("       .with_pool(Pool::Raydium)");
    println!("       .with_skip_preflight(false)");
    println!("       .with_jito_only(true);");
    println!("   ");
    println!("   let response = client.trade(request).await?;");

    Ok(())
}
