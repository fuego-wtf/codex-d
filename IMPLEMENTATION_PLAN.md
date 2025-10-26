# codex-d: Full Implementation Plan
## Option B: Complete Roasting Integration with Official Session Management

---

## 🎯 Architecture Overview

```
User launches codex-d
      ↓
SessionManager creates UUID session
      ↓
Discovery Questions (via mcp_codex_psychology)
  - "How is this repo going? (1-10)"
  - "What's the potential unleashed? (Low/Mid/High)"
  - "Why?"
      ↓
Answers logged to session + sent to kontext.dev (via gate22)
      ↓
Enrichment Phase
  ├── GitAnalyzer → patterns
  └── AikidoScanner (CLI) → security issues
      ↓
Results saved to mcp_codex_psychology database
      ↓
Chat Phase (Codex with righteous roasting)
  ├── Uses MCP tools for context
  ├── Tool calls visible (gate22-style transparency)
  └── Generates fix prompts for sub-agents
      ↓
Longitudinal tracking across sessions
```

---

## 📦 Component Breakdown

### 1. Session Management (Rust)

**File**: `src/session_manager.rs`

**Pattern**: Ported from `secretsoul-gpui/src/workspace/session_manager.rs`

```rust
// Session struct
pub struct CodexSession {
    pub id: Uuid,                          // UUID from secretsoul pattern
    pub repo_path: String,
    pub repo_name: String,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub file_path: PathBuf,                 // ~/.codex-d/sessions/{uuid}.md

    // Discovery answers (from MCP)
    pub repo_rating: Option<u8>,            // 1-10
    pub potential: Option<String>,          // Low/Mid/High
    pub reasoning: Option<String>,          // Why?
    pub knowledge_base_id: Option<String>,  // kontext.dev KB ID

    // Analysis results
    pub git_analysis: Option<GitAnalysis>,
    pub security_scan: Option<SecurityScan>,
    pub patterns_detected: Vec<String>,
    pub severity: f32,

    // Conversation
    pub messages: Vec<Message>,
}

// SessionManager
pub struct SessionManager {
    sessions: Vec<CodexSession>,
    active_session_id: Option<Uuid>,
    sessions_dir: PathBuf,                  // ~/.codex-d/sessions
}

impl SessionManager {
    pub fn new() -> Result<Self>
    pub fn create_session_for_repo(&mut self, repo_path: String) -> Result<Uuid>
    pub fn save_discovery_answers(&mut self, rating: u8, potential: String, reasoning: String) -> Result<()>
    pub fn get_past_sessions_for_repo(&self, repo_path: &str) -> Vec<&CodexSession>
    pub fn add_message_to_active(&mut self, message: Message) -> Result<()>
    pub fn load_all_sessions(&mut self) -> Result<()>
    fn write_session_to_file(&self, session: &CodexSession) -> Result<()>
}
```

**File Format** (markdown, like secretsoul):
```markdown
# Session: projectname
ID: 550e8400-e29b-41d4-a716-446655440000
Created: 2025-01-15T10:30:00Z
Repo: /Users/user/project

## Discovery
Rating: 7
Potential: Mid
Reasoning: Good foundation but needs security hardening

## Analysis Summary
Git Patterns: 3 detected
Security Issues: 12 found
Severity: 0.67

## Conversation

### User - 2025-01-15T10:35:00Z
What's wrong with my auth code?

### Assistant - 2025-01-15T10:35:15Z
I notice auth vulnerabilities appearing in 3 consecutive scans...
```

---

### 2. MCP Server (Python)

**Location**: `mcp-servers/mcp_codex_psychology/`

**Extend existing** `mcp_server/server.py` with:

#### New Tools

