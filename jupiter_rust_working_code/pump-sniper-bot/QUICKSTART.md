# Quick Start Guide

## Test the Launch Detector (Safe - No Trading)

### 1. Build the Example
```bash
cd /Users/ogsolas/active-projects/rust\ sol/jup-rust-sdk
cargo build --bin sniper_bot_live
```

### 2. Run the Live Monitor
```bash
RUST_LOG=info cargo run --bin sniper_bot_live
```

### 3. Watch for Launches
You'll see output like this when new tokens are created:

```
🔍 Starting PumpPortal launch detector...
   WebSocket: wss://pumpportal.fun/api/data
Connecting to PumpPortal WebSocket...
✅ Connected to PumpPortal
📡 Subscribed to new token events
👀 Monitoring for new pump.fun launches...

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🚀 NEW LAUNCH DETECTED
   Name: Cool Token (COOL)
   Mint: GgoaCoyqvnDE5KGLSpLPiyNVGeF8w8rm8b1Hd9JFpump
   Creator: 8x7kN...
   Market Cap: $5000.00
   🔗 https://pump.fun/GgoaCoyqvnDE5KGLSpLPiyNVGeF8w8rm8b1Hd9JFpump
   ✅ Token is snipeable!

💡 This token would be sniped automatically
   Entry: 0.025 SOL
   Expected execution: ~700ms via PumpPortal
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

👀 Monitoring for next launch...
```

**This is 100% SAFE** - it only monitors, does NOT execute any trades.

---

## What You're Seeing

### Launch Rate
Pump.fun launches vary by time of day and market conditions:
- **Busy**: 50-200 launches/hour
- **Normal**: 10-50 launches/hour
- **Slow**: 5-20 launches/hour

Most active during US market hours (9am-5pm EST).

### Snipeability Filter
You'll see two types of launches:

**✅ Snipeable** (will trade):
- Fresh token (not graduated)
- Has name and symbol
- Not marked NSFW
- Valid metadata

**⚠️ Not Snipeable** (will skip):
- Already graduated to Raydium
- Marked as NSFW
- Missing metadata
- Suspicious

Most launches (60-80%) are filtered out.

---

## Testing the Full Bot (With Trading)

⚠️ **WARNING: This uses REAL MONEY!**

### 1. Set Up Environment
```bash
cd pump-sniper-bot
cp .env.example .env
```

Edit `.env`:
```bash
PUMPPORTAL_API_KEY=your-api-key-here
HELIUS_RPC_URL=your-helius-url-here
SNIPE_AMOUNT_SOL=0.001  # Start VERY small!
```

### 2. Test With Tiny Amounts First
```bash
# $0.20 per trade at $200/SOL
SNIPE_AMOUNT_SOL=0.001 RUST_LOG=info cargo run
```

### 3. Monitor Results
The bot will:
1. Detect new launches
2. Execute buy orders (~$0.20 each)
3. Monitor for 60 seconds
4. Exit based on strategy

**Watch the console output carefully!**

---

## Current Limitations

### ⚠️ What Works
- ✅ Launch detection (real-time)
- ✅ Buy execution (~700ms)
- ✅ Strategy logic
- ✅ Time-based exits

### ⚠️ What Doesn't Work Yet
- ❌ Real momentum analysis (uses mock data)
- ❌ Position tracking (can't verify balance)
- ❌ Rug detection (placeholder)
- ❌ Transaction verification (trusts API)
- ❌ Whale monitoring (not active)

**Translation**: The bot will:
- Detect and buy new tokens ✅
- Exit after 60s regardless of performance ❌
- Not know actual position value ❌
- Not detect rugs until manual check ❌

**DO NOT USE SIGNIFICANT AMOUNTS** until remaining components are implemented.

---

## Recommended Testing Progression

### Phase 1: Monitoring Only (Current)
```bash
cargo run --bin sniper_bot_live
```
- Monitor launches for 1-2 hours
- Note launch frequency
- Watch snipeability filtering
- **NO RISK**

### Phase 2: Micro Trading
```bash
SNIPE_AMOUNT_SOL=0.001 cargo run  # $0.20 per trade
```
- Execute 5-10 real snipes
- Manually check positions on pump.fun
- Manually verify transactions on Solscan
- Track outcomes in spreadsheet
- **MINIMAL RISK** ($1-2 total)

### Phase 3: Small Trading (After Position Tracking)
```bash
SNIPE_AMOUNT_SOL=0.005 cargo run  # $1 per trade
```
- Execute 10-20 real snipes
- Bot tracks positions automatically
- Compare bot tracking vs actual
- Refine strategy based on results
- **LOW RISK** ($10-20 total)

### Phase 4: Normal Trading (After All Components)
```bash
SNIPE_AMOUNT_SOL=0.025 cargo run  # $5 per trade
```
- Full bot operation
- All safety systems active
- Monitor performance metrics
- Adjust strategy parameters
- **MODERATE RISK** (as designed)

---

## Monitoring and Control

### Stop the Bot
Press `CTRL+C` at any time to gracefully stop.

### Check Recent Trades
PumpPortal dashboard: https://pumpportal.fun/dashboard

### Verify Positions
1. Go to https://pump.fun
2. Connect wallet
3. Check "Your Tokens" tab
4. Compare with bot logs

### Check Transactions
Format: `https://solscan.io/tx/{signature}`

Bot logs signature after each trade.

---

## Expected Outcomes (Once Complete)

### Typical Session (1 hour)
- Launches detected: 10-50
- Snipeable launches: 2-10
- Trades executed: 2-10
- Exits at 2x+: 1-3 (20-30%)
- Exits at breakeven: 2-4 (30-40%)
- Exits at loss: 3-5 (40-50%)

### Performance Targets
- Win rate: 30-40%
- Average win: 2-5x
- Average loss: -50% to -80%
- Net expectation: Slightly positive

**Reality**: Most pump.fun tokens fail. The strategy is designed to:
1. Catch occasional winners
2. Exit losers quickly
3. Protect against rugs
4. Keep moon bags for rare moonshots

---

## Troubleshooting

### No Launches Appearing
- Check internet connection
- Verify PumpPortal API is online
- Try during US market hours
- Wait 5-10 minutes (launches are sporadic)

### WebSocket Disconnects
- Normal behavior
- Bot auto-reconnects in 5 seconds
- Check logs for reconnection messages

### Build Errors
```bash
# Update dependencies
cargo update

# Clean build
cargo clean
cargo build
```

### Runtime Errors
Check `.env` file:
- Valid API key
- Valid RPC URL
- Numeric SNIPE_AMOUNT_SOL

---

## Next Steps

1. **Now**: Test launch detector (safe monitoring)
2. **Next**: Implement position tracking
3. **Then**: Implement transaction verification
4. **Finally**: Implement momentum + rug detection

See [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) for detailed roadmap.

---

## Questions?

Check the documentation:
- [README.md](README.md) - Overview and strategy
- [STRATEGY.md](STRATEGY.md) - Detailed strategy explanation
- [FRONTRUN_STRATEGY.md](FRONTRUN_STRATEGY.md) - Whale protection
- [LAUNCH_DETECTION.md](LAUNCH_DETECTION.md) - Launch detector details
- [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) - What's done and what's left

Or review the code:
- [src/launch_detector.rs](src/launch_detector.rs) - Launch detection
- [src/strategy.rs](src/strategy.rs) - Strategy logic
- [src/frontrun.rs](src/frontrun.rs) - Front-running protection
- [examples/src/sniper_bot_live.rs](../examples/src/sniper_bot_live.rs) - Test example
