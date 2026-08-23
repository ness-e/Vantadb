//! Physical scan operator: iterates all nodes of a given entity type.
//!
//! Split out of the monolithic `physical_plan` module (REVIEW-05).

use crate::error::Result;
use crate::node::UnifiedNode;
use crate::query::PhysicalOperator;
use crate::storage::StorageEngine;

// ─── Physical Scan Operator ──────────────────────────────────

/// Physical scan operator that iterates all nodes of a given entity type.
pub struct PhysicalScan<'a> {
    /// Storage engine reference.
    storage: &'a StorageEngine,
    /// Entity type to scan.
    entity: String,
    /// Pre-fetched nodes.
    prefetched: Vec<UnifiedNode>,
    /// Current position in the prefetched list.
    cursor: usize,
}

impl<'a> PhysicalScan<'a> {
    /// Create a new scan operator for the given entity.
    pub fn new(storage: &'a StorageEngine, entity: String) -> Self {
        Self {
            storage,
            entity,
            prefetched: Vec::new(),
            cursor: 0,
        }
    }
}

impl PhysicalOperator for PhysicalScan<'_> {
    fn open(&mut self) -> Result<()> {
        self.prefetched.clear();
        self.cursor = 0;

        let parts: Vec<&str> = self.entity.split('#').collect();
        if parts.len() == 2 {
            if let Ok(id) = parts[1].parse::<u128>() {
                self.prefetched = self.storage.get_many(&[id])?;
                return Ok(());
            }
        }

        let records = self
            .storage
            .backend
            .scan(crate::backend::BackendPartition::Default)?;
        let ids: Vec<u128> = records
            .iter()
            .filter_map(|(key_bytes, _)| {
                let arr: [u8; 16] = key_bytes.as_slice().try_into().ok()?;
                Some(u128::from_le_bytes(arr))
            })
            .collect();
        self.prefetched = self.storage.get_many(&ids)?;
        Ok(())
    }

    fn next(&mut self) -> Result<Option<UnifiedNode>> {
        while self.cursor < self.prefetched.len() {
            let node = &self.prefetched[self.cursor];
            self.cursor += 1;

            if self.storage.is_deleted(node.id)? {
                continue;
            }

            if self.entity.contains('#') || self.entity == "*" {
                return Ok(Some(node.clone()));
            }
            if let Some(crate::node::FieldValue::String(t)) = node.relational.get("type") {
                if t == &self.entity {
                    return Ok(Some(node.clone()));
                }
            }
            // MCP-29: memory records carry no `type` field; each namespace is
            // exposed as an IQL table named by its sanitized form
            // (`iql_table_name_for_namespace`) so `SELECT * FROM <ns>` reaches
            // them — legacy records included, no migration needed.
            if let Some(crate::node::FieldValue::String(ns)) = node
                .relational
                .get(crate::sdk::serialization::FIELD_NAMESPACE)
            {
                if crate::sdk::serialization::iql_table_name_for_namespace(ns) == self.entity {
                    return Ok(Some(node.clone()));
                }
            }
        }
        Ok(None)
    }

    fn close(&mut self) -> Result<()> {
        self.prefetched.clear();
        Ok(())
    }
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use crate::config::VantaConfig;
    use crate::executor::Executor;
    use crate::node::FieldValue;
    use crate::sdk::serialization::iql_table_name_for_namespace;
    use crate::storage::{BackendKind, StorageEngine};

    fn in_memory_engine() -> StorageEngine {
        let config = VantaConfig {
            backend_kind: BackendKind::InMemory,
            read_only: false,
            ..VantaConfig::default()
        };
        StorageEngine::open_with_config(":memory:", Some(config)).expect("open in-memory engine")
    }

    /// Node with the exact relational shape `memory_record_to_node_owned`
    /// produces (reserved `__vanta_*` fields, no `type`).
    fn memory_shaped_node(id: u128, namespace: &str, key: &str) -> UnifiedNode {
        let mut node = UnifiedNode::new(id);
        node.set_field(
            crate::sdk::serialization::FIELD_NAMESPACE,
            FieldValue::String(namespace.to_string()),
        );
        node.set_field(
            crate::sdk::serialization::FIELD_KEY,
            FieldValue::String(key.to_string()),
        );
        node.set_field(
            crate::sdk::serialization::FIELD_PAYLOAD,
            FieldValue::String("payload".to_string()),
        );
        node
    }

    fn select_ids(storage: &StorageEngine, query: &str) -> Vec<u128> {
        let executor = Executor::new(storage);
        match executor.execute_hybrid(query).expect("execute") {
            crate::executor::ExecutionResult::Read(nodes) => nodes.iter().map(|n| n.id).collect(),
            other => panic!("expected Read result, got {other:?}"),
        }
    }

    #[test]
    fn sanitization_maps_invalid_ident_chars() {
        assert_eq!(iql_table_name_for_namespace("ProbeNs"), "ProbeNs");
        assert_eq!(
            iql_table_name_for_namespace("mmd/s1/history"),
            "mmd_s1_history"
        );
        assert_eq!(iql_table_name_for_namespace("my-ns"), "my_ns");
        // ident parser requires a leading letter or '_'
        assert_eq!(iql_table_name_for_namespace("9lives"), "_9lives");
        assert_eq!(iql_table_name_for_namespace(".hidden"), "_.hidden");
    }

    /// MCP-29 happy path + legacy policy: records written before this feature
    /// (no `type` field) are visible immediately — no migration needed.
    #[test]
    fn select_from_namespace_reaches_memory_record_without_migration() {
        let storage = in_memory_engine();
        storage
            .insert(&memory_shaped_node(1001, "ProbeNs", "k1"))
            .expect("insert");
        assert_eq!(select_ids(&storage, "SELECT * FROM ProbeNs"), vec![1001]);
    }

    #[test]
    fn namespace_with_slashes_is_queryable_via_sanitized_table() {
        let storage = in_memory_engine();
        storage
            .insert(&memory_shaped_node(2002, "mmd/s1/history", "k2"))
            .expect("insert");
        assert_eq!(
            select_ids(&storage, "SELECT * FROM mmd_s1_history"),
            vec![2002]
        );
    }

    /// Collision policy: a graph type and a namespace sanitizing to the same
    /// table name are a UNION (both returned). Documented behavior.
    #[test]
    fn collision_between_graph_type_and_namespace_returns_union() {
        let storage = in_memory_engine();
        let mut graph_node = UnifiedNode::new(3003);
        graph_node.set_field("type", FieldValue::String("Foo".to_string()));
        storage.insert(&graph_node).expect("insert graph node");
        storage
            .insert(&memory_shaped_node(3004, "Foo", "k3"))
            .expect("insert memory record");

        let mut ids = select_ids(&storage, "SELECT * FROM Foo");
        ids.sort_unstable();
        assert_eq!(ids, vec![3003, 3004]);
    }

    #[test]
    fn unknown_table_still_returns_empty_without_error() {
        let storage = in_memory_engine();
        storage
            .insert(&memory_shaped_node(4005, "RealNs", "k4"))
            .expect("insert");
        assert!(select_ids(&storage, "SELECT * FROM MissingNs").is_empty());
    }
}
