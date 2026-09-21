# TBH-04 — Add `develop` to 6 workflows + `release.yml` (D6 Dependabot alignment)

**Plan:** `docs/plans/2026-08-30-testing-bench-harden.md`
**Owner decision D6 (2026-08-30):** align `push.branches` with `.github/dependabot.yml` `target-branch: develop`.

## Impacto mapeado (Regla 0)

- **Files modified (7):**
  - `.github/workflows/ci-rust-10.yml` line 5
  - `.github/workflows/ci-web-11.yml` line 5
  - `.github/workflows/gate-docs-21.yml` line 5
  - `.github/workflows/ci-examples-12.yml` line 9
  - `.github/workflows/chaos-45.yml` line 5
  - `.github/workflows/perf-bench-40.yml` line 5
  - `.github/workflows/release.yml` line 8
- **Blast radius:** dependabot PRs currently opened against `develop` (target-branch set in `.github/dependabot.yml`) and they trigger nothing in these 6 workflows + release. After the fix, dependabot PRs run CI exactly as a human PR does.
- **Side-effect of each change:** NONE. Pure 1-line addition to an existing `branches:` array. No new triggers, no removal of triggers, no impact on `pull_request` (PRs always target a specific branch — adding develop to push only affects direct pushes).
- **WHAT BREAKS without this fix:** Dependabot PRs (cargo, npm, pip, github-actions, docker) bypass 6 CI workflows. A bad cargo bump or malicious pip action update would merge to develop without being caught by `cargo fmt`, `cargo clippy`, `cargo audit`, `cargo deny`, examples smoke, docs lint, web build, or release-plz. Decision D6 confirmed with owner 2026-08-30.

### Mapping of `push.branches` filters

| Workflow | Line | Current | Target | Style preservation |
|---|---|---|---|---|
| `ci-rust-10.yml` | 5 | `[ "main" ]` | `[ "main", "develop" ]` | keep quotes + space style |
| `ci-web-11.yml` | 5 | `[main]` | `[main, develop]` | keep unquoted + no space style |
| `gate-docs-21.yml` | 5 | `[main]` | `[main, develop]` | keep unquoted + no space style |
| `ci-examples-12.yml` | 9 | `[ "main" ]` | `[ "main", "develop" ]` | keep quotes + space style |
| `chaos-45.yml` | 5 | `[ "main" ]` | `[ "main", "develop" ]` | keep quotes + space style |
| `perf-bench-40.yml` | 5 | `[ "main" ]` | `[ "main", "develop" ]` | keep quotes + space style |
| `release.yml` | 8 | `[main]` | `[main, develop]` | keep unquoted + no space style |

**NOT touched (already correct or out of scope):**
- `ci-rustdoc.yml` line 18 already has `["main", "develop"]` ✅
- `desktop.yml` line 14 already has `[main, develop]` ✅
- `pull_request.branches` in any of the 7 files — adding develop to PR would not make sense (dependabot targets `develop` but PRs already target whatever branch the PR is filed against; the `branches` filter on PR is just an extra guard, not the trigger).
- `workflow_dispatch` and `schedule` triggers in any of the 7 files — branch-independent.
- `release-npm-61.yml`, `release-npm-node.yml`, `release-wheels-60.yml`, `sec-codeql-30.yml` — these are explicitly out of scope per the task brief.

## Steps

### Step 1 — Apply edits (one line per file)

Each edit is a literal single-line modification: append `, develop` (or `, "develop"` to match existing style) inside the existing `branches:` array. No YAML reformat. No renames. No new comments.

### Step 2 — Verify

```bash
python -c "import yaml; [yaml.safe_load(open(f'.github/workflows/{f}')) for f in ['ci-rust-10.yml', 'ci-web-11.yml', 'gate-docs-21.yml', 'ci-examples-12.yml', 'chaos-45.yml', 'perf-bench-40.yml', 'release.yml']]; print('all 7 YAMLs parse OK')"

# Must be empty (all 7 files now contain `develop`):
grep -L "develop" .github/workflows/{ci-rust-10,ci-web-11,gate-docs-21,ci-examples-12,chaos-45,perf-bench-40,release}.yml
```

### Step 3 — Confirm out-of-scope files unchanged

```bash
# ci-rustdoc.yml line 18 must still show ['main', 'develop']
# desktop.yml line 14 must still show ['main', 'develop']
# release-npm-61.yml, release-npm-node.yml, release-wheels-60.yml, sec-codeql-30.yml
#   must still have ONLY `main` (out of scope per task brief)
grep -n "branches:" .github/workflows/ci-rustdoc.yml .github/workflows/desktop.yml .github/workflows/release-npm-61.yml .github/workflows/release-npm-node.yml .github/workflows/release-wheels-60.yml .github/workflows/sec-codeql-30.yml
```

### Step 4 — Commit

```bash
git add .github/workflows/{ci-rust-10,ci-web-11,gate-docs-21,ci-examples-12,chaos-45,perf-bench-40,release}.yml .opencode/skills/campaign-executor/tasks/TBH-04.md
git commit -m "ci(TBH-04): add develop branch to 6 workflows + release.yml (D6 Dependabot alignment)"
```

## Verificación contractual

- [ ] Each of the 7 workflows has `push.branches` containing both `main` and `develop`
- [ ] No `pull_request.branches` modified (PR branches are unaffected)
- [ ] No `workflow_dispatch` / `schedule` modified
- [ ] `ci-rustdoc.yml` line 18 unchanged
- [ ] `desktop.yml` line 14 unchanged
- [ ] `release-npm-61.yml`, `release-npm-node.yml`, `release-wheels-60.yml`, `sec-codeql-30.yml` unchanged
- [ ] YAML parses for all 7
- [ ] `grep -L develop` on the 7 files returns empty
- [ ] Conventional commit `ci(TBH-04):`
- [ ] One line modified per file (no reformat)

## Out of scope

- No tocar `release-npm-*`, `release-wheels-*`, `sec-codeql-30.yml` (no Dependabot target-branch gap or explicitly excluded by task brief)
- No crear ADR (D6 ya documentada en el plan)
- No reformatear YAML, mantener estilo exacto de cada archivo
- No agregar comentarios nuevos
- No tocar `pull_request.branches` en ninguno de los 7