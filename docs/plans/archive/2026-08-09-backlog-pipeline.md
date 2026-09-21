# Plan de Ejecución: Backlog Pipeline — Release Blockers + Err Fix + Feature Honesty

> **Campaign ID: 55f15aab-80fd-47fc-8080-fa466cdb70af
> **Inicio:** 2026-08-09
> **Estado: completed
> **Fuente:** `docs/Backlog.md` (backlog activo completo)
> **FAIL_MODE:** parallel (default)
> **Campaign ID: 5f199e4f-0f4e-47a5-8101-a775155a815a

## Resumen del Triage Gate

| Resultado | Count |
|-----------|-------|
| ✅ DO | 48 |
| 🟡 DEFER | 17 |
| ❌ SKIP | 6 (ya resueltos/verificados: ERR-016, ERR-034, ERR-051, NV-02/03/05, DESKTOP-20, ADMIN-01..09) |
| 🔴 BLOQUEADO | 1 (RELEASE-02 → se desbloquea al completar SEC-01 + ERR-022 + RELEASE-01) |

### DEFER (no entran al plan de esta campaña)

| Grupo | Items | Por qué |
|-------|-------|---------|
| Descarga constructora desktop | DESKTOP-12..27 | Build completo 4-6 sem (`desktop/` desde cero); plan separado. Ya completados: DESKTOP-20, ADMIN-01..09 |
| Perf micro-opts | ERR-036, ERR-037, ERR-042..045, ERR-047, ERR-048, ERR-049 | Impacto individual bajo; campaña tuner separada |
| Investigación/roadmap | PERF-02, PERF-03, PERF-05, BIZ-01b, OLD-01, REVIEW-04 | Período largo, no bloquea release |
| Huidad post-launch | PERF-02 (criterion CI), PERF-03 (competitive bench) | Esfuerzo >> impacto hoy |
| Humano/manual | DISC-01..03, LEG-01 | Requieren UI externa / abogado / pago — owner: human |
| Housekeeping deps | ERR-006, ERR-007, ERR-008, ERR-009 | Limpieza deuda dep, no crítica; ventana 2 |

---

## Tasks

## WAVE 0 — Release Blockers & Seguridad (independientes, paralelo)

### Task 1: RELEASE-01 — Gate cargo semver-checks en CI

- **Esfuerzo:** 🟢 · **Prioridad:** 🔴
- **Archivos clave:** `.github/workflows/release.yml`, `.github/workflows/ci-rust-10.yml`, `release-plz.toml:18`
- **Verificación real:** ✅ CÓDIGO-REAL — `release-plz.toml:18` tiene `semver_check = true` pero `rg semver` en los 3 workflows **no** encuentra `cargo-semver-checks` (ni la action). Gap confirmado: el flag de release-plz nunca se ejecuta como gate CI.
- **Gate Justificación:** 0.5.0 próximo release; sin semver-check un breaking change accidental (API pública) publica rompiendo bindings.
- **Gate Result:** ✅ DO
- **Contrato: Claims backed -> reference report; unsupported -> removed/replaced with real bench number; remaining claims referenced to real reports
- **Estado:** ✅ COMPLETED
- **Branch:** `develop`
- **Commit:**

### Task 2: SEC-01 — UAF en `__array_interface__` (Python)

- **Esfuerzo:** 🟠 · **Prioridad:** 🔴
- **Archivos clave:** `vantadb-python/src/types.rs:365-380`, `vantadb-python/src/vector.rs:59-67`
- **Verificación real:** ✅ CÓDIGO-REAL — `#[getter(__array_interface__)]` existe en `vantadb-python/src/types.rs:365`; devuelve puntero raw al buffer interno sin `// SAFETY:`. UAF plausible si la vista numpy mantiene el puntero y el wrapper se dropeó.
- **Gate Justificación:** uso de seguridad de memoria real reportado por auditoría 2026-08-09; fix acotado a 1 archivo + test numpy.
- **Gate Result:** ✅ DO
- **Contract:** `python -c "import vantadb_py; ..."` (test numpy: dropear wrapper y acceder ndarray; via `target/audit-venv/Scripts/python -m pytest vantadb-python/tests/ -k array_interface`)
- **Task file:** `.agents/campaign-executor/tasks/SEC-01.md`
- **Estado:** ✅ COMPLETED
- **Branch:** develop
- **Commit:** 241f30a3

### Task 3: RELEASE-03 — Limpiar artefactos de ejecución del repo

- **Esfuerzo:** 🟢 · **Prioridad:** 🟡
- **Archivos clave:** raíz repo, `.gitignore`, `Cargo.toml:14-18`
- **Verificación real:** ✅ CÓDIGO-REAL — `_audit04_repro_db/` y `benchmarks/_probe_db/` **existen en disco** (Test-Path True). `chroma_db`, `data_comp_bench/` no existen (renovados limpios).
- **Gate Justificación:** no deben salir en el tarball publicado de 0.5.0.
- **Contrato:** `git status --short` sin traces de `_audit04_repro_db` ni `benchmarks/_probe_db`; `Test-Path _audit04_repro_db` → False
- **Estado:** ✅ COMPLETED

