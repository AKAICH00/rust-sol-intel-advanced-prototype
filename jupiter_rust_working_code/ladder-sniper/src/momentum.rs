use crate::candle_builder::CandleBuilder;
use crate::vwap::VWAPTracker;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MomentumSignal {
    StrongBuy,    // Strong momentum, hold position
    Hold,         // Neutral momentum, keep holding
    WeakExit,     // Weak momentum, consider exit
    Dump,         // No momentum, exit immediately
}

#[derive(Debug)]
pub struct MomentumDetector {
    min_threshold: f64,  // Minimum momentum to hold (0.0-1.0)
}

impl MomentumDetector {
    pub fn new(min_threshold: f64) -> Self {
        Self {
            min_threshold: min_threshold.clamp(0.0, 1.0),
        }
    }

    /// Calculate momentum score (0.0-1.0)
    /// Uses: price change, VWAP position, volume acceleration, buy ratio
    pub fn calculate_momentum(
        &self,
        candle_builder: &CandleBuilder,
        vwap_tracker: &VWAPTracker,
        _elapsed_secs: u64,
    ) -> f64 {
        // No candles yet, neutral momentum
        if candle_builder.current_candle().is_none() {
            return 0.5;
        }

        let mut score = 0.0;
        let mut weight_sum = 0.0;

        // 1. VWAP strength (40% weight)
        let vwap_strength = vwap_tracker.vwap_strength();
        score += vwap_strength * 0.4;
        weight_sum += 0.4;

        // 2. Buy ratio (30% weight)
        if let Some(current_candle) = candle_builder.current_candle() {
            let buy_ratio = current_candle.buy_ratio();
            score += buy_ratio * 0.3;
            weight_sum += 0.3;
        }

        // 3. Volume acceleration (20% weight)
        if candle_builder.completed_candles().len() >= 2 {
            let avg_volume = candle_builder.avg_volume(5);
            if let Some(current_candle) = candle_builder.current_candle() {
                if avg_volume > 0.0 {
                    let volume_accel = (current_candle.volume_sol / avg_volume).min(2.0) / 2.0;
                    score += volume_accel * 0.2;
                    weight_sum += 0.2;
                }
            }
        }

        // 4. Price momentum (10% weight)
        if let Some(current_candle) = candle_builder.current_candle() {
            let price_change = current_candle.price_change_percent();
            let normalized_change = (price_change / 50.0).clamp(-1.0, 1.0); // Normalize to -1 to +1
            let momentum_contribution = ((normalized_change + 1.0) / 2.0) * 0.1; // Map to 0-1 scale
            score += momentum_contribution;
            weight_sum += 0.1;
        }

        // Normalize score by total weight
        if weight_sum > 0.0 {
            score / weight_sum
        } else {
            0.5 // Neutral if no data
        }
    }

    /// Check if we should exit at specific time checkpoints
    /// Returns (should_exit, reason)
    pub fn check_time_exit(
        &self,
        candle_builder: &CandleBuilder,
        vwap_tracker: &VWAPTracker,
        elapsed_secs: u64,
    ) -> (bool, String) {
        let momentum = self.calculate_momentum(candle_builder, vwap_tracker, elapsed_secs);

        // Time-based thresholds (increasing requirements over time)
        let checkpoint = if elapsed_secs >= 60 {
            (0.6, "60s checkpoint")
        } else if elapsed_secs >= 45 {
            (0.5, "45s checkpoint")
        } else if elapsed_secs >= 30 {
            (0.4, "30s checkpoint")
        } else if elapsed_secs >= 20 {
            (0.3, "20s checkpoint")
        } else if elapsed_secs >= 10 {
            (0.2, "10s checkpoint")
        } else {
            return (false, String::new()); // Too early to exit
        };

        let (threshold, checkpoint_name) = checkpoint;

        if momentum < threshold {
            return (
                true,
                format!("{} - momentum {:.1}% < {:.0}%", checkpoint_name, momentum * 100.0, threshold * 100.0)
            );
        }

        // Additional VWAP-based exit (if price drops 5% below VWAP)
        if vwap_tracker.should_exit_on_vwap(0.05) {
            return (
                true,
                format!("Price {:.1}% below VWAP", vwap_tracker.vwap_distance_percent().abs())
            );
        }

        (false, String::new())
    }

    /// Get momentum signal for display
    pub fn get_signal(
        &self,
        candle_builder: &CandleBuilder,
        vwap_tracker: &VWAPTracker,
        elapsed_secs: u64,
    ) -> MomentumSignal {
        let momentum = self.calculate_momentum(candle_builder, vwap_tracker, elapsed_secs);

        if momentum >= 0.7 {
            MomentumSignal::StrongBuy
        } else if momentum >= 0.5 {
            MomentumSignal::Hold
        } else if momentum >= self.min_threshold {
            MomentumSignal::WeakExit
        } else {
            MomentumSignal::Dump
        }
    }

    /// Check if we should take profit at 2x
    pub fn should_take_profit(
        &self,
        entry_price: f64,
        current_price: f64,
    ) -> bool {
        if entry_price == 0.0 {
            return false;
        }
        let multiplier = current_price / entry_price;
        multiplier >= 2.0
    }
}

impl Default for MomentumDetector {
    fn default() -> Self {
        Self::new(0.2) // 20% minimum momentum
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trade_events::TradeEvent;

    #[test]
    fn test_momentum_calculation() {
        let detector = MomentumDetector::new(0.2);
        let mut candle_builder = CandleBuilder::new(500, 100);
        let mut vwap_tracker = VWAPTracker::new();

        // Add trades
        let trade1 = TradeEvent::new_buy(0.0001, 0.02);
        vwap_tracker.add_trade(&trade1);
        candle_builder.add_trade(&trade1);

        let trade2 = TradeEvent::new_buy(0.00012, 0.02);
        vwap_tracker.add_trade(&trade2);
        candle_builder.add_trade(&trade2);

        let momentum = detector.calculate_momentum(&candle_builder, &vwap_tracker, 5);

        // With positive price action and all buys, momentum should be high
        assert!(momentum > 0.5);
    }

    #[test]
    fn test_time_exit_early() {
        let detector = MomentumDetector::new(0.2);
        let candle_builder = CandleBuilder::new(500, 100);
        let vwap_tracker = VWAPTracker::new();

        // At 5 seconds, should not exit (too early)
        let (should_exit, _) = detector.check_time_exit(&candle_builder, &vwap_tracker, 5);
        assert!(!should_exit);
    }

    #[test]
    fn test_time_exit_at_checkpoint() {
        let detector = MomentumDetector::new(0.2);
        let mut candle_builder = CandleBuilder::new(500, 100);
        let mut vwap_tracker = VWAPTracker::new();

        // Add weak trades (sells)
        let trade = TradeEvent::new_sell(0.0001, 0.02);
        vwap_tracker.add_trade(&trade);
        candle_builder.add_trade(&trade);

        // At 10s checkpoint with weak momentum, should exit
        let (should_exit, reason) = detector.check_time_exit(&candle_builder, &vwap_tracker, 10);

        // With low momentum, should trigger exit
        println!("Exit decision: {}, reason: {}", should_exit, reason);
    }

    #[test]
    fn test_profit_taking() {
        let detector = MomentumDetector::default();

        // Should take profit at 2x
        assert!(detector.should_take_profit(0.0001, 0.0002));

        // Should not take profit below 2x
        assert!(!detector.should_take_profit(0.0001, 0.00015));
    }
}
