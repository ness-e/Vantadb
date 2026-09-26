---
title: "Vanta Memory Engine — API Reference (`vanta-memory`)"
type: api
status: active
tags: [vantadb, api, vanta-memory, memory-engine]
last_reviewed: 2026-09-26
aliases: []
related: []
---

# Vanta Memory Engine — API Reference (`vanta-memory`)

> **Estado:** ✅ documentación canónica del crate (cierra la cita de ADR-029 §Nota mecánica).
> **Nota:** `scripts/validate-docs-coverage.ps1` hoy NO escanea `vanta-memory`; esta página es
> la referencia manual. Las superficies F1-F3 (search profile, entity_*, skills) viven en
> `EMBEDDED_SDK.md`.

Crate LLM-driven para memoria de agentes: captura L0, extracción/dedup L1, escenas L2,
persona L3, recall con scope, context engine (compresión), offload y generación wiki.
**Principio rector:** el LLM es opcional (P4) — todo flujo degrada sin perder datos cuando
el runner falla o no está configurado.

## Scope & stability (Gate P — 2026-09-24)

- **Core-only by decision** (`API-STD-15`, Gate P): `vanta-memory` is an internal workspace
  member (`publish = false`) consumed in-process by `vantadb-mcp`, `vanta-proxy` and
  `desktop/src-tauri`. It is **not exposed by any binding** (Python/TS/Node/WASM) and is not
  part of the published versioning contract; exposing it requires new Rust bindings plus
  demonstrated demand (post-release, D42/D43).
- **Stable Rust API:** the public surface documented here is the stable contract for in-repo
  consumers. Binding re-export: none — `0` symbols by design (checked with
  `rg "vanta[_-]memory" vantadb-python vantadb-ts vantadb-node vantadb-wasm` → 0 matches).

## Feature flags

All optional; the default build stays lean and LLM-free.

| Feature | Enables | Default |
|---|---|---|
| `llm-driver` | Real HTTP transport for `StandaloneLlmRunner` + `AsyncLlmRunner` (reqwest) | off |
| `embeddings` | `core_embedding_hook` (wraps the core `EmbeddingProvider`, `remote-inference`) | off |
| `embed-local` | Local ONNX provider (`LocalOnnxProvider`) + `local_embedding_hook()` auto-on in `L1DedupConfig::default` | off |
| `precise-tokens` | Exact cl100k_base BPE counts (tiktoken-rs) instead of `chars/3` (D21 amendment) | off |
| `mock` | Deterministic `MockLlmRunner` for tests | off |
| `fjall` | Persistent backend for the `vanta-seed` binary | off |
| `http-server` | Core HTTP server bridge (`/conversation/add` hook) | off |

## Degradation contract (Principio 4)

With `llm-driver` off (default) the runner is a placeholder and every LLM-dependent path
degrades to its LLM-free equivalent — **never blocks, never loses data**:

| Layer | Degraded behavior |
|---|---|
| L1 extraction | `success: false`; L0 turns stay untouched (store-all path) |
| L1 dedup | heuristic keyword overlap; unmatched candidates are stored |
| L2 scenes / L3 persona | generation reported failed; prior state untouched |
| Recall | keyword-overlap fallback; `effective_mode` reports it |
| Context engine | LLM-free by construction (compression/MMD/injection) |
| Wiki ingest | new pages verbatim; required merges recorded as skipped |

Pinned by `src/adapters/standalone/llm_runner.rs::llm_free_mode_reports_not_configured`
(`:237-250` — cited as `:209-218` before WIRE-11 shifted the lines) plus the per-layer degrade
tests (`tests/l1_extractor.rs`, `tests/l1_dedup.rs`, `tests/scene_strategy.rs`,
`tests/persona.rs`, `tests/ingest.rs`, `tests/generation_log.rs`). The inverse contract holds
with `llm-driver` on: failures are loud, never a silent `NotConfigured`
(`llm_driver_fails_loud_on_unreachable_endpoint`, WIRE-11).

