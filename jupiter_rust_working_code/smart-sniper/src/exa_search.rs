use anyhow::Result;
use log::{info, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct SocialBuzz {
    pub twitter_mentions: u32,
    pub telegram_activity: bool,
    pub community_sentiment: f64, // 0.0 = negative, 1.0 = positive
    pub buzz_score: f64,          // 0.0-1.0
}

#[derive(Debug, Serialize)]
struct ExaSearchRequest {
    query: String,
    #[serde(rename = "type")]
    search_type: String,
    num_results: u32,
    text: bool,
}

#[derive(Debug, Deserialize)]
struct ExaSearchResponse {
    results: Vec<ExaResult>,
}

#[derive(Debug, Deserialize)]
struct ExaResult {
    title: Option<String>,
    url: Option<String>,
    text: Option<String>,
}

/// Search web for token mentions using Exa AI
pub async fn search_token_buzz(
    token_name: &str,
    token_symbol: &str,
    api_key: &str,
) -> Result<SocialBuzz> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    info!("   🔎 Searching web for: {} ({})", token_name, token_symbol);

    // Search Twitter mentions
    let twitter_query = format!(
        "{} {} crypto token solana pump.fun",
        token_name, token_symbol
    );

    let search_request = ExaSearchRequest {
        query: twitter_query,
        search_type: "neural".to_string(),
        num_results: 10,
        text: true,
    };

    let response = client
        .post("https://api.exa.ai/search")
        .header("x-api-key", api_key)
        .header("Content-Type", "application/json")
        .json(&search_request)
        .send()
        .await;

    let mut twitter_mentions = 0u32;
    let mut telegram_activity = false;
    let mut positive_mentions = 0u32;
    let mut negative_mentions = 0u32;

    match response {
        Ok(resp) => {
            if let Ok(search_results) = resp.json::<ExaSearchResponse>().await {
                twitter_mentions = search_results.results.len() as u32;

                // Analyze sentiment from results
                for result in &search_results.results {
                    if let Some(ref text) = result.text {
                        let text_lower = text.to_lowercase();

                        // Check for Telegram mentions
                        if text_lower.contains("t.me") || text_lower.contains("telegram") {
                            telegram_activity = true;
                        }

                        // Simple sentiment analysis
                        let positive_words = ["moon", "gem", "bullish", "buying", "amazing", "potential"];
                        let negative_words = ["scam", "rug", "dump", "avoid", "warning", "fake"];

                        let pos_count = positive_words
                            .iter()
                            .filter(|w| text_lower.contains(*w))
                            .count();
                        let neg_count = negative_words
                            .iter()
                            .filter(|w| text_lower.contains(*w))
                            .count();

                        if pos_count > neg_count {
                            positive_mentions += 1;
                        } else if neg_count > pos_count {
                            negative_mentions += 1;
                        }
                    }
                }

                info!("   📊 Found {} web mentions", twitter_mentions);
                if telegram_activity {
                    info!("   ✅ Telegram activity detected");
                }
            }
        }
        Err(e) => {
            warn!("   ⚠️  Exa search failed: {}", e);
        }
    }

    // Calculate community sentiment (0.0-1.0)
    let total_sentiment = positive_mentions + negative_mentions;
    let community_sentiment = if total_sentiment > 0 {
        positive_mentions as f64 / total_sentiment as f64
    } else {
        0.5 // Neutral if no data
    };

    // Calculate buzz score
    let mention_score = (twitter_mentions as f64 / 10.0).min(1.0);
    let telegram_score = if telegram_activity { 0.3 } else { 0.0 };
    let sentiment_score = community_sentiment * 0.3;

    let buzz_score = mention_score * 0.4 + telegram_score + sentiment_score;

    let result = SocialBuzz {
        twitter_mentions,
        telegram_activity,
        community_sentiment,
        buzz_score,
    };

    if buzz_score > 0.7 {
        info!("   🔥 HIGH BUZZ DETECTED! Score: {:.2}", buzz_score);
    } else if buzz_score > 0.4 {
        info!("   📈 Moderate buzz. Score: {:.2}", buzz_score);
    } else {
        warn!("   📉 Low/no buzz. Score: {:.2}", buzz_score);
    }

    Ok(result)
}

/// Search for specific Twitter handle mentions
pub async fn search_twitter_account(
    token_symbol: &str,
    twitter_handle: &str,
    api_key: &str,
) -> Result<u32> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    let query = format!("@{} {} crypto", twitter_handle, token_symbol);

    let search_request = ExaSearchRequest {
        query,
        search_type: "neural".to_string(),
        num_results: 5,
        text: false,
    };

    let response = client
        .post("https://api.exa.ai/search")
        .header("x-api-key", api_key)
        .header("Content-Type", "application/json")
        .json(&search_request)
        .send()
        .await?;

    let results = response.json::<ExaSearchResponse>().await?;
    Ok(results.results.len() as u32)
}
