use dotenv::dotenv;
use std::env;
use solana_sdk::signature::Signer;

#[tokio::main]
async fn main() {
    println!("🔑 Testing private key...\n");

    // Load .env
    dotenv().ok();

    // Try to load the key
    match env::var("PRIVATE_KEY") {
        Ok(key) => {
            println!("✅ PRIVATE_KEY loaded from .env");
            println!("   Length: {} characters", key.len());
            println!("   Preview: {}...{}", &key[..10], &key[key.len()-10..]);

            // Try to decode it
            match bs58::decode(&key).into_vec() {
                Ok(bytes) => {
                    println!("\n✅ Valid base58 encoding");
                    println!("   Decoded to {} bytes", bytes.len());

                    if bytes.len() == 64 {
                        println!("\n✅ Correct length for Solana keypair (64 bytes)");

                        // Try to create keypair
                        match solana_sdk::signature::Keypair::from_bytes(&bytes) {
                            Ok(keypair) => {
                                println!("\n✅ Valid Solana keypair!");
                                println!("   Public key: {}", keypair.pubkey());
                                println!("\n🎉 Your private key is valid and ready to use!");
                            }
                            Err(e) => {
                                println!("\n❌ Could not create keypair: {}", e);
                            }
                        }
                    } else {
                        println!("\n⚠️  Warning: Expected 64 bytes for Solana keypair, got {} bytes", bytes.len());
                    }
                }
                Err(e) => {
                    println!("\n❌ Invalid base58 encoding: {}", e);
                    println!("   Make sure your key doesn't have any extra characters or spaces");
                }
            }
        }
        Err(_) => {
            println!("❌ PRIVATE_KEY not found in .env file");
            println!("   Make sure you have created examples/.env with your private key");
        }
    }
}
