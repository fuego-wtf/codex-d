#!/usr/bin/env python3
"""Direct upload to Kontext vault without MCP wrapper."""

import asyncio
import json
import httpx
import ssl
import certifi
from pathlib import Path
from dotenv import load_dotenv
import os

# Set SSL certificate environment variable for httpx
os.environ['SSL_CERT_FILE'] = certifi.where()
os.environ['REQUESTS_CA_BUNDLE'] = certifi.where()

# Load environment variables
env_path = Path(__file__).parent / '.env'
load_dotenv(dotenv_path=env_path)

# Kontext API Configuration
KONTEXT_API_KEY = os.getenv("KONTEXT_API_KEY")
KONTEXT_BASE_URL = "https://staging-api.kontext.dev"
KONTEXT_ORG_ID = os.getenv("KONTEXT_ORG_ID")
KONTEXT_DEVELOPER_ID = os.getenv("KONTEXT_DEVELOPER_ID")
KONTEXT_USER_ID = "uSAne3UgSr3ucjhF5Kc4tDION6cjOjZf"  # App user ID


async def upload_to_kontext(documentation_path: str):
    """Upload codex-d documentation to Kontext vault."""
    try:
        # Use system cert path that curl uses (works on macOS)
        cert_path = '/etc/ssl/cert.pem'
        async with httpx.AsyncClient(verify=cert_path, timeout=30.0) as client:
            # Upload file using correct Kontext API format
            with open(documentation_path, 'rb') as f:
                files = {'file': ('kontextd.md', f, 'text/markdown')}

                print(f"📤 Uploading to: {KONTEXT_BASE_URL}/vault/files")
                print(f"🔑 Using API key: {KONTEXT_API_KEY[:20]}...")
                print(f"👤 User ID: {KONTEXT_USER_ID}")
                print()

                # Upload file using multipart form-data (per Kontext docs)
                upload_response = await client.post(
                    f"{KONTEXT_BASE_URL}/vault/files",
                    headers={
                        'x-api-key': KONTEXT_API_KEY,
                        'x-as-user': KONTEXT_USER_ID
                    },
                    files=files
                )

            print(f"📊 Response status: {upload_response.status_code}")
            print(f"📄 Response body:")
            print(upload_response.text)
            print()

            result = upload_response.json()
            file_id = result.get('id') or result.get('fileId')

        return {
            "status": "success",
            "file_id": file_id,
            "message": f"Documentation uploaded successfully to Kontext. File ID: {file_id}",
            "result": result
        }

    except Exception as e:
        return {
            "status": "error",
            "message": f"Failed to upload to Kontext: {str(e)}",
            "error_type": type(e).__name__
        }


async def main():
    """Main upload function."""
    print("🚀 Uploading codex-d documentation to Kontext vault...")
    print("=" * 60)

    doc_path = "/Users/resatugurulu/Downloads/kontextd.md"

    print(f"📄 Documentation file: {doc_path}")

    # Check if file exists
    if not Path(doc_path).exists():
        print(f"❌ Error: File not found at {doc_path}")
        return

    # Get file size
    file_size = Path(doc_path).stat().st_size
    print(f"📊 File size: {file_size} bytes ({file_size / 1024:.2f} KB)")
    print()

    # Upload
    result = await upload_to_kontext(doc_path)

    print("=" * 60)
    print("📤 Upload Result:")
    print("=" * 60)
    print(json.dumps(result, indent=2))
    print("=" * 60)

    if result.get("status") == "success":
        print()
        print("✅ SUCCESS!")
        print(f"   File ID: {result.get('file_id')}")
        print(f"   Message: {result.get('message')}")
    else:
        print()
        print("❌ FAILED!")
        print(f"   Error: {result.get('message')}")
        print(f"   Type: {result.get('error_type')}")


if __name__ == "__main__":
    asyncio.run(main())
