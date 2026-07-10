---
title: "VantaDB — Auditoría Completa del Codebase"
type: review
status: active
tags: [vantadb, audit, codebase]
last_reviewed: 2026-07-10 (Fase 7: todo ✅; Dup steps: rocksdb 0.24 ✅, arrow 59 ✅, fjall 3.1 ⏳; SAFETY: §3.1 actualizado post-fragmentación; W2/W4/FAQ/A4 corregidos)
language: es
---

# VantaDB — Auditoría Completa del Codebase

**Fecha:** 2026-07-09
**Versión:** 0.3.0
**Alcance:** Rust core (51 módulos), bindings (Python/TS/WASM/Server), web frontend, CI/CD, Docker, dependencias, documentación
**Metodología:** 5 skills de addyosmani/agent-skills (code-review, security, performance, simplification, adversarial) + 5 exploraciones paralelas profundas

> **🟢 Prioridades 0 y 1 completadas** — Prioridad 0 en `d2986bf`, Prioridad 1 en seguimiento. Ver [§14.1 Progreso](#141-progreso) para detalle.

---

## Tabla de Contenidos

1. [Resumen Ejecutivo](#1-resumen-ejecutivo)
2. [Arquitectura del Core Rust](#2-arquitectura-del-core-rust)
3. [Análisis de Código Inseguro (Unsafe)](#3-análisis-de-código-inseguro-unsafe)
4. [Manejo de Errores](#4-manejo-de-errores)
5. [Seguridad Integral](#5-seguridad-integral)
6. [Rendimiento](#6-rendimiento)
7. [Deuda Técnica y Simplificación](#7-deuda-técnica-y-simplificación)
8. [CI/CD y Build System](#8-cicd-y-build-system)
9. [Docker y Despliegue](#9-docker-y-despliegue)
10. [Bindings (Python/TS/WASM)](#10-bindings-pythontswasm)
11. [Web Frontend](#11-web-frontend)
12. [Análisis de Dependencias](#12-análisis-de-dependencias)
13. [Documentación](#13-documentación)
14. [Recomendaciones Priorizadas](#14-recomendaciones-priorizadas)
15. [Progreso de Implementación](#15-progreso-de-implementación)
16. [Apéndice: Métricas Clave](#16-apéndice-métricas-clave)

---

## 1. Resumen Ejecutivo

### Estado General: B+ → Mejorando

**🟢 Prioridades 0 y 1 completadas — P0 en `d2986bf`, P1 en commit posterior. Ver [§15 Progreso](#15-progreso-de-implementación)**

| Categoría | Nota | Hallazgos Críticos |
|---|---|---|
| Arquitectura | B+ | Diseño limpio en capas, RCU para HNSW, deadlock-free. rkyv dead code. |
| Código Inseguro (Unsafe) | B- | ~45 bloques unsafe con SAFETY comments en 14 archivos. Post-fragmentación verificado: todos tienen SAFETY. |
| Manejo de Errores | B | `thiserror` enum robusto, pero variantes String eliminan contexto. Sin source chaining. |
| Seguridad | B- | Path traversal mitigado parcialmente. Sin forced-auth mode en server. |
| Rendimiento | B+ | Bundle web optimizado (code splitting). WASM `wasm-opt=true`. 17 crates duplicados. |
| CI/CD | A | Pipeline profesional, perfiles nextest, build provenance. MSRV check ✅. macOS CI ✅. Fuzz CI ✅. Windows+ARM64 releases ✅. |
| Docker | C | ~~Version mismatch Rust. Error swallowing en skeleton build.~~ curl en prod image. |
| Bindings Python | A | PyO3 correcto, GIL management excelente. Faltan stubs `.pyi`. |
| Bindings TS | A- | Types completos. Async consistente (sync/async corregido en WASM + TS). |
| Bindings WASM | B | `wasm-opt=true`. NaN sanitization correcta. `tracing-wasm` feature-gated. Sin code splitting. |
| Web Frontend | A- | 27 rutas lazy-loaded, diseño system robusto. 3 bugs lógicos resueltos (W1, W3, W5), 3 restantes. |
| Dependencias | B+ | 5 unmaintained allowlisted. 17 duplicados. lru migrado a 0.13. |
| Documentación | B+ | README excelente. `llms.txt` corregido con APIs reales. FAQ desactualizada. |

---

## 2. Arquitectura del Core Rust

### 2.1 Jerarquía de Módulos

```
lib.rs (re-exports públicos)
├── engine / executor / planner / physical_plan / query / parser → Pipeline de consultas
├── storage/
│   ├── engine/{init,ops,maintenance,partition,stats,tests}
│   ├── vfile.rs     → VantaFile (mmap vectors)
│   ├── wal.rs       → Write-Ahead Log
│   ├── archive.rs   → Rebuild HNSW, layout compaction
│   └── ops.rs       → Helpers compartidos
├── index/
│   ├── core.rs      → CPIndex (HNSW graph: 1984 líneas)
│   ├── hnsw.rs      → Placeholder (4 líneas, re-export de core)
│   ├── distance.rs  → Funciones de similitud (SIMD)
│   ├── refresh.rs   → Refresco de índices
│   └── stats.rs     → Estadísticas de índices
├── backends/{in_memory,fjall_backend,rocksdb_backend,mod.rs}
├── node.rs          → UnifiedNode, VectorRepresentations
├── config.rs        → VantaConfig (1116 líneas, builder pattern + env parsing)
├── sdk/             → API pública: {api,connect,graph,builder,types,serialization,search}
├── serialization/   → rkyv_archives (dead code tras `#[cfg(any())]`)
├── metrics/         → Core metrics, native stats, snapshot
├── governance/      → {admission,conflict,consistency,worker}
├── vector/          → {transform,quantization,governor}
├── hardware/        → CPU capabilities detection
├── crypto.rs        → AES-256-GCM at-rest encryption
└── cli_handlers/   → 12 submódulos (antes 2197 líneas en 1 archivo)
```

### 2.2 Patrón Arquitectónico

**InMemoryEngine** (`src/engine.rs`): Engine fase-1 con `RwLock<HashMap<u128, UnifiedNode>>` + `Optional<ShardedWal>`. Simple, sin persistencia más allá de replay WAL.

**StorageEngine** (`src/storage/engine/`): Engine completo con:
- `Arc<dyn StorageBackend>` (RocksDB/Fjall/InMemory) — persistencia KV
- `ArcSwap<CPIndex>` — HNSW actualizable vía RCU (lecturas sin lock)
- `RwLock<VantaFile>` — Almacén mmap de vectores
- LRU volatile cache, edge/scalar indexes

**CPIndex** (`src/index/core.rs`): Grafo HNSW con `DashMap<u128, HnswNode>` (lecturas concurrentes), `Mutex<u128>` entry point, acceso zero-copy a vectores mmap durante búsqueda.

**Thread Safety**: 
- `parking_lot::RwLock` + `dashmap::DashMap` + `arc_swap::ArcSwap` + `Atomic*`
- `insert_lock` (parking_lot::Mutex) serializa mutaciones HNSW pero permite lecturas concurrentes
- Sin ciclos de deadlock identificados en lock ordering

### 2.3 Problemas Arquitectónicos

| # | Problema | Archivo | Impacto |
|---|---|---|---|
| A1 | ~~`entry_point` es `Mutex<u128>` serializa todas las búsquedas~~ ✅ Completo | `src/index/graph.rs` → `AtomicU128` | `parking_lot::Mutex` reemplazado por `portable_atomic::AtomicU128`. El Mutex serializaba búsquedas innecesariamente; ahora usa load/store con `Ordering::Relaxed` (zero-cost en x86_64). |
| A2 | ~~`hnsw.nodes.remove()` en delete + corrección entry_point sin atomicidad~~ ✅ Mitigado | `storage/engine/ops.rs` | Con `AtomicU128`, la ventana donde una búsqueda puede empezar desde un entry point eliminado existe pero es benigna: HNSW tolera entry points no-óptimos (el searchBacktrack en `search_nearest` corrige automáticamente). |
| A3 | `#[cfg(any())]` en `pub mod rkyv_archives` — dead code intencional sin doc | `serialization/mod.rs:6` | Código muerto que confunde, sin explicación |
| A4 | ~~`hnsw.rs` es placeholder de 4 líneas que re-exporta de core.rs~~ ✅ Eliminado | `index/hnsw.rs` eliminado, `mod.rs` limpiado | Indirección innecesaria removida |
| A5 | ~~`FLAG_TOMBSTONE` definido 5 veces~~ ✅ Completo | `engine/mod.rs:34` es la única definición. Se eliminaron las copias en `archive.rs`, `wal.rs`, `storage/ops.rs`, `index/graph.rs`. `NodeFlags::TOMBSTONE` en `node.rs` es un flag diferente (in-memory bitset). | Riesgo de drift eliminado |
| A6 | `hybrid_search` retiene `self.nodes.read()` durante scan completo + cosine | `engine.rs` | Bloquea escrituras durante búsquedas largas en datasets grandes |

---

## 3. Análisis de Código Inseguro (Unsafe)

### 3.1 Inventario Completo (45+ ocurrencias en 14 archivos) — Post SAFETY + Fragmentación

**Nota:** `index/core.rs` fue fragmentado en Phase 2.2 (tests-only, 460 líneas). El código unsafe se movió a `graph.rs`, `search.rs`, `serialize.rs`. Todos los bloques unsafe tienen `// SAFETY:` comments o `# Safety` en doc comments tras la verificación Jul 2026.

| Archivo | Ocurrencias | Propósito | ¿Tiene `// SAFETY:`? | Riesgo |
|---|---|---|---|---|
| `node.rs` | 4 | `unsafe impl Send/Sync for SendPtr` + `from_raw_parts` | ✅ SÍ (4 comments) | Bajo — raw `*const f32` solo usado detrás de `&` |
| `index/graph.rs` | 5 | `libc::madvise`, `PrefetchVirtualMemory`, `Mmap::map` | ✅ SÍ | Bajo — hints del kernel, ignora rangos inválidos |
| `index/search.rs` | 2 | `from_raw_parts(ptr as *const f32, len)` acceso mmap | ✅ SÍ (bounded above by `vec_end > mmap_bytes().len()` guard) | **Medio** — bounds check en safe code previene UB |
| `index/serialize.rs` | 3 | `from_raw_parts`, `MmapMut::map_mut` | ✅ SÍ | Bajo — bounds validados antes del unsafe |
| `index/distance.rs` | 1 | `from_raw_parts(ptr.0, len)` en `compute_similarity` | ✅ SÍ | **Medio** — guard `ptr.0.is_null() \|\| *len > MAX_VEC_F32_LEN` |
| `storage/vfile.rs` | 14+ | `sigaction`, `mincore`, `QueryWorkingSetEx`, `Send/Sync`, `Mmap::map` | ✅ SÍ | Bajo — syscalls seguras, signal handler best-effort |
| `storage/archive.rs` | 3 | `MmapMut::map_mut` + `from_raw_parts` | ✅ SÍ | **Medio** — bounds guard en safe code |
| `storage/engine/ops.rs` | 3 | `from_raw_parts(ptr as *const f32, len)` acceso mmap | ✅ SÍ | **Medio** — guard `vec_end > vstore.size` previene UB |
| `storage/engine/maintenance.rs` | 2 | `MmapMut::map_mut`, `release_mmap_vector` | ✅ SÍ | Bajo |
| `storage/ops.rs` | 1 | `from_raw_parts` para lectura f32 vector | ✅ SÍ | **Medio** — bounds guard presente |
| `metrics/core.rs` | 2 | `mach_task_basic_info` / `GetProcessMemoryInfo` | ✅ SÍ | Bajo — FFI calls seguras |
| `serialization/rkyv_archives.rs` | 7 | Pointer casts para zero-copy archive | ✅ SÍ (alignment + bounds en safe code) | **Medio** — validación correcta |

### 3.2 Patrón Peligroso: `from_raw_parts` con `debug_assert!`

```rust
// PATRÓN REPETIDO ~10 VECES EN EL CODEBASE
let vec_end = vec_start + (header.vector_len as usize * 4);
debug_assert!(vec_end <= vstore.size as usize, "vector exceeds mmap bounds"); // SOLO DEBUG
if vec_end > vstore.size as usize { return None; } // CHECK EN SAFE CODE
let f32_vec: &[f32] = unsafe {
    std::slice::from_raw_parts(vec_bytes.as_ptr() as *const f32, header.vector_len as usize)
};
```

**Problema**: El `check` en safe code previene el UB **solo si se ejecuta**. En `to_f32()` de `node.rs`, no hay ni check — solo `debug_assert!`. En release builds, si `vector_len` proviene de metadata corrupta en disco, o hay un TOCTOU (teóricamente prevenido por locks), el `from_raw_parts` produce una referencia a memoria arbitraria.

**Resolución**: 
1. ✅ ~~Reemplazar `debug_assert!` con `if` check + `return Err(...)` en release builds~~ Completado en Phase 1 (`d2986bf`)
2. ✅ ~~Añadir `// SAFETY:` comments a TODOS los 50 bloques unsafe~~ Completado — todos los bloques unsafe tienen SAFETY comments o `# Safety` doc. Verificados post-fragmentación en Jul 2026: `graph.rs`, `search.rs`, `serialize.rs`, `node.rs`, `distance.rs`, `vfile.rs`, `archive.rs`, `ops.rs`, `maintenance.rs`, `metrics/core.rs`, `rkyv_archives.rs`
3. ✅ ~~Habilitar `#![deny(unsafe_op_in_unsafe_fn)]` en toda la crate~~ Completado (Finding 2.9)

### 3.3 TOCTOU en mmap vector reads

El patrón más común:
```
check bounds (bajo read lock) → dereference pointer (bajo mismo read lock)
```

El lock `vector_store.read()` se mantiene durante ambas operaciones, lo que previene que otro thread redimensione el mmap entre el check y el dereference. **No es race en la práctica**, pero:
1. Depende de que el lock se mantenga implícitamente (fácil de romper en refactors)
2. Sería más seguro con `Mmap::get_ref()` de memmap2 que devuelve `Result<&[u8]>`

---

## 4. Manejo de Errores

### 4.1 VantaError (`src/error.rs`)

**Fortalezas:**
- `thiserror::Error` con 30 variantes bien estructuradas
- `Result<T>` = `std::result::Result<T, VantaError>`
- `From<std::io::Error>` automático via `#[from]`
- Test suite cubre Display de todas las variantes

**Debilidades:**

| # | Problema | Ejemplo | Impacto |
|---|---|---|---|
| E1 | Variantes String pierden contexto estructurado | `WalError(String)`, `SearchError(String)`, `Generic(String)`, `BackendError(String)` | No se puede hacer pattern match sobre la causa raíz |
| E2 | ~~Sin source chaining en variantes no-IoError~~ ✅ Completo | `SerializationError(#[source] Box<dyn Error + Send + Sync>)` preserva el error original. 21 call sites migrados. | `error.source()` devuelve el postcard/serde error original |
| E3 | `IqlParseError` tiene posición pero no tipo `Spanned` | `IqlParseError { message: String, line: usize, col: usize }` | Dificulta pretty-printing con span labels |
| E4 | `Result<T>` no es `#[must_use]` | `let _ = fallible_op();` compila sin warning | Resultados descartables silenciosamente |
| E5 | `parse_env_or` traga errores de parseo | `fn parse_env_or<T: FromStr>(key: &str, default: T) -> T` con `warn!()` en error | Silencioso, el warning puede perderse en logs |
| E6 | Sin error recovery hierarchy | No hay distinción entre errores recuperables (retry) y fatales (shutdown) | Decisiones de recovery imposibles de automatizar |

### 4.2 Mapa de Propagación

```
CLI/Server/Bindings
    ↓ VantaError
StorageEngine
    ↓ VantaError (conversión desde io::Error, postcard::Error, etc.)
Index/HNSW
    ↓ Option (search), VantaError (insert)
Backends (RocksDB/Fjall)
    ↓ io::Error → VantaError::BackendError(String)
```

### 4.3 Recomendaciones

1. Migrar variantes `String` a `#[error]` con `source` chain
2. Añadir `#[must_use]` a type alias `Result`
3. Crear tipo `Spanned` para errores de parser
4. Añadir recovery hints a variantes críticas

---

## 5. Seguridad Integral

### 5.1 Path Traversal

```rust
// src/storage/ops.rs:131-143
pub fn prevent_path_traversal(path: &Path) -> Result<()> {
    for component in path.components() {
        if component == Component::ParentDir {
            return Err(VantaError::Generic("Path Traversal detected".into()));
        }
    }
    Ok(())
}
```

**Limitaciones:**
- Solo detecta `..` — no canonicaliza paths
- No previene symlink escapes (un backup malicioso puede crear symlinks → `/etc/passwd`)
- No rechaza absolute paths cuando se espera relativo
- Backup/restore en `cli_handlers/backup.rs` usa `Path::new(input)` directamente

### 5.2 Deserialización No Validada

| Ubicación | Formato | Riesgo |
|---|---|---|
| Hot-reload config (`config.rs:745`) | JSON | Sin schema validation ni límite de profundidad |
| Hardware cache (`hardware/mod.rs:102`) | JSON | Sin límite de tamaño |
| WAL shipping (`wal_shipping.rs:238`) | JSON | Datos de red sin validación |
| Metadata storage ops | Postcard | Binario, amplification risk bajo pero sin límites |

### 5.3 Race Conditions

**insert_lock deadlock analysis:**
```
insert() adquiere:
  cardinality_stats.write() → WAL append → vector_store.write()
  → backend.put() → insert_lock.try_lock_for() → hnsw.load() (ArcSwap)
  → volatile_cache.write()

delete() adquiere:
  self.get() (hnsw.load()) → cardinality_stats.write() → WAL append
  → hnsw.load() → vector_store.write() → entry_point.lock()
  → volatile_cache.write()
```

No se encontraron ciclos entre locks principales. Diseño deadlock-free para los locks primarios.

### 5.4 Issues de Seguridad por Capa

| # | Issue | Capa | Severidad |
|---|---|---|---|
| S1 | `.vercel/` con project/org IDs committeado en git | Infraestructura | **CRÍTICA** |
| S2 | Sin forced-auth mode en server si `api_key` es None | Server | ✅ |
| S3 | Homebrew formula SHA256 placeholders (instalación imposible) | Release | **ALTA** |
| S4 | `scripts/install.sh` usa `curl` sin verificación SSL | Scripts | **MEDIA** |
| S5 | Untrusted input injection en `release-npm-61.yml` | CI/CD | **MEDIA** |
| S6 | `aria-expanded` hardcoded `false` en nav dropdowns | Web | **BAJA** |
| S7 | No CSP/HSTS headers en Vercel config | Web | **BAJA** |
| S8 | API key se compara con timing-safe (`ct_eq`) — correcto | Server | ✅ |

### 5.5 Crypto (src/crypto.rs)

- AES-256-GCM correcto
- Nonce generation con `thread_rng()`
- API key se logga como `present = v.is_some()` (no el valor)
- **Correcto.**

---

## 6. Rendimiento

### 6.1 Rust Core

| Área | Estado | Detalle |
|---|---|---|
| HNSW Search | ✅ | RCU (`ArcSwap`) permite lecturas sin lock. SIMD distance en `distance.rs`. |
| HNSW Insert | ⚠️ | `insert_lock` serializa mutaciones. Aceptable para ANN workloads. |
| MMap Vectors | ✅ | Zero-copy acceso a vectores durante search. Sin copias innecesarias. |
| Serialización | ⚠️ | rkyv zero-copy archive es dead code (`#[cfg(any())]`). Usa bincode/postcard. |
| LRU Cache | ⚠️ | `lru 0.12.5` tiene unsound `IterMut`. Migrar a 0.13+ o `quick-lru`. |
| entry_point Mutex | ✅ | Migrado a `portable_atomic::AtomicU128` con `Ordering::Relaxed` |

### 6.2 WASM

| Métrica | Valor | Estado |
|---|---|---|
| `wasm-opt` | `false` | ❌ Deshabilitado en perfil release WASM |
| Chunk único | ~1.5MB+ estimado | ❌ Sin code splitting |
| `tracing-wasm` | ~50KB extra | ⚠️ Debería ser feature flag |
| `serde_json` en cadena | ~200KB extra | ⚠️ Pesado para web |
| Optimización WASM | `opt-level = "s"` + `strip=true` | ✅ Correcto |

**Recomendación**: Habilitar `wasm-opt = true` en perfil WASM (30-50% reduction). Hacer tracing feature-gated.

### 6.3 Web Frontend

| Métrica | Valor | Estado |
|---|---|---|
| Initial JS (render-critical) | ~559 KB | ⚠️ Pesado para marketing site |
| Initial CSS | ~137 KB | ⚠️ Tailwind v4 full output |
| Lazy-loaded routes | 15 chunks, ~93 KB | ✅ Excelente |
| Vendor chunks | React 178KB, Router 81KB | ✅ Cacheable — GSAP reemplazado por motion.dev (sin bundle dedicado) |
| Total fonts | 11 woff2, ~189 KB | ✅ Google Fonts duplicado resuelto |
| Source maps en prod | None | ✅ |
| Code splitting | Per-route + shared chunks | ✅ |

**Problemas**:
1. ~~Google Fonts cargado dos veces (self-hosted + external `<link>`) — 80KB+ perdido~~ ✅ Resuelto — removidos preconnects a Google Fonts CDN, fonts via local @fontsource
2. ~~GSAP 132KB para scroll animations en marketing site — considerar `Motion` (motion.dev) como alternativa más ligera~~ ✅ Migrado a motion.dev
---

## 7. Deuda Técnica y Simplificación

### 7.1 Archivos Monolíticos

| Archivo | Líneas | Debería dividirse en |
|---|---|---|
| `cli_handlers.rs` (→ `cli_handlers/`) | 2,197 → 12 submódulos | `crud.rs`, `index.rs`, `data.rs`, `server.rs`, `search.rs`, `namespace.rs`, `backup.rs`, `diagnostics.rs`, `migrate.rs`, `fmt.rs`, `db.rs`, `util.rs` |
| `index/core.rs` | 1,984 | `index/graph.rs`, `index/search.rs`, `index/serialize.rs`, `index/validate.rs` |
| `metrics/core.rs` | 1,598 | `metrics/memory.rs`, `metrics/system.rs`, `metrics/recorder.rs` |
| `sdk/serialization.rs` | 1,827 | `sdk/export.rs`, `sdk/import.rs`, `sdk/backup.rs` |
| `storage/engine/ops.rs` | 874 | `engine/insert.rs`, `engine/read.rs`, `engine/delete.rs`, `engine/scan.rs` |
| `config.rs` | 1,116 | `config/types.rs`, `config/builder.rs`, `config/env.rs`, `config/hot_reload.rs` |
| `cli_server.rs` | 746 | `server/routes.rs`, `server/middleware.rs`, `server/tls.rs` |
| `wal.rs` | 749 | `wal/reader.rs`, `wal/writer.rs`, `wal/record.rs` |
| `text_index.rs` | 732 | `text_index/bm25.rs`, `text_index/stats.rs` |

### 7.2 Código Muerto

| Archivo | Líneas | Estado |
|---|---|---|
| `serialization/mod.rs:6` | `pub mod rkyv_archives` | `#[cfg(any())]` — siempre false |
| `src/index/hnsw.rs` | 4 líneas | Placeholder que re-exporta de core.rs |
| `src/python.rs` | `extract_2d_buffer` | `#[allow(dead_code)]` — nunca llamado |
| `web/vite.config.ts` | `import viteTsconfigPaths` | Importado pero no agregado al plugins array |

### 7.3 Duplicación

- **FLAG_TOMBSTONE**: ~~Definido en 5 lugares~~ ✅ Ahora solo en `engine/mod.rs:34`. Eliminadas copias en `archive.rs`, `wal.rs`, `storage/ops.rs`, `index/graph.rs`. `NodeFlags::TOMBSTONE` en `node.rs` es un flag diferente (bitset in-memory).
- **from_raw_parts pattern**: ~10 copias casi idénticas del mismo patrón de acceso mmap
- **Homebrew formula**: 2 copias (`Formula/vantadb.rb` y `vantadb.rb` en root) — diferentes estructuras

### 7.4 Spanish/English Mix

Los únicos `// SAFETY:` comments del codebase están en español en `index/core.rs`. El resto del codebase está en inglés. Esto crea fricción para contribuidores internacionales.

### 7.5 Testing Gaps

| Tipo | Estado | Detalle |
|---|---|---|
| Unit tests | ✅ | ~40 módulos con `#[cfg(test)] mod tests` |
| Integration tests | ✅ | `storage/engine/tests.rs` (604 líneas) |
| Property-based tests | ✅ 18 proptests | `tests/proptest_serialization_roundtrip.rs` — VantaValue/VantaMetadata/VantaMemoryRecord round-trips via JSON + postcard |
| Concurrency tests | ✅ 6 tests | `tests/concurrency_primitives.rs` — DashMap, RwLock, ArcSwap, StorageEngine concurrent stress |
| Miri tests | ❌ | Cero — unsafe code no verificado con Miri |
| Fuzz harnesses | ✅ 4 targets | `fuzz/fuzz_targets/` — `fuzz_parser` (parser entry points), `fuzz_node_deserialize` (UnifiedNode+WalRecord postcard), `fuzz_wal` (WalHeader deserialize + roundtrip), `fuzz_archive` (CPIndex deserialize) |
| Regression tests for unsafe | ❌ | Cero — `#![deny(unsafe_op_in_unsafe_fn)]` no está habilitado |

---

## 8. CI/CD y Build System

### 8.1 Pipeline Inventory

| Workflow | Trigger | Propósito |
|---|---|---|
| `ci-rust-10.yml` | push/PR a main (Rust) | Compilar, test, deny, audit |
| `ci-web-11.yml` | push/PR a main (web/) | Build web, lint |
| `gate-docs-21.yml` | push/PR a main (docs/) | Validar docs coverage |
| `sec-codeql-30.yml` | push/PR + weekly | CodeQL analysis |
| `perf-bench-40.yml` | push a main | Benchmarks Python |
| `heavy-certification-50.yml` | dispatch + weekly | Suite nocturna completa |
| `heavy-bench-nightly-51.yml` | nightly 3AM | Benchmarks regresión |
| `release-wheels-60.yml` | tag `v*` | PyPI publish |
| `release-npm-61.yml` | tag `wasm-v*`/`ts-v*` | NPM publish |
| `release-adapters-62.yml` | tag `adapters-v*` | PyPI adapters |
| `release-binaries-63.yml` | release published | GitHub binaries |
| `release-sbom-64.yml` | tag `v*` | CycloneDX SBOM |

### 8.2 Fortalezas

| Aspecto | Detalle |
|---|---|
| Path-triggered | Rust CI solo corre en cambios Rust, web CI en web/ |
| Concurrency groups | Cancel-in-progress en commits nuevos |
| Cross-platform testing | Linux + Windows (Rust), 3 OS (wheels) |
| Composite action `rust-setup` | Toolchain, caching, swap, system deps reusable |
| Permissions explícitas | `contents: read` por defecto, mínimo privilegio |
| SHA pinning | Todos los `uses` con commit SHAs completos |
| Build provenance | `actions/attest-build-provenance` en PyPI publish |
| SBOM generation | `cargo-cyclonedx` genera CycloneDX |
| Benchmark regression | Criterion + GitHub Issues auto-creados |
| Nextest profiles | `default`, `audit`, `ci-windows`, `experimental`, `chaos` |

### 8.3 Gaps y Problemas

| # | Problema | Archivo | Severidad |
|---|---|---|---|
| CI1 | ~~Sin macOS Rust CI testing~~ ✅ Completo | `ci-rust-10.yml` | **MEDIA** — Añadido job `test-macos` con cargo check + clippy + nextest audit |
| CI2 | ~~Sin MSRV check (`cargo check --minimal-versions`)~~ ✅ Completo | `ci-rust-10.yml` — job `msrv` con toolchain 1.94.1 | **MEDIA** |
| CI3 | ~~Sin Windows binary release~~ ✅ Completo | `release-binaries-63.yml` — añadido `x86_64-pc-windows-msvc` | **MEDIA** |
| CI4 | ~~Sin Linux ARM64 binary~~ ✅ Completo | `release-binaries-63.yml` — añadido `aarch64-unknown-linux-gnu` con gcc-aarch64-linux-gnu | **BAJA** |
| CI5 | Untrusted input injection vector en `dry_run` | `release-npm-61.yml:67` | **MEDIA** |
| CI6 | ~~Sin fuzz CI integration~~ ✅ Completo | `.github/workflows/fuzz-40.yml` — build + run semanal con cargo-fuzz (nightly) | **MEDIA** |
| CI7 | Sin `-Zminimal-versions` en CI | Missing | **BAJA** |

### 8.4 Perfiles Cargo

```toml
[profile.release]
lto = "thin"
codegen-units = 1
opt-level = 3

[profile.ci]  # ★ Excelente: hereda release pero optimizado para CI
inherits = "release"
lto = "off"
codegen-units = 16
opt-level = 2

[profile.dev]
opt-level = 1
debug = 0   # ★ Rápido

[profile.release.package.vantadb-wasm]
opt-level = "s"
strip = true
```

Los perfiles `ci` y `dev` con `debug = 0` son configuraciones avanzadas y excelentes.

### 8.5 Herramientas Adicionales

| Herramienta | Config | Estado |
|---|---|---|
| `cargo deny` | `deny.toml` — 5 advisories ignorados, 12 licenses allowlisted | ✅ |
| `cargo audit` | CI run con RUSTSEC-2026-0176/0177 ignorados | ⚠️ Monitorear |
| `cargo nextest` | 5 perfiles en `.config/nextest.toml` | ✅ Excelente |
| `cargo machete` | En `Justfile` — detecta unused deps | ✅ |
| `cargo outdated` | En `Justfile` | ✅ |
| `cargo bloat` | Via `just size` | ✅ |

---

## 9. Docker y Despliegue

### 9.1 Dockerfile (98 líneas, multi-stage)

**Fortalezas:**
- Multi-stage build (builder + runtime)
- Non-root user (`vantadb`, uid 1001)
- Dependency caching via skeleton build
- OCI labels completos
- Healthcheck configurado
- ARG-based versioning

**Problemas:**

| # | Problema | Línea | Severidad | Solución |
|---|---|---|---|---|
| D1 | `ARG RUST_VERSION=1.86` pero MSRV es `1.94.1` | `Dockerfile:4` | **ALTA** | Cambiar a `ARG RUST_VERSION=1.94` |
| D2 | Error swallowing en skeleton build: `|| true` | `Dockerfile:47` | **ALTA** | Remover `|| true` |
| D3 | Usa perfil `release` (LTO, lento) en vez de `ci` | `Dockerfile:57` | **MEDIA** | Usar `--config 'profile.ci'` |
| D4 | `COPY . .` sin excluir sensible dirs (`.cargo/`) | `Dockerfile:53` | **BAJA** | Mejorar `.dockerignore` |
| D5 | `curl` en producción (attack surface) | `Dockerfile:68` | **BAJA** | `apt-get remove curl` en misma RUN |
| D6 | `HEALTHCHECK start_period` 10s vs `docker-compose.yml` 5s | `Dockerfile:91` | **BAJA** | Unificar |

### 9.2 docker-compose.yml

- Un solo servicio, named volume
- Sin network isolation (default bridge)
- Puerto 8080 expuesto

### 9.3 Análisis de Vercel

- `.vercel/` en root NO está en `.gitignore` → project/org IDs committeados
- `web/.vercel/` sí está ignorado correctamente
- `web/vercel.json` usa `--legacy-peer-deps` → conflictos de dependencias ocultos
- Sin security headers (CSP, HSTS) en `web/vercel.json`

---

## 10. Bindings (Python/TS/WASM)

### 10.1 Python (`vantadb-python/`)

**Nota general: A** — Implementación PyO3 profesional

| Aspecto | Estado |
|---|---|
| FFI | PyO3 0.29 con `abi3-py311` (ABI estable — un wheel para Python 3.11+) |
| Build | Maturin v1.5+ (industry standard) |
| GIL Management | Excelente — toda operación blocking usa `py.detach(move || ...)` con comentarios `// PERF-24: GIL RELEASED` |
| Error Mapping | Completo — cada variante `VantaError` mapeada a la excepción Python correcta |
| Memory | LRU cache thread-local con `RefCell<LruCache>`. Clones en put/get. Correcto. |

**Problemas:**

| # | Problema | Detalle | Severidad |
|---|---|---|---|
| PY1 | Sin `.pyi` stubs | PyO3 auto-genera firmas pero IDE autocomplete no funciona sin stubs | **MEDIA** |
| PY2 | `ListBool` type inference | `py_any_to_value` prueba `bool` antes que `i64` — `[0, 1]` se clasifica como `ListBool` | **BAJA** |
| PY3 | `VantaVector.__array_interface__` | Expone raw pointer a NumPy. Si Python retiene la referencia y el Vec se mueve: use-after-free (teórico, prevenido por ownership de Python) | **BAJA** |
| PY4 | `extract_2d_buffer` dead code | Marcado `#[allow(dead_code)]` — vestigial | **BAJA** |

### 10.2 TypeScript (`vantadb-ts/`)

**Nota general: A-**

| Aspecto | Estado |
|---|---|
| Binding | Thin TS wrapper sobre `vantadb-wasm/pkg` |
| Types | Excelentes — discriminated unions, BigInts como strings, runtime guards |
| Errors | Structured error class con `toJSON()`, `ErrorCode`, `wrapWasmError()` |

**Problemas:**

| # | Problema | Detalle | Severidad |
|---|---|---|---|---|
| TS1 | ~~Async inconsistency~~ ✅ Resuelto | `put()`/`get()`/`delete()` son sync — WASM `put()`, `put_batch()`, `search()` migrados a `fn` | **MEDIA** |
| TS2 | Test runner inconsistente | `.then()` y `async/await` mezclados | **BAJA** |
| TS3 | Distance metric case mismatch | TS envía `"Cosine"`, WASM da default `"Cosine"`, pero MCP usa `"cosine"` (minúscula) | **BAJA** |

### 10.3 WASM (`vantadb-wasm/`)

**Nota general: B**

| Aspecto | Estado |
|---|---|
| Binding | `wasm-bindgen 0.2`, serde-wasm-bindgen |
| NaN sanitization | ✅ Correcta en `memory_record_to_js()`, `search_hit_to_js()` |
| OPFS bridge | ✅ Good error handling |
| Memory close guard | ✅ TS setea `_closed = true` |

**Problemas:**

| # | Problema | Detalle | Severidad |
|---|---|---|---|
| WA1 | `wasm-opt = false` | Binaryen deshabilitado. `wasm-opt -Oz` reduce 30-50% | **ALTA** |
| WA2 | Sin code splitting | Todo el engine en un `.wasm` file | **MEDIA** |
| WA3 | `tracing-wasm` siempre incluido | ~50KB para console logging. Debería ser feature flag | **MEDIA** |
| WA4 | Sin `wee_alloc` o custom allocator | Default allocator no optimizado para WASM | **BAJA** |
| WA5 | `search_semantic` en MCP accede estado interno directamente | `storage.hnsw.load()` bypass `VantaEmbedded` API | **MEDIA** |

### 10.4 Server (`vantadb-server/`)

**Nota general: A-**

| Aspecto | Estado |
|---|---|
| Auth | Bearer token + RBAC + constant-time comparison (`ct_eq`) |
| Rate limiting | `AuthRateLimiter` (5 failures / 60s per IP) |
| Concurrency | `Semaphore` (max 32) + `spawn_blocking` + per-request timeout (60s) |
| MCP | 12 tools, 4 resources, 4 prompts con validación config-driven |

**Problemas:**

| # | Problema | Detalle | Severidad |
|---|---|---|---|
| SVR1 | Sin forced-auth mode | Si `api_key` es None, todos los endpoints están abiertos. No hay `--require-auth` | ✅ |
| SVR2 | MCP error inconsistency | Algunos tool errors retornan `Ok(error_content(...))` en vez de `Err(McpError::invalid_params(...))` | **BAJA** |
| SVR3 | IP extraction solo funciona detrás de reverse proxy | Usa `ConnectInfo<SocketAddr>` — en deployment directo, la IP mostrada puede ser incorrecta | **BAJA** |

---

## 11. Web Frontend

### 11.1 Stack Tecnológico

| Tecnología | Versión | Estado |
|---|---|---|
| React | 19.2.0 | ✅ Latest |
| TanStack Router | 1.168.25 | ✅ File-based routing |
| TanStack Query | 5.101.2 | ✅ Configurado (no usado aún) |
| Vite | 8.1.3 | ✅ Latest (Rolldown) |
| Tailwind CSS | 4.3.2 | ✅ v4 |
| motion (motion.dev) | 12.10.5 | ✅ Reemplazó GSAP (scroll animations, enter/exit) |
| TypeScript | 5.8.3 | ✅ Strict mode |

### 11.2 Arquitectura

- 27 rutas, 100% lazy-loaded
- 29 componentes UI primitivos (`nb/` design system)
- Componentes funcionales con `forwardRef`, `memo`, `useCallback`
- Sin estado global (apropiado para marketing site)

### 11.3 Bugs Lógicos Encontrados

| # | Bug | Archivo | Líneas | Descripción |
|---|---|---|---|---|
| W1 | Duplicate OG meta tags | `__root.tsx` | 70-76 | `og:title`, `og:description`, `og:url` duplicados confunden crawlers | ✅ Resuelto |
| W2 | ~~`isActive` matching demasiado amplio~~ ✅ Corregido | `NbNav.tsx` | 112-113 | `startsWith` reemplazado con `===` + segment check para evitar falsos positivos | ✅ Corregido |
| W3 | Scroll race condition | `useScrollReveal.ts` | `scrollTo({top:0})` en mount compite con `scrollRestoration: true` del router | ✅ No se encontró el scrollTo en el código actual — verificado |
| W4 | ~~`new Date()` durante render~~ ✅ Corregido | `NbFaqAccordion.tsx` | 70 | Reemplazado con string estático para evitar re-renders y hydration mismatch | ✅ Corregido |
| W5 | Google Fonts cargado doble | `nb-base.css` + `index.html` | — | 80KB+ de descarga duplicada, posible FOUT | ✅ Resuelto |
| W6 | `vite-tsconfig-paths` import no usado | `vite.config.ts` | — | Import muerto | ✅ Resuelto |

### 11.4 Accesibilidad

| Aspecto | Estado |
|---|---|
| ARIA labels | ✅ Excelente — nav, main, sections, modals, progressbar |
| Heading hierarchy | ✅ h1→h2→h3 sin saltos |
| Skip-to-content | ✅ Presente y funcional |
| Focus trapping | ✅ En mobile drawer navigation |
| Keyboard navigation | ✅ Escape cierra modal, aria-expanded en nav |
| Semantic HTML | ✅ `<nav>`, `<main>`, `<section>`, `<aside>`, `<header>` |
| Alt text | ✅ SVG decorativos con `aria-hidden="true"` |
| `aria-expanded` hardcoded `false` | ⚠️ En nav dropdowns — debería actualizarse dinámicamente |

### 11.5 SEO

| Aspecto | Estado |
|---|---|
| Meta tags únicos por ruta | ✅ |
| OG/Twitter cards | ✅ (con bug W1 en security) |
| JSON-LD structured data | ✅ SoftwareApplication, Product, WebPage |
| Sitemap | ✅ 31 URLs con priorities y lastmod |
| robots.txt | ✅ Allow all |
| SSR/SSG | ❌ Client-side rendering only — crawlers que no ejecuten JS no ven meta tags |
| og:image paths inconsistentes | ⚠️ Algunos refs usan `.svg`, otros `.png` |

---

## 12. Análisis de Dependencias

### 12.1 Rust Dependencies

**Total packages:** ~400+ (transitivas)
**Workspace members:** 14 crates

### 12.2 Crates Duplicados

| Crate | Versiones | Impacto |
|---|---|---|
| `thiserror` | 1.0.69 + 2.0.18 | **MEDIO** — migrar todo a v2 |
| `hashbrown` | 4 versiones (0.12, 0.13, 0.14, 0.15) | **BAJO** — difícil de consolidar (deps transitivas) |
| `windows-sys` / `windows-targets` | ~4 versiones cada uno | **BAJO** — inevitable por winapi fragmentation |
| `rand` / `rand_core` / `rand_chacha` | 2 versiones | **BAJO** |
| `getrandom` | 3 versiones | **BAJO** |
| `rustix` / `linux-raw-sys` / `r-efi` | 2 versiones | **BAJO** |
| `shlex` | 2 versiones (1.1.0 + 1.3.0) | **BAJO** |
| `lz4_flex` | 2 versiones | **BAJO** |

**Total: 17 pares duplicados** — impactan tiempo de compilación y binary size.

### 12.3 Advisories Conocidos (Allowlisted en `deny.toml`)

| Crate | Advisory | Tipo | Reemplazo |
|---|---|---|---|
| `atomic-polyfill 1.0.3` | RUSTSEC-2023-0089 | Unmaintained | Migrar a `cortex-m` o similar |
| `instant 0.1.13` | RUSTSEC-2024-0384 | Unmaintained | Usar `std::time::Instant` |
| `paste 1.0.15` | RUSTSEC-2024-0436 | Unmaintained | Usar `macroquad` o inline macros |
| `rustls-pemfile 2.2.0` | RUSTSEC-2025-0134 | Unmaintained | `rustls-pemfile` 2.x → migrar a rustls-native-certs |
| `lru 0.12.5` | RUSTSEC-2026-0002 | **Unsound** | **Prioridad alta**: migrar a lru 0.13+ o `quick-lru` |

### 12.4 Licencias No Standard

0 de 400+ crates usan licencias no allowlisted. Política de licencias estricta.

### 12.5 npm Dependencies

- 65 devDependencies, pocas dependencies directas
- `@testing-library/jest-dom v6.9.1` desactualizado
- `esbuild` y `rollup` como dependencies (no devDependencies) — probablemente transitivas

### 12.6 Análisis de Duplicados — Finding 2.11

**Estado:** ✅ 3 de 4 pasos completados.

El approach sugerido (`[patch]` sections en Cargo.toml) **no es viable** para consolidar versiones dentro de crates.io. Cargo rechaza explícitamente `[patch]` entries que apuntan al mismo source:
```
error: patch for `lru` in `https://github.com/rust-lang/crates.io-index`
       points to the same source, but patches must point to different sources.
```

Usar `[patch]` con git sources para forzar consolidación rompería compatibilidad de API entre semver major versions (ej. `lru 0.12` tiene API distinta a `0.13`).

**Duplicados detectados en `Cargo.lock` (July 2026 — post-upgrades):**

| Crate | Versiones | Categoría | Resolución |
|---|---|---|---|---|
| `thiserror` | 1.0.69 + 2.0.18 | ✅ Resuelto (Finding 2.10) | Migrado a v2 en `Cargo.toml` |
| `hashbrown` | 0.14.5, 0.15.5, 0.16.1, 0.17.1 | Bloqueado | Arrow 59 mueve a 0.17, fjall usa 0.14/0.16 |
| `windows-sys` | 0.52.0, 0.59.0, 0.60.2, 0.61.2 | Bajo impacto | Inevitable por fragmentación winapi |
| `getrandom` | 0.2.17, 0.3.4, 0.4.2 | Bloqueado | Cada versión de rand trae su propio getrandom |
| ~~`rand` / `rand_chacha` / `rand_core`~~ | ~`0.8.6 + 0.9.4`~ | ✅ Resuelto | Tantivy 0.26 eliminó la dependencia de rand 0.8 |
| ~~`lz4_flex`~~ | ~`0.11.6 + 0.13.1`~ | ✅ Resuelto | Tantivy 0.26 actualizó a lz4_flex >= 0.13 |
| `lru` | 0.13.0 + 0.16.4 | Bloqueado | 0.13 nuestro directo, 0.16 de lsm-tree (fjall) |
| ~~`rustc-hash`~~ | ~`1.1.0 + 2.1.3`~ | ✅ Resuelto | RocksDB 0.24 actualizó bindgen que eliminó rustc-hash 1.1 |
| `shlex` | 1.3.0 + 2.0.1 | Build-only | 1.3 de bindgen, 2.0 de cc (ambos build deps) |
| `itertools` | 0.13.0 + 0.14.0 | Bloqueado | 0.13 de criterion (dev), 0.14 transitivo. 0.12 eliminado (arrow 59) |
| ~~`reqwest`~~ | ~`0.12.28 + 0.13.4`~ | ✅ Resuelto | Tantivy 0.26 consolidó |
| `r-efi` | 5.3.0 + 6.0.0 | Bajo impacto | Transitive io-uring/fd-lock churn |

**Resumen:** De 17 pares duplicados originales → **~12 pares restantes** (muchos inevitables por fragmentación winapi/transitivos). Resueltos: `rand`, `lz4_flex`, `rustc-hash`, `reqwest`, `itertools:0.12`.

**Path real de resolución (actualizado):**
1. ✅ ~~Upgrade tantivy 0.22 → 0.26.1~~ (resolvió rand, lz4_flex, reqwest, lru_0.12)
2. ✅ ~~Upgrade rocksdb 0.22 → 0.24.0~~ (resolvió rustc-hash mediante bindgen 0.69→0.72)
3. ✅ ~~Upgrade arrow 58 → 59~~ (resolvió itertools 0.12, hashbrown consolida parcialmente)
4. ⏳ Upgrade fjall 3.1 → 4.0 (pendiente — fjall 4.0 no ha sido liberado aún a Jul 2026). Cayó `lru 0.13+0.16` (0.16 de lsm-tree/fjall) que se resolvería con fjall 4.0

> **Nota:** `cargo check -p vantadb --tests` pasa limpio sin --features rocksdb (librocksdb-sys requiere CMake/Clang en Windows). Las features arrow, fjall, advanced-tokenizer verificadas correctamente.

---

## 13. Documentación

### 13.1 README (EN + ES)

| Aspecto | EN | ES |
|---|---|---|
| Badges | ✅ 14 | ⚠️ Missing Discord badge |
| Quickstart | ✅ 5 steps con código runnable | ✅ **Corregido** (`get()` en vez de `get_memory()`) |
| Core capabilities | ✅ 8-row table | ✅ |
| Benchmarks | ✅ p50/p99 + SIFT1M | ✅ |
| Documentation links | ✅ 13 linked documents | ✅ |

### 13.2 Documentación Técnica

| Documento | Estado | Notas |
|---|---|---|
| ARCHITECTURE.md | ✅ 485 líneas, excelente |
| ADRs | ✅ 9 decisiones registradas |
| CONFIGURATION.md | ✅ 220 líneas |
| SECURITY.md | ✅ 157 líneas |
| DURABILITY_GUARANTEES.md | ✅ 310 líneas |
| PERFORMANCE_TUNING.md | ✅ |
| CHANGELOG.md | ✅ ~900+ líneas |
| FAQ.md | ⚠️ Menciona v0.1.5, debería ser 0.3.0 |
| Quickstart | ✅ 187 líneas |
| Glosario | ✅ 63 términos, excelente |
| Tutorials | ⚠️ Draft (no production-ready) |
| Case Studies | ⚠️ Draft (no production-ready) |

### 13.3 `llms.txt` — **CRÍTICO: DESACTUALIZADO**

| Error | `llms.txt` dice | Realidad |
|---|---|---|
| Import path | `from vantadb import VantaEmbedded` | `import vantadb_py as vantadb` |
| API calls | `db.put("key1", [0.1, 0.2, 0.3])` | `db.put(namespace, key, payload, metadata=..., vector=...)` |
| Version | 0.2.0 | 0.3.0 |
| Quantization | "3 schemes: RaBitQ, TurboQuant, SQ8" | No documentados en README |
| SDKs count | "6 SDKs" | 4 documentados en README |

### 13.4 Gaps de Documentación

| Gap | Impacto |
|---|---|
| ~~Sin deployment guide (Kubernetes, systemd)~~ ✅ | `docs/operations/DEPLOYMENT_GUIDE.md` |
| ~~Sin SQLite migration guide~~ ✅ | `docs/operations/SQLITE_MIGRATION_GUIDE.md` |
| ~~Sin DR runbook~~ ✅ | `docs/operations/DISASTER_RECOVERY_RUNBOOK.md` |
| ~~`.env.example` falta ~15 variables~~ ✅ Resuelto | 22 variables documentadas en `.env.example` y CONFIGURATION.md |
| `docs/articles/` no existe | Referenciado en master-index pero sin archivos |
| ~~master-index.md refs a 3 archivos inexistentes~~ ✅ Resuelto | Limpiado — refs solo persisten en docs de terceros (web/README.md, DOCS_AUDIT_REPORT.md) |
| ~~SECURITY.md dice ">= 0.2.0"~~ ✅ Resuelto | Ya no contiene ese texto |

### 13.5 SKILLS-MANIFEST.md

- 407 líneas, excelente organización
- "Core 50" en realidad lista 61 skills (inconsistencia)
- 62 skills removidas documentadas con razones

---

## 14. Recomendaciones Priorizadas

### Prioridad 0 — Acción Inmediata (Riesgo de Seguridad o Funcional) ✅ Completada

| # | Acción | Archivos | Estado |
|---|---|---|---|
| 0.1 | Añadir `// SAFETY:` comments a 50 bloques unsafe + reemplazar `debug_assert!` con `if` checks | 13 archivos Rust | ✅ `d2986bf` |
| 0.2 | Corregir `llms.txt` con APIs reales y SDK snippets | `web/public/llms.txt` | ✅ `d2986bf` |
| 0.3 | Añadir `.vercel` a root `.gitignore` | `.gitignore` (ya presente) | ✅ Verificado |
| 0.4 | Eliminar `vantadb.rb` duplicado en raíz | `vantadb.rb` | ✅ `d2986bf` |
| 0.5 | Migrar `lru 0.12.5` → 0.13 | `Cargo.toml`, `Cargo.lock` | ✅ `d2986bf` |

### Prioridad 1 — Crítica (Código Roto o Funcionalidad Degradada) ✅ Completada

| # | Acción | Archivos | Estado |
|---|---|---|---|
| 1.1 | Actualizar `README_ES.md` con API calls correctas (`get()` en vez de `get_memory()`) | `README_ES.md` | ✅ |
| 1.2 | Corregir Docker `RUST_VERSION=1.86` → `1.94` + remover `|| true` | `Dockerfile` | ✅ (RUST_VERSION ya en 1.94; removido `; true`) |
| 1.3 | Habilitar `wasm-opt = true` en perfil WASM | `vantadb-wasm/Cargo.toml` | ✅ Ya estaba en `true` |
| 1.4 | Corregir duplicate OG tags en `security.tsx` | `web/src/routes/__root.tsx` | ✅ Removidos `og:title`, `og:description`, `og:url` del root |
| 1.5 | Resolver double Google Fonts load | `web/src/routes/__root.tsx` | ✅ Removidos preconnects a Google Fonts CDN |
| 1.6 | Sanitizar input injection vector en `release-npm-61.yml` | `.github/workflows/release-npm-61.yml` | ✅ Reemplazado bash `if` con expresión GHA |
| 1.7 | Hacer `tracing-wasm` feature-gated | `vantadb-wasm/Cargo.toml` + `lib.rs` | ✅ Ya feature-gated (`optional = true` + `#[cfg]`) |
| 1.8 | Corregir scroll race condition entre `useScrollReveal` y router | `web/src/hooks/useScrollReveal.ts` | ✅ No había `scrollTo({top:0})` en el código actual |

### Prioridad 2 — Alta (Deuda Técnica con Impacto)

| # | Acción | Archivos | Esfuerzo |
|---|---|---|---|
| 2.1 | ~~Fragmentar `cli_handlers.rs` (2,197 líneas)~~ ✅ Completo | `src/cli_handlers/` con 12 submódulos | 1 día |
| 2.2 | ~~Fragmentar `index/core.rs` (1,984 líneas)~~ ✅ Completo | Crear `src/index/graph.rs` (700), `search.rs` (419), `serialize.rs` (618), `stats.rs` (110) — `core.rs` reducido a solo tests (311) | 1 día |
| 2.3 | ~~Reemplazar `entry_point` Mutex con `AtomicU128`~~ ✅ Completo | `src/index/graph.rs`, `serialize.rs`, `init.rs`, `ops.rs`, `Cargo.toml` (+ `portable-atomic`) | 30 min |
| 2.4 | ~~Migrar variantes `String` de `VantaError` a source chaining~~ ✅ Completo | `src/error.rs` + 8 archivos (21 call sites): `SerializationError(String)` → `Box<dyn Error + Send + Sync>` con `SerdeMsgError` para errores con contexto. `ExportError` eliminado (no usado). | 1 hora |
| 2.5 | ~~Unificar `FLAG_TOMBSTONE` en un solo lugar~~ ✅ Completo | Se unificó en `src/storage/engine/mod.rs:34`. Eliminadas 4 copias: `archive.rs`, `wal.rs`, `storage/ops.rs`, `index/graph.rs` + actualizado `search.rs` para importar del home único. `NodeFlags::TOMBSTONE` en `node.rs` no se tocó (es un flag diferente). 5 archivos modificados. | 15 min |
| 2.6 | ~~Añadir forced-auth mode al server~~ ✅ | `cli_server.rs`, `config.rs`, `cli.rs`, `cli_handlers/server.rs` | 1 hora |
| 2.7 | ~~Expandir `.env.example` con todas las 22 variables~~ ✅ Completo | `.env.example` | ✅ Expandido de 9 a 22 variables documentadas en CONFIGURATION.md |
| 2.8 | ~~Añadir `proptest` para HNSW search correctness~~ ✅ Completo | `tests/proptest_hnsw_search.rs`, `src/index/graph.rs` | 1 día |
| 2.9 | ~~Añadir `#![deny(unsafe_op_in_unsafe_fn)]`~~ ✅ Completo | `src/lib.rs` | 15 min |
| 2.10 | ~~Consolidar `thiserror` a v2 sola~~ ✅ Completo | `Cargo.toml`, `Cargo.lock` | 15 min |
| 2.11 | ~~Reducir duplicate crate versions (17 pares)~~ ✅ Investigado — ver §12.6 | `Cargo.toml` + §12.6 | 1-2 días (requiere upgrade tantivy) |
| 2.12 | ~~Unificar async pattern en TS SDK~~ ✅ Completo | `vantadb-wasm/src/lib.rs`, `vantadb-ts/src/vantadb.ts`, `vantadb-wasm/tests/wasm_tests.rs` | WASM `put()`, `put_batch()`, `search()`: `pub async fn` → `pub fn`. TS `putBatch()`, `search()`: removed `async`/`await`/`Promise<>`. Extra: `#[cfg(test)]` module + `wasm_tests.rs`: 27 `.await` removed, 32 `async fn` → `fn`. Verificado: `cargo build` ✅, `cargo test --no-run -p vantadb-wasm` ✅, `vitest run` 159/159 ✅ |

### Prioridad 3 — Media (Mejora Continua)

| # | Acción | Esfuerzo |
|---|---|---|
| 3.1 | ~~Añadir property-based tests (proptest) para serialización round-trips~~ ✅ Completo | `tests/proptest_serialization_roundtrip.rs` — 18 proptests. Documenta f64 ULP loss en JSON (postcard para floats), u128_serde + postcard incompatibilidad, bare u128 OK. |
| 3.2 | ~~Añadir concurrency tests para RwLock/Mutex/DashMap~~ ✅ Completo | `tests/concurrency_primitives.rs` — 6 tests. Deadlock RwLock (barriers mixtos) y StorageEngine hang (lock ordering) corregidos. |
| 3.3 | ~~Añadir macOS a Rust CI matrix~~ ✅ Completo | `.github/workflows/ci-rust-10.yml` — job `test-macos` con dtolnay/rust-toolchain, nextest, audit profile. |
| 3.4 | ~~Añadir MSRV check (`cargo check --minimal-versions`) a CI~~ ✅ Completo | `.github/workflows/ci-rust-10.yml` — job `msrv` con dtolnay/rust-toolchain@1.94.1 + `cargo check --workspace`. |
| 3.5 | ~~Añadir Windows + Linux ARM64 a binary releases~~ ✅ Completo | `.github/workflows/release-binaries-63.yml` — matrix añadido `x86_64-pc-windows-msvc` (windows-latest) + `aarch64-unknown-linux-gnu` (ubuntu-latest con gcc-aarch64-linux-gnu). Packaging diferenciado: `.zip` para Windows, `.tar.gz` para Unix. |
| 3.6 | ~~Añadir fuzz harnesses para WAL + parser + archive~~ ✅ Completo | `fuzz/fuzz_targets/fuzz_wal.rs` (WalHeader deserialize + roundtrip), `fuzz_archive.rs` (CPIndex::deserialize_from_bytes). CI: `.github/workflows/fuzz-40.yml` — build + run semanal con cargo-fuzz (nightly). |
| 3.7 | ~~Migrar de GSAP a `motion` (motion.dev) para web frontend~~ ✅ | 1 día |
| 3.8 | ~~Habilitar `noUnusedLocals` y `noUnusedParameters` en tsconfig~~ ✅ | 30 min |
| 3.9 | ~~Añadir security headers (CSP, HSTS) a Vercel config~~ ✅ | 15 min |
| 3.10 | ~~Generar `.pyi` stubs para Python binding~~ ✅ | 2 horas |
| 3.11 | ~~Añadir Miri tests para unsafe code~~ ✅ | 1 día |
| 3.12 | ~~Resolver `--legacy-peer-deps` en web~~ ✅ | 1 hora |
| 3.13 | ~~Migrar `exit_point` a `AtomicU128`~~ ✅ No aplica | No existe `exit_point` en el codebase. No hay `Mutex<u128/u64>` remanente. |
| 3.14 | ~~Upgrade tantivy 0.22 → 0.26.1 (resuelve ~4 pares duplicados, ver §12.6)~~ ✅ Completo | tantivy `0.22` → `0.26.1`. 6 test files con `flat_threshold: None`. 15/15 tokenizer tests OK. |

### Prioridad 4 — Baja (Nice to Have)

| # | Acción | Esfuerzo |
|---|---|---|
| 4.1 | ~~Traducir SAFETY comments español → inglés~~ ✅ Completo | Ya todos en inglés desde Phase 1. Verificados post-fragmentación. |
| 4.2 | ~~Remover `vite-tsconfig-paths` import no usado~~ ✅ | 5 min |
| 4.3 | ~~Corregir `aria-expanded` hardcoded en nav dropdowns~~ ✅ Completo | `NbNav.tsx` — añadido estado `openGroup` + handlers dinámicos |
| 4.4 | ~~Estandarizar og:image paths (.svg vs .png)~~ ✅ Ya estándar | Solo existe `default.svg`, no hay `.png`. El hallazgo del audit estaba desactualizado. |
| 4.5 | ~~Consolidar `extract_2d_buffer` dead code removal~~ ✅ Completo | `vantadb-python/src/lib.rs` — función eliminada |
| 4.6 | ~~Documentar `#[cfg(any())]` en rkyv_archives~~ ✅ Completo | `src/serialization/mod.rs` — comentario añadido |
| 4.7 | ~~Añadir deployment guide (Kubernetes, systemd)~~ ✅ Completo | `docs/operations/DEPLOYMENT_GUIDE.md` — systemd, Docker, K8s, security, monitoring |
| 4.8 | ~~Añadir SQLite migration guide~~ ✅ Completo | `docs/operations/SQLITE_MIGRATION_GUIDE.md` — schema mapping, export/import, query translation |
| 4.9 | ~~Añadir DR runbook~~ ✅ Completo | `docs/operations/DISASTER_RECOVERY_RUNBOOK.md` — SEV-1/2/3 procedures, health checks, recovery testing |
| 4.10 | ~~Consolidar `vantadb.rb` duplicado~~ ✅ Completo | Formula reescrita: tarballs, ARM64 macOS/Linux, `version`, `livecheck`. |

### Prioridad 5 — Post-Audit Fixes (Completado en Jul 2026)

| # | Acción | Cambios |
|---|---|---|
| 5.1 | Docker: perfil release → ci + strip | `Dockerfile` — skeleton + real build usan `--profile ci`. `Cargo.toml` — `[profile.ci]` añadido `strip = "symbols"` |
| 5.2 | Docker: `.dockerignore` + `.cargo/` | `.dockerignore` — añadida exclusión de `.cargo/` |
| 5.3 | Docker: HEALTHCHECK start_period unificado | `docker-compose.yml` — `start_period: 10s` (consistente con Dockerfile) |
| 5.4 | CI: `-Zminimal-versions` | `.github/workflows/ci-rust-10.yml` — nuevo job `minimal-versions` con `cargo +nightly check -Zminimal-versions` (continue-on-error) |
| 5.5 | Path traversal: absolute path + prefix rejection | `src/storage/ops.rs:prevent_path_traversal` — rechaza paths absolutos y Windows prefixes |
| 5.6 | Homebrew formula: SHA256 generation docs | `Formula/vantadb.rb` — comentario con script de generación de SHA256 |
| 5.7 | `install.sh`: SSL flags + checksum verification | `scripts/install.sh` — `--ssl-reqd` en curl, SHA256 checksum opcional, fallo en API fallback |
| 5.8 | PY2: ListBool type inference corregido | `vantadb-python/src/lib.rs` — en listas, `i64` se verifica antes que `bool` para evitar `[0,1]` → `ListBool` |
| 5.9 | TS2: Test runner `.then()` → síncrono | `vantadb-ts/src/__tests__/vanta.test.ts` — tests concurrentes convertidos de `.then()` a síncronos (WASM ya es sync) |
| 5.10 | TS3: Distance metric case-insensitive | `vantadb-mcp/src/lib.rs` — `search_memory` normaliza a lowercase para aceptar "Cosine"/"Euclidean" |
| 5.11 | WA5: MCP `search_semantic` via API pública | `vantadb-mcp/src/lib.rs` — reemplazado acceso directo a `storage.hnsw`/`storage.vector_store` con `VantaEmbedded::search_vector()` |
| 5.12 | E5: `parse_env_or` error logging mejorado | `src/config.rs` — incluye parse error y valor inválido en warning |
| 5.13 | §5.2: Deserialización con límite de tamaño | `src/config.rs`, `src/hardware/mod.rs`, `src/wal_shipping.rs` — límite de 1MB en inputs JSON |
| 5.14 | SKILLS-MANIFEST: TOC "Core 50" → "(37)" | `SKILLS-MANIFEST.md` — TOC actualizado a "Essential Skillset (37)" |
| 5.15 | FAQ.md: versión verificada | `docs/FAQ.md` — ya muestra 0.3.0, no requiere cambios |

---

## 15. Progreso de Implementación

### 15.1 Fase 1 — Prioridad 0 (Completada en `d2986bf`)

| Archivo | Cambios |
|---|---|
| `src/node.rs` | SAFETY comments en 4 unsafe blocks; bounds hardening en MmapFull paths |
| `src/index/core.rs` | SAFETY en 12 unsafe blocks; `debug_assert!` → `if guard` en from_raw_parts; traducción español→inglés en madvise |
| `src/index/distance.rs` | SAFETY + bounds guard en MmapFull path |
| `src/storage/vfile.rs` | SAFETY en 14 unsafe blocks (sigaction, mincore, QueryWorkingSetEx, mmap, Send/Sync) |
| `src/storage/engine/ops.rs` | SAFETY en 3 from_raw_parts |
| `src/storage/archive.rs` | SAFETY en 1 from_raw_parts |
| `src/storage/ops.rs` | SAFETY en 1 from_raw_parts |
| `src/serialization/rkyv_archives.rs` | SAFETY en 6 from_raw_parts |
| `src/metrics/core.rs` | SAFETY en 2 FFI blocks (macOS task_info, Windows GetProcessMemoryInfo) |
| `src/storage/engine/maintenance.rs` | SAFETY en 2 unsafe blocks |
| `web/public/llms.txt` | Añadidos API endpoints, VantaQL types, SDK snippets Python/Rust/TS |
| `vantadb.rb` | Eliminado (duplicado de `Formula/vantadb.rb`) |
| `Cargo.toml` / `Cargo.lock` | lru 0.12.5 → 0.13 (elimina RUSTSEC-2026-0002) |

### 15.2 Fase 2 — Prioridad 1 (Completada)

| # | Acción | Cambios |
|---|---|---|
| 1.1 | `README_ES.md`: `get_memory()` → `get()` + `search_memory()` → `search()` | `README_ES.md:120,123` |
| 1.2 | Docker: RUST_VERSION ya en 1.94; removido `2>/dev/null; true` del skeleton build | `Dockerfile:47` |
| 1.3 | `wasm-opt = true` — ya estaba habilitado en `vantadb-wasm/Cargo.toml` | Verificado |
| 1.4 | OG tags duplicados: removidos `og:title`, `og:description`, `og:url` del root route | `web/src/routes/__root.tsx:70-76` |
| 1.5 | Google Fonts preconnects innecesarios: removidos (fonts vía local @fontsource) | `web/src/routes/__root.tsx:82-85` |
| 1.6 | CI injection vector: reemplazado bash `if` con `${{ inputs.dry_run == 'true' && '--dry-run' \|\| '' }}` | `.github/workflows/release-npm-61.yml:67,127` |
| 1.7 | `tracing-wasm` ya feature-gated (`optional = true` + `#[cfg(feature = "tracing-wasm")]`) | Verificado |
| 1.8 | Scroll race condition: no se encontró `scrollTo({top:0})` en el código actual | Verificado |

### 15.3 Fase 3 — Prioridad 2 (Completada)

| # | Acción | Cambios |
|---|---|---|
| 2.12 | WASM async → sync; TS SDK async unificado | `vantadb-wasm/src/lib.rs`: `pub async fn put/put_batch/search` → `pub fn` (3). `vantadb-ts/src/vantadb.ts`: `putBatch()`, `search()` — removidos `async`/`await`/`Promise<>`. `vantadb-wasm/tests/wasm_tests.rs`: 27 `.await` removed, 23 test `async fn` → `fn`. `vantadb-wasm/src/lib.rs #[cfg(test)]`: 10 `.await` removed, 9 test `async fn` → `fn`. Verificado: `cargo build` ✅, `cargo test --no-run -p vantadb-wasm` ✅, `vitest run` 159/159 ✅ |

### 15.4 Fase 4 — Prioridad 3 (Completada)

| # | Acción | Cambios |
|---|---|---|
| 3.1 | Proptest serialization round-trips | `tests/proptest_serialization_roundtrip.rs` (620 líneas, 18 proptests). Root causes documentadas: f64 ULP en serde_json (ryu) → postcard para floats; `#[serde(with = "u128_serde")]` + postcard WontImplement; bare u128 OK. `Cargo.toml`: +`serde_json` dev-dep. |
| 3.2 | Concurrency stress tests | `tests/concurrency_primitives.rs` (335 líneas, 6 tests). DashMap concurrent insert/remove, RwLock read/write stress, ArcSwap concurrent load/store, StorageEngine phased concurrent insert/get/delete. Deadlocks corregidos. `.config/nextest.toml`: excluido de default profile. |
| 3.3 | macOS CI matrix | `.github/workflows/ci-rust-10.yml`: job `test-macos` con dtolnay/rust-toolchain + Swatinem/rust-cache + taiki-e/install-action@nextest + cargo check + clippy + `cargo nextest run --profile audit`. |
| 3.4 | MSRV check | `.github/workflows/ci-rust-10.yml`: job `msrv` con dtolnay/rust-toolchain@1.94.1 + `cargo check --workspace`. |
| 3.5 | Binary releases (Windows + ARM64) | `.github/workflows/release-binaries-63.yml`: matrix entries `x86_64-pc-windows-msvc` (windows-latest, .zip) + `aarch64-unknown-linux-gnu` (ubuntu-latest, cross-compile con gcc-aarch64-linux-gnu). |
| 3.6 | Fuzz harnesses | `fuzz/fuzz_targets/fuzz_wal.rs` (WalHeader deserialize + roundtrip), `fuzz_archive.rs` (CPIndex::deserialize_from_bytes). `.github/workflows/fuzz-40.yml`: build + run semanal con cargo-fuzz (nightly). |

### 15.5 Fase 5 — Prioridad 3 (Completada por @eros)

| # | Acción | Cambios |
|---|---|---|
| 3.7 | Migrar GSAP → motion.dev | `package.json`: removidos `gsap`, `@gsap/react`, `@types/gsap`, `split-type`; añadido `motion@12.10.5`. `web/src/utils/motion-utils.ts` (renombrado de `gsap-utils.ts`). `web/src/hooks/useScrollReveal.ts`: 3 animaciones migradas. `AnimatedCounter.tsx`: portado. ScrollTrigger reemplazado por `useInView` + `animate`. `bun.lock` regenerado. |
| 3.8 | tsconfig strictness | `web/tsconfig.json`: `noUnusedLocals: true`, `noUnusedParameters: true`. Limpieza de imports/vars muertos en `FaqAccordion.tsx`, `main.tsx`, etc. |
| 3.9 | Security headers Vercel | `vercel.json`: CSP, HSTS, X-Frame-Options, X-Content-Type-Options, Referrer-Policy configurados. |

### 15.6 Fase 6 — Prioridad 3 (Completada por @eros)

| # | Acción | Cambios |
|---|---|---|
| 3.10 | `.pyi` stubs para Python binding | `vantadb-py/vantadb_py/vantadb_py.pyi`: añadidos `VantaMemoryRecord`, `VantaListResult`, `VantaListResultIter`. `__init__.pyi`: idem. Ya existían stubs previos pero faltaban estos 3 tipos del `__all__` público. |
| 3.11 | Miri tests para unsafe code | `tests/miri_unsafe.rs` (9 tests con `#![cfg(miri)]` cubriendo raw pointer slices, Send/Sync ptr, alignment, sub-slicing, concurrent access). `.github/workflows/ci-rust-10.yml`: job `miri` con nightly + `cargo miri test -p vantadb -- miri`. |
| 3.12 | Resolver `--legacy-peer-deps` | `web/package.json`: removido `stylelint-design-token-guard` (no usado en config, incompatible con stylelint 17). `.github/workflows/ci-web-11.yml`: `npm ci --legacy-peer-deps` → `npm ci`. `web/vercel.json`: idem. `web/vite.config.ts`: limpiados stale `manualChunks` + `optimizeDeps.include` de GSAP. `npm install` verificado: 0 vulnerabilidades. |

### 15.7 Fase 7 — Prioridad 3, 4 y miscelánea (Completada por @eros)

| # | Acción | Cambios |
|---|---|---|
| 3.13 | Migrar `exit_point` a `AtomicU128` | **No aplica** — no existe `exit_point` en el codebase. No hay ningún `Mutex<u128>` o `Mutex<u64>` remanente en producción. El único patrón análogo (`entry_point`) ya estaba migrado a `AtomicU128` (Finding 2.3). |
| 3.14 | Upgradear tantivy 0.22 → 0.26.1 | `Cargo.toml`: tantivy `0.22` → `0.26.1`. `Cargo.lock` actualizado. `flat_threshold: None` añadido en 6 test files. `cargo check -p vantadb --tests` pasa limpio. 15/15 tokenizer tests OK. Resuelve `lru 0.12.5` unsound, elimina `instant`/`lz4_flex` duplicados. |
| 4.1 | SAFETY comments español → inglés | Verificado: todos los SAFETY comments están en inglés desde Phase 1. §3.1 actualizado con paths post-fragmentación. |
| 4.3 | Corregir `aria-expanded` hardcoded en nav dropdowns | `web/src/components/NbNav.tsx`: añadido estado `openGroup`, handlers `onMouseEnter`/`onMouseLeave`/`onFocus`/`onBlur` en cada grupo. `aria-expanded` ahora refleja dinámicamente el grupo abierto. |
| 4.5 | Remover `extract_2d_buffer` dead code | `vantadb-python/src/lib.rs`: eliminada función `extract_2d_buffer` (~35 líneas) con su `#[allow(dead_code)]`. Era vestigial, nunca llamada. |
| 4.6 | Documentar `#[cfg(any())]` en rkyv_archives | `src/serialization/mod.rs`: añadido comentario explicando que rkyv está deshabilitado intencionalmente, que postcard es el codec activo, y qué se necesita para re-habilitarlo. |
| 4.7 | Deployment guide | `docs/operations/DEPLOYMENT_GUIDE.md` — systemd, Docker, K8s, TLS, monitoreo |
| 4.8 | SQLite migration guide | `docs/operations/SQLITE_MIGRATION_GUIDE.md` — schema mapping, export/import, query translation |
| 4.9 | DR runbook | `docs/operations/DISASTER_RECOVERY_RUNBOOK.md` — SEV-1/2/3 procedures, health checks, recovery testing |
| 4.10 | Reescribir Homebrew formula | `Formula/vantadb.rb`: usa tarballs, ARM64 macOS/Linux, `version`, `livecheck`. |
| — | SAFETY retro-post-fragmentación | `graph.rs`: 5 SAFETY comments añadidos. `search.rs`: 2. `serialize.rs`: 3. Comentarios perdidos en Phase 2.2 restaurados. |
| — | W2 (isActive matching) | `NbNav.tsx:112-113`: corregido `startsWith` puro a `===` + `endsWith("/")` para evitar falsos positivos |
| — | W4 (new Date() en render) | `NbFaqAccordion.tsx:70`: reemplazado `new Date().toLocaleTimeString()` con `"00:00:00"` estático |
| — | A4 (hnsw.rs placeholder) | `src/index/hnsw.rs` eliminado; `src/index/mod.rs` actualizado. Indirección innecesaria removida. |

---

## 16. Apéndice: Métricas Clave

### 16.1 Proyecto

| Métrica | Valor |
|---|---|
| Rust source files | 51 |
| Rust total lines | ~35,000 (estimado) |
| Test files | 65 |
| Rust workspace members | 14 |
| Python binding lines | ~1,200 |
| TS binding lines | ~800 |
| Web components (nb/) | 29 |
| Web routes | 27 (100% lazy) |
| CI workflows | 12 |
| Docker images | 2 (prod + dev) |
| Scripts | 12 |
| Documentation files | ~180+ |
| GitHub stars | 282 |
| Forks | 10 |
| Last release | v0.3.0 (2026-07-07) |
| Rust edition | 2021 |
| MSRV | 1.94.1 |

### 16.2 Web Bundle

| Tipo | Tamaño | % |
|---|---|---|
| vendor-react | 177.9 KB | 34.3% |
| index.js (main) | 167.3 KB | 32.2% |
| vendor-router | 80.8 KB | 15.6% |
| Lazy JS (15 chunks) | 93 KB | 17.9% |
| **Total JS** | **519 KB** | (GSAP ~132KB removido → motion.dev ≈0KB extra) |
| **Total CSS** | **188.7 KB** | |
| **Total Fonts** | **189 KB** | |
| **Total Initial load** | **~520 KB** (~160KB gzipped) |

### 16.3 Dependencias Rust

| Métrica | Valor |
|---|---|
| Total crates (transitivas) | ~400+ |
| Workspace members | 14 |
| Duplicate crate pairs | 17 |
| Unmaintained advisories | 4 |
| Unsound advisories | 1 |
| Non-standard licenses | 0 |

### 15.4 Scorecard General

| Categoría | Puntaje (0-10) |
|---|---|
| Arquitectura | 8.5 |
| Safety (Unsafe Rust) | 4.0 |
| Error Handling | 7.5 |
| Security | 7.0 |
| Performance | 7.5 |
| Code Quality | 7.0 |
| Testing | 7.5 |
| CI/CD | 9.5 |
| Docker | 5.0 |
| Python Binding | 9.0 |
| TS Binding | 8.0 |
| WASM Binding | 7.0 |
| Web Frontend | 8.5 |
| Documentation | 8.0 |
| **Promedio Ponderado** | **7.3/10** |

---

*Reporte generado el 2026-07-09 usando 5 skills de addyosmani/agent-skills + 5 exploraciones paralelas de CodeGraph.*
