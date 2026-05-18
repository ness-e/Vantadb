use crate::backend::{BackendPartition, BackendWriteOp, StorageBackend};
use crate::backends::fjall_backend::FjallBackend;
use crate::backends::in_memory::InMemoryBackend;
use crate::backends::rocksdb_backend::RocksDbBackend;
use crate::error::{Result, VantaError};
use crate::index::{CPIndex, IndexBackend};
use crate::node::{DiskNodeHeader, UnifiedNode};
use memmap2::{Mmap, MmapMut, MmapOptions};
use parking_lot::RwLock;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
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

#[cfg(target_os = "linux")]
fn mapped_file_resident_bytes(path: &Path) -> Option<u64> {
    let canonical = path.canonicalize().ok()?;
    let needle = canonical.to_string_lossy();
    let smaps = std::fs::read_to_string("/proc/self/smaps").ok()?;
    let mut in_mapping = false;
    let mut resident_bytes = 0u64;

    for line in smaps.lines() {
        if line.contains('-') && line.split_whitespace().count() >= 5 {
            in_mapping = line
                .split_whitespace()
                .last()
                .is_some_and(|candidate| candidate == needle);
            continue;
        }
        if in_mapping {
            if let Some(rest) = line.strip_prefix("Rss:") {
                if let Some(kb) = rest
                    .split_whitespace()
                    .next()
                    .and_then(|value| value.parse::<u64>().ok())
                {
                    resident_bytes += kb * 1024;
                }
            }
        }
    }

    Some(resident_bytes)
}

#[cfg(not(target_os = "linux"))]
fn mapped_file_resident_bytes(_path: &Path) -> Option<u64> {
    None
}

fn engine_mmap_resident_bytes(hnsw: &CPIndex, vector_store: &VantaFile) -> Option<u64> {
    let mut total = None;
    for resident in [
        vector_store.mmap_resident_bytes(),
        hnsw.backend
            .mmap_path()
            .and_then(mapped_file_resident_bytes),
    ]
    .into_iter()
    .flatten()
    {
        total = Some(total.unwrap_or(0) + resident);
    }
    total
}

pub struct VantaFile {
    pub file: File,
    mmap: VantaFileMap,
    pub path: PathBuf,
    pub size: u64,
    pub write_cursor: u64,
    read_only: bool,
}

enum VantaFileMap {
    ReadOnly(Mmap),
    ReadWrite(MmapMut),
}

impl VantaFileMap {
    fn as_slice(&self) -> &[u8] {
        match self {
            VantaFileMap::ReadOnly(mmap) => mmap,
            VantaFileMap::ReadWrite(mmap) => mmap,
        }
    }

    fn as_mut_slice(&mut self) -> Result<&mut [u8]> {
        match self {
            VantaFileMap::ReadOnly(_) => Err(VantaError::Execution(
                "VantaFile is read-only; write operation rejected".to_string(),
            )),
            VantaFileMap::ReadWrite(mmap) => Ok(mmap),
        }
    }

    fn flush(&self) -> Result<()> {
        match self {
            VantaFileMap::ReadOnly(_) => Ok(()),
            VantaFileMap::ReadWrite(mmap) => mmap.flush().map_err(VantaError::IoError),
        }
    }
}

// VantaFile must be Send + Sync for multi-threaded Python usage (FastAPI/Gunicorn).
// Safety: Access to VantaFile is governed by a RwLock in the StorageEngine,
// ensuring that mutation (write_header, save_cursor) and shared reads (read_header)
// never occur simultaneously across threads.
unsafe impl Send for VantaFile {}
unsafe impl Sync for VantaFile {}

impl VantaFile {
    pub fn open(path: PathBuf, initial_size: u64) -> Result<Self> {
        Self::open_with_mode(path, initial_size, false)
    }

    pub fn open_read_only(path: PathBuf) -> Result<Self> {
        Self::open_with_mode(path, 0, true)
    }

