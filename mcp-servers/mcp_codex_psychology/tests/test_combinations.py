#!/usr/bin/env python3
"""Test all combinations of org IDs and user IDs for Kontext upload."""

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

ORG_ID = os.getenv("KONTEXT_ORG_ID")
ORG_PRV_ID = os.getenv("KONTEXT_ORG_PRV_ID")
DEVELOPER_ID = os.getenv("KONTEXT_DEVELOPER_ID")
USER_ID = os.getenv("KONTEXT_USER_ID")


async def test_upload(endpoint: str, as_user_id: str, combo_name: str):
    """Test upload with specific endpoint and user ID combination."""
    try:
        cert_path = '/etc/ssl/cert.pem'
        async with httpx.AsyncClient(verify=cert_path, timeout=30.0) as client:
            doc_path = "/Users/resatugurulu/Downloads/kontextd.md"

            with open(doc_path, 'rb') as f:
                files = {'file': ('kontextd.md', f, 'text/markdown')}

                print("=" * 80)
                print(f"Testing: {combo_name}")
                print(f"Endpoint: {endpoint}")
                print(f"x-as-user: {as_user_id}")
                print("=" * 80)

                upload_response = await client.post(
                    endpoint,
                    headers={
                        'x-api-key': KONTEXT_API_KEY,
                        'x-as-user': as_user_id
                    },
                    files=files
                )

            print(f"Status: {upload_response.status_code}")
            try:
                result = upload_response.json()
                print(f"Response: {json.dumps(result, indent=2)}")

                # Check for success
                if upload_response.status_code in [200, 201]:
                    print(f"\n🎉🎉🎉 SUCCESS! Working combination found!")
                    print(f"Endpoint: {endpoint}")
                    print(f"x-as-user: {as_user_id}")
                    return True
            except:
                print(f"Response: {upload_response.text}")

            print()
            return False

    except Exception as e:
        print(f"Error: {e}\n")
        return False


async def main():
    """Test all combinations of org + user IDs."""
    print("🔍 Testing all ORG + USER ID combinations for Kontext vault upload...")
    print("=" * 80)
    print()

    # Test combinations
    combinations = [
        # (endpoint, x-as-user, description)

        # 1. ORG_ID + DEVELOPER_ID
        (f"{KONTEXT_BASE_URL}/vault/files", DEVELOPER_ID, "Simple endpoint + DEVELOPER"),
        (f"{KONTEXT_BASE_URL}/vault/organizations/{ORG_ID}/files", DEVELOPER_ID, "ORG_ID endpoint + DEVELOPER"),

        # 2. ORG_PRV_ID + DEVELOPER_ID
        (f"{KONTEXT_BASE_URL}/vault/organizations/{ORG_PRV_ID}/files", DEVELOPER_ID, "ORG_PRV_ID endpoint + DEVELOPER"),

        # 3. ORG_ID + USER_ID
        (f"{KONTEXT_BASE_URL}/vault/files", USER_ID, "Simple endpoint + USER"),
        (f"{KONTEXT_BASE_URL}/vault/organizations/{ORG_ID}/files", USER_ID, "ORG_ID endpoint + USER"),
        (f"{KONTEXT_BASE_URL}/vault/users/{USER_ID}/files", ORG_ID, "USER endpoint + ORG_ID as user (reversed)"),

        # 4. ORG_PRV_ID + USER_ID
        (f"{KONTEXT_BASE_URL}/vault/organizations/{ORG_PRV_ID}/files", USER_ID, "ORG_PRV_ID endpoint + USER"),

        # Additional permutations
        (f"{KONTEXT_BASE_URL}/vault/users/{DEVELOPER_ID}/files", DEVELOPER_ID, "DEVELOPER as both path and header"),
        (f"{KONTEXT_BASE_URL}/vault/users/{USER_ID}/files", USER_ID, "USER as both path and header"),
        (f"{KONTEXT_BASE_URL}/vault/users/{DEVELOPER_ID}/files", USER_ID, "DEVELOPER path + USER header"),
        (f"{KONTEXT_BASE_URL}/vault/users/{USER_ID}/files", DEVELOPER_ID, "USER path + DEVELOPER header"),
    ]

    for endpoint, as_user, description in combinations:
        success = await test_upload(endpoint, as_user, description)
        if success:
            print("\n✅ FOUND WORKING COMBINATION!")
            break
        await asyncio.sleep(0.5)  # Small delay between requests

    print("\n" + "=" * 80)
    print("Test complete")


if __name__ == "__main__":
    asyncio.run(main())
