---
name: unified-review
description: >
  Universal project review, audit, and certification gate for OpenCode.
  Orchestrates parallel sub-agents (via the task tool) with auto-detection of
  language, bindings, web framework, CI/CD, and docs. Ships with a generic
  profile that works on any software project (Rust, Python, TS, Go, mixed)
  and a VantaDB profile that replaces vantadb-full-review, vantadb-certify,
  and vantadb-audit with 1:1 or superior capability. Modes: quick, certify,
  review, full. Configurable scoring (ISO 25010, SonarQube, CII, OWASP,
  CodeClimate) and a 12-category findings taxonomy. Respects Ponytail lazy
  mode. Integrates with Campaign task system if available. Entry points:
  skill unified-review, /review [mode] [--profile NAME], /audit (legacy alias).
compatibility: opencode
license: MIT
metadata:
  audience: maintainers
  category: review-audit
  replaces: vantadb-full-review,vantadb-certify,vantadb-audit
  version: "1.0.0"
---

# Unified Review & Audit

> **Canonical, project-agnostic orchestrator** for code review, audit, and
> pre-push certification. Built around **parallel sub-agent fan-out** so the
> orchestrator stays under 10% context budget even on large workspaces
> (VantaDB is ~790 files / 17+ crates and runs end-to-end without truncation).
> Replaces the three legacy skills `vantadb-full-review`, `vantadb-certify`,
> and `vantadb-audit` with a single installable skill plus two YAML profiles.

---

## What I do

1. **Phase 0 — Detect.** I inspect the working tree, identify the project's
   languages, bindings, web framework, CI/CD, docs, and optional tooling
   (codegraph, etc.), and load the matching profile.
2. **Phase 1..N — Fan-out.** I launch one sub-agent per phase in parallel
   using the `task` tool. Each sub-agent runs mechanical checks (compile,
   lint, test), executes cognitive review skills, and returns a structured
   JSON report.
3. **Consolidate — Fan-in.** I aggregate findings from every sub-agent,
   compute scores, detect cross-cutting patterns, and build a single
   prioritized report.
4. **Report.** I write the final markdown report to
   `docs/reviews/review-<mode>-<timestamp>.md` and (optionally) update the
   Campaign task system.

## When to use me

| Trigger | Suggested mode |
|---------|---------------|
| "Did I break anything?" before saving | `quick` |
| About to `git push` | `certify` |
| Reviewing a PR / diff | `review` |
| Quarterly deep review / release prep | `full` |
| Any time you would have run `/audit` | `full` (alias) |

I auto-detect the project. If a profile for the current project exists
(e.g. `profiles/vantadb.yml`), use it explicitly with `--profile vantadb`.

---

## Entry Points

| Entry point | Behavior |
|-------------|----------|
| `skill unified-review` | Load this skill. Auto-detect project + run `review` mode with the default profile. |
| `/review` | Same as above — interactive. |
| `/review quick` | Only Phase 0 + Phase 1 (core language check). One sub-agent. |
| `/review certify` | Pre-push gate. All mechanical phases in parallel; abort on critical fail. |
| `/review review` | Core check + architecture + cognitive code review. Skills may veto. |
| `/review full` | All phases + scoring (ISO/Sonar/CII/OWASP/CodeClimate if enabled) + security + performance. |
| `/review --profile NAME` | Use profile `profiles/NAME.yml`. |
| `/review MODE --profile NAME` | Mode + profile combination. |
| `/audit` | **Legacy alias** for `/review full`. Kept for backwards compatibility with `vantadb-audit`. |
| `/audit quick\|certify\|review\|full` | Legacy alias for the corresponding `/review` mode. |

---

## Detection Engine (Phase 0)

Phase 0 runs **synchronously in the orchestrator** — no sub-agent. It must
complete in seconds and consume minimal context. It produces a `DetectionResult`
JSON object that every downstream sub-agent receives in its prompt.

### Steps (in order)

1. **Working tree state**
   ```
   git status --short
   git diff --name-only HEAD
   git log --oneline -10
   ```
   If nothing changed and mode is `quick`/`certify` → suggest `full` and stop.

2. **Language detection** — check marker files at the repo root and inside
   workspace subdirectories:

   | Marker | Language | Notes |
   |--------|----------|-------|
   | `Cargo.toml` (with `[workspace]`) | Rust (multi-crate) | Inspect `members` |
   | `Cargo.toml` (single) | Rust | |
   | `pyproject.toml` / `setup.py` / `setup.cfg` | Python | Check `[build-system]` for maturin |
   | `package.json` | TypeScript / JavaScript | Inspect `workspaces` |
   | `go.mod` | Go | |
   | `pom.xml` / `build.gradle` | Java / Kotlin | |
   | `CMakeLists.txt` | C / C++ | |
   | `mix.exs` | Elixir | |
   | `Gemfile` | Ruby | |

3. **Bindings / SDK detection**
   - `pyproject.toml` with `maturin` build-backend → Python bindings via PyO3
   - `Cargo.toml` with `crate-type = ["cdylib"]` + `wasm-bindgen` → WASM
   - `package.json` + `tsconfig.json` + `index.ts` → TypeScript SDK
   - `bindings/` or `sdk/` directory → inspect contents

4. **Web framework detection**
   - `next.config.{js,ts,mjs}` → Next.js
   - `vite.config.{js,ts}` → Vite
   - `astro.config.{js,ts,mjs}` → Astro
   - `nuxt.config.{js,ts}` → Nuxt
   - `remix.config.{js,ts}` → Remix
   - `svelte.config.js` → SvelteKit
   - `angular.json` → Angular
   - Plain `index.html` + framework-less → static

5. **CI/CD detection**
   - `.github/workflows/*.yml` → GitHub Actions
   - `.gitlab-ci.yml` → GitLab CI
   - `.circleci/config.yml` → CircleCI
   - `azure-pipelines.yml` → Azure Pipelines
   - `Jenkinsfile` → Jenkins
   - `.drone.yml` → Drone

6. **Documentation detection**
   - `docs/` directory → check for mdbook, docusaurus, mkdocs, sphinx, etc.
   - `README.md`, `CHANGELOG.md`, `CONTRIBUTING.md`, `SECURITY.md`
   - `docs/plans/` directory (Campaign integration)

7. **Optional tooling detection**
   - `.codegraph/` directory or `codegraph.json` → CodeGraph MCP available
   - `.opencode/agents/*.md` → custom sub-agents available (read names)
   - `ponytail` in environment → Ponytail plugin active (read mode)

