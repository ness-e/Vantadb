//! Append-only JSONL audit log of business operations.
//!
//! Records *operations* (put/delete/export/import) with an ISO 8601 timestamp,
//! subject, target, and outcome — distinct from runtime `tracing` logs. Opt-in
//! via [`VantaConfig::audit_log_path`](crate::config::VantaConfig::audit_log_path);
//! when unset, `VantaEmbedded` operations skip audit entirely.

use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Default audit rotation threshold: rotate the JSONL once it reaches 10 MiB.
pub const DEFAULT_AUDIT_MAX_BYTES: u64 = 10 * 1024 * 1024;
/// Default rotated-file retention: keep `.1`..`.5`, delete older archives.
pub const DEFAULT_AUDIT_MAX_FILES: u32 = 5;

/// A single audit record: timestamp + operation + subject + target + outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// ISO 8601 UTC timestamp (e.g. `2026-08-02T12:34:56Z`).
    pub timestamp: String,
    /// Operation name: `put`, `delete`, `delete_by_filter`, `put_batch`,
    /// `export_namespace`, `export_all`, `import_file`.
    pub op: String,
    pub namespace: String,
    /// Target record key, or `"N/A"` for operations without a single key.
    pub key: String,
    /// `"ok"` or `"err"`.
    pub outcome: String,
    /// Optional reason (e.g. the delete reason).
    pub reason: Option<String>,
    /// Optional caller-supplied request/tracing id (SRV-02): first match of
    /// `x-request-id` / `x-tracing-id` / `traceparent` on the HTTP request,
    /// truncated to 256 chars. Absent for SDK-internal (non-HTTP) events.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl AuditEvent {
    /// Build an event stamped with the current UTC time.
    pub fn new(
        op: &str,
        namespace: &str,
        key: &str,
        outcome: &str,
        reason: Option<String>,
    ) -> Self {
        Self {
            timestamp: now_iso(),
            op: op.to_string(),
            namespace: namespace.to_string(),
            key: key.to_string(),
            outcome: outcome.to_string(),
            reason,
            request_id: None,
        }
    }

    /// Attach a request/tracing id (e.g. from an `x-request-id` header).
    pub fn with_request_id(mut self, id: impl Into<String>) -> Self {
        self.request_id = Some(id.into());
        self
    }

    /// Attach an optional request/tracing id; no-op when `None`.
    pub fn with_request_id_opt(mut self, id: Option<String>) -> Self {
        self.request_id = id;
        self
    }

    /// Build a memory-pipeline audit event (`memory_{layer}` op) for the
    /// vanta-memory layer (F4): `l1`, `l2`, `l3`, `recall`, `offload`.
    ///
    /// `layer` is validated against the known layers; unknown layers fall back
    /// to the raw string so the JSONL remains appendable (error-silent).
    pub fn memory(
        layer: &str,
        namespace: &str,
        key: &str,
        outcome: &str,
        reason: Option<String>,
    ) -> Self {
        let op = match layer {
            "l1" | "l2" | "l3" | "recall" | "offload" => format!("memory_{layer}"),
            other => format!("memory_{other}"),
        };
        Self::new(&op, namespace, key, outcome, reason)
    }

    /// Build an authentication event (`auth_{l1|l2|l3}` op) for the 3-layer
    /// server auth (MEM-05): `l1` (Bearer token), `l2` (service-id), `l3`
    /// (user-key → user identity).
    ///
    /// `layer` is validated against the known layers; unknown layers fall back
    /// to the raw string so the JSONL remains appendable (error-silent).
    /// `key` carries the *subject* (user_id / service_id), never the secret.
    pub fn auth(
        layer: &str,
        namespace: &str,
        key: &str,
        outcome: &str,
        reason: Option<String>,
    ) -> Self {
        let op = match layer {
            "l1" | "l2" | "l3" => format!("auth_{layer}"),
            other => format!("auth_{other}"),
        };
        Self::new(&op, namespace, key, outcome, reason)
    }
}

/// Append-only JSONL writer for audit events. One JSON object per line.
///
/// Rotates by size (SRV-01): once the active file reaches `max_bytes` it is
/// renamed to `<path>.1`, older archives shift to `.2`..`.N`, and files beyond
/// `max_files` are deleted. Rotation happens under the same mutex as the
/// append, so the log is always consistent for readers.
#[derive(Debug)]
pub struct AuditLogger {
    writer: Mutex<BufWriter<File>>,
    path: PathBuf,
    max_bytes: u64,
    max_files: u32,
}