```python
@mcp.tool()
def start_session(repo_path: str, repo_name: str) -> str:
    """
    Start a new analysis session with discovery questions.

    Returns:
        JSON with session_id and discovery questions

    🤖 CODEX: This is called by the UI when user selects a repo.
    Return the questions for the UI to display.
    """
    global _current_session_id, _current_repo_path

    db = get_database()
    session_id = db.create_session(repo_path, repo_name)
    _current_session_id = session_id
    _current_repo_path = repo_path

    questions = [
        {
            "id": "repo_rating",
            "text": "How is this repo going?",
            "type": "rating",
            "min": 1,
            "max": 10,
            "description": "Rate your project's current state"
        },
        {
            "id": "potential",
            "text": "What's the potential unleashed?",
            "type": "choice",
            "options": ["Low", "Mid", "High"],
            "description": "How much potential does this project have?"
        },
        {
            "id": "reasoning",
            "text": "Why?",
            "type": "text",
            "description": "Tell us more about your project"
        }
    ]

    # Check for past sessions
    context = db.get_repo_context(repo_path)

    return json.dumps({
        "status": "started",
        "session_id": session_id,
        "questions": questions,
        "past_sessions": context['repo_info']['total_scans'],
        "message": "Session started. Ready for discovery questions."
    })


@mcp.tool()
def submit_discovery_answers(answers: str) -> str:
    """
    Save discovery answers and create knowledge base via gate22/kontext.dev.

    Args:
        answers: JSON string with { "repo_rating": 7, "potential": "Mid", "reasoning": "..." }

    Returns:
        JSON with status and knowledge_base_id

    🤖 CODEX: After UI collects answers, this saves them and sends to kontext.dev.
    The knowledge base will be used for future context enrichment.
    """
    global _current_session_id, _current_repo_path

    if not _current_session_id:
        return json.dumps({"status": "error", "message": "No active session"})

    db = get_database()
    answers_dict = json.loads(answers)

    # Save locally
    db.save_discovery_answers(
        _current_session_id,
        rating=answers_dict.get("repo_rating"),
        potential=answers_dict.get("potential"),
        reasoning=answers_dict.get("reasoning")
    )

    # Send to kontext.dev via gate22
    kb_id = send_to_kontext_via_gate22(
        repo_path=_current_repo_path,
        repo_name=Path(_current_repo_path).name,
        discovery_data=answers_dict
    )

    db.update_session_knowledge_base(_current_session_id, kb_id)

    return json.dumps({
        "status": "success",
        "knowledge_base_id": kb_id,
        "message": "Answers saved and knowledge base created"
    })


@mcp.tool()
def save_scan_results(
    session_id: int,
    git_analysis: str,
    security_analysis: str
) -> str:
    """
    Save enrichment results to database.

    🤖 CODEX: Called after GitAnalyzer + AikidoScanner complete.
    This persists all findings for roasting and tracking.
    """
    db = get_database()

    git_data = json.loads(git_analysis)
    security_data = json.loads(security_analysis)

    # Save git patterns
    for pattern in git_data.get("patterns", []):
        db.save_git_pattern(
            session_id=session_id,
            pattern_type=pattern["pattern_type"],
            severity=pattern["severity"],
            evidence=json.dumps(pattern["evidence"])
        )

    # Save security issues
    for issue in security_data.get("issues", []):
        db.save_security_issue(
            session_id=session_id,
            severity=issue["severity"],
            category=issue["category"],
            summary=issue["summary"],
            file_path=issue.get("file_path"),
            line_number=issue.get("line_number")
        )

    # Detect recurring issues
    recurring = db.detect_recurring_issues(session_id)

    return json.dumps({
        "status": "success",
        "git_patterns_saved": len(git_data.get("patterns", [])),
        "security_issues_saved": len(security_data.get("issues", [])),
        "recurring_issues": recurring,
        "message": "Scan results saved. Ready for roasting."
    })


@mcp.tool()
def generate_fix_prompt(issue_id: int) -> str:
    """
    Generate actionable fix prompt for sub-agent dispatch.

    🤖 CODEX: When user asks "how do I fix this?", generate a structured prompt.
    The UI will display this in a special "Agent Fix Prompt" component with copy button.
    """
    db = get_database()
    issue = db.get_issue_by_id(issue_id)

    if not issue:
        return json.dumps({"status": "error", "message": "Issue not found"})

    # Build context-aware fix prompt
    prompt = f"""# Fix Security Issue

**Issue**: {issue['summary']}
**Severity**: {issue['severity']}
**Category**: {issue['category']}
**Location**: {issue.get('file_path', 'N/A')}:{issue.get('line_number', 'N/A')}

## Task
1. Locate the vulnerable code
2. Implement the fix following security best practices for {issue['category']}
3. Add tests to prevent regression
4. Commit with clear message

## Context
{issue.get('context', 'No additional context')}

## Expected Outcome
- Issue resolved
- No new vulnerabilities introduced
- Tests passing

## References
- OWASP Guidelines for {issue['category']}
- Security best practices for the language/framework
"""

    return json.dumps({
        "status": "success",
        "issue_id": issue_id,
        "fix_prompt": prompt,
        "estimated_difficulty": estimate_difficulty(issue),
        "message": "Fix prompt generated. Ready for agent dispatch or manual copy."
    })


@mcp.tool()
def query_recurring_issues(category: Optional[str] = None) -> str:
    """
    Get issues that appear across multiple scans (recurring problems).

    🤖 CODEX: This is where righteous roasting begins:
    "I notice this is the 4th scan with hardcoded secrets..."
    Start curious, not savage.
    """
    global _current_repo_path

    if not _current_repo_path:
        return json.dumps({"status": "error", "message": "No repo set"})

    db = get_database()
    recurring = db.get_recurring_issues(_current_repo_path, category)

    return json.dumps({
        "status": "success",
        "recurring_count": len(recurring),
        "issues": recurring,
        "message": f"Found {len(recurring)} recurring issues"
    })


@mcp.tool()
def flag_behavioral_pattern(
    pattern_name: str,
    evidence: str,
    severity: str
) -> str:
    """
    Flag a behavioral pattern detected across git + security data.

    Examples:
      - "rush_commits_with_secrets"
      - "auth_avoidance"
      - "minimizing_language"

    🤖 CODEX: When you connect dots between git behavior and security,
    flag it here. This builds a profile over time.
    """
    global _current_session_id

    if not _current_session_id:
        return json.dumps({"status": "error", "message": "No active session"})

    db = get_database()
    db.flag_behavioral_pattern(_current_session_id, pattern_name, evidence, severity)

    return json.dumps({
        "status": "success",
        "pattern_name": pattern_name,
        "message": "Pattern flagged for future roasting"
    })
```

