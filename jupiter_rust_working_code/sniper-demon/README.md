# 🔮 Sniper Demon - AI Trading Assistant

**Lightweight binary that watches your positions and triggers AI-assisted decisions when conditions conflict**

## What It Does

The Sniper Demon runs alongside your main trading bot and provides **AI-powered decision-making** when automated rules aren't enough:

### Trigger Scenarios

1. **High Rug Risk** → "Speed is 100% more important"
   - When `rug_risk > 0.7` → AI instantly recommends emergency exit
   - Priority: Safety > Everything

2. **2x Profit + High Momentum** → Conflicting signals
   - Rule says: Exit and recover 110%
   - Momentum says: This could 5x
   - AI decides: Partial exit vs hold

3. **Momentum Stalled** → Fast exit or wait?
   - 60s passed with low momentum
   - But holder health improving
   - AI assesses: Dead or accumulation?

4. **High Momentum in Profit** → Let it run or secure?
   - Strong upward movement
   - Already profitable
   - AI optimizes: Trail vs exit

## Architecture

```
┌─────────────────────┐
│  Sniper Bot         │  Main trading loop
│  (pump-sniper-bot)  │  - Detects launches
└──────────┬──────────┘  - Executes trades
           │             - Tracks positions
           │
    Shares Database
           │
┌──────────▼──────────┐
│  Sniper Demon       │  AI assistant (this)
│  (tiny binary)      │  - Watches positions
└──────────┬──────────┘  - Detects triggers
           │             - Gets AI decisions
           │             - Logs recommendations
           ▼
     DeepSeek API
     (or Claude/OpenAI)
```

## Communication

**Via SQLite Database** (shared with main bot):

### Demon → Bot
```sql
ai_recommendations table:
- mint (token address)
- action (Hold|ExitFull|ExitPartial|Trail|Emergency)
- confidence (0.0-1.0)
- reasoning (AI explanation)
- suggested_stop (optional trailing stop %)
```

### Bot → Demon
```sql
positions table: Current positions
momentum_snapshots table: Real-time signals
```

### History
```sql
ai_decisions table: Every AI decision logged for learning
```

## Setup

### 1. Get DeepSeek API Key
```bash
# Visit: https://platform.deepseek.com/
# Create account and get API key
```

### 2. Configure
```bash
cp .env.example .env
# Edit .env:
AI_PROVIDER=deepseek
AI_API_KEY=your-key-here
DATABASE_PATH=../pump-sniper-bot/sniper_bot.db
```

### 3. Build
```bash
cargo build --release
```

### 4. Run (alongside main bot)
```bash
# Terminal 1: Main bot
cd ../pump-sniper-bot
RUST_LOG=info cargo run

# Terminal 2: AI Demon
cd ../sniper-demon
RUST_LOG=info cargo run
```

## How It Works

### Event Loop (Every 5 seconds)

```rust
1. Check all active positions
2. Get latest momentum data
3. Detect trigger conditions:
   - High rug risk (>0.7)
   - Momentum stalled (60s + <0.3 momentum)
   - Hit 2x profit
   - Conflicting signals
4. If triggered:
   - Build decision context
   - Send to AI (DeepSeek)
   - Parse JSON response
   - Log decision
   - Save recommendation for bot
```

### Trigger Priority

```
1. High Rug Risk      → ALWAYS checked first
2. Momentum Stalled   → Rule #7 enforcement
3. Profit Target 2x   → Rule #9 with AI override
4. High Momentum      → Optimization opportunity
```

### AI Prompt System

Each trigger gets a specialized prompt:

**High Rug Risk** ([prompts/high_rug_risk.txt](prompts/high_rug_risk.txt)):
```
"When rug_risk > 0.7: SPEED IS 100% MORE IMPORTANT
Better to exit early than lose everything."
```

**Conflicting Signals** ([prompts/conflicting_signals.txt](prompts/conflicting_signals.txt)):
```
"At 2x BUT momentum = 0.85 (very high)
Weigh: Momentum (40%) + Risk (40%) + Profit Protection (20%)
Your role: Make the judgment call that automation can't."
```

### AI Response Format

```json
{
  "action": "ExitPartial",
  "exit_percent": 55.0,
  "confidence": 0.85,
  "reasoning": "2x hit with momentum 0.82. Exit 55% to recover 110%, hold 45% for continued run. Low rug risk supports holding.",
  "suggested_stop": 7.5
}
```

## AI Providers

