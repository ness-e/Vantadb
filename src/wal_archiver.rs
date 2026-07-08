//! WAL archival and point-in-time recovery (PITR).
//!
//! Feature-gated behind `"pitr"`. Provides:
//! - [`WalArchiver`] for rotating committed WAL segments into an archive directory
//!   with configurable retention (max age, max total size).
//! - [`PitrRestorer`] for replaying archived segments up to a target timestamp.

use crate::error::{Result, VantaError};
use crate::wal::{WalReader, WalRecord};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::{debug, info, warn};

/// Configuration for WAL archival and retention.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalArchiveConfig {
    /// Maximum age of archived segments in seconds (default: 7 days = 604800).
    #[serde(default = "default_max_age_secs")]
    pub max_age_secs: u64,
    /// Maximum total size of archived segments in bytes (default: 1 GB = 1073741824).
    #[serde(default = "default_max_size_bytes")]
    pub max_size_bytes: u64,
}

fn default_max_age_secs() -> u64 {
    604800
}
fn default_max_size_bytes() -> u64 {
    1_073_741_824
}

impl Default for WalArchiveConfig {
    fn default() -> Self {
        Self {
            max_age_secs: 604800,
            max_size_bytes: 1_073_741_824,
        }
    }
}

/// Manages archival of full WAL segments.
///
/// When a WAL segment is full (reaches `wal_buffer_size`), the engine calls
/// [`WalArchiver::archive_segment`] to move the rotated file into the archive
/// directory `{storage_path}/wal/archive/` with a millisecond-precision
/// timestamp in its filename.
pub struct WalArchiver {
    archive_dir: PathBuf,
    config: WalArchiveConfig,
}

impl WalArchiver {
    /// Create a new `WalArchiver`. Creates the archive directory if missing.
    pub fn new(archive_dir: impl AsRef<Path>, config: WalArchiveConfig) -> Result<Self> {
        let archive_dir = archive_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&archive_dir)?;
        Ok(Self {
            archive_dir,
            config,
        })
    }

    /// Move a rotated WAL segment into the archive directory.
    ///
    /// The file is renamed to include a current timestamp, ensuring unique
    /// ordering for PITR recovery.
    pub fn archive_segment(&self, source_path: &Path) -> Result<PathBuf> {
        if !source_path.exists() {
            return Err(VantaError::WalError(format!(
                "WAL segment not found: {}",
                source_path.display()
            )));
        }

        let timestamp = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        let filename = source_path.file_name().unwrap_or_default();
        let archive_name = format!("{}.{}", filename.to_string_lossy(), timestamp);
        let dest = self.archive_dir.join(&archive_name);

        if dest.exists() {
            std::fs::remove_file(&dest)?;
        }
        std::fs::rename(source_path, &dest)?;

        info!(
            source = %source_path.display(),
            dest = %dest.display(),
            "Archived WAL segment"
        );

        Ok(dest)
    }

    /// Apply retention policy: remove segments exceeding `max_age_secs` or
    /// total `max_size_bytes` (removes oldest first when over size limit).
    ///
    /// Returns the number of segments removed.
    pub fn apply_retention(&self) -> Result<u64> {
        let segments = self.list_archived_segments()?;
        if segments.is_empty() {
            return Ok(0);
        }

        let now = web_time::SystemTime::now();
        let max_age = Duration::from_secs(self.config.max_age_secs);
        let mut removed = 0u64;
        let mut total_size = 0u64;

        // First pass: size summation and age-based expiry
        let mut age_expired = Vec::new();
        for seg in &segments {
            if let Ok(meta) = std::fs::metadata(seg) {
                total_size += meta.len();
                if let Ok(modified) = meta.modified() {
                    if now.duration_since(modified).unwrap_or(Duration::ZERO) > max_age {
                        age_expired.push(seg.clone());
                    }
                }
            }
        }

        // Remove age-expired segments
        for seg in &age_expired {
            if let Ok(meta) = std::fs::metadata(seg) {
                if std::fs::remove_file(seg).is_ok() {
                    total_size -= total_size.saturating_sub(meta.len());
                    removed += 1;
                    debug!(path = %seg.display(), "Removed age-expired WAL segment");
                }
            }
        }

        // Second pass: if still over size limit, remove oldest
        if total_size > self.config.max_size_bytes {
            let mut to_reduce = total_size - self.config.max_size_bytes;
            let remaining: Vec<PathBuf> = self
                .list_archived_segments()?
                .into_iter()
                .filter(|p| !age_expired.contains(p))
                .collect();

            for seg in &remaining {
                if to_reduce == 0 {
                    break;
                }
                if let Ok(meta) = std::fs::metadata(seg) {
                    if std::fs::remove_file(seg).is_ok() {
                        to_reduce = to_reduce.saturating_sub(meta.len());
                        removed += 1;
                        debug!(path = %seg.display(), "Removed oversized WAL segment");
                    }
                }
            }
        }

        if removed > 0 {
            info!(removed = removed, "Retention policy applied to WAL archive");
        }

        Ok(removed)
    }

    /// List all archived segments sorted by timestamp (ascending).
    pub fn list_archived_segments(&self) -> Result<Vec<PathBuf>> {
        if !self.archive_dir.exists() {
            return Ok(Vec::new());
        }

        let mut segments: Vec<PathBuf> = std::fs::read_dir(&self.archive_dir)?
            .filter_map(|entry| entry.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();

        segments.sort_by_key(|p| {
            std::fs::metadata(p)
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        });

        Ok(segments)
    }

    /// Return the archive directory path.
    pub fn archive_dir(&self) -> &Path {
        &self.archive_dir
    }

    /// Return a reference to the archive config.
    pub fn config(&self) -> &WalArchiveConfig {
        &self.config
    }
}

