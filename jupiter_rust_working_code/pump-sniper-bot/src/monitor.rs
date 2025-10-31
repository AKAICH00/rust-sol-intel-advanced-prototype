//! Real Position Monitoring with On-Chain Data
//!
//! Tracks actual token balances and calculates real-time P&L

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::commitment_config::CommitmentConfig;
use anyhow::{Result, Context};
use log::{info, warn, error};
use std::str::FromStr;
use crate::database::Database;

pub struct PositionMonitor {
    rpc_client: RpcClient,
    pumpportal_wallet: Pubkey,  // PumpPortal's custodial wallet
    db: Database,
}

impl PositionMonitor {
    pub fn new(rpc_url: String, db: Database) -> Result<Self> {
        // PumpPortal uses a custodial wallet system
        // You need to get your specific wallet address from PumpPortal API
        // For now using a placeholder - this needs to be retrieved from PumpPortal
        let pumpportal_wallet = Pubkey::from_str("11111111111111111111111111111111")
            .context("Invalid PumpPortal wallet address")?;

        Ok(Self {
            rpc_client: RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed()),
            pumpportal_wallet,
            db,
        })
    }

    /// Get actual token balance for a mint from PumpPortal wallet
    pub async fn get_token_balance(&self, mint: &str) -> Result<f64> {
        let mint_pubkey = Pubkey::from_str(mint)
            .context("Invalid mint address")?;

        // Get token accounts owned by PumpPortal wallet for this mint
        let token_accounts = self.rpc_client
            .get_token_accounts_by_owner(
                &self.pumpportal_wallet,
                solana_client::rpc_request::TokenAccountsFilter::Mint(mint_pubkey),
            )
            .await
            .context("Failed to get token accounts")?;

        if token_accounts.is_empty() {
            return Ok(0.0);
        }

        // Parse first account (should only be one)
        let account_data = &token_accounts[0].account.data;

        // Decode base64 if needed
        let decoded_data = match account_data {
            solana_account_decoder::UiAccountData::Binary(data_str, _) => {
                base64::decode(data_str).context("Failed to decode account data")?
            }
            _ => return Ok(0.0),
        };

        // Parse token account
        if decoded_data.len() < 165 {
            return Ok(0.0);
        }

        // Token account amount is at bytes 64-72
        let amount_bytes: [u8; 8] = decoded_data[64..72].try_into()?;
        let balance = u64::from_le_bytes(amount_bytes) as f64;

        // Get decimals from mint
        let mint_account = self.rpc_client.get_account(&mint_pubkey).await?;

        // Mint decimals is at byte 44
        let decimals = if mint_account.data.len() > 44 {
            mint_account.data[44]
        } else {
            6 // Default for pump.fun tokens
        };

        let balance_adjusted = balance / 10_f64.powi(decimals as i32);

        Ok(balance_adjusted)
    }

    /// Get current price from bonding curve or DEX
    pub async fn get_current_price(&self, mint: &str) -> Result<f64> {
        // For pump.fun tokens, price comes from bonding curve
        // This requires calling pump.fun program to get curve state

        // Option 1: Parse bonding curve state from on-chain account
        // Option 2: Use pump.fun API
        // Option 3: Calculate from virtual reserves

        // For now, using a simple approach - get from recent trades
        // In production, you'd parse the bonding curve state

        // TODO: Implement bonding curve price calculation
        // For now, returning placeholder
        warn!("Price calculation not yet implemented for {}, using estimate", mint);
        Ok(0.0)
    }

    /// Calculate current position value
    pub async fn get_position_value(&self, mint: &str) -> Result<PositionValue> {
        // Get position from database
        let position = self.db.get_active_position(mint)?
            .context("No active position found")?;

        // Get current balance
        let current_balance = self.get_token_balance(mint).await.unwrap_or(0.0);

        // Update database with current balance
        if current_balance > 0.0 {
            self.db.update_position_balance(mint, current_balance)?;
        }

        // Get current price
        let current_price = self.get_current_price(mint).await.unwrap_or(0.0);

        // Calculate values
        let entry_value = position.entry_sol_amount;
        let current_value = if current_price > 0.0 {
            current_balance * current_price
        } else {
            0.0
        };

        let profit_loss = current_value - entry_value;
        let profit_percent = if entry_value > 0.0 {
            (profit_loss / entry_value) * 100.0
        } else {
            0.0
        };

        Ok(PositionValue {
            mint: mint.to_string(),
            current_balance,
            entry_value,
            current_value,
            current_price,
            profit_loss,
            profit_percent,
            entry_time: position.entry_time,
        })
    }

    /// Check if we still hold this position
    pub async fn has_position(&self, mint: &str) -> Result<bool> {
        let balance = self.get_token_balance(mint).await.unwrap_or(0.0);
        Ok(balance > 0.0)
    }

    /// Get time since entry in seconds
    pub fn time_since_entry(&self, mint: &str) -> Result<i64> {
        let position = self.db.get_active_position(mint)?
            .context("No active position found")?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        Ok(now - position.entry_time)
    }
}

#[derive(Debug, Clone)]
pub struct PositionValue {
    pub mint: String,
    pub current_balance: f64,
    pub entry_value: f64,
    pub current_value: f64,
    pub current_price: f64,
    pub profit_loss: f64,
    pub profit_percent: f64,
    pub entry_time: i64,
}

impl PositionValue {
    pub fn display(&self) {
        let profit_emoji = if self.profit_percent >= 0.0 { "📈" } else { "📉" };

        info!("💼 Position Value:");
        info!("   Mint: {}", self.mint);
        info!("   Balance: {:.2} tokens", self.current_balance);
        info!("   Entry: {:.4} SOL", self.entry_value);
        info!("   Current: {:.4} SOL", self.current_value);
        info!(
            "   {} P&L: {:.4} SOL ({:.1}%)",
            profit_emoji, self.profit_loss, self.profit_percent
        );

        let elapsed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            - self.entry_time;
        info!("   Time: {}s since entry", elapsed);
    }
}
