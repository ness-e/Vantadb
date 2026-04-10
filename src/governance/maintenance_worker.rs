use crate::governance::invalidations::{InvalidationDispatcher, InvalidationEvent};
use crate::node::{AccessTracker, FieldValue, NodeFlags, NodeTier, UnifiedNode};
use crate::storage::StorageEngine;
use rocksdb::CompactOptions;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use tokio::time::sleep;

/// Maximum duration the maintenance cycle may spend on data compression.
const MAX_COMPRESSION_DURATION_MS: u128 = 8_000; 

/// Minimum combined hit-weight for a group to deserve compression.
const MIN_GROUP_WEIGHT_FOR_COMPRESSION: u32 = 3;

pub struct MaintenanceWorker;

impl MaintenanceWorker {
    pub async fn start(
        storage: Arc<StorageEngine>,
        invalidation_tx: mpsc::Sender<InvalidationEvent>,
    ) {
        let cycle_duration = Duration::from_secs(10);
        let inactivity_threshold_ms = 5000;

        loop {
            if storage.emergency_maintenance_trigger.load(Ordering::Acquire) {
                println!("🚨 [Maintenance] EMERGENCY TRIGGER: Volatile Cache at limit. Starting aggressive maintenance (OOM Guard).");
                storage
                    .emergency_maintenance_trigger
                    .store(false, Ordering::Release);
                Self::run_maintenance_cycle(&storage, &invalidation_tx).await;
            }

            sleep(cycle_duration).await;

            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            let last_activity = storage.last_query_timestamp.load(Ordering::Acquire);

            if now - last_activity > inactivity_threshold_ms {
                Self::run_maintenance_cycle(&storage, &invalidation_tx).await;
            }
        }
    }

