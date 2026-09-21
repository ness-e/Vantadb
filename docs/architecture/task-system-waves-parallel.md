# Task-System: Parallel Waves + Lead Merge (TSYS-12)

> **Status:** Proposed — **not implemented**.
> **Date:** 2026-08-11
> **Scope:** Design/documentation only. No harness code changes.
> **Source:** `docs/research/2026-08-10-agent-engineering/agent-03-orchestration.md` §7.2-7.4 · REPORTE-FINAL §3.4-4 (L375) · plan `docs/plans/archive/2026-08-11-residuo-consolidado.md` Task 24.
> **Backlog:** `docs/Backlog.md` §P17 → `TSYS-12`.

## 1. Context

The task harness currently runs plans as a **single loop**: one task at a time,
sequentially, even when tasks are independent. `FAIL_MODE=parallel` exists
(`.opencode/task-system/prompts/pipeline-run.md` step 7) and already groups
tasks into waves by a dependency DAG, but it is a **synchronous fork/join**:
the lead spawns up to `MAX_CONCURRENT = min(3, tasks_in_wave)` sub-agents,
**waits for all of them to finish** before starting the next wave, and has **no
structural merge step** in the prompts — only a weak closing check
("verify there are no conflicts between parallel branches" via `git log`).

The research report flags this exact gap:

> **Parallel fan-out** — `FAIL_MODE parallel` exists and the only endorsed
> pattern is fan-out with merge step (`AGENTS.md`), but there is no structural
> merge step in the prompts nor DAG/critical-path/waves modeling.
> — REPORTE-FINAL §3.4-4

Anthropic's two-level parallelism shows the upside: a lead spawning **3-5
sub-agents in parallel** (not serial), each doing **3+ tool calls in parallel**,
yielded ~90% research-time reduction for complex queries (agent-03 §7.4).

## 2. Problem

1. **Synchronous waves create a bottleneck.** The lead waits for the slowest
   agent in a wave before continuing. It cannot direct in-flight sub-agents and
   is blocked by the straggler (agent-03 §7.2: "Crea cuellos de botella").
2. **No structural merge contract.** Wave outputs of different quality and
   granularity are consolidated ad hoc. There is no defined way to cross
   overlaps (duplicates), fill gaps, or resolve conflicts between branches
   (agent-03 §7.3: "la síntesis es trabajo del lead, no del user").
3. **No critical-path awareness.** The DAG exists, but nothing identifies
   which chain of waves dominates total latency, so scheduling cannot
   prioritize it.
4. **Merge responsibility is undefined in prompts.** The `pipeline-run.md`
   closing step only runs `git log --oneline` after the last sequential commit;
   it does not dedupe artifacts, check plan coverage, or resolve conflicts
   between sub-agents that touched the same files.

## 3. Proposed Design

### 3.1 Wave model

- **Wave** = set of sub-agents with **no dependencies between them**, run in
  parallel.
- **Width:** 3-5 sub-agents per wave (Anthropic §7.4). Keep the current
  `MAX_CONCURRENT = min(3, tasks_in_wave)` as the floor for Windows/RAM; allow
  an optional `WAVE_WIDTH` override up to 5 when the user confirms budget.
- **Ordering:** built from the existing dependency DAG (pipeline-run.md 7.a-7.b).
  A task with a dependency is placed in the first wave **after** its
  dependency completes. **Workers with dependencies must NOT share a wave**
  (agent-03 §7.2).
- **Critical path:** the longest wave chain (sum of expected latencies from
  wave 0 to the final join) is the lower bound of plan duration. Wave
  construction should schedule critical-path tasks in earlier slots and treat
  off-path waves as best-effort.

```
                    ┌──────────────────────────────────────────┐
                    │                 LEAD                      │
                    │  plan → DAG → waves → merge → synthesize  │
                    └──────────────────────────────────────────┘
                       fork (3-5 sub-agents, no deps between)
         ┌───────────────────┼───────────────────┐
         ▼                   ▼                   ▼
   sub-A (wave 0)     sub-B (wave 0)      sub-C (wave 0)
         │                   │                   │
         └───────────────────┼───────────────────┘
                             ▼
                   MERGE step (wave 0):
                   dedupe artifacts / fill gaps / resolve conflicts
                             │
                             ▼
         ┌───────────────────┼───────────────────┐
         ▼                   ▼                   ▼
   sub-D (wave 1, dep A)  sub-E (wave 1, dep B)      ← depends on wave 0
         │                   │
         └───────────────────┘
                             ▼
                   MERGE step final → lead synthesizes
                   (join decides: new wave or exit loop)
```

