use std::collections::{HashSet, VecDeque};
use parking_lot::RwLock;

/// ThalamicGate limits the reingestion of known rejected/hallucinated nodes.
/// It operates in strict bounds (max 5,000 IDs) simulating an LRU set
/// using a HashSet alongside a VecDeque, guaranteeing zero payload dependency
/// and O(1) checks.
pub struct ThalamicGate {
    max_capacity: usize,
    rejected_set: RwLock<HashSet<u64>>,
    eviction_queue: RwLock<VecDeque<u64>>,
}

impl ThalamicGate {
    pub fn new(max_capacity: usize) -> Self {
        Self {
            max_capacity,
            rejected_set: RwLock::new(HashSet::with_capacity(max_capacity)),
            eviction_queue: RwLock::new(VecDeque::with_capacity(max_capacity)),
        }
    }

    /// Mark an ID as rejected. If hitting capacity, eviction FIFO kicks in.
    pub fn record_rejection(&self, node_id: u64) {
        let mut set = self.rejected_set.write();
        
        if set.insert(node_id) {
            let mut q = self.eviction_queue.write();
            q.push_back(node_id);
            
            if q.len() > self.max_capacity {
                if let Some(oldest) = q.pop_front() {
                    set.remove(&oldest);
                }
            }
        }
    }

    /// Fast O(1) check if a node is blocked.
    pub fn is_rejected(&self, node_id: u64) -> bool {
        self.rejected_set.read().contains(&node_id)
    }

    /// Optional explicit amnesty
    pub fn grant_amnesty(&self, node_id: u64) {
        let mut set = self.rejected_set.write();
        if set.remove(&node_id) {
            let mut q = self.eviction_queue.write();
            if let Some(pos) = q.iter().position(|&id| id == node_id) {
                q.remove(pos);
            }
        }
    }
}
