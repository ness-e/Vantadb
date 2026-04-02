use std::sync::Arc;
use arrow::array::{UInt64Array, Float32Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use crate::error::Result;
use crate::node::UnifiedNode;

/// Converts a collection of UnifiedNodes into an Apache Arrow RecordBatch.
/// This enables zero-copy SIMD analytical scans directly inside the executor or 
/// zero-cost transmission to a Python client (Pandas/Polars).
pub fn nodes_to_record_batch(nodes: &[UnifiedNode]) -> Result<RecordBatch> {
    let mut ids = Vec::with_capacity(nodes.len());
    let mut vec_coords = Vec::new(); // Naive flattened vector logic for MVP
    
    for node in nodes {
        ids.push(node.id);
        // Only push first vector dimension to prove columnar packing capabilities
        if let crate::node::VectorData::F32(ref v) = node.vector {
            if !v.is_empty() {
                vec_coords.push(v[0]);
            } else {
                vec_coords.push(0.0);
            }
        } else {
            vec_coords.push(0.0);
        }
    }

    let id_array = UInt64Array::from(ids);
    let coords_array = Float32Array::from(vec_coords);

    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::UInt64, false),
        Field::new("vector_d0", DataType::Float32, true),
    ]));

    let batch = RecordBatch::try_new(
        schema,
        vec![Arc::new(id_array), Arc::new(coords_array)],
    ).map_err(|e| crate::error::IadbmsError::Execution(e.to_string()))?;

    Ok(batch)
}