### DeepSeek (Primary) ✅
- Fast and cheap
- Good at structured decisions
- JSON output reliable
- Cost: ~$0.0001 per decision

### Claude (Future)
- More nuanced reasoning
- Better at complex scenarios
- Higher cost
- **Stub ready** at [src/ai/claude.rs](src/ai/claude.rs)

### OpenAI (Future)
- Alternative option
- GPT-4 for complex cases
- **Stub ready** at [src/ai/openai.rs](src/ai/openai.rs)

## Performance

### Resource Usage
- **Binary Size**: ~5MB
- **Memory**: <20MB
- **CPU**: <1%
- **Network**: One API call per trigger (~100 bytes)

### Speed
- Database query: <1ms
- Trigger detection: <1ms
- AI API call: 200-500ms
- Total: <1 second per decision

### Cost
- DeepSeek: ~$0.0001 per decision
- Expected triggers: 2-5 per position
- Monthly cost: <$1 for 100 positions

## Learning System

Every decision is logged:

```sql
SELECT mint, action, confidence, reasoning, timestamp
FROM ai_decisions
ORDER BY timestamp DESC;
```

This data feeds back into:
1. Prompt optimization
2. Confidence calibration
3. Strategy evolution
4. Performance analysis

## Integration with Main Bot

The main bot checks for AI recommendations:

```rust
// In position management loop
if let Some(ai_rec) = db.get_ai_recommendation(mint)? {
    if ai_rec.confidence > 0.75 {
        // Execute AI decision
        match ai_rec.action {
            "ExitPartial(55%)" => execute_partial_exit(mint, 55.0).await?,
            "Emergency" => execute_exit(mint, 100.0).await?,
            _ => {}
        }
    }
}
```

## Example Output

```
🔮 Sniper Demon Starting...
   AI-Assisted Trading Decisions
   Watching for trigger conditions

🔍 Checking AI provider: DeepSeek
✅ DeepSeek is healthy

👀 Monitoring positions every 5s...

🎯 TRIGGER DETECTED: ConflictingSignals for GgoaCo...pump
🧠 DeepSeek analyzing position: GgoaCo...pump
   Trigger: ConflictingSignals
   P&L: 2.10x | Momentum: 0.82 | Rug Risk: 0.45
✅ AI Decision: ExitPartial(45%)
   Confidence: 0.85
   Reasoning: At 2.1x with momentum 0.82 (very strong) but rug risk 0.45 (moderate). Exit 45% to secure 110% recovery per Rule #9, hold 55% with tight trailing stop. Momentum justifies holding majority but risk requires partial derisking.
💾 AI recommendation saved for main bot to execute
```

## Development

### Adding New Triggers

1. Add to `TriggerType` enum in [src/ai/mod.rs](src/ai/mod.rs)
2. Create prompt template in `prompts/`
3. Add detection logic in `detect_trigger()` in [src/main.rs](src/main.rs)

### Adding New AI Provider

1. Create `src/ai/your_provider.rs`
2. Implement `AiProvider` trait
3. Add to factory in [src/ai/mod.rs](src/ai/mod.rs)

### Testing

```bash
# Test specific position
sqlite3 ../pump-sniper-bot/sniper_bot.db "SELECT * FROM positions WHERE status='active'"

# Test AI decision manually
cargo run

# Check decisions
sqlite3 ../pump-sniper-bot/sniper_bot.db "SELECT * FROM ai_decisions ORDER BY timestamp DESC LIMIT 5"
```

## Why This Architecture?

### Separation of Concerns
- **Main Bot**: Fast execution, position tracking
- **Demon**: Slow AI thinking, complex decisions

### Independent Scaling
- Run multiple demons with different AI providers
- A/B test different strategies
- Hot-swap without restarting main bot

### Clear Audit Trail
- Every AI decision logged with reasoning
- Confidence scores tracked
- Performance measurable

### Cost Optimization
- Only call AI when needed
- No API costs for simple rules
- Shared database = no network overhead

## Summary

**Sniper Demon = Your AI trading partner**

- Watches positions for you
- Triggers AI analysis at critical moments
- Provides human-level judgment
- Logs everything for learning
- Tiny, fast, cheap

**When to use:**
- "At 2x but momentum is pumping" → AI decides
- "Rug risk high but could be FUD" → AI assesses
- "Stalled but accumulation phase?" → AI judges

**When NOT to use:**
- Simple rule-based exits → Main bot handles
- Clear rug pulls → Automated emergency exit
- No conflicting signals → Follow rules

**This is compound intelligence: Automation + AI + Your rules** 🎯
