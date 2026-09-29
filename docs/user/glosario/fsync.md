---
title: fsync — File Synchronization
kind: glossary
status: stable
description: "##Definition"
aliases: [File Sync, Disk Synchronization]
tags: [persistence, durabilidad, io, syscall]
links: "[[README.md]]"
---

# fsync — File Synchronization

##Definition

**fsync** is an operating system syscall that **forces the writing of all in-memory buffers to the physical disk**, ensuring that data is persistently stored and survives power outages or system crashes.

## The Problem: Write Buffers

### No fsync (Data Loss)

```
Aplicación: write(fd, data, len)
    │
    ▼
User Buffer (en proceso)
    │
    │ write() retorna "éxito"
    │ (pero datos aún no están en disco)
    ▼
Kernel Page Cache (en RAM del OS)
    │
    │ [CORTE DE ENERGÍA]
    │
    ▼
   ❌ Datos perdidos
```

### With fsync (Guaranteed Durability)

```
Aplicación: write(fd, data, len)
    │
    ▼
User Buffer
    │
    ▼
Kernel Page Cache
    │
    │ fsync(fd)
    │ (bloquea hasta que datos estén en disco)
    ▼
Disco Físico (platter/SSD)
    │
    │ fsync() retorna "éxito"
    │
    ▼
   ✅ Datos persistentes
```

## Why fsync is Critical

### The Durability Contract

> **Golden Rule:** A [Transactional] database (Transactional.md) should NOT commit a write to the client until fsync() has returned successfully.

### Loss Scenario without fsync

```python
# Cliente
db.put("doc1", vector, text)
# Base de datos retorna "éxito" (sin fsync)

# [POWER OUTAGE 1 second later]

# Reboot
import vantadb
db = vantadb.Client("./data")
result = db.memory.get("default", "doc1")
# result = None ❌ The data was lost!
```

### Scenario with fsync

```python
# Cliente
db.put("doc1", vector, text)
# Base de datos hace fsync() antes de retornar
# Retorna "éxito" (datos en disco)

# [POWER OUTAGE 1 second later]

# Reboot
import vantadb
db = vantadb.Client("./data")
result = db.memory.get("default", "doc1")
# result = {...} ✅ Data recovered
```

## Implementation in VantaDB

### Writing Flow with fsync

```rust
// src/sdk/api/memory.rs:335
impl Embedded {
    pub fn put(&self, input: MemoryInput) -> Result<MemoryRecord> {
        // 1. El WAL se appendea con el record serializado (postcard)
        //    y `maybe_sync` decide si sincroniza según SyncMode.
        // 2. Aplicar a storage + índices derivados.
        // 3. ACK al cliente — solo después del sync del WAL.
    }
}
```

> La firma real es `put(&self, input: MemoryInput) -> Result<MemoryRecord>`
> (namespace + key + payload + metadata + vector), no un `put(key, vector, text)`
> posicional. El sync no es una llamada explícita en el camino de escritura:
> ocurre dentro de `WalWriter::maybe_sync`, que el writer invoca en cada
> append.
*Note: Write operations are logged to the [Write-Ahead Log (WAL)](./wal.md) prior to fsync.*

### fsync implementation

```rust
// src/wal.rs:397
impl WalWriter {
    pub fn sync(&mut self) -> Result<()> {
        self.writer.flush()?;
        self.writer.get_ref().sync_data()?;  // fdatasync en POSIX
        self.records_since_sync = 0;
        Ok(())
    }
}
```

> El método real se llama `WalWriter::sync()`, no `fsync()`, y usa
> `File::sync_data()` — es decir **`fdatasync`** en POSIX, no `fsync` completo.
> La diferencia práctica es menor para el WAL (el tamaño del archivo no se
> sincroniza en cada write), pero conviene saberlo al razonar sobre
> rendimiento.

## fsync cost

### Latencia por Operación

| Storage | fsync Latency |
|---------|---------------|
| **HDD (7200 RPM)** | 5-15 ms |
| **SATA SSD** | 1-5 ms |
| **NVMe SSD** | 0.1-1 ms |
| **Enterprise NVMe** | 0.05-0.5 ms |

### Impact on Throughput

| Modo | Writes/segundo (NVMe) |
|------|----------------------|
| **Sin fsync** | ~100,000 |
| **fsync cada write** | ~1,000-10,000 |
| **fsync cada 100 writes** | ~50,000 |

**Trade-off:** Durability vs Performance.

##Sync Modes

VantaDB implementa los tres modos vía `SyncMode` (`src/config.rs:88`). **No son
toggles-fiction: el código los evalúa en `WalWriter::maybe_sync`**
(`src/wal.rs:377`).

```rust
pub enum SyncMode {
    Always,   // fsync en cada write
    Periodic, // fsync cada N registros (default)
    Never,    // sin sync automático; solo el page cache del OS
}
```

| Modo | Comportamiento en `maybe_sync` | Uso | Riesgo |
|------|--------------------------------|-----|--------|
| `Always` | `self.sync()?` en cada append | Financial, medical, legal | Cero pérdida (salvo hardware) |
| `Periodic` *(default)* | `sync()` cuando `records_since_sync >= flush_threshold`; si no hay threshold, usa `DEFAULT_PERIODIC_THRESHOLD = 1` → **fsync en cada write** | General | ≤ `flush_threshold - 1` registros |
| `Never` | No hace nada; el OS decide | Caches, datos temporales | Pérdida de los últimos writes en crash |

