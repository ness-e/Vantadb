//! Core in-memory engine driving VantaDB's query lifecycle.
//!
//! Owns the [`RwLock`]-guarded node map, handles insert/update/delete/relate
//! operations, and coordinates with the WAL, GC, and storage layers.

use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use parking_lot::{Mutex, RwLock};

const DEFAULT_INITIAL_CAPACITY: usize = 1024;

use crate::error::{Result, VantaError};
use crate::node::{FieldValue, FilterBitset, UnifiedNode, VectorRepresentations};
use crate::wal::{WalReader, WalRecord, WalWriter};

// ─── Query Result ──────────────────────────────────────────

/// How the result was produced.
#[derive(Debug, Clone, PartialEq)]
pub enum SourceType {
    /// Full scan of all nodes.
    FullScan,
    /// Filtered by bitset mask.
    BitsetFilter,
    /// Vector similarity search.
    VectorSearch,
    /// Graph traversal (BFS/DFS).
    GraphTraversal,
    /// Hybrid (vector + relational filter).
    Hybrid,
}

/// Query result with exhaustivity metadata.
#[derive(Debug, Clone)]
pub struct QueryResult {
    /// Resulting nodes.
    pub nodes: Vec<UnifiedNode>,
    /// `true` if resource limits truncated results.
    pub is_partial: bool,
    /// Search completeness (0.0–1.0).
    pub exhaustivity: f32,
    /// Which index or scan strategy produced the result.
    pub source_type: SourceType,
}

/// Engine statistics snapshot.
#[derive(Debug, Clone, Default)]
pub struct EngineStats {
    /// Number of alive nodes.
    pub node_count: u64,
    /// Total edge count.
    pub edge_count: u64,
    /// Number of nodes with vectors.
    pub vector_count: u64,
    /// Sum of vector dimensions across all vector nodes.
    pub total_dimensions: u64,
    /// Estimated heap memory usage in bytes.
    pub memory_estimate_bytes: u64,
}

// ─── In-Memory Engine ──────────────────────────────────────

/// Fase 1 storage engine: HashMap + optional WAL.
///
/// Thread-safe: RwLock for reads, Mutex for WAL writes.
/// Fase 2: Replace HashMap with RocksDB-backed MemTable.
pub struct InMemoryEngine {
    /// RwLock-guarded in-memory node map.
    nodes: RwLock<HashMap<u64, UnifiedNode>>,
    /// Optional WAL writer for durability.
    wal: Mutex<Option<WalWriter>>,
    /// Monotonic ID generator.
    next_id: AtomicU64,
}

impl InMemoryEngine {
    /// Create engine (in-memory only, no persistence)
    pub fn new() -> Self {
        Self {
            nodes: RwLock::new(HashMap::with_capacity(DEFAULT_INITIAL_CAPACITY)),
            wal: Mutex::new(None),
            next_id: AtomicU64::new(1),
        }
    }

    /// Create engine with WAL durability. Replays existing WAL on open.
    pub fn with_wal(wal_path: impl AsRef<Path>) -> Result<Self> {
        let path = wal_path.as_ref().to_path_buf();
        let mut nodes_map = HashMap::with_capacity(DEFAULT_INITIAL_CAPACITY);
        let mut max_id: u64 = 0;

        // Replay existing WAL
        if path.exists() {
            let mut reader = WalReader::open(&path)?;
            reader.replay_all(|record| {
                match record {
                    WalRecord::Insert(node) => {
                        max_id = max_id.max(node.id);
                        nodes_map.insert(node.id, node);
                    }
                    WalRecord::Update { id, node } => {
                        max_id = max_id.max(id);
                        nodes_map.insert(id, node);
                    }
                    WalRecord::Delete { id } => {
                        nodes_map.remove(&id);
                    }
                    WalRecord::Checkpoint { .. } => {}
                }
                Ok(())
            })?;
        }

        let writer = WalWriter::open(&path, crate::config::SyncMode::Periodic)?;

        Ok(Self {
            nodes: RwLock::new(nodes_map),
            wal: Mutex::new(Some(writer)),
            next_id: AtomicU64::new(max_id + 1),
        })
    }

