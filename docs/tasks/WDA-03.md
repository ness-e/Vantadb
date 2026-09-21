# WDA-03 — F3 Información: corregir claims falsos del sitio

**Plan:** docs/plans/2026-08-19-web-design-audit.md · Task 4 §6 · Estado inicial ⬜ PENDING
**Ruta:** vanta-worker · Sin git commit (instrucción del orquestador)

## Impacto mapeado (Regla 0)

- **Fuentes de verdad leídas completas:** `docs/operations/BENCHMARKS.md` (231L), `README.md` §instalación/quickstart (60-140), `docs/CHANGELOG.md` §0.4.0/0.5.0 (650-701). Primer commit repo: `2026-04-02` (`git log --reverse`). API Python verificada en código: `vantadb-python/vantadb_py/__init__.py` → `put()`, `get_memory(ns,key)`, `search_memory(ns,query_vector,top_k)`; import canónico `import vantadb` (README.md:66-68).
- **Archivos a editar (17):** `vanta-data.ts` (recall/snippets/tabla competitiva/changelog/blog/company), `competitive-benchmark.json` (números §7), `competitive-table.tsx` (footnote), `site-navbar.tsx:311`, `dictionaries.ts` (claims ×2 idiomas), `showcase/page.tsx`, `case-studies/page.tsx` + `[slug]/page.tsx`, `changelog/layout.tsx`, `trust-section.tsx:12`, `architecture.tsx:162`, `opengraph-image.tsx` (71,143), `easter-egg.tsx:92`, `core-engine.tsx:204`, `code-terminal.tsx` (89,108,173), `metrics-bar.tsx:23` (dead code — se edita solo para cumplir grep del contrato; se elimina en WDA-05), `docs-view.tsx` (34,108,234).
- **Referencias entrantes verificadas:** QUICKSTART_PYTHON ← code-terminal.tsx, docs-view.tsx. COMPETITIVE_BENCHMARK JSON ← competitive-table.tsx vía lib/vanta-data.ts (solo re-export tipado, sin números). CHANGELOG array ← changelog-section.tsx. BLOG_POSTS ← blog pages + layouts (date usado como publishedTime metadata).
- **No se toca:** web/remotion/, desktop/, plan file, navbar.tsx (dead code, WDA-05).
- **Veredicto:** cambios de contenido/copy only, sin lógica nueva. Riesgo bajo; build es el gate.

## Números autorizados (Regla 11 — solo BENCHMARKS.md/README/docs/CHANGELOG.md)

| Claim nuevo | Fuente |
|---|---|
| Recall@10 = 99.8% (scaling 10K–100K) | BENCHMARKS.md §1 líneas 28-30 (0.9980/1.0000/0.9980) |
| Tabla competitiva VantaDB/LanceDB/Chroma: 39.74ms p50 / 24.3 QPS / recall 24.50% / ingest 598.3 / index 16039.9 / RSS 236.5 etc. | BENCHMARKS.md §7 línea 190-192 |
| Snippet quickstart | README.md 91-121 verbatim |
| Versiones 0.4.0 (2026-07-20), 0.5.0 (2026-07-31) | docs/CHANGELOG.md 654-701 |
| Fechas blog 2026 post-first-commit | instrucción tarea (opción plausibles) |

## Steps

### S1 — Recall cherry-pick → 99.8% ✅
vanta-data.ts (metrics.recallAt10, comentarios, FAQ, USE_CASES ×2), dictionaries.ts (faq.a2 ×2, architecture.node.hits.body ×2, hero.caption ×2), architecture.tsx:162, opengraph-image.tsx:143, easter-egg.tsx:92, core-engine.tsx:204, code-terminal.tsx:173, metrics-bar.tsx:23 (dead code — solo para grep; se elimina en WDA-05).

