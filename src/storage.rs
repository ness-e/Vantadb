use rocksdb::{Options, DB, WriteBatch, FlushOptions};
use rocksdb::checkpoint::Checkpoint;
use std::env;
use crate::error::{ConnectomeError, Result};
use crate::node::UnifiedNode;
use crate::index::CPIndex;
use crate::governance::AuditableTombstone;
use std::sync::RwLock;

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct StorageEngine {
    pub db: DB, // Expuesto pub temporalmente si se necesita compactación desde sleep_worker
    pub hnsw: RwLock<CPIndex>,
    pub cortex_ram: RwLock<std::collections::HashMap<u64, UnifiedNode>>,
    pub last_query_timestamp: AtomicU64,
}

impl StorageEngine {
    pub fn open(path: &str) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts.set_max_background_jobs(4);
        opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        
        opts.set_write_buffer_size(128 * 1024 * 1024); // 128MB
        opts.set_max_write_buffer_number(4);

        // Optimización Bloom Filter & Block Cache
        let mut bopts = rocksdb::BlockBasedOptions::default();
        bopts.set_bloom_filter(10.0, false);
        bopts.set_block_cache(&rocksdb::Cache::new_lru_cache(2 * 1024 * 1024 * 1024)); // 2GB
        opts.set_block_based_table_factory(&bopts);
        
        let cfs = vec!["default", "shadow_kernel", "tombstones"];
        let db = DB::open_cf(&opts, path, cfs).map_err(|e| ConnectomeError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
        
        Ok(Self { 
            db,
            hnsw: RwLock::new(CPIndex::new()),
            cortex_ram: RwLock::new(std::collections::HashMap::new()),
            last_query_timestamp: AtomicU64::new(
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
            )
        })
    }

