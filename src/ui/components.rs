// Reusable UI components for codex-d
use gpui::*;
use gpui::prelude::*;
use gpui_component::v_flex;

use crate::types::{TimelineEvent, ToolCallStatus, ToolCallEvent, McpServerType};

// ============================================================================
// Timeline Item Renderer
// ============================================================================

pub fn render_timeline_event(event: &TimelineEvent) -> Div {
    match event {
        TimelineEvent::UserMessage { content, .. } => render_user_message(content),
        TimelineEvent::Thought { content, .. } => render_thought(content),
        TimelineEvent::ToolCall { title, kind, status, locations, output, mcp_server, routed_via, .. } => {
            render_tool_call(title, kind, status, locations, output.as_deref(), mcp_server.as_ref(), routed_via.as_ref())
        }
        TimelineEvent::AssistantMessage { content, .. } => render_assistant_message(content),
        TimelineEvent::McpServerConnected { server_type, host, port, .. } => {
            render_mcp_server_connected(server_type, host, *port)
        }
        TimelineEvent::McpServerDisconnected { server_type, reason, .. } => {
            render_mcp_server_disconnected(server_type, reason.as_deref())
        }
        TimelineEvent::AgentFixPrompt { prompt, .. } => {
            render_agent_fix_prompt(prompt)
        }
        TimelineEvent::SecurityFinding {
            vulnerability_id, severity, title, description,
            file_path, line_number, cwe_id, recommendation, ..
        } => {
            render_security_finding(
                vulnerability_id,
                severity,
                title,
                description,
                file_path,
                *line_number,
                cwe_id.as_deref(),
                recommendation
            )
        }
    }
}

// ============================================================================
// Message Components
// ============================================================================

pub fn render_user_message(content: &str) -> Div {
    div()
        .flex()
        .w_full()
        .px_2()
        .py_2()
        .justify_end()
        .child(
            v_flex()
                .max_w(rems(40.0))  // Changed from px(600) to rems for better responsiveness
                .px_3()
                .py_2()
                .bg(rgb(0xe8f2ff))
                .border_1()
                .border_color(rgb(0x90caf9))
                .rounded_md()
                .gap_1()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0x1976d2))
                        .child("👤 You")
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0x212121))
                        .line_height(relative(1.5))
                        .overflow_x_hidden()  // Prevent horizontal overflow
                        .child(content.to_string())
                )
        )
}

pub fn render_assistant_message(content: &str) -> Div {
    div()
        .flex()
        .w_full()
        .px_2()
        .py_2()
        .justify_start()
        .child(
            v_flex()
                .max_w(rems(40.0))  // Changed from px(600) to rems for better responsiveness
                .px_3()
                .py_2()
                .bg(rgb(0xf0f4f8))
                .border_1()
                .border_color(rgb(0xcfd8dc))
                .rounded_md()
                .gap_1()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0x546e7a))
                        .child("🤖 Assistant")
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0x212121))
                        .line_height(relative(1.5))
                        .overflow_x_hidden()  // Prevent horizontal overflow
                        .child(content.to_string())
                )
        )
}

// ============================================================================
// Thought Component (Perplexity-style thinking indicator)
// ============================================================================

pub fn render_thought(content: &str) -> Div {
    div()
        .flex()
        .w_full()
        .px_2()
        .py_2()
        .justify_start()
        .child(
            v_flex()
                .max_w(rems(40.0))  // Changed from px(600) to rems for better responsiveness
                .px_3()
                .py_2()
                .bg(rgb(0xfff8e1))
                .border_1()
                .border_color(rgb(0xffd54f))
                .rounded_md()
                .gap_1()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0xf57c00))
                        .child("💭 Thinking...")
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0x5d4037))
                        .line_height(relative(1.5))
                        .overflow_x_hidden()  // Prevent horizontal overflow
                        .child(content.to_string())
                )
        )
}

// ============================================================================
// Tool Call Component (Step-by-step trajectory item)
// ============================================================================

fn get_status_text(status: &ToolCallStatus) -> (&'static str, Rgba) {
    match status {
        ToolCallStatus::InProgress => ("🔄 Running", rgb(0x1976d2)),
        ToolCallStatus::Completed => ("✅ Completed", rgb(0x388e3c)),
        ToolCallStatus::Failed => ("❌ Failed", rgb(0xd32f2f)),
    }
}

