//! Append-only JSONL audit log of business operations.
//!
//! Records *operations* (put/delete/export/import) with an ISO 8601 timestamp,
//! subject, target, and outcome — distinct from runtime `tracing` logs. Opt-in
//! via [`Config::audit_log_path`](crate::config::Config::audit_log_path);
//! when unset, `Embedded` operations skip audit entirely.

use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Default audit rotation threshold: rotate the JSONL once it reaches 10 MiB.
pub const DEFAULT_AUDIT_MAX_BYTES: u64 = 10 * 1024 * 1024;
/// Default rotated-file retention: keep `.1`..`.5`, delete older archives.
pub const DEFAULT_AUDIT_MAX_FILES: u32 = 5;

/// Genesis hash for the initial entry in the Merkle trail (64 hex zeroes).
pub const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// A single audit record: timestamp + operation + subject + target + outcome + Merkle trail (Art. 12).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// ISO 8601 UTC timestamp (e.g. `2026-08-02T12:34:56Z`).
    pub timestamp: String,
    /// Unix epoch millisecond timestamp (Art. 12).
    #[serde(default = "now_ms_audit")]
    pub timestamp_ms: u64,
    /// Operation name: `put`, `delete`, `delete_by_filter`, `put_batch`,
    /// `export_namespace`, `export_all`, `import_file`, `search`, etc.
    pub op: String,
    /// Explicit/aliased operation field (Art. 12 parity with op).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
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

    // --- EU AI Act Art. 12 Merkle Trail fields ---
    /// SHA-256 hash of the query text or payload for search/recall/query operations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query_hash: Option<String>,
    /// Retrieved IDs and scores: `[(id, score)]`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retrieved_ids_and_scores: Vec<(String, f32)>,
    /// IDs omitted because they were superseded/invalidated.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub omitted_superseded_ids: Vec<String>,
    /// Hash of the previous audit entry in the log (Merkle trail).
    #[serde(default)]
    pub prev_entry_hash: String,
    /// SHA-256(prev_entry_hash || canonical_json) of this entry.
    #[serde(default)]
    pub entry_hash: String,
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
            timestamp_ms: now_ms_audit(),
            op: op.to_string(),
            operation: Some(op.to_string()),
            namespace: namespace.to_string(),
            key: key.to_string(),
            outcome: outcome.to_string(),
            reason,
            request_id: None,
            query_hash: None,
            retrieved_ids_and_scores: Vec::new(),
            omitted_superseded_ids: Vec::new(),
            prev_entry_hash: String::new(),
            entry_hash: String::new(),
        }
    }

    /// Build a search audit event documenting retrieved and omitted superseded IDs (Art. 12).
    pub fn search(
        namespace: &str,
        query: &str,
        retrieved: Vec<(String, f32)>,
        omitted_superseded: Vec<String>,
    ) -> Self {
        use sha2::Digest;
        let query_hash = format!("{:x}", sha2::Sha256::digest(query.as_bytes()));
        let mut evt = Self::new("search", namespace, "N/A", "ok", None);
        evt.query_hash = Some(query_hash);
        evt.retrieved_ids_and_scores = retrieved;
        evt.omitted_superseded_ids = omitted_superseded;
        evt
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
    /// (user-key → user identity). MEMG-10 adds the `rbac` layer
    /// (`auth_rbac`) for authorization denials: subject = role, namespace =
    /// the denied namespace (or `"N/A"`), reason = `action`/`enforced`/`scope`
    /// — never the token.
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

    /// Build an injection-governance event (`injection` op) — VER-04: which
    /// memory entered which prompt, under which budget and ACL decision.
    ///
    /// - `namespace`: source namespace of the injected memory (e.g.
    ///   `l1/<session>`, `persona/<session>`), or the session for a
    ///   block-level record.
    /// - `key`: source record key (or `"N/A"` for aggregate records).
    /// - `outcome`: `"ok"` when injected, `"denied"` when skipped by the ACL.
    /// - `reason`: compact `k=v;` metadata — `surface`, `tool`, `session`,
    ///   `kind`, `score`, `budget`, `truncated`, `acl`. **Never** payload
    ///   values (metadata-only by contract; VER-03 kinds precedent).
    ///
    /// WORM-ready: append-only JSONL via [`AuditLogger`] (rotation included);
    /// the VER-01 WAL hash-chain is the tamper-evidence consumer of memory
    /// transitions — cited, not duplicated here.
    pub fn injection(namespace: &str, key: &str, outcome: &str, reason: Option<String>) -> Self {
        Self::new("injection", namespace, key, outcome, reason)
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
    last_entry_hash: Mutex<String>,
}

fn read_last_entry_hash(path: &Path) -> String {
    if let Ok(content) = fs::read_to_string(path) {
        for line in content.lines().rev() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
                    if let Some(h) = val.get("entry_hash").and_then(|v| v.as_str()) {
                        if !h.is_empty() {
                            return h.to_string();
                        }
                    }
                }
            }
        }
    }
    GENESIS_HASH.to_string()
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
        let initial_hash = read_last_entry_hash(path);
        Ok(Self {
            writer: Mutex::new(BufWriter::new(file)),
            path: path.to_path_buf(),
            max_bytes: max_bytes.max(1),
            max_files: max_files.max(1),
            last_entry_hash: Mutex::new(initial_hash),
        })
    }

    /// Append one event as a JSON line and flush (best-effort durability).
    pub fn record(&self, event: &AuditEvent) -> crate::error::Result<()> {
        let mut guard = self.writer.lock().unwrap_or_else(|e| e.into_inner());
        let mut hash_guard = self
            .last_entry_hash
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if self.should_rotate(&guard) {
            self.rotate_locked(&mut guard)?;
            *hash_guard = GENESIS_HASH.to_string();
        }

        let mut entry = event.clone();
        if entry.timestamp_ms == 0 {
            entry.timestamp_ms = now_ms_audit();
        }
        if entry.operation.is_none() {
            entry.operation = Some(entry.op.clone());
        }

        entry.prev_entry_hash = hash_guard.clone();
        entry.entry_hash = String::new();

        // Canonical JSON serialization without entry_hash
        let canonical_bytes =
            serde_json::to_vec(&entry).map_err(crate::error::Error::serialization)?;
        use sha2::Digest;
        let mut hasher = sha2::Sha256::new();
        hasher.update(entry.prev_entry_hash.as_bytes());
        hasher.update(&canonical_bytes);
        let digest = format!("{:x}", hasher.finalize());
        entry.entry_hash = digest.clone();
        *hash_guard = digest;

        serde_json::to_writer(&mut *guard, &entry).map_err(crate::error::Error::serialization)?;
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

/// Report returned by [`verify_merkle_trail`] on successful verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditVerifyReport {
    /// Total audit records verified.
    pub events_verified: usize,
    /// Head (most recent) hash in the chain.
    pub head_hash: String,
}

