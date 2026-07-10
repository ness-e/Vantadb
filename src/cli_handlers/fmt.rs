//! CLI formatting helpers — spinners, styled output, confirm prompts.

use console::{Style, Term};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub const MIB: u64 = 1024 * 1024;
pub const KIB_F64: f64 = 1024.0;

/// Create a styled spinner for indeterminate operations
pub fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.cyan} {msg}")
            .expect("valid spinner template"),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

pub(crate) fn success_style() -> Style {
    Style::new().green().bold()
}

pub(crate) fn error_style() -> Style {
    Style::new().red().bold()
}

pub(crate) fn info_style() -> Style {
    Style::new().cyan()
}

pub(crate) fn warning_style() -> Style {
    Style::new().yellow()
}

pub(crate) fn header_style() -> Style {
    Style::new().white().bold()
}

/// Print a green success message to stdout
pub fn print_success(msg: &str) {
    let term = Term::stdout();
    let _ = term.write_line(&format!("{} {}", success_style().apply_to("✓"), msg));
}

/// Print a red error message to stderr
pub fn print_error(msg: &str) {
    let term = Term::stderr();
    let _ = term.write_line(&format!("{} {}", error_style().apply_to("✗"), msg));
}

/// Print a cyan info message to stdout
pub fn print_info(msg: &str) {
    let term = Term::stdout();
    let _ = term.write_line(&format!("{} {}", info_style().apply_to("ℹ"), msg));
}

/// Print a yellow warning message to stdout
pub fn print_warning(msg: &str) {
    let term = Term::stdout();
    let _ = term.write_line(&format!("{} {}", warning_style().apply_to("⚠"), msg));
}

/// Prompt the user for a yes/no confirmation
pub fn confirm_action(prompt: &str) -> std::io::Result<bool> {
    let term = Term::stdout();
    let _ = term.write_str(prompt);
    let _ = term.write_str(" [y/N] ");
    let _ = term.flush();
    let result = term.read_line()?;
    Ok(result.trim().eq_ignore_ascii_case("y") || result.trim().eq_ignore_ascii_case("yes"))
}
