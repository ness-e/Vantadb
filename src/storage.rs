use crate::backend::{BackendPartition, BackendWriteOp, StorageBackend};
use crate::backends::in_memory::InMemoryBackend;
use crate::backends::rocksdb_backend::RocksDbBackend;
use crate::error::{Result, VantaError};
use crate::index::{CPIndex, IndexBackend};
use crate::node::{DiskNodeHeader, UnifiedNode};
use memmap2::{MmapMut, MmapOptions};
use parking_lot::RwLock;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn};
use zerocopy::{FromBytes, IntoBytes};

// ─── Internal Metadata Persistence ──────────────────────────

#[derive(serde::Serialize, serde::Deserialize)]
struct NodeMetadata {
    relational: crate::node::RelFields,
    edges: Vec<crate::node::Edge>,
}

// ─── VantaFile: Zero-Copy MMap Wrapper ──────────────────────

/// Magic bytes written at position 0 of the vector store file.
/// Allows format validation on open and prevents loading arbitrary binary files.
pub const VANTA_FILE_MAGIC: &[u8; 8] = b"VNTAFILE";

/// Format version for the VantaFile binary layout.
/// Increment when the DiskNodeHeader layout or write_cursor position changes.
pub const VANTA_FILE_VERSION: u32 = 1;

pub struct VantaFile {
    pub file: File,
    pub mmap: MmapMut,
    pub path: PathBuf,
    pub size: u64,
    pub write_cursor: u64,
}

// VantaFile must be Send + Sync for multi-threaded Python usage (FastAPI/Gunicorn).
// Safety: Access to VantaFile is governed by a RwLock in the StorageEngine,
// ensuring that mutation (write_header, save_cursor) and shared reads (read_header)
// never occur simultaneously across threads.
unsafe impl Send for VantaFile {}
unsafe impl Sync for VantaFile {}

impl VantaFile {
    pub fn open(path: PathBuf, initial_size: u64) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .map_err(VantaError::IoError)?;

        let mut current_size = file.metadata().map_err(VantaError::IoError)?.len();
        if current_size < 8 {
            current_size = initial_size.max(8);
            file.set_len(current_size).map_err(VantaError::IoError)?;
        }

        let mmap = unsafe {
            MmapOptions::new()
                .map_mut(&file)
                .map_err(VantaError::IoError)?
        };

        // El primer u64 del mmap es nuestro write_cursor persistente
        let write_cursor = u64::from_le_bytes(mmap[0..8].try_into().unwrap());
        let write_cursor = if write_cursor < 64 || write_cursor > current_size {
            64
        } else {
            // Ensure any existing cursor is aligned to 64
            (write_cursor + 63) & !63
        };

        Ok(Self {
            file,
            mmap,
            path,
            size: current_size,
            write_cursor,
        })
    }

    /// Guarda el cursor actual en el archivo para persistencia entre reinicios
    pub fn save_cursor(&mut self) {
        self.mmap[0..8].copy_from_slice(&self.write_cursor.to_le_bytes());
    }

    /// Read a DiskNodeHeader from a specific offset without cloning (Zero-Copy)
    pub fn read_header(&self, offset: u64) -> Option<&DiskNodeHeader> {
        let header_size = std::mem::size_of::<DiskNodeHeader>() as u64;
        if offset + header_size > self.size || !offset.is_multiple_of(64) {
            return None;
        }

        let slice = &self.mmap[offset as usize..(offset + header_size) as usize];
        DiskNodeHeader::ref_from_bytes(slice).ok()
    }

    /// Write a DiskNodeHeader to a specific offset
    pub fn write_header(&mut self, offset: u64, header: &DiskNodeHeader) -> Result<()> {
        let header_size = std::mem::size_of::<DiskNodeHeader>() as u64;

        // Alignment Check: Must be 64-byte aligned for Zero-Copy casting
        if !offset.is_multiple_of(64) {
            return Err(VantaError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "VantaFile: Misaligned header write at {} (must be 64B aligned)",
                    offset
                ),
            )));
        }

        if offset + header_size > self.size {
            return Err(VantaError::IoError(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "VantaFile: Offset out of bounds",
            )));
        }

        let dest = &mut self.mmap[offset as usize..(offset + header_size) as usize];
        dest.copy_from_slice(header.as_bytes());
        Ok(())
    }

    /// Sincroniza los cambios con el disco
    pub fn flush(&self) -> Result<()> {
        self.mmap.flush().map_err(VantaError::IoError)
    }

    /// Implementación de Warm-up Strategy (Phase 3.4)
    /// Protege capas superiores del HNSW con pre-fetching para evitar page faults iniciales.
    pub fn warmup_top_layers(&self, size: usize) {
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            unsafe {
                libc::madvise(
                    self.mmap.as_ptr() as *mut libc::c_void,
                    size.min(self.mmap.len()),
                    libc::MADV_WILLNEED,
                );
            }
        }
        #[cfg(windows)]
        {
            // En Windows, leer secuencialmente es suficiente para que el OS pre-cachee.
            let mut _sum = 0u8;
            for i in (0..size.min(self.mmap.len())).step_by(4096) {
                _sum ^= self.mmap[i];
            }
        }
    }
}

