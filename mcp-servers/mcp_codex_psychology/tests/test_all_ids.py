#!/usr/bin/env python3
"""Test upload with all 4 ID combinations from .env"""

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

# All 4 IDs to test
ids_to_test = {
    "ORG_ID": os.getenv("KONTEXT_ORG_ID"),
    "ORG_PRV_ID": os.getenv("KONTEXT_ORG_PRV_ID"),
    "DEVELOPER_ID": os.getenv("KONTEXT_DEVELOPER_ID"),
    "USER_ID": os.getenv("KONTEXT_USER_ID")
}


async def test_upload_with_id(id_name: str, id_value: str):
    """Test upload with a specific ID as x-as-user."""
    try:
        cert_path = '/etc/ssl/cert.pem'
        async with httpx.AsyncClient(verify=cert_path, timeout=30.0) as client:
            doc_path = "/Users/resatugurulu/Downloads/kontextd.md"

            with open(doc_path, 'rb') as f:
                files = {'file': ('kontextd.md', f, 'text/markdown')}

                print("=" * 70)
                print(f"Testing with {id_name}: {id_value}")
                print("=" * 70)

                upload_response = await client.post(
                    f"{KONTEXT_BASE_URL}/vault/files",
                    headers={
                        'x-api-key': KONTEXT_API_KEY,
                        'x-as-user': id_value
                    },
                    files=files
                )

            print(f"Status: {upload_response.status_code}")
            try:
                print(f"Response: {json.dumps(upload_response.json(), indent=2)}")
            except:
                print(f"Response: {upload_response.text}")

            # Check if successful
            if upload_response.status_code == 200 or upload_response.status_code == 201:
                print(f"\n🎉 SUCCESS with {id_name}!")
                return True
            else:
                print(f"\n❌ Failed with {id_name}")
                return False

    except Exception as e:
        print(f"Error: {e}")
        return False


async def main():
    """Test all ID combinations."""
    print("🔍 Testing all 4 ID combinations for Kontext vault upload...")
    print("=" * 70)
    print()

    for id_name, id_value in ids_to_test.items():
        print(f"\n{id_name}: {id_value}")
        success = await test_upload_with_id(id_name, id_value)
        if success:
            print(f"\n✅ FOUND WORKING ID: {id_name} = {id_value}")
            break
        print()

    print("\n" + "=" * 70)
    print("Test complete")


if __name__ == "__main__":
    asyncio.run(main())
