use crate::trade_events::TradeEvent;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct VWAPTracker {
    start_time: Instant,
    cumulative_pv: f64,      // Sum of price * volume
    cumulative_volume: f64,  // Sum of volume
    vwap: f64,
    last_price: f64,
    trade_count: u32,
}

impl VWAPTracker {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            cumulative_pv: 0.0,
            cumulative_volume: 0.0,
            vwap: 0.0,
            last_price: 0.0,
            trade_count: 0,
        }
    }

    /// Add a trade and update VWAP
    pub fn add_trade(&mut self, trade: &TradeEvent) {
        let pv = trade.price * trade.volume_sol;
        self.cumulative_pv += pv;
        self.cumulative_volume += trade.volume_sol;
        self.last_price = trade.price;
        self.trade_count += 1;

        // Recalculate VWAP
        if self.cumulative_volume > 0.0 {
            self.vwap = self.cumulative_pv / self.cumulative_volume;
        }
    }

    /// Get current VWAP
    pub fn vwap(&self) -> f64 {
        self.vwap
    }

    /// Get last trade price
    pub fn last_price(&self) -> f64 {
        self.last_price
    }

    /// Get price deviation from VWAP (-1.0 to +1.0, where 0 = at VWAP)
    pub fn price_deviation(&self) -> f64 {
        if self.vwap == 0.0 {
            return 0.0;
        }
        (self.last_price - self.vwap) / self.vwap
    }

    /// Is current price above VWAP? (bullish signal)
    pub fn above_vwap(&self) -> bool {
        self.last_price > self.vwap
    }

    /// Is current price below VWAP? (bearish signal)
    pub fn below_vwap(&self) -> bool {
        self.last_price < self.vwap
    }

    /// Get price percentage above/below VWAP
    pub fn vwap_distance_percent(&self) -> f64 {
        if self.vwap == 0.0 {
            return 0.0;
        }
        ((self.last_price - self.vwap) / self.vwap) * 100.0
    }

    /// Get elapsed time since first trade
    pub fn elapsed_ms(&self) -> u128 {
        self.start_time.elapsed().as_millis()
    }

    /// Get total volume traded
    pub fn total_volume(&self) -> f64 {
        self.cumulative_volume
    }

    /// Get trade count
    pub fn trade_count(&self) -> u32 {
        self.trade_count
    }

    /// Check if we should exit based on VWAP deviation
    /// Exit if price drops > threshold below VWAP
    pub fn should_exit_on_vwap(&self, threshold: f64) -> bool {
        if self.vwap == 0.0 {
            return false;
        }
        let deviation = self.price_deviation();
        deviation < -threshold // Negative deviation = below VWAP
    }

    /// Get VWAP strength signal (0.0-1.0)
    /// 1.0 = strong buy (well above VWAP)
    /// 0.5 = neutral (at VWAP)
    /// 0.0 = weak/exit (well below VWAP)
    pub fn vwap_strength(&self) -> f64 {
        let deviation = self.price_deviation();

        // Map deviation to 0.0-1.0 scale
        // +0.2 (20% above) = 1.0 (strong)
        // 0.0 (at VWAP) = 0.5 (neutral)
        // -0.2 (20% below) = 0.0 (weak)

        let normalized = (deviation + 0.2) / 0.4; // Map [-0.2, +0.2] to [0, 1]
        normalized.clamp(0.0, 1.0)
    }

    /// Reset VWAP calculation (for new position)
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
        self.cumulative_pv = 0.0;
        self.cumulative_volume = 0.0;
        self.vwap = 0.0;
        self.last_price = 0.0;
        self.trade_count = 0;
    }
}

impl Default for VWAPTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vwap_calculation() {
        let mut tracker = VWAPTracker::new();

        // Trade 1: Buy 0.02 SOL at 0.0001
        tracker.add_trade(&TradeEvent::new_buy(0.0001, 0.02));
        assert_eq!(tracker.vwap(), 0.0001);

        // Trade 2: Buy 0.02 SOL at 0.0002
        tracker.add_trade(&TradeEvent::new_buy(0.0002, 0.02));

        // VWAP = (0.0001 * 0.02 + 0.0002 * 0.02) / (0.02 + 0.02)
        //      = (0.000002 + 0.000004) / 0.04
        //      = 0.000006 / 0.04
        //      = 0.00015
        assert!((tracker.vwap() - 0.00015).abs() < 0.0000001);
    }

    #[test]
    fn test_vwap_deviation() {
        let mut tracker = VWAPTracker::new();

        tracker.add_trade(&TradeEvent::new_buy(0.0001, 0.02));
        tracker.add_trade(&TradeEvent::new_buy(0.0001, 0.02));

        // At VWAP, deviation should be 0
        assert_eq!(tracker.price_deviation(), 0.0);

        // Add trade 10% above VWAP
        tracker.add_trade(&TradeEvent::new_buy(0.00011, 0.02));
        assert!(tracker.above_vwap());
        assert!(tracker.price_deviation() > 0.0);
    }

    #[test]
    fn test_should_exit_on_vwap() {
        let mut tracker = VWAPTracker::new();

        tracker.add_trade(&TradeEvent::new_buy(0.0002, 0.02));
        tracker.add_trade(&TradeEvent::new_buy(0.0002, 0.02));

        // Price drops 10% below VWAP
        tracker.add_trade(&TradeEvent::new_sell(0.00018, 0.02));

        // Should exit if threshold is < 10%
        assert!(tracker.should_exit_on_vwap(0.05)); // 5% threshold
        assert!(!tracker.should_exit_on_vwap(0.15)); // 15% threshold
    }

    #[test]
    fn test_vwap_strength() {
        let mut tracker = VWAPTracker::new();

        tracker.add_trade(&TradeEvent::new_buy(0.0001, 0.02));

        // At VWAP, strength should be 0.5 (neutral)
        assert!((tracker.vwap_strength() - 0.5).abs() < 0.01);

        // 20% above VWAP, strength should be 1.0
        tracker.add_trade(&TradeEvent::new_buy(0.00012, 0.02));
        assert!(tracker.vwap_strength() > 0.7);

        // 20% below VWAP, strength should be 0.0
        tracker.add_trade(&TradeEvent::new_sell(0.00008, 0.02));
        assert!(tracker.vwap_strength() < 0.3);
    }
}
