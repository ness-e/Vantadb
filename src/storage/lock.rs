//! Storage lock management with PID liveness probing and auto-recovery.
//!
//! Replaces uninformative empty lock files with structured JSON metadata:
//! - `pid`: Operating system process ID holding the lock.
//! - `timestamp_ms`: Unix timestamp in milliseconds when the lock was acquired.
//! - `process_name`: Name of the executable holding the lock.
//! - `hostname`: Machine hostname.
//!
//! When lock acquisition fails or during diagnostics, `is_process_alive(pid)`
//! determines if the holder is still active in the kernel process table:
//! - Windows: `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid)`.
//! - Unix: `libc::kill(pid as i32, 0)`.

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

/// Structured lock metadata written to `.vanta.lock`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LockMetadata {
    /// Process ID of the process holding the lock.
    pub pid: u32,
    /// Unix timestamp in milliseconds when the lock was acquired.
    pub timestamp_ms: u64,
    /// Executable name of the process.
    pub process_name: String,
    /// Hostname where the process was spawned.
    pub hostname: String,
}

impl LockMetadata {
    /// Generate current process lock metadata.
    pub fn current() -> Self {
        let pid = std::process::id();
        let timestamp_ms = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let process_name = std::env::current_exe()
            .ok()
            .and_then(|p| p.file_name().map(|s| s.to_string_lossy().to_string()))
            .unwrap_or_else(|| "vantadb".to_string());
        let hostname = std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .unwrap_or_else(|_| "localhost".to_string());

        Self {
            pid,
            timestamp_ms,
            process_name,
            hostname,
        }
    }
}

/// Check if a process with the given PID is currently alive on the host system.
pub fn is_process_alive(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }

    #[cfg(target_arch = "wasm32")]
    {
        true
    }

    #[cfg(windows)]
    {
        use windows_sys::Win32::Foundation::{CloseHandle, FALSE};
        use windows_sys::Win32::System::Threading::{
            GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
        };

        // SAFETY: OpenProcess called with limited query permissions.
        let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid) };
        if handle.is_null() {
            return false;
        }

        let mut exit_code: u32 = 0;
        let success = unsafe { GetExitCodeProcess(handle, &mut exit_code) };
        unsafe { CloseHandle(handle) };

        if success != 0 {
            // 259 is STILL_ACTIVE
            exit_code == 259
        } else {
            false
        }
    }

    #[cfg(all(unix, not(target_arch = "wasm32")))]
    {
        // Signal 0 checks process existence without delivering a signal.
        let res = unsafe { libc::kill(pid as libc::pid_t, 0) };
        if res == 0 {
            true
        } else {
            let err = std::io::Error::last_os_error();
            // If EPERM, process exists but is owned by another user.
            err.raw_os_error() == Some(libc::EPERM)
        }
    }
}

/// Read and parse `LockMetadata` from a `.vanta.lock` file path.
pub fn read_lock_metadata(path: &Path) -> Option<LockMetadata> {
    let mut file = File::open(path).ok()?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).ok()?;
    serde_json::from_str(&contents).ok()
}

/// Write `LockMetadata` to an open lock file.
pub fn write_lock_metadata(file: &File, meta: &LockMetadata) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(meta)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    file.set_len(0)?;
    let mut writer = file;
    writer.seek(SeekFrom::Start(0))?;
    writer.write_all(json.as_bytes())?;
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_process_is_alive() {
        let current_pid = std::process::id();
        assert!(is_process_alive(current_pid));
    }

    #[test]
    fn test_dead_process_is_not_alive() {
        // PID 0 is never a valid user process.
        assert!(!is_process_alive(0));
        // High unlikely PID (reserved or dead)
        assert!(!is_process_alive(4_194_300));
    }

    #[test]
    fn test_lock_metadata_roundtrip() {
        let temp = tempfile::NamedTempFile::new().expect("temp file");
        let path = temp.path();

        let file = File::options()
            .read(true)
            .write(true)
            .open(path)
            .expect("open temp file");

        let meta = LockMetadata::current();
        write_lock_metadata(&file, &meta).expect("write lock metadata");

        let read_meta = read_lock_metadata(path).expect("read metadata");
        assert_eq!(read_meta.pid, meta.pid);
        assert_eq!(read_meta.process_name, meta.process_name);
    }
}
