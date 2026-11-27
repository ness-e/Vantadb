//! Public API snapshot test — breaking-change rails for the `vantadb` crate (HARD-01).
//!
//! Regenerates the public API surface of the current tree (rustdoc JSON via the
//! `nightly` toolchain) and compares it against the committed golden file
//! `tests/api/public-api.txt`. Any change to the public surface (added, removed
//! or changed items) fails this test until the author acknowledges it by
//! updating the snapshot and getting the update reviewed.
//!
//! The test is `#[ignore]`d: it needs a nightly toolchain plus a full rustdoc
//! JSON build, so only the dedicated `public-api-snapshot` CI job runs it
//! (`--run-ignored ignored-only`); general suites and sanitizers skip it.
//!
//! Update the snapshot:
//!   pwsh:  $env:VANTADB_PUBLIC_API_UPDATE="1"; cargo nextest run -p vantadb --test public_api --run-ignored ignored-only
//!   bash:  VANTADB_PUBLIC_API_UPDATE=1 cargo nextest run -p vantadb --test public_api --run-ignored ignored-only
//!
//! FIND-191 — the cold-cache property, and why it is not a test here:
//! only a run that still has something to compile (`cargo rustdoc` on a cold
//! target) can kill the child compiler with `STATUS_STACK_OVERFLOW`
//! (`0xC00000FD`, whose low byte is `253`), so the *first* run can look poisoned
//! while the second passes. The budget is fixed by [`CHILD_STACK_BYTES`]; the
//! property itself needs an empty target dir, so it is declared, not automated.
//! Measured (empty `CARGO_TARGET_DIR`, Windows/MSVC, nightly 1.101.0 from
//! 2026-10-01), both through nextest and through the test binary directly:
//!   $env:CARGO_TARGET_DIR = "<empty dir>"; $env:RUST_MIN_STACK = "2097152"
//!   cargo nextest run -p vantadb --test public_api --run-ignored ignored-only
//! `thread 'rustc' has overflowed its stack` → the build aborts inside the first
//! dependency (`typenum`, ~8 s in) with exit code `0xC00000FD`; the same command
//! is 1/1 PASS after the fix. 3 MiB and 4 MiB ambient already pass — the floor
//! below is margin above that boundary, not the boundary itself.
//!
//! Generation parameters (must stay in sync with the snapshot header):
//! - toolchain: `nightly` (rustdoc JSON is nightly-only)
//! - features: default (`vantadb` default feature set)
//! - `public_api` omit_blanket_impls(true) — equivalent to `cargo public-api --simplified` (one -s)
//!
//! Policy: docs/api/VERSIONING.md + docs/api/COMPATIBILITY.md.
//! ponytail: regeneration needs a nightly rustdoc build (minutes) — this test is
//! for a dedicated CI job, not the <5 min Fast Gate.

// ponytail: blanket allow — expects carry documented invariants at each call site.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::ffi::OsString;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

/// Golden file, relative to the workspace root (test CWD = package root).
const SNAPSHOT: &str = "tests/api/public-api.txt";

/// Human-readable header written at the top of the golden file. Lines starting
/// with `//` are ignored by the comparison.
const HEADER: &str = "\
// Public API snapshot of the `vantadb` crate — golden file for `tests/api/public_api.rs`.
//
// Regenerate with: VANTADB_PUBLIC_API_UPDATE=1 cargo nextest run -p vantadb --test public_api
//
// Generation parameters: nightly rustdoc JSON; default features; public-api 0.52.x with
// omit_blanket_impls(true) (equivalent to `cargo public-api --simplified`, a single -s).
// Excludes: none — the surface below is the full simplified default-features API.
// Lines starting with `//` are ignored by the snapshot comparison (see public_api.rs).
";

/// Set to a non-empty value (e.g. `1`) to rewrite the snapshot instead of
/// comparing against it. An empty variable is treated as "not set" (FIND-731).
const UPDATE_ENV: &str = "VANTADB_PUBLIC_API_UPDATE";

