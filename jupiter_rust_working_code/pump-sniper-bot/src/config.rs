//! Configuration from Sniper Rules

use std::env;

#[derive(Debug, Clone)]
pub struct SniperConfig {
    // Rule #1: Gas Management
    pub gas_reserve_sol: f64,

    // Rule #2: Position Limits
    pub max_positions: usize,

    // Rule #3: Capital Allocation (0 = use 100% available)
    pub snipe_amount_sol: f64,

    // Rule #4: Jito preload
    pub use_jito: bool,

    // Rule #5: Target criteria
    pub filter_nsfw: bool,
    pub filter_no_metadata: bool,

    // Rule #9: Profit mechanics
    pub profit_target_multiplier: f64,
    pub recovery_percent: f64,

    // APIs
    pub pumpportal_api_key: String,
    pub helius_rpc_url: String,
    pub database_path: String,
}

impl SniperConfig {
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            gas_reserve_sol: env::var("GAS_RESERVE_SOL")
                .unwrap_or_else(|_| "0.01".to_string())
                .parse()
                .map_err(|_| "Invalid GAS_RESERVE_SOL")?,

            max_positions: env::var("MAX_POSITIONS")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .map_err(|_| "Invalid MAX_POSITIONS")?,

            snipe_amount_sol: env::var("SNIPE_AMOUNT_SOL")
                .unwrap_or_else(|_| "0".to_string())
                .parse()
                .map_err(|_| "Invalid SNIPE_AMOUNT_SOL")?,

            use_jito: env::var("USE_JITO")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),

            filter_nsfw: env::var("FILTER_NSFW")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),

            filter_no_metadata: env::var("FILTER_NO_METADATA")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),

            profit_target_multiplier: env::var("PROFIT_TARGET_MULTIPLIER")
                .unwrap_or_else(|_| "2.0".to_string())
                .parse()
                .map_err(|_| "Invalid PROFIT_TARGET_MULTIPLIER")?,

            recovery_percent: env::var("RECOVERY_PERCENT")
                .unwrap_or_else(|_| "110.0".to_string())
                .parse()
                .map_err(|_| "Invalid RECOVERY_PERCENT")?,

            pumpportal_api_key: env::var("PUMPPORTAL_API_KEY")
                .map_err(|_| "PUMPPORTAL_API_KEY not set")?,

            helius_rpc_url: env::var("HELIUS_RPC_URL")
                .map_err(|_| "HELIUS_RPC_URL not set")?,

            database_path: env::var("DATABASE_PATH")
                .unwrap_or_else(|_| "sniper_bot.db".to_string()),
        })
    }

    /// Calculate snipe amount based on rules
    pub fn calculate_snipe_amount(&self, wallet_balance: f64, active_positions: usize) -> f64 {
        // Rule #1: Always reserve gas
        let available = (wallet_balance - self.gas_reserve_sol).max(0.0);

        if available < 0.001 {
            warn!("⚠️ Balance too low: {} SOL (need {} + gas)", wallet_balance, 0.001);
            return 0.0;
        }

        // Rule #3: Use configured amount or 100% of available
        if self.snipe_amount_sol > 0.0 {
            // Fixed amount mode
            self.snipe_amount_sol.min(available)
        } else {
            // Rule #3: 100% deployment
            // Divide by remaining slots to maintain Rule #2
            let remaining_slots = (self.max_positions - active_positions).max(1);
            available / remaining_slots as f64
        }
    }

    pub fn display(&self) {
        info!("⚙️  SNIPER RULES CONFIG");
        info!("   Rule #1 - Gas Reserve: {} SOL", self.gas_reserve_sol);
        info!("   Rule #2 - Max Positions: {}", self.max_positions);
        info!("   Rule #3 - Capital Mode: {}",
            if self.snipe_amount_sol > 0.0 {
                format!("Fixed {} SOL", self.snipe_amount_sol)
            } else {
                "100% Available".to_string()
            }
        );
        info!("   Rule #4 - Jito: {}", if self.use_jito { "ON" } else { "OFF" });
        info!("   Rule #5 - Filters: {}",
            if !self.filter_nsfw && !self.filter_no_metadata {
                "NONE (Speed > Selectivity)"
            } else {
                "ACTIVE"
            }
        );
        info!("   Rule #9 - Profit Target: {}x → Recover {}%",
            self.profit_target_multiplier,
            self.recovery_percent
        );
    }
}

use log::{info, warn};
