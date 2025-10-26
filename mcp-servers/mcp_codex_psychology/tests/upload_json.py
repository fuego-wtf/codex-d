#!/usr/bin/env python3
"""Upload to Kontext using JSON format (not multipart)."""

import asyncio
import json
import httpx
import base64
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


async def upload_as_json():
    """Try uploading with JSON format instead of multipart."""
    cert_path = '/etc/ssl/cert.pem'
    doc_path = "/Users/resatugurulu/Downloads/kontextd.md"

    # Read file content
    with open(doc_path, 'r', encoding='utf-8') as f:
        content = f.read()

    # Try different JSON payload formats
    payloads = [
        # Format 1: file as direct content
        {
            "file": content
        },
        # Format 2: file with metadata
        {
            "file": content,
            "name": "kontextd.md"
        },
        # Format 3: file with full metadata
        {
            "file": content,
            "name": "kontextd.md",
            "contentType": "text/markdown"
        },
        # Format 4: file as object
        {
            "file": {
                "content": content,
                "name": "kontextd.md"
            }
        },
        # Format 5: Base64 encoded file
        {
            "file": base64.b64encode(content.encode()).decode()
        }
    ]

    user_ids = [DEVELOPER_ID, USER_ID]

    for user_id in user_ids:
        for idx, payload in enumerate(payloads, 1):
            try:
                async with httpx.AsyncClient(verify=cert_path, timeout=30.0) as client:
                    print("=" * 80)
                    print(f"Testing JSON format #{idx} with user: {user_id[:20]}...")
                    print(f"Payload keys: {list(payload.keys()) if payload else 'empty'}")
                    print("=" * 80)

                    response = await client.post(
                        f"{KONTEXT_BASE_URL}/vault/files",
                        headers={
                            'Content-Type': 'application/json',
                            'x-api-key': KONTEXT_API_KEY,
                            'x-as-user': user_id
                        },
                        json=payload
                    )

                    print(f"Status: {response.status_code}")
                    try:
                        result = response.json()
                        print(f"Response: {json.dumps(result, indent=2)}")

                        if response.status_code in [200, 201]:
                            print(f"\n🎉 SUCCESS with format #{idx}!")
                            return True
                    except:
                        print(f"Response: {response.text}")

                    print()

            except Exception as e:
                print(f"Error: {e}\n")

    return False


async def main():
    """Main function."""
    print("🚀 Uploading with JSON format (Content-Type: application/json)...")
    print("=" * 80)
    print()

    success = await upload_as_json()

    print("\n" + "=" * 80)
    if success:
        print("✅ Upload successful!")
    else:
        print("❌ All JSON formats failed")


if __name__ == "__main__":
    asyncio.run(main())
