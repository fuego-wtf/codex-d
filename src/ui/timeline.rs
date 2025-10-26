// Timeline renderer - chronological trajectory display (Perplexity-style)
use gpui::*;

use crate::types::TimelineEvent;
use super::components::render_timeline_event;

/// Renders the full chronological timeline of events
/// Events are sorted by timestamp ascending (oldest first)
pub fn render_timeline(events: &[TimelineEvent]) -> Stateful<Div> {
    // Sort events chronologically
    let mut sorted_events = events.to_vec();
    sorted_events.sort_by_key(|e| e.timestamp());

    div()
        .id("timeline")
        .flex()
        .flex_col()
        .gap_4()
        .children(
            sorted_events
                .iter()
                .map(|event| render_timeline_event(event))
        )
}
