# /audit — Full Automated Run

Ejecutá el pipeline completo de auditoría sobre VantaDB.

## Instructions

1. Load `skill vantadb-audit`
2. Follow the phases for the requested mode
3. For each phase:
   - Print "Phase N: [name]" in bold
   - Run the real commands (no dry-run)
   - Capture output
   - Print ✅ or ❌
4. At the end, write full report to `docs/audit-reports/audit-<mode>-<timestamp>.md`

## Modes

### quick
```
Phase 0: skip
Phase 1: pwsh -NoProfile -File dev-tools/audit-all.ps1 -Mode quick
Phases 2-8: skip
```

### certify
```
Phase 0: git diff check
Phase 1: pwsh -NoProfile -File dev-tools/audit-all.ps1 -Mode ci
Phase 2: skill security-and-hardening
Phase 3: skill performance-optimization
Phase 4: skill code-review-and-quality + skill ponytail-review
Phase 5: skip
Phase 6: skip
Phase 7: skip
Phase 8: skill vantadb-certify + pwsh -NoProfile -File dev-tools/nocturnal_suite.ps1
```

### review
```
Phase 0: git diff check
Phase 1: skip (assumes code compiles)
Phase 2: skip
Phase 3: skip
Phase 4: skill code-review-and-quality + skill ponytail-review
Phase 5: skill systematic-debugging
Phase 6: skill review-deep
Phase 7: skip
Phase 8: skip
```

### full
```
Phase 0: git diff check
Phase 1: pwsh -NoProfile -File dev-tools/audit-all.ps1 -Mode full
Phase 2: skill security-and-hardening
Phase 3: skill performance-optimization
Phase 4: skill code-review-and-quality + skill ponytail-review
Phase 5: skill systematic-debugging (si hay issues en fases anteriores)
Phase 6: skill review-deep
Phase 7: skill vantadb-full-review
Phase 8: skill vantadb-certify + pwsh -NoProfile -File dev-tools/nocturnal_suite.ps1
```

## Report Template

```markdown
# Audit Report: <mode>
**Date:** <timestamp>
**Duration:** <elapsed>

## Summary
- Phases passed: X/8
- Blocking issues: N
- Recommendations: N

## Phase Results

| Phase | Status | Details |
|-------|--------|---------|
| 0. Pre-check | ✅/❌ | <detail> |
| 1. CLI | ✅/❌ | <detail> |
| ... | ... | ... |

## Findings

### Finding 1: <title>
- **File:** path:line
- **Severity:** high/medium/low
- **Description:** ...
- **Recommendation:** ...

### Finding 2: <title>
...

## Scoreboard

| Category | Score (0-10) | Notes |
|----------|-------------|-------|
| Code Quality | 8 | Minor clippy warnings |
| Security | 9 | No vulnerabilities |
| Performance | 7 | Binary could be smaller |
| Architecture | 8 | Clean module boundaries |
| Tests | 9 | 434/434 passing |
| Docs | 6 | Missing API docs for new modules |
```

## After Report

If `/audit full` or `/audit review`: add findings to `docs/Backlog.md` as new tasks.

If `/audit certify`: if all phases pass, print "✅ CERTIFY PASSED — safe to push". If any fail, "❌ CERTIFY FAILED — fix errors above before pushing".
