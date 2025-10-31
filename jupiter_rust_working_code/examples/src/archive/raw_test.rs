use dotenv::dotenv;
use std::env;
use solana_sdk::signature::{Keypair, Signer};

pub async fn raw_test() {
    println!("🔍 Testing Jupiter API directly...\n");

    // Load wallet
    dotenv().ok();
    let key = env::var("PRIVATE_KEY").expect("PRIVATE_KEY not set");
    let key_bytes = bs58::decode(&key).into_vec().expect("Failed to decode");
    let keypair = Keypair::from_bytes(&key_bytes).expect("Failed to create Keypair");
    let wallet_address = keypair.pubkey().to_string();

    println!("💼 Wallet: {}", wallet_address);
    println!("   Balance: 0.128 SOL\n");

    let client = reqwest::Client::new();

    // Test 1: Try the exact endpoint from the SDK
    println!("📡 Test 1: Ultra API endpoint");
    let url = format!(
        "https://lite-api.jup.ag/ultra/v1/order?inputMint={}&outputMint={}&amount={}&taker={}",
        "So11111111111111111111111111111111111111112",
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        1_000_000,
        wallet_address
    );

    match client.get(&url).send().await {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            println!("   Status: {}", status);
            println!("   Response: {}\n", body);
        }
        Err(e) => println!("   Error: {}\n", e),
    }

    // Test 2: Try regular Quote API instead
    println!("📡 Test 2: Regular Quote API (v6)");
    let url2 = format!(
        "https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}&slippageBps=50",
        "So11111111111111111111111111111111111111112",
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        1_000_000
    );

    match client.get(&url2).send().await {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            println!("   Status: {}", status);

            if status.is_success() {
                println!("   ✅ Quote API works!");
                println!("   Response preview: {}...\n", &body[..body.len().min(200)]);
                println!("💡 Suggestion: Use regular Swap API instead of Ultra API");
            } else {
                println!("   Response: {}\n", body);
            }
        }
        Err(e) => println!("   Error: {}\n", e),
    }
}
