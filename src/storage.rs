pub use crate::backend::BackendPartition;
use crate::backend::{BackendWriteOp, StorageBackend};
use crate::backends::fjall_backend::FjallBackend;
use crate::backends::in_memory::InMemoryBackend;
use crate::backends::rocksdb_backend::RocksDbBackend;
use crate::error::{Result, VantaError};
use crate::index::{CPIndex, IndexBackend};
use crate::node::{DiskNodeHeader, UnifiedNode};
use fs2::FileExt;
use memmap2::{Mmap, MmapMut, MmapOptions};
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet, VecDeque};
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

#[cfg(unix)]
pub fn get_resident_bytes(addr: *const u8, len: usize) -> Option<u64> {
    if len == 0 || addr.is_null() {
        return Some(0);
    }

    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    let page_size = if page_size <= 0 {
        4096
    } else {
        page_size as usize
    };

    let addr_val = addr as usize;
    let aligned_addr = addr_val & !(page_size - 1);
    let offset = addr_val - aligned_addr;
    let aligned_len = (len + offset + page_size - 1) & !(page_size - 1);
    let num_pages = aligned_len / page_size;

    const CHUNK_PAGES: usize = 65536;
    let mut resident_pages = 0u64;
    let mut vec_buffer = vec![0u8; CHUNK_PAGES.min(num_pages)];

    for chunk_start_page in (0..num_pages).step_by(CHUNK_PAGES) {
        let pages_in_chunk = (num_pages - chunk_start_page).min(CHUNK_PAGES);
        let chunk_addr = (aligned_addr + chunk_start_page * page_size) as *mut libc::c_void;
        let chunk_len = pages_in_chunk * page_size;

        let vec_ptr = vec_buffer.as_mut_ptr();
        #[cfg(target_os = "macos")]
        let res = unsafe { libc::mincore(chunk_addr, chunk_len, vec_ptr as *mut libc::c_char) };
        #[cfg(not(target_os = "macos"))]
        let res = unsafe { libc::mincore(chunk_addr, chunk_len, vec_ptr) };
        if res == 0 {
            for &page_state in vec_buffer.iter().take(pages_in_chunk) {
                if (page_state & 1) != 0 {
                    resident_pages += 1;
                }
            }
        } else {
            let err = std::io::Error::last_os_error();
            warn!("mincore syscall failed: {:?}", err);
            return None;
        }
    }

    Some(resident_pages * page_size as u64)
}

#[cfg(target_os = "windows")]
pub fn get_resident_bytes(addr: *const u8, len: usize) -> Option<u64> {
    use windows_sys::Win32::System::ProcessStatus::{
        QueryWorkingSetEx, PSAPI_WORKING_SET_EX_INFORMATION,
    };
    use windows_sys::Win32::System::Threading::GetCurrentProcess;

    if len == 0 || addr.is_null() {
        return Some(0);
    }

    let page_size = 4096;
    let addr_val = addr as usize;
    let aligned_addr = addr_val & !(page_size - 1);
    let offset = addr_val - aligned_addr;
    let aligned_len = (len + offset + page_size - 1) & !(page_size - 1);
    let num_pages = aligned_len / page_size;

    const CHUNK_PAGES: usize = 65536;
    let mut resident_pages = 0u64;

    let h_process = unsafe { GetCurrentProcess() };
    let mut info_buffer = vec![
        unsafe { std::mem::zeroed::<PSAPI_WORKING_SET_EX_INFORMATION>() };
        CHUNK_PAGES.min(num_pages)
    ];

    for chunk_start_page in (0..num_pages).step_by(CHUNK_PAGES) {
        let pages_in_chunk = (num_pages - chunk_start_page).min(CHUNK_PAGES);

        for (i, info_entry) in info_buffer.iter_mut().enumerate().take(pages_in_chunk) {
            let page_addr = aligned_addr + (chunk_start_page + i) * page_size;
            info_entry.VirtualAddress = page_addr as *mut _;
            #[allow(unused_unsafe)]
            unsafe {
                info_entry.VirtualAttributes.Flags = 0;
            }
        }

        let cb = (pages_in_chunk * std::mem::size_of::<PSAPI_WORKING_SET_EX_INFORMATION>()) as u32;
        let res = unsafe { QueryWorkingSetEx(h_process, info_buffer.as_mut_ptr() as *mut _, cb) };

        if res != 0 {
            for info_entry in info_buffer.iter().take(pages_in_chunk) {
                let flags = unsafe { info_entry.VirtualAttributes.Flags };
                if (flags & 1) != 0 {
                    resident_pages += 1;
                }
            }
        } else {
            let err = std::io::Error::last_os_error();
            warn!("QueryWorkingSetEx failed: {:?}", err);
            return None;
        }
    }

    Some(resident_pages * page_size as u64)
}

