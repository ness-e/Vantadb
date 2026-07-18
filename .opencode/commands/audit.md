---
description: "Audit pipeline unificado: CLI checks + skills de review en subagentes paralelos/secuenciales + reporte. Modos: full | quick | certify | review"
---

> **ENTRY POINT — Audit Command**
> El agente DEBE leer este archivo cuando el usuario envía un mensaje que empieza con `/audit`.
> Path resolution: `prompts/X.md` → `.opencode/task-system/prompts/X.md`
> Skills: `skills/X` → `.opencode/skills/X/`
> Instrucciones: cargar skills listados, ejecutar fases según modo. Waves paralelas vía sub-agentes o secuencial según modo.
> Al finalizar: escribir reporte en `docs/audit-reports/`.

# /audit — VantaDB Audit Pipeline

Pipeline de auditoría completo. Unifica CLI checks + skills de review en subagentes + reporte estructurado.

Cargá las skills `progreso`, `ponytail` (full) primero.
LEÉ `prompts/audit-full.md` con Read tool y seguí sus fases EXACTAMENTE — es la instrucción activa para ejecución completa.
El prompt define las 9 fases, waves paralelas, formato de output, y FODA estratégico.

## Router: detectar modo según el argumento

- **Sin argumento o `full`** → pipeline completo: fases 0→1→2→3→4→5→6→7→8 (waves paralelas)
- **`quick`** → solo Phase 1 (CLI checks via `just verify`), ~2min
- **`certify`** → pre-push gate secuencial: fases 0→1→2→3→4→7a→7b→8, hard stop al primer error
- **`review`** → revisión profunda sin CLI: fases 0→4→5→6→7

---

## Fases

| Fase | Ejecución | Skills | Depende de |
|------|-----------|--------|------------|
| **0. Pre-check** | directo | — | — |
| **1. CLI Mechanical** | directo | `just verify`, `just audit-cargo`, `just machete`, `just size` | — |
| **2. Security** | sub-agente | `security-and-hardening` | Phase 1 |
| **3. Performance** | sub-agente | `performance-optimization` | Phase 1 |
| **4. Code Review** | sub-agente | `code-review-and-quality`, `ponytail-review` | Phase 1 |
| **5. Root Cause** | sub-agente | `systematic-debugging` | failures previos |
| **6. Deep Module** | sub-agente | `review-deep` | Waves 1-3 |
| **7. Full ISO** | sub-agente | `vantadb-full-review` | Waves 1-3 |
| **7a. CI/CD Parity** | directo | verificar que cambios en Cargo.toml/package.json/pyproject.toml se reflejen en `.github/workflows/*.yml` | Phase 1 |
| **7b. Skills Review** | sub-agente | cargar skills de review una por una con `skill <nombre>`, cada skill puede vetar | Phase 1 |
| **8. Certify** | sub-agente | `vantadb-certify` + `just certify` | Todas |

---

## Wave Execution Plan (modo `full`)

```
Wave 1: Phase 0 + Phase 1          (directo, blocking)
Wave 2: Phase 2 | Phase 3 | Phase 4 (paralelo, 3 sub-agentes)
Wave 3: Phase 5                      (condicional, si hay failures)
Wave 4: Phase 6 | Phase 7            (paralelo, 2 sub-agentes)
Wave 5: Phase 8                      (blocking, última)
```

Skills en Wave 2 son independientes entre sí → spawn 3 sub-agentes en paralelo via `task` tool. Nombra el `subagent_type` según la fase: Fase 2 → `vanta-audit`, Fase 3 → `vanta-tuner`, Fase 4 → genérico (code review, no mapea a un vanta-* específico).
Skills en Wave 4 son independientes entre sí → spawn 2 sub-agentes en paralelo (genéricos, skills específicas en cada fase).

**Budget note:** Waves paralelas consumen ~3-5× tokens por el contexto de cada sub-agente. Para cambios < 50 líneas, preferí `/audit quick` o `/audit review`.

**Upstream/downstream:** `/build` (genera el diff a auditar) → `/audit` → `/ship` (consume findings como pre-flight).

---

## Modo `certify` — Ejecución secuencial con hard stop

Cada layer se ejecuta una por una. Detenerse al primer error.

