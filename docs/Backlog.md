---
title: "Active Backlog — VantaDB"
type: backlog-tracking
status: active
tags: [vantadb, backlog, engineering, phases, priorities]
last_reviewed: 2026-07-16
---

# Active Backlog — VantaDB

> **Purpose:** Single source of truth for all project tasks.
> **Completed tasks:** `docs/CHANGELOG.md` + `docs/progreso/README.md`
> **Verification method:** All claims cross-checked against actual codebase via 4 sub-agents (Jul 13). See `docs/archive/` for superseded audit reports.
> **Total open items:** 165 (108 previos + 22 del rescate OLD Jul 16 + 5 cross-ref Wave 3 + 30 competitive features COMP Jul 16)
> **Origen docs-audit:** `docs/strategy/ROADMAP.md`, `docs/progreso/bitacora.md`, `docs/reviews/FULL_CODEBASE_AUDIT_2026-07-11.md`, `docs/reviews/analisis_proyecto.md`, `docs/operations/PERFORMANCE_TUNING.md`, `docs/operations/REPO_CHECKLIST.md`, `docs/architecture/STORAGE_VERSIONING.md`, `docs/plans/2026-07-13-workflow-repair-campaign.md`, `docs/Investigaciones/cargo-check-optimizacion.md`, `docs/discord/todo.md`

---

## TIER 0 — 🔴 Bloqueantes de Release

> Items que bloquean cualquier release seguro.

### 📦 Publicación de Integraciones

| ID | Tarea | Esfuerzo | Prioridad | Estado | Verificación |
|----|-------|----------|-----------|--------|-------------|
| `INT-01` | **LangChain adapter → PyPI** | 🟡 1-2d | 🔴 | ✅ | Código existe, CI configurado (`release-adapters-62.yml`), 5/5 tests pasan. Push tag `adapters-v0.3.0` para publicar. |
| `INT-02` | **LlamaIndex adapter → PyPI** | 🟡 1-2d | 🔴 | ✅ | Código existe, CI configurado (`release-adapters-62.yml`), 5/5 tests pasan. Push tag `adapters-v0.3.0` para publicar. |
| `DEVOPS-05` | Pipeline CI unificado para publicar los 9 adapters a PyPI | 🟡 1-2d | 🔴 | ✅ | `release-adapters-62.yml` existente: test → build → TestPyPI (dispatch) → PyPI (tag `adapters-v*`). Los 9 adapters en `integrations/` build correctos. |
| `REL-02` | **Publicar `vantadb-ts` en npm** (WASM build) | 🟡 1-2d | 🔴 | ⏳ | 3 cambios aplicados: (1) `impl_text_index.rs` visibility fix (`fn` → `pub(crate)` en 2 métodos), (2) `wasm-opt = false` en `vantadb-wasm/Cargo.toml` (toolchain local no soporta bulk-memory), (3) CI `release-npm-61.yml` fix: `ts-v*` tag ahora ejecuta `publish-wasm`. Build WASM ✅, Build TS ✅, npm dry-run ✅. Tests: ⚠️ 80/219 fail (pre-existing WASM panics `unreachable!` en Node.js, pasan 113, 26 skip — bug no relacionado a REL-02). npm names `vantadb` + `vantadb-wasm` disponibles. |

### 🌐 Web & Landing

| ID | Tarea | Esfuerzo | Prioridad | Estado | Verificación |
|----|-------|----------|-----------|--------|-------------|
| `MKT-13` | **Enlazar demo WASM desde la hero** — Ruta `/demo` existe, demo funcional. Falta botón "Try in browser" en `NbTerminalHero` | 🟡 1-2h | 🔴 | ⏳ | `NbTerminalHero.tsx` no tiene link a `/demo`. Verificado. |

---

## TIER 1 — 🟠 Pre-Lanzamiento

> Necesario ANTES del Show HN.

### 📖 Documentación & Community

| ID | Tarea | Esfuerzo | Prioridad | Estado | Verificación |
|----|-------|----------|-----------|--------|-------------|
| `MKT-14` | **Publicar 2 case studies** + ruta `/case-studies/` | 🟡 1-2d | 🔴 | ❌ | `docs/case_studies/` drafts existen, no desplegados |
| `TSK-106` | **Habilitar GitHub Discussions** | 🟢 1h | 🟠 | ❌ | No verificable desde repo local |
| `NUEVO-01` | **README hero** con readme-aura + benchmark gráfico + GIF demo WASM | 🟡 2-3d | 🟠 | ❌ | No implementado |
| `NUEVO-07` | **Migration tools: Chroma→Vanta, LanceDB→Vanta** | 🟡 3-5d | 🟠 | ❌ | No existen scripts de migración automatizados |
| `NUEVO-08` | **Learning path estructurado** en tutorials/ (5-7 ejemplos) | 🟡 2-3d | 🟠 | ❌ | Tutorials existen (3) pero sin progresión clara |
| `NUEVO-10` | **Benchmark suite pública reproducible** | 🟡 3-5d | 🟠 | ❌ | Benchmarks internos existen, no hay publish |
| `TSK-107` | Community showcase page | 🟢 4-6h | 🟡 | ❌ | No existe |
| `—` | Good first issues (20+ tagged) | 🟢 2-4h | 🟠 | ❌ | No hay issues etiquetados |

### 🚀 Launch Campaign

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `LEG-01` | **Registrar trademark "VantaDB" (USPTO + EUIPO)** | 🟡 2-4h | 🔴 | ❌ |
| `MKT-03` | **Show HN post** | 🟢 2h | 🔴 | ❌ |
| `MKT-04` | Reddit posts (r/rust, r/MachineLearning, r/LocalLLaMA) | 🟢 2-4h | 🟠 | ❌ |
| `MKT-05` | Technical blog posts (5+ pre-launch) | 🟡 2-3d | 🟠 | ❌ |
| `MKT-10` | "AI Agent Memory" campaign | 🟡 2-3d | 🟠 | ❌ |
| `MKT-15` | **Página de benchmarks competitivos** (`/product/benchmarks`) | 🟡 2-3d | 🔴 | ❌ |
| `MKT-16` | **Publicar metodología de benchmark GraphRAG** | 🟡 1-2d | 🟡 | ❌ |
| `TSK-103` | Public benchmark site | 🟡 2-3d | 🟠 | ❌ |
| `TSK-104` | Demo agent: LangChain + Ollama + VantaDB | 🟡 1-2d | 🟠 | ❌ |
| `DEVOPS-12` | **Production PyPI signing pipeline** (OIDC + Sigstore) | 🟡 1-2d | 🟡 | ❌ |
| `DEVOPS-10` | **Firma de binarios Windows (SmartScreen)** | 🟡 2-3d | 🟢 | ❌ |

### 🌐 Conversión y SEO

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `MKT-17` | Página de comparación competitiva interactiva | 🟡 2-3d | 🟢 | ❌ |

### 🚨 Hallazgos de Docs Audit (nuevos)

> Items verificados como pendientes en docs/ pero no trackeados previamente. Referencia: `docs/progreso/bitacora.md`, `docs/reviews/FULL_CODEBASE_AUDIT_2026-07-11.md`.

| ID | Tarea | Origen | Esfuerzo | Prioridad | Estado |
|----|-------|--------|----------|-----------|--------|
| `SEC-13` | **CSP unsafe-inline en prod + HSTS + nonce system** — Sin nonce, `style-src 'unsafe-inline'`, `/metrics` endpoint público sin auth | bitacora P12, CSP2/CSP3, W6 | 🟡 1-2d | 🔴 | ✅ |
| `SEC-14` | **Evaluar migrar bincode → postcard/rkyv** — Crate no mantenido desde 2021, propuesto en STORAGE_VERSIONING.md | `docs/architecture/STORAGE_VERSIONING.md:100` | 🟡 1d | 🟠 | ❌ |
| `WEB-02` | **Corregir claims falsos en landing** — Benchmarks web (50x vs real 40x), mención "SQL support", "auto-embeddings", "cloud tiers" sin infraestructura | bitacora W1–W4 | 🟡 2-3d | 🔴 | ❌ |
| `WEB-03` | **Async WAL batching fsyncs** — Recomendado en PERFORMANCE_TUNING.md para alta throughput | `docs/operations/PERFORMANCE_TUNING.md:264` | 🟡 2-3d | 🟡 | ❌ |
| `WEB-04` | **Storage format versioning (draft→implement)** — STORAGE_VERSIONING.md Phases 1-3, sin migration path para VantaFile/HNSW/WAL | `docs/architecture/STORAGE_VERSIONING.md` | 🟠 3-5d | 🔵 | ❌ |
| `DEVOPS-13` | **Pin all workflow actions a SHA + Node 22** — 11 workflows sin SHA pinning, Node 20 deprecated | bitacora C1, plan repair campaign | 🟡 1-2d | 🟡 | ✅ |
| `DEVOPS-14` | **Extract composite action para Rust setup** — 5+ workflows duplican inline | bitacora C1 | 🟢 4h | 🟡 | ❌ |
| `DEVOPS-15` | **Mover features heavies fuera de default + consolidar deps duplicadas** — Optimización compilación workspace | `docs/Investigaciones/cargo-check-optimizacion.md` T5/T7 | 🟡 1-2d | 🟡 | ❌ |
| `TEST-11` | **Frontend tests (Vitest + Playwright)** + cross-browser WASM testing | bitacora T1/T4 | 🟡 2-3d | 🟡 | ❌ |
| `TEST-12` | **Security testing: fuzzing expand + regression/snapshot suite** — Solo parser fuzzed, sin regression gates | bitacora T2/T3 | 🟡 2-3d | 🟡 | ❌ |
| `DOC-20` | **mdBook adoption for docs site** — Docs fragmentados, sin search unificado, sin versioning | bitacora D1, D6 | 🟡 2-3d | 🟡 | ❌ |

### 🧪 Issues Técnicos Verificados (Nuevos)

> Items descubiertos durante la verificación cross-code del backlog. No estaban registrados previamente.

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `VFY-001` | **TS SDK `catch {}` silencia errores** — 4+ bloques catch vacíos | `vantadb-ts/src/vantadb.ts:176,215,249` | 🟢 2h | 🟡 | ❌ |
| `VFY-002` | **`get_nns_by_id` spawn por llamada** — Sin batching | `vantadb-ts/src/vantadb.ts:325` | 🟢 2h | 🟢 | ❌ |
| `VFY-003` | **`reindex_hnsw_from_text` riesgo OOM** — Sin batch processing | `vantadb-python/src/lib.rs:1584` | 🟡 1d | 🟡 | ❌ |
| `VFY-004` | **`flat.rs` O(n²) en filter** — Sin índice para filtros | `src/index/flat.rs:32` | 🟡 1-2d | 🟡 | ❌ |
| `VFY-005` | **TS `OperationalMetrics` 70% incompleto** — 3 de 10+ métricas mapeadas | `vantadb-ts/src/types.ts:148-168` | 🟢 4h | 🟢 | ❌ |
| `VFY-006` | **`add_node` escribe lock durante toda inserción** | `src/index/graph.rs:476-490` | 🟡 1-2d | 🟡 | ❌ |
| `VFY-007` | **`remove_node` O(n²) neighbor fixup** — Deletes costosos | `src/index/core.rs` | 🟡 1-2d | 🟢 | ❌ |
| `VFY-008` | **WAL fsync por escritura** — Write amplification | `src/storage/wal.rs` | 🟡 1-2d | 🟡 | ❌ |
| `VFY-009` | **637 inline styles no migrados a Tailwind** | `web/src/` | 🟡 3-5d | 🟢 | ❌ |
| `VFY-010` | **ACID Phase 2: Buffered write transactions** — No implementado | `src/wal.rs` | 🟡 2-3d | 🔵 | ❌ |
| `VFY-011` | **ACID Phase 3: Snapshot isolation / MVCC** | — | 🟠 3-5d | 🔵 | ❌ |
| `VFY-012` | **DEVOPS-03: musllinux target gap** — Algunos targets sin soporte | CI config | 🟢 4h | 🟢 | ❌ |

### 🔍 Hallazgos del Full Review (2026-07-13)

> Items descubiertos durante `vantadb-full-review`. Referencia: `docs/reviews/PROJECT_FULL_REVIEW_2026-07-13.md`.

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `RC1-RC4` | **23 `.expect("RwLock/Mutex poisoned")` en governance/ + vector/governor.rs** — Ya resuelto: helpers `RwLockExt`/`MutexExt` existen en `src/sync_ext.rs`, governance/ ya los usa. No requiere acción | `src/sync_ext.rs`, `src/governance/*`, `src/vector/governor.rs` | 🟢 N/A | ✅ | ✅ |
| `RC5` | **Mejorar mensaje de `Aes256Gcm::new_from_slice().expect()`** — Mensaje genérico, incluir razón técnica en panic | `src/crypto.rs:104` | 🟢 15min | 🟡 | ❌ |
| `RC6` | **Evaluar propagar `CryptoError` desde `encrypt()` vs mantener expect** — 45+ call sites vía `EncryptionStream`. Decidir si vale la ruptura | `src/crypto.rs:126` | 🟡 1d | 🟡 | ❌ |
| `RC7` | **`.expect("GovernorConfig build failed")`** — Startup path fatal, abort intencional. **Ponytail: keep as-is** | `src/cli_server.rs:139` | 🟢 N/A | ℹ️ | — |
| `RC8` | **`auth_middleware` .expect("keys")** — Middleware debe devolver 401 en vez de panic cuando el invariante se viola | `src/cli_server.rs:758` | 🟢 2h | 🟡 | ❌ |
| `RC9` | **SystemTime::duration_since** — Ya tiene `.unwrap_or_default()` en L32. **Ya resuelto** | `src/binary_header.rs:32` | 🟢 N/A | ✅ | ✅ |
| `RC10` | **`.expect("reqwest blocking client")`** — Startup-only, fatal abort aceptable. **Ponytail: keep as-is** | `src/wal_shipping.rs:78` | 🟢 N/A | ℹ️ | — |
| `RC11` | **`ClientEngine::default().expect()`** — Sin engine = sin bindings, abort intencional. **Ponytail: keep as-is** | `src/python.rs:21` | 🟢 N/A | ℹ️ | — |

### 🔍 Hallazgos del Review Deep — SDK (DRV)

> Items descubiertos durante `review-deep` del módulo `vantadb-sdk` (Wave 0). Referencia: `.opencode/skills/review-deep/`.

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-001` | **search.rs: 1085L god file** — Contiene BM25 scoring, snippet generation, hybrid fusion, RRF, debug explain. 4+ responsabilidades en un solo impl block. `#[allow(clippy::type_complexity)]` en L642 | `src/sdk/search.rs:1-1085` | 🟡 2-3d | 🟡 | ❌ |
| `DRV-002` | **put_batch duplica lógica de put()** — ~40 líneas idénticas (validación, node_id collision, timestamp, version). DRY violation | `src/sdk/api.rs:117-193` | 🟢 1d | 🟢 | ❌ |
| `DRV-003` | **purge_expired llama replace_derived_indexes por nodo** — O(n) index rebuilds en loop. Si purga 10K registros, hace 10K rebuilds | `src/sdk/api.rs:380-383` | 🟢 2h | 🟡 | ❌ |
| `DRV-004` | **list() carga ALL records a memoria antes de paginar** — `records_for_namespace()` devuelve todo. Namespace con 100K+ registros → OOM | `src/sdk/api.rs:296-315` | 🟡 1d | 🟡 | ❌ |
| `DRV-005` | **SDK sin unit tests** — No hay `#[cfg(test)]` en api.rs, search.rs, types.rs. Solo integration tests en `tests/`. Edge cases de validación sin cobertura | `src/sdk/` | 🟡 1-2d | 🟡 | ❌ |

### 🔍 Hallazgos del Review Deep — Engine (DRV)

