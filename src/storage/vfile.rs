//! Memory-mapped vector store file (VantaFile) with read/write and in-memory variants.
//!
//! When the `encryption` feature is enabled, VantaFile can optionally hold a
//! [`Cipher`] instance for transparent at-rest encryption. The cipher is stored
//! for use by the storage layer and can be retrieved via [`VantaFile::cipher`].

use crate::binary_header::VantaHeader;
#[cfg(feature = "encryption")]
use crate::crypto::{Cipher, EncryptionStream};
use crate::error::{Result, VantaError};
use crate::index::CPIndex;
use crate::node::DiskNodeHeader;
use std::fs::{File, OpenOptions};
#[cfg(not(feature = "memmap2"))]
use std::io::Read;
use std::path::PathBuf;
#[cfg(unix)]
use std::sync::atomic::AtomicBool;
use zerocopy::{FromBytes, IntoBytes};

use crate::storage::engine::STORAGE_ALIGNMENT;

/// Current VantaFile format version.
/// Version history:
///   - v1: initial format
///   - v2: migrated (bumped header only, data layout identical to v1)
pub(crate) const VFILE_VERSION: u16 = 2;

#[cfg(feature = "memmap2")]
pub(crate) use memmap2::{Mmap, MmapMut, MmapOptions};

/// Shim module providing Mmap/MmapMut when the memmap2 feature is disabled.
#[cfg(not(feature = "memmap2"))]
pub(crate) mod mmap_shim {
    #![allow(dead_code)]
    use super::*;
    /// A read-only memory-mapped file backed by a `Vec<u8>`.
    #[derive(Debug)]
    pub struct Mmap(Vec<u8>);
    /// A read-write memory-mapped file backed by a `Vec<u8>`.
    #[derive(Debug)]
    pub struct MmapMut(Vec<u8>);
    /// Options for creating memory-mapped regions (no-op shim).
    pub struct MmapOptions;

