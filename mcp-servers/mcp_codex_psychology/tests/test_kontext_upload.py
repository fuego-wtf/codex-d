#!/usr/bin/env python3
"""Test script to upload codex-d documentation to Kontext vault."""

import asyncio
import sys
from pathlib import Path

# Add mcp_server to path
sys.path.insert(0, str(Path(__file__).parent))

from mcp_server.server import upload_project_to_kontext


async def main():
    """Upload the kontextd.md documentation to Kontext."""
    print("🚀 Uploading codex-d documentation to Kontext vault...")
    print("=" * 60)

    # Default path from the upload function
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

    # Call the upload function
    result = await upload_project_to_kontext(doc_path)

    print("📤 Upload Result:")
    print("=" * 60)
    print(result)
    print("=" * 60)

    # Parse and display summary
    import json
    result_data = json.loads(result)

    if result_data.get("status") == "success":
        print()
        print("✅ SUCCESS!")
        print(f"   File ID: {result_data.get('file_id')}")
        print(f"   Message: {result_data.get('message')}")
    else:
        print()
        print("❌ FAILED!")
        print(f"   Error: {result_data.get('message')}")


if __name__ == "__main__":
    asyncio.run(main())
