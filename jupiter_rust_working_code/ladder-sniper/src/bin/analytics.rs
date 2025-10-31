use anyhow::Result;
use duckdb::Connection;
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let db_path = if args.len() > 1 {
        &args[1]
    } else {
        "./data/research.duckdb"
    };

    let conn = Connection::open(db_path)?;

    println!("\n📊 RESEARCH DATABASE ANALYTICS");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Overall stats
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM trades")?;
    let total_trades: i64 = stmt.query_row([], |row| row.get(0))?;

    let mut stmt = conn.prepare("SELECT COUNT(*) FROM positions WHERE exit_time_micros IS NOT NULL")?;
    let closed_positions: i64 = stmt.query_row([], |row| row.get(0))?;

    let mut stmt = conn.prepare("SELECT COUNT(*) FROM positions WHERE exit_time_micros IS NULL")?;
    let open_positions: i64 = stmt.query_row([], |row| row.get(0))?;

    println!("\n📈 OVERVIEW:");
    println!("   Total Trades: {}", total_trades);
    println!("   Closed Positions: {}", closed_positions);
    println!("   Open Positions: {}", open_positions);

    // Win/Loss stats
    if closed_positions > 0 {
        let mut stmt = conn.prepare(
            "SELECT
                COUNT(*) FILTER (WHERE pnl_sol > 0) as wins,
                COUNT(*) FILTER (WHERE pnl_sol <= 0) as losses,
                AVG(pnl_percent) as avg_pnl_pct,
                SUM(pnl_sol) as total_pnl_sol,
                AVG(hold_duration_secs) as avg_hold_secs
            FROM positions
            WHERE exit_time_micros IS NOT NULL"
        )?;

        let (wins, losses, avg_pnl_pct, total_pnl_sol, avg_hold_secs) = stmt.query_row([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, f64>(2)?,
                row.get::<_, f64>(3)?,
                row.get::<_, f64>(4)?,
            ))
        })?;

        let win_rate = (wins as f64 / closed_positions as f64) * 100.0;

        println!("\n💰 PERFORMANCE:");
        println!("   Win Rate: {:.1}% ({} W / {} L)", win_rate, wins, losses);
        println!("   Avg P&L: {:+.2}%", avg_pnl_pct);
        println!("   Total P&L: {:+.4} SOL", total_pnl_sol);
        println!("   Avg Hold Time: {:.1}s", avg_hold_secs);
    }

    // Holder count analysis
    if closed_positions > 0 {
        let mut stmt = conn.prepare(
            "SELECT
                AVG(holder_count_entry) as avg_entry_holders,
                AVG(holder_count_exit) as avg_exit_holders,
                AVG(CASE WHEN pnl_sol > 0 THEN holder_count_entry END) as avg_win_entry_holders,
                AVG(CASE WHEN pnl_sol <= 0 THEN holder_count_entry END) as avg_loss_entry_holders
            FROM positions
            WHERE exit_time_micros IS NOT NULL"
        )?;

        let (avg_entry, avg_exit, avg_win_entry, avg_loss_entry) = stmt.query_row([], |row| {
            Ok((
                row.get::<_, f64>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, Option<f64>>(2)?,
                row.get::<_, Option<f64>>(3)?,
            ))
        })?;

        println!("\n👥 HOLDER COUNT ANALYSIS:");
        println!("   Avg Entry Holders: {:.0}", avg_entry);
        println!("   Avg Exit Holders: {:.0}", avg_exit);
        if let Some(win_holders) = avg_win_entry {
            println!("   Avg Winning Entry Holders: {:.0}", win_holders);
        }
        if let Some(loss_holders) = avg_loss_entry {
            println!("   Avg Losing Entry Holders: {:.0}", loss_holders);
        }
    }

    // Top winners
    println!("\n🏆 TOP 5 WINNERS:");
    let mut stmt = conn.prepare(
        "SELECT mint, pnl_percent, pnl_sol, hold_duration_secs, holder_count_entry, holder_count_exit
         FROM positions
         WHERE exit_time_micros IS NOT NULL
         ORDER BY pnl_sol DESC
         LIMIT 5"
    )?;

    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let mint: String = row.get(0)?;
        let pnl_pct: f64 = row.get(1)?;
        let pnl_sol: f64 = row.get(2)?;
        let hold_secs: i64 = row.get(3)?;
        let entry_holders: i64 = row.get(4)?;
        let exit_holders: Option<i64> = row.get(5)?;

        println!(
            "   {} | {:+.1}% ({:+.4} SOL) | {}s hold | Holders: {} → {}",
            &mint[0..8],
            pnl_pct,
            pnl_sol,
            hold_secs,
            entry_holders,
            exit_holders.unwrap_or(0)
        );
    }

    // Top losers
    println!("\n📉 TOP 5 LOSERS:");
    let mut stmt = conn.prepare(
        "SELECT mint, pnl_percent, pnl_sol, hold_duration_secs, holder_count_entry, holder_count_exit
         FROM positions
         WHERE exit_time_micros IS NOT NULL
         ORDER BY pnl_sol ASC
         LIMIT 5"
    )?;

    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let mint: String = row.get(0)?;
        let pnl_pct: f64 = row.get(1)?;
        let pnl_sol: f64 = row.get(2)?;
        let hold_secs: i64 = row.get(3)?;
        let entry_holders: i64 = row.get(4)?;
        let exit_holders: Option<i64> = row.get(5)?;

        println!(
            "   {} | {:+.1}% ({:+.4} SOL) | {}s hold | Holders: {} → {}",
            &mint[0..8],
            pnl_pct,
            pnl_sol,
            hold_secs,
            entry_holders,
            exit_holders.unwrap_or(0)
        );
    }

    // Time-series metrics sample
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM position_metrics")?;
    let metric_count: i64 = stmt.query_row([], |row| row.get(0))?;

    println!("\n📊 TIME-SERIES DATA:");
    println!("   Total Metric Snapshots: {}", metric_count);
    println!("   (Use SQL queries for detailed time-series analysis)");

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    Ok(())
}