    async fn run_maintenance_cycle(
        storage: &Arc<StorageEngine>,
        invalidation_tx: &mpsc::Sender<InvalidationEvent>,
    ) {
        println!("🌙 [Maintenance] Starting maintenance cycle (Memory cleanup)...");

        let mut to_consolidate = Vec::new();
        let mut to_purge: Vec<(u64, bool, Option<String>)> = Vec::new(); // (id, is_invalidated, slashed_role)
        let mut compression_candidates: Vec<UnifiedNode> = Vec::new();

        {
            // ── Stage 0: ConsistencyBuffer Decay ──
            let stats = &storage.consistency_buffer.stats;
            let resolved = stats.pending_to_resolved.load(Ordering::Relaxed) as f64;
            let decayed = stats.pending_to_decayed.load(Ordering::Relaxed) as f64;
            let total = resolved + decayed;

            let _shrinks_deadline = total > 10.0 && (decayed / total) > 0.7;

            let mut buffer = storage.consistency_buffer.records.write();
            let mut keys_to_resolve = Vec::new();
            let mut keys_to_purge = Vec::new();

            for (&id, record) in buffer.iter_mut() {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;

                let mut best_confidence = -1.0;
                for cand in &record.candidates {
                    if cand.confidence_score() > best_confidence {
                        best_confidence = cand.confidence_score();
                    }
                }

                if now > record.resolution_deadline_ms {
                    keys_to_resolve.push(id);
                } else if best_confidence < 0.2 {
                    keys_to_purge.push(id);
                } else {
                    if _shrinks_deadline {
                        record.resolution_deadline_ms =
                            record.resolution_deadline_ms.saturating_sub(100);
                    }
                    for cand in &mut record.candidates {
                        cand.confidence_score *= 0.9;
                    }
                }
            }

            let discarded = keys_to_purge.len() as u64;
            if discarded > 0 {
                storage
                    .consistency_buffer
                    .stats
                    .pending_to_decayed
                    .fetch_add(discarded, Ordering::Relaxed);
            }

            let mut winners_to_insert = Vec::new();
            let mut losers_to_log = Vec::new();

            for id in keys_to_purge {
                if let Some(purged) = buffer.remove(&id) {
                    storage.admission_filter.block_record(id);
                    for cand in purged.candidates {
                        if cand.importance > 0.8 {
                            losers_to_log.push((
                                id,
                                cand.id,
                                "Consistency Decay: Total expiration".to_string(),
                            ));
                        }
                    }
                }
            }

            for id in keys_to_resolve {
                if let Some(mut record) = buffer.remove(&id) {
                    storage
                        .consistency_buffer
                        .stats
                        .pending_to_resolved
                        .fetch_add(1, Ordering::Relaxed);

                    let mut best_idx = 0;
                    let mut best_confidence = -1.0;
                    for (i, cand) in record.candidates.iter().enumerate() {
                        if cand.confidence_score() > best_confidence {
                            best_confidence = cand.confidence_score();
                            best_idx = i;
                        }
                    }

                    if !record.candidates.is_empty() {
                        let winner = record.candidates.remove(best_idx);
                        winners_to_insert.push(winner);

                        for cand in record.candidates {
                            losers_to_log.push((
                                id,
                                cand.id,
                                "Consistency Resolution: Rejected candidate".to_string(),
                            ));
                        }
                    }
                }
            }

            drop(buffer);

            for winner in winners_to_insert {
                let _ = storage.insert(&winner);
            }

            use crate::governance::AuditableTombstone;
            if !losers_to_log.is_empty() {
                if let Some(cf_shadow) = storage.db.cf_handle("tombstone_storage") {
                    for (id, hash, reason) in losers_to_log {
                        let tomb = AuditableTombstone::new(id, reason, hash);
                        let key = id.to_le_bytes();
                        if let Ok(tomb_val) = bincode::serialize(&tomb) {
                            let _ = storage.db.put_cf(&cf_shadow, key, &tomb_val);
                        }
                    }
                }
            }
        }

        let total_nodes;

        {
            // ── Stage 1 & 2: Eviction & Persistence Evaluation ──
            let mut cache = storage.volatile_cache.write();
            total_nodes = cache.len();
            let max_priority_shielded = (total_nodes as f32 * 0.05).ceil() as usize;
            let mut current_shielded = 0;

            let mut keys_to_remove = Vec::new();

            let caps = crate::hardware::HardwareCapabilities::global();
            let max_consolidations = match caps.profile {
                crate::hardware::HardwareProfile::Enterprise => 5000,
                crate::hardware::HardwareProfile::Performance => 500,
                crate::hardware::HardwareProfile::Survival => 50,
            };

            for (&id, node) in cache.iter_mut() {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;

                if node.flags.is_set(NodeFlags::RECOVERED) {
                    keys_to_remove.push(id);
                    continue;
                }
                if now - storage.last_query_timestamp.load(Ordering::Acquire) < 5000 {
                    println!(
                        "🔌 [Maintenance] Cycle interrupted (I/O activity detected)."
                    );
                    break;
                }

                if node.importance >= 0.8 && current_shielded < max_priority_shielded {
                    current_shielded += 1;
                    continue;
                }

                if node.flags.is_set(NodeFlags::INVALIDATED) {
                    println!("🧨 [Maintenance] Invalidated node detected: {}. Purging immediately.", id);
                    keys_to_remove.push(id);
                    let slashed_role: Option<String> = node
                        .relational
                        .get("_owner_role")
                        .and_then(|v: &crate::node::FieldValue| v.as_str())
                        .map(|s: &str| s.to_string());
                    to_purge.push((id, true, slashed_role));
                    continue;
                }

                node.hits = (node.hits as f32 * 0.5) as u32;

                if node.confidence_score() < 0.2 {
                    keys_to_remove.push(id);
                    to_purge.push((id, false, None));
                } else if node.hits < 10 && !node.is_pinned() && (now - node.last_accessed > 60_000)
                {
                    keys_to_remove.push(id);

                    if node.hits < 5 {
                        compression_candidates.push(node.clone());
                    }

                    if to_consolidate.len() < max_consolidations {
                        to_consolidate.push(node.clone());
                    } else {
                        keys_to_remove.pop();
                    }
                }
            }

            for id in keys_to_remove {
                cache.remove(&id);
            }
        }

        for node in &to_consolidate {
            if let Err(e) = storage.consolidate_node(node) {
                eprintln!("⚠️ [Maintenance] Error consolidating node {}: {}", node.id, e);
            }
        }

        let mut deleted_count = 0usize;
        for (id, is_invalidated, slashed_role) in &to_purge {
            if *is_invalidated {
                if let Some(role) = slashed_role {
                    {
                        let mut tracker = storage.conflict_resolver.collision_tracker.write();
                        tracker.slash_origin(&role);
                    }
                    storage.admission_filter.block_role(&role);
                    println!("🔥 [Maintenance] Origin Slashing: agent '{}' blocked → ConfidenceScore=0.0", role);
                }

                InvalidationDispatcher::emit_invalidated_purged(
                    invalidation_tx,
                    *id,
                    "Flagged INVALIDATED during maintenance cycle".to_string(),
                )
                .await;
                let _ = storage.delete(*id, "Reactive Purge: INVALIDATED flag");
            } else {
                let _ = storage.delete(*id, "Low Confidence Eviction (Score < 0.2)");
            }
            deleted_count += 1;
        }

        if !compression_candidates.is_empty() {
            Self::execute_data_compression(storage, &compression_candidates).await;
        }

        if deleted_count > 10_000 {
            println!("🧹 [Maintenance] Triggering disk compaction due to high tombstone volume.");
            let mut c_opts = CompactOptions::default();
            c_opts.set_exclusive_manual_compaction(false);
            storage
                .db
                .compact_range_opt(None::<&[u8]>, None::<&[u8]>, &c_opts);
        }

        println!(
            "☀️  [Maintenance] Cycle finished. Analyzed: {} nodes.",
            total_nodes
        );
    }

