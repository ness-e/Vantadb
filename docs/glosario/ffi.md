---
type: glossary-entry
status: stable
tags: [interoperabilidad, ffi, bindings, c-abi]
last_refined: 2026-06
links: "[[README.md]]"
aliases: [Foreign Function Interface, C FFI]
---
#FFI—Foreign Function Interface

##Definition

**FFI** (Foreign Function Interface) is a mechanism that allows code written in one programming language to call functions written in another language. In the context of VantaDB, FFI refers to the boundary between **Rust** (core engine) and **Python** (SDK).

## How FFI Works

### General Architecture

```
┌─────────────────────────────────────┐
│      Lenguaje A (Python)             │
│  - Gestión de memoria: GC           │
│  - Tipos: dinámicos                 │
│  - Threading: GIL                   │
└──────────────┬──────────────────────┘
               │
               │  FFI Boundary
               │  - Conversión de tipos
               │  - Marshalling de datos
               │  - Gestión de errores
               │
               ▼
┌─────────────────────────────────────┐
│      Lenguaje B (Rust)               │
│  - Gestión de memoria: Ownership    │
│  - Tipos: estáticos                 │
│  - Threading: Nativo                │
└─────────────────────────────────────┘
```

### C ABI (Application Binary Interface)

Most FFIs use **C ABI** as a lingua franca:

```rust
// Rust: Exponer función con C ABI
#[no_mangle]
pub extern "C" fn vanta_search(
    db: *mut VantaDB,
    vector: *const f32,
    vector_len: usize,
    top_k: usize,
) -> *mut SearchResult {
    // Implementación
}
```

```c
// C: Llamar función
extern SearchResult* vanta_search(
    VantaDB* db,
    const float* vector,
    size_t vector_len,
    size_t top_k
);
```

## FFI in VantaDB

### Technology Stack

```
Python
  │
  ▼
[[pyo3]] (Framework de bindings)
  │
  ▼
FFI Boundary (C ABI)
  │
  ▼
Rust Core (VantaDB Engine)
```

### Example of FFI Crossover

```python
# Python
db.search(vector=[0.1, 0.2, 0.3], top_k=10)
```

```rust
// PyO3: Recibir llamada
#[pymethods]
impl VantaEmbedded {
    fn search(&self, py: Python<'_>, vector: Vec<f32>, top_k: usize) -> PyResult<Vec<SearchResult>> {
        // 1. FFI: Python → Rust (conversión de tipos)
        let vector_rust: Vec<f32> = vector;
        
        // 2. Ejecutar en Rust
        let results = self.engine.search(&vector_rust, top_k);
        
        // 3. FFI: Rust → Python (conversión de retorno)
        Ok(results)
    }
}
```

## FFI Challenges

### 1. Type Conversion

| Python | Rust | Desafío |
|--------|------|---------|
| `list[float]` | `Vec<f32>` | Copia de datos |
| `dict[str, Any]` | `HashMap<String, Value>` | Validación de tipos |
| `None` | `Option<T>` | Representación de null |
| `str` | `String` | Encoding UTF-8 |

### 2. Memory Management

```rust
// ❌ PELIGRO: Retornar referencia a datos locales
fn bad_ffi(&self) -> &str {
    let local_string = String::from("hello");
    &local_string  // ERROR: local_string se libera al retornar
}

// ✅ CORRECT: Return owned data
fn good_ffi(&self) -> String {
    String::from("hello") // Python receives ownership
}
```

### 3. Error Handling

```rust
// Rust: Result<T, E>
fn search(&self, vector: &[f32]) -> Result<Vec<SearchResult>, VantaError> {
    // ...
}

// Python: Exceptions
#[pymethods]
impl VantaEmbedded {
    fn search(&self, vector: Vec<f32>) -> PyResult<Vec<SearchResult>> {
        self.engine.search(&vector)
            .map_err(|e| match e {
                VantaError::InvalidVector => PyErr::new::<pyo3::exceptions::PyValueError, _>("Invalid vector"),
                VantaError::NotFound => PyErr::new::<pyo3::exceptions::PyKeyError, _>("Not found"),
                _ => PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()),
            })
    }
}
```

### 4. Concurrency and [[gil]]

```rust
// Liberar [[gil]] antes de operación pesada
fn search(&self, py: Python<'_>, vector: Vec<f32>) -> PyResult<Vec<SearchResult>> {
    py.allow_threads(|| {
        // Sin GIL: otros threads Python pueden correr
        self.engine.search(&vector, 10)
    })
}
```

## FFI Overhead

### Typical Costs