    /// Generate next unique node ID
    pub fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    /// Append a record to the WAL if present (no-op without WAL).
    fn append_to_wal(&self, record: &WalRecord) -> Result<()> {
        if let Some(ref mut wal) = *self.wal.lock() {
            wal.append(record)?;
        }
        Ok(())
    }

    /// Insert a node. Auto-assigns ID if node.id == 0.
    pub fn insert(&self, mut node: UnifiedNode) -> Result<u64> {
        if node.id == 0 {
            node.id = self.next_id();
        }
        let id = node.id;

        // WAL first (durability before visibility)
        self.append_to_wal(&WalRecord::Insert(node.clone()))?;

        let mut nodes = self.nodes.write();
        if nodes.contains_key(&id) {
            return Err(VantaError::DuplicateNode(id));
        }
        nodes.insert(id, node);
        Ok(id)
    }

    /// Get a node by ID (cloned)
    pub fn get(&self, id: u64) -> Option<UnifiedNode> {
        self.nodes.read().get(&id).cloned()
    }

    /// Check if node exists
    pub fn contains(&self, id: u64) -> bool {
        self.nodes.read().contains_key(&id)
    }

    /// Update existing node
    pub fn update(&self, id: u64, node: UnifiedNode) -> Result<()> {
        self.append_to_wal(&WalRecord::Update {
            id,
            node: node.clone(),
        })?;
        let mut nodes = self.nodes.write();
        if !nodes.contains_key(&id) {
            return Err(VantaError::NodeNotFound(id));
        }
        nodes.insert(id, node);
        Ok(())
    }

    /// Delete a node
    pub fn delete(&self, id: u64) -> Result<()> {
        self.append_to_wal(&WalRecord::Delete { id })?;
        let mut nodes = self.nodes.write();
        if nodes.remove(&id).is_none() {
            return Err(VantaError::NodeNotFound(id));
        }
        Ok(())
    }

    /// Scan nodes matching a bitset mask (all bits in mask must be set)
    pub fn scan_bitset(&self, mask: &FilterBitset) -> Vec<u64> {
        self.nodes
            .read()
            .values()
            .filter(|n| n.is_alive() && n.matches_mask(mask))
            .map(|n| n.id)
            .collect()
    }

