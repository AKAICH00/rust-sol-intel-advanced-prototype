//! AI Provider Abstraction Layer
//!
//! Supports multiple AI providers with unified interface

pub mod deepseek;
pub mod claude;
pub mod openai;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// AI-assisted decision recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiDecision {
    pub action: DecisionAction,
    pub confidence: f64,  // 0.0-1.0
    pub reasoning: String,
    pub suggested_stops: Option<f64>,  // For trailing
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionAction {
    Hold,                           // Keep position, no action
    ExitFull,                       // Exit 100% immediately
    ExitPartial { percent: f64 },   // Exit X% (e.g., Rule #9 recovery)
    Trail { stop_percent: f64 },    // Activate trailing stop
    AdjustStop { new_stop: f64 },   // Modify existing stop
    Emergency,                      // Rug detected - exit NOW
}

/// Unified AI provider interface
#[async_trait::async_trait]
pub trait AiProvider: Send + Sync {
    /// Get AI decision based on context
    async fn get_decision(&self, context: &DecisionContext) -> Result<AiDecision>;

    /// Provider name
    fn name(&self) -> &str;

    /// Check if provider is available
    async fn health_check(&self) -> Result<bool>;
}

/// Context for AI decision-making
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionContext {
    pub mint: String,
    pub entry_sol: f64,
    pub current_sol: f64,
    pub profit_multiple: f64,
    pub time_elapsed: i64,  // seconds

    // Momentum signals
    pub momentum_score: f64,
    pub rug_risk: f64,
    pub volume_velocity: f64,
    pub price_momentum: f64,
    pub holder_health: f64,

    // Position state
    pub has_recovered_initial: bool,
    pub trailing_active: bool,
    pub current_stop: Option<f64>,

    // Trigger reason
    pub trigger_type: TriggerType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerType {
    ProfitTarget2x,              // Hit 2x profit
    HighMomentum,                // Strong upward momentum
    MomentumStalled,             // 60s timeout with low momentum
    HighRugRisk,                 // Rug detection triggered
    ConflictingSignals,          // E.g., 2x + high momentum
    TrailingStopHit,             // Stop loss triggered
    ManualReview,                // Human requested AI review
}

/// Factory for creating AI providers
pub struct AiProviderFactory;

impl AiProviderFactory {
    pub fn create(provider_type: &str, api_key: String) -> Result<Box<dyn AiProvider>> {
        match provider_type.to_lowercase().as_str() {
            "deepseek" => Ok(Box::new(deepseek::DeepSeekProvider::new(api_key))),
            "claude" => Ok(Box::new(claude::ClaudeProvider::new(api_key))),
            "openai" => Ok(Box::new(openai::OpenAiProvider::new(api_key))),
            _ => Err(anyhow::anyhow!("Unknown AI provider: {}", provider_type)),
        }
    }
}
