---
type: glosario-entry
status: stable
tags: [concurrencia, lock, sincronizacion, rust]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
aliases: [Read-Write Lock]
---

# RwLock — Read-Write Lock

## Definición

Un **RwLock** (Read-Write Lock) es una primitiva de sincronización que permite **múltiples lectores simultáneos** o **un solo escritor exclusivo**, optimizando para workloads con más lecturas que escrituras.

## Cómo Funciona

```
RwLock<T>
├── Múltiples lectores pueden acceder simultáneamente
│   lock.read() → ReadGuard<T>
│
└── Un solo escritor puede acceder (exclusivo)
    lock.write() → WriteGuard<T>
```

## Uso en VantaDB

```rust
use std::sync::{Arc, RwLock};

pub struct VantaEmbedded {
    engine: Arc<RwLock<Engine>>,
}

// Lectura (múltiples threads)
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

## Ventajas vs Mutex

| Dimensión | RwLock | Mutex |
|-----------|--------|-------|
| **Lecturas concurrentes** | ✅ Sí | ❌ No |
| **Escrituras concurrentes** | ❌ No | ❌ No |
| **Overhead** | Mayor | Menor |
| **Caso de uso** | Read-heavy (90%+ lecturas) | Balanced o write-heavy |

## Problemas Conocidos

### AUD-03: Rebuild sin Lock Global

**Severidad:** ⚠️ Alta

**Descripción:** `rebuild_index()` no adquiere un lock exclusivo, permitiendo lecturas concurrentes durante la reconstrucción.

**Impacto:** Lectores pueden ver índice parcialmente reconstruido.

**Mitigación:**
```rust
fn rebuild_index(&self) -> Result<()> {
    let mut engine = self.engine.write().unwrap();  // Lock exclusivo
    engine.rebuild_index()
}
```

## Véase También

- [File Locking](File Locking.md) — Lock a nivel de proceso
- [GIL](GIL.md) — Lock global de Python (diferente)
- [Transaccional](Transaccional.md) — RwLock ayuda a garantizar aislamiento

---

*RwLock es la primitiva de concurrencia interna de VantaDB para proteger el estado del engine.*

