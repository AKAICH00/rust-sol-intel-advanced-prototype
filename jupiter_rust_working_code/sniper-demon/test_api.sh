#!/bin/bash
# Quick test of DeepSeek API connection

set -e

echo "🔍 Testing DeepSeek API Connection..."
echo ""

# Load .env
if [ -f .env ]; then
    export $(cat .env | grep DEEPSEEK_API_KEY | xargs)
else
    echo "❌ No .env file found. Run: cp .env.example .env"
    exit 1
fi

if [ -z "$DEEPSEEK_API_KEY" ]; then
    echo "❌ DEEPSEEK_API_KEY not set in .env"
    exit 1
fi

echo "✅ API Key found: ${DEEPSEEK_API_KEY:0:10}..."
echo ""
echo "📡 Sending test request to DeepSeek..."
echo ""

# Test API call
response=$(curl -s https://api.deepseek.com/v1/chat/completions \
  -H "Authorization: Bearer $DEEPSEEK_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "deepseek-chat",
    "messages": [{"role": "user", "content": "ping"}],
    "temperature": 0.3,
    "max_tokens": 10
  }')

# Check response
if echo "$response" | grep -q "choices"; then
    echo "✅ API Connection Successful!"
    echo ""
    echo "Response:"
    echo "$response" | python3 -m json.tool 2>/dev/null || echo "$response"
    echo ""
    echo "🎉 DeepSeek is ready for AI trading decisions!"
else
    echo "❌ API Error:"
    echo "$response" | python3 -m json.tool 2>/dev/null || echo "$response"
    exit 1
fi
