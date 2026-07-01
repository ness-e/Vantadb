---
type: glossary-entry
status: stable
tags: [storage, backend, lsm-tree, cpp]
last_refined: 2026-06
links: "[[README.md]]"
aliases: [RocksDB Storage Engine]
description: "High-performance LSM-tree storage engine developed by Facebook in C++. Alternative/fallback backend in VantaDB for benchmarking and compatibility"
---
# RocksDB

## Definition

**RocksDB** is a high-performance **[[lsm-tree]]** storage engine developed by Facebook, written in **C++**. It is widely used in the industry as a storage backend for databases and distributed systems. In VantaDB, RocksDB is the **fallback/alternative backend**, maintained for benchmarking and compatibility.

## History and Adoption

| Año | Hito |
|-----|------|
| 2012 | Facebook inicia RocksDB (fork de LevelDB de Google) |
| 2013 | Open-source bajo licencia BSD |
| 2015+ | Adopción masiva: CockroachDB, TiKV, Flink, Kafka |
| 2020+ | Estándar de facto para LSM-trees en producción |
| 2024+ | VantaDB adopta RocksDB, luego migra a [[fjall]] como default |

## Key Features

| Característica | Descripción |
|---------------|-------------|
| **C++ Nativo** | Performance optimizada, bindings para múltiples lenguajes |
| **LSM-Tree** | Estructura optimizada para escrituras |
| **Column Families** | Aislamiento lógico de datos |
| **Transacciones** | Soporte para optimistic/pessimistic transactions |
| **Compresión** | Múltiples algoritmos: LZ4, Zstd, Snappy |
| **Tuning Avanzado** | Cientos de parámetros configurables |
| **Madurez** | 10+ años en producción a gran escala |

## Why VantaDB Supports RocksDB

### Historical Reasons

1. **Initially the default:** VantaDB started with RocksDB
2. **Proven Maturity:** Used in production by Facebook, CockroachDB, TiKV
3. **Extensive documentation:** Official Wiki with hundreds of pages
4. **Advanced features:** Backup engine, checkpoints, transactions

### Current Reasons (Fallback)

1. **Comparative benchmarking:** Fjall vs RocksDB
2. **Incremental migration:** Users with existing data
3. **Specific Features:** Some users require unique RocksDB capabilities

## RocksDB architecture

### Main Components

```
┌─────────────────────────────────────┐
│         RocksDB Database             │
├─────────────────────────────────────┤
│  Column Family 1    Column Family 2 │
│  ┌─────────┐        ┌─────────┐     │
│  │ MemTable│        │ MemTable│     │
│  └────┬────┘        └────┬────┘     │
│       │                  │          │
│  ┌────▼────┐        ┌────▼────┐     │
│  │ SST     │        │ SST     │     │
│  │ (Level 0)│       │ (Level 0)│    │
│  └────┬────┘        └────┬────┘     │
│       │                  │          │
│  ┌────▼────┐        ┌────▼────┐     │
│  │ SST     │        │ SST     │     │
│  │ (Level 1+)       │ (Level 1+)    │
│  └─────────┘        └─────────┘     │
├─────────────────────────────────────┤
│         Write-Ahead Log              │
└─────────────────────────────────────┘
```

## Usage in VantaDB

### Enable RocksDB

```toml
# Cargo.toml
[dependencies]
vantadb = { version = "0.1.4", features = ["rocksdb"] }
```

### Initialization

```rust
use vantadb::storage::RocksDbBackend;

let backend = RocksDbBackend::open("./vantadb_data")?;
```

### Advanced Settings

```rust
use rocksdb::{Options, DB};

let mut opts = Options::default();
opts.create_if_missing(true);
opts.set_write_buffer_size(64 * 1024 * 1024);  // 64MB
opts.set_max_write_buffer_number(3);
opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
opts.set_level_compaction_dynamic_level_bytes(true);

let db = DB::open(&opts, path)?;
```

## Comparación Detallada: Fjall vs RocksDB

### Performance

| Operación | Fjall | RocksDB | Ganador |
|-----------|-------|---------|---------|
| **Escritura secuencial** | 150K ops/s | 180K ops/s | RocksDB (+20%) |
| **Lectura aleatoria** | 45K ops/s | 50K ops/s | RocksDB (+11%) |
| **Compilación (clean)** | 30s | 5-10min | **Fjall** (10-20x) |
| **Memoria base** | 20 MB | 50 MB | **Fjall** (2.5x) |

