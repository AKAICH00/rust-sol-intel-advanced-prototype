//! Transaction Verification
//!
//! Verifies transactions actually exist on-chain (PumpPortal has false positives)

use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, signature::Signature};
use log::{info, warn, error};
use std::str::FromStr;
use crate::database::Database;

pub struct TransactionVerifier {
    rpc: RpcClient,
    db: Database,
}

impl TransactionVerifier {
    pub fn new(rpc_url: String, db: Database) -> Self {
        Self {
            rpc: RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed()),
            db,
        }
    }

    /// Verify a transaction exists on-chain
    pub async fn verify_transaction(&self, signature_str: &str) -> Result<bool> {
        let signature = Signature::from_str(signature_str)?;

        match self.rpc.get_transaction(&signature, solana_transaction_status::UiTransactionEncoding::Json).await {
            Ok(_) => {
                info!("✅ Transaction verified: {}", signature_str);
                self.db.mark_transaction_verified(signature_str, true)?;
                Ok(true)
            }
            Err(e) => {
                warn!("❌ Transaction NOT found on-chain: {} - {}", signature_str, e);
                self.db.mark_transaction_verified(signature_str, false)?;
                Ok(false)
            }
        }
    }

    /// Verify with retries (transaction may not be confirmed yet)
    pub async fn verify_with_retries(&self, signature_str: &str, max_retries: u32, delay_ms: u64) -> Result<bool> {
        for attempt in 1..=max_retries {
            match self.verify_transaction(signature_str).await {
                Ok(true) => return Ok(true),
                Ok(false) if attempt < max_retries => {
                    info!("Retry {}/{} for {}", attempt, max_retries, signature_str);
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                }
                Ok(false) => return Ok(false),
                Err(e) => {
                    error!("Verification error: {}", e);
                    if attempt < max_retries {
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                    }
                }
            }
        }

        Ok(false)
    }
}