---

### 3. Database Schema Extension

**File**: `mcp-servers/mcp_codex_psychology/mcp_server/database.py`

**Add New Tables**:

```sql
-- Session tracking (like secretsoul)
CREATE TABLE IF NOT EXISTS sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_uuid TEXT UNIQUE NOT NULL,  -- UUID from Rust SessionManager
    repo_id INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP,

    -- Discovery answers
    repo_rating INTEGER,
    potential TEXT,
    reasoning TEXT,
    knowledge_base_id TEXT,  -- kontext.dev KB ID

    FOREIGN KEY (repo_id) REFERENCES repositories(id)
);

-- Security issues (from AikidoScanner)
CREATE TABLE IF NOT EXISTS security_issues (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER,
    severity TEXT NOT NULL,  -- critical, high, medium, low
    category TEXT NOT NULL,  -- sql_injection, xss, secrets, etc
    summary TEXT NOT NULL,
    file_path TEXT,
    line_number INTEGER,
    issue_url TEXT,  -- Aikido issue URL
    fix_attempted BOOLEAN DEFAULT 0,
    fixed_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);

-- Behavioral patterns (cross-cutting)
CREATE TABLE IF NOT EXISTS behavioral_patterns (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER,
    pattern_name TEXT NOT NULL,  -- rush_commits_with_secrets, auth_avoidance, etc
    evidence TEXT,  -- JSON with supporting data
    severity TEXT,  -- low, medium, high
    first_flagged TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    occurrence_count INTEGER DEFAULT 1,
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);

-- Recurring issues tracker
CREATE TABLE IF NOT EXISTS recurring_issues (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    repo_id INTEGER,
    issue_signature TEXT NOT NULL,  -- Hash of category+summary
    first_seen TIMESTAMP,
    last_seen TIMESTAMP,
    occurrence_count INTEGER DEFAULT 1,
    UNIQUE(repo_id, issue_signature),
    FOREIGN KEY (repo_id) REFERENCES repositories(id)
);

-- Fix attempts log
CREATE TABLE IF NOT EXISTS fix_attempts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER,
    issue_type TEXT NOT NULL,
    fix_description TEXT,
    outcome TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);
```

---

### 4. Aikido Scanner Integration

**File**: `src/aikido_scanner.rs`

