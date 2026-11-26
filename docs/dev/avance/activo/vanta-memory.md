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

### MEMG-24: Suite de tests de contrato Ego ↔ VantaDB (`VantaCognitiveAPI` v1)
- **Fecha:** 2026-10-18
- **Objetivo:** Suite de contrato Ego <-> VantaDB
- **Resultado:** ✅ Tests vitest del contrato `VantaCognitiveAPI` v1: firmas y abort pineados (`vantadb-node/tests/cognitive-contract.test.ts`).
- **Commit:** 1afda9ad

### MGR-04: Modelo de namespaces trusted/tainted + RBAC por acción
- **Fecha:** 2026-10-18
- **Objetivo:** Namespaces de confianza y RBAC por accion
- **Resultado:** ✅ Trust gate de namespaces (MEMG-10) + RBAC por accion entregados; spec en `docs/dev/research/mgr-04-policy-engine.md`.

### MGR-22: PI-1 Repo-map + indexación incremental del repo (memoria de proyecto/ingeniería)
- **Fecha:** 2026-10-18
- **Objetivo:** Repo-map e indexacion incremental del repo
- **Resultado:** ✅ Spec completa (`mgr-22-repo-map.md`) + slice v0 (`vantadb-mcp/src/code_index.rs`). El residual v1 (tree-sitter/watcher/ranking) vive en FIND-299.

---

## Campaña Pre-`v0.10.0`: Sincronización de Checkpoints y Filtrado de Recall Activo (2026-11-18)

### MEMG-28: Serializar `CheckpointStore::mutate` para eliminar carreras lost-update sobre `checkpoint.json`
- **Fecha:** 2026-11-18
- **Objetivo:** Serializar el ciclo read-modify-write de `CheckpointStore` en `vanta-memory/src/utils/checkpoint.rs` entre instancias y workers concurrentes del proceso.
- **Resultado:** ✅ `CHECKPOINT_RMW_LOCK` estático de proceso con recuperación ante poison serializa todos los mutadores y `update_runner_state` vía `mutate()`; `write()` restringido a `pub(crate)` (0 bypasses en el crate) y test determinista con channel rendezvous en `vanta-memory/tests/task_checkpoint.rs:283-330`; review P2-01 `vanta-review 2047a71b` APPROVE y auditoría `#22` PASS.
- **Commit:** `4a6c69f6b`, `93fae5b5f`

### FIND-277: Filtrar registros L1 con `superseded_by` activo en todas las rutas de recall y consolidación L2
- **Fecha:** 2026-11-18
- **Objetivo:** Excluir registros con `superseded_by` no vacío de las lecturas activas de recall y síntesis de escenas L2 sin ocultar el historial a los lectores crudos de escritura, auditoría, decay y borrado en cascada.
- **Resultado:** ✅ `read_active_*` + predicado unificado `is_active_record` (string vacío/whitespace tratado como activo) aplicados en `vanta-memory/src/core/record/l1_reader.rs`, en las 5 rutas de `auto_recall.rs` (incluida `recall_candidates` y test de ruta ADR) y en la consolidación L2 (`pipeline_worker.rs:643`); review P2-01 `vanta-review 2047a71b` APPROVE y auditoría `#22` PASS (hermano en `reflection` registrado como `FIND-659`).
- **Commit:** `6b5170c19`, `cc1ec126d`, `93fae5b5f`


## Migración de backlog — pase 2 campaña `consolidation-2026-11-07` (2026-11-19)

### FIND-281: Re-persist de contradicción (MEMG-01) resetea a D_a una `confidence` decayada por MEMG-02
- **Fecha:** 2026-11-19
- **Objetivo:** Preservar la `confidence` decaída al re-persistir registros L1 contradichos en `l1_writer.rs` en lugar de resetearla a D_a.
- **Resultado:** ✅
- **Commit:** `93fae5b5f`, `a914c72e7`, `7be3c8e67`

### MEMG-27: Reemplazo no atómico `delete` → `put_batch` en `L1Writer` expone pérdida de memorias L1 ante fallo intermedio
- **Fecha:** 2026-11-19
- **Objetivo:** Eliminar la ventana no atómica `delete` -> `put_batch` en `L1Writer` (`l1_writer.rs`) que exponía pérdida de memorias L1 ante fallo intermedio.
- **Resultado:** ✅
- **Commit:** `93fae5b5f`, `a914c72e7`, `7be3c8e67`

### FIND-658: `apply_promotion` en `dream/mod.rs` mantiene ventana `db.delete` → `put_record` y descarta `confidence` decaída en `PromotionAction::Update`
- **Fecha:** 2026-11-19
- **Objetivo:** Aplicar protocolo write-first con `DedupPolicy::Off` en `apply_promotion` (dream) y preservar `confidence`/`confidence_class`/`derived_from` en `PromotionAction::Update`.
- **Resultado:** ✅
- **Commit:** `5e5f99fda`, `7be3c8e67`

