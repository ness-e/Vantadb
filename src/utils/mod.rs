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

/// First of two environment variables that **transports** a value; empty counts
/// as unset on both sides.
///
/// Exactly `env_value(first).or_else(|| env_value(second))`, factored so the
/// fall-through is testable: a bare `first.or_else(|| second)` stops at
/// `Some("")` because `var_os` reports `HOME=` as present-and-empty, so
/// `USERPROFILE` was never consulted (FIND-751, and FIND-738 before it).
/// Order is the caller's: `HOME` first on POSIX-shaped call sites,
/// `USERPROFILE` first on Windows-shaped ones.
pub(crate) fn env_chain(first: Option<OsString>, second: Option<OsString>) -> Option<OsString> {
    env_value(first).or_else(|| env_value(second))
}

#[cfg(test)]
mod tests {
    use super::*;

    // FIND-751: the chain must fall through an empty first variable. The bug
    // this guards was shipped once: `var_os("HOME").or_else(|| var_os("USERPROFILE"))`
    // returns `Some("")` for `HOME=` and never reads the second variable.
    #[test]
    fn env_chain_falls_through_an_empty_first_value() {
        let home = OsString::from("/home/tester");

        assert_eq!(env_chain(None, None), None);
        assert_eq!(env_chain(Some(OsString::new()), None), None);
        assert_eq!(env_chain(None, Some(home.clone())), Some(home.clone()));
        assert_eq!(
            env_chain(Some(OsString::new()), Some(home.clone())),
            Some(home.clone()),
            "an empty first variable must not stop the chain"
        );
        assert_eq!(
            env_chain(Some(OsString::from("/first")), Some(home)),
            Some(OsString::from("/first")),
            "a real first value wins"
        );
    }
}