### Developer Experience

| Aspecto | Fjall | RocksDB |
|---------|-------|---------|
| **Instalación** | `cargo add fjall` | Requiere CMake, Clang, libstdc++ |
| **Windows** | Funciona out-of-the-box | Requiere Visual Studio + vcpkg |
| **Cross-compile** | Trivial | Problemático |
| **Debugging** | Rust debugger | C++ debugger + Rust bindings |
| **Error messages** | Rust nativo | C++ errors envueltos |

### Security

| Aspecto | Fjall | RocksDB |
|---------|-------|---------|
| **Memory safety** | 100% safe Rust | `unsafe` en bindings |
| **UB potential** | Cero | Posible en FFI |
| **Auditability** | Fácil (Rust) | Difícil (C++ + Rust) |

## RocksDB issues in VantaDB

### 1. Compilation Friction

```
Error: failed to run custom build command for `librocksdb-sys v0.11.0`

Caused by:
  process didn't exit successfully: `.../build-script-build`
  
  ---stderr
  CMake Error: CMake was unable to find a build program
  ...
```

**Impact:** Slow CI/CD, Windows issues, difficult onboarding.

### 2. System Dependencies

```bash
# Ubuntu
sudo apt-get install libclang-dev cmake

# macOS
brew install cmake llvm

#Windows
# Install Visual Studio 2022 with C++ workload
# Install CMake
# Configure environment variables
```

**Impact:** It is not zero-config, it contradicts [Embedded] philosophy (Embebido.md).

### 3. Inflated Binaries

| Backend | Tamaño del Binario |
|---------|-------------------|
| **Fjall** | ~15 MB |
| **RocksDB** | ~45 MB |

**Impact:** 3x larger, affects distribution.

### 4. Cross-Platform Issues

**Windows:**
- Requires `rust-lld` as linker
- Problems with `dbghelp.lib`
- Incompatibilities with MSVC

**macOS ARM64:**
- Problems with `mincore` (different signature)
- Requires conditional compilation

## Migración a Fjall

### Timeline

| Fecha | Hito |
|-------|------|
| **2026-Q1** | VantaDB usa RocksDB como default |
| **2026-Q2** | Fjall introducido como alternativa |
| **2026-Q3** | Fjall se convierte en default (v0.1.4) |
| **2026-Q4** | Herramienta de migración disponible |
| **2027-Q1** | RocksDB marcado como deprecated |
| **2027-Q2** | RocksDB removido del default |

### Migration Tool

```bash
# Migrar datos de RocksDB a Fjall
vanta migrate \
  --from rocksdb \
  --to fjall \
  --source ./vantadb_rocksdb \
  --target ./vantadb_fjall \
  --verify
```

## When to Use RocksDB (Yet)

### Valid Cases

1. **Migración pendiente:** Todavía no has migrado a Fjall
2. **Benchmarking:** Comparar performance vs Fjall
3. **Features específicas:** Necesitas algo que solo RocksDB tiene
4. **Compatibilidad:** Integración con sistemas que usan RocksDB

### Invalid Cases

1. **"RocksDB is more mature":** Fjall is stable enough for production
2. **"Better performance":** Marginal difference (<20%), does not justify friction
3. **"We always use it":** Technical debt, not technical reason

## RocksDB optimizations in VantaDB

### Configuration for VantaDB

```rust
let mut opts = Options::default();

// Write performance
opts.set_write_buffer_size(64 * 1024 * 1024);  // 64MB
opts.set_max_write_buffer_number(3);
opts.set_min_write_buffer_number_to_merge(2);

// Read performance
opts.set_block_based_table_factory(&BlockBasedOptions::default());
opts.set_bloom_filter(10.0, false);  // 10 bits per key

// Compression
opts.set_compression_type(DBCompressionType::Lz4);
opts.set_bottommost_compression_type(DBCompressionType::Zstd);

// Compaction
opts.set_level_compaction_dynamic_level_bytes(true);
opts.set_target_file_size_base(64 * 1024 * 1024);  // 64MB
```

## See Also

- [[fjall]] — Default canonical backend
- [[lsm-tree]] — Underlying data structure
- [[wal]] — Durability
- [[embedded]] — Philosophy that RocksDB partially contradicts

---

*RocksDB is an excellent engine, but its integration friction makes it suboptimal for VantaDB as default.*

