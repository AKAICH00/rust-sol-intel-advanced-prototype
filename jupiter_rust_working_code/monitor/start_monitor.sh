#!/bin/bash
# Quick start script for pump.fun sniper monitor dashboard

set -e

echo "🚀 Starting Pump.fun Sniper Monitor..."
echo ""

# Check if database exists
DB_PATH="../pump-sniper-bot/sniper_bot.db"
if [ ! -f "$DB_PATH" ]; then
    echo "⚠️  Database not found at $DB_PATH"
    echo "   Please run the main bot first to create the database:"
    echo "   cd pump-sniper-bot && cargo run"
    echo ""
    exit 1
fi

echo "✅ Database found: $DB_PATH"
echo ""

# Change to monitor directory
cd "$(dirname "$0")"

# Start server
echo "📊 Starting monitor server on http://localhost:8080"
echo "   Dashboard: http://localhost:8080/dashboard.html"
echo ""
echo "   Press Ctrl+C to stop"
echo ""

python3 server.py
