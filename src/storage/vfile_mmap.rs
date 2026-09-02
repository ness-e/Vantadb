// ponytail: mmap-resident byte accounting invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]

//! Memory-mapped primitives for [`crate::storage::vfile::VantaFile`]: the
//! memmap2 re-export / fallback shim, the Unix SIGBUS fault handler, resident
//! byte accounting, and the 4-aligned in-memory buffer.
//!
//! Split from the original vfile.rs god module (REVIEW-04). Items here are
//! re-exported from `vfile.rs` so existing `crate::storage::vfile::*` paths
//! keep resolving unchanged.

use std::fs::File;
#[cfg(not(feature = "memmap2"))]
use std::io::Read;
#[cfg(unix)]
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
#[cfg(unix)]
use tracing::warn;

#[cfg(unix)]
use libc;

use crate::error::{Result, VantaError};

#[cfg(feature = "memmap2")]
pub(crate) use memmap2::{Mmap, MmapMut, MmapOptions};

/// Shim module providing Mmap/MmapMut when the memmap2 feature is disabled.
#[cfg(not(feature = "memmap2"))]
pub(crate) mod mmap_shim {
    #![allow(dead_code)]
    use super::*;
    /// A read-only memory-mapped file backed by an aligned buffer.
    #[derive(Debug)]
    pub struct Mmap(AlignedBytes);
    /// A read-write memory-mapped file backed by an aligned buffer plus a
    /// write-back handle so `flush` can persist buffer writes to disk.
    #[derive(Debug)]
    pub struct MmapMut {
        buf: AlignedBytes,
        /// Cloned backing handle used by `flush` to write the buffer back.
        /// The caller keeps its own handle; this clone drops with the mapping,
        /// so callers can `rename` once the map is dropped (Windows).
        file: File,
    }
    /// Options for creating memory-mapped regions (no-op shim).
    pub struct MmapOptions;

    impl MmapOptions {
        /// Create a new default MmapOptions.
        pub fn new() -> Self {
            Self
        }
        /// Read a file's contents into an aligned buffer — safe, no actual mmap.
        pub fn map(&self, file: &File) -> std::io::Result<Mmap> {
            use std::io::Seek;
            // AlignedBytes guarantees a 4-aligned base so `f32` vector reads are
            // never misaligned in shim (non-memmap2) builds (AUDIT-03).
            let len = file.metadata()?.len() as usize;
            let mut buf =
                AlignedBytes::zeroed(len).map_err(|e| std::io::Error::other(e.to_string()))?;
            let mut f = file.try_clone()?;
            // See `map_mut`: cloned handles share the file position with the
            // caller — read from 0 regardless of where a prior op left it.
            f.seek(std::io::SeekFrom::Start(0))?;
            f.read_exact(buf.as_mut_slice())?;
            Ok(Mmap(buf))
        }
        /// Read a file's contents into a writable buffer — safe, no actual mmap.
        /// The backing handle is cloned and retained so `flush` can write the
        /// buffer back to disk (AUD-044).
        pub fn map_mut(&self, file: &File) -> std::io::Result<MmapMut> {
            use std::io::Seek;
            let len = file.metadata()?.len() as usize;
            let mut buf =
                AlignedBytes::zeroed(len).map_err(|e| std::io::Error::other(e.to_string()))?;
            let mut backing = file.try_clone()?;
            // Cloned handles share the file position with the caller's handle
            // (dup/DuplicateHandle). Seek to 0 so reads start at the beginning
            // even after a previous map left the position at EOF — otherwise
            // remap/grow (`grow_to`, compact_layout's grow path) hit
            // UnexpectedEof (AUD-044 colateral, same root cause family).
            backing.seek(std::io::SeekFrom::Start(0))?;
            backing.read_exact(buf.as_mut_slice())?;
            Ok(MmapMut { buf, file: backing })
        }
    }

