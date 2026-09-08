//! Mmap prefetch helpers and prefetch-mode flag for the HNSW graph.
//! Split from graph.rs (FIND-48) — re-exported via graph/mod.rs.

use crate::config::PrefetchMode;
use std::sync::OnceLock;

#[inline(always)]
#[allow(unused_variables)]
pub(crate) fn prefetch_mmap_vector(mmap_ptr: *const u8, offset: usize, len: usize) {
    #[cfg(unix)]
    {
        // SAFETY: `madvise` is async-signal-safe. Takes a pointer+len derived from
        // the owned mmap; invalid offsets are ignored by the kernel.
        unsafe {
            libc::madvise(
                mmap_ptr.add(offset) as *mut libc::c_void,
                len,
                libc::MADV_WILLNEED,
            );
        }
    }

    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Memory::{PrefetchVirtualMemory, WIN32_MEMORY_RANGE_ENTRY};
        use windows_sys::Win32::System::Threading::GetCurrentProcess;
        // SAFETY: `GetCurrentProcess` returns a pseudo-handle (always valid).
        // `PrefetchVirtualMemory` takes a validated pointer+len from the owned mmap;
        // invalid ranges are best-effort.
        unsafe {
            let addr = mmap_ptr.add(offset) as *mut core::ffi::c_void;
            let entry = WIN32_MEMORY_RANGE_ENTRY {
                VirtualAddress: addr,
                NumberOfBytes: len,
            };
            let process_handle = GetCurrentProcess();
            PrefetchVirtualMemory(process_handle, 1, std::ptr::addr_of!(entry), 0);
        }
    }

    #[cfg(not(any(unix, windows)))]
    let _ = (mmap_ptr, offset, len);
}
#[inline(always)]
/// # Safety
///
/// `mmap_ptr` must point to a valid mmap region, and `offset + len` must be
/// within that region. The caller must ensure the mapping is not concurrently
/// unmapped or resized.
#[allow(unused_variables)]
pub unsafe fn release_mmap_vector(mmap_ptr: *const u8, offset: usize, len: usize) {
    #[cfg(unix)]
    {
        // SAFETY: caller guarantees `mmap_ptr` + `offset + len` is within a valid
        // mmap region. `madvise` with `MADV_DONTNEED` is async-signal-safe; the
        // mapping itself remains valid after the hint.
        unsafe {
            libc::madvise(
                mmap_ptr.add(offset) as *mut libc::c_void,
                len,
                libc::MADV_DONTNEED,
            );
        }
    }

    #[cfg(windows)]
    {
        let _ = (mmap_ptr, offset, len);
    }

    #[cfg(not(any(unix, windows)))]
    let _ = (mmap_ptr, offset, len);
}
static PREFETCH_MODE: OnceLock<PrefetchMode> = OnceLock::new();

pub fn set_prefetch_mode(mode: PrefetchMode) {
    let _ = PREFETCH_MODE.set(mode);
}

#[inline(always)]
pub(crate) fn should_prefetch() -> bool {
    if let Some(mode) = PREFETCH_MODE.get() {
        return mode.is_prefetch_enabled();
    }
    let mode = std::env::var("VANTA_PREFETCH")
        .ok()
        .map(|v| PrefetchMode::from_env_value(&v));
    let disabled = std::env::var("VANTA_DISABLE_PREFETCH")
        .ok()
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false);
    match (mode, disabled) {
        (Some(m), _) => m.is_prefetch_enabled(),
        (_, true) => false,
        // PERF-04: no configuration at all → default OFF (no prefetch syscalls).
        _ => false,
    }
}
