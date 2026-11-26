//! REPL mode — interactive query input with history.

use std::fmt;
use std::sync::Arc;

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::storage::StorageEngine;

/// Maximum history entries kept.
const MAX_HISTORY: usize = 200;
/// Maximum results shown.
const MAX_RESULTS_DISPLAY: usize = 50;

/// REPL state.
pub struct ReplState {
    pub engine: Arc<StorageEngine>,
    pub input_line: String,
    pub cursor_pos: usize,
    pub history: Vec<String>,
    pub history_pos: Option<usize>,
    pub output_lines: Vec<String>,
    pub scroll_offset: usize,
}

impl fmt::Debug for ReplState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReplState")
            .field("input_line", &self.input_line)
            .field("cursor_pos", &self.cursor_pos)
            .field("history_len", &self.history.len())
            .field("output_lines", &self.output_lines.len())
            .finish()
    }
}

impl ReplState {
    pub fn new(engine: Arc<StorageEngine>) -> Self {
        Self {
            engine,
            input_line: String::new(),
            cursor_pos: 0,
            history: Vec::with_capacity(64),
            history_pos: None,
            output_lines: vec!["VantaDB REPL — type a query and press Enter".into()],
            scroll_offset: 0,
        }
    }

    /// Byte offset of the character at `char_idx`.
    ///
    /// The cursor is a **character** index, not a byte offset (FIND-601):
    /// `String::insert`/`remove` panic when the index is not on a character
    /// boundary, and a byte-based cursor lands in the middle of a multibyte
    /// character — the next keystroke panicked the TUI.
    fn byte_index(&self, char_idx: usize) -> usize {
        self.input_line
            .char_indices()
            .nth(char_idx)
            .map(|(i, _)| i)
            .unwrap_or(self.input_line.len())
    }

    /// Number of characters in the input line.
    fn char_len(&self) -> usize {
        self.input_line.chars().count()
    }

    /// Execute the current input line as a query.
    pub fn execute(&mut self) {
        let input = self.input_line.trim().to_string();
        if input.is_empty() {
            return;
        }

        // Add to history
        self.history.push(input.clone());
        if self.history.len() > MAX_HISTORY {
            self.history.remove(0);
        }
        self.history_pos = None;

        self.output_lines.push(format!(">>> {}", input));

        // Handle special commands
        match input.as_str() {
            ".help" | ".h" => {
                self.output_lines.push("Commands:".into());
                self.output_lines.push("  .help / .h   — this help".into());
                self.output_lines
                    .push("  .clear / .cl — clear output".into());
                self.output_lines
                    .push("  .stats / .st — show engine stats".into());
                self.output_lines
                    .push("  .exit / .q   — return to dashboard".into());
                self.output_lines
                    .push("  <any IQL query> — execute query".into());
                self.output_lines
                    .push("  IQL writes — read-only session: use `vanta-cli query`".into());
            }
            ".clear" | ".cl" => {
                self.output_lines.clear();
            }
            ".stats" | ".st" => {
                let stats = self.engine.stats();
                self.output_lines.push(format!(
                    " Nodes: {} | Cache: {} | Evictions: {} | Memory: {} logical",
                    stats.node_count,
                    stats.cache_entries,
                    stats.eviction_count,
                    fmt_bytes(stats.logical_bytes),
                ));
            }
            ".exit" | ".q" => {
                self.output_lines
                    .push("Press Esc to return to dashboard.".into());
            }
            _ => {
                // FIND-117: the TUI engine is long-lived read-only (shared
                // lock), so a mutating statement cannot run on it — and a
                // second read-write handle would hit the file lock. Report the
                // documented limit instead of the raw error. On a writable
                // engine the statement runs and flushes (FIND-101 spirit).
                if self.engine.read_only && crate::cli_handlers::data::query_is_mutating(&input) {
                    self.push_readonly_write_limit();
                } else {
                    self.run_iql(&input);
                }
            }
        }

        // Trim output to prevent runaway memory
        while self.output_lines.len() > 1000 {
            self.output_lines.remove(0);
        }
    }

