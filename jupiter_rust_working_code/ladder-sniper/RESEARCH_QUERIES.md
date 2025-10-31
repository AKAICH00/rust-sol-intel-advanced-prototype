# Research Database SQL Queries

## 3 Ways to Access Your Data

### Method 1: Analytics Binary (Fastest)
```bash
cd ladder-sniper
cargo run --release --bin analytics
# Or after build:
../target/release/analytics ./data/research.duckdb
```

### Method 2: DuckDB CLI (Most Flexible)
```bash
# Install DuckDB CLI
brew install duckdb

# Open database
duckdb ./data/research.duckdb

# Interactive SQL queries
D SELECT * FROM trades LIMIT 10;
```

### Method 3: Python with DuckDB
```python
import duckdb

conn = duckdb.connect('./data/research.duckdb')
df = conn.execute("SELECT * FROM positions").df()
print(df.describe())
```

---

## Database Schema

### Tables
1. **trades** - Every buy/sell transaction with microsecond timestamps
2. **positions** - Complete position lifecycle with holder counts
3. **position_metrics** - Time-series snapshots with microsecond precision

### Schema Details
```sql
-- Trades table
CREATE TABLE trades (
    trade_id VARCHAR PRIMARY KEY,
    timestamp_micros BIGINT NOT NULL,
    mint VARCHAR NOT NULL,
    trade_type VARCHAR NOT NULL,
    price DOUBLE NOT NULL,
    sol_amount DOUBLE NOT NULL,
    tokens DOUBLE NOT NULL,
    fee_sol DOUBLE NOT NULL,
    priority_fee_sol DOUBLE NOT NULL,
    balance_after DOUBLE NOT NULL,
    signature VARCHAR NOT NULL
);

-- Positions table
CREATE TABLE positions (
    position_id VARCHAR PRIMARY KEY,
    mint VARCHAR NOT NULL,
    entry_time_micros BIGINT NOT NULL,
    exit_time_micros BIGINT,
    entry_price DOUBLE NOT NULL,
    exit_price DOUBLE,
    sol_invested DOUBLE NOT NULL,
    sol_returned DOUBLE,
    tokens DOUBLE NOT NULL,
    pnl_sol DOUBLE,
    pnl_percent DOUBLE,
    hold_duration_secs BIGINT,
    holder_count_entry BIGINT NOT NULL,
    holder_count_exit BIGINT,
    exit_reason VARCHAR,
    profits_taken BOOLEAN DEFAULT FALSE
);

-- Position Metrics table
CREATE TABLE position_metrics (
    metric_id VARCHAR PRIMARY KEY,
    position_id VARCHAR NOT NULL,
    mint VARCHAR NOT NULL,
    timestamp_micros BIGINT NOT NULL,
    elapsed_secs BIGINT NOT NULL,
    current_price DOUBLE NOT NULL,
    pnl_multiplier DOUBLE NOT NULL,
    pnl_percent DOUBLE NOT NULL,
    vwap DOUBLE NOT NULL,
    vwap_distance_percent DOUBLE NOT NULL,
    momentum_score DOUBLE NOT NULL,
    buy_ratio DOUBLE NOT NULL,
    holder_count BIGINT NOT NULL
);
```

---

## Key Research Queries

### Overall Performance
```sql
-- Win rate and P&L summary
SELECT
    COUNT(*) as total_positions,
    COUNT(*) FILTER (WHERE pnl_sol > 0) as wins,
    COUNT(*) FILTER (WHERE pnl_sol <= 0) as losses,
    ROUND((COUNT(*) FILTER (WHERE pnl_sol > 0)::FLOAT / COUNT(*) * 100), 2) as win_rate_pct,
    ROUND(AVG(pnl_percent), 2) as avg_pnl_pct,
    ROUND(SUM(pnl_sol), 4) as total_pnl_sol,
    ROUND(AVG(hold_duration_secs), 1) as avg_hold_secs,
    ROUND(SUM(CASE WHEN pnl_sol > 0 THEN pnl_sol ELSE 0 END), 4) as total_wins_sol,
    ROUND(ABS(SUM(CASE WHEN pnl_sol <= 0 THEN pnl_sol ELSE 0 END)), 4) as total_losses_sol
FROM positions
WHERE exit_time_micros IS NOT NULL;
```

