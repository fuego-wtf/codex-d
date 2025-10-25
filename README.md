# codex'd

**Developer Psychology Analysis via Git Patterns**

> "It's like therapy for your codebase - but it actually tells you what you're avoiding."

## What is codex'd?

codex'd analyzes your git commit history to detect behavioral patterns and generates psychological observations about your development habits. It's not static analysis—it's pattern recognition that reveals:

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
- [Codex ACP](https://agentclientprotocol.com/llms.txt) integration (uses your auth)
- [GPUI](https://github.com/zed-industries/zed) native macOS app with [gpui-component](https://longbridge.github.io/gpui-component/llms.txt) UI library
- SQLite for observation history
- **~720 lines of Rust**

### Stack References

**UI Components:**
- [GPUI Component Library](https://longbridge.github.io/gpui-component/llms.txt) - TextInput, Button, Scrollable, and other UI components
- [GPUI Framework](https://github.com/zed-industries/zed/tree/main/crates/gpui) - GPU-accelerated native UI framework

**AI Integration:**
- [Agent Client Protocol (ACP)](https://agentclientprotocol.com/llms.txt) - Protocol specification for AI agent communication
- [codex-acp](https://github.com/zed-industries/codex-acp) - ACP client implementation for Codex
- [Codex Documentation](https://github.com/openai/codex/tree/main/docs) - Codex AI platform docs

### Component Usage

**UI Components (from gpui-component):**
- `TextInput` - Single-line text input with built-in state management
- `Button` - Clickable button with variants (primary, ghost, etc.)
- `Scrollable` - Scrollable container with custom scrollbars
- `Root` - Required wrapper component for gpui-component apps

**State Management:**
- `InputState` - Manages text input state (text, selection, history)
- `Entity<T>` - GPUI entity for reactive state
- `Context<T>` - Component context for updates and notifications

**Example:**
```rust
// Create input state
let input_state = cx.new(|cx| InputState::new(window, cx));

// Render input with button
TextInput::new(&input_state).w(px(350.0))
Button::new("my-button").label("Click Me").primary()
```

## Environment Variables

**None required!** Codex authentication is handled by codex-acp automatically.

Optional:
```bash
CODEXD_DB_PATH=/custom/path/observations.db  # Default: ~/.local/share/codex-d/observations.db
RUST_LOG=debug                               # Default: info
```

## Developer Guide

### Application Flow

codex'd uses a **3-page state machine** architecture:

```
AwaitingRepoSelection → Enriching → ChatActive
```

**Page 1: Repository Selection**
- User enters repository path via `TextInput` or clicks "Browse Folders" `Button`
- On submit: validate `.git` folder exists
- Transition to Enriching state

**Page 2: Enrichment**
- Run `GitAnalyzer::analyze()` asynchronously (using `tokio::task::spawn_blocking`)
- Stream progress updates via `LifecycleEvent`
- Initialize `CodexAdapter` with system prompt from git analysis
- Transition to ChatActive state

**Page 3: Chat Interface**
- User sends messages via `TextInput` + "Send" `Button`
- Messages saved to SQLite via `Storage`
- Stream Codex responses via `CodexAdapter::send_message()`
- Display lifecycle events (tool calls, permissions) in timeline

### Key Implementation Patterns

**Async Git Analysis:**
```rust
// Git2 is blocking, so we use spawn_blocking
let analysis = tokio::task::spawn_blocking(move || {
    GitAnalyzer::analyze_blocking(&repo_path)
}).await?;
```

**Codex Streaming:**
```rust
adapter.send_message(user_message, |event| {
    match event {
        StreamEvent::MessageChunk(text) => {
            // Append to assistant message
        }
        StreamEvent::LifecycleEvent(event) => {
            // Show tool usage
        }
    }
})?;
```

**Component State Updates:**
```rust
// Reading input text
let content = self.input_state.read(cx).text().to_string();

// Clearing input
self.input_state.update(cx, |state, cx| {
    state.set_value("", window, cx);
});
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
- [x] Type system (AppState, Message, StreamEvent, LifecycleEvent)
- [x] Git analyzer (async pattern detection with spawn_blocking)
- [x] Codex adapter (JSON-RPC client for codex-acp subprocess)
- [x] SQLite storage (conversation persistence)
- [x] GPUI UI (3-page state machine with gpui-component)
  - [x] Page 1: Repository selection (TextInput + Button)
  - [x] Page 2: Enrichment progress
  - [x] Page 3: Chat interface (TextInput + Button + Messages)
- [ ] Wire up functionality
  - [ ] File picker integration
  - [ ] Git analyzer → enrichment flow
  - [ ] Codex streaming responses
- [ ] End-to-end test

## License

MIT
