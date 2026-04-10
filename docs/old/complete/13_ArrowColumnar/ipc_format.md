# Apache Arrow Columnar Storage
> **Status**: 🟡 In Progress — FASE 11

## 1. Columnar Memory Layout
While `bincode` serialization provides extreme throughput for single insertions (`RocksDB::put()`), executing aggregations (e.g., computing an average across a relational field in 1 million nodes) becomes memory-bound due to row-based deserialization.
By exposing an Apache Arrow `IPC` format, we allow analytical scans (OLAP) directly over vectors and relational properties using CPU SIMD instructions.

## 2. IPC Conversion
The `columnar.rs` module takes an iterator of `UnifiedNode` structs and maps their scalar properties (`id`, vector contents) into tightly packed `PrimitiveArray` structures. These structures are then exposed to our query executor or exported via IPC payload arrays to LangChain/Pandas wrappers without serialization costs.