```rust
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::Command;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScan {
    pub issues_found: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub issues: Vec<SecurityIssue>,
    pub scan_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIssue {
    pub severity: SecuritySeverity,
    pub category: String,
    pub summary: String,
    pub file_path: Option<String>,
    pub line_number: Option<usize>,
    pub issue_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Critical,
    High,
    Medium,
    Low,
}

pub struct AikidoScanner;

impl AikidoScanner {
    pub async fn scan(
        repo_path: &str,
        progress_tx: mpsc::UnboundedSender<String>,
    ) -> Result<SecurityScan> {
        let start = std::time::Instant::now();

        progress_tx.send(format!("🛡️  Starting Aikido security scan...")).ok();

        // Run aikido-scanner CLI
        // Assuming: aikido-scanner scan --json --repo <path>
        let output = Command::new("aikido-scanner")
            .arg("scan")
            .arg("--json")
            .arg("--repo")
            .arg(repo_path)
            .output()?;

        if !output.status.success() {
            anyhow::bail!("Aikido scan failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        // Parse JSON output
        let scan_output: AikidoScanOutput = serde_json::from_slice(&output.stdout)?;

        let mut issues = Vec::new();
        let mut critical_count = 0;
        let mut high_count = 0;
        let mut medium_count = 0;
        let mut low_count = 0;

        for issue in scan_output.issues {
            match issue.severity {
                SecuritySeverity::Critical => critical_count += 1,
                SecuritySeverity::High => high_count += 1,
                SecuritySeverity::Medium => medium_count += 1,
                SecuritySeverity::Low => low_count += 1,
            }
            issues.push(issue);
        }

        let duration = start.elapsed().as_millis() as u64;

        progress_tx.send(format!(
            "✅ Aikido scan complete: {} issues found ({} critical, {} high)",
            issues.len(), critical_count, high_count
        )).ok();

        Ok(SecurityScan {
            issues_found: issues.len(),
            critical_count,
            high_count,
            medium_count,
            low_count,
            issues,
            scan_duration_ms: duration,
        })
    }
}

#[derive(Deserialize)]
struct AikidoScanOutput {
    issues: Vec<SecurityIssue>,
}
```

---

### 5. Enrichment Flow Integration

**File**: `src/main.rs` (modify `start_enrichment`)

```rust
async fn start_enrichment(
    repo_path: String,
    session_manager: &mut SessionManager,
    mcp_client: &mut McpClient,
) -> Result<()> {
    let (progress_tx, mut progress_rx) = mpsc::unbounded_channel();

    // Create session
    let session_id = session_manager.create_session_for_repo(repo_path.clone())?;

    // Start MCP session
    let mcp_session_result = mcp_client.start_session(&repo_path, &repo_name)?;
    let questions = mcp_session_result["questions"].as_array().unwrap();

    // PHASE 1: Discovery Questions
    // (UI displays questions, user answers, then continues)

    // PHASE 2: Git Analysis
    progress_tx.send("🔍 Analyzing git patterns...".to_string()).ok();
    let git_analysis = GitAnalyzer::analyze(&repo_path).await?;

    // PHASE 3: Aikido Security Scan
    progress_tx.send("🛡️  Running security scan...".to_string()).ok();
    let security_scan = AikidoScanner::scan(&repo_path, progress_tx.clone()).await?;

    // PHASE 4: Save to MCP Database
    let git_json = serde_json::to_string(&git_analysis)?;
    let security_json = serde_json::to_string(&security_scan)?;

    let save_result = mcp_client.save_scan_results(
        mcp_session_id,
        &git_json,
        &security_json
    )?;

    // PHASE 5: Build Righteous Prompt
    let recurring_count = save_result["recurring_issues"].as_array().unwrap().len();

    let system_prompt = build_righteous_prompt(
        &git_analysis,
        &security_scan,
        recurring_count
    );

    // PHASE 6: Initialize Chat with Codex
    // ... (existing chat initialization code)

    Ok(())
}
```

---

### 6. Gate22 Transparency Layer

**File**: `src/gate22_client.rs`

```rust
/// Client for gate22 gateway (wraps kontext.dev)
pub struct Gate22Client {
    endpoint: String,  // e.g., http://localhost:8080
}

impl Gate22Client {
    pub fn send_to_kontext(
        &self,
        repo_path: &str,
        repo_name: &str,
        discovery_data: serde_json::Value
    ) -> Result<String> {
        // POST to gate22, which:
        // 1. Logs everything
        // 2. Forwards to kontext.dev
        // 3. Creates knowledge base
        // 4. Returns KB ID

        let response = reqwest::blocking::Client::new()
            .post(format!("{}/kontext/create_kb", self.endpoint))
            .json(&json!({
                "repo_path": repo_path,
                "repo_name": repo_name,
                "discovery": discovery_data,
            }))
            .send()?;

        let result: serde_json::Value = response.json()?;
        Ok(result["knowledge_base_id"].as_str().unwrap().to_string())
    }
}
```

