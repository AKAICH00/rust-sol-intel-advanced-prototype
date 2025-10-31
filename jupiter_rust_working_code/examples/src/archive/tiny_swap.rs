use jup::sign_transaction;
use jup_ag_sdk::{
    JupiterClient,
    types::{UltraExecuteOrderRequest, UltraOrderRequest},
};
use dotenv::dotenv;
use std::env;
use solana_sdk::signature::Signer;
use solana_sdk::signature::Keypair;

pub async fn tiny_swap() {
    println!("🔄 Starting tiny test swap...\n");

    // Load wallet
    dotenv().ok();
    let key = env::var("PRIVATE_KEY").expect("PRIVATE_KEY not set");
    let key_bytes = bs58::decode(&key).into_vec().expect("Failed to decode");
    let keypair = Keypair::from_bytes(&key_bytes).expect("Failed to create Keypair");
    let wallet_address = keypair.pubkey().to_string();

    println!("💼 Wallet: {}", wallet_address);

    // Initialize Jupiter client
    let client = JupiterClient::new("https://lite-api.jup.ag");

    // Tiny swap: 0.001 SOL to USDC (minimum for Jupiter Ultra)
    let input_token = "So11111111111111111111111111111111111111112"; // SOL (native)
    let output_token = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // USDC
    let amount = 1_000_000; // 0.001 SOL (SOL has 9 decimals: 0.001 * 1_000_000_000 = 1_000_000)

    println!("📊 Swap Details:");
    println!("   From: SOL (native)");
    println!("   To: USDC");
    println!("   Amount: 0.001 SOL (1,000,000 lamports) - about $0.20");
    println!("   Taker: {}\n", wallet_address);

    // Create order request
    let order_request = UltraOrderRequest::new(input_token, output_token, amount)
        .add_taker(&wallet_address);

    println!("⏳ Fetching quote from Jupiter...");

    // Fetch the unsigned transaction
    let order_response = match client.get_ultra_order(&order_request).await {
        Ok(response) => {
            println!("✅ Quote received!");
            response
        }
        Err(e) => {
            println!("❌ Failed to get quote: {}", e);
            return;
        }
    };

    let unsigned_tx_base64 = match order_response.transaction {
        Some(tx) => tx,
        None => {
            println!("❌ No transaction in response");
            return;
        }
    };

    println!("🔏 Signing transaction...");
    let signed_tx_base64 = sign_transaction(unsigned_tx_base64);
    println!("✅ Transaction signed!");

    // Execute the swap
    let execute_request = UltraExecuteOrderRequest {
        signed_transaction: signed_tx_base64,
        request_id: order_response.request_id,
    };

    println!("📤 Sending to Jupiter for execution...");

    match client.ultra_execute_order(&execute_request).await {
        Ok(execute_response) => {
            if let Some(signature) = execute_response.signature {
                println!("\n✅ SWAP SUCCESSFUL! 🎉");
                println!("Transaction: {}", signature);
                println!("\n🔗 View on Solana Explorer:");
                println!("   https://solscan.io/tx/{}", signature);
                println!("   https://explorer.solana.com/tx/{}", signature);
            } else {
                println!("⚠️  Transaction submitted but no signature returned");
            }
        }
        Err(e) => {
            println!("❌ Failed to execute swap: {:?}", e);
            println!("\nPossible reasons:");
            println!("   - Insufficient SOL balance for swap + fees (~0.001 SOL needed)");
            println!("   - Amount too small (try 0.001 SOL or larger)");
            println!("   - Network congestion");
            println!("   - Slippage tolerance exceeded");
            println!("\n💡 Try:");
            println!("   1. Check wallet has enough SOL for swap + transaction fees");
            println!("   2. Increase swap amount to 0.001 SOL (1,000,000 lamports)");
            println!("   3. Use regular Swap API instead of Ultra for tiny amounts");
        }
    }
}