> **Default real:** `Periodic` **con threshold 1** equivale a fsync por write.
> El modo por defecto NO es "batching perezoso": solo se relaja si subes
> `flush_threshold` (constructor `with_flush_threshold(n)`, env
> `VANTADB_FLUSH_THRESHOLD`, o `Config { flush_threshold: Some(n) }`). El doc
> canónico es [DURABILITY_GUARANTEES](../operations/DURABILITY_GUARANTEES.md).

Configuración:

```rust
// Rust: sin fsync por batch (persistimos 1000 writes por sync)
let config = Config::default()
    .with_sync_mode(SyncMode::Periodic)
    .with_flush_threshold(1_000);
```

## fdatasync vs fsync

| Syscall | Qué Sincroniza | Performance |
|---------|----------------|-------------|
| **fsync** | Datos + metadata (timestamps, permissions) | Más lento |
| **fdatasync** | Solo datos | Más rápido |

### When to Use Each

```rust
// fsync: Cuando metadata importa
// Ej: Sistema de archivos, base de datos con timestamps críticos
self.file.sync_all()?;  // fsync

// fdatasync: When only data matters
// Ex: Database WAL (non-critical metadata)
#[cfg(unix)]
unsafe {
    libc::fdatasync(self.file.as_raw_fd());
}
```

## Known Issues

### ~~AUD-01: fsync Not Verified~~ — resuelto

**Estado:** ✅ Cerrado. El sync-before-ACK está implementado y es verificable en
el código:

- `WalWriter::maybe_sync` (`src/wal.rs:377`) se invoca en cada append y llama a
  `WalWriter::sync()` (`src/wal.rs:397`), que hace `flush()` + `sync_data()`
  **antes** de que el writer retorne al camino de escritura.
- El modo por defecto `Periodic` usa `DEFAULT_PERIODIC_THRESHOLD = 1`
  (`src/wal.rs:375`), es decir sincroniza en cada write.
- La suite de crash-injection (`tests/storage/crash_injection.rs`) valida que un
  `SIGKILL` no pierde datos comprometidos.

```rust
// src/wal.rs:375-402
const DEFAULT_PERIODIC_THRESHOLD: u64 = 1;

fn maybe_sync(&mut self) -> Result<()> {
    match self.sync_mode {
        SyncMode::Always => self.sync()?,
        SyncMode::Never => {}
        SyncMode::Periodic => {
            let threshold = self.flush_threshold
                .map(|t| t as u64)
                .unwrap_or(Self::DEFAULT_PERIODIC_THRESHOLD);
            if self.records_since_sync >= threshold {
                self.sync()?;
            }
        }
    }
    Ok(())
}

pub fn sync(&mut self) -> Result<()> {
    self.writer.flush()?;
    self.writer.get_ref().sync_data()?;
    self.records_since_sync = 0;
    Ok(())
}
```

### Problem: SSDs with Power-Loss Protection

Some enterprise SSDs have **capacitors** that allow writes to be completed on the fly after a power outage. On these drives, fsync() may return before the data is physically on NAND, but the capacitor guarantees that it will be written.

**Implication:** fsync() does not always guarantee absolute durability. It depends on the hardware.

**Mitigation:**
- Use SSDs with PLP (Power-Loss Protection)
- Configure RAID with BBU (Battery Backup Unit)
- Accept residual risk in consumer hardware

## Comparison with Other Systems

| Sistema | fsync Default | Configurable |
|---------|---------------|--------------|
| **VantaDB** | ✅ Sí (`Periodic`, threshold 1 = sync por write) | ✅ `SyncMode` + `flush_threshold` |
| **SQLite** | Siempre | Sí (PRAGMA synchronous) |
| **PostgreSQL** | Siempre | Sí (synchronous_commit) |
| **RocksDB** | Configurable | Sí (sync_wal) |
| **Redis** | Nunca (AOP opcional) | Sí (appendfsync) |

### SQLite: Gold Standard

```sql
-- SQLite: 3 modos de durabilidad
PRAGMA synchronous = FULL;    -- fsync en cada transacción (default)
PRAGMA synchronous = NORMAL;  -- fsync en checkpoints
PRAGMA synchronous = OFF;     -- Sin fsync (rápido pero riesgoso)
```

**VantaDB ofrece el equivalente**, vía `SyncMode` + `flush_threshold` en la config
del engine Rust (el constructor Python no expone el knob de sync):

```python
import vantadb

# fsync policy is controlled by the Rust engine config, not the constructor
db = vantadb.Client("./data")  # Periodic + threshold 1 → sync por write
```

## Durability Testing

### Chaos Testing: Kill -9

```bash
# Script de testing
for i in {1..1000}; do
    # Iniciar proceso que escribe datos
    python write_test.py &
    PID=$!
    
    # Esperar tiempo aleatorio (10-100 ms)
    sleep 0.0$((RANDOM % 9 + 1))
    
    # Matar proceso abruptamente
    kill -9 $PID
    
    # Reiniciar y verificar integridad
    python verify_integrity.py || exit 1
done

echo "✅ 1000 simulated crashes, zero corruption"
```

## See Also

- [wal](./wal.md) — System that uses fsync for durability
- [transactional](./transactional.md) — Property that fsync guarantees
- [crc32c](./crc32c.md) — Integrity complementary to durability
- [chaos-testing](./chaos-testing.md) — How to validate durability

---

*fsync is the line between "saved data" and "actually persistent data".*