    impl MmapOptions {
        /// Create a new default MmapOptions.
        pub fn new() -> Self {
            Self
        }
        /// Read a file's contents into a `Vec<u8>` — safe, no actual mmap.
        pub fn map(&self, file: &File) -> std::io::Result<Mmap> {
            let mut v = vec![0u8; file.metadata()?.len() as usize];
            let mut f = file.try_clone()?;
            f.read_exact(&mut v)?;
            Ok(Mmap(v))
        }
        /// Read a file's contents into a writable `Vec<u8>` — safe, no actual mmap.
        pub fn map_mut(&self, file: &File) -> std::io::Result<MmapMut> {
            let mut v = vec![0u8; file.metadata()?.len() as usize];
            let mut f = file.try_clone()?;
            f.read_exact(&mut v)?;
            Ok(MmapMut(v))
        }
    }
    impl Mmap {
        /// Create a new read-only Mmap by reading the file contents.
        /// # Safety
        /// Mirrors memmap2::Mmap::map's safety contract for API compatibility.
        pub unsafe fn map(file: &File) -> std::io::Result<Self> {
            // SAFETY: safe implementation, unsafe for API parity with memmap2
            unsafe { MmapOptions::new().map(file) }
        }
        /// Return a raw pointer to the mapped memory.
        pub fn as_ptr(&self) -> *const u8 {
            self.0.as_ptr()
        }
        /// Return the length of the mapped memory.
        pub fn len(&self) -> usize {
            self.0.len()
        }
        /// No-op flush for the in-memory shim.
        pub fn flush(&self) -> std::io::Result<()> {
            Ok(())
        }
        /// No-op async flush for the in-memory shim.
        pub fn flush_async(&self) -> std::io::Result<()> {
            Ok(())
        }
        /// No-op flush range for the in-memory shim.
        pub fn flush_range(&self, _offset: usize, _len: usize) -> std::io::Result<()> {
            Ok(())
        }
        /// Returns true if the mapped memory is empty.
        pub fn is_empty(&self) -> bool {
            self.0.is_empty()
        }
    }
    impl std::ops::Deref for Mmap {
        type Target = [u8];
        fn deref(&self) -> &[u8] {
            &self.0
        }
    }
    impl MmapMut {
        /// Create a new read-write MmapMut by reading the file contents.
        /// # Safety
        /// Mirrors memmap2::MmapMut::map_mut's safety contract for API compatibility.
        pub unsafe fn map_mut(file: &File) -> std::io::Result<Self> {
            // SAFETY: safe implementation, unsafe for API parity with memmap2
            unsafe { MmapOptions::new().map_mut(file) }
        }
        /// Return a raw pointer to the mapped memory.
        pub fn as_ptr(&self) -> *const u8 {
            self.0.as_ptr()
        }
        /// Return a mutable raw pointer to the mapped memory.
        pub fn as_mut_ptr(&mut self) -> *mut u8 {
            self.0.as_mut_ptr()
        }
        /// Return the length of the mapped memory.
        pub fn len(&self) -> usize {
            self.0.len()
        }
        /// No-op flush for the in-memory shim.
        pub fn flush(&self) -> std::io::Result<()> {
            Ok(())
        }
        /// No-op async flush for the in-memory shim.
        pub fn flush_async(&self) -> std::io::Result<()> {
            Ok(())
        }
        /// No-op flush range for the in-memory shim.
        pub fn flush_range(&self, _offset: usize, _len: usize) -> std::io::Result<()> {
            Ok(())
        }
        /// Returns true if the mapped memory is empty.
        pub fn is_empty(&self) -> bool {
            self.0.is_empty()
        }
    }
    impl std::ops::Deref for MmapMut {
        type Target = [u8];
        fn deref(&self) -> &[u8] {
            &self.0
        }
    }
    impl std::ops::DerefMut for MmapMut {
        fn deref_mut(&mut self) -> &mut [u8] {
            &mut self.0
        }
    }
}
#[cfg(not(feature = "memmap2"))]
pub(crate) use mmap_shim::{Mmap, MmapMut, MmapOptions};

#[cfg(unix)]
use std::sync::atomic::AtomicPtr;
#[cfg(unix)]
use std::sync::atomic::Ordering;
#[cfg(unix)]
use tracing::warn;

#[cfg(unix)]
use libc;

/// Atomic flag set by the SIGBUS handler instead of logging directly.
/// Replaced the previous `warn!()` approach to avoid reentrancy issues
/// inside a signal handler (async-signal-unsafe functions).
#[cfg(unix)]
static SIGBUS_OCCURRED: AtomicBool = AtomicBool::new(false);
#[cfg(unix)]
static SIGBUS_FAULT_ADDR: AtomicPtr<u8> = AtomicPtr::new(std::ptr::null_mut());

/// Install a SIGBUS handler to gracefully catch mmap page faults on Unix.
#[cfg(unix)]
pub(crate) fn install_sigbus_handler() -> Result<()> {
    use std::sync::Once;
    static INSTALL_ONCE: Once = Once::new();
    // SAFETY: `sigaction` is called exactly once (via `Once`). The handler
    // (`sigbus_handler`) is signal-safe (only atomic stores). `sigemptyset`
    // and `sigaction` are async-signal-safe POSIX functions.
    INSTALL_ONCE.call_once(|| unsafe {
        let mut sa: libc::sigaction = std::mem::zeroed();
        sa.sa_sigaction = sigbus_handler as *const () as usize;
        sa.sa_flags = libc::SA_SIGINFO;
        libc::sigemptyset(&mut sa.sa_mask);
        if libc::sigaction(libc::SIGBUS, &sa, std::ptr::null_mut()) != 0 {
            warn!(
                "Failed to install SIGBUS handler: {}",
                std::io::Error::last_os_error()
            );
        }
    });
    Ok(())
}

