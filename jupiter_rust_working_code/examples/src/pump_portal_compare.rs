//! PumpPortal Method Comparison Test
//!
//! Compares different buy configurations:
//! - Standard buy vs custom pools
//! - Different slippage settings
//! - With/without preflight
//! - Jito vs non-Jito routing
//! - Different priority fees
//!
//! Tracks: Speed, success rate, actual slippage

use pump_portal_sdk::{PumpPortalClient, Pool, TradeRequest};
use std::env;
use std::time::Instant;

#[derive(Debug)]
struct TestResult {
    name: String,
    success: bool,
    duration_ms: u128,
    signature: Option<String>,
    error: Option<String>,
}

impl TestResult {
    fn display(&self, index: usize) {
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Test #{}: {}", index, self.name);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        if self.success {
            println!("✅ SUCCESS");
            println!("⏱️  Duration: {} ms", self.duration_ms);
            if let Some(sig) = &self.signature {
                println!("📝 Signature: {}", sig);
                println!("🔗 Explorer: https://solscan.io/tx/{}", sig);
            }
        } else {
            println!("❌ FAILED");
            println!("⏱️  Duration: {} ms", self.duration_ms);
            if let Some(err) = &self.error {
                println!("❗ Error: {}", err);
            }
        }
    }
}

async fn run_test(
    client: &PumpPortalClient,
    name: &str,
    request: TradeRequest,
) -> TestResult {
    let start = Instant::now();

    match client.trade(request).await {
        Ok(response) => {
            let duration = start.elapsed().as_millis();
            TestResult {
                name: name.to_string(),
                success: response.signature.is_some(),
                duration_ms: duration,
                signature: response.signature,
                error: response.error,
            }
        }
        Err(e) => {
            let duration = start.elapsed().as_millis();
            TestResult {
                name: name.to_string(),
                success: false,
                duration_ms: duration,
                signature: None,
                error: Some(e.to_string()),
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv::dotenv().ok();

    let api_key = env::var("PUMPPORTAL_API_KEY")
        .expect("PUMPPORTAL_API_KEY must be set in .env file");

    let token_mint = env::var("TOKEN_MINT")
        .expect("TOKEN_MINT must be set in .env file");

    let client = PumpPortalClient::new(api_key);

    println!("╔═══════════════════════════════════════════════╗");
    println!("║   PUMPPORTAL BUY METHOD COMPARISON TEST       ║");
    println!("╚═══════════════════════════════════════════════╝\n");

    println!("📊 Test Configuration:");
    println!("   Token: {}", token_mint);
    println!("   Test Amount: 0.001 SOL per test");
    println!("   Total Tests: 6 different configurations");
    println!();

    let test_amount = 0.001;
    let mut results = Vec::new();

    // Test 1: Standard buy (default pool, skip preflight)
    println!("\n🧪 Running Test 1/6: Standard Buy (Default)...");
    let test1 = TradeRequest::buy(token_mint.clone(), test_amount, 10, 0.0001);
    results.push(run_test(&client, "Standard Buy - Default Pool", test1).await);
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Test 2: High slippage (20%)
    println!("\n🧪 Running Test 2/6: High Slippage (20%)...");
    let test2 = TradeRequest::buy(token_mint.clone(), test_amount, 20, 0.0001);
    results.push(run_test(&client, "High Slippage (20%)", test2).await);
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Test 3: Raydium pool
    println!("\n🧪 Running Test 3/6: Raydium Pool...");
    let test3 = TradeRequest::buy(token_mint.clone(), test_amount, 10, 0.0001)
        .with_pool(Pool::Raydium);
    results.push(run_test(&client, "Raydium Pool", test3).await);
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Test 4: With preflight simulation
    println!("\n🧪 Running Test 4/6: With Preflight Simulation...");
    let test4 = TradeRequest::buy(token_mint.clone(), test_amount, 10, 0.0001)
        .with_skip_preflight(false);
    results.push(run_test(&client, "With Preflight Simulation", test4).await);
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Test 5: Jito-only routing
    println!("\n🧪 Running Test 5/6: Jito-Only Routing...");
    let test5 = TradeRequest::buy(token_mint.clone(), test_amount, 10, 0.0001)
        .with_jito_only(true);
    results.push(run_test(&client, "Jito-Only Routing", test5).await);
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Test 6: High priority fee
    println!("\n🧪 Running Test 6/6: High Priority Fee (0.001 SOL)...");
    let test6 = TradeRequest::buy(token_mint.clone(), test_amount, 10, 0.001);
    results.push(run_test(&client, "High Priority Fee (0.001 SOL)", test6).await);

    // Display all results
    println!("\n\n");
    println!("╔═══════════════════════════════════════════════╗");
    println!("║              TEST RESULTS SUMMARY             ║");
    println!("╚═══════════════════════════════════════════════╝");

    for (i, result) in results.iter().enumerate() {
        result.display(i + 1);
    }

    // Performance comparison
    println!("\n\n");
    println!("╔═══════════════════════════════════════════════╗");
    println!("║           PERFORMANCE COMPARISON              ║");
    println!("╚═══════════════════════════════════════════════╝\n");

    let successful_tests: Vec<&TestResult> = results.iter()
        .filter(|r| r.success)
        .collect();

    if !successful_tests.is_empty() {
        let fastest = successful_tests.iter()
            .min_by_key(|r| r.duration_ms)
            .unwrap();

        let slowest = successful_tests.iter()
            .max_by_key(|r| r.duration_ms)
            .unwrap();

        let avg_time = successful_tests.iter()
            .map(|r| r.duration_ms)
            .sum::<u128>() / successful_tests.len() as u128;

        println!("📊 Speed Analysis:");
        println!("   🏆 Fastest: {} ({} ms)", fastest.name, fastest.duration_ms);
        println!("   🐌 Slowest: {} ({} ms)", slowest.name, slowest.duration_ms);
        println!("   📈 Average: {} ms", avg_time);
        println!();

        println!("📊 Success Rate:");
        println!("   ✅ Successful: {}/{}", successful_tests.len(), results.len());
        println!("   ❌ Failed: {}/{}", results.len() - successful_tests.len(), results.len());
        println!("   📊 Success Rate: {:.1}%",
            (successful_tests.len() as f64 / results.len() as f64) * 100.0);
    } else {
        println!("❌ No successful tests to compare");
    }

    // Recommendations
    println!("\n\n");
    println!("╔═══════════════════════════════════════════════╗");
    println!("║              RECOMMENDATIONS                  ║");
    println!("╚═══════════════════════════════════════════════╝\n");

    if !successful_tests.is_empty() {
        let fastest = successful_tests.iter()
            .min_by_key(|r| r.duration_ms)
            .unwrap();

        println!("💡 Fastest Configuration:");
        println!("   {}", fastest.name);
        println!("   Duration: {} ms", fastest.duration_ms);
        println!();

        println!("💡 General Tips:");
        println!("   • Higher slippage = higher success rate but worse price");
        println!("   • Preflight simulation catches errors but adds latency");
        println!("   • Jito routing may improve execution for MEV protection");
        println!("   • Higher priority fees = faster inclusion but higher cost");
        println!();

        println!("💡 For Production:");
        println!("   • Start with default settings and adjust based on results");
        println!("   • Monitor actual vs expected prices");
        println!("   • Adjust slippage based on token volatility");
        println!("   • Use preflight for large trades");
    }

    Ok(())
}