    /// Brute-force vector similarity search.
    /// Fase 3: Replace with CP-Index HNSW for O(log n).
    pub fn vector_search(
        &self,
        query: &[f32],
        top_k: usize,
        min_score: f32,
        bitset_filter: Option<&FilterBitset>,
    ) -> QueryResult {
        let query_vec = VectorRepresentations::Full(query.to_vec());
        let nodes = self.nodes.read();

        let mut scored: Vec<(u64, f32)> = nodes
            .values()
            .filter(|n| {
                n.is_alive()
                    && !n.vector.is_none()
                    && bitset_filter.is_none_or(|m| n.matches_mask(m))
            })
            .filter_map(|n| {
                n.vector
                    .cosine_similarity(&query_vec)
                    .filter(|&s| s >= min_score)
                    .map(|s| (n.id, s))
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);

        let result_nodes: Vec<UnifiedNode> = scored
            .iter()
            .filter_map(|(id, _)| nodes.get(id).cloned())
            .collect();

        QueryResult {
            nodes: result_nodes,
            is_partial: false,
            exhaustivity: 1.0, // brute-force = exhaustive
            source_type: if bitset_filter.is_some() {
                SourceType::Hybrid
            } else {
                SourceType::VectorSearch
            },
        }
    }

    /// BFS graph traversal from start, following edges with matching label.
    /// Returns (node_id, depth) pairs within [min_depth, max_depth].
    pub fn traverse(
        &self,
        start: u64,
        label: &str,
        min_depth: u32,
        max_depth: u32,
    ) -> Result<Vec<(u64, u32)>> {
        let nodes = self.nodes.read();
        if !nodes.contains_key(&start) {
            return Err(VantaError::NodeNotFound(start));
        }

        let mut visited = HashMap::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((start, 0u32));
        visited.insert(start, 0u32);

        let mut results = Vec::new();

        while let Some((current_id, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }
            if let Some(node) = nodes.get(&current_id) {
                for edge in &node.edges {
                    if edge.label == label {
                        if let std::collections::hash_map::Entry::Vacant(e) =
                            visited.entry(edge.target)
                        {
                            let next_depth = depth + 1;
                            e.insert(next_depth);
                            if next_depth >= min_depth {
                                results.push((edge.target, next_depth));
                            }
                            queue.push_back((edge.target, next_depth));
                        }
                    }
                }
            }
        }
        Ok(results)
    }

    /// Filter nodes by relational field equality
    pub fn filter_field(&self, field: &str, value: &FieldValue) -> Vec<u64> {
        self.nodes
            .read()
            .values()
            .filter(|n| n.is_alive() && n.get_field(field) == Some(value))
            .map(|n| n.id)
            .collect()
    }

    /// Hybrid search: vector similarity + bitset filter + field predicates.
    /// Evaluates filters in cost order: bitset → relational → vector.
    pub fn hybrid_search(
        &self,
        query_vector: &[f32],
        top_k: usize,
        min_score: f32,
        bitset_mask: Option<&FilterBitset>,
        field_filters: &[(String, FieldValue)],
    ) -> QueryResult {
        let query_vec = VectorRepresentations::Full(query_vector.to_vec());
        let nodes = self.nodes.read();

        let mut scored: Vec<(u64, f32)> = nodes
            .values()
            .filter(|n| {
                if !n.is_alive() || n.vector.is_none() {
                    return false;
                }
                // Bitset first (cheapest: single AND)
                if let Some(mask) = bitset_mask {
                    if !n.matches_mask(mask) {
                        return false;
                    }
                }
                // Relational second
                for (field, value) in field_filters {
                    if n.get_field(field) != Some(value) {
                        return false;
                    }
                }
                true
            })
            .filter_map(|n| {
                n.vector
                    .cosine_similarity(&query_vec)
                    .filter(|&s| s >= min_score)
                    .map(|s| (n.id, s))
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);

        let result_nodes = scored
            .iter()
            .filter_map(|(id, _)| nodes.get(id).cloned())
            .collect();

        QueryResult {
            nodes: result_nodes,
            is_partial: false,
            exhaustivity: 1.0,
            source_type: SourceType::Hybrid,
        }
    }

    /// Flush WAL to disk
    pub fn flush_wal(&self) -> Result<()> {
        if let Some(ref mut wal) = *self.wal.lock() {
            wal.sync()?;
        }
        Ok(())
    }

    /// Total number of alive nodes
    pub fn node_count(&self) -> usize {
        self.nodes.read().values().filter(|n| n.is_alive()).count()
    }

    /// Get engine statistics
    pub fn stats(&self) -> EngineStats {
        let nodes = self.nodes.read();
        let mut stats = EngineStats::default();
        for node in nodes.values() {
            if !node.is_alive() {
                continue;
            }
            stats.node_count += 1;
            stats.edge_count += node.edges.len() as u64;
            if !node.vector.is_none() {
                stats.vector_count += 1;
                stats.total_dimensions += node.vector.dimensions() as u64;
            }
            stats.memory_estimate_bytes += node.memory_size() as u64;
        }
        stats
    }
}

impl Default for InMemoryEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use crate::node::{FieldValue, FilterBitset, UnifiedNode};

    fn create_node(id: u64) -> UnifiedNode {
        UnifiedNode::new(id)
    }

    fn create_vector_node(id: u64, vec: Vec<f32>) -> UnifiedNode {
        let mut node = UnifiedNode::new(id);
        node.vector = VectorRepresentations::Full(vec);
        node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
        node
    }

    // ── Construction ──

    #[test]
    fn test_new_engine_empty() {
        let engine = InMemoryEngine::new();
        assert_eq!(engine.node_count(), 0);
        assert!(engine.get(1).is_none());
    }

    #[test]
    fn test_default_equals_new() {
        let a = InMemoryEngine::new();
        let b = InMemoryEngine::default();
        assert_eq!(a.node_count(), b.node_count());
    }

    // ── next_id ──

    #[test]
    fn test_next_id_starts_at_one() {
        let engine = InMemoryEngine::new();
        assert_eq!(engine.next_id(), 1);
    }

    #[test]
    fn test_next_id_increments() {
        let engine = InMemoryEngine::new();
        assert_eq!(engine.next_id(), 1);
        assert_eq!(engine.next_id(), 2);
        assert_eq!(engine.next_id(), 3);
    }

    // ── Insert ──

    #[test]
    fn test_insert_with_explicit_id() {
        let engine = InMemoryEngine::new();
        let id = engine.insert(create_node(42)).unwrap();
        assert_eq!(id, 42);
        assert!(engine.contains(42));
    }

    #[test]
    fn test_insert_auto_assigns_id_zero() {
        let engine = InMemoryEngine::new();
        let id = engine.insert(create_node(0)).unwrap();
        assert!(id > 0);
        assert!(engine.contains(id));
    }

    #[test]
    fn test_insert_duplicate_errors() {
        let engine = InMemoryEngine::new();
        engine.insert(create_node(1)).unwrap();
        let err = engine.insert(create_node(1)).unwrap_err();
        assert!(matches!(err, VantaError::DuplicateNode(1)));
    }

    #[test]
    fn test_insert_and_get() {
        let engine = InMemoryEngine::new();
        engine.insert(create_node(7)).unwrap();
        let node = engine.get(7).unwrap();
        assert_eq!(node.id, 7);
    }

    #[test]
    fn test_insert_and_get_vector_node() {
        let engine = InMemoryEngine::new();
        let vec = vec![0.1, 0.2, 0.3];
        engine.insert(create_vector_node(10, vec.clone())).unwrap();
        let node = engine.get(10).unwrap();
        assert_eq!(node.vector, VectorRepresentations::Full(vec));
    }

    // ── Contains ──

    #[test]
    fn test_contains_missing() {
        let engine = InMemoryEngine::new();
        assert!(!engine.contains(999));
    }

    // ── Update ──

    #[test]
    fn test_update_existing_node() {
        let engine = InMemoryEngine::new();
        engine.insert(create_node(5)).unwrap();
        let mut updated = create_node(5);
        updated.set_field("name", FieldValue::String("bob".into()));
        engine.update(5, updated).unwrap();
        let node = engine.get(5).unwrap();
        assert_eq!(
            node.get_field("name"),
            Some(&FieldValue::String("bob".into()))
        );
    }

    #[test]
    fn test_update_nonexistent_errors() {
        let engine = InMemoryEngine::new();
        let err = engine.update(999, create_node(999)).unwrap_err();
        assert!(matches!(err, VantaError::NodeNotFound(999)));
    }

    // ── Delete ──

    #[test]
    fn test_delete_existing_node() {
        let engine = InMemoryEngine::new();
        engine.insert(create_node(3)).unwrap();
        engine.delete(3).unwrap();
        assert!(!engine.contains(3));
    }

    #[test]
    fn test_delete_nonexistent_errors() {
        let engine = InMemoryEngine::new();
        let err = engine.delete(999).unwrap_err();
        assert!(matches!(err, VantaError::NodeNotFound(999)));
    }

    // ── scan_bitset ──

    #[test]
    fn test_scan_bitset_matches() {
        let engine = InMemoryEngine::new();
        let mut node = create_node(1);
        node.set_bit(0);
        node.set_bit(2);
        engine.insert(node).unwrap();
        let mut node2 = create_node(2);
        node2.set_bit(0);
        engine.insert(node2).unwrap();
        engine.insert(create_node(3)).unwrap(); // no bits

        let mask = FilterBitset::from_u128(1 << 0);
        let hits = engine.scan_bitset(&mask);
        assert_eq!(hits.len(), 2);
        assert!(hits.contains(&1));
        assert!(hits.contains(&2));
    }

    #[test]
    fn test_scan_bitset_no_match() {
        let engine = InMemoryEngine::new();
        let mut node = create_node(1);
        node.set_bit(1);
        engine.insert(node).unwrap();
        let mask = FilterBitset::from_u128(1 << 0);
        assert!(engine.scan_bitset(&mask).is_empty());
    }

    #[test]
    fn test_scan_bitset_tombstone_not_counted() {
        let engine = InMemoryEngine::new();
        let mut node = create_node(1);
        node.set_bit(0);
        engine.insert(node).unwrap();
        engine.delete(1).unwrap();
        let mask = FilterBitset::from_u128(1 << 0);
        assert!(engine.scan_bitset(&mask).is_empty());
    }

    // ── vector_search ──

    #[test]
    fn test_vector_search_returns_top_k() {
        let engine = InMemoryEngine::new();
        engine
            .insert(create_vector_node(1, vec![1.0, 0.0]))
            .unwrap();
        engine
            .insert(create_vector_node(2, vec![0.0, 1.0]))
            .unwrap();
        engine
            .insert(create_vector_node(3, vec![0.9, 0.1]))
            .unwrap();

        let result = engine.vector_search(&[1.0, 0.0], 2, 0.0, None);
        assert_eq!(result.nodes.len(), 2);
        assert_eq!(result.nodes[0].id, 1); // closest
    }

    #[test]
    fn test_vector_search_min_score_filters() {
        let engine = InMemoryEngine::new();
        engine
            .insert(create_vector_node(1, vec![1.0, 0.0]))
            .unwrap();
        engine
            .insert(create_vector_node(2, vec![-1.0, 0.0]))
            .unwrap();

        let result = engine.vector_search(&[1.0, 0.0], 10, 0.5, None);
        assert_eq!(result.nodes.len(), 1);
        assert_eq!(result.nodes[0].id, 1);
    }

    #[test]
    fn test_vector_search_with_bitset_filter() {
        let engine = InMemoryEngine::new();
        let mut node_a = create_vector_node(1, vec![1.0, 0.0]);
        node_a.set_bit(0);
        engine.insert(node_a).unwrap();
        engine
            .insert(create_vector_node(2, vec![0.9, 0.1]))
            .unwrap();

        // Only node 1 has bit 0
        let mask = FilterBitset::from_u128(1 << 0);
        let result = engine.vector_search(&[1.0, 0.0], 10, 0.0, Some(&mask));
        assert_eq!(result.nodes.len(), 1);
        assert_eq!(result.nodes[0].id, 1);
    }

    #[test]
    fn test_vector_search_empty_when_no_vectors() {
        let engine = InMemoryEngine::new();
        engine.insert(create_node(1)).unwrap(); // no vector
        let result = engine.vector_search(&[1.0, 0.0], 10, 0.0, None);
        assert!(result.nodes.is_empty());
    }

    #[test]
    fn test_vector_search_exhaustive_flag() {
        let engine = InMemoryEngine::new();
        engine
            .insert(create_vector_node(1, vec![1.0, 0.0]))
            .unwrap();
        let result = engine.vector_search(&[1.0, 0.0], 10, 0.0, None);
        assert_eq!(result.source_type, SourceType::VectorSearch);
        assert!(!result.is_partial);
        assert_eq!(result.exhaustivity, 1.0);
    }

    // ── traverse (BFS) ──

    #[test]
    fn test_traverse_returns_neighbors() {
        let engine = InMemoryEngine::new();
        let mut n1 = create_node(1);
        n1.add_edge(2, "knows");
        n1.add_edge(3, "knows");
        engine.insert(n1).unwrap();
        let mut n2 = create_node(2);
        n2.add_edge(4, "knows");
        engine.insert(n2).unwrap();
        engine.insert(create_node(3)).unwrap();
        engine.insert(create_node(4)).unwrap();

        let results = engine.traverse(1, "knows", 1, 3).unwrap();
        let ids: Vec<u64> = results.iter().map(|(id, _)| *id).collect();
        assert!(ids.contains(&2));
        assert!(ids.contains(&3));
    }

    #[test]
    fn test_traverse_respects_depth() {
        let engine = InMemoryEngine::new();
        let mut n1 = create_node(1);
        n1.add_edge(2, String::from("edge"));
        engine.insert(n1).unwrap();
        let mut n2 = create_node(2);
        n2.add_edge(3, String::from("edge"));
        engine.insert(n2).unwrap();
        engine.insert(create_node(3)).unwrap();

        let results = engine.traverse(1, "edge", 1, 1).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, 2);
    }

