# Reporte de Investigación — INV-004: mimalloc como Global Allocator

> **ID:** `INV-004`  
> **Categoría:** Phase 4 — Engineering Health & Architecture  
> **Fecha:** 2026-07-31  
> **Estado:** ✅ Investigación Completada — Propuesta Lista  

---

## 1. Contexto y Objetivos

En sistemas intensivos en memoria como VantaDB (que manejan índices HNSW, grafos en RAM y serialización frecuente de vectores f32), el asignador de memoria por defecto del sistema operativo (`glibc malloc` en Linux, `MSVC CRT` en Windows) puede causar fragmentación de heap (*RSS drift*) y contención en asignaciones multihilo.

La investigación evaluó la integración de `mimalloc` (el asignador compacto y de alto rendimiento de Microsoft) frente a `jemalloc` y el asignador del sistema.

---

## 2. Hallazgos del Análisis de Código e Infraestructura

### 2.1 Configuración Existente en `Cargo.toml`
La base de código **ya cuenta con soporte para `mimalloc` y `jemalloc`** mediante feature flags opcionales:

```toml
custom-allocator = ["mimalloc"]
jemalloc = ["dep:tikv-jemallocator", "dep:tikv-jemalloc-ctl", "tikv-jemalloc-ctl/stats"]

[dependencies.mimalloc]
version = "0.1"
optional = true
```

> **Nota de verificación (2026-08-04):** `tikv-jemallocator` y `tikv-jemalloc-ctl` están restringidos a `cfg(any(target_os = "linux", target_os = "macos"))` en `Cargo.toml`. En Windows, `jemalloc` no compila — solo `custom-allocator` (mimalloc) está disponible. La fila "Windows → mimalloc" de la matriz (Sección 3) es por tanto un requisito de compilación, no solo una recomendación.

### 2.2 Declaración en Binarios (`src/bin/vanta-cli.rs`, `vantadb-server/src/main.rs`)
La integración está **duplicada en los dos binarios distribuibles** — `src/bin/vanta-cli.rs` y `vantadb-server/src/main.rs` — y ambos implementan la misma selección condicional del asignador global:

```rust
#[cfg(all(feature = "jemalloc", not(target_os = "windows")))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[cfg(all(
    feature = "custom-allocator",
    any(not(feature = "jemalloc"), target_os = "windows")
))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
```

---

## 3. Matriz Comparativa por Plataforma

| Plataforma | Allocator Estándar | `jemalloc` | `mimalloc` | Recomendación |
|---|---|---|---|---|
| **Linux** | glibc malloc | ✅ Excelente (Estándar en DBs) | ✅ Excelente | `jemalloc` (o `mimalloc` como fallback) |
| **macOS** | system malloc | ✅ Bueno | ✅ Bueno | `mimalloc` / `jemalloc` |
| **Windows** | MSVC CRT | ❌ Complicado / No recomendado | ✅ **Óptimo (Nativo)** | **`mimalloc` via `custom-allocator`** |
| **WASM** | wee_alloc / system | ❌ No soportado | ❌ No soportado | System / dlmalloc por defecto |

---

## 4. Conclusiones y Recomendaciones

1. **Windows:** Usar `custom-allocator` (`mimalloc`) proporciona la mayor reducción de fragmentación de heap y mejora la velocidad de asignación en pruebas de carga concurrentes.
2. **Impacto en Binarios:** `mimalloc` añade < 100 KB al tamaño final del binario (estimación de la investigación — no verificado por build en este repo), lo que representa un impacto mínimo comparado con los beneficios en latencia.
3. **Recomendación para Builds de Producción:**
   - Para ejecutables de servidor o CLI en Windows: compilar con `--features custom-allocator`.
   - Para entornos Linux/macOS: compilar con `--features jemalloc` o `custom-allocator`.

---
*Reporte generado automáticamente como parte de INV-004.*
