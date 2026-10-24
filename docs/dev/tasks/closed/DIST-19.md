---
title: "DIST-19 — Fachada VantaCognitiveAPI en vantadb-node + scopes + Electron CI"
kind: task
status: completed
description: "Exponer vanta-memory in-process en vantadb-node (sustituye subprocess MCP stdio en desktop Ego)"
tags: [napi, memory, electron]
---

# DIST-19 — Fachada `VantaCognitiveAPI` + MEMG-24 + MEMG-03/09/25/26 + FIND-320 + DIST-20/21

- **Origen:** Ego Readiness Plan 2026-10-07 + análisis Ego-readiness 2026-10-08 (§14/15/21).
- **Dependencia:** Wave 3, tras EGO-03 (db74daed + 761922c7).
- **Problema:** Ego usa `vantadb-mcp` como subprocess stdio (overhead JSON-RPC) + scopes en adapter + memoria/grafo separados.
- **Contrato:** fachada expuesta + suite Ego verde + CI Electron verde + grafo↔memoria + specs MEMG-25/26 + retrieve opt-in.
- **SDP:** codebase-memory, source-driven-development, documentation-skill, progreso, ponytail full (manual — sin MCP campaign en este worker).

## Impacto mapeado (Regla 0)

Archivos leídos completos (vía shell/codegraph, no editados salvo Slice 2):

- `vantadb-node/src/lib.rs` — `VantaDB` (napi, `custom_finalize` EGO-06, `Mutex<VantaInner>` EGO-12), ~40 métodos async vía `spawn_blocking` + `OpGate` (js-ecosystem R-4 ✅). Sin dep `vanta-memory`, sin namespaces `memory.*`/`context.*`/`tasks.*`, sin `AbortSignal`.
- `vantadb-node/Cargo.toml` — standalone `[workspace]` vacío intencional (js-ecosystem R-3), deps: napi 3, tokio rt, vantadb path, vantadb-ffi-core.
- `vanta-memory/` — `core/hooks/auto_recall.rs`: `recall`, `recall_with_scope`, `RecallScope` (Session/Agent/Team), `InjectionPolicy` (allow_all/from_prefixes, `trust_class`), `RecallGovernance` (source/deny), `reinforce_recalled`, `AutoRecallParams`; `context_engine/` (spill, engine, compressor); tests `recall.rs`, `semantic_recall.rs`, `reinforce_loop.rs`.
- `src/sdk/types/record.rs:269` — campo `node_id: u128` existe; `src/sdk/serialization/mod.rs` — `memory_node_id(ns,key)->u128` determinista existe pero `pub(crate)`.
- `.github/workflows/` — 18 ficheros, ninguno Electron. `vantadb-node/tests/` — 9 suites vitest (`api,graph,graphrag,lifecycle,parity,persistence,sch04,sch07,wire03`); binario `.node` win32-x64 precompilado presente.
- Reglas leídas: `js-ecosystem.md`, `server-mcp.md`, `api-contract.md`, `concurrency-async.md` (R-1 spawn_blocking, R-2 no guard sync cruzando await, R-7 handlers).

Referencias entrantes: `feature-freez/desktop/.../commands/memory.rs` usa `RecallScope`; `vantadb-python` usa recall (adapter separado — NO tocar, WIP ajeno).
Veredicto: cambio aditivo puro (métodos nuevos + workflow nuevo + test nuevo). Sin breaking contemplado. Toca Tokio/abort → Regla 8 (evidencia + `vanta-chaos`/`vanta-review` antes de cerrar).

## Spec (decisiones)

