---
title: "fsync — File Synchronization"
type: glossary-entry
status: stable
tags: [persistence, durabilidad, io, syscall]
last_refined: 2026-06
links: "[[README.md]]"
aliases: [File Sync, Disk Synchronization]
description: "Syscall from the operating system that forces the writing of all buffers in memory to the physical disk, ensuring that data survives power outages"
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
db = VantaEmbedded("./data")
result = db.get("doc1")
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
db = VantaEmbedded("./data")
result = db.get("doc1")
# result = {...} ✅ Data recovered
```

## Implementation in VantaDB

### Writing Flow with fsync

```rust
impl VantaEmbedded {
    pub fn put(&self, key: &str, vector: &[f32], text: &str) -> Result<()> {
        // 1. Serializar mutación
        let mutation = Mutation::Put {
            key: key.to_string(),
            vector: vector.to_vec(),
            text: text.to_string(),
        };
        
        // 2. Append al WAL
        self.wal.append(&mutation)?;
        
        // 3. fsync() del WAL ← DURABILIDAD
        self.wal.fsync()?;
        
        // 4. Aplicar a storage
        self.storage.apply(&mutation)?;
        
        // 5. ACK al cliente (solo después de fsync)
        Ok(())
    }
}
```
*Note: Write operations are logged to the [[wal|Write-Ahead Log (WAL)]] prior to fsync.*

### fsync implementation

```rust
use std::fs::File;
use std::os::unix::io::AsRawFd;

impl WalWriter {
    pub fn fsync(&self) -> Result<()> {
        #[cfg(unix)]
        unsafe {
            let ret = libc::fsync(self.file.as_raw_fd());
            if ret != 0 {
                return Err(Error::Io(std::io::Error::last_os_error()));
            }
        }
        
        #[cfg(windows)]
        {
            self.file.sync_all()?;
        }
        
        Ok(())
    }
}
```

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

### 1. SyncAlways (Maximum Durability)

```rust
pub enum SyncMode {
    Always,  // fsync en cada write
}

// Use: Financial, medical, legal systems
// Latency: High (~1-5 ms per write)
// Data loss: Zero
```

### 2. SyncPeriodic (Balance)

```rust
pub enum SyncMode {
    Periodic(Duration),  // fsync cada N ms
}

// Use: General applications
// Latency: Low (<1 ms)
// Data loss: Last N ms (ex: 100 ms)
```

### 3. SyncNever (Maximum Performance)

```rust
pub enum SyncMode {
    Never,  // OS decide cuándo hacer fsync
}

// Use: Caches, temporary data, logs
// Latency: Minimal (~0.1 ms)
// Data loss: Potentially high
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

### AUD-01: fsync Not Verified

**Severity:** 🔒 Blocking

**Description:** The VantaDB snapshot does not demonstrate that fsync() is executed before the ACK to the client.

**Impact:** Unverifiable durability claims. Possible data loss in crashes.

**Mitigation Required:**
``rust
pub fn put(&self, mutation: &Mutation) -> Result<()> {
    self.wal.append(mutation)?;
    
    // CRITICAL: fsync before ACK
    self.wal.fsync()?;
    
    // Only now confirm
    Ok(())
}
```

**Validation Test:**
``rust
#[test]
fn test_fsync_before_ack() {
    let db = VantaEmbedded::open("./test_data")?;
    
    // Insert data
    db.put("key1", &vec![1.0, 2.0], "test")?;
    
    // Simulate immediate crash
    std::process::exit(1);
    
    // In another process:
    let db = VantaEmbedded::open("./test_data")?;
    assert!(db.get("key1")?.is_some());  // Must exist
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
| **VantaDB** | ⚠️ No verificado | ⬜ Pendiente |
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

**VantaDB should implement something similar:**
```python
db = VantaEmbedded("./data", sync_mode="always") # Maximum durability
db = VantaEmbedded("./data", sync_mode="periodic") # Balance
db = VantaEmbedded("./data", sync_mode="never") # Maximum performance
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

- [[wal]] — System that uses fsync for durability
- [[transactional]] — Property that fsync guarantees
- [[crc32c]] — Integrity complementary to durability
- [[chaos-testing]] — How to validate durability

---

*fsync is the line between "saved data" and "actually persistent data".*

