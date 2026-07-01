---
type: glossary-entry
status: stable
tags: [storage, backend, lsm-tree, rust]
last_refined: 2026-06
links: "[[README.md]]"
aliases: [Fjall Storage Engine]
description: "LSM-tree storage engine written in 100% Rust-safe, designed to be embeddable, transactional and high-performance. VantaDB Default Canonical Backend"
---
#Fjall

##Definition

**Fjall** is a **[[lsm-tree]]** (Log-Structured Merge-Tree) storage engine written in **100% secure Rust**, designed to be embeddable, transactional and high-performance. It is the **default canonical** VantaDB backend since version 0.1.4.

## Características Clave

| Característica | Descripción |
|---------------|-------------|
| **100% Safe Rust** | Sin `unsafe`, sin dependencias C++ |
| **LSM-Tree** | Estructura optimizada para escrituras |
| **MVCC** | [[mvcc]] nativo para transacciones concurrentes |
| **Keyspaces** | Aislamiento lógico (similar a column families) |
| **Transacciones ACID** | Soporte completo para atomicidad y durabilidad |
| **Compresión** | LZ4 por defecto, configurable |
| **Bloom Filters** | Para reducir lecturas de disco |

## Why VantaDB Chooses Fjall

### Comparación: Fjall vs RocksDB

| Dimensión | Fjall | RocksDB |
|-----------|-------|---------|
| **Lenguaje** | 100% Rust | C++ (bindings Rust) |
| **Compilación** | Rápida (solo Cargo) | Lenta (requiere CMake, Clang) |
| **Dependencias** | Cero (pure Rust) | libstdc++, libclang |
| **Cross-platform** | Trivial | Problemático en Windows |
| **Seguridad** | Safe Rust | `unsafe` en bindings |
| **Madurez** | Joven (2023+) | Maduro (2012+, Facebook) |
| **Performance** | Excelente | Excelente |
| **Comunidad** | Creciendo | Establecida |

### Strategic Decision

**Fjall as default** because:
1. ✅ Reduce compilation friction (non-C++)
2. ✅ Eliminate system dependencies
3. ✅ Safer (safe Rust)
4. ✅ Aligned with VantaDB's Rust-native identity

**[[rocksdb]] as fallback** for:
- Comparative benchmarking
- Users who require specific RocksDB features
- Migration from existing systems

## Fjall architecture

### Main Components

```
┌─────────────────────────────────────┐
│         Fjall Database               │
├─────────────────────────────────────┤
│  Keyspace 1    Keyspace 2    ...    │
│  ┌─────────┐   ┌─────────┐         │
│  │ MemTable│   │ MemTable│         │
│  └────┬────┘   └────┬────┘         │
│       │              │              │
│  ┌────▼────┐   ┌────▼────┐         │
│  │ SSTable │   │ SSTable │         │
│  │  (L0)   │   │  (L0)   │         │
│  └────┬────┘   └────┬────┘         │
│       │              │              │
│  ┌────▼────┐   ┌────▼────┐         │
│  │ SSTable │   │ SSTable │         │
│  │  (L1)   │   │  (L1)   │         │
│  └─────────┘   └─────────┘         │
├─────────────────────────────────────┤
│         Write-Ahead Log              │
└─────────────────────────────────────┘
```

### Writing Flow

1. **Append to WAL** (durability)
2. **Insert into MemTable** (in memory)
3. **Flush to SSTable** (when MemTable reaches limit size)
4. **Compaction** (merge of SSTables in background)

### Reading Flow

1. **Search MemTable** (latest)
2. **Search in SSTables** (L0 → L1 → ... → Ln)
3. **Bloom Filter** to avoid unnecessary readings
4. **Return value** or `None`

## Usage in VantaDB

### Initialization

```rust
use fjall::{Config, Keyspace};

let db = Config::new("./vantadb_data")
    .open()
    .expect("Failed to open Fjall database");

let documents_ks = db.open_keyspace("documents")?;
let vectors_ks = db.open_keyspace("vectors")?;
let metadata_ks = db.open_keyspace("metadata")?;
```

### Transactions

```rust
// Transacción de escritura
let mut tx = db.write_tx();
tx.insert(&documents_ks, b"doc1", b"contenido");
tx.insert(&vectors_ks, b"doc1", &vector_bytes);
tx.commit()?;  // Atómico: todo o nada

// Read transaction (snapshot)
let tx = db.read_tx();
let doc = tx.get(&documents_ks, b"doc1")?;
let vec = tx.get(&vectors_ks, b"doc1")?;
// Both reads see the same snapshot
```