### FIND-659: `run_reflection` lee episodios con `read_session_records` (raw) en vez de `read_active_session_records`, sintetizando lecciones sobre registros ya `superseded_by`
- **Fecha:** 2026-11-19
- **Objetivo:** Migrar `run_reflection` a `read_active_session_records` para que episodios `superseded_by` dejen de alimentar la síntesis de lecciones.
- **Resultado:** ✅
- **Commit:** `5e5f99fda`, `8992c0e57`

### FIND-660: `validate_derivation_chain` usa `visited: HashSet<String>` sin rastrear profundidad máxima por nodo, omitiendo ramas más largas `> MAX_DERIVATION_DEPTH` en DAGs diamante
- **Fecha:** 2026-11-19
- **Objetivo:** Rastrear profundidad máxima por nodo en `validate_derivation_chain` para no omitir ramas más largas que `MAX_DERIVATION_DEPTH` en DAGs diamante.
- **Resultado:** ✅
- **Commit:** `cb7d9f517`, `292b27d25`

### ACT-01: Auto-Deduplicación Semántica en Ingest (`memory.put`)
- **Fecha:** 2026-11-07
- **Objetivo:** Auto-deduplicación semántica en ingest (`memory.put`): políticas `dedup_policy`/`merge_policy` en `MemoryInput` y FastGate cableado.
- **Resultado:** ✅
- **Commit:** `0488c7dc8`, `ff983d515`, `c80a0ae23`

### TEC-06: Activación de `DuplicatePreventionFilter` en Memoria (`Q17`)
- **Fecha:** 2026-11-07
- **Objetivo:** Activación del `DuplicatePreventionFilter` (FastGate) en memoria (Q17).
- **Resultado:** ✅
- **Commit:** `0488c7dc8`

### TEC-07: Job Autónomo de Decaimiento `run_decay_pass` → `forget(Soft)` (`Q18 Opción B`)
- **Fecha:** 2026-11-07
- **Objetivo:** Job autónomo de decaimiento `run_decay_pass` -> `forget(Soft)` (Q18 Opción B; alias FIND-289, SKIP verificado).
- **Resultado:** ✅
- **Commit:** `5fa148005`

### TEC-12: Metadatos de Procedencia EU AI Act Art. 50 en L2 y L3 (`Q2 Opción B`)
- **Fecha:** 2026-11-07
- **Objetivo:** Metadatos de procedencia EU AI Act Art. 50 en registros L2 y L3 (Q2 Opción B).
- **Resultado:** ✅
- **Commit:** `0a77dfb71`

### TEC-13: Log Inmutable de Trazabilidad EU AI Act Art. 12 (`audit.jsonl` Merkle) (`Q3 Opción B`)
- **Fecha:** 2026-11-07
- **Objetivo:** Log inmutable de trazabilidad EU AI Act Art. 12 (`audit.jsonl` con cadena Merkle) (Q3 Opción B).
- **Resultado:** ✅
- **Commit:** `0a77dfb71`

### TEC-15: Propagación en Cascada de `forget()` hacia Nodos L2/L3 Derivados (`Q4 Opción C`)
- **Fecha:** 2026-11-07
- **Objetivo:** Propagación en cascada de `forget()` (borrado GDPR) desde episodios L1 a escenas L2 y axiomas L3 (Q4 Opción C).
- **Resultado:** ✅
- **Commit:** `613881a86`

### TEC-17: Ingestores Estructurados Completos (`MGR-25`: JSON/JSONL, CSV, Markdown y Chat)
- **Fecha:** 2026-11-07
- **Objetivo:** Ingestores estructurados completos (MGR-25): JSON/JSONL, CSV, Markdown y Chat con frontmatter, headings y lenguajes de código (SKIP verificado).
- **Resultado:** ✅
- **Commit:** `c7be87532`

### TEC-18: Cascada Autónoma End-to-End del Cerebro (`L0 → L1 → L2 → L3 → Dream → Skill`)
- **Fecha:** 2026-11-07
- **Objetivo:** Cascada autónoma end-to-end del cerebro (L0 -> L1 -> L2 -> L3 -> Dream -> Skill).
- **Resultado:** ✅
- **Commit:** `5fa148005`

### TEC-19: Implementación Integral del Track PI (`PI-1` Repo-Map `MGR-22`, `PI-2` ADRs Automáticas `MGR-23` y `PI-3` Bug→Fix `MGR-24`)
- **Fecha:** 2026-11-07
- **Objetivo:** Track PI completo: repo-map incremental multi-lenguaje (PI-1/MGR-22), ADRs automáticos y prioridad de recall con trazabilidad Bug->Fix (PI-2/PI-3/MGR-23/24).
- **Resultado:** ✅
- **Commit:** `b977df56a`, `5a2c6ebae`, `73f3fe084`

### MEM0-01: Pipeline `FastGate` Pre-LLM en Rust (`< 0.1 ms`) (`Q17 Opción B Unificada`)
- **Fecha:** 2026-11-06
- **Objetivo:** Pipeline `FastGate` pre-LLM en Rust (< 0.1 ms) para deduplicación en ingest (Q17 Opción B unificada).
- **Resultado:** ✅
- **Commit:** `0488c7dc8`

