---
title: "PyO3"
type: glossary-entry
status: stable
tags: [ffi, python, rust, bindings]
last_refined: 2026-06
links: "[[README.md]]"
aliases: [PyO3 Bindings, Rust-Python Bindings]
description: "Rust framework to create Python extensions and bidirectional bindings between Rust and Python, allowing you to expose Rust code as native Python modules"
---
# PyO3

##Definition

**PyO3** es un framework de Rust para crear **extensiones de Python** y **bindings bidireccionales** entre Rust y Python. Permite exponer código Rust como módulos Python nativos, manteniendo seguridad de tipos y gestión de memoria automática.

## How It Works

### Architecture

```
┌─────────────────────────────────────┐
│         Código Python                │
│  import vantadb                      │
│  db = vantadb.VantaEmbedded("./data")│
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│         PyO3 Layer                   │
│  - Conversión de tipos               │
│  - Gestión de referencias            │
│  - Manejo de GIL                     │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│         Código Rust                  │
│  pub struct VantaEmbedded { ... }    │
│  impl VantaEmbedded { ... }          │
└─────────────────────────────────────┘
```
*Concurrency management:* [[gil|GIL]]


### Binding Example

```rust
use pyo3::prelude::*;

#[pyclass]
pub struct VantaEmbedded {
    inner: Arc<RwLock<Engine>>,
}

#[pymethods]
impl VantaEmbedded {
    #[new]
    fn new(path: &str) -> PyResult<Self> {
        let engine = Engine::open(path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        Ok(Self { inner: Arc::new(RwLock::new(engine)) })
    }
    
    fn put(&self, py: Python<'_>, key: &str, vector: Vec<f32>, text: &str) -> PyResult<()> {
        // Liberar GIL para operaciones pesadas
        py.allow_threads(|| {
            let engine = self.inner.write().unwrap();
            engine.put(key, &vector, text)
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }
}

#[pymodule]
fn vantadb(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<VantaEmbedded>()?;
    Ok(())
}
```

## Usage in VantaDB

### Python SDK Structure

```
vantadb-python/
├── Cargo.toml          # Dependencias PyO3
├── pyproject.toml      # Configuración Maturin
├── src/
│   └── lib.rs          # Bindings PyO3
└── python/
    └── vantadb/
        ├── __init__.py
        └── _vantadb.so  # Binario compilado
```

### Facility

```bash
# Desde PyPI (binarios precompilados)
pip install vantadb-py

# Desde fuente (requiere Rust)
pip install maturin
maturin develop
```

### Use from Python

```python
from vantadb import VantaEmbedded

# Create instance
db = VantaEmbedded("./agent_memory")

# Insert document with vector
db.put(
    key="doc1",
    vector=[0.12, -0.34, 0.56, ...],
    text="VantaDB is an embedded database",
    metadata={"source": "web", "date": "2026-06-12"}
)

# Search by vector similarity
results = db.search(
    vector=[0.11, -0.33, 0.55, ...],
    top_k=10
)

# hybrid-search (vector + lexical)
results = db.search(
    vector=[0.11, -0.33, 0.55, ...],
    text="database",
    top_k=10,
    mode="hybrid"
)
```

## Management of [[gil]]

### The Problem

Python's [[gil]] (Global Interpreter Lock) prevents multiple threads from executing Python code simultaneously. If a Rust operation is long and keeps the GIL, it **crashes the entire Python interpreter**.

### Solution: `py.allow_threads()`

```rust
fn search(&self, py: Python<'_>, vector: Vec<f32>, top_k: usize) -> PyResult<Vec<SearchResult>> {
    // Liberar GIL antes de operación pesada
    py.allow_threads(|| {
        // Este código corre sin GIL
        // Otros threads Python pueden ejecutarse
        let engine = self.inner.read().unwrap();
        engine.search(&vector, top_k)
    })
}
```

### Critical Rules

1. ✅ **Release GIL** for operations >10ms
2. ❌ **DO NOT access** Python objects within `allow_threads`
3. ✅ **Clone data** before releasing GIL
4. ❌ **DO NOT make callbacks** to Python from code without GIL

### Correct Example

