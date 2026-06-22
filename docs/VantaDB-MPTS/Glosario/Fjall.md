---
type: glosario-entry
status: stable
tags: [storage, backend, lsm-tree, rust]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
aliases: [Fjall Storage Engine]
description: "Motor de almacenamiento LSM-tree escrito en 100% Rust seguro, diseñado para ser embebible, transaccional y de alto rendimiento. Backend canónico por defecto de VantaDB"
---

# Fjall

## Definición

**Fjall** es un motor de almacenamiento **[LSM-Tree](LSM-Tree.md)** (Log-Structured Merge-Tree) escrito en **100% Rust seguro**, diseñado para ser embebible, transaccional y de alto rendimiento. Es el **backend canónico por defecto** de VantaDB desde la versión 0.1.4.

## Características Clave

| Característica | Descripción |
|---------------|-------------|
| **100% Safe Rust** | Sin `unsafe`, sin dependencias C++ |
| **LSM-Tree** | Estructura optimizada para escrituras |
| **MVCC** | [MVCC](MVCC.md) nativo para transacciones concurrentes |
| **Keyspaces** | Aislamiento lógico (similar a column families) |
| **Transacciones ACID** | Soporte completo para atomicidad y durabilidad |
| **Compresión** | LZ4 por defecto, configurable |
| **Bloom Filters** | Para reducir lecturas de disco |

## Por Qué VantaDB Elige Fjall

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

### Decisión Estratégica

**Fjall como default** porque:
1. ✅ Reduce fricción de compilación (no C++)
2. ✅ Elimina dependencias de sistema
3. ✅ Más seguro (safe Rust)
4. ✅ Alineado con identidad Rust-native de VantaDB

**[RocksDB](RocksDB.md) como fallback** para:
- Benchmarking comparativo
- Usuarios que requieren features específicas de RocksDB
- Migración desde sistemas existentes

## Arquitectura de Fjall

### Componentes Principales

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

### Flujo de Escritura

1. **Append al WAL** (durabilidad)
2. **Insert en MemTable** (en memoria)
3. **Flush a SSTable** (cuando MemTable alcanza tamaño límite)
4. **Compacción** (merge de SSTables en background)

### Flujo de Lectura

1. **Buscar en MemTable** (más reciente)
2. **Buscar en SSTables** (L0 → L1 → ... → Ln)
3. **Bloom Filter** para evitar lecturas innecesarias
4. **Retornar valor** o `None`

## Uso en VantaDB

### Inicialización

```rust
use fjall::{Config, Keyspace};

let db = Config::new("./vantadb_data")
    .open()
    .expect("Failed to open Fjall database");

let documents_ks = db.open_keyspace("documents")?;
let vectors_ks = db.open_keyspace("vectors")?;
let metadata_ks = db.open_keyspace("metadata")?;
```

### Transacciones

```rust
// Transacción de escritura
let mut tx = db.write_tx();
tx.insert(&documents_ks, b"doc1", b"contenido");
tx.insert(&vectors_ks, b"doc1", &vector_bytes);
tx.commit()?;  // Atómico: todo o nada

// Transacción de lectura (snapshot)
let tx = db.read_tx();
let doc = tx.get(&documents_ks, b"doc1")?;
let vec = tx.get(&vectors_ks, b"doc1")?;
// Ambas lecturas ven el mismo snapshot
```

### Trait StorageBackend

VantaDB abstrae Fjall detrás de un trait genérico:

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
    // ...
}
```

## Ventajas de Fjall para VantaDB

### 1. Compilación Rápida

| Backend | Tiempo de Compilación (clean) |
|---------|------------------------------|
| **Fjall** | ~30 segundos |
| **RocksDB** | ~5-10 minutos |

**Impacto:** Inner loop de desarrollo más ágil.

### 2. Cross-Platform Sin Dolor

```bash
# Linux
cargo build  # Funciona

# macOS
cargo build  # Funciona

# Windows
cargo build  # Funciona (sin Visual Studio, sin CMake)
```

### 3. Seguridad de Memoria

```rust
// Fjall: Safe Rust
let value = db.get(key)?;  // Sin unsafe

// RocksDB bindings: unsafe interno
let value = unsafe { db.get_unsafe(key) }?;  // Posible UB
```

### 4. MVCC Nativo

Transacciones concurrentes sin locks globales:

```rust
// Thread 1: Lee
let tx1 = db.read_tx();
let v1 = tx1.get(key)?;

// Thread 2: Escribe (no bloquea Thread 1)
let mut tx2 = db.write_tx();
tx2.insert(key, new_value);
tx2.commit()?;

// Thread 1: Sigue viendo el snapshot antiguo
let v1_again = tx1.get(key)?;  // v1 == v1_again
```

## Configuración en VantaDB

### Parámetros por Defecto

```rust
Config::new(path)
    .block_cache_size(64 * 1024 * 1024)  // 64 MB
    .write_buffer_size(16 * 1024 * 1024)  // 16 MB
    .max_write_buffer_number(2)
    .compression(CompressionType::Lz4)
```

### Tuning para Diferentes Cargas

| Caso de Uso | block_cache | write_buffer | Compresión |
|-------------|-------------|--------------|------------|
| **Escritura intensiva** | 32 MB | 64 MB | None |
| **Lectura intensiva** | 256 MB | 16 MB | LZ4 |
| **Memoria limitada** | 16 MB | 8 MB | LZ4 |
| **Disco lento** | 128 MB | 32 MB | None |

## Problemas Conocidos

### Migración desde RocksDB

**Estado:** En progreso (FASE 2)

**Desafío:** Usuarios con datos en RocksDB necesitan migrar a Fjall.

**Solución:** Herramienta de migración:

```bash
vanta migrate --from rocksdb --to fjall --data ./vantadb_data
```

### Write Amplification

**Problema:** LSM-trees sufren de write amplification (escribir múltiples veces el mismo dato debido a compactaciones).

**Mitigación en Fjall:**
- Compresión LZ4 reduce I/O
- Bloom filters reducen lecturas
- Compactación tiered (no leveled) para writes intensivos

## Comparación con Otros Backends Rust

| Backend | LSM-Tree | Transacciones | Madurez | Caso de Uso |
|---------|----------|---------------|---------|-------------|
| **Fjall** | ✅ | ✅ ACID | Joven | VantaDB, apps embebidas |
| **Sled** | ✅ | ⚠️ Básico | Estable | Prototipos |
| **Redb** | B-tree | ✅ ACID | Estable | Apps simples |
| **Sanakirja** | B-tree | ✅ ACID | Estable | Pijul VCS |

## Roadmap de Fjall en VantaDB

| Fase | Objetivo | Estado |
|------|----------|--------|
| **FASE 1** | Prototipo de integración | ✅ Completado |
| **FASE 2** | Backend por defecto | 🔄 En progreso |
| **FASE 3** | Herramienta de migración | ⬜ Pendiente |
| **FASE 4** | Optimizaciones avanzadas | ⬜ Pendiente |

## Véase También

- [RocksDB](RocksDB.md) — Backend alternativo
- [LSM-Tree](LSM-Tree.md) — Estructura de datos subyacente
- [MVCC](MVCC.md) — Control de concurrencia
- [WAL](WAL.md) — Durabilidad
- [Transaccional](Transaccional.md) — Garantías ACID

---

*Fjall representa la apuesta de VantaDB por un stack 100% Rust, seguro y embebible.*