### Holder Count Correlation
```sql
-- Analyze holder count impact on success
SELECT
    CASE
        WHEN holder_count_entry < 10 THEN '0-10'
        WHEN holder_count_entry < 50 THEN '10-50'
        WHEN holder_count_entry < 100 THEN '50-100'
        WHEN holder_count_entry < 500 THEN '100-500'
        ELSE '500+'
    END as holder_range,
    COUNT(*) as positions,
    COUNT(*) FILTER (WHERE pnl_sol > 0) as wins,
    ROUND((COUNT(*) FILTER (WHERE pnl_sol > 0)::FLOAT / COUNT(*) * 100), 2) as win_rate_pct,
    ROUND(AVG(pnl_percent), 2) as avg_pnl_pct,
    ROUND(AVG(holder_count_entry), 0) as avg_entry_holders,
    ROUND(AVG(holder_count_exit), 0) as avg_exit_holders
FROM positions
WHERE exit_time_micros IS NOT NULL
GROUP BY 1
ORDER BY avg_pnl_pct DESC;
```

### Holder Growth Analysis
```sql
-- Positions where holder count increased vs decreased
SELECT
    CASE
        WHEN holder_count_exit > holder_count_entry THEN 'Holders Increased'
        WHEN holder_count_exit < holder_count_entry THEN 'Holders Decreased'
        ELSE 'No Change'
    END as holder_change,
    COUNT(*) as positions,
    COUNT(*) FILTER (WHERE pnl_sol > 0) as wins,
    ROUND((COUNT(*) FILTER (WHERE pnl_sol > 0)::FLOAT / COUNT(*) * 100), 2) as win_rate_pct,
    ROUND(AVG(pnl_percent), 2) as avg_pnl_pct,
    ROUND(AVG(holder_count_exit - holder_count_entry), 0) as avg_holder_change
FROM positions
WHERE exit_time_micros IS NOT NULL
    AND holder_count_exit IS NOT NULL
GROUP BY 1
ORDER BY win_rate_pct DESC;
```

### Hold Time Analysis
```sql
-- Win rate by hold duration
SELECT
    CASE
        WHEN hold_duration_secs <= 10 THEN '0-10s'
        WHEN hold_duration_secs <= 20 THEN '10-20s'
        WHEN hold_duration_secs <= 30 THEN '20-30s'
        WHEN hold_duration_secs <= 45 THEN '30-45s'
        WHEN hold_duration_secs <= 60 THEN '45-60s'
        ELSE '60s+'
    END as hold_time_range,
    COUNT(*) as positions,
    COUNT(*) FILTER (WHERE pnl_sol > 0) as wins,
    ROUND((COUNT(*) FILTER (WHERE pnl_sol > 0)::FLOAT / COUNT(*) * 100), 2) as win_rate_pct,
    ROUND(AVG(pnl_percent), 2) as avg_pnl_pct,
    ROUND(AVG(hold_duration_secs), 1) as avg_hold_secs
FROM positions
WHERE exit_time_micros IS NOT NULL
GROUP BY 1
ORDER BY avg_hold_secs;
```

### VWAP Effectiveness
```sql
-- Analyze positions that exited based on VWAP deviation
-- (exit_reason would need to be recorded in your implementation)
SELECT
    exit_reason,
    COUNT(*) as positions,
    COUNT(*) FILTER (WHERE pnl_sol > 0) as wins,
    ROUND((COUNT(*) FILTER (WHERE pnl_sol > 0)::FLOAT / COUNT(*) * 100), 2) as win_rate_pct,
    ROUND(AVG(pnl_percent), 2) as avg_pnl_pct
FROM positions
WHERE exit_time_micros IS NOT NULL
    AND exit_reason IS NOT NULL
GROUP BY 1
ORDER BY win_rate_pct DESC;
```

### Best Entry Points
```sql
-- Find optimal holder count ranges for entry
SELECT
    ROUND(holder_count_entry / 10) * 10 as holder_bucket,
    COUNT(*) as positions,
    COUNT(*) FILTER (WHERE pnl_sol > 0) as wins,
    ROUND((COUNT(*) FILTER (WHERE pnl_sol > 0)::FLOAT / COUNT(*) * 100), 2) as win_rate_pct,
    ROUND(AVG(pnl_percent), 2) as avg_pnl_pct,
    ROUND(MAX(pnl_percent), 2) as max_pnl_pct
FROM positions
WHERE exit_time_micros IS NOT NULL
GROUP BY 1
HAVING COUNT(*) >= 5  -- Minimum sample size
ORDER BY win_rate_pct DESC;
```

### Time-Series Analysis
```sql
-- Analyze position metrics over time
SELECT
    position_id,
    mint,
    elapsed_secs,
    pnl_percent,
    holder_count,
    vwap_distance_percent,
    momentum_score,
    buy_ratio
FROM position_metrics
WHERE position_id = 'POSITION_ID_HERE'
ORDER BY elapsed_secs;
```

