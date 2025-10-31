use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tracing::{info, warn};

/// Database client for position and trade tracking
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Create a new database connection
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_schema()?;
        info!("Database initialized");
        Ok(db)
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Positions table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS positions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                symbol TEXT NOT NULL,
                mint_address TEXT,
                entry_price REAL NOT NULL,
                current_price REAL NOT NULL,
                size_usd REAL NOT NULL,
                entry_time TEXT NOT NULL,
                status TEXT NOT NULL,
                peak_price REAL,
                trailing_stop REAL,
                unrealized_pnl REAL,
                unrealized_pnl_pct REAL,
                exit_price REAL,
                exit_time TEXT,
                realized_pnl REAL,
                realized_pnl_pct REAL,
                exit_reason TEXT,
                confidence_score REAL,
                volatility REAL
            )",
            [],
        )?;

        // Trades table (execution records)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS trades (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                position_id INTEGER,
                trade_type TEXT NOT NULL,
                symbol TEXT NOT NULL,
                price REAL NOT NULL,
                size_usd REAL NOT NULL,
                timestamp TEXT NOT NULL,
                signature TEXT,
                slippage_bps REAL,
                fees_usd REAL,
                execution_time_ms INTEGER,
                FOREIGN KEY (position_id) REFERENCES positions(id)
            )",
            [],
        )?;

        // Risk metrics snapshots
        conn.execute(
            "CREATE TABLE IF NOT EXISTS risk_snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                total_capital REAL NOT NULL,
                available_capital REAL NOT NULL,
                total_position_value REAL NOT NULL,
                unrealized_pnl REAL NOT NULL,
                realized_pnl REAL NOT NULL,
                daily_pnl REAL NOT NULL,
                daily_pnl_pct REAL NOT NULL,
                num_positions INTEGER NOT NULL,
                win_rate REAL,
                sharpe_estimate REAL,
                max_drawdown_pct REAL
            )",
            [],
        )?;

        // Signals table (ML predictions)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS signals (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                symbol TEXT NOT NULL,
                confidence REAL NOT NULL,
                predicted_return REAL,
                predicted_volatility REAL,
                embedding_vector TEXT,
                anomaly_score REAL,
                similar_patterns_count INTEGER,
                executed BOOLEAN DEFAULT 0,
                position_id INTEGER,
                FOREIGN KEY (position_id) REFERENCES positions(id)
            )",
            [],
        )?;

        info!("Database schema initialized");
        Ok(())
    }

    /// Insert new position
    pub fn insert_position(&self, pos: &PositionRecord) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO positions (
                symbol, mint_address, entry_price, current_price, size_usd,
                entry_time, status, peak_price, trailing_stop, unrealized_pnl,
                unrealized_pnl_pct, confidence_score, volatility
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                pos.symbol,
                pos.mint_address,
                pos.entry_price,
                pos.current_price,
                pos.size_usd,
                pos.entry_time.to_rfc3339(),
                "open",
                pos.peak_price,
                pos.trailing_stop,
                pos.unrealized_pnl,
                pos.unrealized_pnl_pct,
                pos.confidence_score,
                pos.volatility,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Update position with current market data
    pub fn update_position(&self, id: i64, current_price: f64, unrealized_pnl: f64, unrealized_pnl_pct: f64, peak_price: f64, trailing_stop: f64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE positions SET
                current_price = ?1,
                unrealized_pnl = ?2,
                unrealized_pnl_pct = ?3,
                peak_price = ?4,
                trailing_stop = ?5
            WHERE id = ?6",
            params![current_price, unrealized_pnl, unrealized_pnl_pct, peak_price, trailing_stop, id],
        )?;
        Ok(())
    }

    /// Close position with exit details
    pub fn close_position(&self, id: i64, exit_price: f64, realized_pnl: f64, realized_pnl_pct: f64, exit_reason: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let exit_time = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE positions SET
                status = 'closed',
                exit_price = ?1,
                exit_time = ?2,
                realized_pnl = ?3,
                realized_pnl_pct = ?4,
                exit_reason = ?5
            WHERE id = ?6",
            params![exit_price, exit_time, realized_pnl, realized_pnl_pct, exit_reason, id],
        )?;
        info!("Position {} closed: P&L=${:.2} ({:.2}%), Reason: {}",
              id, realized_pnl, realized_pnl_pct * 100.0, exit_reason);
        Ok(())
    }

    /// Get all open positions
    pub fn get_open_positions(&self) -> Result<Vec<(i64, String, f64, f64, f64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, symbol, entry_price, current_price, size_usd
             FROM positions
             WHERE status = 'open'"
        )?;

        let positions = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(positions)
    }

    /// Record trade execution
    pub fn insert_trade(&self, trade: &TradeRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO trades (
                position_id, trade_type, symbol, price, size_usd, timestamp,
                signature, slippage_bps, fees_usd, execution_time_ms
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                trade.position_id,
                trade.trade_type,
                trade.symbol,
                trade.price,
                trade.size_usd,
                trade.timestamp.to_rfc3339(),
                trade.signature,
                trade.slippage_bps,
                trade.fees_usd,
                trade.execution_time_ms,
            ],
        )?;
        Ok(())
    }

    /// Record risk metrics snapshot
    pub fn insert_risk_snapshot(&self, metrics: &crate::risk_manager::RiskMetrics) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO risk_snapshots (
                timestamp, total_capital, available_capital, total_position_value,
                unrealized_pnl, realized_pnl, daily_pnl, daily_pnl_pct,
                num_positions, win_rate, sharpe_estimate, max_drawdown_pct
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                Utc::now().to_rfc3339(),
                metrics.total_capital,
                metrics.available_capital,
                metrics.total_position_value,
                metrics.unrealized_pnl,
                metrics.realized_pnl,
                metrics.daily_pnl,
                metrics.daily_pnl_pct,
                metrics.num_positions,
                metrics.win_rate,
                metrics.sharpe_estimate,
                metrics.max_drawdown_pct,
            ],
        )?;
        Ok(())
    }

    /// Record ML signal
    pub fn insert_signal(&self, signal: &SignalRecord) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO signals (
                timestamp, symbol, confidence, predicted_return, predicted_volatility,
                embedding_vector, anomaly_score, similar_patterns_count
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                signal.timestamp.to_rfc3339(),
                signal.symbol,
                signal.confidence,
                signal.predicted_return,
                signal.predicted_volatility,
                signal.embedding_vector,
                signal.anomaly_score,
                signal.similar_patterns_count,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Mark signal as executed and link to position
    pub fn mark_signal_executed(&self, signal_id: i64, position_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE signals SET executed = 1, position_id = ?1 WHERE id = ?2",
            params![position_id, signal_id],
        )?;
        Ok(())
    }

    /// Get performance statistics
    pub fn get_performance_stats(&self) -> Result<PerformanceStats> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT
                COUNT(*) as total_trades,
                SUM(CASE WHEN realized_pnl > 0 THEN 1 ELSE 0 END) as winning_trades,
                AVG(realized_pnl_pct) as avg_return_pct,
                MAX(realized_pnl_pct) as max_return_pct,
                MIN(realized_pnl_pct) as min_return_pct,
                SUM(realized_pnl) as total_pnl
             FROM positions
             WHERE status = 'closed'"
        )?;

        let stats = stmt.query_row([], |row| {
            Ok(PerformanceStats {
                total_trades: row.get(0)?,
                winning_trades: row.get(1)?,
                avg_return_pct: row.get(2).unwrap_or(0.0),
                max_return_pct: row.get(3).unwrap_or(0.0),
                min_return_pct: row.get(4).unwrap_or(0.0),
                total_pnl: row.get(5).unwrap_or(0.0),
            })
        })?;

        Ok(stats)
    }
}

