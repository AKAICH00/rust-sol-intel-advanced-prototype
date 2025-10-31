use jup::sign_transaction;
use dotenv::dotenv;
use std::env;
use solana_sdk::signature::Signer;
use solana_sdk::signature::Keypair;

pub async fn debug_swap() {
    println!("🔍 Debug Swap - Checking Jupiter API responses...\n");

    // Load wallet
    dotenv().ok();
    let key = env::var("PRIVATE_KEY").expect("PRIVATE_KEY not set");
    let key_bytes = bs58::decode(&key).into_vec().expect("Failed to decode");
    let keypair = Keypair::from_bytes(&key_bytes).expect("Failed to create Keypair");
    let wallet_address = keypair.pubkey().to_string();

    println!("💼 Wallet: {}", wallet_address);
    println!("   Balance: ~0.009387 SOL\n");

    // Test 1: Try to get a quote
    let client = reqwest::Client::new();
    let amount = 100_000; // 0.0001 SOL

    let url = format!(
        "https://lite-api.jup.ag/v1/ultra/order?inputMint={}&outputMint={}&amount={}&taker={}",
        "So11111111111111111111111111111111111111112", // SOL
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
        amount,
        wallet_address
    );

    println!("📡 Requesting quote from:");
    println!("   {}\n", url);

    match client.get(&url).send().await {
        Ok(response) => {
            let status = response.status();
            println!("✅ Response status: {}", status);

            let body = response.text().await.unwrap_or_else(|_| "Could not read body".to_string());
            println!("📄 Response body:");
            println!("{}\n", body);

            if status.is_success() {
                println!("✅ Quote API is working! The issue might be with execution.");
            } else {
                println!("❌ Quote API returned error. Possible reasons:");
                println!("   - Amount too small (try 0.001 SOL or larger)");
                println!("   - Invalid token addresses");
                println!("   - API temporarily unavailable");
            }
        }
        Err(e) => {
            println!("❌ Failed to connect to Jupiter API: {}", e);
        }
    }

    // Test 2: Check if we should use a different endpoint
    println!("\n💡 Suggestion:");
    println!("   Jupiter Ultra API may have minimum amounts.");
    println!("   Try increasing to 0.001 SOL (1,000,000 lamports) or higher.");
    println!("   Or use the regular Swap API instead of Ultra for tiny amounts.");
}
