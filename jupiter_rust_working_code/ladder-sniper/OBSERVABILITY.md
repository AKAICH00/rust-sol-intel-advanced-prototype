# 📊 Complete Observability Guide
## Real-Time Monitoring for $1M Research Mission

**Easy visibility for both humans and AI agents**

---

## 🎯 Quick Start

### Option 1: Live Dashboard (Human-Friendly)
```bash
cd ladder-sniper
../target/release/monitor
```

**What you see:**
- Updates every 5 seconds
- Clear formatted output
- Win rate, P&L, holder analysis
- Time since last trade
- Auto-refreshing terminal display

**Controls:**
- Press `Ctrl+C` to stop
- Runs indefinitely while bot trades

---

### Option 2: JSON Mode (Bot/Agent-Friendly)
```bash
../target/release/monitor --json
```

**Output Example:**
```json
{
  "timestamp": "2025-10-30T20:15:00Z",
  "total_trades": 142,
  "open_positions": 3,
  "closed_positions": 68,
  "wins": 45,
  "losses": 23,
  "win_rate_pct": 66.2,
  "total_pnl_sol": 12.5473,
  "avg_pnl_pct": 18.4,
  "avg_hold_secs": 28.3,
  "avg_entry_holders": 127,
  "avg_exit_holders": 184,
  "last_trade_ago_secs": 12
}
```

**Use case:**
- Parse with `jq`
- Feed to AI agents
- Automated monitoring scripts
- Alerting systems

---

### Option 3: Static Report (Detailed Analysis)
```bash
../target/release/analytics
```

**What you get:**
- Complete performance summary
- Top 5 winners and losers
- Holder count correlation
- Time-series snapshot count
- One-time detailed report

---

## 🔧 Advanced Usage

### Custom Update Interval
```bash
# Update every 10 seconds
../target/release/monitor --interval 10

# Update every 1 second (high frequency)
../target/release/monitor --interval 1
```

### Custom Database Path
```bash
../target/release/monitor /path/to/custom.duckdb
../target/release/analytics /path/to/custom.duckdb
```

### JSON + Custom Interval
```bash
# Get JSON snapshot every 30 seconds in a loop
while true; do
    ../target/release/monitor --json
    sleep 30
done
```

---

## 📡 Bot Agent Integration

### Python Bot Example
```python
import subprocess
import json

def get_live_stats():
    result = subprocess.run(
        ['../target/release/monitor', '--json'],
        capture_output=True,
        text=True
    )
    return json.loads(result.stdout)

# Get stats
stats = get_live_stats()
print(f"Win Rate: {stats['win_rate_pct']}%")
print(f"Total P&L: {stats['total_pnl_sol']} SOL")
print(f"Avg Entry Holders: {stats['avg_entry_holders']}")

# Alert if win rate drops
if stats['win_rate_pct'] < 60 and stats['closed_positions'] > 20:
    print("⚠️  WIN RATE ALERT: Below 60%")
```

### Node.js Bot Example
```javascript
const { exec } = require('child_process');
const { promisify } = require('util');
const execAsync = promisify(exec);

async function getLiveStats() {
    const { stdout } = await execAsync('../target/release/monitor --json');
    return JSON.parse(stdout);
}

// Monitor continuously
setInterval(async () => {
    const stats = await getLiveStats();
    console.log(`Win Rate: ${stats.win_rate_pct}%`);
    console.log(`P&L: ${stats.total_pnl_sol} SOL`);

    // Alert conditions
    if (stats.last_trade_ago_secs > 300) {
        console.warn('⚠️  No trades in 5 minutes');
    }
}, 5000);
```

### Shell Script Monitoring
```bash
#!/bin/bash
# monitor.sh - Continuous monitoring with alerts

while true; do
    STATS=$(../target/release/monitor --json)
    WIN_RATE=$(echo "$STATS" | jq -r '.win_rate_pct')
    TOTAL_PNL=$(echo "$STATS" | jq -r '.total_pnl_sol')
    LAST_TRADE=$(echo "$STATS" | jq -r '.last_trade_ago_secs')

    # Alert if no trades in 5 minutes
    if [ "$LAST_TRADE" -gt 300 ]; then
        echo "⚠️  ALERT: No trades in 5 minutes" | tee -a alerts.log
    fi

    # Alert if win rate drops below 60%
    if (( $(echo "$WIN_RATE < 60" | bc -l) )); then
        echo "⚠️  ALERT: Win rate below 60%: $WIN_RATE%" | tee -a alerts.log
    fi

    sleep 10
done
```

---

## 📊 Multi-Terminal Setup

**Recommended 3-terminal layout:**

### Terminal 1: Trading Bot
```bash
cd ladder-sniper
RUST_LOG=info ../target/release/ladder-sniper
```

### Terminal 2: Live Monitor
```bash
cd ladder-sniper
../target/release/monitor
```

### Terminal 3: DuckDB SQL Queries
```bash
duckdb ladder-sniper/data/research.duckdb
```
Then run any queries from [RESEARCH_QUERIES.md](RESEARCH_QUERIES.md)

---

