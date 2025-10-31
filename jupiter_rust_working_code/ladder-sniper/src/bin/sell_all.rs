use anyhow::Result;
use dotenv::dotenv;
use log::info;
use pump_portal_sdk::{PumpPortalClient, TradeRequest};
use std::env;

// List of mints from the last bot run
const MINTS_TO_SELL: &[&str] = &[
    "DqXMpdkSxq7uxFCpTVkWNgNuz96xSZLuWEn3yY8spump",  // negative67
    "H7SUNxQ68u2nQ1JXRm5s5Q7BzvxgKFuJgzWnBznCpump",  // Proton
    "2eJZFR47Wib47SEarbBxZSdtXApanCRKrXxPfYfgpump",  // Amazon Robot
    "HXCZtPAzPqHBwzJgpg5ArUWU3JnHQHbcuAotkPespump",  // Boxiumus
    "EKPteuctVqxmDm9MXoh2tVyXbfP6JuKi5KmBSrAVpump",  // K.I.T.
    "5ADHoSssWeSzo6daKGxY8JWu3oL44j2iunvA71sJpump",  // 1st402.fun
    "9K3XSk9U19iHvQShZYJ7KqAARELWttELBGfqUkTMpump",  // 3lixir
    "GBXDgRWfdZFomSqd8Zy8jLuwstzmVE7cJTMf4qHMpump",  // RIP Kanzi
    "7T1Ta1xsgiEqsVo1wry2Tr7sSfCzr1UuMTNKJZubpump",  // Lens402
    "2SDNfhr5L56Q5EPsofgV7Fms5uRxA8u9zBLAGdfApump",  // TITAN
    "Hokm69BwcRj2Tdbf3C9TEstjsYt1vso6FBTdyWenpump",  // EESEE
    "AA4TAqovYb2MftgCCUxpNX16xv76yNr8ymZUUMZepump",  // shifu
    "G2jYcuvycEvMgvLJm64dksjxFJJH9zQXm9iVo5Xopump",  // Cannoli
    "7aazFv1rkEEsFo3j6PYNzU37CFELuj8MF3aPMd8pump",  // The Brick Lady
];

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    dotenv().ok();

    info!("🔥 EMERGENCY SELL ALL POSITIONS");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let api_key = env::var("PUMPPORTAL_API_KEY").expect("PUMPPORTAL_API_KEY required");
    let client = PumpPortalClient::new(api_key);

    info!("📊 Positions to sell: {}", MINTS_TO_SELL.len());
    info!("");

    for (i, mint) in MINTS_TO_SELL.iter().enumerate() {
        info!("🔄 [{}/{}] Selling {}...", i + 1, MINTS_TO_SELL.len(), &mint[0..8]);

        match execute_sell(&client, mint, 100).await {
            Ok(sig) => {
                info!("   ✅ SOLD - Signature: {}", sig);
            }
            Err(e) => {
                info!("   ❌ FAILED: {}", e);
            }
        }

        // Small delay to avoid rate limits
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    info!("");
    info!("✅ SELL ALL COMPLETE!");

    Ok(())
}

async fn execute_sell(client: &PumpPortalClient, mint: &str, percent: u32) -> Result<String> {
    let amount = format!("{}%", percent);
    let request = TradeRequest::sell(
        mint.to_string(),
        amount,
        20,
        0.0001,
    ).with_jito_only(true);

    let response = client.trade(request).await?;
    Ok(response.signature.unwrap_or_else(|| "unknown".to_string()))
}
