---
title: "rwlock"
type: glossary-entry
status: stable
tags: [concurrencia, lock, sincronizacion, rust]
last_refined: 2026-06
links: "[[README.md]]"
aliases: [Read-Write Lock]
---
#RwLock—Read-Write Lock

##Definition

A **RwLock** (Read-Write Lock) is a synchronization primitive that allows **multiple simultaneous readers** or **a single dedicated writer**, optimizing for workloads with more reads than writes.

## Cómo Funciona

```
RwLock<T>
├── Múltiples lectores pueden acceder simultáneamente
│   lock.read() → ReadGuard<T>
│
└── Un solo escritor puede acceder (exclusivo)
    lock.write() → WriteGuard<T>
```

## Usage in VantaDB

```rust
use std::sync::{Arc, RwLock};

pub struct VantaEmbedded {
    engine: Arc<RwLock<Engine>>,
}

// Reading (multiple threads)
fn get(&self, key: &str) -> Result<Option<Value>> {
    let engine = self.engine.read().unwrap();
    engine.get(key)
}

// Escritura (exclusivo)
fn put(&self, key: &str, value: Value) -> Result<()> {
    let mut engine = self.engine.write().unwrap();
    engine.put(key, value)
}
```

## Advantages vs Mutex

| Dimensión | RwLock | Mutex |
|-----------|--------|-------|
| **Lecturas concurrentes** | ✅ Sí | ❌ No |
| **Escrituras concurrentes** | ❌ No | ❌ No |
| **Overhead** | Mayor | Menor |
| **Caso de uso** | Read-heavy (90%+ lecturas) | Balanced o write-heavy |

## Known Issues

### AUD-03: Rebuild without Global Lock

**Severity:** ⚠️ High

**Description:** `rebuild_index()` does not acquire an exclusive lock, allowing concurrent reads during rebuild.

**Impact:** Readers can see partially reconstructed index.

**Mitigation:**
```rust
fn rebuild_index(&self) -> Result<()> {
    let mut engine = self.engine.write().unwrap();  // Exclusive lock
    engine.rebuild_index()
}
```

## See Also

- [[file-locking]] — Lock at the process level
- [[gil]] — Python global lock (different)
- [[transactional]] — RwLock helps ensure isolation

---

*RwLock is VantaDB's internal concurrency primitive to protect engine state.*

