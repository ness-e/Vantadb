# Plan de Implementación: Corrección docs/operations/ — Post-Auditoría Completa

> **Campaign ID: ops-docs-2026-07-21
> **Inicio:** 2026-07-21
> **Estado: completed
> **Fuente:** Auditoría completa docs/operations/ (vanta-lead, Jul 21) — 25 archivos evaluados
> **Score actual:** 5.9/10

## Resumen

Auditoría completa de `docs/operations/` (25 archivos) realizada con 3 sub-agentes en paralelo,
verificando cada archivo contra código real (codegraph + grep + read).

### Scores por archivo

| Archivo | Score | Veredicto |
|---------|-------|-----------|
| CONFIGURATION.md | 4/10 | ❌ |
| CI_POLICY.md | 5/10 | ❌ |
| DEPLOYMENT_GUIDE.md | 4/10 | ❌ |
| PERFORMANCE_TUNING.md | 6/10 | ⚠️ |
| GC_TTL.md | 5/10 | ❌ |
| EDITOR_INTEGRATIONS.md | 3/10 | ❌ |
| GRAFANA_SETUP.md | 2/10 | ❌ |
| grafana-dashboard.json | 2/10 | ❌ |
| PERFORMANCE_GUIDE.md | 4/10 | ❌ |
| SECURITY.md | 5/10 | ❌ |
| PILOT_PROGRAM.md | 5/10 | ❌ |
| REPO_CHECKLIST.md | 6/10 | ❌ |
| RELIABILITY_GATE.md | 5/10 | ❌ |
| BACKUP_POLICY.md | 7/10 | ⚠️ |
| DISASTER_RECOVERY_RUNBOOK.md | 6/10 | ⚠️ |
| DURABILITY_GUARANTEES.md | 7/10 | ⚠️ |
| MEMORY_TELEMETRY.md | 7/10 | ⚠️ |
| AGENT_INSTRUCTIONS.md | 7/10 | ⚠️ |
| BENCHMARKS.md | 8/10 | ✅ |
| EXPERIMENTAL_FEATURES.md | 8/10 | ✅ |
| FUZZING.md | 9/10 | ✅ |
| COMMUNITY_GOVERNANCE.md | 9/10 | ✅ |
| PUBLIC_ISSUE_DRAFTS.md | 9/10 | ✅ |
| PYTHON_RELEASE_POLICY.md | 9/10 | ✅ |
| SQLITE_MIGRATION_GUIDE.md | 8/10 | ✅ |
| **TOTAL (promedio ponderado)** | **5.9/10** | **❌** |

### Problemas estructurales globales

1. **7 archivos faltan en `docs/master-index.md`**: DISASTER_RECOVERY_RUNBOOK, DEPLOYMENT_GUIDE, GC_TTL, PERFORMANCE_GUIDE, PERFORMANCE_TUNING, SECURITY, SQLITE_MIGRATION_GUIDE — no están listados en la sección Operations & Configuration
2. **25 archivos es excesivo**: varias categorías podrían consolidarse (PERFORMANCE_GUIDE + PERFORMANCE_TUNING → uno solo, BENCHMARKS podría fusionarse)
3. **Falta documentación de logging**: CONFIGURATION.md lo menciona pero no hay doc dedicada
4. **Cross-contradicciones**: EXPERIMENTAL_FEATURES.md dice IQL/LISP archivado, pero FUZZING.md tiene target `fuzz_parser` activo y MCP tiene tool `query_lisp`

---

## Dependencias

