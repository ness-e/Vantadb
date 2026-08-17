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
RESULTADO: ✅ COMPLETO
STEPS_OK: 5/5
PROXIMO_STEP: ninguno — esperar commit del lead
COMMIT_HASH: ninguno (lead commitea)
ARCHIVOS: web/src/components/vanta/vanta-data.ts, web/src/components/vanta/benchmarks-view.tsx, web/src/components/vanta/latency-comparator.tsx, web/src/components/vanta/benchmark-race.tsx, web/src/components/vanta/site-navbar.tsx, web/src/components/vanta/navbar.tsx, web/src/components/vanta/metrics-bar.tsx, web/src/components/vanta/use-cases.tsx, web/src/components/vanta/core-engine.tsx, web/src/components/vanta/architecture.tsx, web/src/components/vanta/code-terminal.tsx, web/src/components/vanta/easter-egg.tsx, web/src/lib/dictionaries.ts, web/src/app/layout.tsx, web/src/app/latency/page.tsx, web/src/app/latency/layout.tsx, web/src/app/benchmarks/layout.tsx, web/src/app/engine/layout.tsx, web/src/app/opengraph-image.tsx, web/src/app/[...slug]/page.tsx, src/storage/engine/tests/ops.rs
VERIFY_CONTRATO: pasa (grep claims fantasma → 0; rg números serie vieja superseded → 0; tsc --noEmit y cargo check -p vantadb limpios; build completo no corrido — tsc/lint elegidos por tiempo, documentado en step 4)
BLOQUEO: ninguno
```

## Post-review (P2-01 changes-required menor — aplicado)
- Re-sincronizado BENCH01/ENGINES/bars con BENCHMARKS.md §2 vigente (FND-13-F2): Ingestion 13.174ms/74 ops/sec, HNSW 2.024ms/494 qps, Hybrid 3.114ms/321 qps. Serie vieja (95 ops, 10.7/62.0/179.8ms) eliminada — marcada superseded en BENCHMARKS.md:65.
- BM25 eliminado de comparadores y table (p50 0.0035ms es outlier degenerado no representativo, N/D en fuente).
- `bm25Latency` (campo muerto) removido de PRODUCT.metrics.
- src/storage/engine/tests/ops.rs:1250 comentario FND-02 "apply the HNSW entry" → "apply the volatile_cache entry" (version check real es contra volatile_cache, confirmado en maintenance.rs:328).