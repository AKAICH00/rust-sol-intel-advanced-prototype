use crate::database::{Database, PositionRecord, TradeRecord};
use crate::risk_manager::{RiskError, RiskManager};
use crate::types::Signal;
use anyhow::{anyhow, Result};
use chrono::Utc;
use jup_ag_sdk::types::{QuoteRequest, SwapRequest};
use jup_ag_sdk::JupiterClient;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    transaction::Transaction,
};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;
use tracing::{error, info, warn};

/// Execution engine for memecoin trading via Jupiter
pub struct ExecutionEngine {
    jupiter: JupiterClient,
    rpc_client: Arc<RpcClient>,
    wallet: Arc<Keypair>,
    risk_manager: Arc<tokio::sync::Mutex<RiskManager>>,
    database: Database,
    config: ExecutionConfig,
}

#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    pub max_slippage_bps: u64,        // Max slippage in basis points (e.g., 50 = 0.5%)
    pub max_price_impact_bps: u64,    // Max price impact (e.g., 100 = 1%)
    pub priority_fee_lamports: u64,   // Priority fee for transactions
    pub sol_mint: String,             // SOL mint address
    pub confirmation_timeout_sec: u64, // Transaction confirmation timeout
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            max_slippage_bps: 50,                    // 0.5% slippage
            max_price_impact_bps: 100,               // 1% price impact
            priority_fee_lamports: 5000,             // 0.000005 SOL priority fee
            sol_mint: "So11111111111111111111111111111111111111112".to_string(),
            confirmation_timeout_sec: 60,
        }
    }
}

impl ExecutionEngine {
    pub fn new(
        jupiter_url: String,
        rpc_url: String,
        wallet: Keypair,
        risk_manager: Arc<tokio::sync::Mutex<RiskManager>>,
        database: Database,
        config: ExecutionConfig,
    ) -> Self {
        let jupiter = JupiterClient::new(&jupiter_url);
        let rpc_client = Arc::new(RpcClient::new_with_commitment(
            rpc_url,
            CommitmentConfig::confirmed(),
        ));

        Self {
            jupiter,
            rpc_client,
            wallet: Arc::new(wallet),
            risk_manager,
            database,
            config,
        }
    }

    /// Execute a buy order based on ML signal
    pub async fn execute_buy(
        &self,
        signal: &Signal,
        symbol: &str,
        mint_address: &str,
    ) -> Result<ExecutionResult> {
        let start_time = Instant::now();
        info!("🎯 Executing BUY: {} ({})", symbol, mint_address);

        // 1. Calculate volatility for position sizing
        let volatility = 0.02; // TODO: Get from feature buffer

        // 2. Calculate position size via RiskManager
        let mut rm = self.risk_manager.lock().await;
        let size_usd = rm.calculate_position_size(signal, volatility)?;

        // 3. Validate trade
        rm.validate_trade(signal, size_usd, volatility)
            .map_err(|e| anyhow!("Risk validation failed: {}", e))?;

        // Get current SOL balance
        let sol_balance = self.get_sol_balance().await?;
        let sol_to_spend = (size_usd / sol_balance) * sol_balance; // Simplified price calculation

        info!("   Size: ${:.2} (~{:.4} SOL)", size_usd, sol_to_spend);

        // 4. Get quote from Jupiter
        let amount_lamports = (sol_to_spend * 1_000_000_000.0) as u64;
        let quote_req = QuoteRequest::new(
            &self.config.sol_mint,
            mint_address,
            amount_lamports,
        )
        .slippage_bps(self.config.max_slippage_bps);

        let quote = self.jupiter.get_quote(&quote_req).await
            .map_err(|e| anyhow!("Jupiter quote failed: {:?}", e))?;

        info!("   Quote: {} SOL → {} tokens",
              quote.in_amount as f64 / 1e9,
              quote.out_amount);

        // 5. Get swap transaction
        let user_pubkey = self.wallet.pubkey().to_string();
        let swap_req = SwapRequest::new(&user_pubkey, &quote);

        let swap_response = self.jupiter.get_swap_transaction(&swap_req).await
            .map_err(|e| anyhow!("Jupiter swap request failed: {:?}", e))?;

        // 6. Sign and send transaction
        let signature = self.sign_and_send_transaction(&swap_response.swap_transaction).await?;

        let execution_time_ms = start_time.elapsed().as_millis() as i64;
        info!("   ✅ BUY EXECUTED: {} ({:.0}ms)", signature, execution_time_ms);

        // 7. Calculate entry details
        let entry_price = quote.in_amount as f64 / quote.out_amount as f64;
        let actual_slippage = 0.0; // TODO: Calculate actual vs expected

        // 8. Record position in risk manager
        let position_size_usd = size_usd;
        rm.open_position(symbol.to_string(), entry_price, position_size_usd)?;
        drop(rm); // Release lock

        // 9. Record position in database
        let position_record = PositionRecord {
            symbol: symbol.to_string(),
            mint_address: Some(mint_address.to_string()),
            entry_price,
            current_price: entry_price,
            size_usd: position_size_usd,
            entry_time: Utc::now(),
            peak_price: entry_price,
            trailing_stop: entry_price * 0.97, // Initial 3% trailing stop
            unrealized_pnl: 0.0,
            unrealized_pnl_pct: 0.0,
            confidence_score: signal.confidence,
            volatility,
        };

        let position_id = self.database.insert_position(&position_record)?;

        // 10. Record trade execution
        let trade_record = TradeRecord {
            position_id: Some(position_id),
            trade_type: "buy".to_string(),
            symbol: symbol.to_string(),
            price: entry_price,
            size_usd: position_size_usd,
            timestamp: Utc::now(),
            signature: Some(signature.to_string()),
            slippage_bps: Some(actual_slippage),
            fees_usd: Some(quote.price_impact_pct as f64 * position_size_usd),
            execution_time_ms: Some(execution_time_ms),
        };

        self.database.insert_trade(&trade_record)?;

        Ok(ExecutionResult {
            signature,
            entry_price,
            amount: quote.out_amount as f64,
            size_usd: position_size_usd,
            slippage_bps: actual_slippage,
            execution_time_ms,
            position_id,
        })
    }