```rust
fn put(&self, py: Python<'_>, key: String, vector: Vec<f32>, metadata: HashMap<String, String>) -> PyResult<()> {
    // 1. Clonar datos ANTES de liberar GIL
    let key_clone = key.clone();
    let vector_clone = vector.clone();
    let metadata_clone = metadata.clone();
    
    // 2. Liberar GIL
    py.allow_threads(move || {
        // 3. Usar solo datos clonados (no objetos Python)
        let engine = self.inner.write().unwrap();
        engine.put(&key_clone, &vector_clone, &metadata_clone)
    })
}
```

## Type Conversion

### Supported Types

| Python | Rust | PyO3 |
|--------|------|------|
| `str` | `String`, `&str` | ✅ Automático |
| `int` | `i64`, `u64` | ✅ Automático |
| `float` | `f64` | ✅ Automático |
| `list` | `Vec<T>` | ✅ Automático |
| `dict` | `HashMap<K, V>` | ✅ Automático |
| `None` | `Option<T>` | ✅ Automático |
| `bytes` | `Vec<u8>`, `&[u8]` | ✅ Automático |

### Custom Types

```rust
#[pyclass]
pub struct SearchResult {
    #[pyo3(get)]
    key: String,
    #[pyo3(get)]
    score: f32,
    #[pyo3(get)]
    text: String,
}

#[pymethods]
impl SearchResult {
    fn __repr__(&self) -> String {
        format!("SearchResult(key='{}', score={:.4})", self.key, self.score)
    }
}
```

## Compilation with Maturin

### pyproject.toml

```toml
[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[project]
name = "vantadb-py"
version = "0.1.4"
requires-python = ">=3.8"

[tool.maturin]
bindings="pyo3"
features = ["pyo3/extension-module"]
```

### Build

```bash
# Desarrollo (instala en el entorno actual)
maturin develop

# Release (generate wheel)
maturin build --release

# Publish to PyPI
maturin publish
```

### Wheels Multi-Platform

PyO3 + Maturin generan wheels para:
- Linux x86_64
- Linux ARM64
- macOS x86_64
- macOS ARM64 (Apple Silicon)
- Windows x86_64

## Advantages of PyO3

| Ventaja | Descripción |
|---------|-------------|
| **Performance** | Código Rust corre a velocidad nativa |
| **Seguridad** | Type safety de Rust + gestión de memoria automática |
| **Ergonomía** | API declarativa con macros |
| **GIL management** | `allow_threads` para concurrencia real |
| **Ecosistema** | Integración con NumPy, Pandas, etc. |

## Desventajas de PyO3

| Desventaja | Descripción |
|-----------|-------------|
| **Compilación** | Requiere toolchain de Rust |
| **Debugging** | Más complejo que Python puro |
| **Overhead FFI** | Cruce de frontera tiene costo (~1-10μs) |
| **Learning curve** | Requiere entender Rust + PyO3 |

## Known Issues in VantaDB

### AUD-01: GIL Not Consistently Released

**Severity:** ⚠️ High

Some heavy operations do not release the GIL, causing crashes in multi-threaded applications.

**Mitigation:** Audit all `#[pymethods]` to ensure `py.allow_threads()`.

### AUD-03: Incomplete Type Conversion

**Severity:** ℹ️ Medium

Some metadata types are not validated correctly at the FFI border.

**Mitigation:** Strict type validation at the boundary.

## Comparison with Alternatives

| Framework | Lenguaje | Performance | Ergonomía | Caso de Uso |
|-----------|----------|-------------|-----------|-------------|
| **PyO3** | Rust | Excelente | Buena | VantaDB, extensiones de alto rendimiento |
| **Cython** | C/Python | Buena | Media | NumPy, SciPy |
| **cffi** | C | Buena | Baja | Legacy C libraries |
| **ctypes** | C | Regular | Baja | Prototipos rápidos |
| **SWIG** | Multi | Variable | Baja | Legacy multi-lenguaje |

## See Also

- [[gil]] — Lock that PyO3 must manage
- [[ffi]] — Boundary that PyO3 crosses
- [[transactional]] — Guarantees maintained through FFI

---

*PyO3 es el puente que permite a VantaDB ofrecer performance de Rust con ergonomía de Python.*