fn render_tool_call(
    title: &str,
    _kind: &str,
    status: &ToolCallStatus,
    locations: &[crate::types::ToolCallLocation],
    output: Option<&str>,
    mcp_server: Option<&McpServerType>,
    routed_via: Option<&McpServerType>,
) -> Div {
    let bg_tool_call = rgb(0xe8f5e9);
    let (status_text, status_color) = get_status_text(status);

    div()
        .flex()
        .w_full()
        .px_4()
        .justify_start()
        .child(
            div()
                .max_w(px(600.0))
                .px_3()
                .py_1p5()
                .bg(bg_tool_call)
                .border_1()
                .border_color(rgb(0x81c784))
                .rounded_md()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    // Header with title and status
                    div()
                        .flex()
                        .justify_between()
                        .items_center()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0x2e7d32))
                                .child(format!("🔧 {}", title))
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(status_color)
                                .child(status_text)
                        )
                )
                // MCP Server source (for transparency)
                .when_some(mcp_server, |container, server| {
                    container.child(
                        div()
                            .flex()
                            .gap_2()
                            .items_center()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x616161))
                                    .child(format!("{} via {}",
                                        server.icon(),
                                        server.display_name()
                                    ))
                            )
                            .when_some(routed_via, |div_inner, gateway| {
                                div_inner.child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0x757575))
                                        .child(format!("→ {} {}",
                                            gateway.icon(),
                                            gateway.display_name()
                                        ))
                                )
                            })
                    )
                })
                // File locations if present
                .when(!locations.is_empty(), |container| {
                    container.child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x616161))
                            .child(
                                locations
                                    .iter()
                                    .map(|loc| loc.path.clone())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            )
                    )
                })
                // Tool output if present
                .when_some(output, |container, out| {
                    container.child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x424242))
                            .font_family("monospace")
                            .line_height(relative(1.5))
                            .child(out.to_string())
                    )
                })
        )
}

// ============================================================================
// STREAMING Components (for real-time display)
// ============================================================================

/// Renders a streaming thought (while it's being received)
pub fn render_streaming_thought(content: &str) -> Div {
    let bg_thought = rgb(0xfff8e1);

    div()
        .flex()
        .w_full()
        .px_2()
        .py_2()
        .justify_start()
        .child(
            div()
                .max_w(rems(40.0))  // Changed from px(600) to rems for better responsiveness
                .px_3()
                .py_2()
                .bg(bg_thought)
                .border_1()
                .border_color(rgb(0xffd54f))
                .rounded_lg()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0xf57c00))
                        .child("💭 Thinking...")
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0x5d4037))
                        .line_height(relative(1.5))
                        .overflow_x_hidden()  // Prevent horizontal overflow
                        .child(format!("{}▊", content)) // Cursor animation
                )
        )
}

/// Renders a streaming assistant message (while it's being received)
pub fn render_streaming_message(content: &str) -> Div {
    let bg_assistant = rgb(0xf0f4f8);
    let border_assistant = rgb(0xcfd8dc);

    div()
        .flex()
        .w_full()
        .px_2()
        .py_2()
        .justify_start()
        .child(
            div()
                .max_w(rems(40.0))  // Changed from px(600) to rems for better responsiveness
                .px_3()
                .py_2()
                .bg(bg_assistant)
                .border_1()
                .border_color(border_assistant)
                .rounded_lg()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0x546e7a))
                        .child("🤖 Assistant")
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0x212121))
                        .line_height(relative(1.5))
                        .overflow_x_hidden()  // Prevent horizontal overflow
                        .child(format!("{}▊", content)) // Cursor animation
                )
        )
}

/// Renders an active tool call (while it's running)
pub fn render_streaming_tool_call(tool_call: &ToolCallEvent, output: &str) -> Div {
    let bg_tool_call = rgb(0xe8f5e9);
    let (status_text, status_color) = get_status_text(&tool_call.status);

    div()
        .flex()
        .w_full()
        .px_2()
        .py_2()
        .justify_start()
        .child(
            div()
                .max_w(rems(40.0))  // Changed from px(600) to rems for better responsiveness
                .px_3()
                .py_2()
                .bg(bg_tool_call)
                .border_1()
                .border_color(rgb(0x81c784))
                .rounded_lg()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .items_center()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0x2e7d32))
                                .child(format!("🔧 {}", tool_call.title))
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(status_color)
                                .child(status_text)
                        )
                )
                .when(!tool_call.locations.is_empty(), |container| {
                    container.child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x616161))
                            .child(
                                tool_call.locations
                                    .iter()
                                    .map(|loc| loc.path.clone())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            )
                    )
                })
                .when(!output.is_empty(), |container| {
                    container.child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x424242))
                            .font_family("monospace")
                            .line_height(relative(1.5))
                            .overflow_x_hidden()  // Prevent horizontal overflow
                            .child(format!("{}▊", output)) // Cursor animation
                    )
                })
        )
}

// ============================================================================
// MCP Server Connection Components (for transparency)
// ============================================================================

pub fn render_mcp_server_connected(server_type: &McpServerType, host: &str, port: u16) -> Div {
    let bg_mcp = rgb(0xe3f2fd);
    let border_mcp = rgb(0x90caf9);

    div()
        .flex()
        .w_full()
        .px_4()
        .justify_start()
        .child(
            div()
                .max_w(px(600.0))
                .px_3()
                .py_1p5()
                .bg(bg_mcp)
                .border_1()
                .border_color(border_mcp)
                .rounded_md()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0x1976d2))
                        .child("✅ Connected")
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x424242))
                        .child(format!("{} {} at {}:{}",
                            server_type.icon(),
                            server_type.display_name(),
                            host,
                            port
                        ))
                )
        )
}

