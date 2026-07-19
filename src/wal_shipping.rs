//! Async WAL shipping to a remote replica.
//!
//! Feature-gated behind `"wal-shipping"`, which activates `reqwest` (blocking).
//! Ships committed WAL segments via HTTP POST in batches with retry logic.

use crate::error::{Result, VantaError};
use crate::wal::{compute_crc32c, WalReader, WalRecord};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::{error, info, warn};

/// Configuration for shipping WAL segments to a remote replica.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalShipConfig {
    /// URL of the remote replica endpoint (e.g. `http://replica:8080/wal`).
    pub replica_url: String,
    /// Interval in milliseconds between shipping batches (default: 1000).
    #[serde(default = "default_batch_interval")]
    pub batch_interval_ms: u64,
    /// Maximum number of records per batch (default: 100).
    #[serde(default = "default_max_batch_size")]
    pub max_batch_size: usize,
}

fn default_batch_interval() -> u64 {
    1000
}

fn default_max_batch_size() -> usize {
    100
}

impl Default for WalShipConfig {
    fn default() -> Self {
        Self {
            replica_url: String::new(),
            batch_interval_ms: 1000,
            max_batch_size: 100,
        }
    }
}

/// Persistent marker tracking the last successfully shipped WAL position.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ShipMarker {
    last_segment: String,
    last_offset: u64,
    shipped_at: u64,
}

/// Ships committed WAL segments to a remote replica via HTTP POST.
///
/// Scans the WAL directory for rotated (archived) segments, reads their
/// records, batches them, and ships each batch with retry logic (3 attempts,
/// exponential backoff). Tracks progress in a `.wal_ship_marker` file to
/// avoid re-shipping already-delivered segments.
pub struct WalShipper {
    config: WalShipConfig,
    wal_dir: PathBuf,
    archive_dir: PathBuf,
    marker_path: PathBuf,
    client: reqwest::blocking::Client,
}