### S2 — Coherencia p50 / tabla competitiva ✅
competitive-benchmark.json → valores §7 (schema v2, generated_at 2026-06-06, source → BENCHMARKS.md §7); competitive-table.tsx footnote reescrito sin claims del run fabricado + `methodology` unused removido; vanta-data.ts COMPETITIVE_TABLE rows → §7 + sourceLink a BENCHMARKS.md. latency-comparator/benchmarks-view ya citaban §2 correctamente (sin cambios).

### S3 — Snippet QUICKSTART canónico ✅
QUICKSTART_PYTHON = README verbatim (`import vantadb`, get_memory/search_memory); USE_CASES_DETAIL ×3, TUTORIALS (incluye fix de API inventada `ef_search=` → `distance_metric="cosine"` verificada en vantadb_py/__init__.py:117-141), flows de solutions ×6 (dict ES/EN), architecture/core-engine pseudo-tags. code-playground.tsx queda: modela la API real del TS SDK (`db.search({namespace,...})` — verificado en vantadb-ts tests). docs-view note/keywords; solutionPage.codeLang ×2 → "Python · vantadb".

### S4 — Versión navbar v0.5.0 ✅
site-navbar.tsx:311 → "v0.5.0 · embedded rust"; dictionaries ctaFinal.eyebrow ×2; docs-view.tsx:108; opengraph-image.tsx:71.

### S5 — Showcase = Official Examples ✅
showcase/page.tsx author "@ness-e"→"Official example" ×6 + fallbacks título/subtitle/tag/gridSubtitle; dictionaries showcasePage.title/subtitle/tag/items.0-5 ×2 idiomas alineados a los 6 ejemplos first-party reales (antes no coincidían con las URLs).

### S6 — Case studies composite disclaimer ✅
[slug]/page.tsx banner disclaimer visible tras PageHeader ("Composite scenario based on typical usage patterns — not a specific customer", clave caseStudy.compositeDisclaimer nueva); list subtitle sin "real-world deployments" + note honesta; dictionaries ×2 idiomas.

### S7 — CHANGELOG real 0.4.0/0.5.0 ✅
CHANGELOG array → 2 entradas fieles desde docs/CHANGELOG.md (0.5.0 2026-07-31 IVF+LSM; 0.4.0 2026-07-20 initial release); changelog/layout.tsx metadata.

### S8 — Blog fechas 2026 + trust honesto ✅
BLOG_POSTS dates → 2026-04-10/04-24/05-15/06-05 (todas > primer commit 2026-04-02); excerpt "in 2025/en 2025"→"today/hoy" (data + dicts ×2); COMPANY_INFO.founded "2025"→"2026"; trust-section TRUST_METRICS[2] → "Embedded · Self-contained engine — no server, no cloud". Bonus: session metadata "2025-W04"→"2026-W15" en snippet.

### S9 — Verify + cierre ✅
Contrato verificado 2026-08-24:
- `rg "100% Recall" web/src` = 0 matches ✅
- QUICKSTART_PYTHON contiene `import vantadb` (línea 287) y 14 refs get_memory/search_memory; 0 refs `import vantadb_py` en snippets ✅
- site-navbar.tsx: 0 matches "v0.1" ✅
- Disclaimer composite visible en [slug]/page.tsx:67 + list note ✅
- CHANGELOG del sitio: solo versiones 0.5.0/0.4.0 (existen en docs/CHANGELOG.md) ✅
- `npm run build` en web/: ✓ Compiled successfully, exit 0 (35 rutas) ✅
Sin commit (instrucción explícita del orquestador).

## Context Save Point
Tarea COMPLETA. Notas para WDA-04+: navbar.tsx (dead) aún tiene "v0.1 · embedded rust" — desaparece al borrar el componente en WDA-05. metrics-bar.tsx sigue muerto pero ya corregido. code-playground.tsx usa API del TS SDK real (no tocar). El run competitivo histórico de docs/blog/benchmarks_vs_lancedb_chroma.md difiere de §7 — el sitio ahora cita §7 como pidió el orquestador; si MKT quiere publicar el run 2026-08-04 debe regenerarlo en hardware limpio y actualizar competitive-benchmark.json completo.

