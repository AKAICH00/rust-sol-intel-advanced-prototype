//! Claude AI Provider (Future)

use super::{AiProvider, AiDecision, DecisionContext, DecisionAction};
use anyhow::Result;

pub struct ClaudeProvider {
    api_key: String,
}

impl ClaudeProvider {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

#[async_trait::async_trait]
impl AiProvider for ClaudeProvider {
    async fn get_decision(&self, _context: &DecisionContext) -> Result<AiDecision> {
        // TODO: Implement Claude API integration
        // Use: https://api.anthropic.com/v1/messages
        Err(anyhow::anyhow!("Claude provider not yet implemented"))
    }

    fn name(&self) -> &str {
        "Claude"
    }

    async fn health_check(&self) -> Result<bool> {
        // TODO: Implement health check
        Ok(false)
    }
}
