use anyhow::Result;
use chrono::{DateTime, Utc};
use duckdb::{Connection, params};
use log::{info, warn};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct TradeRecord {
    pub trade_id: String,
    pub timestamp_micros: i64,
    pub mint: String,
    pub trade_type: String,  // "BUY" or "SELL"
    pub price: f64,
    pub sol_amount: f64,
    pub tokens: f64,
    pub fee_sol: f64,
    pub priority_fee_sol: f64,
    pub balance_after: f64,
    pub signature: String,
}

#[derive(Debug, Clone)]
pub struct PositionRecord {
    pub position_id: String,
    pub mint: String,
    pub entry_time_micros: i64,
    pub exit_time_micros: Option<i64>,
    pub entry_price: f64,
    pub exit_price: Option<f64>,
    pub sol_invested: f64,
    pub sol_returned: Option<f64>,
    pub tokens: f64,
    pub pnl_sol: Option<f64>,
    pub pnl_percent: Option<f64>,
    pub hold_duration_secs: Option<i64>,
    pub holder_count_entry: u64,
    pub holder_count_exit: Option<u64>,
    pub exit_reason: Option<String>,
    pub profits_taken: bool,
}

#[derive(Debug, Clone)]
pub struct PositionMetricRecord {
    pub metric_id: String,
    pub position_id: String,
    pub mint: String,
    pub timestamp_micros: i64,
    pub elapsed_secs: i64,
    pub current_price: f64,
    pub pnl_multiplier: f64,
    pub pnl_percent: f64,
    pub vwap: f64,
    pub vwap_distance_percent: f64,
    pub momentum_score: f64,
    pub buy_ratio: f64,
    pub holder_count: u64,
}

pub struct DataExporter {
    conn: Connection,
    enabled: bool,
}

impl DataExporter {
    pub fn new(db_path: &str, enabled: bool) -> Result<Self> {
        if !enabled {
            return Ok(Self {
                conn: Connection::open_in_memory()?,
                enabled: false,
            });
        }

        let conn = Connection::open(db_path)?;

        // Initialize schema
        Self::init_schema(&conn)?;

        info!("📊 DuckDB Analytics initialized at: {}", db_path);

        Ok(Self { conn, enabled })
    }

    fn init_schema(conn: &Connection) -> Result<()> {
        // Trades table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS trades (
                trade_id VARCHAR PRIMARY KEY,
                timestamp_micros BIGINT NOT NULL,
                mint VARCHAR NOT NULL,
                trade_type VARCHAR NOT NULL,
                price DOUBLE NOT NULL,
                sol_amount DOUBLE NOT NULL,
                tokens DOUBLE NOT NULL,
                fee_sol DOUBLE NOT NULL,
                priority_fee_sol DOUBLE NOT NULL,
                balance_after DOUBLE NOT NULL,
                signature VARCHAR NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_trades_mint ON trades(mint)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_trades_timestamp ON trades(timestamp_micros)",
            [],
        )?;

