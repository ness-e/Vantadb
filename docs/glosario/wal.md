---
type: glosario-entry
status: stable
tags: [persistencia, wal, durabilidad, recovery]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
aliases: [Write-Ahead Log, Journal, Transaction Log]
description: "Mecanismo de journaling donde las mutaciones se escriben primero en un log secuencial antes de aplicarse al storage principal, garantizando durabilidad ACID"
---

# WAL — Write-Ahead Log

## Definición

El **Write-Ahead Log (WAL)** es un registro secuencial y append-only de todas las mutaciones de datos, donde **cada cambio se escribe en el log ANTES de aplicarse a la base de datos principal**. Esto garantiza durabilidad y permite recuperación tras crashes.

## Principio Fundamental

> **Regla de Oro del WAL:** Ninguna mutación se confirma al cliente hasta que su registro esté físicamente escrito en disco (fsync).

```
Orden correcto:
1. Append al WAL
2. fsync() del WAL
3. Aplicar cambio al storage
4. ACK al cliente

Orden INCORRECTO (pérdida de datos):
1. Aplicar cambio al storage
2. ACK al cliente
3. Append al WAL (asíncrono)
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

## Implementación en VantaDB

### Flujo de Escritura

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

### Flujo de Recovery

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

1. **Flush** de todos los datos pendientes al storage principal
2. **Truncar** el WAL (eliminar registros ya aplicados)
3. **Marcar** el nuevo punto de inicio

```
WAL antes de checkpoint:
[rec1][rec2][rec3][rec4][rec5][rec6][rec7][rec8]
                        ↑
                   checkpoint aquí

WAL después de checkpoint:
[rec5][rec6][rec7][rec8]  ← Solo registros nuevos
```

## Modos de Durabilidad

| Modo | fsync | Latencia | Riesgo de Pérdida |
|------|-------|----------|-------------------|
| **Always** | Cada write | ~5-10ms | Cero |
| **Periodic** | Cada N ms | <1ms | Últimos N ms |
| **Never** | OS decide | ~0.1ms | Alta |

## Garantías del WAL en VantaDB

### Lo que WAL Garantiza

✅ **Durabilidad:** Una vez confirmado, el dato sobrevive a crashes
✅ **Atomicidad:** Transacciones completas o ninguna
✅ **Recuperación determinista:** Replay produce el mismo estado

### Lo que WAL NO Garantiza

❌ **Consistencia entre índices:** Eso depende del protocolo de coherencia
❌ **Aislamiento de concurrencia:** Eso depende de locks/MVCC
❌ **Performance:** fsync síncrono añade latencia

## Problemas Conocidos (Auditoría)

### AUD-01: Falta de fsync Configurable

**Severidad:** 🔒 Bloqueante

El snapshot no demuestra que VantaDB implemente fsync síncrono antes del ACK. Sin esto, los claims de durabilidad no son verificables.

**Mitigación requerida:**
```rust
pub enum SyncMode {
    Always,    // fsync en cada write
    Periodic(Duration), // fsync cada N ms
    Never,     // OS decide
}

impl VantaEmbedded {
    pub fn put(&self, ...) -> Result<()> {
        self.wal.append(&mutation)?;
        
        match self.sync_mode {
            SyncMode::Always => self.wal.fsync()?,
            SyncMode::Periodic(_) => {/* batch */},
            SyncMode::Never => {/* no fsync */},
        }
        
        self.apply_to_storage(&mutation)?;
        Ok(()) // ACK solo después de fsync
    }
}
```

### AUD-02: Recovery sin Checksums

**Severidad:** 🔒 Bloqueante

Sin [CRC32C](CRC32C.md) en cada registro, el recovery no puede distinguir entre:
- Registro válido
- Registro corrupto (debe ignorarse)
- Registro parcialmente escrito (debe truncarse)

**Mitigación:** Añadir checksum a cada registro y validar en replay.

## Comparación con Otros Sistemas

| Sistema | WAL | fsync Default | Checksum | Recovery |
|---------|-----|---------------|----------|----------|
| **VantaDB** | ✅ | ⚠️ No verificado | ⚠️ CRC32C (no verificado) | ✅ Replay + rebuild |
| **SQLite** | ✅ | Siempre | ✅ CRC32 | ✅ Automático |
| **PostgreSQL** | ✅ | Siempre | ✅ CRC32 | ✅ Automático |
| **RocksDB** | ✅ | Configurable | ✅ CRC32 | ✅ Automático |
| **FAISS** | ❌ | N/A | N/A | ❌ Sin persistencia |

## Véase También

- [fsync](fsync.md) — Garantía de persistencia física
- [CRC32C](CRC32C.md) — Integridad de registros
- [Fjall](Fjall.md) — Backend con WAL propio
- [Transaccional](Transaccional.md) — Propiedad que el WAL habilita
- [Chaos Testing](Chaos Testing.md) — Cómo validar durabilidad

---

*El WAL es el contrato de durabilidad de una base de datos. Sin él, no hay garantías.*