    /// Execute a sell order
    pub async fn execute_sell(
        &self,
        position_id: i64,
        symbol: &str,
        mint_address: &str,
        sell_amount: f64,
        exit_reason: &str,
    ) -> Result<ExecutionResult> {
        let start_time = Instant::now();
        info!("💰 Executing SELL: {} ({:.0} tokens)", symbol, sell_amount);

        // 1. Get quote from Jupiter (sell tokens for SOL)
        let amount_lamports = sell_amount as u64;
        let quote_req = QuoteRequest::new(
            mint_address,
            &self.config.sol_mint,
            amount_lamports,
        )
        .slippage_bps(self.config.max_slippage_bps);

        let quote = self.jupiter.get_quote(&quote_req).await
            .map_err(|e| anyhow!("Jupiter quote failed: {:?}", e))?;

        info!("   Quote: {} tokens → {:.4} SOL",
              quote.in_amount,
              quote.out_amount as f64 / 1e9);

        // 2. Get swap transaction
        let user_pubkey = self.wallet.pubkey().to_string();
        let swap_req = SwapRequest::new(&user_pubkey, &quote);

        let swap_response = self.jupiter.get_swap_transaction(&swap_req).await
            .map_err(|e| anyhow!("Jupiter swap request failed: {:?}", e))?;

        // 3. Sign and send transaction
        let signature = self.sign_and_send_transaction(&swap_response.swap_transaction).await?;

        let execution_time_ms = start_time.elapsed().as_millis() as i64;
        info!("   ✅ SELL EXECUTED: {} ({:.0}ms)", signature, execution_time_ms);

        // 4. Calculate exit details
        let exit_price = quote.out_amount as f64 / quote.in_amount as f64;
        let sol_received = quote.out_amount as f64 / 1e9;

        // 5. Close position in risk manager
        let mut rm = self.risk_manager.lock().await;
        let realized_pnl = rm.close_position(symbol, exit_price, exit_reason)?;
        drop(rm);

        // 6. Update database
        let realized_pnl_pct = 0.0; // TODO: Calculate from entry price
        self.database.close_position(position_id, exit_price, realized_pnl, realized_pnl_pct, exit_reason)?;

        // 7. Record trade
        let trade_record = TradeRecord {
            position_id: Some(position_id),
            trade_type: "sell".to_string(),
            symbol: symbol.to_string(),
            price: exit_price,
            size_usd: sol_received * 100.0, // Approximate USD value
            timestamp: Utc::now(),
            signature: Some(signature.to_string()),
            slippage_bps: None,
            fees_usd: Some(quote.price_impact_pct as f64 * sol_received),
            execution_time_ms: Some(execution_time_ms),
        };

        self.database.insert_trade(&trade_record)?;

        Ok(ExecutionResult {
            signature,
            entry_price: exit_price,
            amount: quote.out_amount as f64,
            size_usd: sol_received * 100.0,
            slippage_bps: 0.0,
            execution_time_ms,
            position_id,
        })
    }

    /// Sign and send a transaction
    async fn sign_and_send_transaction(&self, tx_b64: &str) -> Result<Signature> {
        // Decode base64 transaction
        let tx_bytes = base64::decode(tx_b64)
            .map_err(|e| anyhow!("Failed to decode transaction: {}", e))?;

        // Deserialize transaction
        let mut transaction: Transaction = bincode::deserialize(&tx_bytes)
            .map_err(|e| anyhow!("Failed to deserialize transaction: {}", e))?;

        // Sign transaction
        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;
        transaction.sign(&[&*self.wallet], recent_blockhash);

        // Send transaction
        let signature = self.rpc_client
            .send_and_confirm_transaction(&transaction)
            .map_err(|e| anyhow!("Transaction failed: {}", e))?;

        Ok(signature)
    }

    /// Get current SOL balance
    async fn get_sol_balance(&self) -> Result<f64> {
        let balance = self.rpc_client
            .get_balance(&self.wallet.pubkey())
            .map_err(|e| anyhow!("Failed to get balance: {}", e))?;

        Ok(balance as f64 / 1_000_000_000.0)
    }

    /// Get token balance for a specific mint
    pub async fn get_token_balance(&self, mint_address: &str) -> Result<f64> {
        let mint_pubkey = Pubkey::from_str(mint_address)?;

        // Get token accounts for this mint
        let token_accounts = self.rpc_client
            .get_token_accounts_by_owner(
                &self.wallet.pubkey(),
                solana_client::rpc_request::TokenAccountsFilter::Mint(mint_pubkey),
            )
            .map_err(|e| anyhow!("Failed to get token accounts: {}", e))?;

        if token_accounts.is_empty() {
            return Ok(0.0);
        }

        // Parse balance from first account
        // TODO: Properly parse token account data
        Ok(0.0)
    }
}

/// Execution result
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub signature: Signature,
    pub entry_price: f64,
    pub amount: f64,
    pub size_usd: f64,
    pub slippage_bps: f64,
    pub execution_time_ms: i64,
    pub position_id: i64,
}

/// Stub function for compatibility with existing code
pub async fn execute_trade(signal: Signal) -> Result<()> {
    info!("[Execution] Received signal with confidence {:.3}", signal.confidence);
    info!("[Execution] Note: Use ExecutionEngine for real trading");
    Ok(())
}