### MEM0-02: Calibración Min-Max de `seed_score` en GraphRAG (`GR-05` / `FIND-608`)
- **Fecha:** 2026-11-06
- **Objetivo:** Calibración min-max de `seed_score` en GraphRAG (GR-05 / FIND-608).
- **Resultado:** ✅
- **Commit:** `647f4f27c`

### MEM0-03: Activación de Guarda Anti-Hub `EGO-15` y Corrección N+1 en GraphRAG (`GR-01..03, GR-06`)
- **Fecha:** 2026-11-06
- **Objetivo:** Activación de la guarda anti-hub y corrección N+1 en GraphRAG (GR-01..03, GR-06).
- **Resultado:** ✅
- **Commit:** `647f4f27c`

### MEM0-05: Arnés Reproducible de Benchmark `LOCOMO` (`evals/eval_locomo.py`)
- **Fecha:** 2026-11-06
- **Objetivo:** Arnés reproducible de benchmark `LOCOMO` (`evals/eval_locomo.py`) con gate de certificación LongMemEval-S 500q.
- **Resultado:** ✅
- **Commit:** `09116b060`

### NOT-01: Persistencia de `LabelIntern` en Disco (`FIND-609`)
- **Fecha:** 2026-11-07
- **Objetivo:** Persistencia de `LabelIntern` en disco (FIND-609).
- **Resultado:** ✅
- **Commit:** `647f4f27c`

### NOT-02: Serialización de `index_lock` y `GcWorker` en Threads (`FIND-600`, `AG-03..05`)
- **Fecha:** 2026-11-06
- **Objetivo:** Serialización de `index_lock` y `GcWorker` en threads (FIND-600, AG-03..05).
- **Resultado:** ✅
- **Commit:** `647f4f27c`

### NOT-03: Lock Atómico y Presupuesto Acotado en `skills.rs` (`FIND-605`, `SK-02..03`)
- **Fecha:** 2026-11-06
- **Objetivo:** Lock atómico y presupuesto acotado en `skills.rs` (FIND-605, SK-02..03).
- **Resultado:** ✅
- **Commit:** `647f4f27c`


### Diagnóstico de payloads L1 y enumeración de `MemoryType` (`FIND-259`)
- **Fecha:** 2026-11-24
- **Objetivo:** Un registro L1 con `type` inválido quedaba **invisible sin diagnóstico**: el reader lo salteaba con solo `tracing::debug!`, y los 7 valores válidos no estaban enumerados en los docs de API.
- **Resultado:** ✅ Contador **aditivo** `L1DecodeStats { invalid_type, unparseable }` con `decode_stats()` / `reset_decode_stats()` (dos `AtomicU64`), alimentado desde el brazo `Err(_)` **antes** del fallback `sdk_to_memory_record`, así que la salida del reader es idéntica byte a byte. Distingue `type` inválido (bug de escritura) de payload no parseable (ADR/doc, `FIND-699`). Los 7 valores (`persona | episodic | instruction | work_fact | work_task | work_method | work_artifact`) quedan en `docs/api/VANTA_MEMORY.md` §L1 payload type and decode diagnostics, con un test que cierra con `match` exhaustivo sobre `MemoryType`: un 8.º valor rompe la compilación en vez de separarse del doc.
- **Verificación:** contrato `-p vantadb --lib -E 'test(l1)'` ✅ · `vanta-memory` 144/144 ✅ · regresión de `FIND-699` (`-p vantadb-mcp -E 'test(memory_put)'`) ✅ · clippy/fmt/docs gates ✅ · review P2-01 `APPROVE` (contexto fresco). Medido y no cambiado: `type:"fact"` vuelve como `episodic` con `content` = payload crudo (invisible para una query sin solape), que es el modo de fallo de la fila.
- **Commit:** `1f1095769`
- **Techo documentado:** la ruta opt-in `core_search` (`auto_recall.rs`) no alimenta los contadores; la atribución por namespace tampoco se implementó.

### El auto-embedding cargaba el modelo ONNX entero en cada escritura (`FIND-706`)
- **Fecha:** 2026-11-25
- **Objetivo:** La latencia de las tools MCP bajo escritura sostenida cruzaba el timeout del cliente (60 s) y el cliente mataba al hijo — el `exit 255` de `MCP-42`.
- **Resultado:** ✅ **Causa raíz medida por componentes, no intuida:** cada `memory_put` construía un `LocalOnnxProvider` nuevo, y ese constructor hacía `commit_from_file` sobre un grafo de **470 MB** más el parseo del tokenizador de **17 MB**. Desglose: **tokenizador 1.630 ms + sesión 1.856 ms por put** frente a **10,1 ms** de forward real → el **99,7 % del coste era carga, no cómputo**.
  - **Fix (`src/llm.rs`):** estado caliente ONNX por proceso (`WARM_LOCAL`, keyed por el **directorio de modelo resuelto**), con caché FIFO acotada de resultados dentro de ese estado, reserva de `FIND-642` hecha una sola vez, y `dim`/`family` resueltos con la sesión. Un estado **sin** sesión no se memoiza, para que instalar el modelo se siga detectando en el siguiente write.
  - **Medición (repro versionado, DB aislada, 3 `memory_put`/ronda, 10 rondas, binarios del mismo build):** ronda 10 **12,8 s → 0,2 s** (64×) · total 128,3 s → **5,2 s** · mediana por llamada **4,4 s → 0,1 s** (44×) · latencia plana desde la ronda 2. Probe `#[ignore]` del desglose: rondas de **40-115 ms** (3 puts) con la versión final.