    pub fn touch_activity(&self) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
        self.last_query_timestamp.store(now, Ordering::Release);
    }

    pub fn insert(&self, node: &UnifiedNode) -> Result<()> {
        self.touch_activity();

        // Creamos un clon ejecutable para actualizar metadatos de actividad antes de persistir
        let mut active_node = node.clone();
        active_node.last_accessed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;

        // 1. Durabilidad: Inserción directa (Write-Through)
        let key = active_node.id.to_le_bytes();
        let val = bincode::serialize(&active_node).map_err(|e| ConnectomeError::SerializationError(e.to_string()))?;
        self.db.put(&key, &val).map_err(|e| ConnectomeError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        // 2. L1 Cache / STN Storage
        if active_node.neuron_type == crate::node::NeuronType::STNeuron {
            let mut cache = self.cortex_ram.write().unwrap();
            cache.insert(active_node.id, active_node.clone());
        }

        // 3. In-Memory Index Tracking (HNSW)
        if active_node.flags.is_set(crate::node::NodeFlags::HAS_VECTOR) {
            if let crate::node::VectorData::F32(vec) = &active_node.vector {
                let mut index = self.hnsw.write().unwrap();
                index.add(active_node.id, 0, Some(vec.clone())); // MVP mask 0
            }
        }

        Ok(())
    }

    pub fn get(&self, id: u64) -> Result<Option<UnifiedNode>> {
        self.touch_activity();

        // 1. Buscar en L1 Cache (cortex_ram)
        {
            let mut cache = self.cortex_ram.write().unwrap();
            if let Some(node) = cache.get_mut(&id) {
                // Verificar si es una lápida en RAM (Invalidation check)
                if node.flags.is_tombstone() {
                    return Ok(None);
                }
                // Actualizar Heurísticas Base
                node.hits += 1;
                node.last_accessed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
                return Ok(Some(node.clone()));
            }
        }

        // 2. Si es Miss de RAM, Buscar en LTN (RocksDB)
        let key = id.to_le_bytes();
        match self.db.get_pinned(&key) {
            Ok(Some(slice)) => {
                let mut node: UnifiedNode = bincode::deserialize(&slice).map_err(|e| ConnectomeError::SerializationError(e.to_string()))?;
                
                // --- Dynamic Memory Promotion (Fase 20.5) ---
                // Si el nodo es popular (hits > 50), lo promovemos a RAM (STN)
                if node.hits >= 50 {
                    node.neuron_type = crate::node::NeuronType::STNeuron;
                    let mut cache = self.cortex_ram.write().unwrap();
                    cache.insert(node.id, node.clone());
                }
                
                Ok(Some(node))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(ConnectomeError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))),
        }
    }

    pub fn delete(&self, id: u64, reason: &str) -> Result<()> {
        if let Some(node) = self.get(id)? {
            let key = id.to_le_bytes();
            let val = bincode::serialize(&node).unwrap();

            use std::hash::{Hash, Hasher};
            use std::collections::hash_map::DefaultHasher;
            let mut hasher = DefaultHasher::new();
            val.hash(&mut hasher);
            let hash = hasher.finish();

            let tomb = AuditableTombstone::new(id, reason, hash);
            let tomb_val = bincode::serialize(&tomb).unwrap();

            // 1. Invalidation: Marcar como Tombstone en RAM (Evita lecturas zombies)
            {
                let mut cache = self.cortex_ram.write().unwrap();
                let mut tomb_node = node.clone();
                tomb_node.flags.set(crate::node::NodeFlags::TOMBSTONE);
                cache.insert(id, tomb_node);
            }

            let mut batch = WriteBatch::default();
            
            let cf_default = self.db.cf_handle("default").unwrap();
            let cf_shadow = self.db.cf_handle("shadow_kernel").unwrap();
            let cf_tomb = self.db.cf_handle("tombstones").unwrap();

            batch.put_cf(&cf_shadow, &key, &val);
            batch.put_cf(&cf_tomb, &key, &tomb_val);
            batch.delete_cf(&cf_default, &key);

            self.db.write(batch).map_err(|e| ConnectomeError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
            Ok(())
        } else {
            Ok(())
        }
    }

    pub fn purge_permanent(&self, id: u64) -> Result<()> {
        let key = id.to_le_bytes();
        let mut batch = WriteBatch::default();
        let cf_default = self.db.cf_handle("default").unwrap();
        let cf_shadow = self.db.cf_handle("shadow_kernel").unwrap();
        let cf_tomb = self.db.cf_handle("tombstones").unwrap();

        batch.delete_cf(&cf_default, &key);
        batch.delete_cf(&cf_shadow, &key);
        batch.delete_cf(&cf_tomb, &key);
        
        self.db.write(batch).map_err(|e| ConnectomeError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
        Ok(())
    }

    pub fn is_tombstoned(&self, id: u64) -> Result<bool> {
        let key = id.to_le_bytes();
        let cf_tomb = self.db.cf_handle("tombstones").unwrap();
        match self.db.get_cf(&cf_tomb, &key) {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(e) => Err(ConnectomeError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))),
        }
    }

    pub fn flush(&self) -> Result<()> {
        let mut flush_opt = FlushOptions::default();
        flush_opt.set_wait(true);
        self.db.flush_opt(&flush_opt).map_err(|e| ConnectomeError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
        Ok(())
    }

    pub fn create_life_insurance(&self, timestamp_name: &str) -> Result<()> {
        let cp = Checkpoint::new(&self.db).map_err(|e| ConnectomeError::IoError(std::io::Error::new(std::io::ErrorKind::Other, format!("Error creando inicialización de Checkpoint: {}", e))))?;
        
        let mut save_path = std::path::PathBuf::from("./connectome_snapshots");
        if let Ok(override_dir) = env::var("CONNECTOME_BACKUP_DIR") {
            save_path = std::path::PathBuf::from(override_dir);
        }
        save_path.push(timestamp_name);
        
        // Crear directorio padre si no existe (RocksDB requiere que el padre exista pero el destino no)
        if let Some(parent) = save_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        cp.create_checkpoint(&save_path).map_err(|e| {
            ConnectomeError::IoError(std::io::Error::new(std::io::ErrorKind::Other, format!("Error escribiendo Life Insurance Checkpoint: {}", e)))
        })?;
        
        Ok(())
    }

    /// Dispara un estado de pánico del sistema controlado para proteger el grafo.
    /// Frena la ejecución, sincroniza logs a disco, emite el rastro y termina el proceso.
    pub fn trigger_panic_state(&self, reason: &str, stmt: Option<&str>) -> ! {
        println!("\n=======================================================");
        println!("🔥 CONNECTOMEDB KERNEL PANIC: Security Axiom Violated 🔥");
        println!("=======================================================");
        println!("Reason: {}", reason);
        if let Some(s) = stmt {
            println!("Offending Transaction: {}", s);
        }
        
        println!("Attempting controlled WAL flush...");
        if let Err(e) = self.flush() {
            eprintln!("CRITICAL ERROR: Failed to flush OS buffers during panic: {}", e);
        } else {
            println!("Buffers successfully flushed to disk. Graph state secured.");
        }
        println!("System halted to prevent database corruption.");
        std::process::exit(1);
    }
}
