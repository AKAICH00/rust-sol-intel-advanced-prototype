use crate::trade_events::TradeEvent;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct Candle {
    pub timestamp: Instant,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume_sol: f64,
    pub buy_volume_sol: f64,
    pub sell_volume_sol: f64,
    pub trade_count: u32,
    pub buy_count: u32,
    pub sell_count: u32,
}

impl Candle {
    pub fn new(first_trade: &TradeEvent) -> Self {
        Self {
            timestamp: first_trade.timestamp,
            open: first_trade.price,
            high: first_trade.price,
            low: first_trade.price,
            close: first_trade.price,
            volume_sol: first_trade.volume_sol,
            buy_volume_sol: if first_trade.is_buy { first_trade.volume_sol } else { 0.0 },
            sell_volume_sol: if !first_trade.is_buy { first_trade.volume_sol } else { 0.0 },
            trade_count: 1,
            buy_count: if first_trade.is_buy { 1 } else { 0 },
            sell_count: if !first_trade.is_buy { 1 } else { 0 },
        }
    }

    pub fn add_trade(&mut self, trade: &TradeEvent) {
        // Update OHLC
        if trade.price > self.high {
            self.high = trade.price;
        }
        if trade.price < self.low {
            self.low = trade.price;
        }
        self.close = trade.price;

        // Update volume
        self.volume_sol += trade.volume_sol;
        if trade.is_buy {
            self.buy_volume_sol += trade.volume_sol;
            self.buy_count += 1;
        } else {
            self.sell_volume_sol += trade.volume_sol;
            self.sell_count += 1;
        }
        self.trade_count += 1;
    }

    /// Buy/sell ratio (0.0 = all sells, 1.0 = all buys)
    pub fn buy_ratio(&self) -> f64 {
        if self.volume_sol == 0.0 {
            return 0.5; // Neutral if no volume
        }
        self.buy_volume_sol / self.volume_sol
    }

    /// Price change percentage within candle
    pub fn price_change_percent(&self) -> f64 {
        if self.open == 0.0 {
            return 0.0;
        }
        ((self.close - self.open) / self.open) * 100.0
    }

    /// Is this candle bullish?
    pub fn is_bullish(&self) -> bool {
        self.close > self.open && self.buy_ratio() > 0.5
    }

    /// Is this candle bearish?
    pub fn is_bearish(&self) -> bool {
        self.close < self.open && self.buy_ratio() < 0.5
    }
}

#[derive(Debug, Clone)]
pub struct CandleBuilder {
    interval_ms: u64,
    current_candle: Option<Candle>,
    completed_candles: Vec<Candle>,
    max_candles: usize,
}

impl CandleBuilder {
    pub fn new(interval_ms: u64, max_candles: usize) -> Self {
        Self {
            interval_ms,
            current_candle: None,
            completed_candles: Vec::with_capacity(max_candles),
            max_candles,
        }
    }

    pub fn add_trade(&mut self, trade: &TradeEvent) {
        match &mut self.current_candle {
            None => {
                // Start first candle
                self.current_candle = Some(Candle::new(trade));
            }
            Some(candle) => {
                let elapsed_ms = candle.timestamp.elapsed().as_millis() as u64;

                if elapsed_ms >= self.interval_ms {
                    // Complete current candle and start new one
                    let completed = candle.clone();
                    self.completed_candles.push(completed);

                    // Keep only max_candles in memory
                    if self.completed_candles.len() > self.max_candles {
                        self.completed_candles.remove(0);
                    }

                    // Start new candle
                    self.current_candle = Some(Candle::new(trade));
                } else {
                    // Add to current candle
                    candle.add_trade(trade);
                }
            }
        }
    }

    pub fn current_candle(&self) -> Option<&Candle> {
        self.current_candle.as_ref()
    }

    pub fn completed_candles(&self) -> &[Candle] {
        &self.completed_candles
    }

    pub fn latest_candle(&self) -> Option<&Candle> {
        self.completed_candles.last()
    }

    /// Get average volume over last N candles
    pub fn avg_volume(&self, count: usize) -> f64 {
        if self.completed_candles.is_empty() {
            return 0.0;
        }

        let start = if self.completed_candles.len() > count {
            self.completed_candles.len() - count
        } else {
            0
        };

        let sum: f64 = self.completed_candles[start..]
            .iter()
            .map(|c| c.volume_sol)
            .sum();

        sum / (self.completed_candles.len() - start) as f64
    }

    /// Get average buy ratio over last N candles
    pub fn avg_buy_ratio(&self, count: usize) -> f64 {
        if self.completed_candles.is_empty() {
            return 0.5;
        }

        let start = if self.completed_candles.len() > count {
            self.completed_candles.len() - count
        } else {
            0
        };

        let sum: f64 = self.completed_candles[start..]
            .iter()
            .map(|c| c.buy_ratio())
            .sum();

        sum / (self.completed_candles.len() - start) as f64
    }

    /// Check if momentum is increasing
    pub fn is_accelerating(&self) -> bool {
        if self.completed_candles.len() < 3 {
            return false;
        }

        let len = self.completed_candles.len();
        let last = &self.completed_candles[len - 1];
        let prev = &self.completed_candles[len - 2];

        // Volume and buy ratio both increasing
        last.volume_sol > prev.volume_sol && last.buy_ratio() > prev.buy_ratio()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_trade(price: f64, volume: f64, is_buy: bool) -> TradeEvent {
        if is_buy {
            TradeEvent::new_buy(price, volume)
        } else {
            TradeEvent::new_sell(price, volume)
        }
    }

    #[test]
    fn test_candle_creation() {
        let trade = create_test_trade(0.0001, 0.02, true);
        let candle = Candle::new(&trade);

        assert_eq!(candle.open, 0.0001);
        assert_eq!(candle.close, 0.0001);
        assert_eq!(candle.trade_count, 1);
        assert_eq!(candle.buy_count, 1);
    }

    #[test]
    fn test_candle_builder() {
        let mut builder = CandleBuilder::new(500, 100); // 500ms candles

        // Add first trade
        let trade1 = create_test_trade(0.0001, 0.02, true);
        builder.add_trade(&trade1);

        assert!(builder.current_candle().is_some());
        assert_eq!(builder.completed_candles().len(), 0);

        // Add another trade to same candle
        let trade2 = create_test_trade(0.00012, 0.02, true);
        builder.add_trade(&trade2);

        let current = builder.current_candle().unwrap();
        assert_eq!(current.trade_count, 2);
        assert_eq!(current.high, 0.00012);
    }

    #[test]
    fn test_buy_ratio() {
        let mut candle = Candle::new(&create_test_trade(0.0001, 0.02, true));
        candle.add_trade(&create_test_trade(0.0001, 0.02, true));
        candle.add_trade(&create_test_trade(0.0001, 0.02, false));

        // 2 buys (0.04 SOL) + 1 sell (0.02 SOL) = 0.06 SOL total
        // Buy ratio: 0.04 / 0.06 = 0.666...
        assert!((candle.buy_ratio() - 0.6666).abs() < 0.001);
    }
}
