#!/usr/bin/env python3
"""Query Kontext vault to see what files exist."""

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


async def query_vault():
    """Query the Kontext vault to see what files exist."""
    try:
        # Disable SSL verification temporarily
        async with httpx.AsyncClient(verify=False, timeout=30.0) as client:

            print(f"📤 Querying: {KONTEXT_BASE_URL}/vault/files")
            print(f"🔑 Using API key: {KONTEXT_API_KEY[:20]}...")
            print(f"👤 Developer ID: {KONTEXT_DEVELOPER_ID}")
            print()

            # Try different query methods

            # Method 1: Simple GET
            print("=" * 60)
            print("Method 1: Simple GET /vault/files")
            print("=" * 60)
            response = await client.get(
                f"{KONTEXT_BASE_URL}/vault/files",
                headers={
                    'x-api-key': KONTEXT_API_KEY,
                    'x-as-user': KONTEXT_DEVELOPER_ID
                }
            )
            print(f"Status: {response.status_code}")
            print(f"Response: {response.text}")
            print()

            # Method 2: Query with search parameter
            print("=" * 60)
            print("Method 2: GET /vault/files with search param")
            print("=" * 60)
            response2 = await client.get(
                f"{KONTEXT_BASE_URL}/vault/files",
                headers={
                    'x-api-key': KONTEXT_API_KEY,
                    'x-as-user': KONTEXT_DEVELOPER_ID
                },
                params={'search': 'codex'}
            )
            print(f"Status: {response2.status_code}")
            print(f"Response: {response2.text}")
            print()

            # Method 3: Try /vault/query endpoint (from docs link)
            print("=" * 60)
            print("Method 3: POST /vault/query (semantic search)")
            print("=" * 60)
            response3 = await client.post(
                f"{KONTEXT_BASE_URL}/vault/query",
                headers={
                    'x-api-key': KONTEXT_API_KEY,
                    'x-as-user': KONTEXT_DEVELOPER_ID,
                    'Content-Type': 'application/json'
                },
                json={
                    'query': 'codex-d project architecture',
                    'top_k': 5
                }
            )
            print(f"Status: {response3.status_code}")
            print(f"Response: {response3.text}")
            print()

    except Exception as e:
        print(f"❌ Error: {e}")
        print(f"   Type: {type(e).__name__}")


async def main():
    """Main function."""
    print("🔍 Querying Kontext vault...")
    print("=" * 60)
    print()
    await query_vault()


if __name__ == "__main__":
    asyncio.run(main())
