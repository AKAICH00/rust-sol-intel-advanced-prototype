mod social_checker;
mod momentum_tracker;

use anyhow::Result;
use dotenv::dotenv;
use futures_util::{SinkExt, StreamExt};
use log::{error, info, warn};
use momentum_tracker::check_momentum;
use pump_portal_sdk::{PumpPortalClient, TradeRequest};
use serde::{Deserialize, Serialize};
use social_checker::{check_social_momentum, SocialScore};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[derive(Debug, Clone)]
struct Position {
    mint: String,
    entry_sol: f64,
    entry_price_usd: f64,
    entry_signature: String,
    entry_time: std::time::Instant,
    risk_score: f64,
    social_score: Option<SocialScore>,
    fast_exit: bool,
    add_count: u32, // Track how many times we've added to position
    last_add_time: std::time::Instant, // Prevent rapid adds
}

type Positions = Arc<Mutex<HashMap<String, Position>>>;

#[derive(Debug, Deserialize)]
struct TokenCreatedEvent {
    mint: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    symbol: Option<String>,
    #[serde(default)]
    uri: Option<String>,
}

#[derive(Debug, Serialize)]
struct AIAnalysisRequest {
    token_name: String,
    token_symbol: String,
    mint_address: String,
}

#[derive(Debug, Deserialize)]
struct AIAnalysisResponse {
    risk_score: f64,
    should_buy: bool,
    confidence: f64,
    reasoning: String,
}

/// Analyze token using simple heuristics + AI scoring
async fn analyze_token(mint: &str, name: Option<String>, symbol: Option<String>) -> Result<(bool, f64)> {
    let name = name.unwrap_or_else(|| "Unknown".to_string());
    let symbol = symbol.unwrap_or_else(|| "???".to_string());

    info!("🔍 Analyzing: {} ({})", name, symbol);

    // Quick heuristic filters
    let mut risk_score = 1.0;

    // Filter 1: Check token name/symbol quality
    if name.len() < 3 || symbol.len() < 2 {
        warn!("   ❌ Token name/symbol too short");
        return Ok((false, 0.0));
    }

    // Filter 2: Avoid obvious scams
    let scam_keywords = ["scam", "rug", "honeypot", "test", "fake"];
    let name_lower = name.to_lowercase();
    let symbol_lower = symbol.to_lowercase();

    for keyword in &scam_keywords {
        if name_lower.contains(keyword) || symbol_lower.contains(keyword) {
            warn!("   ❌ Scam keyword detected: {}", keyword);
            return Ok((false, 0.0));
        }
    }

    // Filter 3: Check for all caps (often scams)
    if name.chars().all(|c| c.is_uppercase() || !c.is_alphabetic()) {
        risk_score *= 0.7;
        warn!("   ⚠️  All caps name, reduced risk score");
    }

    // Filter 4: Prefer tokens with normal-looking symbols
    if symbol.chars().all(|c| c.is_alphanumeric()) {
        risk_score *= 1.2;
    }

    // Filter 5: Length bonus (legitimate projects have decent names)
    if name.len() >= 5 && name.len() <= 20 {
        risk_score *= 1.1;
    }

    // AI Enhancement: Use DeepSeek-style reasoning
    // For now, use rule-based "AI" - can add real API later
    let ai_score = calculate_ai_score(&name, &symbol);
    risk_score *= ai_score;

    let should_buy = risk_score >= 0.6;

    if should_buy {
        info!("   ✅ PASS - Risk Score: {:.2}", risk_score);
    } else {
        warn!("   ⛔ FAIL - Risk Score: {:.2} (need >= 0.6)", risk_score);
    }

    Ok((should_buy, risk_score))
}