    impl Mmap {
        /// Create a new read-only Mmap by reading the file contents.
        /// # Safety
        /// Mirrors memmap2::Mmap::map's safety contract for API compatibility.
        pub unsafe fn map(file: &File) -> std::io::Result<Self> {
            // The body is safe (a plain read into an aligned buffer); the
            // `unsafe fn` signature is kept for API parity with memmap2, whose
            // `Mmap::map` is unsafe — callers (graph.rs, vector_data.rs) wrap
            // it identically on both backends.
            MmapOptions::new().map(file)
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
            self.0.len() == 0
        }
    }
    impl std::ops::Deref for Mmap {
        type Target = [u8];
        fn deref(&self) -> &[u8] {
            self.0.as_slice()
        }
    }
    impl MmapMut {
        /// Create a new read-write MmapMut by reading the file contents.
        /// # Safety
        /// Mirrors memmap2::MmapMut::map_mut's safety contract for API compatibility.
        pub unsafe fn map_mut(file: &File) -> std::io::Result<Self> {
            // Body is safe (a plain read into an aligned buffer); the
            // `unsafe fn` signature is kept for API parity with memmap2 (see
            // `Mmap::map`).
            MmapOptions::new().map_mut(file)
        }
        /// Write the in-memory buffer back to the backing file (seek + write_all
        /// + flush), matching memmap2's `flush` semantics: flush = write-back to
        /// disk.
        ///
        /// AUD-044: the previous no-op silently lost buffer writes before
        /// callers renamed the backing file (compact_layout, sync_to_mmap,
        /// save_vector_index). All callers use position-independent operations
        /// (set_len/sync_all/rename), so moving the shared file position here is
        /// benign.
        #[allow(clippy::doc_lazy_continuation)]
        // ponytail: full-file rewrite per flush — O(file size) vs memmap2's
        // dirty-page msync. Correct, but a bulk workload in a non-memmap2 build
        // pays O(n²) if it flushes per step; track dirty ranges if that
        // measurably matters (shim serves wasm32 + any non-memmap2 native build).
        // Note: like memmap2's msync(MS_SYNC), this is not an fsync — it stops
        // at the OS page cache. Power-loss durability is the WAL's/sync_all's
        // job; don't treat VantaFile::flush() as a durability barrier.
        fn write_back(&self) -> std::io::Result<()> {
            use std::io::{Seek, SeekFrom, Write};
            let mut f = self.file.try_clone()?;
            f.seek(SeekFrom::Start(0))?;
            f.write_all(self.buf.as_slice())?;
            f.flush()
        }
        /// Return a raw pointer to the mapped memory.
        pub fn as_ptr(&self) -> *const u8 {
            self.buf.as_ptr()
        }
        /// Return a mutable raw pointer to the mapped memory.
        pub fn as_mut_ptr(&mut self) -> *mut u8 {
            self.buf.as_mut_slice().as_mut_ptr()
        }
        /// Return the length of the mapped memory.
        pub fn len(&self) -> usize {
            self.buf.len()
        }
        /// Flush outstanding buffer writes to disk (write-back).
        pub fn flush(&self) -> std::io::Result<()> {
            self.write_back()
        }
        /// Async flush — implemented as a synchronous write-back, a valid
        /// (stronger) realization of memmap2's MS_ASYNC semantics.
        pub fn flush_async(&self) -> std::io::Result<()> {
            self.write_back()
        }
        /// Flush a range — a full write-back is a superset of the range guarantee.
        pub fn flush_range(&self, _offset: usize, _len: usize) -> std::io::Result<()> {
            self.write_back()
        }
        /// Returns true if the mapped memory is empty.
        pub fn is_empty(&self) -> bool {
            self.buf.len() == 0
        }
    }
    impl std::ops::Deref for MmapMut {
        type Target = [u8];
        fn deref(&self) -> &[u8] {
            self.buf.as_slice()
        }
    }
    impl std::ops::DerefMut for MmapMut {
        fn deref_mut(&mut self) -> &mut [u8] {
            self.buf.as_mut_slice()
        }
    }
}
#[cfg(not(feature = "memmap2"))]
pub(crate) use mmap_shim::{Mmap, MmapMut, MmapOptions};

/// Map `file` read-only.
///
/// Safe wrapper around [`MmapOptions::map`]. In `memmap2` builds this is the
/// single place the OS-level `unsafe` call happens; in shim (non-`memmap2`,
/// e.g. `wasm32`) builds it is a plain read into an aligned buffer.
pub(crate) fn map_readonly(file: &File) -> std::io::Result<Mmap> {
    #[cfg(feature = "memmap2")]
    {
        // SAFETY: `file` is a valid open handle whose size the caller has
        // already validated/truncated before mapping (VantaFile::open_with_mode
        // truncates to min_header_size; archive.rs set_len()'s the temp file
        // before mapping it). The returned Mmap aliases `file`'s pages, so
        // `file` must stay open and its size unchanged for the mapping's
        // lifetime — guaranteed by VantaFile, which owns the `File` and drops/
        // replaces the mapping together with it (remap_mut, replace_backing_file).
        unsafe { MmapOptions::new().map(file) }
    }
    #[cfg(not(feature = "memmap2"))]
    {
        MmapOptions::new().map(file)
    }
}

