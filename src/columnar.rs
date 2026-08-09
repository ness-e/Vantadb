//! Apache Arrow columnar conversion for unified nodes.
//!
//! Converts [`UnifiedNode`] collections into Arrow [`RecordBatch`]es,
//! enabling zero-copy analytical scans and Python/Polars interop.

use crate::error::Result;
use crate::node::UnifiedNode;
use arrow::array::{ArrayRef, Float32Array, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

/// Converts a collection of UnifiedNodes into an Apache Arrow RecordBatch.
/// This enables zero-copy SIMD analytical scans directly inside the executor or
/// zero-cost transmission to a Python client (Pandas/Polars).
///
/// Each node's vector is exported as **complete flat columns** — one
/// [`Float32Array`] column per dimension: `vector_d0`, `vector_d1`, …,
/// `vector_d{N-1}`. Binary/none vectors (or shorter vectors than the widest
/// node) fall back to `0.0` per missing component, mirroring the legacy
/// single-component behavior.
pub fn nodes_to_record_batch(nodes: &[UnifiedNode]) -> Result<RecordBatch> {
    let vector_dims: Vec<Vec<f32>> = nodes
        .iter()
        .map(|node| node.vector.to_f32().unwrap_or_default())
        .collect();

    // One flat Float32 column per dimension. Keep at least `vector_d0` so an
    // empty batch still carries the vector slot (matches legacy schema).
    let max_dim = vector_dims.iter().map(Vec::len).max().unwrap_or(0).max(1);

    let mut fields = vec![Field::new("id", DataType::UInt64, false)];
    let mut columns: Vec<ArrayRef> = vec![Arc::new(UInt64Array::from(
        nodes.iter().map(|node| node.id as u64).collect::<Vec<_>>(),
    ))];

    for dim in 0..max_dim {
        let mut values = Vec::with_capacity(nodes.len());
        for v in &vector_dims {
            values.push(v.get(dim).copied().unwrap_or(0.0));
        }
        fields.push(Field::new(
            format!("vector_d{dim}"),
            DataType::Float32,
            true,
        ));
        columns.push(Arc::new(Float32Array::from(values)));
    }

    let batch = RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
        .map_err(|e| crate::error::VantaError::InvalidInput(e.to_string()))?;

    Ok(batch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::UnifiedNode;

    #[test]
    fn test_empty_nodes() {
        let batch = nodes_to_record_batch(&[]).unwrap();
        assert_eq!(batch.num_rows(), 0);
        assert_eq!(batch.num_columns(), 2);
    }

    #[test]
    fn test_single_node_with_full_vector() {
        let node = UnifiedNode::with_vector(42, vec![0.5, 0.6, 0.7]);
        let batch = nodes_to_record_batch(&[node]).unwrap();
        assert_eq!(batch.num_rows(), 1);

        let id_col = batch
            .column(0)
            .as_any()
            .downcast_ref::<UInt64Array>()
            .unwrap();
        let vec_col = batch
            .column(1)
            .as_any()
            .downcast_ref::<Float32Array>()
            .unwrap();
        assert_eq!(id_col.value(0), 42);
        assert_eq!(vec_col.value(0), 0.5);
    }

    #[test]
    fn test_multiple_nodes() {
        let nodes = vec![
            UnifiedNode::with_vector(10, vec![1.0, 2.0]),
            UnifiedNode::with_vector(20, vec![3.0, 4.0]),
            UnifiedNode::with_vector(30, vec![5.0, 6.0]),
        ];
        let batch = nodes_to_record_batch(&nodes).unwrap();
        assert_eq!(batch.num_rows(), 3);

        let id_col = batch
            .column(0)
            .as_any()
            .downcast_ref::<UInt64Array>()
            .unwrap();
        assert_eq!(id_col.values(), &[10u64, 20, 30]);
    }

    #[test]
    fn test_node_without_full_vector() {
        let node = UnifiedNode::new(7); // VectorRepresentations::None
        let batch = nodes_to_record_batch(&[node]).unwrap();
        assert_eq!(batch.num_rows(), 1);

        let vec_col = batch
            .column(1)
            .as_any()
            .downcast_ref::<Float32Array>()
            .unwrap();
        assert_eq!(vec_col.value(0), 0.0); // falls back to 0.0
    }

    #[test]
    fn test_node_with_empty_vector() {
        let node = UnifiedNode::with_vector(8, vec![]);
        let batch = nodes_to_record_batch(&[node]).unwrap();
        assert_eq!(batch.num_rows(), 1);

        let vec_col = batch
            .column(1)
            .as_any()
            .downcast_ref::<Float32Array>()
            .unwrap();
        assert_eq!(vec_col.value(0), 0.0);
    }

    #[test]
    fn test_record_batch_schema() {
        let batch = nodes_to_record_batch(&[]).unwrap();
        let schema = batch.schema();
        assert_eq!(schema.fields().len(), 2);
        assert_eq!(schema.field(0).name(), "id");
        assert_eq!(schema.field(0).data_type(), &DataType::UInt64);
        assert_eq!(schema.field(1).name(), "vector_d0");
        assert_eq!(schema.field(1).data_type(), &DataType::Float32);
    }

    // FEAT-03: export must return the complete flat vector columns (full f32
    // array, correct dimension N), not just the first component.
    #[test]
    fn test_export_returns_full_vector_flat_columns() {
        let node = UnifiedNode::with_vector(42, vec![0.5, 0.6, 0.7]);
        let batch = nodes_to_record_batch(&[node]).unwrap();
        assert_eq!(batch.num_rows(), 1);
        // id + one flat Float32 column per dimension (N = 3).
        assert_eq!(batch.num_columns(), 4);

        let schema = batch.schema();
        assert_eq!(schema.field(1).name(), "vector_d0");
        assert_eq!(schema.field(2).name(), "vector_d1");
        assert_eq!(schema.field(3).name(), "vector_d2");

        let expected = [0.5, 0.6, 0.7];
        for (dim, want) in expected.iter().enumerate() {
            let col = batch
                .column(1 + dim)
                .as_any()
                .downcast_ref::<Float32Array>()
                .unwrap();
            assert_eq!(col.value(0), *want);
        }
    }

    #[test]
    fn test_multiple_nodes_full_vectors_flat_columns() {
        let nodes = vec![
            UnifiedNode::with_vector(10, vec![1.0, 2.0, 3.0]),
            UnifiedNode::with_vector(20, vec![4.0, 5.0, 6.0]),
        ];
        let batch = nodes_to_record_batch(&nodes).unwrap();
        assert_eq!(batch.num_rows(), 2);
        assert_eq!(batch.num_columns(), 4); // id + vector_d0..d2

        let d1 = batch
            .column(2)
            .as_any()
            .downcast_ref::<Float32Array>()
            .unwrap();
        assert_eq!(d1.value(0), 2.0);
        assert_eq!(d1.value(1), 5.0);
    }

    #[test]
    fn test_mixed_dimensions_pad_with_zero() {
        let nodes = vec![
            UnifiedNode::with_vector(10, vec![1.0, 2.0, 3.0]),
            UnifiedNode::with_vector(20, vec![4.0]), // shorter → padded 0.0
        ];
        let batch = nodes_to_record_batch(&nodes).unwrap();
        assert_eq!(batch.num_columns(), 4); // widest node drives columns

        let d1 = batch
            .column(2)
            .as_any()
            .downcast_ref::<Float32Array>()
            .unwrap();
        assert_eq!(d1.value(0), 2.0);
        assert_eq!(d1.value(1), 0.0);
    }
}
