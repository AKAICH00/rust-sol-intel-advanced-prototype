#!/bin/bash
# Keep the sniper bot running 24/7 with auto-restart

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BOT_PATH="$SCRIPT_DIR/../target/release/simple-sniper"
LOG_FILE="$SCRIPT_DIR/bot_$(date +%Y%m%d).log"

echo "🚀 Starting 24/7 Sniper Bot..."
echo "📁 Log: $LOG_FILE"
echo ""

# Kill any existing instances
pkill -9 simple-sniper 2>/dev/null

# Change to bot directory for .env file
cd "$SCRIPT_DIR"

# Run bot with auto-restart
while true; do
    echo "[$(date)] Starting bot..." | tee -a "$LOG_FILE"

    RUST_LOG=info "$BOT_PATH" 2>&1 | tee -a "$LOG_FILE"

    EXIT_CODE=$?
    echo "[$(date)] Bot exited with code: $EXIT_CODE" | tee -a "$LOG_FILE"

    if [ $EXIT_CODE -eq 0 ]; then
        echo "Bot stopped gracefully. Not restarting." | tee -a "$LOG_FILE"
        break
    fi

    echo "Bot crashed! Restarting in 5 seconds..." | tee -a "$LOG_FILE"
    sleep 5
done
