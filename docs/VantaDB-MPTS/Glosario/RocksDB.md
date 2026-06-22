---
type: glosario-entry
status: stable
tags: [storage, backend, lsm-tree, cpp]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
aliases: [RocksDB Storage Engine]
description: "Motor de almacenamiento LSM-tree de alto rendimiento desarrollado por Facebook en C++. Backend alternativo/fallback en VantaDB para benchmarking y compatibilidad"
---

# RocksDB

## Definición

**RocksDB** es un motor de almacenamiento **[LSM-Tree](LSM-Tree.md)** de alto rendimiento desarrollado por Facebook, escrito en **C++**. Es ampliamente usado en la industria como backend de almacenamiento para bases de datos y sistemas distribuidos. En VantaDB, RocksDB es el **backend alternativo/fallback**, mantenido para benchmarking y compatibilidad.

## Historia y Adopción

| Año | Hito |
|-----|------|
| 2012 | Facebook inicia RocksDB (fork de LevelDB de Google) |
| 2013 | Open-source bajo licencia BSD |
| 2015+ | Adopción masiva: CockroachDB, TiKV, Flink, Kafka |
| 2020+ | Estándar de facto para LSM-trees en producción |
| 2024+ | VantaDB adopta RocksDB, luego migra a [Fjall](Fjall.md) como default |

## Características Clave

| Característica | Descripción |
|---------------|-------------|
| **C++ Nativo** | Performance optimizada, bindings para múltiples lenguajes |
| **LSM-Tree** | Estructura optimizada para escrituras |
| **Column Families** | Aislamiento lógico de datos |
| **Transacciones** | Soporte para optimistic/pessimistic transactions |
| **Compresión** | Múltiples algoritmos: LZ4, Zstd, Snappy |
| **Tuning Avanzado** | Cientos de parámetros configurables |
| **Madurez** | 10+ años en producción a gran escala |

## Por Qué VantaDB Soporta RocksDB

### Razones Históricas

1. **Inicialmente fue el default:** VantaDB comenzó con RocksDB
2. **Madurez probada:** Usado en producción por Facebook, CockroachDB, TiKV
3. **Documentación extensa:** Wiki oficial con cientos de páginas
4. **Features avanzadas:** Backup engine, checkpoints, transactions

### Razones Actuales (Fallback)

1. **Benchmarking comparativo:** Fjall vs RocksDB
2. **Migración incremental:** Usuarios con datos existentes
3. **Features específicas:** Algunos usuarios requieren capabilities únicas de RocksDB

## Arquitectura de RocksDB

### Componentes Principales

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

## Uso en VantaDB

### Habilitar RocksDB

```toml
# Cargo.toml
[dependencies]
vantadb = { version = "0.1.4", features = ["rocksdb"] }
```

### Inicialización

```rust
use vantadb::storage::RocksDbBackend;

let backend = RocksDbBackend::open("./vantadb_data")?;
```

### Configuración Avanzada

```rust
use rocksdb::{Options, DB};

let mut opts = Options::default();
opts.create_if_missing(true);
opts.set_write_buffer_size(64 * 1024 * 1024);  // 64 MB
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

### Seguridad

| Aspecto | Fjall | RocksDB |
|---------|-------|---------|
| **Memory safety** | 100% safe Rust | `unsafe` en bindings |
| **UB potential** | Cero | Posible en FFI |
| **Auditability** | Fácil (Rust) | Difícil (C++ + Rust) |

## Problemas de RocksDB en VantaDB

### 1. Fricción de Compilación

```
Error: failed to run custom build command for `librocksdb-sys v0.11.0`

Caused by:
  process didn't exit successfully: `.../build-script-build`
  
  --- stderr
  CMake Error: CMake was unable to find a build program
  ...
```

**Impacto:** CI/CD lento, problemas en Windows, onboarding difícil.

### 2. Dependencias de Sistema

```bash
# Ubuntu
sudo apt-get install libclang-dev cmake

# macOS
brew install cmake llvm

# Windows
# Instalar Visual Studio 2022 con C++ workload
# Instalar CMake
# Configurar variables de entorno
```

**Impacto:** No es zero-config, contradice filosofía [Embebido](Embebido.md).

### 3. Binarios Inflados

| Backend | Tamaño del Binario |
|---------|-------------------|
| **Fjall** | ~15 MB |
| **RocksDB** | ~45 MB |

**Impacto:** 3x más grande, afecta distribución.

### 4. Cross-Platform Issues

**Windows:**
- Requiere `rust-lld` como linker
- Problemas con `dbghelp.lib`
- Incompatibilidades con MSVC

**macOS ARM64:**
- Problemas con `mincore` (firma diferente)
- Requiere conditional compilation

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

### Herramienta de Migración

```bash
# Migrar datos de RocksDB a Fjall
vanta migrate \
  --from rocksdb \
  --to fjall \
  --source ./vantadb_rocksdb \
  --target ./vantadb_fjall \
  --verify
```

## Cuándo Usar RocksDB (Aún)

### Casos Válidos

1. **Migración pendiente:** Todavía no has migrado a Fjall
2. **Benchmarking:** Comparar performance vs Fjall
3. **Features específicas:** Necesitas algo que solo RocksDB tiene
4. **Compatibilidad:** Integración con sistemas que usan RocksDB

### Casos Inválidos

1. **"RocksDB es más maduro":** Fjall es suficientemente estable para producción
2. **"Mejor performance":** Diferencia marginal (<20%), no justifica fricción
3. **"Siempre lo usamos":** Deuda técnica, no razón técnica

## Optimizaciones de RocksDB en VantaDB

### Configuración para VantaDB

```rust
let mut opts = Options::default();

// Write performance
opts.set_write_buffer_size(64 * 1024 * 1024);  // 64 MB
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
opts.set_target_file_size_base(64 * 1024 * 1024);  // 64 MB
```

## Véase También

- [Fjall](Fjall.md) — Backend canónico por defecto
- [LSM-Tree](LSM-Tree.md) — Estructura de datos subyacente
- [WAL](WAL.md) — Durabilidad
- [Embebido](Embebido.md) — Filosofía que RocksDB contradice parcialmente

---

*RocksDB es un excelente motor, pero su fricción de integración lo hace subóptimo para VantaDB como default.*

