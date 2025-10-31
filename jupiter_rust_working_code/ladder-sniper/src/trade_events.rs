use std::time::Instant;

#[derive(Debug, Clone)]
pub struct TradeEvent {
    pub timestamp: Instant,
    pub price: f64,           // SOL per token
    pub volume_sol: f64,      // SOL amount
    pub is_buy: bool,
}

impl TradeEvent {
    pub fn new_buy(price: f64, volume_sol: f64) -> Self {
        Self {
            timestamp: Instant::now(),
            price,
            volume_sol,
            is_buy: true,
        }
    }

    pub fn new_sell(price: f64, volume_sol: f64) -> Self {
        Self {
            timestamp: Instant::now(),
            price,
            volume_sol,
            is_buy: false,
        }
    }

    /// Calculate price from bonding curve reserves
    /// Formula: price = sol_reserves / token_reserves
    pub fn calculate_price(sol_reserves: f64, token_reserves: f64) -> f64 {
        if token_reserves == 0.0 {
            return 0.0;
        }
        sol_reserves / token_reserves
    }

    /// Calculate price impact for a trade
    /// Returns: (new_price, tokens_received)
    pub fn calculate_trade_impact(
        sol_reserves: f64,
        token_reserves: f64,
        sol_amount: f64,
        is_buy: bool,
    ) -> (f64, f64) {
        if is_buy {
            // Buy: Add SOL to reserves
            let new_sol_reserves = sol_reserves + sol_amount;
            let new_token_reserves = (sol_reserves * token_reserves) / new_sol_reserves;
            let tokens_received = token_reserves - new_token_reserves;
            let new_price = Self::calculate_price(new_sol_reserves, new_token_reserves);
            (new_price, tokens_received)
        } else {
            // Sell: Remove SOL from reserves (simplified)
            let new_sol_reserves = sol_reserves - sol_amount;
            let new_price = Self::calculate_price(new_sol_reserves, token_reserves);
            (new_price, 0.0)
        }
    }

    /// Get elapsed milliseconds since event
    pub fn elapsed_ms(&self) -> u128 {
        self.timestamp.elapsed().as_millis()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_calculation() {
        let sol_reserves = 100.0;
        let token_reserves = 1_000_000.0;
        let price = TradeEvent::calculate_price(sol_reserves, token_reserves);
        assert_eq!(price, 0.0001); // 100 / 1M = 0.0001 SOL per token
    }

    #[test]
    fn test_buy_impact() {
        let sol_reserves = 100.0;
        let token_reserves = 1_000_000.0;
        let buy_amount = 10.0; // Buy with 10 SOL

        let (new_price, tokens) = TradeEvent::calculate_trade_impact(
            sol_reserves,
            token_reserves,
            buy_amount,
            true,
        );

        // After buying, price should be higher
        let old_price = TradeEvent::calculate_price(sol_reserves, token_reserves);
        assert!(new_price > old_price);

        // Should receive some tokens
        assert!(tokens > 0.0);
    }
}
