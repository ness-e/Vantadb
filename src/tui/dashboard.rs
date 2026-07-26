//! Dashboard mode — engine stats, memory usage, WAL state, and system info.

use std::sync::Arc;

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::metrics::operational_metrics_snapshot;
use crate::storage::StorageEngine;

/// Snapshot of dashboard data refreshed periodically.
#[derive(Debug, Default, Clone)]
pub struct DashboardData {
    pub node_count: u64,
    pub vector_count: u64,
    pub memory_logical: u64,
    pub memory_physical: Option<u64>,
    pub memory_limit: u64,
    pub cache_entries: usize,
    pub eviction_count: u64,
    pub quantized_nodes: u64,
    pub namespace_count: usize,
    pub backend: String,
}

impl DashboardData {
    /// Refresh from engine.
    pub fn refresh(&mut self, engine: &Arc<StorageEngine>) {
        let mem = engine.get_memory_stats();
        let metrics = operational_metrics_snapshot();

        self.node_count = mem.node_count;
        self.memory_logical = mem.logical_bytes;
        self.memory_physical = mem.physical_rss;
        self.memory_limit = mem.memory_limit;
        self.cache_entries = mem.cache_entries;
        self.eviction_count = mem.eviction_count;
        self.quantized_nodes = mem.quantized_nodes;
        self.vector_count = metrics.current_quantized_nodes.max(self.node_count);
        self.backend = format!("{:?}", engine.backend_kind());

        // Namespace count from scanning nodes (first call only, cached after)
        if self.namespace_count == 0 {
            if let Ok(nodes) = engine.scan_nodes() {
                let mut nss = std::collections::HashSet::new();
                for n in &nodes {
                    if let Some(crate::node::FieldValue::String(ns)) =
                        n.relational.get(crate::sdk::FIELD_NAMESPACE)
                    {
                        nss.insert(ns.clone());
                    }
                }
                self.namespace_count = nss.len();
            }
        }
    }

    /// Render the dashboard into the given frame.
    pub fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Length(3),
                ratatui::layout::Constraint::Min(10),
                ratatui::layout::Constraint::Length(3),
            ])
            .split(area);

        // ── Title ──
        let title = Paragraph::new(
            " VantaDB TUI — [1] Dashboard  [2] Monitor  [3] REPL  |  q / Esc: quit ",
        )
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // ── Stats grid ──
        let grid = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                ratatui::layout::Constraint::Percentage(50),
                ratatui::layout::Constraint::Percentage(50),
            ])
            .split(chunks[1]);

        let left = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Length(8),
                ratatui::layout::Constraint::Length(8),
            ])
            .split(grid[0]);

        let right = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Length(8),
                ratatui::layout::Constraint::Length(8),
            ])
            .split(grid[1]);

        let memory_pct = if self.memory_limit > 0 {
            let effective = self.memory_physical.unwrap_or(self.memory_logical);
            effective as f64 / self.memory_limit as f64 * 100.0
        } else {
            0.0
        };

        // Engine stats block
        let engine_block = Block::default()
            .title(" Engine ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));
        let engine_text = Text::from(vec![
            Line::from(vec![
                Span::raw("Nodes:     "),
                Span::styled(
                    format!("{}", self.node_count),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::raw("Vectors:   "),
                Span::styled(
                    format!("{}", self.vector_count),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::raw("Cache:     "),
                Span::styled(
                    format!("{} entries", self.cache_entries),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::raw("Evictions: "),
                Span::styled(
                    format!("{}", self.eviction_count),
                    Style::default().fg(Color::Red),
                ),
            ]),
            Line::from(vec![
                Span::raw("Namespaces:"),
                Span::styled(
                    format!(" {}", self.namespace_count),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::raw("Backend:   "),
                Span::styled(&self.backend, Style::default().fg(Color::Cyan)),
            ]),
        ]);
        frame.render_widget(
            Paragraph::new(engine_text)
                .block(engine_block)
                .alignment(Alignment::Left),
            left[0],
        );

        // Memory block
        let mem_block = Block::default()
            .title(" Memory ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));
        let mem_text = Text::from(vec![
            Line::from(vec![
                Span::raw("Logical:  "),
                Span::styled(
                    fmt_bytes(self.memory_logical),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::raw("Physical: "),
                Span::styled(
                    fmt_bytes(self.memory_physical.unwrap_or(0)),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::raw("Limit:    "),
                Span::styled(
                    fmt_bytes(self.memory_limit),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::raw("Usage:    "),
                Span::styled(
                    format!("{:.1}%", memory_pct),
                    if memory_pct > 80.0 {
                        Style::default().fg(Color::Red)
                    } else if memory_pct > 50.0 {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::Green)
                    },
                ),
            ]),
            Line::from(vec![
                Span::raw("Quantized:"),
                Span::styled(
                    format!(" {} nodes", self.quantized_nodes),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
        ]);
        frame.render_widget(
            Paragraph::new(mem_text)
                .block(mem_block)
                .alignment(Alignment::Left),
            right[0],
        );

        // System info block
        let sys_block = Block::default()
            .title(" System ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));
        let sys_text = Text::from(vec![
            Line::from("VantaDB Embedded Persistent Memory Engine"),
            Line::from("ratatui TUI | 1/2/3 switch mode | q/Esc quit"),
        ]);
        frame.render_widget(
            Paragraph::new(sys_text)
                .block(sys_block)
                .alignment(Alignment::Left),
            left[1],
        );

        // WAL info block
        let wal_block = Block::default()
            .title(" WAL ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));
        let wal_text = Text::from(vec![
            Line::from(format!("Backend: {}", self.backend)),
            Line::from("WAL: active"),
        ]);
        frame.render_widget(
            Paragraph::new(wal_text)
                .block(wal_block)
                .alignment(Alignment::Left),
            right[1],
        );

        // ── Status bar ──
        let status = Paragraph::new(format!(
            " [1] Dashboard  [2] Monitor  [3] REPL  [q] Quit  |  Nodes: {}  Memory: {}",
            self.node_count,
            fmt_bytes(self.memory_physical.unwrap_or(self.memory_logical)),
        ))
        .style(Style::default().fg(Color::Black).bg(Color::Gray));
        frame.render_widget(status, chunks[2]);
    }
}

fn fmt_bytes(b: u64) -> String {
    if b >= 1_000_000_000 {
        format!("{:.1} GB", b as f64 / 1_000_000_000.0)
    } else if b >= 1_000_000 {
        format!("{:.1} MB", b as f64 / 1_000_000.0)
    } else if b >= 1_000 {
        format!("{:.1} KB", b as f64 / 1_000.0)
    } else {
        format!("{b} B")
    }
}
