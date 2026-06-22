---
type: glosario-entry
status: stable
tags: [interoperabilidad, ffi, bindings, c-abi]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
aliases: [Foreign Function Interface, C FFI]
---

# FFI — Foreign Function Interface

## Definición

**FFI** (Foreign Function Interface) es un mecanismo que permite que código escrito en un lenguaje de programación **llame funciones escritas en otro lenguaje**. En el contexto de VantaDB, FFI se refiere a la frontera entre **Rust** (core engine) y **Python** (SDK).

## Cómo Funciona FFI

### Arquitectura General

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

La mayoría de FFIs usan el **C ABI** como lingua franca:

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

## FFI en VantaDB

### Stack Tecnológico

```
Python
  │
  ▼
[PyO3](PyO3.md) (Framework de bindings)
  │
  ▼
FFI Boundary (C ABI)
  │
  ▼
Rust Core (VantaDB Engine)
```

### Ejemplo de Cruce FFI

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

## Desafíos de FFI

### 1. Conversión de Tipos

| Python | Rust | Desafío |
|--------|------|---------|
| `list[float]` | `Vec<f32>` | Copia de datos |
| `dict[str, Any]` | `HashMap<String, Value>` | Validación de tipos |
| `None` | `Option<T>` | Representación de null |
| `str` | `String` | Encoding UTF-8 |

### 2. Gestión de Memoria

```rust
// ❌ PELIGRO: Retornar referencia a datos locales
fn bad_ffi(&self) -> &str {
    let local_string = String::from("hello");
    &local_string  // ERROR: local_string se libera al retornar
}

// ✅ CORRECTO: Retornar owned data
fn good_ffi(&self) -> String {
    String::from("hello")  // Python recibe ownership
}
```

### 3. Manejo de Errores

```rust
// Rust: Result<T, E>
fn search(&self, vector: &[f32]) -> Result<Vec<SearchResult>, VantaError> {
    // ...
}

// Python: Excepciones
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

### 4. Concurrencia y [GIL](GIL.md)

```rust
// Liberar [GIL](GIL.md) antes de operación pesada
fn search(&self, py: Python<'_>, vector: Vec<f32>) -> PyResult<Vec<SearchResult>> {
    py.allow_threads(|| {
        // Sin GIL: otros threads Python pueden correr
        self.engine.search(&vector, 10)
    })
}
```

## Overhead de FFI

### Costos Típicos

| Operación | Overhead |
|-----------|----------|
| **Llamada FFI** | ~1-10 μs |
| **Conversión list→Vec** | ~0.1 μs por elemento |
| **Conversión dict→HashMap** | ~1 μs por par |
| **Liberar GIL** | ~0.5 μs |

### Ejemplo: Impacto en Latencia

```
Búsqueda vectorial (100K vectores):
- Tiempo en Rust: 6 ms
- Overhead FFI: 0.05 ms
- Overhead conversión: 0.01 ms
- Total en Python: 6.06 ms

Overhead FFI: ~1% (despreciable)
```

### Cuándo el Overhead Importa

| Caso | Overhead FFI | Impacto |
|------|--------------|---------|
| **1 llamada grande** (search 10K vectores) | 1% | Despreciable |
| **10K llamadas pequeñas** (get por key) | 50%+ | **Significativo** |

**Solución:** Batch operations para reducir cruces FFI.

```python
# ❌ Lento: 10K cruces FFI
for key in keys:
    db.get(key)

# ✅ Rápido: 1 cruce FFI
db.get_batch(keys)
```

## Seguridad en FFI

### Código `unsafe` en FFI

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

### Reglas de Seguridad FFI

1. ✅ **Validar punteros** antes de dereferenciar
2. ✅ **Validar longitudes** de arrays
3. ✅ **Manejar null pointers** explícitamente
4. ✅ **Documentar invariantes** (quién libera memoria)
5. ❌ **NO asumir** que el caller sigue el contrato

### Ejemplo Seguro

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

## Herramientas FFI en Rust

### PyO3 (Python)

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

## Comparación de FFIs

| FFI | Lenguajes | Performance | Ergonomía | Caso de Uso |
|-----|-----------|-------------|-----------|-------------|
| **PyO3** | Rust ↔ Python | Excelente | Excelente | VantaDB |
| **N-API** | C/C++ ↔ Node.js | Buena | Buena | Node modules |
| **JNI** | Java ↔ C/C++ | Regular | Mala | Android |
| **Cgo** | Go ↔ C | Regular | Regular | Legacy C libs |
| **WASM** | Multi ↔ Browser | Buena | Buena | Web apps |

## Problemas Conocidos en VantaDB

### AUD-01: Validación de Tipos en FFI

**Severidad:** ℹ️ Media

**Descripción:** Algunos tipos de metadata no se validan correctamente en la frontera FFI.

**Impacto:** Posibles panics o comportamiento inesperado con inputs inválidos.

**Mitigación:**
```rust
fn put(&self, metadata: HashMap<String, Value>) -> PyResult<()> {
    // Validar tipos ANTES de procesar
    for (key, value) in &metadata {
        if !is_valid_metadata_value(value) {
            return Err(PyValueError::new_err(
                format!("Invalid metadata value for key '{}'", key)
            ));
        }
    }
    // ...
}
```

## Véase También

- [PyO3](PyO3.md) — Framework FFI para Python
- [GIL](GIL.md) — Lock que se libera en FFI
- [Transaccional](Transaccional.md) — Garantías que cruzan FFI

---

*FFI es el puente que permite combinar la ergonomía de Python con la performance de Rust.*

