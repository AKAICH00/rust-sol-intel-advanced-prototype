//! Full Cost Analysis: Jupiter vs PumpPortal
//!
//! Analyzes:
//! - API costs
//! - Transaction fees
//! - Priority fees
//! - Actual SOL deducted from wallet
//! - Price execution (tokens received)

mod lib;

use pump_portal_sdk::{PumpPortalClient, TradeRequest};
use jup::sign_transaction;
use dotenv::dotenv;
use std::env;
use std::time::Instant;
use solana_sdk::signature::{Keypair, Signer};
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
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
    output_amount_result: Option<String>,
}

#[derive(Debug)]
struct CostAnalysis {
    method: String,
    success: bool,

    // Speed
    time_ms: u128,

    // Costs
    sol_balance_before: f64,
    sol_balance_after: f64,
    sol_deducted: f64,
    priority_fee_paid: f64,

    // Output
    tokens_received: Option<f64>,

    // References
    signature: Option<String>,
    wallet_used: String,

    error: Option<String>,
}

impl CostAnalysis {
    fn display(&self) {
        println!("\n╔═══════════════════════════════════════════════╗");
        println!("║  {} COST ANALYSIS", self.method.to_uppercase());
        println!("╚═══════════════════════════════════════════════╝\n");

        if self.success {
            println!("✅ SUCCESS ({} ms)", self.time_ms);

            println!("\n💰 COSTS:");
            println!("   SOL Balance Before: {:.6} SOL", self.sol_balance_before);
            println!("   SOL Balance After:  {:.6} SOL", self.sol_balance_after);
            println!("   Total Deducted:     {:.6} SOL", self.sol_deducted);
            println!("   Priority Fee:       {:.6} SOL", self.priority_fee_paid);

            if let Some(tokens) = self.tokens_received {
                println!("\n📊 OUTPUT:");
                println!("   Tokens Received:    {:.2}", tokens);

                if self.sol_deducted > 0.0 && tokens > 0.0 {
                    let cost_per_token = self.sol_deducted / tokens;
                    println!("   Cost per Token:     {:.10} SOL", cost_per_token);
                }
            }

            println!("\n🔗 DETAILS:");
            println!("   Wallet: {}", self.wallet_used);
            if let Some(sig) = &self.signature {
                println!("   Tx: https://solscan.io/tx/{}", sig);
            }
        } else {
            println!("❌ FAILED ({} ms)", self.time_ms);
            if let Some(err) = &self.error {
                println!("\n❗ Error: {}", err);
            }
        }
    }
}

async fn get_sol_balance(rpc_client: &RpcClient, wallet: &Pubkey) -> f64 {
    match rpc_client.get_balance(wallet) {
        Ok(lamports) => lamports as f64 / 1_000_000_000.0,
        Err(_) => 0.0,
    }
}