###Trait StorageBackend

VantaDB abstracts Fjall behind a generic trait:

```rust
pub trait StorageBackend: Send + Sync {
    fn put(&self, key: &[u8], value: &[u8]) -> Result<()>;
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn delete(&self, key: &[u8]) -> Result<()>;
    fn flush(&self) -> Result<()>;
}

pub struct FjallBackend {
    db: fjall::Database,
    keyspaces: HashMap<String, Keyspace>,
}

impl StorageBackend for FjallBackend {
    fn put(&self, key: &[u8], value: &[u8]) -> Result<()> {
        let mut tx = self.db.write_tx();
        tx.insert(&self.keyspaces["default"], key, value);
        tx.commit().map_err(|e| e.into())
    }
    //...
}
```

## Advantages of Fjall for VantaDB

### 1. Quick Compilation

| Backend | Tiempo de Compilación (clean) |
|---------|------------------------------|
| **Fjall** | ~30 segundos |
| **RocksDB** | ~5-10 minutos |

**Impact:** More agile development inner loop.

### 2. Painless Cross-Platform

```bash
# Linux
cargo build  # Funciona

# macOS
cargo build # It works

#Windows
cargo build # Works (no Visual Studio, no CMake)
```

### 3. Memory Security

```rust
// Fjall: Safe Rust
let value = db.get(key)?;  // Sin unsafe

// RocksDB bindings: internal unsafe
let value = unsafe { db.get_unsafe(key) }?;  // Possible UB
```

### 4. MVCC Native

Concurrent transactions without global locks:

```rust
// Thread 1: Lee
let tx1 = db.read_tx();
let v1 = tx1.get(key)?;

// Thread 2: Write (does not block Thread 1)
let mut tx2 = db.write_tx();
tx2.insert(key, new_value);
tx2.commit()?;

// Thread 1: Keep seeing the old snapshot
let v1_again = tx1.get(key)?;  // v1 == v1_again
```

## Configuration in VantaDB

### Default Parameters

```rust
Config::new(path)
    .block_cache_size(64 * 1024 * 1024)  // 64 MB
    .write_buffer_size(16 * 1024 * 1024)  // 16 MB
    .max_write_buffer_number(2)
    .compression(CompressionType::Lz4)
```

### Tuning for Different Loads

| Caso de Uso | block_cache | write_buffer | Compresión |
|-------------|-------------|--------------|------------|
| **Escritura intensiva** | 32 MB | 64 MB | None |
| **Lectura intensiva** | 256 MB | 16 MB | LZ4 |
| **Memoria limitada** | 16 MB | 8 MB | LZ4 |
| **Disco lento** | 128 MB | 32 MB | None |

## Known Issues

### Migration from RocksDB

**Status:** In progress (PHASE 2)

**Challenge:** Users with data in RocksDB need to migrate to Fjall.

**Solution:** Migration Tool:

```bash
vanta migrate --from rocksdb --to fjall --data ./vantadb_data
```

### Write Amplification

**Problem:** LSM-trees suffer from write amplification (writing the same data multiple times due to compactions).

**Mitigation in Fjall:**
- LZ4 compression reduces I/O
- Bloom filters reduce reads
- Tiered (not leveled) compaction for intensive writes

## Comparison with Other Rust Backends

| Backend | LSM-Tree | Transacciones | Madurez | Caso de Uso |
|---------|----------|---------------|---------|-------------|
| **Fjall** | ✅ | ✅ ACID | Joven | VantaDB, apps embebidas |
| **Sled** | ✅ | ⚠️ Básico | Estable | Prototipos |
| **Redb** | B-tree | ✅ ACID | Estable | Apps simples |
| **Sanakirja** | B-tree | ✅ ACID | Estable | Pijul VCS |

## Fjall roadmap in VantaDB

| Fase | Objetivo | Estado |
|------|----------|--------|
| **FASE 1** | Prototipo de integración | ✅ Completado |
| **FASE 2** | Backend por defecto | 🔄 En progreso |
| **FASE 3** | Herramienta de migración | ⬜ Pendiente |
| **FASE 4** | Optimizaciones avanzadas | ⬜ Pendiente |

## See Also

- [[rocksdb]] — Alternative backend
- [[lsm-tree]] — Underlying data structure
- [[mvcc]] — Concurrency control
- [[wal]] — Durability
- [[transactional]] — ACID Guarantees

---

*Fjall represents VantaDB's commitment to a 100% Rust, secure and embeddable stack.*