// ─── Backend Kind ──────────────────────────────────────────

/// Selects which KV backend `StorageEngine` uses.
///
/// `InMemory` replaces only the KV layer (RocksDB). VantaFile and WAL
/// are still initialized on disk at the provided path. See module docs
/// in `backends::in_memory` for details.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum BackendKind {
    #[default]
    RocksDb,
    InMemory,
}

/// Configuration for `StorageEngine` initialization.
#[derive(Debug, Clone, Default)]
pub struct EngineConfig {
    pub memory_limit: Option<u64>,
    pub force_mmap: bool,
    pub read_only: bool,
    /// Which KV backend to use. Defaults to `RocksDb`.
    pub backend_kind: BackendKind,
}

pub struct StorageEngine {
    /// Abstract KV backend. No RocksDB types leak through this field.
    pub(crate) backend: Arc<dyn StorageBackend>,
    pub hnsw: RwLock<CPIndex>,
    pub volatile_cache: RwLock<std::collections::HashMap<u64, UnifiedNode>>,
    pub admission_filter: crate::governance::admission_filter::AdmissionFilter,
    pub consistency_buffer: crate::governance::consistency::ConsistencyBuffer,
    pub conflict_resolver: crate::governance::conflict_resolver::ConflictResolver,
    pub last_query_timestamp: AtomicU64,
    pub emergency_maintenance_trigger: std::sync::atomic::AtomicBool,
    /// Path to the data directory
    pub data_dir: PathBuf,
    /// Vector Store
    pub vector_store: RwLock<VantaFile>,
    /// Write-Ahead Log for durability
    pub wal: std::sync::Arc<parking_lot::Mutex<Option<crate::wal::WalWriter>>>,
}

impl StorageEngine {
    /// Open with default configuration (backward-compatible).
    /// All existing call sites continue to work without modification.
    pub fn open(path: &str) -> Result<Self> {
        Self::open_with_config(path, None)
    }