    fn open_with_mode(path: PathBuf, initial_size: u64, read_only: bool) -> Result<Self> {
        let file = if read_only {
            OpenOptions::new()
                .read(true)
                .open(&path)
                .map_err(VantaError::IoError)?
        } else {
            OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(&path)
                .map_err(VantaError::IoError)?
        };

        let mut current_size = file.metadata().map_err(VantaError::IoError)?.len();
        if current_size < 8 {
            if read_only {
                return Err(VantaError::Execution(format!(
                    "VantaFile {} is too small for read-only open",
                    path.display()
                )));
            }
            current_size = initial_size.max(8);
            file.set_len(current_size).map_err(VantaError::IoError)?;
        }

        let mmap = if read_only {
            VantaFileMap::ReadOnly(unsafe {
                MmapOptions::new().map(&file).map_err(VantaError::IoError)?
            })
        } else {
            VantaFileMap::ReadWrite(unsafe {
                MmapOptions::new()
                    .map_mut(&file)
                    .map_err(VantaError::IoError)?
            })
        };

        // The first u64 of the mmap is our persistent write_cursor
        let write_cursor = u64::from_le_bytes(mmap.as_slice()[0..8].try_into().unwrap());
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
            read_only,
        })
    }

    /// Saves the current cursor in the file for persistence between restarts
    pub fn save_cursor(&mut self) -> Result<()> {
        let write_cursor = self.write_cursor.to_le_bytes();
        self.mmap.as_mut_slice()?[0..8].copy_from_slice(&write_cursor);
        Ok(())
    }

    pub fn mmap_bytes(&self) -> &[u8] {
        self.mmap.as_slice()
    }

    fn mmap_bytes_mut(&mut self) -> Result<&mut [u8]> {
        self.mmap.as_mut_slice()
    }

    fn remap_mut(&mut self) -> Result<()> {
        if self.read_only {
            return Err(VantaError::Execution(
                "VantaFile is read-only; remap operation rejected".to_string(),
            ));
        }
        self.mmap = VantaFileMap::ReadWrite(unsafe {
            MmapOptions::new()
                .map_mut(&self.file)
                .map_err(VantaError::IoError)?
        });
        Ok(())
    }

    /// Read a DiskNodeHeader from a specific offset without cloning (Zero-Copy)
    pub fn read_header(&self, offset: u64) -> Option<&DiskNodeHeader> {
        let header_size = std::mem::size_of::<DiskNodeHeader>() as u64;
        if offset + header_size > self.size || !offset.is_multiple_of(64) {
            return None;
        }

        let slice = &self.mmap_bytes()[offset as usize..(offset + header_size) as usize];
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

        let dest = &mut self.mmap_bytes_mut()?[offset as usize..(offset + header_size) as usize];
        dest.copy_from_slice(header.as_bytes());
        Ok(())
    }

    /// Synchronizes changes to disk
    pub fn flush(&self) -> Result<()> {
        self.mmap.flush()
    }

    /// Warm-up Strategy Implementation (Phase 3.4)
    /// Protects upper HNSW layers with pre-fetching to avoid initial page faults.
    pub fn warmup_top_layers(&self, _size: usize) {
        #[cfg(unix)]
        {
            use memmap2::Advice;
            let _ = match &self.mmap {
                VantaFileMap::ReadOnly(mmap) => mmap.advise(Advice::WillNeed),
                VantaFileMap::ReadWrite(mmap) => mmap.advise(Advice::WillNeed),
            };
        }
        #[cfg(not(unix))]
        {
            // On platforms without madvise, sequential read to force OS caching.
            let mmap = self.mmap_bytes();
            let len = _size.min(mmap.len());
            let mut _sum = 0u8;
            for i in (0..len).step_by(4096) {
                _sum ^= mmap[i];
            }
        }
    }

    pub fn mmap_resident_bytes(&self) -> Option<u64> {
        mapped_file_resident_bytes(&self.path)
    }
}

// ─── Backend Kind ──────────────────────────────────────────

/// Selects which KV backend `StorageEngine` uses.
///
/// `InMemory` replaces only the KV layer (RocksDB). VantaFile and WAL
/// are still initialized on disk at the provided path. See module docs
/// in `backends::in_memory` for details.
pub use crate::backend::BackendKind;

use crate::config::VantaConfig;

