use dotenv::dotenv;
use std::env;
use std::time::Instant;
use serde::{Deserialize, Serialize};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use bincode::deserialize;
use solana_sdk::transaction::VersionedTransaction;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_client::rpc_client::RpcClient;
use bs58;
use bincode::serialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct QuoteResponse {
    in_amount: String,
    out_amount: String,
    price_impact_pct: String,
    route_plan: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SwapRequest {
    user_public_key: String,
    quote_response: serde_json::Value,
    wrap_and_unwrap_sol: bool,
    compute_unit_price_micro_lamports: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SwapResponse {
    swap_transaction: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExecuteRequest {
    signed_transaction: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExecuteResponse {
    status: String,
    signature: Option<String>,
}

pub async fn v6_swap() {
    dotenv().ok();

    let start_time = Instant::now();

    println!("🔄 Starting Jupiter V6 Swap API test...\n");

    // Get keypair
    let key = env::var("PRIVATE_KEY").expect("PRIVATE_KEY not set");
    let key_bytes = bs58::decode(&key).into_vec().expect("Failed to decode");
    let keypair = Keypair::from_bytes(&key_bytes).expect("Failed to create keypair");
    let wallet_address = keypair.pubkey().to_string();

    println!("💼 Wallet: {}\n", wallet_address);
    println!("📊 Swap Details:");
    println!("   From: SOL");
    println!("   To: USDC");
    println!("   Amount: 0.001 SOL (~$0.20)\n");

    let client = reqwest::Client::new();

    // Step 1: Get Quote
    println!("⏳ Step 1: Getting quote from Jupiter V6...");
    let quote_start = Instant::now();

    let quote_url = format!(
        "https://lite-api.jup.ag/swap/v1/quote?inputMint={}&outputMint={}&amount={}&slippageBps=50",
        "So11111111111111111111111111111111111111112", // SOL
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
        1_000_000 // 0.001 SOL
    );

    let quote_response = client.get(&quote_url)
        .send()
        .await
        .expect("Failed to get quote");

    let quote_duration = quote_start.elapsed();

    if !quote_response.status().is_success() {
        let error_text = quote_response.text().await.unwrap();
        panic!("Quote failed: {}", error_text);
    }

    let quote_json: serde_json::Value = quote_response.json().await.expect("Failed to parse quote");
    let quote: QuoteResponse = serde_json::from_value(quote_json.clone()).expect("Failed to deserialize quote");

    println!("✅ Quote received! ({}ms)", quote_duration.as_millis());
    println!("   Expected output: {} USDC micro-units", quote.out_amount);
    println!("   Price impact: {}%\n", quote.price_impact_pct);

    // Step 2: Get Swap Transaction
    println!("⏳ Step 2: Getting swap transaction...");
    let swap_start = Instant::now();

    let swap_request = SwapRequest {
        user_public_key: wallet_address.clone(),
        quote_response: quote_json,
        wrap_and_unwrap_sol: true,
        compute_unit_price_micro_lamports: Some(200000),
    };

    let swap_response = client.post("https://lite-api.jup.ag/swap/v1/swap")
        .json(&swap_request)
        .send()
        .await
        .expect("Failed to get swap transaction");

    let swap_duration = swap_start.elapsed();

    if !swap_response.status().is_success() {
        let error_text = swap_response.text().await.unwrap();
        panic!("Swap transaction failed: {}", error_text);
    }

    let swap: SwapResponse = swap_response.json().await.expect("Failed to parse swap");

    println!("✅ Swap transaction received! ({}ms)\n", swap_duration.as_millis());

    // Step 3: Sign Transaction
    println!("🔏 Step 3: Signing transaction...");
    let sign_start = Instant::now();

    let swap_tx_bytes = STANDARD.decode(&swap.swap_transaction).expect("Failed to decode");
    let mut tx: VersionedTransaction = deserialize(&swap_tx_bytes).expect("Failed to deserialize");
    let message = tx.message.serialize();
    let signature = keypair.sign_message(&message);

    if tx.signatures.is_empty() {
        tx.signatures.push(signature);
    } else {
        tx.signatures[0] = signature;
    }

    let signed_tx_bytes = serialize(&tx).expect("Failed to serialize");
    let signed_tx_b64 = STANDARD.encode(&signed_tx_bytes);
    let sign_duration = sign_start.elapsed();

    println!("✅ Transaction signed! ({}ms)\n", sign_duration.as_millis());

    // Step 4: Send to Solana (using Helius RPC)
    println!("📤 Step 4: Sending transaction to Solana...");
    let send_start = Instant::now();

    let helius_url = env::var("HELIUS_RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());

    let rpc_client = RpcClient::new_with_commitment(
        helius_url.clone(),
        CommitmentConfig::confirmed()
    );

    match rpc_client.send_and_confirm_transaction(&tx) {
        Ok(signature) => {
            let send_duration = send_start.elapsed();
            let total_duration = start_time.elapsed();

            println!("✅ Transaction confirmed! ({}ms)\n", send_duration.as_millis());

            println!("✅ SWAP SUCCESSFUL! 🎉");
            println!("Transaction: {}\n", signature);

            println!("⏱️  Performance Metrics (V6 Swap API + Helius):");
            println!("   Quote:        {}ms", quote_duration.as_millis());
            println!("   Get Swap TX:  {}ms", swap_duration.as_millis());
            println!("   Signing:      {}ms", sign_duration.as_millis());
            println!("   Send & Confirm: {}ms", send_duration.as_millis());
            println!("   ─────────────────────────────");
            println!("   Total:        {}ms", total_duration.as_millis());

            println!("\n🔗 View on Solana Explorer:");
            println!("   https://solscan.io/tx/{}", signature);
            println!("   https://explorer.solana.com/tx/{}", signature);
        },
        Err(e) => {
            println!("❌ Transaction failed: {:?}", e);
        }
    }
}