    /// Open with explicit configuration for memory budgets and mode overrides.
    /// Used by the Python SDK to inject per-instance memory limits.
    pub fn open_with_config(path: &str, config: Option<EngineConfig>) -> Result<Self> {
        let config = config.unwrap_or_default();
        let caps = crate::hardware::HardwareCapabilities::global();

        let effective_memory = config
            .memory_limit
            .or_else(|| {
                std::env::var("VANTADB_MEMORY_LIMIT")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
            })
            .unwrap_or(caps.total_memory);

        // ── KV Backend initialization ──
        let backend: Arc<dyn StorageBackend> = match config.backend_kind {
            BackendKind::RocksDb => Arc::new(RocksDbBackend::open(path, &config)?),
            BackendKind::InMemory => Arc::new(InMemoryBackend::new()),
        };

        let data_dir = PathBuf::from(path).join("data");
        let _ = std::fs::create_dir_all(&data_dir);
        let index_path = data_dir.join("vector_index.bin");

        let use_mmap = config.force_mmap
            || caps.profile == crate::hardware::HardwareProfile::Survival
            || effective_memory < 16 * 1024 * 1024 * 1024
            || std::env::var("VANTA_FORCE_MMAP").is_ok();

        let hnsw = if let Some(loaded) = CPIndex::load_from_file(&index_path) {
            let mut idx = loaded;
            if use_mmap {
                idx.backend = IndexBackend::new_mmap(index_path.clone());
                info!(
                    backend = "mmap",
                    "HNSW Survival Mode: MMap backend activated (cold-start)"
                );
            }
            idx
        } else {
            if use_mmap {
                info!(
                    backend = "mmap",
                    "HNSW Survival Mode: MMap backend activated (fresh)"
                );
                CPIndex::with_backend(IndexBackend::new_mmap(index_path.clone()))
            } else {
                info!(
                    backend = "in-memory",
                    "HNSW Performance Mode: InMemory backend"
                );
                CPIndex::new()
            }
        };

        let vector_store_path = data_dir.join("vector_store.vanta");
        let vector_store = VantaFile::open(vector_store_path, 1024 * 1024 * 64)?;

        let wal_path = data_dir.join("vanta.wal");
        let wal_writer = crate::wal::WalWriter::open(&wal_path)?;

        let admission_filter = crate::governance::admission_filter::AdmissionFilter::new(100_000);
        let consistency_buffer = crate::governance::consistency::ConsistencyBuffer::new();
        let conflict_resolver = crate::governance::conflict_resolver::ConflictResolver::new();

        Ok(Self {
            backend,
            hnsw: RwLock::new(hnsw),
            volatile_cache: RwLock::new(std::collections::HashMap::new()),
            admission_filter,
            consistency_buffer,
            conflict_resolver,
            last_query_timestamp: AtomicU64::new(0),
            emergency_maintenance_trigger: std::sync::atomic::AtomicBool::new(false),
            data_dir,
            vector_store: RwLock::new(vector_store),
            wal: std::sync::Arc::new(parking_lot::Mutex::new(Some(wal_writer))),
        })
    }

    pub fn touch_activity(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        self.last_query_timestamp.store(now, Ordering::Release);
    }