/// Report returned by explicit ANN index rebuild operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexRebuildReport {
    pub scanned_nodes: u64,
    pub indexed_vectors: u64,
    pub skipped_tombstones: u64,
    pub duration_ms: u64,
    pub index_path: PathBuf,
    pub success: bool,
}

pub struct StorageEngine {
    /// Abstract KV backend. No RocksDB types leak through this field.
    pub(crate) backend: Arc<dyn StorageBackend>,
    /// If true, all mutating operations must be rejected.
    pub read_only: bool,
    pub hnsw: RwLock<CPIndex>,
    pub volatile_cache: RwLock<std::collections::HashMap<u64, UnifiedNode>>,
    #[cfg(feature = "governance")]
    pub admission_filter: crate::governance::admission_filter::AdmissionFilter,
    #[cfg(feature = "governance")]
    pub consistency_buffer: crate::governance::consistency::ConsistencyBuffer,
    #[cfg(feature = "governance")]
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
    pub fn open_with_config(path: &str, config: Option<VantaConfig>) -> Result<Self> {
        let startup_started = Instant::now();
        let config = config.unwrap_or_default();
        let caps = crate::hardware::HardwareCapabilities::global();

        let effective_memory = config.memory_limit.unwrap_or(caps.total_memory);
        let base_path = PathBuf::from(path);

        if config.read_only && !base_path.exists() {
            return Err(VantaError::Execution(format!(
                "StorageEngine read-only open requires an existing database path: {}",
                base_path.display()
            )));
        }
        if !config.read_only {
            std::fs::create_dir_all(&base_path).map_err(VantaError::IoError)?;
        }

        // ── KV Backend initialization ──
        let backend: Arc<dyn StorageBackend> = match config.backend_kind {
            BackendKind::RocksDb => Arc::new(RocksDbBackend::open(path, &config)?),
            BackendKind::Fjall => Arc::new(FjallBackend::open(path, &config)?),
            BackendKind::InMemory => Arc::new(InMemoryBackend::new()),
        };

        let data_dir = base_path.join("data");
        if config.read_only && !data_dir.exists() {
            return Err(VantaError::Execution(format!(
                "StorageEngine read-only open requires an existing data directory: {}",
                data_dir.display()
            )));
        }
        if !config.read_only {
            std::fs::create_dir_all(&data_dir).map_err(VantaError::IoError)?;
        }
        let index_path = data_dir.join("vector_index.bin");

        let use_mmap = config.force_mmap
            || caps.profile == crate::hardware::HardwareProfile::LowResource
            || effective_memory < 16 * 1024 * 1024 * 1024;

        let mut hnsw = if let Some(loaded) = CPIndex::load_from_file(&index_path) {
            let mut idx = loaded;
            if use_mmap {
                idx.backend = IndexBackend::new_mmap(index_path.clone());
                info!(
                    backend = "mmap",
                    "HNSW Resource Governance: MMap backend activated (cold-start)"
                );
            }
            idx
        } else {
            if use_mmap {
                info!(
                    backend = "mmap",
                    "HNSW Resource Governance: MMap backend activated (fresh)"
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
        let mut vector_store = if config.read_only {
            VantaFile::open_read_only(vector_store_path)?
        } else {
            VantaFile::open(vector_store_path, 1024 * 1024 * 64)?
        };

        // ── Index Reconstruction: rebuild HNSW if index file is missing ──────
        if hnsw.nodes.is_empty() {
            let report =
                Self::rebuild_hnsw_from_vstore(&mut hnsw, &vector_store, index_path.clone())?;
            crate::metrics::record_ann_rebuild(report.duration_ms, report.scanned_nodes);
            if report.scanned_nodes > 0 {
                info!(
                    scanned_nodes = report.scanned_nodes,
                    indexed_vectors = report.indexed_vectors,
                    skipped_tombstones = report.skipped_tombstones,
                    duration_ms = report.duration_ms,
                    "Index reconstructed from VantaFile"
                );
            }
        }

        // ── WAL Replay: recover un-flushed mutations ──────────────
        let wal_path = data_dir.join("vanta.wal");
        let mut wal_replay_ms = 0u64;
        let mut wal_records_replayed = 0u64;
        if !config.read_only && wal_path.exists() {
            let wal_replay_started = Instant::now();
            let mut wal_reader = crate::wal::WalReader::open(&wal_path)?;
            while let Some(record) = wal_reader.next_record()? {
                wal_records_replayed += 1;
                match record {
                    crate::wal::WalRecord::Insert(node) => {
                        let offset = Self::write_node_to_vstore(&mut vector_store, &node)?;
                        hnsw.add(node.id, node.bitset, node.vector.clone(), offset);
                    }
                    crate::wal::WalRecord::Update { id, node } => {
                        let offset = Self::write_node_to_vstore(&mut vector_store, &node)?;
                        hnsw.add(id, node.bitset, node.vector.clone(), offset);
                    }
                    crate::wal::WalRecord::Delete { id } => {
                        if let Some(index_node) = hnsw.nodes.get(&id) {
                            let offset = index_node.storage_offset;
                            if let Some(h) = vector_store.read_header(offset).cloned() {
                                let mut tombstoned = h;
                                tombstoned.flags |= 0x8;
                                vector_store.write_header(offset, &tombstoned)?;
                            }
                        }
                    }
                    crate::wal::WalRecord::Checkpoint { .. } => {}
                }
            }
            wal_replay_ms = wal_replay_started.elapsed().as_millis() as u64;
            if wal_records_replayed > 0 {
                info!(
                    replayed = wal_records_replayed,
                    duration_ms = wal_replay_ms,
                    "WAL replay: recovered un-flushed mutations"
                );
            }
        }

        let wal_writer = if config.read_only {
            None
        } else {
            Some(crate::wal::WalWriter::open(&wal_path)?)
        };

        #[cfg(feature = "governance")]
        let admission_filter = crate::governance::admission_filter::AdmissionFilter::new(100_000);
        #[cfg(feature = "governance")]
        let consistency_buffer = crate::governance::consistency::ConsistencyBuffer::new();
        #[cfg(feature = "governance")]
        let conflict_resolver = crate::governance::conflict_resolver::ConflictResolver::new();

        crate::metrics::record_startup(
            startup_started.elapsed().as_millis() as u64,
            wal_replay_ms,
            wal_records_replayed,
        );

        // Capture initial memory breakdown after engine is fully open
        crate::metrics::record_memory_breakdown(
            hnsw.nodes.len() as u64,
            hnsw.estimate_memory_bytes() as u64,
            engine_mmap_resident_bytes(&hnsw, &vector_store),
            0, // volatile cache is empty at startup
            0, // cache cap is set later by SDK; 0 until configured
        );

        Ok(Self {
            backend,
            read_only: config.read_only,
            hnsw: RwLock::new(hnsw),
            volatile_cache: RwLock::new(std::collections::HashMap::new()),
            #[cfg(feature = "governance")]
            admission_filter,
            #[cfg(feature = "governance")]
            consistency_buffer,
            #[cfg(feature = "governance")]
            conflict_resolver,
            last_query_timestamp: AtomicU64::new(0),
            emergency_maintenance_trigger: std::sync::atomic::AtomicBool::new(false),
            data_dir,
            vector_store: RwLock::new(vector_store),
            wal: std::sync::Arc::new(parking_lot::Mutex::new(wal_writer)),
        })
    }

    #[inline]
    fn ensure_writable(&self) -> Result<()> {
        if self.read_only {
            return Err(VantaError::Execution(
                "StorageEngine is read-only; write operation rejected".to_string(),
            ));
        }
        Ok(())
    }

    pub fn touch_activity(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        self.last_query_timestamp.store(now, Ordering::Release);
    }

    fn append_to_vstore(&self, node: &UnifiedNode) -> Result<u64> {
        let mut vstore = self.vector_store.write();
        let offset = vstore.write_cursor;

        let header_size = std::mem::size_of::<DiskNodeHeader>() as u64;
        let vec_len = if let crate::node::VectorRepresentations::Full(ref v) = node.vector {
            v.len()
        } else {
            0
        };
        let vec_size = (vec_len * 4) as u64;
        let total_needed = offset + header_size + vec_size;

        if total_needed > vstore.size {
            let new_size = vstore.size * 2;
            vstore.file.set_len(new_size).map_err(VantaError::IoError)?;
            vstore.size = new_size;
            vstore.remap_mut()?;
        }

        let mut header = DiskNodeHeader::new(node.id);
        header.vector_offset = offset + header_size;
        header.vector_len = vec_len as u32;
        header.flags = node.flags.0;
        header.bitset = node.bitset;
        header.confidence_score = node.confidence_score;
        header.importance = node.importance;
        header.tier = match node.tier {
            crate::node::NodeTier::Hot => 1u8,
            crate::node::NodeTier::Cold => 0u8,
        };
        header.edge_count = node.edges.len() as u16;

        vstore.write_header(offset, &header)?;

        if let crate::node::VectorRepresentations::Full(ref vec) = node.vector {
            let vec_bytes = vec.as_bytes();
            vstore.mmap_bytes_mut()?
                [(offset + header_size) as usize..(offset + header_size + vec_size) as usize]
                .copy_from_slice(vec_bytes);
        }

        vstore.write_cursor = (total_needed + 63) & !63; // Align next header to 64B
        vstore.save_cursor()?;
        Ok(offset)
    }

    /// Write a node to VantaFile during WAL replay (before engine is fully constructed).
    fn write_node_to_vstore(vstore: &mut VantaFile, node: &UnifiedNode) -> Result<u64> {
        let offset = vstore.write_cursor;
        let header_size = std::mem::size_of::<DiskNodeHeader>() as u64;
        let vec_len = if let crate::node::VectorRepresentations::Full(ref v) = node.vector {
            v.len()
        } else {
            0
        };
        let vec_size = (vec_len * 4) as u64;
        let total_needed = offset + header_size + vec_size;

        if total_needed > vstore.size {
            let new_size = (vstore.size * 2).max(total_needed + 4096);
            vstore.file.set_len(new_size).map_err(VantaError::IoError)?;
            vstore.size = new_size;
            vstore.remap_mut()?;
        }

        let mut header = DiskNodeHeader::new(node.id);
        header.vector_offset = offset + header_size;
        header.vector_len = vec_len as u32;
        header.flags = node.flags.0;
        header.bitset = node.bitset;
        header.confidence_score = node.confidence_score;
        header.importance = node.importance;
        header.tier = match node.tier {
            crate::node::NodeTier::Hot => 1u8,
            crate::node::NodeTier::Cold => 0u8,
        };
        header.edge_count = node.edges.len() as u16;

        vstore.write_header(offset, &header)?;

        if let crate::node::VectorRepresentations::Full(ref vec) = node.vector {
            let vec_bytes = vec.as_bytes();
            vstore.mmap_bytes_mut()?
                [(offset + header_size) as usize..(offset + header_size + vec_size) as usize]
                .copy_from_slice(vec_bytes);
        }

        vstore.write_cursor = (total_needed + 63) & !63;
        vstore.save_cursor()?;
        Ok(offset)
    }

    fn fresh_index_like(existing: &CPIndex, index_path: PathBuf) -> CPIndex {
        let config = existing.config.clone();
        if existing.backend.is_mmap() {
            let mut index = CPIndex::with_backend(IndexBackend::new_mmap(index_path));
            index.config = config;
            index
        } else {
            CPIndex::new_with_config(config)
        }
    }

    fn rebuild_hnsw_from_vstore(
        hnsw: &mut CPIndex,
        vstore: &VantaFile,
        index_path: PathBuf,
    ) -> Result<IndexRebuildReport> {
        let started = Instant::now();
        let mut cursor = 64u64;
        let mut scanned_nodes = 0u64;
        let mut indexed_vectors = 0u64;
        let mut skipped_tombstones = 0u64;
        let header_size = std::mem::size_of::<DiskNodeHeader>() as u64;

        while cursor + header_size <= vstore.write_cursor {
            if let Some(header) = vstore.read_header(cursor) {
                if header.id != 0 {
                    scanned_nodes += 1;
                    if (header.flags & 0x8) != 0 {
                        skipped_tombstones += 1;
                    } else {
                        let vec_data = if header.vector_len > 0 {
                            let start = header.vector_offset as usize;
                            let end = start + (header.vector_len as usize * 4);
                            if end <= vstore.size as usize {
                                let slice = &vstore.mmap_bytes()[start..end];
                                let vec: &[f32] = unsafe {
                                    std::slice::from_raw_parts(
                                        slice.as_ptr() as *const f32,
                                        header.vector_len as usize,
                                    )
                                };
                                indexed_vectors += 1;
                                crate::node::VectorRepresentations::Full(vec.to_vec())
                            } else {
                                crate::node::VectorRepresentations::None
                            }
                        } else {
                            crate::node::VectorRepresentations::None
                        };

                        hnsw.add(header.id, header.bitset, vec_data, cursor);
                    }
                }

                let vec_size = (header.vector_len as u64 * 4 + 63) & !63;
                cursor += header_size + vec_size;
            } else {
                cursor += 64;
            }
        }

        Ok(IndexRebuildReport {
            scanned_nodes,
            indexed_vectors,
            skipped_tombstones,
            duration_ms: started.elapsed().as_millis() as u64,
            index_path,
            success: true,
        })
    }

    pub fn rebuild_vector_index(&self) -> Result<IndexRebuildReport> {
        self.ensure_writable()?;
        let index_path = self.data_dir.join("vector_index.bin");
        let mut rebuilt = {
            let hnsw = self.hnsw.read();
            Self::fresh_index_like(&hnsw, index_path.clone())
        };

        let report = {
            let vstore = self.vector_store.read();
            Self::rebuild_hnsw_from_vstore(&mut rebuilt, &vstore, index_path)?
        };

        {
            let mut hnsw = self.hnsw.write();
            *hnsw = rebuilt;
        }
        self.save_vector_index();
        crate::metrics::record_ann_rebuild(report.duration_ms, report.scanned_nodes);

        Ok(report)
    }

    pub fn insert(&self, node: &UnifiedNode) -> Result<()> {
        self.ensure_writable()?;
        #[cfg(feature = "governance")]
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

        let storage_offset = self.append_to_vstore(&active_node)?;

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
        if storage_offset < 64 || !storage_offset.is_multiple_of(64) {
            return;
        }
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
        self.ensure_writable()?;
        let mut persisted = node.clone();
        persisted.tier = crate::node::NodeTier::Cold;

        let key = persisted.id.to_le_bytes();
        let metadata = NodeMetadata {
            relational: persisted.relational.clone(),
            edges: persisted.edges.clone(),
        };
        let metadata_val = bincode::serialize(&metadata)
            .map_err(|e| VantaError::SerializationError(e.to_string()))?;
        self.backend
            .put(BackendPartition::Default, &key, &metadata_val)?;

        // Consolidate doesn't change the vector store offset if already present
        let offset = {
            let hnsw = self.hnsw.read();
            hnsw.nodes
                .get(&node.id)
                .map(|n| n.storage_offset)
                .unwrap_or(0)
        };
        self.refresh_index(&persisted, offset);

        {
            let mut cache = self.volatile_cache.write();
            cache.remove(&node.id);
        }

        Ok(())
    }

    pub fn insert_to_cf(&self, node: &UnifiedNode, cf_name: &str) -> Result<()> {
        self.ensure_writable()?;
        let partition = Self::partition_from_cf_name(cf_name)?;
        let key = node.id.to_le_bytes();
        let val =
            bincode::serialize(node).map_err(|e| VantaError::SerializationError(e.to_string()))?;
        self.backend.put(partition, &key, &val)?;

        let storage_offset = self.append_to_vstore(node)?;
        self.refresh_index(node, storage_offset);
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

        let vec_bytes = &vstore.mmap_bytes()[vec_start..vec_end];
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
        self.ensure_writable()?;
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
        self.ensure_writable()?;
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
        self.ensure_writable()?;
        self.backend.flush()?;
        self.save_vector_index();

        // Update memory breakdown after flush
        let hnsw = self.hnsw.read();
        let vector_store = self.vector_store.read();
        crate::metrics::record_memory_breakdown(
            hnsw.nodes.len() as u64,
            hnsw.estimate_memory_bytes() as u64,
            engine_mmap_resident_bytes(&hnsw, &vector_store),
            self.volatile_cache.read().len() as u64,
            0, // cache cap is tracked at SDK level
        );
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
        self.ensure_writable()?;
        if !self.supports_checkpoint() {
            return Err(VantaError::Execution(format!(
                "Checkpoint (live snapshot) is not supported by the {:?} backend. \
                Live backups are not available natively. Please use filesystem-level snapshots (e.g., EBS, ZFS, LVM) \
                or perform a cold backup by safely shutting down the database process and copying the data directory.",
                self.backend_kind()
            )));
        }

        let mut save_path = std::path::PathBuf::from("./vantadb_snapshots");
        if let Ok(override_dir) = std::env::var("VANTA_BACKUP_DIR") {
            save_path = std::path::PathBuf::from(override_dir);
        }
        save_path.push(timestamp_name);

        self.backend.checkpoint(&save_path)
    }

    pub fn recover_archived_nodes(&self, summary_id: u64) -> Result<Vec<UnifiedNode>> {
        self.ensure_writable()?;
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
                    self.insert(&node)?;
                    recovered.push(node);
                }
            }
        }
        Ok(recovered)
    }

