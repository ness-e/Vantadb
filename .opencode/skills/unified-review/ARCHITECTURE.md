# Architecture — Unified Review & Audit Skill

> Design rationale for the parallel sub-agent architecture. Read this if you
> want to understand **why** the skill is shaped the way it is, or if you
> want to extend it with new phases, profiles, or integrations.

---

## 1. Problem statement

The three legacy skills (`vantadb-full-review`, `vantadb-certify`,
`vantadb-audit`) all run **monolithically** — the primary agent executes
every check itself: `cargo check`, `cargo clippy`, `cargo nextest`,
`pytest`, `npm run build`, security review, performance review, code
review, and report generation. This works for small projects but fails
on VantaDB (~790 files, 17+ Rust crates, 3 SDK bindings, Next.js web):

- A single `cargo check --workspace` produces ~30 KB of output and takes
  2 minutes. Multiplied by every check, the agent's context window fills
  up before the report can be written.
- Skills loaded for review (`code-review-and-quality`,
  `doubt-driven-development`, `code-simplification`, `security-and-hardening`,
  `deprecation-and-migration`, `systematic-debugging`) each consume 5-15%
  of context just to load.
- The agent truncates output, drops findings, or times out before reaching
  the report phase.

## 2. Solution: fan-out / fan-in with sub-agents

The unified skill uses OpenCode's `task` tool to delegate each phase to a
**sub-agent**. Each sub-agent:

1. Receives a self-contained prompt with the `DetectionResult` JSON and
   the phase config.
