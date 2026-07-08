use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use web_time::{Duration, Instant};

/// State of a pending consistency record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordState {
    PendingConflict,
    ResolvedAccept,
    ResolvedReject,
}

/// A candidate in the consistency buffer.
#[derive(Debug, Clone)]
pub struct CandidateEntry<T: Clone> {
    pub value: T,
    pub origin: String,
    pub confidence: f64,
    pub version: u64,
}

/// A pending record awaiting resolution.
#[derive(Debug, Clone)]
pub struct PendingRecord<T: Clone> {
    pub node_id: u64,
    pub candidates: Vec<CandidateEntry<T>>,
    pub state: RecordState,
    pub injected_at: Instant,
    pub deadline: Instant,
    pub last_touched: Instant,
}

/// Outcome of a batch flush.
#[derive(Debug, Clone)]
pub struct FlushResult<T: Clone> {
    pub accepted: Vec<(u64, T)>,
    pub rejected: Vec<(u64, T)>,
    pub tombstones: Vec<u64>,
}

/// Bounded buffer with TTL-based entry expiry, batch flush on threshold,
/// and backpressure when buffer is full.
pub struct ConsistencyBuffer<T: Clone> {
    buffer: RwLock<HashMap<u64, PendingRecord<T>>>,
    max_size: usize,
    ttl: Duration,
    flush_interval: Duration,
    flush_threshold_count: usize,
    pending_reads: AtomicU64,
    pending_writes: AtomicU64,
    flushed_count: AtomicU64,
    rejected_full_count: AtomicU64,
    last_flush: RwLock<Instant>,
}

impl<T: Clone> ConsistencyBuffer<T> {
    pub fn new(
        max_size: usize,
        ttl: Duration,
        flush_interval: Duration,
        flush_threshold_count: usize,
    ) -> Self {
        Self {
            buffer: RwLock::new(HashMap::with_capacity(max_size)),
            max_size,
            ttl,
            flush_interval,
            flush_threshold_count,
            pending_reads: AtomicU64::new(0),
            pending_writes: AtomicU64::new(0),
            flushed_count: AtomicU64::new(0),
            rejected_full_count: AtomicU64::new(0),
            last_flush: RwLock::new(Instant::now()),
        }
    }

