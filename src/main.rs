// codex-d: Developer Psychology Analysis
// Ultra-lean MVP: Git → Codex → Observation

mod types;
mod git_analyzer;
mod codex_adapter;
mod storage;

use gpui::*;
use gpui::prelude::*;

fn main() {
    env_logger::init();

    Application::new().run(|cx: &mut App| {
        // Create window bounds
        let bounds = Bounds::centered(
            None,
            size(px(800.0), px(600.0)),
            cx,
        );

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_window, cx| {
                cx.new(|cx| CodexView::new(cx))
            },
        ).unwrap();
    });
}

struct CodexView {
    app_state: types::AppState,
    selected_repo: Option<String>,
    messages: Vec<types::Message>,
    lifecycle_events: Vec<types::LifecycleEvent>,
}

impl CodexView {
    fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            app_state: types::AppState::AwaitingRepoSelection,
            selected_repo: None,
            messages: Vec::new(),
            lifecycle_events: Vec::new(),
        }
    }
}

impl Render for CodexView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0xffffff))
            .p_4()
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::BOLD)
                    .child("codex-d")
            )
            .child(
                div()
                    .mt_4()
                    .text_sm()
                    .child("Developer Psychology Analysis")
            )
            .child(
                div()
                    .mt_8()
                    .text_sm()
                    .child("Select a git repository to analyze")
            )
    }
}