```
WAVE 1 (HIGH — datos incorrectos que afectan a usuarios)
├── OPS-01: CONFIGURATION.md — defaults, features, deps incorrectos
├── OPS-02: DEPLOYMENT_GUIDE.md — versión 0.6.9 inexistente, repo URL
├── OPS-03: PERFORMANCE_GUIDE.md — line references off by 280 líneas
├── OPS-04: EDITOR_INTEGRATIONS.md — binary name, MCP tools inventados (REESCRITURA)
├── OPS-05: GRAFANA_SETUP.md + grafana-dashboard.json — prefijo métricas (REESCRITURA)
├── OPS-06: SECURITY.md — .github/SECURITY.md faltante, rate limit default
└── OPS-07: RELIABILITY_GATE.md — nextest config, Python API signature

WAVE 2 (MEDIUM — precisión contra código real)
├── OPS-08: PERFORMANCE_TUNING.md — ef_construction 200→400, line refs
├── OPS-09: GC_TTL.md — u64→u128 node IDs
├── OPS-10: CI_POLICY.md — workflow names reales
├── OPS-11: REPO_CHECKLIST.md — 3 archivos referenciados no existen
├── OPS-12: DISASTER_RECOVERY_RUNBOOK.md — env var incorrecta
├── OPS-13: PILOT_PROGRAM.md — Python constructor args incorrectos
├── OPS-14: DURABILITY_GUARANTEES.md — line refs desactualizadas (src/storage.rs → src/storage/engine/)
└── OPS-15: BACKUP_POLICY.md — texto duplicado líneas 34-37

WAVE 3 (LOW — mejora continua)
├── OPS-16: MEMORY_TELEMETRY.md — métricas Prometheus faltantes
├── OPS-17: AGENT_INSTRUCTIONS.md — claim incorrecto sobre C-ABI
├── OPS-18: EXPERIMENTAL_FEATURES.md + FUZZING.md — resolver contradicción IQL/LISP
├── OPS-19: BENCHMARKS.md — nota de ~ inconsistente
└── OPS-20: master-index.md — agregar 7 archivos faltantes
```

---

### Task OPS-01: Fix CONFIGURATION.md — defaults, features, deps incorrectos

- **Archivos clave:** `docs/operations/CONFIGURATION.md`, `src/config.rs`, `Cargo.toml`
- **Gate Justificación:** 2 HIGH (default features incorrectos, insert_lock_timeout_ms), 2 MEDIUM (cli/tls deps incorrectas). Afecta a usuarios que configuran VantaDB basándose en el doc.
- **Issues a corregir:**
  - `default` features en tabla: doc dice incluye `rocksdb`, real NO lo incluye
  - `insert_lock_timeout_ms` default: doc dice `2000`, real `5000`
  - `cli` feature deps: doc lista `rustyline, strsim, color-eyre` — ninguno existe
  - `tls` feature deps: doc dice `axum-server, rustls-pemfile`, real `dep:axum-server, dep:rustls`
  - Saltar sección 8 en numeración
  - Marcar feature gate para `hot_reload_config` y `encryption_key`
- **Contrato: EXPERIMENTAL_FEATURES.md cross-references son válidas. FUZZING.md marcado como archived.
- **Task file:** `tasks/OPS-01.md`
- **Estado:** ✅ COMPLETED

---

### Task OPS-02: Fix DEPLOYMENT_GUIDE.md — versión inexistente, repo URL

- **Archivos clave:** `docs/operations/DEPLOYMENT_GUIDE.md`, `Cargo.toml`
- **Gate Justificación:** `ARG VANTADB_VERSION=0.6.9` — esa versión NO existe. El workspace es `0.4.0`. GitHub URL usa `vantadb/vantadb` en vez de `ness-e/Vantadb`.
- **Issues:**
  - `ARG VANTADB_VERSION=0.6.9` → `0.4.0` (o hacerlo dinámico)
  - `github.com/vantadb/vantadb` → `github.com/ness-e/Vantadb`
  - `vantadb-python/tests/` path ya no existe
- **Contrato:** "DEPLOYMENT_GUIDE.md no contiene referencias a `0.6.9` ni `vantadb/vantadb`. Versión = `0.4.0` o dinámica."
- **Task file:** `tasks/OPS-02.md`
- **Estado:** ✅ COMPLETED

---

### Task OPS-03: Fix PERFORMANCE_GUIDE.md — line references desactualizadas