    /// FIND-117: documented limit for IQL writes on a read-only TUI session.
    fn push_readonly_write_limit(&mut self) {
        self.output_lines.push(
            "⦸ Read-only TUI session (shared lock): IQL writes are disabled here — \
             run them with `vanta-cli query`."
                .into(),
        );
    }

    /// Execute an IQL string that is allowed on this engine (reads always;
    /// writes only when the engine is writable), flushing after writes.
    fn run_iql(&mut self, input: &str) {
        // Try to execute as a query via the executor
        let executor = crate::executor::Executor::new(&self.engine);
        let start = std::time::Instant::now();
        match executor.execute_hybrid(input) {
            Ok(crate::executor::ExecutionResult::Read(nodes)) => {
                let elapsed = start.elapsed();
                let count = nodes.len();
                self.output_lines
                    .push(format!("→ {} result(s) in {:?}", count, elapsed));
                for node in nodes.iter().take(MAX_RESULTS_DISPLAY) {
                    let fields: Vec<String> = node
                        .relational
                        .iter()
                        .map(|(k, v)| format!("{}={:?}", k, v))
                        .collect();
                    self.output_lines.push(format!(
                        "  id={} {}",
                        node.id,
                        if fields.is_empty() {
                            "(no fields)".into()
                        } else {
                            fields.join(", ")
                        }
                    ));
                }
                if count > MAX_RESULTS_DISPLAY {
                    self.output_lines
                        .push(format!("  ... and {} more", count - MAX_RESULTS_DISPLAY));
                }
            }
            Ok(crate::executor::ExecutionResult::Write { message, .. }) => {
                let elapsed = start.elapsed();
                // FIND-117: same-session durability as FIND-101 — the write is
                // WAL-buffered, so flush before reporting success. A Write
                // only happens on a writable engine, so the guard holds.
                match self.engine.flush() {
                    Ok(()) => self
                        .output_lines
                        .push(format!("✓ {} ({:?})", message, elapsed)),
                    Err(e) => self.output_lines.push(format!("✗ Error: {}", e)),
                }
            }
            Ok(crate::executor::ExecutionResult::StaleContext(id)) => {
                self.output_lines
                    .push(format!("⚠ Stale context: id={}", id));
            }
            Err(e) => {
                self.output_lines.push(format!("✗ Error: {}", e));
            }
        }
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

/// Handle a key event in REPL mode. Returns true if the key was handled.
pub fn handle_repl_key(state: &mut ReplState, key: crossterm::event::KeyCode) -> bool {
    match key {
        crossterm::event::KeyCode::Enter => {
            state.execute();
            state.input_line.clear();
            state.cursor_pos = 0;
            state.scroll_offset = 0;
            true
        }
        crossterm::event::KeyCode::Char(c) => {
            let at = state.byte_index(state.cursor_pos);
            state.input_line.insert(at, c);
            state.cursor_pos += 1;
            true
        }
        crossterm::event::KeyCode::Backspace => {
            if state.cursor_pos > 0 {
                // Delete the whole character before the cursor (not one byte).
                let end = state.byte_index(state.cursor_pos);
                let start = state.byte_index(state.cursor_pos - 1);
                state.input_line.replace_range(start..end, "");
                state.cursor_pos -= 1;
            }
            true
        }
        crossterm::event::KeyCode::Delete => {
            if state.cursor_pos < state.char_len() {
                let start = state.byte_index(state.cursor_pos);
                let end = state.byte_index(state.cursor_pos + 1);
                state.input_line.replace_range(start..end, "");
            }
            true
        }
        crossterm::event::KeyCode::Left => {
            state.cursor_pos = state.cursor_pos.saturating_sub(1);
            true
        }
        crossterm::event::KeyCode::Right => {
            if state.cursor_pos < state.char_len() {
                state.cursor_pos += 1;
            }
            true
        }
        crossterm::event::KeyCode::Up => {
            let pos = state.history_pos.get_or_insert(state.history.len());
            if *pos > 0 {
                *pos -= 1;
                state.input_line = state.history[*pos].clone();
                state.cursor_pos = state.char_len();
            }
            true
        }
        crossterm::event::KeyCode::Down => {
            if let Some(pos) = &mut state.history_pos {
                if *pos + 1 < state.history.len() {
                    *pos += 1;
                    state.input_line = state.history[*pos].clone();
                } else {
                    state.history_pos = None;
                    state.input_line.clear();
                }
                state.cursor_pos = state.char_len();
            }
            true
        }
        crossterm::event::KeyCode::PageUp => {
            state.scroll_offset = state.scroll_offset.saturating_add(10);
            true
        }
        crossterm::event::KeyCode::PageDown => {
            state.scroll_offset = state.scroll_offset.saturating_sub(10);
            true
        }
        crossterm::event::KeyCode::Home => {
            state.cursor_pos = 0;
            true
        }
        crossterm::event::KeyCode::End => {
            state.cursor_pos = state.char_len();
            true
        }
        _ => false,
    }
}

/// Render the REPL into the given frame.
pub fn render_repl(frame: &mut Frame, state: &mut ReplState) {
    let area = frame.area();

    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Length(3), // title
            ratatui::layout::Constraint::Min(3),    // output
            ratatui::layout::Constraint::Length(3), // input
        ])
        .split(area);

