# 📊 Pump.fun Sniper Monitor Dashboard

Simple HTML/CSS monitoring dashboard for the pump.fun sniper bot.

## Features

✅ **Live Position Tracking**
- Monitor up to 3 active positions in real-time
- Current P&L for each position (SOL and %)
- Entry price, current value, token holdings

✅ **Overall Statistics**
- Total P&L across all positions
- Unrealized P&L from active positions
- Active position count
- Historical win rate

✅ **DeepSeek AI Stream**
- Live stream of AI decision reasoning
- Confidence scores for each decision
- Action recommendations (Hold, Exit, Trail)

✅ **Control Panel**
- ▶ **START**: Begin trading operations
- ⏸ **PAUSE**: Temporarily halt trading
- 🚨 **SELL ALL**: Emergency panic button (sells all positions)

## Quick Start

### 1. Start the Monitor Server

```bash
cd monitor
python3 server.py
```

**Server will run on**: http://localhost:8080

### 2. Open Dashboard

Open in your browser:
```
http://localhost:8080/dashboard.html
```

Or simply navigate to:
```
http://localhost:8080
```

### 3. Auto-Updates

Dashboard auto-refreshes every 3 seconds with:
- Position data from SQLite database
- AI decision stream
- Overall statistics

## API Endpoints

The Python server exposes these endpoints:

### GET /api/positions
Returns active positions with P&L:
```json
[
  {
    "mint": "Ggoa...pump",
    "entry_sol": 0.33,
    "current_value": 0.66,
    "pnl_sol": 0.33,
    "pnl_percent": 100.0,
    "tokens": 1000000,
    "entry_time": "14:32:15"
  }
]
```

### GET /api/stats
Returns overall statistics:
```json
{
  "total_pnl": 0.45,
  "unrealized_pnl": 0.45,
  "active_positions": 3,
  "win_rate": 65.5,
  "total_closed": 12
}
```

### GET /api/ai-stream
Returns recent AI decisions:
```json
[
  {
    "mint": "Ggoa...",
    "action": "ExitPartial(45%)",
    "confidence": 0.85,
    "reasoning": "2.1x with strong momentum (0.82)...",
    "time": "14:35:22"
  }
]
```

### POST /api/control/start
Starts the bot (placeholder for future integration)

### POST /api/control/pause
Pauses the bot (placeholder for future integration)

### POST /api/control/sell-all
Initiates emergency sell of all positions (placeholder)

## Database Connection

**Database Path**: `../pump-sniper-bot/sniper_bot.db`

The server reads from these tables:
- `positions` - Active and closed positions
- `ai_decisions` - AI decision history
- `momentum_snapshots` - Historical momentum data

## Styling

**Theme**: Cyberpunk terminal
- Background: Dark blue (#0a0e27)
- Primary: Neon green (#00ff88)
- Warning: Orange (#ffaa00)
- Danger: Red (#ff0055)

**Font**: Courier New (monospace)

## Technical Details

### Server
- **Language**: Python 3
- **Framework**: Built-in http.server
- **Database**: SQLite3
- **Port**: 8080
- **CORS**: Enabled for local development

### Frontend
- **HTML5** with semantic markup
- **CSS3** Grid and Flexbox layouts
- **Vanilla JavaScript** (no frameworks)
- **Auto-refresh**: 3-second intervals
- **Responsive**: Mobile-friendly grid

## Usage Example

### Normal Operation
1. Start main bot: `cd pump-sniper-bot && cargo run`
2. Start AI demon: `cd sniper-demon && cargo run`
3. Start monitor: `cd monitor && python3 server.py`
4. Open dashboard in browser

### Monitor Display
```
🎯 Pump.fun Sniper Monitor
━━━━━━━━━━━━━━━━━━━━━━━━

┌─────────────────────────────────────┐
│ Total P&L      │ +0.45 SOL          │
│ Unrealized P&L │ +0.45 SOL          │
│ Active Pos.    │ 3                  │
│ Win Rate       │ 65.5%              │
└─────────────────────────────────────┘

▶ START    ⏸ PAUSE    🚨 SELL ALL

┌─────────────────────────────────────┐
│ 📊 Active Positions                 │
│                                     │
│ Ggoa...pump              14:32:15   │
│ Entry: 0.33 SOL   Current: 0.66    │
│ Tokens: 1,000,000   P&L: +0.33 SOL │
│           +100.0%                   │
└─────────────────────────────────────┘

┌─────────────────────────────────────┐
│ 🧠 DeepSeek AI Stream               │
│                                     │
│ Ggoa... ExitPartial(45%) 85%        │
│ "2.1x with strong momentum (0.82)  │
│  but moderate rug risk (0.45).     │
│  Exit 45% to secure 110% recovery  │
│  per Rule #9, hold 55% with 7.5%   │
│  trailing stop. Momentum justifies │
│  holding majority."                 │
└─────────────────────────────────────┘
```

## Security Notes

⚠️ **Local Use Only**: This server is designed for local monitoring
⚠️ **No Authentication**: Do not expose to the internet
⚠️ **Control Buttons**: Currently placeholders, need bot integration

## Future Enhancements

- [ ] WebSocket for real-time updates (vs. polling)
- [ ] Control button integration with bot process
- [ ] Historical P&L charts
- [ ] Trade history timeline
- [ ] Alert notifications (browser notifications)
- [ ] Dark/light theme toggle
- [ ] Position detail modal

## Troubleshooting

### Port Already in Use
```bash
# Kill existing server
lsof -ti:8080 | xargs kill -9

# Or use different port
python3 server.py 8081
```

### Database Not Found
Ensure the bot has created the database:
```bash
cd pump-sniper-bot
cargo run  # Creates sniper_bot.db
```

### No Data Showing
1. Check bot is running and creating positions
2. Check database path in server.py
3. Check browser console for errors
4. Verify API endpoints: `curl http://localhost:8080/api/stats`

---

**Dashboard is ready!** 🚀

Start the server and watch your positions in real-time.