8. **Profile loading and merge**
   - Always load `profiles/default.yml` first.
   - If `--profile NAME` was passed, load `profiles/NAME.yml` and deep-merge
     on top of default.
   - Else if a file `profiles/<auto-detected-project>.yml` exists, load it.
   - Else stay with default only.
   - Merge semantics: scalars override, objects deep-merge, arrays replace
     (except `phases`, which is merged by `id`).

### DetectionResult JSON contract

Every sub-agent receives this in its prompt so it doesn't need to re-detect:

```json
{
  "profile": "vantadb",
  "mode": "certify",
  "working_tree": {
    "dirty": true,
    "changed_files": ["crates/vantadb-server/src/routes.rs", "Cargo.toml"],
    "staged_files": [],
    "recent_commits": ["abc1234 feat: ...", "def5678 fix: ..."]
  },
  "languages": ["rust", "python", "typescript"],
  "bindings": {"python": "maturin", "wasm": "wasm-pack", "typescript": "tsc"},
  "web_framework": "next",
  "ci_system": "github-actions",
  "docs_system": "mdbook",
  "codegraph_available": true,
  "ponytail_mode": "full",
  "campaign_tools_available": true,
  "phases_to_run": ["L0", "L1", "L2", "L3", "L4", "L5", "L6"]
}
```

---

## Profile System

Profiles are YAML files in `.opencode/skills/unified-review/profiles/`.
Two ship with the skill:

| File | Purpose |
|------|---------|
| `profiles/default.yml` | Generic. Auto-detects Rust/Python/TS/Go/etc. Works on any project. |
| `profiles/vantadb.yml` | VantaDB-specific. Inherits from `default`, adds Cargo workspace commands, VantaDB paths, codegraph integration, ISO 25010 + SonarQube + CII + OWASP + CodeClimate scoring, and pre-push hook generator. |

### Schema (validated by the orchestrator at load time)

```yaml
profile:
  name: string                    # must match filename
  description: string
  inherits: [string]              # optional, name of base profile

project:
  type: string                    # library | service | monorepo | embedded
  languages: [string]             # auto-detected, can be overridden
  bindings: [string]
  web_framework: string
  ci_system: string

phases:                           # see Phase Catalog below for the full list
  - id: string                    # L0..L11
    name: string
    description: string
    critical: bool                # abort pipeline on fail if true
    parallel: bool                # launch as sub-agent if true
    subagent_type: string         # built-in: general|explore|scout
                                   # custom: vanta-worker|vanta-audit|...
    detect: [string]              # conditions; phase runs only if all match
    commands:
      <language>: [string]        # shell commands per language
    skills_to_load: [string]      # cross-cutting skills loaded inside sub-agent
    timeout_ms: int               # default 300000 (5 min)
    score_max: int                # default 10
    quality_gates: [string]       # list of conditions for pass/fail

modes:
  quick:    [phase_id, ...]
  certify:  [phase_id, ...]
  review:   [phase_id, ...]
  full:     [phase_id, ...]

scoring:
  enabled: bool                   # default false in default.yml, true in vantadb.yml
  weights:                        # must sum to 100
    iso_25010: int
    sonarqube: int
    cii: int
    owasp: int
    codeclimate: int
  thresholds:
    pass_score: int               # default 7
    rating_bands:                 # score → rating
      A: int                      # >= 9
      B: int                      # >= 7
      C: int                      # >= 5
      D: int                      # >= 3
      E: int                      # < 3

findings:
  categories:                     # 12 canonical categories (see Taxonomy)
    - id: LOGIC
      name: string
      subcategories: [string]
    # ... 11 more

orchestration:
  max_parallel: int               # default 4
  timeout_ms: int                 # default 300000
  abort_on_critical_fail: bool    # default true
  retry_mechanical: bool          # default true (one retry on L1 fail)

pre_push_hook:
  enabled: bool
  platform: string                # powershell | bash
  template_file: string           # path to template

integrations:
  ponytail:
    respect_mode: bool            # default true
    skip_phases_in_lite: [phase_id]
    escalate_in_ultra: [phase_id]
  campaign:
    enabled: string               # auto | always | never
    plan_file_pattern: string
  codegraph:
    use_if_available: bool
    phase: string                 # which phase uses it (default L0)
```

### Adding a custom profile

1. Create `profiles/<myproject>.yml` next to `default.yml`.
2. Set `profile.inherits: default` to inherit all base behavior.
3. Override only what differs. Arrays replace; objects deep-merge.
4. Run `/review --profile myproject`.

---

## Modes Matrix

| Mode | Phases run | Critical gate | Cognitive review | Scoring | Typical duration |
|------|------------|---------------|-------------------|---------|------------------|
| `quick` | L0 + L1 | L1 must pass | no | no | < 1 min |
| `certify` | L0 + L1 + L2 + L3 + L4 + L5 + L6 | L1 must pass; L2-L5 must pass | L6 warns (no abort) | no | 5-15 min |
| `review` | L0 + L1 + L6 + L9 | L1 must pass | L9 may veto (records objection, continues) | no | 5-10 min |
| `full` | L0 + L1 + L2 + L3 + L4 + L5 + L6 + L7 + L8 + L9 | L1 must pass | L9 may veto | yes (if scoring.enabled) | 15-45 min |

The mode controls **which phases run**. The profile controls **how each phase
runs** (commands, subagent type, thresholds). They compose freely.

---

## Phase Catalog

12 phases. L0 and L10-L11 always run inside the orchestrator. L1-L9 are
fan-out candidates (each runs in its own sub-agent when `parallel: true`).

| ID | Name | Critical | Parallel | Default subagent | Description |
|----|------|----------|----------|------------------|-------------|
| L0 | Diff Impact Analysis | no | no (orchestrator) | — | `git diff` analysis; if codegraph available, query affected modules |
| L1 | Core Language Check | **yes** | yes | `general` | Compile, lint, unit tests for the primary language(s) |
| L2 | Bindings / SDKs | no | yes | `general` | Build + test Python/WASM/TS bindings if detected |
| L3 | Web Frontend | no | yes | `general` | Lint, typecheck, build the web app if detected |
| L4 | CI/CD + Dependencies | no | yes | `general` | Workflow YAML lint, CI parity check, dep audit |
| L5 | Documentation | no | yes | `general` | Docs coverage, broken links, freshness |
| L6 | Architecture Review | no | yes | `general` | Structural analysis via codegraph or static inspection |
| L7 | Security Audit | no | yes (mode=full) | `general` (or `vanta-audit` in VantaDB profile) | `cargo audit`, `npm audit`, `pip-audit`, unsafe review, OWASP checks |
| L8 | Performance Audit | no | yes (mode=full) | `general` (or `vanta-tuner` in VantaDB) | Benchmarks, bundle size, profile-guided review |
| L9 | Code Review (cognitive) | no | yes (mode=review/full) | `general` (or `vanta-audit` in VantaDB) | Multi-axis review via `code-review-and-quality` and related skills. May veto. |
| L10 | Findings Consolidation | no | no (orchestrator) | — | Aggregate, dedupe, detect cross-cutting patterns |
| L11 | Final Report | no | no (orchestrator) | — | Write `docs/reviews/review-<mode>-<timestamp>.md` |

