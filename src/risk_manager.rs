use crate::types::{Signal, TickData};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// Risk configuration with position limits and stop-loss rules
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RiskConfig {
    // Position limits
    pub max_position_size_usd: f64,        // Maximum position size in USD
    pub max_position_pct_portfolio: f64,   // Max % of portfolio per position (0.0-1.0)
    pub max_leverage: f64,                 // Maximum leverage (1.0 = no leverage)

    // Stop-loss rules
    pub hard_stop_loss_pct: f64,           // Hard stop-loss % (e.g., 0.05 = 5%)
    pub trailing_stop_loss_pct: f64,       // Trailing stop % from peak
    pub portfolio_stop_loss_pct: f64,      // Daily portfolio stop-loss %

    // Diversification limits
    pub max_correlated_positions: usize,   // Max number of correlated positions
    pub max_total_positions: usize,        // Max simultaneous positions

    // Volatility-based sizing
    pub vol_target: f64,                   // Target volatility per trade (e.g., 0.02 = 2%)
    pub vol_lookback_periods: usize,       // Periods for volatility calculation

    // Drawdown controls
    pub max_daily_drawdown_pct: f64,       // Max daily drawdown before halt
    pub max_weekly_drawdown_pct: f64,      // Max weekly drawdown
    pub cooldown_after_loss_streak: usize, // Number of losses before cooldown
    pub cooldown_duration_minutes: u64,    // Cooldown duration

    // Kelly Criterion settings
    pub kelly_fraction: f64,               // Fraction of Kelly to use (0.25 = quarter Kelly)
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            max_position_size_usd: 1000.0,
            max_position_pct_portfolio: 0.20,
            max_leverage: 1.0,
            hard_stop_loss_pct: 0.05,
            trailing_stop_loss_pct: 0.03,
            portfolio_stop_loss_pct: 0.15,
            max_correlated_positions: 3,
            max_total_positions: 5,
            vol_target: 0.02,
            vol_lookback_periods: 20,
            max_daily_drawdown_pct: 0.15,
            max_weekly_drawdown_pct: 0.25,
            cooldown_after_loss_streak: 3,
            cooldown_duration_minutes: 60,
            kelly_fraction: 0.25,
        }
    }
}

/// Position tracking
#[derive(Clone, Debug)]
pub struct Position {
    pub symbol: String,
    pub entry_price: f64,
    pub current_price: f64,
    pub size: f64,              // Position size in USD
    pub entry_time: Instant,
    pub peak_price: f64,        // For trailing stop
    pub trailing_stop: f64,     // Current trailing stop level
    pub unrealized_pnl: f64,
    pub unrealized_pnl_pct: f64,
}

impl Position {
    pub fn new(symbol: String, entry_price: f64, size: f64) -> Self {
        Self {
            symbol,
            entry_price,
            current_price: entry_price,
            size,
            entry_time: Instant::now(),
            peak_price: entry_price,
            trailing_stop: entry_price * 0.97, // Initial 3% trailing stop
            unrealized_pnl: 0.0,
            unrealized_pnl_pct: 0.0,
        }
    }

    pub fn update_price(&mut self, price: f64) {
        self.current_price = price;
        self.unrealized_pnl = (price - self.entry_price) * (self.size / self.entry_price);
        self.unrealized_pnl_pct = (price - self.entry_price) / self.entry_price;

        // Update peak and trailing stop
        if price > self.peak_price {
            self.peak_price = price;
        }
    }

    pub fn update_trailing_stop(&mut self, trailing_pct: f64) {
        self.trailing_stop = self.peak_price * (1.0 - trailing_pct);
    }
}

/// Portfolio state tracking
#[derive(Clone, Debug)]
pub struct Portfolio {
    pub starting_capital: f64,
    pub current_capital: f64,
    pub available_capital: f64,
    pub total_pnl: f64,
    pub daily_pnl: f64,
    pub weekly_pnl: f64,
    pub consecutive_losses: usize,
    pub consecutive_wins: usize,
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub day_start_capital: f64,
    pub week_start_capital: f64,
    pub day_start_time: Instant,
    pub week_start_time: Instant,
    pub last_loss_time: Option<Instant>,
    pub peak_capital: f64,
}

