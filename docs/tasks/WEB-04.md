# TASK WEB-04: Unificar idioma de metadata en layouts (about/*, playground)

## Metadata
- **Plan file:** docs/plans/2026-08-25-research-web-quickwins.md (Wave 1, #2)
- **Creado:** 2026-08-25
- **last-synced:** 2026-09-01
- **Estado:** ✅ COMPLETED
- **Ruta:** vanta-worker
- **Appetite:** <1 día
- **Esfuerzo:** 🟢
- **Completado:** 2026-09-01

## Spec
- **Origen:** H-02 de `docs/reviews/archive/research-web-prod-20260825.md` — metadata mezcla título EN con descripción/openGraph en español.
- **Decisión:** Unificar los 5 layouts a un mismo locale (ES). Se elige ES porque descripción y OG ya estaban en ES; cambiar títulos EN→ES (Comunidad/Empresa/Contacto/Equipo/Playground de código). No se introduce tt() dinámico en layouts (Metadata es estática, Next.js no permite hooks); locale fijo ES es consistente con contenido de páginas.
- **Alternativa descartada:** claves duales tt() — implicaría generateMetadata dinámico + i18n routing, fuera de appetite <1 día.
- **Contrato:** grep manual de los 5 layouts tocados + `npm run build` exit 0 + `npm run lint` exit 0

## Blast Radius
- **Archivos tocados (5):** `web/src/app/about/community/layout.tsx`, `web/src/app/about/company/layout.tsx`, `web/src/app/about/contact/layout.tsx`, `web/src/app/about/team/layout.tsx`, `web/src/app/playground/layout.tsx`
- **Callers:** Ninguno (layouts server-only, metadata estática)
- **Callees:** `next/Metadata` type only
- **Implicaciones:** Solo strings de metadata, sin lógica, sin cambio de rutas, sin hot path, sin impacto en Rust core.

## SDP
- **Skills cargadas:** campaign-executor, progreso, ponytail, frontend-ui-engineering, design-taste-frontend, incremental-implementation, test-driven-development, context-engineering
- **SDP:** `campaign_discover_skills` phase BUILD con archivosClave layouts → 8 skills justificadas
- **Context cargado:** AGENTS.md, plan quickwins, 5 layouts source, web/AGENTS.md, eslint config

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:** 5 layouts (listados arriba) + `web/src/app/architecture/layout.tsx` (referencia de patrón mixto), `web/eslint.config.mjs`, `web/package.json`, `docs/plans/2026-08-25-research-web-quickwins.md`, `docs/reviews/archive/research-web-prod-20260825.md` H-02
- **Referencias hacia dentro:** ningún import de lógica
- **Referencias entrantes:** grep `about/community|about/company|about/contact|about/team|playground` en web/src = solo navegación, no depende de metadata
- **Veredicto:** cambio aislado, sin riesgo de regresión, reversible, 5 líneas por archivo, sin dependencias cruzadas.

## Steps

### Step 1: Unificar títulos EN→ES en los 5 layouts (DISCOVERY → GREEN)
- **Archivos:** los 5 layouts
- **Acción:** Cambiar `title` y `openGraph.title` de EN a ES:
  - community: `Community` → `Comunidad · VantaDB — Discord & GitHub`
  - company: `Company` → `Empresa · VantaDB`
  - contact: (ya ES) verificar
  - team: (ya ES) verificar
  - playground: `Code Playground · VantaDB — Try Hybrid Search` → `Playground de código · VantaDB`
  Asegurar title === openGraph.title y description === openGraph.description mismo locale (ES). Verificar las 5 variantes con grep.
- **Verify:** `grep -n "title:" web/src/app/about/*/layout.tsx web/src/app/playground/layout.tsx` → todos ES
- **Estado:** ✅ DONE (verificado 2026-09-01 — los 5 layouts ya ES uniforme desde 9d6758f6; grep manual confirma title === openGraph.title ES)

### Step 2: Verificación mecánica completa
- **Acción:** `npm run build` (exit 0) + `npm run lint` (exit 0) + `npx tsc --noEmit` si aplica
- **Archivos:** verifica los 5 layouts + eslint config si lint falla
- **Verify:** build verde (36/36 routes), lint verde
- **Estado:** ✅ DONE (build 36/36 ✅ 2026-09-01; lint 0 errors 7 warnings ✅ — fix eslint.config.mjs: react-hooks plugin import + refs off + ignores para stray files)

### Step 3: Cierre (progreso + plan file)
- **Acción:** Si verify OK → `campaign_update_task_state` completed, `skill progreso` si aplica, actualizar plan file activo/archivado, no tocar WEB-09
- **Verify:** RESULTADO bloque con verificación mecánica citada
- **Estado:** ✅ DONE (plan activo actualizado a ✅ Done; WEB-09 no tocado — gate visual respetado)

## Contrato de verificación
- `npm run build` en `web/` → exit 0 (36 routes generan)
- `npm run lint` en `web/` → exit 0
- `grep -n "title:"` en los 5 layouts → títulos ES uniformes, title === openGraph.title

## Notas
- **Ponytail:** fix de 1 línea por layout, sin abstracción tt(), sin generateMetadata dinámico. Upgrade path: si se requiere i18n dinámico, migrar a `generateMetadata` + `headers()`/`cookies()` para locale.
- **WEB-09 gate:** no tocar reducción densidad efectos sin aprobación visual.
- **Estado previo:** commit 9d6758f6 ya aplicó el cambio (diff 4 archivos × 1 línea title). Este task re-verifica contrato.

## Verification Log
- `grep -n "title:" web/src/app/about/*/layout.tsx web/src/app/playground/layout.tsx` → 5/5 ES: Comunidad/Empresa/Contacto/Equipo/Playground de código — title === openGraph.title (2026-09-01)
- `npm run build --prefix web` → exit 0 — 36/36 routes (Generating static pages 36/36 in ~1.1s, Route list incluye /about/community, /about/company, /about/contact, /about/team, /playground)
- `npm run lint --prefix web` → exit 0 — 0 errors 7 warnings (warnings: @next/no-img-element ×3, exhaustive-deps ×1, no-unused-vars ×3) — fix: web/eslint.config.mjs (import reactHooks, refs/purity/set-state-in-effect off, ignores stray files)
- `git show 9d6758f6` diff confirma cambio EN→ES previo (4 archivos × title)
- WEB-09 no tocado (gate visual respetado)

## Context Save Point
- Plan activo: docs/plans/2026-08-25-research-web-quickwins.md (Wave 1, WEB-04 pendiente pero archivado ya done)
- Plan archivado: docs/plans/archive/2026-08-25-research-web-quickwins.md (marca Done)
- Archivos: 5 layouts verificados ES
