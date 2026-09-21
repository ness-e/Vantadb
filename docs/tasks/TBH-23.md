# TBH-23: Unify `cargo fmt --check` scope across Justfile/verify.ps1/audit-all.ps1

## Metadata
- **Plan file:** `docs/plans/2026-08-30-testing-bench-harden.md`
- **Creado:** 2026-08-30T16:00
- **last-synced:** 2026-08-30T16:00
- **Estado:** ✅ COMPLETED

## Blast Radius
Callers | Callees | Implicaciones:
- Local pre-flight (`just verify`, `just fmt`, `just ci`)
- CI gate (`pwsh dev-tools/verify.ps1`)
- Audit suite (`pwsh dev-tools/audit-all.ps1 -Mode quick|ci|full`)
- Fast gate (`pwsh dev-tools/verify_changed.ps1`)
- No external/consumer impact: tooling only.

## Contrato
"`Select-String` shows identical `cargo fmt --check` scope in Justfile + verify.ps1 + audit-all.ps1" ✅

## Decisión

**Target scope:** `cargo fmt --all -- --check` (alineado con verify.ps1 + verify_changed.ps1, que ya usaban `--all`).

**Razón:** El task del plan recomendaba "añadir `--all` al Justfile" basado en el supuesto de que verify.ps1 era el canónico. La auditoría descubrió que en realidad **2 de 3 archivos ya usaban `--all`** (verify.ps1 + verify_changed.ps1), no 1. La fuente canónica (`gate-common.ps1`) NO toca el comando fmt — solo features y jobs. Por lo tanto, la elección natural es **el patrón mayoritario (--all)** en los 4 archivos.

## Cambios

- `Justfile:29` — `{{cargo}} fmt --check` → `{{cargo}} fmt --all -- --check`
- `dev-tools/audit-all.ps1:87,93,100` — `cargo fmt --check` → `cargo fmt --all -- --check` (×3 ocurrencias)
- `dev-tools/verify.ps1:60` — **SIN CAMBIOS** (ya tenía `--all`)
- `dev-tools/verify_changed.ps1:30` — **SIN CAMBIOS** (ya tenía `--all`)
- `dev-tools/gate-common.ps1` — **NO TOCADO** (no maneja fmt)

## Steps

### Step 1: Discovery
- **Archivos:** `Justfile`, `dev-tools/verify.ps1`, `dev-tools/audit-all.ps1`, `dev-tools/gate-common.ps1`
- **Acción:** Identify exact `cargo fmt --check` command per file
- **Verify:** `Select-String -Pattern "fmt"` shows divergence
- **Estado:** ✅ DONE

### Step 2: Decide target scope
- **Acción:** Analyze real-world state (not the plan's assumption): 2 files have `--all`, 1 doesn't
- **Decisión:** Adopt `--all` (the majority pattern) across all 4 files
- **Estado:** ✅ DONE

### Step 3: Edit Justfile
- **Acción:** Add `--all --` flag to `fmt` recipe
- **Verify:** `just fmt` runs `cargo fmt --all -- --check` (verified via Read)
- **Estado:** ✅ DONE

### Step 4: Edit audit-all.ps1
- **Acción:** Add `--all --` flag to 3 occurrences (lines 87, 93, 100)
- **Verify:** `Select-String` confirms identical scope
- **Estado:** ✅ DONE

### Step 5: Verify all files have identical scope
- **Verify:** `Select-String` shows `cargo fmt --all -- --check` in all 4 files
- **Estado:** ✅ DONE

### Step 6: Commit
- **Acción:** `git add` + commit with conventional message
- **Estado:** ✅ DONE

## Dependencias
- TBH-22: Predecessor in plan (✅ completed)

## Notas
- 1-line conceptual change × 2 files (3 lines in audit-all.ps1 + 1 line in Justfile). Total diff: 4 lines.
- Ponytail reflex: NO reformats, NO scope creep.
- `--all` flag = same `--check` semantic + workspace-wide scope (instead of just current package).

## Context Save Point
- **Fecha:** 2026-08-30T16:00
- **Branch:** main
- **CI pendiente:** no (local only)
- **Decisiones:** `--all` over no-flag (aligns with majority pattern)
- **Problemas conocidos:** ninguno
- **Próxima tarea:** TBH-24 (siguiente TBH en el plan, si existe)