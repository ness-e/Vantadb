# Phase 35: MMap Neural Index & Survival Mode

## Overview
ConnectomeDB now supports an adaptive in-memory vector index (HNSW graph). By defaults, it uses standard heap-allocated memory. However, to support low-spec hardware profiles ("Survival Mode" or devices with <16GB RAM), we must be capable of offloading index data structure to standard storage without incurring the overhead of RocksDB compaction serialization for graph traversal.

Phase 35 introduces hardware-adaptive index memory management:
- **`IndexBackend` Abstraction:** The `CPIndex` graph can either sit entirely in `InMemory` vectors or be mapped directly from a custom binary file using zero-copy memory mapping (`MMapFile`).
- **Binary Graph Serialization Roundtrip:** The whole HNSW struct `CPIndex` is tightly packed in a `neural_index.bin` file structure: headers, node vector length details, layer routing arrays, and fast contiguous storage.
- **`memmap2` Integration:** By memory-mapping the serialized `neural_index.bin` file into memory (via `unsafe { MmapMut::map_mut }`), the OS handles RAM paging, keeping peak native memory to a minimum. 
- **Cold-Start Resilience:** Upon initializing the storage engine, the system automatically checks for the `neural_index.bin` file inside the `data` directory. If valid, the system retrieves it locally on startup instantly without rebuilding vectors from RocksDB layer by layer. 

## Architectural Design

### 1. Unified CPIndex Backend 
`src/index.rs` manages where data arrays reside.

```rust
pub enum IndexBackend {
    InMemory,
    MMapFile {
        path: PathBuf,
        mmap: Option<MmapMut>,
    },
}
```
Queries function agnostically to the backend because both implementations ultimately work against standard Rust vectors/maps dynamically loaded from the backed slice.

### 2. Auto-Adaptive Bootstrapping
On `StorageEngine::open`, we probe `HardwareCapabilities`. If RAM < 16GB, `MMapFile` is constructed inherently, allocating only pointers in active memory:
```rust
let use_mmap = caps.profile == HardwareProfile::Survival
    || caps.total_memory < 16 * 1024 * 1024 * 1024
    ...
```

### 3. Graceful Synchronization 
Periodic WAL/Flushes trigger serialization of active Index additions appended towards memory mapped binaries via `sync_to_mmap` function (MMap layout updates). It also automatically falls back to clean reconstruction if headers are found corrupted.
