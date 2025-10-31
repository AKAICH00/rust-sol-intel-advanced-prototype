use anyhow::Result;
use dotenv::dotenv;
use futures_util::{SinkExt, StreamExt};
use log::{info, warn, error};
use pump_portal_sdk::{PumpPortalClient, TradeRequest};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[derive(Debug, Clone)]
struct Position {
    mint: String,
    entry_sol: f64,
    entry_signature: String,
    entry_time: std::time::Instant,
}

#[derive(Debug, Deserialize)]
struct TokenCreatedEvent {
    signature: Option<String>,
    mint: Option<String>,
    #[serde(rename = "traderPublicKey")]
    trader_public_key: Option<String>,
    #[serde(rename = "txType")]
    tx_type: Option<String>,
    #[serde(rename = "initialBuy")]
    initial_buy: Option<f64>,
}

type Positions = Arc<Mutex<HashMap<String, Position>>>;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    dotenv().ok();

    info!("🚀 SIMPLE SNIPER BOT - LFG!");

    let api_key = env::var("PUMPPORTAL_API_KEY").expect("PUMPPORTAL_API_KEY required");
    let snipe_amount = env::var("SNIPE_AMOUNT_SOL")
        .unwrap_or_else(|_| "0.05".to_string())
        .parse::<f64>()?;

    let max_positions = env::var("MAX_POSITIONS")
        .unwrap_or_else(|_| "3".to_string())
        .parse::<usize>()?;

    info!("💰 Config:");
    info!("   Snipe Amount: {} SOL per trade", snipe_amount);
    info!("   Max Positions: {}", max_positions);
    info!("   Strategy: Buy launches → 2x exit → Repeat");

    let client = Arc::new(PumpPortalClient::new(api_key));
    let positions: Positions = Arc::new(Mutex::new(HashMap::new()));

    // Start position monitor
    let monitor_client = client.clone();
    let monitor_positions = positions.clone();
    tokio::spawn(async move {
        monitor_positions_loop(monitor_client, monitor_positions).await;
    });

    // Connect to PumpPortal WebSocket
    info!("📡 Connecting to PumpPortal WebSocket...");
    let ws_url = "wss://pumpportal.fun/api/data";
    let (ws_stream, _) = connect_async(ws_url).await?;
    let (mut write, mut read) = ws_stream.split();

    // Subscribe to new token launches
    let subscribe_msg = serde_json::json!({
        "method": "subscribeNewToken"
    });
    write.send(Message::Text(subscribe_msg.to_string())).await?;
    info!("✅ Subscribed to new token launches\n");
    info!("🎯 WATCHING FOR LAUNCHES... Press Ctrl+C to stop\n");

    // Process launch events
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(event) = serde_json::from_str::<TokenCreatedEvent>(&text) {
                    // Check if we have room for another position
                    let current_positions = positions.lock().await.len();
                    if current_positions >= max_positions {
                        warn!("⏸️  Max positions ({}) reached, skipping launch", max_positions);
                        continue;
                    }

                    if let Some(mint) = event.mint {
                        info!("🔔 NEW LAUNCH DETECTED!");
                        info!("   Mint: {}", mint);

                        // Execute buy
                        match execute_buy(&client, &mint, snipe_amount).await {
                            Ok(signature) => {
                                info!("✅ BUY EXECUTED!");
                                info!("   Signature: {}", signature);
                                info!("   Amount: {} SOL", snipe_amount);

                                // Store position
                                let position = Position {
                                    mint: mint.clone(),
                                    entry_sol: snipe_amount,
                                    entry_signature: signature,
                                    entry_time: std::time::Instant::now(),
                                };
                                positions.lock().await.insert(mint, position);

                                let remaining = max_positions - current_positions - 1;
                                info!("💼 Positions: {}/{} ({}left)", current_positions + 1, max_positions, remaining);
                            }
                            Err(e) => {
                                error!("❌ Buy failed: {}", e);
                            }
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => {
                warn!("WebSocket closed, reconnecting...");
                break;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
            }
            _ => {}
        }
    }

    info!("Bot stopped");
    Ok(())
}

async fn execute_buy(client: &PumpPortalClient, mint: &str, amount_sol: f64) -> Result<String> {
    let request = TradeRequest::buy(
        mint.to_string(),
        amount_sol,
        10, // 10% slippage
        0.0001, // priority fee
    ).with_jito_only(true); // Lightning fast Jito routing

    let response = client.trade(request).await?;
    Ok(response.signature.unwrap_or_else(|| "unknown".to_string()))
}

async fn execute_sell(client: &PumpPortalClient, mint: &str, _amount_sol: f64) -> Result<String> {
    // Sell 100% of tokens
    let request = TradeRequest::sell(
        mint.to_string(),
        "100%".to_string(), // Sell everything
        20, // 20% slippage for fast exit
        0.0001, // priority fee
    ).with_jito_only(true);

    let response = client.trade(request).await?;
    Ok(response.signature.unwrap_or_else(|| "unknown".to_string()))
}

async fn monitor_positions_loop(client: Arc<PumpPortalClient>, positions: Positions) {
    info!("👀 Position monitor started\n");

    loop {
        sleep(Duration::from_secs(10)).await;

        let positions_snapshot: Vec<Position> = {
            let locked = positions.lock().await;
            locked.values().cloned().collect()
        };

        if positions_snapshot.is_empty() {
            continue;
        }

        info!("📊 Checking {} positions...", positions_snapshot.len());

        for position in positions_snapshot {
            let elapsed = position.entry_time.elapsed().as_secs();

            info!("   {} - {}s elapsed", &position.mint[..8], elapsed);

            // Simple exit strategy:
            // 1. If > 60s old, exit (assume no momentum)
            // 2. Manual 2x checking would require price tracking

            if elapsed > 60 {
                info!("   ⏰ Position aged > 60s, exiting...");

                match execute_sell(&client, &position.mint, position.entry_sol).await {
                    Ok(sig) => {
                        info!("   ✅ SOLD: {}", sig);
                        positions.lock().await.remove(&position.mint);
                    }
                    Err(e) => {
                        error!("   ❌ Sell failed: {}", e);
                    }
                }
            }
        }
    }
}
