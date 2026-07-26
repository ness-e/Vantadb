//! Monitor mode — live query watch with timing and auto-scroll.

use std::sync::Arc;

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::storage::StorageEngine;

/// A single logged query entry.
#[derive(Debug, Clone)]
pub struct QueryEntry {
    pub query: String,
    pub duration_ms: u64,
    pub result_count: usize,
}

/// Monitor state holding recent queries.
#[derive(Debug)]
pub struct MonitorData {
    pub queries: Vec<QueryEntry>,
    pub auto_scroll: bool,
}

impl MonitorData {
    pub fn new() -> Self {
        Self {
            queries: Vec::with_capacity(256),
            auto_scroll: true,
        }
    }

    /// Refresh (poll engine for new activity — placeholder for now).
    pub fn refresh(&mut self, _engine: &Arc<StorageEngine>) {
        // In a full implementation, hook into engine query tracing.
        // For now, just show the UI frame.
    }

    /// Render the monitor into the given frame.
    pub fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Length(3),
                ratatui::layout::Constraint::Min(5),
                ratatui::layout::Constraint::Length(3),
            ])
            .split(area);

        // ── Title ──
        let title = Paragraph::new(
            " VantaDB TUI — Monitor [1] Dashboard  [2] Monitor  [3] REPL  |  q: quit ",
        )
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // ── Query list ──
        let items: Vec<ListItem> = if self.queries.is_empty() {
            vec![ListItem::new(
                " No queries yet. Run queries in REPL (mode 3) or via the API to see them here.",
            )
            .style(Style::default().fg(Color::DarkGray))]
        } else {
            self.queries
                .iter()
                .rev()
                .take(100)
                .map(|q| {
                    let line = format!(
                        " {}ms | {} results | {}",
                        q.duration_ms, q.result_count, q.query
                    );
                    ListItem::new(line)
                })
                .collect()
        };

        let query_list = List::new(items)
            .block(
                Block::default()
                    .title(" Recent Queries ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue)),
            )
            .highlight_style(Style::default().fg(Color::Yellow));

        // Use a scrollable list if there are many entries
        let list_state = ratatui::widgets::ListState::default();
        let mut state = list_state.clone();
        frame.render_stateful_widget(query_list, chunks[1], &mut state);

        // ── Status bar ──
        let status = Paragraph::new(format!(
            " Tracking {} queries | auto-scroll: {} | 1/2/3 switch mode",
            self.queries.len(),
            if self.auto_scroll { "ON" } else { "OFF" },
        ))
        .style(Style::default().fg(Color::Black).bg(Color::Gray));
        frame.render_widget(status, chunks[2]);
    }
}
