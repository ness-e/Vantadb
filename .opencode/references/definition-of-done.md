# Definition of Done

A standing, project-wide bar that every change must clear before it counts as done. Unlike acceptance criteria, which vary per task and answer "did we build the right thing?", the Definition of Done is the same every time and answers "is this finished to our standard?". Use it as the final gate in `planning-and-task-breakdown`, `incremental-implementation`, and `shipping-and-launch`.

## Definition of Done vs. Acceptance Criteria

| | Acceptance Criteria | Definition of Done |
|---|---|---|
| Scope | Specific to one task or spec | Applies to every increment |
| Changes | Different for each item | Fixed and reused |
| Answers | "Did we build *this thing*?" | "Is it *ready*?" |
| Owner | Defined when planning the task | Defined once for the project |
| Example | "User can reset password via email link" | "Tests pass, no regressions, docs updated" |

The two are complementary. A task is done only when **its** acceptance criteria are met **and** the standing Definition of Done is satisfied. Skipping either leaves work that looks finished but is not.

## The Standing Checklist

Apply this to every change before declaring it done.

### Correctness
- [ ] All acceptance criteria for the task are met
- [ ] Code runs and behaves as intended, verified at runtime, not just compiled or typechecked
- [ ] New behavior is covered by tests that fail without the change and pass with it
- [ ] Existing tests still pass; no regressions introduced
- [ ] Edge cases and error paths are handled, not just the happy path

### Quality
- [ ] Code reveals intent through naming and structure; no comments needed to explain *what* it does
- [ ] No duplicated business logic
- [ ] No dead code, debug output, or commented-out blocks left behind
- [ ] Changes are scoped to the task; no unrelated refactors snuck in
- [ ] Linting and formatting pass

The depth behind these items lives in `code-review-and-quality` (the five-axis review) and `code-simplification` (reducing complexity without changing behavior).

### Integration
- [ ] Change works with the rest of the system, not just in isolation
- [ ] Database migrations, config changes, and feature flags are accounted for
- [ ] Backward compatibility considered for any public interface or API change

### Documentation
- [ ] Public interfaces, APIs, and user-facing behavior are documented
- [ ] Architectural decisions worth preserving are recorded (see `documentation-and-adrs`)
- [ ] Documentation describes the current state in timeless language, not the change history

### Ship-readiness
- [ ] Security implications reviewed for any untrusted input, auth, or data handling (see `security-and-hardening`)
- [ ] Observability in place for new critical paths (logs, metrics, traces) (see `observability-and-instrumentation`)
- [ ] Rollback path exists for anything risky (see `shipping-and-launch`)
- [ ] rollback plan declared explicitly in the task file for features touching production or risky paths (concrete `git revert` steps or a flag-off), with feature flags in place when gradual deployment is required
- [ ] The human has reviewed and approved before merge or deploy

## How to Apply

- **Per task**: confirm the Correctness and Quality sections before checking the task off.
- **Per feature**: confirm Integration and Documentation before considering the feature complete.
- **Per release**: the full checklist is the floor; `shipping-and-launch` adds the deploy-specific gates on top.
- **Per release — post-release**: after releasing, verify in production that the release broke nothing before closing the iteration.
- **Per release — monitoring**: monitor logs, metrics, and error rates as part of that post-release verification.

Tailor the list to the project once, then reuse it unchanged. A Definition of Done that is renegotiated every sprint is not a Definition of Done.

## Red Flags

- "It's done, I just haven't run it yet": unverified work is not done.
- "Tests pass" used as a synonym for done while docs, regressions, or runtime verification are skipped.
- A different bar applied depending on deadline pressure.
- Acceptance criteria treated as the whole bar, with no standing quality floor.
- "Done" declared before human review on changes that need it.

---

# VantaDB — Definition of Ready (DoR)

Applicable to every item admitted to the active backlog (`docs/Backlog.md`). An item is ready to be picked up only when all of the following hold:

- [ ] Unique ID assigned
- [ ] Priority defined (🔴🟠🟡🟢🔵⬜)
- [ ] Involved files known
- [ ] Effort estimated
- [ ] Verified against real code (not assumed)

# VantaDB — Project-specific DoD commands

The standing checklist above applies to every change. VantaDB additionally requires these concrete commands to pass (equivalent to `dev-tools/verify.ps1`):

- [ ] Code compiles: `cargo check` / `tsc --noEmit`
- [ ] Tests pass: `cargo nextest run` / `pytest`
- [ ] Linters pass: `cargo clippy` / `eslint`
- [ ] Docs updated if applicable
- [ ] Task moved to `docs/progreso/README.md` when completed
- [ ] Changelog updated if user-visible change (`docs/CHANGELOG.md` via git-cliff)
- [ ] Change is shippable: PR opened with green CI, or merged to `main` (Regla 7) — to merge a main-branch change, open a PR and get CI green first; task ends when the work can ship, not when it commits
