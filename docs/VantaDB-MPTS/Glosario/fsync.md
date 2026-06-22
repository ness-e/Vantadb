---
type: glosario-entry
status: stable
tags: [persistencia, durabilidad, io, syscall]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
aliases: [File Sync, Sincronización a Disco]
description: "Syscall del sistema operativo que fuerza la escritura de todos los buffers en memoria hacia el disco físico, garantizando que los datos sobrevivan a cortes de energía"
---

# fsync — File Synchronization

## Definición

**fsync** es una syscall del sistema operativo que **fuerza la escritura de todos los buffers en memoria hacia el disco físico**, garantizando que los datos estén persistentemente almacenados y sobrevivan a cortes de energía o crashes del sistema.

## El Problema: Buffers de Escritura

### Sin fsync (Pérdida de Datos)

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

### Con fsync (Durabilidad Garantizada)

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

## Por Qué fsync es Crítico

### El Contrato de Durabilidad

> **Regla de Oro:** Una base de datos [Transaccional](Transaccional.md) NO debe confirmar una escritura al cliente hasta que fsync() haya retornado exitosamente.

### Escenario de Pérdida sin fsync

```python
# Cliente
db.put("doc1", vector, text)
# Base de datos retorna "éxito" (sin fsync)

# [CORTE DE ENERGÍA 1 segundo después]

# Reinicio
db = VantaEmbedded("./data")
result = db.get("doc1")
# result = None  ❌ ¡El dato se perdió!
```

### Escenario con fsync

```python
# Cliente
db.put("doc1", vector, text)
# Base de datos hace fsync() antes de retornar
# Retorna "éxito" (datos en disco)

# [CORTE DE ENERGÍA 1 segundo después]

# Reinicio
db = VantaEmbedded("./data")
result = db.get("doc1")
# result = {...}  ✅ Dato recuperado
```

## Implementación en VantaDB

### Flujo de Escritura con fsync

```rust
impl VantaEmbedded {
    pub fn put(&self, key: &str, vector: &[f32], text: &str) -> Result<()> {
        // 1. Serializar mutación
        let mutation = Mutation::Put {
            key: key.to_string(),
            vector: vector.to_vec(),
            text: text.to_string(),
        };
        
        // 2. Append al [WAL](WAL.md)
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

### Implementación de fsync

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

## Costo de fsync

### Latencia por Operación

| Storage | fsync Latency |
|---------|---------------|
| **HDD (7200 RPM)** | 5-15 ms |
| **SATA SSD** | 1-5 ms |
| **NVMe SSD** | 0.1-1 ms |
| **Enterprise NVMe** | 0.05-0.5 ms |

### Impacto en Throughput

| Modo | Writes/segundo (NVMe) |
|------|----------------------|
| **Sin fsync** | ~100,000 |
| **fsync cada write** | ~1,000-10,000 |
| **fsync cada 100 writes** | ~50,000 |

**Trade-off:** Durabilidad vs Performance.

## Modos de Sincronización

### 1. SyncAlways (Máxima Durabilidad)

```rust
pub enum SyncMode {
    Always,  // fsync en cada write
}

// Uso: Sistemas financieros, médicos, legales
// Latencia: Alta (~1-5 ms por write)
// Pérdida de datos: Cero
```

### 2. SyncPeriodic (Balance)

```rust
pub enum SyncMode {
    Periodic(Duration),  // fsync cada N ms
}

// Uso: Aplicaciones generales
// Latencia: Baja (<1 ms)
// Pérdida de datos: Últimos N ms (ej: 100 ms)
```

### 3. SyncNever (Máxima Performance)

```rust
pub enum SyncMode {
    Never,  // OS decide cuándo hacer fsync
}