/// Map `file` read-write. See [`map_readonly`] for the safety contract.
///
/// Shim note (AUD-044): the no-memmap2 shim's `map_mut`/`flush` move the
/// caller's file position to EOF (clone handles share the offset via
/// dup/DuplicateHandle). This is only safe because every VantaDB caller treats
/// the backing handle as position-independent (set_len/sync_all/rename) — never
/// add a caller that does positional read/write on a mapped handle.
pub(crate) fn map_readwrite(file: &File) -> std::io::Result<MmapMut> {
    #[cfg(feature = "memmap2")]
    {
        // SAFETY: same invariants as `map_readonly`; additionally the caller
        // must not keep another writable mapping of the same region alive —
        // archive.rs drops `tmp_mmap` before extending the file, and VantaFile
        // never holds two mappings of the same file.
        unsafe { MmapOptions::new().map_mut(file) }
    }
    #[cfg(not(feature = "memmap2"))]
    {
        MmapOptions::new().map_mut(file)
    }
}

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
/// (atomic stores on static variables and `_exit`) and never calls into the
/// allocator, libc I/O, or any non-signal-safe function.
///
/// The handler NEVER returns: returning from a SIGBUS handler restores the
/// interrupted context and the kernel re-executes the faulting instruction.
/// Because a SIGBUS occurs when no accessible page backs the faulting address
/// (e.g. a mmap access beyond EOF) and the handler does not repair the
/// mapping, that re-execution faults again → the kernel re-raises SIGBUS →
/// the handler runs again → infinite loop (ERR-002). Terminating with
/// `_exit` is the corrective action: it is async-signal-safe, sets the
/// observable flags first, and deterministically stops the process with the
/// conventional "died by signal" exit code (128 + SIGBUS) instead of hanging.
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
    // SAFETY: `_exit` is async-signal-safe (POSIX) and never returns. It must
    // be the last statement: the handler must not return to the faulting
    // instruction, which would restart the unresolvable fault in a loop.
    libc::_exit(128 + libc::SIGBUS);
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
    #[cfg(miri)]
    {
        // Miri has no kernel-backed page-residency semantics: it does not
        // implement the `mincore`/`QueryWorkingSetEx` host syscalls this fn
        // relies on. The only mapping reachable under Miri is the in-memory
        // (`Vec<u8>`) variant, whose bytes are all in heap RAM → fully
        // resident. Report the whole region as resident (telemetry-only) instead
        // of calling an unsupported host syscall (AUDIT-03).
        let _ = addr;
        return Some(len as u64);
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

/// An owned byte buffer whose base pointer is guaranteed to be 4-byte aligned.
///
/// `Vec<u8>` only guarantees 1-byte alignment, which is fine for the mmap-backed
/// store (mmap returns page-aligned base) but is a latent UB risk for the
/// in-memory store: `f32` vector reads (`from_raw_parts` in `engine/ops.rs`,
/// `index/search.rs`, `storage/archive.rs`) require `base + vector_offset` to be
/// 4-aligned, and that invariant can silently break for an unaligned `Vec<u8>`
/// base in release (AUDIT-03 / INV-024 alignment finding). Giving the in-memory
/// buffer a fixed 4-byte alignment makes the store-side invariant exactly match
/// the mmap-side (page-aligned) one.
#[derive(Debug)]
pub(crate) struct AlignedBytes {
    ptr: std::ptr::NonNull<u8>,
    len: usize,
}

impl AlignedBytes {
    pub(crate) fn zeroed(len: usize) -> Result<Self> {
        // Callers pass `len >= STORAGE_ALIGNMENT`, so size is non-zero. Even so,
        // report, rather than panic on, a layout-overflow (H01-CODE-001): this
        // is a long-lived store path where the error must propagate.
        let layout = std::alloc::Layout::from_size_align(len, 4).map_err(|_| {
            VantaError::ValidationError {
                field: "alloc".into(),
                reason: format!("in-memory vstore buffer size {len} overflows layout with align 4"),
            }
        })?;
        // SAFETY: `layout` is valid (size >= 4, powers-of-two alignment).
        // `alloc_zeroed` returns a pointer to `len` zero-initialized bytes, or
        // null on OOM (checked below); ownership transfers to `AlignedBytes`,
        // whose Drop frees it with the identical layout.
        let ptr = unsafe { std::alloc::alloc_zeroed(layout) };
        let ptr = std::ptr::NonNull::new(ptr).ok_or_else(|| {
            VantaError::ResourceLimit(format!("in-memory vstore allocation of {len} bytes failed"))
        })?;
        Ok(Self { ptr, len })
    }

    pub(crate) fn as_slice(&self) -> &[u8] {
        // SAFETY: `self.ptr` is a valid allocation of exactly `self.len` bytes,
        // live for `self`'s lifetime (forwarded to the returned slice).
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    pub(crate) fn as_ptr(&self) -> *const u8 {
        self.ptr.as_ptr()
    }

    pub(crate) fn as_mut_slice(&mut self) -> &mut [u8] {
        // SAFETY: `&mut self` makes this the only live reference to the buffer,
        // so yielding `&mut [u8]` over the whole allocation is a sound, unique
        // borrow for the slice's lifetime.
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    /// Grow to `new_len` bytes (>= current), preserving content and alignment.
    /// The backing buffer is reallocated with a fresh 4-aligned allocation.
    pub(crate) fn grow_zeroed(&mut self, new_len: usize) -> Result<()> {
        if new_len <= self.len {
            return Ok(());
        }
        let mut grown = AlignedBytes::zeroed(new_len)?;
        grown.as_mut_slice()[..self.len].copy_from_slice(self.as_slice());
        // Drops the old buffer (frees it with its original layout) and moves the
        // new, aligned buffer into place.
        *self = grown;
        Ok(())
    }
}

impl Drop for AlignedBytes {
    fn drop(&mut self) {
        // SAFETY: `self.ptr` was allocated with `{len, align 4}` in `zeroed` and
        // `len` never changes, so this deallocate layout exactly matches the
        // allocation layout (allocator contract).
        unsafe {
            std::alloc::dealloc(
                self.ptr.as_ptr(),
                std::alloc::Layout::from_size_align(self.len, 4)
                    .expect("len with align 4 is valid"),
            )
        }
    }
}

// SAFETY: `AlignedBytes` exclusively owns its aligned `u8` buffer — the same
// ownership model as the `Vec<u8>` backing it replaces (which is Send + Sync).
// The raw pointer is never exposed for mutation without `&mut self` and never
// shared unsafely; there is no interior mutability. So sharing across threads
// (`Sync`) and transferring ownership (`Send`) are both sound.
unsafe impl Send for AlignedBytes {}
unsafe impl Sync for AlignedBytes {}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;

    // ── get_resident_bytes ──

    #[test]
    fn test_get_resident_bytes_empty() {
        // Null or zero-length should return Some(0)
        let result = get_resident_bytes(std::ptr::null(), 0);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_get_resident_bytes_small() {
        // A small valid pointer should not crash
        let data = [0u8; 64];
        let result = get_resident_bytes(data.as_ptr(), data.len());
        // On most platforms this should return Some (≤64 bytes fit in one page)
        // But it's platform-dependent, so just verify it doesn't panic
        let _ = result;
    }

    // ── SIGBUS handler (ERR-002) ──

    /// Install the SIGBUS handler and verify the guards it exposes. A runtime
    /// test of the fault path itself is impractical in-process: triggering a
    /// real SIGBUS requires accessing an mmap page past EOF, and the handler
    /// then terminates the process (`_exit`) by design — the test binary would
    /// die, so only the inert install/flags are asserted here.
    #[cfg(unix)]
    #[test]
    fn test_sigbus_handler_install_is_idempotent() {
        install_sigbus_handler().expect("handler install should succeed");
        // Second install (Once-guarded) must not error or double-register.
        install_sigbus_handler().expect("handler re-install should be a no-op");
        // No fault has occurred: both observability flags stay in default state,
        // proving the handler never ran and installed cleanly.
        assert!(!SIGBUS_OCCURRED.load(Ordering::SeqCst));
        assert!(SIGBUS_FAULT_ADDR.load(Ordering::SeqCst).is_null());
    }

    // ── AUD-044: shim MmapMut flush write-back ──

    /// The no-memmap2 shim's `flush()` used to be a no-op: buffer writes never
    /// reached the backing file before a caller renamed it (compact_layout,
    /// sync_to_mmap, save_vector_index) — silent data loss. `flush()` must
    /// write the buffer back to disk.
    #[cfg(not(feature = "memmap2"))]
    #[test]
    fn shim_mmap_mut_flush_writes_buffer_to_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("shim_flush.vanta");
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .unwrap();
        file.set_len(256).unwrap();

        let mut mmap = unsafe { MmapMut::map_mut(&file).unwrap() };
        let payload: Vec<u8> = (0..256usize).map(|i| (i % 251) as u8).collect();
        mmap.copy_from_slice(&payload);
        mmap.flush().unwrap();
        drop(mmap);
        drop(file);

        let on_disk = std::fs::read(&path).unwrap();
        assert_eq!(
            on_disk, payload,
            "flush() must write the buffer back to the backing file"
        );
    }
}
