# Constraints — VantaDB Quality Bar

> **Este archivo es la fuente canonica del quality bar de VantaDB.**
> Leido por agents en cada sesion (via `constraint-driven-development` skill).
> No debilitar para que un cambio pase — tightening es silencioso, loosening es ruidoso.
> Ver: `.opencode/skills/constraint-driven-development/SKILL.md` y `.opencode/references/floor-guard.md`.

Last reviewed: 2026-09-01 by @ness-e
Next review: 2026-12-01

---

## Floor (siempre enforced, no setup required)

Estos 5 checks **nunca** se violan, sin importar la configuracion:

- No new suppression comments: `#[allow(...)]` en Rust, `@ts-ignore`, `eslint-disable`, `# noqa`, `# type: ignore`
- No unimplemented stubs: `unimplemented!()`, `todo!()`, `panic!("not implemented")`, `throw new Error("Not implemented")`, empty `catch {}`
- No skipped or deleted tests without a reason in the commit message
- No secrets in source (API keys, tokens, passwords) — gitleaks must pass
- This file does not get weakened to make a change pass

**Checked by:** `dev-tools/floor-guard.ps1` (diff-scoped, exit 0/1/2) — ver `.opencode/references/floor-guard.md`
**Runs at:** every edit (diff), every commit, CI

---

## Enforced with numbers

| Dimension | Rule | Checked by | Runs at |
|-----------|------|-----------|---------|
| Types (Rust) | Zero `cargo check` errors, zero `cargo clippy -- -D warnings` | `cargo check -p vantadb && cargo clippy -- -D warnings` | every edit |
| Format | Zero `cargo fmt --check` diff | `cargo fmt --check` | every edit |
| Lint (JS/TS) | Zero ESLint errors from our config | `eslint .` / `biome check` (if configured) | every edit |
| Secrets | No secrets in source | `gitleaks detect --redact --no-banner` | every edit |
| Coverage (changed lines) | Changed lines >= 80% covered | `cargo test --coverage` + `git diff` (nextest + llvm-cov) | task end, CI |
| Coverage (project) | Project coverage must not fall (ratchet: today 68.2%) | `cargo llvm-cov --workspace` | CI |
| Security: code | No high findings | `cargo audit` | CI |
| Security: deps | Nothing at high or above | `cargo deny check advisories && cargo audit` | CI |
| Security: licenses | Only MIT/Apache-2.0 (deny.toml) | `cargo deny check licenses` | CI |
| Performance | p99 not regressed vs baseline `benches/canonical_p99` | `cargo bench --bench canonical_p99` | CI (release profile) |
| Binary size | `cargo bloat --crates` justified for new deps | `cargo bloat --crates` | PR review |

Every row names the command that produces the verdict. A dimension with a number and no command is an aspiration, not a constraint.

---

## Measured, not yet enforced

| Metric | Today | Direction |
|--------|-------|-----------|
| Project coverage | 68.2% | must not fall |
| Binary size (release) | ~12 MB (vantadb) | must not grow >5% per release |
| p50 latency (canonical_p99) | TBD (run `cargo bench`) | must not regress |
| p99 latency (canonical_p99) | TBD (run `cargo bench`) | must not regress |
| Bundle size (web/) | TBD | must not grow |
| Accessibility (web/) | TBD (axe-core) | zero critical or serious |

> **TODO:** Run `cargo bench --bench canonical_p99 -- --save-baseline main` to populate p50/p99. Run `cargo llvm-cov` for exact coverage.

---

## Exceptions

| ID | Rule | Path | Reason | Owner | Expires |
|----|------|------|--------|-------|---------|
| W1 | `clippy::too_many_arguments` | `src/engine.rs:search*` | Search API has 7 params, refactor tracked in DRV-XXX | @ness-e | 2026-12-01 |
| W2 | `unsafe` without Miri | `vantadb-wasm/src/opfs.rs` | OPFS shim, Miri not applicable to WASM | @ness-e | 2027-03-01 |

