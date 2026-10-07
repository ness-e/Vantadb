---
title: Avance — Vanta Memory
kind: review
status: active
tags: [vantadb, avance, vanta-memory, tdam, memory, persona, recall]
---

# Avance — Vanta Memory

> Registro consolidado del trabajo completado sobre el crate `vanta-memory/`: pipeline TDAM (L0 capture → L1 extract/dedup → L2 escenas → L3 persona → recall → offload → gateway), skills/tools MCP, seeds CLI, embeddings y schedulers. **IDs originales conservados.** Catch-up por campaña (no commit-por-commit).

## Cobertura rápida

- **P27 (F1-F4):** port TDAM completo — search profile IQL, entidades/RBAC, auth 3 capas, skills multi-versión, crate `vanta-memory` end-to-end con trait host-neutral `LlmRunner`.
- **P29 (F5):** superficie de memoria para el context engine — seeds/import CLI, generation-log, recall_scope híbrido, auto-sync scheduler, GC offload, ADR-0029.
- **P31 (Cierre Final):** wiring productivo al pipeline, e2e cross-crate MCP, embeddings semánticos opt-in, recall dual-pool RRF, compresión con scores reales.

---

## Campaña P27 — Vanta Memory Engine (F1-F4)

### MEM-01..21 (+34, 35): Port TDAM completo — catch-up por campaña
- **Fecha:** 2026-08-18 → 2026-08-20
- **Objetivo:** Crate nuevo `vanta-memory/` con el pipeline completo: L0 capture idempotente LLM-free (`9c0dd213`) → L1 extractor + dedup 2 fases (`91c9068e`, `7356aa7d`) → L2 escenas (contrato META + nodo escena, `a6526f70`, `7356aa7d`… `c6c06c75` tools sandboxed) → L3 persona first/incremental con triggers (`5fc0cb11`) → recall 3 modos prepend/append (`fb1d2dd4`) → offload cursor persistente por sesión (`9a9fea41`) → gateway con tools MCP scene_read/list/query (`31e676b1`). Orquestación timers+locks (`2634e9bd`), skill extract transcript (`31e24f88`), sanitize/truncación code-point (`42940f6d`), métricas snapshot por capa + audit (`84f28a18`), search profile IQL `PROFILE` (`6a50b8ee`, `32b09daf`), entidades entity_* + RBAC allow-only (`23719e23`, `9717bf03`), auth 3 capas L1/L2/L3 (`01a5de66`), skills multi-versión con optimistic lock (`92cf709f`, `4763bf44`), data plane REST `/conversation/add` + `/skill/listing` (`9693d0ff`), contracts F4 + `LlmRunner` host-neutral con degradación LLM-free (`76a73969`).
- **Resultado:** ✅ 24/24 tareas. Suite final 361/361 tests en vanta-memory; fmt/clippy `-D warnings` limpios. E2E L0→L1→L2→L3→recall (`5e462792`). Plan archivado: `docs/dev/plans/archive/` (cierre `30198d5e`).
- **Ids:** `MEM-01`, `MEM-02`, `MEM-03`, `MEM-04`, `MEM-05`, `MEM-06`, `MEM-07`, `MEM-08a/b`, `MEM-09`, `MEM-10`, `MEM-11`, `MEM-12`, `MEM-13`, `MEM-14`, `MEM-15`, `MEM-16`, `MEM-17`, `MEM-18`, `MEM-19`, `MEM-20`, `MEM-21`, `MEM-34`, `MEM-35`
- **Cruce:** entrada espejo en `core-engine.md` (planner/core) y `bindings.md` (MEM-21 handlers MCP).

---

## Campaña P29 — Vanta Context Engine (superficie de memoria, F5)

### MEM-38..42 (+ADR-0029): Superficie F5 sobre vanta-memory — catch-up por campaña
- **Fecha:** 2026-08-20 → 2026-08-21
- **Objetivo:** Preparar la memoria como fuente del context engine: seed/import CLI vía bin propio `src/bin/vanta-seed.rs` con idempotencia content-hash (`d3eba4fc`, `MEM-39`), generation-log provenance best-effort L1/L2/L3 bajo `genlog/<session>` cap 100 (`1f89c0b6`, `MEM-41`), `recall_scope` híbrido session|agent|team default agent + primer test `search_multi` (`89777704`, `MEM-40`), auto-sync scheduler con ManagedTimer pull-based + busy guard (`2dba254f`, `MEM-45`*), reclaimer GC offload con retention_days post-cursor estricto e idempotente (`214a7820`, `MEM-42`), ADR-0029 borrador + superficies F5 documentadas en EMBEDDED_SDK (`badb5b9c`, `MEM-38`).
- **Nota:** \*MEM-45 se materializó dentro de la ventana P31 (commit `2dba254f` posterior al cierre formal de P29 `00f18662`); se registra aquí por pertenecer a la línea de auto-sync de F5.
- **Resultado:** ✅ 9/9 tareas de campaña (las de ensamblado puro viven en `context-engine.md`). Plan cerrado (`00f18662`).
- **Ids:** `MEM-38`, `MEM-39`, `MEM-40`, `MEM-41`, `MEM-42`

---

## Campaña P31 — Cierre Final (wiring productivo)

### MEM-43..49: Wiring, embeddings semánticos y recall real — catch-up por campaña
- **Fecha:** 2026-08-21 → 2026-08-22
- **Objetivo:** Llevar la memoria de "crate funcional" a "integrada en producto":
  - **MEM-43** (`a0bcb112`): wire context engine → pipeline worker como fase post-L3, flag de config, budget de tokens compartido entre compresión e inyección.
  - **MEM-44** (`785db22c`): e2e ingest→wiki_\* roundtrip cross-crate en `vantadb-mcp` (dev-dep vanta-memory, sin ciclo de paquetes).
  - **MEM-45** (`2dba254f`): auto-sync scheduler re-ingest programado (ver P29).
  - **MEM-46** (`e22b496a`): embeddings en L1 writer vía `EmbeddingProvider` core, feature opt-in (Principio 4 best-effort).
  - **MEM-47** (`f32e4d51`): semantic recall dual-pool + fusión RRF en recall/dedup/query, fallback keyword D38.
  - **MEM-48** (`4fbaa4a3`): compresión consume scores L1 reales (MemoryScoreMap + fallback heurístico).
  - **MEM-49** (`437bfee3`): guía socrática de revisión ADR-0029 + decisiones D21-D37 (prep articulación humana, Regla 5).