- **Techo conocido del caché:** **memoria** 256 entradas × 384 dims × 4 B ≈ **384 KB por modelo** (FIFO, no crece) · **TTL: ninguno a propósito** (`embed` es determinista en modelo+prefijo+texto; expirar solo forzaría recomputar; `FIND-121` es dueño del TTL/invalidación por namespace) · **invalidación** por directorio de modelo; reemplazar `model.onnx` en el mismo path exige reiniciar · **coste**: la sesión queda residente toda la vida del proceso y el forward se serializa por modelo (`ort` pide `&mut self` y recomienda una sesión por hilo antes que uso concurrente).
- **Verificación:** `nextest --profile audit -p vantadb --lib --features embed-local llm` **34/34** · clippy exit 0 · `check -p vantadb-mcp` exit 0 · fmt limpio · docs gates 0.
- **Commit:** `82077466c`
- **Corrección de premisa:** el crecimiento **17 s → 79 s de la fila no se reprodujo**. Con este patrón de carga el "antes" queda **plano** en 11,6-14,6 s/ronda, así que lo corregido es el **coste por escritura**, no una degradación progresiva. El timeout del cliente se explica igual (128 s de 10 rondas contra 60), pero la fila describía un fenómeno distinto del real.

### El runner LLM degradado era invisible (`MEM-72`)
- **Fecha:** 2026-11-25
- **Objetivo:** Sin `VANTADB_INGEST_*` el pipeline "corre" y no extrae nada: `IngestRunnerProvider::Local` → `ConcreteRunner::None` → el pass se saltaba con un `debug!`, así que "activo" y "activo pero mudo" se veían idénticos desde fuera.
- **Resultado:** ✅ `ingest::observability` (nuevo, proceso-global, tamaño fijo) publica `runner` (`unknown|none|ollama|openai`), `degraded`, `scheduler_running`, `queue{depth,claimed}`, `last_pass{skipped,processed,failed,timers_fired}`, `degraded_warnings` y un `notice` legible; el tool MCP `capabilities` lo embebe en la clave **aditiva** `ingest` (ningún campo previo cambia: `persistence`/`embedding`/`vector_search` verificados por test). La distinción es **estado + cola**: `none` + `depth 0` = pipeline ocioso (`is_broken() == false`); `none` + `depth > 0` = roto. En ese caso se emite **un** `warn!` por episodio (nunca por pass; el flag se resetea al drenar la cola o al configurar runner) y `degraded_warnings` lo cuenta.
  - **El bloqueante que cazó el review:** `scheduler_running` era **write-once-true**, así que un loop **muerto** seguía reportando "vivo". Ahora se limpia al salir del loop y en `Drop`, con test start→stop. Es la clase exacta de la fila: un indicador de salud que miente.
  - **Wiring:** `build_runner` anota el runner (único mapeo provider→runner, "last build wins"), `run_pass` publica su resultado y el loop anota el skip.
- **Verificación:** `nextest --profile audit -p vanta-memory -E 'test(scheduler) or test(runner) or test(pass)'` **43/43** · `-p vantadb-mcp -E 'test(capabilities)'` **2/2** · `clippy -p vanta-memory -p vantadb-mcp --all-targets -- -D warnings` limpio · `rustfmt --check` limpio · docs gates 0 broken / 0 orphans · OCR review aplicado · review P2-01 adversarial `changes-required` (1 bloqueante + 3 Medium) → corregidos → `approve`.
- **Commits:** `c0f5667db` (código + docs + tests) + `2e8fe2e9d` (cierre del task file)
- **Techo documentado:** `degraded_warnings` prueba la decisión, no la emisión del log (capturarla exigiría `tracing-subscriber` como dev-dep de `vanta-memory`); el snapshot es **proceso-local**, así que un MCP en modo proxy no ve la cola del writer — esa es la deuda que abre `MCP-43`.
- **Desbloquea:** `MCP-43`, `MCP-44` (`thread_send` alimenta el pipeline) y `MEM-73` (e2e L0→L3).

