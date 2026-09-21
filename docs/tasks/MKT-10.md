# MKT-10: "AI Agent Memory" campaign (rescatar con DoD)

## Metadata
- **Plan file:** `docs/plans/2026-08-05-backlog-validation-actions.md` → Task 53
- **Creado:** 2026-08-05
- **last-synced:** 2026-08-05
- **Estado:** ✅ COMPLETED — Camino A (reescrito con checklist medible)

## Commit Save Point
- **Commit:** `docs(MKT-10): reescribir campaña AI Agent Memory con checklist medible`
- **Hash:** (ver `git log -1 --format="%H"`)
- **Staging:** selectivo — solo `docs/blog/campaign-ai-agent-memory.md` + este archivo. NO se tocó el plan file.

## Decisión (Camino A — reescrito con deliverables medibles)
La campaña se reescribe en `docs/blog/campaign-ai-agent-memory.md` con checklist verificable. NO se cierra como cubierta por INV-006/BLOG_SERIES_PLAN porque:

1. **INV-006 está completada pero es plan-only** (`BLOG_SERIES_PLAN.md:14` — "Scope: Planning only. No new content is written"). Un plan no cubre deliverables de campaña.
2. **D1 (landing) y D3 (demo) sí están cubiertos** por assets reales (ver checklist).
3. **D2 (blog benchmark vs full-context) es un gap real**: los posts existentes miden perf de engine (QPS/recall vs LanceDB/Chroma), NO el trade memory-vs-full-context. El post GraphRAG "reducción de tokens 40-60%" está planeado pero no draftado (`BLOG_SERIES_PLAN.md` 4.3), y backlog-validation advierte "métricas parecen claims, no runs". Cerrar como cubierto sería deshonesto.

## Checklist entregables (verificado con paths reales)
| ID | Entregable | Estado | Evidencia |
|----|-----------|--------|-----------|
| D1 | Landing "AI Agent Memory" | ✅ COVERED | `web/src/app/solutions/ai-agents/page.tsx` (ruta `/solutions/ai-agents`); copy `solutionsAgents.*` en `web/src/lib/dictionaries.ts`; tutorial `docs/tutorials/01-ai-agent-memory.md` |
| D2 | Blog benchmark memory vs full-context | ❌ OPEN | Sin post dedicado; solo parcial en `docs/blog/sqlite_for_ai_agents.md` + `benchmarks_vs_lancedb_chroma.md` (perf de engine, no full-context) |
| D3 | Demo interactiva | ✅ COVERED | `web/src/app/demo/page.tsx` + `web/src/app/playground/page.tsx` + `web/src/components/vanta/code-playground.tsx` (WASM in-memory) |

## Contrato cumplido
Checklist de campaña con entregables verificables en `docs/blog/campaign-ai-agent-memory.md`; sin items vagos. Regla anti-invención de cifras para D2 (precedente MKT-05: script reproducible + run real).

## Files created
- `docs/blog/campaign-ai-agent-memory.md` — doc de campaña con checklist medible
- `.opencode/skills/campaign-executor/tasks/MKT-10.md` — este commit save point

## Dependencias / next
- D2 queda como tarea downstream de contenido (patrón MKT-05: script en `benchmarks/`, post en `docs/blog/` con `run_date`).
