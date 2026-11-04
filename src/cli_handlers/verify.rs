//! CLI handler for audit log verification (EU AI Act Art. 12 / TEC-13).

use crate::audit::verify_merkle_trail;
use crate::cli_handlers::fmt::{print_error, print_json, print_success};
use crate::error::Result;

/// Verify the Merkle trail of an audit.jsonl log file.
/// Returns exit code (0 for success, 1 for failure/tamper).
#[tracing::instrument]
pub fn cmd_audit_verify(path: &str, json_output: bool) -> Result<i32> {
    match verify_merkle_trail(path) {
        Ok(report) => {
            if json_output {
                print_json(&serde_json::json!({
                    "status": "ok",
                    "events_verified": report.events_verified,
                    "head_hash": report.head_hash,
                }))?;
            } else {
                print_success(&format!(
                    "Verified {} audit events in '{}'. Merkle chain intact.\nHead hash: {}",
                    report.events_verified, path, report.head_hash
                ));
            }
            Ok(0)
        }
        Err(err) => {
            if json_output {
                print_json(&serde_json::json!({
                    "status": "error",
                    "line": err.line_number,
                    "reason": err.reason,
                    "expected_hash": err.expected_hash,
                    "actual_hash": err.actual_hash,
                }))?;
            } else {
                print_error(&format!(
                    "Audit verification failed at line {}: {} (expected {}, found {})",
                    err.line_number, err.reason, err.expected_hash, err.actual_hash
                ));
            }
            Ok(1)
        }
    }
}
