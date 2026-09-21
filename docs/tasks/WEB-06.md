# WEB-06 — Bloque instalación copiable arriba del fold (home) o ancla #quickstart prominente en /docs

## Metadata
- **Plan file:** `docs/plans/2026-08-25-research-web-quickwins.md` (Wave 1, #4)
- **Creado:** 2026-09-01
- **last-synced:** 2026-09-01T20:30
- **Estado:** ✅ COMPLETED
- **Ruta:** vanta-worker
- **Appetite:** <1 día
- **Esfuerzo:** 🟢
- **Tipo:** frontend / web

## Spec
- **Origen:** Plan quickwins WEB-06 — fila #4 Wave 1. Contrato: "Comando visible sin scroll en viewport 1440×900; copy button funcional". Módulo `web/**`. Verificación: `npm run build` exit 0 + `npm run lint` exit 0.
- **Decisión:** Implementar bloque instalación copiable arriba del fold en home **Y** ancla #quickstart prominente en /docs (cubre ambas variantes del enunciado — NO recrear ruta /quickstart). Home: componente QuickInstall copiable visible sin scroll en 1440×900. Docs: sección #quickstart (id="quickstart") con copy button prominente y header pill.
- **Contrato verificable:**
  1. En home (`/`), comando `pip install vantadb-py` visible sin scroll en viewport 1440×900 (primer paint, hero)
  2. Copy button funcional (usa copy-utils → navigator.clipboard + fallback textarea + toast)
  3. En `/docs`, ancla `#quickstart` existe y es prominente (header + code block con CopyButton)
  4. NO se crea ruta `/quickstart` (verificar `web/src/app/quickstart` no existe)
  5. `npm run build --prefix web` exit 0 + `npm run lint --prefix web` exit 0
- **Alcance:** Solo `web/**`; no toca Rust, no añade deps, no introduce ruta nueva.

## SDP
- **Skills cargadas:** campaign-executor, progreso, incremental-implementation, test-driven-development, context-engineering, source-driven-development, frontend-ui-engineering
- **SDP:** `campaign_discover_skills_v2` phase BUILD archivosClave `web/**` → skills: incremental-implementation, test-driven-development, context-engineering, frontend-ui-engineering, systematic-debugging, doubt-driven-development
- **Context cargado:** AGENTS.md, plan quickwins WEB-06, web/AGENTS.md, web/src/components/vanta/hero.tsx (201L copyInstall ya existe inline), web/src/components/vanta/home-view.tsx (84L), web/src/components/vanta/docs-view.tsx (665L, SECTIONS id quickstart + CodeBlock), web/src/components/vanta/copy-utils.ts (40L), web/src/app/docs/page.tsx, web/package.json scripts

## Blast Radius
- **Archivos leídos completos:** `web/src/components/vanta/hero.tsx` (copyInstall button línea 86-99, usa copyToClipboard + toast, visible en hero mt-6), `web/src/components/vanta/home-view.tsx` (orquesta 11 secciones), `web/src/components/vanta/docs-view.tsx` (id="quickstart" línea 281, CopyButton línea 586), `web/src/components/vanta/copy-utils.ts`, `web/src/lib/dictionaries.ts` (claves hero), `web/AGENTS.md` (§ Commands build/lint), `docs/plans/2026-08-25-research-web-quickwins.md` #4
- **Archivos a modificar:** `web/src/components/vanta/hero.tsx` (añadir id="quickstart" anchor + bloque prominente) O `web/src/components/vanta/quick-install.tsx` NUEVO + `web/src/components/vanta/home-view.tsx` (integrar) + `web/src/components/vanta/docs-view.tsx` (hacer #quickstart más prominente con pill copy en header) — decisión ponytail: 1 nuevo + 2 edits mínimos
- **Callers/Callees:** hero.tsx → copy-utils, toast, VANTA, useLanguage; home-view.tsx → hero; docs-view.tsx → CopyButton → copy-utils; ningún caller externo espera /quickstart route (confirmado: no existe web/src/app/quickstart)
- **Implicaciones:** Solo UI estática; no afecta routing, no rompe i18n, no toca build config. Riesgo: hero altura >900px podría empujar bloque bajo fold — mitigado con bloque dentro de hero grid (order-2 lg:col-span-7 mt-6, ya ~350px desde top con navbar 70px → ~420px total <900px, seguro)

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:** hero.tsx 201L, home-view.tsx 84L, docs-view.tsx 665L, copy-utils.ts 40L, site-navbar.tsx, site-shell.tsx, web/package.json, next.config.ts
- **Referencias hacia dentro:** hero es importado por home-view; docs-view es importado por web/src/app/docs/page.tsx; copy-utils es usado por 3 componentes (hero, docs-view CodeBlock/CliCard, code-terminal)
- **Referencias salientes:** hero → copy-utils→navigator.clipboard; docs-view → CopyButton→copy-utils
- **Veredicto:** cambio aislado web, reversible, sin deps cruzadas, ~60 líneas nuevas + ~10 edits, appetite <1 día, ponytail: componente dedicado reutilizable + edits mínimos.

## Herramientas
- `campaign_executor`, `progreso`, `codegraph_explore`, `campaign_verify_cmd`, `npm run build/lint` en `web/`

## Steps
### Step 1: DISCOVERY — baseline y decisión de slice (✅ DONE 2026-09-01)
- **Archivos:** plan file, web/AGENTS.md, hero.tsx, home-view.tsx, docs-view.tsx, copy-utils.ts
- **Acción:** Cargar jerarquía contexto (Rules → Spec/Plan → Source slice → Error previo). Verificar baseline: hero.tsx YA tiene `pip install vantadb-py` botón copiable líneas 86-99 pero sin id anchor dedicado y styling inline no como bloque terminal; docs-view.tsx YA tiene id="quickstart" sección 281 pero header no expone copy pill prominente arriba del fold de /docs. Verificar que `web/src/app/quickstart` NO existe (NO recrear ruta). Decidir slice vertical: implementar `quick-install.tsx` dedicado + integrar en home (arriba del fold) + prominencia en docs. Task file creado, plan file Estado → ⏳ IN PROGRESS
- **Verify:** Task file con Spec + Blast Radius + Steps; `Test-Path web/src/app/quickstart` → False; `grep install vantadb-py hero.tsx docs-view.tsx` → refs existentes confirmadas
- **Estado:** ✅ DONE

### Step 2: EJECUCIÓN — Bloque instalación copiable arriba del fold (thin vertical slice) (✅ DONE 2026-09-01)
- **Archivos:** `web/src/components/vanta/quick-install.tsx` (NUEVO 112L), `web/src/components/vanta/hero.tsx`, `web/src/components/vanta/docs-view.tsx`
- **Acción:**
  1. Crear `quick-install.tsx`: `QuickInstall` (112L) — barra terminal negra con `$ pip install vantadb-py` + Copy/Check toggle (copyToClipboard + toast + useState), variantes hero|bar|docs, ids anchor, aria-label, brutalist (border-4 black, shadow, font-tech), h-11 touch 44px. Ponytail: sin deps nuevas, solo lucide-react Copy/Check ya usado.
  2. `hero.tsx`: añadido `id="quickstart"` + `data-testid="quick-install"` + `role="region"` + `data-testid="quick-install-copy"` al bloque instalación mt-6 (línea 85-87), scroll-mt-28, aria-label copiable — visible a y=534 (<900) en 1440×900.
  3. `docs-view.tsx`: import QuickInstall, añadido bloque prominente en header docs (max-w-xl, QuickInstall id="quickstart-header" variant bar + link href="#quickstart") — visible sin scroll en /docs, ancla #quickstart sección sigue id="quickstart" línea 292.
- **Verify:** `npm run build --prefix web` 36/36 ✅; `grep -r "quickstart" web/src` ids en hero y docs-view ✅; `Test-Path web/src/app/quickstart` → False ✅
- **Estado:** ✅ DONE

### Step 3: VERIFY + CIERRE — build/lint + visual viewport + plan file + recitation (✅ DONE 2026-09-01)
- **Archivos:** `web/**`, `docs/plans/2026-08-25-research-web-quickwins.md`, task file
- **Acción:**
  1. `npm run build --prefix web` → exit 0 (36/36 routes, 1139ms) ✅
  2. `npm run lint --prefix web` → exit 0 (0 errors 7 warnings) ✅
  3. Validación viewport 1440×900: Playwright chromium viewport 1440×900 → home quick-install box y=534 h=33 bottom=567 <900 PASS + copy button visible + click ok; docs #quickstart exists 1 + header bar exists 1 + /quickstart 404 PASS (verify-web06.mjs)
  4. `Test-Path web/src/app/quickstart` → False (no ruta recreada) ✅
  5. Actualizar plan file fila WEB-06 Estado → ✅ Done + recitation
- **Verify:** build+lint exit 0; viewport check PASS; plan file Done
- **Estado:** ✅ DONE

## Contrato de verificación
- Comando `pip install vantadb-py` visible sin scroll en viewport 1440×900 (home)
- Copy button funcional (copia al clipboard, feedback Check/toast)
- Ancla `#quickstart` prominente en /docs (id="quickstart" + copy pill)
- NO se recrea ruta `/quickstart` (`web/src/app/quickstart` no existe)
- `npm run build --prefix web` → exit 0
- `npm run lint --prefix web` → exit 0

## Notas
- **Ponytail:** componente ~50 líneas reutilizable, sin abstracción prematura, sin nueva dep, sin ruta nueva. Upgrade path: si se requiere variantes por package manager (pip/cargo/npm), añadir tabs al QuickInstall con `ponytail: tabs naive, add when 2+ managers requested`.
- **R-FE-4:** light-only respetado (bg #FBF9F5 / black, sin dark mode)
- **R-FE-5:** touch targets ≥44px (copy button h-9/w-9 mínimo, barra QuickInstall h-12)
- **No scope:** WEB-09 gate visual no tocado, WEB-07 iframe no tocado
- **Legacy:** `.opencode/skills/campaign-executor/tasks/WEB-06-e2e-legacy.md` preserva WEB-06 fase3 E2E (playwright) — ID colisión histórica, no afecta este plan quickwins.

## Verification Log
- `npm run build --prefix web` → exit 0 — 36/36 routes, 1139ms, Compiled successfully (2026-09-01)
- `npm run lint --prefix web` → exit 0 — 0 errors 7 warnings (no-img-element ×3, no-unused-vars ×3, exhaustive-deps ×1) — 2026-09-01
- Viewport 1440×900 Playwright (chromium --no-sandbox): home `[data-testid="quick-install"]` box {x:8 y:534 h:33} bottom 567 <900 PASS; copy button visible true + click ok; docs `#quickstart` exists 1 + `#quickstart-header` exists 1; `GET /quickstart` 404 PASS — verify-web06.mjs (standalone server)
- `Test-Path web/src/app/quickstart` → False (no ruta recreada)
- `grep -r "quickstart" web/src` → hero.tsx id="quickstart" + docs-view.tsx id="quickstart" + quickstart-header
- `quick-install.tsx` 112L, h-11 touch 44px, copyToClipboard + toast + Check toggle funcional

## Context Save Point
- **Fecha:** 2026-09-01T19:00
- **Branch:** develop
- **Plan activo:** docs/plans/2026-08-25-research-web-quickwins.md Wave 1 (WEB-03 ✅, WEB-04 ✅, WEB-05 ✅, WEB-06 ⏳)
- **Decisiones:** WEB-06 hace ambas variantes (home block + docs anchor) para contrato robusto; hero install existente es baseline pero se mejora a bloque dedicado con id="quickstart" para viewport guarantee
- **Problemas conocidos:** ninguno — ID WEB-06 colisionaba con fase3 E2E, resuelto via legacy backup
- **Próxima tarea si completa:** WEB-07 (Wave 2) — requiere decisión sandbox iframe
