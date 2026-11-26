//! Product PII audit for a VantaDB store (ICP-02).
//!
//! Scans every byte of the given paths (files or directories, recursive) with
//! the byte-oriented detectors of the proxy `[redact]` policy — `email`,
//! `aws_key`, `aws_secret`, `token` — plus any `--pattern` regex, and exits
//! non-zero when any cleartext match is found.
//!
//! Two honesty invariants, both deliberate:
//!
//! - **Value-free output.** Findings carry `path`, `kind` and `offset` only —
//!   never the matched text (same rule as the redaction provenance kinds).
//! - **Not audited is not clean.** A file over the per-file cap is reported as
//!   `unscanned` and fails the run; symlinks, non-regular entries and unreadable
//!   files are reported the same way instead of being silently skipped.
//!
//! The scan is **binary-safe** ([`Redactor::scan_bytes`]): non-UTF-8 files are
//! scanned byte-wise and reported as `non_utf8_files` — never counted as
//! audited while being skipped. That covers every persisted surface at once —
//! store records, derived indexes, journals, WAL/shrink columns — regardless
//! of the on-disk encoding, plus any explicit extra files (export v2 JSONL).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde_json::json;
use vanta_proxy::redact::{RedactConfig, Redactor};

/// Per-file scan cap. Files above this are reported as `unscanned` (exit ≠ 0):
/// an audit must never call "clean" what it did not read.
const DEFAULT_MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;

fn usage() -> &'static str {
    "vanta-pii-audit — cleartext PII audit for a VantaDB store (ICP-02)

USAGE:
    vanta-pii-audit [--json] [--pattern <regex>]... <path>...

Each <path> is a file or a directory (recursive, deterministic order). Every
byte is scanned with the byte-oriented detectors of the proxy `[redact]`
policy (email, aws_key, aws_secret, token) plus any --pattern regex. Findings
are value-free: path, kind, offset — never the matched text. The scan is
binary-safe: non-UTF-8 files are scanned byte-wise and counted as
`non_utf8_files` (never skipped silently).

A file larger than 64 MiB, a symlink or a non-regular entry is reported as
`unscanned` and fails the run: not audited is not clean.

EXIT CODES:
    0  clean — every byte of every file was scanned and nothing matched
    1  findings and/or unscanned files
    2  usage error (unknown flag, missing path, unreadable path)
"
}

/// Parsed command line.
struct Args {
    json: bool,
    patterns: Vec<String>,
    paths: Vec<PathBuf>,
}

/// One cleartext match — value-free by construction.
struct FindingLine {
    path: String,
    kind: String,
    offset: usize,
}

/// One file the audit could not scan (fail-closed).
struct UnscannedLine {
    path: String,
    reason: String,
}

#[derive(Default)]
struct AuditReport {
    audited_files: usize,
    audited_bytes: u64,
    /// Files scanned byte-wise because they are not valid UTF-8 (coverage
    /// signal: they were scanned, not skipped).
    non_utf8_files: usize,
    findings: Vec<FindingLine>,
    unscanned: Vec<UnscannedLine>,
}

impl AuditReport {
    /// Clean = everything scanned and nothing matched.
    fn is_clean(&self) -> bool {
        self.findings.is_empty() && self.unscanned.is_empty()
    }

    /// Machine-readable summary. Never includes matched values.
    fn to_json(&self) -> String {
        json!({
            "ok": self.is_clean(),
            "audited_files": self.audited_files,
            "audited_bytes": self.audited_bytes,
            "non_utf8_files": self.non_utf8_files,
            "findings": self
                .findings
                .iter()
                .map(|f| json!({ "path": f.path, "kind": f.kind, "offset": f.offset }))
                .collect::<Vec<_>>(),
            "unscanned": self
                .unscanned
                .iter()
                .map(|u| json!({ "path": u.path, "reason": u.reason }))
                .collect::<Vec<_>>(),
        })
        .to_string()
    }

    fn print_human(&self) {
        for f in &self.findings {
            println!(
                "FINDING kind={} offset={} path={}",
                f.kind, f.offset, f.path
            );
        }
        for u in &self.unscanned {
            println!("UNSCANNED path={} reason={}", u.path, u.reason);
        }
        if self.is_clean() {
            println!(
                "CLEAN: {} file(s) ({} non-UTF-8 scanned byte-wise), {} byte(s) audited, 0 findings",
                self.audited_files, self.non_utf8_files, self.audited_bytes
            );
        } else {
            println!(
                "NOT CLEAN: {} finding(s), {} unscanned, {} file(s) audited",
                self.findings.len(),
                self.unscanned.len(),
                self.audited_files
            );
        }
    }
}