/// Error returned by [`verify_merkle_trail`] when the chain is broken or a line is corrupted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditVerifyError {
    /// 1-based line number of the faulty record.
    pub line_number: usize,
    /// Failure description.
    pub reason: String,
    /// Expected hash value.
    pub expected_hash: String,
    /// Actual/recorded hash value found in the record.
    pub actual_hash: String,
}

impl std::fmt::Display for AuditVerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Merkle trail verification failed at line {}: {} (expected {}, found {})",
            self.line_number, self.reason, self.expected_hash, self.actual_hash
        )
    }
}

impl std::error::Error for AuditVerifyError {}

/// Verify the entire SHA-256 Merkle chain in an `audit.jsonl` file (Art. 12 / TEC-13).
pub fn verify_merkle_trail(
    path: impl AsRef<Path>,
) -> std::result::Result<AuditVerifyReport, AuditVerifyError> {
    let path = path.as_ref();
    let file = File::open(path).map_err(|e| AuditVerifyError {
        line_number: 0,
        reason: format!("failed to open audit file: {e}"),
        expected_hash: String::new(),
        actual_hash: String::new(),
    })?;
    use std::io::{BufRead, BufReader};
    let reader = BufReader::new(file);

    let mut prev_hash = GENESIS_HASH.to_string();
    let mut verified_count = 0;

    for (idx, line_res) in reader.lines().enumerate() {
        let line_num = idx + 1;
        let line = line_res.map_err(|e| AuditVerifyError {
            line_number: line_num,
            reason: format!("I/O read error: {e}"),
            expected_hash: String::new(),
            actual_hash: String::new(),
        })?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let entry: AuditEvent = serde_json::from_str(trimmed).map_err(|e| AuditVerifyError {
            line_number: line_num,
            reason: format!("malformed JSON: {e}"),
            expected_hash: String::new(),
            actual_hash: String::new(),
        })?;

        // 1. Verify prev_entry_hash matches previous head
        if entry.prev_entry_hash != prev_hash {
            return Err(AuditVerifyError {
                line_number: line_num,
                reason: "prev_entry_hash broken chain".into(),
                expected_hash: prev_hash,
                actual_hash: entry.prev_entry_hash,
            });
        }

        // 2. Recompute entry_hash = SHA256(prev_entry_hash || canonical_json)
        let recorded_entry_hash = entry.entry_hash.clone();
        let mut test_entry = entry;
        test_entry.entry_hash = String::new();
        let canonical_bytes = serde_json::to_vec(&test_entry).map_err(|e| AuditVerifyError {
            line_number: line_num,
            reason: format!("canonical serialization error: {e}"),
            expected_hash: String::new(),
            actual_hash: String::new(),
        })?;

        use sha2::Digest;
        let mut hasher = sha2::Sha256::new();
        hasher.update(prev_hash.as_bytes());
        hasher.update(&canonical_bytes);
        let computed_hash = format!("{:x}", hasher.finalize());

        if computed_hash != recorded_entry_hash {
            return Err(AuditVerifyError {
                line_number: line_num,
                reason: "entry_hash tampering detected".into(),
                expected_hash: computed_hash,
                actual_hash: recorded_entry_hash,
            });
        }

        prev_hash = recorded_entry_hash;
        verified_count += 1;
    }

    Ok(AuditVerifyReport {
        events_verified: verified_count,
        head_hash: prev_hash,
    })
}

