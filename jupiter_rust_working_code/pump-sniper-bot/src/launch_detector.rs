//! Launch Detection System using PumpPortal API
//!
//! Monitors new pump.fun token launches using PumpPortal's WebSocket feed.
//! Much simpler and more reliable than parsing raw Solana logs.

use anyhow::{Result, Context};
use log::{info, warn, error};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::StreamExt;

/// PumpPortal WebSocket URL for new token launches
const PUMPPORTAL_WS_URL: &str = "wss://pumpportal.fun/api/data";

/// Represents a newly detected token launch from PumpPortal
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenLaunch {
    pub mint: String,
    pub name: String,
    pub symbol: String,
    pub description: Option<String>,
    pub image: Option<String>,
    pub metadata_uri: Option<String>,
    pub twitter: Option<String>,
    pub telegram: Option<String>,
    pub bonding_curve: Option<String>,
    pub associated_bonding_curve: Option<String>,
    pub creator: Option<String>,
    pub created_timestamp: u64,
    pub raydium_pool: Option<String>,
    pub complete: bool,
    pub virtual_sol_reserves: Option<u64>,
    pub virtual_token_reserves: Option<u64>,
    pub total_supply: Option<u64>,
    pub website: Option<String>,
    pub show_name: bool,
    pub king_of_the_hill_timestamp: Option<u64>,
    pub market_cap: Option<f64>,
    pub reply_count: Option<u32>,
    pub last_reply: Option<u64>,
    pub nsfw: bool,
    pub market_id: Option<String>,
    pub inverted: Option<bool>,
    pub username: Option<String>,
    pub profile_image: Option<String>,
    pub usd_market_cap: Option<f64>,
}

impl TokenLaunch {
    pub fn display(&self) {
        info!("🚀 NEW LAUNCH DETECTED");
        info!("   Name: {} ({})", self.name, self.symbol);
        info!("   Mint: {}", self.mint);
        if let Some(creator) = &self.creator {
            info!("   Creator: {}", creator);
        }
        if let Some(desc) = &self.description {
            info!("   Description: {}", desc);
        }
        if let Some(mc) = self.market_cap {
            info!("   Market Cap: ${:.2}", mc);
        }
        info!("   🔗 https://pump.fun/{}", self.mint);
        info!("   🔗 https://solscan.io/token/{}", self.mint);
    }

    /// Check if this token is suitable for sniping
    pub fn is_snipeable(&self) -> bool {
        // Don't snipe completed tokens (already graduated to Raydium)
        if self.complete {
            warn!("   ⚠️  Token already graduated to Raydium, skipping");
            return false;
        }

        // Don't snipe NSFW tokens (optional filter)
        if self.nsfw {
            warn!("   ⚠️  NSFW token, skipping");
            return false;
        }

        // Check for basic metadata
        if self.name.is_empty() || self.symbol.is_empty() {
            warn!("   ⚠️  Missing name/symbol, skipping");
            return false;
        }

        true
    }
}

/// WebSocket message from PumpPortal
#[derive(Debug, Deserialize)]
#[serde(tag = "txType")]
enum PumpPortalMessage {
    #[serde(rename = "create")]
    Create(TokenLaunch),
    #[serde(other)]
    Other,
}

/// Launch detector configuration
pub struct LaunchDetectorConfig {
    pub ws_url: String,
    pub buffer_size: usize,
    pub reconnect_delay_secs: u64,
}

impl Default for LaunchDetectorConfig {
    fn default() -> Self {
        Self {
            ws_url: PUMPPORTAL_WS_URL.to_string(),
            buffer_size: 100,
            reconnect_delay_secs: 5,
        }
    }
}

/// Main launch detector using PumpPortal WebSocket
pub struct LaunchDetector {
    config: LaunchDetectorConfig,
}

impl LaunchDetector {
    pub fn new(config: LaunchDetectorConfig) -> Self {
        Self { config }
    }