/// Parse argv (without the program name).
fn parse_args(argv: Vec<String>) -> Result<Args, String> {
    let mut json = false;
    let mut patterns: Vec<String> = Vec::new();
    let mut paths: Vec<PathBuf> = Vec::new();
    let mut it = argv.into_iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--json" => json = true,
            "--pattern" => {
                let re = it.next().ok_or("--pattern requires a regex argument")?;
                patterns.push(re);
            }
            other if other.starts_with("--") => {
                return Err(format!("unknown flag '{other}'"));
            }
            other => paths.push(PathBuf::from(other)),
        }
    }
    if paths.is_empty() {
        return Err("at least one <path> is required".into());
    }
    Ok(Args {
        json,
        patterns,
        paths,
    })
}

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    if argv.iter().any(|a| a == "--help" || a == "-h") {
        print!("{}", usage());
        return ExitCode::SUCCESS;
    }
    let args = match parse_args(argv) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("vanta-pii-audit: {e}\n\n{}", usage());
            return ExitCode::from(2);
        }
    };
    match audit_paths(&args.paths, &args.patterns, DEFAULT_MAX_FILE_BYTES) {
        Ok(report) => {
            if args.json {
                println!("{}", report.to_json());
            } else {
                report.print_human();
            }
            if report.is_clean() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(e) => {
            eprintln!("vanta-pii-audit: {e}");
            ExitCode::from(2)
        }
    }
}

/// Audit every path with the built-in detectors + `patterns`.
fn audit_paths(
    paths: &[PathBuf],
    patterns: &[String],
    max_file_bytes: u64,
) -> Result<AuditReport, String> {
    let redactor = Redactor::new(&RedactConfig {
        enabled: true,
        patterns: patterns.to_vec(),
        // The per-file cap is enforced by the explicit size check below, so the
        // scan itself never fail-opens on an oversize body.
        max_scan_bytes: usize::MAX,
        ..RedactConfig::default()
    })
    .map_err(|e| format!("invalid --pattern: {e}"))?;

    let mut report = AuditReport::default();
    for path in paths {
        fs::symlink_metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut files: Vec<PathBuf> = Vec::new();
        collect_files(path, &mut files).map_err(|e| format!("{}: {e}", path.display()))?;
        for file in files {
            scan_file(&redactor, &file, max_file_bytes, &mut report);
        }
    }
    Ok(report)
}

/// Collect regular-file candidates under `path` in deterministic order.
/// Symlinked entries are collected too — `scan_file` reports them as
/// `unscanned` (never silently skipped, and never recursed into).
fn collect_files(path: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    let meta = fs::symlink_metadata(path)?;
    if meta.file_type().is_symlink() || meta.is_file() {
        out.push(path.to_path_buf());
        return Ok(());
    }
    if meta.is_dir() {
        let mut entries: Vec<PathBuf> = Vec::new();
        for entry in fs::read_dir(path)? {
            entries.push(entry?.path());
        }
        entries.sort();
        for entry in &entries {
            collect_files(entry, out)?;
        }
    }
    Ok(())
}

