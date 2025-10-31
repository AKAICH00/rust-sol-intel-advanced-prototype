//! DeepSeek AI Provider (Primary)

use super::{AiProvider, AiDecision, DecisionContext, DecisionAction, TriggerType};
use anyhow::Result;
use log::{info, warn, error};
use serde::{Deserialize, Serialize};

pub struct DeepSeekProvider {
    api_key: String,
    client: reqwest::Client,
    base_url: String,
}

impl DeepSeekProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
            base_url: "https://api.deepseek.com/v1".to_string(),
        }
    }

    fn build_prompt(&self, context: &DecisionContext) -> String {
        let prompt_template = match context.trigger_type {
            TriggerType::ProfitTarget2x => include_str!("../../prompts/exit_2x.txt"),
            TriggerType::HighMomentum => include_str!("../../prompts/high_momentum.txt"),
            TriggerType::MomentumStalled => include_str!("../../prompts/momentum_stalled.txt"),
            TriggerType::HighRugRisk => include_str!("../../prompts/high_rug_risk.txt"),
            TriggerType::ConflictingSignals => include_str!("../../prompts/conflicting_signals.txt"),
            TriggerType::TrailingStopHit => include_str!("../../prompts/trailing_stop.txt"),
            TriggerType::ManualReview => include_str!("../../prompts/manual_review.txt"),
        };

        // Replace placeholders with actual data
        prompt_template
            .replace("{mint}", &context.mint)
            .replace("{entry_sol}", &format!("{:.4}", context.entry_sol))
            .replace("{current_sol}", &format!("{:.4}", context.current_sol))
            .replace("{profit_multiple}", &format!("{:.2}", context.profit_multiple))
            .replace("{time_elapsed}", &format!("{}", context.time_elapsed))
            .replace("{momentum_score}", &format!("{:.2}", context.momentum_score))
            .replace("{rug_risk}", &format!("{:.2}", context.rug_risk))
            .replace("{volume_velocity}", &format!("{:.2}", context.volume_velocity))
            .replace("{price_momentum}", &format!("{:.2}", context.price_momentum))
            .replace("{holder_health}", &format!("{:.2}", context.holder_health))
            .replace("{has_recovered}", &format!("{}", context.has_recovered_initial))
    }

    async fn call_api(&self, prompt: String) -> Result<DeepSeekResponse> {
        let request = DeepSeekRequest {
            model: "deepseek-chat".to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "You are a professional crypto trading assistant. Analyze the position data and provide a clear, actionable recommendation. Respond ONLY with valid JSON in this exact format: {\"action\":\"Hold|ExitFull|ExitPartial|Trail|Emergency\",\"confidence\":0.85,\"reasoning\":\"your reasoning\",\"exit_percent\":50.0}".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: prompt,
                },
            ],
            temperature: 0.3,  // Lower = more deterministic
            max_tokens: 500,
        };

        let response = self.client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?
            .json::<DeepSeekResponse>()
            .await?;

        Ok(response)
    }

    fn parse_decision(&self, response: DeepSeekResponse) -> Result<AiDecision> {
        let content = response.choices.first()
            .ok_or_else(|| anyhow::anyhow!("No response from DeepSeek"))?
            .message.content.clone();

        // Parse JSON response
        let parsed: serde_json::Value = serde_json::from_str(&content)?;

        let action_str = parsed["action"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing action field"))?;

        let action = match action_str {
            "Hold" => DecisionAction::Hold,
            "ExitFull" => DecisionAction::ExitFull,
            "ExitPartial" => {
                let percent = parsed["exit_percent"].as_f64()
                    .ok_or_else(|| anyhow::anyhow!("Missing exit_percent"))?;
                DecisionAction::ExitPartial { percent }
            }
            "Trail" => {
                let stop = parsed["stop_percent"].as_f64().unwrap_or(5.0);
                DecisionAction::Trail { stop_percent: stop }
            }
            "Emergency" => DecisionAction::Emergency,
            _ => DecisionAction::Hold,
        };

        let confidence = parsed["confidence"].as_f64().unwrap_or(0.5);
        let reasoning = parsed["reasoning"].as_str().unwrap_or("No reasoning provided").to_string();

        Ok(AiDecision {
            action,
            confidence,
            reasoning,
            suggested_stops: parsed["suggested_stop"].as_f64(),
            timestamp: chrono::Utc::now().timestamp(),
        })
    }
}

#[async_trait::async_trait]
impl AiProvider for DeepSeekProvider {
    async fn get_decision(&self, context: &DecisionContext) -> Result<AiDecision> {
        info!("🧠 DeepSeek analyzing position: {}", context.mint);
        info!("   Trigger: {:?}", context.trigger_type);
        info!("   P&L: {:.2}x | Momentum: {:.2} | Rug Risk: {:.2}",
            context.profit_multiple, context.momentum_score, context.rug_risk);

        // Build prompt
        let prompt = self.build_prompt(context);

        // Call API
        let response = self.call_api(prompt).await?;

        // Parse decision
        let decision = self.parse_decision(response)?;

        info!("✅ DeepSeek Decision: {:?} (confidence: {:.2})",
            decision.action, decision.confidence);
        info!("   Reasoning: {}", decision.reasoning);

        Ok(decision)
    }

    fn name(&self) -> &str {
        "DeepSeek"
    }

    async fn health_check(&self) -> Result<bool> {
        // Simple ping to verify API key works
        let request = DeepSeekRequest {
            model: "deepseek-chat".to_string(),
            messages: vec![
                Message {
                    role: "user".to_string(),
                    content: "ping".to_string(),
                },
            ],
            temperature: 0.3,
            max_tokens: 10,
        };

        match self.client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
        {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

#[derive(Debug, Serialize)]
struct DeepSeekRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct DeepSeekResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}
