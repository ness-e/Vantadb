//! Explicit connection pool for HTTP query execution.
//!
//! A thin RAII wrapper over `tokio::sync::Semaphore` with an acquisition
//! timeout. Guards release their permit on drop, so capacity is always
//! reclaimed. `pool_saturated()` exposes saturation for the circuit breaker.
//!
//! ponytail: intentionally NOT a pooled client (bb8/deadpool/r2d2) — the
//! server execution is `spawn_blocking` per request, so bounding concurrency
//! is all a pool needs to do here.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{OwnedSemaphorePermit, Semaphore};

/// Reason a pool acquisition failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoolError {
    /// The semaphore is closed; no permits can ever be granted.
    Closed,
    /// No permit became available within `pool_acquire_timeout_ms`.
    Timeout,
}

/// Bounded concurrency limiter for query execution.
pub struct ConnectionPool {
    semaphore: Arc<Semaphore>,
    max_connections: usize,
    acquire_timeout: Duration,
    active: Arc<AtomicUsize>,
}

/// An acquired permit; capacity is released when this is dropped.
#[derive(Debug)]
pub struct PoolGuard {
    _permit: OwnedSemaphorePermit,
    active: Arc<AtomicUsize>,
}

impl Drop for PoolGuard {
    fn drop(&mut self) {
        self.active.fetch_sub(1, Ordering::Release);
    }
}

impl ConnectionPool {
    /// Create a pool with the given concurrency cap and acquisition timeout.
    pub fn new(max_connections: usize, acquire_timeout: Duration) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_connections.max(1))),
            max_connections: max_connections.max(1),
            acquire_timeout,
            active: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Acquire a permit, waiting up to the configured timeout.
    pub async fn acquire(&self) -> Result<PoolGuard, PoolError> {
        let semaphore = self.semaphore.clone();
        let active = self.active.clone();
        let permit =
            match tokio::time::timeout(self.acquire_timeout, semaphore.acquire_owned()).await {
                Ok(Ok(permit)) => permit,
                Ok(Err(_)) => return Err(PoolError::Closed),
                Err(_) => return Err(PoolError::Timeout),
            };
        active.fetch_add(1, Ordering::Acquire);
        Ok(PoolGuard {
            _permit: permit,
            active,
        })
    }

    /// Number of permits currently in use.
    pub fn active(&self) -> usize {
        self.active.load(Ordering::Acquire)
    }

    /// Maximum number of concurrent connections.
    pub fn max_connections(&self) -> usize {
        self.max_connections
    }

    /// `true` when every permit is in use (no spare capacity).
    pub fn pool_saturated(&self) -> bool {
        self.active() >= self.max_connections
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU32;

    #[tokio::test]
    async fn test_respects_limit_and_releases_on_drop() {
        let pool = ConnectionPool::new(2, Duration::from_millis(50));
        let g1 = pool.acquire().await.unwrap();
        let g2 = pool.acquire().await.unwrap();
        assert!(pool.pool_saturated());
        assert_eq!(pool.active(), 2);

        drop(g2);
        assert_eq!(pool.active(), 1);
        assert!(!pool.pool_saturated());

        drop(g1);
        assert_eq!(pool.active(), 0);
        // Capacity fully reclaimed — another acquire succeeds instantly.
        let _g3 = pool.acquire().await.unwrap();
        assert_eq!(pool.active(), 1);
    }

    #[tokio::test]
    async fn test_acquire_times_out_when_saturated() {
        let pool = ConnectionPool::new(1, Duration::from_millis(20));
        let _g1 = pool.acquire().await.unwrap();
        assert!(pool.pool_saturated());
        assert!(matches!(pool.acquire().await, Err(PoolError::Timeout)));
    }

    #[tokio::test]
    async fn test_saturation_signal_flips_with_usage() {
        let pool = ConnectionPool::new(3, Duration::from_millis(50));
        assert!(!pool.pool_saturated());
        let g1 = pool.acquire().await.unwrap();
        let g2 = pool.acquire().await.unwrap();
        assert!(!pool.pool_saturated());
        let g3 = pool.acquire().await.unwrap();
        assert!(pool.pool_saturated());
        drop(g1);
        drop(g2);
        drop(g3);
        assert!(!pool.pool_saturated());
    }

    #[tokio::test]
    async fn test_many_guards_drop_under_contention() {
        // Guard against a leak: N sequential acquires with N-1 active must
        // never block forever.
        let pool = Arc::new(ConnectionPool::new(4, Duration::from_millis(200)));
        let counter = Arc::new(AtomicU32::new(0));
        let mut tasks = Vec::new();
        for _ in 0..16 {
            let pool = pool.clone();
            let counter = counter.clone();
            tasks.push(tokio::spawn(async move {
                let _g = pool.acquire().await.unwrap();
                counter.fetch_add(1, Ordering::Relaxed);
            }));
        }
        for t in tasks {
            t.await.unwrap();
        }
        assert_eq!(counter.load(Ordering::Relaxed), 16);
        assert_eq!(pool.active(), 0);
    }
}
