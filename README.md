# codex'd

**Evidence-Based Psychological Archaeology Through Git Forensics**

> "A developer therapist that traces your error loops through commit history."

## What is codex'd?

codex'd conducts **psychological archaeology** on your git repository—analyzing commit patterns to detect poorly designed workflows, error loops, and behavioral blindspots. It's not code review, it's **pattern-to-insight forensics** that builds evidence-based narratives about your development psychology:

- **Error loops**: Refactoring the same code every two weeks but never fixing root causes
- **Avoidance behaviors**: 62% of your codebase untouched while you batch-commit on weekends
- **Workflow anti-patterns**: Massive commits (>200 lines) that treat git as backup, not practice
- **Identity gaps**: What your README claims vs. what your commit history reveals

## How It Works (4-Phase Enrichment Architecture)

```
Phase 1: Enrichment → Phase 2: Synthesis → Phase 3: Streaming → Phase 4: Storage
```

**Phase 1: Parallel Context Gathering**
- **Git Analyzer**: Scans commits for behavioral patterns (refactoring frequency, commit sizes, file avoidance)
- **Kontext.dev API**: Extracts developer persona (commit style, README claims, stated priorities) *(coming soon)*
- **Aikido Scanner**: Detects security blindspots (auth vulnerabilities, injection risks) *(coming soon)*

**Phase 2: Evidence-Based Synthesis**
- Codex receives enriched context from all three sources
- Builds narrative: Evidence → Behavior → Error Loop → Blindspot → Question
- Generates ONE devastating observation grounded in specific metrics

**Phase 3: Real-Time Streaming**
- FastMCP server streams observation via SSE
- UI displays evidence-based narrative as it's generated
- No code review, no suggestions—just psychological archaeology

**Phase 4: Longitudinal Tracking**
- Saves observation to SQLite with timestamp
- Future sessions can reference: "Three months ago you had refactoring loops, now you have security blindspots"
- Tracks whether patterns persist or evolve

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

## Example Observation (Evidence-Based Narrative)

```
Your commit history shows 80% of your work happens on weekends with massive
batch commits averaging 461 lines. Yet 62% of your codebase—including critical
files like cli.ts, merger.ts, and sync.ts—hasn't been touched in your last
10 commits. You're batch-saving work on weekends while avoiding the complex
areas that actually need attention during the week. What are you protecting
yourself from by only working when no one else is watching?
```

**Narrative Structure:**
1. **Evidence**: "80% weekend commits, 461 lines average" (specific metrics)
2. **Behavior**: "Yet 62% untouched—cli.ts, merger.ts, sync.ts" (file-level avoidance)
3. **Error Loop**: "Batch-saving on weekends while avoiding complex areas" (pattern identified)
4. **Blindspot**: "Only working when no one else is watching" (psychological insight)
5. **Question**: Forces reflection on the protective behavior

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

## Related MCP Resources

- [MCP Kontext Server](https://docs.kontext.dev/mcp/kontext.md): Hosted Kontext MCP server https://docs.kontext.dev/llms.txt

## License

MIT
