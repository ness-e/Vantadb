---
title: "ADR-0065: Ingest queue occupancy is a live probe, and ingest health is four-state"
kind: adr
status: accepted
description: "capabilities.ingest.queue.depth is a value read at answer time (or null), never the last pass's sample; health is unknown|idle|draining|broken, so 'cannot tell' stops reading as 'not broken'"
tags: [vantadb, architecture, adr, vanta-memory, mcp, observability]
created: "2026-11-25"
owner: vanta-arch
---

# ADR-0065: Ingest queue occupancy is a live probe, and ingest health is four-state

## Status

Accepted by `vanta-arch` (2026-11-25). Implemented in `dfb51d807`
([`FIND-748`](../../tasks/FIND-748.md), status `completed`): D1-D7 landed as
written, and **D8 was amended on 2026-11-26** because its example contradicted
the code — the implementer documented the wire and returned the conflict instead
of bending the code to a wrong paragraph (see the amendment inside D8). This
record fixes the contract; if the implementer finds a fact that contradicts
D1-D8, it comes back here as a new ADR instead of deviating quietly. Supersedes
nothing. Extends
[ADR-0054](./ADR-0054-scheduler-host-vantadb-server.md) (the queue's home), which
it does not reopen.

## Context and problem statement

`MEM-73` composed the whole L0→L3 cycle in the shipped binary and then measured
the health surface it had been asserting against
([vanta-memory.md § MEM-73](../../avance/activo/vanta-memory.md), finding 4;
backlog row `FIND-748`).

`capabilities.ingest` is published by
`observability::status()` (`vantadb-mcp/src/handlers/tools.rs:3198`). The queue
block inside it is written in exactly two places, and **both are pass results**:

- `note_pass(...)` (`vanta-memory/src/ingest/observability.rs:184`) stores the
  post-pass depth — the healthy path samples the queue only after a pass.
- `note_skipped_pass(...)` (`:202`) stores the depth of a *skipped* pass.

On the degraded path the value still reads live, because two callers happen to
probe the queue right before publishing (`vanta-memory/src/services/scheduler.rs:363`
in the loop, `vantadb-mcp/src/threads.rs:373` at enqueue time). That accident is
why `MEM-72`'s tests passed. On the **healthy** path there is no such probe, so
between two passes `queue.depth` is a photograph of the previous pass — the field
is documented as `"Queue depth as of the last pass"` (`observability.rs:103`) and
`docs/api/VANTA_MEMORY.md:391` says the same, while `docs/api/MCP.md:656` calls it
`"the authoritative backlog signal"`. Those two documents contradict each other,
and the code sides with the first one.

The consequence is concrete and it is the semaphore's only job:

```rust
pub fn is_broken(&self) -> bool {
    self.degraded && self.queue.depth > 0      // observability.rs:131-133
}
```

`is_broken()` returns `false` in **three** situations that are not the same
thing:

1. a just-enqueued turn on a healthy pipeline, before the next pass samples it
   (the reported lie — `MEM-73`);
2. a pipeline whose scheduler loop was never started, or started and stopped,
   with queued work — `scheduler_running` exists but is not part of the formula;
3. a process that cannot see the queue at all (a read-only writer, a process
   whose loop is disabled), where `depth: 0` describes a queue nobody asked
   about — and, one field over, the one-shot `mcp-call` child that answers about
   its *own* empty queue while the writer's backlog is elsewhere (D8).

`MEM-72`'s blocking finding was the same class one field over
(`scheduler_running` was write-once-true, so a dead loop said "alive"). The
regime this ADR establishes: **a health indicator that lies towards "fine" is
worse than no indicator**, because it is the thing an operator checks *instead*
of looking.

## Considered options

### O1 — read the queue at answer time, in the process that owns it (chosen)

`capabilities` asks the queue it already has in scope
(`vantadb_mcp::threads::memory_queue()`, `threads.rs:255`) at the moment it builds
the answer.

- Pros: the value is true by construction; no second copy of the truth to keep
  in sync; the cost is one mutex acquisition on the same lock every enqueue and
  claim already takes, and can be made strictly cheaper than the sampling the
  scheduler already pays (see measurement below).
- Cons: still process-local — a second process cannot read it (unchanged ceiling,
  declared in D8); requires a reader-side seam so `status()` cannot fabricate a
  number.

### O2 — monotonic counters: `pending = enqueued − completed`

Each mutation bumps a counter; the backlog is derived.

- Pros: O(1), no lock on the queue at all, and monotone so a read can never
  travel backwards.
- Cons: **trades a stale-value lie for a drift lie.** The queue has five exits
  today (`consume_task`, `complete_task`, `fail_task` requeue,
  `claim_stale_tasks`, dead-letter — `vanta-memory/src/utils/local_backend.rs:188-350`);
  one path that forgets to bump makes the counter wrong *forever*, silently, in
  a value that only grows. It is a cache of `vec.len()` with no invalidation
  story. Rejected.

### O3 — keep the snapshot, rename it; publish the backlog elsewhere

Rename `queue` → `last_pass_queue` / `queue_at_last_pass` and add a second field
(or an endpoint) for the real backlog.

- Pros: zero semantic ambiguity about the old value; nothing becomes null.
- Cons: it fixes the *name* and leaves `is_broken()` with no live input, so a
  live signal is still needed somewhere — the rename is then pure churn on a
  published surface **plus** the work of O1. Rejected as a standalone option; its
  good half (the snapshot belongs to the pass) survives as D2.

### O4 — process-global probe registry in `observability`

Hosts register a closure (`fn() -> QueueSnapshot`) at start; `status()` calls it.

- Pros: one place owns the truth; `status()` can never emit a stale number for
  any host, present or future.
- Cons: a `OnceLock<Mutex<Option<Arc<dyn Fn>>>>` holding an `Arc` to the queue
  outlives host shutdown, and it introduces a lock-order rule (observability
  mutex vs queue mutex) that every future caller must respect — precisely the
  kind of invariant that decays into a deadlock. It buys nothing O1 does not
  when the only publisher is one handler in the crate that owns the queue.
  Rejected; revisit only if a second crate publishes `ingest`.

### O5 — keep the sample, distrust it when old

Leave the snapshot in place and add `sampled_at_ms`; treat a sample older than
N intervals as unknown.

- Pros: smallest possible diff; degrades gracefully where no live read exists.
- Cons: it converts a wrong number into an *unknown* number, never into a right
  one — an enqueue that just happened is still invisible for up to N intervals,
  which is the whole window an agent reads the semaphore in. It also needs a
  notion of "expected interval" that the on-demand hosts do not have. Rejected.

## Decision outcome

**O1**, with D2 keeping the pass sample where its staleness is inherent, and D3
making "cannot tell" a state instead of a `false`.

### D1 — `queue` is live or `null`; it is never stale

`IngestStatus.queue` becomes `Option<QueueSnapshot>`. `Some` means *read at the
moment this answer was built, in this process*. `None` means this process cannot
read a queue and must not pretend otherwise. The nested shape of the object does
not change (`{depth, claimed}`), so a consumer that already guards
(`ingest.queue?.depth`) is unaffected; the OpenCode plugin is verified to guard
(`.opencode/plugins/vanta-memory-capture.ts:137,144`).

### D2 — the pass sample moves to `last_pass.queue`

`PassSnapshot` gains `queue: QueueSnapshot`. `note_pass` and `note_skipped_pass`
keep receiving the depth they already receive and store it there — where
`at_ms` on the same struct says exactly how old it is. The state-level `queue`
field is **deleted**, not deprecated: a number that means "as of some pass" must
not live under a name that reads as "now".

### D3 — health is four-state, and `unknown` is not `healthy`

`health: unknown | idle | draining | broken`, derived at publish time next to
`notice`:

```text
unknown   queue is None, OR this process has no evidence of hosting the
          pipeline (runner unknown, no pass, no loop) — e.g. a read-only
          writer, or a loop disabled before the first pass (see D8: a
          one-shot child that owns the writer role is *not* an example,
          because its own loop is up)
idle      depth == 0
broken    depth > 0 and drain_path == none
draining  depth > 0 and drain_path != none
```

Precedence is that order. `unknown` exists because a boolean cannot say "I do
not know", and "I do not know" reported as `false` is the defect this ADR
deletes. `degraded` keeps its own meaning ("extraction is impossible here") and
stays orthogonal: `degraded && depth == 0` is `idle` with a loud `notice`, not
`broken`.

### D4 — `drain_path`: who can drain in this process

`drain_path: loop | on_demand | none`, derived from state the module already has
plus one new bit, `scheduler_ever_started`:

```text
loop       scheduler_running
on_demand  not degraded, no loop ever hosted, but last_pass.is_some()
none       otherwise (degraded, or a loop that started and stopped, or no loop
           and no pass ever observed)
```

`on_demand` is what keeps the rule honest for embedders that call `run_pass`
from a request hook instead of hosting a loop
(`vanta-memory/src/services/conversation_hook.rs`): queued work between two
request-driven passes is *draining*, not broken. The `scheduler_ever_started` bit
is what separates "never hosted" from "hosted and died" — the `MEM-72` case
(`observability.rs:172-181`).

### D5 — `is_broken()` reads the live queue, never `last_pass.queue`

`is_broken() == (health == Broken)`. It is fed by the live `queue`, so it can no
longer be `false` because a photograph is old. It still cannot express
`unknown` — the documented rule is therefore: **`is_broken() == false` means "not
proven broken"; read `health` when the answer matters.**

### D6 — the seam is additive: `status_with_queue`

```rust
pub fn status() -> IngestStatus                                  // == status_with_queue(None)
pub fn status_with_queue(queue: Option<QueueSnapshot>) -> IngestStatus
```

`status()` keeps its signature and now always reports `queue: null` /
`health: unknown` — it has no queue, so it says so. The seven existing call sites
that read `runner`, `degraded` or `scheduler_running` keep working untouched; the
one caller that publishes the surface switches to `status_with_queue`.

### D7 — an enqueue stops impersonating a pass

`threads.rs:367` (`publish_backlog_if_degraded`) currently calls
`note_skipped_pass` on the enqueue path, which writes `last_pass.skipped: true`
for a pass that never ran. That is why `docs/api/MCP.md:652-657` has to explain
"`skipped: true` can mean a turn was enqueued". The warn latch is extracted into
`note_backlog(queue)` (warn once per episode, no `last_pass` write) and the
enqueue path calls that. `last_pass: null` then means exactly what the docs say.

### D8 — the multi-process ceiling is unchanged, and it stays declared

The queue is in-process state ([ADR-0054](./ADR-0054-scheduler-host-vantadb-server.md):51-52,
"restart = cola vacía"), so `ingest` describes the process that answered. D8
declares that ceiling; it does **not** convert the one-shot path into `unknown`
(the amendment below records what it used to claim). What the wire actually
reports:

| Case | Who answers | `ingest` in the answer |
|---|---|---|
| `--mcp` writer | the writer | its own queue: `idle` + `drain_path: loop` on an empty queue with the loop up (a *degraded* runner is still `idle` at depth 0, but `drain_path: none` and `notice` is loud) |
| `--mcp` proxy | the writer — the child forwards `tools/call` whole (`docs/api/MCP.md:510-526`) | the **writer's** queue and health; the child's own state never reaches the client |
| one-shot `mcp-call`, child wins the writer role | the child | its **own** (empty) queue: `idle` + `drain_path: loop`. `scheduler_running: true` defeats the "no evidence of hosting" guard, even though `runner` is still `unknown` until the first tick |
| one-shot `mcp-call`, child ends in proxy (another writer holds the DB) | the writer | the writer's queue and health |
| read-only writer, or a process whose loop is disabled (`VANTADB_SCHEDULER_INTERVAL_MS=0`) before the first pass/enqueue | that process | `unknown`: no runner observed, no pass, no loop — it holds its own `{0,0}` but nothing evidences hosting |

Two corollaries the contract depends on:

- **`health: unknown` means "no evidence of hosting here", never "someone else's
  backlog"** — and the honest producers are the read-only writer and the
  loop-disabled process, not the one-shot child.
- **`queue: null` never appears on the `capabilities` wire.** The handler always
  reads its own queue (`vantadb-mcp/src/handlers/tools.rs:3199-3201` passes
  `Some(queue_snapshot())`), so the `null` shape exists only on the library
  surface (`status()`).

What this ADR does **not** do: give a proxy — or a one-shot child that is not the
writer — a view of the writer's queue. That would need a cross-process surface
and is still `MCP-43`-adjacent debt.

**Enmienda (2026-11-26) — what D8 said, what was measured, why it changed.**
As accepted, D8 claimed: *"in one-shot `mcp-call` every invocation is a fresh
process whose empty queue is not the writer's, which is why that case now reads
`unknown` instead of `idle`."* That sentence was wrong when written: it described
what D3's guard would do if nothing else were true in the child, not what the
child does. Measured during the implementation (`dfb51d807`) by reading the
launch path, and re-verified for this amendment on 2026-11-26:

- `vanta-cli mcp-call` spawns `<server> --mcp`
  (`src/cli_handlers/mcp_call.rs:260-266`).
- That child opens the storage as a writer and, unless `read_only`, calls
  `memory_scheduler::start()` **before** the first request
  (`vantadb-mcp/src/server.rs:1057-1065`).
- `start()` calls `note_scheduler_started()` before returning
  (`vantadb-mcp/src/memory_scheduler.rs:134`), so `scheduler_running == true`
  when the first `capabilities` is answered.
- D3's guard is `runner == unknown && !scheduler_running && last_pass.is_none()`
  (`vanta-memory/src/ingest/observability.rs:366`), so it does not fire; the
  child reports `idle` with `drain_path: loop`. (`runner` stays `unknown` until
  the first tick builds one — that field does not decide `health`.)
- `queue: null` was likewise unreachable on the wire: the handler always passes
  `Some(crate::threads::queue_snapshot())` (`handlers/tools.rs:3199-3201`).
- When another writer holds the database the child never answers: it forwards
  the call, so the client reads the **writer's** health — a case D8 did not
  mention at all.

Why the decision changes rather than the code: passing `None` whenever hosting is
not evidenced would make the child hide a queue **it is actually holding** — an
empty queue its own loop can drain — and would put `queue: null` on the wire.
That is less information, not more; the operator loses the ability to tell "this
child answers about itself and its loop is up" from "this process is not a
host at all". D3's guard stays as the definition of `unknown`; the *example* was
what was wrong, and it is now the read-only writer or the loop-disabled process.
The published docs already say this, because the implementer documented the code
instead of the paragraph: `docs/api/MCP.md:528-536` and the one-shot caveat at
`:727-734`, `docs/api/VANTA_MEMORY.md:402-412`. This amendment makes the ADR
agree with them.

## Positive consequences

- The three `false`-as-"fine" cases in the problem statement are gone: a
  just-enqueued turn is visible immediately; a dead or never-started driver with
  work queued is `broken`; a process with no queue to read, or no evidence of
  hosting one, says `unknown`.
- `last_pass.skipped == true` means a skipped pass again (D7), so
  `docs/api/MCP.md:652-657` loses a paragraph that exists only to explain a wart.
- The surface needs no registry, no lock-order rule and no second copy of the
  queue's length.

## Negative consequences

- **`queue` can be `null`.** A consumer doing `ingest.queue.depth` without a
  guard breaks. The only in-repo consumer (the OpenCode plugin) already guards;
  any third-party parser documented against `docs/api/MCP.md` must adopt the
  guard. It is a form change in one state, which is why this ADR exists. Note
  (D8): on today's `capabilities` wire the field is always an object, because the
  handler always reads its own queue — so the `null` is a library-surface shape
  and a forward hazard, not a break in the published wire today.
- **`depth` gains one meaning and loses another.** It is now "queued right now"
  instead of "queued at the last pass"; anyone who wanted the historical value
  moves to `last_pass.queue`.
- **`health` and `drain_path` are derived fields.** They are computed when the
  status is published, so an `IngestStatus` assembled by hand in a test can hold
  an inconsistent triple. Tests must go through `status_with_queue`.
- **A transient false `broken` is possible** during graceful shutdown: the loop
  clears `scheduler_running` before the process exits, so a queued task can be
  reported `broken` in the last moments. The queue dies with the process anyway;
  we accept the alarm over the silence.
- **`unknown` is easy to ignore.** It is the third state an operator has to
  learn. The mitigation is that `notice` names it in words ("cannot see a queue
  here"), which is what `MEM-72` established for `runner: none`.

## Measurement and its limits

Measured by reading the tree at `HEAD` (2026-11-25; the D8 rows re-measured
2026-11-26), not by running:

| Claim | Source |
|---|---|
| The queue block is written only by pass results | `note_pass` `observability.rs:184`; `note_skipped_pass` `:202`; no other writer of `status.queue` |
| Degraded path probes live | `scheduler.rs:363`, `threads.rs:373` |
| The queue is one `Vec` + one `HashMap` behind one mutex | `local_backend.rs:21-34` |
| Live occupancy can be O(1) | `queue.len()` + `pending.len()` (`:27,:30`); `pending_count` is already O(1) at `:347` |
| The existing sampling is O(n) and already paid twice per pass | `queue_depth` splits by priority by scanning the vec (`:217-221`); called at `scheduler.rs:138,363` |
| The plugin survives `null` | `vanta-memory-capture.ts:137,144` use `ingest.queue?.depth ?? 0` |
| The one-shot child starts its loop before the first request | `mcp_call.rs:260-266` → `server.rs:1057-1065` → `memory_scheduler.rs:134` |
| `scheduler_running` alone defeats the "no evidence of hosting" guard | `observability.rs:366` (`!s.scheduler_running`) |
| `capabilities` always publishes a queue object | `handlers/tools.rs:3199-3201` (`Some(queue_snapshot())`) |
| `unknown` on the wire needs a process that starts no loop | `server.rs:1056,1063` (read-only); `memory_scheduler.rs:111-116` (interval `0`) |
| `gen-index` is already stale for an unrelated lane | `node scripts/docs/gen-index.mjs --check` reports `docs/index.md` diverging on `MEMG-27`/`MEMG-28` |

**Not measured: the latency of the live read.** No benchmark was run. What is
claimed is structural, from the source above: one uncontended mutex acquisition
plus two `len()` calls, i.e. strictly less work than the per-pass
`queue_depth()` the scheduler already performs twice, and taken only when
`capabilities` is called (an operator/agent tool, not a hot path).

## Verification

The mechanical contract lives in [`FIND-748`](../../tasks/FIND-748.md). At the
ADR level:

- `queue: null` appears in the JSON of `status()` — the library surface with no
  live read — with `health: unknown` in the same answer. On the `capabilities`
  wire the field is always an object (D8), so the null shape is verified at the
  library surface, not through the tool.
- A one-shot `mcp-call` child that wins the writer role answers `idle` /
  `drain_path: loop` about its own queue; a child that ends in proxy answers with
  the writer's `ingest` (D8 amendment, 2026-11-26).
- A queued task with a healthy runner and a never-started scheduler reads
  `depth > 0`, `drain_path: none`, `health: broken` — the assertion that failed
  against the pre-implementation code.
- `is_broken()` and `last_pass.queue` are shown to be independent, in both
  directions.

## References

- [FIND-748](../../tasks/FIND-748.md) — the implementation contract.
- `FIND-748` backlog row, `docs/dev/Backlog.md:836` — the measured finding.
- [vanta-memory.md § MEM-73](../../avance/activo/vanta-memory.md) — the composition
  that produced findings 3 and 4.
- `docs/api/MCP.md:452,462-536,727-734` — the published `capabilities`
  surface this ADR corrects, and the one-shot caveat the D8 amendment aligns
  with.
- `docs/api/VANTA_MEMORY.md:286,380-419` — the module contract and the
  process-local ceiling.
- [ADR-0054](./ADR-0054-scheduler-host-vantadb-server.md) — the queue's home and
  its in-memory semantics.
- `.opencode/rules/api-contract.md` (R-8: one owner per fact) — why the queue's
  length is not copied into a counter.