### 3.2 Asynchrony (optional phase 2)

Phase 1 keeps the synchronous join per wave (safe default, matches current
harness). Phase 2 (proposed, **not required**) allows the lead to launch the
next wave when the **partial results it depends on** are in, instead of
waiting for the whole wave — accepting the coordination costs documented in
agent-03 §7.2 (result consistency, error propagation). This is opt-in per plan.

## 4. Merge Contract (the lead's job)

After **every wave** (and once at the end), the lead runs a structural merge
before starting the next wave or synthesizing:

| Check | Detection | Resolution |
|---|---|---|
| **Duplicates** | Same artifact (file path, task ID, decision) produced by 2+ sub-agents in the wave. Cross-reference produced files vs plan's `Archivos clave`. | Keep one canonical version; record the drop in the plan/task state. |
| **Gaps** | Plan items (tasks, files) with **no** sub-agent assigned or no output produced by the wave. Diff wave outputs vs remaining `⬜ PENDING` tasks. | Assign to the next wave or the lead; never silently drop. |
| **Conflicts** | Two parallel branches touched the **same file** (should not happen if the DAG was respected; catch via `git status`/`git log` at wave boundary). | Stop, re-run serially, or apply lead decision. Record in decision memory. |
| **Quality variance** | Outputs of different quality/granularity (e.g., one deep report vs one stub). | Lead normalizes to the plan contract before synthesis. |

Rules:
- **The synthesis is the lead's job, not the user's** (agent-03 §7.3).
- Merge runs per-wave, not only at the end — catching conflicts early prevents
  a corrupted final join.
- Wave failure semantics stay as today: a failed task in a wave does not stop
  sibling completion, but **following waves do not start** (pipeline-run.md 7.e).

## 5. Boundaries (non-goals)

- **NOT a CI gate.** This design does not add gates to merge-to-main, CI, or
  pre-push. It is a harness execution optimization, optional per plan.
- **Optional.** Only applies to plans with ≥2 independent tasks
  (`FAIL_MODE=parallel`). Sequential mode is unchanged.
- **No new enforcement states.** The C0 state machine, tool allow/deny lists,
  and SARL escalation ladder are untouched.
- **No changes to sub-agent prompts** (`pipeline-full.md`) in this design;
  waves reuse the existing full-depth prompt with per-role routing.
- **Concurrency limits remain** (rate limits, token budget, `max_rpm`);
  `MAX_CONCURRENT` is capped by RAM (agent-03 §7.3).

## 6. Risks

| Risk | Mitigation |
|---|---|
| Async waves degrade result consistency / state coherence (agent-03 §7.2) | Keep synchronous join as default; async is opt-in phase 2 only. |
| Error propagation across waves | Per-wave merge + existing rule: failed task → siblings finish, following waves don't start (pipeline-run.md 7.e). |
| Merge step becomes the lead's new bottleneck | Merge is per-wave, small N (≤5 outputs); full synthesis only at final join. |
| Duplicate/conflicting file writes between branches | DAG must forbid same-file tasks in one wave; merge contract detects violations at wave boundary. |
| Budget blowup (sub-agents, tool calls, time) | Keep budget caps: 20 sub-agents total, 8 tool calls per agent, ~2 min timeout (pipeline-run.md REGLAS). |

## 7. Open Questions

1. Should `WAVE_WIDTH` be configurable per plan, or fixed at 3 until Windows RAM allows 5?
2. Does phase 2 (async wave start) require a `partial_result` artifact convention (filesystem paths, like agent-03 §8 "artefactos en filesystem")?
3. Where should the merge contract live once implemented — new step in `pipeline-run.md` or a separate `prompts/merge.md` referenced from it?

## 8. Implementation sketch (when approved)

1. Add per-wave merge step to `pipeline-run.md` (step 7.x) applying §4 checks.
2. Optional: `WAVE_WIDTH` override; critical-path identification from the DAG.
3. Optional phase 2: async wave start with partial-result handoff.
4. Update `campaign-executor/SKILL.md` wave table + `REPORTE-FINAL` §3.4-4 closed.
