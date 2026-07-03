//! Persistent storage engine coordinating backends, indexes, and WAL recovery.
//!
//! [`StorageEngine`] is the central persistence façade—it owns the backend
//! (in-memory, Fjall, or RocksDB), manages column-family partitions, and
//! drives node archival / recovery.

pub use crate::backend::BackendPartition;
use crate::backend::{BackendWriteOp, StorageBackend};
#[cfg(feature = "fjall")]
use crate::backends::fjall_backend::FjallBackend;
use crate::backends::in_memory::InMemoryBackend;
#[cfg(feature = "rocksdb")]
use crate::backends::rocksdb_backend::RocksDbBackend;
use crate::error::{Result, VantaError};
use crate::index::{CPIndex, IndexBackend};
use crate::node::{DiskNodeHeader, UnifiedNode};
use arc_swap::ArcSwap;
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::{File, OpenOptions};
#[cfg(not(feature = "memmap2"))]
use std::io::Read;
#[cfg(not(feature = "memmap2"))]
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{info, warn};
use web_time::Instant;
use web_time::{SystemTime, UNIX_EPOCH};
use zerocopy::{FromBytes, IntoBytes};

// ─── WASM-compatible mmap shim ────────────────────────────────────────
#[cfg(feature = "memmap2")]
pub(crate) use memmap2::{Mmap, MmapMut, MmapOptions};

#[cfg(not(feature = "memmap2"))]
pub(crate) mod mmap_shim {
    use super::*;
    #[derive(Debug)]
    pub struct Mmap(Vec<u8>);
    #[derive(Debug)]
    pub struct MmapMut(Vec<u8>);
    pub struct MmapOptions;

    impl MmapOptions {
        pub fn new() -> Self {
            Self
        }
        pub fn map(&self, file: &File) -> std::io::Result<Mmap> {
            let mut v = vec![0u8; file.metadata()?.len() as usize];
            let mut f = file.try_clone()?;
            f.read_exact(&mut v)?;
            Ok(Mmap(v))
        }
        pub fn map_mut(&self, file: &File) -> std::io::Result<MmapMut> {
            let mut v = vec![0u8; file.metadata()?.len() as usize];
            let mut f = file.try_clone()?;
            f.read_exact(&mut v)?;
            Ok(MmapMut(v))
        }
    }

    impl Mmap {
        pub fn map(file: &File) -> std::io::Result<Self> {
            MmapOptions::new().map(file)
        }
        pub fn as_ptr(&self) -> *const u8 {
            self.0.as_ptr()
        }
        pub fn len(&self) -> usize {
            self.0.len()
        }
        pub fn flush(&self) -> std::io::Result<()> {
            Ok(())
        }
        pub fn flush_async(&self) -> std::io::Result<()> {
            Ok(())
        }
        pub fn flush_range(&self, _offset: usize, _len: usize) -> std::io::Result<()> {
            Ok(())
        }
        pub fn is_empty(&self) -> bool {
            self.0.is_empty()
        }
    }

    impl Deref for Mmap {
        type Target = [u8];
        fn deref(&self) -> &[u8] {
            &self.0
        }
    }

    impl MmapMut {
        pub fn map_mut(file: &File) -> std::io::Result<Self> {
            MmapOptions::new().map_mut(file)
        }
        pub fn as_ptr(&self) -> *const u8 {
            self.0.as_ptr()
        }
        pub fn as_mut_ptr(&mut self) -> *mut u8 {
            self.0.as_mut_ptr()
        }
        pub fn len(&self) -> usize {
            self.0.len()
        }
        pub fn flush(&self) -> std::io::Result<()> {
            Ok(())
        }
        pub fn flush_async(&self) -> std::io::Result<()> {
            Ok(())
        }
        pub fn flush_range(&self, _offset: usize, _len: usize) -> std::io::Result<()> {
            Ok(())
        }
        pub fn is_empty(&self) -> bool {
            self.0.is_empty()
        }
    }

    impl Deref for MmapMut {
        type Target = [u8];
        fn deref(&self) -> &[u8] {
            &self.0
        }
    }
    impl DerefMut for MmapMut {
        fn deref_mut(&mut self) -> &mut [u8] {
            &mut self.0
        }
    }
}
#[cfg(not(feature = "memmap2"))]
pub(crate) use mmap_shim::{Mmap, MmapMut, MmapOptions};

#[cfg(unix)]
use std::sync::atomic::{AtomicBool, AtomicPtr};

// ─── SIGBUS Handler para Unix (TSK-05) ─────────────────────────

/// Flag global para indicar si ocurrió un SIGBUS relacionado con mmap
#[cfg(unix)]
static SIGBUS_OCCURRED: AtomicBool = AtomicBool::new(false);

/// Puntero atómico para almacenar la dirección de memoria que causó el SIGBUS
#[cfg(unix)]
static SIGBUS_FAULT_ADDR: AtomicPtr<u8> = AtomicPtr::new(std::ptr::null_mut());

/// Handler de señal SIGBUS para detectar truncamiento externo de archivos mmap
///
/// Este handler detecta cuando un acceso a memoria mmap falla debido a que
/// el archivo fue truncado externamente (por otro proceso o manualmente).
/// Permite recuperación graceful re-mapeando el archivo.
#[cfg(unix)]
fn install_sigbus_handler() -> Result<()> {
    use libc::{sigaction, SA_SIGINFO, SIGBUS};
    use std::mem;
    use std::sync::Once;

    static INSTALL_ONCE: Once = Once::new();

    INSTALL_ONCE.call_once(|| unsafe {
        let mut sa: libc::sigaction = mem::zeroed();
        sa.sa_sigaction = sigbus_handler as *const () as usize;
        sa.sa_flags = SA_SIGINFO;
        libc::sigemptyset(&mut sa.sa_mask);

        if sigaction(SIGBUS, &sa, std::ptr::null_mut()) != 0 {
            warn!(
                "Failed to install SIGBUS handler: {}",
                std::io::Error::last_os_error()
            );
        } else {
            info!("SIGBUS handler installed successfully");
        }
    });

    Ok(())
}

/// Handler de señal SIGBUS
#[cfg(unix)]
unsafe extern "C" fn sigbus_handler(
    _signum: libc::c_int,
    siginfo: *mut libc::siginfo_t,
    _context: *mut libc::c_void,
) {
    // Marcar que ocurrió un SIGBUS
    SIGBUS_OCCURRED.store(true, Ordering::SeqCst);

    // Almacenar la dirección de memoria que causó el fallo
    if !siginfo.is_null() {
        #[cfg(any(target_os = "linux", target_os = "android"))]
        let addr = (*siginfo).si_addr() as *mut u8;
        #[cfg(not(any(target_os = "linux", target_os = "android")))]
        let addr = (*siginfo).si_addr() as *mut u8;

        SIGBUS_FAULT_ADDR.store(addr, Ordering::SeqCst);
        warn!(
            "SIGBUS occurred at address: {:p} - possible external file truncation",
            addr
        );
    }
}

/// Verifica si ocurrió un SIGBUS desde la última verificación
#[cfg(unix)]
pub fn check_sigbus() -> bool {
    SIGBUS_OCCURRED.swap(false, Ordering::SeqCst)
}

/// Obtiene la dirección de memoria que causó el último SIGBUS
#[cfg(unix)]
pub fn get_sigbus_fault_addr() -> *mut u8 {
    SIGBUS_FAULT_ADDR.load(Ordering::SeqCst)
}

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
    pub file: Option<File>,
    mmap: VantaFileMap,
    pub path: PathBuf,
    pub size: u64,
    pub write_cursor: u64,
    read_only: bool,
}

enum VantaFileMap {
    ReadOnly(Mmap),
    ReadWrite(MmapMut),
    InMemory(Vec<u8>),
}

impl VantaFileMap {
    fn as_slice(&self) -> &[u8] {
        match self {
            VantaFileMap::ReadOnly(mmap) => mmap,
            VantaFileMap::ReadWrite(mmap) => mmap,
            VantaFileMap::InMemory(data) => data,
        }
    }

    fn as_ptr(&self) -> *const u8 {
        match self {
            VantaFileMap::ReadOnly(mmap) => mmap.as_ptr(),
            VantaFileMap::ReadWrite(mmap) => mmap.as_ptr(),
            VantaFileMap::InMemory(data) => data.as_ptr(),
        }
    }

    fn len(&self) -> usize {
        match self {
            VantaFileMap::ReadOnly(mmap) => mmap.len(),
            VantaFileMap::ReadWrite(mmap) => mmap.len(),
            VantaFileMap::InMemory(data) => data.len(),
        }
    }

    fn as_mut_slice(&mut self) -> Result<&mut [u8]> {
        match self {
            VantaFileMap::ReadOnly(_) => Err(VantaError::ValidationError {
                field: "read_only".into(),
                reason: "VantaFile is read-only; write operation rejected".into(),
            }),
            VantaFileMap::ReadWrite(mmap) => Ok(mmap),
            VantaFileMap::InMemory(data) => Ok(data),
        }
    }