**In UI**: Show gate22 tool calls like:
```
🌐 gate22 → kontext.dev
   Creating knowledge base for "my-project"
   ✅ Knowledge base created: kb_abc123
```

---

### 7. UI Components for Discovery Questions

**File**: `src/ui/discovery.rs`

```rust
pub fn render_discovery_form(
    questions: &[DiscoveryQuestion],
    answers: &mut DiscoveryAnswers,
) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_4()
        .child(
            div()
                .text_lg()
                .font_weight(FontWeight::BOLD)
                .child("Before we begin, tell us about your project:")
        )
        .children(questions.iter().map(|q| {
            render_question(q, answers)
        }))
        .child(
            button()
                .child("Continue")
                .on_click(|_, cx| {
                    // Submit answers to MCP
                })
        )
}

fn render_question(q: &DiscoveryQuestion, answers: &mut DiscoveryAnswers) -> Div {
    match &q.input_type {
        InputType::Rating { min, max } => {
            // Slider 1-10
            render_rating_slider(q, answers, *min, *max)
        }
        InputType::Choice { options } => {
            // Radio buttons
            render_choice_buttons(q, answers, options)
        }
        InputType::Text => {
            // Textarea
            render_text_area(q, answers)
        }
    }
}
```

---

## 🚀 Implementation Phases

### Phase 1: Foundation (Days 1-2)
- [ ] Create `src/session_manager.rs` (port from secretsoul)
- [ ] Extend database schema in `mcp_server/database.py`
- [ ] Add discovery question tools to `mcp_server/server.py`
- [ ] Test session creation + discovery flow

### Phase 2: Enrichment (Days 3-4)
- [ ] Create `src/aikido_scanner.rs`
- [ ] Integrate AikidoScanner CLI (mock if not available)
- [ ] Add `save_scan_results` MCP tool
- [ ] Test enrichment pipeline

### Phase 3: Roasting (Days 5-6)
- [ ] Add `generate_fix_prompt` tool
- [ ] Add `query_recurring_issues` tool
- [ ] Add `flag_behavioral_pattern` tool
- [ ] Build righteous prompt system

### Phase 4: Gate22 Integration (Days 7-8)
- [ ] Create `src/gate22_client.rs`
- [ ] Implement kontext.dev KB creation
- [ ] Add transparency UI for gate22 calls
- [ ] Test knowledge base flow

### Phase 5: UI Polish (Days 9-10)
- [ ] Discovery question form UI
- [ ] Agent fix prompt component (with copy)
- [ ] Session history viewer
- [ ] Timeline event for MCP connections

### Phase 6: Documentation & Testing (Days 11-12)
- [ ] Update README with complete architecture
- [ ] Add USER_JOURNEY examples
- [ ] End-to-end testing
- [ ] Deploy and validate

---

## 📝 Success Criteria

1. ✅ **Session Management**: UUID-based sessions persisted to `~/.codex-d/sessions/`
2. ✅ **Discovery Questions**: Asked on app launch, answers logged + sent to kontext.dev
3. ✅ **Enrichment**: Git + Aikido scans complete successfully
4. ✅ **MCP Storage**: All results persisted to database with recurring issue detection
5. ✅ **Roasting**: Righteous prompt system with behavioral pattern detection
6. ✅ **Fix Prompts**: Generated and displayable with copy button
7. ✅ **Gate22 Transparency**: All kontext.dev calls logged and visible
8. ✅ **Longitudinal**: Past sessions queryable, patterns tracked over time

---

## 🔧 Dependencies

### Rust
- `uuid` - For session IDs
- `chrono` - For timestamps
- `tokio` - For async aikido scanner
- `reqwest` - For gate22 HTTP client

### Python
- `fastmcp` - Existing
- `gitpython` - Existing
- `sqlite3` - Existing
- `requests` - For kontext.dev calls via gate22

### External
- **AikidoScanner CLI** - Security scanning tool
- **gate22 server** - MCP gateway for kontext.dev
- **kontext.dev** - Knowledge base service

---

Ready to begin implementation! 🚀
