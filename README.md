# Codex-D

**360° Developer Psychology Analysis: Where Mental Health Meets Code Quality**

> Your code already tells your story—we just help you listen.

## Demo

🎥 **Live Demo**: [https://screen.studio/share/olCN1F0V](https://screen.studio/share/olCN1F0V)
🌐 **Landing Page**: [https://codexd.lovable.app/](https://codexd.lovable.app/)

## What is Codex-D?

Codex-D analyzes git repositories to reveal hidden patterns in developer behavior and code quality. It examines commit patterns, message language, and temporal coding habits to detect burnout signals, stress indicators, and how mental state correlates with security vulnerabilities. Using 22 orchestrated MCP tools, it runs behavioral analysis, Aikido security scans, and pulls architecture context from DeepWiki—now fully observable through ACI.dev for transparent agent execution. All insights are stored in private Kontext vaults. Built with Rust + GPUI for native desktop performance, it leverages Agent Client Protocol's adapter (ACI.dev) to route through Codex CLI with personal OpenAI accounts (zero API costs). The result: a 360° analysis connecting developer wellbeing with code quality, making the relationship between mental health and software security visible, measurable, and actionable. Your code already tells your story—we just help you listen.

## Key Features

### 🧠 Behavioral Pattern Detection
- **Commit Patterns**: Detect anxiety, avoidance, and commitment issues through commit frequency and size
- **Message Language**: Identify minimizing ("just", "quick"), defensive ("fix", "oops"), and perfectionist language
- **Temporal Analysis**: Track late-night commits, weekend work patterns, and burst coding sessions
- **Self-Deception Detection**: Compare commit messages vs actual changes to reveal downplaying and vagueness

### 🛡️ Security Integration
- **Aikido Security Scans**: Docker-based SAST, secret detection, dependency analysis, and IaC scanning
- **Vulnerability Correlation**: Connect developer stress patterns with security findings
- **Fix Tracking**: Monitor remediation attempts and recurring security issues

### 📚 Architecture Intelligence
- **DeepWiki Integration**: Pull codebase architecture and context via gate22 gateway
- **Kontext Vault Storage**: Persist all analysis results in private encrypted vaults
- **Session History**: Track behavioral patterns and security trends across multiple scans

### 🔍 Full Observability
- **ACI.dev Transparency**: Every agent action visible through Agent Client Protocol adapter
- **Tool Call Cards**: Clean, collapsible UI showing MCP tool execution in real-time
- **Timeline View**: Chronological display of thoughts, tool calls, and findings

## Architecture

### 22 MCP Tools Orchestrated

**Behavioral Analysis (codex-psychology server):**
- `analyze_commit_patterns()` - Detect commitment and anxiety patterns
- `analyze_message_language()` - Identify linguistic patterns revealing mental state
- `compare_message_vs_diff()` - Find self-deception in commit messages
- `get_temporal_patterns()` - Analyze coding time patterns for stress indicators
- `flag_behavioral_pattern()` - Track recurring behavioral issues

**Security Scanning (codex-psychology server):**
- `run_aikido_security_scan()` - Docker-based comprehensive security analysis
- `save_security_issue()` - Persist vulnerability findings
- `generate_fix_prompt()` - Create actionable remediation instructions

**Session Management (codex-psychology server):**
- `start_session()` - Create roasting scan session
- `close_session()` - Clean up incomplete sessions
- `submit_discovery_answers()` - Save user self-assessment
- `save_scan_results()` - Complete and persist session
- `get_scan_history()` - View past analysis sessions

**Context Storage (codex-psychology server via Kontext):**
- `upload_project_to_kontext()` - Upload documentation to vault
- `query_codex_context()` - Search stored project context
- `save_analysis_to_kontext()` - Persist analysis results
- `get_codex_system_prompt()` - Get comprehensive system context

**Architecture Intelligence (gate22 gateway → DeepWiki):**
- `DEEPWIKI_BOH8VT8Z__READ_WIKI_STRUCTURE()` - Get documentation structure
- `DEEPWIKI_BOH8VT8Z__READ_WIKI_CONTENTS()` - Read specific docs
- `DEEPWIKI_BOH8VT8Z__ASK_QUESTION()` - Query codebase architecture

**Repository Context (codex-psychology server):**
- `set_repository()` - Initialize repo for analysis
- `get_repo_context()` - Get longitudinal history and patterns
- `get_project_summary()` - Overview of repo structure and activity

### Tech Stack

- **UI**: Rust + GPUI (GPU-accelerated native macOS desktop)
- **Agent Protocol**: Agent Client Protocol via ACI.dev adapter
- **MCP Servers**: FastMCP (Python) for psychology tools
- **Security**: Aikido local scanner via Docker
- **Storage**: SQLite (local) + Kontext vaults (cloud encrypted)
- **LLM**: OpenAI via personal account (zero API costs)
- **Observability**: Full agent transparency via ACI.dev

## Installation

### Prerequisites

- macOS (GPUI native app)
- [Codex CLI](https://agentclientprotocol.com/) installed
- Docker Desktop (for Aikido security scans)
- Rust toolchain

### Setup

```bash
# Clone the repository
git clone https://github.com/fuego-wtf/codex-d.git
cd codex-d

# Initialize submodules
git submodule update --init --recursive

# Build codex-acp
cd codex-acp
cargo build --release
cd ..

# Install Python dependencies for MCP server
cd mcp-servers/mcp_codex_psychology
pip install -r requirements.txt
cd ../..

# Build codex-d
cargo build --release

# Run
cargo run --release
```

### Environment Variables

Create `.env` in `mcp-servers/mcp_codex_psychology/`:

```bash
# Aikido Security (required for security scans)
AIKIDO_API_KEY=your_aikido_api_key

# Kontext (required for vault storage)
KONTEXT_API_KEY=your_kontext_api_key
KONTEXT_ORG_ID=your_org_id
KONTEXT_DEVELOPER_ID=your_developer_id
```

## Usage

### 11-Step Mandatory Analysis Workflow

1. **Session Setup**: Create scan session and set repository
2. **Behavioral Analysis**: Run 4 git pattern detections (patterns, language, diff, temporal)
3. **Security Scan**: Execute Aikido Docker scan for vulnerabilities
4. **Architecture Query**: Pull codebase context from DeepWiki
5. **Context Search**: Query Kontext vault for relevant documentation
6. **Save Results**: Persist complete analysis to Kontext vault
7. **Report Findings**: Present comprehensive 360° analysis

### Example Analysis Output

```
## Behavioral Patterns Detected

**Stress Indicators:**
- 47 commits (62%) made between 10pm-2am
- Average commit size: 461 lines (indicates batch work)
- 12 commits with minimizing language ("just", "quick fix")

**Avoidance Patterns:**
- 62% of codebase untouched in last 50 commits
- Critical files (auth.rs, db.rs) avoided for 3 weeks
- Vague messages on significant changes (8 occurrences)

## Security Findings (Aikido)

**Critical Issues: 3**
- Hardcoded secret in config.rs:42
- SQL injection vulnerability in query.rs:156
- Unvalidated user input in api.rs:89

**Correlation:**
Late-night commits (stressed state) show 2.3x higher vulnerability rate
compared to daytime commits.

## Recommendations

1. Address sleep schedule: 62% night commits suggests burnout risk
2. Refactor avoided files: Technical debt accumulating in core modules
3. Fix security issues: All 3 critical issues in late-night commit batches
4. Implement pre-commit security hooks to catch issues before commit
```

## Development

### Project Structure

```
codex-d/
├── src/
│   ├── main.rs              # GPUI app entry + UI
│   ├── types.rs             # Core data types
│   ├── git_analyzer.rs      # Git pattern detection
│   ├── codex_adapter.rs     # ACP client
│   ├── storage.rs           # SQLite persistence
│   └── ui/
│       ├── components.rs    # Reusable UI components
│       └── timeline.rs      # Timeline view
├── mcp-servers/
│   └── mcp_codex_psychology/
│       └── mcp_server/
│           ├── server.py            # FastMCP tools
│           ├── database.py          # SQLite schema
│           └── aikido_integration.py # Docker scanner
└── codex-acp/               # Agent Client Protocol submodule
```

### Architecture Flow

```
┌─────────────┐
│  GPUI UI    │  User selects repo
└──────┬──────┘
       │
       ▼
┌─────────────────┐
│ Git Analyzer    │  Scan commit patterns
└──────┬──────────┘
       │
       ▼
┌─────────────────┐
│ Codex Adapter   │  Send to ACP via ACI.dev
└──────┬──────────┘
       │
       ▼
┌─────────────────────────┐
│ MCP Psychology Server   │  22 tools orchestrated
│  ├─ Behavioral Analysis │
│  ├─ Aikido Scanner      │  (Docker)
│  ├─ DeepWiki Query      │  (via gate22)
│  └─ Kontext Storage     │  (vault)
└──────┬──────────────────┘
       │
       ▼
┌─────────────────┐
│ Timeline UI     │  Display results with tool visibility
└─────────────────┘
```

## Partner Technologies Used

This project integrates **4 partner technologies** for the Open Innovation track:

### Core Integration
- ✅ **OpenAI** - Via personal account through Agent Client Protocol (zero API costs)
- ✅ **Lovable** - Demo site at [https://codexd.lovable.app/](https://codexd.lovable.app/)

### Side Challenges
- ✅ **ACI.dev** - Agent Client Protocol adapter + DeepWiki observability for full transparency
- ✅ **Kontext.dev** - Private encrypted vault storage for persistent analysis results

### Additional Technology
- **Aikido Security** - Docker-based vulnerability scanning (SAST, secrets, dependencies)

## Contributing

Contributions welcome! This is an open-source project exploring the intersection of developer wellbeing and code quality.

### Areas for Contribution

- Additional behavioral pattern detectors
- More security scanner integrations
- Linux/Windows GPUI support
- Additional MCP tool integrations
- Improved timeline UI/UX

## License

MIT

## Acknowledgments

- [GPUI](https://github.com/zed-industries/zed) - Native UI framework
- [Agent Client Protocol](https://agentclientprotocol.com/) - Agent communication standard
- [FastMCP](https://github.com/jlowin/fastmcp) - MCP server framework
- [Aikido Security](https://www.aikido.dev/) - Security scanning platform
- [Kontext.dev](https://kontext.dev/) - Knowledge vault platform

---

**Built for the intersection of developer psychology and code quality.**
