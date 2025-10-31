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
}

pub async fn working_swap() {
    let start_time = Instant::now();
    println!("🔄 Starting WORKING tiny swap...\n");

    // Load wallet
    dotenv().ok();
    let key = env::var("PRIVATE_KEY").expect("PRIVATE_KEY not set");
    let key_bytes = bs58::decode(&key).into_vec().expect("Failed to decode");
    let keypair = Keypair::from_bytes(&key_bytes).expect("Failed to create Keypair");
    let wallet_address = keypair.pubkey().to_string();

    println!("💼 Wallet: {}", wallet_address);
    println!("   Balance: 0.128 SOL\n");

    let client = reqwest::Client::new();
    let amount = 1_000_000; // 0.001 SOL

    println!("📊 Swap Details:");
    println!("   From: SOL");
    println!("   To: USDC");
    println!("   Amount: 0.001 SOL (~$0.20)\n");

    // Step 1: Get quote
    println!("⏳ Fetching quote from Jupiter...");
    let quote_start = Instant::now();
    let url = format!(
        "https://lite-api.jup.ag/ultra/v1/order?inputMint={}&outputMint={}&amount={}&taker={}",
        "So11111111111111111111111111111111111111112",
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        amount,
        wallet_address
    );

    let quote: QuoteResponse = match client.get(&url).send().await {
        Ok(resp) => {
            if !resp.status().is_success() {
                let body = resp.text().await.unwrap_or_default();
                println!("❌ Quote failed: {}", body);
                return;
            }
            match resp.json().await {
                Ok(q) => {
                    let quote_duration = quote_start.elapsed();
                    println!("✅ Quote received! ({}ms)", quote_duration.as_millis());
                    q
                }
                Err(e) => {
                    println!("❌ Failed to parse quote: {}", e);
                    return;
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to fetch quote: {}", e);
            return;
        }
    };

    println!("   Expected output: {} USDC micro-units", quote.out_amount);

    // Step 2: Sign transaction
    println!("\n🔏 Signing transaction...");
    let sign_start = Instant::now();
    let signed_tx = sign_transaction(quote.transaction);
    let sign_duration = sign_start.elapsed();
    println!("✅ Transaction signed! ({}ms)", sign_duration.as_millis());

    // Step 3: Execute swap
    println!("\n📤 Sending to Jupiter for execution...");
    let execute_start = Instant::now();
    let execute_req = ExecuteRequest {
        signed_transaction: signed_tx,
        request_id: quote.request_id,
    };

    let execute_url = "https://lite-api.jup.ag/ultra/v1/execute";
    match client.post(execute_url).json(&execute_req).send().await {
        Ok(resp) => {
            let status_code = resp.status();
            match resp.json::<ExecuteResponse>().await {
                Ok(result) => {
                    let execute_duration = execute_start.elapsed();
                    if let Some(sig) = result.signature {
                        let total_duration = start_time.elapsed();
                        println!("\n✅ SWAP SUCCESSFUL! 🎉");
                        println!("Transaction: {}", sig);
                        println!("\n⏱️  Performance Metrics:");
                        println!("   Quote:     {}ms", quote_start.elapsed().as_millis());
                        println!("   Signing:   {}ms", sign_duration.as_millis());
                        println!("   Execution: {}ms", execute_duration.as_millis());
                        println!("   Total:     {}ms", total_duration.as_millis());
                        println!("\n🔗 View on Solana Explorer:");
                        println!("   https://solscan.io/tx/{}", sig);
                        println!("   https://explorer.solana.com/tx/{}", sig);
                    } else if let Some(err) = result.error {
                        println!("\n❌ Swap failed: {}", err);
                        println!("Status: {}", result.status);
                    } else {
                        println!("\n⚠️  Unknown result: {:?}", result);
                    }
                }
                Err(e) => {
                    println!("\n❌ Failed to parse response: {}", e);
                    println!("HTTP Status: {}", status_code);
                }
            }
        }
        Err(e) => {
            println!("\n❌ Failed to execute: {}", e);
        }
    }
}