### Trade Frequency Analysis
```sql
-- Trades per hour
SELECT
    DATE_TRUNC('hour', to_timestamp(timestamp_micros / 1000000.0)) as hour,
    COUNT(*) as total_trades,
    COUNT(*) FILTER (WHERE trade_type = 'BUY') as buys,
    COUNT(*) FILTER (WHERE trade_type = 'SELL') as sells
FROM trades
GROUP BY 1
ORDER BY 1 DESC;
```

### Fee Analysis
```sql
-- Total fees paid
SELECT
    ROUND(SUM(fee_sol), 4) as total_trade_fees,
    ROUND(SUM(priority_fee_sol), 4) as total_priority_fees,
    ROUND(SUM(fee_sol + priority_fee_sol), 4) as total_fees,
    COUNT(*) as total_trades,
    ROUND(AVG(fee_sol + priority_fee_sol), 6) as avg_fee_per_trade
FROM trades;
```

### Momentum vs Success
```sql
-- Correlate momentum score with success (from position_metrics)
WITH latest_metrics AS (
    SELECT
        position_id,
        MAX(elapsed_secs) as max_elapsed
    FROM position_metrics
    GROUP BY position_id
),
final_momentum AS (
    SELECT
        pm.position_id,
        pm.momentum_score,
        pm.buy_ratio,
        pm.vwap_distance_percent
    FROM position_metrics pm
    JOIN latest_metrics lm
        ON pm.position_id = lm.position_id
        AND pm.elapsed_secs = lm.max_elapsed
)
SELECT
    CASE
        WHEN fm.momentum_score < 0.2 THEN 'Low (< 0.2)'
        WHEN fm.momentum_score < 0.5 THEN 'Medium (0.2-0.5)'
        ELSE 'High (> 0.5)'
    END as momentum_range,
    COUNT(*) as positions,
    COUNT(*) FILTER (WHERE p.pnl_sol > 0) as wins,
    ROUND((COUNT(*) FILTER (WHERE p.pnl_sol > 0)::FLOAT / COUNT(*) * 100), 2) as win_rate_pct,
    ROUND(AVG(p.pnl_percent), 2) as avg_pnl_pct
FROM final_momentum fm
JOIN positions p ON fm.position_id = p.position_id
WHERE p.exit_time_micros IS NOT NULL
GROUP BY 1
ORDER BY win_rate_pct DESC;
```

### Export to CSV
```sql
-- Export all positions to CSV for external analysis
COPY (
    SELECT * FROM positions
    WHERE exit_time_micros IS NOT NULL
) TO 'positions_export.csv' (HEADER, DELIMITER ',');
```

---

## Real-Time Monitoring

### Current Open Positions
```sql
SELECT
    mint,
    entry_price,
    sol_invested,
    tokens,
    holder_count_entry,
    ROUND((EXTRACT(EPOCH FROM NOW()) - entry_time_micros / 1000000.0), 0) as seconds_open
FROM positions
WHERE exit_time_micros IS NULL
ORDER BY entry_time_micros DESC;
```

### Recent Trades (Last Hour)
```sql
SELECT
    trade_type,
    mint,
    price,
    sol_amount,
    fee_sol + priority_fee_sol as total_fee,
    to_timestamp(timestamp_micros / 1000000.0) as trade_time
FROM trades
WHERE timestamp_micros > EXTRACT(EPOCH FROM NOW() - INTERVAL '1 hour') * 1000000
ORDER BY timestamp_micros DESC;
```

---

## DeepSeek Analysis Queries

### Data for AI Analysis
```sql
-- Comprehensive dataset for ML analysis
SELECT
    p.pnl_percent,
    p.pnl_sol,
    p.hold_duration_secs,
    p.holder_count_entry,
    p.holder_count_exit,
    p.holder_count_exit - p.holder_count_entry as holder_change,
    p.entry_price,
    p.exit_price,
    p.exit_price / p.entry_price as price_multiplier,
    p.sol_invested,
    CASE WHEN p.pnl_sol > 0 THEN 1 ELSE 0 END as is_winner
FROM positions p
WHERE p.exit_time_micros IS NOT NULL;
```

### Correlation Matrix Data
```sql
-- Get all variables for correlation analysis
SELECT
    holder_count_entry,
    holder_count_exit,
    holder_count_exit - holder_count_entry as holder_growth,
    hold_duration_secs,
    entry_price,
    exit_price,
    pnl_percent,
    pnl_sol,
    CASE WHEN pnl_sol > 0 THEN 1 ELSE 0 END as success
FROM positions
WHERE exit_time_micros IS NOT NULL;
```
