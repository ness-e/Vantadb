---
description: "Audit pipeline unificado: CLI checks + 8 skills de review en subagentes paralelos + reporte unificado"
---

> **ENTRY POINT — Audit Command**
> El agente DEBE leer este archivo cuando el usuario envía un mensaje que empieza con `/audit`.
> Path resolution: `prompts/X.md` → `.opencode/task-system/prompts/X.md`
> Skills: `skills/X` → `.opencode/skills/X/`
> Instrucciones: cargar skills listados, ejecutar fases según modo. Waves paralelas vía sub-agentes.
> Al finalizar: escribir reporte en `docs/audit-reports/`.

# /audit — VantaDB Audit Pipeline

Pipeline de auditoría completo. Unifica CLI checks (`just` 5 comandos) + 8 skills de review en subagentes paralelos + reporte estructurado.

Cargá las skills `vanta-design-orchestrator`, `progreso`, `ponytail` (full) primero.
Usá `prompts/audit-full.md` para ejecución completa.

## Router: detectar modo según el argumento

- **Sin argumento o `full`** → pipeline completo: fases 0→1→2→3→4→5→6→7→8
- **`quick`** → solo Phase 1 (CLI checks via `just verify`), ~2min
- **`certify`** → pre-push gate: fases 0→1→2→3→4→8
- **`review`** → revisión profunda sin CLI: fases 0→4→5→6→7

## Fases

| Fase | Ejecución | Skills | Depende de |
|------|-----------|--------|------------|
| **0. Pre-check** | directo | — | — |
| **1. CLI Mechanical** | directo | `just verify`, `just audit-cargo`, `just machete`, `just size` | — |
| **2. Security** | sub-agente | `security-and-hardening` | Phase 1 ✅ |
| **3. Performance** | sub-agente | `performance-optimization` | Phase 1 ✅ |
| **4. Code Review** | sub-agente | `code-review-and-quality`, `ponytail-review` | Phase 1 ✅ |
| **5. Root Cause** | sub-agente | `systematic-debugging` | failures previos |
| **6. Deep Module** | sub-agente | `review-deep` | Waves 1-3 |
| **7. Full ISO** | sub-agente | `vantadb-full-review` | Waves 1-3 |
| **8. Certify** | sub-agente | `vantadb-certify` + `just certify` | Todas |

## Wave Execution Plan

```
Wave 1: Phase 0 + Phase 1          (directo, blocking)
Wave 2: Phase 2 | Phase 3 | Phase 4 (paralelo, 3 sub-agentes)
Wave 3: Phase 5                      (condicional, si hay failures)
Wave 4: Phase 6 | Phase 7            (paralelo, 2 sub-agentes)
Wave 5: Phase 8                      (blocking, última)
```

Skills en Wave 2 son independientes entre sí → spawn 3 sub-agentes en paralelo via `task` tool.
Skills en Wave 4 son independientes entre sí → spawn 2 sub-agentes en paralelo.

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

## Quick Reference

| Comando | Qué hace | Fases | ~Dur |
|---------|----------|-------|------|
| `/audit` | Pipeline completo | 0→1→2→3→4→5→6→7→8 | 30min |
| `/audit quick` | Solo CLI checks | 1 | 2min |
| `/audit certify` | Pre-push gate | 0→1→2→3→4→8 | 15min |
| `/audit review` | Deep review | 0→4→5→6→7 | 20min |

## Output

Al finalizar: `docs/audit-reports/audit-<mode>-<timestamp>.md` con findings priorizados, scoreboard, y veredicto.
