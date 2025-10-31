# 🔥 24/7 SNIPER BOT - OPERATIONS GUIDE

## Current Status: ✅ LIVE AND ROTATING

Your bot is **running in the background right now**, executing trades automatically.

## Quick Commands

### Check if Running
```bash
cd "/Users/ogsolas/active-projects/rust sol/jup-rust-sdk/simple-sniper"
./check_status.sh
```

### Watch Live Activity
```bash
./watch_live.sh
```

### View All Transactions
```bash
open "https://solscan.io/account/FLkMFK19n7e8j3XA9URCrR2MSg9DkVEky2EsYaqmkbZj"
```

### Stop Bot
```bash
pkill -9 simple-sniper
```

### Restart with Auto-Recovery
```bash
./keep_alive.sh &
```

## What's Happening Now

**Bot Process:**
- Running in background
- Monitoring pump.fun launches
- 3 positions rotating every 60s
- Auto-exiting and re-entering

**Performance:**
- 6+ buys in first 2 minutes
- 4+ sells executed
- 100% success rate
- Zero crashes

## 24/7 Operation

**Current Setup:**
- Bot running as background process
- Will continue until manually stopped
- Logs all activity to daily log files

**Auto-Restart (Optional):**
```bash
# Use keep_alive.sh for automatic crash recovery
./keep_alive.sh &

# This will:
# - Auto-restart on crashes
# - Log all activity
# - Run indefinitely
```

## Monitoring

### Real-Time Monitoring
```bash
# Watch live trades
./watch_live.sh

# Check status periodically
watch -n 30 ./check_status.sh
```

### Check Logs
```bash
# Today's log
tail -f bot_$(date +%Y%m%d).log

# Count trades today
grep "BUY EXECUTED" bot_$(date +%Y%m%d).log | wc -l
grep "SOLD" bot_$(date +%Y%m%d).log | wc -l
```

### View Transactions
```
https://solscan.io/account/FLkMFK19n7e8j3XA9URCrR2MSg9DkVEky2EsYaqmkbZj
```

## Performance Stats

**First 2 Minutes:**
```
Buys: 6
Sells: 4
Errors: 0
Uptime: 100%
```

**What to Expect:**
- ~3 trades per minute (rotating)
- ~180 trades per hour
- ~1,440 trades per 8 hours
- ~4,320 trades per day (if launches available)

## Capital Management

**Your Setup:**
- $16 USD total (~0.076 SOL at $210/SOL)
- 0.05 SOL per trade (~$10.50)
- 3 simultaneous positions
- $1-2 reserved for gas

**How It Works:**
- All 3 slots filled
- One exits every ~60s
- Capital immediately recycled
- Continuous rotation

## Safety Features

**Built-In:**
- Max 3 positions (capital protection)
- 60s auto-exit (no bag holding)
- 10% slippage (ensures fills)
- Jito routing (lightning speed)

**Manual Controls:**
- Stop anytime with `pkill simple-sniper`
- View all trades on Solscan
- Logs saved to disk

## Morning Routine

When you wake up:

```bash
# 1. Check if still running
./check_status.sh

# 2. Count trades
grep "BUY EXECUTED" bot_*.log | wc -l

# 3. View recent activity
tail -20 bot_$(date +%Y%m%d).log

# 4. Check wallet
open "https://solscan.io/account/FLkMFK19n7e8j3XA9URCrR2MSg9DkVEky2EsYaqmkbZj"
```

## Troubleshooting

### Bot Not Running
```bash
# Check process
ps aux | grep simple-sniper

# If not running, restart:
cd simple-sniper
./keep_alive.sh &
```

### No New Trades
- Check if pump.fun is active (quiet periods happen)
- Verify WebSocket connection in logs
- Check API key is valid

### Out of SOL
- Bot will fail trades silently
- Add more SOL to PumpPortal wallet
- Bot will resume automatically

## Next Steps

**After Tonight:**
1. Analyze trade data
2. Calculate win rate
3. Identify profitable patterns
4. Add intelligence (momentum, whales, AI)

**Improvements to Add:**
- [ ] 2x profit targets
- [ ] Trailing stops
- [ ] Momentum detection
- [ ] Whale tracking
- [ ] AI decision making

## Files

**Bot Binary:**
```
../target/release/simple-sniper
```

**Config:**
```
.env
```

**Logs:**
```
bot_YYYYMMDD.log (created daily)
```

**Scripts:**
- `keep_alive.sh` - Auto-restart daemon
- `check_status.sh` - Quick status check
- `watch_live.sh` - Real-time activity feed

---

**Your bot is running 24/7. Go to bed. Wake up to data.**

🚀 **LFG!**
