//! SQLite Database for Sniper Bot State
//!
//! Tracks positions, transactions, whale wallets, and momentum data

use rusqlite::{Connection, Result as SqlResult, params};
use anyhow::{Result, Context};
use log::{info, error};
use std::path::Path;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.initialize_schema()?;
        Ok(db)
    }

    fn initialize_schema(&self) -> Result<()> {
        info!("Initializing database schema...");

        // Positions table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS positions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mint TEXT NOT NULL UNIQUE,
                entry_signature TEXT NOT NULL,
                entry_time INTEGER NOT NULL,
                entry_sol_amount REAL NOT NULL,
                entry_token_amount REAL,
                entry_price REAL,
                current_token_amount REAL,
                exit_signature TEXT,
                exit_time INTEGER,
                exit_sol_received REAL,
                status TEXT NOT NULL DEFAULT 'active',
                profit_loss_sol REAL,
                profit_loss_percent REAL,
                exit_reason TEXT,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )",
            [],
        )?;

        // Transactions table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS transactions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                signature TEXT NOT NULL UNIQUE,
                mint TEXT NOT NULL,
                tx_type TEXT NOT NULL,
                sol_amount REAL,
                token_amount REAL,
                price REAL,
                verified BOOLEAN NOT NULL DEFAULT 0,
                verification_time INTEGER,
                timestamp INTEGER NOT NULL,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )",
            [],
        )?;

        // Whale wallets table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS whale_wallets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mint TEXT NOT NULL,
                wallet_address TEXT NOT NULL,
                token_amount REAL NOT NULL,
                holdings_percent REAL NOT NULL,
                danger_level TEXT NOT NULL,
                last_check INTEGER NOT NULL,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                UNIQUE(mint, wallet_address)
            )",
            [],
        )?;

        // Momentum snapshots table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS momentum_snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mint TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                score REAL NOT NULL,
                rug_risk REAL NOT NULL,
                volume_velocity REAL NOT NULL,
                price_momentum REAL NOT NULL,
                holder_health REAL NOT NULL,
                buy_count INTEGER NOT NULL,
                sell_count INTEGER NOT NULL,
                unique_buyers INTEGER NOT NULL,
                unique_sellers INTEGER NOT NULL,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )",
            [],
        )?;

        // Create indexes
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_positions_mint ON positions(mint)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_positions_status ON positions(status)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_transactions_mint ON transactions(mint)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_transactions_signature ON transactions(signature)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_whale_wallets_mint ON whale_wallets(mint)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_momentum_snapshots_mint ON momentum_snapshots(mint)",
            [],
        )?;

        info!("✅ Database schema initialized");
        Ok(())
    }

    // Position operations
    pub fn create_position(
        &self,
        mint: &str,
        entry_signature: &str,
        entry_sol_amount: f64,
    ) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        self.conn.execute(
            "INSERT INTO positions (mint, entry_signature, entry_time, entry_sol_amount, status)
             VALUES (?1, ?2, ?3, ?4, 'active')",
            params![mint, entry_signature, now, entry_sol_amount],
        )?;

        info!("✅ Position created: {} @ {} SOL", mint, entry_sol_amount);
        Ok(())
    }

    pub fn update_position_entry_details(
        &self,
        mint: &str,
        token_amount: f64,
        price: f64,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE positions
             SET entry_token_amount = ?1, entry_price = ?2, current_token_amount = ?1
             WHERE mint = ?3 AND status = 'active'",
            params![token_amount, price, mint],
        )?;
        Ok(())
    }

    pub fn update_position_balance(&self, mint: &str, current_amount: f64) -> Result<()> {
        self.conn.execute(
            "UPDATE positions SET current_token_amount = ?1 WHERE mint = ?2 AND status = 'active'",
            params![current_amount, mint],
        )?;
        Ok(())
    }

    pub fn close_position(
        &self,
        mint: &str,
        exit_signature: &str,
        exit_sol: f64,
        reason: &str,
    ) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        // Calculate P&L
        let entry_sol: f64 = self.conn.query_row(
            "SELECT entry_sol_amount FROM positions WHERE mint = ?1 AND status = 'active'",
            params![mint],
            |row| row.get(0),
        )?;

        let profit_loss = exit_sol - entry_sol;
        let profit_percent = (profit_loss / entry_sol) * 100.0;

        self.conn.execute(
            "UPDATE positions
             SET exit_signature = ?1, exit_time = ?2, exit_sol_received = ?3,
                 status = 'closed', profit_loss_sol = ?4, profit_loss_percent = ?5,
                 exit_reason = ?6
             WHERE mint = ?7 AND status = 'active'",
            params![
                exit_signature,
                now,
                exit_sol,
                profit_loss,
                profit_percent,
                reason,
                mint
            ],
        )?;

        info!(
            "✅ Position closed: {} | P&L: {:.4} SOL ({:.1}%) | Reason: {}",
            mint, profit_loss, profit_percent, reason
        );
        Ok(())
    }

    pub fn get_active_position(&self, mint: &str) -> Result<Option<Position>> {
        let result = self.conn.query_row(
            "SELECT mint, entry_signature, entry_time, entry_sol_amount,
                    entry_token_amount, entry_price, current_token_amount
             FROM positions
             WHERE mint = ?1 AND status = 'active'",
            params![mint],
            |row| {
                Ok(Position {
                    mint: row.get(0)?,
                    entry_signature: row.get(1)?,
                    entry_time: row.get(2)?,
                    entry_sol_amount: row.get(3)?,
                    entry_token_amount: row.get(4)?,
                    entry_price: row.get(5)?,
                    current_token_amount: row.get(6)?,
                })
            },
        );

        match result {
            Ok(pos) => Ok(Some(pos)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    // Transaction operations
    pub fn record_transaction(
        &self,
        signature: &str,
        mint: &str,
        tx_type: &str,
        sol_amount: f64,
        timestamp: i64,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO transactions
             (signature, mint, tx_type, sol_amount, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![signature, mint, tx_type, sol_amount, timestamp],
        )?;
        Ok(())
    }

    pub fn mark_transaction_verified(&self, signature: &str, verified: bool) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        self.conn.execute(
            "UPDATE transactions SET verified = ?1, verification_time = ?2 WHERE signature = ?3",
            params![verified, now, signature],
        )?;
        Ok(())
    }

    pub fn is_transaction_verified(&self, signature: &str) -> Result<bool> {
        let result: i32 = self.conn.query_row(
            "SELECT verified FROM transactions WHERE signature = ?1",
            params![signature],
            |row| row.get(0),
        )?;
        Ok(result == 1)
    }

    // Whale tracking
    pub fn update_whale(&self, mint: &str, wallet: &str, amount: f64, percent: f64, danger: &str) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        self.conn.execute(
            "INSERT OR REPLACE INTO whale_wallets
             (mint, wallet_address, token_amount, holdings_percent, danger_level, last_check)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![mint, wallet, amount, percent, danger, now],
        )?;
        Ok(())
    }

    pub fn get_whales(&self, mint: &str) -> Result<Vec<WhaleWallet>> {
        let mut stmt = self.conn.prepare(
            "SELECT wallet_address, token_amount, holdings_percent, danger_level, last_check
             FROM whale_wallets
             WHERE mint = ?1
             ORDER BY holdings_percent DESC"
        )?;

        let whales = stmt.query_map(params![mint], |row| {
            Ok(WhaleWallet {
                wallet_address: row.get(0)?,
                token_amount: row.get(1)?,
                holdings_percent: row.get(2)?,
                danger_level: row.get(3)?,
                last_check: row.get(4)?,
            })
        })?
        .collect::<SqlResult<Vec<_>>>()?;

        Ok(whales)
    }

    // Momentum tracking
    pub fn save_momentum_snapshot(
        &self,
        mint: &str,
        score: f64,
        rug_risk: f64,
        volume_velocity: f64,
        price_momentum: f64,
        holder_health: f64,
        buy_count: i32,
        sell_count: i32,
        unique_buyers: i32,
        unique_sellers: i32,
    ) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        self.conn.execute(
            "INSERT INTO momentum_snapshots
             (mint, timestamp, score, rug_risk, volume_velocity, price_momentum, holder_health,
              buy_count, sell_count, unique_buyers, unique_sellers)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                mint,
                now,
                score,
                rug_risk,
                volume_velocity,
                price_momentum,
                holder_health,
                buy_count,
                sell_count,
                unique_buyers,
                unique_sellers
            ],
        )?;
        Ok(())
    }

    pub fn get_recent_momentum(&self, mint: &str, seconds: i64) -> Result<Vec<MomentumSnapshot>> {
        let cutoff = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64
            - seconds;

        let mut stmt = self.conn.prepare(
            "SELECT timestamp, score, rug_risk, volume_velocity, price_momentum, holder_health,
                    buy_count, sell_count, unique_buyers, unique_sellers
             FROM momentum_snapshots
             WHERE mint = ?1 AND timestamp > ?2
             ORDER BY timestamp ASC"
        )?;

        let snapshots = stmt.query_map(params![mint, cutoff], |row| {
            Ok(MomentumSnapshot {
                timestamp: row.get(0)?,
                score: row.get(1)?,
                rug_risk: row.get(2)?,
                volume_velocity: row.get(3)?,
                price_momentum: row.get(4)?,
                holder_health: row.get(5)?,
                buy_count: row.get(6)?,
                sell_count: row.get(7)?,
                unique_buyers: row.get(8)?,
                unique_sellers: row.get(9)?,
            })
        })?
        .collect::<SqlResult<Vec<_>>>()?;

        Ok(snapshots)
    }
}

#[derive(Debug, Clone)]
pub struct Position {
    pub mint: String,
    pub entry_signature: String,
    pub entry_time: i64,
    pub entry_sol_amount: f64,
    pub entry_token_amount: Option<f64>,
    pub entry_price: Option<f64>,
    pub current_token_amount: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct WhaleWallet {
    pub wallet_address: String,
    pub token_amount: f64,
    pub holdings_percent: f64,
    pub danger_level: String,
    pub last_check: i64,
}

#[derive(Debug, Clone)]
pub struct MomentumSnapshot {
    pub timestamp: i64,
    pub score: f64,
    pub rug_risk: f64,
    pub volume_velocity: f64,
    pub price_momentum: f64,
    pub holder_health: f64,
    pub buy_count: i32,
    pub sell_count: i32,
    pub unique_buyers: i32,
    pub unique_sellers: i32,
}