    #[test]
    fn test_traverse_min_depth_filters() {
        let engine = InMemoryEngine::new();
        let mut n1 = create_node(1);
        n1.add_edge(2, String::from("edge"));
        engine.insert(n1).unwrap();
        let mut n2 = create_node(2);
        n2.add_edge(3, String::from("edge"));
        engine.insert(n2).unwrap();
        engine.insert(create_node(3)).unwrap();

        let results = engine.traverse(1, "edge", 2, 3).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, 3);
    }

    #[test]
    fn test_traverse_nonexistent_start_errors() {
        let engine = InMemoryEngine::new();
        let err = engine.traverse(999, "x", 1, 3).unwrap_err();
        assert!(matches!(err, VantaError::NodeNotFound(999)));
    }

    #[test]
    fn test_traverse_correct_depth_values() {
        let engine = InMemoryEngine::new();
        let mut n1 = create_node(1);
        n1.add_edge(2, String::from("e"));
        engine.insert(n1).unwrap();
        let mut n2 = create_node(2);
        n2.add_edge(3, String::from("e"));
        engine.insert(n2).unwrap();
        engine.insert(create_node(3)).unwrap();

        let results = engine.traverse(1, "e", 1, 3).unwrap();
        let pairs: Vec<(u64, u32)> = results.into_iter().collect();
        assert!(pairs.contains(&(2, 1)));
        assert!(pairs.contains(&(3, 2)));
    }

    // ── filter_field ──

    #[test]
    fn test_filter_field_matches() {
        let engine = InMemoryEngine::new();
        let mut n1 = create_node(1);
        n1.set_field("color", FieldValue::String("red".into()));
        engine.insert(n1).unwrap();
        let mut n2 = create_node(2);
        n2.set_field("color", FieldValue::String("blue".into()));
        engine.insert(n2).unwrap();

        let ids = engine.filter_field("color", &FieldValue::String("red".into()));
        assert_eq!(ids, vec![1]);
    }

    #[test]
    fn test_filter_field_no_match() {
        let engine = InMemoryEngine::new();
        engine.insert(create_node(1)).unwrap();
        let ids = engine.filter_field("color", &FieldValue::String("red".into()));
        assert!(ids.is_empty());
    }

    #[test]
    fn test_filter_field_tombstone_excluded() {
        let engine = InMemoryEngine::new();
        let mut n1 = create_node(1);
        n1.set_field("color", FieldValue::String("red".into()));
        engine.insert(n1).unwrap();
        engine.delete(1).unwrap();
        let ids = engine.filter_field("color", &FieldValue::String("red".into()));
        assert!(ids.is_empty());
    }

    // ── hybrid_search ──

    #[test]
    fn test_hybrid_search_field_filter() {
        let engine = InMemoryEngine::new();
        let mut n1 = create_vector_node(1, vec![1.0, 0.0]);
        n1.set_field("region", FieldValue::String("us".into()));
        engine.insert(n1).unwrap();
        let mut n2 = create_vector_node(2, vec![0.9, 0.1]);
        n2.set_field("region", FieldValue::String("eu".into()));
        engine.insert(n2).unwrap();

        let result = engine.hybrid_search(
            &[1.0, 0.0],
            10,
            0.0,
            None,
            &[("region".into(), FieldValue::String("us".into()))],
        );
        assert_eq!(result.nodes.len(), 1);
        assert_eq!(result.nodes[0].id, 1);
    }

    #[test]
    fn test_hybrid_search_bitset_and_field() {
        let engine = InMemoryEngine::new();
        let mut n1 = create_vector_node(1, vec![1.0, 0.0]);
        n1.set_bit(0);
        n1.set_field("active", FieldValue::Bool(true));
        engine.insert(n1).unwrap();
        let mut n2 = create_vector_node(2, vec![0.9, 0.1]);
        n2.set_bit(0);
        n2.set_field("active", FieldValue::Bool(false));
        engine.insert(n2).unwrap();

        let mask = FilterBitset::from_u128(1 << 0);
        let result = engine.hybrid_search(
            &[1.0, 0.0],
            10,
            0.0,
            Some(&mask),
            &[("active".into(), FieldValue::Bool(true))],
        );
        assert_eq!(result.nodes.len(), 1);
        assert_eq!(result.nodes[0].id, 1);
    }

    #[test]
    fn test_hybrid_search_empty_when_field_mismatch() {
        let engine = InMemoryEngine::new();
        let mut n1 = create_vector_node(1, vec![1.0, 0.0]);
        n1.set_field("region", FieldValue::String("us".into()));
        engine.insert(n1).unwrap();

        let result = engine.hybrid_search(
            &[1.0, 0.0],
            10,
            0.0,
            None,
            &[("region".into(), FieldValue::String("eu".into()))],
        );
        assert!(result.nodes.is_empty());
    }

    // ── flush_wal (no-op without WAL) ──

    #[test]
    fn test_flush_wal_noop_without_wal() {
        let engine = InMemoryEngine::new();
        // Should not error when no WAL is configured
        assert!(engine.flush_wal().is_ok());
    }

    // ── node_count / stats ──

    #[test]
    fn test_node_count_alive_only() {
        let engine = InMemoryEngine::new();
        engine.insert(create_node(1)).unwrap();
        engine.insert(create_node(2)).unwrap();
        engine.delete(1).unwrap();
        assert_eq!(engine.node_count(), 1);
    }

    #[test]
    fn test_stats_counts_alive_nodes() {
        let engine = InMemoryEngine::new();
        engine.insert(create_node(1)).unwrap();
        engine.insert(create_node(2)).unwrap();
        let stats = engine.stats();
        assert_eq!(stats.node_count, 2);
    }

    #[test]
    fn test_stats_counts_vectors() {
        let engine = InMemoryEngine::new();
        engine
            .insert(create_vector_node(1, vec![0.1, 0.2]))
            .unwrap();
        engine.insert(create_node(2)).unwrap(); // no vector
        let stats = engine.stats();
        assert_eq!(stats.vector_count, 1);
        assert_eq!(stats.total_dimensions, 2);
    }

    #[test]
    fn test_stats_counts_edges() {
        let engine = InMemoryEngine::new();
        let mut n1 = create_node(1);
        n1.add_edge(2, "knows");
        n1.add_edge(3, "knows");
        engine.insert(n1).unwrap();
        engine.insert(create_node(2)).unwrap();
        let stats = engine.stats();
        assert_eq!(stats.edge_count, 2);
    }
}