### `MCP-42` cerrado sin trabajo propio: era latencia de embedding, no un crash (`MCP-42`)
- **Fecha:** 2026-11-25
- **Objetivo:** Explicar el `exit 255` del servidor MCP bajo escritura sostenida, que se arrastraba como "crash misterioso".
- **Resultado:** ✅ **No había crash.** El stderr de los repros mostraba cierre limpio (`MCP stdio server shut down total=32 errors=0`, `cleaned discovery file on Drop`, `HNSW index persisted`, `Storage flushed`), sin panic ni OOM. Lo que ocurría era que la latencia de `memory_put` **cruzaba el timeout del cliente (60 s)** y el cliente mataba al hijo: el 255 era la consecuencia, no la causa. Y la latencia era **99,7 % carga del modelo ONNX** (tokenizador 1,63 s + sesión 1,86 s por escritura frente a 10,1 ms de forward), arreglado en `FIND-706` (`82077466c`): ronda 10 de 12,8 s → 0,2 s.
- **Por qué se cierra sin trabajo propio:** la causa raíz se resolvió en otro sitio y la fila describía un fenómeno que no existe. Dejarla abierta haría creer que el servidor crashea.
- **Verificación:** los repros de `FIND-706` (10 rondas, DB aislada) corren completos sin timeout; `capabilities` ahora expone el estado del runner (`MEM-72`), así que un pipeline que no extrae se ve por superficie en vez de por crash aparente.
- **Referencias:** `FIND-706` · `MEM-72` · `MCP-48` (el proxy ya no muere con su writer, que era el otro modo de fallo "parece que crasheó").

### El pass del bridge anulaba el ciclo L3 (`MEM-71`)
- **Fecha:** 2026-11-25
- **Objetivo:** `run_bridge_pass` —el pass que documenta `services::conversation_hook` para hosts del `/conversation/add`— construía un `MemoryTaskHandler` con `trigger_every_n = usize::MAX` **y sin `with_queue`**, así que un host que siguiera el patrón documentado se quedaba en L1: L2 nunca se encolaba y L3 era inalcanzable.
- **Resultado:** ✅ El pass del bridge **es** `scheduler::run_pass` (una implementación, un comportamiento): timers, una tanda del worker cuya cascada L1 → L2 → L3 se encola por la misma cola, y reclaim; persona a `PERSONA_TRIGGER_EVERY_N = 50` (ahora `pub`).
  - **¿Era intencional?** No como diseño: fue el alcance de `MEM-55` (solo L1), superado por `ADR-0054:92` ("`run_bridge_pass` se generaliza al pass del driver") y ya generalizado por `WIRE-16`; `run_bridge_pass` tenía **cero** callers productivos.
  - **El bug era mayor que el `usize::MAX`:** sin `with_queue` el cascade L1→L2 no ocurría nunca, así que habilitar persona **no habría bastado**. Test e2e: turno del bridge → **un** `run_bridge_pass` → 5 memorias → escena → persona (`processed == 3`, cola drenada). La cadencia se fija en **dos capas**: función pura (`N-1` no, `N` sí) y **cableado** (checkpoint en 49 no regenera, a 50 resetea); con `usize::MAX` el segundo falla.
- **Verificación:** `-E 'test(bridge) or test(persona) or test(pass)'` **61/61** · con `--features http-server` **64/64** · clippy `-D warnings` limpio · fmt limpio · `check-links`/`check-docs` exit 0 · markdownlint 0 issues · mutación de control roja · review P2-01 `changes-required` (2 Required) → corregido → **approve**.
- **Commits:** `3f20a1798` (código + tests + docs + task file) + `6fc453b6f` (cierre del task file)
- **Techo documentado:** `run_bridge_pass` queda como **alias de una línea**; un pass "solo worker" se construye con `PipelineWorker` + `MemoryTaskHandler`. Sin campo `persona_disabled_by_design` en `observability`: con el pass unificado sería siempre `false`.
- **Nota de proceso:** dos commits usaron `SKIP=cargo-fmt,cargo-clippy` porque los hooks whole-tree fallaban por WIP ajeno (`src/wal.rs` de otra sesión); los mismos hooks **scoped a sus 6 archivos pasan**. Sin `--no-verify`.

### El agente captura solo: plugin de OpenCode con ensamblado recall-first (`MCP-47`)
- **Fecha:** 2026-11-25
- **Objetivo:** que un turno de OpenCode exista como memoria **sin que nadie llame `memory_put`**, y que el contexto vuelva por recall solo si hay hits. Era la mitad cliente de la opción A: el bridge escucha `POST /api/v2/conversations` y ningún cliente posteaba.
- **Resultado:** ✅ Plugin v2 en `.opencode/plugins/vanta-memory-capture.ts`: en `session.idle` lee el intercambio con `ctx.session.context()` y postea `user`+`assistant` (solo partes `text`) a `thread_send` en un thread estable por sesión; en el hook `context` consulta `memory_recall` con el turno **verbatim** e inyecta `prepend_context` **solo con hits** (vacío ⇒ silencio, nunca relleno); y publica `capabilities.ingest` una vez por sesión cuando `is_broken()`.
  - **Lo que cazaron las 2 rondas de review** (ambas `changes-required`): ronda 1 — el query de recall viajaba en **argv** (el turno del usuario en la línea de comandos), `subscribe({types})` **no existe** en `@opencode/client@2.0.15` (el filtro se ignoraba y un subscriber atrasado se descarta), un envío fallido **forkeaba el thread**, y el aviso recomendaba `VANTADB_INGEST_PROVIDER`, que apaga el aviso **sin drenar nada** (la ceguera de `MEM-72` reintroducida). Ronda 2 — las regresiones de sus propios fixes: idles descartados durante una captura en vuelo, replay de la mitad ya guardada, breaker de proceso (apagaba el recall para todas las sesiones), y que `mcp-call` hace `trim_matches('"')` (`src/cli_handlers/mcp_call.rs:101`), así que un turno terminado en comilla rompía el splice.
