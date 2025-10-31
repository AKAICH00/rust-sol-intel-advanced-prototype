//! Sniper Bot Strategy Implementation
//!
//! Strategy:
//! 1. Fast entry on new launches (PumpPortal for speed)
//! 2. Monitor momentum in first 30-60 seconds
//! 3. Exit fast if no momentum
//! 4. At 2x: recover initial + 10%, trail the rest
//! 5. Ladder out on way up, keep moon bag

use pump_portal_sdk::{PumpPortalClient, TradeRequest};
use crate::monitor::PositionMonitor;
use crate::detector::MomentumDetector;
use crate::launch_detector::{LaunchDetector, LaunchDetectorConfig};
use anyhow::Result;
use log::{info, warn, error};
use std::time::Duration;

pub struct SniperBot {
    pumpportal: PumpPortalClient,
    monitor: PositionMonitor,
    detector: MomentumDetector,
    trade_amount: f64,
}

impl SniperBot {
    pub fn new(api_key: String, rpc_url: String, trade_amount: f64, db: crate::database::Database) -> Result<Self> {
        Ok(Self {
            pumpportal: PumpPortalClient::new(api_key),
            monitor: PositionMonitor::new(rpc_url.clone(), db.clone())?,
            detector: MomentumDetector::new(rpc_url, db)?,
            trade_amount,
        })
    }

    pub async fn run(self) -> Result<()> {
        info!("🎯 Strategy: Fast In, Smart Exit");
        info!("   Entry: ~$5 per launch");
        info!("   Exit: No momentum → Fast exit");
        info!("   Exit: 2x reached → Recover + 10%, trail rest");
        info!("   Exit: High momentum → Ladder out, keep moon bag\n");

        // Start monitoring for new launches
        self.monitor_launches().await
    }

    async fn monitor_launches(&self) -> Result<()> {
        // Initialize launch detector with PumpPortal WebSocket
        let detector = LaunchDetector::new(LaunchDetectorConfig::default());
        let mut launch_rx = detector.start_monitoring().await?;

        info!("✅ Launch detector running, waiting for new tokens...\n");

        // Process new token launches
        while let Some(launch) = launch_rx.recv().await {
            info!("🎯 New snipeable token detected: {} ({})", launch.name, launch.symbol);

            // Execute snipe
            match self.execute_snipe(&launch.mint).await {
                Ok(signature) => {
                    // Start position management
                    if let Err(e) = self.manage_position(&launch.mint, &signature).await {
                        error!("Position management failed: {}", e);
                    }
                }
                Err(e) => {
                    error!("Snipe failed for {}: {}", launch.mint, e);
                }
            }

            info!("\n👀 Monitoring for next launch...\n");
        }

        Ok(())
    }

    /// Execute snipe on new token
    pub async fn execute_snipe(&self, token_mint: &str) -> Result<String> {
        info!("⚡ SNIPING: {}", token_mint);

        // Use aggressive settings for speed
        let request = TradeRequest::buy(
            token_mint.to_string(),
            self.trade_amount,
            20, // High slippage for launch volatility
            0.0005, // Higher priority fee for speed
        )
        .with_jito_only(true); // Jito for best execution

        match self.pumpportal.trade(request).await {
            Ok(response) => {
                if let Some(sig) = response.signature {
                    info!("✅ SNIPE EXECUTED: {}", sig);
                    info!("   🔗 https://solscan.io/tx/{}", sig);

                    // Verify transaction actually exists
                    tokio::time::sleep(Duration::from_secs(2)).await;

                    // TODO: Verify on-chain

                    Ok(sig)
                } else {
                    error!("❌ Snipe failed: No signature");
                    Err(anyhow::anyhow!("No signature returned"))
                }
            }
            Err(e) => {
                error!("❌ Snipe error: {}", e);
                Err(e.into())
            }
        }
    }