## Arquitectura por capas

| Capa | Módulo | Qué hace |
|---|---|---|
| L0 | `core::conversation::l0_recorder`, `core::hooks::auto_capture` | Captura idempotente de turnos (cursor `l0_cursor/<session>`) |
| L1 | `core::record::{l1_extractor,l1_dedup,l1_reader,l1_writer}` | Extracción 1-call LLM JSON con parse reparado; dedup 2 fases store/update/merge/skip |
| L2 | `core::scene::{scene_index,scene_format,scene_extractor,scene_tools}` | Escenas con META {created,updated,summary,heat}, strategy UPDATE>MERGE>CREATE, soft-delete, tools sandboxed |
| L3 | `core::persona::{persona_generator,persona_trigger}` | Persona first/incremental con triggers P1-P4 y escape XML |
| Recall | `core::hooks::auto_recall`, `core::memory_prompt::*`, `core::profile::profile_sync` | Prepend/append + 3 modos (`RecallScope::Session\|Agent\|Team`, default Agent) |
| Context | `context_engine::{engine,compressor,mmd,mmd_injector,token_estimator,types}` | Compresión LLM-free mild/aggressive/emergency + MMD persistente + budget coordinator |
| Offload | `offload::{state_manager,storage,reclaimer,hooks::after_tool_call}` | Cursor `lastOffloadedToolCallId`, entradas por tool_call_id, GC por retention |
| Ingest | `ingest::{worker,merge,prompts,callback}` | Ingest wiki serial (fallo por página no bloquea), progreso canal interno + polling run_id |
| Skills | `core::skill::skill_extractor` + `conversation_add` | Extracción desde transcript con marcadores anti role-capture; sink idempotente doble cursor+content-hash |
| Orquestación | `services::pipeline_worker`, `utils::{pipeline_manager,stateful_pipeline_manager,managed_timer,checkpoint}` | Timers/locks estado local, trait `Clock` inyectable (FakeClock determinista), worker L0→L1→L2→L3 |
| Gateway | `gateway::knowledge_handlers` | Handlers tipados scene_read/list/query para exposición MCP/server |

## Contratos clave

### Trait `LlmRunner` (host-neutral, sync)
```rust
pub trait LlmRunner {
    fn run(&self, params: &LlmRunParams) -> Result<String, LlmError>;
    // complete_json<T>: helper genérico — NO dyn-compatible; usar <R: LlmRunner>
}
```
Los extractores/generadores son funciones genéricas `<R: LlmRunner>`; el fallo del runner
degrada a `success: false` / skip documentado — jamás bloquea ni corrompe estado previo.

### Context engine
```rust
assemble(messages, budget, estimator, protected_prefix, cfg) -> AssembleOutput
assemble_with_recall(...)  // coordinator único: assemble → inject_mmd → recall, un solo budget
```
- Ratio < 0.5 → skip sin tocar mensajes.
- Mild cascade (MIN=10/INITIAL=7/FLOOR=1) → aggressive one-shot (fingerprint boundary
  `role + primeros 200 chars`, idempotente) → emergency prefix-aware (~2000 chars).
- Los pares tool_call/tool_result son unidades atómicas: nunca se parten.
- Mensajes ≤ cursor `lastOffloadedToolCallId` (MEM-20) van en `protected_prefix`.
- `inject_mmd` agrega `<current_task_context>` tras el prefijo System con dedup fingerprint.

