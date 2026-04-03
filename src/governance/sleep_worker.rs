use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use crate::storage::StorageEngine;
use crate::node::CognitiveUnit;
use std::sync::atomic::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};
use rocksdb::CompactOptions;

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

        let total_nodes;

        {
            // Acquire Write Lock transitorio sobre Cortex RAM
            let mut cortex = storage.cortex_ram.write().unwrap();
            total_nodes = cortex.len();

            let mut keys_to_remove = Vec::new();

            for (&id, node) in cortex.iter_mut() {
                // Check if activity broke the sleep
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
                if now - storage.last_query_timestamp.load(Ordering::Acquire) < 5000 {
                    println!("🔌 [Circadian] Interrupción de Fase REM (Actividad de I/O detectada).");
                    break; // Yield to processing loop
                }

                // 1. Olvido Bayesiano
                node.hits = (node.hits as f32 * 0.5) as u32;

                // 2. Evaluaciones de Supervivencia
                if node.trust_score() < 0.2 {
                    // Criterio de Eliminación Permanente
                    keys_to_remove.push(id);
                    to_purge.push(id);
                } else if node.hits < 10 && !node.is_pinned() && (now - node.last_accessed > 60_000) {
                    // Consolidación (STN -> LTN) - Solo si ha pasado el periodo de gracia de 60s
                    keys_to_remove.push(id);
                    to_consolidate.push(node.clone());
                }
            }

            // Cleanup RAM
            for id in keys_to_remove {
                cortex.remove(&id);
            }
        }

        // Ejecutar las mutaciones físicas (fuera del Write Lock del HashMap para prevenir latencias)
        for node in to_consolidate {
            // Persistir los bits decaídos en RocksDB (LTN)
            let key = node.id.to_le_bytes();
            if let Ok(val) = bincode::serialize(&node) {
                let _ = storage.db.put(&key, &val);
            }
        }

        let mut deleted_count = 0usize;
        for id in to_purge {
            let _ = storage.delete(id, "Olvido Bayesiano (Trust < 0.2)");
            deleted_count += 1;
        }

        if deleted_count > 100 {
            // Higiene Física: Compactación Selectiva
            println!("🧹 [Circadian] Desencadenando compactación de disco debido a alta entropía.");
            let mut c_opts = CompactOptions::default();
            c_opts.set_exclusive_manual_compaction(false);
            storage.db.compact_range_opt(None::<&[u8]>, None::<&[u8]>, &c_opts);
        }

        println!("☀️  [Circadian] Fase REM finalizada. Analizados: {} STN.", total_nodes);
    }
}