/// Current UTC epoch milliseconds.
pub fn now_ms_audit() -> u64 {
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
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
    fn test_injection_event_is_metadata_only() {
        let event = AuditEvent::injection(
            "l1/sess-1",
            "m1",
            "ok",
            Some("surface=proxy;kind=l1;score=3;budget=120/2000;truncated=false;acl=allow".into()),
        );
        let json = serde_json::to_string(&event).unwrap();
        let back: AuditEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(back.op, "injection");
        assert_eq!(back.namespace, "l1/sess-1");
        assert_eq!(back.key, "m1");
        assert_eq!(back.outcome, "ok");
        assert!(back
            .reason
            .as_deref()
            .unwrap_or_default()
            .contains("acl=allow"));
        // ACL denials are first-class outcomes (never a silently dropped row).
        assert_eq!(
            AuditEvent::injection("l1/x", "N/A", "denied", None).outcome,
            "denied"
        );
        assert!(!back.timestamp.is_empty());
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
        // Threshold sized for the Merkle-chained line (two 64-hex hashes make a
        // line ~350 bytes): ~2 events per file keeps 10 events within 5 archives.
        let logger = AuditLogger::with_rotation(&path, 600, 5).unwrap();
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

    #[test]
    fn test_merkle_trail_verification_and_tamper_detection() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("audit.jsonl");
        let logger = AuditLogger::new(&path).unwrap();

        // 1. Write 100 events
        for i in 0..100 {
            let evt = AuditEvent::new(
                "put",
                "default",
                &format!("key_{i}"),
                "ok",
                Some(format!("event {i}")),
            );
            logger.record(&evt).unwrap();
        }

        // 2. Verification of intact file succeeds
        let report = verify_merkle_trail(&path).expect("intact file must verify");
        assert_eq!(report.events_verified, 100);
        assert!(!report.head_hash.is_empty());

        // 3. Altering a single byte in line 42 fails verification
        let content = std::fs::read_to_string(&path).unwrap();
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        assert_eq!(lines.len(), 100);

        // Tamper with line 42 (change "key_41" to "key_4X")
        let tampered_line = lines[41].replace("key_41", "key_4X");
        assert_ne!(tampered_line, lines[41]);
        lines[41] = tampered_line;

        let tampered_path = dir.path().join("tampered_audit.jsonl");
        std::fs::write(&tampered_path, lines.join("\n") + "\n").unwrap();

        let err =
            verify_merkle_trail(&tampered_path).expect_err("tampered file must fail verification");
        assert_eq!(err.line_number, 42);
        assert!(err.reason.contains("tampering"));
    }
}
