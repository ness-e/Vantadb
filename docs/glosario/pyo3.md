---
type: glosario-entry
status: stable
tags: [ffi, python, rust, bindings]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
aliases: [PyO3 Bindings, Rust-Python Bindings]
description: "Framework de Rust para crear extensiones de Python y bindings bidireccionales entre Rust y Python, permitiendo exponer código Rust como módulos Python nativos"
---

# PyO3

## Definición

**PyO3** es un framework de Rust para crear **extensiones de Python** y **bindings bidireccionales** entre Rust y Python. Permite exponer código Rust como módulos Python nativos, manteniendo seguridad de tipos y gestión de memoria automática.

## Cómo Funciona

### Arquitectura

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
│  - Manejo de [GIL](GIL.md)                 │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│         Código Rust                  │
│  pub struct VantaEmbedded { ... }    │
│  impl VantaEmbedded { ... }          │
└─────────────────────────────────────┘
```

### Ejemplo de Binding

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

## Uso en VantaDB

### Estructura del SDK Python

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

### Instalación

```bash
# Desde PyPI (binarios precompilados)
pip install vantadb-py

# Desde fuente (requiere Rust)
pip install maturin
maturin develop
```

### Uso desde Python

```python
from vantadb import VantaEmbedded

# Crear instancia
db = VantaEmbedded("./agent_memory")

# Insertar documento con vector
db.put(
    key="doc1",
    vector=[0.12, -0.34, 0.56, ...],
    text="VantaDB es una base de datos embebida",
    metadata={"source": "web", "date": "2026-06-12"}
)

# Buscar por similitud vectorial
results = db.search(
    vector=[0.11, -0.33, 0.55, ...],
    top_k=10
)

# busqueda-hibrida (vectorial + léxica)
results = db.search(
    vector=[0.11, -0.33, 0.55, ...],
    text="base de datos",
    top_k=10,
    mode="hybrid"
)
```

## Gestión del [GIL](GIL.md)

### El Problema

El [GIL](GIL.md) (Global Interpreter Lock) de Python impide que múltiples threads ejecuten código Python simultáneamente. Si una operación Rust es larga y mantiene el GIL, **bloquea todo el intérprete Python**.

### Solución: `py.allow_threads()`

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

### Reglas Críticas

1. ✅ **Liberar GIL** para operaciones >10ms
2. ❌ **NO acceder** a objetos Python dentro de `allow_threads`
3. ✅ **Clonar datos** antes de liberar GIL
4. ❌ **NO hacer callbacks** a Python desde código sin GIL

### Ejemplo Correcto

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

## Conversión de Tipos

### Tipos Soportados

| Python | Rust | PyO3 |
|--------|------|------|
| `str` | `String`, `&str` | ✅ Automático |
| `int` | `i64`, `u64` | ✅ Automático |
| `float` | `f64` | ✅ Automático |
| `list` | `Vec<T>` | ✅ Automático |
| `dict` | `HashMap<K, V>` | ✅ Automático |
| `None` | `Option<T>` | ✅ Automático |
| `bytes` | `Vec<u8>`, `&[u8]` | ✅ Automático |

### Tipos Personalizados

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

## Compilación con Maturin

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
bindings = "pyo3"
features = ["pyo3/extension-module"]
```

### Build

```bash
# Desarrollo (instala en el entorno actual)
maturin develop

# Release (genera wheel)
maturin build --release

# Publicar en PyPI
maturin publish
```

### Wheels Multi-Platform

PyO3 + Maturin generan wheels para:
- Linux x86_64
- Linux ARM64
- macOS x86_64
- macOS ARM64 (Apple Silicon)
- Windows x86_64

## Ventajas de PyO3

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

## Problemas Conocidos en VantaDB

### AUD-01: GIL No Liberado Consistentemente

**Severidad:** ⚠️ Alta

Algunas operaciones pesadas no liberan el GIL, causando bloqueos en aplicaciones multi-thread.

**Mitigación:** Auditoría de todos los `#[pymethods]` para asegurar `py.allow_threads()`.

### AUD-03: Conversión de Tipos Incompleta

**Severidad:** ℹ️ Media

Algunos tipos de metadata no se validan correctamente en la frontera FFI.

**Mitigación:** Validación estricta de tipos en el boundary.

## Comparación con Alternativas

| Framework | Lenguaje | Performance | Ergonomía | Caso de Uso |
|-----------|----------|-------------|-----------|-------------|
| **PyO3** | Rust | Excelente | Buena | VantaDB, extensiones de alto rendimiento |
| **Cython** | C/Python | Buena | Media | NumPy, SciPy |
| **cffi** | C | Buena | Baja | Legacy C libraries |
| **ctypes** | C | Regular | Baja | Prototipos rápidos |
| **SWIG** | Multi | Variable | Baja | Legacy multi-lenguaje |

## Véase También

- [GIL](GIL.md) — Lock que PyO3 debe gestionar
- [FFI](FFI.md) — Frontera que PyO3 cruza
- [Transaccional](Transaccional.md) — Garantías que se mantienen a través de FFI

---

*PyO3 es el puente que permite a VantaDB ofrecer performance de Rust con ergonomía de Python.*

