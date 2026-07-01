//! Rate limiting and backpressure for query execution.
//!
//! Tracks in-flight memory allocation via [`ALLOCATED_BYTES`] and gates
//! query admission based on budget limits derived from [`LogicalPlan`] cost.

use crate::error::{Result, VantaError};
use crate::query::LogicalPlan;
use std::sync::atomic::{AtomicUsize, Ordering};

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
            return Err(VantaError::ResourceLimit(
                "OOM Guard triggered: query exceeds soft memory limit.".to_string(),
            ));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_allocation_granted() {
        let gov = ResourceGovernor::new(1000, 5000);
        ALLOCATED_BYTES.store(0, Ordering::SeqCst);
        let status = gov.request_allocation(100).unwrap();
        assert_eq!(status, AllocationStatus::Granted);
    }

    #[test]
    fn test_request_allocation_pressure() {
        let gov = ResourceGovernor::new(1000, 5000);
        ALLOCATED_BYTES.store(850, Ordering::SeqCst);
        let status = gov.request_allocation(100).unwrap();
        assert_eq!(status, AllocationStatus::GrantedWithPressure);
    }

    #[test]
    fn test_request_allocation_oom() {
        let gov = ResourceGovernor::new(1000, 5000);
        ALLOCATED_BYTES.store(950, Ordering::SeqCst);
        let result = gov.request_allocation(100);
        assert!(result.is_err());
    }

    #[test]
    fn test_free_allocation_decrements() {
        let gov = ResourceGovernor::new(1000, 5000);
        ALLOCATED_BYTES.store(500, Ordering::SeqCst);
        gov.free_allocation(100);
        assert_eq!(ALLOCATED_BYTES.load(Ordering::SeqCst), 400);
    }

    #[test]
    fn test_request_free_cycle() {
        let gov = ResourceGovernor::new(1000, 5000);
        ALLOCATED_BYTES.store(0, Ordering::SeqCst);
        gov.request_allocation(300).unwrap();
        assert_eq!(ALLOCATED_BYTES.load(Ordering::SeqCst), 300);
        gov.free_allocation(300);
        assert_eq!(ALLOCATED_BYTES.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_apply_temperature_limits_low_temp() {
        let mut plan = LogicalPlan {
            operators: vec![crate::query::LogicalOperator::Traverse {
                min_depth: 1,
                max_depth: 10,
                edge_label: String::new(),
            }],
            temperature: 0.3,
            enforce_role: None,
        };
        let gov = ResourceGovernor::new(1000, 5000);
        gov.apply_temperature_limits(&mut plan);
        if let crate::query::LogicalOperator::Traverse { max_depth, .. } = &plan.operators[0] {
            assert_eq!(*max_depth, 10);
        } else {
            panic!("Expected Traverse operator");
        }
    }

    #[test]
    fn test_apply_temperature_limits_high_temp() {
        let mut plan = LogicalPlan {
            operators: vec![crate::query::LogicalOperator::Traverse {
                min_depth: 1,
                max_depth: 10,
                edge_label: String::new(),
            }],
            temperature: 0.9,
            enforce_role: None,
        };
        let gov = ResourceGovernor::new(1000, 5000);
        gov.apply_temperature_limits(&mut plan);
        if let crate::query::LogicalOperator::Traverse { max_depth, .. } = &plan.operators[0] {
            assert_eq!(*max_depth, 3);
        } else {
            panic!("Expected Traverse operator");
        }
    }
}