    pub fn insert(&self, node: &UnifiedNode) -> Result<()> {
        if self.admission_filter.is_blocked(node.id) {
            return Err(VantaError::Execution(format!(
                "Node {} is blocked by AdmissionFilter (recently rejected)",
                node.id
            )));
        }

        self.touch_activity();

        let mut active_node = node.clone();
        active_node.last_accessed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        if let Some(ref mut wal_writer) = *self.wal.lock() {
            wal_writer.append(&crate::wal::WalRecord::Insert(active_node.clone()))?;
        }

        let storage_offset = {
            let mut vstore = self.vector_store.write();
            let offset = vstore.write_cursor;

            let header_size = std::mem::size_of::<DiskNodeHeader>() as u64;
            let vec_len =
                if let crate::node::VectorRepresentations::Full(ref v) = active_node.vector {
                    v.len()
                } else {
                    0
                };
            let vec_size = (vec_len * 4) as u64;
            let total_needed = offset + header_size + vec_size;

            if total_needed > vstore.size {
                let new_size = vstore.size * 2;
                vstore.file.set_len(new_size).map_err(VantaError::IoError)?;
                vstore.mmap = unsafe {
                    MmapOptions::new()
                        .map_mut(&vstore.file)
                        .map_err(VantaError::IoError)?
                };
                vstore.size = new_size;
            }

            let mut header = DiskNodeHeader::new(active_node.id);
            header.vector_offset = offset + header_size;
            header.vector_len = vec_len as u32;
            header.flags = active_node.flags.0;
            header.bitset = active_node.bitset;
            header.confidence_score = active_node.confidence_score;
            header.importance = active_node.importance;
            header.tier = match active_node.tier {
                crate::node::NodeTier::Hot => 1u8,
                crate::node::NodeTier::Cold => 0u8,
            };
            header.edge_count = active_node.edges.len() as u16;

            vstore.write_header(offset, &header)?;

            if let crate::node::VectorRepresentations::Full(ref vec) = active_node.vector {
                let vec_bytes = vec.as_bytes();
                vstore.mmap
                    [(offset + header_size) as usize..(offset + header_size + vec_size) as usize]
                    .copy_from_slice(vec_bytes);
            }

            vstore.write_cursor = (total_needed + 63) & !63; // Align next header to 64B
            vstore.save_cursor();
            offset
        };

        let key = active_node.id.to_le_bytes();
        let metadata = NodeMetadata {
            relational: active_node.relational.clone(),
            edges: active_node.edges.clone(),
        };
        let metadata_val = bincode::serialize(&metadata)
            .map_err(|e| VantaError::SerializationError(e.to_string()))?;
        self.backend
            .put(BackendPartition::Default, &key, &metadata_val)?;

        {
            let mut hnsw = self.hnsw.write();
            hnsw.add(
                active_node.id,
                active_node.bitset,
                active_node.vector.clone(),
                storage_offset,
            );
        }

        if active_node.tier == crate::node::NodeTier::Hot {
            let mut cache = self.volatile_cache.write();
            cache.insert(active_node.id, active_node.clone());

            let caps = crate::hardware::HardwareCapabilities::global();
            let cache_cap_bytes = caps.total_memory / 4;
            let approx_node_size = 1536;
            let max_nodes = (cache_cap_bytes / approx_node_size) as usize;

            if cache.len() > max_nodes {
                self.emergency_maintenance_trigger
                    .store(true, Ordering::Release);
            }
        }

        Ok(())
    }

    pub fn refresh_index(&self, node: &UnifiedNode, storage_offset: u64) {
        if node.flags.is_set(crate::node::NodeFlags::HAS_VECTOR) {
            if let crate::node::VectorRepresentations::Full(vec) = &node.vector {
                let mut index = self.hnsw.write();
                index.add(
                    node.id,
                    node.bitset,
                    crate::node::VectorRepresentations::Full(vec.clone()),
                    storage_offset,
                );
            }
        }
    }

    pub fn consolidate_node(&self, node: &UnifiedNode) -> Result<()> {
        let mut persisted = node.clone();
        persisted.tier = crate::node::NodeTier::Cold;

        let key = persisted.id.to_le_bytes();
        let val = bincode::serialize(&persisted)
            .map_err(|e| VantaError::SerializationError(e.to_string()))?;
        self.backend.put(BackendPartition::Default, &key, &val)?;

        self.refresh_index(&persisted, 0);

        {
            let mut cache = self.volatile_cache.write();
            cache.remove(&node.id);
        }

        Ok(())
    }

    pub fn insert_to_cf(&self, node: &UnifiedNode, cf_name: &str) -> Result<()> {
        let partition = Self::partition_from_cf_name(cf_name)?;
        let key = node.id.to_le_bytes();
        let val =
            bincode::serialize(node).map_err(|e| VantaError::SerializationError(e.to_string()))?;
        self.backend.put(partition, &key, &val)?;

        self.refresh_index(node, 0);
        Ok(())
    }

