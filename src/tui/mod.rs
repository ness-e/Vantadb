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
    let mut mode = TuiMode::Dashboard;

    let mut dash_data = DashboardData::default();
    let mut mon_data = MonitorData::new();
    let mut repl_state = ReplState::new(engine.clone());
    let tick_rate = Duration::from_millis(250);

    // Initial data load
    dash_data.refresh(&engine);

    loop {
        terminal.draw(|frame| match mode {
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
            TuiMode::Dashboard => dash_data.refresh(&engine),
            TuiMode::Monitor => mon_data.refresh(&engine),
            TuiMode::Repl => {}
        }
    }

    ratatui::restore();
    terminal::disable_raw_mode()?;
    Ok(())
}
