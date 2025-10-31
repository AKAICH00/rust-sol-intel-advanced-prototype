# Sniper Bot Strategy Deep Dive

## 🎯 Core Philosophy

**"Fast in, smart exit"**

The key to profitable pump.fun sniping is:
1. Get in fast (first 5-30 seconds)
2. Read momentum quickly
3. Exit intelligently based on signals

## 📊 Entry Strategy

### Why PumpPortal?
```
Jupiter:    ~1,400-2,000ms
PumpPortal: ~700-800ms
Advantage:  600-1,200ms faster
```

### The Math
On pump.fun bonding curves, price increases exponentially with volume:
```
Entry at 0.8s:  $0.00005 per token
Entry at 1.6s:  $0.0005 per token (10x worse!)

Same $5 invested:
Fast entry:  100,000 tokens
Slow entry:  10,000 tokens

10x more tokens = 10x more profit!
```

### Entry Settings
```rust
Amount: 0.025 SOL (~$5)
Slippage: 20% (launches are volatile)
Priority Fee: 0.0005 SOL (high priority)
Jito: Enabled (MEV protection)
```

## 🚦 Exit Decision Tree

### Decision Point 1: Momentum Check (60s)
```
AFTER 60 seconds:
  IF momentum_score < 0.3:
    → Fast exit 100%
    → Reason: No interest, likely dead
    → Goal: Preserve capital

  ELIF momentum_score > 0.8:
    → Wait for profits
    → Watch for rug signals
    → Prepare ladder strategy

  ELSE:
    → Continue monitoring
    → Set tight stops
```

**Why 60 seconds?**
- Long enough to see real interest
- Short enough to exit before everyone else
- Prevents "slow bleed" losses

### Decision Point 2: 2x Profit Reached
```
WHEN current_value >= 2x entry:
  recovery_amount = entry * 1.1
  recovery_percent = (recovery_amount / current_value) * 100

  → Sell recovery_percent
  → Now playing with "house money"
  → Trail remaining with tight stops
```

**Psychology**:
- Recovered initial + 10% profit
- Remaining position is "free money"
- Can hold longer without stress
- Removes emotional decision making

### Decision Point 3: Trailing Stop
```
AFTER 2x recovery:
  highest_value = current_value
  trailing_stop = highest_value * 0.85  // 15% drop

  LOOP:
    IF current_value > highest_value:
      highest_value = current_value
      Update trailing_stop

    IF current_value < trailing_stop:
      → Sell 100% remaining
      → Lock in profits
```

**Why 15%?**
- Tight enough to protect profits
- Loose enough to ride momentum
- Prevents getting stopped out on noise

### Decision Point 4: Ladder Strategy
```
HIGH MOMENTUM + PROFITS:
  3x  → Sell 25% of remaining
  5x  → Sell 30% of remaining
  10x → Sell 30% of remaining
  20x → Sell 10% of remaining
  ∞   → Keep 5% moon bag
```

**Math Example**:
```
Entry: 0.025 SOL → 100,000 tokens

At 3x (0.075 SOL value):
  Sell 25,000 tokens → 0.01875 SOL

At 5x (0.125 SOL value):
  Sell 30,000 tokens → 0.0375 SOL

At 10x (0.25 SOL value):
  Sell 30,000 tokens → 0.075 SOL

At 20x (0.5 SOL value):
  Sell 10,000 tokens → 0.1 SOL

Keep 5,000 tokens forever (moon bag)

Total Sold: 0.32 SOL (12.8x profit!)
Still Hold: 5% for unlimited upside
```

## 🚨 Rug Detection System

### What We Monitor

#### 1. Whale Concentration
```
IF single holder > 50% supply:
  rug_risk += 0.4

IF top 5 holders > 80% supply:
  rug_risk += 0.3
```

#### 2. Liquidity Events
```
IF liquidity removed:
  rug_risk = 1.0
  → EMERGENCY EXIT

IF liquidity < initial:
  rug_risk += 0.3
```

#### 3. Dev Wallet Activity
```
IF dev selling > 10% supply:
  rug_risk += 0.4

IF dev wallet empty:
  rug_risk += 0.5
```

