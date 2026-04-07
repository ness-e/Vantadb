use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tokio::sync::mpsc;
use crate::storage::StorageEngine;
use crate::node::{CognitiveUnit, UnifiedNode, NeuronType, NodeFlags, FieldValue};
use crate::governance::invalidations::{InvalidationDispatcher, InvalidationEvent};
use std::sync::atomic::Ordering;
use std::time::{SystemTime, UNIX_EPOCH, Instant};
use rocksdb::CompactOptions;
use std::collections::HashMap;

/// Maximum wall-clock time the REM phase may spend on Neural Summarization.
/// If exceeded, pending groups are deferred to the next cycle.
const MAX_SUMMARIZATION_DURATION_MS: u128 = 8_000; // 8 seconds

/// Minimum combined hit-weight a group must have to deserve LLM summarization.
/// Groups below this threshold are directly purged (no CPU waste on garbage).
const MIN_GROUP_WEIGHT_FOR_SUMMARY: u32 = 3;

pub struct SleepWorker;

impl SleepWorker {
    pub async fn start(storage: Arc<StorageEngine>, invalidation_tx: mpsc::Sender<InvalidationEvent>) {
        let sleep_duration = Duration::from_secs(10);
        let inactivity_threshold_ms = 5000;

        loop {
            // Wake up periodically or immediately if emergency memory cap is hit
            if storage.emergency_rem_trigger.load(Ordering::Acquire) {
                println!("🚨 [Circadian] TRIGGER DE EMERGENCIA: Cortex RAM al límite. Iniciando Fase REM Agresiva (OOM Guard).");
                storage.emergency_rem_trigger.store(false, Ordering::Release);
                Self::execute_rem_phase(&storage, &invalidation_tx).await;
            }

            sleep(sleep_duration).await;

            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
            let last_activity = storage.last_query_timestamp.load(Ordering::Acquire);

            if now - last_activity > inactivity_threshold_ms {
                Self::execute_rem_phase(&storage, &invalidation_tx).await;
            }
        }
    }