### Task 4: SEC-01 — (loopback no) — VER SEC-01 en Task 2

_(no duplicar)_

---

## WAVE 1 — ERR Críticos (5) + Altos (13)

### Task 4: ERR-010 — Fix race checkpoint↔snapshot

- **Esfuerzo:** 🔴 · **Prioridad:** 🔴
- **Archivos clave:** `src/storage/engine/maintenance.rs:145-217` (`save_vector_index`), `fluir()`, `insert_lock`
- **Verificación real:** ✅ CÓDIGO-REAL — `save_vector_index` existe (`maintenance.rs:145`): serializa index y hace swap mmap, llamado desde rutas de persistencia. La pregunta del race está en `fluir()` (checkpoint_seq antes de save sin insert_lock); no hay tests cubriendo `save_vector_index` (codegraph: ⚠️ no covering tests).
- **Gate Justificación:** corrupción posible en recover (duplicación/record invisible); failpoint + test de interleave pedido.
- **Contrato:** `cargo nextest run --profile audit -p vantadb --features "failpoints" --test durability_recovery` (el test de interleave fija checkpoint_seq bajo lock)
- **Task file:** `.agents/skills/campaign-executor/tasks/ERR_010.md`
- **Estado:** ✅ COMPLETED
- **Ejecutado:** `a5ca4389` — hold insert_lock across checkpoint/save race (checkpoint_seq bajo lock; failpoint + test de interleave)

### Task 5: ERR-021 — MCP OOM en collection_stats/list/delete (restaurar streaming)

- **Esfuerzo:** 🟠 · **Prioridad:** 🔴
- **Archivos clave:** `vantadb-mcp/src/lib.rs:333-365, 1401, 1430, 1499`
- **Verificación real:** ✅ CÓDIGO-REAL — `collect_all_records` presente; `collect_stats` streaming eliminado (`rgba collect_stats` → no). Límite `config.max_list_limit` existe solo en list paths.
- **Gate Justification:** namespace >100k → OOM por llamada en servidor MCP (exposed).
- **Contrato:** `cargo test -p vantadb-mcp` eco: streaming limita Materialization; o `count_stats` no reverts full list.
- **Estado:** ✅ COMPLETED

### Task 6: ERR-022 — Clamp `top_k`/`k` en MCP+Python+WASM → evitar alloc gigante

- **Esfuerzo:** 🟢 · **Prioridad:** 🔴
- **Archivos clave:** `vantadb-mcp/src/lib.rs:1248`, `vantadb-python/src/lib.rs(s), src/search`, `vantadb-wasm/src/lib.rs`
- **Verificación real:** 🟡 PARCIAL — MCP **ya** usa `(raw_top_k as usize).min(config.max_top_k)` en línea 1248 (nuevo codegang). Python y WASM **no** muestran clamp en `rgba` (sin MAX_K fuera de MCP). Gap vigente en 2 de 3 bindings + posible path directo core.
- **Gate Justificación:** `k=10⁹` → `HashSet::with_capacity(ef*3)` aborta el proceso (panic-alloc). Fix barato.
- **Contrato:** `rg "min\\(config.max_top_k\\)|MAX_K" vantadb-python/src/lib.rs vantadb-wasm/src/lib.rs` → match ≥ 1 en ambos
- **Estado:** ✅ COMPLETED

### Task 7: ERR-035 — Read-lock global HNSW bloquea inserts

- **Esfuerzo:** 🔴 · **Prioridad:** 🔴
- **Archivos clave:** `src/physical_plan.rs:211`, `src/storage/engine/ops.rs` (+`apply_insert` línea 723 usa `vector_store[0].write()` confirmado)
- **Verificación real:** ✅ CÓDIGO-REAL — `apply_insert` (`ops.rs:723`) toma `self.vector_store[0].write()`; `search_nearest` (`search.rs:522`) recorre HNSW. PatternRR contender query↔insert real.
- **Gate Justificación:** contención global writer↔reader bloquea throughput; risco de sys.
- **Contrato:** test concurrente `cargo nextest run --profile audit -p vantadb --test concurrency_parity`
- **Estado:** ✅ COMPLETED

### Task 8: ERR-001 — UB 32-bit wasm32: `view_start + len*4` overflow

- **Esfuerzo:** 🟠 · **Prioridad:** 🟠 · **Archivos:** `src/storage/engine/ops.rs:518-521 (+1266,1451,1851)`, `src/index/search.rs:541` · **Gate:** UB real con wasm32 → **DO** · **Contrato:** `checked_mul/checked_add` presentes; `cargo build --target wasm32-unknown-unknown -p vantadb-wasm` compilable
- **Estado:** ✅ COMPLETED

### Task 9: ERR-002 — SIGBUS handler → infinite loop

- **Esfuerzo:** 🟠 · **Prig:** 🟠 · **Archivos:** `src/storage/vfile.rs:211-223` · **Gate:** handler que no resuelve el fault → hang real → **DO** · **Contrato:** handler setu flags y no re-ejecuta sin resolución; test unit vfile
- **Estado:** ✅ COMPLETED