pub fn render_mcp_server_disconnected(server_type: &McpServerType, reason: Option<&str>) -> Div {
    let bg_mcp = rgb(0xfce4ec);
    let border_mcp = rgb(0xf48fb1);

    div()
        .flex()
        .w_full()
        .px_2()
        .py_2()
        .justify_start()
        .child(
            div()
                .max_w(rems(40.0))  // Changed from px(600) to rems for better responsiveness
                .px_3()
                .py_2()
                .bg(bg_mcp)
                .border_1()
                .border_color(border_mcp)
                .rounded_lg()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xc2185b))
                                .child("⚠️ Disconnected")
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0x424242))
                                .child(format!("{} {}",
                                    server_type.icon(),
                                    server_type.display_name()
                                ))
                        )
                )
                .when_some(reason, |container, r| {
                    container.child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x616161))
                            .child(format!("Reason: {}", r))
                    )
                })
        )
}

// ============================================================================
// Agent Fix Prompt Component (with copy functionality)
// ============================================================================

pub fn render_agent_fix_prompt(prompt: &str) -> Div {
    let bg_prompt = rgb(0xfff9c4);
    let border_prompt = rgb(0xfff176);

    div()
        .flex()
        .w_full()
        .px_2()
        .py_2()
        .justify_start()
        .child(
            div()
                .max_w(rems(40.0))  // Changed from px(600) to rems for better responsiveness
                .px_3()
                .py_2()
                .bg(bg_prompt)
                .border_1()
                .border_color(border_prompt)
                .rounded_lg()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .items_center()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xf57f17))
                                .child("🤖 Agent Fix Prompt")
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0x616161))
                                .child("📋 Hover to copy")
                        )
                )
                .child(
                    div()
                        .px_3()
                        .py_2()
                        .bg(rgb(0xfffde7))
                        .border_1()
                        .border_color(rgb(0xfbc02d))
                        .rounded(px(4.0))
                        .text_xs()
                        .text_color(rgb(0x424242))
                        .font_family("monospace")
                        .line_height(relative(1.5))
                        .overflow_x_hidden()  // Prevent horizontal overflow
                        .child(prompt.to_string())
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x757575))
                        .child("💡 Copy this prompt and feed it to a sub-agent for auto-fixing")
                )
        )
}

// ============================================================================
// Security Finding Component (Aikido scan results)
// ============================================================================

pub fn render_security_finding(
    vulnerability_id: &str,
    severity: &str,
    title: &str,
    description: &str,
    file_path: &str,
    line_number: Option<u32>,
    cwe_id: Option<&str>,
    recommendation: &str,
) -> Div {
    // Severity-based styling
    let (bg_color, border_color, icon) = match severity.to_lowercase().as_str() {
        "critical" => (rgb(0xffebee), rgb(0xef5350), "🚨"),
        "high" => (rgb(0xfff3e0), rgb(0xfb8c00), "⚠️"),
        "medium" => (rgb(0xfff9c4), rgb(0xfdd835), "⚡"),
        "low" => (rgb(0xe8f5e9), rgb(0x66bb6a), "ℹ️"),
        _ => (rgb(0xf5f5f5), rgb(0x9e9e9e), "📋"),
    };

    div()
        .flex()
        .w_full()
        .px_2()
        .py_2()
        .justify_start()
        .child(
            div()
                .max_w(rems(40.0))
                .px_3()
                .py_2()
                .bg(bg_color)
                .border_1()
                .border_color(border_color)
                .rounded_md()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    // Header: Icon + Severity + Title
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_lg()
                                .child(icon)
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_0p5()
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(border_color)
                                        .child(severity.to_uppercase())
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(rgb(0x212121))
                                        .child(title.to_string())
                                )
                        )
                )
                .child(
                    // File location
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0x616161))
                                .child("📄")
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0x616161))
                                .font_family("monospace")
                                .child(
                                    if let Some(line) = line_number {
                                        format!("{}:{}", file_path, line)
                                    } else {
                                        file_path.to_string()
                                    }
                                )
                        )
                )
                .when_some(cwe_id, |container, cwe| {
                    container.child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x757575))
                            .child(format!("CWE: {}", cwe))
                    )
                })
                .child(
                    // Description
                    div()
                        .text_sm()
                        .text_color(rgb(0x424242))
                        .line_height(relative(1.5))
                        .child(description.to_string())
                )
                .child(
                    // Recommendation section
                    div()
                        .mt_2()
                        .pt_2()
                        .border_t_1()
                        .border_color(border_color)
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0x616161))
                                .child("💡 Recommendation")
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0x424242))
                                .line_height(relative(1.5))
                                .child(recommendation.to_string())
                        )
                )
                .when(!vulnerability_id.is_empty(), |container| {
                    container.child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x9e9e9e))
                            .child(format!("ID: {}", vulnerability_id))
                    )
                })
        )
}
