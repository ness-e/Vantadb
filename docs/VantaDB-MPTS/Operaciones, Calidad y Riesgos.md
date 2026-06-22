---
type: mpts-section
status: stable
tags: [vantadb, operaciones, testing, riesgos, calidad, benchmarks, ci-cd, chaos-testing]
last_refined: 2026-06-21 (WASM Browser Build — SystemTime fix)
links: "[Master Index](Master Index.md)"
description: "Estrategia de testing (unit, integration, chaos), pipeline CI/CD, benchmarks certificados, matriz FMEA de riesgos y limitaciones estructurales"
aliases: [Operaciones, Calidad, Riesgos, Testing, CI/CD]
---

# Operaciones, Calidad y Riesgos

> **Dominio:** Técnico & Operaciones
> **Propósito:** Documentar estrategia de testing, CI/CD, benchmarks y riesgos conocidos

---

## Estrategia de Pruebas (Testing)

### Cobertura Actual

| Tipo | Cobertura | Herramienta |
|------|-----------|-------------|
| **Unit tests** | ~75% | `cargo test` |
| **Integration tests** | ~70% | `cargo test --test` (incluye 33 tests CLI) |
| **File locking (Windows)** | 3 tests | `file_locking_stress` (FILE_SHARE_READ, DELETE, stale lock) |
| **Property-based** | ~30% | `proptest` |
| **Fuzzing** | ~10% | `cargo-fuzz` |
| **Chaos testing** | ~90% | Crash injection (AUD-02/03), failpoints, WAL corruption |

### Suites de Testing

#### 1. Fast Gate (Cada PR)

```yaml
# .github/workflows/rust_ci.yml
jobs:
  test:
    steps:
      - cargo fmt --check
      - cargo clippy -- -D warnings
      - cargo test --lib
      - cargo test --doc
```

**Duración:** ~5 minutos
**Frecuencia:** Cada push/PR
**Valida:** [CI/CD](Glosario/CI_CD.md) básico

#### 2. Heavy Certification (Semanal)

```yaml
# .github/workflows/heavy_certification.yml — 11 jobs paralelos
jobs:
  stress-protocol:       cargo test --release --test stress_protocol
  hnsw-validation:       cargo test --release --test hnsw_validation
  hnsw-recall:           cargo test --release --test hnsw_recall_certification
  failpoint-tests:       nextest --test chaos_integrity (--features failpoints)
                         nextest --test wal_resilience
                         nextest --test crash_injection
  storage-persistence:   cargo test --release --features cli (17 tests: backend, storage, gc, mmap, etc.)
  text-index:            cargo test --release --test text_index_recovery
  memory-concurrency:    cargo test --release (6 tests: memory_brutality, memory_telemetry, etc.)
  other-heavy:           cargo test --release --features cli,arrow (12 tests: columnar, hybrid_ranking, etc.)
                         cargo test --package vantadb-server (4 tests: mcp_integration, e2e, etc.)
                         cargo test --package vantadb-mcp --test mcp_tests
                         cargo test --lib concurrent_insert_preserves_hnsw_invariants
```

**Duración:** ~30–180 min (jobs paralelos)
**Frecuencia:** Semanal (domingo 03:00 UTC) + manual
**Valida:** [Chaos Testing](Glosario/Chaos Testing.md), durabilidad [WAL](Glosario/WAL.md), recall HNSW, storage persistence

#### 3. Fuzzing (Nocturno)

```yaml
# .github/workflows/fuzzing.yml
jobs:
  fuzz:
    steps:
      - cargo fuzz run fuzz_parser -- -max_total_time=3600
      - cargo fuzz run fuzz_node_deserialize -- -max_total_time=3600
```

**Duración:** ~6 horas
**Frecuencia:** Nocturno (lunes-viernes)

#### Crash Injection (AUD-02/03)