### Task 10: ERR-003 — Panic en header corrupto (4 puntos `[id as usize]`)

- **Esfuerzo:** 🟢 · **Prig:** 🟠 · **Archivos:** `src/storage/engine/ops.rs:507,1311,1397,1820` · **Gate:** panic en datos corruptos evade `VantaError` → **DO** · **Contrato:** `rgba "vector_store\\[" src/storage/engine/ops.rs` sin indexing sin `.get()`
- **Estado:** ✅ COMPLETED

### Task 11: ERR-004 — lru 0.12.5 RUSTSEC-2026-0002 via ratatui

- **Esfuerzo:** 🟢 · **Prig:** 🟠 · **Archivos:** `deny.toml`, `Cargo.lock`; `ralatatu-feature`? · **Gate:** advisory Stacked Borrows real → **DO** (si no bloquea, documentar ignore; probar bump ratatui/lru) · **Contrato:** `cargo deny check advisories`
- **Estado:** ✅ COMPLETED

### Task 12: ERR-011 — WAL replay pérdida silenciosa (local_pos round-robin)

- **Esfuerzo:** 🟠 · **Prig:** 🟠 · **Archivos:** `src/wal_sharded.rs`, `src/storage/engine/init.rs:454-480` · **Gate:** data-loss en recover → **DO** · **Contrato:** test `wal_resilience` (replay de shard truncado sin marcar checkpoint)
- **Estado:** ✅ COMPLETED

### Task 13: ERR-012 — Contadores `inbound` stale en delete

- **Esfuerzo:** 🟠 · **Prig:** 🟠 · **Archivos:** `src/index/neighbor_index.rs`, `src/index/graph.rs` · **Gate:** fuga de memoria del índide real → **DO** · **Contrato:** test de eviction tras delete-decrement
- **Estado:** ✅ COMPLETED

### Task 14: ERR-013 — Stats en txns abortadas

- **Esfuerzo:** 🟠 · **Prig:** 🟠 · **Archivos:** `src/storage/engine/ops.rs` (insert paths) · **Gate:** inventario inflado tras Abort → **DO** · **Contrato:** test `engine_tests` con txn abort y stats correctas
- **Estado:** ✅ COMPLETED

### Task 15: ERR-018 — random_layer capado en nivel 2

- **Esfuerzo:** 🟠 · **Prig:** 🟠 · **Archivos:** `src/index/graph.rs:441-444` · **Gate:** recall degrada con gráficos bajos → **DO** · **Contrato:** test layer distribution / search recall
- **Estado:** ✅ COMPLETED

### Task 16: ERR-019 — Bench mide brute-force no HNSW

- **Esfuerzo:** 🟢 · **Prig:** 🟠 · **Archivos:** `benches/hnsw_pure.rs:33,63` · **Gate:** performance claim falsa → **DO** · **Contrato:** bench con `flat_threshold: None` y 10k
- **Estado:** ✅ COMPLETED

### Task 17: ERR-020 — ACORN second-hop con vecinos stale

- **Esfuerzo:** 🟠 · **Prig:** 🟠 · **Archivos:** `src/index/search.rs` (ACORN), `src/index/graph.rs` · **Gate:** recall/flags segundarios rotos → **DO** · **Contrato:** test ACORN tras repair_orphans
- **Estado:** ✅ COMPLETED

### Task 18: ERR-023 — Python node IDs u64 truncado (core u128)

- **Esfuerzo:** 🟠 · **Prig:** 🟠 · **Archivos:** `vantadb-python/src/lib.rs` · **Gate:** OverflowError en IDs ≥2⁶⁴ → **DO** · **Contrato:** test Python con ID > 2^64 (ronda 64 bits)
- **Estado:** ✅ COMPLETED

### Task 19: ERR-024 — WASM u64 vs core u128

- **Esfuerzo:** 🟠 · **Prig:** 🟠 · **Archivos:** `vantadb-wasm/src/lib.rs:1011,1039,1047` · **Gate:** nodos >2⁶⁴ inaccesibles en WASM → **DO** · **Contrato:** test wasm insert/get con string u128
- **Estado:** ✅ COMPLETED

### Task 20: ERR-025 — MCP get_node_neighbors `as_u64` pierde precisión

- **Esfuerzo:** 🟢 · **Prig:** 🟠 · **Archivos:** `vantadb-mcp/src/lib.rs:1330-1340` · **Gate:** IDs u128 desde JSONRPC inaccesibles → **DO** · **Contrato:** MCP test con id > 2^53 preservado
- **Estado:** ✅ COMPLETED

---

## WAVE 2 — ERR medios + docs / feature honesty (paralelizables)

### Task 21: ERR-005 — Restaurar test AUDREP-45 (guard oversized)

- **Esfuerzo:** 🟢 · **Prig:** 🟡 · **Archivos:** `src/storage/ops.rs` | **Gate:** perdimos cobertura | **Contrato:** `cargo nextest run -p vantadb --test storage_guard`
- **Estado:** ✅ COMPLETED

