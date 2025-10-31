//! Type definitions for the PumpPortal API

use serde::{Deserialize, Serialize};

/// Trading action type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TradeAction {
    /// Buy tokens
    Buy,
    /// Sell tokens
    Sell,
}

/// Pool/Exchange options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Pool {
    /// Pump.fun exchange
    Pump,
    /// Raydium exchange
    Raydium,
    /// Pump AMM
    PumpAmm,
    /// LaunchLab
    Launchlab,
    /// Raydium CPMM
    RaydiumCpmm,
    /// Bonk
    Bonk,
    /// Auto-select best pool
    Auto,
}

impl Default for Pool {
    fn default() -> Self {
        Pool::Pump
    }
}

/// Trade request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeRequest {
    /// Action: "buy" or "sell"
    pub action: TradeAction,

    /// Token contract address (mint)
    pub mint: String,

    /// Amount in SOL or tokens
    pub amount: String,

    /// Whether amount is denominated in SOL
    #[serde(serialize_with = "serialize_bool_as_string")]
    pub denominated_in_sol: bool,

    /// Slippage percentage (e.g., 10 for 10%)
    pub slippage: u32,

    /// Priority fee for faster transactions
    pub priority_fee: f64,

    /// Pool/Exchange to use (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pool: Option<Pool>,

    /// Skip preflight simulation
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "serialize_optional_bool_as_string")]
    pub skip_preflight: Option<bool>,

    /// Route only through Jito
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "serialize_optional_bool_as_string")]
    pub jito_only: Option<bool>,
}

/// Helper function to serialize bool as string
fn serialize_bool_as_string<S>(value: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(if *value { "true" } else { "false" })
}

/// Helper function to serialize Option<bool> as Option<String>
fn serialize_optional_bool_as_string<S>(
    value: &Option<bool>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match value {
        Some(v) => serializer.serialize_str(if *v { "true" } else { "false" }),
        None => serializer.serialize_none(),
    }
}

/// Trade response from the API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeResponse {
    /// Transaction signature if successful
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,

    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Additional response fields (API may include extra data)
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

impl TradeRequest {
    /// Create a new buy request
    pub fn buy(mint: String, sol_amount: f64, slippage: u32, priority_fee: f64) -> Self {
        Self {
            action: TradeAction::Buy,
            mint,
            amount: sol_amount.to_string(),
            denominated_in_sol: true,
            slippage,
            priority_fee,
            pool: None,
            skip_preflight: Some(true),
            jito_only: None,
        }
    }

    /// Create a new sell request
    pub fn sell(mint: String, token_amount: String, slippage: u32, priority_fee: f64) -> Self {
        Self {
            action: TradeAction::Sell,
            mint,
            amount: token_amount,
            denominated_in_sol: false,
            slippage,
            priority_fee,
            pool: None,
            skip_preflight: Some(true),
            jito_only: None,
        }
    }

    /// Set the pool/exchange to use
    pub fn with_pool(mut self, pool: Pool) -> Self {
        self.pool = Some(pool);
        self
    }

    /// Enable/disable preflight simulation
    pub fn with_skip_preflight(mut self, skip: bool) -> Self {
        self.skip_preflight = Some(skip);
        self
    }

    /// Enable/disable Jito-only routing
    pub fn with_jito_only(mut self, jito: bool) -> Self {
        self.jito_only = Some(jito);
        self
    }
}