```rust
#[test]
fn test_crash_during_active_writes_with_tight_loop() {
    // Helper en modo "tight": sin sleep entre writes
    let mut child = Command::new(&helper_path)
        .arg(db_path).arg("500").arg("tight")
        .spawn();

    // Matar inmediatamente tras el primer WRITTEN (write en vuelo)
    for line in reader.lines() {
        if line.unwrap().starts_with("WRITTEN:") { break; }
    }
    child.kill();

    // Verificar DB recuperable + nodo confirmado presente + HNSW válido
    let engine = StorageEngine::open(db_path).unwrap();
    assert!(engine.get(written_id).unwrap().is_some());
    assert!(engine.hnsw.load().validate_index().is_ok());
}
```

**20 iteraciones, modo tight loop (sin delay). Valida que kill -9 durante writes activos no corrompe la DB.**

### Tests Críticos

#### HNSW Recall Validation

```rust
#[test]
fn test_hnsw_recall_sift1m() {
    let db = VantaEmbedded::open("./sift1m_test")?;
    
    // Cargar dataset SIFT1M
    let (vectors, queries, ground_truth) = load_sift1m()?;
    
    // Indexar
    for (i, vector) in vectors.iter().enumerate() {
        db.put(&format!("vec_{}", i), Some(vector), None, None)?;
    }
    
    // Medir recall@10 ([ANN](Glosario/ANN.md) metric)
    let mut total_recall = 0.0;
    for (query, gt) in queries.iter().zip(ground_truth.iter()) {
        let results = db.search(query, 10, SearchMode::Vector)?;
        let retrieved: HashSet<_> = results.iter().map(|r| r.key.clone()).collect();
        let relevant: HashSet<_> = gt.iter().cloned().collect();
        
        let recall = retrieved.intersection(&relevant).count() as f32 / 10.0;
        total_recall += recall;
    }
    
    let avg_recall = total_recall / queries.len() as f32;
    assert!(avg_recall >= 0.95, "Recall@10 debe ser >= 0.95, actual: {}", avg_recall);
    
    Ok(())
}
```

#### WAL Durability Test

```rust
#[test]
fn test_wal_crash_recovery() {
    for i in 0..1000 {
        let db = VantaEmbedded::open("./crash_test")?;
        
        // Escribir datos
        for j in 0..100 {
            db.put(&format!("key_{}_{}", i, j), Some(&vec![1.0; 128]), None, None)?;
        }
        
        // Simular crash (sin close)
        drop(db);
        
        // Reabrir y verificar
        let db = VantaEmbedded::open("./crash_test")?;
        let count = db.count()?;
        assert_eq!(count, 100, "Iteración {}: datos perdidos", i);
        
        // Limpiar
        std::fs::remove_dir_all("./crash_test")?;
    }
}
```

#### WAL Compaction (TSK-75)

Se implementó `compact_wal()` en el SDK que:
- Rotación de segmentos WAL obsoletos (posterior a checkpoint)
- Trigger automático al superar 256 MB de tamaño acumulado
- Exposición vía `vanta-cli wal compact`
- Reducción de uso de disco sin interrumpir operaciones de lectura/escritura

---

## Pipeline de CI/CD

### Workflows de GitHub Actions

| Workflow | Trigger | Duración | Propósito |
|----------|---------|----------|-----------|
| **rust_ci.yml** | push/PR | 5 min | Tests rápidos, lint, format |
| **python_wheels.yml** | tag `v*` | 15 min | Build + publish wheels |
| **release.yml** | tag `v*` | 20 min | Build binarios multi-platform |
| **heavy_certification.yml** | weekly | 2 horas | Stress tests, chaos testing |
| **fuzzing.yml** | nightly | 6 horas | Fuzz testing continuo |
| **benchmarks.yml** | push a main | 30 min | Regresión de performance |

#### Auditoría de CI/CD (Junio 2026)

Se realizó una auditoría completa de todos los workflows con las siguientes correcciones:

| Workflow | Corrección |
|----------|------------|
| **rust_ci.yml** | `dtolnay/rust-toolchain` actualizado a `@stable` |
| **release.yml** | Runner `windows-2025-vs2026` → `windows-latest` (runner inexistente) |
| **python_wheels.yml** | Eliminado `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24` obsoleto |
| **heavy_certification.yml** | Eliminado `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24` obsoleto |
| **bench.yml** | Push mejorado con `GITHUB_TOKEN` explícito para branch protection |
| **nextest** | `.config/nextest.toml`: crash_injection excluido del profile audit |

