# 🎯 SIMPLE SNIPER BOT - READY TO DEGEN!

## 🚀 Quick Start (YOUR $16 WAITING TO HUNT)

```bash
cd "/Users/ogsolas/active-projects/rust sol/jup-rust-sdk"
RUST_LOG=info ./target/release/simple-sniper
```

**That's it!** The bot will:
1. Connect to PumpPortal WebSocket
2. Watch for NEW pump.fun token launches
3. BUY IMMEDIATELY with 0.05 SOL (~3 trades with your $16)
4. Auto-sell after 60 seconds (momentum strategy)

## 💰 Your Config

```
Snipe Amount: 0.05 SOL per trade
Max Positions: 3 trades max
Strategy: Fast in → 60s momentum check → Exit
Jito Routing: ENABLED (Lightning fast ~700ms)
```

## 🎮 What You'll See

```
🚀 SIMPLE SNIPER BOT - LFG!
💰 Config:
   Snipe Amount: 0.05 SOL per trade
   Max Positions: 3
   Strategy: Buy launches → 2x exit → Repeat
📡 Connecting to PumpPortal WebSocket...
✅ Subscribed to new token launches

🎯 WATCHING FOR LAUNCHES... Press Ctrl+C to stop

🔔 NEW LAUNCH DETECTED!
   Mint: Ggoa...pump
✅ BUY EXECUTED!
   Signature: 4xH2...
   Amount: 0.05 SOL
💼 Positions: 1/3 (2 slots left)
```

## 📊 How It Works

1. **Launch Detection**: Connects to PumpPortal WebSocket feed
2. **Instant Buy**: Buys with Jito routing for speed (~700ms execution)
3. **Position Tracking**: Monitors all open positions every 10 seconds
4. **Auto-Exit**: Sells positions older than 60 seconds
5. **Repeat**: Frees up slot for next launch!

## ⚡ Features

- **Jito Priority**: Lightning fast execution
- **Max 3 Positions**: Never overextend
- **Auto Exit**: No babysitting needed
- **Simple Strategy**: Time-based momentum
- **Real Money**: Connected to YOUR wallet

## 🛑 To Stop

Just press `Ctrl+C` in the terminal

## 🔥 Pro Tips

1. **Watch the first trade** to see how fast it executes
2. **Pump.fun launches** happen frequently - you'll catch one soon!
3. **60-second exits** are automatic - bot handles it
4. **3 max positions** = never over-leveraged

## 📁 Files

- Binary: `/Users/ogsolas/active-projects/rust sol/jup-rust-sdk/target/release/simple-sniper`
- Config: `/Users/ogsolas/active-projects/rust sol/jup-rust-sdk/simple-sniper/.env`
- Source: `/Users/ogsolas/active-projects/rust sol/jup-rust-sdk/simple-sniper/src/main.rs`

## 🎯 Overnight Mode

Want to leave it running overnight? Use tmux or screen:

```bash
# Start tmux session
tmux new -s sniper

# Run bot
cd "/Users/ogsolas/active-projects/rust sol/jup-rust-sdk"
RUST_LOG=info ./target/release/simple-sniper

# Detach: Ctrl+B then D
# Reattach later: tmux attach -t sniper
```

## ⚙️ Adjust Settings

Edit `.env` file in `simple-sniper/` folder:

```bash
# Trade bigger
SNIPE_AMOUNT_SOL=0.1

# More positions
MAX_POSITIONS=5
```

---

**LFG! YOUR DEGEN SNIPER IS READY! 🚀**

Just run the command above and watch it hunt!