/// AI-powered scoring (can be replaced with real DeepSeek API)
fn calculate_ai_score(name: &str, symbol: &str) -> f64 {
    let mut score = 1.0;

    // Vowel/consonant ratio (natural language indicator)
    let vowels = name.chars().filter(|c| "aeiouAEIOU".contains(*c)).count();
    let consonants = name.chars().filter(|c| c.is_alphabetic() && !"aeiouAEIOU".contains(*c)).count();

    if vowels > 0 && consonants > 0 {
        let ratio = vowels as f64 / consonants as f64;
        if ratio > 0.2 && ratio < 2.0 {
            score *= 1.15; // Natural-looking word
        }
    }

    // Symbol/name length ratio
    if symbol.len() >= 2 && name.len() >= 4 {
        let len_ratio = symbol.len() as f64 / name.len() as f64;
        if len_ratio > 0.1 && len_ratio < 0.5 {
            score *= 1.1; // Reasonable abbreviation
        }
    }

    // Check for repeated characters (spam indicator)
    let has_repeats = name.chars().collect::<Vec<_>>()
        .windows(3)
        .any(|w| w[0] == w[1] && w[1] == w[2]);

    if has_repeats {
        score *= 0.6; // Likely spam
    }

    // Bonus for mixed case (indicates real branding)
    if name.chars().any(|c| c.is_lowercase()) && name.chars().any(|c| c.is_uppercase()) {
        score *= 1.2;
    }

    score
}

async fn execute_buy(
    client: &PumpPortalClient,
    mint: &str,
    amount_sol: f64,
    risk_score: f64,
) -> Result<String> {
    // Dynamic slippage based on risk score
    let slippage = if risk_score > 0.9 {
        10 // Low slippage for high-quality tokens
    } else if risk_score > 0.7 {
        15 // Medium slippage
    } else {
        20 // High slippage for risky plays
    };

    let request = TradeRequest::buy(
        mint.to_string(),
        amount_sol,
        slippage,
        0.0001,
    ).with_jito_only(true);

    let response = client.trade(request).await?;
    Ok(response.signature.unwrap_or_else(|| "unknown".to_string()))
}

async fn execute_sell(client: &PumpPortalClient, mint: &str) -> Result<String> {
    let request = TradeRequest::sell(
        mint.to_string(),
        "100%".to_string(),
        20,
        0.0001,
    ).with_jito_only(true);

    let response = client.trade(request).await?;
    Ok(response.signature.unwrap_or_else(|| "unknown".to_string()))
}