/// Stack budget handed to the child compiler, in bytes (FIND-191).
///
/// `rustdoc_json` spawns `rustup run nightly cargo rustdoc`, i.e. the nightly
/// **`rustc`**, and parts of the compiler's work run on threads sized from
/// `RUST_MIN_STACK`. Below that floor the compiler dies with
/// `STATUS_STACK_OVERFLOW` (`0xC00000FD`, low byte `253`) — and only when there
/// is something left to compile, so it lands on the **first** run and disappears
/// on the second:
///
/// ```text
/// thread 'rustc' has overflowed its stack
/// process didn't exit successfully: ...rustc.exe --crate-name typenum ... (exit code: 0xc00000fd, STATUS_STACK_OVERFLOW)
/// ```
///
/// Measured on this crate with an empty target dir: 2 MiB ambient reproduces the
/// abort (8 s in, on the first dependency), while 3 MiB and 4 MiB complete the
/// cold build. The budget below is **margin above that boundary**, not the
/// boundary itself — the boundary moves with the crate's dependency graph and
/// with each nightly, so tracking it here would be a number that goes stale.
/// The recursion is inside the third-party compiler, not in this crate, so it
/// cannot be bounded here; what this test *can* stop is inheriting whatever
/// budget the caller's shell happened to export. 32 MiB is the value the repo's
/// own gate already uses (`dev-tools/verify.ps1:63`) — not a number invented for
/// this file.
const CHILD_STACK_BYTES: u32 = 32 * 1024 * 1024;

/// Compile-time rail (FIND-191): lowering the floor toward the measured abort
/// re-opens the cold-cache overflow, so that must be a build error rather than a
/// review note. 2 MiB reproduces the abort and 3 MiB passes; 8 MiB keeps margin
/// without pretending to know where the next boundary is.
const _: () = assert!(
    CHILD_STACK_BYTES >= 8 * 1024 * 1024,
    "FIND-191: 2 MiB reproduces the cold-cache abort - keep margin above it"
);

/// The stack budget to pass to the child compiler: the ambient value when it is
/// already at or above [`CHILD_STACK_BYTES`], otherwise the floor (FIND-191).
///
/// A larger operator setting is preserved (someone with 64 MiB keeps it); a
/// smaller one, a non-numeric one, or an absent one gets the floor. Never
/// returns the ambient value when it is below the floor — that is the bug.
fn child_stack_budget(ambient: Option<&str>) -> String {
    let floor = CHILD_STACK_BYTES as u64;
    match ambient.and_then(|v| v.trim().parse::<u64>().ok()) {
        Some(bytes) if bytes >= floor => bytes.to_string(),
        _ => CHILD_STACK_BYTES.to_string(),
    }
}

/// Whether the snapshot should be rewritten.
///
/// The flag is ON only when the variable exists **and is not empty**. `var_os`
/// returns `Some("")` for a defined-but-empty variable, so a bare `is_some()`
/// made `VANTADB_PUBLIC_API_UPDATE= cargo nextest run ...` rewrite the golden
/// and report PASS — a false green that auto-accepted any contract delta
/// (FIND-731).
fn update_requested(value: Option<OsString>) -> bool {
    value.is_some_and(|v| !v.is_empty())
}

fn generate_public_api() -> Result<String, String> {
    let target_dir = std::env::var("CARGO_TARGET_DIR")
        .ok()
        .filter(|v| !v.is_empty());
    let mut builder = rustdoc_json::Builder::default()
        .toolchain("nightly")
        .manifest_path("Cargo.toml")
        .package("vantadb")
        .quiet(true)
        // FIND-191: the child compiler must not inherit a stack budget too small
        // to compile this crate's dependency graph (cold-cache 0xC00000FD / 253).
        .env(
            "RUST_MIN_STACK",
            child_stack_budget(std::env::var("RUST_MIN_STACK").ok().as_deref()),
        );
    if let Some(dir) = target_dir.as_deref() {
        builder = builder.target_dir(Path::new(dir));
    }
    let json_path = builder
        .build()
        .map_err(|e| format!("rustdoc JSON build failed: {e}"))?;
    let api = public_api::Builder::from_rustdoc_json(json_path)
        .omit_blanket_impls(true)
        .build()
        .map_err(|e| format!("public-api build failed: {e}"))?;
    Ok(api.to_string())
}

