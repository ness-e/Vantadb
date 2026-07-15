---
name: vantadb-audit
description: >
  Orchestrador único de auditoría VantaDB. Unifica 3 skills de review,
  5 skills de audit, y todas las herramientas CLI en un pipeline de 9 fases
  automáticas. Ejecutá con /audit. No requiere args — detecta modo solo.
---

# VantaDB Audit Orchestrator

Pipeline de auditoría completo. Corre como `/audit` desde el slash command.

## Fases

### Phase 0: Pre-check
```
git diff --name-only HEAD     # ¿hay cambios sin commit?
git status --short            # estado general del working tree
```
Si no hay cambios → `/audit full` sugerido.
Si hay cambios staged → `/audit certify` sugerido.

### Phase 1: CLI Mechanical Checks
Ejecuta `dev-tools/audit-all.ps1 -Mode <mode>`. Según el modo:

| Check | quick | certify | review | full |
|-------|-------|---------|--------|------|
| `cargo fmt --check` | ✅ | ✅ | — | ✅ |
| `cargo clippy --workspace -- -D warnings` | ✅ core | ✅ | — | ✅ |
| `cargo nextest --profile audit --workspace --build-jobs 2` | ✅ core | ✅ | — | ✅ |
| `cargo deny check` | ✅ | ✅ | — | ✅ |
| `cargo audit` | — | ✅ | — | ✅ |
| `cargo machete` | — | — | ✅ | ✅ |
| `cargo bloat --crates` | — | — | — | ✅ |

Ruta real: `pwsh -NoProfile -File dev-tools/audit-all.ps1 -Mode quick`

Si falla → abortar con error. No seguir a Phase 2.

### Phase 2: Security Audit
```
skill security-and-hardening    # threat modeling, input validation
cargo audit                    # advisory check (ya corre en Phase 1 si es full)
```
La skill puede vetar.

### Phase 3: Performance Audit
```
skill performance-optimization  # profile-guided review
cargo bloat --crates           # binary size (ya corre en Phase 1 si es full)
```

### Phase 4: Code Review
```
skill code-review-and-quality   # 5-axis code review
skill ponytail-review           # over-engineering check
cargo clippy -- -W clippy::pedantic
```
Cada skill puede vetar.

### Phase 5: Root-Cause Debugging
```
skill systematic-debugging      # solo si hay issues conocidos
```
Si las fases 1-4 pasaron sin issues, skipear esta fase.
Si hay failures, la skill analiza logs y propone fixes.

### Phase 6: Deep Module Review
```
skill review-deep               # per-module loop con web research
```
Iterativa. Solo en modo `full` o `review`.

### Phase 7: Full ISO 25010 Review
```
skill vantadb-full-review       # comprehensive one-shot report
```
Produce reporte estructurado con scores. Solo en modo `full`.

### Phase 8: Certification Gate
```
pwsh -NoProfile -File dev-tools/nocturnal_suite.ps1   # Heavy cert
skill vantadb-certify                                   # Pre-push gate
```
Solo en modo `certify` o `full`.

## Modos vs Fases Matrix

| Fase | quick | certify | review | full |
|------|-------|---------|--------|------|
| 0. Pre-check | — | ✅ | ✅ | ✅ |
| 1. CLI | ✅ | ✅ | — | ✅ |
| 2. Security | — | ✅ | — | ✅ |
| 3. Performance | — | ✅ | — | ✅ |
| 4. Code Review | — | ✅ | ✅ | ✅ |
| 5. Root Cause | — | — | ✅ | ✅ |
| 6. Deep Module | — | — | ✅ | ✅ |
| 7. Full ISO | — | — | — | ✅ |
| 8. Certify | — | ✅ | — | ✅ |

## Report

Al finalizar, escribir reporte estructurado en `docs/audit-reports/audit-<mode>-<timestamp>.md`:

```markdown
# Audit Report: <mode>
**Date:** <timestamp>
**Mode:** <mode>

## Per-Phase Results

| Phase | Status | Details |
|-------|--------|---------|
| 1. CLI | ✅ | fmt ✅ clippy ✅ test ✅ deny ✅ audit ✅ |
| 2. Security | ✅ | 0 issues found |
| 3. Performance | ❌ | bloated crate: vantadb-server (2.3MB → target <1.5MB) |

## Code Review Findings
- `src/engine.rs:142`: unsafe block without SAFETY comment
- `vantadb-server/src/routes.rs:88`: unwrap() on user input

## Recommendations
1. (priority: high) Remove unsafe in engine.rs
2. (priority: medium) Shrink vantadb-server binary
```

## Rules

- Stop on Phase 1 failure (no point reviewing broken code)
- Each review skill can VETO — if veto, record objection and continue (don't abort full pipeline, but flag it)
- Report covers ALL phases regardless of pass/fail
- Timestamp all reports for audit trail
