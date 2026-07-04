---
title: "Single-thread: 1 second"
type: glossary-entry
status: stable
tags: [python, concurrencia, lock, threading]
last_refined: 2026-06
links: "[[README.md]]"
aliases: [Global Interpreter Lock, Python GIL]
description: "Global mutex in the CPython interpreter that protects access to Python objects, ensuring that only one thread can execute Python bytecode at a time within a process"
---
#GIL—Global Interpreter Lock

##Definition

The **GIL** (Global Interpreter Lock) is a **global mutex** in the CPython interpreter that protects access to Python objects, ensuring that **only one thread can execute Python bytecode at a time** within a process.

## Why does the GIL exist?

### Historical Reasons

1. **Memory management:** CPython reference counting is not thread-safe
2. **Simplicity:** Avoid granular locks on each Python object
3. **Single-thread performance:** No locking overhead in sequential code
4. **C Integration:** C extensions do not need to worry about concurrency

### Consequence

```python
import threading

def cpu_bound_task():
    total = 0
    for i in range(10_000_000):
        total += i
    return total

# Single-thread: 1 second
cpu_bound_task()

# Multi-thread (4 threads): Also ~1 second!
# The GIL prevents real parallelism
threads = [threading.Thread(target=cpu_bound_task) for _ in range(4)]
for t in threads:
    t.start()
```

**Result:** Python threads do not achieve parallelism in CPU-bound tasks.

## How the GIL Affects VantaDB

### Problematic Scenario

```python
from vantadb import VantaEmbedded
import threading

db = VantaEmbedded("./data")

def search_task(query_vector):
    # If PyO3 does NOT release the GIL:
    # This thread blocks all other Python threads
    return db.search(vector=query_vector, top_k=10)

# 4 threads buscando simultáneamente
threads = [threading.Thread(target=search_task, args=(v,)) for v in vectors]

# Without freeing GIL: sequential lookups (4x slower)
# With release GIL: parallel searches (4x faster)
```

### Solution: Release the GIL in PyO3

```rust
#[pymethods]
impl VantaEmbedded {
    fn search(&self, py: Python<'_>, vector: Vec<f32>, top_k: usize) -> PyResult<Vec<SearchResult>> {
        // Liberar GIL antes de operación pesada
        py.allow_threads(|| {
            // Ahora otros threads Python pueden ejecutarse
            // Rust corre en paralelo real
            let engine = self.inner.read().unwrap();
            engine.search(&vector, top_k)
        })
    }
}
```

## When to Release the GIL

### ✅ Release GIL (Long Operations)

| Operación | Duración Típica | Liberar GIL |
|-----------|----------------|-------------|
| busqueda-vectorial (HNSW) | 5-50 ms | ✅ Sí |
| busqueda-lexica (BM25) | 10-100 ms | ✅ Sí |
| Inserción con indexing | 10-50 ms | ✅ Sí |
| Rebuild de índice | 1-60 segundos | ✅ Sí |
| Lectura de disco | 1-100 ms | ✅ Sí |
| Cálculo de distancias | 1-10 ms | ✅ Sí |

### ❌ DO NOT Release GIL (Quick Operations)

| Operación | Duración Típica | Liberar GIL |
|-----------|----------------|-------------|
| Get por key | <1 ms | ❌ No (overhead > beneficio) |
| Metadata lookup | <1 ms | ❌ No |
| Conversión de tipos | <0.1 ms | ❌ No |

## Critical rules for `allow_threads`

### 1. DO NOT Access Python Objects

```rust
// ❌ INCORRECTO: Acceder a PyAny dentro de allow_threads
fn bad_example(&self, py: Python<'_>, py_dict: &PyDict) -> PyResult<()> {
    py.allow_threads(|| {
        // ERROR: py_dict es un objeto Python
        // Accederlo sin GIL = undefined behavior
        let value = py_dict.get_item("key");
        // ...
    })
}

// ✅ CORRECT: Extract data BEFORE releasing GIL
fn good_example(&self, py: Python<'_>, py_dict: &PyDict) -> PyResult<()> {
    // Extract data with GIL
    let key: String = py_dict.get_item("key")
        .unwrap()
        .extract()?;
    
    py.allow_threads(move || {
        // Use only extracted data (Rust types)
        self.engine.process(&key)
    })
}
```

### 2. DO NOT Make Callbacks to Python