impl AuditLogger {
    /// Open (creating if needed) the audit file, creating parent directories,
    /// with the default rotation settings ([`DEFAULT_AUDIT_MAX_BYTES`] /
    /// [`DEFAULT_AUDIT_MAX_FILES`]).
    pub fn new(path: impl AsRef<Path>) -> std::io::Result<Self> {
        Self::with_rotation(path, DEFAULT_AUDIT_MAX_BYTES, DEFAULT_AUDIT_MAX_FILES)
    }

    /// Open the audit file with explicit rotation settings.
    ///
    /// `max_bytes` is the size threshold that triggers rotation (clamped to
    /// ≥1); `max_files` is the number of rotated archives retained (clamped to
    /// ≥1, so at least `.1` is kept).
    pub fn with_rotation(
        path: impl AsRef<Path>,
        max_bytes: u64,
        max_files: u32,
    ) -> std::io::Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Self {
            writer: Mutex::new(BufWriter::new(file)),
            path: path.to_path_buf(),
            max_bytes: max_bytes.max(1),
            max_files: max_files.max(1),
        })
    }

    /// Append one event as a JSON line and flush (best-effort durability).
    pub fn record(&self, event: &AuditEvent) -> crate::error::Result<()> {
        let mut guard = self.writer.lock().unwrap_or_else(|e| e.into_inner());
        if self.should_rotate(&guard) {
            self.rotate_locked(&mut guard)?;
        }
        serde_json::to_writer(&mut *guard, event)
            .map_err(crate::error::VantaError::serialization)?;
        guard.write_all(b"\n")?;
        guard.flush()?;
        Ok(())
    }

    /// Whether the active file has reached the rotation threshold.
    fn should_rotate(&self, writer: &BufWriter<File>) -> bool {
        writer
            .get_ref()
            .metadata()
            .map(|m| m.len() >= self.max_bytes)
            .unwrap_or(false)
    }

    /// Rotate while holding the write lock: flush, shift archives down
    /// (`.N` → `.N+1`), move the active file to `.1`, and reopen a fresh one.
    fn rotate_locked(&self, writer: &mut BufWriter<File>) -> std::io::Result<()> {
        writer.flush()?;
        let base = &self.path;
        let archive = |i: u32| PathBuf::from(format!("{}.{}", base.display(), i));
        // Drop the oldest archive beyond the retention cap.
        let oldest = archive(self.max_files);
        if oldest.exists() {
            fs::remove_file(&oldest)?;
        }
        // Shift `.1`..`.N-1` up one slot.
        for i in (1..self.max_files).rev() {
            let src = archive(i);
            if src.exists() {
                fs::rename(&src, archive(i + 1))?;
            }
        }
        // Move the active file to `.1` and start a fresh one.
        fs::rename(base, archive(1))?;
        let file = OpenOptions::new().create(true).append(true).open(base)?;
        *writer = BufWriter::new(file);
        Ok(())
    }

    /// Whether this logger is active. Always `true` once constructed — the
    /// opt-in/out is expressed by `Option<AuditLogger>` at the SDK layer.
    pub fn is_enabled(&self) -> bool {
        true
    }
}