**Resultado:** Todos los workflows compilan y ejecutan correctamente en GitHub Actions.

#### Tests de Seguridad del Servidor (TSK-14/15/16 — Junio 2026)

Se agregaron 14 tests unitarios para el servidor HTTP cubriendo:

| Categoría | Tests | Escenarios cubiertos |
|-----------|-------|---------------------|
| **Auth Bearer Token** | 6 tests | No auth, valid token, invalid token, missing header, wrong scheme, health exempt |
| **Rate Limiting** | 3 tests | RPM=0 (10 requests pasan), RPM>0 (burst limit + 429), health no afectado |
| **TLS/HTTPS** | 2 tests | Config loading with PEM, server health/query over HTTPS (requires `--features tls`) |
| **Concurrencia** | 3 tests | 20 parallel requests, 10 requests with semaphore=2, 10 concurrent with auth |
| **E2E Integration** | 6 tests | Real TCP server socket: health+metrics, insert+query+delete, auth over real HTTP, persistence after restart, rate limiting, bad request 400 |

### Publicación a PyPI

```yaml
# .github/workflows/python_wheels.yml
name: Python Wheels
on:
  push:
    tags:
      - 'v*'

jobs:
  build-wheels:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: PyO3/maturin-action@v1
        with:
          command: build
          args: --release --out dist
      - uses: actions/upload-artifact@v4
        with:
          name: wheels-${{ matrix.os }}
          path: dist/*.whl

  publish:
    needs: build-wheels
    runs-on: ubuntu-latest
    permissions:
      id-token: write  # OIDC
    steps:
      - uses: actions/download-artifact@v4
      - uses: pypa/gh-action-pypi-publish@release/v1
```

### Matrix de Compilación

| Platform | Architecture | Runner | Status |
|----------|--------------|--------|--------|
| Linux | x86_64 | ubuntu-latest | ✅ |
| Linux | ARM64 | ubuntu-latest (cross) | ⚠️ Experimental |
| macOS | x86_64 | macos-latest | ✅ |
| macOS | ARM64 (Apple Silicon) | macos-latest | ✅ |
| Windows | x86_64 | windows-latest | ✅ |

---

## Estrategia de Benchmarking

### Métricas Clave

#### Performance

| Métrica | Objetivo | Actual | Status |
|---------|----------|--------|--------|
| **Search p50** | <20ms | 62ms | ⚠️ Necesita optimización |
| **Search p99** | <100ms | 180ms | ⚠️ Necesita optimización |
| **Ingesta** | >100 ops/s | 95 ops/s | ⚠️ Cerca del objetivo |
| **Recall@10** | ≥0.95 | 0.998 | ✅ Excede objetivo |

#### Escalabilidad

| Dataset | Vectors | Memory | Recall | Latency p50 |
|---------|---------|--------|--------|-------------|
| Small | 10K | ~12 MB | 0.956 | 1.2 ms |
| Medium | 50K | ~58 MB | 1.000 | 6.1 ms |
| Large | 100K | ~117 MB | 0.998 | 12.4 ms |
| X-Large | 500K | ~585 MB | TBD | TBD |
| XX-Large | 1M | ~1.17 GB | TBD | TBD |

### Benchmarks Públicos

**Ubicación:** `benchmarks/` en el repositorio
**Datasets:**
- SIFT1M (1M vectores, 128d)
- GIST1M (1M vectores, 960d)
- Custom RAG corpus (100K documentos)

**Ejecución:**
```bash
python benchmarks/run.py --suite standard --output results.json
```

**Resultados:** Publicados en README.md y docs/benchmarks.md

---

## Riesgos y Limitaciones Conocidas

### Riesgos Bloqueantes (Severidad: 🔒)

#### AUD-01: [WAL](Glosario/WAL.md) Durabilidad No Verificada ✅ RESUELTO