async fn analyze_jupiter_cost(
    token_mint: &str,
    test_amount_sol: f64,
) -> CostAnalysis {
    let start = Instant::now();

    println!("🧪 Analyzing Jupiter Ultra + Helius Costs");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let key = env::var("PRIVATE_KEY").expect("PRIVATE_KEY not set");
    let helius_url = env::var("HELIUS_RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());

    let key_bytes = bs58::decode(&key).into_vec().expect("Failed to decode");
    let keypair = Keypair::from_bytes(&key_bytes).expect("Failed to create Keypair");
    let wallet_pubkey = keypair.pubkey();
    let wallet_address = wallet_pubkey.to_string();

    let rpc_client = RpcClient::new_with_commitment(
        helius_url,
        CommitmentConfig::confirmed()
    );

    // Get balance before
    println!("📊 Checking wallet balance...");
    let balance_before = get_sol_balance(&rpc_client, &wallet_pubkey).await;
    println!("   Balance: {:.6} SOL", balance_before);

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

    println!("\n⏳ Getting quote...");
    let quote: QuoteResponse = match http_client.get(&url).send().await {
        Ok(resp) => resp.json().await.unwrap(),
        Err(e) => {
            return CostAnalysis {
                method: "Jupiter + Helius".to_string(),
                success: false,
                time_ms: start.elapsed().as_millis(),
                sol_balance_before: balance_before,
                sol_balance_after: balance_before,
                sol_deducted: 0.0,
                priority_fee_paid: 0.0001,
                tokens_received: None,
                signature: None,
                wallet_used: wallet_address,
                error: Some(e.to_string()),
            };
        }
    };

    println!("✅ Quote received: {} tokens expected", quote.out_amount);

    // Sign and execute
    let signed_tx = sign_transaction(quote.transaction.clone());

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
                    println!("✅ Swap executed!");

                    // Wait for confirmation
                    println!("⏳ Waiting for confirmation...");
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

                    // Get balance after
                    let balance_after = get_sol_balance(&rpc_client, &wallet_pubkey).await;
                    let sol_deducted = balance_before - balance_after;

                    println!("📊 Final balance: {:.6} SOL", balance_after);
                    println!("💸 Total cost: {:.6} SOL", sol_deducted);

                    let tokens_out = execute_response.output_amount_result
                        .as_ref()
                        .and_then(|s| s.parse::<f64>().ok());

                    CostAnalysis {
                        method: "Jupiter + Helius".to_string(),
                        success: true,
                        time_ms: start.elapsed().as_millis(),
                        sol_balance_before: balance_before,
                        sol_balance_after: balance_after,
                        sol_deducted,
                        priority_fee_paid: 0.0001,
                        tokens_received: tokens_out,
                        signature: execute_response.signature,
                        wallet_used: wallet_address,
                        error: None,
                    }
                } else {
                    CostAnalysis {
                        method: "Jupiter + Helius".to_string(),
                        success: false,
                        time_ms: start.elapsed().as_millis(),
                        sol_balance_before: balance_before,
                        sol_balance_after: balance_before,
                        sol_deducted: 0.0,
                        priority_fee_paid: 0.0001,
                        tokens_received: None,
                        signature: None,
                        wallet_used: wallet_address,
                        error: execute_response.error,
                    }
                }
            } else {
                CostAnalysis {
                    method: "Jupiter + Helius".to_string(),
                    success: false,
                    time_ms: start.elapsed().as_millis(),
                    sol_balance_before: balance_before,
                    sol_balance_after: balance_before,
                    sol_deducted: 0.0,
                    priority_fee_paid: 0.0001,
                    tokens_received: None,
                    signature: None,
                    wallet_used: wallet_address,
                    error: Some(body_text),
                }
            }
        }
        Err(e) => CostAnalysis {
            method: "Jupiter + Helius".to_string(),
            success: false,
            time_ms: start.elapsed().as_millis(),
            sol_balance_before: balance_before,
            sol_balance_after: balance_before,
            sol_deducted: 0.0,
            priority_fee_paid: 0.0001,
            tokens_received: None,
            signature: None,
            wallet_used: wallet_address,
            error: Some(e.to_string()),
        },
    }
}