### Recall
```rust
RecallConfig { scope: RecallScope, .. }   // Session | Agent | Team — default Agent
perform_auto_recall(db, params) -> RecallResult { prepend_context, append_system_context, .. }
```
Prepend = dynamic per-turn memories; Append = stable persona/scenes (prompt-cache friendly).
Embedding/Hybrid actually run when an embedding hook is attached **and** the pool carries
vectors (MEM-47 dual-pool: cosine ranking + keyword gate, fused with RRF via `rrf_merge`) —
a legacy record is never dropped *just because* it lacks a vector; it still ranks through the
keyword-overlap gate (`min_overlap`). With `embed-local` compiled,
`L1DedupConfig::default` wires `local_embedding_hook()` automatically (MEM-63); without a
provider both modes degrade to keyword-overlap (`RecallMode::effective`,
`core/hooks/auto_recall.rs:87-98`).

### Wiki ingest (F7)
```rust
worker::run(store, sources_root, runner_opt, cfg) -> IngestResult
worker::run_with_progress(..., Option<&ProgressTracker>)   // throttle 500ms
progress_tracker.wiki_status(run_id) -> Option<IngestProgress>
```
Serial por página; fallo de página no bloquea las siguientes; STRUCTURAL_FILES protegidos;
`ensure_sources` fuerza frontmatter. Fallback P4 sin runner: páginas nuevas verbatim,
merges requeridos se registran como skipped.

## Namespaces (sanitizados `[A-Za-z0-9._/-]` ≤128B)

| Namespace | Contenido |
|---|---|
| `l0/<session>` · `l0_cursor/<session>` | turnos crudos · cursor de captura |
| `l1/<session>` | memories extraídas (+ dedup state) |
| `scene/<session>` · `mmd/<session>/{active,history}` | escenas · memoria de tarea |
| `persona/<session>` · `profile/{scope}` | persona · perfil sincronizado |
| `offload/<session>` · `offload_state/<session>` | entradas offload · cursor |
| `pipeline_checkpoint` | contadores del orquestador |
| `genlog/<session>` | provenance de generaciones (best-effort, cap 100) |
| skills_extract/<scope> | seed/import CLI |

## CLI

- `vanta-seed <seed.json> [--db <path>]` — import inicial de skills/persona, idempotente por
  content-hash.

## Debts — resolved & deferred (upgrade paths)

### Resolved (API-08 stabilization, 2026-09-26)

1. **D37 — embeddings-based recall/dedup**: paid by MEM-46 (`e22b496a`), MEM-47 (`f32e4d51`)
   and MEM-63 (`6058cc84`). Dual-pool ranking (cosine + keyword, RRF) is live and
   `embed-local` auto-wires the local provider. *Residual (founded DEFER):* without a
   provider attached the pipeline stays keyword-only by design (P4) — trigger to revisit:
   a host attaches a provider / hosted-embeddings demand.
2. **D21 — `TokenEstimator`**: paid by the ADR-029 amendment + BND-03 (`784b27b9`).
   `precise-tokens` gives exact cl100k counts (golden tests pinned); the `chars/3` default is
   an accepted ±20% approximation (CJK ~2×). *Trigger:* drift >15% or CJK default-precision
   demand.
3. **MEM-16 — context engine ↔ pipeline worker**: paid by MEM-43 (`a0bcb112`) — the worker
   runs `assemble_with_recall` post-L3 under one shared budget
   (`services/pipeline_worker.rs::run_context_assembly`; e2e
   `d19_worker_assembles_context_post_l3_with_compression_active`). No residual.
4. **Heuristic compression scoring**: partially paid by MEM-48 (`4fbaa4a3`) — messages linked
   to persisted L1 memories are scored from their real max priority (`MemoryScoreMap`), wired
   in production by `run_context_assembly`. *Residual (founded DEFER):* messages without
   linked memories (and the module's upstream ceiling) keep the deterministic heuristic —
   upgrade path: consume persisted offload-entry scores (`context_engine/compressor.rs:7-9`);
   trigger: those scores land in the store.

### Open (trigger-gated DEFER)

5. **Fetcher HTTPS/git** — deferred (D30/D36); implement with a non-disableable SSRF
   blocklist when remote sources land.

See ADR-029 for the full D21–D23 rationale and `API-STD-15` for the Gate P decision
(core-only + stable Rust API).
