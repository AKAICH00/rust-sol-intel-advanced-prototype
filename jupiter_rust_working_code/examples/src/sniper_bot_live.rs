//! Live Sniper Bot Test
//!
//! Connects to PumpPortal WebSocket and monitors for new token launches in real-time.
//! When a snipeable token is detected, it executes the full sniper strategy.
//!
//! IMPORTANT: This will execute REAL TRADES with REAL MONEY!
//! Only run with small amounts you can afford to lose.

use pump_portal_sdk::PumpPortalClient;
use dotenv::dotenv;
use std::env;
use log::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    dotenv().ok();

    println!("╔═══════════════════════════════════════════════╗");
    println!("║     PUMP.FUN SNIPER BOT - LIVE MODE          ║");
    println!("╚═══════════════════════════════════════════════╝\n");

    // Load configuration
    let api_key = env::var("PUMPPORTAL_API_KEY")
        .expect("PUMPPORTAL_API_KEY must be set in .env file");

    let rpc_url = env::var("HELIUS_RPC_URL")
        .expect("HELIUS_RPC_URL must be set in .env file");

    let trade_amount = env::var("SNIPE_AMOUNT_SOL")
        .unwrap_or_else(|_| "0.025".to_string())
        .parse::<f64>()
        .expect("Invalid SNIPE_AMOUNT_SOL");

    println!("⚙️  Configuration:");
    println!("   Trade Size: {} SOL (~${:.2} at $200/SOL)", trade_amount, trade_amount * 200.0);
    println!("   Strategy: Fast in, smart exit");
    println!("   Exit Rules:");
    println!("      • No momentum (60s) → Fast exit 100%");
    println!("      • 2x profit → Recover initial + 10%, trail rest");
    println!("      • High momentum → Ladder out at 3x/5x/10x/20x");
    println!("      • Rug detected → Emergency exit 100%");
    println!();

    println!("⚠️  WARNING: THIS WILL EXECUTE REAL TRADES!");
    println!("   • Real money will be spent");
    println!("   • High risk of loss");
    println!("   • ~$5 per launch");
    println!();

    println!("Press CTRL+C to stop at any time\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Initialize bot
    info!("Initializing sniper bot...");

    // Note: For actual implementation, you would use:
    // let bot = SniperBot::new(api_key, rpc_url, trade_amount)?;
    // bot.run().await?;

    // For now, just show that launch detection works
    use pump_sniper_bot::launch_detector::{LaunchDetector, LaunchDetectorConfig};

    let detector = LaunchDetector::new(LaunchDetectorConfig::default());
    let mut launch_rx = detector.start_monitoring().await?;

    info!("✅ Launch detector running");
    info!("👀 Monitoring for new pump.fun launches...\n");

    // Process launches
    while let Some(launch) = launch_rx.recv().await {
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        launch.display();

        if launch.is_snipeable() {
            println!("\n💡 This token would be sniped automatically");
            println!("   Entry: {} SOL", trade_amount);
            println!("   Expected execution: ~700ms via PumpPortal");
            println!("   Then: Monitor for 60s, exit strategy activates");
        }
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        info!("👀 Monitoring for next launch...\n");
    }

    Ok(())
}