```rust
// ❌ INCORRECTO: Callback a Python sin GIL
fn bad_callback(&self, py: Python<'_>, callback: &PyAny) -> PyResult<()> {
    py.allow_threads(|| {
        // ERROR: Llamar a Python sin GIL
        callback.call0()?;
        Ok(())
    })
}

// ✅ CORRECT: Callback with GIL
fn good_callback(&self, py: Python<'_>, callback: &PyAny) -> PyResult<()> {
    // Maintain GIL for callbacks
    callback.call0()?;
    Ok(())
}
```

### 3. Clone Data Before Release

```rust
// ✅ CORRECTO: Clonar antes de liberar
fn search(&self, py: Python<'_>, vector: Vec<f32>) -> PyResult<Vec<SearchResult>> {
    let vector_clone = vector.clone();  // Clonar con GIL
    
    py.allow_threads(move || {
        // Usar vector_clone (no vector original)
        self.engine.search(&vector_clone, 10)
    })
}
```

## Impact on Performance

### Benchmark: Concurrent Search

| Configuración | 1 Thread | 4 Threads | Speedup |
|--------------|----------|-----------|---------|
| **Sin liberar GIL** | 62 ms | 248 ms (secuencial) | 1.0x |
| **Con liberar GIL** | 62 ms | 18 ms (paralelo) | **3.4x** |

### CPU Efficiency

```python
# Sin liberar GIL
CPU usage: 25% (1 core de 4)
Throughput: 16 queries/segundo

# With release GIL
CPU usage: 95% (4 cores of 4)
Throughput: 220 queries/second # 13.75x more throughput
```

## Alternatives to GIL

### 1. Multiprocessing

```python
from multiprocessing import Process

# Cada proceso tiene su propio GIL
processes = [Process(target=search_task, args=(v,)) for v in vectors]

# Advantage: True parallelism
# Disadvantage: IPC overhead, duplicate memory
```

### 2. Async/Await

```python
import asyncio

async def async_search(vector):
    # Does not release GIL, but allows concurrent I/O
    return await loop.run_in_executor(None, db.search, vector)

# Advantage: I/O-bound concurrency
# Disadvantage: Does not help in CPU-bound
```

### 3. Subinterpreters (Python 3.12+)

```python
# Experimental: Cada subinterpreter tiene su propio GIL
# Futuro: Paralelismo real en Python puro
```

### 4. VantaDB (Real Solution)

```python
# PyO3 + allow_threads = paralelismo real en código Rust
# Sin overhead de IPC, sin memoria duplicada
db.search(vector=v, top_k=10)  # Libera GIL internamente
```

## Known Issues in VantaDB

### AUD-01: GIL Not Consistently Released

**Severity:** ⚠️ High

**Description:** Some heavy operations (rebuild_index, export) do not release the GIL.

**Impact:** Multi-threaded applications hang during these operations.

**Mitigación:**
```rust
// Todas las operaciones >10ms deben usar allow_threads
fn rebuild_index(&self, py: Python<'_>) -> PyResult<()> {
    py.allow_threads(|| {
        self.engine.rebuild_index()
    })
}
```

## Debugging GIL Issues

### Symptoms of Unreleased GIL

1. **Low CPU usage** in multi-thread operations
2. **Throughput does not scale** with more threads
3. **High latency** under concurrency
4. **Other Python threads freeze** during long operations

### Diagnostic Tools

```python
import threading
import time

def measure_concurrency():
    def task():
        db.search(vector=v, top_k=10)
    
    start = time.time()
    threads = [threading.Thread(target=task) for _ in range(4)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()
    elapsed = time.time() - start
    
    # If elapsed ≈ 4x single-thread time:
    # GIL is not being released
    print(f"Speedup: {single_thread_time / elapsed}")
```

## Comparison with Other Languages

| Lenguaje | Lock Global | Paralelismo Real |
|----------|-------------|------------------|
| **Python (CPython)** | ✅ GIL | ❌ No (sin workarounds) |
| **Python (PyPy)** | ✅ GIL | ❌ No |
| **Ruby (MRI)** | ✅ GVL | ❌ No |
| **JavaScript (Node)** | ✅ Single-thread | ❌ No (sin workers) |
| **Rust** | ❌ No | ✅ Sí (threads nativos) |
| **Go** | ❌ No | ✅ Sí (goroutines) |
| **Java** | ❌ No | ✅ Sí (threads JVM) |

## See Also

- [[pyo3]] — Framework that manages the GIL
- [[ffi]] — Border where the GIL is released
- [[rwlock]] — Internal concurrency in Rust (without GIL)

---

*The GIL is a limitation of CPython, not VantaDB. PyO3 + allow_threads mitigates this completely.*

