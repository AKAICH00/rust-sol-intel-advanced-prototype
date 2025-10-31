use dotenv::dotenv;
use std::env;
use std::time::Instant;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::{Keypair, Signer};

pub async fn benchmark_rpcs() {
    dotenv().ok();

    let key = env::var("PRIVATE_KEY").expect("PRIVATE_KEY not set");
    let key_bytes = bs58::decode(&key).into_vec().expect("Failed to decode");
    let keypair = Keypair::from_bytes(&key_bytes).expect("Failed to create Keypair");
    let wallet = keypair.pubkey();

    println!("🏁 RPC Endpoint Benchmark\n");
    println!("Testing wallet: {}\n", wallet);
    println!("═══════════════════════════════════════════════════════════\n");

    // Collect RPC endpoints
    let mut rpcs = vec![];

    if let Ok(url) = env::var("DEFAULT_RPC_URL") {
        rpcs.push(("Default (Public)", url));
    }

    if let Ok(url) = env::var("HELIUS_RPC_URL") {
        if !url.contains("YOUR_") {
            rpcs.push(("Helius", url));
        }
    }

    if let Ok(url) = env::var("QUICKNODE_RPC_URL") {
        rpcs.push(("QuickNode", url));
    }

    if let Ok(url) = env::var("TRITON_RPC_URL") {
        rpcs.push(("Triton", url));
    }

    if rpcs.is_empty() {
        println!("❌ No RPC endpoints configured in .env");
        return;
    }

    println!("Testing {} endpoints...\n", rpcs.len());

    for (name, url) in rpcs {
        println!("📡 {}", name);
        println!("   URL: {}", url);

        let client = RpcClient::new_with_commitment(
            url,
            CommitmentConfig::confirmed()
        );

        // Test 1: Get slot
        let slot_start = Instant::now();
        match client.get_slot() {
            Ok(slot) => {
                let slot_time = slot_start.elapsed();
                println!("   ✅ get_slot: {}ms (slot: {})", slot_time.as_millis(), slot);
            }
            Err(e) => {
                println!("   ❌ get_slot failed: {}", e);
                continue;
            }
        }

        // Test 2: Get balance
        let balance_start = Instant::now();
        match client.get_balance(&wallet) {
            Ok(balance) => {
                let balance_time = balance_start.elapsed();
                println!("   ✅ get_balance: {}ms (balance: {} SOL)",
                    balance_time.as_millis(),
                    balance as f64 / 1_000_000_000.0
                );
            }
            Err(e) => {
                println!("   ❌ get_balance failed: {}", e);
            }
        }

        // Test 3: Get recent blockhash
        let blockhash_start = Instant::now();
        match client.get_latest_blockhash() {
            Ok(blockhash) => {
                let blockhash_time = blockhash_start.elapsed();
                println!("   ✅ get_blockhash: {}ms", blockhash_time.as_millis());
                println!("      Hash: {}", blockhash);
            }
            Err(e) => {
                println!("   ❌ get_blockhash failed: {}", e);
            }
        }

        println!();
    }

    println!("═══════════════════════════════════════════════════════════");
    println!("\n💡 Recommendations:");
    println!("   • Fastest get_slot: Best for general use");
    println!("   • Fastest get_balance: Best for frequent queries");
    println!("   • Fastest get_blockhash: Best for transaction submission");
}