    async fn execute_rem_phase(storage: &Arc<StorageEngine>, invalidation_tx: &mpsc::Sender<InvalidationEvent>) {
        println!("🌙 [Circadian] Iniciando Fase REM (Mantenimiento de Memoria)...");

        let mut to_consolidate = Vec::new();
        let mut to_purge: Vec<(u64, bool, Option<String>)> = Vec::new(); // (id, is_hallucination, slashed_role)
        let mut summarization_candidates: Vec<UnifiedNode> = Vec::new();

        {
            // ── Stage 0: UncertaintyBuffer Decay & Selective Amnesia ──
            let stats = &storage.uncertainty_buffer.stats;
            let collapsed = stats.superposition_to_collapsed.load(Ordering::Relaxed) as f64;
            let decayed = stats.superposition_to_decayed.load(Ordering::Relaxed) as f64;
            let total = collapsed + decayed;
            
            // Si el ratio de decay es > 70%, el backend de Uncertainty usará plazos más cortos para el colapso.
            let _shrinks_deadline = total > 10.0 && (decayed / total) > 0.7;
            
            let mut uncertainty = storage.uncertainty_buffer.quantum_zones.write();
            let mut keys_to_collapse = Vec::new();
            let mut keys_to_purge = Vec::new();

            for (&id, q_neuron) in uncertainty.iter_mut() {
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;

                let mut best_trust = -1.0;
                for cand in &q_neuron.candidates {
                    if cand.trust_score() > best_trust {
                        best_trust = cand.trust_score();
                    }
                }

                // Si se venció el deadline colapsamos temporalmente a favor del mejor
                if now > q_neuron.collapse_deadline_ms {
                    keys_to_collapse.push(id);
                } else if best_trust < 0.2 {
                    keys_to_purge.push(id); // purga total
                } else {
                    // Decay the trust incrementally so it eventually purges if not salvaged
                    if _shrinks_deadline {
                        q_neuron.collapse_deadline_ms = q_neuron.collapse_deadline_ms.saturating_sub(100);
                    }
                    for cand in &mut q_neuron.candidates {
                        cand.trust_score *= 0.9;
                    }
                }
            }

            let discarded = keys_to_purge.len() as u64;
            if discarded > 0 {
                storage.uncertainty_buffer.stats.superposition_to_decayed.fetch_add(discarded, Ordering::Relaxed);
            }

            let mut winners_to_insert = Vec::new();
            let mut losers_to_shadow = Vec::new();

            for id in keys_to_purge {
                if let Some(purged) = uncertainty.remove(&id) {
                    storage.thalamic_gate.record_rejection(id);
                    for cand in purged.candidates {
                        if cand.semantic_valence > 0.8 {
                            losers_to_shadow.push((id, cand.id, "Amnesia Selectiva: Quantum Decay Total".to_string()));
                        }
                    }
                }
            }

            for id in keys_to_collapse {
                if let Some(mut q_neuron) = uncertainty.remove(&id) {
                    storage.uncertainty_buffer.stats.superposition_to_collapsed.fetch_add(1, Ordering::Relaxed);
                    
                    let mut best_idx = 0;
                    let mut best_trust = -1.0;
                    for (i, cand) in q_neuron.candidates.iter().enumerate() {
                        if cand.trust_score() > best_trust {
                            best_trust = cand.trust_score();
                            best_idx = i;
                        }
                    }

                    if !q_neuron.candidates.is_empty() {
                        let winner = q_neuron.candidates.remove(best_idx);
                        winners_to_insert.push(winner);
                        
                        // Losers
                        for cand in q_neuron.candidates {
                            losers_to_shadow.push((id, cand.id, "Colapso Temporal: Candidato Perdedor".to_string()));
                        }
                    }
                }
            }

            // Drop write lock on uncertainty_buffer to prevent deadlocks during insertion
            drop(uncertainty);

            // Re-inyectamos a los ganadores del colapso para que sigan el ciclo de vida normal
            // (almacenamiento STN e indexación cruzada HNSW)
            for winner in winners_to_insert {
                let _ = storage.insert(&winner); 
            }

            // Purga auditables hacia el shadow kernel
            use crate::governance::AuditableTombstone;
            if !losers_to_shadow.is_empty() {
                if let Some(cf_shadow) = storage.db.cf_handle("shadow_kernel") {
                    for (id, hash, reason) in losers_to_shadow {
                        let tomb = AuditableTombstone::new(id, reason, hash);
                        let key = id.to_le_bytes();
                        if let Ok(tomb_val) = bincode::serialize(&tomb) {
                            let _ = storage.db.put_cf(&cf_shadow, &key, &tomb_val);
                        }
                    }
                }
            }
        }

        let total_nodes;

        {
            // ── Stage 1 & 2: Bayesian Forgetting + Survival Evaluation ──
            let mut cortex = storage.cortex_ram.write().unwrap();
            total_nodes = cortex.len();
            let max_amygdala_shielded = (total_nodes as f32 * 0.05).ceil() as usize;
            let mut current_shielded = 0;

            let mut keys_to_remove = Vec::new();

            // Setup Profile-based Backpressure
            let caps = crate::hardware::HardwareCapabilities::global();
            let max_consolidations = match caps.profile {
                crate::hardware::HardwareProfile::Enterprise => 5000,
                crate::hardware::HardwareProfile::Performance => 500,
                crate::hardware::HardwareProfile::Survival => 50,
            };

            for (&id, node) in cortex.iter_mut() {
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;

                // Fase 30: Purga Arqueológica
                if node.flags.is_set(NodeFlags::REHYDRATED) {
                    // Los recuerdos rehidratados solo viven temporalmente para ser sintetizados.
                    // Si alcanzó la fase REM, su ventana expiró, evict de RAM. (Ya existen intactos en Shadow Archive)
                    keys_to_remove.push(id);
                    continue;
                }
                if now - storage.last_query_timestamp.load(Ordering::Acquire) < 5000 {
                    println!("🔌 [Circadian] Interrupción de Fase REM (Actividad de I/O detectada).");
                    break;
                }

                // Amygdala Budget: Protect high-valence nodes
                if node.semantic_valence >= 0.8 && current_shielded < max_amygdala_shielded {
                    current_shielded += 1;
                    continue;
                }

                // Hallucination Check - Immediate Nullification
                if node.flags.is_set(NodeFlags::HALLUCINATION) {
                    println!("🧨 [Circadian] Alucinación detectada en el nodo {}. Iniciando purga reactiva inmediata.", id);
                    keys_to_remove.push(id);
                    // Phase 36: Extract owner_role for epistemic slashing
                    let slashed_role = node.relational.get("_owner_role")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    to_purge.push((id, true, slashed_role));
                    continue;
                }

                // Stage 1: Olvido Bayesiano
                node.hits = (node.hits as f32 * 0.5) as u32;

                // Stage 2: Evaluaciones de Supervivencia
                if node.trust_score() < 0.2 {
                    keys_to_remove.push(id);
                    to_purge.push((id, false, None));
                } else if node.hits < 10 && !node.is_pinned() && (now - node.last_accessed > 60_000) {
                    keys_to_remove.push(id);

                    // Collect "Onírico" candidates for Stage 3 summarization
                    if node.hits < 5 {
                        summarization_candidates.push(node.clone());
                    }

                    // Apply Hardware Backpressure to Disk I/O
                    if to_consolidate.len() < max_consolidations {
                        to_consolidate.push(node.clone());
                    } else {
                        // Defer to next cycle due to IO saturation 
                        // Note: we pushed the key to keys_to_remove, but we didn't add it to consolidate
                        // it will be evicted from RAM, which is correct for Survival, it essentially acts as a cache miss on defer
                        // wait, actually, if it's deferred from consolidation, it SHOULD stay in RAM if we want to save it!
                        // Let's remove it from keys_to_remove so it stays in Cortex!
                        keys_to_remove.pop();
                    }
                }
            }

            for id in keys_to_remove {
                cortex.remove(&id);
            }
        }

        // ── Mutaciones Físicas (fuera del Write Lock) ──

        // Consolidaciones STN → LTN (with HNSW sync fix)
        for node in &to_consolidate {
            if let Err(e) = storage.consolidate_node(node) {
                eprintln!("⚠️ [Circadian] Error consolidating node {}: {}", node.id, e);
            }
        }

        let mut deleted_count = 0usize;
        for (id, is_hallucination, slashed_role) in &to_purge {
            if *is_hallucination {
                // ── Phase 36: Epistemic Slashing ──
                // If we identified the owner_role, slash it permanently
                if let Some(role) = slashed_role {
                    // 1. Slash in DevilsAdvocate OriginCollisionTracker
                    {
                        let mut tracker = storage.advocate.collision_tracker.write();
                        tracker.slash_origin(role);
                    }
                    // 2. Permanent ban in ThalamicGate Bloom Filter (L1 Hard-Filter)
                    storage.thalamic_gate.record_role_ban(role);
                    println!("🔥 [Circadian] Epistemic Apoptosis: agent '{}' slashed → TrustScore=0.0, L1 banned", role);
                }

                // Emit reactive invalidation event for hallucinated nodes
                InvalidationDispatcher::emit_hallucination_purged(
                    invalidation_tx,
                    *id,
                    "Flagged HALLUCINATION during REM phase".to_string(),
                ).await;
                let _ = storage.delete(*id, "Purga Reactiva: HALLUCINATION flag");
            } else {
                let _ = storage.delete(*id, "Olvido Bayesiano (Trust < 0.2)");
            }
            deleted_count += 1;
        }

        // ── Stage 3: Neural Summarization ──
        if !summarization_candidates.is_empty() {
            Self::execute_neural_summarization(storage, &summarization_candidates).await;
        }

        if deleted_count > 100 {
            println!("🧹 [Circadian] Desencadenando compactación de disco debido a alta entropía.");
            let mut c_opts = CompactOptions::default();
            c_opts.set_exclusive_manual_compaction(false);
            storage.db.compact_range_opt(None::<&[u8]>, None::<&[u8]>, &c_opts);
        }

        println!("☀️  [Circadian] Fase REM finalizada. Analizados: {} STN.", total_nodes);
    }

