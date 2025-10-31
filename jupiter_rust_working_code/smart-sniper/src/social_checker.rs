use anyhow::Result;
use log::{info, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct SocialScore {
    pub has_twitter: bool,
    pub has_telegram: bool,
    pub has_website: bool,
    pub momentum_score: f64,
    pub should_fast_exit: bool,
}

#[derive(Debug, Deserialize)]
struct PumpFunTokenData {
    #[serde(default)]
    twitter: Option<String>,
    #[serde(default)]
    telegram: Option<String>,
    #[serde(default)]
    website: Option<String>,
}

/// Check token socials and momentum
pub async fn check_social_momentum(mint: &str) -> Result<SocialScore> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    info!("   🔍 Checking socials for {}...", &mint[..8]);

    // Try pump.fun API first
    let pump_url = format!("https://frontend-api.pump.fun/coins/{}", mint);

    let mut has_twitter = false;
    let mut has_telegram = false;
    let mut has_website = false;

    match client.get(&pump_url).send().await {
        Ok(response) => {
            if let Ok(data) = response.json::<PumpFunTokenData>().await {
                has_twitter = data.twitter.as_ref().map_or(false, |t| !t.is_empty());
                has_telegram = data.telegram.as_ref().map_or(false, |t| !t.is_empty());
                has_website = data.website.as_ref().map_or(false, |w| !w.is_empty());

                if has_twitter {
                    info!("   ✅ Twitter found");
                }
                if has_telegram {
                    info!("   ✅ Telegram found");
                }
                if has_website {
                    info!("   ✅ Website found");
                }
            }
        }
        Err(e) => {
            warn!("   ⚠️  Failed to fetch pump.fun data: {}", e);
        }
    }

    // Calculate momentum score
    let social_count = [has_twitter, has_telegram, has_website]
        .iter()
        .filter(|&&x| x)
        .count();

    let momentum_score = match social_count {
        3 => 1.0,  // All socials = strong project
        2 => 0.7,  // Two socials = decent
        1 => 0.4,  // One social = weak
        0 => 0.0,  // No socials = RED FLAG
        _ => 0.0,
    };

    // Fast exit if NO socials
    let should_fast_exit = social_count == 0;

    if should_fast_exit {
        warn!("   ❌ ZERO SOCIALS - FAST EXIT MODE");
    } else if social_count == 1 {
        warn!("   ⚠️  Weak socials - reduced hold time");
    }

    Ok(SocialScore {
        has_twitter,
        has_telegram,
        has_website,
        momentum_score,
        should_fast_exit,
    })
}
