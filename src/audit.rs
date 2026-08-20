//! Append-only JSONL audit log of business operations.
//!
//! Records *operations* (put/delete/export/import) with an ISO 8601 timestamp,
//! subject, target, and outcome — distinct from runtime `tracing` logs. Opt-in
//! via [`VantaConfig::audit_log_path`](crate::config::VantaConfig::audit_log_path);
//! when unset, `VantaEmbedded` operations skip audit entirely.

use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::Mutex;

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
        }
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
#[derive(Debug)]
pub struct AuditLogger {
    writer: Mutex<BufWriter<File>>,
}

impl AuditLogger {
    /// Open (creating if needed) the audit file, creating parent directories.
    pub fn new(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Self {
            writer: Mutex::new(BufWriter::new(file)),
        })
    }

    /// Append one event as a JSON line and flush (best-effort durability).
    pub fn record(&self, event: &AuditEvent) -> crate::error::Result<()> {
        let mut guard = self.writer.lock().unwrap_or_else(|e| e.into_inner());
        serde_json::to_writer(&mut *guard, event)
            .map_err(crate::error::VantaError::serialization)?;
        guard.write_all(b"\n")?;
        guard.flush()?;
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
}
