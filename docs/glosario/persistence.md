---
title: "Persistencia"
type: glossary-entry
status: stable
tags: [glosario, persistencia, storage, durabilidad]
last_reviewed: 2026-07-07
aliases: [persistence, storage, almacenamiento]
---

# Persistencia

## Definición

La **persistencia** es la capacidad de un sistema de almacenamiento de datos para mantener la información de forma duradera más allá del ciclo de vida del proceso que la creó. En bases de datos, esto implica garantizar que los datos sobrevivan a reinicios, crashes y fallos de energía.

## En VantaDB

VantaDB implementa persistencia mediante múltiples capas:

### 1. Write-Ahead Log ([WAL](WAL.md))

Registro secuencial de todas las mutaciones, distribuido en **N shards round-robin** para reducir contención.

```rust
// src/wal_sharded.rs
pub struct ShardedWal {
    shards: Vec<Mutex<WalShard>>,
    counter: AtomicU64,
    num_shards: usize,
}

pub fn append(&self, record: &WalRecord) -> Result<()> {
    let idx = self.counter.fetch_add(1, Ordering::Relaxed) as usize % self.num_shards;
    let mut shard = self.shards[idx].lock();
    let payload = bincode::serialize(record)?;
    let crc = crc32c(&payload);  // Checksum [CRC32C](CRC32C.md)
    shard.writer.write_all(&payload)?;
    shard.writer.write_all(&crc.to_le_bytes())?;
    if shard.sync_mode == SyncMode::Always {
        shard.sync()?;  // [fsync](fsync.md) inmediato
    }
    Ok(())
}
```

### 2. Storage Backend

Motor de almacenamiento key-value que mantiene el estado canónico:

| Backend | Características | Uso |
|---------|-----------------|-----|
| **[Fjall](Fjall.md)** | 100% Rust, LSM-tree, transacciones | Default |
| **[RocksDB](RocksDB.md)** | C++, battle-tested, column families | Fallback/benchmarking |

### 3. Memory-Mapped Files ([mmap](mmap.md))

Para índices grandes que exceden RAM disponible:

```rust
// src/storage.rs - VantaFile
pub struct VantaFile {
    mmap: MmapMut,  // memmap2
    path: PathBuf,
}
```

## Protocolo de Durabilidad

El orden de operaciones es crítico para garantizar durabilidad:

```
1. ShardedWal.append(record)   → vanta.shard{idx}.wal (round-robin)
2. fsync del shard             ← DURABILIDAD GARANTIZADA
3. Aplicar mutación a storage backend
4. ACK al cliente
```

En recovery, los registros de todos los shards se leen, se ordenan por `global_seq = shard_idx + N * local_pos`, y se replican en orden secuencial global.

**Regla:** Nunca ACK antes de fsync.

## Tipos de Persistencia

| Modo | Descripción | Performance | Durabilidad |
|------|-------------|-------------|-------------|
| `SyncMode::Always` | fsync en cada write | Baja | Máxima |
| `SyncMode::Periodic` | fsync cada N segundos | Media | Alta |
| `SyncMode::Never` | Sin fsync explícito | Alta | Mínima |

## Véase También

- [WAL](WAL.md) - Write-Ahead Log
- [Fjall](Fjall.md) - Backend de almacenamiento
- [RocksDB](RocksDB.md) - Backend alternativo
- [fsync](fsync.md) - Sincronización a disco
- [CRC32C](CRC32C.md) - Checksum de integridad
- [mmap](mmap.md) - Memory-mapped I/O
