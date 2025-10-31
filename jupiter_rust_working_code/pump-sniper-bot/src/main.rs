mod strategy;
mod monitor;
mod detector;
mod launch_detector;
mod database;

use dotenv::dotenv;
use std::env;
use log::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    env_logger::init();
    dotenv().ok();

    info!("🚀 Pump.fun Sniper Bot Starting...");

    // Verify configuration
    let api_key = env::var("PUMPPORTAL_API_KEY")
        .expect("PUMPPORTAL_API_KEY must be set");

    let rpc_url = env::var("HELIUS_RPC_URL")
        .expect("HELIUS_RPC_URL must be set");

    let trade_amount_sol = env::var("SNIPE_AMOUNT_SOL")
        .unwrap_or_else(|_| "0.025".to_string()) // ~$5 at $200/SOL
        .parse::<f64>()
        .expect("Invalid SNIPE_AMOUNT_SOL");

    info!("📊 Configuration:");
    info!("   Trade Size: {} SOL (~${:.2})", trade_amount_sol, trade_amount_sol * 200.0);
    info!("   RPC: Helius Premium");
    info!("   Strategy: Fast in, smart exit");

    // Initialize database
    let db_path = env::var("DATABASE_PATH").unwrap_or_else(|_| "sniper_bot.db".to_string());
    let db = database::Database::new(&db_path)?;
    info!("   Database: {}", db_path);

    // Start the bot
    let bot = strategy::SniperBot::new(api_key, rpc_url, trade_amount_sol, db)?;

    info!("✅ Bot initialized successfully");
    info!("👀 Monitoring for new pump.fun launches...\n");

    bot.run().await
}