impl Portfolio {
    pub fn new(starting_capital: f64) -> Self {
        Self {
            starting_capital,
            current_capital: starting_capital,
            available_capital: starting_capital,
            total_pnl: 0.0,
            daily_pnl: 0.0,
            weekly_pnl: 0.0,
            consecutive_losses: 0,
            consecutive_wins: 0,
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            day_start_capital: starting_capital,
            week_start_capital: starting_capital,
            day_start_time: Instant::now(),
            week_start_time: Instant::now(),
            last_loss_time: None,
            peak_capital: starting_capital,
        }
    }

    pub fn daily_pnl_pct(&self) -> f64 {
        self.daily_pnl / self.day_start_capital
    }

    pub fn weekly_pnl_pct(&self) -> f64 {
        self.weekly_pnl / self.week_start_capital
    }

    pub fn max_drawdown_pct(&self) -> f64 {
        (self.peak_capital - self.current_capital) / self.peak_capital
    }

    pub fn win_rate(&self) -> f64 {
        if self.total_trades == 0 {
            0.0
        } else {
            self.winning_trades as f64 / self.total_trades as f64
        }
    }

    pub fn reset_daily(&mut self) {
        self.day_start_capital = self.current_capital;
        self.daily_pnl = 0.0;
        self.day_start_time = Instant::now();
    }

    pub fn reset_weekly(&mut self) {
        self.week_start_capital = self.current_capital;
        self.weekly_pnl = 0.0;
        self.week_start_time = Instant::now();
    }
}

/// Risk management errors
#[derive(Debug)]
pub enum RiskError {
    MaxPositionsReached,
    DrawdownLimitExceeded,
    LossStreakCooldown,
    ExtremeVolatility,
    PositionSizeTooLarge,
    InsufficientCapital,
    HardStopTriggered,
    TrailingStopTriggered,
}

impl std::fmt::Display for RiskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskError::MaxPositionsReached => write!(f, "Maximum number of positions reached"),
            RiskError::DrawdownLimitExceeded => write!(f, "Drawdown limit exceeded"),
            RiskError::LossStreakCooldown => write!(f, "In cooldown after loss streak"),
            RiskError::ExtremeVolatility => write!(f, "Extreme volatility detected"),
            RiskError::PositionSizeTooLarge => write!(f, "Position size exceeds limits"),
            RiskError::InsufficientCapital => write!(f, "Insufficient capital available"),
            RiskError::HardStopTriggered => write!(f, "Hard stop-loss triggered"),
            RiskError::TrailingStopTriggered => write!(f, "Trailing stop-loss triggered"),
        }
    }
}

impl std::error::Error for RiskError {}

/// Main risk management system
pub struct RiskManager {
    pub config: RiskConfig,
    pub portfolio: Portfolio,
    pub positions: HashMap<String, Position>,
    pub volatility_cache: HashMap<String, f64>,
}

impl RiskManager {
    pub fn new(config: RiskConfig, starting_capital: f64) -> Self {
        info!("Initializing RiskManager with capital: ${}", starting_capital);
        Self {
            config,
            portfolio: Portfolio::new(starting_capital),
            positions: HashMap::new(),
            volatility_cache: HashMap::new(),
        }
    }

    /// Calculate optimal position size using Kelly Criterion with volatility scaling
    pub fn calculate_position_size(
        &self,
        signal: &Signal,
        estimated_volatility: f64,
    ) -> Result<f64> {
        let win_rate = signal.confidence as f64;

        // For simplicity, assume win/loss ratio is 1.5:1 (adjust based on backtesting)
        let win_loss_ratio = 1.5;

        // Kelly formula: f* = (p * b - q) / b
        // where p = win prob, q = 1-p, b = win/loss ratio
        let kelly = (win_rate * win_loss_ratio - (1.0 - win_rate)) / win_loss_ratio;
        let kelly = kelly.max(0.0); // No negative positions

        // Apply fraction of Kelly (quarter Kelly by default)
        let fractional_kelly = kelly * self.config.kelly_fraction;

        // Volatility scaling: reduce size in high volatility
        let vol_scalar = if estimated_volatility > 0.0 {
            (self.config.vol_target / estimated_volatility).min(1.0)
        } else {
            1.0
        };

        // Calculate base size
        let available = self.portfolio.available_capital;
        let base_size = available * fractional_kelly * vol_scalar;

        // Apply hard limits
        let max_pct_size = available * self.config.max_position_pct_portfolio;
        let max_abs_size = self.config.max_position_size_usd;

        let final_size = base_size.min(max_pct_size).min(max_abs_size);

        info!(
            "Position sizing: Kelly={:.3}, Vol_scalar={:.3}, Base=${:.2}, Final=${:.2}",
            fractional_kelly, vol_scalar, base_size, final_size
        );

        Ok(final_size)
    }

