//! Utility modules extracted from the governance framework (archived 2024-06); these are production-ready.
//!
//! These are stateless, useful utilities for multi-writer and multi-agent scenarios
//! that don't require the full runtime governance framework.

pub mod confidence_metrics;
pub mod duplicate_prevention;
pub mod fs;

pub use confidence_metrics::compute_confidence_friction;

use std::ffi::OsString;

/// A boolean environment flag is ON only when the variable exists **and is not
/// empty**.
///
/// `var_os` returns `Some("")` for a defined-but-empty variable, so a bare
/// `is_some()` lets `VAR=` flip a flag — the form a shell user produces with
/// `VAR= cargo …`, and the form a mistyped CI `env:` leaves behind. Same class
/// as FIND-731 (where the empty form auto-accepted the public contract) and
/// FIND-738 (five more sites).
///
/// Takes the `Option` instead of reading the environment so it is testable
/// without `set_var`, which is racy under nextest.
#[allow(dead_code)] // callers are feature-gated (cli / sysinfo) and test-gated
pub(crate) fn env_present(value: Option<OsString>) -> bool {
    value.is_some_and(|v| !v.is_empty())
}

/// The value of an environment variable that **transports** something (a path,
/// a binary name), or `None` when it is absent **or empty**.
///
/// Empty is treated as unset so `VAR=` can never yield an empty path
/// (`Command::new("")`, `PathBuf::from("")`) — FIND-738.
#[allow(dead_code)] // callers are feature-gated (sysinfo) and test-gated
pub(crate) fn env_value(value: Option<OsString>) -> Option<OsString> {
    value.filter(|v| !v.is_empty())
}