// Uso: Caches, datos temporales, logs
// Latencia: Mínima (~0.1 ms)
// Pérdida de datos: Potencialmente alta
```

## fdatasync vs fsync

| Syscall | Qué Sincroniza | Performance |
|---------|----------------|-------------|
| **fsync** | Datos + metadata (timestamps, permissions) | Más lento |
| **fdatasync** | Solo datos | Más rápido |

### Cuándo Usar Cada Uno

```rust
// fsync: Cuando metadata importa
// Ej: Sistema de archivos, base de datos con timestamps críticos
self.file.sync_all()?;  // fsync

// fdatasync: Cuando solo datos importan
// Ej: WAL de base de datos (metadata no crítica)
#[cfg(unix)]
unsafe {
    libc::fdatasync(self.file.as_raw_fd());
}
```

## Problemas Conocidos

### AUD-01: fsync No Verificado

**Severidad:** 🔒 Bloqueante

**Descripción:** El snapshot de VantaDB no demuestra que fsync() se ejecute antes del ACK al cliente.

**Impacto:** Claims de durabilidad no verificables. Posible pérdida de datos en crashes.

**Mitigación Requerida:**
```rust
pub fn put(&self, mutation: &Mutation) -> Result<()> {
    self.wal.append(mutation)?;
    
    // CRÍTICO: fsync antes de ACK
    self.wal.fsync()?;
    
    // Solo ahora confirmar
    Ok(())
}
```

**Test de Validación:**
```rust
#[test]
fn test_fsync_before_ack() {
    let db = VantaEmbedded::open("./test_data")?;
    
    // Insertar dato
    db.put("key1", &vec![1.0, 2.0], "test")?;
    
    // Simular crash inmediato
    std::process::exit(1);
    
    // En otro proceso:
    let db = VantaEmbedded::open("./test_data")?;
    assert!(db.get("key1")?.is_some());  // Debe existir
}
```

### Problema: SSDs con Power-Loss Protection

Algunos SSDs enterprise tienen **capacitores** que permiten completar escrituras en vuelo tras corte de energía. En estos discos, fsync() puede retornar antes de que los datos estén físicamente en NAND, pero el capacitor garantiza que se escribirán.

**Implicación:** fsync() no siempre garantiza durabilidad absoluta. Depende del hardware.

**Mitigación:**
- Usar SSDs con PLP (Power-Loss Protection)
- Configurar RAID con BBU (Battery Backup Unit)
- Aceptar riesgo residual en hardware consumer

## Comparación con Otros Sistemas

| Sistema | fsync Default | Configurable |
|---------|---------------|--------------|
| **VantaDB** | ⚠️ No verificado | ⬜ Pendiente |
| **SQLite** | Siempre | Sí (PRAGMA synchronous) |
| **PostgreSQL** | Siempre | Sí (synchronous_commit) |
| **RocksDB** | Configurable | Sí (sync_wal) |
| **Redis** | Nunca (AOP opcional) | Sí (appendfsync) |

### SQLite: Estándar de Oro

```sql
-- SQLite: 3 modos de durabilidad
PRAGMA synchronous = FULL;    -- fsync en cada transacción (default)
PRAGMA synchronous = NORMAL;  -- fsync en checkpoints
PRAGMA synchronous = OFF;     -- Sin fsync (rápido pero riesgoso)
```

**VantaDB debería implementar algo similar:**
```python
db = VantaEmbedded("./data", sync_mode="always")   # Máxima durabilidad
db = VantaEmbedded("./data", sync_mode="periodic") # Balance
db = VantaEmbedded("./data", sync_mode="never")    # Máxima performance
```

## Testing de Durabilidad

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

echo "✅ 1000 crashes simulados, cero corrupción"
```

## Véase También

- [WAL](WAL.md) — Sistema que usa fsync para durabilidad
- [Transaccional](Transaccional.md) — Propiedad que fsync garantiza
- [CRC32C](CRC32C.md) — Integridad complementaria a durabilidad
- [Chaos Testing](Chaos Testing.md) — Cómo validar durabilidad

---

*fsync es la línea entre "datos guardados" y "datos realmente persistentes".*