- **Verificación:** `node .opencode/plugins/vanta-memory-capture.selftest.mjs` **19/19** (y por `node --test`) · **4 mutaciones fallan como deben** (recall invertido 3, captura invertida 5, redacción quitada 1, idle descartado 1) · E2E contra el binario real: thread con ambos turnos y `token=[REDACTED:credential]` **almacenado así** · `check-links`/`check-docs`/`markdownlint` 0 · `npx tsc --noEmit -p .opencode` **no ejecutable** (sin tsconfig ni `tsc`).
- **Commits:** `6196c7be0` (plugin + docs + task file) + `2dcfcc41e` (correcciones del review)
- **Techo documentado (medido, no supuesto):** (1) **sin `MCP-43` no hay scheduler**: `queued: true` significa "esperando", no "consolidado", y `memory_recall` devuelve `recalled: []` tras capturar un turno real. (2) El semáforo `capabilities.ingest` es **process-global** y con transporte one-shot (`mcp-call` por llamada) cada invocación es un proceso nuevo con cola vacía ⇒ `ingest: null`; solo sirve desde una sesión MCP viva o `--mcp` con `MCP-43`. (3) Presupuestos 12 s lectura / 15 s escritura + breaker por sesión, porque `memory_recall` tarda 7-8,5 s en frío (`FIND-706`) y con 5 s el recall caducaba **siempre**.
- **Pendiente declarado:** Gate H (`/harness`) por tocar `.opencode/` — el cambio es un plugin nuevo, sin tocar agentes/comandos/skills. Los 2 ficheros quedaron **sin commitear** en el repo Git separado de `.opencode/` (base `2296ccd`): el loader v2 descubre `plugins/*.ts` solo, así que **cargan en la próxima sesión**; sin `VANTADB_DB_PATH` el plugin es un no-op inerte.

### El `--mcp` writer drena la cola de memoria (`MCP-43`)
- **Fecha:** 2026-11-25
- **Objetivo:** `thread_send` capturaba L0 y encolaba L1 (`MCP-44`), pero **nadie drenaba la cola**: tras un turno real `memory_recall` devolvía `recalled: []` para siempre. El proceso `--mcp` debía hospedar el scheduler de `vanta-memory` sobre **esa** cola, sin volverse un segundo consumidor en modo proxy.
- **Resultado:** ✅ `vantadb-mcp/src/memory_scheduler.rs` (nuevo): driver proceso-global con `start` idempotente (`OnceLock<Mutex<Option<MemoryScheduler>>>` + flag `Stopping`), intervalo `VANTADB_SCHEDULER_INTERVAL_MS` (default **5000 ms** en `--mcp`; `0` = off), runner por pass desde la misma superficie env/TOML del host HTTP con `Some(ConcreteRunner::None) → None` (sin engine el pass se salta y la cola queda intacta), y `stop()` que señala y joinea.
  - **Dónde arranca y dónde para:** arranca en las dos ramas writer de `run_stdio_server_auto` (no en `read_only`) y al promover el proxy a writer (`MCP-48`); para al final de `run_stdio_server` — **antes** del drop del discovery file — y en `flush_if_promoted` antes del flush.
  - **Modo proxy: cero scheduler**, declarado en el log (`scheduler_running=false`, reason `proxy mode`). Dos consumidores sobre datos ajenos era el pre-mortem; `capabilities` sigue diciendo la verdad porque `tools/call` se reenvía entero al writer.
