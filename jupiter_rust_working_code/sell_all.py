#!/usr/bin/env python3
import os
import requests
import time
from dotenv import load_dotenv

load_dotenv()

API_KEY = os.getenv("PUMPPORTAL_API_KEY")
API_URL = "https://pumpportal.fun/api/trade-local"

# Mints from last bot run
MINTS_TO_SELL = [
    "DqXMpdkSxq7uxFCpTVkWNgNuz96xSZLuWEn3yY8spump",  # negative67
    "H7SUNxQ68u2nQ1JXRm5s5Q7BzvxgKFuJgzWnBznCpump",  # Proton
    "2eJZFR47Wib47SEarbBxZSdtXApanCRKrXxPfYfgpump",  # Amazon Robot
    "HXCZtPAzPqHBwzJgpg5ArUWU3JnHQHbcuAotkPespump",  # Boxiumus
    "EKPteuctVqxmDm9MXoh2tVyXbfP6JuKi5KmBSrAVpump",  # K.I.T.
    "5ADHoSssWeSzo6daKGxY8JWu3oL44j2iunvA71sJpump",  # 1st402.fun
    "9K3XSk9U19iHvQShZYJ7KqAARELWttELBGfqUkTMpump",  # 3lixir
    "GBXDgRWfdZFomSqd8Zy8jLuwstzmVE7cJTMf4qHMpump",  # RIP Kanzi
    "7T1Ta1xsgiEqsVo1wry2Tr7sSfCzr1UuMTNKJZubpump",  # Lens402
    "2SDNfhr5L56Q5EPsofgV7Fms5uRxA8u9zBLAGdfApump",  # TITAN
    "Hokm69BwcRj2Tdbf3C9TEstjsYt1vso6FBTdyWenpump",  # EESEE
    "AA4TAqovYb2MftgCCUxpNX16xv76yNr8ymZUUMZepump",  # shifu
    "G2jYcuvycEvMgvLJm64dksjxFJJH9zQXm9iVo5Xopump",  # Cannoli
    "7aazFv1rkEEsFo3j6PYNzU37CFELuj8MF3aPMd8pump",  # The Brick Lady
]

def sell_token(mint):
    """Sell 100% of a token position"""
    payload = {
        "publicKey": API_KEY,
        "action": "sell",
        "mint": mint,
        "amount": "100%",
        "denominatedInSol": "false",
        "slippage": 20,
        "priorityFee": 0.0001,
        "pool": "pump"
    }

    try:
        response = requests.post(API_URL, json=payload, timeout=30)
        response.raise_for_status()
        data = response.json()
        return data.get("signature", "unknown")
    except Exception as e:
        return f"ERROR: {str(e)}"

def main():
    print("🔥 EMERGENCY SELL ALL POSITIONS")
    print("━" * 50)
    print()
    print(f"📊 Positions to sell: {len(MINTS_TO_SELL)}")
    print()

    for i, mint in enumerate(MINTS_TO_SELL, 1):
        print(f"🔄 [{i}/{len(MINTS_TO_SELL)}] Selling {mint[:8]}...")

        sig = sell_token(mint)

        if sig.startswith("ERROR"):
            print(f"   ❌ FAILED: {sig}")
        else:
            print(f"   ✅ SOLD - Signature: {sig}")

        # Small delay to avoid rate limits
        time.sleep(0.5)

    print()
    print("✅ SELL ALL COMPLETE!")

if __name__ == "__main__":
    main()
