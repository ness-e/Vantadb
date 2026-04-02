use std::sync::atomic::{AtomicUsize, Ordering};
use crate::error::{IadbmsError, Result};
use crate::query::LogicalPlan;

/// Global counter of bytes currently allocated by queries in flight.
pub static ALLOCATED_BYTES: AtomicUsize = AtomicUsize::new(0);

pub struct ResourceGovernor {
    pub max_memory_bytes: usize,
    pub query_timeout_ms: u64,
}

impl ResourceGovernor {
    pub fn new(max_memory_bytes: usize, query_timeout_ms: u64) -> Self {
        Self {
            max_memory_bytes,
            query_timeout_ms,
        }
    }

    /// Request allocation before executing an expensive step
    pub fn request_allocation(&self, bytes: usize) -> Result<()> {
        let current = ALLOCATED_BYTES.load(Ordering::Relaxed);
        if current + bytes > self.max_memory_bytes {
            return Err(IadbmsError::ResourceLimit("OOM Guard triggered: query exceeds soft memory limit.".to_string()));
        }
        ALLOCATED_BYTES.fetch_add(bytes, Ordering::SeqCst);
        Ok(())
    }

    /// Free allocation
    pub fn free_allocation(&self, bytes: usize) {
        ALLOCATED_BYTES.fetch_sub(bytes, Ordering::SeqCst);
    }

    /// Adapts the query plan based on TEMPERATURE
    pub fn apply_temperature_limits(&self, plan: &mut LogicalPlan) {
        if plan.temperature > 0.8 {
            // Aggressive pruning: modify traverse limits, reduce Top-K implicitly if large
            for op in plan.operators.iter_mut() {
                if let crate::query::LogicalOperator::Traverse { max_depth, .. } = op {
                    if *max_depth > 3 {
                        *max_depth = 3; // cap depth due to high heat
                    }
                }
            }
        }
    }
}
