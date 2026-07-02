---
type: glossary-entry
status: stable
tags: [persistence, wal, durabilidad, recovery]
last_refined: 2026-06
links: "[[README.md]]"
aliases: [Write-Ahead Log, Journal, Transaction Log]
description: "Journaling mechanism where mutations are first written to a sequential log before being applied to the main storage, guaranteeing ACID durability"
---
#WAL—Write-Ahead Log

## Definition

The **Write-Ahead Log (WAL)** is a sequential, append-only record of all data mutations, where **each change is written to the log BEFORE being applied to the main database**. This guarantees durability and allows recovery after crashes.

## Fundamental Principle

> **WAL Golden Rule:** No mutation is committed to the client until its record is physically written to disk (fsync).

```
Orden correcto:
1. Append al WAL
2. fsync() del WAL
3. Aplicar cambio al storage
4. ACK al cliente

INCORRECT order (data loss):
1. Apply change to storage
2. ACK to the client
3. Append to WAL (asynchronous)
```

## Estructura de un Registro WAL

```
┌─────────────────────────────────────┐
│ Header (8 bytes)                    │
│ ├── Length: u32                     │
│ ├── Type: u8 (Insert/Delete/Update) │
│ └── Flags: u8                       │
├─────────────────────────────────────┤
│ Payload (variable)                  │
│ ├── Key: [u8]                       │
│ ├── Vector: [f32] (si aplica)       │
│ ├── Text: [u8]                      │
│ └── Metadata: [u8]                  │
├─────────────────────────────────────┤
│ Checksum: u32 (CRC32C)              │
└─────────────────────────────────────┘
```

## Implementation in VantaDB

### Writing Flow

```
Cliente: put("doc1", vector, text, metadata)
    │
    ▼
┌────────────────────────┐
│ Serializar mutación    │
└──────────┬─────────────┘
           │
           ▼
┌────────────────────────┐
│ Calcular CRC32C        │
└──────────┬─────────────┘
           │
           ▼
┌────────────────────────┐
│ Append a wal.log       │
└──────────┬─────────────┘
           │
           ▼
┌────────────────────────┐
│ fsync() ← DURABLE      │
└──────────┬─────────────┘
           │
           ▼
┌────────────────────────┐
│ Aplicar a Fjall/HNSW   │
└──────────┬─────────────┘
           │
           ▼
      ACK al cliente
```

### Recovery Flow

```
Arranque tras crash
    │
    ▼
┌────────────────────────┐
│ Leer wal.log           │
└──────────┬─────────────┘
           │
           ▼
┌────────────────────────┐
│ Para cada registro:    │
│  - Verificar CRC32C    │
│  - Si válido: aplicar  │
│  - Si inválido: truncar│
└──────────┬─────────────┘
           │
           ▼
┌────────────────────────┐
│ Reconstruir índices    │
│ (HNSW, BM25) desde     │
│ estado canónico        │
└──────────┬─────────────┘
           │
           ▼
   Base de datos lista
```

## Checkpointing

El WAL crece indefinidamente si no se gestiona. **Checkpointing** es el proceso de:

1. **Flush** of all pending data to main storage
2. **Truncate** the WAL (remove already applied records)
3. **Mark** the new starting point

```
WAL antes de checkpoint:
[rec1][rec2][rec3][rec4][rec5][rec6][rec7][rec8]
                        ↑
                   checkpoint aquí

WAL después de checkpoint:
[rec5][rec6][rec7][rec8]  ← Solo registros nuevos
```

## Durability Modes

| Modo | fsync | Latencia | Riesgo de Pérdida |
|------|-------|----------|-------------------|
| **Always** | Cada write | ~5-10ms | Cero |
| **Periodic** | Cada N ms | <1ms | Últimos N ms |
| **Never** | OS decide | ~0.1ms | Alta |

## Garantías del WAL en VantaDB

### What WAL Guarantees

✅ **Durability:** Once confirmed, the data survives crashes
✅ **Atomicity:** Complete transactions or none
✅ **Deterministic recovery:** Replay produces the same state

### Lo que WAL NO Garantiza

❌ **Consistency between indexes:** That depends on the coherence protocol
❌ **Concurrency isolation:** That depends on locks/MVCC
❌ **Performance:** Synchronous fsync adds latency

## Known Issues (Audit)

### AUD-01: Configurable fsync missing

**Severity:** 🔒 Blocking

El snapshot no demuestra que VantaDB implemente fsync síncrono antes del ACK. Sin esto, los claims de durabilidad no son verificables.

**Mitigation required:**
``rust
pub enum SyncMode {
    Always, // fsync on each write
    Periodic(Duration), // fsync every N ms
    Never, // OS decides
}

impl VantaEmbedded {
    pub fn put(&self, ...) -> Result<()> {
        self.wal.append(&mutation)?;
        
        match self.sync_mode {
            SyncMode::Always => self.wal.fsync()?,
            SyncMode::Periodic(_) => {/* batch */},
            SyncMode::Never => {/* not fsync */},
        }
        
        self.apply_to_storage(&mutation)?;
        Ok(()) // ACK only after fsync
    }
}
```

### AUD-02: Recovery without Checksums

**Severity:** 🔒 Blocking

Sin [[crc32c]] en cada registro, el recovery no puede distinguir entre:
- Registro válido
- Registro corrupto (debe ignorarse)
- Registro parcialmente escrito (debe truncarse)

**Mitigation:** Add checksum to each record and validate in replay.

## Comparison with Other Systems

| Sistema | WAL | fsync Default | Checksum | Recovery |
|---------|-----|---------------|----------|----------|
| **VantaDB** | ✅ | ⚠️ No verificado | ⚠️ CRC32C (no verificado) | ✅ Replay + rebuild |
| **SQLite** | ✅ | Siempre | ✅ CRC32 | ✅ Automático |
| **PostgreSQL** | ✅ | Siempre | ✅ CRC32 | ✅ Automático |
| **RocksDB** | ✅ | Configurable | ✅ CRC32 | ✅ Automático |
| **FAISS** | ❌ | N/A | N/A | ❌ Sin persistencia |

## See Also

- [[fsync]] — Garantía de persistencia física
- [[crc32c]] — Integridad de registros
- [[fjall]] — Backend con WAL propio
- [[transactional]] — Propiedad que el WAL habilita
- [[chaos-testing]] — Cómo validar durabilidad

### Related Implementation Documentation
- [[../architecture/wal_durability|WAL Durability Architecture]]

---

*The WAL is the durability contract of a database. Without it, there are no guarantees.*