> Items descubiertos durante `review-deep` del módulo `vantadb-engine` (Wave 0). Referencia: `.opencode/skills/review-deep/`.

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-006` | **Race condition en `delete()`: write lock dropped antes de index cleanup** — `drop(nodes)` libera `nodes.write()` L241, luego actualiza `edge_index` y `scalar_index` sin protección. Ventana donde un `insert` concurrente con mismo ID target puede interleaver, corrompiendo índices | `src/engine.rs:235-248` | 🟢 30min | 🔴 | ✅ |
| `DRV-007` | **Data race en `filter_field()`: accede `scalar_index` sin lock** — No adquiere el `nodes` RwLock. Mutaciones concurrentes (insert/update/delete) acceden a `scalar_index` bajo `nodes.write()`, pero `filter_field` no. Comportamiento indefinido | `src/engine.rs:354` | 🟢 30min | 🟡 | ✅ |
| `DRV-008` | **Duplicate scoring pipeline en `vector_search()` y `hybrid_search()`** — ~25 líneas idénticas (sort_by, truncate, collect, QueryResult build). DRY violation entre L288-305 y L399-413 | `src/engine.rs:288-305,399-413` | 🟢 1h | 🔵 | ❌ |
| `DRV-009` | **`node_count()` O(n) full scan bajo read lock** — itera todos los nodos vivos cada vez. 1M nodos → 1M iteraciones por cada `node_count()`. Sin contador cacheado | `src/engine.rs:424-426` | 🟢 1h | ⚪ | ❌ |
| `DRV-010` | **63 `unwrap()` en tests** — Todos en `#[cfg(test)]`, aceptable para test helpers. Solo documentar como deuda de estilo | `src/engine.rs:460-932` | 🟢 N/A | ℹ️ | ❌ |

### 🔍 Hallazgos del Review Deep — WAL (DRV)

> Items descubiertos durante `review-deep` del módulo `vantadb-wal` (Wave 0). Referencia: `.opencode/skills/review-deep/`.

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-011` | **Scan-forward recovery duplicado en WalWriter y WalReader** — ~40 líneas de algoritmo byte-scan idéntico para localizar el siguiente registro válido tras corrupción. DRY violation entre open_with_buffer L287-332 y next_record L593-630 | `src/wal.rs:287-332,593-630` | 🟢 2h | 🔵 | ❌ |
| `DRV-012` | **`append()` y `batch_append()` duplican lógica de sync** — Las líneas L390-396 y L430-436 son idénticas (check de sync_mode + flush_threshold). Podría extraerse a fn `maybe_sync()` | `src/wal.rs:390-396,430-436` | 🟢 30min | ⚪ | ❌ |
| `DRV-013` | **ShardedWal sin unit tests** — `src/wal_sharded.rs` no tiene `#[cfg(test)]`. 168 líneas de lógica concurrente (locks, round-robin, batch_append, rotate_all) sin cobertura directa | `src/wal_sharded.rs` | 🟡 4h | ⚪ | ❌ |
| `DRV-014` | **ShardedWal::batch_append() clona todos los records por shard** — Llama `record.clone()` en L88 para cada elemento, clonando `UnifiedNode`. Para batches grandes (>1000), overhead de alloc significativo | `src/wal_sharded.rs:85-89` | 🟢 2h | ℹ️ | ❌ |
| `DRV-015` | **`WalWriter::open_with_buffer()` función monolítica de 170L** — Mezcla apertura de archivo + validación de header + recovery scanning + truncation. 2+ responsabilidades, dificulta testeo unitario | `src/wal.rs:201-367` | 🟢 1d | ℹ️ | ❌ |

### 🔍 Hallazgos del Review Deep — Vector (DRV)

> Items descubiertos durante `review-deep` del módulo `vantadb-vector` (Wave 0). Referencia: `.opencode/skills/review-deep/`.

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-016` | **Inconsistencia de Mutex: `governor.rs` usa `std::sync::Mutex` en vez de `parking_lot`** — Todo el codebase usa parking_lot (no poison, no unwrap). governor.rs usa std Mutex con `MutexExt` que hace `.lock().unwrap()`. Si el lock se envenena, paniquea. Refactor trivial | `src/vector/governor.rs:94,111,147,160` | 🟢 30min | 🔵 | ❌ |
| `DRV-017` | **`search.rs` (416L) y `serialize.rs` (615L) sin tests unitarios** — Lógica de búsqueda HNSW con mmap zero-copy y serialización/deserialización no tienen cobertura directa. Solo tests de integración en `core.rs` | `src/index/search.rs`, `src/index/serialize.rs` | 🟡 1d | ⚪ | ❌ |
| `DRV-018` | **`refresh.rs` es stub vacío (4L)** — Archivo planeado para background refresh pero solo contiene un comment. Sin implementación, sin tests | `src/index/refresh.rs` | 🟢 N/A | ℹ️ | ❌ |
| `DRV-019` | **14 `.expect()` en hot-path SIMD loops en `distance.rs`** — En `cosine_sim_f32`, `euclidean_distance_squared_f32`, etc. Correctos (chunks_exact garantiza tamaño) pero overhead en cada chunk del loop. Preferir `unreachable_unchecked` para 0 overhead | `src/index/distance.rs:97,100,129,132,204,207,238,241,268,271,297,300,379,420` | 🟢 1h | ℹ️ | ❌ |
| `DRV-020` | **`serialize.rs:21` — `unwrap()` en producción en `serialize_to_bytes()`** — Write a Vec, no puede fallar realmente, pero rompe convención del proyecto de no usar unwrap en prod. Refactor a `.expect("Vec write")` o usar `?</* ! */` | `src/index/serialize.rs:21` | 🟢 5min | ℹ️ | ❌ |

### 🔍 Hallazgos del Review Deep — Index (DRV)

> Items descubiertos durante `review-deep` del módulo `vantadb-index` (Wave 0). Referencia: `.opencode/skills/review-deep/`.

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-021` | **`#[allow(dead_code)]` en `tokenize()` y `tokenize_with_spec()`** — 2 funciones convenience envueltas que no se usan directamente (el código llama `*_with_config`). Sin impacto, pero muerto | `src/text_index.rs:153,159` | 🟢 N/A | ℹ️ | ❌ |

### 🔍 Hallazgos del Review Deep — Governance (DRV)

> Items descubiertos durante `review-deep` del módulo `vantadb-governance` (Wave 0). Referencia: `.opencode/skills/review-deep/`.

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-022` | **`governance/` completo (1235L) gated tras feature no-default sin consumidores** — 4 módulos (admission, conflict, consistency, worker) bajo `#[cfg(feature = "governance")]` pero ningún crate externo ni módulo interno importa `AdmissionFilter`, `ConflictResolver`, etc. Feature existe como flag vacío, nunca activado. Adicionalmente depende de `sync_ext` que hace compilación inviable incluso si se activara | `src/governance/` | 🟢 30min | 🔵 | ❌ |
| `DRV-023` | **`ResourceGovernor` + `ALLOCATED_BYTES` sin callers** — `governor.rs` exporta struct y static global, pero cero referencias fuera del archivo. Ni query engine ni execution path lo usan | `src/governor.rs` | 🟢 15min | 🔵 | ❌ |
| `DRV-024` | **`memory_governor.rs` muerto con `#![allow(dead_code)]`** — Todo el archivo tiene dead_code explicitamente permitido. `pub(crate)` pero nada en el crate lo invoca | `src/memory_governor.rs:1` | 🟢 N/A | ℹ️ | ❌ |
| `DRV-025` | **TOCTOU race en `ResourceGovernor::request_allocation()`** — `ALLOCATED_BYTES.load(Ordering::Relaxed)` L41 + `fetch_add` L57 sin CAS. Dos threads concurrentes pueden pasar el check OOM y sobre-asignar `2x` del límite. El OOM guard no protege bajo concurrencia | `src/governor.rs:41-57` | 🟢 30min | 🟡 | ❌ |
| `DRV-026` | **Redundant `unwrap()` en `three_way_merge()`** — `ours_val.unwrap()` y `theirs_val.unwrap()` en L272-273 cuando el match ya garantiza `Some`. Código correcto pero redundante | `src/governance/conflict.rs:272-273` | 🟢 5min | ℹ️ | ❌ |

### 🔍 Hallazgos del Review Deep — Python SDK (DRV)

> Items descubiertos durante `review-deep` del módulo `vantadb-python` (Wave 0). PyO3 bindings, 3077L total (src/ + tests/). Referencia: `.opencode/skills/review-deep/`.

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-027` | **God module en lib.rs (1978L)** — 90 funciones en un archivo: VantaDB pyclass (~35 métodos), 19 conversores report-to-PyDict, error mapping, LRU cache, extract_vector, py_any_to_value, módulo setup. 6 pyclasses exportadas. Todos los helpers de conversión (`*_to_pydict`) son funciones libres en el mismo archivo — candidatos a módulo separado `src/convert.rs` | `vantadb-python/src/lib.rs` | 🟡 1d | 🟡 | ❌ |
| `DRV-028` | **Hand-rolled LRU cache con O(n) por operación** — `LruCache` usa `Vec<String>` para tracking de orden (L38-88). Cada `get()` y `put()` recorre el vector con `.position()` (O(n)). Para capacity=64 es ~32 comparaciones promedio. `indexmap` o `linked_hash_map` darían O(1). `ponytail:` — funciona a escala actual, reemplazar si throughput requiere | `vantadb-python/src/lib.rs:38-88` | 🟢 30min | ⚪ | ❌ |
| `DRV-029` | **Cache-key overhead en py_dict_to_metadata** — Construye string serializado + sorted de todo el dict (L609-635) solo para verificar cache hit. Para dicts ≤4 entradas, la serialización + sort probablemente cuesta más que construir el BTreeMap directamente. Hit solo en llamadas subsecuentes con contenido idéntico | `vantadb-python/src/lib.rs:602-662` | 🟢 15min | ℹ️ | ❌ |
| `DRV-030` | **19 conversores report-to-PyDict duplicados (~280L)** — Funciones como `operational_metrics_to_pydict`, `search_explanation_to_pydict`, etc. (L306-599) iteran campos manualmente uno a uno con `dict.set_item()`. Boilerplate mecánico refactorizable vía macro con `#[derive(IntoPyDict)]` | `vantadb-python/src/lib.rs:306-599` | 🟡 1d | ℹ️ | ❌ |
| `DRV-031` | **Comentario doc duplicado** — `/// Put or update a namespace-scoped persistent memory record.` aparece dos veces (L1140-1142) | `vantadb-python/src/lib.rs:1140-1142` | 🟢 2min | ℹ️ | ❌ |
| `DRV-032` | **4 métodos con #[allow(clippy::too_many_arguments)]** — `put_batch` (8 params), `put_batch_raw` (6), `put` (6), `explain_memory_search` (6). `put_batch` además implementa dos API (tuple-list legacy + keyword moderna) en 130L (L835-966) duplicando lógica batch | `vantadb-python/src/lib.rs:976,1143,1234,1652` | 🟢 2h | ℹ️ | ❌ |
| `DRV-033` | **✅ Sin unsafe, unwrap ni expect en producción** — 0 `unsafe`, 0 `unwrap()`, 0 `.expect()` en producción. 34/34 `py.detach()` envuelven GIL release correctamente. Navegación segura del GIL | `vantadb-python/src/lib.rs` | 🟢 N/A | ✅ | ❌ |

### 🔍 Hallazgos del Review Deep — TypeScript SDK (DRV)

