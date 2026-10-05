---
title: "Vanta Memory Engine — API Reference (`vanta-memory`)"
kind: reference
status: active
description: "Crate LLM-driven para memoria de agentes: captura L0, extracción/dedup L1, escenas L2,"
tags: [vantadb, api, vanta-memory, memory-engine]
---

# Vanta Memory Engine — API Reference (`vanta-memory`)

> **Estado:** ✅ documentación canónica del crate (cierra la cita de ADR-0029 §Nota mecánica).
> **Nota:** `scripts/validate-docs-coverage.ps1` hoy NO escanea `vanta-memory`; esta página es
> la referencia manual. Las superficies F1-F3 (search profile, entity_*, skills) viven en
> `EMBEDDED_SDK.md`.

Crate LLM-driven para memoria de agentes: captura L0, extracción/dedup L1, escenas L2,
persona L3, recall con scope, context engine (compresión), offload y generación wiki.
**Principio rector:** el LLM es opcional (P4) — todo flujo degrada sin perder datos cuando
el runner falla o no está configurado.

## Scope & stability (Gate P — 2026-09-24)

- **Core-only by decision (TS/Node/WASM)** (`API-STD-15`, Gate P): `vanta-memory` is an internal workspace
  member consumed in-process by `vantadb-mcp`, `vanta-proxy` and `desktop/src-tauri`.
  Exposing it requires new Rust bindings plus demonstrated demand (post-release, D42/D43).