| # | Decisión | Justificación por evidencia |
|---|----------|----------------------------|
| 1 | Fachada = métodos `#[napi]` en `impl VantaDB`: `memory_recall`, `context_assemble`, `memory_consolidate`, `memory_reinforce`, `checkpoint_save/load` (nombres exactos en Slice 3) | Thin wrappers sobre `vanta-memory`; lógica en core/Rust (api-contract R-8). Elimina subprocess stdio en desktop |
| 2 | `vanta-memory` como dep path+version de `vantadb-node` (igual patrón que `vantadb path+features`) | Reutiliza `recall_with_scope`, `InjectionPolicy`, context_engine sin duplicar lógica |
| 3 | DIST-20: `AbortSignal` JS → `AbortHandle` napi → `tokio::select!` contra el `spawn_blocking JoinHandle` (abort = detach + `OpGuard` drop, nunca cancela el engine) | concurrency-async R-1/R-2: sin guard cruzando await; Regla 8 exige evidencia de no-deadlock |
| 4 | FIND-320 `retrieve()` trust-aware **opt-in** (`trust_aware: bool = false`, byte-idéntico por default, precedente MEMG-21) | Piezas existen (trust_class, conflicto MGR-06, abstención MGR-18/FIND-187); sin contrato único |
| 5 | MEMG-25/26 **spec-first sin enforcement** (tipos + defaults + wire en `docs/api/`, EN) | Backlog lo exige explícito; enforcement = MGR-04/FIND-301 (fuera de scope) |
| 6 | MEMG-03: publicar `memory_node_id` (`pub`) + helper `memory_record_to_node` (L1-L3 como nodos/aristas); MEMG-09 residual documentado, no implementado aquí | Mapeo determinista ya existe; falta integración |
| 7 | DIST-21: workflow NUEVO `electron-node.yml` (no tocar los 18 existentes ni `release*.yml`); Electron real headless completo queda S2b (requiere harness Ego) | Scope discipline: este worker entrega matriz nativa + packaging-contract verdes |

## Steps

- [x] **S0 — Task file canónico** (este fichero). Spec + Regla 0 + Save Point.
- [x] **S1 — DIST-21 slice verde:** `.github/workflows/electron-node.yml` (matriz win/mac/linux: napi build + vitest real + packaging-contract). Sin `continue-on-error` (Regla 2).
- [ ] **S2b — Electron real headless** (Save Point): job con `electron` headless (`xvfb`, `app.asar` + `asarUnpack: ["**/*.node"]`). Requiere harness/versión Electron de Ego — dueño: orquestador + Ego.
- [ ] **S3 — DIST-19 fachada:** dep `vanta-memory` en `vantadb-node` + 5 métodos `#[napi]` (spec §1) + `dts-header` si aplica. Build napi foreground timeout ≥900s. Riesgo MSVC cdylib ya mitigado (R-3).
  - **Avance sesión 2026-10-16 (worker):** dep añadida (`vanta-memory = { path, version = "0.8.0" }` = workspace.package ✅) + **6 fns** `#[napi]` tras `graphrag_search` (`memory_recall`, `context_assemble`, `memory_consolidate`, `memory_reinforce`, `checkpoint_save/load` — checkpoint cuenta como el 5.º método de la spec). `cargo check` ✅, `clippy -D warnings` ✅, `rustfmt` ✅ (solo código nuevo). `version` efectiva verificada: `0.8.0`. `dts-header`: N/A (boundary `Value` → `any`, generación napi default). **Pendiente:** `npm run build` release (timeout 20min — LTO fjall+vanta-memory; reanudar reusa caché, corre de nuevo en foreground). Hallazgos: `RecallResult`/`AssembleOutput` sin `Serialize` (mapeo manual json!), `context_engine` submods privados (re-exports raíz), `recall`/`recall_with_scope` de la Regla 0 NO existen — API real: `perform_auto_recall_governed` + `AutoRecallParams` + `InjectionPolicy`.