/// Current UTC time as an ISO 8601 string (`YYYY-MM-DDTHH:MM:SSZ`).
pub fn now_iso() -> String {
    let secs = web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    chrono::DateTime::from_timestamp(secs, 0)
        .map(|dt| dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_event_op_names() {
        assert_eq!(
            AuditEvent::memory("l1", "ns", "k", "ok", None).op,
            "memory_l1"
        );
        assert_eq!(
            AuditEvent::memory("l2", "ns", "k", "ok", None).op,
            "memory_l2"
        );
        assert_eq!(
            AuditEvent::memory("l3", "ns", "k", "ok", None).op,
            "memory_l3"
        );
        assert_eq!(
            AuditEvent::memory("recall", "ns", "k", "ok", None).op,
            "memory_recall"
        );
        assert_eq!(
            AuditEvent::memory("offload", "ns", "k", "ok", None).op,
            "memory_offload"
        );
        // Unknown layers stay error-silent and appendable.
        assert_eq!(
            AuditEvent::memory("bogus", "ns", "k", "ok", None).op,
            "memory_bogus"
        );
    }

    #[test]
    fn test_memory_event_jsonl_roundtrip() {
        let event = AuditEvent::memory("l3", "persona", "alice", "ok", Some("drift=0.25".into()));
        let json = serde_json::to_string(&event).unwrap();
        let back: AuditEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(back.op, "memory_l3");
        assert_eq!(back.namespace, "persona");
        assert_eq!(back.key, "alice");
        assert_eq!(back.outcome, "ok");
        assert_eq!(back.reason.as_deref(), Some("drift=0.25"));
        assert!(!back.timestamp.is_empty());
    }

    #[test]
    fn test_auth_event_op_names() {
        assert_eq!(
            AuditEvent::auth("l1", "auth", "N/A", "err", None).op,
            "auth_l1"
        );
        assert_eq!(
            AuditEvent::auth("l2", "auth", "svc-1", "ok", None).op,
            "auth_l2"
        );
        assert_eq!(
            AuditEvent::auth("l3", "auth", "usr-1", "ok", None).op,
            "auth_l3"
        );
        // Unknown layers stay error-silent and appendable.
        assert_eq!(
            AuditEvent::auth("bogus", "auth", "k", "ok", None).op,
            "auth_bogus"
        );
    }

    #[test]
    fn test_auth_event_jsonl_roundtrip() {
        let event = AuditEvent::auth(
            "l3",
            "auth",
            "usr-1",
            "err",
            Some("invalid_user_key".into()),
        );
        let json = serde_json::to_string(&event).unwrap();
        let back: AuditEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(back.op, "auth_l3");
        assert_eq!(back.namespace, "auth");
        assert_eq!(back.key, "usr-1");
        assert_eq!(back.outcome, "err");
        assert_eq!(back.reason.as_deref(), Some("invalid_user_key"));
        assert!(!back.timestamp.is_empty());
    }

    #[test]
    fn test_request_id_roundtrip_and_default_absent() {
        // With request_id attached → serialized and deserialized.
        let event = AuditEvent::new("put", "ns", "k", "ok", None).with_request_id("abc-123");
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"request_id\":\"abc-123\""));
        let back: AuditEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(back.request_id.as_deref(), Some("abc-123"));

        // Without request_id → field omitted from JSON, `None` on read
        // (backwards-compatible with pre-SRV-02 logs).
        let plain = AuditEvent::new("put", "ns", "k", "ok", None);
        let plain_json = serde_json::to_string(&plain).unwrap();
        assert!(!plain_json.contains("request_id"));
        let back: AuditEvent = serde_json::from_str(&plain_json).unwrap();
        assert_eq!(back.request_id, None);
    }

    #[test]
    fn test_rotation_creates_archives_and_respects_cap() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("audit.jsonl");
        // Tiny threshold + cap of 2 archives → rapid rotation, bounded history.
        let logger = AuditLogger::with_rotation(&path, 120, 2).unwrap();
        let event = AuditEvent::new("put", "ns", "k", "ok", None);
        for _ in 0..10 {
            logger.record(&event).unwrap();
        }
        assert!(
            dir.path().join("audit.jsonl.1").exists(),
            "active file must have rotated to .1"
        );
        assert!(
            !dir.path().join("audit.jsonl.3").exists(),
            "archives beyond the cap must be deleted"
        );
        // The active file keeps receiving events after rotation.
        let active = std::fs::read_to_string(&path).unwrap();
        assert!(active.lines().count() > 0, "active file must not be empty");
    }

    #[test]
    fn test_rotation_never_torn_lines() {
        // With a generous retention cap no event is ever lost: every line of
        // every retained file parses as a complete JSONL event (rotation
        // renames whole files — a torn/partial line must never appear).
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("audit.jsonl");
        let logger = AuditLogger::with_rotation(&path, 120, 5).unwrap();
        let event = AuditEvent::new("put", "ns", "k", "ok", None);
        for _ in 0..10 {
            logger.record(&event).unwrap();
        }
        let mut total = 0;
        for i in 0..=5u32 {
            let p = if i == 0 {
                path.clone()
            } else {
                dir.path().join(format!("audit.jsonl.{i}"))
            };
            if p.exists() {
                for line in std::fs::read_to_string(&p).unwrap().lines() {
                    serde_json::from_str::<AuditEvent>(line).unwrap();
                    total += 1;
                }
            }
        }
        assert_eq!(
            total, 10,
            "no event may be lost while under the retention cap"
        );
    }
}