2. Runs its commands (cargo, npm, pytest, etc.) in its own context window.
3. Loads any cross-cutting skills it needs (in its own context, not the
   orchestrator's).
4. Returns a structured JSON report — **only the JSON, no prose, no logs**.
5. Exits. Its context is freed.

The orchestrator's context stays under 10% because it only ever sees the
JSON reports (typically 1-5 KB each), never the raw command output.

### Visual flow

```
                ┌──────────────────────────────────────────┐
                │       ORCHESTRATOR (primary agent)        │
                │       Context budget: < 10%               │
                │                                           │
                │  Phase 0: detect (sync, ~5s)              │
                │     ↓                                     │
                │  Build DetectionResult JSON               │
                │     ↓                                     │
                │  Load profile (default + project)         │
                │     ↓                                     │
                │  Resolve phases for mode                  │
                └─────────────────────┬─────────────────────┘
                                      │
                                      ↓
                ┌──────────────────────────────────────────┐
                │  WAVE 1: L1 alone (critical gate)        │
                │                                           │
                │  task("L1-core", prompt, subagent_type)  │
                │     ↓                                     │
                │  Await result. If fail → abort.          │
                └─────────────────────┬─────────────────────┘
                                      │
                                      ↓
                ┌──────────────────────────────────────────┐
                │  WAVE 2: L2..L6 in parallel (max 4)      │
                │                                           │
                │  ├── task("L2-bindings",  ...)            │
                │  ├── task("L3-web",       ...)            │
                │  ├── task("L4-ci-cd",     ...)            │
                │  ├── task("L5-docs",      ...)            │
                │  └── task("L6-arch",      ...)            │
                │                                           │
                │  Await all (timeout = 5min each)         │
                └─────────────────────┬─────────────────────┘
                                      │
                                      ↓ (mode == full only)
                ┌──────────────────────────────────────────┐
                │  WAVE 3: L7, L8, L9 in parallel          │
                │                                           │
                │  ├── task("L7-security",  ...)            │
                │  ├── task("L8-perf",      ...)            │
                │  └── task("L9-review",    ...)            │
                │                                           │
                │  Await all                                │
                └─────────────────────┬─────────────────────┘
                                      │
                                      ↓
                ┌──────────────────────────────────────────┐
                │  Phase L10: consolidate (sync)           │
                │  Phase L11: report (sync)                │
                │                                           │
                │  Write docs/reviews/review-<mode>-<ts>.md│
                │  Optionally update Campaign task         │
                └──────────────────────────────────────────┘
```

## 3. Data contracts

### 3.1 DetectionResult (orchestrator → every sub-agent)

Built by Phase 0, passed to every sub-agent in its prompt. See
`SKILL.md → DetectionResult JSON contract` for the full schema.

### 3.2 PhaseReport (sub-agent → orchestrator)

Each sub-agent MUST return a single JSON object and nothing else.

```json
{
  "phase": "L1",
  "phase_name": "Core Language Check",
  "status": "pass|fail|warn|skip|timeout|error",
  "score": 8,                          // 0-score_max, null if phase has no score
  "score_max": 10,
  "duration_ms": 23456,
  "subagent_type_used": "vanta-worker",// actual type used (may differ if fallback)
  "commands_run": [
    {
      "cmd": "cargo check --workspace --tests -j 2",
      "exit": 0,
      "duration_ms": 12345,
      "summary": "ok. 17 crates checked."  // ≤ 200 chars
    },
    {
      "cmd": "cargo clippy --workspace --tests -j 2 -- -D warnings",
      "exit": 1,
      "duration_ms": 8901,
      "summary": "error: found 2 warnings, -D warnings turned them into errors"
    }
  ],
  "quality_gates": [
    {"condition": "All compile/lint commands exit 0", "passed": false},
    {"condition": "All unit tests pass", "passed": true}
  ],
  "findings": [
    {
      "id": "H01-CODE-001",
      "severity": "high",
      "category": "CODE",
      "subcategory": "Panic in library code",
      "file": "crates/vantadb-core/src/engine.rs",
      "line": 142,
      "description": "unwrap() on user-provided vector index",
      "recommendation": "Use ok_or_else(|| VantaError::IndexOutOfBounds { ... })",
      "skills_loaded": ["code-review-and-quality"],
      "cross_cutting": false
    }
  ],
  "vetoes": [                           // only L9 fills this; cognitive phase
    {
      "skill": "doubt-driven-development",
      "reason": "Insufficient test coverage on new public API",
      "severity": "high"
    }
  ],
  "scoring_breakdown": {                 // only if scoring.enabled
    "iso_25010": {"functional_suitability": 8, "reliability": 7, ...},
    "sonarqube": {"quality_gate": "FAIL", "conditions_passed": 7, "conditions_total": 9},
    "cii": {"level": "Passing", "gaps_for_silver": [...]},
    "owasp": {"level_reached": "L1", "failed_checks": [...]},
    "codeclimate": {"rating": "B", "issues_by_severity": {...}}
  },
  "raw_log_path": "docs/reviews/logs/L1-2026-07-26-1430.log"  // only if keep_raw_logs
}
```

### 3.3 ConsolidatedReport (orchestrator builds internally)

The orchestrator holds this in-memory and writes it to disk as the final
markdown report. Schema:

```json
{
  "profile": "vantadb",
  "mode": "certify",
  "started_at": "2026-07-26T14:30:00-04:00",
  "ended_at": "2026-07-26T14:38:42-04:00",
  "duration_ms": 522000,
  "phases": [/* PhaseReport objects */],
  "overall_score": 7.2,
  "overall_rating": "B",
  "quality_gate_passed": false,
  "findings_count": {"critical": 1, "high": 4, "medium": 10, "low": 3, "info": 0},
  "cross_cutting_findings": [...],
  "top_critical_findings": [...]
}
```

## 4. Sub-agent prompt template

The orchestrator builds each sub-agent's prompt by interpolating
DetectionResult, the phase config, and the diff context into this template:

```
You are running Phase {phase_id} ({phase_name}) of the unified-review skill.

## Context

Mode: {mode}
Profile: {profile_name}
Project: {project_type}
Languages: {languages}
Ponytail mode: {ponytail_mode}

## DetectionResult

{detection_result_json}

## Phase configuration

{phase_config_yaml}

## Diff context (changed files)

{changed_files_list}

## Your task

1. Run every command listed in `commands` for the detected language(s).
   For each, capture: command, exit code, duration, summary (≤ 200 chars).

2. Evaluate each `quality_gates` condition. Mark passed/failed.

3. Scan the changed files for findings. Each finding:
   - Gets a unique ID: H<phase_num><CATEGORY>-<NNN>
   - Has severity: critical|high|medium|low|info
   - Has category: one of LOGIC|PATTERN|ARCH|DIRECTION|CLARITY|CODE|DESIGN|ERROR|MISSING|FEATURE|ALGO|ANY
   - Has file + line + description + recommendation

4. {if scoring.enabled} Compute per-system scores per the schema above. {endif}

5. {if phase.cognitive} Load each skill in `skills_to_load` (one at a time,
   in order). Apply its framework. Record findings. If a skill issues a
   VETO, record it (do not abort). {endif}

6. {if ponytail_mode == "lite"} Skip findings where category=DIRECTION and
   subcategory=Over-engineering. {endif}
   {if ponytail_mode == "ultra"} Escalate DIRECTION/Over-engineering findings
   by one severity level (info→low→medium→high→critical, but don't escalate
   beyond critical). {endif}

7. Compute your phase score (0-score_max). Suggested formula:
   - Start at score_max
   - Subtract 2 per failed quality gate
   - Subtract 1 per critical finding, 0.5 per high, 0.2 per medium
   - Floor at 0

8. {if orchestration.keep_raw_logs} Write your full raw output (commands +
   findings + skill outputs) to {raw_log_path}. Do NOT include it in the
   JSON below. {endif}

## Return

Return ONLY a single JSON object matching the PhaseReport schema. No prose,
no markdown fences, no commentary. The orchestrator will parse it.

If you cannot produce JSON for any reason, return:
  {"phase": "{phase_id}", "status": "error", "error": "<one-line reason>"}
```

## 5. Failure handling

### 5.1 Critical phase failure (L1)

L1 is the only critical phase. If L1 fails:

1. Orchestrator stops launching further waves.
2. Already-running wave-1 sub-agents (none in this case, L1 runs alone)
   are allowed to complete.
3. Orchestrator writes a **failure report** with:
   - The L1 PhaseReport (with failed commands and findings).
   - Skipped phases listed with `status: "skipped"`.
   - Overall Quality Gate: FAIL.
4. Exit.

### 5.2 Mechanical phase failure (L2-L5)

If L2-L5 fail:
1. Mark the phase `status: "fail"`.
2. Continue with other parallel phases.
3. If `orchestration.retry_mechanical: true`, the orchestrator relaunches
   the failed sub-agent ONCE with the same prompt.
4. After retry, status stays `fail` if it failed again.
5. The overall Quality Gate is marked FAIL iff any mechanical phase has
   status `fail` (not `warn`).

### 5.3 Cognitive phase failure / veto (L6, L9)

Cognitive phases produce subjective findings. They can't "fail" in the
mechanical sense, but they can issue **vetoes**:

- A veto is a structured objection from a loaded skill.
- Vetoes are recorded in the PhaseReport under `vetoes: [...]`.
- Vetoes do NOT abort the pipeline (per legacy `vantadb-certify` behavior).
- BUT if `veto_policy.escalate_if_vetoes_ge: N`, the orchestrator marks
  the overall Quality Gate as FAIL when a single phase has N or more vetoes.
  Default N = 3.

### 5.4 Sub-agent timeout

If a sub-agent doesn't return within `phase.timeout_ms`:

1. Orchestrator marks `status: "timeout"`.
2. Records `duration_ms: <timeout_ms>`.
3. No findings recorded for this phase.
4. Continues with other phases.
5. Includes a warning in the report: "Phase X timed out after Y minutes."

### 5.5 Sub-agent crash / malformed JSON

If a sub-agent returns non-JSON or crashes:

1. Orchestrator marks `status: "error"`.
2. Stores the raw output (truncated to 500 chars) in
   `phase_report.error_raw`.
3. Continues with other phases.
4. Includes a warning in the report.

### 5.6 Sub-agent type not found

If the profile references a `subagent_type` that isn't built-in and isn't
defined in `.opencode/agents/` or `opencode.json`:

1. Orchestrator falls back to `general`.
2. Records `subagent_type_used: "general"` (different from configured).
3. Emits a warning at the top of the report.
4. Continues.

## 6. Context budget verification

### 6.1 Orchestrator budget

The orchestrator should stay under 10% of the model's context window.
Approximate budget per mode:

| Mode | Phases | PhaseReports (~3KB each) | Orchestrator overhead | Total |
|------|--------|--------------------------|-----------------------|-------|
| quick | L0, L1 | 1 × 3KB | 5KB | ~8KB |
| certify | L0-L6 | 6 × 3KB | 10KB | ~28KB |
| review | L0, L1, L6, L9 | 4 × 3KB | 8KB | ~20KB |
| full | L0-L9 | 10 × 3KB + scoring (5KB) | 15KB | ~50KB |

Even on a 100K-context model, `full` mode consumes ~50KB / 100K = 0.5%.
The orchestrator has plenty of headroom for prompt parsing, profile
loading, and report writing.

### 6.2 Sub-agent budget

Each sub-agent runs in its own context. Worst case (L1 on VantaDB):

- Prompt: ~5KB (DetectionResult + phase config + diff context)
- Skill load (e.g. `code-review-and-quality`): ~10KB
- Command output (cargo check + clippy + nextest): ~30KB
- Findings JSON: ~2KB
- Total: ~47KB

On a 100K-context model, that's 47%. Sub-agent has plenty of headroom to
load multiple skills and process output.

### 6.3 What breaks the budget

Things that would blow the orchestrator's context:

- ❌ Streaming sub-agent logs into the orchestrator (logs are 30KB each × 10 phases = 300KB).
- ❌ Loading cross-cutting skills in the orchestrator (each is 5-15KB × 6 skills = 60KB).
- ❌ Re-reading source files in the orchestrator (a single 1000-line file is ~10KB).
- ❌ Including raw command output in the final report (the report should be
  structured markdown, not a log dump).

The skill is designed to avoid all of these. Raw logs go to
`docs/reviews/logs/` (if `keep_raw_logs: true`) and are linked from the
report, not embedded.

## 7. Subagent type definitions (reference)

Custom VantaDB subagent types referenced by the VantaDB profile. To use
them, create markdown files in `.opencode/agents/` — see OpenCode's agents
docs for the full format.

### vanta-worker.md

```markdown
---
description: VantaDB mechanical worker — runs cargo, pytest, wasm-pack, npm commands.
mode: subagent
model: anthropic/claude-sonnet-4-20250514
temperature: 0.1
permission:
  edit: deny
  bash:
    "*": ask
    "cargo *": allow
    "python *": allow
    "npm *": allow
    "npx *": allow
    "pwsh *": allow
    "git status *": allow
    "git diff *": allow
    "git log *": allow
    "wasm-pack *": allow
    "maturin *": allow
  webfetch: deny
  websearch: deny
---
You are a mechanical worker for the VantaDB project. You run commands,
capture their output, and report findings as structured JSON. You do not
make architectural judgments — that's vanta-arch's job. You do not do
security review — that's vanta-audit's job. You run the commands listed
in your prompt, evaluate the quality gates, and report.
```

### vanta-audit.md

```markdown
---
description: VantaDB security and cognitive reviewer.
mode: subagent
model: anthropic/claude-sonnet-4-20250514
temperature: 0.2
permission:
  edit: deny
  bash:
    "*": ask
    "cargo audit*": allow
    "cargo deny*": allow
    "grep *": allow
    "rg *": allow
    "git *": allow
  skill: allow
  webfetch: ask
---
You are a security and code review expert for VantaDB. You load
cross-cutting skills (code-review-and-quality, security-and-hardening,
doubt-driven-development, code-simplification) and apply them to the
diff provided in your prompt. You can VETO — record your objection
clearly with skill name and reason. You do not run mechanical checks
(cargo, npm) — that's vanta-worker's job.
```

### vanta-tuner.md

```markdown
---
description: VantaDB performance auditor.
mode: subagent
model: anthropic/claude-sonnet-4-20250514
temperature: 0.1
permission:
  edit: deny
  bash:
    "*": ask
    "cargo bench*": allow
    "cargo bloat*": allow
    "cargo build*": allow
    "git *": allow
  skill: allow
---
You are a performance engineer for VantaDB. You run benchmarks, check
binary size, and review hot paths for unnecessary allocations. You focus
on WAL write throughput, HNSW query latency, and binary size of
vantadb-server.
```

### vanta-docs.md

```markdown
---
description: VantaDB documentation reviewer.
mode: subagent
model: anthropic/claude-haiku-4-20250514
temperature: 0.1
permission:
  edit: deny
  bash:
    "*": ask
    "cargo doc*": allow
    "cargo test --doc*": allow
    "pwsh scripts/validate-docs-coverage*": allow
    "markdownlint*": allow
    "lychee *": allow
    "git *": allow
  skill: allow
---
You are a technical writer for VantaDB. You verify docs coverage, check
for broken links, and ensure every public API has rustdoc. You do not
modify docs — you only report findings.
```

### vanta-arch.md

```markdown
---
description: VantaDB architecture reviewer — uses codegraph.
mode: subagent
model: anthropic/claude-sonnet-4-20250514
temperature: 0.2
permission:
  edit: deny
  bash:
    "*": ask
    "codegraph*": allow
    "cargo modules*": allow
    "git *": allow
  skill: allow
---
You are a software architect for VantaDB. You use codegraph to verify
layering boundaries (WAL/HNSW/server), detect circular dependencies,
and identify god modules. You enforce VantaDB-specific invariants.
```

### vanta-lead.md

```markdown
---
description: VantaDB CI/CD parity reviewer — lead authority.
mode: subagent
model: anthropic/claude-sonnet-4-20250514
temperature: 0.2
permission:
  edit: deny
  bash:
    "*": ask
    "actionlint*": allow
    "cargo deny*": allow
    "cargo audit*": allow
    "npm audit*": allow
    "git *": allow
  skill: allow
---
You are a CI/CD lead for VantaDB. You verify CI/CD parity: every new
dependency in Cargo.toml/package.json/pyproject.toml has a corresponding
install step in .github/workflows/*.yml. You have authority to flag
issues that would break the release pipeline.
```

## 8. Extension points

### 8.1 Add a new phase

1. Add an entry to `phases:` in `default.yml` (or your project profile).
2. Use the next available ID (L12, L13, ...). L0-L11 are reserved.
3. Add the phase ID to the relevant `modes:` entries.
4. If the phase should be parallel, set `parallel: true` and pick a
   `subagent_type`.
5. Document it in `SKILL.md → Phase Catalog`.

### 8.2 Add a new profile

1. Create `profiles/<myproject>.yml`.
2. Set `profile.inherits: default` (or `vantadb`).
3. Override only what differs.
4. Use with `/review <mode> --profile myproject`.

### 8.3 Add a new scoring system

1. Add an entry under `scoring.weights` (must keep sum at 100).
2. Update each phase's `scoring_breakdown` to include the new system.
3. Update the report template in `SKILL.md → Report Format` to render it.

### 8.4 Add a new integration

1. Add an entry under `integrations:` in the profile.
2. Implement the detection logic in Phase 0 (check for available tools).
3. Implement the integration behavior in the relevant phase or in L10/L11.
4. Document it in `SKILL.md`.

## 9. Testing the skill

### 9.1 Manual smoke test

```
/review quick
```

Should:
- Run Phase 0 (detect).
- Launch one sub-agent for L1.
- Return a single-page report at `docs/reviews/review-quick-<timestamp>.md`.
- Take < 2 minutes.

### 9.2 Verify context budget

After running `/review full --profile vantadb`:

1. Check the orchestrator's session token count. Should be < 50K tokens
   (well under the model's context window).
2. Check that each sub-agent session exists as a child session
   (`session_child_first` keybind).
3. Verify the report contains links to raw logs (if `keep_raw_logs: true`),
   not inline logs.

### 9.3 Verify failure handling

1. Introduce a compile error in a Rust file.
2. Run `/review certify --profile vantadb`.
3. Verify: L1 fails, pipeline aborts, L2-L6 are skipped, report shows
   Quality Gate: FAIL, the L1 failure is in the report.

### 9.4 Verify Ponytail integration

1. Set Ponytail mode to `lite` (`/ponytail lite` or env var).
2. Run `/review review --profile vantadb`.
3. Verify: L6 is skipped, no `DIRECTION/Over-engineering` findings in report.

4. Set Ponytail mode to `ultra`.
5. Run the same command.
6. Verify: any `DIRECTION/Over-engineering` findings are escalated one
   severity level.

## 10. Open questions / future work

- **Streaming consolidation**: currently the orchestrator waits for all
  sub-agents before consolidating. Could stream partial results if a phase
  exceeds timeout, but this complicates the contract.
- **Cross-project profile inheritance**: currently `inherits` is a single
  string. Could be an array for diamond inheritance, but the merge semantics
  get complex.
- **Per-phase model selection**: profiles can already override
  `subagent_type`, but not the underlying model. Could add a `model:` field
  per phase (e.g. use Haiku for L5 docs, Sonnet for L9 review, Opus for L6
  architecture).
- **Persistent findings database**: store findings across runs to detect
  regressions and track fix velocity. Would need a `findings.jsonl` per
  project.
- **Plan file diffing**: when running `/review certify` repeatedly, diff
  the new plan file against the previous one to highlight what changed.

---

_Document version 1.0.0 — 2026-07-26. Matches unified-review skill v1.0.0._
