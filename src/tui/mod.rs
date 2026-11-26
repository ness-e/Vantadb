//! VantaDB TUI — ratatui + crossterm interactive terminal.
//!
//! Three modes: Dashboard (engine stats), Monitor (live query watch), REPL (interactive queries).
//! Switch with 1/2/3, quit with q/Ctrl+C.

mod dashboard;
mod monitor;
mod repl;

use std::sync::Arc;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal;

use crate::error::Result;
use crate::storage::StorageEngine;

use dashboard::DashboardData;
use monitor::MonitorData;
use repl::ReplState;

/// TUI operational mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiMode {
    Dashboard,
    Monitor,
    Repl,
}

/// Run the TUI loop. Blocks until the user quits (q / Ctrl+C).
pub fn run_tui(engine: Arc<StorageEngine>) -> Result<()> {
    terminal::enable_raw_mode()?;
    let mut terminal = ratatui::init();
    with_terminal_restore(
        || {
            ratatui::restore();
            let _ = crossterm::terminal::disable_raw_mode();
        },
        || tui_event_loop(&mut terminal, &engine),
    )
}

/// Restores the terminal when dropped — on the happy path, on an error and
/// on a panic.
struct TerminalGuard<R: Fn()> {
    restore: R,
}

impl<R: Fn()> Drop for TerminalGuard<R> {
    fn drop(&mut self) {
        (self.restore)();
    }
}

/// Runs `body` with the terminal guard installed, so the terminal is
/// restored on **every** exit path.
///
/// FIND-602: the restore used to live only on the happy path. `ratatui::init`
/// installs a panic hook that restores the terminal, but it does not cover
/// the `?` return: any I/O error inside the event loop (a resize, a closed
/// pty) left the user's shell in raw mode — no echo, no newlines — with no
/// way back.
fn with_terminal_restore<T, E>(
    restore: impl Fn(),
    body: impl FnOnce() -> std::result::Result<T, E>,
) -> std::result::Result<T, E> {
    let _guard = TerminalGuard { restore };
    body()
}

/// The TUI event loop: draws, reads events and switches modes until the
/// user quits.
fn tui_event_loop(term: &mut ratatui::DefaultTerminal, engine: &Arc<StorageEngine>) -> Result<()> {
    let mut mode = TuiMode::Dashboard;

    let mut dash_data = DashboardData::default();
    let mut mon_data = MonitorData::new();
    let mut repl_state = ReplState::new(engine.clone());
    let tick_rate = Duration::from_millis(250);

    // Initial data load
    dash_data.refresh(engine);

    loop {
        term.draw(|frame| match mode {
            TuiMode::Dashboard => dash_data.render(frame),
            TuiMode::Monitor => mon_data.render(frame),
            TuiMode::Repl => repl::render_repl(frame, &mut repl_state),
        })?;

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                // Global quit
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    break;
                }
                match mode {
                    TuiMode::Dashboard | TuiMode::Monitor => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('1') => mode = TuiMode::Dashboard,
                        KeyCode::Char('2') => mode = TuiMode::Monitor,
                        KeyCode::Char('3') => mode = TuiMode::Repl,
                        _ => {}
                    },
                    TuiMode::Repl => {
                        repl::handle_repl_key(&mut repl_state, key.code);
                        if key.code == KeyCode::Esc {
                            mode = TuiMode::Dashboard;
                        }
                    }
                }
            }
        }

        // Refresh data each tick
        match mode {
            TuiMode::Dashboard => dash_data.refresh(engine),
            TuiMode::Monitor => mon_data.refresh(engine),
            TuiMode::Repl => {}
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    /// FIND-602: the terminal must be restored on **every** exit path of the
    /// TUI loop. Before the fix the restore only ran on the happy path, so
    /// an I/O error inside the loop (a resize, a closed pty) returned
    /// without restoring: the user's shell stayed in raw mode (no echo, no
    /// newlines) with no way back.
    ///
    /// The restore action is injected so the contract can be checked
    /// headless (no console in CI ⇒ the raw-mode flag is not observable).
    #[test]
    fn tui_restores_the_terminal_when_the_loop_fails() {
        let restored = Arc::new(AtomicBool::new(false));
        let flag = restored.clone();
        let result = with_terminal_restore(
            move || flag.store(true, Ordering::SeqCst),
            || -> std::result::Result<(), std::io::Error> {
                Err(std::io::Error::other("I/O error"))
            },
        );
        assert!(result.is_err(), "the error must reach the caller");
        assert!(
            restored.load(Ordering::SeqCst),
            "the terminal must be restored on the error path"
        );
    }
}
