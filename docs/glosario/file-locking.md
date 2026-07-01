---
type: glossary-entry
status: stable
tags: [concurrencia, lock, sincronizacion]
last_refined: 2026-06
links: "[[README.md]]"
aliases: [File Lock, Advisory Lock]
---
#FileLocking

##Definition

**File Locking** is an operating system mechanism to **prevent multiple processes from simultaneously accessing** the same file, avoiding data corruption due to concurrent writes.

##Why it Matters in VantaDB

If two processes open the same VantaDB database simultaneously:

```
Proceso A: db.put("key1", value1)
Proceso B: db.put("key1", value2)

Without file locking:
- Both write to the WAL
- Scriptures are interspersed
- ❌ Data corruption

With file locking:
- Process A acquires lock
- Process B receives error: "Database already open"
- ✅ Secure data
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

// Lock is automatically released when dropping
```

## Known Issues

### AUD-04: Lack of File Locking

**Severity:** ⚠️ High

**Description:** VantaDB does not implement file locking, allowing multiple processes to open the same DB.

**Impact:** Guaranteed data corruption if two processes write simultaneously.

**Mitigation:** Implement advisory lock in `open()`.

## See Also

- [[transactional]] — File locking is a requirement for multi-threading
- [[wal]] — Shared WAL without lock = corruption

---

*File locking prevents corruption when multiple processes try to access the same database.*