/// Position record for database
#[derive(Debug, Clone)]
pub struct PositionRecord {
    pub symbol: String,
    pub mint_address: Option<String>,
    pub entry_price: f64,
    pub current_price: f64,
    pub size_usd: f64,
    pub entry_time: DateTime<Utc>,
    pub peak_price: f64,
    pub trailing_stop: f64,
    pub unrealized_pnl: f64,
    pub unrealized_pnl_pct: f64,
    pub confidence_score: f32,
    pub volatility: f64,
}

/// Trade execution record
#[derive(Debug, Clone)]
pub struct TradeRecord {
    pub position_id: Option<i64>,
    pub trade_type: String, // "buy" or "sell"
    pub symbol: String,
    pub price: f64,
    pub size_usd: f64,
    pub timestamp: DateTime<Utc>,
    pub signature: Option<String>,
    pub slippage_bps: Option<f64>,
    pub fees_usd: Option<f64>,
    pub execution_time_ms: Option<i64>,
}

/// ML signal record
#[derive(Debug, Clone)]
pub struct SignalRecord {
    pub timestamp: DateTime<Utc>,
    pub symbol: String,
    pub confidence: f32,
    pub predicted_return: Option<f32>,
    pub predicted_volatility: Option<f32>,
    pub embedding_vector: Option<String>, // JSON serialized
    pub anomaly_score: Option<f32>,
    pub similar_patterns_count: Option<i32>,
}

/// Performance statistics
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub total_trades: i32,
    pub winning_trades: i32,
    pub avg_return_pct: f64,
    pub max_return_pct: f64,
    pub min_return_pct: f64,
    pub total_pnl: f64,
}

impl PerformanceStats {
    pub fn win_rate(&self) -> f64 {
        if self.total_trades > 0 {
            self.winning_trades as f64 / self.total_trades as f64
        } else {
            0.0
        }
    }
}
