# BLOG-CTA — CTAs + metadata serie blogs + posts 6-7

**Plan:** `docs/dev/plans/2026-09-08-backlog.md` → Task 10 · **Estado:** ⏳ IN PROGRESS
**Branch:** develop · **Appetite:** max 3d · **Esfuerzo:** 🟡 (2 slices; este intento: slice completo si cabe)
**Contrato:** `npm run lint` web 0 + `npx tsc --noEmit` 0 + M3/M4 drifts corregidos + CTA en 2 posts + drafts posts 6-7 (Ollama+VantaDB, Claude Code MCP) + sin claims performance sin benchmark (Regla 11)
**SDP:** campaign-executor, frontend-ui-engineering, design-taste-frontend, incremental-implementation, test-driven-development, context-engineering, source-driven-development, doubt-driven-development + documentation-and-adrs, writing-guidelines (persona vanta-docs §6)

## Spec (decisiones con evidencia)

| # | Decisión | Elección | Evidencia |
|---|----------|----------|-----------|
| 1 | Dirección M4 (title drift) | Ganan títulos web (live, SEO-indexados); drafts alinean frontmatter+H1 | web es superficie publicada (sitemap, canonicals, OG); drafts `draft:true` sin publicar. Blast radius menor |
| 2 | Dirección M3 (date drift) | Drafts adoptan fechas web: introducing→2026-04-10, how_hybrid→2026-04-24, sqlite→2026-05-15, why_i_built→2026-06-05 | `vanta-data.ts:883,923,967,1003` leído; drafts todos 2026-06-06 |
| 3 | Canonicals drafts | `vantadb.dev` → `https://vantadb.vercel.app/blog/<slug>` | `SITE_URL=https://vantadb.vercel.app` (`web/lib/site-config.ts`, AGENTS.md web); vantadb.dev no existe |
| 4 | Posts 6-7 = solo drafts `docs/user/blog/` | NO añadir a `BLOG_POSTS` web (eso = publicar) | Plan: "solo publicación final es humana — los drafts quedan en repo" |
| 5 | Dict authors 0-2 ES+EN `ness-e`→`VantaDB Team` | Manifiesto live manda; layout.tsx metadata ya usa `post.author` directo | `layout.tsx:30` usa author sin tt; page.tsx usa tt → hoy metadata dice Team y display dice ness-e (inconsistente) |
| 6 | Claim 59%/750→1195 QPS (sqlite draft) | SIN FUENTE → quitar números, dejar cualitativo + citar bench real donde aplique | `rg` BENCHMARKS.md: 0 hits page-fault/750 QPS; −59% citado es de IVF clones (línea 672-675, otro experimento); 1195.9 es p50 µs (línea 692, no QPS); INV-012 fija mejora BFS real ~7-9% (wontfix) |
| 7 | Claims 4.01x/2.43ms (sqlite draft §3) | SE MANTIENEN — con fuente | BENCHMARKS.md §6 Líneas 130-135: tabla batch 243.01ms/2.43ms/4.01x |
| 8 | "single CPU clock cycle" (how_hybrid draft §3) | → "single instruction" | Impreciso a nivel HW (throughput/latencia ≠ 1 ciclo); `wide::f32x8` procesa 8 lanes por instrucción |

## Impacto mapeado (Regla 0)

