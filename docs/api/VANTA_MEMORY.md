---
title: "Vanta Memory Engine — API Reference (`vanta-memory`)"
type: api
status: active
tags: [vantadb, api, vanta-memory, memory-engine]
last_reviewed: 2026-09-27
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

## Facade — capture / recall / seed / ingest (candidate, not published)

> **Status: `candidate, not published`.** Conceptual signatures for the four
> host-facing operations in-repo consumers build against today. They summarize
> the stable Rust surface (§Scope & stability) as a reading aid — they are **not**
> a published contract, not part of the versioning surface, and not re-exported
> by any binding. Exposure starts only from a fired trigger (§Exposure triggers).

| Operation | Conceptual signature | Degradation (see §Degradation contract) |
|---|---|---|
| **capture** | `AutoCaptureHook::capture(&self, session_id: &str, messages: Vec<RawMessage>) -> Result<AutoCaptureResult, L0Error>` | none — LLM-free by construction (L0 store-all) |
| **recall** | `perform_auto_recall(db: &Embedded, params: AutoRecallParams<'_>, embed: Option<&EmbedFn>) -> Result<Option<RecallResult>, RecallError>` | keyword-overlap fallback; `RecallMode::effective` reports it |
| **seed** | `seed::import_seed_file(db: &Embedded, path: &Path) -> Result<SeedCounts, SeedError>` | none — no LLM dependency |
| **ingest** | `ingest::worker::run<R: LlmRunner>(store, namespace, slug, root, runner: Option<&R>, config: &IngestConfig) -> Result<IngestReport, IngestError>` | new pages verbatim; required merges recorded as skipped |

**capture** — build with `AutoCaptureHook::new(db, AutoCaptureConfig)`; `capture`
role-filters and sanitizes turns, then records them through the idempotent L0
recorder honoring the persisted cursor or the plugin-start floor
(`core/hooks/auto_capture.rs:67-88`).

