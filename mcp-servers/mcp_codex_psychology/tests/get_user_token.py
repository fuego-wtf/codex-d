#!/usr/bin/env python3
"""Get user token to verify which user ID is valid."""

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


async def get_user_token():
    """Get user token to verify valid user ID."""
    cert_path = '/etc/ssl/cert.pem'

    # Test both user IDs
    user_ids_to_test = [
        ("DEVELOPER_ID", DEVELOPER_ID),
        ("USER_ID", USER_ID)
    ]

    for id_name, user_id in user_ids_to_test:
        try:
            async with httpx.AsyncClient(verify=cert_path, timeout=30.0) as client:
                print("=" * 80)
                print(f"Testing: POST /auth/user_token with {id_name}")
                print(f"User ID: {user_id}")
                print("=" * 80)

                # Try different payload formats
                payloads_to_try = [
                    {"user_id": user_id},  # Without expires_in
                    {"user_id": user_id, "expires_in": {}},  # Empty object (from curl example)
                    {"user_id": user_id, "expires_in": "3600"},  # String
                ]

                for payload in payloads_to_try:
                    response = await client.post(
                        f"{KONTEXT_BASE_URL}/auth/user_token",
                        headers={
                            'Content-Type': 'application/json',
                            'x-api-key': KONTEXT_API_KEY
                        },
                        json=payload
                    )

                    if response.status_code != 400:
                        break  # Found working format

                print(f"Status: {response.status_code}")

                try:
                    result = response.json()
                    print(f"Response: {json.dumps(result, indent=2)}")

                    if response.status_code in [200, 201]:
                        print(f"\n✅ SUCCESS! {id_name} is valid!")
                        print(f"Valid user ID: {user_id}")

                        # Now try uploading with this valid user ID
                        print(f"\nNow trying file upload with validated user ID...")
                        upload_success = await try_upload(user_id)

                        if upload_success:
                            print(f"\n🎉🎉🎉 UPLOAD SUCCESSFUL with {id_name}!")
                            return user_id
                        else:
                            print(f"Upload failed with {id_name}, trying next ID...")
                except:
                    print(f"Response: {response.text}")

                print()

        except Exception as e:
            print(f"Error: {e}\n")

    return None


async def try_upload(user_id):
    """Try uploading file with validated user ID."""
    cert_path = '/etc/ssl/cert.pem'
    doc_path = "/Users/resatugurulu/Downloads/kontextd.md"

    try:
        async with httpx.AsyncClient(verify=cert_path, timeout=30.0) as client:
            print("-" * 80)
            print(f"Uploading file with user ID: {user_id}")
            print("-" * 80)

            with open(doc_path, 'rb') as f:
                files = {
                    'file': ('kontextd.md', f, 'text/markdown')
                }

                response = await client.post(
                    f"{KONTEXT_BASE_URL}/vault/files",
                    headers={
                        'x-api-key': KONTEXT_API_KEY,
                        'x-as-user': user_id
                    },
                    files=files
                )

            print(f"Upload Status: {response.status_code}")

            try:
                result = response.json()
                print(f"Upload Response: {json.dumps(result, indent=2)}")

                # 202 = Accepted (file queued for processing)
                if response.status_code in [200, 201, 202]:
                    if result.get('file_id'):
                        print(f"\n✅ FILE UPLOADED SUCCESSFULLY!")
                        print(f"   File ID: {result.get('file_id')}")
                        print(f"   Status: {result.get('status')}")
                        print(f"   Job ID: {result.get('job_id')}")
                        return True
            except:
                print(f"Upload Response: {response.text}")

            return False

    except Exception as e:
        print(f"Upload Error: {e}")
        return False


async def main():
    """Main function."""
    print("🔍 Getting user token to verify valid user ID...")
    print("=" * 80)
    print()

    print(f"Current IDs in .env:")
    print(f"  DEVELOPER_ID: {DEVELOPER_ID}")
    print(f"  USER_ID: {USER_ID}")
    print()

    valid_user_id = await get_user_token()

    print("\n" + "=" * 80)
    if valid_user_id:
        print(f"✅ Found valid user ID: {valid_user_id}")
    else:
        print("❌ Could not find valid user ID")


if __name__ == "__main__":
    asyncio.run(main())
