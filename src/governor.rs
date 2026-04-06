use std::sync::atomic::{AtomicUsize, Ordering};
use crate::error::{ConnectomeError, Result};
use crate::query::LogicalPlan;

/// Global counter of bytes currently allocated by queries in flight.
pub static ALLOCATED_BYTES: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, PartialEq)]
pub enum AllocationStatus {
    Granted,
    GrantedWithPressure, // Usado para invocar NMI si es necesario
}

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
    pub fn request_allocation(&self, bytes: usize) -> Result<AllocationStatus> {
        let current = ALLOCATED_BYTES.load(Ordering::Relaxed);
        let new_total = current + bytes;
        
        if new_total > self.max_memory_bytes {
            return Err(ConnectomeError::ResourceLimit("OOM Guard triggered: query exceeds soft memory limit.".to_string()));
        }
        
        let pressure_threshold = (self.max_memory_bytes as f64 * 0.9) as usize;
        let status = if new_total > pressure_threshold {
            AllocationStatus::GrantedWithPressure
        } else {
            AllocationStatus::Granted
        };

        ALLOCATED_BYTES.fetch_add(bytes, Ordering::SeqCst);
        Ok(status)
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
