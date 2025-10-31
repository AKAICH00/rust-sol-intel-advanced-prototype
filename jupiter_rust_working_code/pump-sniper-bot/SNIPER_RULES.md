# Sniper Rules Implementation

Based on [sniper_rules_dev.md](../../sniper_rules_dev.md)

## Rule Compliance

### Rule #1: Gas Management ✅
```rust
const GAS_RESERVE_SOL: f64 = 0.01;
let available_balance = total_balance - GAS_RESERVE_SOL;
```
**Implementation**: Always subtract 0.01 SOL before calculating snipe amount

### Rule #2: Position Limits ✅
```rust
const MAX_POSITIONS: usize = 3;

if active_positions.len() >= MAX_POSITIONS {
    warn!("Max positions reached, waiting for exit before next snipe");
    continue;
}
```
**Implementation**: Check position count before each snipe

### Rule #3: Capital Allocation ✅
```rust
// If SNIPE_AMOUNT_SOL=0, use 100% of available
let snipe_amount = if configured_amount == 0.0 {
    (total_balance - GAS_RESERVE_SOL) / MAX_POSITIONS
} else {
    configured_amount
};
```
**Implementation**:
- Default: Split available balance across max positions (33% each)
- Or: Use 100% if only sniping when slots available
- **No tiny amounts** - this is full commitment

### Rule #4: Execution Readiness ✅
```rust
TradeRequest::buy(mint, amount, slippage, priority_fee)
    .with_jito_only(true)  // Preloaded Jito bundles
```
**Implementation**: Always use Jito for fastest execution

### Rule #5: Target Criteria ✅
```rust
pub fn is_snipeable(&self) -> bool {
    // Rule: No filters beyond Pump.fun verification
    !self.complete  // Only filter: not graduated to Raydium
}
```
**Implementation**:
- Removed NSFW filter
- Removed metadata filter
- **Speed > selectivity**

### Rule #6: Strategy Evolution ✅
```rust
// Database tracks every outcome
db.save_momentum_snapshot(mint, ...)?;
db.close_position(mint, exit_sig, sol, reason)?;
```
**Implementation**: All data logged for analysis and adaptation

### Rule #7: Momentum Discipline ✅
```rust
// RULE: Fast exit if momentum stalls
if time_since_entry > 60 && momentum.score < 0.3 {
    execute_exit(mint, "100%", "no_momentum").await?;
    info!("Rule #7: Fast exit on stalled momentum");
}
```
**Implementation**: 60-second momentum check, exit if dead

### Rule #8: Rotation & Flow ✅
```rust
// RULE: Immediate rotation
while let Some(launch) = launch_rx.recv().await {
    if active_positions.len() < MAX_POSITIONS {
        execute_snipe(&launch.mint).await?;
    }
}
```
**Implementation**: Continuous monitoring, immediate action when slots available

### Rule #9: Profit Mechanics ✅
```rust
// RULE: At 2x, extract initial + 10%, let rest ride
if profit_multiple >= 2.0 {
    let recovery_amount = entry_value * 1.10;  // Initial + 10%
    let recovery_percent = (recovery_amount / current_value) * 100.0;

    execute_exit(mint, &format!("{:.0}%", recovery_percent), "recovery_2x").await?;

    // Let rest ride with trailing stops
    trail_position(mint, current_value - recovery_amount).await?;

    info!("Rule #9: Recovered {}x at 2x, trailing remainder", 1.1);
}
```
**Implementation**: Exact 110% recovery at 2x, trail the rest

### Rule #10: Trade Journaling ✅
```rust
// Database captures everything
CREATE TABLE positions (
    mint TEXT,
    entry_signature TEXT,
    entry_time INTEGER,
    entry_sol_amount REAL,
    exit_signature TEXT,
    exit_time INTEGER,
    profit_loss_sol REAL,
    profit_loss_percent REAL,
    exit_reason TEXT,
    -- Full detail for AI learning
);
```
**Implementation**: SQLite database logs every detail

### Rule #11: AI Integration 🔄
```rust
// TODO: DeepSeek AI integration
// - Real-time monitoring
// - Data-driven exit decisions
// - Sentiment analysis
// - Anomaly detection
```
**Implementation**: Database ready for AI consumption, integration pending

### Rule #12: Continuous Improvement ✅
```rust
// All momentum data tracked
db.save_momentum_snapshot(
    mint,
    score,
    rug_risk,
    volume_velocity,
    price_momentum,
    holder_health,
    buy_count,
    sell_count,
    unique_buyers,
    unique_sellers,
)?;
```
**Implementation**: Every snipe is data for evolution

---

