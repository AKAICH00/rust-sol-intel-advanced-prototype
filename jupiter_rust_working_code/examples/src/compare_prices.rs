//! Price Comparison: Jupiter vs PumpPortal
//!
//! Compare actual tokens received, not just speed

mod lib;

use pump_portal_sdk::{PumpPortalClient, TradeRequest};
use jup::sign_transaction;
use dotenv::dotenv;
use std::env;
use std::time::Instant;
use solana_sdk::signature::{Keypair, Signer};
use serde::{Deserialize, Serialize};

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
    total_input_amount: Option<String>,
    total_output_amount: Option<String>,
    input_amount_result: Option<String>,
    output_amount_result: Option<String>,
}

#[derive(Debug)]
struct PriceResult {
    method: String,
    success: bool,
    time_ms: u128,
    signature: Option<String>,
    sol_input: f64,
    tokens_output: Option<f64>,
    price_per_token_sol: Option<f64>,
    error: Option<String>,
}

impl PriceResult {
    fn display(&self) {
        println!("\n╔═══════════════════════════════════════════════╗");
        println!("║  {}", self.method.to_uppercase());
        println!("╚═══════════════════════════════════════════════╝\n");

        if self.success {
            println!("✅ SUCCESS ({} ms)", self.time_ms);
            println!("\n💰 PRICING:");
            println!("   Input:  {} SOL", self.sol_input);

            if let Some(tokens) = self.tokens_output {
                println!("   Output: {:.2} tokens", tokens);

                if let Some(price) = self.price_per_token_sol {
                    println!("   Price:  {:.10} SOL per token", price);
                }
            }

            if let Some(sig) = &self.signature {
                println!("\n📝 Signature: {}", sig);
                println!("🔗 https://solscan.io/tx/{}", sig);
            }
        } else {
            println!("❌ FAILED ({} ms)", self.time_ms);
            if let Some(err) = &self.error {
                println!("\n❗ Error: {}", err);
            }
        }
    }
}

async fn test_jupiter_price(token_mint: &str, test_amount_sol: f64) -> PriceResult {
    let start = Instant::now();

    println!("🧪 Testing Jupiter Ultra + Helius");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let key = env::var("PRIVATE_KEY").expect("PRIVATE_KEY not set");
    let helius_url = env::var("HELIUS_RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());

    let key_bytes = bs58::decode(&key).into_vec().expect("Failed to decode");
    let keypair = Keypair::from_bytes(&key_bytes).expect("Failed to create Keypair");
    let wallet_address = keypair.pubkey().to_string();

    let http_client = reqwest::Client::new();
    let amount_lamports = (test_amount_sol * 1_000_000_000.0) as u64;

    // Get quote
    let url = format!(
        "https://lite-api.jup.ag/ultra/v1/order?inputMint={}&outputMint={}&amount={}&taker={}",
        "So11111111111111111111111111111111111111112",
        token_mint,
        amount_lamports,
        wallet_address
    );

    println!("⏳ Getting quote...");
    let quote_result = http_client.get(&url).send().await;

    let quote: QuoteResponse = match quote_result {
        Ok(resp) => {
            if !resp.status().is_success() {
                let body = resp.text().await.unwrap_or_default();
                return PriceResult {
                    method: "Jupiter + Helius".to_string(),
                    success: false,
                    time_ms: start.elapsed().as_millis(),
                    signature: None,
                    sol_input: test_amount_sol,
                    tokens_output: None,
                    price_per_token_sol: None,
                    error: Some(format!("Quote failed: {}", body)),
                };
            }
            resp.json().await.unwrap()
        }
        Err(e) => {
            return PriceResult {
                method: "Jupiter + Helius".to_string(),
                success: false,
                time_ms: start.elapsed().as_millis(),
                signature: None,
                sol_input: test_amount_sol,
                tokens_output: None,
                price_per_token_sol: None,
                error: Some(format!("Request error: {}", e)),
            };
        }
    };

    println!("✅ Quote: {} tokens expected", quote.out_amount);

    // Sign
    let signed_tx = sign_transaction(quote.transaction.clone());

    // Execute
    println!("⏳ Executing swap...");
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
            let body_text = resp.text().await.unwrap_or_default();

            if let Ok(execute_response) = serde_json::from_str::<ExecuteResponse>(&body_text) {
                if execute_response.status.to_lowercase() == "success" {
                    let tokens_out = execute_response.output_amount_result
                        .as_ref()
                        .and_then(|s| s.parse::<f64>().ok());

                    let price_per_token = tokens_out.map(|t| test_amount_sol / t);

                    println!("✅ Swap executed!");
                    println!("   Actual output: {:?} tokens", tokens_out);

                    PriceResult {
                        method: "Jupiter + Helius".to_string(),
                        success: true,
                        time_ms: start.elapsed().as_millis(),
                        signature: execute_response.signature,
                        sol_input: test_amount_sol,
                        tokens_output: tokens_out,
                        price_per_token_sol: price_per_token,
                        error: None,
                    }
                } else {
                    PriceResult {
                        method: "Jupiter + Helius".to_string(),
                        success: false,
                        time_ms: start.elapsed().as_millis(),
                        signature: None,
                        sol_input: test_amount_sol,
                        tokens_output: None,
                        price_per_token_sol: None,
                        error: execute_response.error,
                    }
                }
            } else {
                PriceResult {
                    method: "Jupiter + Helius".to_string(),
                    success: false,
                    time_ms: start.elapsed().as_millis(),
                    signature: None,
                    sol_input: test_amount_sol,
                    tokens_output: None,
                    price_per_token_sol: None,
                    error: Some(body_text),
                }
            }
        }
        Err(e) => PriceResult {
            method: "Jupiter + Helius".to_string(),
            success: false,
            time_ms: start.elapsed().as_millis(),
            signature: None,
            sol_input: test_amount_sol,
            tokens_output: None,
            price_per_token_sol: None,
            error: Some(format!("Execute error: {}", e)),
        },
    }
}