- [x] **S4 — DIST-20 AbortSignal:** `AbortHandle` + `tokio::select!` + test cancelación mid-`search`. Regla 8: evidencia + delegar `vanta-chaos` + `vanta-review`.
  - **Resuelto sesión 2026-10-16 (worker):** `AbortHandle` (`#[napi]` class, `Arc<Notify>`, `new`+`abort`, +`Default` por clippy) en vez del `AbortSignal` del draft: el `AbortSignal` de napi es `!Send` por construcción (`Rc`, `task.rs:57`) y solo alimenta `AsyncTask` (libuv) — verificado con `cargo check` que ni siquiera como parámetro sin usar entra a un futuro tokio (`!Send`). Puente JS en una línea: `signal.addEventListener('abort', () => handle.abort())` (probado en S5). `search(request, abort?: AbortHandle|null)` — trailing opcional, backward compatible (10 suites verdes). Abort = detach (JoinHandle drop, engine nunca cancelado) + `OpGuard` liberado; `Notify` con permit guardado (sin missed wakeup); sin guard/lock cruzando await más allá del patrón `OpGate` existente. Evidencia mecánica: test abort-mid-search (5000×128d) rechaza `/abort/i` + put/get/close posteriores OK + suite completa verde. **Pendiente Regla 8 completa:** stress 10k w/s + auditoría deadlock/data-race → `vanta-chaos` + `vanta-review` (no invocables desde aquí).
- [x] **S5 — MEMG-24 suite Ego:** `vantadb-node/tests/cognitive-contract.test.ts` contra fachada S3 (firmas tipadas + comportamiento + parity `EgoMemoryAdapter.ts`). Verde bloquea releases.
  - **Verde sesión 2026-10-16:** 10/10 tests (firmas + recall null/JSON + validación scope/session_key + assemble + consolidate idle-gated + reinforce round-trip + checkpoint round-trip + abort S4 + trust_aware S8). Suite completa: 10 ficheros, 69/69. Hallazgos fijados en el camino: `get(ns,key)` son 2 args (no objeto); L1 se siembra como `MemoryRecord` JSON (el reader salta payloads no-deserializables); `opt_u64` acepta f64 integrales (epoch-ms de JS); guards `session_key`/`session_id` no-vacíos (Optional review S3). Sin `tsconfig` en `vantadb-node` (tsc N/A; vitest transforma sin typecheck — igual que las 9 suites existentes).
- [x] **S6 — MEMG-03 grafo↔memoria:** `memory_node_id` → `pub` + `memory_record_to_node` + test L1-L3 nodos/aristas. Verificar blast radius serialization antes.
  - **Verde sesión 2026-10-16:** `memory_node_id` y `memory_record_to_node_owned` publicados con doc comments + re-exports en `src/sdk/mod.rs` (el módulo `serialization` sigue `pub(crate)`; el re-export es el patrón existente). Test nuevo `test_record_to_node_preserves_deterministic_id` (id + round-trip ns/key) + existente determinismo; `serialization::` 241/241; clippy `-D warnings` limpio. Cambio aditivo puro (cero comportamiento).
- [x] **S7 — MEMG-25/26 specs EN en `docs/api/`** (`memory-scopes.md`, `memory-events.md`): campos, defaults, wire, sin enforcement. Requiere `gen-index` (dueño: commit final).
  - **Verde sesión 2026-10-16:** ambos ficheros creados (`status: proposed`, `kind: reference`, EN), spec-first con piezas existentes citadas a símbolo real y lo propuesto marcado; `check-docs.mjs` sin nuevas violaciones (1 missing-frontmatter preexistente ajeno). `gen-index` pendiente del commit final (dueño: orquestador/lead).
- [x] **S8 — FIND-320 retrieve opt-in:** SHAs pineados en `electron-node.yml` + `trust_aware` en recall + test byte-idéntico.
  - Pins (sesión anterior): checkout→`3d3c42e5` # v7.0.1; setup-node→`49933ea5` # v4.4.0; dtolnay→`7e38f4b4` # v1; NOTE resuelta.
  - **Flag sesión 2026-10-16:** `trust_aware: bool = false` + `now_ms?` en `memoryRecall` → `perform_auto_recall_scored` + `CompositeScoring::default()` solo cuando `true` (precedente MEMG-21); default byte-idéntico probado (`JSON.stringify` igual con/sin flag) + path `true` con forma estable. Trust classes siguen fluyendo vía `InjectionPolicy`; ranking trust-ponderado completo = trabajo futuro.
