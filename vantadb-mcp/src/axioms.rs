//! Axiom storage keys and resolution.

use serde_json::{json, Value};
use std::sync::Arc;
use vantadb::storage::StorageEngine;

/// Well-known namespace for system-level metadata storage.
pub(crate) const SYSTEM_NAMESPACE: &str = "_system";

/// Key under which axioms are stored in the system namespace.
pub(crate) const AXIOMS_STORAGE_KEY: &str = "axioms";

/// Hardcoded Iron Axioms definition (Devil's Advocate rules) — fallback
/// when no stored axioms exist in the metadata storage.
pub(crate) const HARDCODED_AXIOMS: &str = r#"[
    {"id":1,"name":"Topological Axiom","description":"References (edges) to orphan nodes or nodes in Tombstone storage are not allowed."},
    {"id":2,"name":"Confidence Constraint","description":"Divergent vector mutations with high historical Confidence Score are rejected."},
    {"id":3,"name":"Immortal Axiom","description":"Maintenance: Nodes marked as PINNED evade degradation by Data Decay."},
    {"id":4,"name":"Resource Allocation","description":"Maintenance: 5% of memory reserved for nodes with semantic priority >= 0.8."}
]"#;

/// Resolve active axioms, preferring stored metadata over hardcoded defaults.
pub(crate) fn resolve_axioms(storage: &Arc<StorageEngine>) -> Value {
    let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
    match embedded.get(SYSTEM_NAMESPACE, AXIOMS_STORAGE_KEY) {
        Ok(Some(record)) => serde_json::from_str(&record.payload).unwrap_or_else(|_| {
            serde_json::from_str(HARDCODED_AXIOMS).unwrap_or_else(|_| json!([]))
        }),
        _ => serde_json::from_str(HARDCODED_AXIOMS).unwrap_or_else(|_| json!([])),
    }
}