/// # Safety
///
/// This function is used exclusively as a signal handler for SIGBUS,
/// registered via `sigaction`. It only performs async-signal-safe operations
/// (atomic stores on static variables) and never calls into the allocator,
/// libc I/O, or any non-signal-safe function.
#[cfg(unix)]
unsafe extern "C" fn sigbus_handler(
    _signum: libc::c_int,
    siginfo: *mut libc::siginfo_t,
    _context: *mut libc::c_void,
) {
    SIGBUS_OCCURRED.store(true, Ordering::SeqCst);
    if !siginfo.is_null() {
        // SAFETY: si_addr() is safe to call when siginfo is non-null and
        // we are inside a SIGBUS signal handler (guaranteed by sigaction registration).
        let addr = unsafe { (*siginfo).si_addr() as *mut u8 };
        SIGBUS_FAULT_ADDR.store(addr, Ordering::SeqCst);
    }
}

/// Returns the number of resident (in-RAM) bytes for the given memory region.
pub fn get_resident_bytes(addr: *const u8, len: usize) -> Option<u64> {
    get_resident_bytes_impl(addr, len)
}

/// Platform-specific implementation of resident byte counting via mincore or QueryWorkingSetEx.
pub fn get_resident_bytes_impl(addr: *const u8, len: usize) -> Option<u64> {
    if len == 0 || addr.is_null() {
        return Some(0);
    }
    #[cfg(unix)]
    {
        // SAFETY: `sysconf` is async-signal-safe and POSIX guarantees it returns
        // a positive value for `_SC_PAGESIZE`. This is called during metrics
        // collection; no heap or lock is held that could cause reentrancy issues.
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
        let mut resident_pages = 0u64;
        let mut vec_buffer = vec![0u8; num_pages.min(65536)];
        for chunk_start_page in (0..num_pages).step_by(65536) {
            let pages_in_chunk = (num_pages - chunk_start_page).min(65536);
            let chunk_addr = (aligned_addr + chunk_start_page * page_size) as *mut libc::c_void;
            let chunk_len = pages_in_chunk * page_size;
            // SAFETY: `mincore` is async-signal-safe on both Linux and macOS.
            // `chunk_addr` points to the current aligned region of the mmap;
            // `chunk_len` is bounded by page-aligned size checks above.
            // `vec_buffer` is a valid writable buffer of at least `pages_in_chunk` bytes.
            #[cfg(target_os = "macos")]
            let res = unsafe {
                libc::mincore(
                    chunk_addr,
                    chunk_len,
                    vec_buffer.as_mut_ptr() as *mut libc::c_char,
                )
            };
            #[cfg(not(target_os = "macos"))]
            // SAFETY: same invariants as the macOS branch above — `chunk_addr` is
            // page-aligned, `chunk_len` is bounded, and `vec_buffer` is a valid
            // writable buffer. The pointer cast differs between platforms but the
            // kernel contract is identical.
            let res = unsafe { libc::mincore(chunk_addr, chunk_len, vec_buffer.as_mut_ptr()) };
            if res == 0 {
                for &page_state in vec_buffer.iter().take(pages_in_chunk) {
                    if (page_state & 1) != 0 {
                        resident_pages += 1;
                    }
                }
            } else {
                return None;
            }
        }
        Some(resident_pages * page_size as u64)
    }
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::System::ProcessStatus::{
            QueryWorkingSetEx, PSAPI_WORKING_SET_EX_INFORMATION,
        };
        use windows_sys::Win32::System::Threading::GetCurrentProcess;
        let page_size = 4096usize;
        let addr_val = addr as usize;
        let aligned_addr = addr_val & !(page_size - 1);
        let aligned_len =
            ((len + (addr_val - aligned_addr) + page_size - 1) & !(page_size - 1)).max(page_size);
        let num_pages = aligned_len / page_size;
        let mut resident_pages = 0u64;
        // SAFETY: `GetCurrentProcess` is a trivial Win32 call that always succeeds
        // (returns a pseudo-handle, no cleanup needed).
        let h_process = unsafe { GetCurrentProcess() };
        // SAFETY: `PSAPI_WORKING_SET_EX_INFORMATION` is a POD struct;
        // zero-initialization is valid and fills the buffer for subsequent per-page queries.
        let mut info_buffer = vec![
            unsafe { std::mem::zeroed::<PSAPI_WORKING_SET_EX_INFORMATION>() };
            num_pages.min(65536)
        ];
        for chunk_start_page in (0..num_pages).step_by(65536) {
            let pages_in_chunk = (num_pages - chunk_start_page).min(65536);
            for (i, entry) in info_buffer.iter_mut().enumerate().take(pages_in_chunk) {
                entry.VirtualAddress =
                    (aligned_addr + (chunk_start_page + i) * page_size) as *mut _;
            }
            let cb =
                (pages_in_chunk * std::mem::size_of::<PSAPI_WORKING_SET_EX_INFORMATION>()) as u32;
            // SAFETY: `QueryWorkingSetEx` is a synchronous Win32 API call.
            // `h_process` is a valid pseudo-handle; `info_buffer` is a valid writable
            // buffer of the expected size. Each entry is a POD with the `Flags` field
            // that the kernel populates.
            if unsafe { QueryWorkingSetEx(h_process, info_buffer.as_mut_ptr() as *mut _, cb) } != 0
            {
                for entry in info_buffer.iter().take(pages_in_chunk) {
                    // SAFETY: The kernel has written the entry; reading `Flags` is a
                    // safe bitfield read on the initialized POD.
                    if (unsafe { entry.VirtualAttributes.Flags } & 1) != 0 {
                        resident_pages += 1;
                    }
                }
            } else {
                return None;
            }
        }
        Some(resident_pages * page_size as u64)
    }
    #[cfg(not(any(unix, target_os = "windows")))]
    {
        None
    }
}

