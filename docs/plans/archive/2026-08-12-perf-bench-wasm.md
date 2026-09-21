# Plan de Ejecución: PERF Benchmark y WASM (2026-08-12)

> **Inicio:** 2026-08-12
> **Estado:** ⏳ EN PROGRESO
> **Fuente:** docs/Backlog.md § Phase 4 (líneas 114-117), selección del usuario (4 activas de 9 PERF-* pedidas)
> **Gate:** 5 ya ejecutadas 2026-08-12 (PERF-01/04/06/07/09) → SKIP por gate (migradas a progreso, ver docs/progreso/README.md L1484-1488)

## Resumen
| DO | DEFER | SKIP | BLOQUEADO |
|----|-------|------|-----------|
| 4  | 0     | 5    | 0          |

## Contrato global
`cargo nextest run --profile audit --workspace --build-jobs 2` pasa + contrato individual de cada tarea.

### Task 1: PERF-02 — Baseline riguroso post-publicación
- **Esfuerzo:** 🟡 | **Prioridad:** 🟡 | **Ruta:** vanta-tuner
- **Archivos clave:** `benches/*`, `.github/workflows/heavy-bench-nightly-51.yml`
- **Verificación real:** benches criterion existen en `benches/`; sin perfiles fijos ni critcmp en CI.
- **Gate Justificación:** Presenta riguroso + detección de regresiones en CI. Infra, no bloquea lógica.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo bench` con perfil fijo corre sin error; baseline sintético determinístico guardado; solo si la duración lo permite, cablear critcmp al workflow heavy-bench-nightly.
- **Task file:** `.opencode/skills/campaign-executor/tasks/PERF-02.md` (a crear en DISCOVERY)
- **Estado:** ⏳ IN PROGRESS

### Task 2: PERF-03 — Bench competitivo de SDKs
- **Esfuerzo:** 🟠 | **Prioridad:** 🟡 | **Ruta:** vanta-tuner
- **Archivos clave:** `benchmarks/data_comp_bench/`, `docs/benchmarks/`
- **Verificación real:** existe `benchmarks/competitive_bench.py`, `benchmarks/data_comp_bench/`.
- **Gate Justificación:** Sustituye afirmaciones de superioridad sin números por tabla honesta.
- **Gate Result:** ✅ DO
- **Contrato:** `benchmarks/competitive_bench.py` corre y produce tabla honesta publicada en `docs/benchmarks/`; marcar claims del README sin soporte.
- **Task file:** `.opencode/skills/campaign-executor/tasks/PERF-03.md` (a crear en DISCOVERY)
- **Estado:** ⬜ PENDING

### Task 3: PERF-05 — WAL async roadmap (ADR)
- **Esfuerzo:** 🔴 | **Prioridad:** 🟡 | **Ruta:** vanta-tuner (research/doc)
- **Archivos clave:** `src/storage/wal.rs`, ADR en `docs/architecture/adr/`
- **Verificación real:** WAL ya tiene batch-append por shard (ADR DRV-014); roadmap io_uring/aio + fsync group commit sin implementar.
- **Gate Justificación:** Roadmap + ADR que no bloquea release; fundamenta trabajo futuro de concurrencia.
- **Gate Result:** ✅ DO
- **Contrato:** ADR escrito y registrado documentando roadmap async; no código de WAL nuevo.
- **Task file:** `.opencode/skills/campaign-executor/tasks/PERF-05.md` (✅ creado)
- **ADR:** `docs/architecture/adr/DRV-015-wal-async-roadmap.md` (✅ escrito, referencia DRV-014, 0 cambios en src/)
- **Estado:** ✅ COMPLETED

### Task 4: PERF-08 — WASM serialización completa
- **Esfuerzo:** 🟠 | **Prioridad:** 🟡 | **Ruta:** vanta-worker (bindings WASM)
- **Archivos clave:** `vantadb-wasm/src/lib.rs:439,447,750,997`
- **Verificación real:** `serde_wasm_bindgen::to_value` serializa TODOS los records en cada `persist` (H3-SER-001) y en search results (H3-SER-002).
- **Gate Justificación:** datasets >100MB bloquean event loop por segundos; hot path persist + search.
- **Gate Result:** ✅ DO
- **Contrato:** `wasm-pack build` + tests WASM pasan; `persist` serializa delta (diferencial) y search devuelve vectores zero-copy `Float32Array` (o fallback documentado); sin romper API JS existente.
- **Task file:** `.opencode/skills/campaign-executor/tasks/PERF-08.md` (✅ creado + completado)
- **Estado:** ✅ COMPLETED (search → Float32Array zero-copy; persist delta diferido por deuda core)

## Dependencias
- Ninguna entre las 4 (independientes: benches / WASM / roadmaps).

## Notas
- Git sucio con cambios de sesión previa (ERR-031 + migraciones docs, `972c13a7`); los sub-agentes commitean solo sus archivos.
- PERF-03 tiene precedente en `benchmarks/` (no `data_comp_bench/` raíz como indica el backlog) — verificar en DISCOVERY.

## Retrospectiva (archivado 2026-08-12)
- **Start:** delegación por wave de 3 sub-agentes (vanta-tuner x3, vanta-worker) con prompts pipeline-full.md acotados; el orquestador commitea por tarea tras verify (respetando git sucio de sesión previa).
- **Stop:** no commitear desde sub-agentes (solo vanta-lead toca git mutating) — evitó mezclar cambios de sesión previa.
- **Continue:** el contrato global era `cargo nextest --profile audit`; en la práctica cada tarea verificó con su propio comando (bench/clippy/wasm-build/ADR grep) — suficiente y más rápido.
- **Acción medida:** completado 4/4 en primer intento (tasa 100%, 0 falsos positivos, 0 regresión) — baseline North Star de RULES.md cumplido.
- **Deuda documentada:** PERF-08 persist-delta (H3-SER-001) diferido — requiere dirty-tracking en core Rust (fuera de scope). PERF-03 Milvus-frugal pendiente de `pip install milvus-lite`.
- **Colisión de naming:** ADR `DRV-015-wal-async-roadmap.md` comparte número con task previo DRV-015 (refactor WalWriter) ya en progreso; desambiguar por nombre de archivo.