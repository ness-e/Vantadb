pub mod sleep_worker;
pub mod invalidations;
pub mod thalamic_gate;
pub mod uncertainty;

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// A permanent record of a node that has been logically deleted.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditableTombstone {
    pub id: u64,
    pub timestamp_deleted: u64,
    pub reason: String,
    pub original_hash: u64,
}

impl AuditableTombstone {
    pub fn new(id: u64, reason: impl Into<String>, original_hash: u64) -> Self {
        let timestamp_deleted = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            id,
            timestamp_deleted,
            reason: reason.into(),
            original_hash,
        }
    }
}

// ─── Soberanía Cognitiva (Devil's Advocate) ────────────────
use crate::node::{UnifiedNode, CognitiveUnit};

#[derive(Debug, Clone)]
pub enum ResolutionResult {
    Accept,
    Reject(String),               // Razón basada en Trust Score o gatekeep
    Superposition(crate::governance::uncertainty::QuantumNeuron), // Zona de incertidumbre con múltiples candidatos (Fase 32B)
    Merge { new_trust: f32 },     // Combinar aserciones bajando certeza
}

pub trait TrustArbiter {
    fn evaluate_conflict(&self, incumbent: &UnifiedNode, challenger: &UnifiedNode) -> ResolutionResult;
}

pub struct DevilsAdvocate;

impl DevilsAdvocate {
    pub fn new() -> Self {
        Self
    }
}

impl TrustArbiter for DevilsAdvocate {
    fn evaluate_conflict(&self, incumbent: &UnifiedNode, challenger: &UnifiedNode) -> ResolutionResult {
        // Obtenemos similitud de vectores
        if let Some(sim) = incumbent.vector.cosine_similarity(&challenger.vector) {
            // Umbral del 95% de similitud para evaluar si hablan del mismo tema
            if sim > 0.95 {
                // Heurística de conflictos base (ej: Campos vacíos o mutaciones sospechosas)
                if challenger.trust_score() < incumbent.trust_score() {
                    return ResolutionResult::Superposition(
                        crate::governance::uncertainty::QuantumNeuron::new_superposition(
                            incumbent.clone(),
                            challenger.clone(),
                            10000 // 10s default collapse deadline
                        )
                    );
                }
            }
        }
        ResolutionResult::Accept
    }
}
