//! Sniper Demon - AI-Assisted Trading Decision Agent
//!
//! Watches positions and triggers AI analysis when conditions require human-level judgment

mod ai;

use ai::{AiProvider, AiProviderFactory, DecisionContext, TriggerType, DecisionAction};
use anyhow::Result;
use log::{info, warn, error};
use rusqlite::Connection;
use std::time::Duration;
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    dotenv().ok();

    info!("🔮 Sniper Demon Starting...");
    info!("   AI-Assisted Trading Decisions");
    info!("   Watching for trigger conditions\n");

    // Load configuration
    let ai_provider_type = env::var("AI_PROVIDER").unwrap_or_else(|_| "deepseek".to_string());

    // Try provider-specific key first, then generic
    let ai_api_key = match ai_provider_type.as_str() {
        "deepseek" => env::var("DEEPSEEK_API_KEY")
            .or_else(|_| env::var("AI_API_KEY"))
            .expect("DEEPSEEK_API_KEY not set"),
        "claude" => env::var("CLAUDE_API_KEY")
            .or_else(|_| env::var("AI_API_KEY"))
            .expect("CLAUDE_API_KEY not set"),
        "openai" => env::var("OPENAI_API_KEY")
            .or_else(|_| env::var("AI_API_KEY"))
            .expect("OPENAI_API_KEY not set"),
        _ => env::var("AI_API_KEY").expect("AI_API_KEY not set"),
    };

    let database_path = env::var("DATABASE_PATH").unwrap_or_else(|_| "../pump-sniper-bot/sniper_bot.db".to_string());
    let check_interval_secs: u64 = env::var("CHECK_INTERVAL_SECS")
        .unwrap_or_else(|_| "5".to_string())
        .parse()
        .unwrap_or(5);

    // Initialize AI provider
    let ai_provider = AiProviderFactory::create(&ai_provider_type, ai_api_key)?;

    // Health check
    info!("🔍 Checking AI provider: {}", ai_provider.name());
    match ai_provider.health_check().await {
        Ok(true) => info!("✅ {} is healthy", ai_provider.name()),
        Ok(false) => warn!("⚠️  {} health check failed", ai_provider.name()),
        Err(e) => error!("❌ {} error: {}", ai_provider.name(), e),
    }

    // Main event loop
    info!("\n👀 Monitoring positions every {}s...\n", check_interval_secs);

    loop {
        if let Err(e) = check_positions(&database_path, &*ai_provider).await {
            error!("Error checking positions: {}", e);
        }

        tokio::time::sleep(Duration::from_secs(check_interval_secs)).await;
    }
}

