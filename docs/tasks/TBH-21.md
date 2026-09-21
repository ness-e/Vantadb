# TASK-TBH-21: Document CoverageThreshold=60 review cadence in CI_POLICY.md

## Metadata
- **Plan file:** docs/plans/2026-08-30-testing-bench-harden.md
- **Created:** 2026-08-30T18:00
- **last-synced:** 2026-08-30T18:00
- **Estado:** ✅ COMPLETED
- **Tipo (campaign_detect_task_type):** docs / config-note (single-file docs edit, no code change)
- **Esfuerzo:** 🟢 Bajo
- **Prioridad:** BAJA (Phase 3, hardening)

## Impacto mapeado (Regla 0)

### Archivos leídos completos
| Archivo | Líneas | Notas |
|---------|--------|-------|
| `docs/operations/CI_POLICY.md` | 392 | Single source of truth para gates CI; contiene §"Coverage Gate" (P2-06) línea 272-324 |
| `dev-tools/verify.ps1` | 95 | Línea 49: `$CoverageThreshold = 60`; línea 48: comment `P2-06: prudent initial llvm-cov line-coverage floor. Raise after first runs (see CI_POLICY.md).` |
| `dev-tools/gate-common.ps1` | 27 | NO contiene threshold (líneas 13-19: `Get-CoreFeatures`; 21-27: `Get-AdaptiveJobs`). No tocar. |

### Referencias hacia dentro (outbound references de los archivos tocados)
- `verify.ps1:48` ya apunta a `CI_POLICY.md` ("Raise after first runs (see CI_POLICY.md)"). El nuevo comentario "Last reviewed: YYYY-MM-DD" + la nueva sección CI_POLICY crean el lazo de trazabilidad explícito.

### Referencias hacia afuera (inbound — quién lee estos archivos)
- `dev-tools/verify.ps1` → ejecutada por hooks `pre-push` (`.githooks/pre-push` per AGENTS.md Regla 1) y por el step `coverage` del Fast Gate en `ci-rust-10.yml`.
- `docs/operations/CI_POLICY.md` → referenciado desde `.opencode/AGENTS.md` (CI Architecture section), desde `dev-tools/verify.ps1:48`, desde CI workflows, desde docs/architecture/adr/ADR-015-coverage-policy.md (P2-06 source).

### Veredicto de impacto
**Mínimo.** Cambios son:
1. Anexar sub-sección "Coverage Threshold Review Cadence" a `CI_POLICY.md` después del bloque §"Coverage Gate — Minimum Mechanized Coverage (P2-06)" (línea 272-324).
2. NO se modifica el threshold (`60`) en `verify.ps1` — el contrato dice "NO TOCAR el threshold actual".
3. NO se toca `gate-common.ps1` (no contiene threshold).

Ningún caller downstream cambia. La nota de fecha agrega trazabilidad sin alterar comportamiento.

### SDP (Skill Discovery Protocol)
- `campaign-executor` (base, auto-cargada via MCP) — orquestación de la tarea.
- `documentation-and-adrs` — cómo documentar decisiones de policy con el "why".
- `ci-cd-and-automation` — confirmar el contexto del coverage gate.
- `campaign-load-skills base + 2 lifecycle mappings` ya cubre el caso docs/policy.

### Gates
- **P (pre-flight):** no requerido (cambio de docs, sin API pública, sin blast radius >10).
- **D (discovery):** disparado — leído CI_POLICY.md + verify.ps1 + gate-common.ps1; veredicto arriba.
- **V (verify):** el contrato mecánico es `Select-String` matchea el patrón en CI_POLICY.md y `git grep 60` sigue matcheando en verify.ps1.
- **C (commit):** conventional commit `docs(TBH-21):`.

## Contrato (verificable mecánicamente)

```
1. CI_POLICY.md contiene sección "Coverage Threshold Review Cadence" con:
   - "Coverage threshold: 60% (floor)"
   - "Review cadence: quarterly"
   - "Last reviewed: 2026-08-30"
2. CI_POLICY.md contiene "Next review due: <fecha>" (90 días después).
3. git grep "60" en dev-tools/verify.ps1 sigue matcheando (threshold NO removido).
4. dev-tools/gate-common.ps1 NO modificado (no contiene threshold).
```

## Herramientas
- Edit / Read (filesystem)
- Select-String (PowerShell grep)
- git (commit)

## Steps

### Step 1: Crear task file + marcar plan ⏳ IN PROGRESS ✅
- **Archivos:** `.opencode/skills/campaign-executor/tasks/TBH-21.md`, `docs/plans/2026-08-30-testing-bench-harden.md`
- **Acción:** Editar checkbox TBH-21 a `[⏳]` en el plan file + escribir este task file.
- **Verify:** `grep TBH-21 docs/plans/...md` → muestra `[⏳]`.
- **Estado:** ✅ COMPLETED

### Step 2: Insertar sub-sección "Coverage Threshold Review Cadence" en CI_POLICY.md
- **Archivos:** `docs/operations/CI_POLICY.md`
- **Acción:** Edit tool para insertar nueva sub-sección entre `**Merge / union:**` (línea 304) y `### 3. Web CI (`ci-web-11.yml`)` (línea 326). Contenido: threshold + cadence + last reviewed + next due.
- **Verify:** `Select-String -Path CI_POLICY.md -Pattern "Coverage Threshold Review Cadence"` → 1 match.
- **Estado:** ⬜ PENDING

### Step 3: Verificar contrato + commit + actualizar plan ✅
- **Archivos:** CI_POLICY.md + task file + plan file
- **Acción:** verificar ambos greps → `git add` → commit `docs(TBH-21):` → actualizar plan a ✅ + RECITATION.
- **Verify:** `git log -1 --format=%s` muestra `docs(TBH-21):`.
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna. TBH-21 es independiente de las otras tareas del plan.

## Notas

- **Ponytail reflex:** 1 sub-sección de docs + 0 cambios de código. NO abstracciones, NO scripts nuevos, NO refactors.
- El plan original sugería "Update `verify.ps1` `coverageThreshold=60` to source from policy" — **NO aplico esto** porque el contrato del orquestador dice "NO TOCAR el threshold actual (60) — es lo que el equipo decidió mantener" + "1 entrada de docs, 1 comment opcional". Mantener el literal `60` evita un PR mecánico que requiere sync de policy.
- El "comment opcional" (`// reviewed YYYY-MM-DD per CI_POLICY.md`) no es necesario porque la nota vive en CI_POLICY.md (single source of truth); duplicar la fecha en verify.ps1 sería derivar estado.

## Context Save Point
- **Fecha:** 2026-08-30
- **Branch:** develop
- **CI pendiente:** no (cambio solo de docs)
- **Decisiones:**
  - NO modificar verify.ps1 (mantener threshold literal 60) — el contrato lo prohíbe explícitamente.
  - NO modificar gate-common.ps1 (no contiene threshold — verificado).
  - Insertar sub-sección **después** del bloque "Merge / union" (línea 304) y **antes** de "### 3. Web CI" (línea 326) — ubicación lógica: cierre de la cobertura mecánica, dentro de la §"Coverage Gate".
- **Problemas conocidos:** ninguno.
- **Próxima tarea:** handoff al orquestador.