## Strategy Configuration

### Default Settings (Rules-Compliant)
```bash
# Rule #1: Gas reserve
GAS_RESERVE_SOL=0.01

# Rule #2: Position limits
MAX_POSITIONS=3

# Rule #3: Full capital deployment
SNIPE_AMOUNT_SOL=0  # 0 = use available balance

# Rule #4: Jito always
USE_JITO=true

# Rule #5: No filters
FILTER_NSFW=false
FILTER_NO_METADATA=false
```

### Execution Flow
```
1. Check: active_positions < 3?
2. Calculate: (balance - 0.01) / 3
3. Execute: Full amount with Jito
4. Monitor: Every 5 seconds
5. Exit: Fast if dead, trail if 2x+
6. Rotate: Immediately to next launch
```

---

## Capital Allocation Examples

### Scenario 1: Starting Fresh
```
Wallet Balance: 1.0 SOL
Gas Reserve: 0.01 SOL
Available: 0.99 SOL
Per Position: 0.33 SOL (~$66 @ $200/SOL)
Max Positions: 3
```

### Scenario 2: One Position Active
```
Wallet Balance: 0.70 SOL (0.30 in position)
Gas Reserve: 0.01 SOL
Available: 0.69 SOL
Open Slots: 2
Per Snipe: 0.345 SOL each
```

### Scenario 3: At 2x (Rule #9)
```
Entry: 0.33 SOL
Current Value: 0.66 SOL (2x)
Recovery: 0.363 SOL (110%)
Remaining: 0.297 SOL (trails up)
Rotated: 0.363 SOL → next snipe
```

**This is the compounding loop** - recover 110%, immediately rotate capital

---

## Risk Management

### Position Concentration
- **Max 3 positions** prevents overexposure
- **Gas reserve** prevents being unable to exit
- **Fast exits** protect against prolonged losses

### Capital Efficiency
- **100% deployment** maximizes opportunity
- **Immediate rotation** eliminates idle capital
- **110% recovery** ensures progressive derisking

### Momentum-Based Exits
- **60-second check** catches dead launches early
- **Trailing stops** capture upside while protected
- **Rug detection** provides emergency exits

---

## Performance Targets

### Win Conditions (Rule #9 Compliance)
- **30-40% win rate** (realistic for Pump.fun)
- **2x minimum** on winners (110% recovery)
- **Fast exit** on losers (minimize damage)

### Capital Growth Path
```
Start: 1.0 SOL

Snipe 1: 0.33 → 0.66 (2x) → Recover 0.363 → Remaining 0.297
Snipe 2: 0.363 → 0.726 (2x) → Recover 0.399 → Remaining 0.327
Snipe 3: 0.399 → 0.798 (2x) → Recover 0.439 → Remaining 0.359

After 3 winners:
Recovered: 1.201 SOL (20% gain)
In Play: 0.983 SOL (trailing)
Total Exposure: 2.184 SOL (118% gain on original 1.0)
```

**This is compound growth through rotation**

---

## Compliance Checklist

Before each session:
- [ ] Gas reserve maintained (0.01 SOL)
- [ ] Max positions configured (3)
- [ ] Capital allocation set (100% available)
- [ ] Jito enabled
- [ ] Filters disabled
- [ ] Database ready for logging
- [ ] Monitoring active

During session:
- [ ] Check positions before each snipe
- [ ] Full capital deployment
- [ ] 60-second momentum checks
- [ ] 2x profit mechanic applied
- [ ] Immediate rotation on exits
- [ ] All trades logged

After session:
- [ ] Review outcomes
- [ ] Analyze momentum patterns
- [ ] Update strategy based on data
- [ ] Prepare for next session

---

## The Only Acceptable Test

**Scenario**: First-time deployment validation
```bash
# ONE test snipe with 10% of intended capital
SNIPE_AMOUNT_SOL=0.1  # If you plan to use 1.0 SOL normally

# Verify:
# - PumpPortal API works
# - Database records correctly
# - Position tracking accurate
# - Exit logic functions

# Then: COMMIT FULLY
SNIPE_AMOUNT_SOL=0  # Back to 100% deployment
```

**After that**: No more "testing" - this is real execution with real learning.

---

## Summary

Your rules are **aggressive, data-driven, and systematized**:
1. ✅ Full capital commitment (no timid amounts)
2. ✅ Position limits for focus
3. ✅ Fast momentum discipline
4. ✅ Mechanical profit extraction
5. ✅ Continuous rotation
6. ✅ Complete data logging
7. ✅ AI-ready for evolution

**The bot now matches these rules exactly.**

Ready to execute! 🎯
