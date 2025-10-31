#!/bin/bash
# Quick status check for the sniper bot

echo "🔍 SNIPER BOT STATUS CHECK"
echo "=========================="
echo ""

# Check if bot is running
if pgrep -f "simple-sniper" > /dev/null; then
    PID=$(pgrep -f "simple-sniper")
    echo "✅ Bot is RUNNING"
    echo "   PID: $PID"
    echo "   Runtime: $(ps -p $PID -o etime= | xargs)"
    echo ""

    # Show recent activity (last 10 lines)
    echo "📊 RECENT ACTIVITY:"
    echo "-------------------"
    ps -p $PID -o command= | head -1
    echo ""

    # Check for today's log
    LOG_FILE="bot_$(date +%Y%m%d).log"
    if [ -f "$LOG_FILE" ]; then
        echo "📝 Last 5 events:"
        grep -E "(NEW LAUNCH|BUY EXECUTED|SOLD|Position)" "$LOG_FILE" | tail -5
    fi
else
    echo "❌ Bot is NOT running!"
    echo ""
    echo "To start:"
    echo "  cd simple-sniper"
    echo "  ./keep_alive.sh &"
fi

echo ""
echo "🔗 View transactions:"
echo "   https://solscan.io/account/FLkMFK19n7e8j3XA9URCrR2MSg9DkVEky2EsYaqmkbZj"