- **Archivos clave:** `docs/operations/PERFORMANCE_GUIDE.md`, `vantadb-python/src/lib.rs`
- **Gate Justificación:** 2 HIGH — line references off por ~280 líneas (`extract_vector`, `VantaVector/__array_interface__`). Afecta a desarrolladores que hacen performance debugging.
- **Issues:**
  - §7.1: `extract_vector` line ref `lib.rs:199-243` → código real empieza en línea 234
  - §7.2: `VantaVector`/`__array_interface__` line ref `lib.rs:1485-1552` → real línea 1768
  - Code snippet de `__array_interface__` no coincide exactamente con código real
- **Contrato:** "Todas las line references en PERFORMANCE_GUIDE.md coinciden con el código real verificado con `rust-analyzer-mcp` o grep."
- **Task file:** `tasks/OPS-03.md`
- **Estado:** ✅ COMPLETED

---

### Task OPS-04: Rewrite EDITOR_INTEGRATIONS.md — binary name, MCP tools

- **Archivos clave:** `docs/operations/EDITOR_INTEGRATIONS.md`, `vantadb-mcp/src/lib.rs`, `vantadb-server/Cargo.toml`
- **Gate Justificación:** 6 HIGH issues. Binary name `vanta-server` no existe (real: `vantadb-server`). Tools list (`memory_put`, `memory_get`, etc.) completamente inventada — los tools reales son otros. Custom tokenizer config y custom metrics config no existen.
- **Acción:** Reescribir completamente el archivo. Verificar tools reales en MCP server. Si no hay tiempo para reescritura completa, agregar disclaimer `> **⚠️ DRAFT — Requires verification against MCP server v0.4.x**`
- **Contrato:** "EDITOR_INTEGRATIONS.md refleja binary name real (`vantadb-server`) y lista tools reales del MCP server, o tiene disclaimer de draft."
- **Task file:** `tasks/OPS-04.md`
- **Estado:** ✅ COMPLETED

---

### Task OPS-05: Rewrite GRAFANA_SETUP.md + fix grafana-dashboard.json — prefijo métricas

- **Archivos clave:** `docs/operations/GRAFANA_SETUP.md`, `docs/operations/grafana-dashboard.json`, `src/metrics/core/registry.rs`
- **Gate Justificación:** 2 HIGH document + 2 HIGH dashboard. Todas las métricas usan prefijo `vantadb_` que NO existe. El prefijo real es `vanta_`. 6 métricas documentadas son incorrectas (no existen). `--metrics-addr` flag no verificado.
- **Acción:** Reescribir GRAFANA_SETUP.md con métricas reales. Regenerar grafana-dashboard.json con expresiones PromQL correctas.
- **Métricas reales existentes:**
  - `vanta_process_rss_bytes` ✅
  - `vanta_process_virtual_bytes` ✅
  - `vanta_hnsw_nodes_count` ✅
  - `vanta_hnsw_logical_bytes` ✅
  - `vanta_mmap_resident_bytes` ✅
  - `vanta_volatile_cache_entries` ✅
  - `vanta_volatile_cache_cap_bytes` ✅
  - `vanta_jemalloc_*` (7 gauges) ✅
- **Contrato:** "GRAFANA_SETUP.md usa prefijo `vanta_` en todas las métricas. grafana-dashboard.json contiene queries PromQL que existen en el código real."
- **Task file:** `tasks/OPS-05.md`
- **Estado:** ✅ COMPLETED

---

### Task OPS-06: Fix SECURITY.md — reporting link roto, rate limit default

- **Archivos clave:** `docs/operations/SECURITY.md`, `.github/SECURITY.md`, `src/config.rs`
- **Gate Justificación:** 2 HIGH — `.github/SECURITY.md` no existe (link a reporting es 404 en-repo). `VANTADB_RATE_LIMIT_RPM` default es 100, doc dice 0.
- **Issues:**
  - Line 152 ref a `.github/SECURITY.md` — archivo no existe (fue creado y luego posiblemente eliminado)
  - Line 137: "`0` (default) = Rate limiting disabled" — real default es 100
  - Missing `VANTADB_REQUIRE_AUTH` env var documentation
