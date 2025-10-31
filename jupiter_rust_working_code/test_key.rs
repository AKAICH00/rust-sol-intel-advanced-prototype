use dotenv::dotenv;
use std::env;

fn main() {
    dotenv().ok();

    let key = env::var("PRIVATE_KEY").expect("PRIVATE_KEY not set");
    println!("Private key loaded: {} chars", key.len());

    // Try to decode from base58
    match bs58::decode(&key).into_vec() {
        Ok(bytes) => {
            println!("✅ Valid base58 format");
            println!("Key length: {} bytes", bytes.len());

            // Check if it's valid Solana keypair length (64 bytes)
            if bytes.len() == 64 {
                println!("✅ Valid Solana keypair length (64 bytes)");

                // Try to create a keypair
                match solana_sdk::signature::Keypair::from_bytes(&bytes) {
                    Ok(keypair) => {
                        println!("✅ Valid Solana keypair!");
                        println!("Public key: {}", keypair.pubkey());
                    }
                    Err(e) => {
                        println!("❌ Invalid keypair: {}", e);
                    }
                }
            } else {
                println!("⚠️  Unexpected length. Solana keys are 64 bytes, got {} bytes", bytes.len());
            }
        }
        Err(e) => {
            println!("❌ Invalid base58 format: {}", e);
        }
    }
}
