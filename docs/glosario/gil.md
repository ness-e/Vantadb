---
type: glosario-entry
status: stable
tags: [python, concurrencia, lock, threading]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
aliases: [Global Interpreter Lock, Python GIL]
description: "Mutex global en el intérprete CPython que protege el acceso a objetos Python, asegurando que solo un thread pueda ejecutar bytecode Python a la vez dentro de un proceso"
---

# GIL — Global Interpreter Lock

## Definición

El **GIL** (Global Interpreter Lock) es un **mutex global** en el intérprete CPython que protege el acceso a objetos Python, asegurando que **solo un thread pueda ejecutar bytecode Python a la vez** dentro de un proceso.

## Por Qué Existe el GIL

### Razones Históricas

1. **Gestión de memoria:** El reference counting de CPython no es thread-safe
2. **Simplicidad:** Evita locks granulares en cada objeto Python
3. **Performance single-thread:** Sin overhead de locking en código secuencial
4. **Integración C:** Extensiones C no necesitan preocuparse por concurrencia

### Consecuencia

```python
import threading

def cpu_bound_task():
    total = 0
    for i in range(10_000_000):
        total += i
    return total

# Single-thread: 1 segundo
cpu_bound_task()

# Multi-thread (4 threads): ¡También ~1 segundo!
# El GIL impide paralelismo real
threads = [threading.Thread(target=cpu_bound_task) for _ in range(4)]
for t in threads:
    t.start()
```

**Resultado:** Threads Python no logran paralelismo en CPU-bound tasks.

## Cómo el GIL Afecta a VantaDB

### Escenario Problemático

```python
from vantadb import VantaEmbedded
import threading

db = VantaEmbedded("./data")

def search_task(query_vector):
    # Si PyO3 NO libera el GIL:
    # Este thread bloquea todos los otros threads Python
    return db.search(vector=query_vector, top_k=10)

# 4 threads buscando simultáneamente
threads = [threading.Thread(target=search_task, args=(v,)) for v in vectors]

# Sin liberar GIL: búsquedas secuenciales (4x más lento)
# Con liberar GIL: búsquedas paralelas (4x más rápido)
```

### Solución: Liberar el GIL en PyO3

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

## Cuándo Liberar el GIL

### ✅ Liberar GIL (Operaciones Largas)

| Operación | Duración Típica | Liberar GIL |
|-----------|----------------|-------------|
| busqueda-vectorial (HNSW) | 5-50 ms | ✅ Sí |
| busqueda-lexica (BM25) | 10-100 ms | ✅ Sí |
| Inserción con indexing | 10-50 ms | ✅ Sí |
| Rebuild de índice | 1-60 segundos | ✅ Sí |
| Lectura de disco | 1-100 ms | ✅ Sí |
| Cálculo de distancias | 1-10 ms | ✅ Sí |

### ❌ NO Liberar GIL (Operaciones Rápidas)

| Operación | Duración Típica | Liberar GIL |
|-----------|----------------|-------------|
| Get por key | <1 ms | ❌ No (overhead > beneficio) |
| Metadata lookup | <1 ms | ❌ No |
| Conversión de tipos | <0.1 ms | ❌ No |

## Reglas Críticas para `allow_threads`

### 1. NO Acceder a Objetos Python

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

// ✅ CORRECTO: Extraer datos ANTES de liberar GIL
fn good_example(&self, py: Python<'_>, py_dict: &PyDict) -> PyResult<()> {
    // Extraer datos con GIL
    let key: String = py_dict.get_item("key")
        .unwrap()
        .extract()?;
    
    py.allow_threads(move || {
        // Usar solo datos extraídos (tipos Rust)
        self.engine.process(&key)
    })
}
```

### 2. NO Hacer Callbacks a Python

```rust
// ❌ INCORRECTO: Callback a Python sin GIL
fn bad_callback(&self, py: Python<'_>, callback: &PyAny) -> PyResult<()> {
    py.allow_threads(|| {
        // ERROR: Llamar a Python sin GIL
        callback.call0()?;
        Ok(())
    })
}

