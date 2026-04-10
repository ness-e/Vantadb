use crate::node::UnifiedNode;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum RecordState {
    PendingConflict,   // Pending contextual resolution
    ResolvedAccept,    // Allowed to migrate to persistent storage
    ResolvedReject,    // Heading to AdmissionFilter + Purge
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsistencyRecord {
    pub node_id: u64,
    pub candidates: Vec<UnifiedNode>,
    pub state: RecordState,
    pub injected_at: u64, // Unix ms
    pub resolution_deadline_ms: u64,
}

impl ConsistencyRecord {
    pub fn new_superposition(
        incumbent: UnifiedNode,
        challenger: UnifiedNode,
        deadline_offset_ms: u64,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        Self {
            node_id: incumbent.id,
            candidates: vec![incumbent, challenger],
            state: RecordState::PendingConflict,
            injected_at: now,
            resolution_deadline_ms: now + deadline_offset_ms,
        }
    }

    pub fn add_candidate(&mut self, candidate: UnifiedNode) {
        if self.candidates.len() < 3 {
            self.candidates.push(candidate);
        }
    }
}

pub struct ResolutionStats {
    pub pending_to_resolved: AtomicU64,
    pub pending_to_decayed: AtomicU64,
}

impl Default for ResolutionStats {
    fn default() -> Self {
        Self {
            pending_to_resolved: AtomicU64::new(0),
            pending_to_decayed: AtomicU64::new(0),
        }
    }
}

pub struct ConsistencyBuffer {
    pub records: RwLock<HashMap<u64, ConsistencyRecord>>,
    pub stats: ResolutionStats,
}

impl Default for ConsistencyBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsistencyBuffer {
    pub fn new() -> Self {
        Self {
            records: RwLock::new(HashMap::new()),
            stats: ResolutionStats::default(),
        }
    }

    pub fn insert_record(&self, record: ConsistencyRecord) {
        let mut map = self.records.write();
        map.insert(record.node_id, record);
    }

    pub fn get_record(&self, id: u64) -> Option<ConsistencyRecord> {
        self.records.read().get(&id).cloned()
    }

    pub fn remove_record(&self, id: u64) -> Option<ConsistencyRecord> {
        self.records.write().remove(&id)
    }

    pub fn record_accept(&self) {
        self.stats
            .pending_to_resolved
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_decay(&self) {
        self.stats
            .pending_to_decayed
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Emergency flush of the consistency buffer.
    /// Performs speculative resolution: Integrates the candidate of highest importance
    /// and purges the rest.
    pub fn force_flush(&self) -> Option<UnifiedNode> {
        let mut map = self.records.write();
        if map.is_empty() {
            return None;
        }

        let mut best_candidate: Option<UnifiedNode> = None;
        let mut best_importance = -1.0;

        for record in map.values() {
            for candidate in &record.candidates {
                if candidate.importance > best_importance {
                    best_importance = candidate.importance;
                    best_candidate = Some(candidate.clone());
                }
            }
        }

        let discarded = map.len() as u64; 

        self.stats
            .pending_to_decayed
            .fetch_add(discarded.saturating_sub(1), Ordering::Relaxed);
        map.clear();

        if let Some(winner) = best_candidate {
            self.stats
                .pending_to_resolved
                .fetch_add(1, Ordering::Relaxed);
            return Some(winner);
        }

        None
    }
}