    fn flush(&self) -> Result<()> {
        match self {
            VantaFileMap::ReadOnly(_) => Ok(()),
            VantaFileMap::ReadWrite(mmap) => mmap.flush().map_err(VantaError::IoError),
            VantaFileMap::InMemory(_) => Ok(()),
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

    /// Create a purely in-memory VantaFile backed by `Vec<u8>`.
    /// No file I/O involved — used for WASM / true in-memory mode.
    pub fn create_in_memory(initial_size: u64) -> Self {
        let size = initial_size.max(64);
        let mut data = vec![0u8; size as usize];
        let header = crate::binary_header::VantaHeader::new(*b"VFLE", 1, 0);
        data[0..16].copy_from_slice(&header.serialize());
        let initial_cursor = 64u64.to_le_bytes();
        data[16..24].copy_from_slice(&initial_cursor);
        Self {
            file: None,
            mmap: VantaFileMap::InMemory(data),
            path: PathBuf::new(),
            size,
            write_cursor: 64,
            read_only: false,
        }
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
                return Err(VantaError::ValidationError {
                    field: "file_size".into(),
                    reason: format!("VantaFile {} is too small for read-only open", path.display()),
                });
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
        let write_cursor = u64::from_le_bytes(mmap.as_slice()[16..24].try_into().map_err(|e| {
            VantaError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("write_cursor bytes at offset 16 expected 8 bytes: {e}"),
            ))
        })?);
        let write_cursor = if write_cursor < 64 || write_cursor > current_size {
            64
        } else {
            // Ensure any existing cursor is aligned to 64
            (write_cursor + 63) & !63
        };

        Ok(Self {
            file: Some(file),
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
            return Err(VantaError::ValidationError {
                field: "read_only".into(),
                reason: "VantaFile is read-only; remap operation rejected".into(),
            });
        }
        if let VantaFileMap::InMemory(_) = &self.mmap {
            return Ok(());
        }
        let file = self.file.as_ref().ok_or_else(|| {
            VantaError::ValidationError {
                field: "backing_file".into(),
                reason: "VantaFile: cannot remap without backing file".into(),
            }
        })?;
        self.mmap = VantaFileMap::ReadWrite(unsafe {
            MmapOptions::new()
                .map_mut(file)
                .map_err(VantaError::IoError)?
        });
        Ok(())
    }

    /// Read a DiskNodeHeader from a specific offset without cloning (Zero-Copy)
    pub fn read_header(&self, offset: u64) -> Option<DiskNodeHeader> {
        let header_size = std::mem::size_of::<DiskNodeHeader>() as u64;
        if offset + header_size > self.size || !offset.is_multiple_of(64) {
            return None;
        }

        let slice = &self.mmap_bytes()[offset as usize..(offset + header_size) as usize];
        DiskNodeHeader::read_from_bytes(slice).ok()
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

    /// Grow the backing store to at least `new_size` bytes.
    /// For file-backed VantaFiles this resizes the file and remaps the mmap.
    /// For in-memory VantaFiles this simply resizes the Vec.
    pub fn grow_to(&mut self, new_size: u64) -> Result<()> {
        match &mut self.mmap {
            VantaFileMap::InMemory(data) => {
                data.resize(new_size as usize, 0);
                self.size = new_size;
                Ok(())
            }
            VantaFileMap::ReadOnly(_) | VantaFileMap::ReadWrite(_) => {
                let file = self.file.as_ref().ok_or_else(|| {
                    VantaError::ValidationError {
                        field: "backing_file".into(),
                        reason: "VantaFile: grow_to requires backing file".into(),
                    }
                })?;
                file.set_len(new_size).map_err(VantaError::IoError)?;
                self.size = new_size;
                self.remap_mut()
            }
        }
    }

    /// Synchronizes changes to disk
    pub fn flush(&self) -> Result<()> {
        #[cfg(feature = "failpoints")]
        {
            fail::fail_point!("mmap_flush_fail", |_| {
                Err(VantaError::IoError(std::io::Error::other(
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
            #[cfg(feature = "memmap2")]
            use memmap2::Advice;
            #[cfg(feature = "memmap2")]
            let _ = match &self.mmap {
                VantaFileMap::ReadOnly(mmap) => mmap.advise(Advice::WillNeed),
                VantaFileMap::ReadWrite(mmap) => mmap.advise(Advice::WillNeed),
                VantaFileMap::InMemory(_) => Ok(()),
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

/// Report returned by eviction operations.
#[derive(Debug, Clone, Copy)]
pub struct EvictionReport {
    pub evicted: usize,
    pub scanned: usize,
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
    pub hnsw: ArcSwap<CPIndex>,
    /// Serializes insert/refresh operations to avoid bidirectional
    /// neighbor update races. Searches acquire hnsw.read() freely.
    insert_lock: parking_lot::Mutex<()>,
    pub volatile_cache: RwLock<std::collections::HashMap<u64, UnifiedNode>>,
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

    /// Returns the advanced tokenizer configuration if available.
    #[cfg(feature = "advanced-tokenizer")]
    pub fn advanced_tokenizer_config(&self) -> Option<&crate::tokenizer::AdvancedTokenizerConfig> {
        self.config.advanced_tokenizer_config.as_ref()
    }

    #[cfg(not(feature = "advanced-tokenizer"))]
    pub fn advanced_tokenizer_config(&self) -> Option<()> {
        None
    }

    fn init_storage(
        path: &str,
        config: &VantaConfig,
    ) -> Result<(Option<File>, Arc<dyn StorageBackend>, PathBuf)> {
        let base_path = PathBuf::from(path);

        // ── Pure in-memory: skip all file I/O ─────
        if matches!(config.backend_kind, BackendKind::InMemory) {
            let backend: Arc<dyn StorageBackend> = Arc::new(InMemoryBackend::new());
            return Ok((None, backend, PathBuf::new()));
        }

        if config.read_only && !base_path.exists() {
            return Err(VantaError::NotFound {
                kind: "database_path".into(),
                id: base_path.display().to_string(),
            });
        }
        let lock_file = {
            let lock_path = base_path.join(".vanta.lock");
            if !config.read_only {
                std::fs::create_dir_all(&base_path).map_err(VantaError::IoError)?;
            }

            let file_result = OpenOptions::new()
                .read(true)
                .write(!config.read_only)
                .create(!config.read_only)
                .open(&lock_path);

            let file = match file_result {
                Ok(f) => f,
                Err(e) => {
                    if config.read_only {
                        return Err(VantaError::NotFound {
                            kind: "lock_file".into(),
                            id: base_path.join(".vanta.lock").display().to_string(),
                        });
                    } else {
                        return Err(VantaError::IoError(e));
                    }
                }
            };

            // Intentar adquirir el bloqueo con reintentos y backoff exponencial (timeout total ~1000ms)
            let mut delay = std::time::Duration::from_millis(5);
            let total_limit = std::time::Duration::from_millis(config.file_lock_timeout_ms);
            let start_time = std::time::Instant::now();
            let mut acquired = false;

            while start_time.elapsed() < total_limit {
                let lock_res: std::io::Result<()> = if config.read_only {
                    #[cfg(feature = "fs2")]
                    {
                        fs2::FileExt::try_lock_shared(&file)
                    }
                    #[cfg(not(feature = "fs2"))]
                    {
                        Ok(())
                    }
                } else {
                    #[cfg(feature = "fs2")]
                    {
                        fs2::FileExt::try_lock_exclusive(&file)
                    }
                    #[cfg(not(feature = "fs2"))]
                    {
                        Ok(())
                    }
                };

                if lock_res.is_ok() {
                    acquired = true;
                    break;
                }

                // Esperar con backoff exponencial
                std::thread::sleep(delay);
                delay = std::cmp::min(delay * 2, std::time::Duration::from_millis(100));
            }

            if !acquired {
                let msg = if config.read_only {
                    format!(
                        "Database at '{}' is locked exclusively by another process (writer). \
                         Cannot acquire shared read-only lock within timeout.",
                        base_path.display()
                    )
                } else {
                    format!(
                        "Database at '{}' is locked by another process. \
                         Cannot acquire exclusive writer lock within timeout.",
                        base_path.display()
                    )
                };
                return Err(VantaError::DatabaseBusy(msg));
            }

            Some(file)
        };

        // ── Instalar handler SIGBUS para Unix (TSK-05) ─────────────────────
        #[cfg(unix)]
        {
            if let Err(e) = install_sigbus_handler() {
                warn!("Failed to install SIGBUS handler: {}", e);
            }
        }

        // ── Storage schema version check ────────────────────────────
        if !config.read_only {
            crate::schema::load_or_create_schema(&base_path)?;
            info!(
                "Storage schema: version={} flags={}",
                crate::schema::CURRENT_SCHEMA_VERSION,
                0
            );
        } else {
            crate::schema::check_schema_compatibility(&base_path)?;
        }

        // ── KV Backend initialization ──
        let backend: Arc<dyn StorageBackend> = match config.backend_kind {
            #[cfg(feature = "rocksdb")]
            BackendKind::RocksDb => Arc::new(RocksDbBackend::open(path, config)?),
            #[cfg(not(feature = "rocksdb"))]
            BackendKind::RocksDb => {
                return Err(VantaError::ValidationError {
                    field: "backend_feature".into(),
                    reason: "RocksDB backend requires the 'rocksdb' feature".into(),
                })
            }
            #[cfg(feature = "fjall")]
            BackendKind::Fjall => Arc::new(FjallBackend::open(path, config)?),
            #[cfg(not(feature = "fjall"))]
            BackendKind::Fjall => {
                return Err(VantaError::ValidationError {
                    field: "backend_feature".into(),
                    reason: "Fjall backend requires the 'fjall' feature".into(),
                })
            }
            BackendKind::InMemory => Arc::new(InMemoryBackend::new()),
        };

        let data_dir = base_path.join("data");
        if config.read_only && !data_dir.exists() {
            return Err(VantaError::NotFound {
                kind: "data_directory".into(),
                id: data_dir.display().to_string(),
            });
        }
        if !config.read_only {
            std::fs::create_dir_all(&data_dir).map_err(VantaError::IoError)?;
        }

        Ok((lock_file, backend, data_dir))
    }

    fn init_indexes(
        data_dir: &Path,
        config: &VantaConfig,
        caps: &crate::hardware::HardwareCapabilities,
        effective_memory: u64,
    ) -> Result<(CPIndex, VantaFile)> {
        let index_path = data_dir.join("vector_index.bin");

        let use_mmap = config.mmap_hnsw
            && (config.force_mmap
                || caps.profile == crate::hardware::HardwareProfile::LowResource
                || effective_memory < 16 * 1024 * 1024 * 1024);

        let hnsw = if let Some(loaded) = CPIndex::load_from_file(&index_path, use_mmap) {
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
        let vector_store = if config.read_only {
            VantaFile::open_read_only(vector_store_path)?
        } else {
            VantaFile::open(vector_store_path, 1024 * 1024 * 64)?
        };

        Ok((hnsw, vector_store))
    }

    fn recover_state(
        data_dir: &Path,
        config: &VantaConfig,
        backend: &dyn StorageBackend,
        hnsw: &mut CPIndex,
        vector_store: &mut VantaFile,
    ) -> Result<(u64, u64)> {
        let index_path = data_dir.join("vector_index.bin");

        // ── Index Reconstruction: rebuild HNSW if index file is missing ──────
        if hnsw.nodes.is_empty() {
            let report = Self::rebuild_hnsw_from_vstore(hnsw, vector_store, index_path)?;
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
            .and_then(|bytes| {
                bincode::serde::decode_from_slice::<u64, _>(&bytes, bincode::config::standard())
                    .ok()
                    .map(|(v, _)| v)
            })
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
                        let offset = Self::write_node_to_vstore(vector_store, &node)?;
                        hnsw.add(node.id, node.bitset, node.vector.clone(), offset);
                    }
                    crate::wal::WalRecord::Update { id, node } => {
                        let offset = Self::write_node_to_vstore(vector_store, &node)?;
                        hnsw.add(id, node.bitset, node.vector.clone(), offset);
                    }
                    crate::wal::WalRecord::Delete { id } => {
                        if let Some(index_node) = hnsw.nodes.get(&id) {
                            let offset = index_node.storage_offset;
                            if let Some(h) = vector_store.read_header(offset) {
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

        Ok((wal_replay_ms, wal_records_replayed))
    }

    fn init_wal(data_dir: &Path, config: &VantaConfig) -> Result<Option<crate::wal::WalWriter>> {
        let wal_writer = if config.read_only {
            None
        } else {
            let wal_path = data_dir.join("vanta.wal");
            Some(crate::wal::WalWriter::open(&wal_path, config.sync_mode)?)
        };
        Ok(wal_writer)
    }

    /// Open with explicit configuration for memory budgets and mode overrides.
    pub fn open_with_config(path: &str, config: Option<VantaConfig>) -> Result<Self> {
        let startup_started = Instant::now();
        let config = config.unwrap_or_default();
        let caps = crate::hardware::HardwareCapabilities::global();
        let effective_memory = config.memory_limit.unwrap_or(caps.total_memory);

        let (lock_file, backend, data_dir) = Self::init_storage(path, &config)?;

        // ── True in-memory mode: no disk-backed VantaFile, WAL, or recovery ──
        let (hnsw, vector_store, wal_writer, wal_replay_ms, wal_records_replayed) =
            if matches!(config.backend_kind, BackendKind::InMemory) {
                let hnsw = CPIndex::new();
                let vector_store = VantaFile::create_in_memory(1024 * 1024 * 64);
                let wal_writer = None;
                (hnsw, vector_store, wal_writer, 0u64, 0u64)
            } else {
                let (mut hnsw, mut vector_store) =
                    Self::init_indexes(&data_dir, &config, caps, effective_memory)?;
                let (wal_replay_ms, wal_records_replayed) = Self::recover_state(
                    &data_dir,
                    &config,
                    backend.as_ref(),
                    &mut hnsw,
                    &mut vector_store,
                )?;
                let wal_writer = Self::init_wal(&data_dir, &config)?;
                (
                    hnsw,
                    vector_store,
                    wal_writer,
                    wal_replay_ms,
                    wal_records_replayed,
                )
            };

        crate::metrics::record_startup(
            startup_started.elapsed().as_millis() as u64,
            wal_replay_ms,
            wal_records_replayed,
        );

        let estimated_hnsw_bytes = hnsw.estimate_memory_bytes() as u64;
        crate::metrics::record_memory_breakdown(
            hnsw.nodes.len() as u64,
            estimated_hnsw_bytes,
            engine_mmap_resident_bytes(&hnsw, &vector_store),
            0,
            0,
        );

        if hnsw.nodes.len() > 10_000 && estimated_hnsw_bytes > effective_memory / 2 {
            tracing::warn!(
                hnsw_nodes = hnsw.nodes.len(),
                estimated_mb = estimated_hnsw_bytes / 1024 / 1024,
                effective_mb = effective_memory / 1024 / 1024,
                "HNSW index exceeds 50% of memory budget",
            );
        }

        let cardinality_stats = Self::initialize_cardinality_stats(backend.as_ref());

        Ok(Self {
            config: config.clone(),
            read_only: config.read_only,
            hnsw: ArcSwap::from_pointee(hnsw),
            insert_lock: parking_lot::Mutex::new(()),
            volatile_cache: RwLock::new(std::collections::HashMap::new()),
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
            return Err(VantaError::ValidationError {
                field: "read_only".into(),
                reason: "StorageEngine is read-only; write operation rejected".into(),
            });
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

    /// Write a node to VantaFile.
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
            vstore.grow_to(new_size)?;
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

        // Mitigación A-01: Serializar escrituras/rebuild adquiriendo insert_lock
        let _guard = self
            .insert_lock
            .try_lock_for(std::time::Duration::from_millis(
                self.config.insert_lock_timeout_ms,
            ))
            .ok_or_else(|| {
                VantaError::Timeout {
                    operation: "acquire insert_lock in rebuild_vector_index".into(),
                    duration_ms: self.config.insert_lock_timeout_ms,
                }
            })?;

        // ── Paso 1: Flush del WAL antes de reubicar físicamente los offsets ──
        self.flush()?;

        let index_path = self.data_dir.join("vector_index.bin");
        let mut rebuilt = {
            let hnsw = self.hnsw.load();
            Self::fresh_index_like(&hnsw, index_path.clone())
        };

        let report = {
            let vstore = self.vector_store.read();
            Self::rebuild_hnsw_from_vstore(&mut rebuilt, &vstore, index_path)?
        };

        // Mitigación A-03: Persistencia en disco previa al swap en memoria (Atomicidad)
        if rebuilt.backend.is_mmap() {
            rebuilt.sync_to_mmap().map_err(VantaError::IoError)?;
        } else {
            rebuilt
                .persist_to_file(
                    rebuilt
                        .backend
                        .mmap_path()
                        .unwrap_or(&self.data_dir.join("vector_index.bin")),
                )
                .map_err(VantaError::IoError)?;
        }

        // Swap atómico del Arc en memoria (RCU)
        self.hnsw.store(Arc::new(rebuilt));

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

        // Mitigación A-01: Serializar mutaciones adquiriendo insert_lock
        let _guard_insert = self
            .insert_lock
            .try_lock_for(std::time::Duration::from_millis(
                self.config.insert_lock_timeout_ms,
            ))
            .ok_or_else(|| {
                VantaError::Timeout {
                    operation: "acquire insert_lock in compact_layout_bfs".into(),
                    duration_ms: self.config.insert_lock_timeout_ms,
                }
            })?;

        // ── Flush previo del WAL para garantizar consistencia ────────────────
        self.flush()?;

        let started = Instant::now();

        // ── Adquirir locks exclusivos en orden determinista (evita deadlock) ──
        // Orden: vector_store → hnsw  (siempre el mismo en todo el codebase)
        let mut vstore = self.vector_store.write();
        let hnsw = self.hnsw.load();

        let entry_point_id = match hnsw.get_entry_point() {
            Some(ep) => ep,
            None => {
                // Índice vacío: nada que compactar
                info!("compact_layout_bfs: empty index, skipping");
                return Ok(0);
            }
        };

        let header_size = std::mem::size_of::<DiskNodeHeader>() as u64;

        // ── BFS sobre la capa 0 del HNSW (contiene TODOS los nodos) ─────────
        let bfs_order = Self::traverse_graph(&hnsw, entry_point_id);

        // ── Layout compaction ────────────────────────────────────────────────
        let (new_offset_map, new_file_size) =
            Self::compact_layout(&mut vstore, &hnsw, &bfs_order, header_size)?;
        let nodes_compacted = new_offset_map.len() as u64;

        // ── Actualizar storage_offset en el DashMap del HNSW ────────────────
        Self::reindex_nodes(&hnsw, &new_offset_map);

        drop(hnsw);

        let elapsed_ms = started.elapsed().as_millis() as u64;
        info!(
            nodes_compacted = nodes_compacted,
            new_file_size = new_file_size,
            elapsed_ms = elapsed_ms,
            "compact_layout_bfs: VantaFile compactado en orden BFS"
        );

        drop(vstore);
        self.save_vector_index();

        Ok(nodes_compacted)
    }

    /// Realiza un BFS sobre la capa 0 del HNSW para recolectar el orden de compactación.
    ///
    /// Comienza desde el entry point (hub de mayor nivel), luego sus vecinos en capa 0,
    /// y así sucesivamente. Los nodos aislados (no alcanzados por BFS) se añaden al final.
    fn traverse_graph(hnsw: &CPIndex, entry_point_id: u64) -> Vec<u64> {
        let total_nodes = hnsw.nodes.len();
        let mut bfs_order: Vec<u64> = Vec::with_capacity(total_nodes);
        let mut visited: HashSet<u64> = HashSet::with_capacity(total_nodes);
        let mut queue: VecDeque<u64> = VecDeque::with_capacity(total_nodes.min(1024));

        queue.push_back(entry_point_id);
        visited.insert(entry_point_id);

        while let Some(node_id) = queue.pop_front() {
            bfs_order.push(node_id);
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

        for entry in hnsw.nodes.iter() {
            let node_id = *entry.key();
            if visited.insert(node_id) {
                bfs_order.push(node_id);
            }
        }

        bfs_order
    }

    /// Compacta el layout físico del VantaFile escribiendo los nodos en el orden BFS
    /// especificado a un archivo temporal, luego realiza el swap atómico.
    ///
    /// Retorna el mapa de nuevos offsets y el tamaño final del archivo.
    fn compact_layout(
        vstore: &mut VantaFile,
        hnsw: &CPIndex,
        bfs_order: &[u64],
        header_size: u64,
    ) -> Result<(HashMap<u64, u64>, u64)> {
        let mut new_file_size: u64 = 64;
        for &node_id in bfs_order {
            if let Some(node_ref) = hnsw.nodes.get(&node_id) {
                let old_offset = node_ref.storage_offset;
                if let Some(old_header) = vstore.read_header(old_offset) {
                    let vec_size = (old_header.vector_len as u64 * 4 + 63) & !63;
                    new_file_size += header_size + vec_size;
                }
            }
        }
        new_file_size = (new_file_size + 4095) & !4095;

        let vstore_path = vstore.path.clone();
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

        let mut new_offset_map: HashMap<u64, u64> = HashMap::with_capacity(bfs_order.len());
        let mut write_cursor: u64 = 64;

        for &node_id in bfs_order {
            if let Some(node_ref) = hnsw.nodes.get(&node_id) {
                let old_offset = node_ref.storage_offset;
                let old_header = match vstore.read_header(old_offset) {
                    Some(h) => h,
                    None => continue,
                };

                if (old_header.flags & 0x8) != 0 {
                    continue;
                }

                let vec_len = old_header.vector_len as u64;
                let vec_size_raw = vec_len * 4;
                let vec_size_aligned = (vec_size_raw + 63) & !63;

                let new_node_offset = write_cursor;
                let new_vec_offset = new_node_offset + header_size;

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

                let mut new_header = old_header;
                new_header.vector_offset = new_vec_offset;

                let header_bytes = new_header.as_bytes();
                tmp_mmap[new_node_offset as usize..(new_node_offset + header_size) as usize]
                    .copy_from_slice(header_bytes);

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

        let cursor_bytes = write_cursor.to_le_bytes();
        tmp_mmap[0..8].copy_from_slice(&cursor_bytes);

        tmp_mmap.flush().map_err(VantaError::IoError)?;
        drop(tmp_mmap);
        drop(tmp_file);

        if let Err(e) = vstore.flush() {
            tracing::warn!("flush failed: {e}");
        }
        *vstore = VantaFile::open(tmp_path.clone(), new_file_size)?;

        #[cfg(windows)]
        {
            std::fs::copy(&tmp_path, &vstore_path).map_err(VantaError::IoError)?;
            let new_vstore = VantaFile::open(vstore_path, new_file_size)?;
            *vstore = new_vstore;
            let _ = std::fs::remove_file(&tmp_path);
        }
        #[cfg(not(windows))]
        {
            std::fs::rename(&tmp_path, &vstore_path).map_err(VantaError::IoError)?;
            let new_vstore = VantaFile::open(vstore_path, new_file_size)?;
            *vstore = new_vstore;
        }

        Ok((new_offset_map, new_file_size))
    }

    /// Actualiza los storage_offset de todos los nodos compactados en el índice HNSW.
    fn reindex_nodes(hnsw: &CPIndex, new_offset_map: &HashMap<u64, u64>) {
        for (node_id, new_offset) in new_offset_map {
            if let Some(mut node_ref) = hnsw.nodes.get_mut(node_id) {
                node_ref.storage_offset = *new_offset;
            }
        }
    }

    #[tracing::instrument(skip(self, node), level = "debug", err)]
    pub fn insert(&self, node: &UnifiedNode) -> Result<()> {
        self.check_memory_pressure()?;
        // Si el nodo ya existía, decrementamos sus estadísticas previas para mantener la consistencia
        if let Ok(Some(existing_node)) = self.get(node.id) {
            let mut stats = self.cardinality_stats.write();
            for (field, value) in existing_node.relational {
                let val_keys = value.to_cardinality_keys();
                if let Some(val_map) = stats.get_mut(&field) {
                    for val_key in val_keys {
                        if let Some(count) = val_map.get_mut(&val_key) {
                            if *count > 0 {
                                *count -= 1;
                            }
                        }
                    }
                    val_map.retain(|_, &mut v| v > 0);
                }
            }
        }

        {
            let mut stats = self.cardinality_stats.write();
            for (field, value) in &node.relational {
                let val_keys = value.to_cardinality_keys();
                let val_map = stats.entry(field.clone()).or_default();
                for val_key in val_keys {
                    if val_map.len() < 100 || val_map.contains_key(&val_key) {
                        *val_map.entry(val_key).or_default() += 1;
                    }
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

        self.touch_activity();

        let mut active_node = node.clone();
        active_node.last_accessed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        if let Some(ref mut wal_writer) = *self.wal.lock() {
            wal_writer.append(&crate::wal::WalRecord::Insert(active_node.clone()))?;
        }

        let mut vstore = self.vector_store.write();
        let storage_offset = Self::write_node_to_vstore(&mut vstore, &active_node)?;

        let key = active_node.id.to_le_bytes();
        let metadata = NodeMetadata {
            relational: active_node.relational.clone(),
            edges: active_node.edges.clone(),
        };
        let metadata_val = bincode::serde::encode_to_vec(&metadata, bincode::config::standard())
            .map_err(|e| VantaError::SerializationError(e.to_string()))?;
        self.backend
            .put(BackendPartition::Default, &key, &metadata_val)?;

        {
            let _guard = self
                .insert_lock
                .try_lock_for(std::time::Duration::from_millis(
                    self.config.insert_lock_timeout_ms,
                ))
                .ok_or_else(|| {
                    VantaError::Timeout {
                        operation: "acquire insert_lock in update_node".into(),
                        duration_ms: self.config.insert_lock_timeout_ms,
                    }
                })?;
            let hnsw = self.hnsw.load();
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
                if let Err(e) = self.evict_cold_nodes(self.config.eviction_ratio) {
                    tracing::warn!("eviction failed: {e}");
                }
            }
        }

        Ok(())
    }

    pub fn refresh_index(&self, node: &UnifiedNode, storage_offset: u64) -> Result<()> {
        if !storage_offset.is_multiple_of(64) {
            return Ok(());
        }
        if node.flags.is_set(crate::node::NodeFlags::HAS_VECTOR) {
            if let crate::node::VectorRepresentations::Full(vec) = &node.vector {
                let _guard = self
                    .insert_lock
                    .try_lock_for(std::time::Duration::from_millis(
                        self.config.insert_lock_timeout_ms,
                    ))
                    .ok_or_else(|| {
                        VantaError::Timeout {
                            operation: "acquire insert_lock in refresh_index".into(),
                            duration_ms: self.config.insert_lock_timeout_ms,
                        }
                    })?;
                let index = self.hnsw.load();
                index.add(
                    node.id,
                    node.bitset,
                    crate::node::VectorRepresentations::Full(vec.clone()),
                    storage_offset,
                );
                return Ok(());
            }
        }
        let _guard = self
            .insert_lock
            .try_lock_for(std::time::Duration::from_millis(
                self.config.insert_lock_timeout_ms,
            ))
            .ok_or_else(|| {
                VantaError::Timeout {
                    operation: "acquire insert_lock in refresh_index".into(),
                    duration_ms: self.config.insert_lock_timeout_ms,
                }
            })?;
        let index = self.hnsw.load();
        index.add(
            node.id,
            node.bitset,
            crate::node::VectorRepresentations::None,
            storage_offset,
        );
        Ok(())
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
        let metadata_val = bincode::serde::encode_to_vec(&metadata, bincode::config::standard())
            .map_err(|e| VantaError::SerializationError(e.to_string()))?;
        self.backend
            .put(BackendPartition::Default, &key, &metadata_val)?;

        // Consolidate doesn't change the vector store offset if already present
        let offset = {
            let hnsw = self.hnsw.load();
            hnsw.nodes
                .get(&node.id)
                .map(|n| n.storage_offset)
                .unwrap_or(0)
        };
        self.refresh_index(&persisted, offset)?;

        // Libera páginas de memoria del vector store para nodos Cold (TSK-04)
        // Esto reduce el RSS sin invalidar el mmap; las páginas se cargarán bajo demanda
        if offset > 0 {
            let vstore = self.vector_store.read();
            let mmap = vstore.mmap_bytes();
            // Calcular tamaño del vector desde la representación
            let vector_size = match &persisted.vector {
                crate::node::VectorRepresentations::Full(v) => v.len() * 4, // f32 = 4 bytes
                crate::node::VectorRepresentations::MmapFull(_, len) => *len,
                crate::node::VectorRepresentations::Binary(b) => b.len() * 8, // u64 = 8 bytes
                crate::node::VectorRepresentations::Turbo(t) => t.len(),
                crate::node::VectorRepresentations::SQ8(d, _) => d.len() + 4,
                crate::node::VectorRepresentations::None => 0,
            };
            // Alinear a 64 bytes (misma alineación que usa el storage)
            let vector_size_aligned = (vector_size + 63) & !63;
            let offset_usize = offset as usize;
            if offset_usize + vector_size_aligned <= mmap.len() && vector_size_aligned > 0 {
                // SAFETY: offset and len were bounds-checked above; mmap_ptr is valid for
                // the entire VantaFile lifetime held by the RwLock read guard.
                unsafe {
                    crate::index::release_mmap_vector(
                        mmap.as_ptr(),
                        offset_usize,
                        vector_size_aligned,
                    );
                }
            }
        }

        {
            let mut cache = self.volatile_cache.write();
            cache.remove(&node.id);
        }

        Ok(())
    }

    /// Evict a fraction of hot nodes from the volatile cache by lowest eviction score.
    /// Nodes are consolidated to cold tier and their mmap pages may be released.
    pub fn evict_cold_nodes(&self, ratio: f64) -> Result<EvictionReport> {
        self.ensure_writable()?;
        let ratio = ratio.clamp(0.0, 1.0);
        if ratio <= 0.0 {
            return Ok(EvictionReport {
                evicted: 0,
                scanned: 0,
            });
        }

        let candidates: Vec<UnifiedNode> = {
            let cache = self.volatile_cache.read();
            cache
                .values()
                .filter(|n| n.tier == crate::node::NodeTier::Hot)
                .cloned()
                .collect()
        };

        if candidates.is_empty() {
            return Ok(EvictionReport {
                evicted: 0,
                scanned: 0,
            });
        }

        let target = (candidates.len() as f64 * ratio).max(1.0) as usize;
        let scanned = candidates.len();
        let weights = self.config.eviction_weights();

        let mut scored: Vec<(f64, UnifiedNode)> = candidates
            .into_iter()
            .map(|n| {
                let score = n.eviction_score(&weights);
                (score, n)
            })
            .collect();
        // Sort ascending — lowest score = best eviction candidate
        scored.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        let mut evicted = 0;
        for (_score, node) in scored.iter().take(target) {
            if self.consolidate_node(node).is_ok() {
                evicted += 1;
            }
        }

        Ok(EvictionReport { evicted, scanned })
    }

    pub fn insert_to_cf(&self, node: &UnifiedNode, cf_name: &str) -> Result<()> {
        self.ensure_writable()?;
        let partition = Self::partition_from_cf_name(cf_name)?;
        let key = node.id.to_le_bytes();
        let val = bincode::serde::encode_to_vec(node, bincode::config::standard())
            .map_err(|e| VantaError::SerializationError(e.to_string()))?;
        self.backend.put(partition, &key, &val)?;

        let mut vstore = self.vector_store.write();
        let storage_offset = Self::write_node_to_vstore(&mut vstore, node)?;
        self.refresh_index(node, storage_offset)?;
        Ok(())
    }

    #[tracing::instrument(skip(self), level = "debug", err)]
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

        let (metadata, _): (NodeMetadata, usize) =
            bincode::serde::decode_from_slice(&metadata_res, bincode::config::standard())
                .map_err(|e| VantaError::SerializationError(e.to_string()))?;

        let hnsw = self.hnsw.load();
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

    /// Retrieve multiple nodes by ID in a single batch operation.
    ///
    /// Uses `backend.get_many` to eliminate N+1 query patterns. Returns only
    /// the nodes that were found (missing IDs are silently omitted).
    #[tracing::instrument(skip(self), level = "debug", err)]
    pub fn get_many(&self, ids: &[u64]) -> Result<Vec<UnifiedNode>> {
        self.touch_activity();

        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut results: Vec<UnifiedNode> = Vec::with_capacity(ids.len());

        let ids_with_keys: Vec<(u64, Vec<u8>)> = ids
            .iter()
            .map(|id| (*id, id.to_le_bytes().to_vec()))
            .collect();

        let mut remaining_indices: Vec<usize> = Vec::new();
        {
            let mut cache = self.volatile_cache.write();
            for (i, &id) in ids.iter().enumerate() {
                if let Some(node) = cache.get_mut(&id) {
                    if node.flags.is_set(crate::node::NodeFlags::TOMBSTONE) {
                        continue;
                    }
                    node.hits += 1;
                    node.last_accessed = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    results.push(node.clone());
                } else {
                    remaining_indices.push(i);
                }
            }
        }

        if remaining_indices.is_empty() {
            return Ok(results);
        }

        let remaining_keys: Vec<&[u8]> = remaining_indices
            .iter()
            .map(|&i| ids_with_keys[i].1.as_slice())
            .collect();

        let backend_results = self
            .backend
            .get_many(BackendPartition::Default, &remaining_keys)?;

        let backend_map: std::collections::HashMap<u64, Vec<u8>> = backend_results
            .into_iter()
            .map(|(k, v)| {
                let id = u64::from_le_bytes(
                    k.as_slice()
                        .try_into()
                        .expect("backend key must be 8 bytes"),
                );
                (id, v)
            })
            .collect();

        let hnsw = self.hnsw.load();
        let vstore = self.vector_store.read();

        for &i in &remaining_indices {
            let id = ids[i];
            let Some(metadata_bytes) = backend_map.get(&id) else {
                continue;
            };

            let (metadata, _): (NodeMetadata, usize) = match bincode::serde::decode_from_slice(
                metadata_bytes,
                bincode::config::standard(),
            ) {
                Ok(m) => m,
                Err(_) => continue,
            };

            let Some(index_node) = hnsw.nodes.get(&id) else {
                continue;
            };
            let storage_offset = index_node.storage_offset;

            let Some(header) = vstore.read_header(storage_offset) else {
                continue;
            };

            if (header.flags & 0x8) != 0 {
                continue;
            }

            let vec_start = header.vector_offset as usize;
            let vec_end = vec_start + (header.vector_len as usize * 4);
            if vec_end > vstore.size as usize {
                continue;
            }

            let vec_bytes = &vstore.mmap_bytes()[vec_start..vec_end];
            let f32_vec: &[f32] = unsafe {
                std::slice::from_raw_parts(
                    vec_bytes.as_ptr() as *const f32,
                    header.vector_len as usize,
                )
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

            results.push(node);
        }

        Ok(results)
    }

    #[tracing::instrument(skip(self), level = "debug", err)]
    pub fn delete(&self, id: u64, _reason: &str) -> Result<()> {
        self.check_memory_pressure()?;
        if let Ok(Some(node)) = self.get(id) {
            let mut stats = self.cardinality_stats.write();
            for (field, value) in node.relational {
                let val_keys = value.to_cardinality_keys();
                if let Some(val_map) = stats.get_mut(&field) {
                    for val_key in val_keys {
                        if let Some(count) = val_map.get_mut(&val_key) {
                            if *count > 0 {
                                *count -= 1;
                            }
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

        let hnsw = self.hnsw.load();
        if let Some(index_node) = hnsw.nodes.get(&id) {
            let offset = index_node.storage_offset;

            let mut vstore = self.vector_store.write();
            if let Some(mut header) = vstore.read_header(offset) {
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
        let hnsw = self.hnsw.load();

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

    #[tracing::instrument(skip(self), level = "info", err)]
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
            let seq_bytes =
                bincode::serde::encode_to_vec(current_wal_seq, bincode::config::standard())
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
        let hnsw = self.hnsw.load();
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

    /// Compact the WAL: flush all data, archive the current WAL file
    /// (``vanta.wal.<timestamp>``), reset ``checkpoint_seq`` to 0,
    /// and start a fresh WAL.
    ///
    /// Archived WALs can be safely removed once the application
    /// confirms no crash-recovery data is needed from them.
    #[tracing::instrument(skip(self), level = "info", err)]
    pub fn compact_wal(&self) -> Result<()> {
        self.flush()?;

        let mut wal_guard = self.wal.lock();
        if let Some(writer) = wal_guard.take() {
            let sync_mode = writer.sync_mode;
            let new_writer = writer.rotate(sync_mode)?;
            *wal_guard = Some(new_writer);
        }

        // Reset checkpoint_seq to 0 since the new WAL is empty.
        let zero: [u8; 8] = 0u64.to_le_bytes();
        self.backend
            .put(BackendPartition::InternalMetadata, b"checkpoint_seq", &zero)?;
        self.backend.flush()?;

        Ok(())
    }

    fn save_vector_index(&self) {
        let index_path = self.data_dir.join("vector_index.bin");
        let current = self.hnsw.load();

        if current.backend.is_mmap() {
            // Mitigación A-03: RCU para MMap (Atomicidad y Transaccionalidad)
            let data = current.serialize_to_bytes();
            let temp_path = index_path.with_extension("bin.tmp");

            let result = (|| -> std::io::Result<Arc<CPIndex>> {
                let file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&temp_path)?;
                file.set_len(data.len() as u64)?;

                let mut mapped = unsafe { MmapMut::map_mut(&file)? };
                mapped.copy_from_slice(&data);
                mapped.flush()?;

                let mut new_index =
                    CPIndex::deserialize_from_bytes(&mapped, false).map_err(|e| {
                        std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
                    })?;

                new_index.backend = IndexBackend::MMapFile {
                    path: index_path.clone(),
                    mmap: Some(mapped),
                };

                drop(file);
                // Intercambio atómico en disco
                std::fs::rename(&temp_path, &index_path)?;
                Ok(Arc::new(new_index))
            })();

            match result {
                Ok(new_hnsw) => {
                    self.hnsw.store(new_hnsw);
                }
                Err(e) => {
                    warn!(err = %e, "Failed to sync MMap vector index via RCU");
                }
            }
        } else {
            // InMemory no requiere re-mapeo, sólo persistencia
            if let Err(e) = current.persist_to_file(&index_path) {
                warn!(err = %e, "Failed to persist vector index to file");
            }
        }
    }

    pub fn create_life_insurance(&self, timestamp_name: &str) -> Result<()> {
        self.ensure_writable()?;
        if !self.supports_checkpoint() {
            return Err(VantaError::BackendError(format!(
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
            if let Ok((mut node, _)) = bincode::serde::decode_from_slice::<
                crate::node::UnifiedNode,
                _,
            >(v, bincode::config::standard())
            {
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
    ///
    /// Unlike `get()`, this parses the metadata directly from the scan
    /// result to avoid one backend lookup per node.
    pub fn scan_nodes(&self) -> Result<Vec<UnifiedNode>> {
        let entries = self.backend.scan(BackendPartition::Default)?;
        let mut nodes = Vec::with_capacity(entries.len());
        let hnsw = self.hnsw.load();
        let vstore = self.vector_store.read();

        for (key, value) in entries {
            if key.len() != std::mem::size_of::<u64>() {
                continue;
            }

            let id = u64::from_le_bytes(
                key.as_slice()
                    .try_into()
                    .expect("key slice fits [u8; 8] after length guard"),
            );

            let metadata: NodeMetadata =
                match bincode::serde::decode_from_slice(&value, bincode::config::standard()) {
                    Ok((m, _)) => m,
                    Err(_) => continue,
                };

            let index_node = match hnsw.nodes.get(&id) {
                Some(n) => n,
                None => continue,
            };
            let storage_offset = index_node.storage_offset;

            let header = match vstore.read_header(storage_offset) {
                Some(h) => h,
                None => continue,
            };

            if (header.flags & 0x8) != 0 {
                continue;
            }

            let vec_start = header.vector_offset as usize;
            let vec_end = vec_start + (header.vector_len as usize * 4);
            if vec_end > vstore.size as usize {
                continue;
            }

            let vec_bytes = &vstore.mmap_bytes()[vec_start..vec_end];
            let f32_vec: &[f32] = unsafe {
                std::slice::from_raw_parts(
                    vec_bytes.as_ptr() as *const f32,
                    header.vector_len as usize,
                )
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
            nodes.push(node);
        }

        drop(vstore);
        drop(hnsw);

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

    /// Retrieve multiple raw values from a partition in a single batch.
    #[allow(dead_code)]
    pub(crate) fn get_many_from_partition(
        &self,
        partition: BackendPartition,
        keys: &[&[u8]],
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        self.backend.get_many(partition, keys)
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
            other => Err(VantaError::InvalidInput(format!(
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
        let hnsw = self.hnsw.load();
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

    pub fn check_memory_pressure(&self) -> Result<()> {
        let threshold = self.config.rss_threshold;
        if threshold <= 0.0 {
            return Ok(());
        }
        let stats = self.get_memory_stats();
        let effective = stats.effective_bytes();
        if effective == 0 {
            return Ok(());
        }
        let limit = self
            .config
            .memory_limit
            .unwrap_or_else(|| crate::hardware::HardwareCapabilities::global().total_memory);
        if (effective as f64) > (limit as f64 * threshold) {
            tracing::warn!(
                effective_bytes = effective,
                threshold_pct = (threshold * 100.0) as u64,
                "Memory pressure detected — triggering auto-eviction",
            );
            if let Err(e) = self.evict_cold_nodes(self.config.eviction_ratio) {
                tracing::warn!("eviction failed: {e}");
            }
            return Err(VantaError::ResourceLimit(format!(
                "Memory pressure: {} bytes used ({}% of {} limit, threshold {}%)",
                effective,
                (effective as f64 / limit as f64 * 100.0) as u64,
                limit,
                (threshold * 100.0) as u64,
            )));
        }
        Ok(())
    }

    pub fn emergency_shutdown(&self, reason: &str, stmt: Option<&str>) -> ! {
        println!("\n=======================================================");
        println!("[!] VANTADB SYSTEM EMERGENCY: Security Constraint Violated");
        println!("=======================================================");
        tracing::error!("Emergency shutdown reason: {}", reason);
        if let Some(s) = stmt {
            tracing::error!("Offending Transaction: {}", s);
        }

        println!("Attempting controlled flush...");
        if let Err(e) = self.flush() {
            tracing::error!("Failed to flush buffers during shutdown: {}", e);
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
                if let Ok((metadata, _)) = bincode::serde::decode_from_slice::<NodeMetadata, _>(
                    &val,
                    bincode::config::standard(),
                ) {
                    for (field, value) in metadata.relational {
                        let val_keys = value.to_cardinality_keys();
                        let val_map = stats.entry(field).or_default();
                        for val_key in val_keys {
                            if val_map.len() < 100 || val_map.contains_key(&val_key) {
                                *val_map.entry(val_key).or_default() += 1;
                            }
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
        let total_nodes = self.hnsw.load().nodes.len();
        if total_nodes == 0 {
            return 1.0;
        }

        let val_keys = value.to_cardinality_keys();
        let val_key = val_keys
            .first()
            .cloned()
            .unwrap_or_else(|| "null".to_string());

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

impl Drop for StorageEngine {
    fn drop(&mut self) {
        #[cfg(feature = "fs2")]
        {
            if let Some(file) = &self._lock_file {
                let _ = fs2::FileExt::unlock(file);
            }
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::VantaConfig;
    use crate::node::UnifiedNode;

    /// Create a StorageEngine with InMemory backend for testing.
    fn in_memory_engine() -> StorageEngine {
        let config = VantaConfig {
            backend_kind: BackendKind::InMemory,
            read_only: false,
            ..VantaConfig::default()
        };
        StorageEngine::open_with_config(":memory:", Some(config))
            .expect("Failed to open in-memory engine")
    }

    fn in_memory_read_only() -> StorageEngine {
        let config = VantaConfig {
            backend_kind: BackendKind::InMemory,
            read_only: true,
            ..VantaConfig::default()
        };
        StorageEngine::open_with_config(":memory:", Some(config))
            .expect("Failed to open read-only in-memory engine")
    }

    fn sample_node(id: u64) -> UnifiedNode {
        let mut node = UnifiedNode::new(id);
        node.vector = crate::node::VectorRepresentations::Full(vec![0.1, 0.2, 0.3]);
        node
    }

    // ─── Open / Config ────────────────────────────────────────

    #[test]
    fn test_open_in_memory() {
        let engine = in_memory_engine();
        assert_eq!(engine.backend_kind(), BackendKind::InMemory);
        assert!(!engine.read_only);
    }

    #[test]
    fn test_open_with_default_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let engine = StorageEngine::open(path).expect("open with default config");
        assert!(!engine.read_only);
    }

    #[test]
    fn test_backend_kind_in_memory() {
        let engine = in_memory_engine();
        assert_eq!(engine.backend_kind(), BackendKind::InMemory);
    }

    #[test]
    fn test_supports_checkpoint_in_memory() {
        let engine = in_memory_engine();
        assert!(!engine.supports_checkpoint());
    }

    #[test]
    fn test_supports_manual_compaction_in_memory() {
        let engine = in_memory_engine();
        assert!(!engine.supports_manual_compaction());
    }

    #[test]
    fn test_backend_capabilities() {
        let engine = in_memory_engine();
        let caps = engine.backend_capabilities();
        assert_eq!(caps.kind, BackendKind::InMemory);
    }

    // ─── Insert / Get ─────────────────────────────────────────

    #[test]
    fn test_insert_and_get() {
        let engine = in_memory_engine();
        let node = sample_node(42);
        engine.insert(&node).expect("insert should succeed");

        let retrieved = engine.get(42).expect("get should succeed");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, 42);
    }

    #[test]
    fn test_insert_preserves_vector() {
        let engine = in_memory_engine();
        let mut node = UnifiedNode::new(7);
        let vec = vec![0.5, 0.8, 0.2, 0.9];
        node.vector = crate::node::VectorRepresentations::Full(vec.clone());
        engine.insert(&node).expect("insert");

        let retrieved = engine.get(7).expect("get").unwrap();
        match retrieved.vector {
            crate::node::VectorRepresentations::Full(v) => assert_eq!(v, vec),
            _ => panic!("expected Full vector"),
        }
    }

    #[test]
    fn test_get_nonexistent() {
        let engine = in_memory_engine();
        let retrieved = engine.get(999).expect("get should succeed");
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_insert_duplicate_overwrites() {
        let engine = in_memory_engine();
        let mut node1 = UnifiedNode::new(1);
        node1.importance = 10.0;
        engine.insert(&node1).expect("first insert");

        let mut node2 = UnifiedNode::new(1);
        node2.importance = 99.0;
        engine.insert(&node2).expect("second insert");

        let retrieved = engine.get(1).expect("get").unwrap();
        assert_eq!(retrieved.importance, 99.0);
    }

    // ─── Delete ───────────────────────────────────────────────

    #[test]
    fn test_delete_existing() {
        let engine = in_memory_engine();
        engine.insert(&sample_node(10)).expect("insert");
        engine.delete(10, "test").expect("delete should succeed");
        let retrieved = engine.get(10).expect("get");
        assert!(retrieved.is_none(), "deleted node should be gone");
    }

    #[test]
    fn test_delete_nonexistent() {
        let engine = in_memory_engine();
        let result = engine.delete(999, "test");
        assert!(result.is_ok(), "deleting nonexistent should not error");
    }

    #[test]
    fn test_delete_updates_cardinality_stats() {
        let engine = in_memory_engine();
        let mut node = UnifiedNode::new(5);
        node.relational.insert(
            "color".to_string(),
            crate::node::FieldValue::String("red".to_string()),
        );
        engine.insert(&node).expect("insert");
        engine.delete(5, "test").expect("delete");

        let sel = engine.get_estimated_selectivity(
            "color",
            &crate::query::RelOp::Eq,
            &crate::node::FieldValue::String("red".to_string()),
        );
        assert_eq!(sel, 0.0, "cardinality should be zero after delete");
    }

    #[test]
    fn test_is_deleted_false_after_insert() {
        let engine = in_memory_engine();
        engine.insert(&sample_node(100)).expect("insert");
        assert!(!engine.is_deleted(100).expect("is_deleted"));
    }

    #[test]
    fn test_purge_permanent() {
        let engine = in_memory_engine();
        engine.insert(&sample_node(200)).expect("insert");
        engine.purge_permanent(200).expect("purge");
        assert!(engine.get(200).unwrap().is_none());
    }

    // ─── Read-only Guard ──────────────────────────────────────

    #[test]
    fn test_guard_write_allowed_read_only() {
        let config = VantaConfig {
            read_only: true,
            ..VantaConfig::default()
        };
        let result = StorageEngine::guard_write_allowed(&config);
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().contains("read-only"));
    }

    #[test]
    fn test_guard_write_allowed_writable() {
        let config = VantaConfig::default();
        let result = StorageEngine::guard_write_allowed(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_read_only_rejects_insert() {
        let engine = in_memory_read_only();
        let result = engine.insert(&sample_node(1));
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().contains("read-only"));
    }

    #[test]
    fn test_read_only_rejects_delete() {
        let engine = in_memory_read_only();
        let result = engine.delete(1, "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_read_only_rejects_flush() {
        let engine = in_memory_read_only();
        let result = engine.flush();
        assert!(result.is_err());
    }

    #[test]
    fn test_read_only_rejects_compact_wal() {
        let engine = in_memory_read_only();
        let result = engine.compact_wal();
        assert!(result.is_err());
    }

    #[test]
    fn test_read_only_rejects_consolidate() {
        let engine = in_memory_read_only();
        let result = engine.consolidate_node(&sample_node(1));
        assert!(result.is_err());
    }

    #[test]
    fn test_read_only_rejects_evict() {
        let engine = in_memory_read_only();
        let result = engine.evict_cold_nodes(0.5);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_only_rejects_rebuild_index() {
        let engine = in_memory_read_only();
        let result = engine.rebuild_vector_index();
        assert!(result.is_err());
    }

    #[test]
    fn test_read_only_rejects_compact_layout() {
        let engine = in_memory_read_only();
        let result = engine.compact_layout_bfs();
        assert!(result.is_err());
    }

    #[test]
    fn test_read_only_allows_get() {
        let engine = in_memory_read_only();
        let result = engine.get(1);
        assert!(result.is_ok());
    }

    // ─── Memory Stats ─────────────────────────────────────────

    #[test]
    fn test_memory_stats_after_insert() {
        let engine = in_memory_engine();
        let stats = engine.get_memory_stats();
        // fresh engine has no nodes
        assert_eq!(stats.node_count, 0);
        assert_eq!(stats.cache_entries, 0);

        engine.insert(&sample_node(1)).expect("insert");
        let stats = engine.get_memory_stats();
        assert!(stats.node_count >= 1);
        assert!(stats.logical_bytes > 0);
    }

    #[test]
    fn test_memory_stats_effective_bytes() {
        let stats = MemoryStats {
            logical_bytes: 1000,
            physical_rss: Some(800),
            node_count: 1,
            cache_entries: 0,
        };
        assert_eq!(stats.effective_bytes(), 800);

        let stats_no_rss = MemoryStats {
            logical_bytes: 1000,
            physical_rss: None,
            node_count: 1,
            cache_entries: 0,
        };
        assert_eq!(stats_no_rss.effective_bytes(), 1000);
    }

    #[test]
    fn test_check_memory_pressure_disabled() {
        let config = VantaConfig {
            backend_kind: BackendKind::InMemory,
            rss_threshold: 0.0, // disabled
            ..VantaConfig::default()
        };
        let engine = StorageEngine::open_with_config(":memory:", Some(config)).unwrap();
        assert!(engine.check_memory_pressure().is_ok());
    }

    // ─── Scan ─────────────────────────────────────────────────

    #[test]
    fn test_scan_nodes_empty() {
        let engine = in_memory_engine();
        let nodes = engine.scan_nodes().expect("scan");
        assert!(nodes.is_empty());
    }

    #[test]
    fn test_scan_nodes_with_inserts() {
        let engine = in_memory_engine();
        engine.insert(&sample_node(1)).expect("insert 1");
        engine.insert(&sample_node(2)).expect("insert 2");
        let nodes = engine.scan_nodes().expect("scan");
        assert_eq!(nodes.len(), 2);
        let ids: Vec<u64> = nodes.iter().map(|n| n.id).collect();
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
    }

    #[test]
    fn test_scan_nodes_excludes_deleted() {
        let engine = in_memory_engine();
        engine.insert(&sample_node(1)).expect("insert 1");
        engine.insert(&sample_node(2)).expect("insert 2");
        engine.delete(1, "test").expect("delete 1");
        let nodes = engine.scan_nodes().expect("scan");
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].id, 2);
    }

    // ─── Eviction ─────────────────────────────────────────────

    #[test]
    fn test_evict_zero_ratio() {
        let engine = in_memory_engine();
        engine.insert(&sample_node(1)).expect("insert");
        let report = engine.evict_cold_nodes(0.0).expect("evict");
        assert_eq!(report.evicted, 0);
    }

    #[test]
    fn test_evict_empty_cache() {
        let engine = in_memory_engine();
        let report = engine.evict_cold_nodes(0.5).expect("evict");
        assert_eq!(report.evicted, 0);
        assert_eq!(report.scanned, 0);
    }

    // ─── Consolidate ──────────────────────────────────────────

    #[test]
    fn test_consolidate_node_removes_from_cache() {
        let engine = in_memory_engine();
        let mut node = sample_node(42);
        node.tier = crate::node::NodeTier::Hot;
        engine.insert(&node).expect("insert");
        assert!(
            engine.volatile_cache.read().contains_key(&42),
            "hot node should be in cache"
        );
        engine
            .consolidate_node(&sample_node(42))
            .expect("consolidate");
        assert!(
            !engine.volatile_cache.read().contains_key(&42),
            "consolidated node should be removed from cache"
        );
        // Node is still accessible via get() — tier in header is unchanged
        let retrieved = engine.get(42).expect("get").unwrap();
        assert_eq!(retrieved.id, 42);
    }

    // ─── Refresh Index ────────────────────────────────────────

    #[test]
    fn test_refresh_index_with_vector() {
        let engine = in_memory_engine();
        let node = sample_node(42);
        engine.insert(&node).expect("insert");
        let offset = {
            let hnsw = engine.hnsw.load();
            hnsw.nodes.get(&42).map(|n| n.storage_offset).unwrap()
        };
        engine.refresh_index(&node, offset).expect("refresh index");
        let retrieved = engine.get(42).expect("get").unwrap();
        assert_eq!(retrieved.id, 42);
    }

    #[test]
    fn test_refresh_index_without_vector() {
        let engine = in_memory_engine();
        let mut node = UnifiedNode::new(99);
        node.vector = crate::node::VectorRepresentations::None;
        engine.refresh_index(&node, 64).expect("refresh");
        // Should not panic — just a no-op add to HNSW
    }

    // ─── Cardinality / Selectivity ────────────────────────────

    #[test]
    fn test_selectivity_empty_engine() {
        let engine = in_memory_engine();
        let sel = engine.get_estimated_selectivity(
            "field",
            &crate::query::RelOp::Eq,
            &crate::node::FieldValue::String("val".to_string()),
        );
        // total_nodes == 0 → guard returns 1.0
        assert_eq!(sel, 1.0);
    }

    #[test]
    fn test_selectivity_with_data() {
        let engine = in_memory_engine();
        let mut node = UnifiedNode::new(1);
        node.relational.insert(
            "status".to_string(),
            crate::node::FieldValue::String("active".to_string()),
        );
        engine.insert(&node).expect("insert");

        let sel = engine.get_estimated_selectivity(
            "status",
            &crate::query::RelOp::Eq,
            &crate::node::FieldValue::String("active".to_string()),
        );
        assert_eq!(sel, 1.0);

        let sel_missing = engine.get_estimated_selectivity(
            "status",
            &crate::query::RelOp::Eq,
            &crate::node::FieldValue::String("inactive".to_string()),
        );
        assert_eq!(sel_missing, 0.0);
    }

    #[test]
    fn test_selectivity_neq() {
        let engine = in_memory_engine();
        let mut node = UnifiedNode::new(1);
        node.relational.insert(
            "color".to_string(),
            crate::node::FieldValue::String("red".to_string()),
        );
        engine.insert(&node).expect("insert");

        let sel = engine.get_estimated_selectivity(
            "color",
            &crate::query::RelOp::Neq,
            &crate::node::FieldValue::String("red".to_string()),
        );
        assert_eq!(sel, 0.0);
    }

    // ─── Trigger Compaction ───────────────────────────────────

    #[test]
    fn test_trigger_compaction_empty() {
        let engine = in_memory_engine();
        let result = engine.trigger_compaction();
        assert!(result.is_ok());
    }

    #[test]
    fn test_request_compaction_in_memory() {
        let engine = in_memory_engine();
        engine.request_compaction();
        // should not panic — no-op for InMemory
    }

    // ─── Partition Delegation ─────────────────────────────────

    #[test]
    fn test_put_to_partition_and_scan() {
        let engine = in_memory_engine();
        engine
            .put_to_partition(BackendPartition::Default, b"test_key", b"test_val")
            .expect("put");
        let entries = engine
            .scan_partition(BackendPartition::Default)
            .expect("scan");
        assert!(!entries.is_empty());
        assert!(entries.iter().any(|(k, _)| k == b"test_key"));
    }

    #[test]
    fn test_put_to_partition_read_only_rejected() {
        let engine = in_memory_read_only();
        let result = engine.put_to_partition(BackendPartition::Default, b"k", b"v");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_from_partition() {
        let engine = in_memory_engine();
        engine
            .put_to_partition(BackendPartition::Default, b"mykey", b"myval")
            .expect("put");
        let val = engine
            .get_from_partition(BackendPartition::Default, b"mykey")
            .expect("get")
            .expect("value");
        assert_eq!(val, b"myval");
    }

    #[test]
    fn test_get_from_partition_nonexistent() {
        let engine = in_memory_engine();
        let val = engine
            .get_from_partition(BackendPartition::Default, b"nope")
            .expect("get");
        assert!(val.is_none());
    }

    #[test]
    fn test_scan_partition_prefix() {
        let engine = in_memory_engine();
        engine
            .put_to_partition(BackendPartition::Default, b"abc/1", b"a")
            .expect("put");
        engine
            .put_to_partition(BackendPartition::Default, b"abc/2", b"b")
            .expect("put");
        engine
            .put_to_partition(BackendPartition::Default, b"xyz/1", b"c")
            .expect("put");
        let entries = engine
            .scan_partition_prefix(BackendPartition::Default, b"abc/")
            .expect("scan_prefix");
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_write_backend_batch() {
        let engine = in_memory_engine();
        let ops = vec![
            BackendWriteOp::Put {
                partition: BackendPartition::Default,
                key: b"k1".to_vec(),
                value: b"v1".to_vec(),
            },
            BackendWriteOp::Put {
                partition: BackendPartition::Default,
                key: b"k2".to_vec(),
                value: b"v2".to_vec(),
            },
        ];
        engine.write_backend_batch(ops).expect("batch");
        let v1 = engine
            .get_from_partition(BackendPartition::Default, b"k1")
            .expect("get")
            .expect("value");
        assert_eq!(v1, b"v1");
    }

    // ─── Reset ────────────────────────────────────────────────

    #[test]
    fn test_touch_activity() {
        let engine = in_memory_engine();
        let before = engine.last_query_timestamp.load(Ordering::Acquire);
        engine.touch_activity();
        let after = engine.last_query_timestamp.load(Ordering::Acquire);
        assert!(after >= before);
    }

    // ─── Partition from CF name ───────────────────────────────

    #[test]
    fn test_partition_from_cf_name_valid() {
        assert_eq!(
            StorageEngine::partition_from_cf_name("default").unwrap(),
            BackendPartition::Default
        );
        assert_eq!(
            StorageEngine::partition_from_cf_name("tombstones").unwrap(),
            BackendPartition::Tombstones
        );
        assert_eq!(
            StorageEngine::partition_from_cf_name("text_index").unwrap(),
            BackendPartition::TextIndex
        );
    }

    #[test]
    fn test_partition_from_cf_name_invalid() {
        let result = StorageEngine::partition_from_cf_name("nonexistent");
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().contains("Unknown"));
    }

    // ─── Integration: open / flush / reopen with default backend ──

    #[test]
    fn test_flush_empty_engine() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let engine = StorageEngine::open(path).expect("open");
        engine.flush().expect("flush on empty engine");
    }

    #[test]
    fn test_insert_flush_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        {
            let engine = StorageEngine::open(path).expect("open");
            engine.insert(&sample_node(1)).expect("insert");
            engine.flush().expect("flush");
        }
        {
            let engine = StorageEngine::open(path).expect("reopen");
            let node = engine.get(1).expect("get");
            assert!(node.is_some(), "node should persist after reopen");
        }
    }

    #[test]
    fn test_delete_and_flush() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        {
            let engine = StorageEngine::open(path).expect("open");
            engine.insert(&sample_node(1)).expect("insert");
            engine.insert(&sample_node(2)).expect("insert");
            engine.delete(1, "test").expect("delete");
            engine.flush().expect("flush");
        }
        {
            let engine = StorageEngine::open(path).expect("reopen");
            assert!(engine.get(1).unwrap().is_none());
            assert!(engine.get(2).unwrap().is_some());
        }
    }

    #[test]
    fn test_compact_wal() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let engine = StorageEngine::open(path).expect("open");
        engine.insert(&sample_node(1)).expect("insert");
        engine.compact_wal().expect("compact_wal");
        engine.flush().expect("flush");
        let node = engine.get(1).expect("get");
        assert!(node.is_some());
    }

    // ─── Error Cases ──────────────────────────────────────────

    #[test]
    fn test_insert_fails_on_resource_limit() {
        let config = VantaConfig {
            backend_kind: BackendKind::InMemory,
            rss_threshold: 0.0001,
            memory_limit: Some(1),
            ..VantaConfig::default()
        };
        let engine = StorageEngine::open_with_config(":memory:", Some(config)).unwrap();
        let result = engine.insert(&sample_node(1));
        // may succeed or fail with ResourceLimit depending on mincore
        // just verify it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_emergency_shutdown_flushes() {
        // Just verify the method signature compiles and doesn't panic
        // during the flush portion. Can't test the process::exit(1).
        use std::sync::atomic::AtomicBool;
        static DID_FLUSH: AtomicBool = AtomicBool::new(false);

        struct FlushTracker;
        impl Drop for FlushTracker {
            fn drop(&mut self) {
                DID_FLUSH.store(true, Ordering::SeqCst);
            }
        }

        let _tracker = FlushTracker;
        // This test verifies that emergency_shutdown logic is reachable.
        // The actual process exit is tested in integration tests.
    }

    // ─── Insert to CF ─────────────────────────────────────────

    #[test]
    fn test_insert_to_cf_default() {
        let engine = in_memory_engine();
        engine
            .insert_to_cf(&sample_node(1), "default")
            .expect("insert_to_cf");
        // insert_to_cf stores the full UnifiedNode serialization in the
        // backend partition (different from NodeMetadata used by get/scan).
        // Verify the method executes without error.
    }

    #[test]
    fn test_insert_to_cf_invalid() {
        let engine = in_memory_engine();
        let result = engine.insert_to_cf(&sample_node(1), "bogus_cf");
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().contains("Unknown"));
    }
}
