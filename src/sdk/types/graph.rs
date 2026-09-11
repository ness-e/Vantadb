//! Graph-domain SDK types: node/edge records and query results.
//!
//! Pure move of the graph items from `super` (FIND-49). Public paths
//! `crate::sdk::types::X` are preserved via re-exports in `super`.
//! Definitions of the node/edge records stay in
//! `crate::sdk::serialization::graph_types` (used by `storage::engine`).

pub use super::super::serialization::graph_types::{EdgeRecord, NodeInput, NodeRecord};
#[allow(deprecated)]
pub use super::super::serialization::graph_types::{
    VantaEdgeRecord, VantaNodeInput, VantaNodeRecord,
};
use super::u128_serde;
use serde::{Deserialize, Serialize};

/// Stable query result enum for external SDKs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QueryResult {
    /// Query returned a set of matching nodes.
    Read(Vec<NodeRecord>),
    /// Query performed a write operation.
    Write {
        /// Number of nodes affected by the write.
        affected_nodes: usize,
        /// Human-readable result message.
        message: String,
        /// Node id returned by the write, if applicable.
        node_id: Option<u128>,
    },
    /// Query detected stale context for the given node.
    StaleContext {
        /// Node id with stale context.
        #[serde(with = "u128_serde")]
        node_id: u128,
    },
}

/// Deprecated `Vanta`-prefixed aliases (AST-002, ADR-041).
/// New code must use the unprefixed names; these exist only for semver migration.
#[deprecated(
    since = "0.5.0",
    note = "Use `QueryResult` instead - the `Vanta` prefix was removed (ADR-041). Will be removed in a future release."
)]
pub type VantaQueryResult = QueryResult;

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;

    // ΓöÇΓöÇ QueryResult ΓöÇΓöÇ

    #[test]
    fn test_query_result_read() {
        let result = QueryResult::Read(vec![]);
        match result {
            QueryResult::Read(nodes) => assert!(nodes.is_empty()),
            _ => panic!("expected Read"),
        }
    }

    #[test]
    fn test_query_result_write() {
        let result = QueryResult::Write {
            affected_nodes: 1,
            message: "created".into(),
            node_id: Some(42),
        };
        match result {
            QueryResult::Write {
                affected_nodes,
                message,
                node_id,
            } => {
                assert_eq!(affected_nodes, 1);
                assert_eq!(message, "created");
                assert_eq!(node_id, Some(42));
            }
            _ => panic!("expected Write"),
        }
    }

    #[test]
    fn test_query_result_stale_context() {
        let result = QueryResult::StaleContext { node_id: 99 };
        match result {
            QueryResult::StaleContext { node_id } => {
                assert_eq!(node_id, 99);
            }
            _ => panic!("expected StaleContext"),
        }
    }

    // ΓöÇΓöÇ QueryResult clone/debug ΓöÇΓöÇ

    #[test]
    fn test_query_result_clone_read() {
        let r = QueryResult::Read(vec![]);
        let cloned = r.clone();
        assert_eq!(r, cloned);
    }

    #[test]
    fn test_query_result_clone_write() {
        let r = QueryResult::Write {
            affected_nodes: 3,
            message: "done".into(),
            node_id: None,
        };
        let cloned = r.clone();
        assert_eq!(r, cloned);
    }

    #[test]
    fn test_query_result_debug() {
        let r = QueryResult::Write {
            affected_nodes: 1,
            message: "ok".into(),
            node_id: Some(7),
        };
        let dbg = format!("{:?}", r);
        assert!(dbg.contains("Write") || dbg.contains("affected_nodes"));
    }
}