    /// Validate if a trade should be allowed
    pub fn validate_trade(
        &self,
        signal: &Signal,
        size: f64,
        estimated_volatility: f64,
    ) -> Result<(), RiskError> {
        // Check position count limit
        if self.positions.len() >= self.config.max_total_positions {
            warn!("Trade rejected: max positions reached ({}/{})",
                  self.positions.len(), self.config.max_total_positions);
            return Err(RiskError::MaxPositionsReached);
        }

        // Check daily drawdown
        let daily_dd = self.portfolio.daily_pnl_pct();
        if daily_dd < -self.config.max_daily_drawdown_pct {
            warn!("Trade rejected: daily drawdown {:.2}% exceeds limit {:.2}%",
                  daily_dd * 100.0, self.config.max_daily_drawdown_pct * 100.0);
            return Err(RiskError::DrawdownLimitExceeded);
        }

        // Check weekly drawdown
        let weekly_dd = self.portfolio.weekly_pnl_pct();
        if weekly_dd < -self.config.max_weekly_drawdown_pct {
            warn!("Trade rejected: weekly drawdown {:.2}% exceeds limit {:.2}%",
                  weekly_dd * 100.0, self.config.max_weekly_drawdown_pct * 100.0);
            return Err(RiskError::DrawdownLimitExceeded);
        }

        // Check loss streak cooldown
        if self.portfolio.consecutive_losses >= self.config.cooldown_after_loss_streak {
            if let Some(last_loss) = self.portfolio.last_loss_time {
                let cooldown = Duration::from_secs(self.config.cooldown_duration_minutes * 60);
                if last_loss.elapsed() < cooldown {
                    warn!("Trade rejected: in cooldown after {} consecutive losses",
                          self.portfolio.consecutive_losses);
                    return Err(RiskError::LossStreakCooldown);
                }
            }
        }

        // Check volatility regime (don't trade in extreme volatility)
        if estimated_volatility > 0.50 {
            warn!("Trade rejected: extreme volatility {:.1}%", estimated_volatility * 100.0);
            return Err(RiskError::ExtremeVolatility);
        }

        // Check position size
        if size > self.portfolio.available_capital {
            warn!("Trade rejected: insufficient capital (need ${:.2}, have ${:.2})",
                  size, self.portfolio.available_capital);
            return Err(RiskError::InsufficientCapital);
        }

        if size > self.config.max_position_size_usd {
            warn!("Trade rejected: size ${:.2} exceeds max ${:.2}",
                  size, self.config.max_position_size_usd);
            return Err(RiskError::PositionSizeTooLarge);
        }

        info!("Trade validation passed: size=${:.2}, confidence={:.3}, vol={:.3}",
              size, signal.confidence, estimated_volatility);

        Ok(())
    }

    /// Open a new position
    pub fn open_position(&mut self, symbol: String, entry_price: f64, size: f64) -> Result<()> {
        let position = Position::new(symbol.clone(), entry_price, size);

        // Update portfolio
        self.portfolio.available_capital -= size;

        // Store position
        self.positions.insert(symbol.clone(), position);

        info!("Opened position: {} at ${:.4}, size=${:.2}", symbol, entry_price, size);

        Ok(())
    }

    /// Update all positions with current prices and check stops
    pub fn update_positions(&mut self, current_prices: &HashMap<String, f64>) -> Vec<(String, String)> {
        let mut stops_triggered = Vec::new();

        for (symbol, position) in self.positions.iter_mut() {
            if let Some(&price) = current_prices.get(symbol) {
                position.update_price(price);
                position.update_trailing_stop(self.config.trailing_stop_loss_pct);

                // Check hard stop-loss
                if position.unrealized_pnl_pct < -self.config.hard_stop_loss_pct {
                    warn!("{}: Hard stop triggered at {:.2}% loss",
                          symbol, position.unrealized_pnl_pct * 100.0);
                    stops_triggered.push((symbol.clone(), "hard_stop".to_string()));
                }

                // Check trailing stop-loss
                else if price < position.trailing_stop {
                    warn!("{}: Trailing stop triggered (price=${:.4} < stop=${:.4})",
                          symbol, price, position.trailing_stop);
                    stops_triggered.push((symbol.clone(), "trailing_stop".to_string()));
                }
            }
        }

        stops_triggered
    }

