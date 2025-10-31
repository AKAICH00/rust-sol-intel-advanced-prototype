//! PumpPortal API client implementation

use crate::{PumpPortalError, Result, TradeRequest, TradeResponse};
use reqwest::Client;
use serde_json::json;

const BASE_URL: &str = "https://pumpportal.fun/api/trade";

/// PumpPortal API client
///
/// Handles authentication and communication with the PumpPortal Trading API.
pub struct PumpPortalClient {
    client: Client,
    api_key: String,
}

impl PumpPortalClient {
    /// Create a new PumpPortal client with the provided API key
    ///
    /// # Arguments
    ///
    /// * `api_key` - Your PumpPortal API key
    ///
    /// # Example
    ///
    /// ```no_run
    /// use pump_portal_sdk::PumpPortalClient;
    ///
    /// let client = PumpPortalClient::new("your-api-key".to_string());
    /// ```
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    /// Execute a trade request
    ///
    /// # Arguments
    ///
    /// * `request` - Trade request parameters
    ///
    /// # Returns
    ///
    /// Returns the transaction signature if successful
    ///
    /// # Errors
    ///
    /// Returns `PumpPortalError` if the request fails or the API returns an error
    pub async fn trade(&self, request: TradeRequest) -> Result<TradeResponse> {
        let url = format!("{}?api-key={}", BASE_URL, self.api_key);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        // Try to parse as JSON first
        let trade_response: TradeResponse = serde_json::from_str(&body)
            .unwrap_or_else(|_| TradeResponse {
                signature: None,
                error: Some(body.clone()),
                extra: json!({}),
            });

        // Check if there was an error
        if !status.is_success() || trade_response.error.is_some() {
            let error_msg = trade_response
                .error
                .unwrap_or_else(|| format!("HTTP {}: {}", status, body));
            return Err(PumpPortalError::ApiError(error_msg));
        }

        Ok(trade_response)
    }

    /// Execute a buy order
    ///
    /// # Arguments
    ///
    /// * `mint` - Token contract address
    /// * `sol_amount` - Amount of SOL to spend
    /// * `slippage` - Slippage percentage (e.g., 10 for 10%)
    /// * `priority_fee` - Priority fee for faster execution
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use pump_portal_sdk::PumpPortalClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = PumpPortalClient::new("your-api-key".to_string());
    /// let response = client.buy(
    ///     "TokenMintAddress".to_string(),
    ///     0.1,  // 0.1 SOL
    ///     10,   // 10% slippage
    ///     0.0001, // priority fee
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn buy(
        &self,
        mint: String,
        sol_amount: f64,
        slippage: u32,
        priority_fee: f64,
    ) -> Result<TradeResponse> {
        let request = TradeRequest::buy(mint, sol_amount, slippage, priority_fee);
        self.trade(request).await
    }

    /// Execute a sell order
    ///
    /// # Arguments
    ///
    /// * `mint` - Token contract address
    /// * `token_amount` - Amount of tokens to sell (can be percentage like "100%" or absolute amount)
    /// * `slippage` - Slippage percentage (e.g., 10 for 10%)
    /// * `priority_fee` - Priority fee for faster execution
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use pump_portal_sdk::PumpPortalClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = PumpPortalClient::new("your-api-key".to_string());
    /// let response = client.sell(
    ///     "TokenMintAddress".to_string(),
    ///     "100%".to_string(), // Sell all tokens
    ///     10,   // 10% slippage
    ///     0.0001, // priority fee
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn sell(
        &self,
        mint: String,
        token_amount: String,
        slippage: u32,
        priority_fee: f64,
    ) -> Result<TradeResponse> {
        let request = TradeRequest::sell(mint, token_amount, slippage, priority_fee);
        self.trade(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = PumpPortalClient::new("test-key".to_string());
        assert_eq!(client.api_key, "test-key");
    }
}
