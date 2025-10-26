#!/usr/bin/env python3
"""Get profile information from Kontext to find the correct user ID."""

import asyncio
import json
import httpx
from pathlib import Path
from dotenv import load_dotenv
import os

# Load environment variables
env_path = Path(__file__).parent / '.env'
load_dotenv(dotenv_path=env_path)

# Kontext API Configuration
KONTEXT_API_KEY = os.getenv("KONTEXT_API_KEY")
KONTEXT_BASE_URL = "https://staging-api.kontext.dev"
DEVELOPER_ID = os.getenv("KONTEXT_DEVELOPER_ID")
USER_ID = os.getenv("KONTEXT_USER_ID")


async def get_profile():
    """Get profile information to find correct user ID."""
    cert_path = '/etc/ssl/cert.pem'

    # Per docs: POST with Authorization: Bearer and JSON body
    user_ids_to_test = [
        ("DEVELOPER_ID", DEVELOPER_ID),
        ("USER_ID", USER_ID)
    ]

    for id_name, user_id in user_ids_to_test:
        try:
            async with httpx.AsyncClient(verify=cert_path, timeout=30.0) as client:
                print("=" * 80)
                print(f"Testing: POST /profile with {id_name}")
                print(f"User ID: {user_id}")
                print("=" * 80)

                # Per docs: POST with Authorization Bearer and X-As-User
                response = await client.post(
                    f"{KONTEXT_BASE_URL}/profile",
                    headers={
                        'Authorization': f'Bearer {KONTEXT_API_KEY}',
                        'X-As-User': user_id,
                        'Content-Type': 'application/json'
                    },
                    json={
                        "userId": user_id,
                        "task": "general"
                    }
                )

                print(f"Status: {response.status_code}")

                try:
                    result = response.json()
                    print(f"Response: {json.dumps(result, indent=2)}")

                    if response.status_code == 200:
                        print(f"\n✅ SUCCESS! Found profile with {id_name}!")
                        return result
                except:
                    print(f"Response: {response.text}")

                print()

        except Exception as e:
            print(f"Error: {e}\n")

    return None


async def main():
    """Main function."""
    print("🔍 Getting Kontext profile to find correct user ID...")
    print("=" * 80)
    print()

    print(f"Current IDs in .env:")
    print(f"  DEVELOPER_ID: {DEVELOPER_ID}")
    print(f"  USER_ID: {USER_ID}")
    print()

    profile = await get_profile()

    print("\n" + "=" * 80)
    if profile:
        print("✅ Found profile!")
        print("\nUse the user ID from the profile response for uploads.")
    else:
        print("❌ Could not retrieve profile")


if __name__ == "__main__":
    asyncio.run(main())