**Descripción:** El snapshot no demuestra que fsync() se ejecute antes del ACK al cliente.

**Impacto:** Claims de durabilidad no verificables. Posible pérdida de datos en crashes.

**Mitigación:**
```rust
pub fn put(&self, mutation: &Mutation) -> Result<()> {
    self.wal.append(mutation)?;
    self.wal.fsync()?;  // ← CRÍTICO: fsync antes de ACK
    self.apply_to_storage(mutation)?;
    Ok(())
}
```

**Test Requerido:** Chaos testing con kill -9 durante writes.

**Tests Implementados:**
- `test_crash_injection_and_cold_recovery_loop` (AUD-02): 10 iteraciones, kill entre writes, verifica recuperación de todos los nodos confirmados + validación estructural HNSW
- `test_crash_during_active_writes_with_tight_loop` (AUD-03): 20 iteraciones, kill inmediato tras primer write confirmado (tight loop sin sleep), verifica integridad post-crash con writes en pleno vuelo

**Status:** ✅ Resuelto (FASE 3) - WAL durabilidad verificada con 30 iteraciones de crash injection

#### AUD-02: [WAL](Glosario/WAL.md) sin Checksums ✅ RESUELTO

**Descripción:** Registros del WAL no tienen CRC32C, imposibilitando detección de corrupción.

**Impacto:** Recovery puede aplicar registros corruptos silenciosamente.

**Mitigación Implementada:**
```rust
// src/wal.rs:13-16
use crc32c::crc32c;

#[inline]
pub fn compute_crc32c(data: &[u8]) -> u32 {
    crc32c::crc32c(data)
}

// Validación en WalHeader::deserialize() líneas 96-102
let computed_crc = header.compute_crc();
if computed_crc != crc {
    return Err(VantaError::WalError(format!(
        "WAL header CRC mismatch: stored={:#x}, computed={:#x}",
        crc, computed_crc
    )));
}
```

**Status:** ✅ Resuelto (FASE 2) - CRC32C implementado y validado en todos los registros WAL

### Riesgos Altos (Severidad: ⚠️)

#### AUD-03: Concurrencia en Rebuild

**Descripción:** `rebuild_index()` no adquiere lock exclusivo, permitiendo lecturas concurrentes.

**Impacto:** Lectores pueden ver índice parcialmente reconstruido.

**Mitigación:**
```rust
fn rebuild_index(&self) -> Result<()> {
    let mut engine = self.engine.write().unwrap();  // Lock exclusivo
    engine.rebuild_index()
}
```

**Status:** 🔄 En progreso (FASE 2)

#### AUD-04: Falta de [File Locking](Glosario/File Locking.md) ✅ RESUELTO

**Descripción:** VantaDB no implementa file locking, permitiendo que múltiples procesos abran la misma DB.

**Impacto:** Corrupción de datos garantizada si dos procesos escriben simultáneamente.

**Mitigación Implementada:**
```rust
// src/storage.rs:9
use fs2::FileExt;

// src/storage.rs:540-557
let lock_file = if !config.read_only {
    std::fs::create_dir_all(&base_path).map_err(VantaError::IoError)?;
    let lock_path = base_path.join(".vanta.lock");
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)
        .map_err(VantaError::IoError)?;

    file.try_lock_exclusive().map_err(|_| {
        VantaError::Execution(format!(
            "Database at '{}' is locked by another process. \
             Only one VantaDB instance can open a database directory at a time.",
            base_path.display()
        ))
    })?;
    Some(file)
} else {
    None
};
```

**Status:** ✅ Resuelto (FASE 2) - File locking exclusivo implementado con fs2

#### AUD-05: [GIL](Glosario/GIL.md) No Liberado Consistentemente ✅ RESUELTO

**Descripción:** Algunas operaciones pesadas no liberan el GIL en PyO3.

**Impacto:** Aplicaciones multi-thread se bloquean durante estas operaciones.

