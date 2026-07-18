---
description: Run the pre-launch checklist via parallel fan-out to specialist personas, then synthesize a go/no-go decision with a rollback plan
---

Invoke the agent-skills:shipping-and-launch skill.

`/ship` is a **fan-out orchestrator**. It runs three specialist personas in parallel against the current change, then merges their reports into a single go/no-go decision with a rollback plan. The personas operate independently — no shared state, no ordering — which is what makes parallel execution safe and useful here.

> **Pre-flight:** Run `/audit quick` or `/audit certify` before `/ship`. If audit failed, `/ship` should default to NO-GO.
> **Output:** Always writes `docs/ship-reports/ship-<timestamp>.md` and `docs/last-ship-state.json`.

## Phase A — Parallel fan-out

Spawn three subagents concurrently using the Agent tool. **Issue all three Agent tool calls in a single assistant turn so they execute in parallel** — sequential calls defeat the purpose of this command.

In Claude Code, each call passes `subagent_type` matching the persona's `name` field:

1. **`vanta-audit`** — Run a security + memory-safety audit. Check `unsafe` blocks, FFI boundaries, `cargo audit`/`deny`, supply chain risk. Also performs five-axis code review for non-security aspects (correctness, readability, architecture, performance). Output the standard audit template.
2. **`vanta-chaos`** — Analyze test coverage and resilience for the change. Identify gaps in fuzzing, chaos tests, edge cases, race conditions, and concurrency scenarios. Output coverage analysis with recommended test additions.
3. **`vanta-tuner`** — Run a performance and observability audit. Check RED metrics on any new endpoints, benchmark regression for changed hot paths, binary size impact, and backpressure requirements. Output the optimization report template.

**If no Agent tool is available** (e.g., plain OpenCode): invoke each persona sequentially and treat their outputs as if returned in parallel — the merge phase still works.

## Phase B — Merge in main context

Once all three reports are back, the main agent (not a sub-persona) synthesizes them:

1. **Code Quality** — Aggregate Critical/Important findings from `vanta-audit` and any failing tests, lint, or build output. Resolve duplicates between reviewers.
2. **Security** — Promote any Critical/High `vanta-audit` findings to launch blockers. Cross-reference with its security axis.
3. **Performance** — Pull from `vanta-tuner`'s report; cross-check Core Web Vitals if applicable.
4. **Accessibility** — Verify keyboard nav, screen reader support, contrast (not covered by the three personas — handle directly here, or invoke the accessibility checklist).
5. **Infrastructure** — Env vars, migrations, monitoring, feature flags. Verify directly.
6. **Documentation** — README, ADRs, changelog. Verify directly.

## Phase C — Decision and rollback

Produce a single output:

```markdown
## Ship Decision: GO | NO-GO

### Blockers (must fix before ship)
- [Source persona: Critical finding + file:line]

### Recommended fixes (should fix before ship)
- [Source persona: Important finding + file:line]

### Acknowledged risks (shipping anyway)
- [Risk + mitigation]

### Rollback plan
- Trigger conditions: [what signals would prompt rollback]
- Rollback procedure: [exact steps]
- Recovery time objective: [target]
- Previous SHA: [commit to revert to]

### Specialist reports (full)
- [vanta-audit report]
- [vanta-chaos report]
- [vanta-tuner report]
```

**Then write to disk:**

1. `docs/ship-reports/ship-<timestamp>.md` — full report with all sections above
2. `docs/last-ship-state.json` — lightweight state for `/rollback` and `/status`:

```json
{
  "timestamp": "ISO8601",
  "decision": "GO",
  "sha_shipped": "abc1234",
  "previous_sha": "def5678",
  "rollback_plan": "exact bash/git commands",
  "trigger_conditions": "what to watch for"
}
```

## Rules

1. The three Phase A personas run in parallel — never sequentially.
2. Personas do not call each other. The main agent merges in Phase B.
3. The rollback plan is mandatory before any GO decision.
4. If any persona returns a Critical finding, the default verdict is NO-GO unless the user explicitly accepts the risk.
5. **Skip the fan-out only if all of the following are true:** the change touches 2 files or fewer, the diff is under 50 lines, and it does not touch auth, payments, data access, or config/env. Otherwise, default to fan-out. `/ship` is designed for production-bound changes — when the blast radius is non-trivial, run the parallel review even if the diff looks small.
6. Always write `docs/ship-reports/` and `docs/last-ship-state.json` — they are required by `/rollback` and `/status`.
