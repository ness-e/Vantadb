---
type: glossary-entry
status: stable
tags: [glosario, persistencia, storage, durabilidad]
aliases: [persistence, storage, almacenamiento]
---

# Persistencia

## Definición

La **persistencia** es la capacidad de un sistema de almacenamiento de datos para mantener la información de forma duradera más allá del ciclo de vida del proceso que la creó. En bases de datos, esto implica garantizar que los datos sobrevivan a reinicios, crashes y fallos de energía.

## En VantaDB

VantaDB implementa persistencia mediante múltiples capas:

### 1. Write-Ahead Log ([WAL](WAL.md))

Registro secuencial de todas las mutaciones antes de aplicarlas al almacenamiento principal.

```rust
// src/wal.rs
pub struct WalWriter {
    writer: BufWriter<File>,
    sync_mode: SyncMode,
}

pub fn append(&mut self, record: &WalRecord) -> Result<()> {
    let payload = bincode::serialize(record)?;
    let crc = crc32c(&payload);  // Checksum [CRC32C](CRC32C.md)
    
    self.writer.write_all(&payload)?;
    self.writer.write_all(&crc.to_le_bytes())?;
    
    if self.sync_mode == SyncMode::Always {
        self.sync()?;  // [fsync](fsync.md) inmediato
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
1. Escribir registro en WAL
2. fsync del WAL ← DURABILIDAD GARANTIZADA
3. Aplicar mutación a storage backend
4. ACK al cliente
```

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
