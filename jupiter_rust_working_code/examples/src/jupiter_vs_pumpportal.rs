//! Jupiter Ultra + Helius vs PumpPortal Lightning Comparison
//!
//! Head-to-head comparison of two trading methods:
//! 1. Jupiter Ultra API + Helius Premium RPC
//! 2. PumpPortal Lightning API (dedicated wallet system)

mod lib;

use pump_portal_sdk::{PumpPortalClient, TradeRequest};
use jup::sign_transaction;
use dotenv::dotenv;
use std::env;
use std::time::Instant;
use solana_sdk::signature::{Keypair, Signer};
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct QuoteResponse {
    transaction: String,
    request_id: String,
    in_amount: String,
    out_amount: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExecuteRequest {
    signed_transaction: String,
    request_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExecuteResponse {
    status: String,
    signature: Option<String>,
    error: Option<String>,
}

#[derive(Debug)]
struct TestResult {
    method: String,
    total_time_ms: u128,
    quote_time_ms: Option<u128>,
    sign_time_ms: Option<u128>,
    execute_time_ms: Option<u128>,
    success: bool,
    signature: Option<String>,
    error: Option<String>,
}

impl TestResult {
    fn display(&self) {
        println!("\n╔═══════════════════════════════════════════════╗");
        println!("║  {} RESULTS", self.method.to_uppercase());
        println!("╚═══════════════════════════════════════════════╝\n");

        if self.success {
            println!("✅ SUCCESS");
            println!("⏱️  Total Time: {} ms", self.total_time_ms);

            if let Some(quote_ms) = self.quote_time_ms {
                println!("   └─ Quote: {} ms", quote_ms);
            }
            if let Some(sign_ms) = self.sign_time_ms {
                println!("   └─ Sign: {} ms", sign_ms);
            }
            if let Some(exec_ms) = self.execute_time_ms {
                println!("   └─ Execute: {} ms", exec_ms);
            }

            if let Some(sig) = &self.signature {
                println!("\n📝 Signature: {}", sig);
                println!("🔗 https://solscan.io/tx/{}", sig);
            }
        } else {
            println!("❌ FAILED");
            println!("⏱️  Total Time: {} ms", self.total_time_ms);
            if let Some(err) = &self.error {
                println!("\n❗ Error Details:");
                println!("   {}", err);
            }
        }
    }
}

async fn test_jupiter_helius(
    token_mint: &str,
    test_amount_sol: f64,
) -> TestResult {
    let total_start = Instant::now();

    println!("🧪 Testing: Jupiter Ultra + Helius RPC");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Load config
    let key = env::var("PRIVATE_KEY").expect("PRIVATE_KEY not set");
    let helius_url = env::var("HELIUS_RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());

    let key_bytes = bs58::decode(&key).into_vec().expect("Failed to decode");
    let keypair = Keypair::from_bytes(&key_bytes).expect("Failed to create Keypair");
    let wallet_address = keypair.pubkey().to_string();

    println!("💼 Wallet: {}", wallet_address);
    println!("🌐 RPC: Helius Premium\n");

    let rpc_client = RpcClient::new_with_commitment(
        helius_url,
        CommitmentConfig::confirmed()
    );

    let http_client = reqwest::Client::new();
    let amount_lamports = (test_amount_sol * 1_000_000_000.0) as u64;

    // Step 1: Get quote
    println!("⏳ Step 1: Fetching quote from Jupiter Ultra...");
    let quote_start = Instant::now();

    let url = format!(
        "https://lite-api.jup.ag/ultra/v1/order?inputMint={}&outputMint={}&amount={}&taker={}",
        "So11111111111111111111111111111111111111112", // SOL
        token_mint,
        amount_lamports,
        wallet_address
    );

    let quote_result = http_client.get(&url).send().await;

    let quote: QuoteResponse = match quote_result {
        Ok(resp) => {
            if !resp.status().is_success() {
                let body = resp.text().await.unwrap_or_default();
                let total_time = total_start.elapsed().as_millis();
                return TestResult {
                    method: "Jupiter + Helius".to_string(),
                    total_time_ms: total_time,
                    quote_time_ms: Some(quote_start.elapsed().as_millis()),
                    sign_time_ms: None,
                    execute_time_ms: None,
                    success: false,
                    signature: None,
                    error: Some(format!("Quote failed: {}", body)),
                };
            }
            match resp.json().await {
                Ok(q) => {
                    let quote_duration = quote_start.elapsed().as_millis();
                    println!("✅ Quote received ({} ms)", quote_duration);
                    q
                }
                Err(e) => {
                    let total_time = total_start.elapsed().as_millis();
                    return TestResult {
                        method: "Jupiter + Helius".to_string(),
                        total_time_ms: total_time,
                        quote_time_ms: Some(quote_start.elapsed().as_millis()),
                        sign_time_ms: None,
                        execute_time_ms: None,
                        success: false,
                        signature: None,
                        error: Some(format!("Parse error: {}", e)),
                    };
                }
            }
        }
        Err(e) => {
            let total_time = total_start.elapsed().as_millis();
            return TestResult {
                method: "Jupiter + Helius".to_string(),
                total_time_ms: total_time,
                quote_time_ms: Some(quote_start.elapsed().as_millis()),
                sign_time_ms: None,
                execute_time_ms: None,
                success: false,
                signature: None,
                error: Some(format!("Request error: {}", e)),
            };
        }
    };

    let quote_time = quote_start.elapsed().as_millis();

    // Step 2: Sign transaction
    println!("⏳ Step 2: Signing transaction...");
    let sign_start = Instant::now();

    let signed_tx = sign_transaction(quote.transaction.clone());
    let sign_time = sign_start.elapsed().as_millis();
    println!("✅ Transaction signed ({} ms)", sign_time);

    // Step 3: Execute
    println!("⏳ Step 3: Executing swap via Helius...");
    let execute_start = Instant::now();

    let execute_req = ExecuteRequest {
        signed_transaction: signed_tx,
        request_id: quote.request_id,
    };

    let execute_result = http_client
        .post("https://lite-api.jup.ag/ultra/v1/execute")
        .json(&execute_req)
        .send()
        .await;

    match execute_result {
        Ok(resp) => {
            let status = resp.status();
            let body_text = resp.text().await.unwrap_or_default();
            let execute_time = execute_start.elapsed().as_millis();
            let total_time = total_start.elapsed().as_millis();

            // Try to parse as JSON
            if let Ok(execute_response) = serde_json::from_str::<ExecuteResponse>(&body_text) {
                if execute_response.status.to_lowercase() == "success" {
                    println!("✅ Swap executed ({} ms)", execute_time);
                    TestResult {
                        method: "Jupiter + Helius".to_string(),
                        total_time_ms: total_time,
                        quote_time_ms: Some(quote_time),
                        sign_time_ms: Some(sign_time),
                        execute_time_ms: Some(execute_time),
                        success: true,
                        signature: execute_response.signature,
                        error: None,
                    }
                } else {
                    let error_msg = execute_response.error.unwrap_or_else(|| body_text.clone());
                    println!("❌ Execution failed: {}", error_msg);
                    TestResult {
                        method: "Jupiter + Helius".to_string(),
                        total_time_ms: total_time,
                        quote_time_ms: Some(quote_time),
                        sign_time_ms: Some(sign_time),
                        execute_time_ms: Some(execute_time),
                        success: false,
                        signature: None,
                        error: Some(error_msg),
                    }
                }
            } else {
                // Failed to parse JSON
                let error_msg = format!("HTTP {}: {}", status, body_text);
                println!("❌ Execution failed: {}", error_msg);
                TestResult {
                    method: "Jupiter + Helius".to_string(),
                    total_time_ms: total_time,
                    quote_time_ms: Some(quote_time),
                    sign_time_ms: Some(sign_time),
                    execute_time_ms: Some(execute_time),
                    success: false,
                    signature: None,
                    error: Some(error_msg),
                }
            }
        }
        Err(e) => {
            let total_time = total_start.elapsed().as_millis();
            let error_msg = format!("Request error: {}", e);
            println!("❌ Execution failed: {}", error_msg);
            TestResult {
                method: "Jupiter + Helius".to_string(),
                total_time_ms: total_time,
                quote_time_ms: Some(quote_time),
                sign_time_ms: Some(sign_time),
                execute_time_ms: Some(execute_start.elapsed().as_millis()),
                success: false,
                signature: None,
                error: Some(error_msg),
            }
        }
    }
}

async fn test_pumpportal(
    token_mint: &str,
    test_amount_sol: f64,
) -> TestResult {
    let total_start = Instant::now();

    println!("\n\n🧪 Testing: PumpPortal Lightning API");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let api_key = env::var("PUMPPORTAL_API_KEY")
        .expect("PUMPPORTAL_API_KEY not set");

    let wallet_pubkey = env::var("Wallet_Public_Key")
        .unwrap_or_else(|_| "N/A".to_string());

    println!("💼 PumpPortal Wallet: {}", wallet_pubkey);
    println!("⚡ API: Lightning (Single Request)\n");

    let client = PumpPortalClient::new(api_key);

    println!("⏳ Executing buy (single API call)...");

    // PumpPortal is single-step (no quote, sign, execute - all handled by API)
    let request = TradeRequest::buy(
        token_mint.to_string(),
        test_amount_sol,
        10,  // 10% slippage
        0.0001,
    )
    .with_jito_only(true); // Use Jito for best speed

    match client.trade(request).await {
        Ok(response) => {
            let total_time = total_start.elapsed().as_millis();

            if let Some(sig) = response.signature {
                println!("✅ Buy executed ({} ms)", total_time);
                TestResult {
                    method: "PumpPortal Lightning".to_string(),
                    total_time_ms: total_time,
                    quote_time_ms: None,  // Single-step API
                    sign_time_ms: None,
                    execute_time_ms: None,
                    success: true,
                    signature: Some(sig),
                    error: None,
                }
            } else {
                TestResult {
                    method: "PumpPortal Lightning".to_string(),
                    total_time_ms: total_time,
                    quote_time_ms: None,
                    sign_time_ms: None,
                    execute_time_ms: None,
                    success: false,
                    signature: None,
                    error: response.error,
                }
            }
        }
        Err(e) => {
            let total_time = total_start.elapsed().as_millis();
            TestResult {
                method: "PumpPortal Lightning".to_string(),
                total_time_ms: total_time,
                quote_time_ms: None,
                sign_time_ms: None,
                execute_time_ms: None,
                success: false,
                signature: None,
                error: Some(e.to_string()),
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    println!("╔═══════════════════════════════════════════════╗");
    println!("║   JUPITER + HELIUS  vs  PUMPPORTAL LIGHTNING  ║");
    println!("╚═══════════════════════════════════════════════╝\n");

    let token_mint = env::var("TOKEN_MINT")
        .expect("TOKEN_MINT must be set in .env");

    let test_amount = 0.001; // 0.001 SOL

    println!("📊 Test Configuration:");
    println!("   Token: {}", token_mint);
    println!("   Amount: {} SOL", test_amount);
    println!("   Slippage: 10%");
    println!("   Priority Fee: 0.0001 SOL\n");

    // Test Jupiter + Helius
    let jupiter_result = test_jupiter_helius(&token_mint, test_amount).await;

    // Wait between tests
    println!("\n\n⏸️  Waiting 3 seconds between tests...\n");
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // Test PumpPortal
    let pumpportal_result = test_pumpportal(&token_mint, test_amount).await;

    // Display results
    jupiter_result.display();
    pumpportal_result.display();

    // Comparison
    println!("\n\n╔═══════════════════════════════════════════════╗");
    println!("║              HEAD-TO-HEAD COMPARISON          ║");
    println!("╚═══════════════════════════════════════════════╝\n");

    if jupiter_result.success && pumpportal_result.success {
        println!("⏱️  SPEED COMPARISON:");
        println!("   Jupiter + Helius:  {} ms", jupiter_result.total_time_ms);
        println!("   PumpPortal:        {} ms", pumpportal_result.total_time_ms);

        if pumpportal_result.total_time_ms < jupiter_result.total_time_ms {
            let speedup = jupiter_result.total_time_ms as f64 / pumpportal_result.total_time_ms as f64;
            println!("\n   🏆 PumpPortal is {:.1}x FASTER", speedup);
        } else {
            let speedup = pumpportal_result.total_time_ms as f64 / jupiter_result.total_time_ms as f64;
            println!("\n   🏆 Jupiter is {:.1}x FASTER", speedup);
        }

        println!("\n📊 BREAKDOWN:");
        if let (Some(q), Some(s), Some(e)) = (
            jupiter_result.quote_time_ms,
            jupiter_result.sign_time_ms,
            jupiter_result.execute_time_ms
        ) {
            println!("   Jupiter + Helius:");
            println!("      Quote:   {} ms", q);
            println!("      Sign:    {} ms", s);
            println!("      Execute: {} ms", e);
            println!("      Total:   {} ms", jupiter_result.total_time_ms);
        }

        println!("\n   PumpPortal Lightning:");
        println!("      Single API call: {} ms", pumpportal_result.total_time_ms);
        println!("      (quote, sign, execute all handled server-side)");

    } else {
        println!("⚠️  Cannot compare - one or both tests failed");
        println!("   Jupiter: {}", if jupiter_result.success { "✅" } else { "❌" });
        println!("   PumpPortal: {}", if pumpportal_result.success { "✅" } else { "❌" });
    }

    println!("\n\n💡 KEY DIFFERENCES:\n");
    println!("   JUPITER + HELIUS:");
    println!("   ✅ Uses your own wallet");
    println!("   ✅ Full control over transaction");
    println!("   ✅ Works with any token");
    println!("   ❌ 3-step process (quote → sign → execute)");
    println!("   ❌ Requires RPC connection");
    println!("   ❌ More latency\n");

    println!("   PUMPPORTAL LIGHTNING:");
    println!("   ✅ Single API call (instant)");
    println!("   ✅ Dedicated infrastructure");
    println!("   ✅ No RPC needed");
    println!("   ✅ Built-in Jito routing");
    println!("   ❌ Uses PumpPortal's wallet system");
    println!("   ❌ Optimized for pump.fun tokens");
}