> Items descubiertos durante `review-deep` del módulo `vantadb-ts` (Wave 0). WASM-powered TypeScript SDK, 4 src files + 6 test files (~92KB). Referencia: `.opencode/skills/review-deep/`.

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-034` | **30+ try-catch blocks repetidos siguiendo el mismo patrón** — Cada método público sigue el molde `_assertOpen() → try { this.inner.X() } catch { wrapWasmError(e, "X") }`. Defensiva pero repetitiva: 30 instancias de lógica idéntica que podría abstraerse via decorador/función wrapper. `ponytail:` — funciona, refactor si la tax de catch crece | `vantadb-ts/src/vantadb.ts` (todo el archivo) | 🟡 1d | ⚪ | ❌ |
| `DRV-035` | **Type mismatch: metadata usa formato shorthand en tests vs tipo definido** — Los tests usan `metadata: { source: { String: "test" } }` (ej: L355 vanta.test.ts, L210 hardening.test.ts), pero `VantaValue` define formato `{ type: "String", value: "test" }`. Los tests pasan (solo verifican `r.payload`, no `r.metadata`), y `tsconfig` excluye `__tests__/` del typecheck. Bug dormido: metadatos no se serializan correctamente a través del bridge WASM | `vantadb-ts/src/__tests__/*.test.ts` + `src/types.ts:1-10` | 🟢 30min | 🟡 | ❌ |
| `DRV-036` | **`_mapRecord` valida 3 campos pero retorna `as MemoryRecord` sin validar el resto** — Verifica namespace, key, payload son strings (L25-51) pero no valida metadata, vector, timestamps, node_id del objeto devuelto por la capa WASM. `as MemoryRecord` omite cualquier verificación en los campos restantes | `vantadb-ts/src/vantadb.ts:25-52` | 🟢 1h | ⚪ | ❌ |
| `DRV-037` | **`types.test.ts` usa types incorrectos que tsc no detecta** — `created_at_ms: 1000` (number) vs type `string`, `node_id: 42` (number) vs type `string`. Pasa porque `tsconfig` excluye `__tests__/`. Si se incluyeran tests en el typecheck, romperían compilación | `vantadb-ts/src/__tests__/types.test.ts:11-14` | 🟢 15min | ⚪ | ❌ |
| `DRV-038` | **TypeScript numeric fields tipados como `string` inconsistentes con otros SDKs** — `created_at_ms`, `updated_at_ms`, `version`, `node_id` son `string` en TS pero `u64` en Rust/Python. El bridge WASM convierte a string para JSON, pero obliga a conversión `Number()` en el consumidor TS | `vantadb-ts/src/types.ts:28-31` | 🟢 1h | ℹ️ | ❌ |
| `DRV-039` | **No ESLint config presente** — `vantadb-ts/` no tiene `.eslintrc.*`. Sin linting estático de TS. Tests usan `any` (dx04.test.ts, load.test.ts) sin detección | `vantadb-ts/` | 🟢 30min | ℹ️ | ❌ |
| `DRV-040` | **`unsafe` en simd.rs sin `// SAFETY:` comment** — L34-77: bloque `unsafe` con `v128_load` requiere punteros alineados a 16 bytes. `Vec<f32>` garantiza alineación, pero sin SAFETY docs el invariante no es verificable en code review. 1 bloque unsafe (L34) sin documentación | `vantadb-wasm/src/simd.rs:34-77` | 🟢 30min | 🟡 | ❌ |
| `DRV-041` | **`worker.rs` Promise constructor con inline JS string** — L201-209: `js_sys::Function::new_no_args` con string JS crudo + `arguments[0]`/`arguments[1]`. El callback `_reject` nunca se invoca: el Promise cuelga para siempre si el mensaje nunca llega. Response parsing vía `serde_json::from_str` (L229) agrega round-trip JSON innecesario vs `serde_wasm_bindgen::from_value` | `vantadb-wasm/src/worker.rs:201-229` | 🟢 2h | 🔵 | ❌ |
| `DRV-042` | **Test duplicación entre `lib.rs` mod tests y `tests/wasm_tests.rs`** — ~15 tests idénticos (put/get, delete, batch, capabilities, flush/compact, list_namespaces, search_without_results, large_metadata, concurrent_put_get) repartidos en 207L de tests en `lib.rs` + 751L en `wasm_tests.rs`. Mantenimiento duplicado | `vantadb-wasm/src/lib.rs:901-1107`, `vantadb-wasm/tests/wasm_tests.rs` | 🟢 1h | ⚪ | ❌ |
| `DRV-043` | **Core crate compilation errors bloquean `cargo check -p vantadb-wasm`** — `ensure_text_index_current_with` y `adjust_text_index_state_after_replace` son privados en `impl_text_index.rs` pero llamados desde `impl_index.rs`. 2 errores E0624. No afecta al wasm module per se, pero impide CI check del wasm crate | `vantadb/src/sdk/serialization/impl_index.rs:20,210` | 🟢 30min | 🟡 | ❌ |

### 🔍 Hallazgos del Review Deep — Server Binary (DRV)

> Items descubiertos durante `review-deep` del módulo `vantadb-server` (Wave 0). Binary wrapper thin crate (78L source, 4 files) — toda la lógica del servidor HTTP vive en `vantadb::cli_server`. Referencia: `.opencode/skills/review-deep/`.

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-044` | **MCP shutdown via `std::process::exit(0)` antes de que `run_stdio_server` reciba señal de parada** — `main.rs:46-55`: SIGTERM handler flushes storage y llama `exit(0)`, matando el proceso inmediatamente. `vantadb_mcp::run_stdio_server(storage)` (L57) nunca recibe señal de shutdown; in-flight JSON-RPC requests en stdin se pierden sin respuesta. Fix: usar `CancellationToken` para señalizar `run_stdio_server` primero, luego `return` de main en vez de `exit(0)` | `vantadb-server/src/main.rs:46-57` | 🟢 2h | 🔵 | ❌ |
| `DRV-045` | **Test setup factory duplicado en 3 test files** — `tests/server.rs::build_context`, `tests/e2e.rs::build_e2e_context`, `tests/benchmarks.rs::setup_bench` implementan el mismo patrón (tempdir + StorageEngine + ServerState) con ~15L cada uno. Algunos retornan `TestContext` struct, otros tuplas `(TempDir, Arc<ServerState>)`. Refactor a helper compartido reduciría ~40L de duplicación | `vantadb-server/tests/server.rs:26-39, tests/e2e.rs:52-66, tests/benchmarks.rs:23-55` | 🟢 30min | ⚪ | ❌ |

### 🔍 Hallazgos del Review Deep — MCP Protocol Interface (DRV)

> Items descubiertos durante `review-deep` del módulo `vantadb-mcp` (Wave 0). Custom MCP protocol sobre stdio sin librería externa. 1 source file `src/lib.rs` (1309L) + 1 test file `tests/mcp_tests.rs` (956L). `cargo fmt` OK; `cargo check` bloqueado por 2 errores de visibilidad en `vantadb::sdk::serialization` (DRV-043). Web research no disponible (timeouts). Referencia: `.opencode/skills/review-deep/`.

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-046` | **Blocking stdio I/O en tokio runtime impide graceful shutdown** — `run_stdio_server` (async fn) ejecuta `stdin.lock().lines()` sincrónicamente en el worker de tokio, bloqueando la ejecución de otras tareas async (incluyendo el handler de SIGINT). Ctrl+C termina el proceso via señal OS sin ejecutar shutdown graceful ni responder in-flight JSON-RPC requests. Fix: `spawn_blocking` para el loop de I/O o `tokio::io::AsyncBufReadExt::lines()` | `vantadb-mcp/src/lib.rs:320-384` | 🟢 2h | 🟡 | ❌ |
| `DRV-047` | **Hardcoded validation limits en handle_resources_read** — Líneas 549,553,575 usan literales `256` y `512` en vez de `config.max_namespace_length` / `config.max_key_length`. Consistencia: otros handlers usan `config.*`. Bajo impacto porque los default coinciden | `vantadb-mcp/src/lib.rs:549,553,575` | 🟢 15min | ⚪ | ❌ |
| `DRV-048` | **JSON-RPC 2.0 spec — versión no-2.0 descartada silenciosamente** — L359-363: si `req.jsonrpc != "2.0"`, se loggea warning y se hace `continue` sin enviar respuesta de error. La especificación (§7) dice que el servidor DEBE responder con un error de tipo invalid-request (-32600). El cliente nunca sabe que su request fue rechazado | `vantadb-mcp/src/lib.rs:359-363` | 🟢 30min | 🔵 | ❌ |
| `DRV-049` | **collection_delete no atómico** — Fetch all records via `collect_all_records`, luego delete one-by-one. Si el proceso crashea a mitad, el namespace queda parcialmente borrado. Sin transacción ni batch delete | `vantadb-mcp/src/lib.rs:1269-1305` | 🟢 1h | 🔵 | ❌ |
| `DRV-050` | **inject_context construye LISP query via string interpolation** — Naive escaping (`content.replace('\\', "\\\\").replace('"', "\\\"")`) antes de interpolación en query LISP via `format!`. No escapa paréntesis, newlines u otros metacaracteres LISP. Potencial injection vector si content no es confiable | `vantadb-mcp/src/lib.rs:1154-1187` | 🟢 1h | 🟡 | ❌ |
| `DRV-051` | **search_semantic N+1 query pattern** — Por cada hit de HNSW, llama `embedded.get_node()` individualmente (L1114-1122). Para top_k=1000, 1000 queries separadas. Optimización: batch get | `vantadb-mcp/src/lib.rs:1114-1122` | 🟢 1h | ⚪ | ❌ |
| `DRV-052` | **McpMetrics trackeadas pero nunca reportadas** — Struct `McpMetrics` (requests_total, errors_total, active_requests) solo se loggea una vez en shutdown. Sin endpoint `/metrics`, sin log periódico. Datos operacionales no visibles en runtime | `vantadb-mcp/src/lib.rs:161-166,386-390` | 🟢 1h | ℹ️ | ❌ |
| `DRV-053` | ❌ **DESCARTADO** — Duplicado de DRV-047 | — | — | — | — |
| `DRV-054` | **read_axioms hardcoded como JSON literal** — 4 axioms definidos inline en L1191-1198 como array JSON hardcoded. Si los axioms se actualizan en el metadata module/database, la copia MCP deriva. DRV: leer del metadata module o storage | `vantadb-mcp/src/lib.rs:1190-1198` | 🟢 30min | 🔵 | ❌ |
| `DRV-055` | **Test test_mcp_invalid_json testea serde_json no MCP** — L697-701: `serde_json::from_str::<Value>(malformed)` verifica que serde_json rechace JSON inválido. Esto testea la librería third-party, no la lógica del MCP server. La porción McpError y handle_tools_call(None) del test es válida | `vantadb-mcp/tests/mcp_tests.rs:697-721` | 🟢 15min | ⚪ | ❌ |
| `DRV-056` | **stdout write errors silenciosamente ignorados** — `write_json` y el main loop usan `let _ = writeln!(...)` y `let _ = stdout.flush()`, ignorando errores de I/O. Si stdout se cierra (proceso padre termina), los errores se tragan sin feedback | `vantadb-mcp/src/lib.rs:394-399,378-383` | 🟢 30min | ⚪ | ❌ |

### 🔍 Hallazgos del Review Deep — OpenAI Adapter (DRV)

> Items descubiertos durante `review-deep` del módulo `vantadb-openai` (Wave 0, DEPTH=quick). Thin PyO3 wrapper crate (10L lib.rs + 170L python.rs + 32L tests). 1 Python class `VantaDBOpenAI` con 4 métodos. `cargo fmt` OK; `cargo check` bloqueado por 2 errores de visibilidad en `vantadb::sdk::serialization` (DRV-043). Referencia: `.opencode/skills/review-deep/`.

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-057` | **OpenAI client recreado en cada llamada `embed()`** — Sin caching: `openai.OpenAI(api_key=...)` se instancia en cada llamada (L63-69). El cliente interno maneja connection pooling + TLS; recrearlo por request evita reuso de conexión y añade handshake TLS por llamada. Fix: cachear `Py<PyAny>` del cliente en el struct | `vantadb-openai/src/python.rs:63-69` | 🟢 1h | 🔵 | ✅ |
| `DRV-058` | **Metadata no-string values silenciosamente ignorados** — L149-155: `v.extract::<String>()` descarta bool/int/float. `if let (Ok(key), Ok(val))` silencia el error. Usuario pasa `metadata={"count": 5}` y el valor desaparece sin warning | `vantadb-openai/src/python.rs:149-155` | 🟢 30min | 🔵 | ✅ |
| `DRV-059` | **RwLock<String> namespace con overhead concurrente innecesario** — `RwLock` en L39 sugiere mutabilidad, pero nunca se escribe (0 `.write()` calls). Únicas operaciones: 2 `.read().unwrap().clone()`. Podría ser `String` plano, ahorrando 2 allocs + lock acquisition por operación | `vantadb-openai/src/python.rs:39,109,142` | 🟢 15min | ⚪ | ❌ |
| `DRV-060` | **Sin método para cambiar namespace runtime** — RwLock sugiere que namespace sería mutable, pero no hay setter expuesto. YAGNI candidate o feature incompleta | `vantadb-openai/src/python.rs:43-163` | 🟢 30min | ℹ️ | ❌ |
| `DRV-061` | **Test coverage mínima: 5 tests/32L** — Sin tests para: error de API key inválida, network timeout, results vacíos, metadatos edge cases, delete/update operations. Solo happy path | `vantadb-openai/tests/test_openai.py:1-32` | 🟢 1h | ⚪ | ❌ |

### 🔍 Hallazgos del Review Deep — Ollama Adapter (DRV)

> Items descubiertos durante `review-deep` del módulo `vantadb-ollama` (Wave 0, DEPTH=quick). Thin PyO3 wrapper crate (10L lib.rs + 161L python.rs + 32L tests). 1 Python class `VantaDBOllama` con 4 métodos. `cargo fmt` OK; `cargo check` bloqueado por 2 errores de visibilidad en `vantadb::sdk::serialization` (DRV-043). Comparte estructura y patrones con `vantadb-openai`. Referencia: `.opencode/skills/review-deep/`.

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-062` | **Ollama client recreado en cada llamada `embed()`** — Sin caching: `ollama.Client(host=...)` se instancia en cada llamada (L63-70). El cliente interno maneja connection pooling; recrearlo evita reuso de conexión. Fix: cachear `Py<PyAny>` del cliente en el struct | `vantadb-ollama/src/python.rs:63-70` | 🟢 1h | 🔵 | ✅ |
| `DRV-063` | **Metadata no-string values silenciosamente ignorados** — L141: `v.extract::<String>()` descarta bool/int/float igual que en DRV-058. Bug duplicado por copy-paste del adapter openai | `vantadb-ollama/src/python.rs:139-145` | 🟢 30min | 🔵 | ✅ |
| `DRV-064` | **embed() llama API secuencialmente por texto** — L73-90: cada texto hace una llamada RPC individual. Ollama soporta `client.embed(model=..., input=[...])` para batch embedding. N textos = N RPCs vs 1 batch | `vantadb-ollama/src/python.rs:73-90` | 🟢 1h | ⚪ | ❌ |
| `DRV-065` | **RwLock<String> namespace con overhead concurrente innecesario** — Ídem DRV-059. Nunca escrito, solo 2 `.read().unwrap().clone()`. Podría ser `String` plano | `vantadb-ollama/src/python.rs:39,100,133` | 🟢 15min | ⚪ | ❌ |
| `DRV-066` | **Sin método para cambiar namespace runtime** — Ídem DRV-060. YAGNI candidate o feature incompleta | `vantadb-ollama/src/python.rs:43-154` | 🟢 30min | ℹ️ | ❌ |
| `DRV-067` | **Test coverage mínima: 5 tests/32L** — Ídem DRV-061. Sin tests para error paths, delete, edge cases | `vantadb-ollama/tests/test_ollama.py:1-32` | 🟢 1h | ⚪ | ❌ |

### 🔍 Hallazgos del Review Deep — LiteLLM Adapter (DRV)

> Items descubiertos durante `review-deep` del módulo `vantadb-litellm` (Wave 0, DEPTH=quick). Thin PyO3 wrapper crate (156L python.rs + 31L tests). 1 Python class `VantaDBLiteLLM`. `cargo fmt` OK. **Regresión: no libera GIL en search/store, a diferencia de openai/ollama**. Referencia: `.opencode/skills/review-deep/`.

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-068` | **GIL no liberado en search()** — `search()` recibe `py: Python` (L96) pero no usa `py.detach()` para la búsqueda vectorial (L111). openai/ollama sí lo hacen. Bloquea GIL durante `engine.search()` impidiendo ejecución concurrente de Python threads | `vantadb-litellm/src/python.rs:94-122` | 🟢 15min | 🔵 | ❌ |
| `DRV-069` | **store() sin parámetro py — no puede liberar GIL** — `fn store(&self, text: &str, ...)` (L124) no acepta `py: Python`, a diferencia de openai/ollama que sí lo hacen y usan `py.detach()` para GIL release. Para liberar GIL necesita cambio de firma + caller update | `vantadb-litellm/src/python.rs:124-148` | 🟢 30min | 🔵 | ❌ |
| `DRV-070` | **Metadata no-string values silenciosamente ignorados** — L138: `v.extract::<String>()` descarta bool/int/float, igual que DRV-058/063. Tercera copia del mismo bug | `vantadb-litellm/src/python.rs:136-142` | 🟢 30min | 🔵 | ❌ |
| `DRV-071` | **RwLock<String> namespace con overhead concurrente innecesario** — Ídem DRV-059/065. Nunca escrito, solo 2 `.read().unwrap().clone()` | `vantadb-litellm/src/python.rs:38,100,130` | 🟢 15min | ⚪ | ❌ |
| `DRV-072` | **Sin método para cambiar namespace runtime** — Ídem DRV-060/066 | `vantadb-litellm/src/python.rs:43-148` | 🟢 30min | ℹ️ | ❌ |
| `DRV-073` | **Test coverage mínima: 4 tests/31L** — Ni siquiera test de store con metadata. Sin tests para error paths | `vantadb-litellm/tests/test_litellm.py:1-31` | 🟢 1h | ⚪ | ❌ |

### vantadb-mem0 (quick — 5 findings)

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-074` | **`delete_col()` solo pagina 1 — data loss** — `VantaMemoryListOptions::default()` tiene limit=100 (L326). Si collection tiene >100 registros, los de páginas posteriores sobreviven al delete. No es atómico ni completo | `vantadb-mem0/src/python.rs:324-344` | 🟡 2h | 🟠 | ❌ |
| `DRV-075` | **`search()` ignora text_query** — `let _ = query;` (L202) descarta el texto de búsqueda de Mem0. Solo vector search, sin hybrid/text-reranking. Gap de feature | `vantadb-mem0/src/python.rs:201-202` | 🟢 30min | ⚪ | ❌ |
| `DRV-076` | **`update()` TOCTOU entre get() y put()** — Dos `py.detach()` separados (L268, L289) permiten modificación concurrente entre lectura y escritura del registro | `vantadb-mem0/src/python.rs:254-291` | 🟢 1h | ⚪ | ❌ |
| `DRV-077` | **Collection name sanitización lazy/late** — `create_col()` guarda el raw name (L144), sanitización solo ocurre al usar como namespace. Namespace efectivo ≠ collection_name si contiene chars inválidos | `vantadb-mem0/src/python.rs:88-93,137-146` | 🟢 30min | ⚪ | ❌ |
| `DRV-078` | **Test coverage: 8 tests/60L** — Sin tests para: `delete_col()` multi-page, `update()` con vector, `create_col()`, `list_cols()`, `reset()`, edge cases (empty vectors, empty strings) | `vantadb-mem0/tests/test_mem0.py:1-60` | 🟢 1h | ⚪ | ❌ |

### vantadb-letta (quick — 6 findings)

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-079` | **`list_memories` solo pagina 1 — truncación** — `VantaMemoryListOptions::default()` (L128) limit=100. Si user/agent tiene >100 memorias, las extra no aparecen. Mismo patrón que DRV-074 | `vantadb-letta/src/python.rs:122-139` | 🟡 2h | 🔵 | ❌ |
| `DRV-080` | **`retrieve_memory` expone distancia VantaDB raw** — `hit.score` (L116) pasa distancia Cosine (0→2, 0=idéntico) directa. Consumidores Letta esperan score 0-1. Sin normalización vs `vanta_distance_to_mem0_score` en mem0 | `vantadb-letta/src/python.rs:111-118` | 🟢 30min | ⚪ | ❌ |
| `DRV-081` | **`AtomicU64 counter` reset en restart** — Counter inicializado en 0 (L64). Tras cerrar/reabrir store, nuevos inserts pueden colisionar con memorias existentes. Para agentes efímeros, impacto bajo | `vantadb-letta/src/python.rs:64,76-77` | 🟢 30min | ⚪ | ❌ |
| `DRV-082` | **Test coverage: 5 tests/42L** — Sin tests para: retrieve vacío, múltiples agentes, invalid memory_id, memory_id con `:` extra, payloads grandes | `vantadb-letta/tests/test_letta.py:1-42` | 🟢 1h | ⚪ | ❌ |
| `DRV-083` | **`store_memory` sin dedup** — Mismo user/agent/content almacenado dos veces → dos entradas separadas. Sin idempotencia | `vantadb-letta/src/python.rs:67-85` | 🟢 30min | ℹ️ | ❌ |
| `DRV-084` | **No `delete_col` ni `reset`** — No hay API para limpiar todas las memorias de un user/agent. Letta consumers no pueden resetear estado | `vantadb-letta/src/python.rs:42-163` | 🟢 1h | ℹ️ | ❌ |

### vantadb-crewai (quick — 6 findings)

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-085` | **`clear()` solo pagina 1 — data loss** — `Default::default()` limit=100 (L126). Si namespace tiene >100 records, el resto sobrevive al clear. Mismo bug que DRV-074/079 | `vantadb-crewai/src/python.rs:120-138` | 🟡 2h | 🔵 | ❌ |
| `DRV-086` | **Metadata no-string values silenciosamente ignorados** — `py_dict_to_string_map` (L144): `v.extract::<String>()` descarta Bool/Int/Float. Mismo bug que DRV-058/063/070. A diferencia de openai/ollama/litellm, metadata se serializa a JSON con serde_json, pero los valores no-string se pierden antes | `vantadb-crewai/src/python.rs:141-148` | 🟢 30min | 🔵 | ❌ |
| `DRV-087` | **RwLock\<String\> namespace con overhead innecesario** — 0 `.write()` calls, solo 3 `.read().unwrap().clone()` (L65, L90, L121). Dead lock. Mismo que DRV-059/065/071 | `vantadb-crewai/src/python.rs:37,65,90,121` | 🟢 15min | ⚪ | ❌ |
| `DRV-088` | **serde_json dependency extra** — Único adapter que añade serde_json para convertir BTreeMap<String,String> a String. Podría ser format!/join sin dependencia | `vantadb-crewai/Cargo.toml:20` | 🟢 15min | ⚪ | ❌ |
| `DRV-089` | **Test coverage: 5 tests/35L** — Sin tests para: metadata con non-string values, empty embedding, threshold filtering, clear con >100 records | `vantadb-crewai/tests/test_crewai.py:1-35` | 🟢 1h | ⚪ | ❌ |
| `DRV-090` | **search() threshold filtering post-GIL** — Score normalization (L107) ocurre tras `py.detach()`, después de recibir todos los hits. Para top_k=1000 con threshold alto, se traen 1000 results para posiblemente devolver 0 | `vantadb-crewai/src/python.rs:105-116` | 🟢 1h | ⚪ | ❌ |

### vantadb-dspy (quick — 5 findings)

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-091` | **RwLock\<String\> collection dead overhead** — `collection` nunca escrito (0 `.write()`), solo 2 `.read().unwrap().clone()`. Mismo patrón que DRV-059/065/071/087 | `vantadb-dspy/src/python.rs:37,60,94` | 🟢 15min | ⚪ | ❌ |
| `DRV-092` | **Metadata no-string values silenciosamente ignorados** — `add_passage()` L102: `v.extract::<String>()` descarta Bool/Int/Float. Mismo bug que DRV-058/063/070/086 (pero superficie menor — DSPy raramente usa metadata rica) | `vantadb-dspy/src/python.rs:100-107` | 🟢 30min | ⚪ | ❌ |
| `DRV-093` | **forward() expone distancia VantaDB raw** — `hit.score` (L79) pasa Cosine distance (0→2) directa. Sin normalización como crewai. Mismo que DRV-080 | `vantadb-dspy/src/python.rs:76-81` | 🟢 30min | ⚪ | ❌ |
| `DRV-094` | **AtomicU64 counter reset en restart** — Mismo que DRV-081. Counter en 0 al abrir store; nuevos inserts pueden colisionar con existentes | `vantadb-dspy/src/python.rs:38,95-96` | 🟢 30min | ℹ️ | ❌ |
| `DRV-095` | **Test coverage: 6 tests/39L** — Cubre init, add+forward, metadata, empty forward. Sin tests para: múltiples passages, metadata con non-string, large payloads | `vantadb-dspy/tests/test_dspy.py:1-39` | 🟢 1h | ⚪ | ❌ |

### vantadb-haystack (quick — 6 findings)

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-096` | **RwLock\<String\> namespace dead overhead** — `namespace` nunca escrito (0 `.write()`), solo 5 `.read().unwrap().clone()` (L59/123/152/164). Dead lock. Mismo que DRV-059/065/071/087/091 | `vantadb-haystack/src/python.rs:37,59,123,152,164` | 🟢 15min | ⚪ | ❌ |
| `DRV-097` | **count_documents() truncates at 100** — Usa `Default::default()` como VantaMemoryListOptions, que tiene `limit: Some(100)`. Si namespace tiene >100 docs, `page.records.len()` devuelve 100. Mismo bug que DRV-074/079/085. Carga además todos los records en memoria solo para contar | `vantadb-haystack/src/python.rs:167-171` | 🟢 1h | 🔵 | ❌ |
| `DRV-098` | **Metadata no-string values silenciosamente ignorados en write_documents** — `write_documents` L92: `entry.1.extract::<String>()` descarta Bool/Int/Float. Pero `py_dict_to_vanta_metadata` (usado por `filter_documents`) maneja los 4 tipos correctamente (L181-189). Inconsistencia intra-archivo: metadata no-string se pierde al escribir pero se parsea al filtrar. Documentos escritos con metadata no-string no son encontrables por filtro | `vantadb-haystack/src/python.rs:88-98` | 🟢 30min | 🔵 | ❌ |
| `DRV-099` | **No implementa protocolo Haystack Document real** — Métodos aceptan/retornan `list[dict]` en vez de `list[haystack.dataclasses.Document]`. No maneja `DuplicatePolicy`. No retorna `int` de `write_documents`. No es compatible con pipelines Haystack reales sin conversión manual. La .pyi solo declara dicts — no importa `haystack` en absoluto | `vantadb-haystack/src/python.rs:42-173`, `vantadb-haystack/vantadb_haystack.pyi:1-8` | 🟡 4h | 🔴 | ✅ |
| `DRV-100` | **Test coverage: 7 tests/48L** — Cubre init (2), write+filter, metadata write, count, delete. Sin tests para: filter con filtros reales, empty results, count >100 docs, metadata no-string, auto-generated IDs, error paths (path inválido, namespace vacío) | `vantadb-haystack/tests/test_haystack.py:1-48` | 🟢 1h | ⚪ | ❌ |
| `DRV-101` | **AtomicU64 doc_counter no persistido** — Counter se reinicia a 0 al abrir store. Nuevos inserts pueden generar IDs que colisionan con docs existentes (mismo que DRV-081/094) | `vantadb-haystack/src/python.rs:38,68-70` | 🟢 30min | ℹ️ | ❌ |

### vantadb-langchain (quick — 7 findings)

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-102` | **Missing GIL release en TODOS los métodos** — `add_texts` (L82-85), `similarity_search_by_vector` (L109-112), `delete` (L126-133) corren sin `py.detach()`/`py.allow_threads()`. Parámetro `py: Python` se recibe en 2/3 métodos pero nunca se usa. Contraste: los otros 7 adapters (haystack, dspy, crewai, mem0, letta, openai, ollama, litellm) usan `py.detach()` correctamente | `vantadb-langchain/src/python.rs:82-85,109-112,126-133` | 🟢 1h | 🔴 | ✅ |
| `DRV-103` | **Metadata no-string values silenciosamente ignorados** — `v.extract::<String>()` (L73) descarta Bool/Int/Float. Mismo bug que DRV-058/063/070/086/092/098 | `vantadb-langchain/src/python.rs:70-78` | 🟢 30min | 🔵 | ❌ |
| ✅ ~~`DRV-104`~~ | ~~**similarity_search_by_vector no retorna metadata** — Solo id/text/score (L116-120). Metadata almacenada via `add_texts` se pierde en la respuesta. LangChain espera metadata para filtering en pipeline~~ | ~~`vantadb-langchain/src/python.rs:114-121`~~ | ~~🟢 30min~~ | ~~🔵~~ | ✅ |
| ✅ ~~`DRV-105`~~ | ~~**delete() silenciosamente no-op en IDs malformados** — `id.split(':')` (L127) con `parts.len() != 2` (L128) ignora el delete sin error ni warning. Si un ID externo no tiene formato `namespace:key`, el delete falla silenciosamente~~ | ~~`vantadb-langchain/src/python.rs:125-133`~~ | ~~🟢 30min~~ | ~~🔵~~ | ✅ |
| ✅ ~~`DRV-106`~~ | ~~**from_texts class method no implementado pese a docstring** — Docstring (L11) afirma implementar VectorStore protocol incluyendo `from_texts`, pero el método no existe en código. LangChain usa `from_texts` como entry point principal para crear stores~~ | ~~`vantadb-langchain/src/python.rs:11`~~ | ~~🟡 2h~~ | ~~🟡~~ | ✅ |
| `DRV-107` | **Test coverage: 5 tests/43L** — Cubre init, add+search, metadata, delete, unique keys. Sin tests para: delete con IDs malformados, empty texts/embeddings, metadata no-string, metadata return en search, large payloads, error paths | `vantadb-langchain/tests/test_langchain.py:1-43` | 🟢 1h | ⚪ | ❌ |
| `DRV-108` | **AtomicU64 counter no persistido** — Counter se reinicia a 0 al abrir store (mismo que DRV-081/094/101) | `vantadb-langchain/src/python.rs:23,59,63` | 🟢 30min | ℹ️ | ❌ |

### vantadb-llamaindex (quick — 6 findings)

Nota: El código es casi byte-for-byte idéntico a `vantadb-langchain`. Los hallazgos duplican DRV-102→108. Solo cambian nombres de métodos (`add`/`query` vs `add_texts`/`similarity_search_by_vector`).

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-109` | **Missing GIL release en TODOS los métodos** — `add` (L82-85), `query` (L104-107), `delete` (L124-126) corren sin `py.detach()`. Mismo bug que DRV-102 | `vantadb-llamaindex/src/python.rs:82-85,104-107,124-126` | 🟢 1h | 🔴 | ✅ |
| `DRV-110` | **Metadata no-string values silenciosamente ignorados** — `v.extract::<String>()` (L73). Mismo bug que DRV-103 | `vantadb-llamaindex/src/python.rs:70-78` | 🟢 30min | 🔵 | ❌ |
| ✅ ~~`DRV-111`~~ | ~~**query() no retorna metadata** — Solo id/text/score (L112-114). Metadata almacenada via `add` se pierde. Mismo bug que DRV-104~~ | ~~`vantadb-llamaindex/src/python.rs:109-116`~~ | ~~🟢 30min~~ | ~~🔵~~ | ✅ |
| `DRV-112` | **delete() silenciosamente no-op en IDs malformados** — `split(':')` con `parts.len() != 2` ignora error. Mismo bug que DRV-105 | `vantadb-llamaindex/src/python.rs:120-128` | 🟢 30min | 🔵 | ❌ |
| `DRV-113` | **Test coverage: 5 tests/43L** — Idéntico a langchain. Sin tests para: delete con IDs malformados, empty, metadata return, no-string metadata | `vantadb-llamaindex/tests/test_llamaindex.py:1-43` | 🟢 1h | ⚪ | ❌ |
| `DRV-114` | **AtomicU64 counter no persistido** — Mismo que DRV-108 | `vantadb-llamaindex/src/python.rs:23,59,63` | 🟢 30min | ℹ️ | ❌ |
| `DRV-115` | **🚫 `vantadb-openai` STATUS_STACK_BUFFER_OVERRUN en MSVC linker** — Compila individualmente (`cargo check -p vantadb-openai`) pero revienta en workspace build. El linker `link.exe` de MSVC se desborda con pyo3 + dependencias grandes. `build-jobs = 2` en `.cargo/config.toml` mitiga parcialmente pero no alcanza. Cascadea a `vantadb-wasm` (48 errores: `can't find crate for vantadb`). Fix: excluir adaptadores pyo3 de workspace build o usar `rust-lld` | `.cargo/config.toml`, todo workspace | 🟡 4h | 🔴 | ❌ |
| `DRV-116` | **10 warnings de compilación en `vantadb` core** — 9x `unnecessary unsafe block` en `graph.rs:178`, `serialize.rs:528,598`, `archive.rs:74,104`, `maintenance.rs:141`, `vfile.rs:481,485,567` (Mmap::map y MmapMut::map_mut son safe en vfile. MCP v0.6). 4 dead code methods en `vfile.rs:79-91` (`flush`, `flush_async`, `flush_range`, `is_empty`) | `src/index/graph.rs`, `src/index/serialize.rs`, `src/storage/archive.rs`, `src/storage/engine/maintenance.rs`, `src/storage/vfile.rs` | 🟢 30min | ⚪ | ❌ |
| `DRV-117` | **2 stale advisory ignores en `deny.toml`** — `RUSTSEC-2024-0436` (paste) y `RUSTSEC-2025-0134` (rustls-pemfile) ya no matchean ningún crate. Limpiar del ignore list | `deny.toml:11,15` | 🟢 5min | ⚪ | ❌ |
| `DRV-118` | **Windows builds missing from CI release matrix** — release.yml builds Linux+macOS only. No Windows binaries available. Blocks Windows adoption | `.github/workflows/release.yml` | 🟡 1d | 🔴 | ❌ |
| `DRV-119` | **No multi-layer storage rollback (ACID Phase 0)** — WAL/VantaFile/HNSW/KV writes uncoordinated; partial failure leaves inconsistent state. Pre-requisite for ACID Phase 1-3 | `src/storage/engine/ops.rs` | 🟠 3-5d | 🔴 | ❌ |
| `DRV-120` | **HNSW layer-0-only navigation** — Search is O(n) instead of sub-linear. Multi-layer graph algorithm never completed; HNSW degrades to flat scan at scale | `src/index/graph.rs` | 🟠 3-5d | 🔴 | ❌ |
| `DRV-121` | **Planner AST/LogicalPlan/PhysicalPlan not implemented** — IQL parsed directly to execution with no intermediate representation. No query optimization, no cost-based planning | `src/query.rs` | 🟠 3-5d | 🟠 | ❌ |
| `DRV-122` | **IQL lacks JOINs, subqueries, SQL compatibility** — Biggest feature gap vs Qdrant/Chroma. No FROM/JOIN/WHERE/GROUP BY | `src/query.rs` | 🟠 5-10d | 🟠 | ❌ |
| `DRV-123` | **Auto-embedding on INSERT not implemented** — LlmClient.generate_embedding() exists but never called from executor. Landing page falsely claims auto-embedding support | `src/llm.rs`, `src/executor.rs` | 🟡 2-3d | 🟠 | ❌ |
| `DRV-124` | **macOS code signing/notarization missing** — No Apple Developer Account, no codesign, no Gatekeeper notarization. macOS users get security warnings | — | 🟡 2-3d | 🟡 | ❌ |
| `DRV-125` | **No Miri tests for UB detection** — Unsafe code paths (SIMD, mmap, zero-copy deserialize) without undefined behavior testing | `src/index/distance.rs`, `src/index/graph.rs` | 🟡 1-2d | 🟡 | ❌ |
| `DRV-126` | **No regression benchmarks in CI** — Performance regressions undetected. nightly_bench.yml exists but not gated | `.github/workflows/ci-rust-10.yml` | 🟡 1d | 🟡 | ❌ |
| `DRV-127` | **WAL encryption does not exist (plain text)** — WAL data stored unencrypted even when enterprise feature enabled. Enterprise encryption is a no-op stub | `vantadb-enterprise/src/encryption.rs`, `src/storage/wal.rs` | 🟡 2-3d | 🟡 | ❌ |
| `DRV-128` | **No governance production tests** — AdmissionFilter, ConflictResolver lack covering tests for production paths | `src/governance/admission.rs`, `src/governance/conflict.rs` | 🟡 1d | 🟡 | ❌ |
| `DRV-129` | **Enterprise crate fully disconnected from main crate** — vantadb_enterprise never imported; 96% placeholder code (267L, ~10L real logic). Should integrate or delete | `vantadb-enterprise/` | 🟡 1d | 🟡 | ❌ |
| `DRV-130` | **SIFT 1M high-recall 127s bottleneck** — Known performance blocker for certification benchmarks. Anti-locality in SSD layout | `src/index/search.rs` | 🟡 2-3d | 🟡 | ❌ |
| `DRV-131` | **Missing index types beyond HNSW** — Only 1 index type vs 8 in Quiver. No Flat/IVF/PQ/Int8/FP16/Binary for diverse workload optimization | `src/index/` | 🟠 5-10d | 🔵 | ❌ |
| `DRV-132` | **AuthRateLimiter unbounded HashMap — DoS memory exhaustion** — `AuthRateLimiter` en `cli_server.rs` usa HashMap sin límite de crecimiento. Un atacante con IPs distintas puede llenar la memoria del servidor. Fix: LRU cache o `BoundedHashMap` con TTL | `src/cli_server.rs:146-211` | 🟢 2h | 🔴 | ❌ |
| `DRV-133` | **Tombstoned nodes contaminate HNSW search_layer heap** — Nodos marcados como tombstone no se filtran durante `search_layer`, contaminando el heap de resultados. Puede devolver nodos eliminados como hits válidos | `src/index/core.rs:562-570` | 🟢 2h | 🔴 | ❌ |
| `DRV-134` | **NbAccordion sin keyboard navigation** — Componente accordion no soporta navegación por teclado (Enter/Space/ArrowKeys). Violación WCAG 2.1. Sin focus management ni aria-expandido dinámico | `web/src/components/nb/NbAccordion.tsx` | 🟢 2h | 🟡 | ❌ |
| `DRV-135` | **3 unmaintained dependencies** — atomic-polyfill, paste, rustls-pemfile (via axum-server) no mantenidos. Solo paste+RUSTSEC-2024-0436 en deny.toml, faltan los otros 2 | `deny.toml`, `Cargo.lock` | 🟢 30min | 🟡 | ❌ |
| `DRV-136` | **vantadb-wasm monolítico — CRUD user descarga 1MB+** — WASM build no feature-gated. Usuario que solo necesita CRUD básico descarga graph, governance, crypto, MCP. Sin tree-shaking WASM posible | `vantadb-wasm/Cargo.toml` | 🟡 2-3d | 🟡 | ❌ |

### 🔍 Hallazgos del Cross-Ref Docs-vs-Code (Wave 3)

> Items descubiertos durante la reconciliación cross-ref docs vs code (Jul 16). Ver `docs/audit-reports/cross-ref-wave3-final-report.md`.

| ID | Tarea | Archivo
|----|-------|---------|----------|-----------|--------|
| `REV-001` | **CI: Rust fails on main — TSan ABI mismatch** — `-Zsanitizer=thread` incompatible con toolchain 1.94.1 | `.github/workflows/ci-rust-10.yml` → H05-ERROR-001 | 🟢 2h | 🔴 | ✅ |
| `REV-002` | **CI: Web fails on main — 21 lint issues** — 14 ESLint errors + 7 warnings rompen build | `.github/workflows/ci-web-11.yml` → H05-ERROR-002 | 🟢 2h | 🔴 | ✅ |
| `REV-003` | **No code coverage measurement** — CII Silver requiere ≥80%, sin herramienta configurada | CI config → H05-MISSING-001 | 🟡 1d | 🔴 | ✅ |
| `REV-004` | **`tantivy` rlib not found** — Test builds de `vantadb-openai` fallan por dependencia faltante | `vantadb-openai/Cargo.toml` → H08-ARCH-001 | 🟡 1d | 🟡 | ✅ |
| `REV-005` | **14 ESLint errors en web frontend** — 6x `no-explicit-any`, 8x prettier en `demo.lazy.tsx` + `why-vantadb.tsx` | `web/src/routes/demo.lazy.tsx`, `web/src/routes/why-vantadb.tsx` → H03-CODE-001 | 🟢 1h | 🟡 | ✅ |
| `REV-006` | **No workspace-level clippy en CI** — Solo `-p vantadb`, adapters excluded | `.github/workflows/ci-rust-10.yml` → H05-MISSING-002 | 🟢 2h | 🟡 | ✅ |
| `REV-007` | **`reducedMotion` missing from useEffect deps** — Stale closure risk en 3 componentes | `NbMonolith.tsx:61`, `NbVectorNebula.tsx:239`, `__root.tsx:181` → H03-CODE-002 | 🟢 30min | 🟡 | ✅ |
| `REV-008` | **Node 20 actions deprecated** — `actions/checkout` y `setup-node` usan Node 20, runner usa Node 24 | `.github/workflows/*.yml` → H05-CODE-005 | 🟢 30min | 🟡 | ✅ |
| `REV-009` | **19 workspace crates compilation overhead** — Adaptadores (10 crates) dependen de pyo3, rebuild en cascada | `Cargo.toml` → H08-ARCH-002 | 🟡 2-3d | 🟡 | ✅ |
| `REV-010` | **`serialization.rs` god module 1827L** — Candidato a split, documentado pero no ejecutado | `src/sdk/serialization/mod.rs` → H08-PATTERN-001 | 🟡 1d | 🟡 | ❌ |
| `REV-011` | **`insert_hnsw` monolithic 177L** — Función sin descomponer en sub-operaciones | `src/index/graph.rs` → H08-PATTERN-002 | 🟡 4h | 🟡 | ✅ |
| `REV-012` | **HNSW `insert_lock` contention** — Micro-batching implementado (P1), posible bottleneck bajo alta concurrencia | `src/index/graph.rs` → H08-ALGO-001 | 🟡 1-2d | 🟡 | ❌ |
| `REV-013` | **`spin 0.9.8` yanked dependency** — Usado transitivamente vía fjall/flume, monitoreado | `deny.toml` → H08-LOGIC-001 | 🟢 1h | 🟡 | ❌ |
| `REV-014` | **24 stale dependabot branches** — No auto-delete después de merge | `origin/dependabot/*` → H05-DIRECTION-001 | 🟢 30min | 🔵 | ❌ |
| `REV-015` | **6 `any` types sin justificación** — `demo.lazy.tsx` usa `any` sin `eslint-disable` ni type alias | `web/src/routes/demo.lazy.tsx` → H03-CLARITY-001 | 🟢 1h | 🟡 | ✅ |
| `REV-016` | **`vantadb-enterprise` abstracción prematura** — Crate existe pero features no definidos públicamente | `vantadb-enterprise/` → H08-ARCH-003 | 🟢 2h | 🔵 | ✅ |
| `REV-017` | **`why-vantadb.tsx` prettier error** — Trailing newline rompe formateo | `web/src/routes/why-vantadb.tsx:43` → H03-CODE-003 | 🟢 5min | 🟢 | ✅ |
| `REV-018` | **`NbToast.tsx` react-refresh warning** — Archivo exporta más que solo componentes | `web/src/components/nb/NbToast.tsx:15` → H03-CODE-004 | 🟢 5min | 🟢 | ✅ |

### WASM & Performance

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `NUEVO-11` | **WASM IndexedDB fallback** | 🟡 2-3d | 🟡 | ❌ |
| `NUEVO-12` | **WASM multi-tab coordination** (Web Locks + BroadcastChannel) | 🟡 2-3d | 🟡 | ❌ |
| `NUEVO-13` | **HNSW auto-tuning PID loop** (ef_search dinámico) | 🟡 3-5d | 🟡 | ❌ |
| `NUEVO-14` | **WASM bundle size <500KB gzip** | 🟡 1-2d | 🟡 | ❌ |
| `NUEVO-15` | **Code coverage report en CI** + upload | 🟢 1d | 🟡 | ❌ |
| `NUEVO-19` | **Mover SourceDesign/ fuera de web/src/** | 🟢 1h | 🔵 | ❌ |

---

## TIER 2 — 🟡 Launch Campaign

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `CLI-01` | **CLI polish: backup/restore/doctor/stats/inspect/REPL/TUI** — Phase 4.E del roadmap, no implementado | 🟡 3-5d | 🟡 | ❌ |
| `DEVOPS-HOMEBREW` | **Homebrew formula** — Phase 4.F, no existe | 🟢 4h | 🟡 | ❌ |
| `DEVOPS-PY313` | **Python 3.13 wheels en CI matrix** — Phase 4.F, no configurado | 🟢 2h | 🟡 | ❌ |
| `DEVEX-DEMO` | **Demo app (Rust + Python)** — Phase 4.G, no existe | 🟡 2-3d | 🟡 | ❌ |
| `DEVEX-EXAMPLES` | **Rust examples en `docs/examples/`** — Phase 4.G, no existe | 🟢 4-6h | 🟡 | ❌ |
| `NUEVO-16` | **Product Quantization (PQ) 96x** — compresión para datasets >RAM | Alto | 🔵 | ❌ |
| `NUEVO-17` | **Segment LSM-style** — hot/warm/cold tiers | Muy alto | 🔵 | ❌ |
| `NUEVO-18` | **Sparse vectors nativos** — hybrid search real | Alto | 🔵 | ❌ |
| `NUEVO-20` | **Server Docker image** | 🟡 1-2d | 🔵 | ❌ |
| `NUEVO-21` | **Vectara competitive research** | 🟢 2-4h | ⬜ | ❌ |
| `TSK-107b` | Audit logging enterprise (JSONL, timestamp + op) | 🟡 2-3d | 🟡 | ❌ |
| `ENT-04` | Connection pooling + circuit breaker | 🟡 2-3d | 🟡 | ❌ |
| `BIZ-01` | Enterprise crate (encryption, audit, RBAC, replication) | 🟡 3-5d | 🟡 | ⏳ |

---

## TIER 3 — 🟢 Comunidad y Operaciones

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `COM-02` | **Configurar Discord: reaction roles, autorole, logging, welcome DM, onboarding** — ~10 items operacionales de `docs/discord/todo.md` | 🟡 2-3d | 🟢 | ❌ |
| `COM-03` | **Discord: AutoMod (spam, mass-mention, invites), stickers/emojis, forums seed** | 🟢 4-6h | 🟢 | ❌ |
| `COM-04` | **Discord: ticketing system, stage channel, Server Discovery, Canny.io roadmap, cross-promotion** | 🟢 4-6h | 🟢 | ❌ |

---

## TIER 4 — 🔵 Post-Lanzamiento

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `—` | Publicar 8 workspace members en crates.io | 🟡 2-3d | 🟡 | ❌ |
| `WEB-001` | **Re-add interactive WASM demo page** — Restaurar demo después de publicar `@vantadb/wasm` en npm | 🟢 30min | 🟡 | ❌ |

---

## ✅ Resumen de Verificación (Jul 13, 2026)

### Documentos archivados (13 → `docs/archive/`)

| Archivo | Razón |
|---------|-------|
| `reviews/agent-01-local-AK.md` | Raw agent output, consolidado en FINAL-REVIEW |
| `reviews/agent-02-local-LZ.md` | Raw agent output, consolidado en FINAL-REVIEW |
| `reviews/agent-03-global-agents.md` | Raw agent output, consolidado en FINAL-REVIEW |
| `reviews/agent-04-global-claude.md` | Raw agent output, consolidado en FINAL-REVIEW |
| `reviews/agent-05-internet-research.md` | Raw agent output, consolidado en FINAL-REVIEW |
| `reviews/EXECUTIVE_TECHNICAL_AUDIT.md` | Superseded por audits Jul 11/13 |
| `reviews/AUDITORIA_COMPLETA_VantaDB_WEB.md` | Web-only, superseded |
| `reviews/FULL_CODEBASE_AUDIT_2026-07-09.md` | Superseded por 2026-07-11 |
| `reviews/web-audit-report.md` | Superseded |
| `research/DOCS_TOOLS_RESEARCH.md` | Cold research, no tool adopted |
| `research/SQL_ANALYSIS.md` | Decisión tomada (no SQL), sin acción pendiente |
| `research/COGNEE_EVALUATION.md` | Pure research, cero implementación |
| `research/DOCS_AUDIT_REPORT.md` | Issues tracked en bitácora/backlog |

### Claims verificados contra código: 100% precisos

De ~150 claims de estado en el backlog anterior, todos fueron verificados contra el código real usando `codegraph_explore`, `grep`, y lectura directa. Ver `docs/reviews/FULL_CODEBASE_AUDIT_2026-07-11.md` y `docs/reviews/2026-07-13-full-review.md` para el detalle.

### Documentos de investigación aún vigentes

| Documento | Estado | Nota |
|-----------|--------|------|
| `research/ACID_TRANSACTIONS.md` | ⚠️ Parcial | Phase 1 implementada; Phase 2-3 no; WAL shipping + PITR existen |
| `research/SIGNED_RELEASES.md` | ⚠️ Parcial | Attestations + Windows builds OK; GPG/sigstore no |
| `research/VantaDB_RESEARCH_UNIFIED.md` | ✅ Vigente | Mejor referencia de arquitectura |
| `research/VantaDB_RESEARCH_VALIDADO.md` | ✅ Vigente | Meta-validación precisa |
| `research/VantaDB_ANALISIS_COMPLETO.md` | ⚠️ Parcial | Version sync ya resuelto |

---

## 📈 Timeline Consolidado

```
Jul 14-18  TIER 0 (🔴 6+ items):
               ─ INT-01/02: LangChain + LlamaIndex → PyPI
               ─ DEVOPS-05: Pipeline CI unificado
               ─ REL-02: vantadb-ts → npm
               ─ MKT-13: Link WASM demo en hero ⏳
               ─ **REV-001/002: Fix CI (TSan + ESLint)**
               ─ **SEC-13: CSP unsafe-inline + HSTS + nonce** ← docs-audit
               ─ **WEB-02: Fix false claims landing** ← docs-audit
Jul 18-25  TIER 1 (🟠 35+ items):
               ─ Docs: MKT-14 (case studies), TSK-106 (Discussions)
               ─ NUEVO-01/07/08/10: README, migrations, learning, benchmarks
               ─ Launch: LEG-01, MKT-03/04/05/10/15/16
                ─ Code health: VFY-001→012, **REV-003→018**, **DRV-001→045**, **DRV-046→056**, **DRV-057→061**, **DRV-062→067**, **DRV-068→073**, **DRV-074→078**, **DRV-079→084**, **DRV-085→090**, **DRV-091→095**, **DRV-096→101**, **DRV-102→108**, **DRV-109→114**, **DRV-118→136** (cross-ref Wave 3 gaps)
               ─ WASM: NUEVO-11→15
               ─ Docs-audit: SEC-14, WEB-03/04, DEVOPS-13/14/15, TEST-11/12, DOC-20
Ago+       TIER 2-4:
               ─ CLI-01, DEVOPS-HOMEBREW, DEVOPS-PY313, DEVEX-DEMO, DEVEX-EXAMPLES
               ─ NUEVO-16/17/18: PQ, LSM, sparse vectors
               ─ Enterprise: ENT-04, BIZ-01
               ─ COM-02/03/04: Discord setup
               ─ Publishing: crates.io, WEB-001
```

---

## ✅ Definition of Ready (DoR)

- [ ] ID único asignado
- [ ] Prioridad definida (🔴🟠🟡🟢🔵⬜)
- [ ] Archivos involucrados conocidos
- [ ] Esfuerzo estimado
- [ ] Verificado contra código real (no asumido)

## ✅ Definition of Done (DoD)

- [ ] Código compila (`cargo check` / `tsc --noEmit`)
- [ ] Tests pasan (`cargo test` / `vitest run`)
- [ ] Linters pasan (`cargo clippy` / `eslint`)
- [ ] Docs actualizados si aplica
- [ ] Tarea movida a `progreso/README.md`
- [ ] Changelog actualizado si es cambio visible al usuario

---

## TIER 5 — 🔴🧩 Rescate de Old Docs (Features Perdidas)

> Recuperado de `VANTADB DOC OLD` (~280 archivos .md analizados vía 21 sub-agents). Ver `docs/REPORTE_EVALUACION_COMPLETO.md` secciones 6 y 7 para detalles completos de cada item.
> **Total items:** 22 (6 🔴 + 8 🟡 + 4 🟢 + 4 ⚪)

### 🗂 Referencia de Archivos Old Docs

Los items de este tier se identificaron a partir de archivos en `C:\Users\Eros\VantaDB Proyect\VANTADB DOC OLD\`. Los números de "Batch" en cada item referencian el batch de lectura del reporte, que contiene estos archivos:

| Batch | Archivos old docs relevantes |
|-------|------------------------------|
| Batch 2 | `spec-20-sleep-worker.md`, `spec-26-bayesian-forgetfulness.md` |
| Batch 3 | `spec-30-rehydration.md`, `spec-31-contextual-priming.md` |
| Batch 4 | `ADR-001.md`, `ADR-002.md`, `ADR-003.md`, `cp-index-design.md`, `pipeline-architecture.md` |
| Batch 8 | `report.md` (auditoría), `plan-accion-alto-rendimiento.md`, `FASE-5-auditoria.md` |
| Batch 9 | `implementation_plan.md`, `FASE-2-implementation.md`, `SEC-WAL-plan.md` |
| Batch 11 | `python_sdk_design.md`, `pgwire_spec.md`, `langchain_integration.md` |
| Batch 12 | `hnsw_execution.md`, `temporal_scoring.md`, `tiered_storage.md`, `eviction_policy.md`, `electric_vs_chemical_synapse.md` |
| Batch 13 | `marketing_strategy.md`, `investor_pitch_deck.md`, `gtm_timeline.md` |
| Batch 14 | `BENCHMARKS.md`, `FUZZING.md`, `MEMORY_TELEMETRY.md`, `REPO_CHECKLIST.md` |
| Batch 16 | `ADVANCED_TOKENIZER.md` (docs_backup) |
| Batch 17 | `pilot/PILOT_ONBOARDING.md`, `pilot/PILOT_FEEDBACK_FORM.md` |
| Batch 18 | `GraphRAG/README.md` (VantaDB-MPTS) |
| Batch 20 | `VantaDB_CLI_TUI_Design_Spec.md`, `how_hybrid_search_works.md`, `sqlite_for_ai_agents.md`, `why_i_built.md`, `FROM_CHROMADB.md`, `FROM_LANCEDB.md` |

---

### 🔴 Alta — Esto sí se perdió y duele

> Features implementables, iban con el mercado, abandonadas sin razón técnica sólida.

---
#### OLD-01: PGWire (PostgreSQL wire protocol)

- **Archivos old docs:** Batch 11 (`pgwire_spec.md`)
- **Qué era:** Compatibilidad con protocolo PostgreSQL para usar psql, pgAdmin, DBeaver, y todo el ecosistema PG.
- **Por qué es pérdida grave:** pgvector es el vector DB más usado del mundo — no porque sea el mejor, sino porque ya está en PostgreSQL. PGWire le daría a VantaDB acceso instantáneo a todo el ecosistema PG: ORMs (SQLAlchemy, Prisma), tools (pgAdmin, DBeaver), hosting (Supabase, Render). **Esto solo valdría más que todas las specs cognitivas juntas.** Sería el diferenciador #1: "PostgreSQL-compatible vector DB embedida en Rust."
- **Viable:** Sí. Existen crates Rust maduros: `pgwire` (tokio-postgres wire protocol), `pg_extend` (para extensiones PG nativas). Implementación factible en 2-3 semanas. No requiere ser PostgreSQL completo — solo el protocolo wire para aceptar conexiones PG y traducir queries SELECT/INSERT a IQL internamente.
- **Dependencias:** Ninguna técnica. Decisión de producto.

---
#### OLD-02: GraphRAG pipeline formal

- **Archivos old docs:** Batch 18 (`GraphRAG/README.md` en VantaDB-MPTS), Batch 20 (`GraphRAG/README.md` en docs_backup)
- **Qué era:** Pipeline completo de GraphRAG: seed node → expand edges → retrieve subgraph → generate context. Aprovecha que VantaDB tiene grafo + vector en el mismo motor.
- **Por qué es pérdida grave:** VantaDB es la **ÚNICA** embedida que tiene grafo + vector en el mismo motor. Qdrant no tiene grafo. LanceDB no tiene grafo. Chroma no tiene grafo. Armar el pipeline de GraphRAG es un feature que ningún competidor directo puede igualar sin cambiar su arquitectura. El traversal de grafo (`bfs_traversal` en `src/graph/`) existe y está implementado; lo que falta es la orquestación: wrapper que recibe un seed, expande N hops, recupera los vectores de los nodos alcanzados, scores por similitud semántica, y devuelve el subgrafo enriquecido.
- **Viable:** ~1-2 semanas. El traversal BFS existe y funciona. Solo falta: (1) wrapper `GraphRAGPipeline` en `src/graph/rag.rs`, (2) API endpoint `graphrag(query, seed_id, max_hops, top_k)`, (3) integración con el contexto generator para producir el prompt final con el subgrafo serializado como contexto. No requiere cambios en el motor HNSW/BM25.
- **Dependencias:** Auto-embedding (DRV-123 en backlog TIER 4) recomendado para DX completa, pero funcional sin él si el usuario provee vectores.

---
#### OLD-03: Chaos testing (Jepsen/Maelstrom)

- **Archivos old docs:** Batch 8 (`report.md`, `FASE-5-auditoria.md`), Batch 14 (`FUZZING.md`)
- **Qué era:** Suite de chaos testing para certificar linearizabilidad y comportamiento ACID bajo caos de red, crashes, particiones.
- **Por qué es pérdida grave:** Sin esto, los claims ACID de VantaDB no tienen respaldo. Cualquier cliente enterprise que evalúe la DB va a preguntar "¿y cómo sabés que no perdés datos bajo partición de red o crash?" Hoy la respuesta es "implementamos WAL con CRC32C", que no es lo mismo que "probado bajo Jepsen". La metodología ya está documentada en `docs/report.md` y `docs/FUZZING.md` — plan de 6 fases con escenarios de caos específicos para VantaDB. Solo falta ejecutar.
- **Viable:** ~2-3 semanas. Usar Maelstrom (herramienta de tests distribuidos de Jepsen) con el driver personalizado para VantaDB. Los escenarios ya están definidos: (1) kill -9 del proceso durante INSERT, (2) fsync falla intermitentemente, (3) WAL truncado a mitad de escritura, (4) partición de red si se usa WAL shipping. Ejecutar en CI como test semanal, no blocking.
- **Dependencias:** Docker. WAL shipping existente.

---
#### OLD-04: OpenTelemetry tracing

- **Archivos old docs:** Batch 14 (`MEMORY_TELEMETRY.md`)
- **Qué era:** Instrumentación completa con OpenTelemetry para tracing de queries, WAL writes, index builds, memory usage.
- **Por qué es pérdida grave:** Enterprise observabilidad es requisito #1 para cualquier cliente que pague. Sin tracing, debuggear una DB embedida en producción es a ciegas — no sabés qué query es lenta, dónde está el bottleneck, por qué crece la memoria. `MEMORY_TELEMETRY.md` documenta el contrato completo de telemetría planificada, pero no hay código de OTel en el binary. El contrato incluye: span por query (parse → plan → exec → score), span por WAL write (fsync latency, batch size), span por index build (HNSW insert, BM25 merge), métricas de memory pool, métricas de cache hit rate.
- **Viable:** ~1 semana. La crate `opentelemetry` para Rust es madura (`.01` estable). Instrumentar los 3 hotspots: (1) `executor.execute()` crear span raíz, (2) `wal::write()` crear span hijo, (3) `hnsw::search()` crear span de index. Export via OTLP o stdout. Configurable vía `unified_config` con feature flag.
- **Dependencias:** Ninguna. Feature flag independiente.

---
#### OLD-05: Search Quality v2 (Unicode + snippets)

- **Archivos old docs:** Batch 14 (`REPO_CHECKLIST.md`), Batch 16 (`ADVANCED_TOKENIZER.md` en docs_backup)
- **Qué era:** Tokenizer avanzado con Tantivy, stemming, stopwords, Unicode folding, soporte multilingüe, phrase queries con snippets públicos.
- **Por qué es pérdida grave:** El tokenizer actual (`lowercase-ascii-alnum`) no soporta español (tildes, ñ), ni stemming, ni snippets. Para ser "SQLite para agentes" la búsqueda de texto debe funcionar en cualquier idioma. El advanced-tokenizer ya existe como default feature (TSK-123 completado), pero los snippets públicos y el Unicode folding completo siguen pendientes desde RELIABILITY_GATE.
- **Viable:** ~3-4 días. Advanced tokenizer ya es default. Cerrar: (1) Unicode normalization (NFC/NFD) en el tokenizer, (2) exponer `snippet()` en la API pública que devuelva el contexto alrededor del match, (3) agregar stopword lists para EN/ES.
- **Dependencias:** Ninguna. Continuación de trabajo existente.

---
#### OLD-06: Blog posts (3 artículos técnicos completos)

- **Archivos old docs:** Batch 13 (`marketing_strategy.md`), Batch 20 (`how_hybrid_search_works.md`, `sqlite_for_ai_agents.md`, `why_i_built.md`)
- **Qué era:** Tres artículos técnicos ya escritos y listos para publicar: (1) how_hybrid_search_works, (2) sqlite_for_ai_agents, (3) why_i_built_vantadb.
- **Por qué es pérdida grave:** Están **completos**. No falta escribirlos, no falta investigar — falta publicarlos. Esto es tráfico orgánico gratis, credibilidad técnica, y contenido para HN launch. Cada día que no se publican es contenido muerto. El artículo "sqlite_for_ai_agents" es el posicionamiento exacto de VantaDB y debería ser la página principal del docs site.
- **Viable:** ~2-3 días. (1) Revisar y actualizar a v0.3.0 API, (2) publicar en blog.vantadb.com o Medium/Dev.to, (3) linkear desde README. El artículo "why_i_built" funciona como post de HN launch directo.
- **Dependencias:** Decisión de publicación. No técnica.

---

### 🟡 Medio — Podría tener valor pero no es crítico

---
#### OLD-07: AutoHot/Cold tiering (STN/LTN simplificado)

- **Archivos old docs:** Batch 12 (`tiered_storage.md`, `governor_design.md`)
- **Qué era:** Separación lógica automática entre datos calientes (RAM, FP32) y fríos (disco, SQ8 mmap). Promoción/democión automática basada en patrones de acceso.
- **Por qué es viable:** Para "memoria de agente AI" a largo plazo, algunos vectores se acceden siempre y otros nunca. Sin tiering automático, todo vive en RAM o todo en disco. `VectorRepresentations::Full` vs `QuantizedSQ8` ya existe en `src/vector/governor.rs` como semilla, pero sin política automática de movimiento entre tiers.
- **Implementación:** ~1 semana. La semilla `QuantizationGovernor` existe. Solo falta: (1) medir hit rate por página, (2) si baja de un umbral → demover a SQ8 en mmap, (3) si sube → promover a FP32 en RAM. Política configurable vía `unified_config`.
- **Dependencias:** Ninguna. Usa infraestructura existente.

---
#### OLD-08: Life Insurance / snapshots hard-link

- **Archivos old docs:** Batch 13 (`marketing_strategy.md` — referenciado como "life insurance feature")
- **Qué era:** Sistema de snapshots zero-cost antes de operaciones destructivas usando hard links (o `cp --reflink` en COW filesystems). `vanta-cli snapshot create` toma un snapshot inmediato, `vanta-cli snapshot restore` lo restaura.
- **Por qué es viable:** En una DB embedida, las operaciones destructivas (delete_all, drop collection, vacuum) no tienen "undo". Con hard links, un snapshot es instantáneo (0 bytes extra en disco si nada cambia) porque hard linkea el directorio de datos. Solo los archivos que cambian después del snapshot ocupan espacio nuevo. Es el mismo mecanismo que usan ZFS/Btrfs snapshots pero a nivel filesystem POSIX.
- **Implementación:** ~3-4 días. (1) `SnapshotManager` en `src/storage/snapshot.rs` que hace hard link del directorio de datos a `./snapshots/<name>/`, (2) comando `vanta-cli snapshot create <name>` y `vanta-cli snapshot restore <name>`, (3) integración con WAL: antes de restore, hacer backup del WAL actual. Usar `std::fs::hard_link()` en Windows y `link()` en Unix. Para COW filesystems, detectar y usar `ioctl` de reflink.
- **Dependencias:** Ninguna. Solo syscalls POSIX estándar.

---
#### OLD-09: Olvido Bayesiano (hit decay)

- **Archivos old docs:** Batch 2 (`spec-26-bayesian-forgetfulness.md`), Batch 12 (`temporal_scoring.md`, `eviction_policy.md`)
- **Qué era:** Sistema de decaimiento de scores basado en frecuencia de acceso. Una entrada con hits=0 durante semanas se degrada automáticamente.
- **Por qué es viable:** Para el caso de uso principal (memoria de agente AI), el olvido automático de memorias no accedidas es real. No es "bayesiano" literal — es un TTL extendido con scoring exponencial. El `temporal_scoring` ya existe con `hits`, `last_accessed`, `eviction_score`; solo falta agregar el decaimiento temporal en el cálculo de score.
- **Implementación:** ~3-4 días. Modificar `EvictionPolicy` en `src/storage/eviction.rs` (línea ~14 actual: select based on score) para agregar decay factor exponencial: `score = hits / (1 + α·elapsed_days)`. El `ResourceGovernor` en `src/governor/mod.rs` ya maneja watermarks.
- **Dependencias:** Ninguna. Usa infraestructura existente.

---
#### OLD-10: Sinapsis eléctrica (index-free adjacency)

- **Archivos old docs:** Batch 12 (`electric_vs_chemical_synapse.md`), Batch 4 (arquitectura grafos)
- **Qué era:** Para edges en el grafo, saltar por punteros directos (direcciones de memoria/offset) en lugar de lookup por ID. Inspirado en Neo4j.
- **Por qué es viable:** Para grafos traversal-heavy (GraphRAG, multi-hop queries), el ID lookup por HashMap es O(1) amortizado pero con penalidad de caché. Index-free adjacency usando offsets dentro del mmap permitiría O(1) real con localidad de referencia. VantaDB almacena edges como `(source, target, weight)` en `src/storage/graph.rs`; cambiarlos a offsets raw dentro del page mmap es una optimización contenida.
- **Implementación:** ~1 semana. No cambiar la API pública (edges siguen siendo source→target). Solo cambiar representación interna en `GraphEdge` para que `target` sea un offset dentro del mismo page en lugar de un ID global. Aplicable solo a edges intra-page.
- **Dependencias:** Requiere page structure mmap consolidada. Postergable hasta después de HNSW multi-capa.

---
#### OLD-11: CLI/TUI interactivo

- **Archivos old docs:** Batch 20 (`VantaDB_CLI_TUI_Design_Spec.md` — 1106 líneas)
- **Qué era:** REPL shell tipo `vantadb-cli` con `\connect`, `\status`, comandos tipo psql. TUI con ratatui con paneles, mocks diseñados.
- **Por qué es pérdida grave:** El CLI actual es imperativo (`vanta-cli put --key X --value Y`). No hay REPL, no hay exploración interactiva de datos. Para una DB que se posiciona como "SQLite para agentes", no tener una shell interactiva es un gap enorme en developer experience. El spec está completamente diseñado en `VantaDB_CLI_TUI_Design_Spec.md` (1106 líneas). CLI-01 existente cubre "REPL/TUI" genéricamente pero no referencia el spec completo.
- **Viable:** Spec de 1106 líneas ya escrito. ratatui + clap. ~1-2 semanas. Incluir: REPL con historial, paneles de exploración de colecciones, query builder visual, exportación de resultados.
- **Dependencias:** Ninguna. Proyecto aparte dentro del repo.

---
#### OLD-12: Pilot program formal

- **Archivos old docs:** Batch 17 (`pilot/PILOT_ONBOARDING.md`, `pilot/PILOT_FEEDBACK_FORM.md` en docs_backup)
- **Qué era:** Programa de early adopters con onboarding estructurado: script `agent_memory_loop.py`, formulario de feedback, sesiones de pairing, canal privado. Diseñado para conseguir los primeros 10-20 usuarios reales antes del launch público.
- **Por qué es viable:** El GTM actual es "build it and they will come" — no funciona para DBs embedidas. Los early adopters necesitan: (1) un caso de uso obvio (agent memory), (2) un script que funcione en 5 minutos, (3) un humano que los ayude cuando algo falla. El script `agent_memory_loop.py` ya existe en `python/examples/`. El formulario de feedback está diseñado. El onboarding está redactado. Solo falta lanzarlo: anuncio en r/LocalLLaMA, HN, y Discord del proyecto.
- **Implementación:** ~1 semana. (1) Verificar que `agent_memory_loop.py` funciona con v0.3.0, (2) publicar docs/pilot/ en `docs/`, (3) crear canal `#pilot` en Discord, (4) escribir post de reclutamiento para HN/r/LocalLLaMA, (5) ofrecer sesiones de 30min a los primeros 10 inscritos.
- **Dependencias:** PyPI publication (gap analysis 🔴 Alta) — sin PyPI, el onboarding es manual.

---
#### OLD-13: Explainable ranking (explain flag)

- **Archivos old docs:** Batch 9 (`implementation_plan.md`, `FASE-2-implementation.md`)
- **Qué era:** Flag `explain` en queries de búsqueda para obtener desglose detallado de scores BM25/RRF por resultado.
- **Por qué es viable:** `explain_memory_search()` existe en Python SDK pero no expuesto en API pública completa. Devuelve desglose BM25/RRF por resultado: qué término de la query matcheó, qué peso tuvo el vector score, cómo se fusionó en RRF. Esto es debug UX para developers que ningún competidor ofrece.
- **Implementación:** ~2-3 días. Exponer `explain` como parámetro en `search()` de Python SDK y CLI. El backend `ExplainableRanking` ya existe en Rust core.
- **Dependencias:** Ninguna. Feature ya implementada en backend, solo falta exponer.

---
#### OLD-14: MessageThread / GcWorker para agentic chat

- **Archivos old docs:** Batch 11 (`python_sdk_design.md`, `agent_memory_api.md`)
- **Qué era:** Primitivas conversacionales para agentes AI: MessageThread (hilo de mensajes con context window management) y GcWorker (garbage collector de memorias viejas).
- **Por qué es viable:** Para posicionar VantaDB como "memoria de agente", tener primitivas de chat (crear hilo, agregar mensaje, resumir contexto, podar historia) es un feature esperado. GCWorker existe parcial en código (`src/worker/gc.rs`) pero nunca conectado al flujo de chat.
- **Implementación:** ~1 semana. (1) `MessageThread` struct con add_message(), get_context(), trim_history(), (2) conectar GCWorker para podar mensajes viejos automáticamente, (3) exponer en Python SDK.
- **Dependencias:** Ninguna. Código de worker existe.

---

### 🟢 Bajo — Quick wins de ~1 día

---
#### OLD-15: Distancia Euclidiana L2

- **Archivos old docs:** Batch 9 (`implementation_plan.md`)
- **Qué era:** Soportar distancia Euclidiana (L2) como métrica de similitud además de Cosine.
- **Por qué es viable:** Hoy solo Cosine similarity. Muchos modelos de embedding (OpenAI text-embedding-3-small, Cohere embed-multilingual) prefieren o recomiendan L2. `distance.rs` en `src/index/` ya tiene estructura SIMD para `euclidean_distance_squared_f32` (usada internamente en HNSW) pero no expuesta como métrica de query.
- **Implementación:** ~2 días. (1) Agregar `DistanceMetric::Euclidean` enum variant, (2) hook en query path para usar L2 en vez de Cosine, (3) exponer en Python SDK como `metric="cosine"|"l2"`.
- **Dependencias:** Ninguna. Código de distancia existe.

---
#### OLD-16: WAL rotation a 256MB

- **Archivos old docs:** Batch 9 (`SEC-WAL-plan.md`)
- **Qué era:** Rotación automática del WAL cuando alcanza 256MB para evitar crecimiento infinito.
- **Por qué es viable:** WAL compaction existe pero rotación formal no. Sin rotación, el WAL puede crecer sin límite en workloads de alta escritura. Un solo archivo WAL enorme es difícil de backup, replica, y debuggear.
- **Implementación:** ~1 día. En `src/storage/wal.rs`, agregar check post-append: si `wal_size > 256MB`, iniciar rotación (cerrar WAL actual → rename a `.old` → abrir nuevo WAL). Usar `max_wal_size` de `unified_config`.
- **Dependencias:** Ninguna.

---
#### OLD-17: Migration guides públicos (FROM_CHROMADB, FROM_LANCEDB)

- **Archivos old docs:** Batch 20 (`FROM_CHROMADB.md`, `FROM_LANCEDB.md`)
- **Qué era:** Guías completas de migración desde ChromaDB y LanceDB a VantaDB.
- **Por qué es viable:** Están **completas y escritas**. Solo falta: (1) mover a `docs/guides/`, (2) linkear desde README en sección "Migrating from...", (3) actualizar API calls si cambiaron desde que se escribieron.
- **Implementación:** ~1 día. Copy + review + link.
- **Dependencias:** Ninguna.

---
#### OLD-18: Query TEMPERATURE parameter (diversidad controlada)

- **Archivos old docs:** Batch 12 (`temperature_control.md`)
- **Qué era:** Parámetro TEMPERATURE para controlar softmax temperature sobre scores de resultados. Temp alta → resultados más diversos (scores planos). Temp baja → resultados más precisos (scores sharp).
- **Por qué es viable:** Existe como parámetro interno de query en `src/query/temperature.rs` pero no expuesto en API pública. Útil para recommendation systems (temp alta → más exploración) y agent memory (temp baja → más precisión).
- **Implementación:** ~1 día. Exponer `temperature: Option<f32>` en search() de Python SDK y CLI. Default = 1.0 (sin cambio). Rango 0.1-2.0.
- **Dependencias:** Ninguna. Código interno existe.

---

### ⚪ Futuro / Con Dependencias

---
#### OLD-19: Rehidratación desde shadow archive

- **Archivos old docs:** Batch 3 (`spec-30-rehydration.md`), Batch 12 (`shadow_kernel.md`)
- **Qué era:** Capacidad de "recordar" datos archivados cuando un patrón de acceso similar reaparece. Equivalente a cache promotion/demotion de disco a RAM.
- **Por qué es viable:** El concepto `shadow_kernel` ya existe como semilla en `src/kernel/shadow_kernel.rs` pero sin política de rehidratación. Para agente memory, una memoria que estuvo fría y vuelve a ser relevante debería promoverse automáticamente a hot tier.
- **Implementación:** ~1 semana. Hook en el path de query: si el score de un resultado frío supera un umbral, copiarlo de vuelta a hot storage.
- **Dependencias:** AutoHot/Cold tiering (OLD-07). Hacer después de ese.

---
#### OLD-20: Contextual Priming (cache warming predictivo)

- **Archivos old docs:** Batch 3 (`spec-31-contextual-priming.md`), Batch 12
- **Qué era:** Pre-cargar vecinos frecuentes de un nodo cuando se accede a él. Predictivo basado en patrones históricos de traversal.
- **Por qué es viable:** Cache warming predictivo es una técnica estándar. Cuando haces una query HNSW por el vector A y siempre terminas en los mismos vecinos B, C, D, esos vecinos deberían estar precargados. Se implementa con un HashMap de frecuencias de co-acceso: `(source, neighbor) → count`.
- **Implementación:** ~2-3 días. Agregar `CoAccessTracker` en `src/hnsw/search.rs` que registre pares query→resultado. Segundo lookup: al buscar por A, pre-cargar en L2 cache los vecinos con co-access score alto. Desactivado por defecto (feature flag).
- **Dependencias:** Ninguna. Es puramente aditivo.

---
#### OLD-21: CP-Index formal (query routing inteligente)

- **Archivos old docs:** Batch 4 (`cp-index-design.md`, `pipeline-architecture.md`)
- **Qué era:** Content-Provider Index: ruteador de queries al sub-index correcto (BM25 vs HNSW vs Graph) basado en el tipo de query.
- **Por qué es viable:** Hoy el routing BM25 vs HNSW vs Graph es ad-hoc en el executor (`src/executor.rs`). Formalizarlo en un CP-Index mejoraría: (1) performance en queries mixtas, (2) extensibilidad para nuevos index types, (3) capacidades de explain.
- **Implementación:** ~1 semana. (1) Crear `CPIndexRouter` que inspeccione el query plan y decida qué índice(s) usar, (2) migrar routing actual del executor al CP-Index, (3) exponer métricas de ruteo.
- **Dependencias:** DRV-121/122 (Planner AST + IQL completo). Hacer después de esos.

---
#### OLD-22: Apache Arrow columnar export

- **Archivos old docs:** Batch 4 (`columnar-export-design.md`)
- **Qué era:** Exportación de resultados de query en formato Apache Arrow columnar para integración directa con Pandas, DataFusion, y ML pipelines.
- **Por qué es viable:** `columnar.rs` existe en `src/sdk/columnar.rs` pero como implementación mínima. Productizarlo como feature público permitiría: (1) zero-copy a Pandas via PyArrow, (2) integración con DataFusion para SQL queries, (3) exportación eficiente de batches grandes.
- **Implementación:** ~3-4 días. (1) Expandir `columnar.rs` para soportar schemas completos (metadata, vectores, scores), (2) exponer `to_arrow()` en Python SDK que devuelva `pyarrow.Table`, (3) agregar batch export para resultados >1M filas.
- **Dependencias:** Ninguna.
---

## TIER 6 — 🆕 Competitive Features (Vector + Graph DBs)

> **Fuente:** Análisis de 27 archivos de `VANTADB DOC OLD/` (9 vector DBs + 8 graph DBs + 10 arquitectura). Ver `docs/audit-reports/competitive-features-consolidated-report.md` y `docs/audit-reports/deep-analysis-{vector,graph,arch}.md` para análisis completo.
> **Total:** 30 items (7 🔴 + 17 🟠 + 6 🟡)
> **IDs:** COMP-001→COMP-030

### 🔴 Alta — Features competitivas críticas para adopción

---

#### COMP-001: SQ8/PQ Quantization (4x-16x compression)
- **Fuente:** ARC-019 (Arquitectura) / QDR-009 (Qdrant)
- **Qué es:** Cuantización escalar (f32→i8, 4x) y Product Quantization (subespacios, 16x). SQ8 existe como `VectorRepresentations::SQ8` pero no expuesto en query path. PQ requiere entrenamiento K-means por subespacio.
- **Por qué es crítico:** Sin cuantización, VantaDB no puede procesar datasets >1M vectores en RAM. El benchmark SIFT 1M (127s) es síntoma directo — 4x compresión = 4x más vectores en RAM = page faults eliminados. Además, habilita el tier Pro (monetizable).
- **Esfuerzo:** 🟡 2-3 semanas. SQ8: ~1 semana (exponer en query path). PQ: ~2 semanas (entrenamiento + distancia asimétrica).
- **Dependencias:** ARC-014 (HNSW Persistence) recomendado antes para evitar rebuild post-cuantización.

---

#### COMP-002: HNSW Persistence (no rebuild en startup)
- **Fuente:** ARC-014 (wal_strategy.md)
- **Qué es:** Serializar neighbor lists del grafo HNSW a disco en lugar de rebuildear desde vectores en cada cold start. Hoy cada startup rebuild: 3-5s para 100K, 30-60s para 1M+.
- **Por qué es crítico:** Una DB "embebida" que tarda 30s+ en abrir no es embebida. Para agentes AI en serverless/edge (que hacen cold starts frecuentes), este tiempo es unacceptable.
- **Esfuerzo:** 🟡 1-2 semanas. CPIndex.nodes (DashMap) ya tiene neighbor lists en memoria. Serializar con bincode + load condicional.
- **Dependencias:** Ninguna técnica.

---

#### COMP-003: In-filter traversal (bitset durante HNSW walk)
- **Fuente:** QDR-004 (Qdrant) / GRF-006 (Neo4j)
- **Qué es:** Durante la caminata del grafo HNSW, intersectar un bitset de filtro en cada hop. Solo se exploran nodos que matchean el filtro. Elimina el post-filter overhead y el problema de zero-results.
- **Por qué es crítico:** Para RAG con filtros de metadata (el caso de uso #1), el post-filtering actual puede descartar todos los K resultados. Sin in-filter traversal, VantaDB no es competitivo para filtered search.
- **Esfuerzo:** 🟢 ~50 líneas en `graph.rs:search_nearest()`. `FilterBitset` ya existe en `HnswNode`. Solo falta intersectarlo durante el traversal.
- **Dependencias:** PIN-005 (RoaringBitmaps) para construir los bitsets de metadata eficientemente.

---

#### COMP-004: Bitset-based filtering + soft deletes
- **Fuente:** MLV-005 (Milvus)
- **Qué es:** Usar RoaringBitmaps para tracking de deletes (soft deletes: marcar bit, no remover nodo) y filtros. La búsqueda AND-ea bitsets de deleted mask + filter mask. Periodically compact.
- **Por qué es crítico:** Base para CRUD en HNSW sin rebuild completo. Sin soft deletes, cada delete requiere reconstruir el índice o dejar tombstones que degradan recall.
- **Esfuerzo:** 🟢 3-5 días. `croaring` crate está disponible. VantaDB ya tiene `FilterBitset` como concepto.
- **Dependencias:** Previo a COMP-011 (HNSW CRUD con tombstones).

---

#### COMP-005: HNSW params configurables (M, ef_construction, ef_search)
- **Fuente:** GRF-005 (Neo4j/universal)
- **Qué es:** Exponer M (conexiones por nodo), ef_construction (calidad build), ef_search (calidad search) como parámetros configurables por índice. Valor por defecto sensible pero override por colección.
- **Por qué es crítico:** Feature mínima esperada por cualquier usuario de bases vectoriales. Sin parámetros ajustables, no hay tuning para tradeoff recall/latencia. Todos los competidores (Qdrant, Milvus, pgvector) los soportan.
- **Esfuerzo:** 🟢 2-3 días. `HnswConfig` ya existe. Extender y exponer en API pública.
- **Dependencias:** Ninguna.

---

#### COMP-006: Edge Label Interning (u32 label_id)
- **Fuente:** ARC-004 (unified_node.md)
- **Qué es:** Reemplazar `Edge.label: String` por `Edge.label_id: u32` + lookup table global. Ahorra ~20 bytes/edge (de 36B a 16B). Matching de labels pasa de O(n) string compare a O(1) integer compare.
- **Por qué es crítico:** Para 1M nodos con ~4 edges/nodo, ~80MB en strings repetidos. Label interning reduce a ~12MB. En travesías SIGUE (hot path de GraphRAG), el matching de labels es O(n) hoy → O(1).
- **Esfuerzo:** 🟢 ~2 días. `Edge` struct tiene solo 6 callers. Label interning es patrón estándar.
- **Dependencias:** Ninguna.

---

#### COMP-007: Bitset inline u128 en UnifiedNode
- **Fuente:** ARC-002 (unified_node.md)
- **Qué es:** Reemplazar `FilterBitset` (Vec<u64> heap-allocated) por u128 inline en UnifiedNode. Ahorra 24-56 bytes/nodo + elimina indirección de heap. Scan de filtros con instrucción única AND (no loop de memoria).
- **Por qué es crítico:** Impacta TODAS las operaciones de filtrado (~40% más rápido). Cada nodo en cada query paga el costo del FilterBitset actual.
- **Esfuerzo:** 🟡 1 semana. Cambio localizado en `node.rs:705` (UnifiedNode) y `node.rs:15` (FilterBitset). ~20 callers en el mismo módulo.
- **Dependencias:** Ninguna. Migración de formato con versionado simple.

---

### 🟠 Media-Alta — Features competitivas importantes

---

#### COMP-008: Pluggable index engine (VecIndex trait)
- **Fuente:** MLV-016 (Milvus)
- **Qué es:** Abstraer operaciones de indexación detrás de un trait VecIndex: Train, Add, Search, Serialize, Load. Permite múltiples implementaciones (HNSW, IVF, DiskANN) hot-swappables.
- **Por qué es valioso:** Desacopla la API de indexación de la implementación. Permite agregar nuevos index types sin cambiar el query pipeline. Habilita third-party index plugins.
- **Esfuerzo:** 🟡 1-2 semanas. Refactor de `CPIndex` para implementar `VecIndex` trait. No cambia el algoritmo, solo la organización.
- **Dependencias:** Previo a COMP-027 (múltiples index types).

---

#### COMP-009: Binary bulk import (5-10x faster than INSERT)
- **Fuente:** PGV-008 (pgvector)
- **Qué es:** Formato binario para importación masiva de vectores. Datos vectoriales directamente memory-mappeados/streamed, bypassing serialization. Similar a PostgreSQL COPY BINARY.
- **Por qué es valioso:** Para benchmarks y migraciones, la diferencia entre minutos y horas. Esencial para onboarding de usuarios con datasets existentes.
- **Esfuerzo:** 🟢 3-4 días. Formato FlatBuffer/bincode para vectores + metadata. Endpoint CLI `vanta-cli import --binary dataset.vec`.
- **Dependencias:** Ninguna.

---

#### COMP-010: Auto-embedding (embedding function abstraction)
- **Fuente:** CHR-004/CHR-011 (Chroma)
- **Qué es:** Capa de abstracción para proveedores de embedding: EF(texts) → vectors. Soporta OpenAI, Ollama, HuggingFace, custom providers. Usuario pasa texto crudo, VantaDB genera vectores automáticamente.
- **Por qué es valioso:** Elimina la fricción de generar embeddings client-side. Chroma y Pinecone lo tienen como feature principal. Mejora DX significativamente.
- **Esfuerzo:** 🟡 1-2 semanas. Trait `EmbeddingProvider` + implementaciones para Ollama (ya existe LlmClient) y OpenAI.
- **Dependencias:** DRV-123 (auto-embedding on INSERT) ya existe como feature flag.

---

#### COMP-011: HNSW CRUD con tombstones + async cleanup
- **Fuente:** WEV-001 (Weaviate)
- **Qué es:** Custom HNSW con soporte de updates/deletes elementales sin rebuild completo. Tombstones mask deleted nodes; periodic cleanup thread remueve tombstoned entries y repara enlaces.
- **Por qué es valioso:** Esencial para workloads con updates frecuentes (memoria de agente, datasets cambiantes). Sin esto, cada delete/update requiere rebuild completo del índice.
- **Esfuerzo:** 🟡 2-3 semanas. Depende de COMP-004 (soft deletes) como base. Cleanup thread similar al GcWorker existente.
- **Dependencias:** COMP-004 (bitset + soft deletes), COMP-014 (FreshHNSW).

---

#### COMP-012: RoaringBitmaps for metadata indexing
- **Fuente:** PIN-005 (Pinecone) / MLV-005 (Milvus)
- **Qué es:** Cada valor único de metadata tiene su propio RoaringBitmap apuntando a nodos que lo contienen. Filtro `color=red AND size>10` = bitmap intersection. O(1) lookup por valor + compresión nativa.
- **Por qué es valioso:** Sin un metadata index eficiente, el bitset de COMP-003 no tiene de dónde venir. Es el complemento necesario para in-filter traversal.
- **Esfuerzo:** 🟡 1 semana. `croaring` crate para Rust. Metadata index separado del HNSW. Actualización en upsert/delete.
- **Dependencias:** Previo a COMP-003 (in-filter traversal necesita los bitsets).

---

#### COMP-013: Segment optimizer pipeline (Vacuum/Merge/Index)
- **Fuente:** QDR-003 (Qdrant)
- **Qué es:** Tres tipos de optimizadores: VacuumOptimizer (remueve soft-deletes), MergeOptimizer (combina segmentos pequeños), IndexOptimizer (construye HNSW/cuantización en segmentos sealed).
- **Por qué es valioso:** Previene fragmentación del storage. Los soft deletes y writes incrementales fragmentan el dataset sin compactación periódica.
- **Esfuerzo:** 🟡 1-2 semanas. Background thread con scheduler configurable. Similar al GcWorker existente.
- **Dependencias:** COMP-004 (soft deletes), COMP-011 (tombstones).

---

#### COMP-014: FreshHNSW (background repair de enlaces huérfanos)
- **Fuente:** ARC-022 (Documento Maestro)
- **Qué es:** Hilos background que reparan enlaces huérfanos en HNSW generados por borrados masivos, sin bloquear lecturas. Mantiene recall estable bajo cargas delete-heavy.
- **Por qué es valioso:** Sin FreshHNSW, el recall de HNSW se degrada con borrados. Los agents de IA escriben/borran memorias frecuentemente — el recall debe mantenerse estable durante la vida del agente.
- **Esfuerzo:** 🟡 1 semana. Background thread similar al repair worker. Operación O(M×degree) por nodo afectado.
- **Dependencias:** COMP-004 (soft deletes), COMP-011 (tombstones).

---

#### COMP-015: Hybrid Graph+Vector search pipeline
- **Fuente:** GRF-017 (TigerGraph/ArangoDB)
- **Qué es:** Pipeline de búsqueda híbrida: vector search → graph traversal en misma query. Los resultados vectoriales alimentan la navegación estructural sin round-trips externos: `(vector_search(query, k=10) → SIGUE 1..3 --relacion→)`.
- **Por qué es valioso:** Diferenciador #1 de VantaDB. Ningún competidor (Qdrant, Pinecone, Chroma) tiene grafo+vector nativo. Es la feature que justifica "graph+vector en Rust, single binary".
- **Esfuerzo:** 🟡 2-3 semanas. Requiere integrar `CPIndex::search()` con el traversal `SIGUE` existente. El BFS traversal ya funciona.
- **Dependencias:** COMP-005 (HNSW params), COMP-003 (in-filter filtering).

---

#### COMP-016: Supernode mitigation (indexed relationships)
- **Fuente:** GRF-009 (Neo4j)
- **Qué es:** Cuando un nodo tiene millones de relaciones, agrupar edges por label en `HashMap<label_id, Vec<VantaEdgeRecord>>` para búsqueda O(1) por label, evitando escaneo lineal.
- **Por qué es valioso:** Sin esto, un solo nodo popular puede degradar toda la query. Especialmente relevante para RAG donde un "documento" puede tener miles de "chunks". Edge Label Interning (COMP-006) lo complementa.
- **Esfuerzo:** 🟢 3-5 días. Cambio en `UnifiedNode.edges` de Vec plano a HashMap agrupado. Impacto localizado.
- **Dependencias:** COMP-006 (Edge Label Interning para usar u32 labels como keys).

---

#### COMP-017: Accumulators for parallel graph algorithms
- **Fuente:** GRF-050 (TigerGraph)
- **Qué es:** Variables especiales con exclusión mutua para recolectar información durante travesías paralelas: Global (@@), Local (@), Collection (List/Set/Map). Base para algoritmos GDS: PageRank, Centrality, Community Detection.
- **Por qué es valioso:** Sin accumulators, los algoritmos de grafo requieren locks pesados o son secuenciales. Con accumulators lock-free en Rust (AtomicU64, fetch_add), VantaDB puede ejecutar PageRank nativo sin mover datos a Python.
- **Esfuerzo:** 🟡 1-2 semanas. Implementar tipos Accumulator en Rust con crossbeam para epoch-based reclamation.
- **Dependencias:** Ninguna. Base para COMP-022 (GDS library).

---

#### COMP-018: Double-linked relationship chains
- **Fuente:** GRF-003 (Neo4j)
- **Qué es:** Cada edge almacena punteros previo/siguiente tanto para origen como destino. Navegar relaciones de un nodo es O(k) (solo las que existen), no O(n) (todas las relaciones del nodo).
- **Por qué es valioso:** En travesías multi-hop (SIGUE 1..3), cada salto requiere escanear todas las relaciones del nodo. Con cadenas doblemente enlazadas, cada salto cuesta solo las relaciones del label específico.
- **Esfuerzo:** 🟡 1-2 semanas. Implementar como AdjacencyList separada, no modificar UnifiedNode. EdgeIndex existente (DashSet) puede coexistir.
- **Dependencias:** COMP-006 (Edge Label Interning).

---

#### COMP-019: Binary protocol (rkyv/FlatBuffers over gRPC)
- **Fuente:** GRF-004 (Neo4j Bolt / TigerGraph RESTPP)
- **Qué es:** Reemplazar serialización JSON por formato binario zero-copy (rkyv o FlatBuffers) para transporte API. gRPC para streaming bidireccional. Reduce CPU de serialización ~10-100x.
- **Por qué es valioso:** Para queries que devuelven miles de nodos, la serialización JSON es el bottleneck dominante. rkyv permite zero-copy desde la estructura en memoria directo al wire.
- **Esfuerzo:** 🟡 1-2 semanas. Usar rkyv para el formato + tonic (gRPC) como transporte. No requiere protocolo custom.
- **Dependencias:** Ninguna. Puede coexistir con JSON legacy.

---

#### COMP-020: Hybrid search with RRF (Reciprocal Rank Fusion)
- **Fuente:** QDR-016 (Qdrant) / CHR-012 (Chroma) / GRF-029 (ArangoDB)
- **Qué es:** Búsqueda híbrida que combina dense vector similarity + sparse BM25 retrieval usando RRF: score = 1/(k + rank). K configurable (default 60). No requiere normalización de scores entre retrievers.
- **Por qué es valioso:** RRF es más simple y robusto que weighted sum fusion. Los scores de BM25 y cosine similarity tienen distribuciones incompatibles — RRF las fusiona basado en ranking, no en score absoluto.
- **Esfuerzo:** 🟡 1 semana. BM25 existe. Sparse retriever existe. RRF fusion es simple: rank fusion sobre los resultados de ambos retrievers.
- **Dependencias:** Ninguna. BM25 y vector search existen.

---

#### COMP-021: Temporal edges (timestamp-aware relationships)
- **Fuente:** ARC-021 (Documento Maestro)
- **Qué es:** Timestamp en edges para búsquedas cronológicas. Permite queries como "qué conexiones tenía este nodo antes de fecha X" y windowed graph traversal. Pruning automático de relaciones expiradas.
- **Por qué es valioso:** Feature diferenciador para "memoria de agente AI". Los agentes necesitan memoria episódica (qué sabía el agente en momento T). Sin edges temporales, no hay time-travel en el grafo.
- **Esfuerzo:** 🟡 1 semana. Agregar `created_at: u64` (timestamp) y `ttl: Option<u64>` a Edge struct. Modificar traversal para aceptar `BEFORE <timestamp>`.
- **Dependencias:** Ninguna. Feature flag independiente.

---

#### COMP-022: Graph Data Science library (PageRank, centrality, community)
- **Fuente:** GRF-056 (Neo4j GDS / TigerGraph)
- **Qué es:** Librería de algoritmos de grafos nativa en Rust: PageRank, Betweenness Centrality, Louvain Community Detection, BFS, DFS, Shortest Path. Ejecutados directo sobre storage, sin mover datos a sistemas externos.
- **Por qué es valioso:** VantaDB es la única DB embedida que podría tener GDS nativa (grafo+vector en el mismo motor). Para GraphRAG, PageRank y community detection son esenciales. La alternativa hoy es exportar a networkx.
- **Esfuerzo:** 🟡 2-3 semanas. PageRank primero (más simple, más útil). Louvain después (más complejo).
- **Dependencias:** COMP-017 (Accumulators) como base para PageRank.

---

#### COMP-023: 3 filtering strategies (pre/post/in-index)
- **Fuente:** GRF-048 (SurrealDB/Neo4j/ArangoDB/TigerGraph)
- **Qué es:** Las tres estrategias de filtrado implementadas: pre-filter (aplica filtro durante walk HNSW), post-filter (ANN luego filtra), in-index filtering (bitsets en hot path). Optimizador elige según selectividad.
- **Por qué es valioso:** No hay una estrategia superior en todos los casos. Pre-filter gana con filtros poco selectivos. Post-filter gana con filtros muy selectivos. In-index es el default balanceado.
- **Esfuerzo:** 🟡 1-2 semanas. Pre-filter es COMP-003. Post-filter ya existe. In-index requiere integración con metadata index (COMP-012).
- **Dependencias:** COMP-003 (in-filter), COMP-012 (RoaringBitmaps), COMP-028 (SCE).

---

#### COMP-024: ACORN algorithm (second-hop filtered search)
- **Fuente:** QDR-005 (Qdrant)
- **Qué es:** Cuando in-filter traversal da pocos candidatos (filtro muy selectivo), ACORN expande a vecinos-de-vecinos (second hop) para densificar el pool de candidatos antes de re-ranking.
- **Por qué es valioso:** Soluciona el "empty result" problem en filtered search de alta selectividad. Complemento necesario de COMP-003.
- **Esfuerzo:** 🟡 1-2 semanas. Segundo nivel de indirección en el traversal. Feature flag detrás de HnswConfig.
- **Dependencias:** COMP-003 (in-filter traversal).

---

#### COMP-025: JSON shredding (dynamic schema to columns)
- **Fuente:** MLV-003 (Milvus)
- **Qué es:** Dynamic JSON fields se "shreddean" en columnas tipeadas automáticamente. Cada unique key se convierte en columna. Permite SQL-like filtering sin schema definition.
- **Por qué es valioso:** Para metadata heterogénea (cada nodo tiene campos distintos), JSON shredding permite filtrar por cualquier campo sin schema definitions manuales. Milvus lo usa como feature destacado.
- **Esfuerzo:** 🟡 2-3 semanas. Analizador de schemas + column store auxiliar + query pushdown.
- **Dependencias:** Ninguna. Feature independiente.

---

#### COMP-026: Multi-level LSM compaction (L0→L1→L2→L3)
- **Fuente:** PIN-001/PIN-011 (Pinecone)
- **Qué es:** Slabs inmutables promovidos por niveles: L0 (small, merges frecuentes) → L3 (large, merges infrecuentes). Capacidad exponencial por nivel. Spread compaction cost over time.
- **Por qué es valioso:** Para write-heavy workloads con actualizaciones frecuentes. La compactación multi-nivel evita picos de I/O cuando se mergean todos los segmentos a la vez.
- **Esfuerzo:** 🟡 1-2 semanas. Implementar sobre el segment lifecycle actual. Política de promoción configurable.
- **Dependencias:** COMP-013 (segment optimizer pipeline).

---

#### COMP-027: Multiple index types (IVF, DiskANN, SCANN)
- **Fuente:** MLV-007 (Milvus)
- **Qué es:** Además de HNSW, ofrecer IVF_FLAT (balance velocidad/calidad), DiskANN (billones de vectores en NVMe), SCANN (PQ + SIMD). Selector por colección.
- **Por qué es valioso:** HNSW no es óptimo para todos los workloads. IVF es mejor para alta dimensionalidad (>1024). DiskANN es necesario para datasets >RAM (billones de vectores).
- **Esfuerzo:** 🟠 5-10 días. DiskANN es el más complejo (requiere I/O asíncrono + clustering compaction). IVF es más simple.
- **Dependencias:** COMP-008 (VecIndex trait) para abstraer los backends.

---

### 🟡 Medio — Features de madurez y ecosystem

---

#### COMP-028: Semantic Cost Estimator (SCE)
- **Fuente:** ARC-001 (cbo_design.md)
- **Qué es:** Estimador de costos semántico que usa Density Metadata (out-degree promedio) y Radius Entropy (selectividad de vector search) para orden dinámico de ejecución cross-model.
- **Por qué es valioso:** Reemplaza el orden fijo actual (bitset→graph→vector) por routing dinámico. Para queries híbridas complejas, puede elegir el orden óptimo.
- **Esfuerzo:** 🟡 2 semanas. Metadata collector + cost model + planner integration.
- **Dependencias:** DRV-121/122 (Planner AST + IQL completo).

---

#### COMP-029: Node.js/TS bindings via napi-rs
- **Fuente:** ARC-025 (Documento Maestro)
- **Qué es:** Bindings nativos para Node.js via crate napi-rs, generando módulo .node para ecosistema Vercel AI SDK, LangChain.js y agentes TypeScript.
- **Por qué es valioso:** El ecosistema JS/TS es el más grande para AI agents. WASM build existe pero es limitado (sin FS, sin threading). napi-rs da acceso completo al engine desde Node.js.
- **Esfuerzo:** 🟡 2-3 semanas. napi-rs tiene macros para generar bindings automáticamente. El desafío es la API surface.
- **Dependencias:** Ninguna técnica. Decisión de producto.

---

#### COMP-030: Survival Mode (backpressure + Docker OOM prevention)
- **Fuente:** ARC-028 (Documentación Actualizada)
- **Qué es:** Mecanismo que respeta límites de Cgroups/Docker con 10% safe margin. Al acercarse al umbral de memoria, reduce block cache y memtables para evitar OOMKilled.
- **Por qué es valioso:** Para despliegue en contenedores (el formato de deploy más común para agentes). Sin esto, VantaDB puede ser OOMKilled sin warning. La confianza en producción depende de esto.
- **Esfuerzo:** 🟡 1-2 semanas. Integrar memory_governor.rs con Cgroups + Docker memory limits. Tests bajo `--memory 512m`.
- **Dependencias:** Ninguna. ResourceGovernor existe.

---

**Fuente de REV-001→018:** `docs/reviews/2026-07-13-full-review.md` — generado por `vantadb-full-review` skill.
**Fuente de DRV-118→136:** `docs/plans/2026-07-15-cross-ref-docs-vs-code.md` + `docs/plans/2026-07-16-cross-ref-full-pipeline.md` — reconciliación cross-ref findings vs backlog. Reportes: `docs/audit-reports/cross-ref-wave3-report.md` y `docs/audit-reports/cross-ref-wave3-final-report.md`.
**Fuente de OLD-001→022:** `docs/REPORTE_EVALUACION_COMPLETO.md` secciones 6 y 7 — análisis de ~280 archivos VANTADB DOC OLD vía 21 sub-agentes.
**Fuente de COMP-001→030:** `docs/audit-reports/competitive-features-consolidated-report.md` + `docs/audit-reports/deep-analysis-vector.md` + `docs/audit-reports/deep-analysis-graph.md` + `docs/audit-reports/deep-analysis-arch.md` — análisis de 27 archivos VANTADB DOC OLD, 172 features, top 30 priorizados.