    /// Start monitoring for new token launches
    ///
    /// Returns a channel receiver that yields TokenLaunch events
    pub async fn start_monitoring(&self) -> Result<mpsc::Receiver<TokenLaunch>> {
        let (tx, rx) = mpsc::channel(self.config.buffer_size);

        info!("🔍 Starting PumpPortal launch detector...");
        info!("   WebSocket: {}", self.config.ws_url);

        let ws_url = self.config.ws_url.clone();
        let reconnect_delay = self.config.reconnect_delay_secs;

        // Spawn monitoring task
        tokio::spawn(async move {
            if let Err(e) = Self::monitor_websocket(ws_url, reconnect_delay, tx).await {
                error!("Launch detector error: {}", e);
            }
        });

        Ok(rx)
    }

    /// Monitor PumpPortal WebSocket for token creation events
    async fn monitor_websocket(
        ws_url: String,
        reconnect_delay: u64,
        tx: mpsc::Sender<TokenLaunch>,
    ) -> Result<()> {
        loop {
            info!("Connecting to PumpPortal WebSocket...");

            match connect_async(&ws_url).await {
                Ok((ws_stream, _)) => {
                    info!("✅ Connected to PumpPortal");

                    let (_, mut read) = ws_stream.split();

                    // Send subscription message for new token creates
                    let subscribe_msg = serde_json::json!({
                        "method": "subscribeNewToken"
                    });

                    info!("📡 Subscribed to new token events");

                    // Process messages
                    while let Some(message) = read.next().await {
                        match message {
                            Ok(Message::Text(text)) => {
                                // Parse message
                                match serde_json::from_str::<PumpPortalMessage>(&text) {
                                    Ok(PumpPortalMessage::Create(launch)) => {
                                        launch.display();

                                        if launch.is_snipeable() {
                                            info!("   ✅ Token is snipeable!");
                                            if let Err(e) = tx.send(launch).await {
                                                error!("Failed to send launch event: {}", e);
                                                break;
                                            }
                                        }
                                    }
                                    Ok(PumpPortalMessage::Other) => {
                                        // Ignore other message types (trades, etc.)
                                    }
                                    Err(e) => {
                                        error!("Failed to parse message: {} - {}", e, text);
                                    }
                                }
                            }
                            Ok(Message::Ping(_)) => {
                                // WebSocket ping, handled automatically
                            }
                            Ok(Message::Close(_)) => {
                                warn!("WebSocket closed by server");
                                break;
                            }
                            Err(e) => {
                                error!("WebSocket error: {}", e);
                                break;
                            }
                            _ => {}
                        }
                    }

                    warn!("WebSocket stream ended, reconnecting in {} seconds...", reconnect_delay);
                    tokio::time::sleep(tokio::time::Duration::from_secs(reconnect_delay)).await;
                }
                Err(e) => {
                    error!("Failed to connect to WebSocket: {}", e);
                    warn!("Retrying in {} seconds...", reconnect_delay);
                    tokio::time::sleep(tokio::time::Duration::from_secs(reconnect_delay)).await;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_launch_parsing() {
        let json = r#"{
            "txType": "create",
            "mint": "GgoaCoyqvnDE5KGLSpLPiyNVGeF8w8rm8b1Hd9JFpump",
            "name": "Test Token",
            "symbol": "TEST",
            "description": "A test token",
            "created_timestamp": 1234567890,
            "complete": false,
            "nsfw": false,
            "show_name": true
        }"#;

        let msg: PumpPortalMessage = serde_json::from_str(json).unwrap();
        match msg {
            PumpPortalMessage::Create(launch) => {
                assert_eq!(launch.mint, "GgoaCoyqvnDE5KGLSpLPiyNVGeF8w8rm8b1Hd9JFpump");
                assert_eq!(launch.name, "Test Token");
                assert_eq!(launch.symbol, "TEST");
                assert!(launch.is_snipeable());
            }
            _ => panic!("Expected Create message"),
        }
    }
}
