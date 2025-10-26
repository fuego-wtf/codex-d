#!/usr/bin/env python3
"""Upload to Kontext using the correct multipart format from curl example."""

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


async def upload_file():
    """Upload file using exact format from curl example."""
    cert_path = '/etc/ssl/cert.pem'
    doc_path = "/Users/resatugurulu/Downloads/kontextd.md"

    # Test with both user IDs
    user_ids = [
        ("DEVELOPER_ID", DEVELOPER_ID),
        ("USER_ID", USER_ID)
    ]

    for id_name, user_id in user_ids:
        try:
            async with httpx.AsyncClient(verify=cert_path, timeout=30.0) as client:
                print("=" * 80)
                print(f"Testing with {id_name}: {user_id}")
                print("=" * 80)

                # Open file and create proper multipart form-data
                # curl: -F "file=@notes.txt;type=text/plain"
                # httpx equivalent:
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

                print(f"Status: {response.status_code}")

                try:
                    result = response.json()
                    print(f"Response: {json.dumps(result, indent=2)}")

                    if response.status_code in [200, 201]:
                        print(f"\n🎉🎉🎉 SUCCESS with {id_name}!")
                        print(f"User ID: {user_id}")
                        return True
                except:
                    print(f"Response: {response.text}")

                print()

        except Exception as e:
            print(f"Error: {e}\n")

    return False


async def main():
    """Main function."""
    print("🚀 Uploading file using correct multipart format...")
    print("   curl: -F \"file=@notes.txt;type=text/plain\"")
    print("   httpx: files={'file': (filename, file_obj, mime_type)}")
    print("=" * 80)
    print()

    success = await upload_file()

    print("\n" + "=" * 80)
    if success:
        print("✅ Upload successful!")
    else:
        print("❌ Upload failed with both user IDs")


if __name__ == "__main__":
    asyncio.run(main())
