use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use parking_lot::RwLock;
use crate::node::UnifiedNode;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum QuantumState {
    Superposition,     // Pending contextual collapse
    CollapsedAccept,   // Allowed to migrate to LTN/STN
    CollapsedReject,   // Heading to ThalamicGate + Purge
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuantumNeuron {
    pub node_id: u64,
    pub payload: UnifiedNode,
    pub state: QuantumState,
    pub injected_at: u64, // Unix ms
    pub collapse_deadline_ms: u64, 
}

impl QuantumNeuron {
    pub fn new(payload: UnifiedNode, deadline_offset_ms: u64) -> Self {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
        Self {
            node_id: payload.id,
            payload,
            state: QuantumState::Superposition,
            injected_at: now,
            collapse_deadline_ms: now + deadline_offset_ms,
        }
    }
}

pub struct UncertaintyBuffer {
    pub quantum_zones: RwLock<HashMap<u64, QuantumNeuron>>,
}

impl UncertaintyBuffer {
    pub fn new() -> Self {
        Self {
            quantum_zones: RwLock::new(HashMap::new()),
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
}