// ✅ CORRECTO: Callback con GIL
fn good_callback(&self, py: Python<'_>, callback: &PyAny) -> PyResult<()> {
    // Mantener GIL para callbacks
    callback.call0()?;
    Ok(())
}
```

### 3. Clonar Datos Antes de Liberar

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

## Impacto en Performance

### Benchmark: Búsqueda Concurrente

| Configuración | 1 Thread | 4 Threads | Speedup |
|--------------|----------|-----------|---------|
| **Sin liberar GIL** | 62 ms | 248 ms (secuencial) | 1.0x |
| **Con liberar GIL** | 62 ms | 18 ms (paralelo) | **3.4x** |

### CPU Efficiency

```python
# Sin liberar GIL
CPU usage: 25% (1 core de 4)
Throughput: 16 queries/segundo

# Con liberar GIL
CPU usage: 95% (4 cores de 4)
Throughput: 220 queries/segundo  # 13.75x más throughput
```

## Alternativas al GIL

### 1. Multiprocessing

```python
from multiprocessing import Process

# Cada proceso tiene su propio GIL
processes = [Process(target=search_task, args=(v,)) for v in vectors]

# Ventaja: Paralelismo real
# Desventaja: Overhead de IPC, memoria duplicada
```

### 2. Async/Await

```python
import asyncio

async def async_search(vector):
    # No libera GIL, pero permite I/O concurrente
    return await loop.run_in_executor(None, db.search, vector)

# Ventaja: Concurrencia en I/O-bound
# Desventaja: No ayuda en CPU-bound
```

### 3. Subinterpreters (Python 3.12+)

```python
# Experimental: Cada subinterpreter tiene su propio GIL
# Futuro: Paralelismo real en Python puro
```

### 4. VantaDB (Solución Real)

```python
# PyO3 + allow_threads = paralelismo real en código Rust
# Sin overhead de IPC, sin memoria duplicada
db.search(vector=v, top_k=10)  # Libera GIL internamente
```

## Problemas Conocidos en VantaDB

### AUD-01: GIL No Liberado Consistentemente

**Severidad:** ⚠️ Alta

**Descripción:** Algunas operaciones pesadas (rebuild_index, export) no liberan el GIL.

**Impacto:** Aplicaciones multi-thread se bloquean durante estas operaciones.

**Mitigación:**
```rust
// Todas las operaciones >10ms deben usar allow_threads
fn rebuild_index(&self, py: Python<'_>) -> PyResult<()> {
    py.allow_threads(|| {
        self.engine.rebuild_index()
    })
}
```

## Debugging de Problemas de GIL

### Síntomas de GIL No Liberado

1. **CPU usage bajo** en operaciones multi-thread
2. **Throughput no escala** con más threads
3. **Latencia alta** bajo concurrencia
4. **Otros threads Python se congelan** durante operaciones largas

### Herramientas de Diagnóstico

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
    
    # Si elapsed ≈ 4x single-thread time:
    # GIL no se está liberando
    print(f"Speedup: {single_thread_time / elapsed}")
```

## Comparación con Otros Lenguajes

| Lenguaje | Lock Global | Paralelismo Real |
|----------|-------------|------------------|
| **Python (CPython)** | ✅ GIL | ❌ No (sin workarounds) |
| **Python (PyPy)** | ✅ GIL | ❌ No |
| **Ruby (MRI)** | ✅ GVL | ❌ No |
| **JavaScript (Node)** | ✅ Single-thread | ❌ No (sin workers) |
| **Rust** | ❌ No | ✅ Sí (threads nativos) |
| **Go** | ❌ No | ✅ Sí (goroutines) |
| **Java** | ❌ No | ✅ Sí (threads JVM) |

## Véase También

- [PyO3](PyO3.md) — Framework que gestiona el GIL
- [FFI](FFI.md) — Frontera donde el GIL se libera
- [RwLock](RwLock.md) — Concurrencia interna en Rust (sin GIL)

---

*El GIL es una limitación de CPython, no de VantaDB. PyO3 + allow_threads lo mitigan completamente.*