Each phase has a `detect` field in its profile entry. A phase is **skipped
silently** when its `detect` conditions don't match (e.g. L3 Web Frontend
is skipped if no web framework was detected).

---

## Fan-Out Orchestration

### High-level pattern

```
ORCHESTRATOR (Phase 0 + L10 + L11 — context budget < 10%)
│
│  Phase 0: detect (sync, fast)
│
│  Launch L1 ALONE (critical — must pass before fan-out)
│    └── task("L1-core-language", prompt, subagent_type=...)
│        └── returns { phase, status, score, findings }
│
│  if L1 failed AND orchestration.abort_on_critical_fail:
│      → write failure report, exit
│
│  Launch L2..L6 in PARALLEL (max orchestration.max_parallel at once)
│  ├── task("L2-bindings",   prompt, subagent_type=...)
│  ├── task("L3-web",        prompt, subagent_type=...)
│  ├── task("L4-ci-cd",      prompt, subagent_type=...)
│  ├── task("L5-docs",       prompt, subagent_type=...)
│  └── task("L6-arch",       prompt, subagent_type=...)
│
│  (only phases whose `detect` matched are launched)
│
│  Await all (with per-task timeout = orchestration.timeout_ms)
│
│  if mode == "full":
│      Launch L7, L8, L9 in PARALLEL
│      ├── task("L7-security",   prompt, subagent_type=...)
│      ├── task("L8-performance",prompt, subagent_type=...)
│      └── task("L9-code-review",prompt, subagent_type=...)
│
│  Phase L10: consolidate (sync, orchestrator)
│  Phase L11: write report (sync, orchestrator)
```

### Sub-agent invocation contract

Use the `task` tool to launch each sub-agent. The exact argument shape
matches OpenCode's `task` tool: `description`, `prompt`, `subagent_type`.

**Example — generic Rust project:**

```
task(
  description = "L1 core language check (Rust)",
  subagent_type = "general",
  prompt = """
You are running Phase L1 (Core Language Check) of the unified-review skill.

DetectionResult:
{...paste DetectionResult JSON here...}

Phase config:
{...paste the L1 entry from the profile...}

Run the following commands in order and capture pass/fail + output summary:
  1. cargo fmt --all -- --check
  2. cargo check --workspace --all-targets
  3. cargo clippy --workspace --all-targets -- -D warnings
  4. cargo test --workspace

For each command, record: command, exit_code, output_summary (≤ 200 chars).

Then scan the diff (file list provided in DetectionResult) for:
  - new `unsafe` blocks
  - new `unwrap()` / `expect()` calls on user input
  - new dependencies in Cargo.toml

Return a JSON report and ONLY the JSON report:

{
  "phase": "L1",
  "status": "pass" | "fail" | "warn",
  "score": 0-10,                   // 10 if all pass, deduct 2 per failed command, min 0
  "duration_ms": <int>,
  "commands_run": [
    {"cmd": "...", "exit": 0, "summary": "..."}
  ],
  "findings": [
    {
      "id": "H01-CODE-001",
      "severity": "critical|high|medium|low|info",
      "category": "CODE",
      "file": "src/lib.rs",
      "line": 142,
      "description": "unwrap() on user input",
      "recommendation": "Use ok_or_else with proper error"
    }
  ]
}

Do not produce any prose outside the JSON. Do not write files.
"""
)
```

**Example — VantaDB certify (uses vanta-worker subagent):**

```
task(
  description = "L1 core language check (VantaDB Rust workspace)",
  subagent_type = "vanta-worker",
  prompt = """..."""
)
```

### Subagent type resolution

The profile specifies `subagent_type` per phase. The orchestrator verifies
the type exists (built-in or defined in `opencode.json` / `.opencode/agents/`)
before launching. If missing, it falls back to `general` and emits a warning
in the report.

| Type | Built-in? | When to use |
|------|-----------|-------------|
| `general` | yes | Default for mechanical phases |
| `explore` | yes | Read-only codebase exploration (L6 architecture when no codegraph) |
| `scout` | yes | External dependency research (L4 dep audit) |
| `vanta-worker` | no (custom) | VantaDB mechanical phases (cargo workspace) |
| `vanta-audit` | no (custom) | VantaDB security + cognitive review |
| `vanta-tuner` | no (custom) | VantaDB performance |
| `vanta-docs` | no (custom) | VantaDB docs |
| `vanta-arch` | no (custom) | VantaDB architecture |
| `vanta-lead` | no (custom) | VantaDB CI/CD parity (lead-level review) |

To define custom subagent types, create markdown files in
`.opencode/agents/` (e.g. `.opencode/agents/vanta-worker.md`) — see the
OpenCode agents documentation for the format.

### Concurrency control

- `orchestration.max_parallel` (default 4) caps simultaneous sub-agents.
- L1 always runs alone first (because L2-L6 depend on it conceptually).
- L7-L9 launch as a second wave after L1-L6 complete (mode `full` only).
- The orchestrator **never** streams sub-agent logs into its own context —
  it only reads the final JSON report from each `task` call.

### Failure handling matrix

| Phase | Failure type | Action |
|-------|-------------|--------|
| L1 (critical) | any | Abort. Write failure report. Exit. |
| L2-L5 (mechanical) | command failure | Mark `fail`, continue other parallel phases. |
| L6 (cognitive) | skill veto | Mark `warn`, record objection, continue. |
| L7-L8 (optional) | failure | Mark `warn`, continue. |
| L9 (cognitive review) | skill veto | Mark `warn`, record veto, continue. Pipeline result includes veto count. |
| any | sub-agent timeout | Mark `timeout`, continue. Include `timeout: true` in report. |
| any | sub-agent crash / no JSON | Mark `error`, continue. Log raw output truncated to 500 chars. |

