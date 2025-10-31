# 🚀 Quick Start - AI Trading Demon

## ✅ Prerequisites Complete

- [x] DeepSeek API key configured
- [x] API connection tested
- [x] .env file created

## Test the Demon (No Real Trades)

### 1. Build the Demon
```bash
cd sniper-demon
cargo build --release
```

### 2. Create Test Database
```bash
# Create a test database with mock position
sqlite3 test_sniper.db << 'SQL'
CREATE TABLE positions (
    mint TEXT PRIMARY KEY,
    entry_sol_amount REAL,
    entry_time INTEGER,
    entry_token_amount REAL,
    current_token_amount REAL,
    status TEXT
);

CREATE TABLE momentum_snapshots (
    mint TEXT,
    timestamp INTEGER,
    score REAL,
    rug_risk REAL,
    volume_velocity REAL,
    price_momentum REAL,
    holder_health REAL
);

-- Insert test position (at 2x profit with high momentum)
INSERT INTO positions VALUES (
    'GgoaCoyqvnDE5KGLSpLPiyNVGeF8w8rm8b1Hd9JFpump',
    0.33,
    1234567890,
    1000000.0,
    1000000.0,
    'active'
);

-- Insert conflicting signals (2x + high momentum)
INSERT INTO momentum_snapshots VALUES (
    'GgoaCoyqvnDE5KGLSpLPiyNVGeF8w8rm8b1Hd9JFpump',
    1234567950,
    0.85,  -- High momentum
    0.45,  -- Moderate rug risk
    0.75,
    0.80,
    0.65
);
SQL
```

### 3. Test AI Decision (Mock Data)
```bash
# Point to test database
export DATABASE_PATH=test_sniper.db

# Run demon
RUST_LOG=info cargo run

# You should see:
# 🎯 TRIGGER DETECTED: ConflictingSignals
# 🧠 DeepSeek analyzing...
# ✅ AI Decision: ExitPartial(45%)
```

### 4. Check AI Recommendations
```bash
sqlite3 test_sniper.db << 'SQL'
SELECT * FROM ai_recommendations;
SELECT * FROM ai_decisions ORDER BY timestamp DESC LIMIT 5;
SQL
```

## Production Setup

### 1. Main Bot Setup
```bash
cd ../pump-sniper-bot

# Configure
cp .env.example .env
# Edit .env with:
# - PUMPPORTAL_API_KEY
# - HELIUS_RPC_URL
# - SNIPE_AMOUNT_SOL=0 (full capital)
```

### 2. Run Both Together
```bash
# Terminal 1: Main trading bot
cd pump-sniper-bot
RUST_LOG=info cargo run

# Terminal 2: AI Demon (wait until bot creates positions)
cd sniper-demon
RUST_LOG=info cargo run
```

## How It Works Together

```
Main Bot:
1. Detects launch → Executes snipe → Creates position in DB
2. Monitors position → Updates momentum_snapshots table
3. Checks ai_recommendations table → Executes if confidence > 0.75

AI Demon:
1. Watches positions table every 5s
2. Reads momentum_snapshots
3. Detects triggers → Calls DeepSeek AI
4. Writes to ai_recommendations table
```

## Trigger Scenarios

### Scenario 1: High Rug Risk
```
Position: 0.33 SOL entry
Momentum: rug_risk = 0.85
Trigger: HighRugRisk
AI: "Exit immediately, rug risk critical"
→ Writes: ExitFull (100%)
```

### Scenario 2: 2x + High Momentum (Your Use Case)
```
Position: 0.33 SOL entry → 0.66 SOL current (2x)
Momentum: score = 0.82 (high), rug_risk = 0.45 (moderate)
Trigger: ConflictingSignals
AI: "Exit 45% to recover 110%, hold 55% with trailing stop"
→ Writes: ExitPartial (45%)
```

### Scenario 3: Momentum Stalled
```
Position: 70 seconds elapsed
Momentum: score = 0.18 (very low)
Trigger: MomentumStalled
AI: "No interest, fast exit per Rule #7"
→ Writes: ExitFull (100%)
```

## Monitoring

### Watch AI Decisions Live
```bash
# Terminal 3: Watch decisions
watch -n 2 'sqlite3 sniper_bot.db "SELECT mint, action, confidence, reasoning FROM ai_recommendations"'
```

### Performance Stats
```bash
sqlite3 sniper_bot.db << 'SQL'
-- AI decision accuracy
SELECT 
    action,
    AVG(confidence) as avg_confidence,
    COUNT(*) as count
FROM ai_decisions
GROUP BY action;

-- Recent decisions
SELECT 
    datetime(timestamp, 'unixepoch') as time,
    mint,
    action,
    confidence,
    reasoning
FROM ai_decisions
ORDER BY timestamp DESC
LIMIT 10;
SQL
```

## Cost Tracking

Each AI decision costs ~$0.0001 with DeepSeek.

```bash
# Count AI calls today
sqlite3 sniper_bot.db << 'SQL'
SELECT COUNT(*) * 0.0001 as estimated_cost_usd
FROM ai_decisions
WHERE timestamp > strftime('%s', 'now', '-1 day');
SQL
```

Expected costs:
- 10 positions/day × 3 AI calls each = 30 calls
- 30 × $0.0001 = $0.003/day
- Monthly: ~$0.09

## Troubleshooting

### No Triggers Detected
```bash
# Check positions exist
sqlite3 sniper_bot.db "SELECT * FROM positions WHERE status='active'"

# Check momentum data exists  
sqlite3 sniper_bot.db "SELECT * FROM momentum_snapshots ORDER BY timestamp DESC LIMIT 5"
```

### AI API Errors
```bash
# Test API directly
cd sniper-demon
./test_api.sh
```

### Demon Not Running
```bash
# Check logs
RUST_LOG=debug cargo run
```

## Next Steps

1. **Test with mock data** (above)
2. **Run main bot** to create real positions
3. **Watch demon** make decisions
4. **Review outcomes** in database
5. **Iterate prompts** based on results

## Advanced: Custom Triggers

Edit `src/main.rs` in sniper-demon:

```rust
fn detect_trigger(position: &ActivePosition, momentum: &MomentumData) -> Option<TriggerType> {
    // Add your custom trigger logic
    if momentum.volume_velocity > 0.9 && position.profit_multiple > 3.0 {
        return Some(TriggerType::CustomTrigger);
    }
    // ... existing triggers
}
```

Add prompt in `prompts/custom_trigger.txt`:
```
# CUSTOM TRIGGER: High Volume at 3x

Your custom prompt here...
```

## Summary

✅ API working
✅ Demon built
✅ Ready to watch positions

**Your AI trading assistant is ready!** 🎯
