# Plan de Ejecución: Launch Web Campaign

> **Campaign ID: abd7ecf8-f034-4362-ad18-c2cb21e59e88
> **Inicio:** 2026-08-04
> **Estado: completed
> **Fuente:** `docs/Backlog.md` (scope: WEB-18, MKT-15, MKT-05, TSK-103, GH-119, WEB-001, OLD-01)

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 5 |
| 🟡 DEFER | 1 |
| ❌ SKIP | 1 |
| 🔴 BLOQUEADO | 0 |

## Gate Justification (scope completo)

- **WEB-18** → ✅ DO: conflicto verificado — `vanta-data.ts:540-541` expone plan Team $49/mes; `docs/strategy/GO_TO_MARKET.md` no lo define (tier Business $499/mo). Incoherencia visible en sitio público pre-launch.
- **MKT-15** → ✅ DO: `/benchmarks` existe (BenchmarksView + BenchmarkRace, BENCH-01/SIFT1M) pero sin tabla competitiva VantaDB vs Pinecone/Weaviate/Chroma — asset marketing #1 para audiencia técnica.
- **MKT-05** → ✅ DO: 4 posts en `docs/blog/` (introducing, why_i_built, how_hybrid_search_works, sqlite_for_ai_agents); BLOG_SERIES_PLAN requiere 5 pre-launch.
- **TSK-103** → ❌ SKIP: ya implementado — `benchmarks/README.md` § Quick start (standalone path desde PyPI, sin toolchain Rust) + `requirements.txt` (`vantadb-py>=0.5.0`) + `competitive_bench.py` (LanceDB/Chroma, datasets ann-benchmarks). Cubierto por NUEVO-10 (completada 2026-08-02).
- **GH-119** → ✅ DO: `docs/tutorials/migrate-from-vectara.md` no existe; Vectara cerró self-service tier → gap de mercado local-first (research en `docs/audit-reports/vectara-competitive-research-2026-07-27.md`).
- **WEB-001** → ✅ DO: `/playground` usa `CodePlayground` (simulador, no WASM real); `vantadb-wasm/pkg/` existe (`vantadb_wasm.d.ts` + `.wasm`) listo para integrar.
- **OLD-01** → 🟡 DEFER: PGWire 2-3 semanas, roadmap post-launch (Phase 8 🔵). Esfuerzo >> impacto para campaña de lanzamiento actual.

## Tasks

### Task 1: WEB-18 — Alinear pricing web con GO_TO_MARKET

- **Esfuerzo:** 🟡
- **Prioridad:** 🔴
- **Archivos clave:** `web/src/components/vanta/vanta-data.ts`, `docs/strategy/GO_TO_MARKET.md`
- **Gate Justificación:** conflicto activo entre sitio (Team $49) y estrategia GTM ($499 Business); decisión (a) agregar Team $49 a GTM, (b) alinear vanta-data.ts a GTM, o (c) eliminar pricing del sitio.
- **Gate Result:** ✅ DO
- **Contrato: 
- **Commit:**

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:** Decisión de producto — requiere confirmación del usuario sobre cuál opción (a/b/c). Default sugerido: (b) alinear a GTM.

### Task 2: MKT-15 — Tabla competitiva en /benchmarks

- **Esfuerzo:** 🟡
- **Prioridad:** 🔴
- **Archivos clave:** `web/src/components/vanta/benchmarks-view.tsx`, `web/src/components/vanta/vanta-data.ts`, `benchmarks/competitive_bench.py` (datos fuente)
- **Gate Justificación:** página existe pero sin comparativa VantaDB vs Pinecone/Weaviate/Chroma; datos reales ya disponibles de `competitive_bench.py`.
- **Gate Result:** ✅ DO
- **Contrato:** "grep 'Pinecone' en web/src/components/vanta/benchmarks-view.tsx → presente; `npm run build` pasa"
- **Task file:** `skills/campaign-executor/tasks/MKT-15.md`
- **Estado:** ✅ COMPLETED
- **Branch:**
- **Commit:**

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:** Puede delegarse a vanta-worker (web) o vanta-docs (datos). Validar cifras contra últimos resultados de `competitive_bench.py`.

### Task 3: MKT-05 — 5º blog post pre-launch

