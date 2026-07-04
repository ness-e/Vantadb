//! Audit logging (JSONL format, timestamped operations)

/// An audited operation event.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditEvent {
    /// Unix timestamp of the event.
    pub timestamp: u64,
    /// Name of the operation performed.
    pub operation: String,
    /// Namespace the operation targeted.
    pub namespace: String,
    /// User who performed the operation.
    pub user: String,
    /// Additional event metadata as JSON.
    pub details: serde_json::Value,
}

/// Audit log writer that appends to a JSONL file.
pub struct AuditLogger {
    // TODO: file handle, rotation, retention policy
}

impl AuditLogger {
    /// Create a new `AuditLogger` writing to the given log path.
    pub fn new(_log_path: &std::path::Path) -> Self {
        Self {}
    }

    /// Append an audit event to the log file.
    pub fn log(&self, _event: AuditEvent) -> std::io::Result<()> {
        // TODO: append JSON line to log file
        Ok(())
    }

    /// Query the audit log with the given filter.
    pub fn query(&self, _filter: AuditFilter) -> Vec<AuditEvent> {
        // TODO: search audit log by time range, user, operation
        Vec::new()
    }
}

/// Filter for querying audit events.
pub struct AuditFilter {
    /// Inclusive start time (Unix timestamp).
    pub start_time: Option<u64>,
    /// Inclusive end time (Unix timestamp).
    pub end_time: Option<u64>,
    /// Optional user name to filter by.
    pub user: Option<String>,
    /// Optional operation name to filter by.
    pub operation: Option<String>,
}
