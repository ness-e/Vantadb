---
title: "ADR-0063: Home of the host wiring — the FIND-112 runner surface moves to vanta-memory"
kind: adr
status: accepted
description: "The env/TOML runner surface both server hosts need lives in the binary crate vantadb-server, which vantadb-mcp cannot depend on (Cargo cycle). It moves by owner into vanta-memory (runner_config + services::scheduler); wire_memory stays in the server"
tags: [vantadb, architecture, adr, vanta-memory, mcp, server, crate-boundary]
created: "2026-11-25"
owner: vanta-arch
---

# ADR-0063: Home of the host wiring — the FIND-112 runner surface moves to `vanta-memory`

## Status

Accepted. This record decides the shape for the `WIRE-19` row of
[docs/dev/Backlog.md](../../Backlog.md) and unblocks `MCP-43` (scheduler inside
`--mcp`), which must not start until this ADR is approved. It **does not
supersede** [ADR-0054](./ADR-0054-scheduler-host-vantadb-server.md): the host
stays `vantadb-server` (HTTP and `--mcp`); only the *configuration surface*
both hosts resolve moves. Implementation is a separate task; this ADR contains
no product code.

Reviewed P2-01 by `vanta-review` on 2026-11-25 (round 1); the findings are
folded into this revision — see §Review.

## Contents

This record runs past the usual 300-line budget because every section answers a
review finding, and an ADR is atomic: splitting it would create two sources of
truth for one decision. The map:

[Context](#context-and-problem-statement) ·
[Options](#considered-options) ·
[Decision](#decision-outcome) ·
[Hyrum surface](#hyrum-surface) ·
[Compatibility](#compatibility) ·
[Migration surface](#migration-surface-executable-without-further-decisions) ·
[Consequences](#consequences) ·
[Verification](#verification) ·
[Review](#review) ·
[References](#references)

## Context and problem statement

ADR-0054 made `vantadb-server` the home of the memory scheduler and deferred
`--mcp` as a second host of **the same binary**. That second host now has a
task (`MCP-43`), and with it a mechanical question the first host never had to
answer: *where does the code that turns the operator's env + TOML into a
runner factory live, so that both hosts call one copy?*

Today it lives in `vantadb-server/src/scheduler.rs`. `vantadb-mcp` cannot reach
it.

### What is actually duplicated today (measured, not assumed)

The `WIRE-19` row states the risk as *"obligaría a duplicar el mapeo de
proveedores y las reglas R-5 de secretos en dos hosts"*. That is wrong about
what would be duplicated and right about the consequence. Checked against
`HEAD`:

| Piece | Where it lives today | Duplicated? |
|---|---|---|
| Provider → runner mapping (`IngestRunnerProvider` → `ConcreteRunner`) | `vanta-memory/src/ingest/runner_config.rs:110,259-284` | **No** — already shared; both hosts call `build_ingest_runner` |
| Secret read (`VANTADB_OPENAI_API_KEY`, env-only, never TOML) | `runner_config.rs:216-217` inside `apply_env` (at `HEAD`; the `:63` hit is a doc string, and `ingest/observability.rs:225` is a help string in an *untracked* file of `MEM-72`) | **No** — already shared, and already in the crate the ticket wants to move things to |
| Env overrides (`VANTADB_INGEST_*`, inherited `VANTADB_LLM_*`) | `runner_config.rs:170-222` | **No** |
| TOML path resolution (`VANTADB_INGEST_CONFIG` → `<storage>/data/vanta-ingest.toml`) | `vantadb-server/src/scheduler.rs:78-96` **and** `vantadb-mcp/src/wiki.rs:388-395` | **Yes — two copies that do not agree** |
| Pass interval parse (`VANTADB_SCHEDULER_INTERVAL_MS`) | `vantadb-server/src/scheduler.rs:50-74` | Single copy today; a second host copies it |
| P4 degradation mapping (`Some(ConcreteRunner::None) → None`) | `vantadb-server/src/scheduler.rs:108-115` | Single copy today; a second host copies it |

So the surface at risk is the six host-facing items in
`vantadb-server/src/scheduler.rs:44-115` (~40 lines of code plus doc comments),
not the provider mapping and not the secret rules. Those were already
centralised when `IngestRunnerCfg` was built (`FIND-112`). The consolidation is
smaller than the ticket implies and the *drift* is larger than the ticket
implies, because one of the three pieces already exists twice:

- server: `Path::new(storage_path).join("data").join("vanta-ingest.toml")`
  (`scheduler.rs:81-84`);
- mcp: `storage.data_dir().join("vanta-ingest.toml")` (`wiki.rs:394`).

For a disk backend the engine sets `data_dir = base_path.join("data")`
(`src/storage/engine/init.rs:459`) with `base_path = PathBuf::from(path)`
(`init.rs:313-315`, no canonicalisation, no `~` expansion), so the two agree
**by accident, byte for byte**. For `BackendKind::InMemory` `init_storage`
returns `PathBuf::new()` (`init.rs:425-428`), so `wiki_ingest` resolves a
**CWD-relative** `vanta-ingest.toml` while the scheduler resolves
`<root>/data/vanta-ingest.toml`. Same operator file, two answers, one of them
CWD-dependent. That is the drift class `FIND-704` already produced once with a
model list, and it is live right now — not hypothetical.

### The binding constraint is a Cargo cycle, not "a binary cannot be a dependency"

`vantadb-server` **does** have a library target (`vantadb-server/src/lib.rs`,
12 lines, `pub mod scheduler`) and `vantadb-server/src/main.rs:88-94` consumes
it as a library. So the naive blocker is false, and a design built on it would
be wrong. The real blockers, in order of weight:

1. **Cycle.** `vantadb-server/Cargo.toml:31` depends on `vantadb-mcp`, so
   `vantadb-mcp → vantadb-server` closes a dependency cycle. Both crates are
   `publish = false`, so no registry-mediated version escape exists either.
2. **Wrong seam.** The only host-bound item in that file is `wire_memory`
   (`scheduler.rs:124-146`), typed on
   `vantadb::cli_server::ServerHooks` — the *HTTP* bootstrap hook. The MCP
   stdio path (`vantadb-mcp/src/server.rs:354`) has no `ServerHooks`.
3. **Same process, same env.** `--http` and `--mcp` are branches of the *same
   binary* (`vantadb-server/src/main.rs:57,60-72,75-99`). `VANTADB_SCHEDULER_INTERVAL_MS`
   must mean one thing in that process; two parsers is a semantic fork waiting
   to happen.

### What must not move

Per `.opencode/rules/open-core-licensing.md` (core stays Apache-2.0, Pro stays
outside the workspace) and R-5 of `.opencode/rules/core-engine.md` (single
`VANTADB_*` prefix, warn + documented default, never panic), plus the
`FIND-112` G0 rule (*"the TOML never carries a key"*):

- **No new env name**, no rename, no alias, and **one read site per variable**:
  the per-host *default* is expressed as an `Option` returned from that single
  site (D4), not as a second `env::var` call. R-5 forbids the duplicate read,
  not a documented default.
- **No new read site for a credential.** `VANTADB_OPENAI_API_KEY` stays read at
  `vanta-memory/src/ingest/runner_config.rs:216-217`. The only variable this
  ADR moves is `VANTADB_INGEST_CONFIG`, which is a **path**, not a secret.
- **No new dynamic dispatch.** ADR-0054 §Forma chose `S1` delegation over
  `Box<dyn>` (`runner_config.rs:82-88`); this ADR preserves that.

## Considered options

### O1 — place each item with the owner that already exists in `vanta-memory` (chosen)

The runner side joins `ingest/runner_config.rs`, which already owns
`from_toml_str` / `apply_env` / `from_env_toml` / `build_runner`. The interval
side joins `services/scheduler.rs`, where the loop it drives lives.

- **Pros:** no new module; every item lands next to the code that gives it
  meaning (`from_storage_path` beside `from_env_toml`, `parse_interval_ms`
  beside `spawn_memory_scheduler`); the call site improves from a free function
  to a constructor (`IngestRunnerCfg::from_storage_path(&path)`) and a method
  (`cfg.runner_factory()`); `vantadb-mcp` already depends on this crate
  (`vantadb-mcp/Cargo.toml:27`), so the MCP host gets it free; `vanta-proxy`,
  the third dependent (`vanta-proxy/Cargo.toml:34`), inherits it with no
  manifest change.
- **Cons:** seven new public items in a **publishable** crate
  (`vanta-memory/Cargo.toml:12-15`, DIST-01 — the crate has no `publish = false`)
  → semver and Hyrum surface for a crate with no release yet (release-plz on
  hold, so the window is open now and closes at first publish). Two files
  touched instead of one.
- **Cost:** ~40 LOC moved plus its 4 tests, in 2 existing files.

### O2 — one new module `vanta-memory/src/ingest/host.rs` for all seven

- **Pros:** "one file, one review, one table row"; the whole host surface is
  greppable by module name.
- **Cons:** same seven public items as O1, *plus* one `pub mod` line, and it puts
  a scheduler interval parser inside `ingest`. It buys tidiness with two
  self-inflicted costs (a new module to name and document; a file that mixes
  two concerns) and saves nothing O1 does not already have.
- **Rejected:** strictly dominated by O1. Recorded because "just make a
  `host` module" is the first thing a reader will propose.

### O3 — a new intermediate crate (`vanta-host`)

Both hosts depend on a crate whose whole content is those ~40 lines.

- **Pros:** no growth on `vanta-memory`'s published surface; a neutral home if
  a fourth host ever appears.
- **Cons:** a workspace member (root `Cargo.toml:786-798`), a version to bump, a
  `license`/`description`/`keywords`, a docs.rs entry, a `cargo-deny` and
  `cargo-machete` surface, a docs page, and a release-ordering question — all
  to avoid a move that is already acyclic. Its only real dependency would be
  `vanta-memory` (the seven items reference std + `runner_config` and nothing
  from `vantadb`), so the graph gains an edge to save an edge. ADR-0031 exists
  precisely to price this kind of promotion.
- **Rejected:** a crate that exists only to hold a move is scaffolding, not
  architecture. Revisit only if a second consumer of the surface appears that
  cannot depend on `vanta-memory`.

### O4 — injectable trait (or factory parameter) supplied by the host at `run_stdio_server*`

- **Pros:** no crate change at all; the wiring is composed exactly once, at the
  entry point.
- **Cons:** `run_stdio_server` (`vantadb-mcp/src/server.rs:354`) takes an
  already-open `Arc<StorageEngine>`, so the host would have to pass the factory
  in — a signature change on a **`pub`** function (`vantadb-mcp/src/lib.rs:60`)
  that `vantadb-server` and any out-of-tree embedder must follow, to save ~40
  lines. The factory is generic (`Fn() -> Option<R>`, `R: LlmRunner`), so the
  trait method either boxes (`dyn`) — contradicting ADR-0054 `S1` — or forces a
  concrete runner type into the signature. And the standalone default still
  needs an implementation *inside* `vantadb-mcp`: the copy returns, labelled
  "fallback". Per Seemann, *"Only applications should have Composition Roots.
  Libraries and frameworks shouldn't"* — O4 imports an application concern into
  a library.
- **Rejected.**

### O5 — a feature of the binary (`vantadb-server --mcp` passes the wiring in)

- **Pros:** today the only in-tree path into MCP *is* `vantadb-server --mcp`
  (`main.rs:72`), so the entry point already has the config.
- **Cons:** the entry point cannot build a runner factory before the engine
  opens — writer discovery happens inside `run_stdio_server_auto`
  (`server.rs:993-1059`), and on `DatabaseBusy` the process becomes a *proxy*
  where no scheduler may run. So the factory would have to be a
  lazily-resolved closure threaded through the fallback path: O4's signature
  change, plus a lifetime question. And the copy returns for any embedder that
  does not go through this binary.
- **Rejected as the home.** Its good instinct — *compose at the entry point* —
  is preserved: `wire_memory` (the composition) stays in the server.

### O6 — copy the ~40 lines into `vantadb-mcp` (status quo trajectory)

- **Pros:** zero design work; each host owns its own glue.
- **Cons:** this is what already produced the two disagreeing
  `ingest_toml_path` copies. The copy is *safe today* only because
  `vanta-memory` owns the parts that must not diverge (provider mapping,
  secrets); the parts that would be copied — path resolution, interval parse,
  P4 mapping — are exactly the parts that must not diverge either, because
  `--http` and `--mcp` are the same process with the same env. `FIND-704` is
  the precedent: a copied list that drifted silently.
- **Rejected.**

### O7 — `#[path = "../vantadb-server/src/scheduler.rs"]`

- **Pros:** one file, two owners, no move.
- **Cons:** it does not even compile — the file's `wire_memory` imports
  `vantadb::cli_server::ServerHooks` and its module doc refers to the HTTP
  bootstrap. Worse, it breaks `cargo package -p vanta-memory`: Cargo packages
  files under the package root (`include`/`exclude` default to *"all files from
  the root of the package"*,
  [Cargo Book §The Manifest Format](https://doc.rust-lang.org/cargo/reference/manifest.html)),
  so a `#[path]` reaching into `vantadb-server/` yields a crate that cannot be
  packaged — and `vanta-memory` is the one crate here that must package.
- **Rejected.**

## Decision outcome

Chosen: **O1**, in five rules.

### D1 — the host-facing surface moves by owner, and the host-visible names are preserved

| Item | Signature | Destination |
|---|---|---|
| `ingest_toml_path` | `pub fn ingest_toml_path(storage_path: &str) -> PathBuf` | `ingest::runner_config` (beside `from_env_toml`) |
| `IngestRunnerCfg::from_storage_path` | `pub fn from_storage_path(storage_path: &str) -> Self` | `ingest::runner_config` (**new** assoc fn; resolves the path, then delegates to `from_env_toml` — pure delegation) |
| `IngestRunnerCfg::runner_factory` | `pub fn runner_factory(&self) -> impl Fn() -> Option<ConcreteRunner> + Send + Sync + 'static` | `ingest::runner_config` (**new** method; `build_runner` + the P4 mapping, moved verbatim) |
| `DEFAULT_INTERVAL_MS` | `pub const DEFAULT_INTERVAL_MS: u64 = 60_000` | `services::scheduler` (beside the loop) |
| `parse_interval_ms` | `pub fn parse_interval_ms(raw: Option<&str>) -> u64` | `services::scheduler` |
| `scheduler_interval_ms_from_env_opt` | `pub fn scheduler_interval_ms_from_env_opt() -> Option<u64>` | `services::scheduler` (**new**; the single env read, `None` = unset) |
| `scheduler_interval_ms_from_env` | `pub fn scheduler_interval_ms_from_env() -> u64` | `services::scheduler` (now `_opt().unwrap_or(DEFAULT_INTERVAL_MS)`) |

`resolve_ingest_toml_path` stays **private** in `ingest::runner_config` — it is
an implementation detail of `ingest_toml_path` and never needed a public name.
`services::scheduler` is an ungated `pub mod` (`vanta-memory/src/services/mod.rs:16`);
only the loop helper inside it is `#[cfg(feature = "http-server")]`, so the
interval parsers are available to any build — which is what `vantadb-mcp` needs
before it enables `http-server` in `MCP-43`.

**What is preserved, precisely.** The *resolver semantics* move byte-identically
(`resolve_ingest_toml_path`, `ingest_toml_path`, `parse_interval_ms` and the P4
mapping keep their bodies and their signatures). The host-visible **names** are
preserved by the D3 shims. Two wrappers are genuinely new —
`from_storage_path` and `runner_factory` — and they are pure delegation
(`IngestRunnerCfg::from_storage_path(p)` is `from_env_toml(&ingest_toml_path(p))`;
`cfg.runner_factory()` is the moved P4 closure). So the move's failure modes are
"forgot a call site" **and** "got a wrapper wrong", and both are covered by the
verification block rather than by the shape of the diff. An earlier revision
claimed "signatures do not change, exactly one failure mode"; that was true only
of the resolver half and is corrected here.

Two call-site consequences, both one-line:

- server: `IngestRunnerCfg::from_storage_path(&storage_path)` where the cfg was
  loaded (`vantadb-server/src/main.rs:89`);
- mcp: `IngestRunnerCfg::from_storage_path(&storage.config().storage_path)`
  (`src/storage/engine/mod.rs:505`, a public accessor since `FIND-656`).
  `storage.data_dir()` is deliberately **not** the parameter: it is already
  `<root>/data`, one level too deep for a rule that appends
  `data/vanta-ingest.toml`. This is what removes the divergence: one rule,
  `<storage>/data/vanta-ingest.toml`, for both hosts and every backend kind.

### D2 — `wire_memory` stays in `vantadb-server` (and why, precisely)

`wire_memory` (`scheduler.rs:124-146`) is a *composition*, not configuration:
it builds the queue, injects the `HttpCaptureBridge` as the server's
`ConversationTrigger`, and pushes the `MemoryScheduler` into
`ServerHooks::background_services`.

The reason it stays is **host neutrality, not a cycle**: `vanta-memory`
*already* implements the core's server traits — `ConversationTrigger` for
`HttpCaptureBridge` (`services/conversation_hook.rs:47`) and
`BackgroundService` for `MemoryScheduler` (`services/scheduler.rs:216`) — under
`http-server = ["vantadb/server", …]` (`vanta-memory/Cargo.toml:96`). Moving
`wire_memory` would therefore compile. It would still be wrong: it is the
application's composition (queue + trigger + service registration), it is typed
on the HTTP bootstrap struct, and a library that composes itself is a
Composition Root in the wrong place. ADR-0054 constraint 3
(*"el core **no puede** alojar el pipeline"*, `ADR-0054:53-56`) is about the
core, not about this edge — citing it as a cycle was the first revision's
error, corrected here.

`MCP-43` needs nothing from this ADR beyond D1 and D4: `vanta-memory` already
exposes the host-agnostic half of the composition,
`spawn_memory_scheduler(queue, db, interval, factory) -> MemoryScheduler`
(`services/scheduler.rs:256-267`, `http-server`).

### D3 — the compatibility re-export goes **inside** `scheduler.rs`, not `lib.rs`

`vantadb-server/src/lib.rs` declares `pub mod scheduler;` (a file module), so a
`pub use` written in `lib.rs` would create `vantadb_server::ingest_runner_cfg`,
**not** `vantadb_server::scheduler::ingest_runner_cfg`. The re-export must be a
line in the body of `vantadb-server/src/scheduler.rs`, and it must cover **all
six** names the old module exported (five `pub fn` plus the const):

```rust
pub use vanta_memory::ingest::runner_config::{IngestRunnerCfg, ingest_toml_path};
pub use vanta_memory::services::scheduler::{
    DEFAULT_INTERVAL_MS, parse_interval_ms, scheduler_interval_ms_from_env,
};

// `ingest_runner_factory` and `ingest_runner_cfg` were free functions here and
// are the only names that cannot be `pub use`d (the factory is now a method).
// Both shims carry a `///` doc: this crate is `#![warn(missing_docs)]`
// (`vantadb-server/src/lib.rs:1`) and CI runs clippy with `-D warnings`.

/// Load the ingest runner config from env + TOML (FIND-112 surface).
pub fn ingest_runner_cfg(storage_path: &str) -> IngestRunnerCfg {
    IngestRunnerCfg::from_storage_path(storage_path)
}

/// Production runner factory, built from `cfg`; a degraded config maps to `None` (P4).
pub fn ingest_runner_factory(
    cfg: IngestRunnerCfg,
) -> impl Fn() -> Option<vanta_memory::ingest::runner_config::ConcreteRunner> + Send + Sync + 'static {
    cfg.runner_factory()
}
```

That keeps every name the crate exported before resolving on the old path,
which is the whole point: `vantadb-pro` cannot be inspected from this repo, so
the move must be **additive** for it. The factory shim compiles as written —
`IngestRunnerCfg` derives `Clone` (`runner_config.rs:64`) and the crate is
edition 2021 (`Cargo.toml:821`), so the returned `impl Fn` does not capture
`&cfg`. Nothing in CI would catch a regression here —
`vantadb-server/tests/scheduler_e2e.rs:28` imports `wire_memory`, which does not
move — so the ADR records the check as a verification line covering **all six**
names.

### D4 — the interface `MCP-43` consumes, and the two guards it must honour

`MCP-43` wires inside `run_stdio_server` (`vantadb-mcp/src/server.rs:354`),
whose parameter is an **already-open engine**: the writer branch of
`run_stdio_server_auto` (`server.rs:993-1059`) is the only caller, and on
`DatabaseBusy` the process becomes a proxy via `run_proxy_stdio_server`
(`server.rs:965,1029`) — where this function is never reached. The
single-writer invariant of ADR-0054 §Decision is therefore a property of the
call graph, not a flag a future edit can get wrong. (`run_stdio_server` is
`pub`, `vantadb-mcp/src/lib.rs:60`, so say "only in-tree caller" — an
out-of-tree embedder could pass any engine.)

Two guards are part of this decision because each one decides whether a
background loop runs at all:

1. **`read_only`.** The HTTP host only wires when `!config.read_only`
   (`vantadb-server/src/main.rs:84`); stdio has the same state live
   (`server.rs:363`). `MCP-43` must guard with `!storage.config().read_only`,
   or a read-only MCP process will run ingest passes that can only fail. A
   read-only process is never the writer, so the guard is consistent with
   ADR-0054.
2. **The one-shot host.** `vanta-cli mcp-call` spawns `vantadb-server --mcp`,
   does one roundtrip and exits (`src/cli_handlers/mcp_call.rs:3,262` — ADR-0054
   constraint 4 called it out by name). `spawn_memory_scheduler` runs its first
   pass *immediately* (`services/scheduler.rs:264-266`), so an
   interval-driven loop would fire **one ingest pass per terminal hook**,
   against `mcp-call`'s `--timeout`. Decision: the MCP host spawns the loop
   **only when `VANTADB_SCHEDULER_INTERVAL_MS` is explicitly set and non-zero**.

   "Explicitly set" needs a value that can say *absent*, and a `u64` cannot —
   `parse_interval_ms(None)` already returns `60_000` by construction. The
   primitive therefore becomes `Option`, read **once, in one module**:

   ```rust
   // services::scheduler, ungated — the only env read for this variable
   pub fn scheduler_interval_ms_from_env_opt() -> Option<u64>;   // None = unset
   pub fn scheduler_interval_ms_from_env() -> u64 {              // HTTP daemon
       scheduler_interval_ms_from_env_opt().unwrap_or(DEFAULT_INTERVAL_MS)
   }
   ```

   MCP then does `match scheduler_interval_ms_from_env_opt() { None | Some(0) => off, Some(ms) => ms }`.
   Both hosts keep one read site and one parser: R-5's *must-not* is about
   reading a variable by hand at each site, not about a documented per-host
   default. The HTTP daemon's effective default is unchanged at `60_000`.

The shape, once both guards hold (note the cfg is built **inside** the guard, so
a read-only or disabled process never reads env/TOML for a config it will not
use):

```rust
if !storage.config().read_only {
    if let Some(ms) = scheduler_interval_ms_from_env_opt().filter(|ms| *ms > 0) {
        let cfg = IngestRunnerCfg::from_storage_path(&storage.config().storage_path);
        let queue = Arc::new(LocalStateBackend::new(SystemClock));
        let sched = spawn_memory_scheduler(queue,
                                           Embedded::from_engine(storage.clone()),
                                           Duration::from_millis(ms), cfg.runner_factory());
        // drained at stdin EOF
    }
}
```

### D5 — no feature gate on any moved item

The seven items are pure path/env/TOML logic: no Tokio, no `ServerHooks`, no
`reqwest`. `ConcreteRunner`, `build_ingest_runner` and `StandaloneLlmRunner`
carry no `cfg(feature)` in `runner_config.rs` (only `#[cfg(test)]` at `:323`),
so `runner_config` and the interval parsers compile in the default build.
`vantadb-mcp` reaches all of it with `llm-driver` alone
(`vantadb-mcp/Cargo.toml:27`) — the `http-server` addition belongs to `MCP-43`,
which is the task that first needs the loop.

### Hyrum surface

What this ADR commits, and what it explicitly does not:

| Observable | Guarantee |
|---|---|
| Path rule | `<storage_path>/data/vanta-ingest.toml`, overridden by `VANTADB_INGEST_CONFIG` when non-blank after `trim()` |
| Missing/unreadable TOML | not an error: defaults + `warn!` (never panic) — R-5 |
| Unparseable `VANTADB_SCHEDULER_INTERVAL_MS` | `warn!` + `DEFAULT_INTERVAL_MS`; `0` is valid and means "loop off" |
| Secrets | env-only. The TOML schema (`TomlFile`, `runner_config.rs:287+`) has no key field; `openai_api_key` is set only to `None` in `defaults()` (`:123`) and from env in `apply_env` (`:216-217`); it reaches the runner only through `standalone_cfg` (`:279`) |
| Degraded config | `runner_factory()` maps `Some(ConcreteRunner::None)` → `None` so the pass skips observably (P4) |
| Not guaranteed | the TOML schema's field set, the factory's concrete type (it is `impl Fn`), and whether a future host adds a second config source |

### Compatibility

- **`vantadb-pro`:** additive by construction — D3 keeps every old path
  resolving, and no env name, path rule or secret read site changes. Nothing to
  port. (Unverifiable from this repo either way, which is why D3 exists.)
- **Semver:** `vanta-memory` gains seven public items → a minor bump under 0.x,
  and it has no published release yet (DIST-01 hold), so today it is free.
  `vantadb-server` is `publish = false`; its lib is not a contract, hence the
  re-export rather than a bare deletion.
- **Docs touched by the move (R-4: cite real paths):**
  `docs/user/operations/CONFIGURATION.md:136` (`VANTADB_SCHEDULER_INTERVAL_MS`)
  and `:137` (`VANTADB_INGEST_CONFIG`) both name
  `vantadb-server/src/scheduler.rs`. `docs/api/VANTA_MEMORY.md` cites
  `scheduler.rs:124-146` at `:284`, which is `wire_memory` and does **not**
  move — no edit there.

## Migration surface (executable without further decisions)

Ordered; the worker executes top to bottom. `MCP-43` follows this table, and
runs **after** `MEM-72` (which also edits `vanta-memory/src/ingest/`).

| # | File | Today | After | If missed |
|---|---|---|---|---|
| 1 | `vanta-memory/src/ingest/runner_config.rs` | `impl IngestRunnerCfg { from_toml_str, apply_env, from_env_toml, build_runner, standalone_cfg }` | add `use std::path::{Path, PathBuf};` (already imported for `from_env_toml`), the private `fn resolve_ingest_toml_path(override_path: Option<&str>, storage_path: &str) -> PathBuf`, the public `ingest_toml_path` (verbatim from `scheduler.rs:87-96`), `IngestRunnerCfg::from_storage_path`, and `IngestRunnerCfg::runner_factory` (verbatim P4 mapping from `scheduler.rs:103-115`). Move the 3 unit tests that cover them | nothing compiles in steps 2-5 |
| 2 | `vanta-memory/src/services/scheduler.rs` | loop helper only | add `DEFAULT_INTERVAL_MS`, `parse_interval_ms` and `scheduler_interval_ms_from_env_opt` **verbatim** (`scheduler.rs:44-46,48-65,67-74`), rewrite `scheduler_interval_ms_from_env` as `_opt().unwrap_or(DEFAULT_INTERVAL_MS)`, and move the `parse_interval_ms_defaults_and_validates` test. All of it **above** the first `#[cfg(feature = "http-server")]` item — they must stay ungated. Reword the `DEFAULT_INTERVAL_MS` doc: it says "mirrors `VANTADB_TTL_SWEEP_INTERVAL_MS` (`src/config.rs`)", which is the *core's* file — from this crate it must read `vantadb/src/config.rs` (R-4) | the interval parser has no ungated home; `MCP-43` cannot express "unset"; the moved doc cites a path that does not exist in this crate |
| 3 | `vantadb-server/src/scheduler.rs` | L44-46, L48-65, L67-74, L76-85, L87-96, L98-101, L103-115 + `mod tests` L148-192 | **delete those ranges**; keep the module doc (L1-31, re-worded: "the host composes; the surface lives in `vanta_memory`"), `wire_memory` L117-146, and add the D3 block: two `pub use` lines plus **two** shims (`ingest_runner_cfg`, `ingest_runner_factory`) — six exported names total. Trim imports to `ServerHooks`, `Embedded`, `HttpCaptureBridge`, `spawn_memory_scheduler`, `LocalStateBackend`, `SystemClock`, `LlmRunner`, `Arc`, `Duration` | duplicate definitions → `cargo check -p vantadb-server` fails on ambiguous items; a missing shim silently breaks `vantadb_server::scheduler::*` |
| 4 | `vantadb-server/src/main.rs:88-94` | `vantadb_server::scheduler::{scheduler_interval_ms_from_env, ingest_runner_cfg, ingest_runner_factory}` | `scheduler_interval_ms_from_env()` unchanged (D3 re-export); replace the cfg+factory pair with `let cfg = IngestRunnerCfg::from_storage_path(&storage_path);` then pass `cfg.runner_factory()` to `wire_memory` | the canonical home is invisible to readers → the copy returns |
| 5 | `vantadb-mcp/src/wiki.rs:385-395` | private `fn ingest_toml_path(storage: &Arc<StorageEngine>) -> PathBuf` | **delete the function**; `:291` becomes `IngestRunnerCfg::from_storage_path(&storage.config().storage_path)`; drop `PathBuf` from `use std::path::PathBuf;` (`:32`) if nothing else in the file uses it (`cargo check` decides) | the InMemory divergence survives; the two-copy drift closes only on paper |
| 6 | `docs/user/operations/CONFIGURATION.md:136` | "… (`vantadb-server/src/scheduler.rs`)" for `VANTADB_SCHEDULER_INTERVAL_MS` | "… (`vanta-memory/src/services/scheduler.rs`)" | R-4 violation on the interval row |
| 7 | `docs/user/operations/CONFIGURATION.md:137` | "… (`vantadb-server/src/scheduler.rs`)" for `VANTADB_INGEST_CONFIG` | "… (`vanta-memory/src/ingest/runner_config.rs`)" | R-4 violation on the config row |

Rows 6 and 7 are two rows because they are two different owners after the move
— which is the O1 trade-off made visible.

Explicitly **not** in the table: `vantadb-mcp/Cargo.toml` gains
`vanta-memory/http-server` — that belongs to `MCP-43`, because step 5 uses only
ungated code. And `llm-driver` in `vantadb-server/Cargo.toml:37` **stays** even
though the server stops naming `ConcreteRunner`: it is what makes the runner
constructed inside `vanta-memory` a real HTTP client rather than
`NotConfigured` (`docs/dev/strategy/VantaDB-Informe-Analisis-Completo.md:159`).
Verify with `cargo machete` rather than deleting it.

## Consequences

### Positive consequences

- **One rule, one place.** The `<storage>/data/vanta-ingest.toml` rule and the
  `VANTADB_INGEST_CONFIG` override stop existing twice, and the
  `InMemory`/CWD divergence disappears with them.
- **The env surface of a single process cannot fork.** `--http` and `--mcp` are
  the same binary; `VANTADB_SCHEDULER_INTERVAL_MS` now has one parser for both.
- **Secrets move nowhere.** The credential read site
  (`runner_config.rs:216-217`) is untouched; the only moved variable is a path.
  Net effect: the number of call sites that resolve *which config file to
  honour* drops from two to one, and `vantadb-mcp` stops reading
  `VANTADB_INGEST_CONFIG` itself.
- **No new module, and the call sites get better**: a constructor
  (`from_storage_path`) and a method (`runner_factory`) replace two free
  functions threaded through the host.
- **`MCP-43` becomes a wiring task, not a design task**, and `vanta-proxy` — the
  third dependent — inherits the surface without a manifest change.
- **No new crate, no new trait, no `dyn`, no breaking signature.** Reversible
  by `git revert` of one commit.

### Negative consequences

- **`vanta-memory`'s published surface grows by seven items** for callers that
  only want the scheduler. They are cheap to document but permanent once
  released (no release yet, so the window is open now and closes at first
  publish). Mitigation: they live on types/modules that already carry the
  concept, and the Hyrum table above states exactly what is and is not
  guaranteed.
- **`vanta-memory` gains knowledge of the on-disk layout** (`<root>/data`),
  which the engine owns (`src/storage/engine/init.rs:459`). It is one `join`,
  but it is a second place that knows the layout — now in the module that
  already owns the TOML. The alternative (a `data_dir: &Path` parameter) would
  force the server to derive it from a path or from the `#[doc(hidden)]`
  `engine_handle()`, trading a visible `join` for a hidden accessor.
- **A compatibility re-export keeps two names alive** — wrong, and corrected
  during review: D3 preserves **all six** names the module exported (two
  `pub use` lines plus the two shims, since a `pub use` cannot express a free
  function that changed shape). The cost is that `scheduler.rs` still answers
  for a surface it no longer owns. Cheap and deliberate: the alternative is
  declaring a break on a crate `vantadb-pro` consumes and this repo cannot read.
  The cost of getting this wrong is silent
  (`tests/scheduler_e2e.rs:28` only imports `wire_memory`), so it is a
  verification line, not a habit.
- **The interval default becomes host-dependent**: `60_000` for the HTTP daemon,
  "off" for the MCP host unless the operator sets the variable. It is the same
  env var with the same parsing, but the *documented default* now depends on
  which host reads it. Declared in D4; the alternative is one ingest pass per
  `mcp-call` roundtrip.
- **One behaviour changes for `BackendKind::InMemory` in the MCP host**: the
  ingest TOML is now looked up at `<storage_path>/data/` instead of relative to
  the process CWD. No test pins either behaviour, and the new answer is the one
  the HTTP host already gave.
- **Two files to review instead of one**, and rows 6/7 of the migration split by
  owner. That is the price of O1 over O2.

## Verification

Design checks (runnable now, from the repo root):

```bash
A=docs/dev/architecture/adr/ADR-0063-host-wiring-home-vanta-memory.md
rg -c '^## Context and problem statement$' "$A"   # 1
rg -c '^## Considered options$'              "$A"   # 1
rg -c '^## Decision outcome$'                "$A"   # 1
rg -c '^### Positive consequences$'          "$A"   # 1
rg -c '^### Negative consequences$'          "$A"   # 1
rg -c '^### O[0-9]+ '                        "$A"   # 7 — not a single-option ADR
node scripts/docs/check-docs.mjs && node scripts/docs/check-links.mjs
```

Falsifiable claims of this ADR, each with its command (all numbers are `HEAD`,
i.e. read with `git show HEAD:<path>`; the working tree may carry other lanes'
in-flight edits):

```bash
# 1-2. The secret read site and the TOML schema (no key field).
git show HEAD:vanta-memory/src/ingest/runner_config.rs | rg -n 'VANTADB_OPENAI_API_KEY|openai_api_key'
#    → :216-217 the read; :79 decl, :123 None in defaults(), :263 match, :279 standalone_cfg

# 3. The cycle claim.
rg -n 'vantadb-mcp' vantadb-server/Cargo.toml     # :31 a real dependency
rg -n 'vantadb-server' vantadb-mcp/Cargo.toml     # :11 a comment — NO dependency line

# 4. The divergence claim.
rg -n 'join\("data"\)|data_dir' vantadb-server/src/scheduler.rs vantadb-mcp/src/wiki.rs
git show HEAD:src/storage/engine/init.rs | rg -n 'base_path.join\("data"\)|PathBuf::new\(\)'
#    → :459 (disk) and :428 (InMemory)

# 5. The surface that moves (identical before and after this analysis —
#    this ADR changes no code). Measured on 2026-11-25: 49 → 49.
rg -n 'VANTADB_INGEST|VANTADB_LLM|ingest_toml_path|ingest_runner_factory' --glob '*.rs' | wc -l
```

Implementation checks (belong to the follow-up task):

- `cargo check -p vanta-memory -p vantadb-server -p vantadb-mcp` — no feature
  matrix change in this step; `vantadb-mcp` still builds with `llm-driver` alone.
- `cargo nextest run --profile audit -p vanta-memory -E 'test(ingest) or test(interval)'`
  — the four moved tests pass in their new homes, none of them changes.
- `cargo nextest run --profile audit -p vantadb-mcp` — the wiki ingest tests
  (`ingest_local_no_model_degrades`, `ingest_ollama_down_degrades`,
  `ingest_openai_no_key_degrades`) still pass, proving the path-rule change for
  the disk backend is behaviour-neutral.
- **The compat check D3 promises:** `cargo test -p vantadb-server --doc` is not
  enough; add one line to an existing `vantadb-server` test that calls **all
  six** names by their old paths —
  `vantadb_server::scheduler::{DEFAULT_INTERVAL_MS, parse_interval_ms, scheduler_interval_ms_from_env, ingest_toml_path, ingest_runner_cfg, ingest_runner_factory}`.
  Without it a future tidy-up deletes a re-export and nothing notices (the only
  in-tree `scheduler::` import is `wire_memory`,
  `tests/scheduler_e2e.rs:28`).
- `rg -n 'fn ingest_toml_path' --glob '*.rs'` — exactly one definition, in
  `vanta-memory`.
- `cargo public-api -p vanta-memory` diff — exactly +7 items, none removed.

## Review

`vanta-review` (P2-01, fresh context, 2026-11-25) — two rounds, both closed.
The full list lives in [WIRE-19](../../tasks/WIRE-19.md) §Review.

**Round 1 — changes required** (4 high, 5 medium, 5 low). The four that changed
the decision were:

- **H1 — the D3 re-export was in `lib.rs`**, which would have published
  `vantadb_server::ingest_runner_cfg` and broken
  `vantadb_server::scheduler::*` for the one consumer the re-export exists for.
  D3 now puts the `pub use` in `scheduler.rs` and records a compat check.
- **H2 — six public items, not five.** Corrected everywhere; the ADR's own
  acceptance check would have failed on its first run.
- **H3 — D2 justified itself with a Cargo cycle that does not exist**:
  `vanta-memory` already implements `vantadb::cli_server::{ConversationTrigger,
  BackgroundService}` (`conversation_hook.rs:47`, `scheduler.rs:216`). D2 is
  rewritten on host neutrality, and the correction is stated in place.
- **H4 — D4 omitted two reachable guards**: `read_only`, and the one-shot
  `vanta-cli mcp-call` host, which would have fired one ingest pass per
  terminal hook. Both are now part of the decision.

One recommendation was adopted as a better option: *place each item with its
existing owner* instead of a new module — that is now **O1**, and the original
single-module plan is priced as **O2**.

**Round 2 — changes required** (2 high, 2 medium, 4 low), both high closed in
this revision:

- **H-A — the D3 re-export missed `ingest_runner_factory`.** The old module
  exported six names; D3 listed five and claimed "no consumer loses a name". D3
  now has two `pub use` lines plus **two** shims, and the verification line
  exercises all six.
- **H-B — "unset means off" was not implementable, and the fix it proposed
  reintroduced a second `env::var` read** of the same variable in another
  crate, contradicting this ADR's own R-5 bullet. The primitive is now
  `scheduler_interval_ms_from_env_opt() -> Option<u64>`, read once in
  `services::scheduler`, with `scheduler_interval_ms_from_env()` as its
  `unwrap_or(DEFAULT_INTERVAL_MS)` wrapper for the HTTP daemon. That is why the
  surface is **+7**, not +6.

Round 2 also corrected the "verbatim, signatures identical, exactly one failure
mode" claim — false for the two new wrappers — and found a stale `:235` (working
tree) presented as `HEAD` (`:216`). Round 1's H2 said "six, not five"; round 2's
H-B then made it seven. The count is seven.

## References

- Owner direction, articulated in the task row: `WIRE-19` in
  [docs/dev/Backlog.md](../../Backlog.md) (2026-11-23, *"Mover la fábrica y la
  resolución de ruta a `vanta-memory` … DoD: un único constructor de runner,
  usado por HTTP y MCP"*); plan
  [2026-11-23-release-v0.10.0-gates.md](../../plans/2026-11-23-release-v0.10.0-gates.md) §9.b.
- Task file (analysis, decision table, contract): [WIRE-19](../../tasks/WIRE-19.md).
- Host decision this ADR narrows: [ADR-0054](./ADR-0054-scheduler-host-vantadb-server.md)
  (constraint 3 = the core cannot host the pipeline; constraint 4 = the
  one-shot `mcp-call` host; §Forma = `S1` delegation, no `Box<dyn>`).
- Rules: `.opencode/rules/server-mcp.md` (R-1…R-3) ·
  `.opencode/rules/core-engine.md` R-5 (env convention) ·
  `.opencode/rules/api-contract.md` (R-4 real paths, R-7 no `cfg` on `pub`
  fields, R-8 bindings are glue) ·
  `.opencode/rules/open-core-licensing.md`.
- Precedent for the drift this closes: `FIND-704`. Precedent for the shape:
  `FIND-112` (`IngestRunnerCfg`, env-only secrets) and `WIRE-16` (`wire_memory`).
- Code cited (verified on `HEAD` 2026-11-25): `vantadb-server/src/scheduler.rs:44-115,124-146,148-192` ·
  `vantadb-server/src/main.rs:57,72,84,88-99` · `vantadb-server/src/lib.rs` (12 L) ·
  `vantadb-mcp/src/wiki.rs:32,285-307,388-395` · `vantadb-mcp/src/server.rs:354,363,965,993-1059` ·
  `vantadb-mcp/src/lib.rs:60` · `vanta-memory/src/ingest/runner_config.rs:79,110,123,170-237,259-284` ·
  `vanta-memory/src/services/mod.rs:16` · `vanta-memory/src/services/scheduler.rs:216,256-267` ·
  `src/storage/engine/init.rs:313-315,425-428,459` · `src/storage/engine/mod.rs:505` ·
  `src/cli_handlers/mcp_call.rs:3,262` · `src/sdk/builder.rs:100-126` ·
  `vantadb-server/Cargo.toml:31,37` · `vantadb-mcp/Cargo.toml:27` ·
  `vanta-proxy/Cargo.toml:34` · `vanta-memory/Cargo.toml:12-15,96` · `Cargo.toml:786-811`.
- Composition Root: [blog.ploeh.dk/2011/07/28/CompositionRoot](https://blog.ploeh.dk/2011/07/28/CompositionRoot/)
  (fetched 2026-11-25 — *"Only applications should have Composition Roots.
  Libraries and frameworks shouldn't"*).
- Cargo packaging scope: [The Cargo Book — The Manifest Format](https://doc.rust-lang.org/cargo/reference/manifest.html)
  (fetched 2026-11-25 — `include`/`exclude` default to all files under the
  package root).
