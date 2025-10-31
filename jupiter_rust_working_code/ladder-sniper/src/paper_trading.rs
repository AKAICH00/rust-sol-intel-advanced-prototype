use anyhow::Result;
use log::info;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use crate::data_export::{DataExporter, TradeRecord, PositionRecord, get_timestamp_micros};

#[derive(Debug, Clone)]
pub struct PaperTradingConfig {
    pub enabled: bool,
    pub starting_balance: f64,
    pub buy_latency_ms: u64,
    pub sell_latency_ms: u64,
    pub trade_fee_percent: f64,
    pub priority_fee_sol: f64,
}

impl PaperTradingConfig {
    pub fn from_env() -> Self {
        let enabled = std::env::var("PAPER_MODE")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        let starting_balance = std::env::var("PAPER_STARTING_BALANCE")
            .unwrap_or_else(|_| "1.0".to_string())
            .parse::<f64>()
            .unwrap_or(1.0);

        let buy_latency_ms = std::env::var("PAPER_BUY_LATENCY_MS")
            .unwrap_or_else(|_| "700".to_string())
            .parse::<u64>()
            .unwrap_or(700);

        let sell_latency_ms = std::env::var("PAPER_SELL_LATENCY_MS")
            .unwrap_or_else(|_| "500".to_string())
            .parse::<u64>()
            .unwrap_or(500);

        let trade_fee_percent = std::env::var("PAPER_TRADE_FEE_PERCENT")
            .unwrap_or_else(|_| "1.0".to_string())
            .parse::<f64>()
            .unwrap_or(1.0);

        let priority_fee_sol = std::env::var("PAPER_PRIORITY_FEE_SOL")
            .unwrap_or_else(|_| "0.0001".to_string())
            .parse::<f64>()
            .unwrap_or(0.0001);

        Self {
            enabled,
            starting_balance,
            buy_latency_ms,
            sell_latency_ms,
            trade_fee_percent,
            priority_fee_sol,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PaperPosition {
    pub mint: String,
    pub amount_sol: f64,
    pub tokens: f64,
    pub entry_price: f64,
}

#[derive(Debug)]
pub struct PaperWallet {
    balance: f64,
    positions: std::collections::HashMap<String, PaperPosition>,
    total_fees_paid: f64,
    total_trades: u32,
    winning_trades: u32,
    losing_trades: u32,
}

impl PaperWallet {
    pub fn new(starting_balance: f64) -> Self {
        Self {
            balance: starting_balance,
            positions: std::collections::HashMap::new(),
            total_fees_paid: 0.0,
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
        }
    }

    pub fn balance(&self) -> f64 {
        self.balance
    }

    pub fn has_position(&self, mint: &str) -> bool {
        self.positions.contains_key(mint)
    }

    pub fn get_position(&self, mint: &str) -> Option<&PaperPosition> {
        self.positions.get(mint)
    }

    pub fn open_positions(&self) -> usize {
        self.positions.len()
    }

    pub fn stats(&self) -> (u32, u32, u32, f64) {
        (self.total_trades, self.winning_trades, self.losing_trades, self.total_fees_paid)
    }

    pub fn buy(&mut self, mint: String, sol_amount: f64, price: f64, fee_percent: f64, priority_fee: f64) -> Result<f64> {
        // Calculate fees
        let trade_fee = sol_amount * (fee_percent / 100.0);
        let total_cost = sol_amount + trade_fee + priority_fee;

        if self.balance < total_cost {
            anyhow::bail!("Insufficient balance: {} SOL < {} SOL", self.balance, total_cost);
        }

        // Calculate tokens received (after fee)
        let net_sol = sol_amount - trade_fee;
        let tokens = net_sol / price;

        // Deduct from balance
        self.balance -= total_cost;
        self.total_fees_paid += trade_fee + priority_fee;

        // Record position
        self.positions.insert(mint.clone(), PaperPosition {
            mint,
            amount_sol: sol_amount,
            tokens,
            entry_price: price,
        });

        self.total_trades += 1;

        Ok(tokens)
    }

    pub fn sell(&mut self, mint: &str, price: f64, fee_percent: f64, priority_fee: f64) -> Result<(f64, f64)> {
        let position = self.positions.get(mint)
            .ok_or_else(|| anyhow::anyhow!("No position found for {}", mint))?
            .clone();

        // Calculate SOL received
        let gross_sol = position.tokens * price;
        let trade_fee = gross_sol * (fee_percent / 100.0);
        let net_sol = gross_sol - trade_fee - priority_fee;

        // Calculate P&L
        let pnl = net_sol - position.amount_sol;
        let pnl_percent = (pnl / position.amount_sol) * 100.0;

        // Update balance
        self.balance += net_sol;
        self.total_fees_paid += trade_fee + priority_fee;

        // Track win/loss
        if pnl > 0.0 {
            self.winning_trades += 1;
        } else {
            self.losing_trades += 1;
        }

        self.total_trades += 1;

        // Remove position
        self.positions.remove(mint);

        Ok((net_sol, pnl_percent))
    }
}

pub type SharedPaperWallet = Arc<Mutex<PaperWallet>>;
pub type SharedExporter = Arc<Mutex<DataExporter>>;

pub struct PaperTradingSimulator {
    config: PaperTradingConfig,
    wallet: SharedPaperWallet,
    exporter: Option<SharedExporter>,
}

impl PaperTradingSimulator {
    pub fn new(config: PaperTradingConfig, exporter: Option<SharedExporter>) -> Self {
        let wallet = Arc::new(Mutex::new(PaperWallet::new(config.starting_balance)));
        Self { config, wallet, exporter }
    }

    pub fn wallet(&self) -> SharedPaperWallet {
        self.wallet.clone()
    }

    /// Simulate a buy order with realistic latency and fees
    pub async fn simulate_buy(&self, mint: String, sol_amount: f64, price: f64) -> Result<String> {
        // Simulate network latency
        sleep(Duration::from_millis(self.config.buy_latency_ms)).await;

        // Execute trade
        let mut wallet = self.wallet.lock().await;
        let tokens = wallet.buy(
            mint.clone(),
            sol_amount,
            price,
            self.config.trade_fee_percent,
            self.config.priority_fee_sol,
        )?;

        let balance = wallet.balance();
        drop(wallet);

        // Generate fake signature
        let signature = format!("PAPER_{}", uuid::Uuid::new_v4().to_string()[0..32].to_string());

        // Record trade and position in database
        if let Some(exporter) = &self.exporter {
            let fee_sol = sol_amount * (self.config.trade_fee_percent / 100.0);
            let timestamp = get_timestamp_micros();

            let trade_record = TradeRecord {
                trade_id: signature.clone(),
                timestamp_micros: timestamp,
                mint: mint.clone(),
                trade_type: "BUY".to_string(),
                price,
                sol_amount,
                tokens,
                fee_sol,
                priority_fee_sol: self.config.priority_fee_sol,
                balance_after: balance,
                signature: signature.clone(),
            };

            // Also record position open
            let position_record = PositionRecord {
                position_id: mint.clone(), // Use mint as position ID
                mint: mint.clone(),
                entry_time_micros: timestamp,
                exit_time_micros: None,
                entry_price: price,
                exit_price: None,
                sol_invested: sol_amount,
                sol_returned: None,
                tokens,
                pnl_sol: None,
                pnl_percent: None,
                hold_duration_secs: None,
                holder_count_entry: 0, // Will be updated by main.rs
                holder_count_exit: None,
                exit_reason: None,
                profits_taken: false,
            };

            let mut exp = exporter.lock().await;
            let _ = exp.record_trade(trade_record);
            let _ = exp.record_position(position_record);
        }

        info!("📝 PAPER BUY:");
        info!("   Mint: {}", &mint[0..8]);
        info!("   Spent: {} SOL", sol_amount);
        info!("   Price: {} SOL/token", price);
        info!("   Tokens: {}", tokens);
        info!("   Fee: {}%", self.config.trade_fee_percent);
        info!("   Remaining: {} SOL", balance);

        Ok(signature)
    }

    /// Simulate a sell order with realistic latency and fees
    pub async fn simulate_sell(&self, mint: &str, price: f64, exit_reason: Option<String>) -> Result<String> {
        // Simulate network latency
        sleep(Duration::from_millis(self.config.sell_latency_ms)).await;

        // Get position info before selling
        let wallet_ref = self.wallet.lock().await;
        let position_info = wallet_ref.get_position(mint).cloned();
        drop(wallet_ref);

        // Execute trade
        let mut wallet = self.wallet.lock().await;
        let (net_sol, pnl_percent) = wallet.sell(
            mint,
            price,
            self.config.trade_fee_percent,
            self.config.priority_fee_sol,
        )?;

        let balance = wallet.balance();
        let (total, wins, losses, fees) = wallet.stats();
        drop(wallet);

        // Generate fake signature
        let signature = format!("PAPER_{}", uuid::Uuid::new_v4().to_string()[0..32].to_string());

        // Record trade and update position in database
        if let Some(exporter) = &self.exporter {
            let fee_sol = net_sol * (self.config.trade_fee_percent / 100.0);
            let timestamp = get_timestamp_micros();

            let trade_record = TradeRecord {
                trade_id: signature.clone(),
                timestamp_micros: timestamp,
                mint: mint.to_string(),
                trade_type: "SELL".to_string(),
                price,
                sol_amount: net_sol,
                tokens: 0.0, // Sold all tokens
                fee_sol,
                priority_fee_sol: self.config.priority_fee_sol,
                balance_after: balance,
                signature: signature.clone(),
            };

            // Update position to closed if we have position info
            if let Some(pos) = position_info {
                let pnl_sol = net_sol - pos.amount_sol;
                let position_record = PositionRecord {
                    position_id: mint.to_string(),
                    mint: mint.to_string(),
                    entry_time_micros: 0, // Will use existing from DB
                    exit_time_micros: Some(timestamp),
                    entry_price: pos.entry_price,
                    exit_price: Some(price),
                    sol_invested: pos.amount_sol,
                    sol_returned: Some(net_sol),
                    tokens: pos.tokens,
                    pnl_sol: Some(pnl_sol),
                    pnl_percent: Some(pnl_percent),
                    hold_duration_secs: None, // Will be calculated by DB update
                    holder_count_entry: 0,
                    holder_count_exit: None,
                    exit_reason,
                    profits_taken: false,
                };

                let mut exp = exporter.lock().await;
                let _ = exp.record_trade(trade_record);
                let _ = exp.record_position(position_record);
            } else {
                let mut exp = exporter.lock().await;
                let _ = exp.record_trade(trade_record);
            }
        }

        let pnl_emoji = if pnl_percent > 0.0 { "📈" } else { "📉" };

        info!("📝 PAPER SELL:");
        info!("   Mint: {}", &mint[0..8]);
        info!("   Received: {} SOL", net_sol);
        info!("   Price: {} SOL/token", price);
        info!("   {} P&L: {:+.1}%", pnl_emoji, pnl_percent);
        info!("   Balance: {} SOL", balance);
        info!("   Stats: {} trades | {}W {}L | {:.4} SOL fees", total, wins, losses, fees);

        Ok(signature)
    }

    pub async fn print_summary(&self) {
        let wallet = self.wallet.lock().await;
        let (total, wins, losses, fees) = wallet.stats();
        let balance = wallet.balance();
        let starting = self.config.starting_balance;
        let pnl = balance - starting;
        let pnl_percent = (pnl / starting) * 100.0;
        let win_rate = if total > 0 { (wins as f64 / total as f64) * 100.0 } else { 0.0 };

        info!("");
        info!("📊 PAPER TRADING SUMMARY");
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("   Starting: {} SOL", starting);
        info!("   Current:  {} SOL", balance);
        info!("   P&L:      {:+.4} SOL ({:+.1}%)", pnl, pnl_percent);
        info!("   Trades:   {} total | {} wins | {} losses", total, wins, losses);
        info!("   Win Rate: {:.1}%", win_rate);
        info!("   Fees:     {:.4} SOL", fees);
        info!("   Open:     {} positions", wallet.open_positions());
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }
}