- **Binding scope (DIST-03, 2026-10-04):** **TS/Node/WASM: not exposed** — declared core-only,
  with the WASM viability evidence in
  [`BINDINGS_NAMESPACES.md` §Cognitive layer scope](./BINDINGS_NAMESPACES.md#cognitive-layer-vanta-memory-scope-per-binding-dist-03-2026-10-04);
  a **minimal Python surface** (`memory_recall`/`memory_capture`) **landed** for the 0.9.0
  train (DIST-02, 2026-10-04 — smoke e2e con wheel local). This page and that matrix are
  reconciled to the landed surface (DIST-04, 2026-10-04).
- **Crate distribution (DIST-01, 2026-10-04):** `vanta-memory` is **publishable**
  (`publish = false` removed; `cargo publish --dry-run` verde + smoke externo) with an
  explicit **hold** in `release-plz.toml` until the owner bootstraps crates.io Trusted
  Publishing (the first publish requires an API token — bootstrap checklist in the `release-plz.toml` hold entry; rationale in `docs/dev/tasks/DIST-01.md` §Decisión).
- **Stable Rust API:** the public surface documented here is the stable contract for in-repo
  consumers. Binding re-export: **none for TS/Node/WASM** — `0` symbols by design (checked
  with `rg "vanta[_-]memory" vantadb-ts vantadb-node vantadb-wasm` → 0 matches).

## Facade — capture / recall / seed / ingest

> **Status: stable in-repo contract; minimal Python surface landed (DIST-02).**
> Conceptual signatures for the four host-facing operations in-repo consumers
> build against today. They summarize the stable Rust surface (§Scope & stability)
> as a reading aid. Re-export today: **Python** exposes `capture`/`recall` as
> `memory_capture`/`memory_recall` (DIST-02); `seed`/`ingest` are **not**
> re-exported by any binding; TS/Node/WASM expose nothing (§Exposure triggers).
> Further exposure still requires new bindings and a fired trigger.

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
(default 5), optional char budgets and `core_search` (default `false`; opts into
ranking via the core hybrid engine — see §Recall). Empty `user_text` skips L1
search but still injects persona + scene navigation; when nothing yields, returns
`Ok(None)` — never an empty block (`core/hooks/auto_recall.rs:193-202`).

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

## Git-friendly Markdown export & rebuild (VER-06)

`vanta-cli export --format md` projects the store (or one namespace) to one
Markdown file per record under
`<out>/<sanitized-namespace>/<sanitized-key>.md`, plus an `index.json`
manifest. The projection is **git-friendly**: unchanged data exports
byte-identically (no wall-clock timestamps; the manifest is sorted by file
path), so `git diff` shows only real memory changes.

**The store is the source of truth; Markdown is a projection.** The flow is
edit → re-import → rebuild:

```bash
# 1. Export the projection (one file per record).
vanta-cli --db ./memory export --format md --out ./memory-md

# 2. Edit a file in your editor, then review the change like any diff.
git -C ./memory-md diff

# 3. Re-import the (possibly edited) directory — idempotent: records whose
#    projection is unchanged are skipped.
vanta-seed import-md ./memory-md --db ./memory

# 4. Rebuild the derived indexes (HNSW / text / derived).
vanta-cli --db ./memory rebuild-index

# 5. Read/search the edited value back.
vanta-cli --db ./memory get --namespace agent/team --key intro
vanta-cli --db ./memory search --namespace agent/team "edited text"
```

The flow is additive only: it never deletes records or prunes stale `.md`
files — deletes go through the store API, not by removing files from the
export. Vectors do not travel in Markdown (`vector_dim` is informational):
re-importing an edited record leaves its `vector`/`sparse_vector` empty for
that record (pre-existing MEM-62 behavior).

Frontmatter is **schema v2** and carries the full record semantics: identity
(`namespace`, `key`, `version`, `node_id`), timestamps (`created_at_ms`,
`updated_at_ms`, `expires_at_ms`), supersession (`superseded_by`,
`superseded_at_ms`), validity window (`valid_at_ms`, `invalid_at_ms`),
provenance and confidence (`confidence_class`, `confidence`,
`last_validated_at_ms`, `derived_from`) and quarantine state
(`quarantined_at_ms`, `quarantine_reason`, `quarantined_by`,
`quarantine_review_due_ms`). The importer accepts `schema_version` 1 and 2:
v1 files are normalized on import (`valid_at_ms = created_at_ms`,
`confidence = 1.0`, class `Asserted`), so exports written before 0.8.0 keep
working.

Records with relations (`superseded_by`, `derived_from`) also get a trailing
informational block of wikilinks:

```markdown
<!-- vanta:links -->

## Related

- [[agent/team/newer]]
```

The block is editor-friendly (Obsidian-style navigation) but **informational
only**: the importer strips it before reading the payload, and reports targets
with no destination record as `links_unresolved` in the import summary
(`created=…, updated=…, unchanged=…, links_unresolved=…`; counts link
occurrences, not distinct targets). It is never a
source of joins — change relations through the store API, not in the
projection.

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
- Zero triggers fired → this page stays the manual reference; **TS/Node/WASM
  binding symbols stay at 0** (`rg vanta[_-]memory vantadb-ts vantadb-node vantadb-wasm`).
  The Python minimal surface **landed** for the 0.9.0 train (DIST-02) is scoped in
  [`BINDINGS_NAMESPACES.md` §Cognitive layer scope](./BINDINGS_NAMESPACES.md#cognitive-layer-vanta-memory-scope-per-binding-dist-03-2026-10-04).
- **DIST-03 (2026-10-04):** TS/WASM exposure was evaluated (compile checks +
  official docs); outcome: per-binding scope declared (TS/Node/WASM core-only)
  and the WASM port gap list captured as `FIND-255`.
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
| `erasure` | Cryptographic erasure (per-scope DEK registry) + verifiable erasure receipts (pulls core `encryption`: AES-256-GCM) | off |
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
| L1 | `core::record::{l1_extractor,l1_dedup,l1_reader,l1_writer}` | Extracción 1-call LLM JSON con parse reparado; dedup 2 fases store/update/merge/skip; contradicción explícita en el juicio (`contradicts`) → flag `superseded_by` (MEMG-01, nunca delete) |
| L2 | `core::scene::{scene_index,scene_format,scene_extractor,scene_tools}` | Escenas con META {created,updated,summary,heat}, strategy UPDATE>MERGE>CREATE, soft-delete, tools sandboxed |
| L3 | `core::persona::{persona_generator,persona_trigger}` | Persona first/incremental con triggers P1-P4 y escape XML |
| Recall | `core::hooks::auto_recall`, `core::memory_prompt::*`, `core::profile::profile_sync` | Prepend/append + 3 modos (`RecallScope::Session\|Agent\|Team`, default Agent) |
| Context | `context_engine::{engine,compressor,mmd,mmd_injector,token_estimator,types}` | Compresión LLM-free mild/aggressive/emergency + MMD persistente + budget coordinator |
| Offload | `offload::{state_manager,storage,reclaimer,hooks::after_tool_call}` | Cursor `lastOffloadedToolCallId`, entradas por tool_call_id, GC por retention |
| Ingest | `ingest::{worker,merge,prompts,callback}` | Ingest wiki serial (fallo por página no bloquea), progreso canal interno + polling run_id |
| Skills | `core::skill::skill_extractor` + `conversation_add` | Extracción desde transcript con marcadores anti role-capture; sink idempotente doble cursor+content-hash |
| Orquestación | `services::pipeline_worker`, `utils::{pipeline_manager,stateful_pipeline_manager,managed_timer,checkpoint,task_checkpoint,backup}` | Timers/locks estado local, trait `Clock` inyectable (FakeClock determinista), worker L0→L1→L2→L3, checkpoints de tarea reanudables (MEMG-20), backup/restore vía snapshot (MEMG-13) |
| Gateway | `gateway::knowledge_handlers` | Handlers tipados scene_read/list/query para exposición MCP/server |

**Contradicciones en ingesta (MEMG-01).** El juicio de dedup L1 —la misma
llamada LLM, sin round-trip extra— acepta un campo opcional `contradicts` con
ids del pool de candidatos que la memoria nueva niega EXPLÍCITAMENTE ("ya no me
gusta X" tras "me gusta X"). Al persistir, el registro viejo se marca con
`superseded_by` vía `mark_contradiction` (MEM-60 — misma semántica que dream:
nunca se borra, queda auditable) y el evento de provenance queda en el tracing
log (`mark_contradiction`; persistencia audit más allá del tracing = deuda
declarada). Señal conservadora:
solo negación explícita (duda → `[]`), registros ya-superseded no se re-marcan,
ids desconocidos se saltan; `DedupDecision.contradicts` es `#[serde(default)]`
(wire retrocompatible). Cuarentena por contradicción queda diferida
(MGR-13 §3.4). El pool de candidatos es intra-sesión; cross-sesión es deuda
declarada.

## Operational modules

Modules that live outside the L0–L3 layer pipeline (MEM-41/45/55/61/68). Same
degradation principle: no runner configured → LLM-free behavior or explicit
skip; never a silent loss.

| Module | What it does | Code refs |
|---|---|---|
| `core::dream` | Idle consolidation (sleep-time tiering, MEM-61): scans `l1/<session>` read-only and writes a consolidated view to `dream/<session>/<run_id>`; promotion (VER-07) applies the view to L1 only via an explicit, gated, idempotent `promote_dream_run` — dry-run by default on the MCP surface | `core/dream/mod.rs:946` (`consolidate_session`), `:720` (`plan_promotion`), `:869` (`promote_dream_run`) |
| `core::memory_generation_log` | Per-session generation provenance at L1/L2/L3 (MEM-41) under `genlog/<session>`; best-effort, capped keep-recent | `core/memory_generation_log/store.rs:17,35,51` |
| `gateway::approval_handlers` | Typed handlers behind the MCP `capture_list_pending` / `capture_approve` / `capture_reject` tools (MEM-68) — boundary validation, no transport | `gateway/approval_handlers.rs:90-117` |
| `ingest::auto_sync` | Pull-based scheduled wiki re-ingest (MEM-45): per-file FNV-1a change detection, disabled by default, interval ≥ 60 s | `ingest/auto_sync.rs:108` (`tick`), `:33-36` |
| `services::conversation_hook` | `HttpCaptureBridge` — implements the core's `ConversationTrigger` for `POST /api/v2/conversations` (MEM-55): L0 capture + L1 task enqueue; **wired by `vantadb-server` HTTP mode** (WIRE-16) | `services/conversation_hook.rs:36-45,93-107` · `vantadb-server/src/scheduler.rs:124-146` |
| `services::scheduler` | Pull-based pass (`run_pass` = expired timers + one worker pass + stale-claim reclaim) + feature-gated loop helper (`spawn_memory_scheduler`, `http-server`); host-driven with graceful join (WIRE-15/16) | `services/scheduler.rs:80` (`run_pass`), `:198` (`spawn_memory_scheduler`) |

**`core::dream`** — `consolidate_session` requires the idle window (`detect_idle`,
default 10 min) and degrades without a runner to LLM-free primitives: hash-bucket
dedup (`merge_duplicates`), deterministic contradiction resolution by
priority+timestamp (`resolve_contradictions`, reusing MEM-60 provenance) and an
es-first relative-date table (`normalize_relative_dates`). LLM tiering is opt-in
via the `Dreamer` trait. The consolidation path never mutates L1; promotion is
the only mutating entry point: `plan_promotion` returns the per-record diff
(`ADD|UPDATE|DELETE|NOOP` with `normalize`/`supersede`/`merge`/`dedup` reasons)
with L1 byte-identical, and `promote_dream_run` applies it — idempotent
(re-apply → all NOOP), fail-closed quality gate on supersedes, and DELETEs
scoped to the run's scanned inputs (`DreamRun::input_ids`; runs persisted before
VER-07 deserialize empty and never delete). The MCP `dream_promote` tool
defaults to `dry_run:true`. `discard_dream_run` deletes the run namespace.
Integration test `tests/dreaming.rs` pins the dry-run L1 byte-identity and the
idempotent-promote invariants.

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
**Wiring status (WIRE-16):** wired in production by `vantadb-server` in HTTP
mode (writable servers — read-only skips the wiring, mirroring the TTL sweeper
guard) — the host attaches `HttpCaptureBridge` through the additive
`ServerHooks::on_storage_ready` seam, which hands it a clone of the server's
`Embedded` handle (single writer: the server owns the process's one open; a
second open fails with `DatabaseBusy`). The trigger reaches `ServerState`
(field `src/server/state.rs:143`, call site `src/server/handlers.rs:1414`).
Defaults stay inert — `ServerHooks::default()` (hence `run(config)`) passes
`None` — and the MCP mode of the same binary does not host the scheduler
(ADR-0054: second host deferred).

**`services::scheduler`** — the reusable pass of the L0→L3 planner (WIRE-15):
`run_pass` dispatches expired `l1_idle:<session>` timers as L1 tasks, runs one
`PipelineWorker` pass and reclaims stale claims; unknown timer members are
skipped with a debug log (L2/Dream timers arrive with their producer). Always
available (no Tokio); the loop helper `spawn_memory_scheduler` is
feature-gated (`http-server`), mirrors the core TTL sweeper (watch + join,
`Drop` best-effort) and is joined through `BackgroundService` after the
server's HTTP loop returns. The host owns the driver: `vantadb-server`
(WIRE-16) wires the bridge **always** and spawns the loop only when
`VANTADB_SCHEDULER_INTERVAL_MS > 0` (default `60000`; `0` = off — captures
keep flowing into L0). Each pass builds its runner from the FIND-112 surface
(`VANTADB_INGEST_CONFIG` or `<storage>/data/vanta-ingest.toml`; secrets
env-only, R-5); no real engine → the pass skips observably and the queue is
untouched (P4). One loop per process — the scheduler runs in the writer
(ADR-0054).

## Audit & backup (MEMG-13)

Consumption of the core `versions` and `snapshot` surfaces — audit/diff and
backup/restore for L1 memory, with no core changes and no reimplementation.

**Version history / diff** (`core::record::{read_record_versions, read_record_version, diff_records}`):

| Function | Contract |
|---|---|
| `read_record_versions(db, session_key, record_id) -> Vec<RecordVersion>` | Every retained version of an L1 record, ascending (`RecordVersion { version: u64, record: MemoryRecord }`); core `Embedded::versions` (VS-CORE-07). Best-effort post-commit: a crash window can leave a version gap (degraded, never corrupt). **Audit surface:** does not apply the quarantine gate (SCH-05) — retained versions expose quarantined state on purpose; never feed this history into model context |
| `read_record_version(db, session_key, record_id, version) -> Option<RecordVersion>` | One version by its storage number (core `Embedded::get_version`); `None` when the record or version is absent |
| `diff_records(older, newer) -> Vec<RecordFieldChange>` | Top-level payload fields that differ, ordered by name; fields skipped by `skip_serializing_if` (unset optionals) read as `Null` — appearing/disappearing is reported, never silent |

**Backup / restore** (`utils::backup::{create_snapshot, list_snapshots, restore_snapshot}`):

```rust
let snap = create_snapshot(&db, "mem-2026-10-05")?;    // point-in-time image
let names = list_snapshots(&db)?;                      // names available
db.close()?;                                           // restore needs no open engine
let db = restore_snapshot(config, "mem-2026-10-05")?;  // reopened over restored data
```

Thin delegation to the core (`Embedded::create_snapshot`/`list_snapshots`/`restore_from`);
the core owns quiesce/mirror/rollback and validates the snapshot name (anti
path-traversal — the wrapper never relaxes it). Requires an on-disk store
(Fjall); `InMemory` keeps no files. Restore scope (core contract): only
`<storage_path>/data` is swapped back — post-snapshot additions disappear,
while post-snapshot deletions/supersessions keep their live backend-KV
tombstones and are **not** rolled back (FIND-287).

**Evaluated, not consumed:** the recall cursor is already consumed
(`read_namespace_records` pages via `MemoryListOptions.cursor`);
`include_quarantined:false` (SCH-05) is consumed too. IQL and the remaining
core filters (`min_confidence`, temporal, metadata) have no memory-side
consumer today → FIND-285 / FIND-286 (Backlog).

## Rollback, erasure & receipts (MEMG-17)

Semantic rollback over the MEMG-13 version history, cryptographic erasure by
DEK destruction and verifiable erasure receipts (VER-02 certificate contract).

**Rollback:**

| Function | Contract |
|---|---|
| `rollback_record(db, session_key, record_id, target_version, now_ms) -> RollbackReport` | Restores the target version's payload as a **new** version (append-only lineage: nothing is rewritten or dropped). Bookkeeping advances to the rollback write (`version`/`updated_at`/`timestamps`); the report carries `from_version`/`to_version`/`new_version`, the store delta (`changes = diff_records(live, restored)`, name-ordered) and the declared `out_of_scope` (supersession restored verbatim, no cross-record cascade, deletes not resurrected, audit surfaces append-only). `NotFound` when the record or version is absent — a deleted record's history is purged, it is not resurrectable from versions |
| `rollback_snapshot(config, name) -> (Embedded, SnapshotRollbackReport)` | `restore_snapshot` plus the explicit declared scope: `reverted` (the `data/` swap + index rebuild) vs `not_reverted` (live backend KV: post-snapshot deletes/supersessions keep their tombstones/metadata — FIND-287) and the flow `caveats`. Same flow as `restore_snapshot`: close → rollback → reopened `Embedded` |

**Cryptographic erasure** (feature `erasure`) — per-scope random DEK registry
under `erasure/dek`, wrapped with the core master `Cipher`
(`VANTADB_ENCRYPTION_KEY`; composed from `vantadb::crypto`, no new cipher):

| Function | Contract |
|---|---|
| `create_scope(db, master, scope)` | Registers a fresh 32-byte CSPRNG DEK for the scope (record key = `sanitize_key(scope)`); an existing scope is rejected (no silent rekey — it would orphan sealed data) |
| `seal(db, master, scope, plaintext) / open(db, master, scope, blob)` | AES-256-GCM under the scope's DEK; a different scope or a tampered blob fails to open. Blobs are caller-held — the module does not track or delete encrypted copies |
| `erase_scope(db, scope, reason) -> ErasureReceipt` | Destroys the wrapped DEK through the core delete path (backend tombstone + version-history purge) and emits the receipt; everything sealed under it becomes cryptographically unrecoverable. Emits `not_found` receipts too — never silent |
| `verify_erasure_receipt(db, receipt_json) -> ErasureVerification` | Schema + integrity hash first, then a live re-scan (`dek_record`, `version_history`). **Not claim-driven**: partial/edited/unknown-status receipts are rejected even with a recomputed hash, and a re-created scope breaks the old receipt's clean claim (`residues reappeared`) |

`ErasureReceipt` mirrors the VER-02 purge-certificate contract: schema version,
timestamp, scope/reason/status, fixed-order surfaces, VER-01 `ChainEvidence`,
non-empty `out_of_scope` and an sha256 self-hash — no key material, and no
signature (ML-DSA-65 is not sanctioned by any spec; an actor who recomputes
the hash is not detected — declared in the receipt limits). Scope of the
erasure: payloads sealed through `seal`; wiring envelopes into the L1 write
path is a declared deferral → FIND-302 (Backlog). Copies of the wrapped DEK
outside the live store (snapshots, backups, WAL archives) are declared
out-of-scope limits — see the receipt and FIND-194.

## Task checkpoints (MEMG-20)

Resumable checkpoints of **host tasks** (dim 1 working memory): current step +
partial results + state, so a new instance resumes after an interruption
without repeating completed steps. **Separate domain from the pipeline
checkpoint** (TDAM `Checkpoint`, namespace `pipeline_checkpoint`): separate
types, separate namespace (`task_checkpoints`), no shared fields. This module
stores progress — it is **not** a task engine (no step execution or
orchestration).

**API** (`utils::task_checkpoint::{TaskCheckpointManager, TaskCheckpoint, TaskState}`):

| Method | Contract |
|---|---|
| `begin(task_id) -> TaskCheckpoint` | Ensure a checkpoint exists: fresh at `step=0` (`InProgress`) or the existing record unchanged — **idempotent**, a re-run never clobbers progress; deliberate restart = `delete` + `begin` |
| `advance(task_id, result) -> TaskCheckpoint` | Record one completed step: appends `result` to `partial` and `step += 1` (RMW); rejects missing (`NotFound`) or finished (`NotInProgress`) checkpoints |
| `complete(task_id)` / `fail(task_id) -> TaskCheckpoint` | Terminal states (RMW); `fail` keeps `partial` for inspection/retry |
| `load(task_id) -> Option<TaskCheckpoint>` | Read one checkpoint (`None` = never begun) |
| `delete(task_id)` | Remove the checkpoint (idempotent) |

`TaskCheckpoint { version, step, state, partial }` — `step` is the **next step
to run** (steps `< step` are completed; `partial[i]` is step `i`'s result, so
`partial.len() == step` while advanced through the manager). `version`
(`TASK_CHECKPOINT_VERSION = 1`) is the schema-migration hook.

```rust
// Host resume loop: interruption-safe by construction.
let checkpoints = TaskCheckpointManager::new(&db);
let cp = checkpoints.begin("task-1")?;          // resume-safe start
for step in cp.step..total_steps {
    let result = run_step(step);                // host work…
    checkpoints.advance("task-1", result)?;     // durable after each step
}
checkpoints.complete("task-1")?;
```

**Persistence:** one JSON record per task under `task_checkpoints` (key =
sanitized `task_id`); RMW per record — keep a single writer per `task_id`
(same in-process atomicity discipline as the pipeline checkpoint). Pinned by
`vanta-memory/tests/task_checkpoint.rs`: namespace separation via raw SDK
(no cross-contamination, no pipeline fields in a task record) and Fjall
close/reopen resume (steps 0–1 done → new instance resumes at 2,
`executed == [2, 3, 4]`, partial preserved).

**Host limit:** resuming after the agent's *context compaction* is the host's
concern — this manager persists the task state, not the agent's conversational
context (that lives in the context engine). Granularity: one entry per
completed step; mid-step progress is not persisted. Resume is
**at-least-once**: a crash between a step's effect and `advance` re-runs that
step — host steps should be idempotent (or reconciled) under resume.
`task_id`s are sanitized (`[A-Za-z0-9._-]`, ≤512 bytes); keep them in the safe
set to avoid silent key collisions. In-repo consumers today:
**none** — the pipeline worker tasks (L1/L2/L3/Dream) are single-pass; the
natural consumer is an agent host (dim 1 proposal) → FIND-288 (Backlog).

## Forgetting curve (MEMG-07)

Declared forgetting curve over L1 records: a per-type half-life policy that
**deprioritizes** memories as they age without access. `heat` (bumped on every
read) is never mutated and nothing is ever deleted — the curve is **read-side**:
consumers derive an *effective* heat from (stored heat, age, type), and the
discard gate stays explicit (`PRUNE_HEAT_THRESHOLD`).

**Policy, not calibration.** No canonical decay formula for semantic memory has
been validated (N-09); the half-lives below are declared, tunable defaults. The
shape is the exponential forgetting form `R = e^(−t/S)` parametrized by
half-life (`R = 2^(−age/half_life)`) — see
[Forgetting curve](https://en.wikipedia.org/wiki/Forgetting_curve); the same
source notes the simple exponential does not fit human data well, which is why
this is a policy. The full Ebbinghaus vision (access frequency, importance,
confirmations, salience — FUT-10) is partially realized by the composite-scoring
work (MEMG-21): the curve is consumed as the recency signal; access
frequency/confirmations remain open.

**API** (`core::record::lifecycle::{DecayPolicy, retention_factor, effective_heat, scan_decay, DecayReport}` + `core::record::run_decay_pass`):

| Function | Contract |
|---|---|
| `DecayPolicy::default()` | Declared per-type half-lives; a type absent from the map never decays (`half_life_ms` → `None`). `set_half_life` / `clear_half_life` / `half_life_ms` configure it |
| `retention_factor(record, policy, now_ms) -> f64` | `2^(−age/half_life)` ∈ [0, 1]; `age` = time since `updated_at` (last touch — `bump_heat` refreshes it on access, so use resets retention). Exempt type / clock skew / unparseable timestamps → `1.0` (what cannot be aged is never forgotten) |
| `effective_heat(record, policy, now_ms) -> u32` | `heat × retention`, rounded — the deprioritized value consumers rank with. Stored `heat` untouched |
| `scan_decay(records, policy, now_ms) -> DecayReport` | Pure scan: disjoint `decayed` / `unchanged` / `exempt` counts, `below_threshold` (effective ≤ `PRUNE_HEAT_THRESHOLD`), `heat_total` / `heat_effective` / `heat_forgotten()` |
| `run_decay_pass(db, session_key, policy, now_ms) -> Result<DecayReport, L1Error>` | The pass: `read_session_records` + `scan_decay` over one session. **Read-only**, pull-based (the owner calls it, like `TimerScanner::run_once`); never mutates or deletes |

**Declared defaults** (policy; tune per deployment):

| Type | Half-life | Rationale |
|---|---|---|
| `persona` | 90 d | stable traits/preferences |
| `episodic` | 7 d | one-off events |
| `instruction` | — (exempt) | followed until contradicted; never forgotten by a curve |
| `work_fact` | 30 d | facts about the user's work/team |
| `work_task` | 14 d | open/completed tasks age fast |
| `work_method` | 60 d | procedures are durable |
| `work_artifact` | 30 d | files/docs/code |

```rust
let policy = DecayPolicy::default();
let report = run_decay_pass(&db, "sess-1", &policy, now_ms)?;
// report.scanned / decayed / unchanged / exempt / below_threshold
// report.heat_total → report.heat_effective  ("how much decays")
```

**Guarantees** (pinned by the `lifecycle.rs` inline tests +
`vanta-memory/tests/forgetting_curve.rs`): the pass is idempotent (same `now_ms`
→ same report, payloads byte-identical); it never deletes or mutates records;
known-value math is exact (`8 × 2^(−1) = 4`, `3 × 2^(−2) = 0.75 → 1`). In-repo
consumers today: **none** — no scheduler calls the decay pass yet (the
WIRE-15/16 scheduler pass covers timers + worker + reclaim only) and the
automatic discard is deliberately deferred → FIND-289 (Backlog).

## Composite scoring (MEMG-21)

Opt-in re-ranking of L1 recall candidates on top of the existing dual-pool
relevance machinery (D38: keyword overlap + cosine fused with RRF):

```text
composite = w_rel · relevance_norm + w_rec · recency + w_imp · importance
```

**Policy, not calibration.** The weighted form is Park et al. §4.1
([Generative Agents](https://arxiv.org/abs/2304.03442), arXiv:2304.03442v2 —
"weighted combination of the three elements", each signal min-max normalized to
[0,1]); the declared defaults are CrewAI's documented composite-scoring
defaults ([unified memory](https://docs.crewai.com/en/concepts/memory) —
`semantic 0.5 / recency 0.3 / importance 0.2`). No calibrated-accuracy claim is
made; tune per deployment.

| Signal | Source | Notes |
|---|---|---|
| `relevance_norm` | The caller's raw pool score (keyword overlap or cosine) or the fused RRF score when both arms contribute — min-max normalized over the candidate set | RRF is rank-based and scale-free, so the arms never compete on raw scales |
| `recency` | [`retention_factor`](#forgetting-curve-memg-07) (MEMG-07) — `2^(−age/half_life)`, age since `updated_at` | Same exponential family as CrewAI's `decay = 0.5^(age_days/half_life_days)`; consumed, not reimplemented |
| `importance` | The record's declared `priority` (0-100) → `[0,1]`; `priority < 0` (strict global instruction) → `1.0`; >100 clamps | Set at encoding time (Park/CrewAI agree); `heat`'s effect rides recency — `bump_heat` refreshes `updated_at` |

**API** (`core::record::{ScoringWeights, CompositeScoring, importance_score, composite_score, composite_rank}` + `core::hooks::perform_auto_recall_scored`):

| Function | Contract |
|---|---|
| `ScoringWeights::default()` | Declared per-mille weights: `relevance: 500`, `recency: 300`, `importance: 200` (CrewAI defaults). Integers keep the config `Eq`-comparable |
| `CompositeScoring::default()` | `weights` (above) + `decay: DecayPolicy` (MEMG-07 per-type half-lives). Serde round-trips |
| `importance_score(record) -> f64` | `priority` mapping (see table above) |
| `composite_score(record, relevance, scoring, now_ms) -> f64` | One candidate; `relevance` expected already normalized. Degenerate all-zero weights → `relevance` (conservative fallback) |
| `composite_rank(candidates, scoring, now_ms) -> Vec<usize>` | Best-first indices; min-max normalizes raw relevance over the set; ties: `updated_at` desc, then `id` asc |
| `perform_auto_recall_scored(db, params, embed, policy, scoring, now_ms)` | The opt-in entry point: same ACL/scope rules as `perform_auto_recall_governed`, re-ranking **before** the `max_results` cut on both routes (in-memory dual-pool and `core_search`). `now_ms` injected for determinism |

```rust
use vanta_memory::core::hooks::{perform_auto_recall_scored, InjectionPolicy};
use vanta_memory::core::record::CompositeScoring;

let out = perform_auto_recall_scored(
    &db,
    params,
    None,                            // embed hook (optional)
    &InjectionPolicy::allow_all(),
    &CompositeScoring::default(),    // declared weights + MEMG-07 decay policy
    now_ms,
)?;
```

**Guarantees** (pinned by `scoring.rs` inline tests +
`vanta-memory/tests/composite_scoring.rs`): known-value math is exact
(`0.5·1 + 0.3·1 + 0.2·0.5 = 0.9`; one half-life → recency `0.5`); the ordering
metric on the fixture improves the fresh+important record's rank 2 → 1
(before `[r1,r2,r3,r4]` → after `[r2,r1,r3,r4]`); uniform signals preserve the
legacy order; the existing entry points keep the legacy ordering
**byte-identical** (opt-in by construction — no caller changes behavior).
In-repo consumers today: **none** — activation is a host decision (FIND-290).

## Reflection (MEMG-21)

Pull-based reflection pass over a session's **episodic** L1 records: groups
episodes by scene and produces lessons that cite their source records, written
to a separate `reflection/<session>/<run_id>` namespace. **The L1 store is never
mutated** — promotion stays explicit and out of scope (FIND-290).

Sources (policy, not calibration): Park et al. §4.2 (*Generative Agents*,
arXiv:2304.03442v2 — reflections are generated periodically, synthesize
higher-level insights and cite the records that served as evidence); the
in-crate precedent is [`core::dream`](#operational-modules): an optional LLM
runner with a deterministic LLM-free degradation (P4) — without a runner the
pass emits a digest lesson per scene; nothing blocks and nothing is lost.

**API** (`core::reflection::{ReflectionConfig, Reflector, ReflectionRun, reflect_episodic, reflect_session, load_reflection_run}`):

| Function | Contract |
|---|---|
| `ReflectionConfig::default()` | `min_episodic: 3` (declared precondition), `runner: None` (LLM-free), `run_id_salt: ""`. Builders: `with_min_episodic` / `with_reflector` / `with_run_id_salt` |
| `Reflector` trait | Host extension point (`label` + `reflect`) for true LLM synthesis; the clone of a config deliberately drops the runner (same choice as `DreamConfig`) |
| `reflect_episodic(records, session_id, now_ms) -> Vec<MemoryRecord>` | Pure LLM-free digest: one `WorkMethod` lesson per scene, top-`DIGEST_TOP` (3) episodes by priority in the content, **every** scene id cited in `metadata.reflection.source_ids`, deterministic id (`reflect-<hash>`) |
| `reflect_session(db, session_id, now_ms, config) -> Result<ReflectionRun, ReflectionError>` | The pass: reads `l1/<session>` (quarantine gate SCH-05 inherited), filters episodic, enforces `min_episodic` (`NotEnoughMaterial` below it — observable skip, never silent), runs the runner or the digest, persists the run under `reflection/<session>/<run_id>` |
| `load_reflection_run(db, session_id, run_id) -> Result<Option<ReflectionRun>, ReflectionError>` | Read one run back (review/replay) |

```rust
use vanta_memory::core::reflection::{reflect_session, ReflectionConfig};

let run = reflect_session(&db, "sess-1", now_ms, &ReflectionConfig::default())?;
// run.lessons: one WorkMethod per scene, provenance in metadata.reflection
// run.runner_label == "none" (LLM-free degradation, P4)
```

**Guarantees** (pinned by `reflection/mod.rs` inline tests +
`vanta-memory/tests/reflection.rs`): the pass leaves `l1/<session>`
**byte-identical** (payload comparison, all records still present); same
salt + `now_ms` → identical run (deterministic lessons); the persona type is
ignored; a configured runner overrides the digest and its label is persisted.
In-repo consumers today: **none** — promotion of lessons to L1 and its wiring
into the scheduler are deferred → FIND-290 (Backlog).

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
assemble_with_recall(..., spill: Option<&mut dyn SpillSink>)  // coordinator único: assemble → inject_mmd → recall, un solo budget (+ spill opt-in MEMG-06)
```
- Ratio < 0.5 → skip sin tocar mensajes.
- Mild cascade (MIN=10/INITIAL=7/FLOOR=1) → aggressive one-shot (fingerprint boundary
  `role + primeros 200 chars`, idempotente) → emergency prefix-aware (~2000 chars).
- Los pares tool_call/tool_result son unidades atómicas: nunca se parten.
- Mensajes ≤ cursor `lastOffloadedToolCallId` (MEM-20) van en `protected_prefix`.
- `inject_mmd` agrega `<current_task_context>` tras el prefijo System con dedup fingerprint.

### Spill to disk + recall (MEMG-06, opt-in)
```rust
pub trait SpillSink { fn spill(&mut self, message: &ChatMessage, original: &str); }
SpillStorage::{spill, recall(session, id), recall_session(session), reclaim, reclaim_as_of}
ContextAssemblyConfig { spill_enabled: bool /* default false */, .. }
```
- With `spill_enabled: true`, every message replaced by a `[compacted N chars]` stub
  persists its full payload under `spill/<session>` **before** the replacement lands
  (get-before-put dedup, D19; key = sanitized message id, or `anon-<fnv1a64>` for
  id-less messages). The engine itself stays store-free: hosts pass a `SpillSink`
  (the worker wires `DbSpillSink`).
- Recall is explicit — `recall(session, id)` / `recall_session(session)` — and is
  never mixed into the L1 recall path (`perform_auto_recall`).
- GC reuses the offload reclaimer rules: `reclaim_as_of` skips passes below
  `MIN_RETENTION_DAYS = 3` and never deletes entries with unparseable timestamps.
  The offload cursor gate does not apply (spilled content was already consumed by
  definition).
- Default `false`: no extra writes, byte-identical assembly semantics.

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

**Core hybrid path (MEMG-11, opt-in).** `RecallConfig { core_search: true, .. }` ranks the
L1 pool with the core `Embedded::search` — BM25 text arm + HNSW vector arm + planner RRF —
instead of the in-memory dual-pool. The D38 promise is preserved (the text arm always runs,
even in `Embedding` mode: a vectorless record still ranks through BM25; tested by
`vanta-memory/tests/recall_core_hybrid.rs`) and the same gates apply (quarantine excluded,
VER-04 injection ACL, D22 agent/team scope across `l1/*` namespaces). Documented divergences:
`min_overlap` is not applied (BM25 relevance replaces the overlap gate); cross-namespace
merging orders by the per-namespace RRF score (approximate across namespaces); `semantic_ran`
reports the embedding arm *executed* (non-empty query vector). Default `false` keeps the
legacy path byte-identical — flip the flag to roll forward or back during the migration.

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
| `task_checkpoints` | checkpoints de tarea reanudables (MEMG-20; un record por `task_id`) |
| `genlog/<session>` | provenance de generaciones (best-effort, cap 100) |
| `dream/<session>/<run_id>` | vista consolidada por corrida (MEM-61; `discard` real; `promote` real con dry-run/gate — VER-07) |
| `erasure/dek` | DEKs por scope wrapped con el master key (MEMG-17; feature `erasure`) |
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
2. **D21 — `TokenEstimator`**: paid by the ADR-0029 amendment + BND-03 (`784b27b9`).
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

See ADR-0029 for the full D21–D23 rationale and `API-STD-15` for the Gate P decision
(core-only + stable Rust API).