impl WalShipper {
    /// Create a new `WalShipper`.
    pub fn new(
        config: WalShipConfig,
        wal_dir: impl AsRef<Path>,
        archive_dir: impl AsRef<Path>,
    ) -> std::result::Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let marker_path = wal_dir.as_ref().join(".wal_ship_marker");
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
        Ok(Self {
            client,
            config,
            wal_dir: wal_dir.as_ref().to_path_buf(),
            archive_dir: archive_dir.as_ref().to_path_buf(),
            marker_path,
        })
    }

    /// Run one shipping cycle: discover unsent segments and ship all their records.
    ///
    /// Returns the number of records shipped in this cycle.
    pub fn ship_once(&self) -> Result<u64> {
        let marker = self.load_marker();
        let segments = self.discover_segments(&marker)?;
        let mut total_shipped = 0u64;

        for segment_path in &segments {
            let segment_name = segment_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let mut reader = WalReader::open(segment_path)?;
            let mut batch = Vec::with_capacity(self.config.max_batch_size);

            while let Some(record) = reader.next_record()? {
                batch.push(record);
                if batch.len() >= self.config.max_batch_size {
                    self.ship_batch(&batch)?;
                    total_shipped += batch.len() as u64;
                    batch.clear();
                }
            }

            if !batch.is_empty() {
                self.ship_batch(&batch)?;
                total_shipped += batch.len() as u64;
            }

            self.save_marker(&ShipMarker {
                last_segment: segment_name,
                last_offset: total_shipped,
                shipped_at: web_time::SystemTime::now()
                    .duration_since(web_time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
            })?;
        }

        Ok(total_shipped)
    }

    /// Run the shipping loop continuously (blocking). Never returns.
    pub fn run_loop(&self) -> ! {
        loop {
            match self.ship_once() {
                Ok(n) => {
                    if n > 0 {
                        info!(records = n, "Shipped WAL records to replica");
                    }
                }
                Err(e) => error!(error = %e, "WAL shipping cycle failed"),
            }
            std::thread::sleep(Duration::from_millis(self.config.batch_interval_ms));
        }
    }

    /// Ship a single batch of records with 3 retries and exponential backoff.
    fn ship_batch(&self, records: &[WalRecord]) -> Result<()> {
        let payload = serde_json::to_vec(records)
            .map_err(|e| VantaError::wal_error(format!("Failed to serialize batch: {}", e)))?;

        let checksum = compute_crc32c(&payload);
        let mut last_err = None;

        for attempt in 0..3u32 {
            let backoff_ms = 100u64 * 2u64.pow(attempt);
            match self
                .client
                .post(&self.config.replica_url)
                .header("Content-Type", "application/json")
                .header("X-WAL-Checksum", format!("{:x}", checksum))
                .body(payload.clone())
                .send()
            {
                Ok(resp) => {
                    if resp.status().is_success() {
                        return Ok(());
                    }
                    let status = resp.status();
                    let body = resp.text().unwrap_or_default();
                    last_err = Some(VantaError::wal_error(format!(
                        "Replica returned status {}: {}",
                        status, body
                    )));
                }
                Err(e) => {
                    last_err = Some(VantaError::wal_error(format!("Request failed: {}", e)));
                }
            }
            warn!(
                attempt = attempt + 1,
                backoff_ms = backoff_ms,
                "Retrying WAL batch shipment"
            );
            std::thread::sleep(Duration::from_millis(backoff_ms));
        }

        Err(last_err
            .unwrap_or_else(|| VantaError::wal_error("WAL shipment failed after 3 retries")))
    }

    /// Discover rotated WAL segments that haven't been shipped yet.
    fn discover_segments(&self, marker: &Option<ShipMarker>) -> Result<Vec<PathBuf>> {
        let mut segments = Vec::new();

        for dir in [&self.wal_dir, &self.archive_dir] {
            if !dir.exists() {
                continue;
            }
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    // Match rotated segments (have a timestamp extension after .wal)
                    if name.contains(".wal.") && !name.ends_with(".wal") {
                        segments.push(path);
                    }
                }
            }
        }

        // Sort by modified time so we ship oldest first
        segments.sort_by_key(|p| {
            std::fs::metadata(p)
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        });

        // Skip segments already shipped
        if let Some(m) = marker {
            segments.retain(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n > m.last_segment.as_str())
                    .unwrap_or(true)
            });
        }

        Ok(segments)
    }

    fn load_marker(&self) -> Option<ShipMarker> {
        if !self.marker_path.exists() {
            return None;
        }
        let data = std::fs::read_to_string(&self.marker_path).ok()?;
        if data.len() > 1024 * 1024 {
            tracing::warn!("WAL shipping marker file exceeds 1MB, ignoring");
            return None;
        }
        serde_json::from_str(&data).ok()
    }

    fn save_marker(&self, marker: &ShipMarker) -> Result<()> {
        let data = serde_json::to_string_pretty(marker)
            .map_err(|e| VantaError::wal_error(format!("Failed to serialize marker: {}", e)))?;
        std::fs::write(&self.marker_path, data)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SyncMode;
    use crate::wal::WalWriter;
    use tempfile::tempdir;

    #[test]
    fn test_wal_ship_config_defaults() {
        let cfg = WalShipConfig::default();
        assert_eq!(cfg.batch_interval_ms, 1000);
        assert_eq!(cfg.max_batch_size, 100);
        assert!(cfg.replica_url.is_empty());
    }

    #[test]
    fn test_discover_segments_empty() {
        let dir = tempdir().unwrap();
        let cfg = WalShipConfig::default();
        let shipper = WalShipper::new(cfg, dir.path(), dir.path().join("archive")).unwrap();
        let segments = shipper.discover_segments(&None).unwrap();
        assert!(segments.is_empty());
    }

    #[test]
    fn test_discover_segments_with_rotated_file() {
        let dir = tempdir().unwrap();
        let wal_path = dir.path().join("vanta.wal");

        // Write some records, then rotate to create a timestamped segment
        let w = WalWriter::open(&wal_path, SyncMode::Periodic).unwrap();
        let _rotated = w.rotate(SyncMode::Periodic).unwrap();

        let cfg = WalShipConfig::default();
        let shipper = WalShipper::new(cfg, dir.path(), dir.path().join("archive")).unwrap();
        let segments = shipper.discover_segments(&None).unwrap();
        assert_eq!(segments.len(), 1);
    }
}