- **Leídos completos:** `web/src/app/blog/page.tsx` (101L), `[slug]/page.tsx` (174L), `[slug]/layout.tsx` (40L), `blog/layout.tsx` (22L), `vanta-data.ts:878-1035` (BLOG_POSTS 4 posts), `sitemap.ts` (blogSlugs 4), `dictionaries.ts` (90 keys blogPost.*), `language-provider` + `i18n-utils.ts` (tt=key→dict??fallback), `docs/user/blog/how_hybrid_search_works.md` (156L), `sqlite_for_ai_agents.md` (117L), `why_i_built.md` + `introducing_vantadb.md` (frontmatter), `BLOG_SERIES_PLAN.md` (211L), `BENCHMARKS.md` §6/§14, `docs/api/MCP.md` (server cmd), `docs/dev/tasks/MKT-18i.md` (stack Ollama).
- **Referencias hacia dentro (lo que toco → qué usa):** vanta-data.ts → page.tsx + [slug]/page.tsx + [slug]/layout.tsx + sitemap.ts (solo lectura de slugs; no añado slugs live). dictionaries.ts ← tt() en ambas páginas blog.
- **Referencias entrantes:** `use-vanta-navigate.ts:64` y `CASE_STUDIES` usan slug `agent-local-memory-ollama` (case-study, NO blog — no colisiona con mi draft `ollama-vantadb-local-memory`). `campaign-ai-agent-memory.md` cita números sqlite (doc interna, fuera de scope; no tocar).
- **Claves i18n:** contenido usa `blogPost.data.<idx>.content.<i>` indexado por posición → añadir bloques AL FINAL no desplaza índices existentes. Nuevos índices sin key en dict → fallback EN (correcto hasta traducir; yo añado ES+EN).
- **Veredicto:** edición aditiva + frontmatter; sin cambio de rutas/slugs live; sin tocar `.opencode`, `opencode.jsonc`, `docs/dev/Backlog.md` (WIP ajeno). Riesgo bajo, reversible por archivo.

## Pasos

- [x] 1. Frontmatter drafts M3/M4/canonical/byline (4 files) + verify
- [x] 2. Regla 11 prose fixes en 3 drafts + verify
- [x] 3. CTA contenido web posts idx1,2 (vanta-data.ts) + dict ES/EN (8 keys contenido + 6 authors) + verify lint/tsc
- [x] 4. CTA UI banner `[slug]/page.tsx` + 4 dict keys ES/EN + verify
- [x] 5. Drafts posts 6-7 (`ollama-vantadb-local-memory.md`, `claude-code-mcp-memory.md`) + verify
- [x] 6. Verify full contrato + commit solo paths propios + cierre

## Verify (2026-09-09)

- `npx tsc --noEmit` → exit 0
- `npm run lint` (eslint .) → exit 0
- M3: draft dates == web dates (04-10/04-24/05-15/06-05) ✅
- M4: draft titles == web titles ✅ (decisión Spec #1: web gana)
- CTA web: idx1 +2 bloques (content.8/.9), idx2 +2 bloques (content.6/.7) + banner UI en `[slug]/page.tsx` ✅
- Posts 6-7: `docs/user/blog/ollama_vantadb_local_memory.md` (3851B) + `claude_code_mcp_memory.md` (3403B), ambos `draft:true` ✅
- Regla 11: `rg '59%|750|1,195|10x|clock cycle'` en 4 drafts → 0 hits; 4.01x/2.43ms conservados con cita BENCHMARKS §6 ✅

## Deuda / NO tocado (scope discipline)

- Dicts ES+EN son copias stale del EN anterior (ej: `blogPost.data.0.content.6` ES menciona 1.2ms que el fallback ya no tiene; nº con fuente BENCHMARKS §1, no es violación Regla 11, solo drift de traducción). Retraducir ~90 keys = fuera de appetite → posible FIND-* futuro, no este task.
- `docs/user/blog/campaign-ai-agent-memory.md` repite "QPS 750→1,195" (doc interna de campaña, no publicada) → notar, no tocar.
- `BLOG_SERIES_PLAN.md` status update (línea 20) queda stale tras este task → lo actualiza el orquestador/human al publicar; no tocar (evitar churn).
- WIP ajeno: `M .opencode`, `M docs/dev/Backlog.md`, `M opencode.jsonc`, `?? Investigacion-plan.md`, `?? docs/dev/plans/2026-09-08-backlog.md` → NO tocar ni stagear.
