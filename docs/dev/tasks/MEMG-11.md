---
title: "TASK MEMG-11: Adopción del motor core en vanta-memory (recall híbrido + escritura batch)"
kind: task
---

# TASK MEMG-11: Adopción del motor core en `vanta-memory` (recall híbrido + escritura batch)

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` — Task 39 (bloque F0-expandido, L1119-1145)
- **Fuente:** Backlog `MEMG-11` (fila :146) + DELTA §P1 + análisis vanta-memory↔core 2026-09-30
- **Esfuerzo:** 🔴 1-2sem
- **Prioridad:** 🔴
- **Tipo:** Rust (crate `vanta-memory` + fix acotado en el core SDK `src/sdk/api/memory.rs` — medido y justificado)
- **Turns estimados:** 30-60 (multi-sesión)
- **Creado:** 2026-10-05
- **last-synced:** 2026-10-05
- **Estado:** ⏳ IN PROGRESS (reservada como taskId `39` en el campaign server)
- **Incógnitas (uphill):** 0 abiertas (paridad dual-pool resuelta en DISCOVERY con test; mecanismo de escritura resuelto por medición: fix core + migración incondicional)
- **Pendientes (downhill):** 3 steps (3-5): cierre del A/B (canonical_p99 run limpio), verify full scoped + OCR, review P2-01 + commit

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vanta-memory/src/services/pipeline_worker.rs:523` (`apply_dedup_batch`), `:716` (`perform_auto_recall` default), `core/scene/auto_consolidate.rs` + `core/record/l1_dedup.rs:141` + `core/record/l1_batch.rs:150` (`recall_candidates` — NO migrado esta iteración), tests `recall.rs` / `semantic_recall.rs` / `ver04_governance.rs` / `l1_dedup.rs` / `capture_approval.rs` / `l1_contradiction.rs` |
| Callees | Core SDK **consumido + un fix acotado** (medición → root cause): `Embedded::search` (`src/sdk/search/mod.rs:73`, BM25+HNSW+RRF vía `src/planner.rs` WIRE-08), `Embedded::put_batch` (`src/sdk/api/memory.rs:611` — `put_batch_inner` pasa de full-rebuild de índices a `replace_derived_indexes` por registro, paridad `put_one`), `Embedded::put` (`:591`), `MemorySearchRequest` (`src/sdk/serialization/vector_types.rs:102`) |
| Implicaciones | `RecallConfig` gana campo `core_search: bool` (default false — no rompe literales con `..Default::default()`, verificado: únicos literales exhaustivos son la propia definición). `apply_dedup_batch` cambia el mecanismo de persistencia (mismos datos/semántica). Core: fix interno de `put_batch` (índices incrementales; API pública/wire intactos — la medición lo exigió: pre-fix 0.07×). Semántica dual-pool legacy preservada por test de exclusión por vector nulo (pre-mortem #1). |

## Impacto mapeado (Regla 0)

> Gate previo a la primera edición — poblado en DISCOVERY (2026-10-05, HEAD `29beacff`).

- **Archivos leídos (completos):** `vanta-memory/src/core/record/l1_reader.rs` (322L), `vanta-memory/src/core/record/l1_writer.rs` (474L, HEAD `29beacff`), `vanta-memory/src/core/record/l1_batch.rs` (306L), `vanta-memory/src/core/hooks/auto_recall.rs` (906L), `src/sdk/search/fusion.rs` (844L), `src/sdk/serialization/vector_types.rs:1-330` (struct `MemorySearchRequest` + defaults), `src/sdk/search/mod.rs:1-240` (`search`/`search_page`/`search_impl`), `src/sdk/api/memory.rs:457-848` (`put_one`/`put_batch_inner`/`get`), `src/sdk/serialization/impl_rebuild.rs:1-115` (rebuild de índices), `src/sdk/serialization/impl_index.rs:154-200` (`replace_derived_indexes`), `src/storage/engine/tests/incremental.rs:180-370` (tests put_batch), `vanta-memory/src/services/pipeline_worker.rs:480-569` (`run_l1_batch`/`run_dream`), `vanta-memory/src/core/record/mod.rs`, `vanta-memory/src/core/hooks/mod.rs`, `vanta-memory/Cargo.toml`
- **Archivos referenciados hacia dentro (imports/deps):** `l1_writer` ← `l1_reader` (`read_record`, `l1_namespace`), `lifecycle` (`mark_contradiction`); `auto_recall` ← `l1_reader` (dual-pool helpers), `l1_writer` (`EmbedFn`), `persona`/`profile`/`scene`; core SDK vía `vantadb::sdk::{Embedded, MemoryInput, MemorySearchRequest, SearchProfileConfig}`
- **Archivos que referencian a los editados (referencias entrantes):** grep completo en DISCOVERY — `apply_dedup_batch`: `pipeline_worker.rs:523` + 3 tests; `write_memory`: `pipeline_manager.rs`, `generation_log.rs`, `l1_contradiction.rs`, `l1_dedup.rs`, `recall.rs` (tests); `perform_auto_recall*`: `pipeline_worker.rs:716` + tests; `RecallConfig`: 8 literales con `..Default::default()` (nuevo campo no rompe)
- **Veredicto impacto:** **medio** — crate `vanta-memory` (ya publicable desde DIST-01) + write path del pipeline + **fix acotado en `src/sdk/api/memory.rs`** (índices de `put_batch`: full-rebuild → ops incrementales; API pública intacta, semántica de datos idéntica — validada por tests core de consistencia list/count/text y suites de bindings que lo consumen). Riesgo principal: semántica dual-pool legacy (mitigado con test ANTES de migrar) y costo de `put_batch` (medido: pre-fix regresaba 0.07×; post-fix 2.6× — ver §A/B).

## Contrato

"`cargo nextest run --profile audit -p vanta-memory --build-jobs 2` verde + `cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings` 0 + `cargo fmt -p vanta-memory -- --check` 0 + core scoped `cargo nextest run -p vantadb --lib -E 'test(put_batch) or test(incremental)'` verde, y: (a) recall L1 puede rankear vía búsqueda híbrida del core (BM25+HNSW+RRF) detrás de flag (`RecallConfig.core_search`, default false = legacy byte-identical) preservando la semántica dual-pool (record sin vector nunca se dropea — test de exclusión por vector nulo); (b) las escrituras del pipeline L1 (`apply_dedup_batch`) usan `put_batch` (group-commit) con el fix core de índices incrementales (sin el fix, `put_batch` regresaba 0.07× — root cause medido); (c) A/B documentado: `canonical_p99` (timed run local vs MGR-19 §1, comando/hardware/seed fijados) + medición de recall (paridad en fixture + latencia legacy vs core ~11×) + medición de escritura (~2.6× post-fix); divergencias del flag path justificadas y documentadas."

## Spec (SDD — obligatoria: feature-add con símbolos públicos)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Mecanismo del dual-path | A) campo `RecallConfig.core_search: bool` default false (plan: "rollback = dual-path por config", L1128) / B) env var / C) feature flag de compilación | A | ✅ decidido-por-evidencia (plan Task 39 L1128; literales existentes usan `..Default::default()` → aditivo) |
| 2 | Alcance del recall híbrido | A) path `perform_auto_recall` (consumidor L1→contexto) ahora; `recall_candidates` (dedup) → FIND / B) ambos en esta iteración | A | ✅ decidido-por-evidencia (stop-condition del plan: entregar (a)+(b) y FIND del resto; dedup cambia el pool del juicio LLM = blast radius mayor) |
| 3 | Escritura pipeline | A) `put_batch` incondicional / B) umbral por tamaño / C) no migrar | **regla pre-registrada:** medir primero; migrar con el mecanismo que NO regrese en el workload real | ✅ **decidido-por-medición:** medición 1 (pre-fix) dio 0.07×–0.64× (regresión: `put_batch` reconstruía derived+text+sparse de TODA la DB por batch, `memory.rs:793-801`) → **root-cause fix en core**: ops incrementales por registro (`replace_derived_indexes`, paridad con `put_one`) → medición 2: **2.36×–2.71× estable e independiente del tamaño del store** → migración **incondicional** (A) en `apply_dedup_batch` |
| 4 | `min_overlap` en flag path | A) post-filtrar overlap / B) sin post-filtro (BM25 decide) | B | ✅ decidido-por-evidencia: post-filtrar con `significant_terms` rompería matches por stemming/stopwords del tokenizer core (BM25 "prefers"→"prefer" con overlap 0) — documentado |
| 5 | Paridad de resultados | A) idéntica / B) divergencia justificada y documentada (contrato L1128 permite B) | B | ✅ decidido-por-evidencia: BM25 tokenizer ≠ significant_terms; fixture mide solapamiento y se documenta |
| 6 | Rollback | flag off = path legacy intacto (byte-identical) | — | ✅ test de regresión (`flag_off_is_legacy`) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  - Record sin vector **nunca se dropea** del recall (dual-pool legacy, D38) — verificado por test ANTES de migrar (pre-mortem #1) y por el test del path core.
  - `core_search=false` → comportamiento legacy byte-identical (todos los tests existentes de `recall.rs`/`semantic_recall.rs`/`ver04_governance.rs` sin tocar).
  - Gate de cuarentena intacto: `include_quarantined: false` en ambos paths (SCH-05, ADR-046 §D5).
  - El registro viejo marcado por contradicción nunca se borra (MEMG-01); `mark_contradiction` única semántica.
  - `put_batch` conserva la consistencia list/count/text (tests core `test_put_batch_list_count_text_consistent` + `test_incremental_put_batch_{small,large}`) y su API pública intacta — el fix de índices es interno (full-rebuild → incremental).
  - Semántica v2 write-side de MEMG-12 intacta: `put_record`/`record_input` escriben `valid_at_ms`/`confidence` (test `l1_semantics_v2` 4/4).
  - Sin `unwrap`/`expect`/`unsafe` en código nuevo de producción (Regla 2).
- **Comandos de verificación:** `cargo nextest run --profile audit -p vanta-memory --build-jobs 2` (suite completa) · `cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings` · `cargo fmt -p vanta-memory -- --check` · core scoped: `cargo nextest run -p vantadb --lib -E 'test(put_batch) or test(incremental)'` · `cargo bench -p vantadb --bench canonical_p99` (A/B)
- **Deuda pendiente:** migración de `recall_candidates` (dedup) al motor core; `query_sparse`/filtros/cursors del core donde apliquen (plan L1128 (c)); BENCH-02/VER-08 (calidad BEIR/LongMemEval) no disponibles (F5) → la métrica de recall es fixture propio + paridad, no claim calibrado.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** ≤0 — sin umbral heurístico (migración incondicional tras el fix core); el fix de `put_batch` **reduce** deuda de performance existente (el `ponytail:` del core pedía exactamente esta migración); sin `unsafe`/clones nuevos en producción (un `record.clone()` por input en `put_batch` para el `seen` in-batch, acotado al batch y compensado por la eliminación de 3 rebuilds full-DB).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable + tests nuevos (dual-pool, batch write, flag path) + suite `-p vanta-memory` verde |
| **Commit** | Commit atómico convencional `feat(memory):`/`perf(memory):`, verify mecánico scoped registrado |
| **Release** | Changelog (minor, vía release-plz) — el release no se ejecuta acá; semver `feat:` correcto |

## Herramientas necesarias
- cargo/nextest/clippy/fmt (terminal), codegraph (blast radius), campaign MCP (estado), OCR delegation

**Skills cargadas (SDP):** campaign-executor · progreso · ponytail · **performance-optimization (pinned policy)** · source-driven-development · doubt-driven-development · incremental-implementation · test-driven-development — `campaign_discover_skills_v2` phase=BUILD (2026-10-05); + base del rol: systematic-debugging si verify falla.

## Investigation Notes

- **`put_batch` = full rebuild de índices por batch (hallazgo crítico DISCOVERY):** `put_batch_inner` (`src/sdk/api/memory.rs:793-801`) reconstruye derived+text+sparse **sobre TODA la DB** tras cada batch (O(total nodes); el propio comentario `ponytail:` lo declara). Para flushes pequeños del pipeline sobre un store mediano, el rebuild puede dominar el costo → la regla pre-registrada (Spec #3) decide por medición, no por intuición (Regla 9).
- **`put` = ops incrementales por registro** (`put_one` → `replace_derived_indexes` incremental + version history 1×1); `put_batch` = 1 WAL batch + 1 KV write_batch por chunk + rebuild full. El A/B mide ambos.
- **Baseline A/B (MGR-19 §1, fijado):** `canonical_p99` insert 100k×1536d seed 42; search p50=2.2388 / p95=4.248 / p99=4.9987 ms; comando `cargo bench -p vantadb --bench canonical_p99`. Pre-mortem #2: comparar contra ese número (baseline móvil) con hardware documentado.
- **Dual-pool legacy (D38, `l1_reader.rs:119-188` + `auto_recall.rs:569-674`):** pool keyword = overlap>0 (TODOS los records, con o sin vector); pool vector = cosine ≥ 0.35 (solo con hook+vector); fusión RRF k=60 local. El core: BM25 (text arm, todos los records) + HNSW (vector arm) + RRF planner. La semántica "sin vector nunca se dropea" se preserva vía el text arm del core — el test RED la fija ANTES de migrar.
- **`query_sparse` (WIRE-03) NO aplica a L1 hoy:** los records L1 se escriben con `sparse_vector: None` (`put_record`, `l1_writer.rs:418`) → no hay sparse que consultar; queda para MEMG-13 (superficies restantes).
- **VER-08/BENCH-02 (calidad BEIR/LongMemEval) = F5 (Task 67), no hard blocker** (plan L1129): la pata de calidad se registra como FIND si falta.
- **Fix core requerido (medición → root cause):** la medición 1 de escritura mostró que `put_batch` **regresaba** (0.07× a store=2000/batch=20) porque `put_batch_inner` hacía full-rebuild de derived+text+sparse sobre TODA la DB por batch (el propio comentario `ponytail:` del core declaraba el techo). Fix aplicado en `src/sdk/api/memory.rs`: `replace_derived_indexes` por registro (mismo path que `put_one`; `previous` capturado del resolve y del `seen` in-batch), eliminando los 3 rebuilds finales. Medición 2: 2.36×–2.71× estable. Los tests core `test_put_batch_list_count_text_consistent` + `test_incremental_put_batch_{small,large}` validan la consistencia list/count/text.
- **Trade-off aceptado (paridad `put_one`):** el state marker de índices (`derived/text/sparse`) solo lo escriben los rebuilds; `put` nunca lo refresca y `put_batch` ahora tampoco. El reconcile de `open_with_config` (`ensure_indexes_current`) reconstruye si los counts no coinciden — mismo comportamiento pre-existente de `put`; la corrección queda garantizada por ese safety net.

## A/B (MEMG-11 — evidencia de medición, 2026-10-05)

> Regla 9/11: todo número cita comando + fixture. Hardware local: Windows (win32), `rustc 1.95.0`, build `test`/`bench` sin optimizar el primero y `release` el segundo; los números son **locales de esta máquina** — comparables entre sí (misma sesión/máquina), no claims absolutos.

### (b) Escritura — `put` secuencial vs `put_batch` (misma DB in-memory, mismo payload)

| Store | Batch | Sequential | put_batch (pre-fix) | put_batch (post-fix) | Ratio post-fix |
|-------|-------|-----------|---------------------|----------------------|----------------|
| 0 | 20 | 37.96 ms | 12.78 ms (2.97×) | 17.24 ms | 2.36× |
| 200 | 20 | 41.98 ms | 65.99 ms (0.64×) | 16.16 ms | 2.63× |
| 2000 | 20 | 67.93 ms | 934.10 ms (0.07×) | 25.97 ms | 2.57× |
| 2000 | 200 | 691.19 ms | 1133.30 ms (0.61×) | 252.43 ms | 2.71× |

Comando: `cargo nextest run -p vanta-memory --test l1_batch_write --run-ignored ignored-only --nocapture` (instrumento `measurement_sequential_puts_vs_put_batch`). Conclusión: con el fix core, `put_batch` gana ~2.4-2.7× **independiente del tamaño del store** (O(batch) confirmado) → migración incondicional. El claim "≥5×" del Backlog era del pipeline async (WIRE-06/FIND-182, otro workload); acá el número medido es ~2.6×.

### (a) Recall — dual-pool legacy vs core híbrido (pool 2000 records, 20 queries, in-memory)

| Path | avg/query |
|------|-----------|
| legacy (scan + `significant_terms` + cosine + `rrf_merge`) | 457.25 ms |
| core (`Embedded::search` BM25+HNSW+RRF, `core_search=true`) | 41.53 ms |
| **ratio** | **~11×** |

Comando: `cargo nextest run -p vanta-memory --test recall_core_hybrid --run-ignored ignored-only --nocapture` (instrumento `measurement_recall_legacy_vs_core`). Paridad funcional: fixture `recall_core_hybrid.rs` (vectorless via BM25, semántico via HNSW, scope cross-session, max_results) + suites legacy intactas.

### canonical_p99 (Regla 9 — contra MGR-19 §1)

> **Entorno (Regla 11):** misma máquina/modelo que MGR-19 §1 (i5-1235U, 12 hilos lógicos, 31 GB, Windows) — `HEAD` de esta corrida: `9d0e371d` + diff MEMG-11 sin commitear; perfil `bench` (release+debuginfo). MGR-19 §1: "los absolutos son locales; el valor versionable es el comando + la metodología, no el número".

| Métrica | MGR-19 §1 (2026-09-25) | MEMG-11 run 1 (contaminado¹) | MEMG-11 run 2 (limpio, máquina idle) | Veredicto |
|---------|------------------------|------------------------------|--------------------------------------|-----------|
| search p50 | 2.2388 ms | 8.2542 ms | **3.201 ms** (1.43×) | sin regresión atribuible (varianza de corrida) |
| search p95 | 4.248 ms | 13.1453 ms | **4.5415 ms** (1.07×) | ✅ paridad |
| search p99 | 4.9987 ms | 15.8482 ms | **5.7773 ms** (1.16×) | ✅ sin regresión p99 |
| batch 1000q (criterion) | 2.8136 s | [3.79–4.97 s] | **[3.3489–3.3890 s]** (1.20×) | sin regresión atribuible |
| insert 100k×1536d | 644.23 s (mean, 10 samples) | no re-corrido (evita ~1.9h; el bench insert usa `CPIndex` puro — fuera del diff SDK; baseline MGR-19 §1 citado) | — | — |

¹ Run 1 corrió con compiles de test concurrentes + térmica acumulada (laptop U-series): números no comparables 1:1. Run 2 (máquina idle, mismo exe) converge a §1: p95 1.07×, p99 1.16×, p50 1.43× — dentro de la varianza de instrumento ya documentada (FIND-233: 1.7-2.4× mismo-código cross-run/cross-VM). **El path medido (`CPIndex` raw, `benches/canonical_p99.rs`) no comparte código con el diff** (SDK `put_batch` + `vanta-memory`): el binario del bench no ejecuta ninguna ruta modificada → regresión por este diff imposible por construcción; el residual vs §1 es condición de corrida (fecha/térmica), no código.

Comando: `cargo bench -p vantadb --bench canonical_p99 -- search_1000_queries_1536d` (run 1) y re-run directo del exe con el mismo filtro (run 2, máquina idle). Seed 42 fijo (`generate_vectors(..., seed 42)`).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — dual-pool resuelta (test + semántica BM25); mecanismo de escritura resuelto por medición (fix core + migración incondicional) |
| Pendientes de ejecución (downhill) | 3 — Steps 3 (cierre A/B), 4 (verify+OCR), 5 (review+commit) |
| % completado | 50% (Steps 0-2 ✅) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — no toca trust boundaries nuevos; reutiliza gate de cuarentena (SCH-05) y ACL de inyección (VER-04) existentes. Sin deps nuevas.
- [x] **PERFORMANCE** — hot path (recall + write del pipeline). Skill `performance-optimization` cargada (pinned). Baseline: `canonical_p99` MGR-19 §1 + medición local de ambos paths (Step 3). Regla 9/11 aplicadas: sin claim sin número reproducible.

## Steps

### Step 0: RED — test de exclusión por vector nulo (pre-mortem #1) + API del flag
- **Archivos:** `vanta-memory/tests/recall_core_hybrid.rs` (nuevo)
- **Acción:** tests: (1) `legacy_recall_keeps_vectorless_record` (baseline legacy, pasa hoy — guard de regresión); (2) `core_hybrid_recall_keeps_vectorless_record` (flag `core_search: true`; record sin vector con términos compartidos debe ser recallable vía BM25) — RED (no compila: campo inexistente); (3) `core_hybrid_recall_finds_semantic_match` (record con vector, sin términos compartidos → recallable por HNSW); (4) `flag_off_is_legacy` (paridad).
- **Verify:** `cargo nextest run -p vanta-memory --test recall_core_hybrid` → RED por razón correcta (E0560/E0609 campo inexistente) ✅ verificado 2026-10-05
- **Estado:** ✅ COMPLETED

### Step 1: GREEN (b) — escrituras del pipeline vía `put_batch` + medición
- **Archivos:** `vanta-memory/src/core/record/l1_writer.rs`, `src/sdk/api/memory.rs` (core: fix de índices de `put_batch`), `vanta-memory/tests/l1_batch_write.rs` (nuevo)
- **Acción:** builders de registro (`plan_write`/`record_input`), `apply_dedup_batch` persiste vía `put_batch` (incondicional, medición pre-registrada); **fix core**: `put_batch_inner` reemplaza el full-rebuild de derived+text+sparse por `replace_derived_indexes` por registro (paridad `put_one`; `previous` capturado del resolve y del `seen` in-batch). Test de paridad batch vs secuencial + medición (instrumento ignored).
- **Verify:** `l1_batch_write` 2/2 ✅ · `l1_dedup`+`l1_contradiction`+`capture_approval`+`generation_log` 35/35 ✅ · core `put_batch`/`incremental`/`quarantine` 14/14 ✅ · MEMG-12 `l1_semantics_v2` 4/4 ✅ (delegación `put_record`→`record_input` preserva v2)
- **Estado:** ✅ COMPLETED

### Step 2: GREEN (a) — recall híbrido core detrás de flag
- **Archivos:** `vanta-memory/src/core/hooks/auto_recall.rs`
- **Acción:** `RecallConfig.core_search: bool` (default false) + `search_records_core` (search por namespace vía core, merge por score, mapeo a `RecallHit`, governance/ACL/scope agent-team preservados, `include_quarantined: false`); `perform_auto_recall_governed` rutea según flag; `semantic_ran` del flag path documentado; divergencia `min_overlap` documentada.
- **Verify:** `recall_core_hybrid` 6/6 ✅ · `recall`+`semantic_recall`+`ver04_governance` 31/31 ✅ (legacy intacto)
- **Estado:** ✅ COMPLETED

### Step 3: A/B documentado (canonical_p99 + recall + escritura)
- **Archivos:** `docs/dev/tasks/MEMG-11.md` (tabla A/B), `docs/api/VANTA_MEMORY.md` (flag)
- **Acción:** mediciones volcadas en §A/B (escritura 2.36×–2.71×; recall ~11×; canonical_p99 run limpio p50=3.201/p95=4.5415/p99=5.7773 ms vs MGR-19 §1 — dentro de varianza de instrumento, path medido ajeno al diff). `docs/api/VANTA_MEMORY.md` documenta `core_search`.
- **Verify:** tabla A/B completa + check-links/check-docs/gen-index exit 0 ✅
- **Estado:** ✅ COMPLETED

### Step 4: Verify full scoped + OCR delegation
- **Archivos:** — (verificación)
- **Acción:** `cargo fmt -p vanta-memory -- --check` ✅ 0 · `cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings` ✅ 0 · `cargo nextest run --profile audit -p vanta-memory --build-jobs 2` ✅ **604/604** · core `put_batch` sweep 8/8 + incremental 14/14 ✅ · `pwsh dev-tools/ocr-review.ps1 -Format json` ✅ exit 0 (input del reviewer en `target/memg11-ocr.json`). Nota: `campaign_verify_cmd` (MCP) no spawnea cargo en este entorno (exit -1 sin output, 0.4s) → verificación mecánica vía shell (documentado).
- **Verify:** los comandos exit 0 ✅
- **Estado:** ✅ COMPLETED

### Step 5: Review P2-01 (agente distinto) + commit local
- **Archivos:** `docs/dev/tasks/MEMG-11.md` (§Review)
- **Acción:** fork `vanta-review` con el diff (paths `vanta-memory/**` = tier Fast según tabla HARD-02; 🔴 → fresh review igual); registrar veredicto; commit **LOCAL** `feat(memory):` (⛔ nunca push).
- **Verify:** veredicto registrado + `git status` limpio de WIP ajeno (opencode.jsonc/master plan NO stageados)
- **Estado:** ⬜ PENDING

## Dependencias
- MEMG-01 (✅ `10b35b41`), MEMG-02 (✅ `fcc17ca7`/`782e111d`) — archivos frescos desde HEAD antes de tocar.
- BENCH-02/VER-08 (F5, Task 67) — no bloqueante; pata de calidad → FIND.
- `src/sdk/**` — solo consumo + el fix acotado de índices de `put_batch` (medido; sin cambios de API pública/wire).

## Review (GATE — agente distinto, P2-01)

- **Revisor:** `vanta-review` (fresco, sesión `ses_ef4d5658cffesIYmJtQxXsCY0r`) — tier **adversarial** por `src/sdk/**` en el diff
- **Ronda 1 (2026-10-05): ❌ cambios requeridos** — 1 Critical + 1 High + 2 Required + 2 Low. Contrato mecánico re-verificado por el reviewer: 604/604, clippy/fmt 0, sweep put_batch 8/8; A/B reproducible (escritura 2.24–2.85×; recall 8.7×). Cifra corregida: `-E 'test(put_batch) or test(incremental)'` = 12/12 (el 14/14 citado incluía quarantine).
- **Ronda 2 (2026-10-05): ❌ cambios requeridos** — fixes 2-5 de r1 APPROVED; Lows como FIND aceptados; **1 Critical nuevo**: el fix de r1 (retener N read-guards) introducía un deadlock reproducible (`put_batch([live, expired])` → el resolve del expirado llama `purge_expired_record` → `purge_lock.write()` en el mismo thread con un read-guard retenido; parking_lot no reentrante; variante cross-thread por fairness). Repro empírica del reviewer (probe temporal eliminado).
- **Fixes ronda 2 → 3:**
  1. **Critical — deadlock resuelto con write-lock único + variantes locked:** `put_batch_inner` toma `purge_lock.write()` UNA vez para toda la sección crítica (resolve + insert + replace) y resuelve vía `resolve_existing_for_write_locked` (sin read-guard) que purga expirados inline con `purge_expired_record_locked` (cuerpo sin lock, caller-documented). Sin read-guards retenidos → sin reentrancia ni bloqueo por fairness; la race DUR-03 queda cerrada (ningún purge interleave durante el batch). `resolve_existing_for_write`/`purge_expired_record` quedan intactos para `put_one`/`purge_expired`.
  2. **Test permanente del deadlock:** `put_batch_mixed_live_and_expired_upserts_do_not_deadlock` (edge_cases.rs; batch [live, expired] con channel-timeout 30s — si regresa, falla en vez de colgar la suite). Sweep put_batch 10/10.
- **Fixes aplicados en ronda 1 (2-5, APPROVED en r2):**
  1. **High — D38 en `mode: Embedding`:** el text arm corre SIEMPRE en el path core → record sin vector nunca se dropea; test `core_hybrid_embedding_mode_keeps_vectorless_record`.
  2. **Required — test UPSERT del core:** `test_put_batch_upsert_replaces_indexes` (incremental.rs, escenario 9): stale text postings borradas, payload/lista/count correctos tras upsert.
  3. **Required — divergencias documentadas:** siempre-hybrid (D38), merge cross-namespace por score RRF (aproximado), `semantic_ran` por ejecución → VANTA_MEMORY.md §Recall + docstring `search_records_core`.
  4. **Low — comentarios stale eliminados** (`memory.rs`). **Lows governance.source/BM25-JSON → FIND (aceptados por el reviewer):** registrados en queda_pendiente.
- **Evidencia ronda 3:** fmt 0 · clippy `vantadb`+`vanta-memory` 0 · core lib 12/12 · sweep put_batch 10/10 · edge_cases (mixed+ttl+racing) 3/3 · vanta-memory suite **605/605** · recall trio 28/28 · OCR input `target/memg11-ocr.json` (exit 0)
- **Enfoque:** (1) ¿el fix de `put_batch` preserva la consistencia de índices en todos los casos (fresh, UPSERT, duplicados in-batch, chunks) sin ventanas vs purge?; (2) ¿el path core del recall preserva dual-pool/ACL/cuarentena/scope D22?; (3) ¿la migración a `put_batch` mantiene semántica (partial-write/deletes/generation-log/contradicciones)?
- **Cómo se probó:** core `put_batch`+incremental 12/12 (incl. UPSERT nuevo) + quarantine sweep 14/14 · vanta-memory suite **605/605** · recall fixture 7/7 + suites legacy 28/28 · clippy vantadb/vanta-memory 0 · fmt 0 · OCR input `target/memg11-ocr.json` (exit 0)
- **Checklist anti-hábitos tóxicos:** a cargo del reviewer (ronda 2)
- **Veredicto ronda 1:** ❌ cambios requeridos → **ronda 2:** ❌ (deadlock en el fix r1) → **ronda 3: ✅ APPROVE** — deadlock resuelto (probe r2 re-ejecutado 2/2 PASS; test permanente bounded 3/3), DUR-03 cerrada, lock order consistente (`purge_lock → engine`), evidencia re-verificada (605/605 · 12/12 · 10/10 · fmt/clippy 0). Apto para COMPLETED + commit local. Optional no bloqueante (angostar la sección crítica antes de shred/version-history/HNSW rebuild) → queda_pendiente.

## Notas
- **Gate D:** pre-respondido por el plan F0 (flag por config es la decisión sancionada L1128; stop conditions fijadas L1130); no se agrega símbolo fuera de lo planificado.
- **Coordinación MEMG-12 (RESUELTA, mismo worktree):** MEMG-12 commiteó `9d0e371d` e **integró mi refactor de `l1_writer.rs`** (plan_write/record_input/apply_dedup_batch→put_batch) junto a su estampado v2 (`valid_at_ms`/`confidence`) — co-autoría documentada en el body de su commit ("COORDINACIÓN MEMG-11"). Verificado post-commit: `git show 9d0e371d:...l1_writer.rs` contiene ambos (record_input con v2 + put_batch) y el working tree == HEAD (sin diff). Mi commit cubre solo mis paths restantes (memory.rs/auto_recall.rs/tests/docs) y **no toca `l1_writer.rs`** → no puede revertir v2. Su suite corrió 602/602 con mis cambios integrados.
- **`opencode.jsonc` + master plan + `docs/pipeline-state.json`:** NO tocados / NO stagear (WIP ajeno; PROHIBIDO).
- **Disco:** si el linker falla por espacio → `dev-tools/target-cleanup.ps1 -Clean -Yes`.
- **Stop condition (plan L1130):** el A/B cerró completo (escritura ~2.6× + recall ~11× + canonical_p99 run limpio) → no se disparó la entrega parcial; el resto (dedup `recall_candidates`, `query_sparse`, filtros/cursors) queda como deuda/FIND.
- **FIND propuestos al cierre (review r1 Lows aceptados + r3 optional):** (1) `governance.source` difiere en queries sin hits (core vs legacy); (2) BM25 indexa el payload JSON completo (nombres de campo afectan relevancia); (3) angostar la sección crítica de `put_batch` (soltar `purge_lock` antes de shred/version-history/rebuild HNSW). El orquestador/lead los materializa en Backlog.
- **`campaign_verify_cmd` no operativo en este entorno** (no spawnea cargo: exit -1 sin output) → verificación mecánica vía shell; registrado para el harness.