async fn check_positions(db_path: &str, ai: &dyn AiProvider) -> Result<()> {
    let conn = Connection::open(db_path)?;

    // Get all active positions
    let mut stmt = conn.prepare(
        "SELECT mint, entry_sol_amount, entry_time, entry_token_amount, current_token_amount
         FROM positions
         WHERE status = 'active'"
    )?;

    let positions = stmt.query_map([], |row| {
        Ok(ActivePosition {
            mint: row.get(0)?,
            entry_sol: row.get(1)?,
            entry_time: row.get(2)?,
            entry_tokens: row.get::<_, Option<f64>>(3)?,
            current_tokens: row.get::<_, Option<f64>>(4)?,
        })
    })?;

    for position_result in positions {
        let position = position_result?;

        // Get latest momentum data
        if let Some(momentum) = get_latest_momentum(&conn, &position.mint)? {
            // Check for trigger conditions
            if let Some(trigger) = detect_trigger(&position, &momentum) {
                info!("🎯 TRIGGER DETECTED: {:?} for {}", trigger, position.mint);

                // Build decision context
                let context = build_context(&position, &momentum, trigger);

                // Get AI decision
                match ai.get_decision(&context).await {
                    Ok(decision) => {
                        info!("✅ AI Decision: {:?}", decision.action);
                        info!("   Confidence: {:.2}", decision.confidence);
                        info!("   Reasoning: {}", decision.reasoning);

                        // Log decision
                        log_decision(&conn, &position.mint, &decision)?;

                        // Record recommendation for main bot
                        record_ai_recommendation(&conn, &position.mint, &decision)?;
                    }
                    Err(e) => {
                        error!("❌ AI decision failed: {}", e);
                    }
                }
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
struct ActivePosition {
    mint: String,
    entry_sol: f64,
    entry_time: i64,
    entry_tokens: Option<f64>,
    current_tokens: Option<f64>,
}

#[derive(Debug)]
struct MomentumData {
    score: f64,
    rug_risk: f64,
    volume_velocity: f64,
    price_momentum: f64,
    holder_health: f64,
}

fn get_latest_momentum(conn: &Connection, mint: &str) -> Result<Option<MomentumData>> {
    let result = conn.query_row(
        "SELECT score, rug_risk, volume_velocity, price_momentum, holder_health
         FROM momentum_snapshots
         WHERE mint = ?1
         ORDER BY timestamp DESC
         LIMIT 1",
        [mint],
        |row| {
            Ok(MomentumData {
                score: row.get(0)?,
                rug_risk: row.get(1)?,
                volume_velocity: row.get(2)?,
                price_momentum: row.get(3)?,
                holder_health: row.get(4)?,
            })
        },
    );

    match result {
        Ok(data) => Ok(Some(data)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

fn detect_trigger(position: &ActivePosition, momentum: &MomentumData) -> Option<TriggerType> {
    let now = chrono::Utc::now().timestamp();
    let time_elapsed = now - position.entry_time;

    // Estimate current value (would be calculated properly with real price data)
    let current_sol = position.entry_sol * 1.5; // Placeholder
    let profit_multiple = current_sol / position.entry_sol;

    // Priority 1: High rug risk (ALWAYS check this first)
    if momentum.rug_risk > 0.7 {
        return Some(TriggerType::HighRugRisk);
    }

    // Priority 2: Momentum stalled (Rule #7)
    if time_elapsed > 60 && momentum.score < 0.3 {
        return Some(TriggerType::MomentumStalled);
    }

    // Priority 3: Hit 2x profit target
    if profit_multiple >= 2.0 {
        // Check for conflicting signals
        if momentum.score > 0.7 {
            return Some(TriggerType::ConflictingSignals);
        } else {
            return Some(TriggerType::ProfitTarget2x);
        }
    }

    // Priority 4: High momentum while in profit
    if profit_multiple > 1.2 && momentum.score > 0.8 {
        return Some(TriggerType::HighMomentum);
    }

    None
}

fn build_context(position: &ActivePosition, momentum: &MomentumData, trigger: TriggerType) -> DecisionContext {
    let now = chrono::Utc::now().timestamp();
    let time_elapsed = now - position.entry_time;

    // TODO: Get real current value from position monitor
    let current_sol = position.entry_sol * 1.5; // Placeholder
    let profit_multiple = current_sol / position.entry_sol;

    DecisionContext {
        mint: position.mint.clone(),
        entry_sol: position.entry_sol,
        current_sol,
        profit_multiple,
        time_elapsed,
        momentum_score: momentum.score,
        rug_risk: momentum.rug_risk,
        volume_velocity: momentum.volume_velocity,
        price_momentum: momentum.price_momentum,
        holder_health: momentum.holder_health,
        has_recovered_initial: false, // TODO: Track this in database
        trailing_active: false,
        current_stop: None,
        trigger_type: trigger,
    }
}

fn log_decision(conn: &Connection, mint: &str, decision: &ai::AiDecision) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ai_decisions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            mint TEXT NOT NULL,
            action TEXT NOT NULL,
            confidence REAL NOT NULL,
            reasoning TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            executed BOOLEAN DEFAULT 0
        )",
        [],
    )?;

    let action_str = match &decision.action {
        DecisionAction::Hold => "Hold".to_string(),
        DecisionAction::ExitFull => "ExitFull".to_string(),
        DecisionAction::ExitPartial { percent } => format!("ExitPartial({}%)", percent),
        DecisionAction::Trail { stop_percent } => format!("Trail({}%)", stop_percent),
        DecisionAction::AdjustStop { new_stop } => format!("AdjustStop({})", new_stop),
        DecisionAction::Emergency => "Emergency".to_string(),
    };

    conn.execute(
        "INSERT INTO ai_decisions (mint, action, confidence, reasoning, timestamp)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            mint,
            action_str,
            decision.confidence,
            decision.reasoning,
            decision.timestamp
        ],
    )?;

    Ok(())
}

fn record_ai_recommendation(conn: &Connection, mint: &str, decision: &ai::AiDecision) -> Result<()> {
    // Create recommendations table for main bot to check
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ai_recommendations (
            mint TEXT PRIMARY KEY,
            action TEXT NOT NULL,
            confidence REAL NOT NULL,
            reasoning TEXT NOT NULL,
            suggested_stop REAL,
            timestamp INTEGER NOT NULL
        )",
        [],
    )?;

    let action_str = match &decision.action {
        DecisionAction::Hold => "Hold".to_string(),
        DecisionAction::ExitFull => "ExitFull".to_string(),
        DecisionAction::ExitPartial { percent } => format!("ExitPartial({}%)", percent),
        DecisionAction::Trail { stop_percent } => format!("Trail({}%)", stop_percent),
        DecisionAction::AdjustStop { new_stop } => format!("AdjustStop({})", new_stop),
        DecisionAction::Emergency => "Emergency".to_string(),
    };

    conn.execute(
        "INSERT OR REPLACE INTO ai_recommendations (mint, action, confidence, reasoning, suggested_stop, timestamp)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            mint,
            action_str,
            decision.confidence,
            decision.reasoning,
            decision.suggested_stops,
            decision.timestamp
        ],
    )?;

    info!("💾 AI recommendation saved for main bot to execute");

    Ok(())
}