/// Drop `//` header lines and trailing whitespace, and guarantee a final
/// newline, so the golden file can carry a readable header.
fn normalize(text: &str) -> String {
    let mut out = String::new();
    for line in text.lines() {
        if line.starts_with("//") {
            continue;
        }
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out
}

/// Small, readable diff: up to 10 differing lines + totals.
fn first_diff(expected: &str, actual: &str) -> String {
    let mut msg = String::new();
    let mut total = 0usize;
    let mut shown = 0usize;
    for (i, (e, a)) in expected.lines().zip(actual.lines()).enumerate() {
        if e != a {
            total += 1;
            if shown < 10 {
                let _ = writeln!(
                    msg,
                    "  line {}:\n    snapshot: {e}\n    current:  {a}",
                    i + 1
                );
                shown += 1;
            }
        }
    }
    let e_count = expected.lines().count();
    let a_count = actual.lines().count();
    if e_count != a_count {
        let _ = writeln!(msg, "  line count: snapshot={e_count} current={a_count}");
    }
    if total > shown {
        let _ = writeln!(msg, "  ... and {} more differing lines", total - shown);
    }
    msg
}

#[test]
#[ignore = "requires nightly rustdoc JSON; runs in the public-api-snapshot CI job (--run-ignored ignored-only)"]
fn public_api_snapshot_matches_committed_file() {
    let current =
        normalize(&generate_public_api().expect("could not generate the current public API"));

    if update_requested(std::env::var_os(UPDATE_ENV)) {
        fs::write(SNAPSHOT, format!("{HEADER}{current}")).expect("failed to write the snapshot");
        eprintln!("[public_api] snapshot updated ({UPDATE_ENV} set): {SNAPSHOT}");
        return;
    }

    let snapshot =
        fs::read_to_string(SNAPSHOT).unwrap_or_else(|e| panic!("cannot read {SNAPSHOT}: {e}"));
    let expected = normalize(&snapshot);
    assert!(
        !expected.trim().is_empty(),
        "{SNAPSHOT} is empty — regenerate it with {UPDATE_ENV}=1"
    );
    assert!(
        expected == current,
        "\npublic API surface changed vs {SNAPSHOT}.\n\
         If the change is intentional, update the snapshot ({UPDATE_ENV}=1) and get it reviewed.\n\
         Differences:\n{}",
        first_diff(&expected, &current)
    );
}

/// FIND-731: an empty variable must not enable the update path. Before the fix
/// these assertions fail for the empty case, which is how a mis-written CI
/// invocation (`VAR= cargo test`) silently rewrote the golden and returned PASS.
#[cfg(test)]
mod update_flag {
    use super::update_requested;
    use std::ffi::OsString;

    #[test]
    fn absent_variable_does_not_request_update() {
        assert!(!update_requested(None));
        // ... and the real lookup path: an env name no tool sets.
        assert!(!update_requested(std::env::var_os(
            "VANTADB_PUBLIC_API_UPDATE_FIND731_ABSENT"
        )));
    }

    #[test]
    fn empty_variable_does_not_request_update() {
        // `var_os` yields Some("") for `VANTADB_PUBLIC_API_UPDATE=` — the false green.
        assert!(!update_requested(Some(OsString::from(""))));
    }

    #[test]
    fn non_empty_variable_requests_update() {
        assert!(update_requested(Some(OsString::from("1"))));
        assert!(update_requested(Some(OsString::from("true"))));
    }

    /// Pins the residual ceiling: only *emptiness* is rejected, so an explicit
    /// `=0` or `= ` still rewrites the golden. Documented so a future reader can
    /// tighten it deliberately (parse falsy values) instead of by accident.
    #[test]
    fn explicit_falsy_values_still_request_update() {
        // ponytail: presence-with-content = ON. A CI writing `=0` to disable
        // would still rewrite; upgrade path = parse {0,false,no,off}.
        assert!(update_requested(Some(OsString::from("0"))));
        assert!(update_requested(Some(OsString::from(" "))));
    }
}

/// FIND-191: the child compiler's stack budget. These assertions are the
/// regression rail — the failure they prevent needs a cold cache (measured
/// 0xC00000FD on `rustc`, 8 s in, on the first dependency), so it cannot be
/// reproduced by a test that owns no disk state. What *can* be pinned is the
/// budget, which is the only input we control.
#[cfg(test)]
mod child_stack {
    use super::{child_stack_budget, CHILD_STACK_BYTES};

    #[test]
    fn unset_ambient_gets_the_floor() {
        assert_eq!(child_stack_budget(None), CHILD_STACK_BYTES.to_string());
    }

    #[test]
    fn ambient_below_the_floor_is_raised() {
        // 2 MiB reproduces the cold-cache abort (FIND-191). The value in
        // docs/user/troubleshooting.md:53 is 4 MiB, which already passes today —
        // both are below this floor, and the floor is margin.
        assert_eq!(
            child_stack_budget(Some("2097152")),
            CHILD_STACK_BYTES.to_string()
        );
        assert_eq!(
            child_stack_budget(Some("4194304")),
            CHILD_STACK_BYTES.to_string()
        );
    }

    #[test]
    fn ambient_at_or_above_the_floor_is_kept() {
        assert_eq!(
            child_stack_budget(Some(&CHILD_STACK_BYTES.to_string())),
            CHILD_STACK_BYTES.to_string()
        );
        assert_eq!(
            child_stack_budget(Some("67108864")),
            "67108864".to_string(),
            "a larger operator setting must survive the fix"
        );
    }

    #[test]
    fn unparseable_ambient_gets_the_floor() {
        assert_eq!(
            child_stack_budget(Some("not-a-number")),
            CHILD_STACK_BYTES.to_string()
        );
        assert_eq!(child_stack_budget(Some("")), CHILD_STACK_BYTES.to_string());
    }
}
