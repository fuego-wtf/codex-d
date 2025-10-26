#!/usr/bin/env python3
"""List files in Kontext vault."""

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
KONTEXT_ORG_ID = os.getenv("KONTEXT_ORG_ID")
KONTEXT_DEVELOPER_ID = os.getenv("KONTEXT_DEVELOPER_ID")
KONTEXT_USER_ID = "uSAne3UgSr3ucjhF5Kc4tDION6cjOjZf"  # App user ID


async def list_vault_files():
    """List files in the Kontext vault - try different endpoints."""
    cert_path = '/etc/ssl/cert.pem'

    endpoints_to_try = [
        f"/vault/files",  # Simple endpoint
        f"/vault/organizations/{KONTEXT_ORG_ID}/files",  # Organization vault
        f"/vault/users/{KONTEXT_USER_ID}/files",  # User vault with actual user ID
    ]

    for endpoint in endpoints_to_try:
        try:
            async with httpx.AsyncClient(verify=cert_path, timeout=30.0) as client:
                url = f"{KONTEXT_BASE_URL}{endpoint}"

                print("=" * 60)
                print(f"📤 Trying: {endpoint}")
                print("=" * 60)

                response = await client.get(
                    url,
                    headers={
                        'x-api-key': KONTEXT_API_KEY,
                        'x-as-user': KONTEXT_USER_ID
                    }
                )

                print(f"Status: {response.status_code}")
                try:
                    print(f"Response: {json.dumps(response.json(), indent=2)}")
                except:
                    print(f"Response: {response.text}")
                print()

        except Exception as e:
            print(f"Error: {e}")
            print()


async def main():
    """Main function."""
    print("🔍 Listing Kontext vault files...")
    print("=" * 60)
    print()
    await list_vault_files()


if __name__ == "__main__":
    asyncio.run(main())