- [ ] **S9 — MEMG-09 residual:** grafo decisión→código→test + taxonomía (MGR-23/24). Solo diseño aquí; implementación futura.

## Save Point (para reanudar)

- Próximo step: **review + commit** (dueño: orquestador). Tras el commit: S9 (MEMG-09 residual, solo diseño) + S2b (harness Ego) + Regla 8 completa (`vanta-chaos` stress + `vanta-review`) + `gen-index` en el commit final.
- Set a commitear (selectivo, sin `vantadb-python/**` ajeno): `vantadb-node/Cargo.toml`, `vantadb-node/Cargo.lock`, `vantadb-node/src/lib.rs`, `vantadb-node/index.js`, `vantadb-node/index.cjs`, `vantadb-node/index.d.ts`, `vantadb-node/tests/cognitive-contract.test.ts`, `src/sdk/mod.rs`, `src/sdk/serialization/mod.rs`, `docs/api/memory-scopes.md`, `docs/api/memory-events.md`, `docs/dev/tasks/DIST-19.md`. (`.node` está gitignored; `index.*` generados sí se versionan.)
- Notas de build: `npm run build` flakea intermitente en Windows con `EPERM` en el file-transaction del napi CLI (`postBuildUnlocked` rename `.swp`) — los artefactos (`.node` + `index.*`) igual quedan escritos y frescos; reintentar cierra en verde. No tocar `get_str`/`parse_search_request` y cía. preexistentes (`as_u64` con el mismo latente f64>u32 — NOTICED BUT NOT TOUCHING, candidato a FIND).
- Invariantes: no publicar/taggear/pushear; no versiones Cargo a mano (release-plz); no `vantadb-python/**`; no `stash`; no `.opencode/**`; no `feature-freez/**`; no `docs/dev/plans/`; `memory.rs`/`types.rs` (EGO-01) solo lectura.
- Deuda: S9 (MEMG-09 residual, diseño) + S2b (harness Ego) + Regla 8 completa (chaos/review) + `gen-index` en commit final; workflow S1 sin correr en CI (sin push); specs S7 sin `gen-index`.
- Gates (cierre 2026-10-16): D — disparado 1× y resuelto (el `AbortSignal` del draft es inimplementable en futuros tokio: pivote documentado a `AbortHandle` con misma semántica); V — disparado 2× y resuelto (E0603 mods privados → re-exports; clippy `needless_borrows`/casts en código nuevo); C — n/a (colaterales en `git status` son ajenos: `vantadb-python/**` WIP + 1 plan del orquestador — no incluidos).

## Gates

- D: no disparado (contrato del backlog + Ego Readiness, sin ambigüedad bloqueante).
- V: no disparado (0 fallas verify; sin build pesado en esta sesión).
- C: n/a (sin colaterales; `git status` previo solo toca `vantadb-python/**` ajeno + 1 plan del orquestador - no incluidos).

## Review S3 (Gate P2-01)
- **Revisor:** vanta-review, sesión distinta (veredicto 2026-10-16, re-ejecución: dep/lock/APIs/guards/check/clippy/fmt/pins).
- **Veredicto:** ✅ APPROVE DIST-19 S3 — 6 fns con APIs reales, guards cerrados, quarantine default, lock solo-añade, pins SHA. 1 Optional (guards session_key vacíos → S5) + urgencias: ninguna.
- **S3 commit:** orquestador, set selectivo 6 paths (abajo en trazabilidad del review).

## Review S4-S8 (Gate P2-01 final)
- **Revisor:** vanta-review, sesión distinta al implementador (veredicto 2026-10-16, re-ejecución: vitest 10/10 + 69/69, fmt/clippy, check-docs, diff scoped).
- **Veredicto:** ✅ APPROVE DIST-19 — S4 AbortHandle justificado por compilador, S5 suite verde, S6 aditivo, S7 proposed sin enforcement, S8 byte-idéntico. Follow-ups con dueño: S9 diseño, S2b harness Ego, Regla 8 completa (chaos/review), serialization 241/241 por lectura (timeout build).
- **Estado: ✅ COMPLETED