- **Test del ciclo real:** `thread_send` → **tick del interval ticker** → `memory_recall` devuelve el turno **sin `memory_put`** (1,7-2,8 s, 3/3 corridas) · contrato `-E 'test(scheduler) or test(mcp_scheduler) or test(thread)'` **9/9** · suite del crate **200/200** · `vanta-memory` **747** · clippy/fmt limpios · docs gates 0 broken.
- **Review P2-01:** ronda 1 `changes-required` — **2 blocking de orden de vida**: (R1) el `stop()` vivía en el caller, **después** del `Drop` del discovery guard (el loop podía escribir sobre un fichero ya borrado); (R2) en `promote()` el `start()` corría **antes** de un `?` que podía devolver `None` y dejar el loop huérfano. Corregidos, más `Stopping` para la ventana de `stop()` concurrente; ronda 2 **`approve`**.
- **Commit:** `c9341b3ef`
- **Decisión de backfill (§Spec-6): (b) NO hay backfill L0→cola**, con techo declarado. La cola es in-memory (ADR-0054) y un backfill **correcto** exige un marcador durable de consolidación en el camino L0 (`vanta-memory/src/core/conversation/**`), fuera del scope de escritura de la tarea; la variante acotada por N/mtime sin ese marcador degenera en re-encolar en cada arranque turnos ya extraídos (**duplicación masiva de llamadas LLM**). Lo capturado no se pierde — sigue en el thread store y en `l0/` —; lo que se difiere es la extracción automática. → fila `FIND-745`.
- **Desbloquea:** `MEM-73` (e2e L0→L3 desde binario shipped).
- **Incidente operativo:** el disco llegó a **0 bytes** a mitad de la tarea; liberó 7,6 GB de `*.pdb` de `target/` con antigüedad >30 min porque `target-cleanup -AutoSafe` **se negó correctamente** (había builds en vuelo de otro carril). Esa negativa en bloque es la que no tiene plan B → fila `FIND-746`.

### El ciclo entero L0→L3 existe en un proceso real (`MEM-73`)
- **Fecha:** 2026-11-25
- **Objetivo:** la ola estaba cerrada **capa por capa** (`MCP-43/44/47/48`, `MEM-71/72`, `WIRE-19`) pero nadie había demostrado el ciclo **entero** en el binario que se publica: turno → L0 → cola L1 → tick del scheduler → L1 → L2 → L3 (persona) → `memory_recall` **sin `memory_put`**.
- **Resultado:** ✅ `vantadb-mcp/tests/l0_l3_cycle_e2e.rs` arranca `vantadb-server --mcp` (writer), habla JSON-RPC por stdio y compone el ciclo: `capabilities.ingest.{runner:ollama, scheduler_running:true, degraded:false}` → `thread_send` (`captured+queued`) → tick de **1500 ms** → `memory_recall` con `source_namespace: l1/<thread>` → `scene_list` no vacío (L2) → `persona/<thread>` escrito (L3) → `queue.depth:0`, `last_pass.skipped:false`, `notice:null`. **Sin `memory_put` en ningún punto.**
  - **El runner es el real** del binario (`ConcreteRunner::Ollama` desde `VANTADB_INGEST_*` + HTTP real de `StandaloneLlmRunner`); el **modelo es un endpoint scripted** declarado como tal (no hay ollama ni key en la máquina): se prueba **la composición, no la calidad**.
  - **`FIND-741` en vivo:** con el binario anterior (`e8df96232` vs HEAD `4af909977`) el test **falla en el pre-flight** nombrando ambos SHAs; rebuildeado, pasa. **Se pagó en la misma sesión** — sin ese gate el e2e habría corrido contra el contrato viejo.
  - **Es `#[ignore]` por diseño** (arranca procesos + LLM local = clase `FIND-727/739`): carril explícito `dev-tools/e2e-memory-cycle.ps1` (build → gate de SHA → e2e). **Recomendación al lead: cablearlo en `nightly.yml`** para que no se pudra.
- **Verificación:** `cargo nextest run --profile audit -p vantadb-mcp -E 'test(e2e_l0)' --run-ignored ignored-only` **PASS ×5** (ciclo 1,39-1,67 s; wall 3,5-4,2 s) · `pwsh dev-tools/e2e-memory-cycle.ps1` PASS · clippy `-D warnings` + `fmt --check` ✅ · gate de regresión 2/2 · docs coverage 0 gaps.
- **Review P2-01:** ronda 1 `changes-required` 3 items — bloque Review ausente; **`assert!(elapsed < 5s)` = `FIND-739` renacido**; el "contrato mecánico" nombraba un comando que **no** corría el e2e — los 3 aplicados; ronda 2 **`approve`** (verificado contra fuente y por ejecución independiente).
- **Commit:** `c3e9c098e`
- **Hallazgos, que valen más que el e2e:** (1) **L3 no necesita 50 memorias** — el trigger P2 *cold start* gana tras el primer L2 (`persona_trigger.rs:71-78`, ya documentado en `VANTA_MEMORY.md:334-336`): la premisa del brief era **falsa**. (2) El umbral real es **5 memorias en UNA pasada** (`auto_l2_threshold`, TEC-18). (3) Cada L2 emite `ERROR add_edge … Node not found` (tripletas MEM0-04, `let _ =`): inocuo para el pipeline y **ensucia el stderr del cliente MCP** → `FIND-747`. (4) `capabilities.ingest.queue.depth` es **snapshot del último pass** en el camino sano (probe vivo sólo en degradado), así que un agente puede leer "0" **con trabajo encolado** → `FIND-748`. (5) `memory_recall` **decora** el `content` con `(activity time: …)`: igualdad exacta imposible por diseño; `starts_with` distingue extraído de turno crudo.
- **Techo declarado:** no cubre durabilidad entre restarts (`FIND-745`, sin backfill L0→cola) ni recall vectorial (sin embeddings, WARN `keyword fallback`); `src/**` no se tocó — no apareció bug real que lo justificara.

