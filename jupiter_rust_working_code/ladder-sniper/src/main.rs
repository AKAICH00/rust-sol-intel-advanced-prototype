use anyhow::Result;
use dotenv::dotenv;
use futures_util::{SinkExt, StreamExt};
use log::{info, warn, error};
use pump_portal_sdk::{PumpPortalClient, TradeRequest};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};

mod trade_events;
mod candle_builder;
mod vwap;
mod momentum;
mod paper_trading;
mod holder_count;
mod data_export;

use candle_builder::CandleBuilder;
use momentum::MomentumDetector;
use trade_events::TradeEvent;
use vwap::VWAPTracker;
use paper_trading::{PaperTradingConfig, PaperTradingSimulator, SharedExporter};
use holder_count::HolderCountClient;
use data_export::DataExporter;

#[derive(Debug, Clone)]
struct Position {
    mint: String,
    entry_time: Instant,
    entry_price: f64,
    total_sol_invested: f64,
    candle_builder: CandleBuilder,
    vwap_tracker: VWAPTracker,
    profits_taken: bool,
    holder_count: u64,
}

#[derive(Debug, Deserialize)]
struct TokenCreatedEvent {
    mint: Option<String>,
    name: Option<String>,
    symbol: Option<String>,
}

type Positions = Arc<Mutex<HashMap<String, Position>>>;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    dotenv().ok();

    info!("⚡ VWAP MOMENTUM SNIPER - Sub-Millisecond Indicators");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let api_key = env::var("PUMPPORTAL_API_KEY").expect("PUMPPORTAL_API_KEY required");

    let base_amount = env::var("BASE_AMOUNT_SOL")
        .unwrap_or_else(|_| "0.02".to_string())
        .parse::<f64>()?;

    let candle_interval_ms = env::var("CANDLE_INTERVAL_MS")
        .unwrap_or_else(|_| "500".to_string())
        .parse::<u64>()?;

    let momentum_threshold = env::var("MOMENTUM_EXIT_THRESHOLD")
        .unwrap_or_else(|_| "0.2".to_string())
        .parse::<f64>()?;

    let vwap_deviation = env::var("VWAP_EXIT_DEVIATION")
        .unwrap_or_else(|_| "0.95".to_string())
        .parse::<f64>()?;

    // Burst mode configuration
    let max_trades = env::var("MAX_TRADES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok());

    if let Some(max) = max_trades {
        info!("🚀 BURST MODE: Will stop after {} trades", max);
    }

    // Paper trading setup
    let paper_config = PaperTradingConfig::from_env();

    // Initialize DuckDB exporter for research
    let exporter = if paper_config.enabled {
        let db_path = env::var("DUCKDB_PATH")
            .unwrap_or_else(|_| "./data/research.duckdb".to_string());

        match DataExporter::new(&db_path, true) {
            Ok(exp) => {
                info!("📊 DuckDB Analytics: ENABLED");
                info!("   Database: {}", db_path);
                Some(Arc::new(Mutex::new(exp)))
            }
            Err(e) => {
                warn!("⚠️  Failed to initialize DuckDB: {}", e);
                None
            }
        }
    } else {
        None
    };

    let paper_sim = if paper_config.enabled {
        Some(Arc::new(PaperTradingSimulator::new(paper_config.clone(), exporter)))
    } else {
        None
    };

    info!("💰 Config:");
    if paper_config.enabled {
        info!("   🧪 PAPER MODE: ENABLED");
        info!("   Starting Balance: {} SOL", paper_config.starting_balance);
        info!("   Buy Latency: {}ms", paper_config.buy_latency_ms);
        info!("   Sell Latency: {}ms", paper_config.sell_latency_ms);
        info!("   Trade Fee: {:.1}%", paper_config.trade_fee_percent);
        info!("   Priority Fee: {} SOL", paper_config.priority_fee_sol);
        info!("");
    }
    info!("   Buy Amount: {} SOL", base_amount);
    info!("   Candle Interval: {}ms", candle_interval_ms);
    info!("   Momentum Threshold: {:.0}%", momentum_threshold * 100.0);
    info!("   VWAP Exit: {:.0}% deviation", (1.0 - vwap_deviation) * 100.0);
    info!("   Time Exits: 10s, 20s, 30s, 45s, 60s");
    info!("");

    let client = Arc::new(PumpPortalClient::new(api_key));
    let positions: Positions = Arc::new(Mutex::new(HashMap::new()));

    // Solana RPC for holder counts
    let rpc_url = env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
    let holder_client = Arc::new(HolderCountClient::new(rpc_url));

    // Start position monitor
    let monitor_client = client.clone();
    let monitor_positions = positions.clone();
    let momentum_detector = Arc::new(MomentumDetector::new(momentum_threshold));
    let monitor_paper_sim = paper_sim.clone();

    tokio::spawn(async move {
        monitor_positions_loop(
            monitor_client,
            monitor_positions,
            momentum_detector,
            candle_interval_ms,
            monitor_paper_sim,
        )
        .await;
    });

    // Connect to PumpPortal WebSocket
    info!("📡 Connecting to PumpPortal WebSocket...");
    let ws_url = "wss://pumpportal.fun/api/data";
    let (ws_stream, _) = connect_async(ws_url).await?;
    let (mut write, mut read) = ws_stream.split();

    let subscribe_msg = serde_json::json!({
        "method": "subscribeNewToken"
    });
    write.send(Message::Text(subscribe_msg.to_string())).await?;
    info!("✅ Subscribed to new token launches\n");
    info!("🎯 BUYING ALL LAUNCHES... Press Ctrl+C to stop\n");

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(event) = serde_json::from_str::<TokenCreatedEvent>(&text) {
                    if let (Some(mint), Some(name), Some(symbol)) = (event.mint, event.name, event.symbol) {
                        info!("🔔 NEW LAUNCH: {} ({})", name, symbol);
                        info!("   Mint: {}", mint);

                        // Execute initial buy
                        match execute_buy(&client, &mint, base_amount, &paper_sim).await {
                            Ok(_sig) => {
                                info!("✅ BOUGHT: {} SOL", base_amount);

                                // Fetch holder count
                                let holder_count = holder_client.get_holder_count(&mint).await.unwrap_or(0);
                                if holder_count > 0 {
                                    info!("👥 HOLDERS: {}", holder_count);
                                }

                                // Initialize position with VWAP + momentum tracking
                                let position = Position {
                                    mint: mint.clone(),
                                    entry_time: Instant::now(),
                                    entry_price: 0.0, // Will be set from first candle
                                    total_sol_invested: base_amount,
                                    candle_builder: CandleBuilder::new(candle_interval_ms, 100),
                                    vwap_tracker: VWAPTracker::new(),
                                    profits_taken: false,
                                    holder_count,
                                };

                                // Record entry trade
                                let entry_trade = TradeEvent::new_buy(0.0, base_amount);
                                positions.lock().await.insert(mint.clone(), position);

                                // Add trade to position trackers
                                if let Some(pos) = positions.lock().await.get_mut(&mint) {
                                    pos.vwap_tracker.add_trade(&entry_trade);
                                    pos.candle_builder.add_trade(&entry_trade);
                                }

                                let pos_count = positions.lock().await.len();
                                info!("💼 Open Positions: {}\n", pos_count);
                            }
                            Err(e) => {
                                error!("❌ Buy failed: {}\n", e);
                            }
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => {
                warn!("WebSocket closed");
                break;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
            }
            _ => {}
        }
    }

    // Print paper trading summary
    if let Some(sim) = &paper_sim {
        sim.print_summary().await;
    }

    Ok(())
}

async fn execute_buy(
    client: &PumpPortalClient,
    mint: &str,
    amount_sol: f64,
    paper_sim: &Option<Arc<PaperTradingSimulator>>,
) -> Result<String> {
    // Paper trading mode
    if let Some(sim) = paper_sim {
        // Use estimated entry price for paper trading (bonding curve start ~0.0000001 SOL/token)
        let estimated_price = 0.0000001;
        return sim.simulate_buy(mint.to_string(), amount_sol, estimated_price).await;
    }

    // Real trading
    let request = TradeRequest::buy(
        mint.to_string(),
        amount_sol,
        15,
        0.0001,
    ).with_jito_only(true);

    let response = client.trade(request).await?;
    Ok(response.signature.unwrap_or_else(|| "unknown".to_string()))
}

async fn execute_sell(
    client: &PumpPortalClient,
    mint: &str,
    percent: u32,
    paper_sim: &Option<Arc<PaperTradingSimulator>>,
    exit_reason: Option<String>,
) -> Result<String> {
    // Paper trading mode
    if let Some(sim) = paper_sim {
        // Use estimated exit price for paper trading (assume similar to entry for now)
        let estimated_price = 0.0000001;
        return sim.simulate_sell(mint, estimated_price, exit_reason).await;
    }

    // Real trading
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

async fn monitor_positions_loop(
    client: Arc<PumpPortalClient>,
    positions: Positions,
    momentum_detector: Arc<MomentumDetector>,
    _candle_interval_ms: u64,
    paper_sim: Option<Arc<PaperTradingSimulator>>,
) {
    info!("👀 VWAP + Momentum Monitor Started\n");

    // Check every 1 second for time-based exits
    loop {
        sleep(Duration::from_secs(1)).await;

        let positions_snapshot: Vec<Position> = {
            let locked = positions.lock().await;
            locked.values().cloned().collect()
        };

        if positions_snapshot.is_empty() {
            continue;
        }

        info!("📊 Monitoring {} positions...", positions_snapshot.len());

        for position in positions_snapshot {
            let elapsed = position.entry_time.elapsed().as_secs();
            let mint_short = &position.mint[..8];

            // Get current candle if exists
            let current_candle = position.candle_builder.current_candle();
            if current_candle.is_none() {
                info!("   {} ({}s) - Building candles...", mint_short, elapsed);
                continue;
            }

            let candle = current_candle.unwrap();
            let current_price = candle.close;

            // Set entry price on first candle
            if position.entry_price == 0.0 {
                if let Some(pos) = positions.lock().await.get_mut(&position.mint) {
                    pos.entry_price = current_price;
                }
            }

            // Calculate P&L
            let entry_price = if position.entry_price > 0.0 {
                position.entry_price
            } else {
                current_price
            };

            let pnl_multiplier = if entry_price > 0.0 {
                current_price / entry_price
            } else {
                1.0
            };
            let pnl_percent = (pnl_multiplier - 1.0) * 100.0;

            // Get VWAP info
            let vwap = position.vwap_tracker.vwap();
            let vwap_distance = position.vwap_tracker.vwap_distance_percent();

            // Get momentum
            let momentum = momentum_detector.calculate_momentum(
                &position.candle_builder,
                &position.vwap_tracker,
                elapsed,
            );

            info!(
                "   {} ({}s) - P&L: {:.1}x ({:+.0}%) | VWAP: {:.8} ({:+.0}%) | Mom: {:.0}% | Buy: {:.0}% | Holders: {}",
                mint_short,
                elapsed,
                pnl_multiplier,
                pnl_percent,
                vwap,
                vwap_distance,
                momentum * 100.0,
                candle.buy_ratio() * 100.0,
                position.holder_count
            );

            // TAKE PROFIT AT 2X
            if !position.profits_taken && momentum_detector.should_take_profit(entry_price, current_price) {
                info!("   🎯 2X PROFIT! Taking 50%");
                match execute_sell(&client, &position.mint, 50, &paper_sim, Some("2X_PROFIT".to_string())).await {
                    Ok(sig) => {
                        info!("   ✅ SOLD 50%: {}", sig);
                        if let Some(pos) = positions.lock().await.get_mut(&position.mint) {
                            pos.profits_taken = true;
                        }
                    }
                    Err(e) => error!("   ❌ Sell failed: {}", e),
                }
                continue;
            }

            // TIME-BASED MOMENTUM EXIT
            let (should_exit, reason) = momentum_detector.check_time_exit(
                &position.candle_builder,
                &position.vwap_tracker,
                elapsed,
            );

            if should_exit {
                info!("   ❌ EXIT - {}", reason);
                match execute_sell(&client, &position.mint, 100, &paper_sim, Some(reason.clone())).await {
                    Ok(sig) => {
                        info!("   ✅ SOLD 100%: {}", sig);
                        positions.lock().await.remove(&position.mint);
                    }
                    Err(e) => error!("   ❌ Sell failed: {}", e),
                }
            }
        }

        info!(""); // Blank line
    }
}
