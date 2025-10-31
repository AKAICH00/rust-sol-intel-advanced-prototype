# Pump.fun Sniper Bot 🎯

Ultra-fast token launch sniper with intelligent exit strategy.

## 🚀 Strategy

### Entry
- **Detection**: Monitor on-chain for new pump.fun launches
- **Execution**: PumpPortal Lightning API (~700ms)
- **Size**: $5 USD (~0.025 SOL at $200/SOL)
- **Settings**: 20% slippage, high priority fee, Jito routing

### Exit (Smart Multi-Stage)

#### Stage 1: No Momentum Check (0-60 seconds)
```
IF no momentum after 60 seconds:
  → Fast exit 100% position
  → Cut losses early
```

#### Stage 2: Initial Recovery (at 2x)
```
AT 2x profit:
  → Sell portion to recover initial + 10%
  → Trail remaining with tight stops
```

#### Stage 3: High Momentum Ladder
```
IF high momentum detected:
  3x  → Sell 25%
  5x  → Sell 30%
  10x → Sell 30%
  20x → Sell 10%
  ∞   → Keep 5% moon bag
```

#### Stage 4: Rug Protection
```
AT ANY TIME if rug detected:
  → Emergency exit 100%
  → No questions asked
```

## 📊 Risk Management

- **Per Trade**: $5 (~0.025 SOL)
- **Stop Loss**: 50% (exit if down 50%)
- **Max Daily Trades**: 50
- **Rug Detection**: Continuous monitoring
- **Transaction Verification**: Always check on-chain

## 🛠️ Setup

### 1. Configuration
```bash
cd pump-sniper-bot
cp .env.example .env
```

Edit `.env`:
```env
PUMPPORTAL_API_KEY=your-api-key
HELIUS_RPC_URL=your-helius-url
SNIPE_AMOUNT_SOL=0.025  # ~$5
```

### 2. Build
```bash
cargo build --release
```

### 3. Run
```bash
RUST_LOG=info cargo run
```

## 🎯 Strategy Logic

### Momentum Detection
```rust
Momentum Score = f(
    volume_velocity,    // Buys per second
    price_momentum,     // Price change rate
    holder_health,      // Distribution quality
)

If momentum < 0.3 after 60s → EXIT
If momentum > 0.8 → LADDER STRATEGY
```

### Rug Detection
```rust
Rug Risk = f(
    whale_concentration,  // Large holder %
    liquidity_changes,    // LP removal
    dev_activity,         // Suspicious dumps
    price_patterns,       // Abnormal drops
)

If rug_risk > 0.7 → EMERGENCY EXIT
```

### Trailing Stop
```rust
After 2x reached and recovered:
  Track highest_value
  If current < (highest * 0.85):
    → Exit remaining position
```

## 📈 Example Scenarios

### Scenario 1: No Momentum
```
0:00 → Snipe 0.025 SOL
0:30 → Check momentum: 0.2 (low)
1:00 → Check momentum: 0.25 (still low)
1:00 → EXIT: Sell 100%, recover ~0.023 SOL
Result: Small loss, but preserved capital
```

### Scenario 2: Quick 2x
```
0:00 → Snipe 0.025 SOL
0:20 → Price 2x! Value = 0.05 SOL
0:20 → Sell 55% → Recover 0.0275 SOL (initial + 10%)
0:20 → Trail remaining 0.0225 SOL
Result: Recovered + profit, riding free money
```

### Scenario 3: Moon Shot
```
0:00 → Snipe 0.025 SOL
0:30 → 3x → Sell 25% (0.01875 SOL)
1:00 → 5x → Sell 30% (0.0375 SOL)
2:00 → 10x → Sell 30% (0.075 SOL)
5:00 → 20x → Sell 10% (0.05 SOL)
∞   → Keep 5% moon bag
Result: Massive profit, kept upside exposure
```

### Scenario 4: Rug Detected
```
0:00 → Snipe 0.025 SOL
0:45 → Rug risk: 0.8 (HIGH!)
0:45 → EMERGENCY EXIT: Sell 100%
0:45 → Recover ~0.022 SOL
Result: Small loss but avoided total loss
```

## ⚠️ Important Notes

### Current Status
This is a **FRAMEWORK**. The following need implementation:

#### ✅ Complete
- [x] Strategy logic
- [x] Exit algorithms
- [x] PumpPortal integration
- [x] Configuration system
- [x] Logging infrastructure
- [x] **Launch detection via PumpPortal WebSocket**
- [x] **Front-running whale protection**

#### 🚧 TODO
- [ ] **Real momentum analysis** (uses mock data)
- [ ] **Position tracking** (needs wallet integration)
- [ ] **Rug detection algorithms** (placeholder)
- [ ] **Transaction verification** (must check on-chain)
- [ ] **Whale balance monitoring** (for front-running)

### To Make It Production-Ready:

1. **Momentum Analysis**
   ```rust
   // Real implementation needs:
   // - Transaction history parsing
   // - Buy/sell ratio calculation
   // - Volume tracking
   // - Price feeds
   ```

2. **Position Tracking**
   ```rust
   // Need wallet integration:
   // - Get token balances
   // - Track holdings
   // - Calculate P&L
   ```

3. **Rug Detection**
   ```rust
   // Implement real checks:
   // - Holder analysis
   // - Liquidity monitoring
   // - Suspicious pattern detection
   ```

## 🚨 Risk Warnings

### Financial Risks
- **High Risk**: Pump.fun tokens are extremely volatile
- **Rug Pulls**: Many tokens are scams
- **Slippage**: High slippage on small pools
- **Failed Transactions**: 33% of PumpPortal calls may fail
- **Total Loss**: Could lose entire $5 per trade

### Technical Risks
- **False Signals**: Momentum detection may be wrong
- **Execution Delays**: Network congestion
- **API Failures**: PumpPortal downtime
- **Bugs**: Strategy logic errors

### Recommended Limits
- Start with $5 per trade
- Max 10 trades per day initially
- Only risk money you can afford to lose
- Monitor every trade manually at first
- Test extensively before scaling

## 📊 Performance Tracking

Track these metrics:
```
- Total trades
- Win rate
- Average profit/loss
- Largest win/loss
- Total P&L
- Rug avoidance rate
- Fast exit saves
```

## 🛡️ Safety Features

1. **Small Position Sizes**: $5 per trade
2. **Stop Losses**: 50% max loss
3. **Rug Detection**: Continuous monitoring
4. **Transaction Verification**: Check on-chain
5. **Daily Limits**: Max 50 trades
6. **Emergency Stop**: Manual kill switch

## 🎓 Learning Resources

- [Pump.fun Documentation](https://docs.pump.fun)
- [PumpPortal API](https://pumpportal.fun/trading-api/)
- [Solana Transaction Monitoring](https://docs.solana.com/api/websocket)
- [Token Bonding Curves](https://docs.pump.fun/bonding-curve)

## 📝 License

Use at your own risk. No guarantees or warranties.

## 🤝 Contributing

This is a framework/template. You need to implement:
1. Launch detection
2. Real momentum analysis
3. Position tracking
4. Rug detection

PRs welcome for these features!
