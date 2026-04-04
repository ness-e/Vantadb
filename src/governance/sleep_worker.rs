use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use crate::storage::StorageEngine;
use crate::node::{CognitiveUnit, UnifiedNode, NeuronType, NodeFlags, FieldValue};
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
    pub async fn start(storage: Arc<StorageEngine>) {
        let sleep_duration = Duration::from_secs(10);
        let inactivity_threshold_ms = 5000;

        loop {
            sleep(sleep_duration).await;

            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
            let last_activity = storage.last_query_timestamp.load(Ordering::Acquire);

            if now - last_activity > inactivity_threshold_ms {
                Self::execute_rem_phase(&storage).await;
            }
        }
    }

    async fn execute_rem_phase(storage: &Arc<StorageEngine>) {
        println!("🌙 [Circadian] Iniciando Fase REM (Mantenimiento de Memoria)...");

        let mut to_consolidate = Vec::new();
        let mut to_purge = Vec::new();
        let mut summarization_candidates: Vec<UnifiedNode> = Vec::new();

        let total_nodes;

        {
            // ── Stage 1 & 2: Bayesian Forgetting + Survival Evaluation ──
            let mut cortex = storage.cortex_ram.write().unwrap();
            total_nodes = cortex.len();
            let max_amygdala_shielded = (total_nodes as f32 * 0.05).ceil() as usize;
            let mut current_shielded = 0;

            let mut keys_to_remove = Vec::new();

            for (&id, node) in cortex.iter_mut() {
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
                if now - storage.last_query_timestamp.load(Ordering::Acquire) < 5000 {
                    println!("🔌 [Circadian] Interrupción de Fase REM (Actividad de I/O detectada).");
                    break;
                }

                // Amygdala Budget: Protect high-valence nodes
                if node.semantic_valence >= 0.8 && current_shielded < max_amygdala_shielded {
                    current_shielded += 1;
                    continue;
                }

                // Stage 1: Olvido Bayesiano
                node.hits = (node.hits as f32 * 0.5) as u32;

                // Stage 2: Evaluaciones de Supervivencia
                if node.trust_score() < 0.2 {
                    keys_to_remove.push(id);
                    to_purge.push(id);
                } else if node.hits < 10 && !node.is_pinned() && (now - node.last_accessed > 60_000) {
                    keys_to_remove.push(id);

                    // Collect "Onírico" candidates for Stage 3 summarization
                    if node.hits < 5 {
                        summarization_candidates.push(node.clone());
                    }

                    to_consolidate.push(node.clone());
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
        for id in to_purge {
            let _ = storage.delete(id, "Olvido Bayesiano (Trust < 0.2)");
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
                summary_node.vector = crate::node::VectorData::F32(vec);
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
