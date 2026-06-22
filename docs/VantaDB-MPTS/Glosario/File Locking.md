---
type: glosario-entry
status: stable
tags: [concurrencia, lock, sincronizacion]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
aliases: [File Lock, Advisory Lock]
---

# File Locking

## Definición

**File Locking** es un mecanismo del sistema operativo para **prevenir que múltiples procesos accedan simultáneamente** al mismo archivo, evitando corrupción de datos por escrituras concurrentes.

## Por Qué Importa en VantaDB

Si dos procesos abren la misma base de datos VantaDB simultáneamente:

```
Proceso A: db.put("key1", value1)
Proceso B: db.put("key1", value2)

Sin file locking:
- Ambos escriben al WAL
- Escrituras se intercalan
- ❌ Corrupción de datos

Con file locking:
- Proceso A adquiere lock
- Proceso B recibe error: "Database already open"
- ✅ Datos seguros
```

## Implementación

```rust
use fs2::FileExt;

pub struct DatabaseLock {
    _lock_file: File,
}

impl DatabaseLock {
    pub fn acquire(path: &Path) -> Result<Self> {
        let lock_path = path.join(".vantadb.lock");
        let file = File::create(&lock_path)?;
        
        file.try_lock_exclusive()
            .map_err(|_| Error::DatabaseAlreadyOpen)?;
        
        Ok(Self { _lock_file: file })
    }
}

// Lock se libera automáticamente al hacer drop
```

## Problemas Conocidos

### AUD-04: Falta de File Locking

**Severidad:** ⚠️ Alta

**Descripción:** VantaDB no implementa file locking, permitiendo que múltiples procesos abran la misma DB.

**Impacto:** Corrupción de datos garantizada si dos procesos escriben simultáneamente.

**Mitigación:** Implementar advisory lock en `open()`.

## Véase También

- [Transaccional](Transaccional.md) — File locking es requisito para multi-proceso
- [WAL](WAL.md) — WAL compartido sin lock = corrupción

---

*File locking previene corrupción cuando múltiples procesos intentan acceder a la misma base de datos.*

