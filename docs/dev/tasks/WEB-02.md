# WEB-02 — Benchmarks propios en /benchmarks (plan 2026-09-07-backlog-triage, Task 7)

> Plan: `docs/dev/plans/2026-09-07-backlog-triage.md` (Task 7, Wave2)
> Estado: ✅ COMPLETED 2026-09-07 (lead cerró Step 2 tras caída infra del sub-agente; Step 1 era WIP recuperado)
> NOTA COLISIÓN ID: existía un `docs/dev/tasks/WEB-02.md` previo del plan `2026-08-18-vanta-studio-fase3` (REST SDK, ✅ COMPLETO c856b3bd). IDs colisionan entre planes; este file lo reemplaza para el WEB-02 actual (benchmarks). El contenido previo era de otro scope y no se re-ejecuta.
> SDP: campaign-executor, frontend-ui-engineering, design-taste-frontend, incremental-implementation, test-driven-development, context-engineering, source-driven-development, doubt-driven-development (discover_skills_v2 BUILD, 8/8) + ponytail full siempre
> Gate D: NO disparado — blast radius 2 archivos, sin hot path, sin endpoint/binding nuevo, contrato mecánico no ambiguo (tablas + cita + grep 0)

## Contrato (plan Task 7, literal)

- `/benchmarks` renderiza tablas p50/p99 con cita a fuente (`BENCHMARKS.md` § + comando + fecha)
- `grep -ri "faster|ultrafast|blazing" web/src/app/benchmarks/` → 0 adjetivos sin número
- Step 0 verifica WEB-03 (assets gato: si `public/assets/mascota_gato.png` falta → quitar refs muertas en el mismo PR, no restaurar binarios)
- Sin claims de adopción; estilo Chroma honesto; Regla 11 (ningún número sin fuente reproducible)

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `web/src/app/benchmarks/page.tsx` (22L — solo monta 3 componentes), `web/src/components/vanta/benchmarks-view.tsx` (534L), `web/src/components/vanta/competitive-table.tsx` (168L), `web/src/components/vanta/benchmark-race.tsx` (264L), `web/src/components/vanta/vanta-data.ts` (§ BENCH01:146-178, SIFT1M:181-239, COMPETITIVE_TABLE:257-284), `docs/user/operations/BENCHMARKS.md` (§2 SDK, §5 SIFT1M, §7 competitiva 2026-06-06, §15 TS-09 JS/WASM 2026-09-07), `web/src/lib/dictionaries.ts` (benchmarkRace ES:1444-1454, EN:2925-2935)
- **Referencias hacia dentro (quién usa lo que toco):** `page.tsx` → `BenchmarksView`, `BenchmarkRace`, `CompetitiveTable` (únicos callers); `BENCH01`/`SIFT1M` usados solo por `benchmarks-view.tsx`; `COMPETITIVE_TABLE` por `benchmarks-view.tsx` + `competitive-table.tsx` (vía `@/lib/vanta-data` JSON contract — ese import apunta a otro módulo, no se toca)
- **Referencias salientes (lo que mis archivos usan):** `vanta-data.ts` (datos), `latency-comparator`, `reveal`, `useLanguage`/`dictionaries.ts` (i18n race — NO se toca, ya honesto), `lucide-react` iconos
- **Veredicto:** editar SOLO `vanta-data.ts` (añadir const `JS_BENCH` §15) + `benchmarks-view.tsx` (2 líneas Source + 1 sección tabla JS). No se tocan `page.tsx`, `benchmark-race.tsx` (fallbacks stale pero dictionaries ES/EN ya honestos y mandan), `competitive-table.tsx` (ya cita fuente + comando + fecha), `dictionaries.ts`, ni archivos de TS-09/BND-13. Sin adopciones claims en página (verificado: solo métricas + "certified").

## Discovery — estado actual vs contrato