    // ── Title ──
    let title = Paragraph::new(
        " VantaDB TUI — REPL [1] Dashboard  [2] Monitor  [3] REPL  |  Esc: dashboard ",
    )
    .style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
    .alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);

    // ── Output area ──
    let output_text: Vec<Line> = state
        .output_lines
        .iter()
        .rev()
        .skip(state.scroll_offset)
        .take(if chunks[1].height > 5 {
            (chunks[1].height - 2) as usize
        } else {
            3
        })
        .map(|line| {
            if line.starts_with(">>>") {
                Line::styled(line.clone(), Style::default().fg(Color::Green))
            } else if line.starts_with("→") {
                Line::styled(line.clone(), Style::default().fg(Color::Yellow))
            } else if line.starts_with("✗") {
                Line::styled(line.clone(), Style::default().fg(Color::Red))
            } else if line.starts_with("✓") {
                Line::styled(line.clone(), Style::default().fg(Color::Green))
            } else {
                Line::raw(line.clone())
            }
        })
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    let output = Paragraph::new(output_text)
        .block(
            Block::default()
                .title(" Output ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .scroll((0, 0));
    frame.render_widget(output, chunks[1]);

    // ── Input line ──
    let input_prefix = ">>> ";
    let input_display = format!("{}{}", input_prefix, state.input_line);
    let input = Paragraph::new(input_display)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::White));
    frame.render_widget(input, chunks[2]);

    // Place cursor at correct position
    frame.set_cursor_position(Position::new(
        // x: after ">>> " prefix + the characters before the cursor
        // (the cursor is a character index, not a byte offset — FIND-601).
        (input_prefix.len() + state.cursor_pos) as u16,
        // y: last line of input area
        chunks[2].y + 1,
    ));
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crossterm::event::KeyCode;

    use super::{handle_repl_key, ReplState};
    use crate::backend::BackendKind;
    use crate::config::Config;
    use crate::storage::StorageEngine;

    fn inmemory_engine(read_only: bool) -> Arc<StorageEngine> {
        let config = Config {
            backend_kind: BackendKind::InMemory,
            read_only,
            ..Config::default()
        };
        Arc::new(StorageEngine::open_with_config(":memory:", Some(config)).unwrap())
    }

    fn run_input(engine: &Arc<StorageEngine>, input: &str) -> Vec<String> {
        let mut state = ReplState::new(engine.clone());
        state.input_line = input.to_string();
        state.execute();
        state.output_lines.clone()
    }

    // FIND-117 (RED): a mutating IQL statement on the long-lived read-only TUI
    // engine must report the documented limit (pointer to `vanta-cli query`),
    // not the raw read-only validation error.
    #[test]
    fn repl_mutating_on_readonly_reports_documented_limit() {
        let engine = inmemory_engine(true);
        let out = run_input(
            &engine,
            r#"INSERT NODE#117 TYPE Usuario { nombre: "Eros" }"#,
        );
        let last = out.last().unwrap();
        assert!(
            last.contains("vanta-cli query"),
            "must point to the CLI write path, got: {last}"
        );
        assert!(
            last.to_lowercase().contains("read-only"),
            "must name the read-only session limit, got: {last}"
        );
    }

    // FIND-117 (RED): `.help` documents the read-only writes limit with motive.
    #[test]
    fn repl_help_documents_readonly_limit() {
        let engine = inmemory_engine(true);
        let out = run_input(&engine, ".help");
        assert!(
            out.iter().any(|l| l.contains("vanta-cli query")),
            "help must document where writes go, got: {out:?}"
        );
    }

    // FIND-117 (rama primaria del contrato en handle writable): INSERT→SELECT
    // visible en la misma sesión TUI — el mutante corre y flushea (FIND-101).
    #[test]
    fn repl_insert_select_visible_on_writable_engine() {
        let engine = inmemory_engine(false);
        let out = run_input(
            &engine,
            r#"INSERT NODE#117 TYPE Usuario { nombre: "Eros" }"#,
        );
        assert!(
            out.iter().any(|l| l.starts_with("✓")),
            "INSERT must succeed on a writable engine, got: {out:?}"
        );
        let out = run_input(&engine, "FROM Usuario");
        assert!(
            out.iter().any(|l| l.contains("1 result")),
            "INSERT must be visible to SELECT in the same session, got: {out:?}"
        );
    }

    // FIND-601: a multibyte character must not panic the REPL. The cursor
    // counts bytes, so after a 2-byte character it points into the middle of
    // it and the next `String::insert`/`remove` panics ("byte index N is not
    // a char boundary") — which, with FIND-602, leaves the shell broken.
    #[test]
    fn repl_handles_multibyte_characters_without_panicking() {
        let engine = inmemory_engine(false);
        let mut state = ReplState::new(engine);

        handle_repl_key(&mut state, KeyCode::Char('ñ'));
        handle_repl_key(&mut state, KeyCode::Char('a'));
        assert_eq!(state.input_line, "ña");
        // The cursor is a **character** index, not a byte offset.
        assert_eq!(state.cursor_pos, 2);

        // Backspace removes the whole character, not one byte.
        handle_repl_key(&mut state, KeyCode::Backspace);
        assert_eq!(state.input_line, "ñ");
        assert_eq!(state.cursor_pos, 1);

        // Left/Right move by characters too.
        handle_repl_key(&mut state, KeyCode::Left);
        assert_eq!(state.cursor_pos, 0);
        handle_repl_key(&mut state, KeyCode::Right);
        assert_eq!(state.cursor_pos, 1);

        // Delete removes the character under the cursor.
        handle_repl_key(&mut state, KeyCode::Char('x'));
        handle_repl_key(&mut state, KeyCode::Home);
        handle_repl_key(&mut state, KeyCode::Delete);
        assert_eq!(state.input_line, "x");
    }

    // FIND-117 (caracterización, pasa antes/después): reads y stats siguen
    // funcionando — guardia de no-regresión del contrato.
    #[test]
    fn repl_select_and_stats_keep_working() {
        let engine = inmemory_engine(false);
        crate::executor::Executor::new(&engine)
            .execute_hybrid(r#"INSERT NODE#117 TYPE Usuario { nombre: "Eros" }"#)
            .unwrap();
        let out = run_input(&engine, "FROM Usuario");
        assert!(
            out.iter().any(|l| l.contains("1 result")),
            "SELECT must stay visible, got: {out:?}"
        );
        let out = run_input(&engine, ".stats");
        assert!(
            out.iter().any(|l| l.contains("Nodes:")),
            "stats must keep working, got: {out:?}"
        );
    }

    // FIND-117 (motivo ejecutable E2, pasa antes/después): con el handle
    // read-only vivo, un segundo handle read-write choca en el lock fs2
    // (`DatabaseBusy`) — por eso el fix no es "abrir rw por query".
    #[test]
    fn tui_write_handle_blocked_while_readonly_open() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_string_lossy().to_string();
        drop(crate::cli_handlers::open_database(&path, false).unwrap());
        let _ro = crate::cli_handlers::open_database(&path, true).unwrap();
        let result = crate::cli_handlers::open_database(&path, false);
        assert!(
            result.is_err(),
            "second rw handle must hit the file lock while readonly is open"
        );
        let err = result.err().unwrap();
        assert!(
            matches!(err, crate::error::Error::DatabaseBusy(_)),
            "second rw handle must hit the file lock, got: {err}"
        );
    }
}
