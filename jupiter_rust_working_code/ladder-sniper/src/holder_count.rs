use anyhow::Result;
use log::warn;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize)]
struct RpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct RpcResponse {
    result: Option<Vec<serde_json::Value>>,
}

pub struct HolderCountClient {
    rpc_url: String,
    client: reqwest::Client,
}

impl HolderCountClient {
    pub fn new(rpc_url: String) -> Self {
        Self {
            rpc_url,
            client: reqwest::Client::new(),
        }
    }

    /// Get the number of holders for a token mint
    pub async fn get_holder_count(&self, mint: &str) -> Result<u64> {
        // Use getProgramAccounts with filters to count token accounts
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "getProgramAccounts".to_string(),
            params: vec![
                json!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"), // SPL Token Program
                json!({
                    "encoding": "base64",
                    "filters": [
                        {
                            "dataSize": 165  // Token account size
                        },
                        {
                            "memcmp": {
                                "offset": 0,
                                "bytes": mint  // Filter by mint address
                            }
                        }
                    ],
                    "dataSlice": {
                        "offset": 64,
                        "length": 8
                    }
                }),
            ],
        };

        let response = match self
            .client
            .post(&self.rpc_url)
            .json(&request)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                warn!("Failed to fetch holder count for {}: {}", mint, e);
                return Ok(0);
            }
        };

        let rpc_response: RpcResponse = match response.json().await {
            Ok(resp) => resp,
            Err(e) => {
                warn!("Failed to parse holder count response for {}: {}", mint, e);
                return Ok(0);
            }
        };

        // Count accounts with non-zero balance
        let holder_count = rpc_response.result.map(|accounts| {
            accounts
                .iter()
                .filter(|account| {
                    // Check if account has non-zero amount (bytes 64-72)
                    account
                        .get("account")
                        .and_then(|acc| acc.get("data"))
                        .and_then(|data| data.as_array())
                        .and_then(|arr| arr.get(0))
                        .and_then(|b64| b64.as_str())
                        .and_then(|b64_str| base64::decode(b64_str).ok())
                        .map(|bytes| {
                            // Check if amount > 0 (8 bytes little-endian at offset 64)
                            if bytes.len() >= 8 {
                                let amount = u64::from_le_bytes([
                                    bytes[0], bytes[1], bytes[2], bytes[3],
                                    bytes[4], bytes[5], bytes[6], bytes[7],
                                ]);
                                amount > 0
                            } else {
                                false
                            }
                        })
                        .unwrap_or(false)
                })
                .count() as u64
        }).unwrap_or(0);

        Ok(holder_count)
    }
}