### Retry policy

- Mechanical phases (L1-L5): one automatic retry on failure
  (`orchestration.retry_mechanical: true`).
- Cognitive phases (L6, L9): no retry — results are subjective, retrying
  wastes context.
- Optional phases (L7, L8): no retry.

---

## Fan-In Consolidation (Phase L10)

After all sub-agents return (or timeout), the orchestrator runs L10
synchronously. It does **not** spawn any sub-agent.

### Steps

1. **Aggregate findings** from every sub-agent's JSON report into a single
   list. Tag each finding with its source phase.
2. **Dedupe** findings that reference the same `file:line` and same
   `category` — keep the highest severity, merge recommendations.
3. **Detect cross-cutting patterns** — same finding in 2+ phases (e.g.
   unwrap-on-user-input found in both L1 and L9) gets a `cross_cutting: true`
   flag and is escalated one severity level.
4. **Compute scores** per phase and overall:
   - If `scoring.enabled`: compute weighted score using
     `scoring.weights` and the per-system scores from each phase.
   - Else: simple average of per-phase scores.
5. **Compute Quality Gate** — pass iff every critical phase passed and no
   finding has severity `critical`.
6. **Build summary tables** for the report (see Report Format below).
7. **Apply Ponytail mode** (if `integrations.ponytail.respect_mode: true`):
   - In `lite` mode: drop all findings whose category is `DIRECTION` and
     subcategory is `Over-engineering` (lazy mode tolerates it).
   - In `ultra` mode: escalate all `DIRECTION/Over-engineering` findings
     by one severity level.
8. **Update Campaign task** (if `integrations.campaign.enabled: auto` and
   campaign tools are available): write findings as subtasks, update task
   state to `completed` or `failed`.

---

## Findings Taxonomy (12 categories)

Every finding must be classified into exactly **one primary category** and
may carry **secondary categories**. The taxonomy is the same one used by
the legacy `vantadb-full-review` skill, generalized (VantaDB-specific
subcategories moved to the VantaDB profile).

| # | ID | Name | What it covers |
|---|----|------|----------------|
| 1 | `LOGIC` | Logic failures | Off-by-one, race conditions, incorrect branching, state management, edge cases, input validation, error handling, type confusion, serialization, async/sync mismatch |
| 2 | `PATTERN` | Pattern failures | GoF violations, language anti-pattern (Rust/Python/TS), framework anti-patterns (React/CSS), API design, database anti-patterns |
| 3 | `ARCH` | Architecture failures | Circular deps, layering violations, god objects, feature leakage, tight coupling, missing/leaky/premature abstractions |
| 4 | `DIRECTION` | Project direction | Scope creep, abandoned effort, unprioritized tech debt, strategic misalignment, over/under-engineering, missing roadmap |
| 5 | `CLARITY` | Clarity failures | Confusing naming, magic numbers, deep nesting, missing comments, dead code, incomprehensible tests, stale docs, confusing errors |
| 6 | `CODE` | Code quality | Compiler warnings, lints, lint suppressions, unsafe without SAFETY, unnecessary clones/allocs, unsafe casts, TOCTOU, error swallowing, panics in library code |
| 7 | `DESIGN` | Design / UX failures | Visual inconsistency, accessibility, broken responsive, excessive motion, AI slop, touch targets, loading/empty/error states |
| 8 | `ERROR` | Concrete errors | Compilation errors, test failures, flaky tests, runtime panics, deadlocks, memory leaks, CI failures, security vulnerabilities, dependency conflicts |
| 9 | `MISSING` | Missing elements | Missing tests, validation, error handling, docs, CI gates, monitoring, recovery, migrations, env config |
| 10 | `FEATURE` | Missing features | Feature gaps vs competitors, missing integrations, missing platform support, missing observability features |
| 11 | `ALGO` | Algorithmic failures | Wrong complexity (O(n²) where O(n) possible), incorrect data structure choice, numerical instability, off-by-one in algorithms |
| 12 | `ANY` | Catch-all | Use only when no other category fits. Always reviewed manually before publishing the report. |

### Finding ID format

```
H<phase-number><CATEGORY>-<NNN>
```

Examples:
- `H01-CODE-001` = Phase L1 (core language), CODE category, finding #1
- `H03-DESIGN-007` = Phase L3 (web frontend), DESIGN category, finding #7
- `H09-ARCH-002` = Phase L9 (code review), ARCH category, finding #2

### Severity levels

| Severity | Meaning | SLA |
|----------|---------|-----|
| `critical` | Blocks release / data loss / security hole | Fix before push |
| `high` | Will break in production soon | Fix this iteration |
| `medium` | Real problem, manageable | Backlog |
| `low` | Minor, polish | When convenient |
| `info` | Observation, no action required | None |

---

## Scoring System

### Default (simplified, in `default.yml`)

Each phase returns `score: 0-10`. The overall score is the unweighted
average across all phases that ran. Quality Gate passes iff:
- every critical phase passed (status = `pass`)
- overall score >= `scoring.thresholds.pass_score` (default 7)

No ISO/Sonar/CII/OWASP/CodeClimate breakdown — those are opt-in via the
profile.

### Extended (enabled in `vantadb.yml`)

When `scoring.enabled: true`, each phase also returns scores per system.
The orchestrator computes:

| System | Weight (configurable) | What it measures |
|--------|----------------------|------------------|
| ISO 25010 | 20% (default) | 8 characteristics scored 0-10 each |
| SonarQube Quality Gate | 25% | 9 conditions ✅/❌ |
| CII Best Practices | 20% | None / Passing / Silver / Gold |
| OWASP ASVS | 15% | L1 / L2 / L3 |
| CodeClimate / Qlty | 20% | A / B / C / D / E |

Final score = `Σ (system_score × weight)`. Weights must sum to 100 (validated
at profile load). Profiles can override weights or disable systems they
don't want.

---

## Report Format

The orchestrator writes the report at Phase L11. The format is controlled by
the `report` block in the profile:

```yaml
report:
  format: markdown             # markdown | html | both
  markdown_template: ~         # built-in (see Markdown template below)
  html_template: "templates/report.html.tmpl"
  output_dir: "docs/reviews"
  filename_pattern: "review-<mode>-<timestamp>"
```

- `markdown` (default in `default.yml`) — writes `review-<mode>-<timestamp>.md`.
  Portable, git-diffable, ideal for code review comments and PRs.