**recall** — `RecallConfig` (`core/hooks/auto_recall.rs:119`) carries the search
`mode` (default hybrid), `scope` (`RecallScope`, default `Agent` — accumulates
across an agent's sessions without the cross-agent leak, D22), `max_results`
(default 5) and optional char budgets. Empty `user_text` skips L1 search but
still injects persona + scene navigation; when nothing yields, returns `Ok(None)`
— never an empty block (`core/hooks/auto_recall.rs:193-202`).

**seed** — idempotent by content-hash; `import_seed_str` / `import_seed` take
JSON input and `import_md_dir` re-imports a directory exported via
`vanta-cli export --format md` (MEM-62). CLI: `vanta-seed <seed.json> [--db <path>]`
and `vanta-seed import-md <dir> [--db <path>]`; without `--db` the import runs
in-memory (validation only, nothing persisted) (`seed/mod.rs:76-89`,
`seed/md_import.rs:216`, `bin/vanta-seed.rs:8-15`).

**ingest** — serial per page; one page failing never blocks the rest.
`run_with_progress` adds `Option<&ProgressTracker>`; progress is polled via
`wiki_status(run_id)` (500 ms throttle). The `begin` / `execute` split (MEM-52)
lets a host return the `run_id` immediately and dispatch the heavy body on a
background thread (`ingest/worker.rs:35-83`, `ingest/callback.rs:173`).

## Exposure triggers (T1–T4)

Core-only is the deliberate default (Gate P, D42/D43): exposure costs new Rust
bindings plus a support contract, so it starts only when a **measurable** trigger
fires. Firing a trigger **opens a design/evaluation task — it never authorizes
code**: any exposure still requires new bindings, the §Scope & stability
constraints, and a re-run of the Gate P HITL decision (post-release, D42/D43).

| # | Trigger | Threshold (measurable) | Measurement source | Owner |
|---|---|---|---|---|
| **T1** | External demand | ≥5 exposure requests from distinct non-maintainers | Issue tracker: issues/discussions opened by non-maintainers asking for the memory surface through a binding (count distinct requesters) | Lead |
| **T2** | ICP-03 adapter blocked | 1 frameworks-track adapter blocked because it needs the memory surface and no binding exists | ICP-03 registry (master roadmap Task 44 / frameworks one-pager): blocker recorded with task ref | Lead |
| **T3** | Stability proven | 2 release trains with 0 breaking changes to the documented Rust surface | `git log -p -- docs/api/VANTA_MEMORY.md` since 2026-09-27 (adoption) + `docs/CHANGELOG.md` | Release process |
| **T4** | Stranger-tests | FASE-A stranger-tests (2–3 unaided testers) confirm local-first demand that only a published memory surface satisfies | `docs/dev/FASE-A.md` kit + EXE-03 report | Research / owner |

Rules:

- One trigger fires → open the exposure design task (post-release only, D42/D43).
- Zero triggers fired → this page stays the manual reference; binding symbols
  stay at 0 (`rg vanta[_-]memory vantadb-python vantadb-ts vantadb-node vantadb-wasm`).
- Triggers are reviewed at release-train time, not continuously.

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

## Operational modules

Modules that live outside the L0–L3 layer pipeline (MEM-41/45/55/61/68). Same
degradation principle: no runner configured → LLM-free behavior or explicit
skip; never a silent loss.

| Module | What it does | Code refs |
|---|---|---|
| `core::dream` | Idle consolidation (sleep-time tiering, MEM-61): scans `l1/<session>` read-only and writes a consolidated view to `dream/<session>/<run_id>` — never mutates the originals | `core/dream/mod.rs:629` (`consolidate_session`), `:614` (`promote_dream_run`) |
| `core::memory_generation_log` | Per-session generation provenance at L1/L2/L3 (MEM-41) under `genlog/<session>`; best-effort, capped keep-recent | `core/memory_generation_log/store.rs:17,35,51` |
| `gateway::approval_handlers` | Typed handlers behind the MCP `capture_list_pending` / `capture_approve` / `capture_reject` tools (MEM-68) — boundary validation, no transport | `gateway/approval_handlers.rs:90-117` |
| `ingest::auto_sync` | Pull-based scheduled wiki re-ingest (MEM-45): per-file FNV-1a change detection, disabled by default, interval ≥ 60 s | `ingest/auto_sync.rs:108` (`tick`), `:33-36` |
| `services::conversation_hook` | `HttpCaptureBridge` — implements the core's `ConversationTrigger` for `POST /api/v2/conversations` (MEM-55): L0 capture + L1 task enqueue; **not wired in production yet** | `services/conversation_hook.rs:36-45,93-107` |

**`core::dream`** — `consolidate_session` requires the idle window (`detect_idle`,
default 10 min) and degrades without a runner to LLM-free primitives: hash-bucket
dedup (`merge_duplicates`), deterministic contradiction resolution by
priority+timestamp (`resolve_contradictions`, reusing MEM-60 provenance) and an
es-first relative-date table (`normalize_relative_dates`). LLM tiering is opt-in
via the `Dreamer` trait; `promote_dream_run` is a count-only stub (no mutation) —
the real promotion into L1 is MEM-65; `discard_dream_run` deletes the run
namespace. Integration test `tests/dreaming.rs` pins the byte-identical-L1
invariant.

**`core::memory_generation_log`** — one `GenerationLogEntry` (`{layer, status,
anchor_id, session_key, ts_ms, error?}`) per L1/L2/L3 generation, queryable per
session ordered by timestamp. Best-effort by design: `record_best_effort`
swallows store errors with a `tracing::warn!` — a logging failure can never fail
a generation; growth is capped keep-recent (`MAX_ENTRIES_PER_SESSION = 100`).

**`gateway::approval_handlers`** — `capture_approve` persists the approved
capture through the queue (approve-time store failure surfaces as a typed error —
an already-approved capture is never silently dropped); `capture_reject` is
idempotent (`rejected: false` when already decided or never queued); ids are
minted by the queue `submit`, never by the gateway. Degradation: none needed —
no LLM dependency on either path (the optional `embed` parameter only adds a
vector).

**`ingest::auto_sync`** — pull-based, zero threads: the owner polls
`AutoSyncScheduler::tick`; the deadline lives in `ManagedTimer` over an injected
`Clock` (FakeClock in tests). While the wiki is `pending|processing` the tick
returns `Busy` without updating the stored hashes (change re-detected next
pass); each build mints a fresh `run_id` (MEM-31) and late packets from older
runs are discarded; the first due pass has no baseline → reconciling re-ingest.
Ceiling: full-content rescan per tick — a watcher is only worth it past ~10k
files (`// ponytail` note in source).

**`services::conversation_hook`** — the core cannot depend on `vanta-memory`
(cycle), so the hook point is the additive
`vantadb::cli_server::ConversationTrigger` trait and the impl lives here.
`trigger` captures the saved turn into L0 (LLM-free — data is never lost) and
enqueues an L1 task; `run_bridge_pass` drains the queue through the MEM-16
worker (`trigger_every_n = usize::MAX`, so this path never regenerates persona).
**Wiring status:** the bridge ships, but the core bootstrap passes `None`
(`src/server/bootstrap.rs:332`, field `src/server/state.rs:134`, call site
`src/server/handlers.rs:1391`) → inactive in production until a host wires it
(MEM-55 residual).

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
worker::run(store, sources_root, runner_opt, cfg) -> IngestReport
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
| `dream/<session>/<run_id>` | vista consolidada por corrida (MEM-61; `discard` real, promote stub) |
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
