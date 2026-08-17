# FND-13-F1 (BENCH01): Claims fantasma de performance en web/src/

## Metadata
- **Plan:** docs/plans/2026-08-16-wave-followups.md (W3)
- **Fuente:** hallazgo BENCH01 de FND-13 (claims ~5,400 vec/s retirados del README por PERF-01 pero vivos en web/src/)
- **Estado:** ⏳ IN PROGRESS · **Sub-agente:** vanta-worker
- **Prioridad:** 🟡

## Objetivo
Encontrar y neutralizar los claims de performance fantasma (~5,400 vec/s y similares) que siguen vivos en `web/src/` (frontend Next.js). Aplicar Regla 11: sin benchmark reproducible citado → quitar el número o el adjetivo de performance. NO romper el diseño (solo texto de claims).

## Archivos clave
- `web/src/` (componentes, landing, secciones de features — grep de "vec/s", "5,400", "queries", "ms", "vectores por segundo", claims numéricos de perf)
- `docs/Investigaciones/FND-13-benchmarks-honestos.md` (inventario — BENCH01 documentado)

## Steps
1. DISCOVERY: grep en web/src de patrones de claims de performance (vec/s, rec/s, ms, QPS, "más rápido", "x faster", números sueltos de throughput); leer los componentes afectados
2. Clasificar: claims con fuente reproducible (mantener) vs fantasma (quitar/reformular)
3. Implementar: editar los textos — quitar números sin fuente; si el claim es cualitativo ("alto rendimiento"), reformular sin número o con referencia al benchmark canónico (docs/operations/BENCHMARKS.md)
4. Verificar: grep post-cambio → 0 claims falsos restantes; build web no roto (`npm run build` en web/ o al menos tsc/next lint si el build es lento)
5. Task file + RESULTADO

## Contrato (verify mecánico)
- grep de claims numéricos falsos en web/src → 0 (o todos reformulados sin número)
- Build web pasa (o lint/tsc si build completo es >5min — documentar cuál)
- Diseño intacto (solo contenido textual)

## Invariantes (handoff)
- NO tocar: docs/Backlog.md, AUD-024.md, verify-log.jsonl, _vanta-cli.ps1, plan file, .opencode/AGENTS.md, .opencode/agents/*, docs/operations/BENCHMARKS.md
- NO git add/commit; NO campaign_update_task_state
- No cambiar estructura/layout/i18n keys — solo el texto visible del claim
- Si i18n (es/en): limpiar ambos idiomas

## Fases
- SECURITY: n/a
- PERFORMANCE: n/a (contenido)

## Resultado
```
RESULTADO: ✅ COMPLETO | 🟡 INCOMPLETO | ❌ FALLIDO
STEPS_OK: <n>/<M>
PROXIMO_STEP: <...>
COMMIT_HASH: ninguno (lead commitea)
ARCHIVOS: <paths tocados>
VERIFY_CONTRATO: <pasa | no-corrido | falla>
BLOQUEO: <ninguno | ...>
```