#### 4. Price Patterns
```
IF price drop > 30% in 10 seconds:
  rug_risk += 0.5

IF no buyers for 60 seconds:
  rug_risk += 0.3
```

### Emergency Exit Trigger
```
IF rug_risk > 0.7:
  → Sell 100% immediately
  → No questions asked
  → Accept whatever price
```

**Better to exit early on false alarm than lose 100%**

## 📈 Momentum Scoring Algorithm

### Inputs
```rust
volume_velocity: f64    // Buys per second
price_momentum: f64     // % change per second
holder_growth: f64      // New holders per second
buy_sell_ratio: f64     // Buys vs sells
average_buy_size: f64   // Size of buys
```

### Calculation
```rust
momentum_score = (
    volume_velocity * 0.3 +
    price_momentum * 0.25 +
    holder_growth * 0.2 +
    buy_sell_ratio * 0.15 +
    average_buy_size * 0.1
) / 1.0

// Normalized to 0.0 - 1.0
```

### Interpretation
```
0.0 - 0.3:  Low momentum (exit)
0.3 - 0.5:  Medium momentum (monitor)
0.5 - 0.8:  Good momentum (hold)
0.8 - 1.0:  Excellent momentum (ladder)
```

## 💰 Expected Performance

### Win/Loss Distribution
```
Scenario                Probability   Result
─────────────────────────────────────────────
Fast exit (no momentum)     40%      -$1 to -$2
Break even                  20%      $0 to $1
Small profit (2-5x)         25%      $5 to $20
Medium profit (5-10x)       10%      $20 to $45
Large profit (10x+)          5%      $45+
```

### Expected Value Per Trade
```
Expected Outcome = Sum(Probability × Result)

= (0.40 × -$1.50) +     // Fast exits
  (0.20 × $0.50) +      // Break even
  (0.25 × $12.50) +     // Small wins
  (0.10 × $32.50) +     // Medium wins
  (0.05 × $75.00)       // Large wins

= -$0.60 + $0.10 + $3.13 + $3.25 + $3.75
= +$9.63 per trade

With $5 per trade: 193% ROI
```

**Note**: These are estimates. Real performance will vary!

## 🎓 Key Lessons

### 1. Speed Matters Most
600ms can be the difference between 10x and missing entirely.

### 2. Cut Losses Fast
Most launches fail. Exit quickly when there's no momentum.

### 3. Recover Initial ASAP
Once you hit 2x, recover your initial + profit. Now you can't lose.

### 4. Ride Winners
Don't exit too early on winners. Let them run with trailing stops.

### 5. Watch for Rugs
Constant monitoring. Exit immediately on rug signals.

### 6. Small Size = Less Stress
$5 per trade means you can make decisions without emotion.

### 7. Verify Everything
Don't trust API responses. Check transactions on-chain.

## 🔄 Continuous Improvement

### Track These Metrics
```
- Entry timing (how fast?)
- Momentum accuracy (correct signals?)
- Exit timing (too early? too late?)
- Rug avoidance (detected?)
- Total P&L per day/week/month
```

### Optimize Based On Data
```
IF fast exits losing too much:
  → Reduce position size
  → Tighten entry criteria

IF missing big moves:
  → Widen trailing stops
  → Adjust ladder levels

IF getting rugged:
  → Improve detection
  → Exit faster on signals
```

## 🚀 Scaling Strategy

### Phase 1: Proof of Concept
```
Position Size: $5
Max Trades: 10/day
Manual Monitoring: Yes
Goal: Validate strategy
```

### Phase 2: Automation
```
Position Size: $5
Max Trades: 50/day
Manual Monitoring: Spot checks
Goal: Prove automation works
```

### Phase 3: Scale
```
Position Size: $10-25
Max Trades: 100/day
Manual Monitoring: Dashboard
Goal: Generate consistent profits
```

**Only scale after proven success at smaller size!**

## ⚠️ What Can Go Wrong

1. **Launch detection fails** → Miss opportunities
2. **Momentum signals wrong** → Bad exits
3. **Rug detection misses** → Lose money
4. **API failures** → Can't execute
5. **Network congestion** → Slow execution
6. **False success responses** → Think you're in but not

**Mitigation**: Start small, verify everything, improve continuously.
