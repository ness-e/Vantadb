//! Public API snapshot test — breaking-change rails for the `vantadb` crate (HARD-01).
//!
//! Regenerates the public API surface of the current tree (rustdoc JSON via the
//! `nightly` toolchain) and compares it against the committed golden file
//! `tests/api/public-api.txt`. Any change to the public surface (added, removed
//! or changed items) fails this test until the author acknowledges it by
//! updating the snapshot and getting the update reviewed.
//!
//! Update the snapshot:
//!   pwsh:  $env:VANTADB_PUBLIC_API_UPDATE="1"; cargo nextest run -p vantadb --test public_api
//!   bash:  VANTADB_PUBLIC_API_UPDATE=1 cargo nextest run -p vantadb --test public_api
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

/// Set to any value to rewrite the snapshot instead of comparing against it.
const UPDATE_ENV: &str = "VANTADB_PUBLIC_API_UPDATE";

fn generate_public_api() -> Result<String, String> {
    let target_dir = std::env::var("CARGO_TARGET_DIR")
        .ok()
        .filter(|v| !v.is_empty());
    let mut builder = rustdoc_json::Builder::default()
        .toolchain("nightly")
        .manifest_path("Cargo.toml")
        .package("vantadb")
        .quiet(true);
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
fn public_api_snapshot_matches_committed_file() {
    let current =
        normalize(&generate_public_api().expect("could not generate the current public API"));

    if std::env::var_os(UPDATE_ENV).is_some() {
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
