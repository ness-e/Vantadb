//! FIND-738: one predicate for "an environment variable exists and carries a
//! value".
//!
//! `std::env::var_os` returns `Some("")` for a defined-but-empty variable, so a
//! bare `is_some()` (or `is_none()`) lets `VAR=` flip a flag or hand an empty
//! path to `Command::new` / `PathBuf::from`.
//!
//! **Why this file duplicates `crate::utils::env_present` / `env_value`:** the
//! five FIND-738 sites straddle a crate boundary. Each `tests/*.rs` is its own
//! crate — a `pub(crate)` helper in `src/` is unreachable from here, and
//! promoting it to `pub` would change the frozen public-API surface that
//! `tests/api/public_api.rs` compares against (the FIND-731 gate). Two
//! definitions with identical names and semantics beat five copies; the lib
//! side owns `src/utils/mod.rs`, this one owns the integration tests.
//!
//! Included with `#[path]` by the tests that need it, so they do not drag in
//! the 22 KB `common/mod.rs` harness.

#![allow(dead_code)] // each includer uses only one of the two predicates

use std::ffi::OsString;

/// A boolean environment flag is ON only when the variable exists **and is not
/// empty**.
pub fn env_present(value: Option<OsString>) -> bool {
    value.is_some_and(|v| !v.is_empty())
}

/// The value of an environment variable that **transports** something, or
/// `None` when it is absent **or empty** (an empty path is not a path).
pub fn env_value(value: Option<OsString>) -> Option<OsString> {
    value.filter(|v| !v.is_empty())
}
