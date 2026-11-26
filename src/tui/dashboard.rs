//! Dashboard mode — engine stats, memory usage, WAL state, and system info.

use std::sync::Arc;
use std::time::{Duration, Instant};

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::metrics::operational_metrics_snapshot;
use crate::storage::StorageEngine;

/// FIND-603: minimum interval between namespace scans. The old
/// `namespace_count == 0` sentinel re-ran a full `scan_nodes()` on EVERY
/// tick (~4×/s) on an empty DB — the state where the answer changes least
/// often — and on any scan error (`if let Ok` swallowed it and retried).
const NAMESPACE_SCAN_INTERVAL: Duration = Duration::from_secs(5);

/// Snapshot of dashboard data refreshed periodically.
#[derive(Debug, Default, Clone)]
pub struct DashboardData {
    pub node_count: u64,
    pub vector_count: u64,
    pub memory_logical: u64,
    /// Process resident set size, OS-reported (whole process — not just the
    /// engine's mmap-resident footprint). `None` until sampled.
    pub process_rss: Option<u64>,
    pub memory_limit: u64,
    pub cache_entries: usize,
    pub eviction_count: u64,
    pub quantized_nodes: u64,
    pub namespace_count: usize,
    pub backend: String,
    /// FIND-603: `(last scan instant, cached count)` — `None` until the first
    /// scan. Keeps `namespace_count == 0` from acting as a "rescan" sentinel.
    namespace_scan: Option<(Instant, usize)>,
}

impl DashboardData {
    /// Refresh from engine.
    pub fn refresh(&mut self, engine: &Arc<StorageEngine>) {
        self.refresh_at(engine, Instant::now());
    }

    /// `Instant`-injectable core of [`Self::refresh`] so the idle-tick cache
    /// behaviour is unit-testable without sleeping (FIND-603).
    fn refresh_at(&mut self, engine: &Arc<StorageEngine>, now: Instant) {
        let mem = engine.stats();
        let metrics = operational_metrics_snapshot();

        self.node_count = mem.node_count;
        self.memory_logical = mem.logical_bytes;
        // TU-05 / FIND-603: show the OS-reported RSS of the whole process
        // (`_get_rss_virt`, the same source `check_pressure` trusts), not the
        // mmap-resident subset — the label must match what is displayed.
        // `mem.physical_rss` (mmap-resident bytes) stays only as a lower-bound
        // fallback on platforms where the RSS syscall is unavailable.
        let (rss, _virt) = crate::metrics::core::_get_rss_virt();
        self.process_rss = (rss > 0).then_some(rss).or(mem.physical_rss);
        self.memory_limit = mem.memory_limit;
        self.cache_entries = mem.cache_entries;
        self.eviction_count = mem.eviction_count;
        self.quantized_nodes = mem.quantized_nodes;
        self.vector_count = metrics.current_quantized_nodes.max(self.node_count);
        self.backend = format!("{:?}", engine.backend_kind());

        self.refresh_namespace_count(engine, now);
    }

    /// Namespace count with a TTL cache: at most one scan per
    /// [`NAMESPACE_SCAN_INTERVAL`], so idle ticks on an empty DB cost O(1).
    fn refresh_namespace_count(&mut self, engine: &Arc<StorageEngine>, now: Instant) {
        if let Some((scanned_at, count)) = self.namespace_scan {
            if now.duration_since(scanned_at) < NAMESPACE_SCAN_INTERVAL {
                self.namespace_count = count;
                return;
            }
        }
        let count = count_namespaces(engine);
        self.namespace_scan = Some((now, count));
        self.namespace_count = count;
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
            let effective = self.process_rss.unwrap_or(self.memory_logical);
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
                Span::raw("Process RSS:"),
                Span::styled(
                    fmt_bytes(self.process_rss.unwrap_or(0)),
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
            fmt_bytes(self.process_rss.unwrap_or(self.memory_logical)),
        ))
        .style(Style::default().fg(Color::Black).bg(Color::Gray));
        frame.render_widget(status, chunks[2]);
    }
}

/// Count distinct namespaces (FIND-603): the derived `NamespaceIndex`
/// partition first — O(index entries), no node materialization — with a
/// `scan_nodes()` fallback only when the index is empty (legacy / no-index
/// databases). Mirrors `Embedded::list_namespaces`.
fn count_namespaces(engine: &StorageEngine) -> usize {
    let mut namespaces = std::collections::BTreeSet::new();
    match engine.scan_partition(crate::backend::BackendPartition::NamespaceIndex) {
        Ok(entries) if !entries.is_empty() => {
            for (key, _value) in entries {
                if let Some(separator) = key.iter().position(|byte| *byte == 0) {
                    if let Ok(namespace) = String::from_utf8(key[..separator].to_vec()) {
                        namespaces.insert(namespace);
                    }
                }
            }
        }
        _ => {
            if let Ok(nodes) = engine.scan_nodes() {
                for n in &nodes {
                    if let Some(crate::node::FieldValue::String(ns)) =
                        n.relational.get(crate::sdk::FIELD_NAMESPACE)
                    {
                        namespaces.insert(ns.clone());
                    }
                }
            }
        }
    }
    namespaces.len()
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::backend::BackendKind;
    use crate::config::Config;
    use crate::node::{FieldValue, UnifiedNode, VectorRepresentations};

    fn in_memory_engine() -> Arc<StorageEngine> {
        let config = Config {
            backend_kind: BackendKind::InMemory,
            read_only: false,
            ..Config::default()
        };
        Arc::new(
            StorageEngine::open_with_config(":memory:", Some(config))
                .expect("open in-memory engine"),
        )
    }

    /// FIND-603: on an empty DB the dashboard must NOT re-run the namespace
    /// scan on every tick (~4×/s). Proven by staleness: a namespaced node
    /// inserted after the first scan stays invisible to `namespace_count`
    /// until [`NAMESPACE_SCAN_INTERVAL`] elapses — a per-tick rescan (the old
    /// `namespace_count == 0` sentinel behaviour) would observe it instantly.
    #[test]
    fn idle_ticks_on_empty_db_do_not_rescan_namespaces() {
        let engine = in_memory_engine();
        let mut data = DashboardData::default();
        let t0 = Instant::now();

        // First refresh scans once: empty DB → 0 namespaces.
        data.refresh_at(&engine, t0);
        assert_eq!(data.namespace_count, 0);
        assert!(data.namespace_scan.is_some());

        let mut node = UnifiedNode::new(1);
        node.vector = VectorRepresentations::Full(vec![0.1, 0.2, 0.3]);
        node.set_field(
            crate::sdk::FIELD_NAMESPACE,
            FieldValue::String("agents".into()),
        );
        engine.insert(&node).expect("insert namespaced node");

        // Simulated idle ticks inside the TTL: the count stays stale at 0 —
        // any rescan would have observed the new namespace immediately.
        for secs in [1_u64, 2, 4] {
            data.refresh_at(&engine, t0 + Duration::from_secs(secs));
            assert_eq!(
                data.namespace_count, 0,
                "tick at +{secs}s must reuse the cached namespace count"
            );
        }

        // Past the TTL the next tick re-scans and observes the namespace.
        data.refresh_at(
            &engine,
            t0 + NAMESPACE_SCAN_INTERVAL + Duration::from_secs(1),
        );
        assert_eq!(data.namespace_count, 1, "post-TTL tick must re-scan");
    }
}
