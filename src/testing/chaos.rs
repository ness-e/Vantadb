//! Reusable harness for failpoint-based chaos testing.
//!
//! # Usage
//!
//! ```rust
//! use vantadb::testing::chaos::ChaosTestHarness;
//!
//! let mut chaos = ChaosTestHarness::new().unwrap();
//!
//! // Enable a failpoint — operations that hit it return errors
//! chaos.enable("wal_append_fail", "return");
//! let result = chaos.engine.insert(&vantadb::node::UnifiedNode::new(1));
//! assert!(result.is_err());
//!
//! // Remove all failpoints and verify recovery
//! chaos.disable_all();
//! chaos.assert_recovery();
//!
//! // Clean up
//! chaos.destroy();
//! ```

use std::cell::RefCell;
use std::sync::Arc;
use tempfile::TempDir;

use crate::error::Result;
use crate::storage::StorageEngine;

/// Reusable harness for failpoint-based chaos testing.
///
/// Sets up a tempdir + engine, tracks activated failpoints,
/// and provides `assert_recovery()` that verifies the engine
/// can still read/write after all failpoints are removed.
///
/// # Recovery Guarantee
///
/// After `disable_all()`, the engine must be able to insert and
/// retrieve data. `assert_recovery()` encodes this contract:
/// failpoints simulate transient I/O failures, not permanent
/// corruption.
pub struct ChaosTestHarness {
    /// Shared storage engine under test.
    pub engine: Arc<StorageEngine>,
    /// Temporary directory backing the engine.
    pub dir: TempDir,
    /// Tracks activated failpoints so `disable_all()` can clean them up.
    failpoints: RefCell<Vec<String>>,
}

impl ChaosTestHarness {
    /// Create a new chaos test harness with a fresh tempdir + engine.
    ///
    /// Panics if the tempdir or engine cannot be created (test precondition).
    pub fn new() -> Result<Self> {
        let dir = TempDir::new().expect("ChaosTestHarness: failed to create temp dir");
        let db_path = dir
            .path()
            .to_str()
            .expect("ChaosTestHarness: non-UTF-8 temp dir path");
        let engine = Arc::new(StorageEngine::open(db_path)?);
        Ok(Self {
            engine,
            dir,
            failpoints: RefCell::new(Vec::new()),
        })
    }

    /// Enable a failpoint with the given action.
    ///
    /// The failpoint name is tracked so `disable_all()` can remove it later.
    /// If the same name is enabled twice, both tracked entries are harmless
    /// — `remove_failpoint` is idempotent.
    pub fn enable(&self, name: &str, action: &str) {
        crate::cfg_failpoint(name, action).unwrap_or_else(|e| {
            panic!("ChaosTestHarness: failed to enable failpoint '{name}': {e}")
        });
        self.failpoints.borrow_mut().push(name.to_string());
    }

    /// Disable a specific failpoint by name (removes it from tracking too).
    pub fn disable(&self, name: &str) {
        crate::remove_failpoint(name);
        self.failpoints.borrow_mut().retain(|fp| fp != name);
    }

    /// Disable all tracked failpoints.
    pub fn disable_all(&self) {
        let mut fps = self.failpoints.borrow_mut();
        for fp in fps.drain(..) {
            crate::remove_failpoint(&fp);
        }
    }

    /// Verify that the engine is still operational after failpoint removal.
    ///
    /// Inserts a sentinel node (id = `u128::MAX`) and reads it back.
    /// Panics if the write or read fails — this signals that the engine
    /// state was corrupted by failpoint injection.
    pub fn assert_recovery(&self) {
        let sentinel_id = u128::MAX;
        let node = crate::node::UnifiedNode::new(sentinel_id);
        self.engine
            .insert(&node)
            .expect("ChaosTestHarness: engine must accept writes after recovery");
        let loaded = self
            .engine
            .get(sentinel_id)
            .expect("ChaosTestHarness: engine must serve reads after recovery");
        assert!(
            loaded.is_some(),
            "ChaosTestHarness: sentinel node not found — engine state may be corrupt"
        );
    }

    /// Convenience: disable all failpoints then drop the harness.
    ///
    /// Call this explicitly at the end of each test to clean up.
    /// The `drop` impl also calls `disable_all`, but calling `destroy()`
    /// makes the intent explicit.
    pub fn destroy(self) {
        // disable_all is called in Drop — nothing else needed.
        drop(self);
    }
}

impl Drop for ChaosTestHarness {
    fn drop(&mut self) {
        self.disable_all();
    }
}