    pub fn get(&self, id: u64) -> Result<Option<UnifiedNode>> {
        self.touch_activity();

        {
            let mut cache = self.volatile_cache.write();
            if let Some(node) = cache.get_mut(&id) {
                if node.flags.is_set(crate::node::NodeFlags::TOMBSTONE) {
                    return Ok(None);
                }
                node.hits += 1;
                node.last_accessed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                return Ok(Some(node.clone()));
            }
        }

        let key = id.to_le_bytes();
        let metadata_res = match self.backend.get(BackendPartition::Default, &key)? {
            Some(res) => res,
            None => return Ok(None),
        };

        let metadata: NodeMetadata = bincode::deserialize(&metadata_res)
            .map_err(|e| VantaError::SerializationError(e.to_string()))?;

        let hnsw = self.hnsw.read();
        let index_node = match hnsw.nodes.get(&id) {
            Some(n) => n,
            None => return Ok(None),
        };
        let storage_offset = index_node.storage_offset;

        let vstore = self.vector_store.read();
        let header = match vstore.read_header(storage_offset) {
            Some(h) => h,
            None => return Ok(None),
        };

        if (header.flags & 0x8) != 0 {
            return Ok(None);
        }

        let vec_start = header.vector_offset as usize;
        let vec_end = vec_start + (header.vector_len as usize * 4);
        if vec_end > vstore.size as usize {
            return Ok(None);
        }

        let vec_bytes = &vstore.mmap[vec_start..vec_end];
        let f32_vec: &[f32] = unsafe {
            std::slice::from_raw_parts(vec_bytes.as_ptr() as *const f32, header.vector_len as usize)
        };

        let mut node = UnifiedNode::new(id);
        node.bitset = header.bitset;
        node.vector = crate::node::VectorRepresentations::Full(f32_vec.to_vec());
        node.relational = metadata.relational;
        node.edges = metadata.edges;
        node.confidence_score = header.confidence_score;
        node.importance = header.importance;
        node.tier = if header.tier == 1 {
            crate::node::NodeTier::Hot
        } else {
            crate::node::NodeTier::Cold
        };
        node.flags = crate::node::NodeFlags(header.flags);

        Ok(Some(node))
    }

    pub fn delete(&self, id: u64, _reason: &str) -> Result<()> {
        if let Some(ref mut wal_writer) = *self.wal.lock() {
            wal_writer.append(&crate::wal::WalRecord::Delete { id })?;
        }

        let hnsw = self.hnsw.write();
        if let Some(index_node) = hnsw.nodes.get(&id) {
            let offset = index_node.storage_offset;

            let mut vstore = self.vector_store.write();
            if let Some(mut header) = vstore.read_header(offset).cloned() {
                header.flags |= 0x8;
                vstore.write_header(offset, &header)?;
            }
        }

        self.volatile_cache.write().remove(&id);

        let key = id.to_le_bytes();
        self.backend.delete(BackendPartition::Default, &key)?;

        Ok(())
    }

    pub fn purge_permanent(&self, id: u64) -> Result<()> {
        let key = id.to_le_bytes();
        self.backend.write_batch(vec![
            BackendWriteOp::Delete {
                partition: BackendPartition::Default,
                key: key.to_vec(),
            },
            BackendWriteOp::Delete {
                partition: BackendPartition::TombstoneStorage,
                key: key.to_vec(),
            },
            BackendWriteOp::Delete {
                partition: BackendPartition::Tombstones,
                key: key.to_vec(),
            },
        ])
    }