> Exceptions requieren owner + expiry. Review en cada `Last reviewed` date. Expiry >1y no permitido.

---

## Ratchets

Cuando no tienes un numero target, registra donde estas y no empeores:

- Project coverage: 68.2% today -> must not fall below 68.2% (tolerance 0.5% for drift)
- Binary size: measure today, hold the line
- p99: first bench is the ratchet baseline

Ver `constraint-driven-development` Step 7: Ratchets.

---

## Where it runs

| Phase | Command | What runs | Budget |
|-------|---------|-----------|--------|
| BUILD | `cargo check` + `cargo fmt` + `gitleaks` | Types, format, secrets, floor | under 5s, changed file only |
| VERIFY | `cargo test` (nextest) | Related tests, coverage on changed lines | under 90s |
| REVIEW | `cargo audit` + `cargo deny` + `floor-guard.ps1` | Everything, plus guards | minutes |
| SHIP | `cargo bench` + `cargo bloat` | Direction checks, no regressions | CI |

Two rules:
1. **Scope to the diff.** Check lines this change touched, not whole repo.
2. **Cost decides placement.** >few seconds -> out of edit loop.

---

## Guard the bar itself

Watch for these 5 moves in the diff (tightening is silent, loosening is loud):

1. **The threshold moved.** Budget lowered, severity dropped, check removed from fast stage. Compare `CONSTRAINTS.md` against branch point.
2. **A test got easier.** `.skip` added, test file deleted, assertions pulled out.
3. **A checker got silenced.** New `#[allow(...)]`, `@ts-ignore`, `eslint-disable`, `istanbul ignore`.
4. **Work is unfinished.** Stub that throws, empty `catch` turning failure into silence, `TODO` where impl should be.
5. **An exception appeared.** New row in Exceptions table nobody discussed.

None needs tooling beyond `git diff`. See `.opencode/references/floor-guard.md` for reference implementation.

---

## Escalation Path

1. **Written only.** This file exists and agents read it. Costs nothing, catches honest mistakes.
2. **Scripted.** `dev-tools/floor-guard.ps1` + `dev-tools/verify.ps1` wired into pre-commit/pre-push and CI. Deterministic.
3. **Tool-backed.** Dedicated runner with diff scoping, budgets, ratchets (future: when config >30 lines shell).

Most projects stop at 2. VantaDB is at 2 (verify.ps1 + floor-guard.ps1).

---

## Sane Defaults (reference)

| Constraint | Default | Why |
|------------|---------|-----|
| Coverage changed lines | >= 80% | High enough to force test, low enough for config line |
| Project coverage | today's value, must not fall | No argument needed |
| Mutation score | >= 60% to start | Typical for suite never mutated before |
| Dependency vulns | nothing at high or above | Below is mostly noise |
| p99 | baseline from `benches/canonical_p99` | Must not regress |
| Exception lifetime | 90 days | Long enough to plan, short enough to remember |
| Ratchet tolerance | 0.5% | Absorbs drift when unrelated file moves number |

---

## Verification

Skill applied correctly when:
- [x] `CONSTRAINTS.md` exists, every number has stated reason
- [x] Floor is enforced and passes on current codebase without changes
- [x] Every dimension has tool installed and command that runs today
- [x] Each constraint says where it runs, fast stage under few seconds
- [x] At least one external constraint present (`cargo audit`, `gitleaks`)
- [x] Measured-only metrics record today's value and direction
- [x] Exceptions have owner and expiry date
- [x] `AGENTS.md` points at this file (add: "Read CONSTRAINTS.md before writing code")
- [ ] Trial run on current branch produces no failures user disagrees with (run `dev-tools/floor-guard.ps1`)

---

## See Also

- `constraint-driven-development` skill — how to maintain this file
- `.opencode/references/floor-guard.md` — floor implementation
- `dev-tools/verify.ps1` — full verification pipeline
- `dev-tools/floor-guard.ps1` — floor guard script (to be created)
- `.opencode/references/definition-of-done.md` — standing quality bar (complementary)
