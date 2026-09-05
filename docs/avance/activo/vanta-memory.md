---
title: "Avance — Vanta Memory"
type: domain-log
status: active
tags: [vantadb, avance, vanta-memory, tdam, memory, persona, recall]
last_reviewed: 2026-08-22
aliases: []
---

# Avance — Vanta Memory

> Registro consolidado del trabajo completado sobre el crate `vanta-memory/`: pipeline TDAM (L0 capture → L1 extract/dedup → L2 escenas → L3 persona → recall → offload → gateway), skills/tools MCP, seeds CLI, embeddings y schedulers. **IDs originales conservados.** Catch-up por campaña (no commit-por-commit).

## Cobertura rápida

- **P27 (F1-F4):** port TDAM completo — search profile IQL, entidades/RBAC, auth 3 capas, skills multi-versión, crate `vanta-memory` end-to-end con trait host-neutral `LlmRunner`.
- **P29 (F5):** superficie de memoria para el context engine — seeds/import CLI, generation-log, recall_scope híbrido, auto-sync scheduler, GC offload, ADR-029.
- **P31 (Cierre Final):** wiring productivo al pipeline, e2e cross-crate MCP, embeddings semánticos opt-in, recall dual-pool RRF, compresión con scores reales.

---

## Campaña P27 — Vanta Memory Engine (F1-F4)

### MEM-01..21 (+34, 35): Port TDAM completo — catch-up por campaña
- **Fecha:** 2026-08-18 → 2026-08-20
- **Objetivo:** Crate nuevo `vanta-memory/` con el pipeline completo: L0 capture idempotente LLM-free (`9c0dd213`) → L1 extractor + dedup 2 fases (`91c9068e`, `7356aa7d`) → L2 escenas (contrato META + nodo escena, `a6526f70`, `7356aa7d`… `c6c06c75` tools sandboxed) → L3 persona first/incremental con triggers (`5fc0cb11`) → recall 3 modos prepend/append (`fb1d2dd4`) → offload cursor persistente por sesión (`9a9fea41`) → gateway con tools MCP scene_read/list/query (`31e676b1`). Orquestación timers+locks (`2634e9bd`), skill extract transcript (`31e24f88`), sanitize/truncación code-point (`42940f6d`), métricas snapshot por capa + audit (`84f28a18`), search profile IQL `PROFILE` (`6a50b8ee`, `32b09daf`), entidades entity_* + RBAC allow-only (`23719e23`, `9717bf03`), auth 3 capas L1/L2/L3 (`01a5de66`), skills multi-versión con optimistic lock (`92cf709f`, `4763bf44`), data plane REST `/conversation/add` + `/skill/listing` (`9693d0ff`), contracts F4 + `LlmRunner` host-neutral con degradación LLM-free (`76a73969`).
- **Resultado:** ✅ 24/24 tareas. Suite final 361/361 tests en vanta-memory; fmt/clippy `-D warnings` limpios. E2E L0→L1→L2→L3→recall (`5e462792`). Plan archivado: `docs/plans/archive/` (cierre `30198d5e`).
- **Ids:** `MEM-01`, `MEM-02`, `MEM-03`, `MEM-04`, `MEM-05`, `MEM-06`, `MEM-07`, `MEM-08a/b`, `MEM-09`, `MEM-10`, `MEM-11`, `MEM-12`, `MEM-13`, `MEM-14`, `MEM-15`, `MEM-16`, `MEM-17`, `MEM-18`, `MEM-19`, `MEM-20`, `MEM-21`, `MEM-34`, `MEM-35`
- **Cruce:** entrada espejo en `core-engine.md` (planner/core) y `bindings.md` (MEM-21 handlers MCP).

---

## Campaña P29 — Vanta Context Engine (superficie de memoria, F5)

### MEM-38..42 (+ADR-029): Superficie F5 sobre vanta-memory — catch-up por campaña
- **Fecha:** 2026-08-20 → 2026-08-21
- **Objetivo:** Preparar la memoria como fuente del context engine: seed/import CLI vía bin propio `src/bin/vanta-seed.rs` con idempotencia content-hash (`d3eba4fc`, `MEM-39`), generation-log provenance best-effort L1/L2/L3 bajo `genlog/<session>` cap 100 (`1f89c0b6`, `MEM-41`), `recall_scope` híbrido session|agent|team default agent + primer test `search_multi` (`89777704`, `MEM-40`), auto-sync scheduler con ManagedTimer pull-based + busy guard (`2dba254f`, `MEM-45`*), reclaimer GC offload con retention_days post-cursor estricto e idempotente (`214a7820`, `MEM-42`), ADR-029 borrador + superficies F5 documentadas en EMBEDDED_SDK (`badb5b9c`, `MEM-38`).
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
  - **MEM-49** (`437bfee3`): guía socrática de revisión ADR-029 + decisiones D21-D37 (prep articulación humana, Regla 5).
- **Resultado:** ✅ 8/8 tareas de campaña. Auditoría final con hallazgos registrados en `docs/api/VANTA_MEMORY.md` canónico (`673f18af`). ADR-029 ACEPTADO con articulación humana completa (`9e76caff`). Plan cerrado (`460ce60a`).
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