**Mitigación Implementada:**
```rust
// vantadb-python/src/lib.rs - TODOS los métodos usan py.allow_threads()

#[pymethods]
impl VantaDB {
    fn put(&self, py: Python, ...) -> PyResult<()> {
        let engine = self.engine.clone();
        py.allow_threads(move || {
            engine.put(input)
                .map_err(|e| PyRuntimeError::new_err(format!("Put error: {:?}", e)))
        })?;
        Ok(())
    }
    
    fn search_memory(&self, py: Python, ...) -> PyResult<Vec<PyObject>> {
        let engine = self.engine.clone();
        let hits = py.allow_threads(move || {
            engine.search(request)
                .map_err(|e| PyRuntimeError::new_err(format!("Search memory error: {:?}", e)))
        })?;
        // ...
    }
    
    fn search_batch(&self, py: Python, ...) -> PyResult<Vec<Vec<(u64, f32)>>> {
        let engine = self.engine.clone();
        py.allow_threads(move || {
            // Búsqueda paralela con Rayon
            vectors.into_par_iter().map(|vector| { /* ... */ }).collect()
        })
    }
    
    // Todos los métodos (insert, get, delete, search, list_memory, export, import, audit, repair, etc.)
    // usan py.allow_threads() consistentemente
}
```

**Status:** ✅ Resuelto (FASE 2) - GIL liberado consistentemente en todas las operaciones del Python SDK

### Riesgos Medios (Severidad: ℹ️)

#### Performance de Python SDK

**Descripción:** Latencia de búsqueda en Python (62ms) excede objetivo (<20ms).

**Causa:** Overhead de FFI + copia de datos en frontera (creación de objetos Python, conversión de Vec<f32> a listas, serialización de metadata a dicts). El GIL ya está liberado consistentemente.

**Mitigación:**
- Optimizar serialización (zero-copy donde sea posible)
- Batch operations para reducir cruces FFI
- Devolver memoryview/NumPy arrays en lugar de listas Python
- Profile y optimizar hot paths

**Status:** 🔄 En progreso (FASE 3) - GIL ya resuelto, ahora optimizar overhead de copia de datos

#### Telemetría de Memoria Inconsistente

**Descripción:** Métricas de RAM reportan valores imposibles (~225 GB en máquina de ~34 GB).

**Causa:** Bug en cálculo de RSS vs mmap.

**Mitigación:**
```rust
pub struct MemoryMetrics {
    pub rss_bytes: u64,           // Resident Set Size (real)
    pub logical_bytes: u64,       // Logical allocations
    pub mmap_resident_bytes: u64, // mmap pages in RAM
}
```

**Status:** ⬜ Pendiente (FASE 3)

---

## Limitaciones Estructurales

### Lo Que VantaDB NO Hace (y No Hará en el Corto Plazo)

#### 1. Distribución y Replicación

**Limitación:** VantaDB es single-node, sin replicación nativa.

**Razón:** Filosofía embedded-first. Distribución añade complejidad masiva.

**Workaround:** Usuarios pueden implementar replicación a nivel de aplicación.

**Roadmap:** Evaluación de replicación asíncrona en FASE 5+ (12+ meses).

#### 2. Multi-Tenancy

**Limitación:** Una instancia = una base de datos = un tenant.

**Razón:** Simplicidad y seguridad. Multi-tenancy requiere aislamiento complejo.

**Workaround:** Múltiples instancias en diferentes directorios.

**Roadmap:** Multi-tenancy en VantaDB Cloud (FASE 5+).

#### 3. Lenguaje de Query Declarativo

**Limitación:** Sin SQL, Cypher, ni GraphQL. Solo API programática.

**Razón:** Enfoque en simplicidad. Query languages añaden complejidad.

**Workaround:** Usuarios implementan lógica de query en su aplicación.

**Roadmap:** Evaluación de query language en FASE 5+.

#### 4. Transacciones Distribuidas

**Limitación:** Transacciones son locales a una sola instancia.

**Razón:** Single-node por diseño.

**Workaround:** 2PC a nivel de aplicación si es necesario.

**Roadmap:** No planeado (contradice filosofía embedded).

#### 5. Streaming de Resultados

**Limitación:** `search()` retorna todos los resultados de una vez.

