# codex-d

**Developer Psychology Analysis via Git Patterns**

> "It's like therapy for your codebase - but it actually tells you what you're avoiding."

## What is codex-d?

codex-d analyzes your git commit history to detect behavioral patterns and generates psychological observations about your development habits. It's not static analysis—it's pattern recognition that reveals:

- Commitment issues (frequent small commits vs. rare large ones)
- Minimizing language in commit messages vs. actual change size
- Avoidance behaviors (what you're NOT working on)
- Self-deception (commit messages vs. actual changes)

## How It Works

```
Git Repo → Pattern Detection → AI Analysis (Codex) → Psychological Observation
```

1. **Git Analysis**: Scans your last 50 commits for behavioral patterns
2. **Codex Inference**: Uses AI to interpret patterns psychologically
3. **Observation**: Displays constructive reflection with a question

## Quick Start

### Prerequisites

- macOS (GPUI native app)
- [Codex](https://codex.com) account (uses your existing authentication)
- Rust toolchain (for building)

### Installation

```bash
# Clone the repository
git clone <your-repo-url>
cd codex-d

# Initialize submodules
git submodule update --init --recursive

# Build codex-acp
cd codex-acp
cargo build --release
cd ..

# Build codex-d
cargo build --release

# Run
cargo run --release
```

### First Analysis

1. Launch the app
2. Select a git repository
3. Click "Analyze"
4. Read your observation

## Architecture

**Ultra-lean MVP:**
- Git pattern detection (local, no API)
- Codex ACP integration (uses your auth)
- GPUI native macOS app
- SQLite for observation history
- **~720 lines of Rust**

## Environment Variables

**None required!** Codex authentication is handled by codex-acp automatically.

Optional:
```bash
CODEXD_DB_PATH=/custom/path/observations.db  # Default: ~/.local/share/codex-d/observations.db
RUST_LOG=debug                               # Default: info
```

## Example Observation

```
You consistently use minimizing language in your commit messages
('quick fix', 'small change') while making substantial modifications—14
commits averaged 176 lines each. This pattern suggests you're downplaying
your work, possibly to manage your own expectations or reduce perceived
risk. What would it feel like to acknowledge the true scope of your work
in your commit messages?
```

## Project Status

**Phase 1 (MVP):** In Progress
- [x] Project setup
- [ ] Git analyzer
- [ ] Codex adapter
- [ ] GPUI UI
- [ ] SQLite storage
- [ ] End-to-end test

## License

MIT