/// Point-in-time recovery via replay of archived WAL segments.
///
/// Accepts a target Unix timestamp (milliseconds), lists archived segments
/// sorted by archive time, and replays each segment's records through a
/// user-provided handler, stopping when all segments up to the target
/// time have been replayed.
pub struct PitrRestorer {
    archive_dir: PathBuf,
}

impl PitrRestorer {
    /// Create a new `PitrRestorer`.
    pub fn new(archive_dir: impl AsRef<Path>) -> Self {
        Self {
            archive_dir: archive_dir.as_ref().to_path_buf(),
        }
    }

    /// Replay all archived WAL segments whose archive timestamp is ≤ the target.
    ///
    /// - `target_timestamp_ms`: Unix timestamp in milliseconds. Only segments
    ///   archived at or before this time are replayed.
    /// - `handler`: Callback invoked for each replayed record.
    ///
    /// Returns the total number of records replayed.
    pub fn restore_to_timestamp<F>(&self, target_timestamp_ms: u64, mut handler: F) -> Result<u64>
    where
        F: FnMut(WalRecord) -> Result<()>,
    {
        let segments = self.segments_up_to(target_timestamp_ms)?;
        if segments.is_empty() {
            info!("No archived WAL segments found up to target timestamp");
            return Ok(0);
        }

        let mut total_replayed = 0u64;

        for segment_path in &segments {
            let seg_ts = Self::parse_segment_timestamp(segment_path);
            if seg_ts > target_timestamp_ms {
                break;
            }

            info!(
                path = %segment_path.display(),
                "Replaying archived WAL segment"
            );

            let mut reader = match WalReader::open(segment_path) {
                Ok(r) => r,
                Err(e) => {
                    warn!(
                        path = %segment_path.display(),
                        error = %e,
                        "Skipping unreadable archived WAL segment"
                    );
                    continue;
                }
            };

            let replayed = reader.replay_all(&mut handler)?;
            total_replayed += replayed;
        }

        info!(records = total_replayed, "PITR restore completed");

        Ok(total_replayed)
    }

    /// Return archived segments whose archive timestamp is ≤ the target.
    fn segments_up_to(&self, target_timestamp_ms: u64) -> Result<Vec<PathBuf>> {
        let archiver = WalArchiver::new(&self.archive_dir, WalArchiveConfig::default())?;
        let mut segments = archiver.list_archived_segments()?;

        segments.retain(|p| Self::parse_segment_timestamp(p) <= target_timestamp_ms);

        Ok(segments)
    }