/// Scan one file into the report; oversize/symlink/IO failures land in
/// `unscanned` (fail-closed).
fn scan_file(redactor: &Redactor, path: &Path, max_file_bytes: u64, report: &mut AuditReport) {
    let display = path.display().to_string();
    let meta = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) => {
            report.unscanned.push(UnscannedLine {
                path: display,
                reason: format!("stat failed: {e}"),
            });
            return;
        }
    };
    if meta.file_type().is_symlink() {
        report.unscanned.push(UnscannedLine {
            path: display,
            reason: "symlink (pass the target path to audit it)".into(),
        });
        return;
    }
    if !meta.is_file() {
        report.unscanned.push(UnscannedLine {
            path: display,
            reason: "not a regular file".into(),
        });
        return;
    }
    let len = meta.len();
    if len > max_file_bytes {
        report.unscanned.push(UnscannedLine {
            path: display,
            reason: format!("oversize: {len} bytes > {max_file_bytes} byte cap"),
        });
        return;
    }
    match fs::read(path) {
        Ok(bytes) => {
            report.audited_files += 1;
            report.audited_bytes += bytes.len() as u64;
            if std::str::from_utf8(&bytes).is_err() {
                report.non_utf8_files += 1;
            }
            // Binary-safe scan: non-UTF-8 content is scanned byte-wise (the
            // wire `scan` fails open there — the audit must not).
            for finding in redactor.scan_bytes(&bytes) {
                report.findings.push(FindingLine {
                    path: display.clone(),
                    kind: finding.kind.as_str().to_string(),
                    offset: finding.start,
                });
            }
        }
        Err(e) => report.unscanned.push(UnscannedLine {
            path: display,
            reason: format!("read failed: {e}"),
        }),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    const EMAIL: &str = "jane.doe@example.com";
    const AWS_KEY: &str = "AKIAIOSFODNN7EXAMPLE";

    #[test]
    fn flags_synthetic_pii_and_never_echoes_values() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("leak.txt");
        fs::write(&file, format!("contact {EMAIL} and deploy {AWS_KEY} now")).unwrap();

        let report = audit_paths(&[file], &[], DEFAULT_MAX_FILE_BYTES).unwrap();
        assert!(!report.is_clean());
        let kinds: Vec<&str> = report.findings.iter().map(|f| f.kind.as_str()).collect();
        assert!(kinds.contains(&"email"), "kinds: {kinds:?}");
        assert!(kinds.contains(&"aws_key"), "kinds: {kinds:?}");

        // Value-free: the report text must not carry the matched values.
        let json = report.to_json();
        assert!(!json.contains(EMAIL), "report leaked the email: {json}");
        assert!(!json.contains(AWS_KEY), "report leaked the key: {json}");
    }

    #[test]
    fn masked_content_is_clean_and_files_are_counted() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("masked.txt");
        fs::write(
            &file,
            "contact [REDACTED_EMAIL] and deploy [REDACTED_AWS_KEY]",
        )
        .unwrap();

        let report = audit_paths(&[file], &[], DEFAULT_MAX_FILE_BYTES).unwrap();
        assert!(report.is_clean(), "masked placeholders are not findings");
        assert_eq!(report.audited_files, 1);
        assert!(report.audited_bytes > 0);
    }

    #[test]
    fn oversize_file_is_unscanned_and_fails_closed() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("big.bin");
        fs::write(&file, vec![b'x'; 64]).unwrap();

        let report = audit_paths(&[file], &[], 8).unwrap();
        assert!(!report.is_clean(), "an unaudited file is not clean");
        assert_eq!(report.unscanned.len(), 1);
        assert_eq!(report.audited_files, 0);
        assert!(report.unscanned[0].reason.contains("oversize"));
    }

    #[test]
    fn directory_walk_is_recursive_and_deterministic() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("b")).unwrap();
        fs::write(dir.path().join("z.txt"), "z").unwrap();
        fs::write(dir.path().join("b/inner.txt"), "i").unwrap();
        fs::write(dir.path().join("a.txt"), "a").unwrap();

        let mut files = Vec::new();
        collect_files(dir.path(), &mut files).unwrap();
        let order: Vec<String> = files
            .iter()
            .map(|p| {
                p.strip_prefix(dir.path())
                    .unwrap()
                    .display()
                    .to_string()
                    // Platform-agnostic: Windows renders `\` as separator.
                    .replace('\\', "/")
            })
            .collect();
        assert_eq!(order, vec!["a.txt", "b/inner.txt", "z.txt"]);
    }

    #[test]
    fn custom_patterns_extend_the_builtins() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("custom.txt");
        fs::write(&file, "token SECRET-1234 here").unwrap();

        let report =
            audit_paths(&[file], &[r"SECRET-\d{4}".into()], DEFAULT_MAX_FILE_BYTES).unwrap();
        assert!(!report.is_clean());
        assert_eq!(report.findings[0].kind, "custom");
    }

    // ── ICP-02 review (Critical): binary files must be scanned, not skipped ──

    /// The old fail-open: a clear email inside a non-UTF-8 file was counted as
    /// audited while `scan` returned empty → `ok:true`. Now it must be a
    /// finding (exit ≠ 0 at the binary level).
    #[test]
    fn binary_file_with_cleartext_leak_is_detected() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("leak.bin");
        let mut bytes = format!("leaked {EMAIL} ").into_bytes();
        bytes.extend_from_slice(&[0xff, 0x00, 0xfe, 0x80]);
        fs::write(&file, bytes).unwrap();

        let report = audit_paths(&[file], &[], DEFAULT_MAX_FILE_BYTES).unwrap();
        assert!(
            !report.is_clean(),
            "a cleartext leak inside binary must fail the audit"
        );
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].kind, "email");
        assert_eq!(report.non_utf8_files, 1, "coverage signal");
    }

    /// The real-store shape: a non-UTF-8 file whose only hits are already
    /// masked placeholders is clean AND counted as scanned one byte at a time.
    #[test]
    fn binary_masked_file_is_clean_and_counted_as_scanned() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("0.jnl");
        let mut bytes = b"record [REDACTED_EMAIL] [REDACTED_AWS_KEY]".to_vec();
        bytes.extend_from_slice(&[0xff, 0xfe, 0x00]);
        fs::write(&file, bytes).unwrap();

        let report = audit_paths(&[file], &[], DEFAULT_MAX_FILE_BYTES).unwrap();
        assert!(report.is_clean(), "masked placeholders are not findings");
        assert_eq!(report.audited_files, 1);
        assert_eq!(report.non_utf8_files, 1);
        assert!(report.to_json().contains("\"non_utf8_files\":1"));
    }
}
