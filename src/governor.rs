//! Rate limiting and backpressure for query execution.
//!
//! Tracks in-flight memory allocation via [`ALLOCATED_BYTES`] and gates
//! query admission based on budget limits derived from [`LogicalPlan`] cost.

use crate::error::{Result, VantaError};
use crate::query::LogicalPlan;
use crate::storage::StorageEngine;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Global counter of bytes currently allocated by queries in flight.
pub static ALLOCATED_BYTES: AtomicUsize = AtomicUsize::new(0);

/// Result of an allocation request.
#[derive(Debug, Clone, PartialEq)]
pub enum AllocationStatus {
    /// Allocation granted.
    Granted,
    /// Allocation granted, but the system is under memory pressure.
    GrantedWithPressure,
}

/// Memory and timeout resource governor for query execution.
pub struct ResourceGovernor {
    /// Maximum memory in bytes before OOM rejection.
    pub max_memory_bytes: usize,
    /// Query timeout in milliseconds.
    pub query_timeout_ms: u64,
}

impl ResourceGovernor {
    /// Create a new resource governor with the given memory and timeout limits.
    pub fn new(max_memory_bytes: usize, query_timeout_ms: u64) -> Self {
        Self {
            max_memory_bytes,
            query_timeout_ms,
        }
    }

    /// Request allocation before executing an expensive step.
    ///
    /// Uses a CAS loop to atomically check-and-reserve so two concurrent
    /// threads cannot both pass the OOM guard and over-allocate 2×.
    pub fn request_allocation(&self, bytes: usize) -> Result<AllocationStatus> {
        loop {
            let current = ALLOCATED_BYTES.load(Ordering::Relaxed);
            let new_total = current + bytes;

            if new_total > self.max_memory_bytes {
                return Err(VantaError::ResourceLimit(
                    "OOM Guard triggered: query exceeds soft memory limit.".to_string(),
                ));
            }

            if ALLOCATED_BYTES
                .compare_exchange_weak(current, new_total, Ordering::SeqCst, Ordering::Relaxed)
                .is_ok()
            {
                let pressure_threshold = (self.max_memory_bytes as f64 * 0.9) as usize;
                let status = if new_total > pressure_threshold {
                    AllocationStatus::GrantedWithPressure
                } else {
                    AllocationStatus::Granted
                };
                return Ok(status);
            }
        }
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

    /// Estimate the peak memory cost of a logical plan (COMP-028).
    ///
    /// Delegates to the unified semantic cost estimator. Consumed by
    /// [`Executor::execute_plan`] to budget query admission (OLD-21).
    pub(crate) fn estimate_plan_cost(
        &self,
        storage: &StorageEngine,
        plan: &LogicalPlan,
    ) -> crate::cost_estimator::PlanCost {
        crate::cost_estimator::CostEstimator::new(storage).estimate_plan(plan)
    }
}

#[cfg(test)]
#[allow(missing_docs)]
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

    // OLD-21: estimate_plan_cost (COMP-028) is now the admission budget source.
    #[test]
    #[serial_test::serial]
    fn test_estimate_plan_cost_feeds_allocation() {
        use crate::backend::BackendKind;
        use crate::config::VantaConfig;
        use crate::query::LogicalOperator;

        let dir = tempfile::tempdir().unwrap();
        let config = VantaConfig {
            backend_kind: BackendKind::InMemory,
            ..Default::default()
        };
        let storage = StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config))
            .expect("open engine");
        let plan = LogicalPlan {
            operators: vec![LogicalOperator::Scan {
                entity: "people".to_string(),
            }],
            temperature: 0.0,
            enforce_role: None,
        };

        let gov = ResourceGovernor::new(2 * 1024 * 1024 * 1024, 50);
        let cost = gov.estimate_plan_cost(&storage, &plan).estimated_bytes;
        assert!(cost > 0, "estimated plan cost must be non-zero");

        // The exact budget is reserved for the query and returned afterwards.
        ALLOCATED_BYTES.store(0, Ordering::SeqCst);
        gov.request_allocation(cost).unwrap();
        assert_eq!(ALLOCATED_BYTES.load(Ordering::SeqCst), cost);
        gov.free_allocation(cost);
        assert_eq!(ALLOCATED_BYTES.load(Ordering::SeqCst), 0);
        ALLOCATED_BYTES.store(0, Ordering::SeqCst);
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
