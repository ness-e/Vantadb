---
title: file-locking
kind: glossary
status: stable
description: "Mitigation: Implement advisory lock in open()"
aliases: [File Lock, Advisory Lock]
tags: [concurrencia, lock, sincronizacion]
links: "[[README.md]]"
---

# FileLocking

## Definition

**File Locking** is an operating system mechanism to **prevent multiple processes from simultaneously accessing** the same file, avoiding data corruption due to concurrent writes.

## Why it Matters in VantaDB

If two processes open the same VantaDB database simultaneously:

```
Proceso A: db.put("key1", value1)
Proceso B: db.put("key1", value2)

Without file locking:
- Both write to the WAL
- Records are interspersed
- ❌ Data corruption

With file locking:
- Process A acquires the exclusive `.vanta.lock`
- Process B retries for `file_lock_timeout_ms`, then gets `Error::DatabaseBusy`
- ✅ Secure data
```

## Implementación

VantaDB **sí** implementa el lock. Al abrir el motor, el engine crea
`<storage_path>/.vanta.lock` y toma un lock `fs2` sobre él
(`src/storage/engine/init.rs:167-251`):

| Modo de apertura | Lock tomado |
|------------------|-------------|
| Lectura-escritura (default) | `fs2::FileExt::try_lock_exclusive` — exclusivo |
| Solo lectura (`read_only = true`) | `fs2::FileExt::try_lock_shared` — compartido |

Si el lock no se consigue dentro de `file_lock_timeout_ms`, el engine reintenta
con backoff exponencial (5 ms → tope 100 ms) y luego devuelve
`Error::DatabaseBusy`. El lock se libera solo al soltar el file handle.

```rust
// src/storage/engine/init.rs (extracto)
let lock_path = base_path.join(".vanta.lock");
let file = OpenOptions::new()
    .read(true)
    .write(!config.read_only)
    .create(!config.read_only)
    .open(&lock_path)?;

let mut delay = std::time::Duration::from_millis(5);
let total_limit = std::time::Duration::from_millis(config.file_lock_timeout_ms);
let start_time = Instant::now();
let mut acquired = false;

while start_time.elapsed() < total_limit {
    let lock_res = if config.read_only {
        fs2::FileExt::try_lock_shared(&file)
    } else {
        fs2::FileExt::try_lock_exclusive(&file)
    };
    if lock_res.is_ok() { acquired = true; break; }
    std::thread::sleep(delay);
    delay = std::cmp::min(delay * 2, std::time::Duration::from_millis(100));
}

if !acquired {
    return Err(Error::DatabaseBusy(format!(
        "Database at '{}' is locked by another process.", base_path.display()
    )));
}
```

**Caveat:** la toma del lock está feature-gated tras `#[cfg(feature = "fs2")]`.
Sin esa feature el código compila pero `try_lock_*` se sustituye por `Ok(())` —
el engine arranca **sin** exclusión entre procesos. Con `fs2` activa (el default)
la garantía se cumple.

## Known Issues

### AUD-04: ~~Lack of File Locking~~ — resuelto

**Estado:** ✅ Cerrado. VantaDB implementa file locking (ver la sección anterior).
El `file_lock_timeout_ms` configurable sustituye el "error inmediato" del diseño
original: un segundo proceso que no consigue el lock recibe `Error::DatabaseBusy`
en vez de corromper datos.

## See Also

- [transactional](./transactional.md) — File locking is a requirement for multi-threading
- [wal](./wal.md) — Shared WAL without lock = corruption

---

*File locking prevents corruption when multiple processes try to access the same database.*
