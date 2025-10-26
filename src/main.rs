// codex-d: Developer Psychology Analysis
// Chat-based UI: Repo selection → Enrichment → Conversation

mod types;
mod git_analyzer;
mod codex_adapter;
mod storage;
mod ui;

use gpui::*;
use gpui::prelude::*;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::runtime::Handle;

use gpui_component::{
    button::{Button, ButtonVariants},
    input::{TextInput, InputState},
    Root,
};

use types::{AppState, Message, TimelineEvent};
use storage::Storage;
use git_analyzer::GitAnalyzer;
use codex_adapter::CodexAdapter;
use ui::timeline::render_timeline;
use ui::components::{render_streaming_thought, render_streaming_message, render_streaming_tool_call};

fn main() {
    env_logger::init();

    // Create Tokio runtime for async operations
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let tokio_handle = runtime.handle().clone();

    // Keep runtime alive
    std::thread::spawn(move || {
        runtime.block_on(async {
            std::future::pending::<()>().await;
        });
    });

    Application::new().run(move |cx: &mut App| {
        // Initialize gpui-component
        gpui_component::init(cx);

        let bounds = Bounds::centered(None, size(px(900.0), px(700.0)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| {
                let codex_view = cx.new(|cx| CodexView::new(tokio_handle.clone(), window, cx));
                cx.new(|cx| Root::new(codex_view.into(), window, cx))
            },
        )
        .unwrap();
    });
}

struct CodexView {
    app_state: AppState,
    selected_repo: Option<String>,
    repo_path_input: Entity<InputState>,
    messages: Vec<Message>,
    timeline_events: Vec<TimelineEvent>,
    lifecycle_events: Vec<types::LifecycleEvent>,
    message_input_state: Entity<InputState>,
    storage: Option<Arc<Storage>>,
    tokio_handle: Handle,
    codex_adapter: Option<Arc<CodexAdapter>>,
    is_loading: bool,
    error_message: Option<String>,
    // Streaming state (temporary until event completes)
    current_thought_buffer: String,
    current_message_buffer: String,
    active_tool_calls: HashMap<String, (types::ToolCallEvent, String)>, // (event, output)
    // Enrichment timer
    enrichment_start_time: Option<std::time::Instant>,
    enrichment_elapsed: f32, // seconds
    // Timeline scrolling
    timeline_scroll_handle: ScrollHandle,
}

impl CodexView {
    fn new(tokio_handle: Handle, window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Initialize storage
        let db_path = dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("codex-d")
            .join("messages.db");

        let storage = Storage::new(&db_path.to_string_lossy())
            .map(|s| Arc::new(s))
            .ok();

        if storage.is_none() {
            eprintln!("Failed to initialize storage");
        }

        // Create input states for text fields
        let repo_path_input = cx.new(|cx| InputState::new(window, cx));
        let message_input_state = cx.new(|cx| InputState::new(window, cx));

        Self {
            app_state: AppState::AwaitingRepoSelection,
            selected_repo: None,
            repo_path_input,
            messages: Vec::new(),
            timeline_events: Vec::new(),
            lifecycle_events: Vec::new(),
            message_input_state,
            storage,
            tokio_handle,
            codex_adapter: None,
            is_loading: false,
            error_message: None,
            current_thought_buffer: String::new(),
            current_message_buffer: String::new(),
            active_tool_calls: HashMap::new(),
            enrichment_start_time: None,
            enrichment_elapsed: 0.0,
            timeline_scroll_handle: ScrollHandle::new(),
        }
    }

    fn on_browse_clicked(&mut self, cx: &mut Context<Self>) {
        // TODO: File picker implementation - complex async pattern
        // For now, user can type path directly in input field

        // Get path from input field
        let path_str = self.repo_path_input.read(cx).text().to_string();

        if path_str.trim().is_empty() {
            self.error_message = Some("Please enter a repository path".to_string());
            cx.notify();
            return;
        }

        let path = std::path::PathBuf::from(&path_str);
        let git_dir = path.join(".git");

        if git_dir.exists() && git_dir.is_dir() {
            // Valid git repo - proceed to enrichment
            self.error_message = None;
            self.on_repo_selected(path_str, cx);
        } else {
            // Not a git repo - show error
            self.error_message = Some(format!(
                "Not a git repository: {}\nPlease select a folder containing a .git directory",
                path_str
            ));
            cx.notify();
        }
    }

    fn on_repo_selected(&mut self, repo_path: String, cx: &mut Context<Self>) {
        self.selected_repo = Some(repo_path.clone());
        self.app_state = AppState::Enriching;
        self.is_loading = true;
        self.lifecycle_events.clear();
        self.messages.clear(); // Clear old messages from previous runs
        self.lifecycle_events.push(types::LifecycleEvent::running("Scanning git history".to_string()));
        cx.notify();

        // TODO: Make this async once we figure out the GPUI async pattern
        // For now, doing synchronous enrichment
        eprintln!("Starting git analysis for: {}", repo_path);

        // Note: This blocks the UI thread - not ideal but gets us working
        // We'll optimize with proper async later
        let tokio_handle = self.tokio_handle.clone();

        // For now, use a no-op progress callback (we'll add UI integration later)
        let analysis_result = tokio_handle.block_on(GitAnalyzer::analyze(&repo_path, |_step, _progress| {
            // Progress callback - will integrate with UI later
        }));

        match analysis_result {
            Ok(analysis) => {
                eprintln!("Git analysis complete: {} commits analyzed, {} patterns found",
                    analysis.total_commits_analyzed, analysis.patterns.len());

                // Build system prompt from analysis
                let patterns_summary = if analysis.patterns.is_empty() {
                    "No significant behavioral patterns detected in git history.".to_string()
                } else {
                    analysis.patterns.iter()
                        .map(|p| format!("• {}: {}", p.title, p.description))
                        .collect::<Vec<_>>()
                        .join("\n")
                };

                let system_prompt = format!(
                    "You are a developer psychologist practicing evidence-based conversational archaeology.\n\n\
                     ## GIT COMMIT PATTERNS (from {} commits analyzed, severity {:.1}/1.0)\n\n\
                     {}\n\n\
                     ## YOUR MISSION: GUIDE USERS TO ENRICH THEIR PROJECT\n\n\
                     You have MCP tools to analyze git patterns. Use them to:\n\
                     1. Surface blindspots users can't see themselves\n\
                     2. Ask questions that make them reflect deeply\n\
                     3. Guide them toward actionable improvements\n\
                     4. Build longitudinal understanding across sessions\n\n\
                     ## CONVERSATION STRATEGY (Socratic Guidance)\n\n\
                     **Phase 1: Discovery** (Current - gather context)\n\
                     - Ask about: project goals, team structure, customer, timeline\n\
                     - Use their answers to understand MOTIVATION and CONSTRAINTS\n\
                     - Build rapport through genuine curiosity\n\
                     - Listen for what they DON'T say\n\n\
                     **Phase 2: Investigation** (use MCP tools)\n\
                     When you have context, use tools to dig deeper:\n\
                     - `analyze_commit_patterns` - find commitment issues\n\
                     - `analyze_message_language` - detect minimizing/defensive patterns\n\
                     - `compare_message_vs_diff` - spot self-deception\n\
                     - `get_temporal_patterns` - reveal stress/overwork\n\
                     - `get_repo_context` - access memory from past sessions\n\n\
                     **Phase 3: Observation** (synthesize evidence)\n\
                     Create a 3-4 sentence observation:\n\
                     1. Cite EXACT git numbers (\"47 commits at night = 62%\")\n\
                     2. Connect to their stated goals (\"but you said X...\")\n\
                     3. Name the pattern (\"This suggests Y anti-pattern\")\n\
                     4. Ask ONE pointed question about the blindspot\n\n\
                     **Phase 4: Guidance** (lead toward action)\n\
                     Based on their response:\n\
                     - Validate their awareness\n\
                     - Suggest concrete experiments\n\
                     - Use `flag_repo_issue` to track the pattern\n\
                     - Offer to check back next session\n\n\
                     ## ABSOLUTE RULES\n\n\
                     - DO NOT read, analyze, or reference source code files\n\
                     - DO NOT do code review or technical assessment\n\
                     - Focus on BEHAVIOR patterns, not code quality\n\
                     - Use EXACT numbers from git data (never approximate)\n\
                     - Be conversational and empathetic - therapist, not linter\n\
                     - Each question should make them think deeper about their project\n\n\
                     **Your goal: Guide them to insights they'd never find alone. Make them WANT to share more about their project.**",
                    analysis.total_commits_analyzed,
                    analysis.severity,
                    patterns_summary
                );

                // Initialize Codex
                match CodexAdapter::new() {
                    Ok(adapter) => {
                        let adapter = Arc::new(adapter);

                        // Spawn codex-acp
                        if let Err(e) = adapter.spawn() {
                            eprintln!("Failed to spawn codex-acp: {}", e);
                            self.error_message = Some(format!("Failed to start Codex: {}", e));
                            self.app_state = AppState::AwaitingRepoSelection;
                            self.is_loading = false;
                            cx.notify();
                            return;
                        }

                        // Initialize ACP
                        if let Err(e) = adapter.initialize() {
                            eprintln!("Failed to initialize ACP: {}", e);
                            self.error_message = Some(format!("Failed to initialize Codex: {}", e));
                            self.app_state = AppState::AwaitingRepoSelection;
                            self.is_loading = false;
                            cx.notify();
                            return;
                        }

                        // Create session
                        match adapter.create_session(system_prompt, repo_path.clone()) {
                            Ok(session_id) => {
                                eprintln!("Codex session created: {}", session_id);

                                // Success!
                                self.codex_adapter = Some(adapter.clone());
                                self.lifecycle_events.push(types::LifecycleEvent::completed("Git analysis".to_string()));
                                self.lifecycle_events.push(types::LifecycleEvent::completed("AI initialized".to_string()));
                                self.app_state = AppState::ChatActive;
                                self.is_loading = false;
                                cx.notify();

                                // Create beautiful discovery greeting (Perplexity-style)
                                eprintln!("✨ Creating discovery experience...");
                                eprintln!("📊 Git patterns ready for synthesis");
                                eprintln!("🔧 MCP tools: codex-psychology available at :52848");

                                // Generate contextual discovery greeting based on git patterns
                                let pattern_count = analysis.patterns.len();
                                let commit_count = analysis.total_commits_analyzed;

                                let discovery_greeting = if pattern_count > 0 {
                                    let top_pattern = &analysis.patterns[0];
                                    format!(
                                        "## 🔍 Analysis Complete\n\n\
                                         I've analyzed **{} commits** and discovered **{} behavioral patterns**.\n\n\
                                         Most notable: *{}*\n\n\
                                         Before I share my observations, I'd like to understand the context.\n\n\
                                         **Tell me about this project:**\n\
                                         - What are you building?\n\
                                         - Who's working on it?\n\
                                         - What's the goal?",
                                        commit_count,
                                        pattern_count,
                                        top_pattern.title
                                    )
                                } else {
                                    format!(
                                        "## 👋 Let's Explore Your Code\n\n\
                                         I've analyzed **{} commits** from your repository.\n\n\
                                         To give you meaningful insights, I need to understand:\n\n\
                                         **What is this project?** Tell me about what you're building and who it's for.",
                                        commit_count
                                    )
                                };

                                // Add beautiful discovery greeting to timeline
                                self.timeline_events.push(TimelineEvent::AssistantMessage {
                                    content: discovery_greeting,
                                    timestamp: chrono::Utc::now().timestamp(),
                                });

                                eprintln!("✅ Discovery phase ready - Claude will use MCP tools when user responds");
                            }
                            Err(e) => {
                                eprintln!("Failed to create session: {}", e);
                                self.error_message = Some(format!("Failed to create session: {}", e));
                                self.app_state = AppState::AwaitingRepoSelection;
                                self.is_loading = false;
                                cx.notify();
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to create CodexAdapter: {}", e);
                        self.error_message = Some(format!("Failed to create Codex: {}", e));
                        self.app_state = AppState::AwaitingRepoSelection;
                        self.is_loading = false;
                        cx.notify();
                    }
                }
            }
            Err(e) => {
                eprintln!("Git analysis error: {}", e);
                self.error_message = Some(format!("Git analysis failed: {}", e));
                self.app_state = AppState::AwaitingRepoSelection;
                self.is_loading = false;
                cx.notify();
            }
        }
    }

    fn on_send_message(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let content = self.message_input_state.read(cx).text().to_string();

        if content.trim().is_empty() {
            return;
        }

        // Clear the input
        self.message_input_state.update(cx, |state, cx| {
            state.set_value("", window, cx);
        });

        // Add user message to timeline
        let now = chrono::Utc::now().timestamp();
        let user_event = TimelineEvent::UserMessage {
            content: content.clone(),
            timestamp: now,
        };
        self.timeline_events.push(user_event);

        // Also save to old messages vec for storage
        let message = Message::user(content.clone());
        self.messages.push(message.clone());

        if let Some(storage) = &self.storage {
            let _ = storage.save_message(&message);
        }

        cx.notify();

        // Send to Codex and stream response asynchronously
        if let Some(adapter) = &self.codex_adapter {
            let adapter = adapter.clone();
            let storage = self.storage.clone();
            let tokio_handle = self.tokio_handle.clone();

            // Create channel for streaming
            let (tx, rx) = smol::channel::bounded::<types::StreamEvent>(100);

            // Send via ACP asynchronously
            let adapter_clone = adapter.clone();
            let tx_clone = tx.clone();

            std::thread::spawn(move || {
                tokio_handle.block_on(async move {
                    let result = adapter_clone.send_message(content, move |event| {
                        let _ = tx.send_blocking(event);
                    });

                    if let Err(e) = result {
                        eprintln!("Failed to send message: {}", e);
                        let _ = tx_clone.send_blocking(types::StreamEvent::MessageChunk(
                            format!("\n\nError: {}", e)
                        ));
                    }
                });
            });

            // Process streaming events and build timeline
            cx.spawn(async move |view: WeakEntity<Self>, cx| {
                while let Ok(event) = rx.recv().await {
                    match event {
                        types::StreamEvent::MessageChunk(chunk) => {
                            view.update(cx, |view, cx| {
                                view.current_message_buffer.push_str(&chunk);
                                cx.notify();
                            })?;
                        }
                        types::StreamEvent::ThoughtChunk(chunk) => {
                            view.update(cx, |view, cx| {
                                view.current_thought_buffer.push_str(&chunk);
                                cx.notify();
                            })?;
                        }
                        types::StreamEvent::ToolCall(tool_call) => {
                            view.update(cx, |view, cx| {
                                view.active_tool_calls.insert(
                                    tool_call.tool_call_id.clone(),
                                    (tool_call, String::new())
                                );
                                cx.notify();
                            })?;
                        }
                        types::StreamEvent::ToolCallUpdate(update) => {
                            view.update(cx, |view, cx| {
                                if let Some((tool_call, output)) = view.active_tool_calls.get_mut(&update.tool_call_id) {
                                    if let Some(status) = &update.status {
                                        tool_call.status = status.clone();
                                    }
                                    if let Some(content) = &update.content {
                                        output.push_str(content);
                                    }
                                }
                                cx.notify();
                            })?;
                        }
                        _ => {}
                    }
                }

                // Stream complete - convert buffers to timeline events
                view.update(cx, |view, cx| {
                    let now = chrono::Utc::now().timestamp();

                    // Add thought to timeline if present
                    if !view.current_thought_buffer.is_empty() {
                        view.timeline_events.push(TimelineEvent::Thought {
                            content: view.current_thought_buffer.clone(),
                            timestamp: now,
                        });
                        view.current_thought_buffer.clear();
                    }

                    // Add tool calls to timeline
                    for (_id, (tool_call, output)) in view.active_tool_calls.drain() {
                        view.timeline_events.push(TimelineEvent::ToolCall {
                            tool_call_id: tool_call.tool_call_id,
                            title: tool_call.title,
                            kind: tool_call.kind,
                            status: tool_call.status,
                            locations: tool_call.locations,
                            output: if output.is_empty() { None } else { Some(output) },
                            timestamp: now,
                            mcp_server: tool_call.mcp_server,
                            routed_via: None, // TODO: Add gateway routing logic
                        });
                    }

                    // Add assistant message to timeline if present
                    if !view.current_message_buffer.is_empty() {
                        let msg_content = view.current_message_buffer.clone();
                        view.timeline_events.push(TimelineEvent::AssistantMessage {
                            content: msg_content.clone(),
                            timestamp: now,
                        });

                        // Save to storage
                        if let Some(storage) = &storage {
                            let msg = Message::assistant(msg_content);
                            let _ = storage.save_message(&msg);
                        }

                        view.current_message_buffer.clear();
                    }

                    cx.notify();
                })
            }).detach();
        } else {
            eprintln!("No Codex adapter available");
            let error_event = TimelineEvent::AssistantMessage {
                content: "Error: Codex not initialized".to_string(),
                timestamp: chrono::Utc::now().timestamp(),
            };
            self.timeline_events.push(error_event);
            cx.notify();
        }
    }
}

impl Render for CodexView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let bg_primary = rgb(0xfefefe);

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(bg_primary)
            .child(match self.app_state {
                AppState::AwaitingRepoSelection => self.render_page_1(cx),
                AppState::Enriching => self.render_page_2(cx),
                AppState::ChatActive => self.render_page_3(cx),
            })
    }
}

// Page implementations
impl CodexView {
    fn render_page_1(&mut self, cx: &mut Context<Self>) -> Div {
        let bg_surface = rgb(0xf5f5f5);
        let border_color = rgb(0xd0d0d0);

        let mut page = div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .size_full()
            .gap_4();

        // Show error message if present
        if let Some(error) = &self.error_message {
            page = page.child(
                div()
                    .w(px(500.0))
                    .px_4()
                    .py_3()
                    .bg(rgb(0xfee))
                    .border_1()
                    .border_color(rgb(0xf88))
                    .rounded_md()
                    .text_color(rgb(0xc00))
                    .child(error.clone())
            );
        }

        page.child(
                div()
                    .text_3xl()
                    .font_weight(FontWeight::BOLD)
                    .child("codex'd")
            )
            .child(
                div()
                    .text_base()
                    .text_color(rgb(0x666666))
                    .child("Developer Psychology Analysis")
            )
            .child(
                div()
                    .mt_8()
                    .w(px(500.0))
                    .h(px(200.0))
                    .border_2()
                    .border_color(border_color)
                    .border_dashed()
                    .rounded_lg()
                    .bg(bg_surface)
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_3()
                    .on_drop(cx.listener(|this, external_paths: &ExternalPaths, _, cx| {
                        // Extract the first path from dropped files
                        if let Some(path) = external_paths.paths().first() {
                            let path_str = path.to_string_lossy().to_string();
                            eprintln!("Dropped path: {}", path_str);

                            // Validate it's a git repo
                            let git_dir = path.join(".git");
                            if git_dir.exists() && git_dir.is_dir() {
                                this.error_message = None;
                                this.on_repo_selected(path_str, cx);
                            } else {
                                this.error_message = Some(format!(
                                    "Not a git repository: {}\nPlease drop a folder containing a .git directory",
                                    path_str
                                ));
                                cx.notify();
                            }
                        }
                    }))
                    .child(
                        div()
                            .text_2xl()
                            .child("📁")
                    )
                    .child(
                        div()
                            .text_base()
                            .child("Drag & drop a git repository here")
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x999999))
                            .child("or")
                    )
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        TextInput::new(&self.repo_path_input)
                            .w(px(350.0))
                    )
                    .child({
                        let view = cx.entity().clone();
                        Button::new("browse-button")
                            .label("Browse Folders")
                            .primary()
                            .on_click(move |_, _window, cx| {
                                view.update(cx, |view, cx| {
                                    view.on_browse_clicked(cx);
                                });
                            })
                    })
            )
    }

    fn render_page_2(&mut self, _cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .flex_col()
            .size_full()
            .p_6()
            .gap_4()
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::BOLD)
                    .child(format!(
                        "Analyzing {}",
                        self.selected_repo.as_deref().unwrap_or("repository")
                    ))
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .child("⏳ Enriching context...")
                    )
                    .child(
                        div()
                            .child("✅ Git history scanned")
                    )
                    .child(
                        div()
                            .child("🔄 Generating observation...")
                    )
            )
    }

    fn render_page_3(&mut self, cx: &mut Context<Self>) -> Div {
        let bg_user = rgb(0xe8f2ff);
        let bg_assistant = rgb(0xf0f4f8);

        div()
            .flex()
            .flex_col()
            .size_full()
            .child(
                // Header
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_6()
                    .py_4()
                    .border_b_1()
                    .border_color(rgb(0xd0d0d0))
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("codex'd")
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x666666))
                            .child(self.selected_repo.clone().unwrap_or_default())
                    )
            )
            .child(
                // Timeline area (Perplexity-style trajectory view) - scrollable
                div()
                    .id("timeline-container")
                    .flex_1()
                    .overflow_y_scroll()
                    .track_scroll(&self.timeline_scroll_handle)
                    .child(
                        // Inner content div (not flex container!)
                        div()
                            .p_6()
                            .child(render_timeline(&self.timeline_events))
                            // Add streaming views for active buffers
                            .when(!self.current_thought_buffer.is_empty(), |parent| {
                                parent.child(render_streaming_thought(&self.current_thought_buffer))
                            })
                            .when(!self.current_message_buffer.is_empty(), |parent| {
                                parent.child(render_streaming_message(&self.current_message_buffer))
                            })
                            .children(self.active_tool_calls.iter().map(|(_, (tool_call, output))| {
                                render_streaming_tool_call(tool_call, output)
                            }))
                    )
            )
            .child(
                // Input area
                div()
                    .flex()
                    .gap_2()
                    .px_6()
                    .py_4()
                    .border_t_1()
                    .border_color(rgb(0xd0d0d0))
                    .on_key_down({
                        let view = cx.entity().clone();
                        move |event, window, cx| {
                            if event.keystroke.key == "enter" && !event.keystroke.modifiers.shift {
                                view.update(cx, |view, cx| {
                                    view.on_send_message(window, cx);
                                });
                            }
                        }
                    })
                    .child(
                        TextInput::new(&self.message_input_state)
                            .flex_1()
                    )
                    .child({
                        let view = cx.entity().clone();
                        Button::new("send-button")
                            .label("Send")
                            .primary()
                            .on_click(move |_, window, cx| {
                                view.update(cx, |view, cx| {
                                    view.on_send_message(window, cx);
                                });
                            })
                    })
            )
    }
}