- **Contrato:** "SECURITY.md no referencia archivos inexistentes. Rate limit default documentado como 100."
- **Task file:** `tasks/OPS-06.md`
- **Estado:** ✅ COMPLETED

---

### Task OPS-07: Fix RELIABILITY_GATE.md — nextest config, Python API

- **Archivos clave:** `docs/operations/RELIABILITY_GATE.md`, `.config/nextest.toml`, `vantadb-python/src/lib.rs`
- **Gate Justificación:** 2 HIGH — `cargo nextest` no está configurado en el proyecto (no hay nextest.toml). Python API sample usa `db.insert(count, ...)` que tiene firma incorrecta.
- **Contrato:** "RELIABILITY_GATE.md no asume nextest config que no existe. Python API samples usan firmas reales (`db.put()` en vez de `db.insert()`)."
- **Task file:** `tasks/OPS-07.md`
- **Estado:** ✅ COMPLETED

---

### Task OPS-08: Fix PERFORMANCE_TUNING.md — ef_construction 200→400

- **Archivos clave:** `docs/operations/PERFORMANCE_TUNING.md`, `src/index/graph.rs`
- **Gate Justificación:** `ef_construction` default en doc = 200, código real = 400. Toda la tabla de parámetros y guía está basada en este valor incorrecto.
- **Contrato:** "PERFORMANCE_TUNING.md documenta `ef_construction` default = 400. Line references actualizadas."
- **Task file:** `tasks/OPS-08.md`
- **Estado:** ✅ COMPLETED

---

### Task OPS-09: Fix GC_TTL.md — u64→u128 node IDs

- **Archivos clave:** `docs/operations/GC_TTL.md`, `src/gc.rs`
- **Gate Justificación:** `GcWorker` usa `u128` para node IDs (código real: `BTreeMap<u64, Vec<u128>>`), doc muestra `Vec<u64>` y `BTreeMap<u64, Vec<u64>>`. Menciona método `purgeExpired()` que no existe.
- **Contrato:** "GC_TTL.md usa tipos `u128` para node IDs. Elimina o corrige ref a `purgeExpired()`."
- **Task file:** `tasks/OPS-09.md`
- **Estado:** ✅ COMPLETED

---

### Task OPS-10: Fix CI_POLICY.md — workflow names reales

- **Archivos clave:** `docs/operations/CI_POLICY.md`, `.github/workflows/`
- **Gate Justificación:** Doc refiere `rust_ci.yml`, `heavy_certification.yml`, `python_wheels.yml`. Reales: `ci-rust-10.yml`, `heavy-certification-50.yml`, `release-wheels-60.yml`.
- **Contrato:** "CI_POLICY.md referencia nombres de workflow reales en `.github/workflows/`."
- **Task file:** `tasks/OPS-10.md`
- **Estado:** ✅ COMPLETED

---

### Task OPS-11: Fix REPO_CHECKLIST.md — archivos referenciados no existen

- **Archivos clave:** `docs/operations/REPO_CHECKLIST.md`
- **Gate Justificación:** 3 archivos referenciados no existen: `docs/operations/ROADMAP.md` (item 92), `docs/archive/TEXT_INDEX_PHASE_1_CLOSEOUT.md` (item 87, dir vacío), `seguimiento de proyecto.csv` (item 86). SDK boundary ref incorrecta (`src/sdk.rs` → `src/sdk/`).
- **Contrato:** "REPO_CHECKLIST.md no referencia archivos inexistentes. SDK path corregido a `src/sdk/`."
- **Task file:** `tasks/OPS-11.md`
- **Estado:** ✅ COMPLETED

---

### Task OPS-12: Fix DISASTER_RECOVERY_RUNBOOK.md — env var incorrecta