- **Resultado:** ✅ 8/8 tareas de campaña. Auditoría final con hallazgos registrados en `docs/api/VANTA_MEMORY.md` canónico (`673f18af`). ADR-0029 ACEPTADO con articulación humana completa (`9e76caff`). Plan cerrado (`460ce60a`).
- **Ids:** `MEM-43`, `MEM-44`, `MEM-45`, `MEM-46`, `MEM-47`, `MEM-48`, `MEM-49`

---

## Campaña Full-Backlog-Parallel 2026-08-29 — Dreaming + heat

### MEM-60: Heat + decay + contradiction provenance (W18-SOLO)
- **Fecha:** 2026-08-30
- **Objetivo:** Lifecycle tracking en L1 records — `bump_heat` on read, `decay_heat` on maintenance pass, `mark_contradiction` para invalidación trackable (old record preservado con `superseded_by`).
- **Resultado:** ✅ Módulo `vanta-memory/src/core/record/lifecycle.rs` (295L) + integration test `tests/heat_decay.rs` (184L, 3 tests). Suite pre-MEM-61: 503/503 integration tests OK. vanta-engine sync.

### MEM-61: Dreaming consolidación idle — sleep-time tiering (W19-SOLO)
- **Fecha:** 2026-08-30
- **Objetivo:** Job en downtime (idle ≥X min o cierre de sesión) que consolida L0/L1 crudo → learned context sin mutar el store original. Patrón Letta sleep-time compute validado via webfetch (`letta.com/blog/sleep-time-compute`, 2025-04-21).
- **Resultado:** ✅ Módulo nuevo `vanta-memory/src/core/dream/mod.rs` (~530L) + integration test `tests/dreaming.rs` (320L, 7 tests). 4 funciones públicas LLM-free + `Dreamer` trait (`Send + Sync`) para sleep-time tiering. Store consolidado en namespace `dream/<session>/<run_id>`; **nunca** toca `l1/<session>` (3 integration tests verifican byte-identical pre/post). `promote_dream_run` queda stub documentado (MEM-65/W21 cubre integración al pipeline_worker). 321/321 lib tests + 508/508 integration tests. vanta-engine staged para vanta-lead commit.
- **Invariante crítica:** la integración al `pipeline_worker.rs` se hace en MEM-65 (W21, parallel). MEM-61 solo entrega la primitiva standalone testeable.

### MEM-63: auto_recall doc + embeddings auto-on (durability-release-readiness Task 3, Wave 0)
- **Fecha:** 2026-09-05
- **Objetivo:** Corregir doc stale (`auto_recall.rs` decía que embeddings "degradan hasta wirear"; MEM-47 ya implementó el hook) + embeddings auto-on con provider configurado, keyword/chars-fallback solo sin provider.
- **Resultado:** ✅ Doc `auto_recall.rs` (módulo + `RecallMode::Embedding/Hybrid`) describe auto-on MEM-63; `L1DedupConfig::default()` wirea `local_embedding_hook()` con `embed-local`, `None` sin feature; tests `default_wires_local_provider_when_feature_on` + `default_stays_keyword_only_without_feature` verdes; suite 328 lib + 1 doc-test; fmt/clippy limpios. Código ya en HEAD vía `6058cc84` (trazabilidad documentada en task file).
- **Commit:** `docs(memory): auto_recall doc + auto-on embeddings (MEM-63)` (registro plan+task+backlog+avance; fuente ya en HEAD).

- MEM-63 (docs): doc stale auto_recall + auto-on - Resultado: verificado ya-en-HEAD via 6058cc84 (sin diff); suite 328/328. Sin commit nuevo (2026-09-05).

### MEM-66: claimStaleTasks multi-worker (plan 2026-09-08-backlog Wave1)
- **Fecha:** 2026-09-09
- **Objetivo:** port TDAM no porteado — worker muerto → otro worker reclama y procesa (exactly-once, lease sobre `lock_ttl_ms`).
- **Resultado:** ✅ test vanta-memory 0 failed + e2e `dead_worker_claim_reclaimed_and_processed_by_new_owner` + clippy 0. WIP +3 absorbido (desbloqueó PRX-08).
- **Commit:** 6ad16fbf

### MEM-68: gate opcional de aprobación de capturas (plan 2026-09-09 Wave0)
- **Fecha:** 2026-09-09
- **Objetivo:** gap #6 — cola pendiente→approve/reject (patrón Cursor), default off.
- **Resultado:** ✅ 27 suites 0 failed (4/4 nuevos) + clippy 0 + fmt; approve reusa `apply_dedup_batch`; `l1_writer.rs` intacto; worker wiring diferido por diseño.
- **Commit:** a4c1e75b

### MEM-69: batch extracción costo-reducida (plan 2026-09-10-code Wave0)
- **Fecha:** 2026-09-10
- **Objetivo:** agrupar split+dedup en 1 llamada LLM por flush (−40/50% tokens) sin perder quality gate.
- **Resultado:** ✅ módulo nuevo `l1_batch.rs` (`extract_dedup_batch` + `EXTRACT_DEDUP_TASK_ID`; juicio `dedup` inline por memoria, tolerancias exactas reutilizadas, pipeline_worker intacto) + `tests/l1_batch.rs` (7 tests incl. comparativo batch≡split: mismas memorias/acciones, 2 llamadas→1) + helper `split_messages` compartido y 2 visibility `pub(crate)`. Suite vanta-memory 0 failed (lib 328 + integración) + clippy `--all-targets --all-features -D warnings` 0 + fmt limpio. Deuda: wiring `pipeline_worker.rs` → slice 2 follow-up.
- **Commit:** 29e5b354

### MEM-70: harness LongMemEval-S/LoCoMo (plan 2026-09-10-code Wave1)
- **Fecha:** 2026-09-10
- **Objetivo:** harness reproducible + tabla BENCHMARKS.md §17 + Regla 11 limpia.
- **Resultado:** ✅ `evals/memory_bench.py` sintético (shape 20×16×40) + metodología; números reales DEFER (datasets licencia/peso); sin claims.
- **Commit:** e1f7daef