**Razón:** Simplicidad de API. Streaming añade complejidad.

**Workaround:** Paginación manual con offset/limit.

**Roadmap:** Streaming API en FASE 4 (6+ meses).

---

## Plan de Mitigación de Riesgos

> **Las tareas detalladas de mitigación viven en [Backlog](../Backlog.md), no aquí.**
> Este documento describe la **estrategia de testing** y las **garantías verificadas**, no el checklist de implementación.

### Prioridades por Fase

#### FASE 2: Hardening Arquitectónico ✅ COMPLETADO

**Logro:** Cero riesgos bloqueantes. WAL con CRC32C, fsync antes de ACK, file locking exclusivo, lock en rebuild_index, GIL liberado consistentemente.

**Validación:** 30 iteraciones chaos injection, 100+ tests de resiliencia.

#### FASE 3: Pre-Lanzamiento 🔄

**Objetivo:** Cobertura de tests >90%, latencia Python <20ms, telemetría correcta.

**Riesgos a mitigar:**
- DISC-05: Telemetría de memoria (RSS vs mmap)
- Performance Python: overhead FFI (TSK-68)
- Sin datasets reales en CI (TSK-55)
- ~~Panics en producción por `unwrap()`/`panic!` en runtime~~ ✅ TSK-97 (Hardening completado)

**Ver [Backlog#3.D Testing y Calidad](../Backlog.md#3d-testing-y-calidad)** para tareas específicas.

#### FASE 4: Community Launch 🔄

**Objetivo:** Ecosistema seguro, distribuido, documentado.

**Riesgos a mitigar:**
- Sin TypeScript SDK = TAM reducido a la mitad (TSK-61)
- Sin backup/restore = producción insegura (TSK-63)
- Sin SECURITY.md = enterprise bloqueado (TSK-106b)

**Ver [Backlog#4.J Seguridad Básica](../Backlog.md#4j-seguridad-bãsica)** y [Backlog#4.G Distribución y Packaging](../Backlog.md#4g-distribuciãn-y-packaging)** para tareas específicas.

---

## Monitoreo y Observabilidad

### Métricas Expuestas

```rust
pub struct Metrics {
    // Performance
    pub search_latency_ms: Histogram,
    pub ingest_throughput: Counter,
    
    // Storage
    pub wal_size_bytes: Gauge,   # Reducible vía compact_wal()
    pub index_size_bytes: Gauge,
    pub total_documents: Gauge,
    
    // Memory
    pub rss_bytes: Gauge,
    pub mmap_resident_bytes: Gauge,
    
    // Operations
    pub search_count: Counter,
    pub ingest_count: Counter,
    pub error_count: Counter,
}
```

### Integración con Prometheus

#### Grafana Dashboard

- **Dashboard oficial:** `docs/operations/grafana-dashboard.json` (6 panels: RSS, latencia p50/p95/p99, ops vectoriales, disco, memoria de índice por capa)
- **Setup:** `docs/operations/GRAFANA_SETUP.md`
- **Requisito:** Prometheus scraping `/metrics` en el servidor HTTP


```python
from prometheus_client import start_http_server
from vantadb import VantaEmbedded

db = VantaEmbedded("./data")

# Exponer métricas en :8000/metrics
start_http_server(8000)

# Métricas disponibles:
# - vantadb_search_latency_ms_bucket
# - vantadb_ingest_throughput_total
# - vantadb_wal_size_bytes
# - vantadb_rss_bytes
# - vantadb_search_count_total
# - vantadb_error_count_total
```

---

## Véase También

- [Master Index](Master Index.md) — Documento padre
- [Arquitectura Técnica y Core Engine](Arquitectura Técnica y Core Engine.md) — Diseño interno
- [Roadmap e Hitos de Ingeniería](Roadmap e Hitos de Ingeniería.md) — Timeline de mitigaciones
- [Estrategia de Ecosistema y GTM](Estrategia de Ecosistema y GTM.md) — Calidad como ventaja competitiva

---

*La calidad y transparencia sobre limitaciones es fundamental para construir confianza con developers que usarán VantaDB en producción.*
