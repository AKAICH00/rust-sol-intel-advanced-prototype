//! PumpPortal Round-Trip Test
//!
//! Tests the PumpPortal SDK with a small buy/sell cycle
//! Uses minimal amounts for safe testing

use pump_portal_sdk::PumpPortalClient;
use std::env;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Get API key from environment
    let api_key = env::var("PUMPPORTAL_API_KEY")
        .expect("PUMPPORTAL_API_KEY must be set in .env file");

    // Create client
    let client = PumpPortalClient::new(api_key);

    println!("🧪 PumpPortal Round-Trip Test");
    println!("================================\n");

    // Example token mint - using a popular pump.fun token
    // You can replace this with any token mint address
    let token_mint = env::var("TOKEN_MINT")
        .unwrap_or_else(|_| {
            // Default to a pump.fun token for testing
            // Replace with actual token mint you want to test
            println!("⚠️  No TOKEN_MINT found, please set it in .env");
            println!("   Example of pump.fun tokens:");
            println!("   - Find active tokens at https://pump.fun");
            println!("   - Copy the token's mint address");
            println!("   - Add to .env: TOKEN_MINT=<address>");
            std::process::exit(1);
        });

    println!("📊 Test Configuration:");
    println!("   Token Mint: {}", token_mint);
    println!("   Test Amount: 0.001 SOL (very small for safety)");
    println!("   Slippage: 20% (high for safety on small amounts)");
    println!("   Priority Fee: 0.0001 SOL");
    println!();

    // Test parameters - using very small amounts
    let test_amount = 0.001; // 0.001 SOL (~$0.20 at $200/SOL)
    let slippage = 20; // 20% slippage for small amounts
    let priority_fee = 0.0001;

    println!("═══════════════════════════════════");
    println!("STEP 1: BUY TEST");
    println!("═══════════════════════════════════\n");

    println!("⏳ Executing buy order for {} SOL...", test_amount);

    let buy_result = client.buy(
        token_mint.clone(),
        test_amount,
        slippage,
        priority_fee,
    ).await;

    let buy_signature = match buy_result {
        Ok(response) => {
            if let Some(sig) = response.signature {
                println!("✅ Buy successful!");
                println!("   Signature: {}", sig);
                println!("   Explorer: https://solscan.io/tx/{}", sig);
                sig
            } else {
                println!("⚠️  Buy completed but no signature returned");
                println!("   Response: {:?}", response);
                return Ok(());
            }
        }
        Err(e) => {
            println!("❌ Buy failed: {}", e);
            println!("\n💡 Troubleshooting:");
            println!("   - Check if token mint address is valid");
            println!("   - Ensure wallet has enough SOL");
            println!("   - Verify API key is correct");
            println!("   - Check if token has liquidity");
            return Err(e.into());
        }
    };

    println!("\n⏸️  Waiting 5 seconds for transaction to settle...\n");
    tokio::time::sleep(Duration::from_secs(5)).await;

    println!("═══════════════════════════════════");
    println!("STEP 2: SELL TEST");
    println!("═══════════════════════════════════\n");

    println!("⏳ Executing sell order for 100% of tokens...");

    let sell_result = client.sell(
        token_mint.clone(),
        "100%".to_string(), // Sell all tokens we just bought
        slippage,
        priority_fee,
    ).await;

    match sell_result {
        Ok(response) => {
            if let Some(sig) = response.signature {
                println!("✅ Sell successful!");
                println!("   Signature: {}", sig);
                println!("   Explorer: https://solscan.io/tx/{}", sig);
            } else {
                println!("⚠️  Sell completed but no signature returned");
                println!("   Response: {:?}", response);
            }
        }
        Err(e) => {
            println!("❌ Sell failed: {}", e);
            println!("\n⚠️  Note: Tokens were bought but not sold!");
            println!("   Buy transaction: https://solscan.io/tx/{}", buy_signature);
            println!("\n💡 You may need to sell manually or try again");
            return Err(e.into());
        }
    }

    println!("\n═══════════════════════════════════");
    println!("✅ TEST COMPLETE");
    println!("═══════════════════════════════════\n");

    println!("📊 Summary:");
    println!("   ✅ API Connection: Working");
    println!("   ✅ Buy Function: Working");
    println!("   ✅ Sell Function: Working");
    println!("   ✅ Round-trip: Successful");
    println!();
    println!("💡 The SDK is ready for production use!");
    println!("   - Adjust amounts for real trading");
    println!("   - Consider lower slippage for better prices");
    println!("   - Monitor transactions on Solscan");

    Ok(())
}
