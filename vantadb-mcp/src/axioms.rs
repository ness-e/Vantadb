//! Axiom storage keys and resolution.
//!
//! VantaDB has no axiom-write API in core (MCP-33): `src/agentic/` has no
//! axioms module. Agent-managed axioms are therefore stored as records in the
//! reserved `_axioms` namespace — one record per axiom, `key` = axiom name,
//! `payload` = JSON `{"id", "name", "description"}`. The Iron Axioms are
//! hardcoded below, are never written, and are always merged into the result.

use serde_json::{json, Value};
use std::sync::Arc;
use vantadb::sdk::VantaMemoryListOptions;
use vantadb::storage::StorageEngine;

/// Reserved namespace for agent-managed axioms (MCP-33 convention).
pub(crate) const AXIOMS_NAMESPACE: &str = "_axioms";

/// Cap on how many agent axioms `resolve_axioms` enumerates. Axioms are rule
/// metadata — a handful at most; the cap guards an accidentally polluted
/// namespace without materializing an unbounded list.
pub(crate) const MAX_AXIOM_RECORDS: usize = 10_000;

/// Hardcoded Iron Axioms definition (Devil's Advocate rules) — always the
/// base of [`resolve_axioms`]. Never stored, never modified by the agent.
pub(crate) const HARDCODED_AXIOMS: &str = r#"[
    {"id":1,"name":"Topological Axiom","description":"References (edges) to orphan nodes or nodes in Tombstone storage are not allowed."},
    {"id":2,"name":"Confidence Constraint","description":"Divergent vector mutations with high historical Confidence Score are rejected."},
    {"id":3,"name":"Immortal Axiom","description":"Maintenance: Nodes marked as PINNED evade degradation by Data Decay."},
    {"id":4,"name":"Resource Allocation","description":"Maintenance: 5% of memory reserved for nodes with semantic priority >= 0.8."}
]"#;

/// Resolve active axioms: the hardcoded Iron Axioms (always present,
/// read-only) merged with agent axioms stored as records in the reserved
/// `_axioms` namespace, sorted by id. Records with an unparseable payload are
/// skipped rather than failing the whole read.
pub(crate) fn resolve_axioms(storage: &Arc<StorageEngine>) -> Value {
    let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
    let mut axioms: Vec<Value> =
        serde_json::from_str(HARDCODED_AXIOMS).unwrap_or_else(|_| Vec::new());
    let options = VantaMemoryListOptions {
        limit: MAX_AXIOM_RECORDS,
        cursor: None,
        #[allow(deprecated)]
        filters: vantadb::sdk::VantaMemoryMetadata::new(),
        filter_ops: None,
        exclude_superseded: false,
    };
    if let Ok(page) = embedded.list(AXIOMS_NAMESPACE, options) {
        for record in page.records {
            if let Ok(v) = serde_json::from_str::<Value>(&record.payload) {
                axioms.push(v);
            }
        }
    }
    axioms.sort_by_key(|a| a["id"].as_u64().unwrap_or(0));
    json!(axioms)
}