    pub fn is_deleted(&self, id: u64) -> Result<bool> {
        let key = id.to_le_bytes();
        match self.backend.get(BackendPartition::Tombstones, &key)? {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    pub fn trigger_compaction(&self) -> Result<()> {
        let vstore = self.vector_store.write();
        let hnsw = self.hnsw.read();

        let tombstone_count = hnsw
            .nodes
            .values()
            .filter(|n| {
                if let Some(h) = vstore.read_header(n.storage_offset) {
                    (h.flags & 0x8) != 0
                } else {
                    false
                }
            })
            .count();

        let total_nodes = hnsw.nodes.len();
        if total_nodes > 0 && (tombstone_count as f32 / total_nodes as f32) > 0.20 {
            warn!(
                tombstone_pct = (tombstone_count as f32 / total_nodes as f32 * 100.0) as u32,
                "Fragmentation >20% — offline compaction triggered"
            );
        }

        Ok(())
    }

    pub fn flush(&self) -> Result<()> {
        self.backend.flush()?;
        self.save_vector_index();
        Ok(())
    }

    fn save_vector_index(&self) {
        let index_path = self.data_dir.join("vector_index.bin");
        let mut index = self.hnsw.write();

        if index.backend.is_mmap() {
            if let Err(e) = index.sync_to_mmap() {
                warn!(err = %e, "Failed to sync MMap vector index");
            }
        } else {
            if let Err(e) = index.persist_to_file(&index_path) {
                warn!(err = %e, "Failed to persist vector index to file");
            }
        }
    }

    pub fn create_life_insurance(&self, timestamp_name: &str) -> Result<()> {
        let mut save_path = std::path::PathBuf::from("./vantadb_snapshots");
        if let Ok(override_dir) = std::env::var("VANTA_BACKUP_DIR") {
            save_path = std::path::PathBuf::from(override_dir);
        }
        save_path.push(timestamp_name);

        self.backend.checkpoint(&save_path)
    }

    pub fn recover_archived_nodes(&self, summary_id: u64) -> Result<Vec<UnifiedNode>> {
        let entries = self.backend.scan(BackendPartition::TombstoneStorage)?;

        let mut recovered = Vec::new();
        for (_k, v) in &entries {
            if let Ok(mut node) = bincode::deserialize::<crate::node::UnifiedNode>(v) {
                if node
                    .edges
                    .iter()
                    .any(|e| e.target == summary_id && e.label == "belonged_to")
                {
                    node.flags.set(crate::node::NodeFlags::ACTIVE);
                    node.flags.set(crate::node::NodeFlags::RECOVERED);
                    node.tier = crate::node::NodeTier::Hot;

                    self.refresh_index(&node, 0);

                    {
                        let mut cache = self.volatile_cache.write();
                        cache.insert(node.id, node.clone());
                    }
                    recovered.push(node);
                }
            }
        }
        Ok(recovered)
    }

    // ─── Delegation methods for external modules ────────────────
    //
    // These replace direct `storage.db.{cf_handle, put_cf, ...}` access
    // from executor.rs and maintenance_worker.rs.

    /// Write a value to a specific backend partition.
    ///
    /// Used by Executor (Collapse) and MaintenanceWorker to write
    /// auditable tombstones to `TombstoneStorage`.
    pub(crate) fn put_to_partition(
        &self,
        partition: BackendPartition,
        key: &[u8],
        value: &[u8],
    ) -> Result<()> {
        self.backend.put(partition, key, value)
    }

    /// Request backend compaction.
    ///
    /// Used by MaintenanceWorker after high tombstone volume.
    /// No-op for backends that don't support compaction.
    pub(crate) fn request_compaction(&self) {
        self.backend.compact();
    }

    // ─── Internal helpers ───────────────────────────────────────

    /// Translate a string-based CF name to a `BackendPartition`.
    /// Temporary compatibility bridge for `insert_to_cf`.
    fn partition_from_cf_name(cf_name: &str) -> Result<BackendPartition> {
        match cf_name {
            "default" => Ok(BackendPartition::Default),
            "tombstone_storage" => Ok(BackendPartition::TombstoneStorage),
            "compressed_archive" => Ok(BackendPartition::CompressedArchive),
            "tombstones" => Ok(BackendPartition::Tombstones),
            other => Err(VantaError::Execution(format!(
                "Unknown column family: '{}'",
                other
            ))),
        }
    }

    pub fn emergency_shutdown(&self, reason: &str, stmt: Option<&str>) -> ! {
        println!("\n=======================================================");
        println!("🔥 VANTADB SYSTEM EMERGENCY: Security Constraint Violated 🔥");
        println!("=======================================================");
        println!("Reason: {}", reason);
        if let Some(s) = stmt {
            println!("Offending Transaction: {}", s);
        }

        println!("Attempting controlled flush...");
        if let Err(e) = self.flush() {
            eprintln!(
                "CRITICAL ERROR: Failed to flush buffers during shutdown: {}",
                e
            );
        } else {
            println!("Buffers flushed successfully.");
        }
        std::process::exit(1);
    }
}
