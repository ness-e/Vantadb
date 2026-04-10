# RocksDB Integration Strategy
> **Status**: 🟡 In Progress — FASE 2B

## 1. Zero-Copy Architecture
We use RocksDB as our persistent LSM tree engine. The objective is to deserialize from RocksDB's block cache directly into our structs without redundant `Vec<u8>` allocation:
- RocksDB stores `[u8]` (bincode serialized `UnifiedNode`).
- `rocksdb::DB::get_pinned()` returns a `DBPinnableSlice`.
- We use direct bincode deserialization from the pinned slice to avoid intermediate allocations.

## 2. WAL vs HNSW
HNSW index is intentionally NOT persisted to RocksDB to save write amplification. We rely on RocksDB only for the ground truth of nodes. On restarts, the CP-Index HNSW is rebuilt from scanning RocksDB values in the background.

## 3. Tiered Storage Configuration
- **Hot**: In-Memory (HashMap cache / Block Cache)
- **Warm**: RocksDB SSD, uncompressed
- **Cold**: RocksDB SSD, LZ4 compressed
We tune RocksDB options with:
- `set_use_direct_reads(true)`
- `set_compression_type(DBCompressionType::Lz4)` for bottommost levels.
