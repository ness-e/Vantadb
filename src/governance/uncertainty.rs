use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use parking_lot::RwLock;
use crate::node::UnifiedNode;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum QuantumState {
    Superposition,     // Pending contextual collapse
    CollapsedAccept,   // Allowed to migrate to LTN/STN
    CollapsedReject,   // Heading to ThalamicGate + Purge
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuantumNeuron {
    pub node_id: u64,
    pub candidates: Vec<UnifiedNode>,
    pub state: QuantumState,
    pub injected_at: u64, // Unix ms
    pub collapse_deadline_ms: u64, 
}

impl QuantumNeuron {
    pub fn new_superposition(incumbent: UnifiedNode, challenger: UnifiedNode, deadline_offset_ms: u64) -> Self {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
        Self {
            node_id: incumbent.id,
            candidates: vec![incumbent, challenger],
            state: QuantumState::Superposition,
            injected_at: now,
            collapse_deadline_ms: now + deadline_offset_ms,
        }
    }

    pub fn add_candidate(&mut self, candidate: UnifiedNode) {
        if self.candidates.len() < 3 {
            self.candidates.push(candidate);
        }
    }
}

pub struct CollapseStats {
    pub superposition_to_collapsed: AtomicU64,
    pub superposition_to_decayed: AtomicU64,
}

impl Default for CollapseStats {
    fn default() -> Self {
        Self {
            superposition_to_collapsed: AtomicU64::new(0),
            superposition_to_decayed: AtomicU64::new(0),
        }
    }
}

pub struct UncertaintyBuffer {
    pub quantum_zones: RwLock<HashMap<u64, QuantumNeuron>>,
    pub stats: CollapseStats,
}

impl UncertaintyBuffer {
    pub fn new() -> Self {
        Self {
            quantum_zones: RwLock::new(HashMap::new()),
            stats: CollapseStats::default(),
        }
    }

    pub fn insert_quantum(&self, neuron: QuantumNeuron) {
        let mut map = self.quantum_zones.write();
        map.insert(neuron.node_id, neuron);
    }
    
    pub fn get_quantum(&self, id: u64) -> Option<QuantumNeuron> {
        self.quantum_zones.read().get(&id).cloned()
    }

    pub fn remove_quantum(&self, id: u64) -> Option<QuantumNeuron> {
        self.quantum_zones.write().remove(&id)
    }

    pub fn record_accept(&self) {
        self.stats.superposition_to_collapsed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_decay(&self) {
        self.stats.superposition_to_decayed.fetch_add(1, Ordering::Relaxed);
    }

    /// Cortocircuito del NMI (Hard-Urgency). 
    /// Realiza un colapso especulativo: Integra el candidato de mayor valencia actual 
    /// de toda la Penumbra y purga el resto. Da prioridad a la velocidad por sobre la precisión.
    pub fn force_collapse_nmi(&self) -> Option<UnifiedNode> {
        let mut map = self.quantum_zones.write();
        if map.is_empty() {
            return None;
        }

        let mut best_candidate: Option<UnifiedNode> = None;
        let mut best_valence = -1.0;

        for neuron in map.values() {
            for candidate in &neuron.candidates {
                if candidate.semantic_valence > best_valence {
                    best_valence = candidate.semantic_valence;
                    best_candidate = Some(candidate.clone());
                }
            }
        }

        let discarded = map.len() as u64; // Approximated discarded superpositions
        
        // We consider all superpositions discarded except one that we saved a candidate from,
        // although technically we are returning a Node not a QuantumNeuron.
        self.stats.superposition_to_decayed.fetch_add(discarded.saturating_sub(1), Ordering::Relaxed);
        map.clear();
        
        if let Some(winner) = best_candidate {
            self.stats.superposition_to_collapsed.fetch_add(1, Ordering::Relaxed);
            return Some(winner);
        }
        
        None
    }
}
