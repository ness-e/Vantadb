# FND-04-F1: Extraer ADR formal de la decisión zero-copy Arrow (diferida)

## Metadata
- **Plan:** docs/plans/2026-08-16-wave-followups.md (W4)
- **Fuente:** follow-up de proceso FND-04 (decisión "DIFERIR con ADR" quedó embebida en el reporte de investigación, sin archivo formal)
- **Estado:** ⏳ IN PROGRESS · **Sub-agente:** vanta-docs
- **Prioridad:** 🟢

## Objetivo
Extraer `docs/architecture/adr/ADR-025-*.md` formal desde `docs/Investigaciones/FND-04-arrow-zero-copy.md` (decisión DIFERIR + razón + señal de re-apertura medible), siguiendo la plantilla `docs/_templates/adr.md`. El contenido ya fue articulado y aprobado en la campaña — esta tarea consolida en formato ADR (disciplina Regla 5).

## Archivos clave
- `docs/Investigaciones/FND-04-arrow-zero-copy.md` (fuente de la decisión), `docs/_templates/adr.md` (plantilla), `docs/architecture/adr/` (numeración: último es ADR-024), `docs/architecture/adr/ADR-023-backend-compaction.md` (modelo con señal de reapertura)

## Steps
1. DISCOVERY: leer el reporte FND-04 (decisión, alternativas, señal), la plantilla ADR, verificar numeración (ADR-025 tras ADR-024)
2. Escribir ADR-025: Contexto (path actual por binding: Python output con copia, Node serde_json peor, WASM input pendiente), Decisión (DIFERIR zero-copy Arrow; output Python con copia es deliberado por SEC-01/AUDIT-01), Consecuencias (deuda de serialización, señal de reapertura: benchmark top_k=10_000 ≥1M records overhead >30%, o necesidad de interop pandas/polars)
3. Referencia cruzada: el reporte FND-04 apunta al ADR-025 (agregar 1 línea si el reporte no lo menciona)
4. Task file + RESULTADO

## Contrato (verify mecánico)
- `ADR-025-*.md` existe en docs/architecture/adr/ con Contexto/Decisión/Consecuencias + señal de reapertura
- Numeración correcta (tras ADR-024)
- Sin contradicción con el reporte FND-04 (misma decisión/evidencia)

## Invariantes (handoff)
- NO tocar: docs/Backlog.md, AUD-024.md, verify-log.jsonl, _vanta-cli.ps1, plan file, .opencode/AGENTS.md, .opencode/agents/*, código
- NO git add/commit; NO campaign_update_task_state
- No borrar el reporte original; no inventar evidencia nueva (todo viene del reporte FND-04)
- Inglés (docs técnicas) — el reporte de investigación es español (planning), el ADR en inglés

## Fases
- SECURITY: n/a
- PERFORMANCE: n/a

## Resultado
```
RESULTADO: ✅ COMPLETO
STEPS_OK: 4/4
PROXIMO_STEP: ninguno
COMMIT_HASH: ninguno (lead commitea)
ARCHIVOS: docs/architecture/adr/ADR-025-zero-copy-arrow-deferred.md (nuevo), docs/Investigaciones/FND-04-arrow-zero-copy.md (1 línea cross-ref), .opencode/skills/campaign-executor/tasks/FND-04-F1.md (resultado)
VERIFY_CONTRATO: pasa
BLOQUEO: ninguno
```