### MCP-41: auto-consolidación local-first (plan 2026-09-10-code Wave5)
- **Fecha:** 2026-09-10
- **Objetivo:** DISCOVERY arch + slice extract→consolidate→recall sin LLM key.
- **Resultado:** ✅ suite 0 failed + clippy/fmt 0 + ADR-0040; commit tras RESUME (bloqueo fmt ajeno).
- **Commit:** 6bf42a89

### FIND-86: wiring MEM-69 (dream TaskKind + batch opt-in) + tool 77 + MEM-70
- **Fecha:** 2026-09-16
- **Objetivo:** cablear memoria diferida ADR-0040 (Gate D A/A/B vía `question`): `TaskKind::Dream` + rama handle + flag batch opt-in + 4 tests wiring; tool 77 ratificada diseñada; MEM-70 DEFER-ratificado.
- **Resultado:** ✅ 542 passed / 0 failed (336 lib + 205 integración + 1 doc) + clippy 0 + fmt; review P2-01 approve.
- **Commit:** 0be84203 (discovery+spec) + 29ec9f02 (feat wiring)

### CODEX-130: `memory_recall` ve L1 del pipeline
- **Fecha:** 2026-09-19
- **Objetivo:** recall con sesión `mcp` vacía + filtro agent/team vs writer None = L1 invisible (Codex P1 PR #182).
- **Resultado:** `default_tenancy()` en writer (Store+Merge), D22 intacto; test sintético RED→GREEN; P2-01 approve.
- **Commit:** e0e74673 (rebase de 5b871993)

### CODEX-132: L0 mismo-ms sin pérdida
- **Fecha:** 2026-09-19
- **Objetivo:** key `t{ts}_0` + `timestamp_ms <= cursor` = 1 mensaje perdido en silencio (Codex P2).
- **Resultado:** key `t{ts}_{idx}_{fnv}` + tie-break probe; test determinista + concurrente verdes (expuso overwrite entre hilos, fixed sin locks); P2-01 approve.
- **Commit:** e0e74673 (rebase de 5b871993)

### FIND-184: const-assert `llm-driver` roto bajo unificación `--workspace` (E0080)
- **Fecha:** 2026-10-02
- **Objetivo:** `vanta-memory/tests/smoke.rs:14-16` (const-assert de opt-in de `llm-driver`) rompía el build bajo unificación `--workspace` (E0080) cuando la feature la habilitan otros miembros (`vantadb-mcp:27`, `vanta-proxy:30`); el gate canónico lo enmascaraba (`verify.ps1` corre `-p vantadb`). Repro: `cargo test -p vanta-memory --features llm-driver --test smoke --no-run` → exit 101.
- **Resultado:** ✅ cfg-gate del const-assert (solo compila con la feature off); verificado con `cargo check -p vanta-memory --tests` y `--features llm-driver` ✓; desbloquea Tests×3/Coverage/ASan/TSan en CI.
- **Commit:** `d4d7961a`

### DIST-01: `vanta-memory` publicable — dry-run verde + smoke externo + hold release-plz
- **Fecha:** 2026-10-04
- **Objetivo:** desbloquear la publicación del crate diferenciador (`publish = false` → metadata crates.io) + decisión de coordinación con el release.
- **Resultado:** ✅ Dry-run `cargo publish --dry-run -p vanta-memory` exit 0 (139 files; el build aislado compiló `vantadb 0.8.0` desde crates.io); smoke externo (`cargo run` en proyecto tmp fuera del workspace) OK; **hold explícito en `release-plz.toml`** (`[[package]] vanta-memory release = false` + checklist de unblock): Trusted Publishing exige que el crate exista → la primera publicación requiere bootstrap manual con token (acción owner en la ventana #238). Review P2-01 APPROVE (repro de `release-plz update` resolvió el riesgo de version-sync). Derivada: FIND-254 (tests empaquetados no compilan desde el `.crate` — no bloquea).
- **Commit:** 53996863 + 0508fc2f + 180bfb94 (local, sin push)

### MEMG-01: Detección de contradicción en ingesta L1 (master plan 0.9.0 F2, Task 37)
- **Fecha:** 2026-10-05
- **Objetivo:** la ingesta L1 detecta contradicciones explícitas contra registros vigentes de la sesión y marca el viejo con `superseded_by` (provenance, nunca delete) — el juicio viaja en el dedup existente (campo `contradicts`, 0 llamadas LLM nuevas, pre-mortem #1 resuelto) y el marcado reusa MEM-60 `mark_contradiction` en `write_memory` (choke point de ambos paths de ingesta: dos-call y MEM-69 fusionado).
- **Resultado:** ✅ Test dedicado `tests/l1_contradiction.rs` (11 tests: caso canónico "me gusta X" → "ya no me gusta X" + E2E por `run_l1_dedup`; conservadurismo: self/desconocidos/ya-superseded saltados, skip no marca, vector del viejo preservado). Suite scoped 587/587 + clippy `--all-features -D warnings` 0 + fmt 0 + gates docs 0. Review P2-01 vanta-review APPROVE (Low aplicados; F7 → FIND-277: consumo read-side pendiente). Docs: `docs/api/VANTA_MEMORY.md` §Contradicciones en ingesta.
- **Commit:** 10b35b41 (local, sin push)

### MEMG-02: Outcome loop — refuerzo de confianza post-recall (`reinforce`)
- **Fecha:** 2026-10-05
- **Objetivo:** la confianza era decorativa sin feedback: `reinforce` no tocaba la confianza y `last_validated_at_ms` no tenía writer.
- **Resultado:** ✅ `Embedded::reinforce(ns, key, outcome)` + `ReinforceOutcome { Used, Corrected, Unused }` (`#[non_exhaustive]`): Used +0.05 saturado (máx 1×/5 min), Corrected −0.10 piso 0.0 (success-only stamping), Unused neutral auditado; `derived` rechazado; state-only + audit `memory_reinforce`. MCP `memory_reinforce` (4 perfiles; counts 81/87). `vanta-memory::reinforce_recalled` + test de loop E2E. Evidencia: RED 35 → GREEN 13/13; vanta-memory 589/589; mcp 261/261; `vantadb --lib` 2318/2318; gates 0. Review P2-01 APPROVE (6 Low → 5 cerrados). FIND-278 (MCP put sin `confidence` declarable). Nota: disco C: a 2.5 MB → `target-cleanup -Clean` liberó 31 GB.
- **Commit:** fcc17ca7 + 782e111d (local, sin push)

### MEMG-12: Semántica v2 write-side en L1 — `confidence`/`valid_at` reales (master plan 0.9.0 F2, Task 40)
- **Fecha:** 2026-10-05
- **Objetivo:** con defaults, confianza y bitemporalidad quedaban inertes para la memoria (research: "confidence=0 en vanta-memory"): el pipeline L1 escribía sin declarar `valid_at_ms` ni confianza. Entregar la semántica v2 real en el punto único de escritura L1 (extracción + promoción dream); TTL/derived diferidos a FIND (stop 1.5sem del plan L1158).
- **Resultado:** ✅ `record_input`/`put_record` (punto único) estampa `valid_at_ms` = nacimiento del contenido (record.created_at ISO→ms; merge = earliest target; fallback al default core si parse falla o es 0) + `confidence` Asserted/D_a explícitas (MGR-12 §3.4; ADR-046 §D7/D8). Cubre ambos sitios L1 y ambos mecanismos (put secuencial + put_batch vía builder compartido). Test dedicado `l1_semantics_v2` 7/7 (RED 4/4 → GREEN; round-trip export/import; fallback; re-persist de contradicción); suite crate 602/602; fmt/clippy 0; gates docs 0; coverage 0 gaps. Review P2-01 vanta-review APPROVE (3 Low; FIND-281 derivado). Coordinación: `l1_writer.rs` co-autoreado con MEMG-11 en vuelo (su refactor `record_input`/put_batch integrado y verde en el mismo commit `9d0e371d`). Derivadas: FIND-279 (TTL/MGR-09), FIND-280 (derived/T1b/T1c).
- **Commit:** 9d0e371d + 9ca40219 (local, sin push)
### MEMG-11: Adopción del motor core — recall híbrido (~11×) + put_batch root-cause (2.4-2.7×)
- **Fecha:** 2026-10-05
- **Objetivo:** el motor híbrido del core (BM25+HNSW+RRF) vs el dual-pool propio más débil de vanta-memory; escrituras por group-commit.
- **Resultado:** ✅ **(a) Recall híbrido detrás de flag:** `RecallConfig.core_search` (default false = legacy byte-identical) + `search_records_core` (merge cross-session, ACL/cuarentena/scope D22); dual-pool preservado (record sin vector nunca se dropea). **Recall: 457.25 → 41.53 ms/query (~11×)**. **(b) Escrituras vía `put_batch`:** group-commit con `plan_write`/`record_input`; la medición reveló regresión (0.07×, full-rebuild O(total) por batch) → **fix root-cause en el core** (`replace_derived_indexes` por registro) → **2.36–2.71× estable**; DUR-03 cerrado (write-lock único + `_locked`; el review cazó un deadlock → rediseño). A/B documentado vs MGR-19 §1. Review adversarial 3 rondas → APPROVE. Coordinación MEMG-12: refactor de `l1_writer.rs` integrado en `9d0e371d` (co-autoría). FINDs: 282/283/284.
- **Commit:** 508e211e (local, sin push)

### MEMG-13: Superficies core restantes en memoria - historia/diff L1 + backup/restore (master plan 0.9.0 F2, Task 41)
- **Fecha:** 2026-10-05
- **Objetivo:** la memoria omitía lo que el core ya expone: consumir `versions` (auditoría/diff de L1) y `snapshot` (backup/restore) con tests por superficie; evaluar IQL + filtros/cursor restantes (stop 1sem del plan L1187: 2 superficies + FIND del resto).
- **Resultado:** ✅ **(a) Historia/diff L1:** `RecordVersion` + `read_record_versions`/`read_record_version` (core `Embedded::versions`/`get_version`, VS-CORE-07; audit-only documentado — no aplica el gate SCH-05) + `diff_records` puro (campos top-level; `skip_serializing_if` ausente = Null). **(b) Backup/restore:** `utils::backup::{create_snapshot,list_snapshots,restore_snapshot}` — delegación fina al core (quiesce/mirror/rollback + validación anti path-traversal del core intacta). Tests: `l1_history` (3 versiones reales + versión interior + diff) + `backup_snapshot` (round-trip Fjall, validación de nombre, NotFound, canary FIND-287). Suite crate 611/611; fmt/clippy 0; gates docs 0; coverage 0 gaps. Review P2-01 vanta-review APPROVE (post-fix: 1 Required corregido; 2 Optional aplicados). **FIND-285** (IQL no consumido — `core_search` MEMG-11 ya cubre el reuse; sin consumidor memory-side), **FIND-286** (filtros restantes de recall con sketch), **FIND-287** descubierto (snapshot_restore data/-only: delete/supersession post-snapshot no se revierten — core/storage).
- **Commit:** eb842343 + aa9e6843 (local, sin push)

### MEMG-17: Rollback semántico + erasure criptográfica + recibos verificables (master plan 0.9.0 F3, Task 55)
- **Fecha:** 2026-10-05
- **Objetivo:** cerrar el ciclo de confianza de la memoria (VMG / EDPS "verifiable proof of unlearning" / GDPR Art.17): (a) rollback semántico a versión/snapshot con linaje y alcance declarado (FIND-287); (b) recibos verificables con el contrato de los certificados VER-02; (c) erasure criptográfica por destrucción de DEK + tombstone; cada pieza con test propio (plan L1585; orden rollback → erasure → recibos).
- **Resultado:** ✅ **(a) Rollback:** `rollback_record` (append-only: escribe el target como versión NUEVA; `RollbackReport{from/to/new_version, changes, out_of_scope}`; `superseded_by` restaurado verbatim y declarado; NotFound para versión/registro/borrado — el delete purga versiones) + `rollback_snapshot` + `SnapshotRollbackReport` (reverted/not_reverted con **FIND-287** explícito, pinneado por test: m1 sigue borrado, m3 desaparece, m2 vuelve). **(b) Recibos:** `ErasureReceipt` espeja el contrato VER-02 (schema + sha256 canonical + `ChainEvidence` VER-01 + re-scan live **no claim-driven**: superficie parcial / status desconocido rechazados aun con hash recomputado; scope recreado → `residues reappeared`); firma ML-DSA-65 **no sancionada** (0 hits en el repo) → declarada fuera de alcance. **(c) Erasure:** registry de DEK por scope (32B CSPRNG, wrapped con el master `Cipher` existente; `seal`/`open` AES-256-GCM compuestos) + `erase_scope` (delete → tombstone + purge de versiones; blob irrecuperable); feature opt-in `erasure` (pull core `encryption`). Tests: rollback 7/7, erasure 11/11, suite **default 701/701** y **feature 712/712**, attestation VER-02 10/10; fmt/clippy/gates docs/coverage 0 gaps. Review P2-01 vanta-review: ronda 1 changes-required (Critical: test sin feature-gate rompía el build default/CI → `#![cfg(feature = "erasure")]`; 2 Low) → ronda 2 **APPROVE**. OCR delegation: fix `decode_hex` panic-free (payload corrupto multibyte) + test de regresión. **FIND-302** (integración envelope/DEK al write path L1 + recibos de rollback + firma).
- **Commit:** 80cb84b0 + 1ec943fe (local, sin push)

### MEMG-20: Checkpoints reanudables de tarea (dim 1) (master plan 0.9.0 F2, Task 43)
- **Fecha:** 2026-10-05
- **Objetivo:** la memoria de trabajo (dim1) no tenía semántica de checkpoints **de tarea** (paso actual + resultados parciales + resume tras interrupción): la base existía cableada (checkpoint de *pipeline* `Checkpoint`/`CheckpointManager` + `RunnerSessionState.last_l1_cursor`) pero sin la semántica para retomar una tarea multi-paso. API mínima sancionada por el plan (pre-mortem #2: paso + payload parcial + versión, sin orquestador; stop L1243).
- **Resultado:** ✅ **API `utils::task_checkpoint`** (módulo nuevo, **separado del pipeline checkpoint** — pre-mortem #1: tipos y namespace propios, `task_checkpoints` ≠ `pipeline_checkpoint`, pineado vía SDK crudo): `TaskCheckpoint{version,step,state,partial}` + `TaskState` + `TaskCheckpointError` (`#[non_exhaustive]`) + `TaskCheckpointManager` (`begin` idempotente / `advance` RMW / `complete`/`fail` / `load` / `delete`; un JSON record por tarea, single-writer in-process). **Test de reanudación (contrato L1241):** Fjall close/reopen — interrupción a mitad (pasos 0-1 de 5) → nueva instancia retoma en step 2, `executed == [2,3,4]` (sin repetir pasos completados), parcial preservado; misuse loud (`NotFound`/`NotInProgress`). Tests 9/9 + suite crate 620/620; fmt/clippy/doctest 2/2; gates docs 0; coverage 0 gaps. Review P2-01 vanta-review APPROVE (adversarial; 2 Optional doc-only aplicados). **Límite host documentado** (resume tras compactación = host; at-least-once). **FIND-288** (consumo por el host pendiente — sin consumer in-repo: tasks del worker single-pass; dim1 [PROPUESTA]).
- **Commit:** c1373528 + b40d929a (local, sin push)

### MEMG-07: Forgetting curves sobre L1 — política declarada por tipo + pass read-only + métrica (master plan 0.9.0 F2, Task 42)
- **Fecha:** 2026-10-05
- **Objetivo:** "el decay existe; el descarte no" (DELTA P1; FUT-10): el heat era entero (shift por pass) sin curva por tipo/edad ni métrica; sin fórmula canónica validada (N-09) → curva como **política declarada** (no calibración) con parámetros tuneables; la curva **deprioriza, nunca purga** (el descarte sigue siendo el gate explícito; stop L1215).
- **Resultado:** ✅ **Curva (`core::record::lifecycle`)**: `DecayPolicy` (half-life por `MemoryType`; defaults declarados: persona 90d · episodic 7d · instruction exento · work_fact/work_artifact 30d · work_task 14d · work_method 60d; `set_half_life`/`clear_half_life`) + `retention_factor = 2^(−age/half_life)` (familia `R=e^(−t/S)`; edad = último toque `updated_at`; exento/skew/ilegible → 1.0; hl=0 clamp sin NaN) + `effective_heat` (redondeo). **Pass read-only** `run_decay_pass(db, session, policy, now)` = `read_session_records` + `scan_decay` (pull-based, patrón `TimerScanner::run_once`) — **idempotente** (mismo `now` → mismo report + payloads byte-idénticos) y **nunca borra/muta**. **Métrica** `DecayReport{scanned,decayed,unchanged,exempt,below_threshold,heat_total,heat_effective}` + `heat_forgotten()`. **Compat por capas:** `heat`/`bump_heat`/`decay_heat`/`is_prune_eligible`/`PRUNE_HEAT_THRESHOLD` intactos; `MemoryRecord` sin campos nuevos (wire intacto). Tests: 11 unit (valores conocidos 0/1/2 half-lives, defaults pinneados, serde) + 4 integración (`forgetting_curve.rs`: totales exactos 15/6/9, idempotencia, nunca-borra, clasificación) → suite crate 635/635; fmt/clippy 0; gates docs 0; coverage 0 gaps. Review P2-01 vanta-review APPROVE (adversarial; 4 NITs doc-only aplicados). **FIND-289** (descarte automático + scheduler del pass sin consumidor in-repo → WIRE-15).
- **Commit:** 7e84244f + a9c778e5 (local, sin push)

### MEMG-21: Scoring multi-señal L1 (recencia + relevancia + importancia) + reflexión periódica (master plan 0.9.0 F2, Task 44)
- **Fecha:** 2026-10-05
- **Objetivo:** el recall L1 ordenaba solo por relevancia léxica/vectorial (overlap + cosine + RRF): el tiempo y la importancia no pesaban y no había reflexión sobre episódica (dim2/DELTA P2; deps MEMG-07 + MGR-15 sin research-doc). Entregar scoring compuesto opt-in + pase de reflexión (stop L1271: 3d → scoring + métrica; reflexión como slice separado con FIND si no cierra).
- **Resultado:** ✅ **(a) Scoring compuesto (`core::record::scoring`):** `composite = w_rel·relevance_norm + w_rec·recency + w_imp·importance` (Park et al. §4.1; forma exacta CrewAI) — pesos per-mille declarados `{relevance 500, recency 300, importance 200}` (defaults de CrewAI citados; configurables) + `DecayPolicy` de MEMG-07; recencia = `retention_factor` consumido (sin reimplementar el decay); importancia = `priority` (0-100; `-1` strict → 1.0); relevancia = score de pool o RRF fusionado, min-max sobre candidatos (Park); `composite_rank` con ties deterministas (`updated_at` desc, `id` asc). **(b) Integración opt-in al recall:** `perform_auto_recall_scored(..., now_ms)` + inner privado; los 11 callers de `perform_auto_recall_governed` quedan **byte-idénticos** (`None`); re-rank antes del `take(max_results)` en ambos routes (dual-pool legacy + `core_search`); `rrf_merge_scored` con `rrf_merge` como wrapper exacto. **(c) Métrica de ordenamiento before/after (fixture):** `[r1,r2,r3,r4]` → `[r2,r1,r3,r4]`, rank(r2) 2→1; neutralidad con señales uniformes; legacy intacto. **(d) Reflexión (`core::reflection`, precedente dream):** pase pull-based sobre episódica → lecciones `WorkMethod` deterministas (digest top-3/escena + provenance `metadata.reflection.source_ids`) o runner `Reflector` opcional (degradación P4); precondición `min_episodic=3` (`NotEnoughMaterial`); escribe solo en `reflection/<s>/<run_id>` — **L1 byte-idéntico pinneado**. Tests: 13+2 unit + 4+5 integración → suite crate 664/664 (2 skipped pre-existentes); fmt/clippy 0; gates docs 0; coverage 0 gaps. Review P2-01 vanta-review APPROVE (adversarial; NIT-1/NIT-2 doc-only aplicados). **FIND-290** (activación opt-in en hosts + promotion de lecciones + wiring del pase → WIRE-15/MGR-15). MGR-15 derivado de las fuentes citadas (Park §4.2 + CrewAI), sin spec inventada.
- **Commit:** 3fd223ab + b3f8df68 (local, sin push)

### MEMG-06: Spill a disco con recall (master plan 0.9.0 F3, Task 51)
- **Fecha:** 2026-10-05
- **Objetivo:** la compactación del context engine reemplazaba contenido por stubs `[compacted N chars]` sin persistir el payload (spill = 0 hits; `offload/` sin recall): hacer la compresión reversible — spill a disco del contenido compactado en el reemplazo + recall explícito por id/sesión, opt-in, GC reutilizado.
- **Resultado:** ✅ **(a) Puerto `SpillSink` (engine puro):** `assemble_inner` privado + `assemble_with_recall(..., spill: Option<&mut dyn SpillSink>)`; `stub_message → Option<String>` (`mem::take`, sin clone extra); el sink recibe el original en el reemplazo; `assemble` intacto; `IntegratedContext` intacto (wire serde). **(b) `context_engine::spill`:** `SpillStorage` (records `spill/<session>`, key = id sanitizado o `anon-<fnv1a64>`, get-before-put D19), `recall`/`recall_session`, `DbSpillSink` (warn-and-continue + contador), `reclaim`/`reclaim_as_of` reutilizando `MIN_RETENTION_DAYS` + `iso_to_epoch_secs` del reclaimer (undatables nunca se borran; el gate de cursor offload no aplica — contenido ya consumido). **(c) Opt-in:** `ContextAssemblyConfig.spill_enabled` (default false) cablea el sink en `run_context_assembly_inner`; sin config = byte-idéntico. Tests: unit sink (captura exacta) + spill storage 10 (round-trip id/sesión, dedup, corrupt-skip, GC) + e2e worker on/off + `precise-tokens` → suite crate 684/684 (2 skipped); fmt/clippy 0; gates docs 0; check `vantadb-mcp` + desktop OK. Review P2-01 vanta-review APPROVE (5 Low doc-only; 2 corregidos). **FIND-294** (aggressive/emergency sin captura) + **FIND-295** (hook offload sin callers).
- **Commit:** a76890f4 + a4dce566 (local, sin push)

### MEMG-03: Grafo ↔ memoria (L1–L3 como nodos/aristas) (master plan 0.9.0 F3, Task 50)
- **Fecha:** 2026-10-05
- **Objetivo:** memoria y grafo vivían separados (se busca, no se razona): integrar el linaje L1 como nodos/aristas reales en el core SDK — aristas en las ops (`supersede`/`derived_from`), query "quién cambió la fuente y por qué" vía BFS + provenance, y fijar la semántica de edges en rewrites de record.
- **Resultado:** ✅ **(a) Fix del wipe (riesgo #1 del plan, confirmado RED antes de tocar código):** `memory_record_to_node_owned` no copiaba `edges` y `engine.insert` es replace → todo rewrite de record (put/put_batch/put_record_exact/supersede/reinforce/quarantine_apply/promote + bulk import, 8 paths) borraba las aristas del nodo. Fix: `carry_graph_state` (preserva `edges`+`label_index`; skip probado barato en fresh inserts). **(b) Aristas de linaje canónicas:** `superseded_by` (supersede, old→new) y `derived_from` (put derivado, child→parent), bidireccionales e idempotentes vía `ensure_edge` `pub(crate)` (chequea forward `!reverse`/reverse `reverse`; sin duplicar en re-put). **(c) Reconciliación campo↔arista** (`reconcile_lineage_edges`: drop de forward stale + cleanup counterpart best-effort + ensure de lo declarado; cubre re-put de record superseded, cambio de parents, bulk import que no declara linaje). **(d) Query de linaje** respondible con API existente (`graph_bfs` Both + `get_node`/`get`) — sin símbolos públicos nuevos. **(e) DX-04:** puente `ns+key ↔ node_id` documentado (`docs/api/EMBEDDED_SDK.md`). Tests: 12/12 target nuevo; scoped core 2357/2358 (1 timeout de carga HNSW, pasa aislado); `vanta-memory` 684/684; fmt/clippy scoped limpios; gates docs 0. Review P2-01 vanta-review adversarial 3 rondas: H1 (stale `superseded_by` al re-put) + M1 (doc bulk) → F1 (bulk sin reconciliar) → **APPROVE**. **FIND-296** (L2/L3 escena/persona sin linkage) + **FIND-297** (contradicción vanta-memory sin arista) + nota MCP `add_edge` — fold pendiente en Backlog (archivo en vuelo por MEMG-08 al cierre).
- **Commit:** e41fef7d + 2b333f04 (local, sin push)

### MEMG-08: Formatos e ingestores — trait `Ingestor` (txt/json/csv end-to-end) (master plan 0.9.0 F3, Task 52)
- **Fecha:** 2026-10-05
- **Objetivo:** el path wiki solo aceptaba `.md` (`sources.rs:82`) y `put` recibía `payload: String` sin extractor de formatos — no se podían ingerir documentos reales (MGR-25; proveniencia AM7). Entregar trait `Ingestor` (formato → chunks `MemoryInput` con `metadata.source` obligatoria) + mínimo viable txt/json/csv end-to-end; stop L1503: 2sem sin los 5 formatos → trait + txt/json/csv + FIND de html/pdf/docx.
- **Resultado:** ✅ **Trait `Ingestor` + pipeline (core `vantadb::wiki::ingestors`):** `extensions()` + `ingest(namespace,file,content)` provisto (chunker 12k/400) con proveniencia **plana** `source`/`page`/`chunk` (decidido por evidencia: `Value` sin variante objeto + parser MCP rechaza anidados; precedente del demo PDF) y keys determinísticas `{file}#{chunk}` (re-scan idempotente); `Txt/Json/CsvIngestor` + `default_ingestors()` (orden declarado txt/json/csv → html → pdf → docx) + `scan_ingestable_sources(root,ns,registry)` con `SOURCE_CHAR_BUDGET` 28k **post-chunk** (overlap incluido; truncación declarada). **Refactor sin duplicar el guard traversal:** `collect_text_files` `pub(crate)` compartido con `scan_local_sources` (firma/semántica idénticas, 7 tests intactos). Tests: 4 integración e2e (proveniencia 3 formatos + put_batch/get/search + budget exacto 28_000) + 8 unit (overlap+cobridad, dispatch por registro fake, skips, subdirs, determinismo) + 15/15 ingest de vanta-memory + 32/32 wiki lib; fmt/clippy 0; rustdoc 0 warnings propios; gates docs 0; snapshot `public-api.txt` +101/−0 refrescado (compare 1/1). Review P2-01 vanta-review **APPROVE** (ronda 1; M1 snapshot + L1 feature-set + L2/L3 docs + N1 log, foldados y re-verificados). **FIND-298** (html/pdf/docx fuera del corte — deps a sancionar por la spec Notion; fila en Backlog). Commits LOCALES (sin push).
- **Commit:** 97039053 + e281c112 (local, sin push)
### MEMG-09: Track PI — spec repo-map + slice `code_index` v0
- **Fecha:** 2026-10-05
- **Objetivo:** el repo como identidad persistente (grafo decisión→código→test + taxonomía); specs MGR-22/23/24 ausentes.
- **Resultado:** ✅ Spec MGR-22 completa (chunker v0/v1, watcher content-hash/hooks, repo-map, API `code_index`/`code_watch`, ranking/budgets con industria validada) + spec MGR-23/24 + **slice v0**: tool `code_index` (Rust, sin deps; file-per-node, edges `defines`, idempotencia content-hash, reconcile stale; perfil `full`). RED→GREEN (10 fail "Tool not found" → 13/13); mcp_tests 115/115; nextest -p vantadb-mcp 170/170; clippy/fmt; docs gates. Review: changes-required → iteración → APPROVE (bloqueante real: 3 meta-tests por `default-filter`). FIND-299/300 + FIND-196 re-scopeada.
- **Commit:** 0905e3c5 + 5c38cf07 (local, sin push)

### MEMG-10: MGR-04 — Policy engine (trusted/tainted + RBAC por acción) (master plan 0.9.0 F3, Task 54)
- **Fecha:** 2026-10-05
- **Objetivo:** gobernanza de memoria: faltaba el modelo de confianza por namespace (trusted/tainted) + RBAC por acción (la base ACL/namespace existía; mgr-13 §4.1/§8 lo listaba como deuda v1.0). Corte declarado por stop L1559: spec + clase mínima (clasificación + gate de inyección) + FIND del enforcement restante.
- **Resultado:** ✅ Spec MGR-04 (`docs/dev/research/mgr-04-policy-engine.md`: modelo, integración retrieval/inyección, complementariedad SCH-05/VER-04, RBAC por acción, superficie L3 no gobernada declarada) + **trust gate end-to-end**: `TrustClass` + `InjectionPolicy::{from_parts,trust_class}` sobre la política compartida VER-04 (tainted no inyecta por defecto; opt-in `include_tainted`; gates ACL AND trust; defaults byte-idénticos) aplicado en las superficies gobernadas (auto_recall L1/persona/escena, proxy block, MCP recall/assemble) con wiring opt-in (proxy TOML `tainted_namespaces`/`include_tainted`; MCP `VANTADB_MCP_TAINTED_NAMESPACES`/`VANTADB_MCP_INCLUDE_TAINTED`) + **audit de denials RBAC por acción** en el middleware (`auth_rbac`: action/enforced/scope; antes el 403 era silencioso; sin cambio de semántica de autorización). Tests: memg10_trust_gate 4/4 + policy 9/9 + vanta-memory 694/694; vanta-proxy 322/322; vantadb-mcp 171/171; rbac_namespace 11/11 (read/write/delete) + auth/rbac 48/48; fmt/clippy scoped 4/4; gates docs + coverage 0 gaps. Review P2-01 vanta-review: changes-required (doc-only) → fixes → **APPROVE**. **FIND-301** (enforcement por acción + roles configurables + trust HTTP + promoción curada + superficie L3 del pipeline).
- **Commit:** e0373d7b + eec590b5 (local, sin push)

### MEMG-04: Multi-tenant — enforcement + cuotas (master plan 0.9.0 F4, Task 56)
- **Fecha:** 2026-10-05
- **Objetivo:** aislamiento real por tenant (hoy namespace = partición de keyspace sin barrera; RBAC ns-scoped solo en path/query y roles hardcodeados) + cuota por namespace + boundary de billing. Corte declarado: enforcement (a) completo + cuota (b) mínima real + boundary (c) registrado + FIND de residuales.
- **Resultado:** ✅ **Modelo (D1):** tenant = namespace (sin entidad nueva; `memory_node_id` ya particiona; 0 migración). **Credenciales (D3):** primaria+alt (L1) mapeadas por `token_role_map` a roles ns-scoped registrables desde `RbacCfg.roles` (`namespace_read`/`namespace_write`; aditivo, built-ins no overridables; N>2 → FIND-304). **Enforcement (D7):** middleware lee el body SOLO para credenciales con rol no-admin en superficies declaradas (records put/batch, search, export, import), extracción precisa por ruta (sin deep-walk) + unión path/query∪body; **fail-closed**: body 0-ns exige además el permiso coarse global (`require_coarse` — cierra el bypass `?namespace=own` + body blank → `search_all`, C1 del review); `POST /search` = read; `extract_namespace` decodifica el segmento (`agent%2Fmain`); audit de denials reusado. **Cuota (D5):** opt-in `Config.max_records_per_namespace` sobre el contador mantenido `doc_count` (text-index) en `put`/`put_batch`/`put_record_exact`; solo inserts nuevos; batch pre-check all-or-nothing; `ResourceLimit` explícito + audit `quota_rejected`; bulk `.vdbdump` exento (FIND-304). **Boundary (D8):** "no billing en motor" registrado (metrado exportable vía audit; host/proxy). Tests: rbac_namespace **34/34** (11 preexistentes + 15 MEMG-04: no-cruce 2 tenants por superficie + 4 regresiones review), quota **8/8**, lib **2445/2445** + HNSW stress 1/1, request_id 3/3; fmt/clippy 0; docs gates (check-links 0, check-docs all clear). Review P2-01 vanta-review adversarial 3 rondas: **C1** (bypass cross-tenant search) + R1-R5 (bulk exento/docs, tightening writer/reader documentado+testeado, ns con `/`, search read-only, FIND-303→304) → **APPROVE**. **FIND-304** (N>2 credenciales, cuota bulk/OPFS/L1, roles env/TOML) + nota en FIND-301. Sin ADR (extensión de cobertura del modelo SRV-05, no cambio de modelo — decisión en `campaign_memory`).
- **Commit:** aa79532f + 6e2491bd (local, sin push)

### MEMG-16: Compartir/colaboración multi-agente — scopes + permisos + revocación (master plan 0.9.0 F4, Task 58)
- **Fecha:** 2026-10-05
- **Objetivo:** el `PermissionChecker` (allow-only, completo y testeado desde MEM-04) no estaba cableado a ninguna superficie de producto; "revocación" ambigua (acceso vs purga); propagación/EXE-07 sin frontera. Stop L1676: scopes + grants + revoke mínimo sobre el checker + doc del modelo + FIND.
- **Resultado:** ✅ **Superficie (D1):** SDK memory API (`Embedded`, `src/sdk/api/sharing.rs` — choke point único de bindings/server/MCP/vanta-memory); reuso puro del checker/EntityStore (0 re-implementación; `checker.rs` intacto). **Ops:** `share_asset`/`add_team_member`/`revoke_team_member`/`grant_access`/`revoke_access`/`check_access`/`get_shared`/`put_shared` (+audit de gestión; deny = `record: None`, sin oráculo). **Semántica (D3):** revocación = acceso futuro (delete ACL / `status=removed`; el checker lee entidades vivas) — NUNCA purga (frontera erasure MEMG-17); fail-closed; grants no saltan membresía. **Scopes (D4):** org→namespace, proyecto→sub-namespace `{org}/{project}`, team→`team_member`, asset→`asset`+`acl`. **Doc:** `docs/api/SHARING.md` (modelo, propagación, fronteras MEMG-17/SCH-05/MEMG-10, containment/EXE-07, trust model, límites de charset). Tests 10/10 (RED→GREEN; incluye regresión de separadores de claves compuestas); lib 2359/2359; fmt/clippy 0; gates docs 0. Review P2-01 vanta-review 2 rondas: **R1** (colisión de claves compuestas por `.` → `validate_entity_id` + test) + **R2** (staging selectivo: Backlog por patch, índices regenerados en worktree reducido, 0 fuga de WIP ajeno) → **APPROVE**. **FIND-305** (exposición server/MCP + recall por grant + EXE-07) + **FIND-306** (clippy drift, resuelto por VER-10). Sin ADR (expone el modelo ya adjudicado; decisiones D1/D3 en `campaign_memory`).
- **Commit:** 97571f3f + 8a039119 + 77e8e9d4 (local, sin push)
### DUR-03 r3 — race de escrituras mismo-key (fix CI Windows del PR #242)
- **Fecha:** 2026-10-06
- **Objetivo:** cerrar el doble decremento de stats del text index con upserts concurrentes de la misma key (CI Windows rojo: `text index df would go negative` en `concurrent_same_ms_triggers_lose_no_messages`).
- **Resultado:** ✅ Root cause: `put_one`/`put_record_exact` solo tomaban el read guard del `purge_lock` (compartido — protege vs purge, no vs otro writer) y `delete_inner` no tomaba nada; dos writers resolvían la misma generación y ambos aplicaban su decremento (FIND-252). Fix: **write guard** en todo el read-modify-write (resolve + insert/delete + index replacement) en `put_one`/`put_record_exact`/`delete_inner` — la misma sección crítica de `put_batch_inner`. Test de regresión `test_concurrent_same_key_upserts_do_not_corrupt_text_stats` (8×100; RED <1s pre-fix → 10/10 GREEN). Review P2-01 **APPROVE** (barrido de reentrancy/lock-order sin hallazgos; residuales anotados en FIND-245). Verificación lead: regresión PASS + `conversation_hook` 4/4 + `edge_cases` 29/29.
- **Commit:** 6f3990aa (local, sin push)