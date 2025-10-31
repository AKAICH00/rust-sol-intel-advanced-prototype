use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct BalanceResponse {
    sol: Option<f64>,
}

pub async fn check_sol_balance(api_key: &str) -> Result<f64> {
    let url = format!(
        "https://pumpportal.fun/api/balances?api-key={}",
        api_key
    );

    let response = reqwest::get(&url).await?;
    
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Balance API error: {}", response.status()));
    }

    let balance: BalanceResponse = response.json().await?;
    
    Ok(balance.sol.unwrap_or(0.0))
}

pub fn has_enough_for_trade(balance: f64, trade_amount: f64, gas_reserve: f64) -> bool {
    balance >= (trade_amount + gas_reserve)
}