    /// Close a position
    pub fn close_position(&mut self, symbol: &str, exit_price: f64, reason: &str) -> Result<f64> {
        let position = self.positions.remove(symbol)
            .ok_or_else(|| anyhow!("Position not found: {}", symbol))?;

        // Calculate realized P&L
        let pnl = (exit_price - position.entry_price) * (position.size / position.entry_price);
        let pnl_pct = (exit_price - position.entry_price) / position.entry_price;

        // Update portfolio
        self.portfolio.available_capital += position.size + pnl;
        self.portfolio.current_capital += pnl;
        self.portfolio.total_pnl += pnl;
        self.portfolio.daily_pnl += pnl;
        self.portfolio.weekly_pnl += pnl;
        self.portfolio.total_trades += 1;

        // Update win/loss tracking
        if pnl > 0.0 {
            self.portfolio.winning_trades += 1;
            self.portfolio.consecutive_wins += 1;
            self.portfolio.consecutive_losses = 0;
        } else {
            self.portfolio.losing_trades += 1;
            self.portfolio.consecutive_losses += 1;
            self.portfolio.consecutive_wins = 0;
            self.portfolio.last_loss_time = Some(Instant::now());
        }

        // Update peak capital
        if self.portfolio.current_capital > self.portfolio.peak_capital {
            self.portfolio.peak_capital = self.portfolio.current_capital;
        }

        info!(
            "Closed position: {} at ${:.4}, P&L=${:.2} ({:.2}%), Reason: {}",
            symbol, exit_price, pnl, pnl_pct * 100.0, reason
        );

        Ok(pnl)
    }

    /// Get current portfolio metrics
    pub fn get_metrics(&self) -> RiskMetrics {
        let total_position_value: f64 = self.positions.values()
            .map(|p| p.current_price * (p.size / p.entry_price))
            .sum();

        let unrealized_pnl: f64 = self.positions.values()
            .map(|p| p.unrealized_pnl)
            .sum();

        let win_rate = self.portfolio.win_rate();

        // Simple Sharpe estimate (would need time series for accurate calculation)
        let sharpe_estimate = if self.portfolio.total_trades > 10 {
            let avg_return = self.portfolio.total_pnl / self.portfolio.total_trades as f64;
            let volatility = 0.02; // Placeholder, should calculate from actual returns
            if volatility > 0.0 {
                (avg_return / self.portfolio.starting_capital) / volatility
            } else {
                0.0
            }
        } else {
            0.0
        };

        RiskMetrics {
            total_capital: self.portfolio.current_capital,
            available_capital: self.portfolio.available_capital,
            total_position_value,
            unrealized_pnl,
            realized_pnl: self.portfolio.total_pnl,
            daily_pnl: self.portfolio.daily_pnl,
            daily_pnl_pct: self.portfolio.daily_pnl_pct(),
            weekly_pnl: self.portfolio.weekly_pnl,
            weekly_pnl_pct: self.portfolio.weekly_pnl_pct(),
            max_drawdown_pct: self.portfolio.max_drawdown_pct(),
            num_positions: self.positions.len(),
            total_trades: self.portfolio.total_trades,
            win_rate,
            sharpe_estimate,
            consecutive_losses: self.portfolio.consecutive_losses,
            consecutive_wins: self.portfolio.consecutive_wins,
        }
    }

    /// Calculate rolling volatility from recent price data
    pub fn calculate_volatility(&mut self, symbol: &str, recent_prices: &[f64]) -> f64 {
        if recent_prices.len() < 2 {
            return 0.02; // Default 2% volatility
        }

        // Calculate log returns
        let mut returns = Vec::new();
        for i in 1..recent_prices.len() {
            let ret = (recent_prices[i] / recent_prices[i - 1]).ln();
            returns.push(ret);
        }

        // Calculate standard deviation
        let mean: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance: f64 = returns.iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / returns.len() as f64;
        let volatility = variance.sqrt();

        // Cache for later use
        self.volatility_cache.insert(symbol.to_string(), volatility);

        volatility
    }
}

/// Risk metrics for monitoring
#[derive(Debug, Clone, Serialize)]
pub struct RiskMetrics {
    pub total_capital: f64,
    pub available_capital: f64,
    pub total_position_value: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub daily_pnl: f64,
    pub daily_pnl_pct: f64,
    pub weekly_pnl: f64,
    pub weekly_pnl_pct: f64,
    pub max_drawdown_pct: f64,
    pub num_positions: usize,
    pub total_trades: usize,
    pub win_rate: f64,
    pub sharpe_estimate: f64,
    pub consecutive_losses: usize,
    pub consecutive_wins: usize,
}
