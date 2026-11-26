//! Type unit tests: QuantizationMaintenanceReport, EvictionReason, EvictionReport,
//! IndexRebuildReport, PendingHnswOp, MemoryStats unit-only tests, constants.

use super::super::*;

// ─── QuantizationMaintenanceReport ────────────────────────────

#[test]
fn test_quantization_maintenance_report_default() {
    let report = QuantizationMaintenanceReport::default();
    assert_eq!(report.scanned, 0);
    assert_eq!(report.quantized, 0);
    assert_eq!(report.promoted, 0);
}

#[test]
fn test_quantization_maintenance_report_creation() {
    let report = QuantizationMaintenanceReport {
        scanned: 42,
        quantized: 10,
        promoted: 2,
    };
    assert_eq!(report.scanned, 42);
    assert_eq!(report.quantized, 10);
    assert_eq!(report.promoted, 2);
}

#[test]
fn test_quantization_maintenance_report_debug() {
    let report = QuantizationMaintenanceReport {
        scanned: 5,
        quantized: 3,
        promoted: 1,
    };
    let s = format!("{report:?}");
    assert!(s.contains("scanned"));
    assert!(s.contains("quantized"));
    assert!(s.contains("promoted"));
}

#[test]
fn test_quantization_maintenance_report_copy() {
    let a = QuantizationMaintenanceReport {
        scanned: 100,
        quantized: 50,
        promoted: 10,
    };
    let b = a;
    assert_eq!(a.scanned, b.scanned);
    assert_eq!(a.quantized, b.quantized);
    assert_eq!(a.promoted, b.promoted);
}

// ─── EvictionReason ───────────────────────────────────────────

#[test]
fn test_eviction_reason_default() {
    assert_eq!(EvictionReason::default(), EvictionReason::Periodic);
}

#[test]
fn test_eviction_reason_variants() {
    assert_eq!(format!("{:?}", EvictionReason::Watermark), "Watermark");
    assert_eq!(format!("{:?}", EvictionReason::Oom), "Oom");
    assert_eq!(format!("{:?}", EvictionReason::Periodic), "Periodic");
    assert_eq!(format!("{:?}", EvictionReason::Manual), "Manual");
    assert_ne!(EvictionReason::Watermark, EvictionReason::Oom);
    assert_ne!(EvictionReason::Manual, EvictionReason::Periodic);
}

#[test]
fn test_eviction_reason_clone_copy() {
    let a = EvictionReason::Watermark;
    let b = a;
    assert_eq!(a, b);
    let c = a;
    assert_eq!(a, c);
}

// ─── EvictionReport ───────────────────────────────────────────

#[test]
fn test_eviction_report_creation() {
    let report = EvictionReport {
        evicted: 10,
        scanned: 100,
        reason: EvictionReason::Watermark,
    };
    assert_eq!(report.evicted, 10);
    assert_eq!(report.scanned, 100);
    assert_eq!(report.reason, EvictionReason::Watermark);
}

#[test]
fn test_eviction_report_debug_clone_copy() {
    let a = EvictionReport {
        evicted: 1,
        scanned: 5,
        reason: EvictionReason::Oom,
    };
    let b = a;
    assert_eq!(a.evicted, b.evicted);
    let c = a;
    assert_eq!(a.reason, c.reason);
}

// ─── IndexRebuildReport ───────────────────────────────────────

#[test]
fn test_index_rebuild_report_creation() {
    let report = IndexRebuildReport {
        scanned_nodes: 1000,
        indexed_vectors: 950,
        skipped_tombstones: 50,
        duration_ms: 1234,
        index_path: std::path::PathBuf::from("/tmp/index.bin"),
        success: true,
    };
    assert_eq!(report.scanned_nodes, 1000);
    assert_eq!(report.indexed_vectors, 950);
    assert_eq!(report.skipped_tombstones, 50);
    assert_eq!(report.duration_ms, 1234);
    assert_eq!(
        report.index_path,
        std::path::PathBuf::from("/tmp/index.bin")
    );
    assert!(report.success);
}

#[test]
fn test_index_rebuild_report_failed() {
    let report = IndexRebuildReport {
        scanned_nodes: 500,
        indexed_vectors: 0,
        skipped_tombstones: 500,
        duration_ms: 0,
        index_path: std::path::PathBuf::from(""),
        success: false,
    };
    assert!(!report.success);
    assert_eq!(report.indexed_vectors, 0);
}

#[test]
fn test_index_rebuild_report_eq() {
    let a = IndexRebuildReport {
        scanned_nodes: 10,
        indexed_vectors: 8,
        skipped_tombstones: 2,
        duration_ms: 100,
        index_path: std::path::PathBuf::from("p"),
        success: true,
    };
    let b = IndexRebuildReport {
        scanned_nodes: 10,
        indexed_vectors: 8,
        skipped_tombstones: 2,
        duration_ms: 100,
        index_path: std::path::PathBuf::from("p"),
        success: true,
    };
    assert_eq!(a, b);
    let c = IndexRebuildReport {
        success: false,
        ..a.clone()
    };
    assert_ne!(a, c);
}

// ─── PendingHnswOp ────────────────────────────────────────────

#[test]
fn test_pending_hnsw_op_insert() {
    use crate::node::FilterBitset;
    let op = PendingHnswOp {
        id: 42,
        bitset: FilterBitset::new(),
        vector: crate::node::VectorRepresentations::Full(vec![0.1, 0.2]),
        storage_offset: 128,
        is_delete: false,
    };
    assert_eq!(op.id, 42);
    assert!(!op.is_delete);
    assert_eq!(op.storage_offset, 128);
}

#[test]
fn test_pending_hnsw_op_delete() {
    let op = PendingHnswOp {
        id: 99,
        bitset: crate::node::FilterBitset::new(),
        vector: crate::node::VectorRepresentations::None,
        storage_offset: 0,
        is_delete: true,
    };
    assert!(op.is_delete);
}

// ─── Constants ────────────────────────────────────────────────

#[test]
fn test_constants() {
    assert_eq!(FLAG_TOMBSTONE, 0x8);
    assert_eq!(MIB, 1024 * 1024);
    assert_eq!(GIB, 1024 * 1024 * 1024);
    const { assert!(STORAGE_ALIGNMENT >= 1) };
}

#[test]
fn test_hnsw_batch_size_default() {
    const { assert!(HNSW_BATCH_SIZE > 0) };
}

#[test]
fn test_storage_alignment_sane_value() {
    const { assert!(STORAGE_ALIGNMENT >= 1) };
    assert_eq!(
        STORAGE_ALIGNMENT % 8,
        0,
        "alignment should be 8-byte aligned"
    );
}

#[test]
fn test_mib_gib_positive() {
    assert_eq!(MIB, 1_048_576);
    assert_eq!(GIB, 1_073_741_824);
}