| Operación | Overhead |
|-----------|----------|
| **Llamada FFI** | ~1-10 μs |
| **Conversión list→Vec** | ~0.1 μs por elemento |
| **Conversión dict→HashMap** | ~1 μs por par |
| **Liberar GIL** | ~0.5 μs |

### Example: Impact on Latency

```
busqueda-vectorial (100K vectores):
- Tiempo en Rust: 6 ms
- Overhead FFI: 0.05 ms
- Overhead conversión: 0.01 ms
- Total en Python: 6.06 ms

Overhead FFI: ~1% (negligible)
```

### When Overhead Matters

| Caso | Overhead FFI | Impacto |
|------|--------------|---------|
| **1 llamada grande** (search 10K vectores) | 1% | Despreciable |
| **10K llamadas pequeñas** (get por key) | 50%+ | **Significativo** |

**Solution:** Batch operations to reduce FFI crossovers.

```python
# ❌ Lento: 10K cruces FFI
for key in keys:
    db.get(key)

# ✅ Fast: 1 FFI crossing
db.get_batch(keys)
```

## Security in FFI

### `unsafe` code in FFI

```rust
// FFI a menudo requiere unsafe
pub unsafe extern "C" fn vanta_get(
    db: *mut VantaDB,
    key: *const u8,
    key_len: usize,
) -> *mut u8 {
    // unsafe: punteros raw, sin garantías de validez
    let db = &*db;  // Dereferenciar puntero
    let key = std::slice::from_raw_parts(key, key_len);
    // ...
}
```

### FFI Safety Rules

1. ✅ **Validate pointers** before dereferencing
2. ✅ **Validate lengths** of arrays
3. ✅ **Handle null pointers** explicitly
4. ✅ **Document invariants** (who frees memory)
5. ❌ **DO NOT assume** that the caller follows the contract

### Insurance Example

```rust
pub unsafe extern "C" fn vanta_get(
    db: *mut VantaDB,
    key: *const u8,
    key_len: usize,
) -> *mut u8 {
    // 1. Validar punteros
    if db.is_null() || key.is_null() {
        return std::ptr::null_mut();
    }
    
    // 2. Validar longitud
    if key_len == 0 || key_len > MAX_KEY_SIZE {
        return std::ptr::null_mut();
    }
    
    // 3. Convertir a tipos seguros
    let db = &*db;
    let key_slice = std::slice::from_raw_parts(key, key_len);
    
    // 4. Ejecutar lógica segura
    match db.get(key_slice) {
        Ok(Some(value)) => {
            // 5. Retornar owned data (caller libera)
            let boxed = value.into_boxed_slice();
            Box::into_raw(boxed) as *mut u8
        }
        _ => std::ptr::null_mut(),
    }
}
```

## FFI tools in Rust

###PyO3 (Python)

```rust
use pyo3::prelude::*;

#[pyclass]
struct MyClass { /* ... */ }

#[pymethods]
impl MyClass {
    fn method(&self) -> PyResult<()> { /* ... */ }
}
```

### cbindgen (C/C++)

```bash
# Generar header C desde código Rust
cbindgen --config cbindgen.toml --output mylib.h
```

### wasm-bindgen (WebAssembly)

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
```

## Comparison of FFIs

| FFI | Lenguajes | Performance | Ergonomía | Caso de Uso |
|-----|-----------|-------------|-----------|-------------|
| **PyO3** | Rust ↔ Python | Excelente | Excelente | VantaDB |
| **N-API** | C/C++ ↔ Node.js | Buena | Buena | Node modules |
| **JNI** | Java ↔ C/C++ | Regular | Mala | Android |
| **Cgo** | Go ↔ C | Regular | Regular | Legacy C libs |
| **WASM** | Multi ↔ Browser | Buena | Buena | Web apps |

## Known Issues in VantaDB

### AUD-01: Type Validation in FFI

**Severity:** ℹ️ Medium

**Description:** Some metadata types are not validated correctly at the FFI border.

**Impact:** Possible panics or unexpected behavior with invalid inputs.

**Mitigation:**
``rust
fn put(&self, metadata: HashMap<String, Value>) -> PyResult<()> {
    // Validate types BEFORE processing
    for (key, value) in &metadata {
        if !is_valid_metadata_value(value) {
            return Err(PyValueError::new_err(
                format!("Invalid metadata value for key '{}'", key)
            ));
        }
    }
    //...
}
```

## See Also

- [[pyo3]] — Framework FFI para Python
- [[gil]] — Lock que se libera en FFI
- [[transactional]] — Garantías que cruzan FFI

---

*FFI is the bridge that allows you to combine the ergonomics of Python with the performance of Rust.*