    /// Return all currently readable nodes from the primary backend partition.
    ///
    /// This is intentionally not a hot path. It supports early product APIs
    /// such as namespace listing before secondary indexes exist.
    pub(crate) fn scan_nodes(&self) -> Result<Vec<UnifiedNode>> {
        let entries = self.backend.scan(BackendPartition::Default)?;
        let mut nodes = Vec::new();

        for (key, _value) in entries {
            if key.len() != std::mem::size_of::<u64>() {
                continue;
            }

            let mut id_bytes = [0u8; 8];
            id_bytes.copy_from_slice(&key);
            let id = u64::from_le_bytes(id_bytes);

            if let Some(node) = self.get(id)? {
                nodes.push(node);
            }
        }

        Ok(nodes)
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
        self.ensure_writable()?;
        self.backend.put(partition, key, value)
    }

    pub(crate) fn write_backend_batch(&self, ops: Vec<BackendWriteOp>) -> Result<()> {
        self.ensure_writable()?;
        self.backend.write_batch(ops)
    }

    pub(crate) fn scan_partition(
        &self,
        partition: BackendPartition,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        self.backend.scan(partition)
    }

    pub(crate) fn scan_partition_prefix(
        &self,
        partition: BackendPartition,
        prefix: &[u8],
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        self.backend.scan_prefix(partition, prefix)
    }