    /// Parse the millisecond timestamp from an archived segment filename.
    ///
    /// Format: `<original_name>.<timestamp_millis>`
    /// Example: `vanta.wal.1712345678901` → `1712345678901`
    fn parse_segment_timestamp(path: &Path) -> u64 {
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if let Some(ts_str) = name.rsplit('.').next() {
            if let Ok(ts) = ts_str.parse::<u64>() {
                return ts;
            }
        }
        // Fallback: use file modification time
        std::fs::metadata(path)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SyncMode;
    use crate::node::UnifiedNode;
    use crate::wal::WalWriter;
    use tempfile::tempdir;

    fn make_wal_segment(dir: &Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        let mut w = WalWriter::open(&path, SyncMode::Periodic).unwrap();
        w.append(&WalRecord::Insert(UnifiedNode::new(1))).unwrap();
        w.sync().unwrap();
        drop(w);
        path
    }

    #[test]
    fn test_archiver_archive_segment() {
        let dir = tempdir().unwrap();
        let wal_dir = dir.path().join("wal");
        std::fs::create_dir_all(&wal_dir).unwrap();
        let seg_path = make_wal_segment(&wal_dir, "vanta.wal");

        let archive_dir = dir.path().join("archive");
        let archiver = WalArchiver::new(&archive_dir, WalArchiveConfig::default()).unwrap();
        let dest = archiver.archive_segment(&seg_path).unwrap();

        assert!(dest.exists());
        assert!(!seg_path.exists());
        assert!(dest
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .contains("vanta.wal."));
    }

    #[test]
    fn test_archiver_list_segments() {
        let dir = tempdir().unwrap();
        let archive_dir = dir.path().join("archive");
        let archiver = WalArchiver::new(&archive_dir, WalArchiveConfig::default()).unwrap();

        // Create two archived segments by making segments then archiving them
        let wal_dir = dir.path().join("wal");
        std::fs::create_dir_all(&wal_dir).unwrap();

        let s1 = make_wal_segment(&wal_dir, "seg1.wal");
        archiver.archive_segment(&s1).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(10));

        let s2 = make_wal_segment(&wal_dir, "seg2.wal");
        archiver.archive_segment(&s2).unwrap();

        let segments = archiver.list_archived_segments().unwrap();
        assert_eq!(segments.len(), 2);
    }

    #[test]
    fn test_restorer_empty_archive() {
        let dir = tempdir().unwrap();
        let archive_dir = dir.path().join("archive");
        std::fs::create_dir_all(&archive_dir).unwrap();

        let restorer = PitrRestorer::new(&archive_dir);
        let count = restorer.restore_to_timestamp(u64::MAX, |_| Ok(())).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_restorer_replays_up_to_target() {
        let dir = tempdir().unwrap();
        let archive_dir = dir.path().join("archive");
        let archiver = WalArchiver::new(&archive_dir, WalArchiveConfig::default()).unwrap();
        let wal_dir = dir.path().join("wal");
        std::fs::create_dir_all(&wal_dir).unwrap();

        let seg = make_wal_segment(&wal_dir, "vanta.wal");
        archiver.archive_segment(&seg).unwrap();

        let restorer = PitrRestorer::new(&archive_dir);
        let now_ms = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // Target way in the future should replay everything
        let mut records = Vec::new();
        let count = restorer
            .restore_to_timestamp(now_ms + 3_600_000, |r| {
                records.push(r);
                Ok(())
            })
            .unwrap();
        assert_eq!(count, 1);

        // Target before the segment should replay nothing
        let mut records2 = Vec::new();
        let count2 = restorer
            .restore_to_timestamp(1, |r| {
                records2.push(r);
                Ok(())
            })
            .unwrap();
        assert_eq!(count2, 0);
    }

    #[test]
    fn test_parse_segment_timestamp() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("vanta.wal.1712345678901");
        assert_eq!(PitrRestorer::parse_segment_timestamp(&path), 1712345678901);

        let path2 = dir.path().join("vanta.wal.shard0.1712345678902");
        assert_eq!(PitrRestorer::parse_segment_timestamp(&path2), 1712345678902);
    }
}
