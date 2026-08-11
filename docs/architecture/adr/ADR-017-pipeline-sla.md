---
title: "ADR-017: Pipeline SLA — SLI/SLO/error budget for the task-system campaign executor"
type: adr
status: accepted
tags: [vantadb, architecture, adr, task-system, sla, observability, quality-gates]
created: 2026-08-11
last_reviewed: 2026-08-11
owner: vanta-lead
---

# ADR-017: Pipeline SLA — SLI/SLO/error budget for the task-system

## Context

`docs/Investigaciones/2026-08-10-agent-engineering/REPORTE-FINAL.md` gap-01 §3.3-23
flagged that the pipeline had no SLA: "hoy no se sabe si el pipeline falla mucho".
RULES.md (`campaign-executor`) states aspirational North Star criteria — >90%
first-try completion, 0 false positives, 0 silent regression — but they are
targets, not an SLA with defined error budgets, measurement windows, or a
recalibration rule.

The metrics the SLA needs are already computable by the existing evaluators
(no new code required):

- `evals/northstar.mjs` (P1-06) → `docs/reports/northstar.md` — first-try
  completion rate, false positives (union of COMPLETED-with-failed-verify,
  verified-then-rerun, budget fails), silent regression (passed→failed).
- `evals/eval-metrics.mjs` (EVAL-01) → `docs/reports/pipeline-evals.md` —
  same North Star set, plus per-type and skill→first-try correlation.
- `evals/dora.mjs` (P3-07) → `docs/reports/dora.md` — CFR (failed verify
  invocations / total invocations), lead/cycle time, throughput.

Real telemetry as of 2026-08-11
(`.opencode/task-system/enforcement/verify-log.jsonl`, 3 invocations, all for
task `T1-residuo-consolidado`):

| # | command | result | elapsed |
|---|---|---|---|
| 1 | `cargo test -p vantadb` | fail, exit 101 (13 failed / 1735 passed) | 26.6s |
| 2 | node JSON-parse validation | pass, exit 0 | 0.1s |
| 3 | `cargo test -p vantadb-server --test server` | pass, exit 0 (19/19) | 3.1s |

Aggregated: 1 task with verify data, first attempt **failed**, 1/3 invocations
failed (CFR 33.3%), 0 false positives, 0 silent regressions. **Sample size is
N=1 task — not statistically meaningful.** The thresholds below are therefore
declared as targets (aligned with the RULES.md North Star) and will be
calibrated against the accumulated log once N≥20 tasks have verify data
(rule P3-2-style, "baseline pendiente").

Note: `eval-metrics.mjs` and `northstar.mjs` disagree on first-try rate for
telemetry-bearing tasks (0.0% vs best-effort 100%) because they use different
denominators: tasks with verify data only (eval-metrics) vs all COMPLETED tasks
best-effort (northstar). This ADR fixes the SLA definition on the eval-metrics
denominator (telemetry-bearing tasks only) and flags the northstar discrepancy
in Consequences.

## Decision

1. **SLI-1 — First-try success rate** (tasa de verify exitoso primer intento):
   tasks whose first verify invocation passed / tasks with ≥1 verify invocation.
   - **SLO:** ≥ 90% per measurement window (matches RULES.md North Star ">90%").
   - **Error budget:** 10% of telemetry-bearing tasks per window may fail their
     first verify.
2. **SLI-2 — CFR, Change Failure Rate** (from `evals/dora.mjs`):
   failed verify invocations / total verify invocations.
   - **SLO:** ≤ 30% per measurement window.
   - **Error budget:** 30% of verify invocations per window may fail.
   - Baseline rationale: current log shows 33.3% (1/3) over a single failing
     task — this is a red flag on the *first real data point*, not a calibrated
     number; the SLO is set as a target and re-evaluated at N≥20.
3. **SLI-3 — Verification integrity** (from `evals/northstar.mjs` /
   `eval-metrics.mjs`): false positives = 0 AND silent regressions = 0.
   - **SLO:** 0 (hard invariants; non-consumable).
   - **Error budget:** 0. A COMPLETED-with-failed-verify or a passed→failed
     pattern is an incident, not budget spend.

**Measurement window:** the accumulated `verify-log.jsonl` evaluated on each
run of the evaluators (the same window `northstar.mjs`, `eval-metrics.mjs` and
`dora.mjs` already use). Cycle time / lead time (DORA) is deliberately NOT an
SLI yet: task timestamps are not structurally normalized (dora.mjs degrades to
file mtime) — that is P2-05/traceId work. When structured timestamps land, add
SLI-4 (cycle time) with a calibrated SLO.

**Calibration rule:** thresholds above are locked to the RULES.md North Star
and may only be recalibrated when (a) N≥20 tasks have verify data AND (b) the
recalibration is recorded as an amendment to this ADR. Thresholds never lower
silently (ratchet, same spirit as DoD thresholds in RULES.md).

**Failure response (burned budget):** when SLI-3 hits 1, or SLI-1/SLI-2 exceed
their SLO for a full window, the pipeline stops shipping new verify-heavy
campaigns until the failure class is root-caused and fixed (Iron Law /
systematic-debugging first — matches RULES.md stagnation rules).

## Consequences

- Pros:
  - The "do we know if the pipeline fails a lot" question now has a
    measurable answer with a budget, not a vibe.
  - Zero new code: all three SLIs are already computed by the existing
    evaluators (`northstar.mjs`, `eval-metrics.mjs`, `dora.mjs`).
  - SLOs align with the existing RULES.md North Star — no contradictory
    thresholds added.
  - Baseline discipline: with N=1 task the numbers are declared targets with
    an explicit calibration path, not invented statistics.
- Cons:
  - **Denominator discrepancy:** `northstar.mjs` reports first-try on all
    COMPLETED tasks best-effort (100% today) while `eval-metrics.mjs` reports
    it on telemetry-bearing tasks only (0% today). This ADR fixes the SLA on
    the eval-metrics denominator; northstar.mjs should be aligned (or its
    definition documented) so the report matches the SLA — tracked as follow-up,
    no evaluator code was touched in this task.
  - SLO ≤30% CFR is a target, not yet evidenced — the single failing task in
    the log already exceeds it. This is intentional (baseline pending), not a
    policy of ignoring red flags.
  - No cycle-time SLI until timestamps are structural (P2-05).

## Related

- `evals/northstar.mjs`, `evals/eval-metrics.mjs`, `evals/dora.mjs`
- `docs/reports/northstar.md`, `docs/reports/pipeline-evals.md`, `docs/reports/dora.md`
- `.opencode/task-system/enforcement/verify-log.jsonl`
- `.opencode/skills/campaign-executor/RULES.md` (North Star: >90% / 0 / 0)
- `docs/Investigaciones/2026-08-10-agent-engineering/REPORTE-FINAL.md` gap-01 §3.3-23
- Backlog `TSYS-05` (P17)
- Precedent: `ADR-015-coverage-policy.md` (quality-gate policy as ADR)

## Future tracking

- Recalibrate SLI-1/SLI-2 once N≥20 tasks have verify data; record amendment.
- Add SLI-4 (cycle time) when P2-05/traceId lands structured timestamps.
- Align `northstar.mjs` denominator with the SLA definition (eval-metrics
  denominator) or document the difference in the report.
- Review date: next pipeline audit or when the verify log passes 50 lines.