    /// Try to insert a pending record. Returns error with backpressure if buffer is full.
    pub fn try_insert(
        &self,
        node_id: u64,
        candidates: Vec<CandidateEntry<T>>,
    ) -> Result<(), BufferFull<T>> {
        let mut buffer = self.buffer.write().expect("RwLock poisoned");

        if buffer.len() >= self.max_size {
            self.rejected_full_count.fetch_add(1, Ordering::Relaxed);
            return Err(BufferFull {
                node_id,
                candidates,
                message: format!(
                    "Consistency buffer full ({} / {}). Apply backpressure.",
                    buffer.len(),
                    self.max_size
                ),
            });
        }

        let now = Instant::now();
        let record = PendingRecord {
            node_id,
            candidates,
            state: RecordState::PendingConflict,
            injected_at: now,
            deadline: now + self.ttl,
            last_touched: now,
        };
        buffer.insert(node_id, record);
        self.pending_writes.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Read a pending record by node ID.
    pub fn get(&self, node_id: &u64) -> Option<PendingRecord<T>> {
        let buffer = self.buffer.read().expect("RwLock poisoned");
        self.pending_reads.fetch_add(1, Ordering::Relaxed);
        buffer.get(node_id).cloned()
    }

    /// Remove a record from the buffer.
    pub fn remove(&self, node_id: &u64) -> Option<PendingRecord<T>> {
        let mut buffer = self.buffer.write().expect("RwLock poisoned");
        buffer.remove(node_id)
    }

    /// Touch a record to update its deadline.
    pub fn touch(&self, node_id: &u64, extension: Duration) -> bool {
        let mut buffer = self.buffer.write().expect("RwLock poisoned");
        if let Some(record) = buffer.get_mut(node_id) {
            record.last_touched = Instant::now();
            record.deadline = Instant::now() + extension;
            true
        } else {
            false
        }
    }

    /// Expire entries past their TTL deadline. Returns tombstoned node IDs.
    pub fn expire_entries(&self) -> Vec<u64> {
        let mut buffer = self.buffer.write().expect("RwLock poisoned");
        let now = Instant::now();
        let mut tombstones = Vec::new();

        buffer.retain(|&node_id, record| {
            if record.deadline <= now {
                tombstones.push(node_id);
                false
            } else {
                true
            }
        });

        tombstones
    }

    /// Check if flush is needed (time or count threshold).
    pub fn should_flush(&self) -> bool {
        let buffer_len = {
            let buffer = self.buffer.read().expect("RwLock poisoned");
            buffer.len()
        };

        if buffer_len >= self.flush_threshold_count {
            return true;
        }

        let last_flush = *self.last_flush.read().expect("RwLock poisoned");
        last_flush.elapsed() >= self.flush_interval
    }

    /// Flush all records, picking the highest-confidence candidate as winner
    /// and tombstoning the rest. All candidates are recorded in the result.
    pub fn flush_all(&self) -> FlushResult<T> {
        let mut buffer = self.buffer.write().expect("RwLock poisoned");
        let now = Instant::now();
        let mut result = FlushResult {
            accepted: Vec::new(),
            rejected: Vec::new(),
            tombstones: Vec::new(),
        };

        for (node_id, record) in buffer.drain() {
            let winner = record
                .candidates
                .iter()
                .max_by(|a, b| {
                    a.confidence
                        .partial_cmp(&b.confidence)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .cloned();

            match winner {
                Some(win) => {
                    result.accepted.push((node_id, win.value));
                    for c in &record.candidates {
                        let c_confidence = c.confidence;
                        let win_confidence = win.confidence;
                        if (c_confidence - win_confidence).abs() > 1e-9 || c.origin != win.origin {
                            result.rejected.push((node_id, c.value.clone()));
                        }
                    }
                }
                None => {
                    result.tombstones.push(node_id);
                }
            }
        }

        if let Ok(mut lf) = self.last_flush.write() {
            *lf = now;
        }
        self.flushed_count
            .fetch_add(result.accepted.len() as u64, Ordering::Relaxed);

        result
    }

    /// Number of pending records.
    pub fn len(&self) -> usize {
        self.buffer.read().expect("RwLock poisoned").len()
    }

    /// Whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Pending reads count.
    pub fn pending_reads(&self) -> u64 {
        self.pending_reads.load(Ordering::Relaxed)
    }

    /// Pending writes count.
    pub fn pending_writes(&self) -> u64 {
        self.pending_writes.load(Ordering::Relaxed)
    }

    /// Total flush count.
    pub fn flushed_count(&self) -> u64 {
        self.flushed_count.load(Ordering::Relaxed)
    }

    /// Rejected (buffer full) count.
    pub fn rejected_full_count(&self) -> u64 {
        self.rejected_full_count.load(Ordering::Relaxed)
    }
}

/// Error returned when the consistency buffer is full.
#[derive(Debug, Clone)]
pub struct BufferFull<T: Clone> {
    pub node_id: u64,
    pub candidates: Vec<CandidateEntry<T>>,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(value: &str, origin: &str, confidence: f64, version: u64) -> CandidateEntry<String> {
        CandidateEntry {
            value: value.to_string(),
            origin: origin.to_string(),
            confidence,
            version,
        }
    }

    #[test]
    fn test_insert_and_get() {
        let buffer =
            ConsistencyBuffer::new(100, Duration::from_secs(10), Duration::from_secs(60), 50);
        let candidates = vec![entry("val1", "alice", 0.9, 1)];
        assert!(buffer.try_insert(1, candidates.clone()).is_ok());
        let retrieved = buffer.get(&1);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().candidates[0].value, "val1");
    }

    #[test]
    fn test_buffer_full_backpressure() {
        let buffer = ConsistencyBuffer::new(2, Duration::from_secs(10), Duration::from_secs(60), 5);
        assert!(buffer
            .try_insert(1, vec![entry("a", "alice", 0.9, 1)])
            .is_ok());
        assert!(buffer
            .try_insert(2, vec![entry("b", "bob", 0.8, 1)])
            .is_ok());
        let result = buffer.try_insert(3, vec![entry("c", "charlie", 0.7, 1)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_ttl_expiry() {
        let buffer = ConsistencyBuffer::<String>::new(
            100,
            Duration::from_millis(1),
            Duration::from_secs(60),
            50,
        );
        buffer
            .try_insert(1, vec![entry("x", "alice", 0.9, 1)])
            .unwrap();
        std::thread::sleep(Duration::from_millis(5));
        let tombstones = buffer.expire_entries();
        assert!(tombstones.contains(&1));
    }

    #[test]
    fn test_flush_picks_highest_confidence() {
        let buffer =
            ConsistencyBuffer::new(100, Duration::from_secs(10), Duration::from_secs(60), 50);
        buffer
            .try_insert(
                1,
                vec![entry("low", "alice", 0.3, 1), entry("high", "bob", 0.9, 2)],
            )
            .unwrap();
        let result = buffer.flush_all();
        assert_eq!(result.accepted.len(), 1);
        assert_eq!(result.accepted[0].1, "high");
    }

    #[test]
    fn test_flush_tombstones_when_no_candidates() {
        let buffer = ConsistencyBuffer::<String>::new(
            100,
            Duration::from_secs(10),
            Duration::from_secs(60),
            50,
        );
        buffer.try_insert(42, vec![]).unwrap();
        let result = buffer.flush_all();
        assert!(result.tombstones.contains(&42));
    }

    #[test]
    fn test_should_flush_by_count() {
        let buffer =
            ConsistencyBuffer::new(100, Duration::from_secs(10), Duration::from_secs(60), 2);
        buffer
            .try_insert(1, vec![entry("a", "alice", 0.9, 1)])
            .unwrap();
        assert!(!buffer.should_flush());
        buffer
            .try_insert(2, vec![entry("b", "bob", 0.8, 1)])
            .unwrap();
        assert!(buffer.should_flush());
    }

    #[test]
    fn test_record_tracking() {
        let buffer =
            ConsistencyBuffer::new(100, Duration::from_secs(10), Duration::from_secs(60), 50);
        buffer
            .try_insert(1, vec![entry("x", "alice", 0.9, 1)])
            .unwrap();
        assert_eq!(buffer.pending_writes(), 1);
        let _ = buffer.get(&1);
        assert_eq!(buffer.pending_reads(), 1);
    }
}
