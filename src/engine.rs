use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use parking_lot::{Mutex, RwLock};

use crate::error::{Result, VantaError};
use crate::node::{FieldValue, UnifiedNode, VectorRepresentations};
use crate::wal::{WalReader, WalRecord, WalWriter};

// ─── Query Result ──────────────────────────────────────────

/// How the result was produced
#[derive(Debug, Clone)]
pub enum SourceType {
    FullScan,
    BitsetFilter,
    VectorSearch,
    GraphTraversal,
    Hybrid,
}

/// Query result with exhaustivity metadata
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub nodes: Vec<UnifiedNode>,
    /// true if resource limits truncated results
    pub is_partial: bool,
    /// 0.0-1.0 search completeness
    pub exhaustivity: f32,
    /// which index/scan was used
    pub source_type: SourceType,
}

/// Engine statistics snapshot
#[derive(Debug, Clone, Default)]
pub struct EngineStats {
    pub node_count: u64,
    pub edge_count: u64,
    pub vector_count: u64,
    pub total_dimensions: u64,
    pub memory_estimate_bytes: u64,
}

// ─── In-Memory Engine ──────────────────────────────────────

/// Fase 1 storage engine: HashMap + optional WAL.
///
/// Thread-safe: RwLock for reads, Mutex for WAL writes.
/// Fase 2: Replace HashMap with RocksDB-backed MemTable.
pub struct InMemoryEngine {
    nodes: RwLock<HashMap<u64, UnifiedNode>>,
    wal: Mutex<Option<WalWriter>>,
    next_id: AtomicU64,
    #[allow(dead_code)]
    wal_path: Option<PathBuf>,
}

impl InMemoryEngine {
    /// Create engine (in-memory only, no persistence)
    pub fn new() -> Self {
        Self {
            nodes: RwLock::new(HashMap::with_capacity(1024)),
            wal: Mutex::new(None),
            next_id: AtomicU64::new(1),
            wal_path: None,
        }
    }

    /// Create engine with WAL durability. Replays existing WAL on open.
    pub fn with_wal(wal_path: impl AsRef<Path>) -> Result<Self> {
        let path = wal_path.as_ref().to_path_buf();
        let mut nodes_map = HashMap::with_capacity(1024);
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

        let writer = WalWriter::open(&path)?;

        Ok(Self {
            nodes: RwLock::new(nodes_map),
            wal: Mutex::new(Some(writer)),
            next_id: AtomicU64::new(max_id + 1),
            wal_path: Some(path),
        })
    }

    /// Generate next unique node ID
    pub fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    /// Insert a node. Auto-assigns ID if node.id == 0.
    pub fn insert(&self, mut node: UnifiedNode) -> Result<u64> {
        if node.id == 0 {
            node.id = self.next_id();
        }
        let id = node.id;

        // WAL first (durability before visibility)
        if let Some(ref mut wal) = *self.wal.lock() {
            wal.append(&WalRecord::Insert(node.clone()))?;
        }

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
        if let Some(ref mut wal) = *self.wal.lock() {
            wal.append(&WalRecord::Update {
                id,
                node: node.clone(),
            })?;
        }
        let mut nodes = self.nodes.write();
        if !nodes.contains_key(&id) {
            return Err(VantaError::NodeNotFound(id));
        }
        nodes.insert(id, node);
        Ok(())
    }

    /// Delete a node
    pub fn delete(&self, id: u64) -> Result<()> {
        if let Some(ref mut wal) = *self.wal.lock() {
            wal.append(&WalRecord::Delete { id })?;
        }
        let mut nodes = self.nodes.write();
        if nodes.remove(&id).is_none() {
            return Err(VantaError::NodeNotFound(id));
        }
        Ok(())
    }

    /// Scan nodes matching a bitset mask (all bits in mask must be set)
    pub fn scan_bitset(&self, mask: u128) -> Vec<u64> {
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
        bitset_filter: Option<u128>,
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
        bitset_mask: Option<u128>,
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