async fn test_pumpportal_price(token_mint: &str, test_amount_sol: f64) -> PriceResult {
    let start = Instant::now();

    println!("\n\n🧪 Testing PumpPortal Lightning");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let api_key = env::var("PUMPPORTAL_API_KEY").expect("PUMPPORTAL_API_KEY not set");
    let client = PumpPortalClient::new(api_key);

    println!("⏳ Executing buy...");

    let request = TradeRequest::buy(
        token_mint.to_string(),
        test_amount_sol,
        10,
        0.0001,
    )
    .with_jito_only(true);

    match client.trade(request).await {
        Ok(response) => {
            if let Some(sig) = response.signature {
                println!("✅ Buy executed!");
                println!("   Note: PumpPortal doesn't return output amount in API response");
                println!("   Check transaction on Solscan for actual tokens received");

                PriceResult {
                    method: "PumpPortal Lightning".to_string(),
                    success: true,
                    time_ms: start.elapsed().as_millis(),
                    signature: Some(sig),
                    sol_input: test_amount_sol,
                    tokens_output: None, // PumpPortal API doesn't return this
                    price_per_token_sol: None,
                    error: None,
                }
            } else {
                PriceResult {
                    method: "PumpPortal Lightning".to_string(),
                    success: false,
                    time_ms: start.elapsed().as_millis(),
                    signature: None,
                    sol_input: test_amount_sol,
                    tokens_output: None,
                    price_per_token_sol: None,
                    error: response.error,
                }
            }
        }
        Err(e) => PriceResult {
            method: "PumpPortal Lightning".to_string(),
            success: false,
            time_ms: start.elapsed().as_millis(),
            signature: None,
            sol_input: test_amount_sol,
            tokens_output: None,
            price_per_token_sol: None,
            error: Some(e.to_string()),
        },
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    println!("╔═══════════════════════════════════════════════╗");
    println!("║        PRICE COMPARISON: JUPITER VS PUMPPORTAL║");
    println!("╚═══════════════════════════════════════════════╝\n");

    let token_mint = env::var("TOKEN_MINT").expect("TOKEN_MINT must be set");
    let test_amount = 0.001;

    println!("📊 Test Configuration:");
    println!("   Token: {}", token_mint);
    println!("   Amount: {} SOL", test_amount);
    println!("   Goal: Compare actual tokens received\n");

    // Test Jupiter
    let jupiter_result = test_jupiter_price(&token_mint, test_amount).await;

    // Wait
    println!("\n⏸️  Waiting 3 seconds...\n");
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // Test PumpPortal
    let pumpportal_result = test_pumpportal_price(&token_mint, test_amount).await;

    // Display results
    jupiter_result.display();
    pumpportal_result.display();

    // Comparison
    println!("\n\n╔═══════════════════════════════════════════════╗");
    println!("║                 PRICE ANALYSIS                ║");
    println!("╚═══════════════════════════════════════════════╝\n");

    if jupiter_result.success && pumpportal_result.success {
        println!("⏱️  SPEED:");
        println!("   Jupiter:    {} ms", jupiter_result.time_ms);
        println!("   PumpPortal: {} ms", pumpportal_result.time_ms);

        if let Some(jup_tokens) = jupiter_result.tokens_output {
            println!("\n💰 TOKENS RECEIVED:");
            println!("   Jupiter:    {:.2} tokens", jup_tokens);
            println!("   PumpPortal: Check Solscan (API doesn't return amount)");

            if let Some(jup_price) = jupiter_result.price_per_token_sol {
                println!("\n📊 PRICE PER TOKEN:");
                println!("   Jupiter:    {:.10} SOL/token", jup_price);
                println!("   PumpPortal: Check Solscan for comparison");
            }
        }

        println!("\n🔗 COMPARE ON SOLSCAN:");
        if let Some(sig) = &jupiter_result.signature {
            println!("   Jupiter:    https://solscan.io/tx/{}", sig);
        }
        if let Some(sig) = &pumpportal_result.signature {
            println!("   PumpPortal: https://solscan.io/tx/{}", sig);
        }

        println!("\n💡 TO COMPARE PRICES:");
        println!("   1. Open both transactions on Solscan");
        println!("   2. Look at 'Token Balances' section");
        println!("   3. Compare how many tokens each received");
        println!("   4. The one with MORE tokens = better price execution");
    } else {
        println!("⚠️  One or both trades failed");
        println!("   Jupiter:    {}", if jupiter_result.success { "✅" } else { "❌" });
        println!("   PumpPortal: {}", if pumpportal_result.success { "✅" } else { "❌" });
    }
}
