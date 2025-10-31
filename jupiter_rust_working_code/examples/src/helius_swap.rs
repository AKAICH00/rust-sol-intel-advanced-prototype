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

pub async fn helius_swap() {
    let start_time = Instant::now();
    println!("🔄 Starting Helius RPC swap...\n");

    // Load config
    dotenv().ok();
    let key = env::var("PRIVATE_KEY").expect("PRIVATE_KEY not set");
    let helius_url = env::var("HELIUS_RPC_URL").unwrap_or_else(|_| {
        println!("⚠️  HELIUS_RPC_URL not set in .env, using default");
        "https://api.mainnet-beta.solana.com".to_string()
    });

    println!("🌐 RPC Endpoint: {}",
        if helius_url.contains("helius") { "Helius (premium)" }
        else { "Default (public)" }
    );

    let key_bytes = bs58::decode(&key).into_vec().expect("Failed to decode");
    let keypair = Keypair::from_bytes(&key_bytes).expect("Failed to create Keypair");
    let wallet_address = keypair.pubkey().to_string();

    println!("💼 Wallet: {}\n", wallet_address);

    // Initialize Helius RPC client
    let rpc_init_start = Instant::now();
    let rpc_client = RpcClient::new_with_commitment(
        helius_url.clone(),
        CommitmentConfig::confirmed()
    );
    let rpc_init_duration = rpc_init_start.elapsed();
    println!("✅ RPC client initialized ({}ms)\n", rpc_init_duration.as_millis());

    let http_client = reqwest::Client::new();
    let amount = 1_000_000; // 0.001 SOL

    println!("📊 Swap Details:");
    println!("   From: SOL");
    println!("   To: USDC");
    println!("   Amount: 0.001 SOL (~$0.20)\n");

    // Step 1: Get quote from Jupiter
    println!("⏳ Fetching quote from Jupiter...");
    let quote_start = Instant::now();
    let url = format!(
        "https://lite-api.jup.ag/ultra/v1/order?inputMint={}&outputMint={}&amount={}&taker={}",
        "So11111111111111111111111111111111111111112",
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        amount,
        wallet_address
    );

    let quote: QuoteResponse = match http_client.get(&url).send().await {
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

    // Step 3: Execute via Jupiter Ultra API (they handle RPC submission)
    println!("\n📤 Sending to Jupiter for execution (via Helius)...");
    let execute_start = Instant::now();
    let execute_req = ExecuteRequest {
        signed_transaction: signed_tx,
        request_id: quote.request_id,
    };

    let execute_url = "https://lite-api.jup.ag/ultra/v1/execute";
    match http_client.post(execute_url).json(&execute_req).send().await {
        Ok(resp) => {
            let status_code = resp.status();
            match resp.json::<ExecuteResponse>().await {
                Ok(result) => {
                    let execute_duration = execute_start.elapsed();
                    if let Some(sig) = result.signature {
                        let total_duration = start_time.elapsed();

                        // Get confirmation via Helius
                        println!("\n⏳ Confirming via Helius RPC...");
                        let confirm_start = Instant::now();
                        match rpc_client.confirm_transaction(&sig.parse().unwrap()) {
                            Ok(_) => {
                                let confirm_duration = confirm_start.elapsed();
                                println!("✅ Confirmed! ({}ms)", confirm_duration.as_millis());

                                println!("\n✅ SWAP SUCCESSFUL! 🎉");
                                println!("Transaction: {}", sig);
                                println!("\n⏱️  Performance Metrics (Helius RPC):");
                                println!("   RPC Init:     {}ms", rpc_init_duration.as_millis());
                                println!("   Quote:        {}ms", quote_start.elapsed().as_millis());
                                println!("   Signing:      {}ms", sign_duration.as_millis());
                                println!("   Execution:    {}ms", execute_duration.as_millis());
                                println!("   Confirmation: {}ms", confirm_duration.as_millis());
                                println!("   ─────────────────────────────");
                                println!("   Total:        {}ms", total_duration.as_millis());
                                println!("\n🔗 View on Solana Explorer:");
                                println!("   https://solscan.io/tx/{}", sig);
                                println!("   https://explorer.solana.com/tx/{}", sig);
                            }
                            Err(e) => {
                                println!("⚠️  Confirmation check failed: {}", e);
                                println!("   (Transaction may still succeed, check explorer)");
                            }
                        }
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
