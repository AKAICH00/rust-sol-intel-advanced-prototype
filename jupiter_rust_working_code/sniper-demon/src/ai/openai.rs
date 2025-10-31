//! OpenAI Provider (Future)

use super::{AiProvider, AiDecision, DecisionContext, DecisionAction};
use anyhow::Result;

pub struct OpenAiProvider {
    api_key: String,
}

impl OpenAiProvider {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

#[async_trait::async_trait]
impl AiProvider for OpenAiProvider {
    async fn get_decision(&self, _context: &DecisionContext) -> Result<AiDecision> {
        // TODO: Implement OpenAI API integration
        // Use: https://api.openai.com/v1/chat/completions
        Err(anyhow::anyhow!("OpenAI provider not yet implemented"))
    }

    fn name(&self) -> &str {
        "OpenAI"
    }

    async fn health_check(&self) -> Result<bool> {
        // TODO: Implement health check
        Ok(false)
    }
}
