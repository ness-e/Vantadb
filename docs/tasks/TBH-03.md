# TBH-03 — Remove `if: schedule` from `ci-gate.yml` (universal gate per D3)

**Plan:** `docs/plans/2026-08-30-testing-bench-harden.md`
**Owner decision D3 (2026-08-30):** universal gate — `if: inputs.event_name == 'schedule'` is a bug.

## Impacto mapeado (Regla 0)

- **File modified:** `.github/workflows/ci-gate.yml` line 24
- **Blast radius (3 invocadores):**
  - `fuzz-40.yml` (line 30-35): calls gate with `event_name: ${{ github.event_name }}` — assumes gate runs for ALL triggers.
  - `heavy-certification-50.yml` (line 25-30): same.
  - `heavy-bench-nightly-51.yml` (line 31-36): same.
- **Side-effect of the `if` line:** NONE. The line has no `|| true`, no `continue-on-error`, no `env:` consolidation. Pure 1-line removal.
- **WHAT BREAKS without this fix:** PR runs, `workflow_dispatch`, and `push` runs all pass the gate trivially (GH Actions evaluates `if: false` → step skipped → success). Heavy jobs (`build`, `fuzz`, `stress-protocol`, etc.) run for hours even when main is red. The documented contract "si main está rojo, los workflows pesados deben fallar" is silently violated.

## Steps

### Step 1 — Remove the line
**Action:** Remove `if: ${{ inputs.event_name == 'schedule' }}` from line 24 of `.github/workflows/ci-gate.yml`.

Before:
```yaml
      - name: Check main CI status
        if: ${{ inputs.event_name == 'schedule' }}
        env:
```

After:
```yaml
      - name: Check main CI status
        env:
```

**Ponytail reflex:** literal 1-line removal. NO YAML reformat. NO new comments.

### Step 2 — Verify
```bash
python -c "import yaml; yaml.safe_load(open('.github/workflows/ci-gate.yml')); print('YAML OK')"
# Expected: "YAML OK"
```

### Step 3 — Confirm `schedule` no longer conditions the gate
```bash
# Should return ONLY line 12 (input declaration), nothing else.
grep -n "schedule" .github/workflows/ci-gate.yml
```

### Step 4 — Commit
```bash
git add .github/workflows/ci-gate.yml .opencode/skills/campaign-executor/tasks/TBH-03.md
git commit -m "ci(TBH-03): remove schedule-only condition from ci-gate (universal gate per D3)"
```

## Verificación contractual

- [ ] Line 24 removed, no `if:` on the gate step
- [ ] YAML parses
- [ ] `grep schedule` returns at most the input declaration (line 12)
- [ ] 3 invocadores unchanged
- [ ] Conventional commit `ci(TBH-03):`

## Out of scope

- No tocar invocadores
- No crear ADR (el plan ya documenta la decisión D3)
- No reformatear YAML
- No agregar comentarios nuevos