Para cada layer:
1. Mostrá "Layer N: [nombre]" antes de empezar
2. Ejecutá el/los comandos mecánicos reales
3. Si falla → mostrá el error exacto + "❌ LAYER N FAILED — abortando"
4. Si pasa → mostrá "✅ Layer N: [nombre]"

### Pre-check (Phase 0)
Si el diff (`git diff --name-only HEAD`) está vacío, usá `git diff --name-only HEAD~1`.

### CI/CD Parity (Phase 7a)
Para cada cambio en Cargo.toml/package.json/pyproject.toml,
verificar que los .github/workflows/*.yml reflejen las nuevas dependencias y env vars.
Si el diff omite actualizar un workflow → FAIL.

### Skills de Review (Phase 7b)
Cargar skills de review una por una con `skill <nombre>`.
Cada skill puede vetar el push. Si veta, registrá su objeción y abortá.

### Mensaje final
- Si todas las layers pasaron ✅: "✅ CERTIFY PASSED — safe to push"
- Si alguna falló ❌: "❌ CERTIFY FAILED — fix errors above before pushing"

---

## Modo `review` — Five-axis code review + deep analysis

Fases: 0 → 4 → 5 → 6 → 7

### Phase 4 — Code Review (five-axis)
Revisar los cambios actuales (staged o recent commits) en estos 5 ejes:

1. **Correctness** — ¿Cumple la spec? ¿Edge cases cubiertos? ¿Tests adecuados?
2. **Readability** — ¿Nombres claros? ¿Lógica directa? ¿Bien organizado?
3. **Architecture** — ¿Sigue patrones existentes? ¿Límites limpios? ¿Nivel de abstracción correcto?
4. **Security** — ¿Input validado? ¿Secrets seguros? ¿Auth chequeado? (Usar `security-and-hardening`)
5. **Performance** — ¿Sin N+1 queries? ¿Sin operaciones sin límite? (Usar `performance-optimization`)

Categorizar findings como: **Critical**, **Important**, o **Suggestion**.
Cada finding debe incluir referencia `archivo:línea` y recomendación de fix.

---

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
| 7a. CI/CD Parity | — | ✅ | — | — |
| 7b. Skills Review | — | ✅ | — | — |
| 8. Certify | — | ✅ | — | ✅ |

---

## Quick Reference

| Comando | Qué hace | Fases | ~Dur |
|---------|----------|-------|------|
| `/audit` | Pipeline completo (waves paralelas) | 0→1→2→3→4→5→6→7→8 | 30min |
| `/audit quick` | Solo CLI checks | 1 | 2min |
| `/audit certify` | Pre-push gate secuencial (hard stop) | 0→1→2→3→4→7a→7b→8 | 15min |
| `/audit review` | Deep review + five-axis | 0→4→5→6→7 | 20min |

## Output

Al finalizar: escribí `docs/audit-reports/audit-<mode>-<timestamp>.md` **y** `docs/last-audit-state.json` con:

```json
{
  "timestamp": "ISO8601",
  "mode": "full|quick|certify|review",
  "veredicto": "PASS|FAIL",
  "findings_critical": 0,
  "report_file": "docs/audit-reports/audit-<mode>-<timestamp>.md"
}
```

Formato del reporte:

```markdown
# Audit Report: <mode> — <timestamp>

## Scoreboard
| Fase | Estado |
|------|--------|
| 0. Pre-check | ✅/❌ |
| 1. CLI | ✅/❌ |
| ... | ... |

## Findings (priorizados)
### Critical
- [file:line] descripción + recomendación

### Important
- [file:line] descripción + recomendación

### Suggestion
- [file:line] descripción + recomendación

## FODA (Deducción Estratégica)

| Dimensión | Hallazgos |
|-----------|-----------|
| **Fortalezas** | Aspectos sólidos identificados: patrones correctos, cobertura de tests, arquitectura limpia |
| **Oportunidades** | Mejoras potenciales: refactors estratégicos, features faltantes, alineación con roadmap |
| **Debilidades** | Problemas concretos: code smells, deuda técnica, gaps de cobertura, falta de tests |
| **Amenazas** | Riesgos externos: dependencias inseguras o desactualizadas, breaking changes upstream, compatibilidad |

## Veredicto
✅ PASS | ❌ FAIL — fix above before shipping
```