- `html` — writes `review-<mode>-<timestamp>.html`. Self-contained (inline CSS,
  no external dependencies), print-friendly, shareable with non-technical
  stakeholders. Renders the same content as the markdown report but visually
  polished with color-coded scores, severity badges, collapsible per-phase
  detail, and ISO 25010 heatmap.
- `both` (default in `vantadb.yml`) — writes both files. Markdown for git
  diffs and code review; HTML for sharing with stakeholders.

### Markdown template

Written to `docs/reviews/review-<mode>-<timestamp>.md` where timestamp is
`YYYY-MM-DD-HHMM` in the project's local timezone.

```markdown
# Unified Review — <mode> — <YYYY-MM-DD>

**Profile:** <profile-name>
**Mode:** <mode>
**Duration:** <N> min
**Quality Gate:** ✅ PASS / ❌ FAIL
**Overall score:** <N>/10  (rating: A|B|C|D|E)

## Executive Summary

<2-3 paragraphs. What was checked, what passed, what failed,
what to do next. Plain language, no jargon.>

## Scoreboard

| Phase | Status | Score | Findings (C/H/M/L/I) | Duration |
|-------|--------|-------|----------------------|----------|
| L0 Diff Impact | ✅ | — | 0/0/0/0/0 | 2s |
| L1 Core Language | ✅ | 9/10 | 0/1/2/0/0 | 1m 12s |
| L2 Bindings | ⚠️ | 7/10 | 0/0/3/1/0 | 45s |
| L3 Web | ⏭️ skip | — | — | — |
| L4 CI/CD | ✅ | 10/10 | 0/0/0/0/0 | 8s |
| L5 Docs | ❌ | 4/10 | 1/2/1/0/0 | 12s |
| L6 Architecture | ⚠️ | 6/10 | 0/1/4/2/0 | 1m 30s |
| **OVERALL** | ❌ | **7.2/10** | **1/4/10/3/0** | 4m 09s |

## Findings by Category

| Category | Critical | High | Medium | Low | Info | Total |
|----------|----------|------|--------|-----|------|-------|
| LOGIC | 0 | 0 | 1 | 0 | 0 | 1 |
| PATTERN | 0 | 1 | 0 | 0 | 0 | 1 |
| ARCH | 0 | 0 | 2 | 1 | 0 | 3 |
| ... | | | | | | |
| **Total** | **1** | **4** | **10** | **3** | **0** | **18** |

## Top 5 Critical Findings

1. **[H05-MISSING-001]** `docs/CHANGELOG.md` missing — Recommend: add CHANGELOG.md following Keep-a-Changelog format. (severity: critical)
2. **[H01-CODE-003]** `src/lib.rs:142` `unwrap()` on user input — Recommend: use `ok_or_else`. (severity: high)
3. ...

## Cross-Cutting Patterns

- `unwrap()` on user input appears in 3 phases (L1, L2, L9) — likely a
  team-wide pattern issue. Recommend: add clippy lint `clippy::unwrap_used`
  to workspace `Cargo.toml`.

## Per-Phase Detail

### L1 — Core Language (9/10, ✅)

**Commands:**
- `cargo fmt --check`: ✅
- `cargo check --workspace`: ✅
- `cargo clippy -- -D warnings`: ⚠️ 1 warning (suppressed)
- `cargo test`: ✅ 234 passed / 0 failed

**Quality Gate conditions:**
- [x] No new compiler warnings
- [x] No new clippy lints
- [x] All tests pass
- [ ] Coverage ≥ 80% (currently 72%)

**Findings:**
- [H01-CODE-001] (high) `src/lib.rs:142` — `unwrap()` on user input
- [H01-CODE-002] (medium) `src/api.rs:88` — `.expect()` with generic message
- [H01-CODE-003] (low) `src/utils.rs:12` — comment refers to deleted function

### L2 — Bindings (7/10, ⚠️)
...

## Recommendations (prioritized)

1. **(critical, before push)** Add `CHANGELOG.md` — blocks certify gate.
2. **(high, this iteration)** Replace `unwrap()` calls on user input (3 occurrences).
3. **(medium, backlog)** Increase test coverage from 72% to 80%.
4. **(low, when convenient)** Update stale comments in `src/utils.rs`.

## ISO 25010 Heatmap (if scoring enabled)

| Characteristic | Score | Coverage |
|----------------|-------|----------|
| Functional suitability | 8/10 | 🟢 |
| Reliability | 7/10 | 🟡 |
| Performance efficiency | 9/10 | 🟢 |
| Compatibility | 8/10 | 🟢 |
| Usability | 6/10 | 🟡 |
| Security | 5/10 | 🔴 |
| Maintainability | 7/10 | 🟡 |
| Portability | 9/10 | 🟢 |

## SonarQube Quality Gate (if scoring enabled)

| Condition | Result |
|-----------|--------|
| No new reliability issues | ✅ |
| No new security issues | ❌ |
| No new maintainability issues | ✅ |
| Coverage ≥ 80% on new code | ❌ (72%) |
| ... | |
| **Overall Quality Gate** | **❌ FAIL** |

## CII Best Practices (if scoring enabled)

- **Current level:** Passing
- **Gaps for Silver:** license SPDX tag, vulnerability response policy
- **Target level:** Silver

---

_Generated by unified-review skill. Profile: <name>. Mode: <mode>._
_Based on ISO/IEC 25010, SonarQube Quality Gates, OpenSSF CII Best Practices,
OWASP ASVS v5.0, and CodeClimate/Qlty maintainability scoring._
```

### HTML template

Written to `docs/reviews/review-<mode>-<timestamp>.html` (when `report.format`
is `html` or `both`). Self-contained — inline CSS, no external CDN/fonts,
works offline, print-friendly, dark-mode aware via `prefers-color-scheme`.

The template lives at `templates/report.html.tmpl` and uses two substitution
syntaxes the orchestrator fills at L11:

| Syntax | Meaning | Example |
|--------|---------|---------|
| `{{NAME}}` | Scalar substitution | `{{MODE}}` → `certify` |
| `{{#NAME}}...{{/NAME}}` | Conditional block (rendered only if `NAME` is truthy) | `{{#HAS_SCORING}}...{{/HAS_SCORING}}` |

#### Full placeholder reference