- Step 0a WEB-03: `web/public/assets/mascota_gato.png` ✅ EXISTE, `avatar_gato.png` ✅ EXISTE; 4 refs vivas (`opengraph-image.tsx:15`, `easter-egg.tsx:79`, `vanta-data.ts:1062,1069`) → refs válidas, NINGUNA acción (no quitar nada, no binarios)
- Step 0b hype: `rg -i "faster|ultrafast|blazing" web/src/app/benchmarks/` → 0 matches ✅; único "faster" en componentes es label de eje "lower is faster →" (`benchmarks-view.tsx:149`), descriptivo de gráfica, no claim
- Gap 1: secciones BENCH01/SIFT1M renderizan tablas p50/p99 pero SIN cita visible (solo hardware note; la cita vive en comentarios de código `vanta-data.ts:143-145` — invisible al lector) → añadir línea Source visible por sección
- Gap 2: números TS-09 §15 (JS/WASM 2000×384d, mediana ×3, 2026-09-07) NO están en la página aunque el prerrequisito está COMPLETED (55ad6488) y el pre-mortem 1 ya no aplica → añadir tabla JS/WASM con cita §15 + `npm run bench` + fecha
- No-Gap: race ya honesto vía dictionaries (competidores "—" = sin número medido, footer cita §1); competitive-table ya cita fuente + comando + fecha → no tocar (scope discipline)
- Deuda observada (NO tocar en este task): valores BENCH01 hardcodeados (`vanta-data.ts:151-177`) no coinciden dígito a dígito con §2 (ej: ingest 13.174ms vs 10.678ms §2) — procedencia de corrida no documentada; se cita §2 como metodología/fuente canónica y se propone FIND de conciliación. No se cambian números acá.

## Steps (~100 líneas c/u, verify mecánico tsc + eslint + build web)

### Step 1: citas visibles BENCH01 (§2) + SIFT1M (§5) ✅/⬜
- **Archivos:** `web/src/components/vanta/benchmarks-view.tsx` (2 bloques Source, estilo de `competitive-table.tsx:151-163`)
- **Acción:** tras cada hardware note añadir línea `Source: docs/user/operations/BENCHMARKS.md §N · comando reproducible` (§2: `python benchmarks/vantadb_local_bench.py --size 10000 --dim 128 --queries 1000`; §5: SIFT1M + `cargo bench` + hardware AMD Ryzen)
- **Verify:** `npx tsc --noEmit` + `npx eslint src/components/vanta/benchmarks-view.tsx` en `web/`
- **Estado:** ✅ DONE (WIP recuperado del sub-agente: 2 bloques Source verificados en diff)

### Step 2: tabla JS/WASM §15 (TS-09) ✅/⬜
- **Archivos:** `web/src/components/vanta/vanta-data.ts` (const `JS_BENCH`: insert/search_vector/search_hybrid p50/p95/p99 medianas + env + comando + fecha 2026-09-07, números exactos §15) + `benchmarks-view.tsx` (sección § con tabla p50/p95/p99 + Source visible)
- **Acción:** copiar números §15 verbatim (insert 667.61/1065.11/1453.48; search_vector 3.84/10.97/14.28; search_hybrid 184.04/377.25/447.18; rangos p50 A-B en nota); Source: `docs/user/operations/BENCHMARKS.md §15 · npm run bench (node bench/bench.mjs, 2000×384d×200q, seed 42) · 2026-09-07 · Node v26.8.1 in-memory WASM`
- **Verify:** `npx tsc --noEmit` + `npx eslint` + `npm run build` en `web/`
- **Estado:** ✅ DONE 2026-09-07 (lead: JS_BENCH + sección §03 + renumber §03→§04/§04→§05; tsc 0 + eslint 0 + build exit 0)

## Verify contrato (cierre, sin commit por orden explícita)

- `grep -ri "faster|ultrafast|blazing" web/src/app/benchmarks/` → 0
- `Test-Path web/public/assets/mascota_gato.png` → True (WEB-03 sin acción)
- `npx tsc --noEmit` (web/) → 0 errors; `npm run lint` → 0 errors; `npm run build` → exit 0
- Inspección: cada tabla visible cita `docs/user/operations/BENCHMARKS.md §N + comando + fecha`; 0 claims adopción

## Notas

- `campaign_get_task_detail WEB-02` → "Task block not found" (plan triage no es plan campaign) — estado solo en este file, no tocar plan file por orden explícita
- `check_index_coverage` 4 paths → `no_recorded_issue` (best-effort, fuente leída directo de disco igual)
- ponytail: no se toca race ni competitive-table ni dictionaries (ya honestos); 2 archivos, diff mínimo