    pub(crate) fn get_from_partition(
        &self,
        partition: BackendPartition,
        key: &[u8],
    ) -> Result<Option<Vec<u8>>> {
        self.backend.get(partition, key)
    }

    /// Request backend compaction.
    ///
    /// Used by MaintenanceWorker after high tombstone volume.
    /// No-op for backends that don't support compaction.
    pub fn request_compaction(&self) {
        if !self.supports_manual_compaction() {
            tracing::info!(
                "Maintenance requested manual disk compaction, but it was skipped. \
                The active backend ({:?}) manages compaction automatically. This is expected behavior.",
                self.backend_kind()
            );
            return;
        }
        self.backend.compact();
    }

    pub fn backend_capabilities(&self) -> crate::backend::BackendCapabilities {
        self.backend.capabilities()
    }

    pub fn backend_kind(&self) -> crate::backend::BackendKind {
        self.backend.capabilities().kind
    }

    pub fn supports_checkpoint(&self) -> bool {
        self.backend.capabilities().supports_checkpoint
    }

    pub fn supports_manual_compaction(&self) -> bool {
        self.backend.capabilities().supports_manual_compaction
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
            "namespace_index" => Ok(BackendPartition::NamespaceIndex),
            "payload_index" => Ok(BackendPartition::PayloadIndex),
            "text_index" => Ok(BackendPartition::TextIndex),
            "internal_metadata" => Ok(BackendPartition::InternalMetadata),
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