- **Archivos clave:** `docs/operations/DISASTER_RECOVERY_RUNBOOK.md`, `src/config.rs`
- **Gate Justificación:** `VANTADB_MEMORY_LIMIT` env var no existe. `memory_limit` solo se configura por constructor o VantaConfig, no por env var directa.
- **Contrato:** "DISASTER_RECOVERY_RUNBOOK.md no referencia env vars que no existen."
- **Task file:** `tasks/OPS-12.md`
- **Estado:** ✅ COMPLETED

---

### Task OPS-13: Fix PILOT_PROGRAM.md — Python constructor args incorrectos

- **Archivos clave:** `docs/operations/PILOT_PROGRAM.md`, `vantadb-python/src/lib.rs`
- **Gate Justificación:** `vantadb_py.VantaDB(DB_PATH, distance_metric="cosine")` — `distance_metric` NO es parámetro del constructor. Constructor acepta `(db_path, memory_limit_bytes=None, read_only=False, backend=None)`.
- **Contrato:** "PILOT_PROGRAM.md usa constructor Python con argumentos reales."
- **Task file:** `tasks/OPS-13.md`
- **Estado:** ✅ COMPLETED

---

### Task OPS-14: Fix DURABILITY_GUARANTEES.md — line refs desactualizadas

- **Archivos clave:** `docs/operations/DURABILITY_GUARANTEES.md`, `src/storage/engine/`
- **Gate Justificación:** Line references apuntan a `src/storage.rs:1721-1749`, `:622-690`, `:745-768` — el código ahora está en `src/storage/engine/`. Esas líneas pueden no corresponder.
- **Contrato:** "Line references en DURABILITY_GUARANTEES.md verificadas contra código real en `src/storage/engine/`."
- **Task file:** `tasks/OPS-14.md`
- **Estado:** ✅ COMPLETED

---

### Task OPS-15: Fix BACKUP_POLICY.md — texto duplicado

- **Archivos clave:** `docs/operations/BACKUP_POLICY.md`
- **Gate Justificación:** Líneas 34-37 tienen texto duplicado ("Cold-copy restore is now part of the fast validation suite for the default" aparece dos veces).
- **Contrato:** "No hay texto duplicado en BACKUP_POLICY.md."
- **Task file:** `tasks/OPS-15.md`
- **Estado:** ✅ COMPLETED

---

### Task OPS-16: Fix MEMORY_TELEMETRY.md — métricas Prometheus faltantes

- **Archivos clave:** `docs/operations/MEMORY_TELEMETRY.md`, `src/metrics/core/registry.rs`
- **Gate Justificación:** Lista de Prometheus metrics incompleta: faltan `vanta_volatile_cache_entries`, `vanta_volatile_cache_cap_bytes`, y 7 gauges `vanta_jemalloc_*`.
- **Contrato:** "MEMORY_TELEMETRY.md lista todas las métricas Prometheus registradas en `src/metrics/core/registry.rs`."
- **Task file:** `tasks/OPS-16.md`
- **Estado:** ✅ COMPLETED

---

### Task OPS-17: Fix AGENT_INSTRUCTIONS.md — claim C-ABI incorrecto

- **Archivos clave:** `docs/operations/AGENT_INSTRUCTIONS.md`
- **Gate Justificación:** Doc afirma que `src/engine.rs` es capa C-ABI. Realidad: `src/engine.rs` es coordinador in-memory, el binding Python usa PyO3 (Rust FFI), no C-ABI. No hay `#[no_mangle]` ni `extern "C"`.
- **Contrato:** "AGENT_INSTRUCTIONS.md no describe `src/engine.rs` como C-ABI layer."
- **Task file:** `tasks/OPS-17.md`
- **Estado:** ✅ COMPLETED

---

### Task OPS-18: Resolver contradicción IQL/LISP entre EXPERIMENTAL_FEATURES.md y FUZZING.md

