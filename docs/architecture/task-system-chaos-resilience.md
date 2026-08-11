# Task-System: Chaos & Resilience Suite (TSYS-06)

> **Status:** Proposed — **not implemented**.
> **Date:** 2026-08-11
> **Scope:** Design/documentation only. No chaos runner code is implemented by
> this document; it defines the scenarios, expectations, and execution strategy
> a future runner must implement.
> **Source:** `docs/Investigaciones/2026-08-10-agent-engineering/REPORTE-FINAL.md` §3.3-24 (FALTA #24) · plan `docs/plans/archive/2026-08-11-residuo-consolidado.md` Task 19.
> **Backlog:** `docs/Backlog.md` §P17 → `TSYS-06`.

## 1. Context

`vanta-chaos` fuzzes the Rust source code (`fuzz/` targets, `chaos_integrity`
with failpoints) but **never exercises the task-system itself**:
`campaign-server.mjs`, `state-tools.mjs`, and the persisted state they
read/write (plan files, budget JSON, verify-log, traces). The pipeline is the
tool that executes every other task — if its state gets corrupted, a stale
`in-progress` blocks all future work (one-task-at-a-time WIP hard-limit), a
corrupt budget silently resets counters, and a torn plan-file write can drop a
task from the DAG. REPORTE-FINAL §3.3-24 flags exactly this blind spot:
chaos engineering applies to the harness, not just the product.

This document is the **design of a chaos/resilience suite** for the task-system.
It is intentionally a document: the runner is out of scope (separate
implementation task), but the scenarios, triggers, and expected behavior below
are the contract the runner must assert.

## 2. State Machine & Persistence Surface

### 2.1 State machines

Two machines coexist:

1. **C0 phase machine** (`state-tools.mjs` `STATE_TOOLS`, canonical):
   `PLAN → ACT → VERIFY → COLLATERAL → RESEARCH → EVALUATE → REVIEW → ACCEPT →
   CLOSE → STALL`. Enforced purely by `validateAction(state, toolName)` /
   `getAllowedTools(state)` — in-memory only, no persistence. Risks are
   functional, not corruptive: unknown state names deny everything; wildcard
   patterns (`campaign_*`) match by prefix.
2. **Task state machine** (persisted in the plan file as `- **Estado:**`):
   `⬜ PENDING → ⏳ EN PROGRESO → ✅ COMPLETED | ❌ FAILED` (and back to
   PENDING on reopen). This is the corruption-sensitive machine: it lives in a
   hand-edited Markdown file parsed by regex (`parseTasks`, `extractState`,
   `findTaskById`, `updateState`).

### 2.2 Persisted state (files)

| File | Written by | Read by | Failure mode if corrupted |
|------|-----------|---------|---------------------------|
| `docs/plans/<plan>.md` | `campaign_update_task_state`, `getOrCreateCampaignId` | every read tool (`parseTasks`) | tasks silently dropped, states misdetected, wrong task updated |
| `docs/plans/<plan>.budget.json` | `writeBudget` (init/consume/reset, `getOrCreateTraceId`) | `readBudget`, `budgetStatus`, `campaign_stalled_tasks` | `readBudget` catch → `{ tasks: {} }` → counters/traceId reset mid-run |
| `.opencode/task-system/enforcement/verify-log.jsonl` | `campaign_verify_cmd` | `campaign_diagnose_pipeline`, TSYS-05 SLA | append-only; a torn line breaks JSONL parsers |
| `.opencode/task-system/memory/{lessons,decisions}.md` | `campaign_memory_write`, update on complete/failed | `campaign_memory_read` | append-only; lost tail on crash |
| `traces/*` | `tracer.mjs` (`traceEmit`) | `campaign_health_status` | lost events, no state impact |
| `traces/active-model.json` | `campaign_set_model` | `campaign_get_active_model` | `readActiveModel` catch → `{ model: "default" }` |

## 3. Risk Surface (with routes)

### 3.1 Non-atomic read-modify-write on the plan file

`campaign_update_task_state` (campaign-server.mjs) does
`readFileSync → updateState (regex replace) → writeFileSync` with **no lock
and no temp-file swap**. A crash between read and write, or two concurrent
updates, loses an update (last-writer-wins). `findTaskById` locates a block
with a regex bounded by `### Task ` / `## ` / `===` — a malformed header or an
injected `- **Estado:**` line inside the block updates the wrong field.
`getOrCreateCampaignId` writes a Campaign ID into the plan file during
`campaign_get_next_task`, racing any concurrent state update on the same file.

### 3.2 Budget JSON corruption is swallowed

`readBudget` catches every parse error and returns `{ tasks: {} }`. A corrupt
`<plan>.budget.json` therefore **silently resets** budget counters, `traceId`
per task, and `lastActivity` — the stalled-task detector then treats an active
task as stalled, and budget limits restart from zero mid-run. No error is
surfaced anywhere.

### 3.3 Task state is regex-derived from hand-edited Markdown

`extractState` inspects only emoji presence (`✅`/`❌`/`⏳`) in the
`- **Estado:**` line; `parseTasks` splits on `### Task \d+` and **skips any
block whose header no longer matches** (e.g. a truncated title). A torn plan
file can silently reduce the task count without any tool reporting a warning.

### 3.4 Concurrency: double update / double get_next_task

The MCP server is single-process, but two sessions can call
`campaign_update_task_state` (or `get_next_task` + `update_task_state`)
concurrently — both read the same original content, both write; the last
writer wins and the first update is silently dropped. The WIP hard-limit scan
(`findInProgressTasks`) also reads plan + task files with no synchronization,
so two `in-progress` claims can both pass the check before either writes.

### 3.5 Crash mid-operation (kill between steps)

`campaign_verify_cmd` runs `execSync` with a timeout — a hard kill leaves no
verify-log entry (acceptable, eval is best-effort). `writeBudget` is a direct
`writeFileSync` of the whole JSON — a kill mid-write leaves a truncated file
that `readBudget` misreads as reset. `campaign_update_task_state` writes the
plan file and *then* emits trace events + appends lessons; a kill between the
write and the append loses the trace/lesson but keeps the state change.

### 3.6 Retries are re-entrant

`campaign_get_next_task` re-initializes the budget for the returned task on
every call (`initTaskBudget` resets `lastActivity` but keeps counters — except
after a corrupt read, where `state.tasks[taskId]` is re-created from scratch).
A retried `verify` after a kill re-runs the command and appends a **second**
verify-log entry for the same task (log grows, no dedupe by traceId).

## 4. Chaos Scenarios (trigger → expected behavior)

The table is the assertion contract for the future runner. Each scenario
describes the injected fault, the expected observable behavior, and which
invariant it protects.

| # | Scenario (trigger) | Expected behavior | Invariant |
|---|--------------------|-------------------|-----------|
| C1 | **Plan file corrupted mid-run** — inject a truncated task header / broken `- **Estado:**` line / stray `### Task` in a note | All read tools still return; parse degrades gracefully: block that cannot be parsed is reported as `parseError` (at minimum: state count changes are visible), no crash, no write to the corrupt file from read-only tools | Reads never panic on malformed input |
| C2 | **Task block removed from plan while task is `in-progress`** | `campaign_update_task_state` returns `updated:false` + warning (already handled); WIP scan no longer counts the removed task; no phantom block | State machine never invents state |
| C3 | **Budget JSON truncated / invalid JSON** | `readBudget` returns `{ tasks: {} }` **and** the tool surfaces a `budgetCorrupted: true` warning instead of silently resetting (this is a required behavior change, not current behavior) | Corruption is visible, never silent |
| C4 | **Double concurrent `campaign_update_task_state` (A→completed, B→in-progress) on same task** | Exactly one final state; the losing update returns `updated:false`/`conflict:true` with the winning state in the payload (required change: version/checksum check before write) | No lost updates |
| C5 | **Double concurrent `campaign_get_next_task`** | One campaign ID wins; both calls return the same `campaignId`; no duplicated ID lines in the plan file | Campaign ID is idempotent |
| C6 | **Kill between read and write of `update_task_state`** | Plan file unchanged (either the whole update lands or none); next read reflects pre-kill state | Atomicity of plan writes |
| C7 | **Kill mid `writeBudget` (truncated JSON)** | Next budget read recovers: counters preserved if possible, else a visible `budgetCorrupted` warning; stalled-detection does not mislabel the task as stalled from a reset `lastActivity` | Budget survives partial writes |
| C8 | **`campaign_verify_cmd` killed mid-run** | Command is a no-op for state (no plan/budget mutation); verify-log may miss the entry but subsequent verifies still append valid JSONL lines | Verify is side-effect-free on state |
| C9 | **Retry of a failed verify (re-entrancy)** | Second verify appends a new log line with the same `traceId`; no counters double-spent beyond budget; budget `withinBudget` still enforced across retries | Retries are bounded and traceable |
| C10 | **Unknown/typo state passed to `validateAction` / `enforce_state`** | `allowed:false` with a descriptive reason (already current); no fallback to allow | Deny-by-default on unknown states |
| C11 | **Plan file missing from `docs/plans/` while running** | `findPlanFile` returns null → tools return `{ error: "No plan file found" }` instead of crashing (current behavior); campaign cannot silently claim a task | Fail loudly, not silently |
| C12 | **WIP hard-limit race: two tasks claimed `in-progress` concurrently** | Only one claim wins; the loser gets `wipBlocked:true` even under the race (required change: atomic claim check-and-set, e.g. lock file or single write) | One-task-at-a-time is never violated |

## 5. Execution Strategy

| Aspect | Decision |
|--------|----------|
| **Who runs it** | `vanta-chaos` (leaf specialist, owns fuzz/stress/chaos scripts) — runs scenarios by injecting faults into a **throwaway copy** of the task-system state (a scratch `docs/plans/` + `.opencode/task-system/enforcement/verify-log.jsonl` + `budget.json`), never the live repo |
| **When** | Manual / on-demand (invoked via `/build prove` or a dedicated chaos command) and pre-release, **not** on every push |
| **Sandbox** | `campaign_run_sandboxed` (already provided by the server) with `blockNetwork:true`; runner stages a minimal plan file + budget fixture, executes the scenario against the real server code in the sandbox workdir, asserts the expected behavior, tears down |
| **How to inject** | Fault injection points: file mutation (truncate/replace/delete) on staged fixtures; process kill (SIGKILL / `Stop-Process` on the spawned server) at scripted points; concurrency via two parallel client invocations of the same tool |
| **Assertions** | Per scenario, a pass/fail check against the "Expected behavior" column; a scenario that fails produces a report entry (which invariant broke) |

## 6. Required Behavior Changes (pre-conditions for the suite)

Current code already satisfies several scenarios (C1, C2, C10, C11) without
changes. The suite **requires** three behavior changes that are **not yet
implemented** (they are the implementation follow-up this design unlocks):

1. **Visible budget corruption** — `readBudget` must not silently swallow parse
   errors; surface `budgetCorrupted` in tool responses (C3, C7).
2. **Concurrency-safe plan writes** — checksum/version check before write in
   `campaign_update_task_state` so concurrent updates return `conflict` instead
   of silently losing (C4); atomic campaign-ID creation (C5).
3. **Atomic WIP claim** — check-and-set for the one-task-at-a-time claim so the
   race in C12 cannot produce two concurrent `in-progress` tasks.

These are implementation tasks, deliberately out of scope for this design doc.

## 7. Limits

- **Not a CI gate.** The suite is manual/on-demand and pre-release only; it
  must never block the Fast Gate or normal pushes (mirrors the existing
  heavy-certification policy).
- **Does not fuzz the LLM/agent layer** — only the server, the state machine,
  and the persisted files. Prompt-level failure modes are covered by other
  TSYS items.
- **No production mutation.** Every scenario runs on staged fixtures inside the
  sandbox; the live repo's plan files, budget, and verify-log are never touched.
- **Does not test git-level resilience** (torn commits, hook failures) — that
  is the pre-push barrier's domain, not the task-system's.

## 8. Out of Scope (deferred)

- The chaos runner implementation (scripts, harness, scenario fixtures).
- The three behavior changes listed in §6 (separate implementation tasks that
  this design gates).
- Metrics/telemetry from chaos runs (would feed TSYS-05 once verify-log is
  populated).
