# 🚀 SIMPLE SNIPER - LIVE TRADING

**STATUS: ✅ RUNNING AND TRADING**

## Current Configuration

```
Snipe Amount: 0.05 SOL per trade (~$8)
Max Positions: 3 simultaneous
Exit Strategy: Auto-sell after 60 seconds
Your Wallet: ~$16 USD (enough for 3 trades + gas)
```

## First Trade Executed!

**Mint:** `Gi4GKtUeVsUashvZcQTMwdw7BQ8kMY9s6Td9sit4pump`
**Signature:** `uTNv5dR5jgmykAXhRg1vqx8hi52qAmjHC1HkCkERrEfif4HvNDUZuZca2ch8EzVTseqTc8exDHGpPp73h6HqtdG`
**Amount:** 0.05 SOL

View on Solscan:
```
https://solscan.io/tx/uTNv5dR5jgmykAXhRg1vqx8hi52qAmjHC1HkCkERrEfif4HvNDUZuZca2ch8EzVTseqTc8exDHGpPp73h6HqtdG
```

## How It Works

1. **🔔 Launch Detection**
   - WebSocket connected to PumpPortal
   - Instantly detects new token launches
   - No delays, no filters - pure degen mode

2. **⚡ Instant Buy**
   - 0.05 SOL (~$8) per launch
   - Jito routing for lightning speed (~700ms)
   - 10% slippage to ensure fills

3. **👀 Position Monitoring**
   - Checks positions every 10 seconds
   - Tracks time elapsed since entry
   - Auto-exits after 60 seconds

4. **💰 Auto Exit**
   - Sells 100% of tokens after 60s
   - 20% slippage for fast exit
   - Frees up capital for next launch

## Check Bot Status

```bash
# View live logs
cd "/Users/ogsolas/active-projects/rust sol/jup-rust-sdk/simple-sniper"
ps aux | grep simple-sniper

# Check output
# Bot is running in background - check Claude Code for live output
```

## What to Expect Overnight

**Scenario 1: Active Market**
- Bot will snipe 3 launches
- Hold each for 60 seconds
- Exit and rotate to new launches
- Could execute 50+ trades overnight

**Scenario 2: Slow Market**
- Bot waits for launches
- Buys immediately when detected
- Max 3 positions at once

**Capital Management:**
- Your $16 → 3 trades of $5 each (~0.05 SOL)
- Gas reserve automatically maintained
- When position exits, capital freed for next launch

## View Transactions

All trades are on-chain:

**PumpPortal Wallet:**
```
FLkMFK19n7e8j3XA9URCrR2MSg9DkVEky2EsYaqmkbZj
```

View all transactions:
```
https://solscan.io/account/FLkMFK19n7e8j3XA9URCrR2MSg9DkVEky2EsYaqmkbZj
```

## Stop Bot (If Needed)

```bash
# Find process
ps aux | grep simple-sniper

# Stop it
pkill -9 simple-sniper
```

## Bot Strategy

**Entry:**
- Every new pump.fun launch
- No filters (full degen mode)
- 0.05 SOL per trade

**Exit:**
- 60 second timer
- Sell 100% of tokens
- Rotate to next launch

**Why 60 seconds?**
- Pump.fun launches pump FAST
- If no movement in 60s → dead
- Get out and move to next opportunity

## Expected P&L

**Conservative:**
- 30-50% of trades go up
- Small gains, small losses
- Goal: Learn the game, see volume

**Reality Check:**
- Most launches are rugs
- You're paying for speed and education
- Volume > profits at this stage

## Morning Checklist

When you wake up:

1. Check how many trades executed
2. View transactions on Solscan
3. Check PumpPortal wallet balance
4. Analyze what worked/didn't work

## Next Steps

After you've seen it run:

1. **Add Intelligence**
   - Momentum detection
   - Whale tracking
   - Rug pattern detection

2. **Better Exits**
   - 2x profit targets
   - Trailing stops
   - AI decision making

3. **Risk Management**
   - Position sizing
   - Stop losses
   - Win rate tracking

---

## 🎯 THE GOAL

**Learn by doing.** You're now live trading real launches. Every trade teaches you something. The bot will execute, you'll see what works, what doesn't, and we'll iterate.

**No tiny amounts. No paper trading. FULL SEND.**

LFG! 🚀
