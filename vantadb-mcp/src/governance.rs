//! VER-04: injection governance for the MCP memory surfaces — shared audit
//! emission for `memory_recall`, `context_assemble` and `inject_context`.
//!
//! Events are metadata-only (`ns`/`key`/`score`/`budget`/`acl`) — never
//! payload values. The sink is opt-in (`VANTADB_MCP_AUDIT_LOG`) and the JSONL
//! is append-only + rotated by the core `AuditLogger` (WORM-ready; the VER-01
//! WAL hash-chain is cited, not duplicated here).

use std::path::PathBuf;
use std::sync::Arc;

use vanta_memory::core::hooks::RecallResult;

use crate::config::McpConfig;

/// Rotation defaults for the MCP injection audit (mirror the core audit).
const AUDIT_MAX_BYTES: u64 = 10 * 1024 * 1024;
const AUDIT_MAX_FILES: u32 = 5;

/// Open the injection-audit JSONL when `path` is configured. A failed open is
/// logged and treated as disabled — audit never blocks the server.
pub(crate) fn open_audit(path: &str) -> Option<Arc<vantadb::audit::AuditLogger>> {
    let path = path.trim();
    if path.is_empty() {
        return None;
    }
    match vantadb::audit::AuditLogger::with_rotation(
        PathBuf::from(path),
        AUDIT_MAX_BYTES,
        AUDIT_MAX_FILES,
    ) {
        Ok(logger) => Some(Arc::new(logger)),
        Err(e) => {
            tracing::warn!(
                path,
                error = %e,
                "injection audit disabled: cannot open VANTADB_MCP_AUDIT_LOG"
            );
            None
        }
    }
}

fn record(logger: &vantadb::audit::AuditLogger, event: vantadb::audit::AuditEvent) {
    if let Err(e) = logger.record(&event) {
        tracing::warn!(op = %event.op, error = %e, "injection audit record failed");
    }
}

/// One event per recalled memory (source ns/key + score) plus one per ACL
/// denial. `budget` is the byte budget applied to the recall response.
pub(crate) fn audit_recall(
    config: &McpConfig,
    tool: &str,
    session: &str,
    result: &RecallResult,
    budget: usize,
) {
    let Some(logger) = &config.audit else {
        return;
    };
    for memory in &result.recalled_memories {
        let namespace = if memory.source_namespace.is_empty() {
            "l1/unknown"
        } else {
            memory.source_namespace.as_str()
        };
        let key = if memory.source_key.is_empty() {
            "N/A"
        } else {
            memory.source_key.as_str()
        };
        let reason = format!(
            "surface=mcp;tool={tool};session={session};kind=l1;score={};budget={budget};acl=allow",
            memory.score
        );
        record(
            logger,
            vantadb::audit::AuditEvent::injection(namespace, key, "ok", Some(reason)),
        );
    }
    for namespace in &result.governance.denied {
        let reason = format!(
            "surface=mcp;tool={tool};session={session};kind=recall;budget={budget};acl=deny"
        );
        record(
            logger,
            vantadb::audit::AuditEvent::injection(namespace, "N/A", "denied", Some(reason)),
        );
    }
}

/// One event per `inject_context` call: which thread received how many bytes
/// under which budget (metadata only — the content never lands in the audit).
pub(crate) fn audit_inject_context(
    config: &McpConfig,
    thread_id: u128,
    bytes: usize,
    budget: usize,
) {
    let Some(logger) = &config.audit else {
        return;
    };
    let reason = format!(
        "surface=mcp;tool=inject_context;session=thread:{thread_id};kind=context;budget={bytes}/{budget};truncated=false;acl=allow"
    );
    record(
        logger,
        vantadb::audit::AuditEvent::injection(
            "threads",
            &thread_id.to_string(),
            "ok",
            Some(reason),
        ),
    );
}