## 🔔 Alert Strategies

### Win Rate Alerts
```bash
# Alert if win rate < 55% after 30+ trades
WIN_RATE=$(../target/release/monitor --json | jq -r '.win_rate_pct')
CLOSED=$(../target/release/monitor --json | jq -r '.closed_positions')

if [ "$CLOSED" -gt 30 ] && (( $(echo "$WIN_RATE < 55" | bc -l) )); then
    # Send alert (Slack, email, etc.)
    curl -X POST https://hooks.slack.com/... \
        -d "{\"text\":\"⚠️ Win rate: $WIN_RATE%\"}"
fi
```

### Holder Count Anomalies
```sql
-- Run in DuckDB to find unusual holder patterns
SELECT
    mint,
    holder_count_entry,
    holder_count_exit,
    pnl_percent,
    CASE
        WHEN holder_count_exit > holder_count_entry * 2 THEN '🚀 Viral'
        WHEN holder_count_exit < holder_count_entry * 0.5 THEN '📉 Dump'
        ELSE 'Normal'
    END as pattern
FROM positions
WHERE exit_time_micros IS NOT NULL
    AND ABS(holder_count_exit - holder_count_entry) > 100
ORDER BY ABS(holder_count_exit - holder_count_entry) DESC
LIMIT 10;
```

### P&L Alerts
```bash
# Alert on major wins (>50% gain)
duckdb ladder-sniper/data/research.duckdb <<EOF
SELECT
    mint,
    pnl_percent,
    pnl_sol,
    holder_count_entry,
    holder_count_exit
FROM positions
WHERE pnl_percent > 50
ORDER BY pnl_percent DESC
LIMIT 5;
EOF
```

---

## 📈 Performance Metrics

### Key Metrics Explained

**`win_rate_pct`**: Percentage of profitable positions
- Target: >60% for viable strategy
- <50% = losing strategy

**`avg_pnl_pct`**: Average return per position
- Target: >10% to offset fees and losses
- Should account for 1% trade fees

**`avg_hold_secs`**: Average position duration
- Current strategy: 10-60s exits
- Useful for timing optimization

**`avg_entry_holders` vs `avg_exit_holders`**:
- Growth = community interest
- Decline = early exit signal
- Correlation with success = research goal

**`last_trade_ago_secs`**: Time since last activity
- >300s (5min) = possible issue
- Normal: 10-60s between trades

---

## 🎛️ Real-Time Commands

### Quick Status Check
```bash
# One-line status
../target/release/monitor --json | jq -r '"WR: \(.win_rate_pct)% | P&L: \(.total_pnl_sol) SOL | Trades: \(.total_trades)"'
```

### Export Current Stats to File
```bash
# Snapshot every minute
while true; do
    TIMESTAMP=$(date +%s)
    ../target/release/monitor --json > "snapshots/stats_$TIMESTAMP.json"
    sleep 60
done
```

### Watch Specific Metrics
```bash
# Watch only win rate and P&L
watch -n 5 "../target/release/monitor --json | jq '{win_rate_pct, total_pnl_sol, closed_positions}'"
```

---

## 🗄️ Database Direct Access

### DuckDB CLI
```bash
# Interactive SQL
duckdb ladder-sniper/data/research.duckdb

# Run queries from file
duckdb ladder-sniper/data/research.duckdb < my_query.sql

# Export to CSV
duckdb ladder-sniper/data/research.duckdb <<EOF
COPY (SELECT * FROM positions) TO 'export.csv' (HEADER, DELIMITER ',');
EOF
```

### Python + DuckDB
```python
import duckdb

conn = duckdb.connect('ladder-sniper/data/research.duckdb')

# Get all positions with holder analysis
df = conn.execute("""
    SELECT
        pnl_percent,
        holder_count_entry,
        holder_count_exit,
        holder_count_exit - holder_count_entry as holder_growth,
        hold_duration_secs
    FROM positions
    WHERE exit_time_micros IS NOT NULL
""").df()

print(df.describe())
print(df.corr())  # Correlation matrix
```

---

## 🚨 Troubleshooting

### Monitor shows zeros
- Check if bot is running
- Verify database path
- Ensure bot has made at least one trade

### JSON parsing fails
- Update to latest version
- Check for stderr output
- Verify DuckDB file isn't corrupted

### Database locked
- Only one writer at a time
- Monitor tools are read-only (safe)
- If stuck, restart bot

---

## 💡 Pro Tips

1. **Run monitor in tmux/screen** for persistent monitoring
2. **Use --json mode for automation**, human mode for watching
3. **Query database for deep analysis**, use monitor for live stats
4. **Alert on anomalies**, not just thresholds
5. **Export snapshots** for historical comparison
6. **Correlate holder_count with success** - that's the research goal!

---

## 📚 See Also

- [RESEARCH_QUERIES.md](RESEARCH_QUERIES.md) - SQL queries for analysis
- `../target/release/analytics` - Detailed static reports
- `duckdb ladder-sniper/data/research.duckdb` - Direct database access

---

**Built for the $1M research mission with microsecond-precision holder tracking** 🎯
