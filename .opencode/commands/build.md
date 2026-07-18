---
description: "Implement tasks incrementally — build, test, verify, commit. Add 'auto' to run the whole plan. For bugs: Prove-It pattern."
---

Invoke the agent-skills:incremental-implementation skill alongside agent-skills:test-driven-development.
For browser-related issues, also invoke agent-skills:browser-testing-with-devtools.
When the build involves framework/library choices or new dependencies, also invoke agent-skills:source-driven-development to verify official docs first.

## Modes

- **`/build`** — implement the *next* pending task, then stop (one slice at a time).
- **`/build auto`** — generate the plan if needed, get a single approval, then implement *every* task without stopping between them.
- **`/build prove`** — bug-fix mode (Prove-It pattern): reproduce the bug first, then fix.

**Agents:** Spawn `vanta-worker` (default implementer) or `vanta-engine` (algorithmic/vector work) via `task` tool with the matching `subagent_type`. For bugs, consider `vanta-chaos` for root cause.

**Sandbox:** For risky build commands (rm, format drive, destructive ops), use `campaign_run_sandboxed` (MCP) or `.opencode/task-system/sandbox/run-sandboxed.ps1` directly.

**State:** Always write `docs/last-build-state.json` after the build task completes. This is read by `/status` and `/ship`.

```json
{
  "timestamp": "ISO8601",
  "mode": "default|auto|prove",
  "tasks_completed": 0,
  "tasks_failed": 0,
  "last_sha": "abc1234",
  "build_ok": true
}
```

`$ARGUMENTS` selects the mode. Treat `auto` (canonical) or `all` as autonomous mode; `prove` as bug-fix mode; anything else (or empty) is the default single-task mode.

**Path sync:** pipeline.md saves plans to `docs/plans/`. build.md expects `tasks/plan.md`.
Bridge: `/build auto` generates `tasks/plan.md` if missing (via planning-and-task-breakdown).
For cross-command flow: `/pipeline plan` → `/build` (reads `tasks/plan.md` if present, else prompts for scope).

---

## Default: one task

Pick the next pending task from the plan. Then:

1. Read the task's acceptance criteria
2. Load relevant context (existing code, patterns, types)
3. Write a failing test for the expected behavior (RED)
4. Implement the minimum code to pass the test (GREEN)
5. **Refactor while keeping tests green** — simplify, rename, extract. Run tests after each change.
6. Run the full test suite to check for regressions
7. Run the build to verify compilation
8. Commit with a descriptive message
9. Mark the task complete and stop

If the change touches browser-related code, also invoke `browser-testing-with-devtools` to verify with Chrome DevTools MCP before committing.

**Next step after `/build`:** run `/audit quick` to check quality, then `/ship` for release.

---

## Bug-fix mode: `/build prove` (Prove-It pattern)

For bug fixes — prove the bug exists before fixing it:

1. Write a test that reproduces the bug (must FAIL)
2. Confirm the test fails — run it and capture the failure output
3. Implement the fix
4. Confirm the test passes
5. Run the full test suite for regressions
6. Commit with a descriptive message referencing the bug

If the bug is browser-related, also invoke `browser-testing-with-devtools` to verify the fix visually.

---

## Autonomous: the whole plan (`/build auto`)

Use this once a spec exists and you want to collapse plan + build into one run. It removes the manual stepping between tasks — **not** the verification. Every task still earns a passing test and its own commit.

1. **Require a spec.** Look only for a spec at a known path: `SPEC.md` at the repo root, `docs/SPEC.md`, or a file under `spec/`. A README or arbitrary doc does **not** count. If none exists, stop and tell the user to run `/spec` first — do not invent requirements.
2. **Establish a clean baseline.** Run `git status --porcelain`. If there are uncommitted changes outside the expected planning artifacts (`SPEC.md`, `docs/SPEC.md`, `spec/*`, `tasks/plan.md`, `tasks/todo.md`), stop and ask the user to commit, stash, or confirm how to handle them. Autonomous per-task commits must not absorb unrelated local work, or the clean-rollback guarantee breaks.
3. **Plan if needed.** If there is no `tasks/plan.md`, invoke agent-skills:planning-and-task-breakdown to generate one. Also check `docs/plans/` for exising plans from `/pipeline plan`.
4. **Single checkpoint.** Present the full plan and wait for an unambiguous affirmative (e.g. "approve", "go", "yes"). Treat hedged responses ("looks reasonable", "I guess") as **not** approved. This is the only human gate — after approval, run autonomously. If you generated `tasks/plan.md`, commit it as a single preparatory commit now so it doesn't bleed into the first task's commit.
5. **Execute every task in dependency order.** Use each task's declared dependencies; if they aren't explicit, execute in the order the plan lists them. For each task, run the full default loop above (RED → GREEN → refactor → regression → build → commit → mark complete). Stage only the files that task touched plus its task-status update — never `git add -A` blindly — and make one commit per task so any point is a clean rollback.
6. **Stop and ask the user** (do not push through) when:
   - a test can't be made to pass or the build breaks without an obvious fix → follow agent-skills:debugging-and-error-recovery
   - the spec is ambiguous, or a task needs a decision the spec doesn't cover
   - a task is high-risk or irreversible — auth/permission changes, destructive data migrations, payments, deletions, deploys, anything touching secrets, **or anything you can't undo with `git revert`** → follow agent-skills:doubt-driven-development and get explicit sign-off before continuing

   After the user resolves a blocker, they re-invoke `/build auto` — it resumes from the next pending task.
7. **Summarize at the end:** tasks completed, tests added, commits made, and anything skipped, flagged, or left for the user. Recommend next step: `/audit quick` → `/ship`.

If any step fails, follow the agent-skills:debugging-and-error-recovery skill.

**Downstream:** `/audit quick` → `/ship`.