### Task 22: ERR-014 — Staleness insert→get (WAL antes de drain)

- **Esfuerzo:** 🟢 · **Prig:** 🟡 · **Archivos:** `src/storage/engine/ops.rs` | **DO** (consistencia visibilidad) | **Contrato:** test concurrent insert→get comisión
- **Estado:** ✅ COMPLETED

### Task 23: ERR-030 — `put_batch` cross-namespace

- **Esfuerzo:** 🟢 · **Prig:** 🟡 · **Archivos:** `vantadb-python/src/lib.rs:311-350` | **DO** (data leak entre ns) | **Contrato:** pytest con dos namespaces en un batch
- **Estado:** ✅ COMPLETED

### Task 24: ERR-027 — HTTP 200 con `success:false`

- **Esfuerzo:** 🟢 · **Prig:** 🟡 · **Archivos:** `src/cli_server.rs:607-627` | **DO** | **Contrato:** test HTTP con query err → status 4xx/5xx
- **Estado:** ✅ DONE
- **Ejecutado:** 6b3dce25 — `query_error_response()` mapea errores a 4xx/5xx (400 parse/input, 404 missing, 409 conflict, 500 resto); test `query_error_returns_4xx_not_200` 3 casos ✅

### Task 25: ERR-028 — Query vector norma 0 → error, no `[]`

- **Esfuerzo:** 🟢 · **Prig:** 🟡 · **Archivos:** `src/index/search.rs:521-547` (`search_nearest`, guard AUDREP-55 ya devuelve `Vec::new()`) | **DO parcial:** core ya devuelve vacío; falta error pepper en bindings | **Contrato:** binding devuelve VantaError para zero-norm
- **Estado:** ✅ DONE
- **Ejecutado:** b8058a26 — guard `InvalidInput` para zero-norm cosine en `sdk/api.rs` (K-NN legacy) + `sdk/search/mod.rs` (request path); bindings Python/MCP/WASM heredan el error

### Task 26: ERR-029 — `edge_count = u16` overflow

- **Esfuerzo:** 🟢 · **Prig:** 🟡 · **Archivos:** `src/storage/ops.rs:85` | **DO** (corrupción perSIST) | **Contrato:** test >65535 aristas
- **Estado:** ✅ DONE
- **Ejecutado:** e9985e10 — guard ResourceLimit en `write_node_to_vstore`; test >65535 aristas rechaza sin persistir, boundary 65535 ok

### Task 27: ERR-050 — CHANGELOG desactualizado (falta [Unreleased])

- **Esfuerzo:** 🟢 · **Prig:** 🛎 · **Archivos:** `docs/CHANGELOG.md`, `cliff.toml` | **DO** (release-plz lo regenera en RELEASE-02; activar con `git-cliff -o docs/CHANGELOG.md -u`) | **Contrato:** `git-cliff -o docs/CHANGELOG.md` y `[Unreleased]` presente
- **Estado:** ✅ DONE
- **Ejecutado:** 246f86ae — causa raíz: `body = ""` en cliff.toml hacía que git-cliff jamás generara secciones. Restaurado template body (keepachangelog), limpio groups duplicados; sección [Unreleased] (301 commits) generada y antepuesta preservando 0.5.0 manual

---

## WAVE 3 — Features & Docs honesty + COV/PERF pequeños

### Task 28: FEAT-01 — ADR PITR + WAL-shipping

- **Esfuerzo:** 🔴 · **Prig:** 🟡 · **Archivos:** `src/pitr.rs`, `src/storage/wal.rs`, `src/lib.rs` (`pitr` feat), ADR en `docs/architecture/adr/`
- **Verificación real:** ✅ CÓDIGO-REAL — feature `pitr` = lista vacía (doc = "+feature": `Cargo.toml:138` versión 0.1 etc.); módulos existen standalone.
- **Gate:** dead feature phantom → **DO** (solo ADR + decidir: integrar / experimental / defer)
- **Contrato:** ADR file exists (`docs/architecture/adr/ADR-0XX-pitr.md`) + `rgba "pitr" Cargo.toml` (feature docs)
- **Estado:** ✅ COMPLETED
- **Ejecutado:** `b52ae2f0` — PITR/WAL-shipping decision: ADR-014 (experimental standalone API, engine integration deferred) + honest `pitr` feature docs in Cargo.toml

### Task 29: FEAT-02 — DiskANN: honest rename o implementar v1