| Placeholder | Type | Description |
|-------------|------|-------------|
| `{{REPORT_TITLE}}` | scalar | e.g. `Unified Review — certify — 2026-07-26` |
| `{{LANG}}` | scalar | ISO 639-1 lang code, e.g. `en`, `es` |
| `{{SKILL_VERSION}}` | scalar | e.g. `1.0.0` |
| `{{MODE}}` | scalar | `quick`, `certify`, `review`, `full` |
| `{{PROFILE}}` | scalar | e.g. `vantadb`, `default` |
| `{{TIMESTAMP}}` | scalar | human-readable generation time |
| `{{DURATION}}` | scalar | total pipeline duration (e.g. `4m 09s`) |
| `{{PONYTAIL_MODE}}` | scalar | `off`, `lite`, `full`, `ultra` |
| `{{CAMPAIGN_TASK_ID}}` | scalar | campaign task ID (if campaign enabled) |
| `{{#CAMPAIGN_TASK_LINK}}...{{/CAMPAIGN_TASK_LINK}}` | block | render campaign row only if campaign active |
| `{{QG_CLASS}}` | scalar | `pass`, `fail`, `warn` (controls banner color) |
| `{{QG_ICON}}` | scalar | `✅`, `❌`, `⚠️` |
| `{{QG_STATUS}}` | scalar | `PASS`, `FAIL`, `WARN` |
| `{{OVERALL_SCORE}}` | scalar | 0-10 (one decimal) |
| `{{OVERALL_SCORE_CLASS}}` | scalar | `high` (≥8), `mid` (5-7), `low` (<5) — controls color |
| `{{RATING}}` | scalar | `A`, `B`, `C`, `D`, `E` |
| `{{EXECUTIVE_SUMMARY}}` | scalar | 2-3 paragraphs HTML (from L10) |
| `{{FINDINGS_CRITICAL}}` | scalar | count |
| `{{FINDINGS_HIGH}}` | scalar | count |
| `{{FINDINGS_MEDIUM}}` | scalar | count |
| `{{FINDINGS_LOW}}` | scalar | count |
| `{{FINDINGS_INFO}}` | scalar | count |
| `{{FINDINGS_TOTAL}}` | scalar | count |
| `{{PHASES_PASSED}}` | scalar | count |
| `{{PHASES_TOTAL}}` | scalar | count |
| `{{VETO_COUNT}}` | scalar | count |
| `{{SCOREBOARD_ROWS}}` | scalar | rendered `<tr>` rows for each phase |
| `{{TOP_CRITICAL_COUNT}}` | scalar | count |
| `{{TOP_CRITICAL_FINDINGS}}` | scalar | rendered finding cards (max 5) |
| `{{#NO_CRITICAL_FINDINGS}}...{{/NO_CRITICAL_FINDINGS}}` | block | rendered if 0 critical findings |
| `{{#HAS_CROSS_CUTTING}}...{{/HAS_CROSS_CUTTING}}` | block | render cross-cutting section only if non-empty |
| `{{CROSS_CUTTING_COUNT}}` | scalar | count |
| `{{CROSS_CUTTING_PATTERNS}}` | scalar | rendered cross-cutting cards |
| `{{CATEGORY_ROWS}}` | scalar | rendered `<tr>` rows for 12 categories |
| `{{PER_PHASE_DETAIL}}` | scalar | rendered `<details>` accordions |
| `{{#HAS_SCORING}}...{{/HAS_SCORING}}` | block | render scoring sections only if enabled |
| `{{ISO_HEATMAP_CELLS}}` | scalar | 8 heatmap cells (one per ISO 25010 characteristic) |
| `{{SONARQUBE_ROWS}}` | scalar | rendered `<tr>` rows for 9 conditions |
| `{{SONARQUBE_OVERALL}}` | scalar | `PASS` / `FAIL` |
| `{{SONARQUBE_OVERALL_CLASS}}` | scalar | `pass` / `fail` |
| `{{CII_CURRENT}}` | scalar | `None`, `Passing`, `Silver`, `Gold` |
| `{{CII_TARGET}}` | scalar | target level |
| `{{CII_GAPS}}` | scalar | rendered `<li>` list |
| `{{OWASP_LEVEL}}` | scalar | `L1`, `L2`, `L3` |
| `{{OWASP_GAPS}}` | scalar | rendered `<li>` list |
| `{{CC_RATING}}` | scalar | `A`, `B`, `C`, `D`, `E` |
| `{{CC_RATING_CLASS}}` | scalar | `pass` (A), `warn` (B-C), `fail` (D-E) |
| `{{CC_ISSUES}}` | scalar | rendered `<li>` list |
| `{{RECOMMENDATIONS}}` | scalar | rendered `<li>` items (priority-ordered) |
| `{{REPORT_FILE_PATH}}` | scalar | path to this report file |
| `{{#RAW_LOGS_DIR}}...{{/RAW_LOGS_DIR}}` | block | render raw logs row only if `keep_raw_logs` |

#### Substitution rules for the orchestrator (at L11)

1. **Scalar substitution**: replace `{{NAME}}` with the value. If value is
   missing or null, replace with empty string.
2. **Conditional blocks**: if `{{#NAME}}` value is truthy (non-empty, non-zero,
   not `false`), render the inner content (recursively, with substitutions).
   Otherwise, drop the entire block including markers.
3. **HTML escaping**: values from findings (descriptions, recommendations)
   MUST be HTML-escaped (`<` → `&lt;`, `>` → `&gt;`, `&` → `&amp;`,
   `"` → `&quot;`) before substitution. Scalar metadata (mode, profile,
   timestamp) does not need escaping.
4. **List rendering**: for `*_ROWS`, `*_CELLS`, `*_PATTERNS`, `*_GAPS`,
   `*_ISSUES`, `RECOMMENDATIONS`, `PER_PHASE_DETAIL`, `TOP_CRITICAL_FINDINGS` —
   the orchestrator renders each list item using the inner template
   (documented as HTML comments inside `report.html.tmpl`) and concatenates
   them with no separators.
5. **Output file**: write the final HTML to
   `docs/reviews/review-<mode>-<timestamp>.html`. Don't minify — keep it
   human-readable for git diffs.

---

## Ponytail Integration

Ponytail is a separate skill/plugin that injects a "lazy senior dev" ruleset
into the agent's context at session start. It has four modes:

| Mode | Behavior |
|------|----------|
| `off` | Ponytail inactive. Standard review. |
| `lite` | Tolerate over-engineering findings. Skip `ponytail-audit` in L6. Drop `DIRECTION/Over-engineering` findings. |
| `full` | Standard behavior. `ponytail-review` runs on diff in L9. |
| `ultra` | Aggressive. Escalate `DIRECTION/Over-engineering` findings by one severity. Add `ponytail-audit` to L6 even if not in profile. |