async fn analyze_pumpportal_cost(
    token_mint: &str,
    test_amount_sol: f64,
) -> CostAnalysis {
    let start = Instant::now();

    println!("\n\n🧪 Analyzing PumpPortal Lightning Costs");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let api_key = env::var("PUMPPORTAL_API_KEY").expect("PUMPPORTAL_API_KEY not set");
    let pumpportal_wallet = env::var("Wallet_Public_Key").expect("Wallet_Public_Key not set");

    // Note: PumpPortal uses their own wallet, so we can't check balance before/after
    println!("⚠️  Note: PumpPortal uses their own wallet system");
    println!("   Wallet: {}", pumpportal_wallet);
    println!("   Cannot check balance changes (different wallet)");

    let client = PumpPortalClient::new(api_key);

    println!("\n⏳ Executing buy...");

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
                println!("\n💡 Cost breakdown:");
                println!("   Input amount: {:.6} SOL", test_amount_sol);
                println!("   Priority fee: 0.0001 SOL (specified)");
                println!("   Total: ~{:.6} SOL + network fees", test_amount_sol + 0.0001);
                println!("\n   Check Solscan for exact fees and output amount");

                CostAnalysis {
                    method: "PumpPortal Lightning".to_string(),
                    success: true,
                    time_ms: start.elapsed().as_millis(),
                    sol_balance_before: 0.0, // Can't check - different wallet
                    sol_balance_after: 0.0,
                    sol_deducted: test_amount_sol + 0.0001, // Estimate
                    priority_fee_paid: 0.0001,
                    tokens_received: None, // API doesn't return this
                    signature: Some(sig),
                    wallet_used: pumpportal_wallet,
                    error: None,
                }
            } else {
                CostAnalysis {
                    method: "PumpPortal Lightning".to_string(),
                    success: false,
                    time_ms: start.elapsed().as_millis(),
                    sol_balance_before: 0.0,
                    sol_balance_after: 0.0,
                    sol_deducted: 0.0,
                    priority_fee_paid: 0.0001,
                    tokens_received: None,
                    signature: None,
                    wallet_used: pumpportal_wallet,
                    error: response.error,
                }
            }
        }
        Err(e) => CostAnalysis {
            method: "PumpPortal Lightning".to_string(),
            success: false,
            time_ms: start.elapsed().as_millis(),
            sol_balance_before: 0.0,
            sol_balance_after: 0.0,
            sol_deducted: 0.0,
            priority_fee_paid: 0.0001,
            tokens_received: None,
            signature: None,
            wallet_used: pumpportal_wallet,
            error: Some(e.to_string()),
        },
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    println!("╔═══════════════════════════════════════════════╗");
    println!("║         COMPLETE COST ANALYSIS                ║");
    println!("║      Jupiter vs PumpPortal                    ║");
    println!("╚═══════════════════════════════════════════════╝\n");

    let token_mint = env::var("TOKEN_MINT").expect("TOKEN_MINT must be set");
    let test_amount = 0.001;

    println!("📊 Test Configuration:");
    println!("   Token: {}", token_mint);
    println!("   Amount: {} SOL", test_amount);
    println!("   Goal: Compare ALL costs\n");

    // Test Jupiter
    let jupiter_result = analyze_jupiter_cost(&token_mint, test_amount).await;

    // Wait
    println!("\n⏸️  Waiting 5 seconds...\n");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // Test PumpPortal
    let pumpportal_result = analyze_pumpportal_cost(&token_mint, test_amount).await;

    // Display results
    jupiter_result.display();
    pumpportal_result.display();

    // Final comparison
    println!("\n\n╔═══════════════════════════════════════════════╗");
    println!("║            FINAL COST COMPARISON              ║");
    println!("╚═══════════════════════════════════════════════╝\n");

    println!("💰 API COSTS:");
    println!("   Jupiter:    FREE (no API fees)");
    println!("   PumpPortal: FREE (no API fees mentioned on site)");
    println!("               Note: Check their docs for enterprise pricing\n");

    if jupiter_result.success {
        println!("💸 JUPITER TOTAL COST:");
        println!("   SOL Deducted:   {:.6} SOL", jupiter_result.sol_deducted);
        println!("   Priority Fee:   {:.6} SOL", jupiter_result.priority_fee_paid);
        println!("   Network Fees:   {:.6} SOL",
            jupiter_result.sol_deducted - test_amount - jupiter_result.priority_fee_paid);
        if let Some(tokens) = jupiter_result.tokens_received {
            println!("   Tokens Got:     {:.2}", tokens);
        }
    }

    if pumpportal_result.success {
        println!("\n💸 PUMPPORTAL ESTIMATED COST:");
        println!("   Input Amount:   {:.6} SOL", test_amount);
        println!("   Priority Fee:   {:.6} SOL", pumpportal_result.priority_fee_paid);
        println!("   Network Fees:   Check Solscan (can't verify - different wallet)");
        println!("   Tokens Got:     Check Solscan (API doesn't return)");
    }

    println!("\n🔍 TO GET EXACT COMPARISON:");
    println!("   1. Check both transactions on Solscan");
    println!("   2. Compare 'Token Balances' (who got more tokens)");
    println!("   3. Compare 'SOL Balance' changes (who spent less SOL)");
    println!("   4. Factor in your wallet preference (own vs managed)");

    if let Some(sig) = &jupiter_result.signature {
        println!("\n   Jupiter:    https://solscan.io/tx/{}", sig);
    }
    if let Some(sig) = &pumpportal_result.signature {
        println!("   PumpPortal: https://solscan.io/tx/{}", sig);
    }

    println!("\n💡 KEY CONSIDERATIONS:");
    println!("   • Jupiter uses YOUR wallet (you control keys)");
    println!("   • PumpPortal uses THEIR wallet (they control keys)");
    println!("   • Both appear to have no API subscription fees");
    println!("   • Network fees may vary based on priority and Jito");
    println!("   • Price execution matters more than small fee differences");
}