- **Archivos clave:** `docs/operations/EXPERIMENTAL_FEATURES.md`, `docs/operations/FUZZING.md`, `vantadb-mcp/src/lib.rs`
- **Gate Justificación:** EXPERIMENTAL_FEATURES.md marca IQL/LISP como "Archived". FUZZING.md referencia target `fuzz_parser` para LISP parser. MCP server tiene tool `query_lisp` activa.
- **Acción:** Decidir estado real y hacer que ambos docs coincidan.
- **Contrato:** "EXPERIMENTAL_FEATURES.md y FUZZING.md no tienen claims contradictorios sobre estado de IQL/LISP."
- **Task file:** `tasks/OPS-18.md`
- **Estado:** ✅ COMPLETED

---

### Task OPS-19: Fix BENCHMARKS.md — nota de ~ inconsistente

- **Archivos clave:** `docs/operations/BENCHMARKS.md`
- **Gate Justificación:** Nota al pie dice "Values marked with `~` are approximations" pero ningún valor en las tablas usa `~`.
- **Contrato:** "BENCHMARKS.md nota sobre `~` coincide con el contenido de las tablas."
- **Task file:** `tasks/OPS-19.md`
- **Estado:** ✅ COMPLETED

---

### Task OPS-20: Actualizar master-index.md — agregar 7 archivos faltantes

- **Archivos clave:** `docs/master-index.md`
- **Gate Justificación:** 7 archivos de docs/operations/ no están listados en la sección "Operations & Configuration":
  - DISASTER_RECOVERY_RUNBOOK.md
  - DEPLOYMENT_GUIDE.md
  - GC_TTL.md
  - PERFORMANCE_GUIDE.md
  - PERFORMANCE_TUNING.md
  - SECURITY.md
  - SQLITE_MIGRATION_GUIDE.md
- **Contrato:** "docs/master-index.md lista los 25 archivos de docs/operations/ en su sección Operations & Configuration."
- **Task file:** `tasks/OPS-20.md`
- **Estado:** ✅ COMPLETED

---

### Post-Condición

- Score docs/operations/ sube de 5.9/10 a ~9.0/10
- No quedan line references desactualizadas
- No quedan flags/features/env vars documentados incorrectamente
- No quedan archivos referenciados que no existen
- No quedan nombres de workflow/binary incorrectos
- master-index.md refleja todos los archivos de docs/operations/
- Grafana dashboard contiene métricas reales con prefijo `vanta_`

### Recomendaciones estructurales adicionales (fuera de scope de este plan)

1. **Consolidar**: PERFORMANCE_GUIDE.md + PERFORMANCE_TUNING.md → un solo PERFORMANCE.md (~30pp → ~20pp)
2. **Mover**: AGENT_INSTRUCTIONS.md → `docs/agent/` (no es operacional)
3. **Mover**: COMMUNITY_GOVERNANCE.md → `docs/community/` (no es operacional)
4. **Mover**: PUBLIC_ISSUE_DRAFTS.md → `docs/` raíz (es policy)
5. **Eliminar o archivar**: PILOT_PROGRAM.md (si el programa terminó)
6. **Crear nuevo**: `docs/operations/TROUBLESHOOTING.md` — errores comunes de operación
7. **Crear nuevo**: `docs/operations/LOGGING.md` — configuración de logging (separado de CONFIGURATION.md)
8. **Verificar**: `docs/references/troubleshooting.md` existe en otra rama del árbol — consolidar

---

## Recitation

=== RECITATION ===
Objetivo activo: Fix EXPERIMENTAL_FEATURES.md + FUZZING.md cross-references and deprecation labels
Estado: plan
Última acción: Applied 4 edits across both files
Resultado: ✅
State: PLAN
Próxima acción: None
Contrato: "Plan file existe en docs/plans/ con 20 tasks definidas, priorizadas en 3 waves"
Próxima tarea si completa: 
last-synced: 2026-07-21T00:00
=== END RECITATION ===