    /// Stage 3: Neural Summarization — Groups "Onírico" nodes by thread,
    /// invokes the LLM for cognitive compression, and atomically moves the
    /// originals to the shadow_kernel while inserting summary nodes into deep_memory.
    async fn execute_neural_summarization(
        storage: &Arc<StorageEngine>,
        candidates: &[UnifiedNode],
    ) {
        // ── Step 1: Cluster candidates by shared thread (via "belongs_to_thread" edges) ──
        let mut thread_groups: HashMap<u64, Vec<&UnifiedNode>> = HashMap::new();

        for node in candidates {
            // Look for an edge labeled "belongs_to_thread" to find the cluster key
            if let Some(thread_edge) = node.edges.iter().find(|e| e.label == "belongs_to_thread") {
                thread_groups.entry(thread_edge.target).or_default().push(node);
            }
        }

        let deadline = Instant::now();
        let llm = crate::llm::LlmClient::new();

        for (thread_id, group) in &thread_groups {
            // ── Time budget enforcement ──
            if deadline.elapsed().as_millis() > MAX_SUMMARIZATION_DURATION_MS {
                println!("⏳ [Circadian] Presupuesto de tiempo de Summarization alcanzado. Difiriendo grupos restantes al siguiente ciclo.");
                break;
            }

            // ── Step 2: Validate group weight ──
            // Only summarize if the combined semantic weight is meaningful
            if group.len() < 2 {
                continue; // Single node — not worth an LLM call
            }
            let group_hit_sum: u32 = group.iter().map(|n| n.hits).sum();
            if group_hit_sum < MIN_GROUP_WEIGHT_FOR_SUMMARY {
                continue; // Garbage — direct purge is more efficient
            }

            // ── Step 3: Invoke LLM with structured context ──
            let node_refs: Vec<&UnifiedNode> = group.iter().copied().collect();
            let summary_text = match llm.summarize_context(&node_refs).await {
                Ok(text) => text,
                Err(e) => {
                    eprintln!("⚠️ [Circadian] LLM summarization failed for thread {}: {}. Skipping group.", thread_id, e);
                    continue; // Non-fatal: originals remain in LTN
                }
            };

            // ── Step 4: Create Summary Neuron with Semantic Lineage ──
            let summary_id = rand::random::<u64>();
            let mut summary_node = UnifiedNode::new(summary_id);
            summary_node.neuron_type = NeuronType::LTNeuron;
            summary_node.flags.set(NodeFlags::PINNED); // Immutable summary
            summary_node.semantic_valence = 0.9; // Protected by Amygdala Budget
            summary_node.trust_score = group.iter().map(|n| n.trust_score).sum::<f32>() / group.len() as f32;
            summary_node.set_field("type", FieldValue::String("NeuralSummary".to_string()));
            summary_node.set_field("content", FieldValue::String(summary_text));
            summary_node.set_field("source_thread", FieldValue::Int(*thread_id as i64));

            // Semantic Lineage: ancestors track the original node IDs for Archaeology
            let ancestor_ids: Vec<String> = group.iter().map(|n| n.id.to_string()).collect();
            summary_node.set_field("ancestors", FieldValue::String(ancestor_ids.join(",")));

            // Embed the summary text for vector searchability
            if let Ok(vec) = llm.generate_embedding(
                summary_node.get_field("content").and_then(|f| f.as_str()).unwrap_or("")
            ).await {
                summary_node.vector = crate::node::VectorRepresentations::Full(vec);
                summary_node.flags.set(NodeFlags::HAS_VECTOR);
            }

            // ── Step 5: Atomic Transaction — insert summary + tombstone originals ──
            // First: Insert summary into deep_memory
            if let Err(e) = storage.insert_to_cf(&summary_node, "deep_memory") {
                eprintln!("⚠️ [Circadian] Failed to persist summary node: {}. Aborting group summarization.", e);
                continue; // CRITICAL: Do NOT delete originals if summary failed
            }

            // Then: Move originals to shadow_kernel as tombstones
            for original in group {
                if let Err(e) = storage.delete(original.id, &format!(
                    "Neural Summarization: condensed into summary node {}",
                    summary_id
                )) {
                    eprintln!("⚠️ [Circadian] Failed to tombstone node {} during summarization: {}", original.id, e);
                }
            }

            println!(
                "🧬 [Circadian] Neural Summarization: {} nodos del thread {} → Neurona de Resumen {} (deep_memory).",
                group.len(), thread_id, summary_id
            );
        }
    }
}