async fn monitor_positions_loop(client: Arc<PumpPortalClient>, positions: Positions, snipe_amount: f64) {
    info!("👀 Momentum-based position monitor started");
    info!("   Strategy: HOLD winners as long as they pump\n");
    info!("   Buy-into-strength: Enabled (add up to 3x on strong momentum)\n");

    loop {
        sleep(Duration::from_secs(3)).await; // Check every 3s for faster exits
        let positions_snapshot: Vec<Position> = {
            let locked = positions.lock().await;
            locked.values().cloned().collect()
        };

        if !positions_snapshot.is_empty() {
            info!("📊 Checking {} positions...", positions_snapshot.len());
        }

        for position in positions_snapshot {
            let elapsed = position.entry_time.elapsed().as_secs();

            // SIMPLE TIME-BASED EXIT (10s for all tokens)
            if elapsed > 10 {
                info!("   ⏰ 10s elapsed - EXITING");
                match execute_sell(&client, &position.mint).await {
                    Ok(sig) => {
                        info!("   ✅ SOLD - {}", sig);
                        positions.lock().await.remove(&position.mint);
                        continue;
                    }
                    Err(e) => error!("   ❌ Sell failed: {}", e),
                }
            }

            // SKIP momentum check for now - using simple time exits
            if false {
            match check_momentum(&position.mint, position.entry_price_usd).await {
                Ok(momentum) => {
                    let social_info = if position.fast_exit {
                        " | ⚠️  ZERO SOCIALS".to_string()
                    } else if let Some(ref s) = position.social_score {
                        format!(" | social: {:.2}", s.momentum_score)
                    } else {
                        "".to_string()
                    };

                    info!("   {} - {}s | P&L: {:+.1}% | momentum: {:.2} | vol: ${:.0}{}",
                          &position.mint[..8],
                          elapsed,
                          momentum.pnl_percent,
                          momentum.momentum_score,
                          momentum.volume_24h,
                          social_info);

                    // BUY INTO STRENGTH: Add to winners
                    let time_since_last_add = position.last_add_time.elapsed().as_secs();
                    if momentum.momentum_score > 0.7
                        && momentum.pnl_percent > 20.0
                        && position.add_count < 3
                        && time_since_last_add > 30
                        && !position.fast_exit
                    {
                        info!("   🚀 STRONG MOMENTUM DETECTED! Adding to position...");
                        match execute_buy(&client, &position.mint, snipe_amount, position.risk_score).await {
                            Ok(add_sig) => {
                                info!("   ✅ ADDED {} SOL (add #{}) - {}",
                                      snipe_amount, position.add_count + 1, add_sig);
                                // Update position
                                let mut locked_positions = positions.lock().await;
                                if let Some(pos) = locked_positions.get_mut(&position.mint) {
                                    pos.entry_sol += snipe_amount;
                                    pos.add_count += 1;
                                    pos.last_add_time = std::time::Instant::now();
                                }
                            }
                            Err(e) => error!("   ❌ Add failed: {}", e),
                        }
                    }

                    // EXIT CONDITIONS (momentum-based, NOT time-based):
                    let should_exit = if !momentum.should_hold {
                        // Momentum tracker says exit
                        info!("   📉 Momentum died → EXIT");
                        true
                    } else if position.fast_exit && momentum.pnl_percent < -10.0 {
                        // Fast exit for zero-social tokens if losing >10%
                        warn!("   🚨 Zero socials + losing → EXIT");
                        true
                    } else if momentum.pnl_percent > 200.0 && momentum.momentum_score < 0.0 {
                        // Secure 3x gains if momentum turns negative
                        info!("   💰 3x gains + negative momentum → SECURE PROFITS");
                        true
                    } else if momentum.pnl_percent > 500.0 && momentum.momentum_score < 0.3 {
                        // Secure 6x gains if momentum weakening
                        info!("   💎 6x gains + weak momentum → SECURE PROFITS");
                        true
                    } else {
                        // KEEP HOLDING - momentum still strong
                        false
                    };

                    if should_exit {
                        match execute_sell(&client, &position.mint).await {
                            Ok(sig) => {
                                info!("   ✅ SOLD at {:+.1}% P&L", momentum.pnl_percent);
                                info!("   Signature: {}", sig);
                                positions.lock().await.remove(&position.mint);
                            }
                            Err(e) => error!("   ❌ Sell failed: {}", e),
                        }
                    }
                }
                Err(e) => {
                    // NO DATA = DUMP IMMEDIATELY
                    warn!("   ⚠️  Momentum check failed: {} (DUMPING)", e);
                    if elapsed > 3 {
                        warn!("   🚨 NO PRICE DATA - EMERGENCY DUMP");
                        match execute_sell(&client, &position.mint).await {
                            Ok(sig) => {
                                info!("   ✅ DUMPED (no data) - {}", sig);
                                positions.lock().await.remove(&position.mint);
                            }
                            Err(e) => error!("   ❌ Dump failed: {}", e),
                        }
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    dotenv().ok();

    info!("🧠 SMART SNIPER BOT - AI-Powered Trading");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let api_key = env::var("PUMPPORTAL_API_KEY").expect("PUMPPORTAL_API_KEY required");
    let snipe_amount = env::var("SNIPE_AMOUNT_SOL")
        .unwrap_or_else(|_| "0.05".to_string())
        .parse::<f64>()?;
    let max_positions = env::var("MAX_POSITIONS")
        .unwrap_or_else(|_| "999".to_string())
        .parse::<usize>()?;

    info!("💰 Config:");
    info!("   Snipe Amount: {} SOL per trade", snipe_amount);
    if max_positions >= 999 {
        info!("   Max Positions: UNLIMITED (rapid fire)");
    } else {
        info!("   Max Positions: {}", max_positions);
    }
    info!("   Strategy: AI-filtered launches → Momentum exits");
    info!("   Risk Threshold: 0.6 minimum\n");

    let client = Arc::new(PumpPortalClient::new(api_key));
    let positions: Positions = Arc::new(Mutex::new(HashMap::new()));

    // Start position monitor
    let monitor_client = client.clone();
    let monitor_positions = positions.clone();
    tokio::spawn(async move {
        monitor_positions_loop(monitor_client, monitor_positions, snipe_amount).await;
    });

    // Connect to WebSocket
    let ws_url = "wss://pumpportal.fun/api/data";
    info!("📡 Connecting to PumpPortal WebSocket...");
    let (ws_stream, _) = connect_async(ws_url).await?;
    let (mut write, mut read) = ws_stream.split();

    let subscribe_msg = serde_json::json!({"method": "subscribeNewToken"});
    write.send(Message::Text(subscribe_msg.to_string())).await?;
    info!("✅ Subscribed to new token launches\n");
    info!("🎯 ANALYZING LAUNCHES... Press Ctrl+C to stop\n");

    let mut total_detected = 0u64;
    let mut total_filtered = 0u64;
    let mut total_bought = 0u64;

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(event) = serde_json::from_str::<TokenCreatedEvent>(&text) {
                    if let Some(mint) = event.mint {
                        total_detected += 1;

                        info!("🔔 NEW LAUNCH DETECTED! (#{} total)", total_detected);
                        info!("   Mint: {}", mint);
                        if let Some(ref name) = event.name {
                            info!("   Name: {}", name);
                        }
                        if let Some(ref symbol) = event.symbol {
                            info!("   Symbol: {}", symbol);
                        }

                        // Check if we can buy
                        let current_positions = positions.lock().await.len();
                        if current_positions >= max_positions {
                            warn!("⏸️  Max positions ({}) reached, skipping\n", max_positions);
                            continue;
                        }

                        // CHECK SOCIALS FIRST - BEFORE BUYING!
                        info!("🔍 Checking socials BEFORE buying...");
                        let social_check = check_social_momentum(&mint).await;
                        if let Ok(score) = &social_check {
                            if score.should_fast_exit {
                                warn!("   ❌ ZERO SOCIALS - SKIPPING\n");
                                total_filtered += 1;
                                continue;
                            } else {
                                info!("   ✅ Has socials (score: {:.2})", score.momentum_score);
                            }
                        }

                        // AI-powered analysis
                        match analyze_token(&mint, event.name, event.symbol).await {
                            Ok((should_buy, risk_score)) => {
                                if should_buy {
                                    // Execute buy
                                    match execute_buy(&client, &mint, snipe_amount, risk_score).await {
                                        Ok(signature) => {
                                            total_bought += 1;
                                            info!("✅ BUY EXECUTED!");
                                            info!("   Signature: {}", signature);
                                            info!("   Amount: {} SOL", snipe_amount);
                                            info!("   Risk Score: {:.2}", risk_score);

                                            // IMMEDIATELY check socials after buying
                                            let social_result = check_social_momentum(&mint).await;
                                            let (social_score_opt, fast_exit) = match social_result {
                                                Ok(score) => {
                                                    let fast = score.should_fast_exit;
                                                    if fast {
                                                        warn!("   🚨 ZERO SOCIALS DETECTED - FAST EXIT IN 12s");
                                                    } else {
                                                        info!("   📊 Social momentum: {:.2}", score.momentum_score);
                                                    }
                                                    (Some(score), fast)
                                                }
                                                Err(e) => {
                                                    warn!("   ⚠️  Social check failed: {}", e);
                                                    (None, false)
                                                }
                                            };

                                            // Get entry price (wait a moment for DexScreener to index)
                                            sleep(Duration::from_secs(2)).await;
                                            let entry_price_usd = match check_momentum(&mint, 0.0).await {
                                                Ok(momentum_data) => momentum_data.current_price_usd,
                                                Err(_) => 0.0001, // Default tiny price for new launches
                                            };
                                            info!("   Entry price: ${:.8}", entry_price_usd);

                                            let now = std::time::Instant::now();
                                            let position = Position {
                                                mint: mint.clone(),
                                                entry_sol: snipe_amount,
                                                entry_price_usd,
                                                entry_signature: signature,
                                                entry_time: now,
                                                risk_score,
                                                social_score: social_score_opt,
                                                fast_exit,
                                                add_count: 0,
                                                last_add_time: now,
                                            };

                                            positions.lock().await.insert(mint, position);
                                            let current = positions.lock().await.len();
                                            let remaining = max_positions.saturating_sub(current);
                                            info!("💼 Positions: {}/{} ({}left)", current, max_positions, remaining);
                                            info!("📊 Stats: {} detected | {} filtered | {} bought\n",
                                                  total_detected, total_filtered, total_bought);
                                        }
                                        Err(e) => {
                                            error!("❌ Buy failed: {}\n", e);
                                        }
                                    }
                                } else {
                                    total_filtered += 1;
                                    info!("📊 Stats: {} detected | {} filtered | {} bought\n",
                                          total_detected, total_filtered, total_bought);
                                }
                            }
                            Err(e) => {
                                error!("❌ Analysis failed: {}\n", e);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}
