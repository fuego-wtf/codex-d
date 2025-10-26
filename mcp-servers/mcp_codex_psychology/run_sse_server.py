#!/usr/bin/env python3
"""Entry point for the codex-psychology MCP server with SSE transport."""

import sys
from pathlib import Path

# Add the project root to the Python path
project_root = Path(__file__).parent
sys.path.insert(0, str(project_root))

from mcp_server.server import mcp

if __name__ == "__main__":
    # Run with SSE transport on port 52848 (different from self-reflect's 52847)
    mcp.run(transport="sse", host="127.0.0.1", port=52848)
