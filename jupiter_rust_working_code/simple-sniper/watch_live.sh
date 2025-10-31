#!/bin/bash
# Watch live bot activity - real-time updates

LOG_FILE="bot_$(date +%Y%m%d).log"

if [ ! -f "$LOG_FILE" ]; then
    echo "❌ No log file found for today"
    echo "Bot might not be running yet."
    exit 1
fi

echo "📺 LIVE BOT FEED - Press Ctrl+C to exit"
echo "=========================================="
echo ""

# Follow the log file, filter for important events
tail -f "$LOG_FILE" | grep --line-buffered -E "(🔔|✅|💼|📊|⏰|SOLD)"