        // Positions table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS positions (
                position_id VARCHAR PRIMARY KEY,
                mint VARCHAR NOT NULL,
                entry_time_micros BIGINT NOT NULL,
                exit_time_micros BIGINT,
                entry_price DOUBLE NOT NULL,
                exit_price DOUBLE,
                sol_invested DOUBLE NOT NULL,
                sol_returned DOUBLE,
                tokens DOUBLE NOT NULL,
                pnl_sol DOUBLE,
                pnl_percent DOUBLE,
                hold_duration_secs BIGINT,
                holder_count_entry BIGINT NOT NULL,
                holder_count_exit BIGINT,
                exit_reason VARCHAR,
                profits_taken BOOLEAN DEFAULT FALSE
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_positions_mint ON positions(mint)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_positions_entry_time ON positions(entry_time_micros)",
            [],
        )?;

        // Position metrics table (time-series with microsecond precision)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS position_metrics (
                metric_id VARCHAR PRIMARY KEY,
                position_id VARCHAR NOT NULL,
                mint VARCHAR NOT NULL,
                timestamp_micros BIGINT NOT NULL,
                elapsed_secs BIGINT NOT NULL,
                current_price DOUBLE NOT NULL,
                pnl_multiplier DOUBLE NOT NULL,
                pnl_percent DOUBLE NOT NULL,
                vwap DOUBLE NOT NULL,
                vwap_distance_percent DOUBLE NOT NULL,
                momentum_score DOUBLE NOT NULL,
                buy_ratio DOUBLE NOT NULL,
                holder_count BIGINT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_metrics_position ON position_metrics(position_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_metrics_timestamp ON position_metrics(timestamp_micros)",
            [],
        )?;

        info!("✅ Database schema initialized");
        Ok(())
    }

    pub fn record_trade(&self, trade: TradeRecord) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        self.conn.execute(
            "INSERT INTO trades VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                trade.trade_id,
                trade.timestamp_micros,
                trade.mint,
                trade.trade_type,
                trade.price,
                trade.sol_amount,
                trade.tokens,
                trade.fee_sol,
                trade.priority_fee_sol,
                trade.balance_after,
                trade.signature,
            ],
        )?;

        Ok(())
    }

    pub fn record_position(&self, position: PositionRecord) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        self.conn.execute(
            "INSERT OR REPLACE INTO positions VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                position.position_id,
                position.mint,
                position.entry_time_micros,
                position.exit_time_micros,
                position.entry_price,
                position.exit_price,
                position.sol_invested,
                position.sol_returned,
                position.tokens,
                position.pnl_sol,
                position.pnl_percent,
                position.hold_duration_secs,
                position.holder_count_entry as i64,
                position.holder_count_exit.map(|h| h as i64),
                position.exit_reason,
                position.profits_taken,
            ],
        )?;

        Ok(())
    }

    pub fn record_metric(&self, metric: PositionMetricRecord) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        self.conn.execute(
            "INSERT INTO position_metrics VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                metric.metric_id,
                metric.position_id,
                metric.mint,
                metric.timestamp_micros,
                metric.elapsed_secs,
                metric.current_price,
                metric.pnl_multiplier,
                metric.pnl_percent,
                metric.vwap,
                metric.vwap_distance_percent,
                metric.momentum_score,
                metric.buy_ratio,
                metric.holder_count as i64,
            ],
        )?;

        Ok(())
    }

    pub fn get_stats(&self) -> Result<(i64, i64, i64)> {
        if !self.enabled {
            return Ok((0, 0, 0));
        }

        let trade_count: i64 = self.conn
            .query_row("SELECT COUNT(*) FROM trades", [], |row| row.get(0))?;

        let position_count: i64 = self.conn
            .query_row("SELECT COUNT(*) FROM positions", [], |row| row.get(0))?;

        let metric_count: i64 = self.conn
            .query_row("SELECT COUNT(*) FROM position_metrics", [], |row| row.get(0))?;

        Ok((trade_count, position_count, metric_count))
    }

    pub fn print_summary(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let (trades, positions, metrics) = self.get_stats()?;

        info!("📊 DATABASE SUMMARY:");
        info!("   Trades: {}", trades);
        info!("   Positions: {}", positions);
        info!("   Metrics: {} (holder count snapshots)", metrics);

        // Calculate win rate if we have closed positions
        if positions > 0 {
            let win_stats: Result<(i64, i64, f64), _> = self.conn.query_row(
                "SELECT
                    SUM(CASE WHEN pnl_sol > 0 THEN 1 ELSE 0 END) as wins,
                    SUM(CASE WHEN pnl_sol <= 0 THEN 1 ELSE 0 END) as losses,
                    AVG(pnl_percent) as avg_pnl
                 FROM positions
                 WHERE exit_time_micros IS NOT NULL",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            );

            if let Ok((wins, losses, avg_pnl)) = win_stats {
                let total = wins + losses;
                if total > 0 {
                    let win_rate = (wins as f64 / total as f64) * 100.0;
                    info!("   Win Rate: {:.1}% ({}/{} trades)", win_rate, wins, total);
                    info!("   Avg P&L: {:.2}%", avg_pnl);
                }
            }
        }

        Ok(())
    }
}

pub type SharedExporter = Arc<Mutex<DataExporter>>;

pub fn get_timestamp_micros() -> i64 {
    Utc::now().timestamp_micros()
}