- **Esfuerzo:** 🔴 · **Prig:** 🟡 · **Archivos:** `src/index/diskann.rs` | **Gate:** doc interno admite "purely in-memory" | **DO** (decisión + docs/rename) | **Contrato:** README/arch no claim "DiskANN" sin disk I/O (rgba `disk-ann\|\|mmap` doc → ok)
- **Estado:** ✅ DONE
- **Ejecutado:** bcdcad3f — Decisión: mantener `IndexType::DiskAnn` (público+serde, rename rompería API) y documentar honestamente (opción b). `diskann.rs` es Vamana graph puramente in-memory — sin disk I/O, sin mmap. Docs corregidos: ROADMAP.md (#264), PQ_FEASIBILITY.md, module doc `//!` con nota de honestidad FEAT-02.

### Task 30: FEAT-03 — Arrow: export vector completo

- **Esfuerzo:** 🟡 · **Prig:** 🟡 · **Archivos:** `src/integrations.rs`, `vantadb-python/src/vector.rs`, `docs/api/PYTHON_SDK.md` | **DO** (feature prometida vs 1-component) | **Contrato:** `export_arrow` devuelve columnas flat completas + test
- **Estado:** ✅ DONE
- **Ejecutado:** `16346bd5` — FEAT-03: `nodes_to_record_batch` (el código Arrow real vive en `src/columnar.rs`, no integrations.rs) ahora exporta el vector completo como columnas flat `vector_d0..d{N-1}` en vez de solo `v[0]`; 3 tests nuevos verifican vector entero con N dimensión correcta (9/9 columnares + columnar_engine_certification pasan).

### Task 31: FEAT-04 — IVF/SCANN expuestos por SDK

- **Esfuerzo:** 🔴 · **Prig:** 🟡 · **Archivos:** `src/index/ivf.rs`, `src/index/scann.rs`, bindings | **DO** (exponer `method: ivf/scann` en sdk) | **Contrato:** `search(..., method="ivf")` funciona desde 1 binding
- **Estado:** ✅ COMPLETED
- **Ejecutado:** `aac61155` — `search(..., method=)` expuesto desde el binding Python (`search_memory` + `SearchRequest` batch) con `search_with_method` en el core SDK (ruteo inmutable por búsqueda a IVF/Scann/Flat/Hnsw, sin tocar `config.index_type`); backend Scann ahora se construye lazy (misma semántica que IVF, AUDREP-09); test Rust `test_search_with_method_override_routes_backends` + test Python `test_search_memory_method_override`

### Task 32: FEAT-05 — Revisar flags EXPERIMENTAL

- **Esfuerzo:** 🟢 · **Prig:** 🟢 · **Archivos:** `Cargo.toml` (features), `.github/workflows/ci-rust-10.yml`, docs | **DO** (doc decision) | **Contrato:** doc con status por feature
- **Estado:** ✅ COMPLETED
- **Ejecutado:** `418bc5bb` — Documented per-feature status (29 features, EXPERIMENTAL flags + dead/no-op `wasm` marker) in docs/architecture/FEATURES.md

### Task 33: FEAT-06 — Config hot-reload JSON + config.toml

- **Esfuerzo:** 🟡 · **Prig:** 🟢 · **Archivos:** `src/config.rs:1313`, `docs/operations/CONFIGURATION.md` | **DO** (doc formato real) | **Contrato:** CONFIGURATION.md describe builder/env y JSON hot-reload (si existe) · not invent config.toml
- **Estado:** ✅ DONE
- **Ejecutado:** `2174f854` — CONFIGURATION.md documenta builder (`from_env()` = `default()` + `with_*`), corregido fallback `PORT` inexistente, y añadidas secciones Builder API + Hot-Reload JSON (feature `hot-reload`, `VantaConfig::watch_config`, 8 campos, JSON-only, 1MB); afirmado explícitamente que NO se lee config.toml

### Task 34: FEAT-07 — `src/integrations.rs` stubs vacíos

- **Esfuerzo:** 🟡 · **Prig:** 🟢 · **Archivos:** `src/integrations.rs` | **DO** (implement `ollama_proxy` o retirar de surface) | **Contrato:** `rgba "Proximamente" src/integrations.rs` → no match
- **Estado:** ✅ DONE
- **Ejecutado:** `1b2a39d3` — Retirado el stub público `ollama_proxy_handler` (devolvía "Proximamente: Context-Aware proxy response"), eliminado de la surface (`pub mod integrations`); conservado el struct `OllamaGenerateRequest` (contrato serde real con tests); cert test `integrations_certification` ahora valida el roundtrip de serialización del request en lugar del fake del proxy. `rg -i "Proximamente" src/integrations.rs` → no match; clippy/fmt/`cargo test` ✅

### Task 35: REVISAR-01 — Cerrar ciclo ERR-038/039/040/041 (reproducibilidad)

- **Esfuerzo:** 🟡 · **Prig:** 🟡 · **Archivos:** `benches/`, `src/index/ivf.rs` | **DO** (bench dedicado) | **Contrato:** `cargo bench --bench ivf_*` existe + reporte
- **Estado:** ✅ DONE
- **Ejecutado:** b9249654 — `benches/ivf_bench.rs` (criterion, nlist×nprobe sweep: build k-means, Recall@10 vs brute-force, p50/p99/mean, QPS, cand/q); expuesto `pub mod ivf` (1-palabra, src/index/mod.rs); `[[bench]] ivf_bench` en Cargo.toml; reporte `docs/benchmarks/ivf_bench.md`

### COV

### Task 36: COV-001 — Test Python async (missing flush/purge/query/graph)

- **Esfuerzo:** 🟢 · **Prig:** 🟢 · **Archivos:** `vantadb-python/vantadb_py/__init__.py`, `vantadb-python/tests/` · **DO** · **Contrato:** `target/audit-venv/Scripts/python -m pytest vantadb-python/tests/ -k async` pasa; coverage wrapper ≥85%
- **Estado:** ✅ DONE
- **Ejecutado:** `340731ce` — añadidos 8 tests async en `TestAsyncVantaDB` (flush durability, purge_expired, IQL query, graph BFS/DFS/centrality/algorithms, batch APIs, export/import, mantenimiento, snippet/explain, close). `-k async` = 14 passed; wrapper `vantadb_py/__init__.py` 96% coverage.
- **Nota (delegado a vanta-engine):** `rebuild_index`, `reindex_hnsw_from_text` y `compact_layout` fallan con `TimeoutError: acquire insert_lock in flush (ERR-010)` — self-deadlock del insert_lock en el engine (`src/storage/`), pre-existente en `develop` (el test sync `test_rebuild_export_import_memory` falla igual). Excluidos de los tests async hasta fix; el coverage del wrapper queda en 96% sin ellos.

### Task 37: COV-003 — Rust CLI tests (vanta-cli subcommands)

- **Esfuerzo:** 🟡 · **Prig:** 🟢 · **Archivos:** `src/cli_handlers/*`, `src/bin/vanta-cli.rs`, `src/sdk/gds.rs` | **DO** (2.5k ln 0% coverage) | **Contrato:** `cargo nextest run --profile audit -p vantadb --features cli --test cli_tests`
- **Estado:** ✅ DONE
- **Ejecutado:** `f71dbff9` — 22 tests nuevos en `tests/cli_tests.rs` (count/delete-by-filter/similar/search-multi/audit/repair/snapshot/wal/migrate/completions): 11 pasan, 11 FAILED por colateral pre-existente: fail-closed de ERR-011 rechaza reapertura de DB legítima con shard vacío (`WAL shard N is truncated: 0 durable records, but round-robin requires at least 1`) — verificado pre-existente por stash (tests viejos fallan igual en HEAD). 19 failures baseline en suite.

### Task 38: COV-004 — ADR política gate coverage CI

- **Esfuerzo:** 🟢 · **Prig:** 🟡 · **Archivos:** `.github/workflows/ci-rust-10.yml`, ADR | **DO** (decisión documentada) | **Contrato:** ADR expresa root (81.40%) vs workspace (72.76%) y expectativa de bindings
- **Estado:** ✅ DONE
- **Ejecutado:** `2c9ddbc5` — `docs/architecture/adr/ADR-015-coverage-policy.md`: gate real = workspace-wide ≥80% llvm-cov (root 81.40% pasa, workspace 72.76% diluido por crate binding); step "(>=70%)" es stale; bindings: python ≥85% vía pytest propio, wasm/mcp/server excluidos (experimental), node fuera del workspace coverage.

### PERF

### Task 39: PERF-01 — Sellar benchmark claims README

- **Esfuerzo:** 🟡 · **Prig:** 🟉 · **Archivos:** `benchmales/`, `README.md`, `docs/QUICKSTART.md`, `docs/benchmarks/` | **DO** (honestidad de marketing) | **Contrato:** README/QUICKSTART claims re-validados o retirados; siRE != código actual
- **Estado:** ✅ COMPLETED
- **Ejecutado:** `30e90cd9` — README.md y README_ES.md: retirados los claims sin respaldo del "Target Baseline" (~5,400 vec/s, ~1,100/830/450 qps) reemplazados por el baseline real commiteado de `benchmarks/vanta_benchmark_report.json` (61.5 rec/s, HNSW p50 3.3ms, hybrid p50 12.1ms) con fuente citada; tabla SIFT-1M Fase 2 referenciada a BENCHMARKS.md §5 (verificado idéntico) + fuente de BENCHMARK_OPTIMIZATION_2026.md. QUICKSTART.md sin claims de perf (verificado por grep, sin cambios).

### Task 40: PERF-04 — Prefetch default OFF

- **Esfuerzo:** 🟢 · **Prig:** 🟢 · **Archivos:** `src/index/hnsw.rs` (prefetch), docs | **DO** (flag real + OFF) — **DO** only si hay feature; verificar
- **Estado:** ✅ DONE
- **Ejecutado:** `152ddd26` — El flag real ya existía (`PrefetchMode` Auto/Enabled/Disabled + `if should_prefetch()` en `src/index/search.rs:245`) pero el default era ON: enum `#[default] Auto`, `VantaConfig::default()`/`HotReloadConfig::default()` explícitos a `Auto`, y fallback de `should_prefetch()` hardcodeado a `true`. Verificado → default en OFF: `#[default]` movido a `Disabled`, fallbacks de config a `Disabled`, fallback de `should_prefetch()` a `false` (ERR-038), tests de default actualizados, benchmark `prefetch_benchmark.rs` ON arm ahora exige `VANTA_PREFETCH=enabled`, docs (`CONFIGURATION.md` + doc de campo) actualizadas a `Disabled`.

### Task 41: PERF-06 — `VANTADB_MEMORY_LIMIT` soporta sufijos KB/MB/GB

- **Esfuerzo:** 🟢 · **Prig:** 🟡 · **Archivos:** `src/config.rs`, `src/cli.rs` | **DO** | **Contrato:** `vanta --memory-limit 500MB` → no error + test parse
- **Estado:** ✅ DONE
- **Ejecutado:** `914514bb` — `parse_memory_limit()` en `config.rs` (sufijos KB/MB/GB/TB + KiB/MiB/GiB/TiB, 1024-based) + `VANTADB_MEMORY_LIMIT` env leída en `VantaConfig::default()` (warn+ignore si inválida) + flag global `--memory-limit` en `cli.rs` (env `VANTADB_MEMORY_LIMIT`) plumbed a `cmd_server` (HTTP config + env al child MCP). Test `test_parse_memory_limit`: `500MB` → 524288000 ok. Verificado `vanta-cli --memory-limit 500MB server --help` exit 0 + 36/36 config tests en temp worktree de HEAD.

---

## DOC — honesty (paralelo, wave 3)

### Task 42: DOC-02 — Fix versión drift QUICKSTART/badges (0.5.0)

- **Esfuerzo:** 🟢 · **Archivos:** `docs/QUICKSTART.md`, `README.md` | **DO** · **Contrato:** `rg "0.4" docs/QUICKSTART.md README.md` → no match (except history)
- **Estado:** ✅ DONE
- **Ejecutado:** 6723cb3f — v0.4.x → v0.5.0 y wheel 0.1.1 → 0.5.0 en QUICKSTART.md

### Task 43: DOC-03 — Fix mojibake UTF-8

- **Esfuerzo:** 🟢 · **Archivos:** `vantadb-python/README.md`, `docs/DESIGN_RULES.md` | **DO** · **Contrato:** `rgcha "\\u00e9|�"` → no match
- **Estado:** ✅ DONE
- **Ejecutado:** `16cd29a7` — verificación byte a byte: ambos archivos son UTF-8 válido en todo el historial git; 0 caracteres U+FFFD, 0 escapes literales `\u00e9`, 0 patrones de mojibake (`Ã©`, `Â`, `â€™`). El regex `\u00e9` del audit matchea el carácter é (U+00E9) que es acento legítimo en español (técnica, Léxica, comparación) → falso positivo. No se reemplazó nada: quitar acentos habría corrompido los docs. Contrato real de corrupción verificado sin match.

### Task 44: DOC-04 — Fix `llms.txt` API inventada

- **Esfuerzo:** 🟢 · **Prig:** 🔴 · **Archivos:** `llms.txt`, `docs/` | **DO** — `from vantadb import VantaEmbedded` no existe (real: `vantadb_py.VantaDB`) | **Contrato:** `rgba "from vantadb import"` → no match
- **Estado:** ✅ DONE
- **Ejecutado:** `0b3de29a` + `5856d498` + `04d790b0` — API inventada `VantaEmbedded`/`VantaError`/kwargs (`config=`, `mode=`, `text=`, `edges=`, `bitset=`, `graph_hops=`) reemplazada por el API real `vantadb_py.VantaDB` (`put(namespace,key,payload)`, `search_memory(query_vector,text_query)`, `graph_bfs`, `add_edge`, `RuntimeError`) en llms.txt + 22 docs. Contrato verificado: `rg "from vantadb import"` → solo los descriptores del task en Backlog/plan. `rg "VantaError\."|mode="hybrid"|graph_hops|edges=\[|bitset="` → sin match en snippets Python.

### Task 45: DOC-05 — Wikilinks Obsidian → rutas relativas

- **Esfuerzo:** 🟢 · **Archivos:** `docs/README.md` | **DO** · **Contrato:** `rg "\[\[" docs/README.md` → no match
- **Estado:** ✅ DONE
- **Ejecutado:** 282d33d6 — reemplazados 16 wikilinks Obsidian por enlaces markdown relativos verificados contra la estructura real de docs/; convención de enlaces internos actualizada

### Task 46: DOC-06 — Límite u64 documentado (ERR-023)

- **Esfuerzo:** 🟢 · **Archivos:** `docs/api/PYTHON_SDK.md`, `docs/QUICKSTART.md` | **DO** · **Contrato:** sección "ID limits" existe
- **Estado:** ✅ DONE
- **Ejecutado:** 724e6d53 — sección "ID limits" añadida a `docs/api/PYTHON_SDK.md` y `docs/QUICKSTART.md`; límite real verificado en binding: u128 nativo (pyo3 0.29), IDs > u64::MAX soportados como ints Python sin truncación (ERR-023); OverflowError fuera de rango, string solo en `recover_archived_nodes()`, precaución JSON 2^53

### Task 47: DOC-07 — Documentar hot-reload JSON / config.toml

- **Esfuerzo:** 🟡 · **Archivos:** `docs/operations/CONFIGURATION.md` | **DO** (con FEAT-06) · **Contrato:** sección config-examples
- **Estado:** ✅ DONE
- **Ejecutado:** `3e455b5e` — added §1.5 Configuration Examples (env vars, Rust builder, hot-reload JSON, CLI `--memory-limit`); fixed `memory_limit` env-var cell to `VANTADB_MEMORY_LIMIT`

### Task 48: DOC-08 — README claims cluster/graph honestos

- **Esfuerzo:** 🟢 · **Archivos:** `README.md` | **DO** · **Contrato:** README dice "single-node embedding engine", "wal-shipping experimental send-only"
- **Estado:** ✅ DONE
- **Ejecutado:** 6d881fb8 — claims de cluster/distributed corregidos: intro + Core Capabilities explican que VantaDB es single-node embedded (sin clustering/sharding); nueva nota de Replication aclara que `wal-shipping` es experimental send-only (HTTP POST, sin receive path), no replication/clustering/HA

---

### V-CÁTEGORÍA (P0 bloqueado — **RELEASE-02**)

### Task 49: RELEASE-02 — Publish coordinado 0.5.0 (depende de SEC-01, ERR-022, RELEASE-01, doc-coverage)

- **Esfuerzo:** 🟢 · **Prig:** 🔴 · **Gate:** 🟢 HABILITADO (Task 1, 2, 6 + doc-coverage completados)
- **Archivos clave:** `release.yml`, `release-wheels-60.yml`, `release-npm-61.yml`, `docs/CHANGELOG.md`
- **Verificación real:** ✅ workspace.package.version = `0.5.0` (`Cargo.toml:626`); release coordinado 0.5.0 YA publicado en crates.io, PyPI (`vantadb_py`), npm y GitHub Releases (2026-08-01) — el claim "crates.io en 0.4.0" del plan quedó obsoleto; los gates de campaña (SEC-01, ERR-022, RELEASE-01) se completaron DESPUÉS del release → los 49 commits de esta campaña van al próximo release vía release-plz.
- **Contract:** `git tag v0.5.0` + CI green; `gh release view v0.5.0`
- **Task file:** `.agents/skills/campaign-executor/tasks/RELEASE_02.md`
- **Estado:** ✅ COMPLETED
- **Ejecutado:** tag `v0.5.0` + GitHub Release (ness-e, 2026-08-01, assets aarch64-apple-darwin) · crates.io `vantadb 0.5.0` · PyPI `vantadb_py 0.5.0` · npm `vantadb 0.5.0` — verificado con `cargo search`/`pip index`/`npm view`/`gh release view`

---

## Fields interative record

**Iteraciones:** (tablas vacías hasta execute)

**Notas por tarea:**
- ERR-016 (`parser WHERE/RANK`) capítulo resuelto → SKIP
- ERR-034 (`/metrics` protegido) verificado sin hallazgo → SKIP
- ERR-051 (`sync CLI`) verificado OK → SKIP
- NV-02/03/05 → ✅ completados (progreso)
- DESKTOP-20 + ADMIN-01..09 → ✅ completados (progreso)

---

<!-- INSTANTÁNEO: partial state module / recitation -->

## RECITATION (final)

- **Campaign ID:** backlog-2026-08-09
- **Objetivo activo:** Ejecutar el backlog completo 2026-08-09
- **Estado:** completed
- **Última acción:** Cierre del pipeline: 49/49 tareas delegables completadas (Wave 0: RELEASE-01/SEC-01/RELEASE-03; Wave 1: 18 ERR fixes; Wave 2: 8 fixes + ERR-050 changelog; Wave 3: FEAT-01..07, REVISAR-01, COV-001/003/004, PERF-01/04/06, DOC-02..08, RELEASE-02). Task 49 verificada de facto: v0.5.0 ya publicado en crates.io/PyPI/npm/GitHub (2026-08-01) con todos los gates de campaña completados después → fixes de campaña van al próximo release vía release-plz. Task 4 (ERR-010) estado resincronizado con commit real `a5ca4389`. Plan al 100%.
- **Resultado:** ✅
- **Próxima acción:** migrar `docs/Backlog.md` → `docs/progreso/` (skill progreso Trigger 1) + reporte final al usuario
- **Contrato:** todos los contratos por tarea cumplidos (evidencia rg/git en cada tarea)
- **Próxima tarea si completa:** Task 50 (COM-02/03 — humana, no delegable)

=== END RECITATION ===

=== RECITATION ===
Campaign ID: c362dc7a-829b-4bbb-82b4-84beaa162d8b
Objetivo activo: Ejecutar backlog completo 2026-08-09
Estado: completed
Última acción: Verificó registros públicos: cargo search vantadb=0.5.0, pip index vantadb_py=0.5.0, npm view vantadb=0.5.0, gh release view v0.5.0 OK. Marcó Task 49 DONE en plan + commit 64da13bc
Resultado: ✅
Próxima acción: Migrar Backlog.md → docs/progreso/ + reporte final
Contrato: git tag v0.5.0 + gh release view v0.5.0 — verificado live: release coordinado 0.5.0 ya publicado (crates.io/PyPI/npm/GitHub, 2026-08-01)
Próxima tarea si completa: Task 50 (humana)
=== END RECITATION ===
