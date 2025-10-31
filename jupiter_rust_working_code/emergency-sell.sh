#!/bin/bash
# EMERGENCY SELL ALL SCRIPT

API_KEY="6dvpax2hf1c5gj1n69472k1f61qpadamdh5m8nujf514pwbk8dd6ytb2e14k8j1gb1t6gd2mchj4ajbfe9r7en1q99x30p9pdrrmrhtfb5bjythq6h27cubfa9kncwuq893kgcbe84yku85aqcp3e85mk6c2e8nh5eha564ad232c3nacnjpcvdamwmmvhn8mqnmh3p958kuf8"

# Get all token accounts with balance
curl -s "https://pumpportal.fun/api/balances?api-key=$API_KEY" | jq -r '.[] | select(.uiAmount > 0) | .mint' | while read mint; do
    echo "🚨 SELLING $mint"
    curl -X POST "https://pumpportal.fun/api/trade" \
        -H "Content-Type: application/json" \
        -d "{
            \"api_key\": \"$API_KEY\",
            \"action\": \"sell\",
            \"mint\": \"$mint\",
            \"amount\": \"100%\",
            \"denominatedInSol\": \"false\",
            \"slippage\": 30,
            \"priorityFee\": 0.0001,
            \"pool\": \"pump\"
        }"
    echo ""
    sleep 1
done