#[cfg(not(any(unix, target_os = "windows")))]
pub fn get_resident_bytes(_addr: *const u8, _len: usize) -> Option<u64> {
    None
}

/// Legacy helper kept for backward compatibility.
/// Prefer `engine_mmap_resident_bytes` or `StorageEngine::get_memory_stats()` for accurate telemetry.
#[allow(dead_code)]
fn mapped_file_resident_bytes(path: &Path) -> Option<u64> {
    let file = File::open(path).ok()?;
    let mmap = unsafe { Mmap::map(&file).ok()? };
    get_resident_bytes(mmap.as_ptr(), mmap.len())
}

/// Computes the approximate Resident Set Size (RSS) of memory-mapped regions
/// used by the HNSW index backend and the vector store file.
///
/// Returns `None` if the platform does not support querying resident pages
/// (e.g., non-Unix/non-Windows), or if a syscall fails.
fn engine_mmap_resident_bytes(hnsw: &CPIndex, vector_store: &VantaFile) -> Option<u64> {
    let mut total = None;
    for resident in [
        vector_store.mmap_resident_bytes(),
        hnsw.backend.mmap_resident_bytes(),
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

    fn as_ptr(&self) -> *const u8 {
        match self {
            VantaFileMap::ReadOnly(mmap) => mmap.as_ptr(),
            VantaFileMap::ReadWrite(mmap) => mmap.as_ptr(),
        }
    }

    fn len(&self) -> usize {
        match self {
            VantaFileMap::ReadOnly(mmap) => mmap.len(),
            VantaFileMap::ReadWrite(mmap) => mmap.len(),
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
        let min_header_size = 64u64;
        if current_size < min_header_size {
            if read_only {
                return Err(VantaError::Execution(format!(
                    "VantaFile {} is too small for read-only open",
                    path.display()
                )));
            }
            current_size = initial_size.max(min_header_size);
            file.set_len(current_size).map_err(VantaError::IoError)?;
        }

        let mut mmap = if read_only {
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

        if !read_only && current_size >= min_header_size {
            let has_magic = &mmap.as_slice()[0..4] == b"VFLE";
            if !has_magic {
                let header = crate::binary_header::VantaHeader::new(*b"VFLE", 1, 0);
                mmap.as_mut_slice()?[0..16].copy_from_slice(&header.serialize());
                let initial_cursor = 64u64.to_le_bytes();
                mmap.as_mut_slice()?[16..24].copy_from_slice(&initial_cursor);
                mmap.flush()?;
            }
        }

        // Validate the binary header
        let header = crate::binary_header::VantaHeader::deserialize(&mmap.as_slice()[0..16])?;
        header.validate(*b"VFLE", 1, "VantaFile format mismatch")?;

        // The write_cursor of the mmap is stored in 16..24
        let write_cursor = u64::from_le_bytes(mmap.as_slice()[16..24].try_into().unwrap());
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
        self.mmap.as_mut_slice()?[16..24].copy_from_slice(&write_cursor);
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
        #[cfg(feature = "failpoints")]
        {
            fail::fail_point!("mmap_flush_fail", |_| {
                Err(VantaError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Injected mmap flush failure",
                )))
            });
        }
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
        get_resident_bytes(self.mmap.as_ptr(), self.mmap.len())
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

/// Memory usage statistics for a `StorageEngine` instance.
///
/// - `logical_bytes`: Estimated logical memory footprint (in-memory structures + mapped file sizes).
/// - `physical_rss`: Approximate Resident Set Size (pages actually in RAM) for mmap'd regions,
///   if the platform supports querying it (`Some(value)`), or `None` otherwise.
/// - `node_count`: Number of nodes currently indexed in HNSW.
/// - `cache_entries`: Number of "hot" nodes cached in the volatile LRU cache.
#[derive(Debug, Clone, Copy)]
pub struct MemoryStats {
    pub logical_bytes: u64,
    pub physical_rss: Option<u64>,
    pub node_count: u64,
    pub cache_entries: usize,
}

impl MemoryStats {
    /// Returns the physical RSS if available, otherwise falls back to logical estimate.
    #[inline]
    pub fn effective_bytes(&self) -> u64 {
        self.physical_rss.unwrap_or(self.logical_bytes)
    }
}

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
    pub config: VantaConfig,
    /// If true, all mutating operations must be rejected.
    pub read_only: bool,
    pub hnsw: RwLock<CPIndex>,
    /// Serializes insert/refresh operations to avoid bidirectional
    /// neighbor update races. Searches acquire hnsw.read() freely.
    insert_lock: parking_lot::Mutex<()>,
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
    /// File handle for multi-process isolation lock
    pub(crate) _lock_file: Option<File>,
    /// In-memory cache for BM25 term stats to avoid redundant I/O during ingestion.
    pub(crate) text_stats_cache:
        RwLock<HashMap<(String, String), crate::text_index::TextTermStats>>,
    /// In-memory cache for BM25 namespace stats.
    pub(crate) text_ns_cache: RwLock<HashMap<String, crate::text_index::TextNamespaceStats>>,
    /// Lightweight cardinality statistics for query optimization.
    pub(crate) cardinality_stats: RwLock<HashMap<String, HashMap<String, usize>>>,
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
        let lock_file = if !config.read_only {
            std::fs::create_dir_all(&base_path).map_err(VantaError::IoError)?;
            let lock_path = base_path.join(".vanta.lock");
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(&lock_path)
                .map_err(VantaError::IoError)?;

            file.try_lock_exclusive().map_err(|_| {
                VantaError::Execution(format!(
                    "Database at '{}' is locked by another process. \
                     Only one VantaDB instance can open a database directory at a time.",
                    base_path.display()
                ))
            })?;
            Some(file)
        } else {
            None
        };

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

        let mut hnsw = if let Some(loaded) = CPIndex::load_from_file(&index_path, use_mmap) {
            if use_mmap {
                info!(
                    backend = "mmap",
                    "HNSW Resource Governance: MMap backend activated (cold-start)"
                );
            }
            loaded
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
        let checkpoint_seq: u64 = backend
            .get(BackendPartition::InternalMetadata, b"checkpoint_seq")?
            .and_then(|bytes| bincode::deserialize::<u64>(&bytes).ok())
            .unwrap_or(0);

        if !config.read_only && wal_path.exists() {
            let wal_replay_started = Instant::now();
            let mut wal_reader = crate::wal::WalReader::open(&wal_path)?;
            let mut current_seq = 0u64;
            while let Some(record) = wal_reader.next_record()? {
                current_seq += 1;
                if current_seq <= checkpoint_seq {
                    continue;
                }
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
                    checkpoint_seq = checkpoint_seq,
                    "WAL replay: recovered un-flushed mutations"
                );
            }
        }

        let wal_writer = if config.read_only {
            None
        } else {
            Some(crate::wal::WalWriter::open(&wal_path, config.sync_mode)?)
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

        let cardinality_stats = Self::initialize_cardinality_stats(backend.as_ref());

        Ok(Self {
            config: config.clone(),
            read_only: config.read_only,
            hnsw: RwLock::new(hnsw),
            insert_lock: parking_lot::Mutex::new(()),
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
            _lock_file: lock_file,
            text_stats_cache: RwLock::new(HashMap::new()),
            text_ns_cache: RwLock::new(HashMap::new()),
            cardinality_stats: RwLock::new(cardinality_stats),
            backend,
        })
    }

    #[inline]
    pub fn guard_write_allowed(config: &VantaConfig) -> Result<()> {
        if config.read_only {
            return Err(VantaError::Execution(
                "StorageEngine is read-only; write operation rejected".to_string(),
            ));
        }
        Ok(())
    }

    #[inline]
    fn ensure_writable(&self) -> Result<()> {
        Self::guard_write_allowed(&self.config)
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

        // ── Paso 1: Flush del WAL antes de reubicar físicamente los offsets ──
        // Si no hacemos flush, los registros del WAL que referencian offsets anteriores
        // quedarán inconsistentes con el nuevo layout compactado.
        self.flush()?;

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

    /// Compacta el VantaFile (`vector_store.vanta`) reescribiendo los nodos en orden
    /// BFS (Breadth-First Search) del grafo HNSW desde el entry point.
    ///
    /// ## Objetivo
    /// Los nodos más conectados del HNSW (hubs y capas superiores) quedan ubicados
    /// en las páginas virtuales iniciales del archivo. Una búsqueda semántica
    /// accede primero a esos nodos, por lo que tras la compactación reduce
    /// drásticamente los page-faults en accesos MMap.
    ///
    /// ## Garantías
    /// - WAL debe estar vacío/flushed antes de llamar a esta función.
    /// - Los `storage_offset` de todos los nodos en el `DashMap` del HNSW se
    ///   actualizan atómicamente al finalizar el swap.
    /// - Los nodos no alcanzados por el BFS (aislados / sin vector) se añaden
    ///   al final, preservando la reachability total del índice.
    /// - Si el índice está vacío, la función retorna sin error.
    pub fn compact_layout_bfs(&self) -> Result<u64> {
        self.ensure_writable()?;

        // ── Flush previo del WAL para garantizar consistencia ────────────────
        self.flush()?;

        let started = Instant::now();

        // ── Adquirir locks exclusivos en orden determinista (evita deadlock) ──
        // Orden: vector_store → hnsw  (siempre el mismo en todo el codebase)
        let mut vstore = self.vector_store.write();
        let hnsw = self.hnsw.write();

        let entry_point_id = match hnsw.get_entry_point() {
            Some(ep) => ep,
            None => {
                // Índice vacío: nada que compactar
                info!("compact_layout_bfs: índice vacío, skip");
                return Ok(0);
            }
        };

        let header_size = std::mem::size_of::<DiskNodeHeader>() as u64;

        // ── BFS sobre la capa 0 del HNSW (contiene TODOS los nodos) ─────────
        // Recolectamos el orden BFS: entry_point primero (hub de mayor nivel),
        // luego sus vecinos en capa 0, y así sucesivamente.
        let total_nodes = hnsw.nodes.len();
        let mut bfs_order: Vec<u64> = Vec::with_capacity(total_nodes);
        let mut visited: HashSet<u64> = HashSet::with_capacity(total_nodes);
        let mut queue: VecDeque<u64> = VecDeque::with_capacity(total_nodes.min(1024));

        queue.push_back(entry_point_id);
        visited.insert(entry_point_id);

        while let Some(node_id) = queue.pop_front() {
            bfs_order.push(node_id);
            // Usar la capa 0 (contiene todos los vecinos del grafo base)
            if let Some(node_ref) = hnsw.nodes.get(&node_id) {
                if let Some(layer0_neighbors) = node_ref.neighbors.first() {
                    for &neighbor_id in layer0_neighbors {
                        if visited.insert(neighbor_id) {
                            queue.push_back(neighbor_id);
                        }
                    }
                }
            }
        }

        // Añadir nodos aislados (no alcanzados por BFS) al final
        for entry in hnsw.nodes.iter() {
            let node_id = *entry.key();
            if visited.insert(node_id) {
                bfs_order.push(node_id);
            }
        }

        // ── Calcular tamaño total del nuevo archivo ──────────────────────────
        let mut new_file_size: u64 = 64; // Primeros 64B reservados para el write_cursor
        for &node_id in &bfs_order {
            if let Some(node_ref) = hnsw.nodes.get(&node_id) {
                let old_offset = node_ref.storage_offset;
                if let Some(old_header) = vstore.read_header(old_offset) {
                    let vec_size = (old_header.vector_len as u64 * 4 + 63) & !63;
                    new_file_size += header_size + vec_size;
                }
                // Nodos sin header en vstore (sin vector) solo ocupan header_size
                // pero no los añadiremos al nuevo archivo si no tienen header válido.
            }
        }
        // Redondear a múltiplo de 4096 para alineación de página
        new_file_size = (new_file_size + 4095) & !4095;

        // ── Crear archivo temporal ───────────────────────────────────────────
        let vstore_path = vstore.path.clone();
        // Construir el path del tmp usando with_file_name para preservar la extensión .vanta
        // (with_extension remplazaría "vanta" → "vanta.tmp", perdiendo la extensión original)
        let tmp_filename = format!(
            "{}.tmp",
            vstore_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("vector_store.vanta")
        );
        let tmp_path = vstore_path.with_file_name(tmp_filename);

        let tmp_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&tmp_path)
            .map_err(VantaError::IoError)?;
        tmp_file
            .set_len(new_file_size)
            .map_err(VantaError::IoError)?;

        let mut tmp_mmap = unsafe {
            MmapOptions::new()
                .map_mut(&tmp_file)
                .map_err(VantaError::IoError)?
        };

        // ── Copiar nodos en orden BFS al archivo temporal ────────────────────
        // new_offset_map: node_id → nuevo storage_offset en el archivo compactado
        let mut new_offset_map: HashMap<u64, u64> = HashMap::with_capacity(total_nodes);
        let mut write_cursor: u64 = 64; // El primer u64 del archivo es el write_cursor

        for &node_id in &bfs_order {
            if let Some(node_ref) = hnsw.nodes.get(&node_id) {
                let old_offset = node_ref.storage_offset;
                // Intentar leer el header del VantaFile actual
                // SAFETY: leemos con borrow de `vstore` que ya tenemos en lock exclusivo.
                let old_header = match vstore.read_header(old_offset) {
                    Some(h) => *h,    // clone del header (64B, stack)
                    None => continue, // sin header válido: skip
                };

                // Saltar tombstones: no los copiamos al nuevo layout
                if (old_header.flags & 0x8) != 0 {
                    continue;
                }

                let vec_len = old_header.vector_len as u64;
                let vec_size_raw = vec_len * 4;
                let vec_size_aligned = (vec_size_raw + 63) & !63;

                let new_node_offset = write_cursor;
                let new_vec_offset = new_node_offset + header_size;

                // Verificar que el nuevo header + vector caben en el archivo tmp
                let end = new_vec_offset + vec_size_aligned;
                if end > new_file_size {
                    warn!(
                        node_id = node_id,
                        end = end,
                        file_size = new_file_size,
                        "compact_layout_bfs: offset fuera de rango, expandiendo archivo tmp"
                    );
                    drop(tmp_mmap);
                    tmp_file.set_len(end + 4096).map_err(VantaError::IoError)?;
                    tmp_mmap = unsafe {
                        MmapOptions::new()
                            .map_mut(&tmp_file)
                            .map_err(VantaError::IoError)?
                    };
                    new_file_size = end + 4096;
                }

                // Construir nuevo header con vector_offset actualizado
                let mut new_header = old_header;
                new_header.vector_offset = new_vec_offset;

                // Escribir header en tmp_mmap
                let header_bytes = new_header.as_bytes();
                tmp_mmap[new_node_offset as usize..(new_node_offset + header_size) as usize]
                    .copy_from_slice(header_bytes);

                // Copiar datos del vector desde el VantaFile original
                if vec_len > 0 {
                    let old_vec_start = old_header.vector_offset as usize;
                    let old_vec_end = old_vec_start + vec_size_raw as usize;
                    if old_vec_end <= vstore.size as usize {
                        let vec_src = &vstore.mmap_bytes()[old_vec_start..old_vec_end];
                        tmp_mmap[new_vec_offset as usize..(new_vec_offset + vec_size_raw) as usize]
                            .copy_from_slice(vec_src);
                    }
                }

                new_offset_map.insert(node_id, new_node_offset);
                write_cursor = new_vec_offset + vec_size_aligned;
            }
        }

        // Persistir el write_cursor al inicio del archivo tmp (primeros 8 bytes)
        let cursor_bytes = write_cursor.to_le_bytes();
        tmp_mmap[0..8].copy_from_slice(&cursor_bytes);

        // Flush del archivo temporal
        tmp_mmap.flush().map_err(VantaError::IoError)?;
        drop(tmp_mmap);
        drop(tmp_file);

        // ── Swap: tmp → vanta (portable Windows/Unix) ───────────────────
        // En Unix: rename es atómico incluso con el MMap del origen abierto.
        // En Windows: rename con un MMap abierto sobre el origen puede fallar.
        //   Usamos fs::copy (no requiere cerrar el Mmap del origen) seguido de
        //   remove_file para limpiar el tmp.
        //
        // Paso 1: Liberar el handle del vstore original (vstore_path).
        //   Reasignamos *vstore al tmp; el drop del VantaFile antiguo cierra
        //   su Mmap y File handle sobre vstore_path.
        vstore.flush().ok();
        *vstore = VantaFile::open(tmp_path.clone(), new_file_size)?;
        // vstore_path ahora sin handles. tmp_path tiene un Mmap (el nuevo *vstore).

        // Paso 2: Hacer el swap según el OS.
        #[cfg(windows)]
        {
            // En Windows usamos copia en lugar de rename para evitar el problema del Mmap.
            std::fs::copy(&tmp_path, &vstore_path).map_err(VantaError::IoError)?;
            let new_vstore = VantaFile::open(vstore_path, new_file_size)?;
            *vstore = new_vstore;
            // Limpiar el tmp después de que *vstore ya apunta al archivo definitivo.
            let _ = std::fs::remove_file(&tmp_path);
        }
        #[cfg(not(windows))]
        {
            // En Unix: rename atómico funciona incluso con Mmap abierto sobre el origen.
            std::fs::rename(&tmp_path, &vstore_path).map_err(VantaError::IoError)?;
            let new_vstore = VantaFile::open(vstore_path, new_file_size)?;
            *vstore = new_vstore;
        }

        let nodes_compacted = new_offset_map.len() as u64;

        // ── Actualizar storage_offset en el DashMap del HNSW ────────────────
        // Hacemos esto DESPUÉS del swap para que la actualización del índice
        // y el archivo físico sean coherentes.
        for (node_id, new_offset) in &new_offset_map {
            if let Some(mut node_ref) = hnsw.nodes.get_mut(node_id) {
                node_ref.storage_offset = *new_offset;
            }
        }

        let elapsed_ms = started.elapsed().as_millis() as u64;
        info!(
            nodes_compacted = nodes_compacted,
            new_file_size = new_file_size,
            elapsed_ms = elapsed_ms,
            "compact_layout_bfs: VantaFile compactado en orden BFS"
        );

        // Persistir el índice HNSW actualizado
        // Liberamos los locks antes de llamar a save_vector_index para evitar re-lock
        drop(hnsw);
        drop(vstore);

        self.save_vector_index();

        Ok(nodes_compacted)
    }

    pub fn insert(&self, node: &UnifiedNode) -> Result<()> {
        // Si el nodo ya existía, decrementamos sus estadísticas previas para mantener la consistencia
        if let Ok(Some(existing_node)) = self.get(node.id) {
            let mut stats = self.cardinality_stats.write();
            for (field, value) in existing_node.relational {
                let val_key = match value {
                    crate::node::FieldValue::String(s) => s,
                    crate::node::FieldValue::Int(i) => i.to_string(),
                    crate::node::FieldValue::Float(f) => f.to_string(),
                    crate::node::FieldValue::Bool(b) => b.to_string(),
                    crate::node::FieldValue::Null => "null".to_string(),
                };
                if let Some(val_map) = stats.get_mut(&field) {
                    if let Some(count) = val_map.get_mut(&val_key) {
                        if *count > 0 {
                            *count -= 1;
                        }
                    }
                    val_map.retain(|_, &mut v| v > 0);
                }
            }
        }

        {
            let mut stats = self.cardinality_stats.write();
            for (field, value) in &node.relational {
                let val_key = match value {
                    crate::node::FieldValue::String(s) => s.clone(),
                    crate::node::FieldValue::Int(i) => i.to_string(),
                    crate::node::FieldValue::Float(f) => f.to_string(),
                    crate::node::FieldValue::Bool(b) => b.to_string(),
                    crate::node::FieldValue::Null => "null".to_string(),
                };
                let val_map = stats.entry(field.clone()).or_default();
                if val_map.len() < 100 || val_map.contains_key(&val_key) {
                    *val_map.entry(val_key).or_default() += 1;
                }
            }
        }

        self.ensure_writable()?;
        #[cfg(feature = "failpoints")]
        fail::fail_point!("storage_insert_fail", |_| {
            Err(VantaError::IoError(std::io::Error::other(
                "Simulated Storage insert catastrophic I/O failure",
            )))
        });
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
            let _guard = self.insert_lock.lock();
            let hnsw = self.hnsw.read();
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
                let _guard = self.insert_lock.lock();
                let index = self.hnsw.read();
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
        if let Ok(Some(node)) = self.get(id) {
            let mut stats = self.cardinality_stats.write();
            for (field, value) in node.relational {
                let val_key = match value {
                    crate::node::FieldValue::String(s) => s,
                    crate::node::FieldValue::Int(i) => i.to_string(),
                    crate::node::FieldValue::Float(f) => f.to_string(),
                    crate::node::FieldValue::Bool(b) => b.to_string(),
                    crate::node::FieldValue::Null => "null".to_string(),
                };
                if let Some(val_map) = stats.get_mut(&field) {
                    if let Some(count) = val_map.get_mut(&val_key) {
                        if *count > 0 {
                            *count -= 1;
                        }
                    }
                    val_map.retain(|_, &mut v| v > 0);
                }
            }
        }

        self.ensure_writable()?;
        if let Some(ref mut wal_writer) = *self.wal.lock() {
            wal_writer.append(&crate::wal::WalRecord::Delete { id })?;
        }

        let hnsw = self.hnsw.read();
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
            .iter()
            .filter(|r| {
                let n = r.value();
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
        self.vector_store.read().flush()?;

        let current_wal_seq = {
            let wal_guard = self.wal.lock();
            if let Some(ref wal_writer) = *wal_guard {
                wal_writer.record_count()
            } else {
                0
            }
        };

        if current_wal_seq > 0 {
            let seq_bytes = bincode::serialize(&current_wal_seq)
                .map_err(|e| VantaError::SerializationError(e.to_string()))?;
            self.backend.put(
                BackendPartition::InternalMetadata,
                b"checkpoint_seq",
                &seq_bytes,
            )?;
            self.backend.flush()?;
        }

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
    pub fn scan_nodes(&self) -> Result<Vec<UnifiedNode>> {
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
    pub fn put_to_partition(
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

    /// Returns detailed memory usage statistics for this engine instance.
    ///
    /// This is useful for host applications (e.g., AI agents) to decide when to
    /// trigger memory pressure handling, such as evicting cold nodes or flushing caches.
    ///
    /// # Example
    /// ```rust,ignore
    /// let stats = engine.get_memory_stats();
    /// if stats.effective_bytes() > MEMORY_BUDGET {
    ///     engine.evict_cold_nodes(0.2)?; // Evict 20% of cold nodes
    /// }
    /// ```
    pub fn get_memory_stats(&self) -> MemoryStats {
        let hnsw = self.hnsw.read();
        let vector_store = self.vector_store.read();
        let cache = self.volatile_cache.read();

        // Logical estimate: HNSW structures + vector store file size + cached nodes
        // Note: This is an upper bound; actual RAM usage may be lower due to OS paging.
        let logical =
            hnsw.estimate_memory_bytes() as u64 + vector_store.size + (cache.len() as u64 * 1536); // ~1.5KB per cached node (conservative estimate)

        let physical = engine_mmap_resident_bytes(&hnsw, &vector_store);

        MemoryStats {
            logical_bytes: logical,
            physical_rss: physical,
            node_count: hnsw.nodes.len() as u64,
            cache_entries: cache.len(),
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

    fn initialize_cardinality_stats(
        backend: &dyn StorageBackend,
    ) -> HashMap<String, HashMap<String, usize>> {
        let mut stats: HashMap<String, HashMap<String, usize>> = HashMap::new();
        if let Ok(records) = backend.scan(BackendPartition::Default) {
            for (_key, val) in records {
                if let Ok(metadata) = bincode::deserialize::<NodeMetadata>(&val) {
                    for (field, value) in metadata.relational {
                        let val_str = match value {
                            crate::node::FieldValue::String(s) => s,
                            crate::node::FieldValue::Int(i) => i.to_string(),
                            crate::node::FieldValue::Float(f) => f.to_string(),
                            crate::node::FieldValue::Bool(b) => b.to_string(),
                            crate::node::FieldValue::Null => "null".to_string(),
                        };
                        let val_map = stats.entry(field).or_default();
                        if val_map.len() < 100 || val_map.contains_key(&val_str) {
                            *val_map.entry(val_str).or_default() += 1;
                        }
                    }
                }
            }
        }
        stats
    }

    pub fn get_estimated_selectivity(
        &self,
        field: &str,
        op: &crate::query::RelOp,
        value: &crate::node::FieldValue,
    ) -> f32 {
        let stats = self.cardinality_stats.read();
        let total_nodes = self.hnsw.read().nodes.len();
        if total_nodes == 0 {
            return 1.0;
        }

        let val_key = match value {
            crate::node::FieldValue::String(s) => s.clone(),
            crate::node::FieldValue::Int(i) => i.to_string(),
            crate::node::FieldValue::Float(f) => f.to_string(),
            crate::node::FieldValue::Bool(b) => b.to_string(),
            crate::node::FieldValue::Null => "null".to_string(),
        };

        if let Some(val_map) = stats.get(field) {
            let freq = *val_map.get(&val_key).unwrap_or(&0) as f32;

            match op {
                crate::query::RelOp::Eq => {
                    if freq > 0.0 {
                        freq / total_nodes as f32
                    } else if val_map.len() >= 100 {
                        1.0 / total_nodes.max(1) as f32
                    } else {
                        0.0
                    }
                }
                crate::query::RelOp::Neq => {
                    let eq_sel = if freq > 0.0 {
                        freq / total_nodes as f32
                    } else if val_map.len() >= 100 {
                        1.0 / total_nodes.max(1) as f32
                    } else {
                        0.0
                    };
                    1.0 - eq_sel
                }
                crate::query::RelOp::Gt
                | crate::query::RelOp::Gte
                | crate::query::RelOp::Lt
                | crate::query::RelOp::Lte => 0.33,
            }
        } else {
            match op {
                crate::query::RelOp::Eq => 0.0,
                crate::query::RelOp::Neq => 1.0,
                _ => 0.5,
            }
        }
    }
}