- **Esfuerzo:** 🟡
- **Prioridad:** 🟠
- **Archivos clave:** `docs/blog/` (4 existentes), `docs/strategy/BLOG_SERIES_PLAN.md`
- **Gate Justificación:** plan de serie exige 5 posts pre-launch; faltan 1. Tema sugerido: benchmarks/hard numbers (en línea con MKT-15).
- **Gate Result:** ✅ DO
- **Contrato:** "glob docs/blog/*.md → 5 archivos; BLOG_SERIES_PLAN.md actualizado con el 5º post"
- **Task file:** `skills/campaign-executor/tasks/MKT-05.md`
- **Estado:** ✅ COMPLETED
- **Branch:**
- **Commit:**

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:** Delegar a vanta-docs. Publicación web separada (web/blog) — solo contenido.

### Task 4: GH-119 — Guía de migración Vectara → VantaDB

- **Esfuerzo:** 🟡
- **Prioridad:** 🟠
- **Archivos clave:** `docs/tutorials/migrate-from-vectara.md` (nuevo), `docs/tutorials/index.md`, `docs/audit-reports/vectara-competitive-research-2026-07-27.md` (research)
- **Gate Justificación:** Vectara cerró self-service tier → equipos buscan alternativas local-first; guía con workflow completo + ejemplos Python funcionales.
- **Gate Result:** ✅ DO
- **Contrato:** "Test-Path docs/tutorials/migrate-from-vectara.md → True; grep 'corpus-export' en el archivo → presente; issue #119 cerrado en GitHub"
- **Task file:** `skills/campaign-executor/tasks/GH-119.md`
- **Estado:** ✅ COMPLETED
- **Branch:**
- **Commit:**

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:** Cierre: `gh issue close 119 --repo ness-e/Vantadb`. Delegar a vanta-docs.

### Task 5: WEB-001 — WASM demo real en /playground

- **Esfuerzo:** 🟡
- **Prioridad:** 🟡
- **Archivos clave:** `web/src/app/playground/page.tsx`, `web/src/components/vanta/code-playground.tsx`, `vantadb-wasm/pkg/`
- **Gate Justificación:** CodePlayground es simulador; `vantadb-wasm/pkg` existe listo para integrar (`@vantadb/wasm`).
- **Gate Result:** ✅ DO
- **Contrato:** "grep 'wasm' en web/src/components/vanta/code-playground.tsx → import real de @vantadb/wasm; `npm run build` pasa"
- **Task file:** `skills/campaign-executor/tasks/WEB-001.md`
- **Estado:** ✅ COMPLETED
- **Branch:**
- **Commit:**

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:** Requiere validar API de `vantadb-wasm/pkg` (`vantadb_wasm.d.ts`). Delegar a vanta-worker (bindings) o vanta-worker web.

## Orden de ejecución recomendado

1. **WEB-18** (🔴 decisión de producto, desbloquea sitio público) — requiere confirmación de usuario (opción a/b/c)
2. **MKT-15** (🔴 asset marketing #1, depende de cifras de competitive_bench — ya disponibles)
3. **MKT-05** (🟠 contenido, independiente)
4. **GH-119** (🟠 docs, independiente)
5. **WEB-001** (🟡 demo, independiente, última por riesgo WASM)

Paralelizables: MKT-05 + GH-119 (ambos docs, sin dependencias). WEB-18 y MKT-15 tocan `vanta-data.ts` → secuenciales entre sí (posible conflicto de edición).

## Checkpoints

- [ ] Después de WEB-18: `npm run build` web pasa
- [ ] Después de MKT-15: `/benchmarks` muestra tabla competitiva
- [ ] Después de MKT-05 + GH-119: coverage docs validado (`scripts/validate-docs-coverage.ps1`)
- [ ] Después de WEB-001: `/playground` ejecuta WASM real (no simulador)

=== RECITATION ===
Campaign ID: 611e14ec-729b-4973-85f4-1f72b84cb4e3
Objetivo activo: Campaña launch web
Estado: completed
Última acción: WEB-18 pricing alineado a GTM, 5/5 DO completadas
Resultado: ✅
Próxima acción: skill progreso + reporte final
Contrato: 
Próxima tarea si completa: null
=== END RECITATION ===
