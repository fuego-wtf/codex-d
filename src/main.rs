// codex-d: Developer Psychology Analysis
// Chat-based UI: Repo selection → Enrichment → Conversation

mod types;
mod git_analyzer;
mod codex_adapter;
mod storage;

use gpui::*;
use gpui::prelude::*;
use std::sync::Arc;
use tokio::runtime::Handle;

use gpui_component::{
    button::{Button, ButtonVariants},
    input::{TextInput, InputState},
    Root,
};

use types::{AppState, Message};
use storage::Storage;
use git_analyzer::GitAnalyzer;
use codex_adapter::CodexAdapter;

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
    lifecycle_events: Vec<types::LifecycleEvent>,
    message_input_state: Entity<InputState>,
    storage: Option<Arc<Storage>>,
    tokio_handle: Handle,
    codex_adapter: Option<Arc<CodexAdapter>>,
    is_loading: bool,
    error_message: Option<String>,
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
            lifecycle_events: Vec::new(),
            message_input_state,
            storage,
            tokio_handle,
            codex_adapter: None,
            is_loading: false,
            error_message: None,
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
        self.lifecycle_events.push(types::LifecycleEvent::running("Scanning git history".to_string()));
        cx.notify();

        // TODO: Make this async once we figure out the GPUI async pattern
        // For now, doing synchronous enrichment
        eprintln!("Starting git analysis for: {}", repo_path);

        // Note: This blocks the UI thread - not ideal but gets us working
        // We'll optimize with proper async later
        let tokio_handle = self.tokio_handle.clone();
        let analysis_result = tokio_handle.block_on(GitAnalyzer::analyze(&repo_path));

        match analysis_result {
            Ok(analysis) => {
                eprintln!("Git analysis complete: {} commits analyzed", analysis.evidence.len());

                // Build system prompt from analysis
                let system_prompt = format!(
                    "You are a developer psychology analyst. Based on git history analysis:\n\n\
                     Pattern Detected: {}\n\
                     Evidence: {} commits\n\
                     Severity: {}\n\
                     Summary: {}\n\n\
                     Provide empathetic, constructive psychological insights about the developer's patterns.",
                    analysis.pattern_type, analysis.evidence.len(), analysis.severity, analysis.summary
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
                        match adapter.create_session(system_prompt) {
                            Ok(session_id) => {
                                eprintln!("Codex session created: {}", session_id);

                                // Success!
                                self.codex_adapter = Some(adapter);
                                self.lifecycle_events.push(types::LifecycleEvent::completed("Git analysis".to_string()));
                                self.lifecycle_events.push(types::LifecycleEvent::completed("AI initialized".to_string()));
                                self.app_state = AppState::ChatActive;
                                self.is_loading = false;

                                // Add initial system message with analysis
                                let initial_msg = Message::assistant(format!(
                                    "I've analyzed {} commits from your repository.\n\n{}\n\nWhat would you like to explore about your development patterns?",
                                    analysis.evidence.len(), analysis.summary
                                ));
                                self.messages.push(initial_msg);

                                cx.notify();
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

        let message = Message::user(content.clone());
        self.messages.push(message.clone());

        if let Some(storage) = &self.storage {
            let _ = storage.save_message(&message);
        }

        cx.notify();

        // Send to Codex and stream response
        if let Some(adapter) = &self.codex_adapter {
            let adapter = adapter.clone();

            // Collect response chunks
            // Note: This blocks the UI thread - not ideal but gets us working
            // We'll optimize with proper async later
            let mut response_text = String::new();

            let result = adapter.send_message(content, |event| {
                match event {
                    types::StreamEvent::MessageChunk(chunk) => {
                        response_text.push_str(&chunk);
                        eprintln!("Received chunk: {}", chunk);
                    }
                    types::StreamEvent::LifecycleEvent(event) => {
                        eprintln!("Lifecycle event: {:?}", event);
                    }
                    types::StreamEvent::PermissionRequest(request) => {
                        eprintln!("Permission request: {:?}", request);
                    }
                }
            });

            match result {
                Ok(_) => {
                    // Add assistant's response
                    let assistant_message = Message::assistant(response_text);
                    self.messages.push(assistant_message.clone());

                    if let Some(storage) = &self.storage {
                        let _ = storage.save_message(&assistant_message);
                    }

                    cx.notify();
                }
                Err(e) => {
                    eprintln!("Failed to send message: {}", e);
                    let error_message = Message::assistant(format!("Error: {}", e));
                    self.messages.push(error_message);
                    cx.notify();
                }
            }
        } else {
            eprintln!("No Codex adapter available");
            let error_message = Message::assistant("Error: Codex not initialized".to_string());
            self.messages.push(error_message);
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
                // Messages area
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .p_6()
                    .gap_4()
                    .children(self.messages.iter().map(|msg| {
                        let is_user = msg.is_user();
                        let bg = if is_user { bg_user } else { bg_assistant };

                        div()
                            .flex()
                            .w_full()
                            .when(is_user, |div| div.justify_end())
                            .child(
                                div()
                                    .max_w(px(600.0))
                                    .px_4()
                                    .py_3()
                                    .bg(bg)
                                    .rounded_lg()
                                    .child(msg.content().to_string())
                            )
                    }))
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