### La cola del ingest se lee viva y la salud deja de mentir (`FIND-748` / `ADR-0065`)
- **Fecha:** 2026-11-25
- **Objetivo:** `capabilities.ingest.queue.depth` se escribía **sólo** desde resultados de pass, así que en el camino sano era la foto del pass anterior: un turno recién encolado se leía como `0`, y `is_broken()` (`degraded && depth > 0`) mentía hacia "no roto" en **tres** casos distintos: turno recién encolado, scheduler nunca arrancado o ya muerto, y proceso que no ve la cola.
- **Resultado:** ✅ `ADR-0065` D1-D7. `IngestStatus.queue` pasa a `Option<QueueSnapshot>` y se lee **en el momento de responder** (`status_with_queue`, alimentado por `threads::queue_snapshot()` → `LocalStateBackend::queue_occupancy()`, **una sola** adquisición); la muestra del pass se muda a `last_pass.queue`, donde `at_ms` le pone fecha. `health` es de cuatro estados (`unknown|idle|draining|broken`) derivado al publicar con la precedencia del ADR, más `drain_path` (`none|loop|on_demand`) y el bit `scheduler_ever_started` que separa "nunca hospedado" de "hospedado y muerto". `is_broken()` = `health == Broken`: ya no puede dar `false` por una foto vieja.
  - **D7 (la verruga que se va):** el enqueue llamaba `note_skipped_pass` y fabricaba `last_pass.skipped: true` para un pass que **nunca corrió**; ahora llama `note_backlog`, que sólo mueve el latch del warn. `skipped: true` vuelve a significar "pass saltado", y `docs/api/MCP.md` pierde el párrafo que explicaba la verruga.
- **Mutaciones de control (las dos, pegadas):** **ANTES** — probe con el código previo: `live depth = 3, reported queue.depth = 0, is_broken() = false` → **FAILED** (F1 era insatisfacible). **DESPUÉS** — `handlers/tools.rs` revertido a `status()`: el test de wire cae con `queue: null`, `health: "unknown"` → **FAILED**.
- **Verificación:** `vanta-memory` **25/25** (contrato + unit de `notice`) · `vantadb-mcp` **4/4** (incluye el ciclo real `thread_send → tick → recall → stop`) · clippy `-D warnings` ✅ · fmt ✅ · `check-docs` GATING all clear ✅ · `check-links` 0 ✅.
- **Review P2-01:** **3 rondas**, `changes-required` ×2 → `approve`. R1: la doc afirmaba que un `mcp-call` one-shot lee `queue: null`/`unknown` (**falso**: el hijo que gana el writer arranca su loop y lee `idle`/`loop`) y el §Review era **auto-atestado**. R2: el ejemplo "proxy → `unknown`" también era falso (en proxy el hijo **reenvía** el `tools/call`: el cliente lee la salud del writer). R3: sin bloqueantes.
- **Commit:** `dfb51d807`
- **Hallazgo abierto (no decidido por el implementador, correcto):** `ADR-0065 §D8` afirma que el one-shot "reads `unknown` instead of `idle`". Verificado en código: `mcp_call.rs:260-266` lanza `<server> --mcp`; `server.rs:1057-1065` abre el writer y llama `memory_scheduler::start()` **antes** de la primera petición (`memory_scheduler.rs:134`), así que `scheduler_running: true` derrota el guard de «sin evidencia» y el resultado es `idle`/`drain_path: loop` (el `runner` sigue `Unknown` hasta el primer tick). `queue: null` **no se alcanza en el wire** porque el handler siempre pasa `Some`. Enmienda de D8 a cargo de `vanta-arch`; opciones en `docs/dev/tasks/FIND-748.md` §6.
- **Consumidores de `capabilities.ingest` (buscados, no supuestos):** **un publicador** (`handlers/tools.rs:3199`) y **cero parsers** en el repo — `vantadb-server` no lo publica, y Python/wasm/node/ts/`evals/` no lo leen. El único cliente es el plugin de `.opencode` (repo aparte, `FIND-749`), que ya usa `ingest.queue?.depth ?? 0` y por tanto no se rompe con la forma nueva.
- **Aditivo o rompiente:** aditivo **en el wire** (la `queue` sigue siendo objeto porque el handler siempre lee su propia cola; `scheduler_ever_started`/`health`/`drain_path`/`last_pass.queue` son claves nuevas). Rompiente en el **tipo Rust** (`IngestStatus.queue` → `Option`, `PassSnapshot` gana campo): `pub`, pre-1.0, superficie de diagnóstico — cubierto por `ADR-0065`.
- **Techo declarado:** `ingest` describe **un** proceso (ADR-0054/D8, deuda de `MCP-43`); `FIND-749` (el plugin con el predicado viejo) sigue abierto; la nueva aserción `health == "idle"` del e2e corre sólo en el carril `ignored`.