    /// Monitor position and execute exit strategy
    pub async fn manage_position(
        &self,
        token_mint: &str,
        entry_signature: &str,
    ) -> Result<()> {
        info!("📊 Managing position for {}", token_mint);

        let mut check_count = 0;
        let max_no_momentum_checks = 6; // 60 seconds of no momentum = exit

        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;
            check_count += 1;

            // Check momentum
            let momentum = self.detector.check_momentum(token_mint).await?;

            info!("📈 Check #{}: Momentum = {:.1}%", check_count, momentum.score * 100.0);

            // RULE 1: No momentum after 60 seconds = fast exit
            if check_count >= max_no_momentum_checks && momentum.score < 0.3 {
                warn!("⚠️  NO MOMENTUM DETECTED - Fast exit!");
                return self.execute_exit(token_mint, "100%", "no_momentum").await;
            }

            // RULE 2: Check for 2x (or current profit)
            let current_value = self.monitor.get_position_value(token_mint).await?;
            let profit_multiple = current_value / self.trade_amount;

            info!("   Current: {:.3} SOL ({:.2}x)", current_value, profit_multiple);

            if profit_multiple >= 2.0 {
                info!("🎯 2X REACHED! Recovering initial + 10%");

                // Sell portion to recover initial + 10%
                let recovery_amount = self.trade_amount * 1.1;
                let recovery_percent = (recovery_amount / current_value) * 100.0;

                self.execute_exit(
                    token_mint,
                    &format!("{:.0}%", recovery_percent),
                    "recover_initial"
                ).await?;

                info!("💰 Recovered {:.3} SOL", recovery_amount);
                info!("🚀 Trailing the rest with high momentum");

                // Now trail the rest
                return self.trail_position(token_mint, current_value - recovery_amount).await;
            }

            // RULE 3: Rug pull detection
            if momentum.rug_risk > 0.7 {
                error!("🚨 RUG PULL DETECTED! Emergency exit!");
                return self.execute_exit(token_mint, "100%", "rug_detected").await;
            }

            // RULE 4: High momentum detected - prepare for ladder
            if momentum.score > 0.8 && profit_multiple > 1.5 {
                info!("🚀 HIGH MOMENTUM + PROFIT - Starting ladder strategy");
                return self.ladder_exit(token_mint, current_value).await;
            }
        }
    }

    /// Trail position with tight stops
    async fn trail_position(&self, token_mint: &str, initial_value: f64) -> Result<()> {
        info!("📈 TRAILING POSITION");

        let mut highest_value = initial_value;
        let trailing_stop_percent = 0.85; // Sell if drops 15% from high

        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;

            let current_value = self.monitor.get_position_value(token_mint).await?;

            if current_value > highest_value {
                highest_value = current_value;
                info!("   New high: {:.3} SOL", highest_value);
            }

            let drop_percent = current_value / highest_value;

            if drop_percent < trailing_stop_percent {
                warn!("⚠️  Trailing stop hit! Exiting remaining position");
                return self.execute_exit(token_mint, "100%", "trailing_stop").await;
            }

            // Check for rug
            let momentum = self.detector.check_momentum(token_mint).await?;
            if momentum.rug_risk > 0.7 {
                error!("🚨 RUG DETECTED during trail! Exit now!");
                return self.execute_exit(token_mint, "100%", "rug_detected").await;
            }
        }
    }

    /// Ladder out on the way up
    async fn ladder_exit(&self, token_mint: &str, current_value: f64) -> Result<()> {
        info!("🪜 LADDER EXIT STRATEGY");

        let ladder_steps = vec![
            (3.0, 25.0, "3x"),   // At 3x, sell 25%
            (5.0, 30.0, "5x"),   // At 5x, sell 30%
            (10.0, 30.0, "10x"), // At 10x, sell 30%
            (20.0, 10.0, "20x"), // At 20x, sell 10%
            // Keep 5% as moon bag
        ];

        let mut remaining_percent = 100.0;

        for (target_multiple, sell_percent, label) in ladder_steps {
            loop {
                tokio::time::sleep(Duration::from_secs(5)).await;

                let current = self.monitor.get_position_value(token_mint).await?;
                let multiple = current / self.trade_amount;

                if multiple >= target_multiple {
                    info!("🎯 {} REACHED! Selling {:.0}%", label, sell_percent);

                    self.execute_exit(
                        token_mint,
                        &format!("{:.0}%", sell_percent),
                        &format!("ladder_{}", label)
                    ).await?;

                    remaining_percent -= sell_percent;
                    info!("   Remaining: {:.0}%", remaining_percent);
                    break;
                }

                // Rug check
                let momentum = self.detector.check_momentum(token_mint).await?;
                if momentum.rug_risk > 0.7 {
                    error!("🚨 RUG! Selling remaining {:.0}%", remaining_percent);
                    return self.execute_exit(
                        token_mint,
                        &format!("{:.0}%", remaining_percent),
                        "rug_detected"
                    ).await;
                }
            }
        }

        info!("🌙 Keeping {:.0}% as moon bag!", remaining_percent);
        Ok(())
    }

    /// Execute exit
    async fn execute_exit(
        &self,
        token_mint: &str,
        amount: &str,
        reason: &str,
    ) -> Result<()> {
        info!("🔴 EXITING: {} ({})", amount, reason);

        let request = TradeRequest::sell(
            token_mint.to_string(),
            amount.to_string(),
            20, // High slippage for fast exit
            0.0005,
        )
        .with_jito_only(true);

        match self.pumpportal.trade(request).await {
            Ok(response) => {
                if let Some(sig) = response.signature {
                    info!("✅ EXIT EXECUTED: {}", sig);
                    info!("   🔗 https://solscan.io/tx/{}", sig);
                    Ok(())
                } else {
                    error!("❌ Exit failed: No signature");
                    Err(anyhow::anyhow!("Exit failed"))
                }
            }
            Err(e) => {
                error!("❌ Exit error: {}", e);
                Err(e.into())
            }
        }
    }
}
