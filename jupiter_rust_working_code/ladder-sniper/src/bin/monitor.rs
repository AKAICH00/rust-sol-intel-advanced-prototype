use anyhow::Result;
use duckdb::Connection;
use std::env;
use std::thread;
use std::time::Duration;
use serde::Serialize;
use serde_json;

#[derive(Debug, Serialize)]
struct LiveStats {
    timestamp: String,
    total_trades: i64,
    open_positions: i64,
    closed_positions: i64,
    wins: i64,
    losses: i64,
    win_rate_pct: f64,
    total_pnl_sol: f64,
    avg_pnl_pct: f64,
    avg_hold_secs: f64,
    avg_entry_holders: f64,
    avg_exit_holders: f64,
    last_trade_ago_secs: i64,
}

fn get_stats(conn: &Connection) -> Result<LiveStats> {
    // Total trades
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM trades")?;
    let total_trades: i64 = stmt.query_row([], |row| row.get(0))?;

    // Position counts
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM positions WHERE exit_time_micros IS NULL")?;
    let open_positions: i64 = stmt.query_row([], |row| row.get(0))?;

    let mut stmt = conn.prepare("SELECT COUNT(*) FROM positions WHERE exit_time_micros IS NOT NULL")?;
    let closed_positions: i64 = stmt.query_row([], |row| row.get(0))?;

    let (wins, losses, avg_pnl_pct, total_pnl_sol, avg_hold_secs, avg_entry_holders, avg_exit_holders) = if closed_positions > 0 {
        let mut stmt = conn.prepare(
            "SELECT
                COUNT(*) FILTER (WHERE pnl_sol > 0) as wins,
                COUNT(*) FILTER (WHERE pnl_sol <= 0) as losses,
                AVG(pnl_percent) as avg_pnl_pct,
                SUM(pnl_sol) as total_pnl_sol,
                AVG(hold_duration_secs) as avg_hold_secs,
                AVG(holder_count_entry) as avg_entry_holders,
                AVG(holder_count_exit) as avg_exit_holders
            FROM positions
            WHERE exit_time_micros IS NOT NULL"
        )?;

        stmt.query_row([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, f64>(2)?,
                row.get::<_, f64>(3)?,
                row.get::<_, f64>(4)?,
                row.get::<_, f64>(5)?,
                row.get::<_, f64>(6)?,
            ))
        })?
    } else {
        (0, 0, 0.0, 0.0, 0.0, 0.0, 0.0)
    };

    let win_rate_pct = if closed_positions > 0 {
        (wins as f64 / closed_positions as f64) * 100.0
    } else {
        0.0
    };

    // Time since last trade
    let last_trade_ago_secs = if total_trades > 0 {
        let mut stmt = conn.prepare(
            "SELECT MAX(timestamp_micros) FROM trades"
        )?;
        let last_trade_micros: i64 = stmt.query_row([], |row| row.get(0))?;
        let now_micros = chrono::Utc::now().timestamp_micros();
        ((now_micros - last_trade_micros) / 1_000_000) as i64
    } else {
        0
    };

    Ok(LiveStats {
        timestamp: chrono::Utc::now().to_rfc3339(),
        total_trades,
        open_positions,
        closed_positions,
        wins,
        losses,
        win_rate_pct,
        total_pnl_sol,
        avg_pnl_pct,
        avg_hold_secs,
        avg_entry_holders,
        avg_exit_holders,
        last_trade_ago_secs,
    })
}

fn print_stats_human(stats: &LiveStats) {
    // Clear screen
    print!("\x1B[2J\x1B[1;1H");

    println!("\n📊 LIVE RESEARCH MONITOR");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("⏰ {}", stats.timestamp);
    println!("");

    println!("📈 TRADING ACTIVITY:");
    println!("   Total Trades: {}", stats.total_trades);
    println!("   Open Positions: {}", stats.open_positions);
    println!("   Closed Positions: {}", stats.closed_positions);
    println!("   Last Trade: {}s ago", stats.last_trade_ago_secs);

    if stats.closed_positions > 0 {
        println!("");
        println!("💰 PERFORMANCE:");
        println!("   Win Rate: {:.1}% ({} W / {} L)", stats.win_rate_pct, stats.wins, stats.losses);
        println!("   Total P&L: {:+.4} SOL", stats.total_pnl_sol);
        println!("   Avg P&L: {:+.2}%", stats.avg_pnl_pct);
        println!("   Avg Hold Time: {:.1}s", stats.avg_hold_secs);

        println!("");
        println!("👥 HOLDER ANALYSIS:");
        println!("   Avg Entry Holders: {:.0}", stats.avg_entry_holders);
        println!("   Avg Exit Holders: {:.0}", stats.avg_exit_holders);

        let holder_change = stats.avg_exit_holders - stats.avg_entry_holders;
        let change_emoji = if holder_change > 0.0 { "📈" } else { "📉" };
        println!("   {} Avg Change: {:+.0}", change_emoji, holder_change);
    }

    println!("");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Updating every 5s... Press Ctrl+C to stop");
}

fn print_stats_json(stats: &LiveStats) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(stats)?);
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut db_path = "./data/research.duckdb";
    let mut json_mode = false;
    let mut interval_secs = 5u64;

    // Parse args
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--interval" => {
                if i + 1 < args.len() {
                    interval_secs = args[i + 1].parse().unwrap_or(5);
                    i += 1;
                }
            },
            _ => {
                if !args[i].starts_with("--") {
                    db_path = &args[i];
                }
            }
        }
        i += 1;
    }

    let conn = Connection::open(db_path)?;

    if json_mode {
        // JSON mode - single output
        let stats = get_stats(&conn)?;
        print_stats_json(&stats)?;
    } else {
        // Live monitor mode - updates every N seconds
        loop {
            let stats = get_stats(&conn)?;
            print_stats_human(&stats);
            thread::sleep(Duration::from_secs(interval_secs));
        }
    }

    Ok(())
}