/// Sum of resident mmap bytes across the HNSW index and vector store.
pub(crate) fn engine_mmap_resident_bytes(hnsw: &CPIndex, vector_store: &VantaFile) -> Option<u64> {
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

enum VantaFileMap {
    ReadOnly(Mmap),
    ReadWrite(MmapMut),
    InMemory(Vec<u8>),
}

impl VantaFileMap {
    fn as_slice(&self) -> &[u8] {
        match self {
            VantaFileMap::ReadOnly(m) => m,
            VantaFileMap::ReadWrite(m) => m,
            VantaFileMap::InMemory(d) => d,
        }
    }
    fn as_ptr(&self) -> *const u8 {
        match self {
            VantaFileMap::ReadOnly(m) => m.as_ptr(),
            VantaFileMap::ReadWrite(m) => m.as_ptr(),
            VantaFileMap::InMemory(d) => d.as_ptr(),
        }
    }
    fn len(&self) -> usize {
        match self {
            VantaFileMap::ReadOnly(m) => m.len(),
            VantaFileMap::ReadWrite(m) => m.len(),
            VantaFileMap::InMemory(d) => d.len(),
        }
    }
    fn as_mut_slice(&mut self) -> Result<&mut [u8]> {
        match self {
            VantaFileMap::ReadOnly(_) => Err(VantaError::ValidationError {
                field: "read_only".into(),
                reason: "VantaFile is read-only".into(),
            }),
            VantaFileMap::ReadWrite(m) => Ok(m),
            VantaFileMap::InMemory(d) => Ok(d),
        }
    }
    fn flush(&self) -> Result<()> {
        match self {
            VantaFileMap::ReadOnly(_) => Ok(()),
            VantaFileMap::ReadWrite(m) => m.flush().map_err(VantaError::IoError),
            VantaFileMap::InMemory(_) => Ok(()),
        }
    }
}

/// A memory-mapped vector store file supporting read, write, and in-memory modes.
pub struct VantaFile {
    /// Optional backing file handle (None for in-memory mode).
    pub file: Option<File>,
    mmap: VantaFileMap,
    /// File system path to the backing file.
    pub path: PathBuf,
    /// Current file size in bytes.
    pub size: u64,
    /// Byte offset for the next write operation.
    pub write_cursor: u64,
    read_only: bool,
    /// AES-256-GCM cipher for at-rest encryption when the `encryption` feature
    /// is enabled and `VANTADB_ENCRYPTION_KEY` is set.
    #[cfg(feature = "encryption")]
    pub cipher: Option<Cipher>,
}

// SAFETY: VantaFile owns a `File` handle, a `VantaFileMap` (Mmap/MmapMut/Vec<u8>),
// a `PathBuf`, and an `AtomicBool` — all of which are `Send`. The mmap pointers
// are managed by the memmap2 crate or the in-memory shim (Vec<u8>), both of which
// are `Send + Sync`. The cipher field (behind `#[cfg(feature = "encryption")]`) is
// Send by construction. No mutable aliasing crosses threads because all mutations
// go through `&mut self` or the storage engine's locks.
unsafe impl Send for VantaFile {}
// SAFETY: same reasoning — all fields are Sync-safe, and the engine serializes
// read-write access through `RwLock<VantaFile>`.
unsafe impl Sync for VantaFile {}

impl VantaFile {
    /// Open or create a VantaFile at the given path with the specified initial size.
    pub fn open(path: PathBuf, initial_size: u64) -> Result<Self> {
        Self::open_with_mode(path, initial_size, false)
    }
    /// Open an existing VantaFile in read-only mode.
    pub fn open_read_only(path: PathBuf) -> Result<Self> {
        Self::open_with_mode(path, 0, true)
    }

    /// Create a VantaFile backed entirely by in-memory storage (no disk I/O).
    pub fn create_in_memory(initial_size: u64) -> Self {
        let size = initial_size.max(STORAGE_ALIGNMENT);
        let mut data = vec![0u8; size as usize];
        let header = VantaHeader::new(*b"VFLE", 1, 0);
        data[0..16].copy_from_slice(&header.serialize());
        data[16..24].copy_from_slice(&STORAGE_ALIGNMENT.to_le_bytes());
        Self {
            file: None,
            mmap: VantaFileMap::InMemory(data),
            path: PathBuf::new(),
            size,
            write_cursor: STORAGE_ALIGNMENT,
            read_only: false,
            #[cfg(feature = "encryption")]
            cipher: None,
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
                    reason: format!("VantaFile {} too small", path.display()),
                });
            }
            current_size = initial_size.max(min_header_size);
            file.set_len(current_size).map_err(VantaError::IoError)?;
        }
        // SAFETY: `file` is a valid open handle at the correct size (already
        // truncated/validated above). `MmapOptions::map`/`map_mut` from memmap2
        // create kernel-backed mappings; the returned `Mmap`/`MmapMut` is stored
        // in `self.mmap` and lives for the `VantaFile`'s lifetime.
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
        if !read_only && current_size >= min_header_size && &mmap.as_slice()[0..4] != b"VFLE" {
            let header = VantaHeader::new(*b"VFLE", 1, 0);
            mmap.as_mut_slice()?[0..16].copy_from_slice(&header.serialize());
            mmap.as_mut_slice()?[16..24].copy_from_slice(&STORAGE_ALIGNMENT.to_le_bytes());
            // Zero-fill the remainder of the header block (bytes 24..64) to
            // ensure a clean slate for a potentially corrupt or uninitialized file.
            mmap.as_mut_slice()?[24..STORAGE_ALIGNMENT as usize].fill(0);
            mmap.flush()?;
        }
        let header = VantaHeader::deserialize(&mmap.as_slice()[0..16])?;
        if header.format_version != 1 && header.format_version != VFILE_VERSION {
            return Err(VantaError::IncompatibleFormat {
                expected_magic: *b"VFLE",
                expected_version: VFILE_VERSION,
                found_magic: header.magic,
                found_version: header.format_version,
                hint: format!(
                    "VantaFile version {} is not supported (expected 1 or {})",
                    header.format_version, VFILE_VERSION
                ),
            });
        }
        let cursor = u64::from_le_bytes(mmap.as_slice()[16..24].try_into().map_err(|e| {
            VantaError::IoError(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        })?);
        let write_cursor = if cursor < STORAGE_ALIGNMENT || cursor > current_size {
            STORAGE_ALIGNMENT
        } else {
            (cursor + 63) & !63
        };
        Ok(Self {
            file: Some(file),
            mmap,
            path,
            size: current_size,
            write_cursor,
            read_only,
            #[cfg(feature = "encryption")]
            cipher: None,
        })
    }

    /// Persist the write cursor position into the file header.
    pub fn save_cursor(&mut self) -> Result<()> {
        self.mmap.as_mut_slice()?[16..24].copy_from_slice(&self.write_cursor.to_le_bytes());
        Ok(())
    }
    /// Return a byte slice over the entire mapped region.
    pub fn mmap_bytes(&self) -> &[u8] {
        self.mmap.as_slice()
    }
    /// Return a mutable byte slice over the entire mapped region.
    pub(crate) fn mmap_bytes_mut(&mut self) -> Result<&mut [u8]> {
        self.mmap.as_mut_slice()
    }
    /// Re-map the backing file into a new mutable memory mapping.
    pub(crate) fn remap_mut(&mut self) -> Result<()> {
        if self.read_only {
            return Err(VantaError::ValidationError {
                field: "read_only".into(),
                reason: "read-only".into(),
            });
        }
        if matches!(&self.mmap, VantaFileMap::InMemory(_)) {
            return Ok(());
        }
        let file = self
            .file
            .as_ref()
            .ok_or_else(|| VantaError::ValidationError {
                field: "backing_file".into(),
                reason: "no backing file".into(),
            })?;
        // SAFETY: `file` is the existing backing file handle at `self.size` bytes.
        // `MmapMut::map_mut` maps the file into writable memory. The previous
        // mapping is dropped (safe — memmap2 unmaps on Drop).
        self.mmap = VantaFileMap::ReadWrite(unsafe {
            MmapOptions::new()
                .map_mut(file)
                .map_err(VantaError::IoError)?
        });
        Ok(())
    }

    /// Replace the backing file with a new one at the same path and re-map.
    pub(crate) fn replace_backing_file(&mut self, new_size: u64) -> Result<()> {
        if self.read_only {
            return Err(VantaError::ValidationError {
                field: "read_only".into(),
                reason: "read-only".into(),
            });
        }
        if matches!(&self.mmap, VantaFileMap::InMemory(_)) {
            self.size = new_size;
            return Ok(());
        }
        let path = self.path.clone();
        let new_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(false)
            .open(&path)
            .map_err(VantaError::IoError)?;
        self.file = Some(new_file);
        self.size = new_size;
        self.remap_mut()
    }

    /// Read a `DiskNodeHeader` from the given aligned offset, if valid.
    pub fn read_header(&self, offset: u64) -> Option<DiskNodeHeader> {
        let header_size = std::mem::size_of::<DiskNodeHeader>() as u64;
        if offset + header_size > self.size || !offset.is_multiple_of(STORAGE_ALIGNMENT) {
            return None;
        }
        let slice = &self.mmap_bytes()[offset as usize..(offset + header_size) as usize];
        DiskNodeHeader::read_from_bytes(slice).ok()
    }

    /// Write a `DiskNodeHeader` at the given aligned offset, replacing existing bytes.
    pub fn write_header(&mut self, offset: u64, header: &DiskNodeHeader) -> Result<()> {
        let header_size = std::mem::size_of::<DiskNodeHeader>() as u64;
        if !offset.is_multiple_of(STORAGE_ALIGNMENT) {
            return Err(VantaError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "misaligned",
            )));
        }
        if offset + header_size > self.size {
            return Err(VantaError::IoError(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "out of bounds",
            )));
        }
        self.mmap_bytes_mut()?[offset as usize..(offset + header_size) as usize]
            .copy_from_slice(header.as_bytes());
        Ok(())
    }

    /// Extend the file to the given new size, zero-filling added space.
    ///
    /// Shrinking is rejected because the VantaFile layout is append-only:
    /// existing node offsets would become invalid. Use `compact_layout` in
    /// `archive.rs` to reclaim space instead.
    pub fn grow_to(&mut self, new_size: u64) -> Result<()> {
        if new_size < self.size {
            return Err(VantaError::ValidationError {
                field: "new_size".into(),
                reason: format!(
                    "grow_to called with new_size {} < current size {}",
                    new_size, self.size
                ),
            });
        }
        match &mut self.mmap {
            VantaFileMap::InMemory(data) => {
                data.resize(new_size as usize, 0);
                self.size = new_size;
                Ok(())
            }
            _ => {
                let file = self
                    .file
                    .as_ref()
                    .ok_or_else(|| VantaError::ValidationError {
                        field: "backing_file".into(),
                        reason: "no backing file".into(),
                    })?;
                file.set_len(new_size).map_err(VantaError::IoError)?;
                self.size = new_size;
                self.remap_mut()
            }
        }
    }

    /// Flush memory-mapped changes to the backing file (no-op for in-memory mode).
    pub fn flush(&self) -> Result<()> {
        #[cfg(feature = "failpoints")]
        {
            fail::fail_point!("mmap_flush_fail", |_| Err(VantaError::IoError(
                std::io::Error::other("injected")
            )));
        }
        self.mmap.flush()
    }

    /// Advise the OS to prefetch the given number of bytes from the mapped region.
    pub fn warmup_top_layers(&self, _size: usize) {
        #[cfg(all(unix, feature = "memmap2"))]
        {
            use memmap2::Advice;
            let _ = match &self.mmap {
                VantaFileMap::ReadOnly(m) => m.advise(Advice::WillNeed),
                VantaFileMap::ReadWrite(m) => m.advise(Advice::WillNeed),
                VantaFileMap::InMemory(_) => Ok(()),
            };
        }
        #[cfg(not(unix))]
        {
            let mmap = self.mmap_bytes();
            let len = _size.min(mmap.len());
            let mut _sum = 0u8;
            for i in (0..len).step_by(4096) {
                _sum ^= mmap[i];
            }
        }
    }

    /// Return the number of resident (in-RAM) bytes for this file's mapping.
    pub fn mmap_resident_bytes(&self) -> Option<u64> {
        get_resident_bytes_impl(self.mmap.as_ptr(), self.mmap.len())
    }

    /// Attach an encryption cipher to this VantaFile.
    ///
    /// When set, the storage layer should use the cipher to encrypt data before
    /// writing and decrypt after reading. Requires the `encryption` feature.
    #[cfg(feature = "encryption")]
    pub fn with_cipher(mut self, cipher: Cipher) -> Self {
        self.cipher = Some(cipher);
        self
    }

    /// Return a reference to the optional encryption cipher.
    #[cfg(feature = "encryption")]
    pub fn cipher(&self) -> Option<&Cipher> {
        self.cipher.as_ref()
    }

    /// Create an [`EncryptionStream`] wrapping this file's backing [`File`].
    ///
    /// Returns `None` if this VantaFile has no backing file (in-memory mode),
    /// or if no cipher is set.
    ///
    /// The stream can be used for transparent encrypt-on-write and
    /// decrypt-on-read operations on the underlying file handle, for example
    /// with WAL or checkpoint files that use stream-based I/O.
    #[cfg(feature = "encryption")]
    pub fn encryption_stream(&self) -> Option<EncryptionStream<&File>> {
        let file = self.file.as_ref()?;
        let stream_cipher = Cipher::from_env().ok()?;
        Some(EncryptionStream::new(file, stream_cipher))
    }
}