    async fn execute_data_compression(
        storage: &Arc<StorageEngine>,
        candidates: &[UnifiedNode],
    ) {
        let mut thread_groups: HashMap<u64, Vec<&UnifiedNode>> = HashMap::new();

        for node in candidates {
            if let Some(thread_edge) = node.edges.iter().find(|e| e.label == "belongs_to_thread") {
                thread_groups
                    .entry(thread_edge.target)
                    .or_default()
                    .push(node);
            }
        }

        let deadline = Instant::now();
        let llm = crate::llm::LlmClient::new();

        for (thread_id, group) in &thread_groups {
            if deadline.elapsed().as_millis() > MAX_COMPRESSION_DURATION_MS {
                println!("⏳ [Maintenance] Compression time budget reached. Deferring remaining groups.");
                break;
            }

            if group.len() < 2 {
                continue; 
            }
            let group_hit_sum: u32 = group.iter().map(|n| n.hits).sum();
            if group_hit_sum < MIN_GROUP_WEIGHT_FOR_COMPRESSION {
                continue; 
            }

            let node_refs: Vec<&UnifiedNode> = group.to_vec();
            let summary_text = match llm.summarize_context(&node_refs).await {
                Ok(text) => text,
                Err(e) => {
                    eprintln!("⚠️ [Maintenance] LLM compression failed for thread {}: {}. Skipping.", thread_id, e);
                    continue; 
                }
            };

            let summary_id = rand::random::<u64>();
            let mut summary_node = UnifiedNode::new(summary_id);
            summary_node.tier = NodeTier::Cold;
            summary_node.flags.set(NodeFlags::PINNED); 
            summary_node.importance = 0.9; 
            summary_node.confidence_score =
                group.iter().map(|n| n.confidence_score).sum::<f32>() / group.len() as f32;
            summary_node.set_field("type", FieldValue::String("Summary".to_string()));
            summary_node.set_field("content", FieldValue::String(summary_text));
            summary_node.set_field("source_thread", FieldValue::Int(*thread_id as i64));

            let ancestor_ids: Vec<String> = group.iter().map(|n| n.id.to_string()).collect();
            summary_node.set_field("ancestors", FieldValue::String(ancestor_ids.join(",")));

            if let Ok(vec) = llm
                .generate_embedding(
                    summary_node
                        .get_field("content")
                        .and_then(|f| f.as_str())
                        .unwrap_or(""),
                )
                .await
            {
                summary_node.vector = crate::node::VectorRepresentations::Full(vec);
                summary_node.flags.set(NodeFlags::HAS_VECTOR);
            }

            if let Err(e) = storage.insert_to_cf(&summary_node, "compressed_archive") {
                eprintln!("⚠️ [Maintenance] Failed to persist summary node: {}. Aborting group compression.", e);
                continue; 
            }

            for original in group {
                if let Err(e) = storage.delete(
                    original.id,
                    &format!(
                        "Data Compression: condensed into summary node {}",
                        summary_id
                    ),
                ) {
                    eprintln!(
                        "⚠️ [Maintenance] Failed to tombstone node {} during compression: {}",
                        original.id, e
                    );
                }
            }

            println!(
                "🧬 [Maintenance] Data Compression: {} nodes from thread {} → Summary Node {} (Cold).",
                group.len(), thread_id, summary_id
            );
        }
    }
}