### How this skill respects Ponytail

- **Do NOT load Ponytail skills directly.** The user (or session config) has
  already loaded Ponytail. This skill reads the active mode and adapts.
- The active mode is read from:
  1. CLI flag: `/review --ponytail ultra`
  2. Environment variable: `PONYTAIL_MODE`
  3. Ponytail's session config (if detectable)
  4. Default: `full`
- Behavior per mode is configured in `integrations.ponytail` in the profile.
- In `lite` mode, this skill also reduces its own verbosity: skips L6
  architecture review entirely (it's the most opinionated phase).

---

## Campaign Task System Integration

Campaign is an MCP server that provides `campaign_*` tools for task tracking.
It is **not** built into OpenCode — it's an optional integration.

### Detection

At Phase 0, the orchestrator checks if `campaign_*` MCP tools are available
(by inspecting the available tool list). Sets
`DetectionResult.campaign_tools_available` accordingly.

### Behavior

| `integrations.campaign.enabled` | `campaign_tools_available` | Action |
|---------------------------------|----------------------------|--------|
| `auto` (default) | true | Use campaign tools |
| `auto` | false | Skip silently |
| `always` | true | Use campaign tools |
| `always` | false | Warn in report, skip |
| `never` | * | Skip |

### Campaign workflow

1. **At skill start**: call `campaign_get_next_task` to claim the review task.
   If no task is available, create one with `campaign_create_task`.
2. **After each phase**: call `campaign_update_task_state` with phase result.
3. **On critical findings**: call `campaign_memory_write` to record the
   decision and context.
4. **At skill end**: mark task as `completed` (Quality Gate passed) or
   `failed` (Quality Gate failed or critical phase aborted).
5. **Plan file**: write or update `docs/plans/plan-review-<timestamp>.md`
   with phase results and findings as subtasks.

### Plan file format

```markdown
# Plan — Unified Review <mode> — <timestamp>

task_id: <campaign task id>
profile: <name>
mode: <mode>
started_at: <ISO>
ended_at: <ISO>
status: completed | failed

## Phases

- L0: ✅ (2s)
- L1: ✅ 9/10 (1m 12s)
- L2: ⚠️ 7/10 (45s)
- ...

## Findings as subtasks

- [ ] (critical) [H05-MISSING-001] Add CHANGELOG.md
- [ ] (high) [H01-CODE-003] Replace unwrap() in src/lib.rs:142
- [ ] (medium) [H06-CODE-002] Add clippy::unwrap_used to workspace
```

---

## Context Budget

The whole point of fan-out is keeping the orchestrator lean. Targets:

| Component | Target % of context | Notes |
|-----------|--------------------|-------|
| Orchestrator (Phase 0 + L10 + L11) | < 10% | Only coordination + final report. Never streams sub-agent logs. |
| Each sub-agent | 15-20% | Runs one phase, returns JSON, exits. Sub-agent context is freed when the task call returns. |
| Consolidation (L10) | < 5% | Aggregate JSON only — no re-reading source files. |
| Final report (L11) | < 5% | Structured markdown, no verbose logs. |

If the orchestrator finds itself exceeding ~50% of context after L10, it
should emit a warning in the report and suggest splitting the review into
smaller modes (e.g. run `certify` instead of `full`).

---

## Permissions Required

This skill needs the following tool permissions in `opencode.json`:

```json
{
  "permission": {
    "bash": "allow",
    "read": "allow",
    "edit": "ask",
    "write": "allow",
    "task": "allow",
    "skill": "allow",
    "webfetch": "ask",
    "websearch": "ask"
  }
}
```

- `bash` — to run cargo, npm, pytest, git, etc.
- `read` — to inspect source files.
- `task` — to launch sub-agents (critical).
- `skill` — to load cross-cutting skills (code-review-and-quality, etc.).
- `write` — to write the report and plan files.
- `webfetch` / `websearch` — only needed for L7 security audit on dependencies
  and L4 supply-chain checks. Can be `ask` or `deny` if not available.

### Subagent type permissions

If using custom subagent types (e.g. `vanta-worker`), the orchestrator agent
needs `permission.task` to allow them:

```json
{
  "agent": {
    "build": {
      "permission": {
        "task": {
          "*": "deny",
          "general": "allow",
          "explore": "allow",
          "scout": "allow",
          "vanta-*": "allow"
        }
      }
    }
  }
}
```

---

## Files this skill reads

- `.opencode/skills/unified-review/profiles/default.yml` (always)
- `.opencode/skills/unified-review/profiles/<project>.yml` (if `--profile` or auto-detected)
- Project markers: `Cargo.toml`, `pyproject.toml`, `package.json`, `go.mod`, etc.
- `.github/workflows/*.yml` (L4 CI/CD parity)
- `.codegraph/` (L0 if available)
- `.opencode/agents/*.md` (to enumerate available subagent types)
- `.git/` (Phase 0 git status / diff)

## Files this skill writes

- `docs/reviews/review-<mode>-<timestamp>.md` — main report (always)
- `docs/plans/plan-review-<timestamp>.md` — plan file (if Campaign enabled)
- `docs/reviews/logs/<phase>-<timestamp>.log` — per-phase raw logs (only if `orchestration.keep_raw_logs: true`)

Raw sub-agent outputs are **never** embedded inline in the report. They are
either dropped or (if `keep_raw_logs`) written to the `logs/` subdirectory
and linked from the main report.

---

## Usage Examples

### Example 1 — Generic Python library, quick mode

```
/review quick
```

Auto-detects `pyproject.toml`, no bindings, no web, no CI yet.
Runs:
- L0 git diff analysis
- L1 `pytest`, `mypy src/`

Output: `docs/reviews/review-quick-2026-07-26-1430.md` (2-page report,
score, top 3 findings if any).

### Example 2 — Generic TypeScript monorepo, certify

```
/review certify
```

Auto-detects `package.json` with workspaces, `.github/workflows/ci.yml`.
Runs in parallel:
- L1 `tsc --noEmit`, `vitest run`
- L4 `actionlint` on workflows, npm dependency audit
- L5 docs coverage

Aborts if `tsc` fails. Otherwise writes a 4-6 page report.

### Example 3 — VantaDB pre-push certify

```
/review certify --profile vantadb
```

Auto-detects: Rust workspace (17+ crates), Python bindings (maturin),
WASM bindings (wasm-pack), Next.js frontend, GitHub Actions, codegraph.
Runs in parallel:
- L0 codegraph impact analysis on staged diff
- L1 `cargo fmt --check`, `cargo check --workspace`, `cargo clippy -- -D warnings`, `cargo nextest run --profile audit`
- L2 `pwsh dev-tools/scripts/validate_python_sdk.ps1`, `wasm-pack build`
- L3 `cd web && npm ci && npm run lint && npx tsc --noEmit && npm run build`
- L4 CI/CD parity: verify Cargo.toml deps have CI install steps
- L5 `pwsh scripts/validate-docs-coverage.ps1`
- L6 codegraph structural analysis

Aborts on L1 fail. Generates PowerShell pre-push hook if all pass.

### Example 4 — VantaDB quarterly full review

```
/review full --profile vantadb
```

Runs all phases (L0-L9) plus scoring (ISO 25010 + SonarQube + CII + OWASP +
CodeClimate). Adds:
- L7 `cargo audit`, `cargo deny check`, OWASP ASVS checklist
- L8 `cargo bloat --crates`, benchmark suite, profile-guided review
- L9 multi-skill code review (`code-review-and-quality`, `doubt-driven-development`,
  `code-simplification`, `security-and-hardening`, `deprecation-and-migration`)

Produces a comprehensive 15-25 page report with scorecard, ISO 25010
heatmap, SonarQube Quality Gate summary, CII assessment, and prioritized
recommendations.

### Example 5 — Legacy `/audit` migration

| Old | New |
|-----|-----|
| `/audit` | `/review full` (alias kept) |
| `/audit quick` | `/review quick` |
| `/audit certify` | `/review certify` |
| `/audit review` | `/review review` |
| `/audit full` | `/review full` |
| `vantadb-full-review` skill | `/review full --profile vantadb` |
| `vantadb-certify` skill | `/review certify --profile vantadb` |
| `vantadb-audit` skill | (this skill — `/audit` is now an alias) |

---

## Migration Guide (from legacy skills)

### From `vantadb-full-review` → `/review full --profile vantadb`

The legacy skill ran 8 sequential phases in the agent's main context. The
new skill:

- Runs the same 8 conceptual layers (now L1-L8) **in parallel sub-agents**.
- Preserves all 5 scoring systems (ISO 25010, SonarQube, CII, OWASP, CodeClimate).
- Preserves the 12-category findings taxonomy.
- Preserves the report format (executive summary, scoreboard, ISO heatmap,
  Sonar gate, prioritized issues).
- Adds codegraph impact analysis (L0) and explicit findings consolidation (L10).

**What you lose:** the legacy skill loaded ~25 cross-cutting skills inline.
The new skill loads them **inside the relevant sub-agent's context** instead,
so the orchestrator stays lean. The same skills are still loaded — just in
the right place.

### From `vantadb-certify` → `/review certify --profile vantadb`

The legacy skill ran 8 layers sequentially. The new skill:

- Runs the same layers (L0-L6 + L9 cognitive) in **parallel**.
- Preserves the CI/CD parity check (L4 in new skill, was L7a in legacy).
- Preserves the cognitive review with veto power (L9 in new skill).
- Preserves the PowerShell pre-push hook generator.
- Adds graceful degradation: if a non-critical phase fails, it continues
  instead of stopping.

### From `vantadb-audit` → `/audit` (alias) or `/review MODE`

The legacy skill was an orchestrator with 4 modes. The new skill:

- Absorbs the 4 modes (`quick`, `certify`, `review`, `full`) directly.
- `/audit` remains as an alias for `/review full` for backwards compatibility.
- `/audit quick|certify|review|full` map directly to `/review <mode>`.

---

## Validation

This skill self-validates at load time. If any of the following fail, it
refuses to run and prints a clear error:

1. Frontmatter `name` matches the directory name (`unified-review`).
2. Frontmatter `description` is ≤ 1024 characters.
3. `profiles/default.yml` exists and parses as valid YAML.
4. If `--profile NAME` was given, `profiles/NAME.yml` exists and parses.
5. Every phase ID referenced in `modes.*` exists in `phases[*].id`.
6. If `scoring.enabled: true`, `scoring.weights` sums to 100.
7. Every `subagent_type` referenced in `phases` is either built-in
   (`general`, `explore`, `scout`) or defined in `.opencode/agents/` /
   `opencode.json`. Missing custom types are warned but not blocked
   (falls back to `general`).

---

## Reference: Built-in OpenCode subagents

For profiles that don't define custom subagent types, this skill uses
OpenCode's built-ins:

| Type | Mode | Tools | Use for |
|------|------|-------|---------|
| `general` | subagent | all (except todo) | Default for mechanical phases (L1-L5). Can run bash, read, edit. |
| `explore` | subagent | read-only | L6 architecture review when no codegraph. Fast codebase exploration. |
| `scout` | subagent | read-only + external | L4 supply-chain audit, dependency research. Clones external repos to OpenCode cache. |

To go beyond built-ins, define custom subagents in `.opencode/agents/*.md`
or in `opencode.json` under the `agent` key. See OpenCode's agents docs
(https://opencode.ai/docs/agents) for the full format.

---

## Reference: Cross-cutting skills loaded inside sub-agents

These are NOT loaded by the orchestrator. Each is loaded by the sub-agent
that needs it, via the `skill({ name: "..." })` tool call inside the
sub-agent's prompt.

| Skill | Loaded by | Purpose |
|-------|-----------|---------|
| `code-review-and-quality` | L9 | 5-axis code review framework |
| `security-and-hardening` | L7, L9 | Input validation, trust boundaries |
| `performance-optimization` | L8 | Profile-guided review |
| `doubt-driven-development` | L9 | Adversarial review for high-stakes changes |
| `code-simplification` | L9 | Detect unnecessary complexity |
| `deprecation-and-migration` | L9 | When public API is removed |
| `systematic-debugging` | L9 | If known issues exist |
| `ponytail-review` | L9 | Over-engineering check on diff (Ponytail full/ultra only) |
| `ponytail-audit` | L6 | Over-engineering audit (Ponytail full/ultra only) |

If a skill is not installed, the sub-agent logs a warning and continues
without it. The finding categories that skill would have produced are
simply absent from that phase's report.

---

## Changelog

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-26 | Initial unified release. Replaces vantadb-full-review, vantadb-certify, vantadb-audit. Adds parallel sub-agent orchestration, profile system, Ponytail integration, Campaign task integration. |
