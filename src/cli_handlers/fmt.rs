// ponytail: `ProgressBar::with_template` template strings are compile-time
// literals validated by `Spinner::new` upstream — `expect` here only fires
// if the literal is malformed, which we control via tests.
#![allow(clippy::expect_used, clippy::unwrap_used)]

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
            // INVARIANT (B2b, cat. (b)): hardcoded template, valid by
            // construction — verified by the crate's CLI smoke tests.
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

/// True when stdout is attached to a terminal (TTY).
pub fn stdout_is_term() -> bool {
    Term::stdout().is_term()
}

/// Truncate `s` to at most `max` characters (appending `...`) **only** when
/// `is_terminal` is true. Piped output is returned whole so scripts get the
/// complete value; `--json` output is always complete regardless.
///
/// Char-based (not byte-based) so multi-byte payloads can never panic.
pub fn truncate_for_term(s: &str, max: usize, is_terminal: bool) -> String {
    if !is_terminal || s.chars().count() <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max).collect();
    out.push_str("...");
    out
}

/// Convert a relational [`FieldValue`] into a plain JSON value for `--json`
/// output (no externally-tagged enum wrappers).
pub fn field_value_to_json(value: &crate::node::FieldValue) -> serde_json::Value {
    use crate::node::FieldValue;
    use serde_json::Value as J;
    match value {
        FieldValue::String(s) => J::String(s.clone()),
        FieldValue::Int(i) => J::from(*i),
        FieldValue::Float(f) => J::from(*f),
        FieldValue::Bool(b) => J::from(*b),
        FieldValue::DateTime(dt) => J::String(dt.to_rfc3339()),
        FieldValue::ListString(v) => J::Array(v.iter().map(|s| J::String(s.clone())).collect()),
        FieldValue::ListInt(v) => J::Array(v.iter().map(|i| J::from(*i)).collect()),
        FieldValue::ListFloat(v) => J::Array(v.iter().map(|f| J::from(*f)).collect()),
        FieldValue::ListBool(v) => J::Array(v.iter().map(|b| J::from(*b)).collect()),
        FieldValue::ListDateTime(v) => {
            J::Array(v.iter().map(|dt| J::String(dt.to_rfc3339())).collect())
        }
        FieldValue::Null => J::Null,
    }
}

/// Print `value` as pretty JSON to stdout. `--json` output is always complete.
pub fn print_json(value: &serde_json::Value) -> crate::error::Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(value).map_err(|e| {
            crate::error::Error::Cli(crate::error::ChainedError::msg(format!(
                "JSON serialization error: {e}"
            )))
        })?
    );
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::FieldValue;

    #[test]
    fn truncate_for_term_truncates_on_tty_only() {
        let long = "x".repeat(50);
        assert_eq!(
            truncate_for_term(&long, 10, true),
            format!("{}...", "x".repeat(10))
        );
        assert_eq!(
            truncate_for_term(&long, 10, false),
            long,
            "piped output must never be truncated"
        );
    }

    #[test]
    fn truncate_for_term_keeps_short_strings() {
        assert_eq!(truncate_for_term("short", 10, true), "short");
        assert_eq!(truncate_for_term("exactly-10", 10, true), "exactly-10");
    }

    #[test]
    fn truncate_for_term_is_char_safe_with_multibyte() {
        // 40 emoji = 40 chars / 160 bytes; byte slicing would panic.
        let s: String = "🎯".repeat(40);
        assert_eq!(
            truncate_for_term(&s, 10, true),
            format!("{}...", "🎯".repeat(10))
        );
    }

    #[test]
    fn field_value_to_json_maps_scalars_lists_and_null() {
        assert_eq!(
            field_value_to_json(&FieldValue::Int(7)),
            serde_json::json!(7)
        );
        assert_eq!(
            field_value_to_json(&FieldValue::String("a".into())),
            serde_json::json!("a")
        );
        assert_eq!(
            field_value_to_json(&FieldValue::Bool(true)),
            serde_json::json!(true)
        );
        assert_eq!(
            field_value_to_json(&FieldValue::Null),
            serde_json::Value::Null
        );
        assert_eq!(
            field_value_to_json(&FieldValue::ListString(vec!["a".into(), "b".into()])),
            serde_json::json!(["a", "b"])
        );
    }
}
