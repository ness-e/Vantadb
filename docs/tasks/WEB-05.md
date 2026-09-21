# WEB-05 — Re-medir Lighthouse `/` y ruta interna post-WDA-05; actualizar registro perf

## Metadata
- **Plan file:** `docs/plans/2026-08-25-research-web-quickwins.md` (Wave 1, #3)
- **Creado:** 2026-09-01
- **last-synced:** 2026-09-01T18:00
- **Estado:** ✅ COMPLETED
- **Ruta:** vanta-worker
- **Appetite:** <1 día
- **Esfuerzo:** 🟢
- **Tipo:** research / docs-sync (no lógica Rust)

## Spec
- **Origen:** Plan quickwins WEB-05 — fila #3 Wave 1. Contrato: "Números nuevos citados con comando + fecha en ambos docs; si EPERM persiste, documentar workaround probado". Módulo `web/**`.
- **Decisión:** Re-medir Lighthouse Performance/Accessibility/Best-Practices/SEO para `/` y una ruta interna (elegida: `/docs` — ruta estática representativa, o `/playground` si /docs no disponible; preferencia `/docs` porque es contenido markdown pesado y valida LCP real). Tras medición, actualizar dos docs: `web/AGENTS.md` § Verificación Lighthouse + `.opencode/references/research-modules.md` fila `web` (nota perf 95-96 → números nuevos + comando + fecha).
- **Comando canónico (web/AGENTS.md):**
  ```sh
  cd web && npx lighthouse http://localhost:3000 --output=json --output-path=./lighthouse-report.json --chrome-flags="--no-sandbox --headless --disable-gpu" --only-categories=performance,accessibility,best-practices,seo
  cd web && npx lighthouse http://localhost:3000/docs --output=json --output-path=./lighthouse-report-docs.json --chrome-flags="--no-sandbox --headless --disable-gpu" --only-categories=performance,accessibility,best-practices,seo
  ```
  Workaround EPERM: contra producción `https://vantadb.vercel.app` o máquina sin contenedor; si persiste, documentar entorno + error + workaround probado.
- **Contrato verificable:**
  1. `web/AGENTS.md` contiene números Lighthouse nuevos con comando + fecha (no claim stale 95-96)
  2. `.opencode/references/research-modules.md` fila web actualizada con mismos números + fecha
  3. `npm run build --prefix web` exit 0 + `npm run lint --prefix web` exit 0 (contrato módulo)
  4. Si EPERM persiste: ambos docs documentan workaround probado + entorno + error
- **Alcance:** Solo docs + medición; no toca lógica web, no añade deps, no modifica componentes.

## SDP
- **Skills cargadas:** campaign-executor, progreso, incremental-implementation, test-driven-development, context-engineering, source-driven-development, doubt-driven-development, frontend-ui-engineering, api-and-interface-design, browser-testing-with-devtools
- **SDP:** `campaign_discover_skills_v2` phase BUILD archivosClave `web/**` → 10 skills (ver comando)
- **Context cargado:** AGENTS.md, plan quickwins WEB-05, web/AGENTS.md § Verificación Lighthouse, .opencode/references/research-modules.md fila web, docs/operations/BENCHMARKS.md (Regla 11), web/package.json + next.config.ts, web/src/app rutas

## Blast Radius
- **Archivos leídos completos:** `web/AGENTS.md` (60L), `.opencode/references/research-modules.md` fila web, `docs/plans/2026-08-25-research-web-quickwins.md` #3, `web/package.json`, `web/next.config.ts`, `web/src/app/page.tsx` (home), `web/src/app/docs/page.tsx` (ruta interna candidata)
- **Archivos a modificar:** `web/AGENTS.md`, `.opencode/references/research-modules.md` (+ opcional `docs/operations/BENCHMARKS.md` si se cita, pero contrato pide solo esos dos)
- **Callers/Callees:** Ninguno (docs estáticos, no código). No hay imports que dependan de estos md.
- **Implicaciones:** Solo documentación; no afecta build runtime. Riesgo: EPERM ambiental (Chrome sandbox en contenedor) — mitigado con workaround documentado + medición contra producción si aplica.

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:** `web/AGENTS.md` (60L, § Verificación Lighthouse con claim stale 95-96), `.opencode/references/research-modules.md` (fila web perf 95-96 WDA-00), `docs/plans/2026-08-25-research-web-quickwins.md` fila WEB-05, `web/package.json` (scripts build/lint), `web/next.config.ts` (standalone, turbopack), `docs/operations/BENCHMARKS.md` (Regla 11)
- **Referencias hacia dentro:** `web/AGENTS.md` es referenciado por `docs/avance/activo/web-frontend.md` (cruce) y por devs; `research-modules.md` es fuente canónica para `/research web` (prompts/research-module.md)
- **Referencias salientes:** ninguno (md no importa código)
- **Veredicto:** cambio aislado docs, reversible, sin deps cruzadas, ~20-30 líneas por archivo, appetite <1 día, ponytail: medición + edición md mínima.

## Herramientas
- `campaign_executor`, `progreso`, `codegraph_explore`, `campaign_verify_cmd`, `web` lighthouse (npx lighthouse), `npm run build/lint` en `web/`

## Steps

### Step 1: DISCOVERY — Preparar medición y baseline (✅ DONE)
- **Archivos:** plan file, web/AGENTS.md, research-modules.md, web/package.json, next.config.ts
- **Acción:** Cargar jerarquía contexto (Rules → Spec/Plan → Source slice → Error previo). Verificar baseline: claim stale perf 95-96 WDA-05 sin medición post lazy command-palette. Elegir ruta interna `/docs` (fallback `/playground` o `/why-vantadb` si /docs no estática). Verificar `web/` builda: `npm run build --prefix web` dry-check. Confirmar comando lighthouse canónico y workaround EPERM (prod URL).
- **Verify:** Task file creado con Spec + Impacto mapeado + Steps atómicos; plan file Estado → ⏳ IN PROGRESS (recitation)
- **Estado:** ✅ DONE (2026-09-01 — baseline 95-96 stale, ruta interna /docs elegida, build 36/36 verificado)

### Step 2: EJECUCIÓN — Re-medir Lighthouse `/` y `/docs` (thin slice)
- **Archivos:** ninguno tocado aún (medición)
- **Acción:**
  1. `npm run build --prefix web` (exit 0, 36/36 routes) — prerequisito para serve local
  2. Intentar `npx lighthouse http://localhost:3000 --output=json ...` para `/` y `/docs` (o `/why-vantadb`). Si requiere server, `npm run start` o `npx serve` en bg; si EPERM (Chrome sandbox, EACCES), capturar error y probar workaround: `npx lighthouse https://vantadb.vercel.app --output=json ...` o documentar entorno sin contenedor.
  3. Guardar `web/lighthouse-report.json` y `web/lighthouse-report-docs.json` (gitignored) o anotar números en memoria si no se persiste.
  4. Capturar Performance/Accessibility/Best-Practices/SEO scores + fecha + comando + entorno (OS, Chrome version, Node)
- **Verify:** Números capturados o EPERM documentado con workaround probado (log de comando + error + alternativa)
- **Estado:** ✅ DONE (2026-09-01 — `node .next/standalone/server.js` en bg → `npx lighthouse http://localhost:3000 --output=json --chrome-flags="--no-sandbox --headless --disable-gpu" --only-categories=performance,accessibility,best-practices,seo` → `/` perf99/a11y96/bp96/seo100 LCP1800 FCP1110 TBT0 CLS0; `/docs` perf98/a11y94/bp96/seo100 LCP1680; lighthouse 13.4.1, Chrome152, reportes en web/lighthouse-report*.json)

### Step 3: Actualizar registro perf en ambos docs (thin slice)
- **Archivos:** `web/AGENTS.md`, `.opencode/references/research-modules.md`
- **Acción:**
  - `web/AGENTS.md` § Verificación Lighthouse: reemplazar claim stale 95-96 por tabla nueva:
    ```
    | Ruta | Performance | Accessibility | Best Practices | SEO | Fecha | Comando | Entorno |
    | `/` | XX | XX | XX | XX | YYYY-MM-DD | `npx lighthouse ...` | ... |
    | `/docs` | XX | XX | XX | XX | ... |
    ```
    Si EPERM: añadir bloque "Workaround EPERM probado: ..." con error + alternativa (prod URL o máquina bare-metal).
  - `.opencode/references/research-modules.md` fila `web`: actualizar nota "CWV baseline perf 95-96 (WDA-00, re-medir: WEB-05)" → "CWV baseline perf XX/XX (WEB-05 2026-09-01, comando: ...)" con mismos números.
- **Verify:** `grep -n "Lighthouse\|perf" web/AGENTS.md .opencode/references/research-modules.md` → números nuevos + comando + fecha presentes en ambos
- **Estado:** ✅ DONE (2026-09-01 — web/AGENTS.md § Verificación Lighthouse con tabla 99/98 + fecha + comando + entorno + workaround EPERM no reprodujo; research-modules.md fila web actualizada a perf 99/98 2026-09-01)

### Step 4: VERIFY + CIERRE — build/lint + plan file + recitation
- **Archivos:** `web/AGENTS.md`, `.opencode/references/research-modules.md`, `docs/plans/2026-08-25-research-web-quickwins.md`
- **Acción:**
  1. `npm run build --prefix web` → exit 0 (36/36)
  2. `npm run lint --prefix web` → exit 0
  3. Actualizar plan file fila WEB-05 Estado → ✅ Done (números + fecha)
  4. `campaign_update_task_state` completed + recitation (contrato + verificación + invariantes)
  5. `skill progreso` si aplica (Trigger 1 no aplica hasta Wave1 completa, pero registrar)
- **Verify:** `campaign_verify_cmd` build + lint pasan; plan file recitation OK; RESULTADO bloque
- **Estado:** ✅ DONE (2026-09-01 — build 36/36 exit0, lint 0 errors 7 warnings exit0, plan file Done)

## Contrato de verificación
- `web/AGENTS.md` contiene números Lighthouse nuevos (Perf/A11y/BP/SEO) para `/` y ruta interna, con comando + fecha + entorno; si EPERM, documenta workaround probado
- `.opencode/references/research-modules.md` fila web actualizada con mismos números + comando + fecha
- `npm run build --prefix web` → exit 0
- `npm run lint --prefix web` → exit 0

## Notas
- **Ponytail:** medición mínima (2 rutas × 4 categorías) + edición md mínima. No se introduce script nuevo ni dep lighthouse persistida si no aporta; upgrade path: automatizar lighthouse en CI con `lhci` si números muestran regresión.
- **EPERM:** ambient persistent desde WDA-05 (command-palette lazy). Workaround probado primero: prod URL `https://vantadb.vercel.app` (si 404/privada, medir local con `--chrome-flags="--no-sandbox --headless --disable-gpu"`). Si ambos fallan, documentar error EPERM + entorno + nota "re-medir en bare-metal".
- **Ruta interna:** preferencia `/docs` (estática, 60+ md files), fallback `/why-vantadb` (comparativa) o `/playground` (client heavy) — elegir la que build reporte como estática.

## Verification Log
- `npm run build --prefix web` → exit 0 — 36/36 routes (Generating static pages 36/36 in 1085ms, 2026-09-01)
- `npm run lint --prefix web` → exit 0 — 0 errors 7 warnings (no-img-element ×3, no-unused-vars ×3, exhaustive-deps ×1) — 2026-09-01
- Lighthouse `/` (2026-09-01, lighthouse 13.4.1, Chrome 152.0.7977.64, Node v26.8.1, Windows 11 26100, `node .next/standalone/server.js` + `npx lighthouse http://localhost:3000 --output=json --chrome-flags="--no-sandbox --headless --disable-gpu" --only-categories=performance,accessibility,best-practices,seo`): perf **99** (0.99), a11y 96, bp 96, seo 100, FCP 1110ms, LCP 1800ms, CLS 0, TBT 0ms, SI 2545ms → `web/lighthouse-report.json`
- Lighthouse `/docs` (mismo entorno/comando, `http://localhost:3000/docs` → `web/lighthouse-report-docs.json`): perf **98** (0.98), a11y 94, bp 96, seo 100, FCP 1110ms, LCP 1680ms, CLS 0, TBT 0ms, SI 3998ms
- `grep -n "perf" web/AGENTS.md` → "perf 99 (`/`) / 98 (`/docs`) re-medido WEB-05 2026-09-01" presente
- `grep -n "perf" .opencode/references/research-modules.md` → "perf 99 (`/`) / 98 (`/docs`) re-medido WEB-05 2026-09-01" presente en fila web
- EPERM workaround probado 2026-09-01: EPERM no reprodujo con `--no-sandbox --headless --disable-gpu` en bare-metal Windows 11; fallback prod `https://vantadb.vercel.app` verificado 200 OK (Invoke-WebRequest) listo si CI contenedor vuelve a dar EPERM
- WEB-09 no tocado (gate visual respetado)

## Context Save Point
- **Fecha:** 2026-09-01T18:00
- **Branch:** develop
- **Plan activo:** docs/plans/2026-08-25-research-web-quickwins.md Wave 1 (WEB-03 ✅, WEB-04 ✅, WEB-05 ✅, WEB-06 ⬜)
- **Decisiones:** ruta interna = /docs (estática, LCP 1680 vs / LCP 1800); comando lighthouse canónico con --chrome-flags no-sandbox; EPERM no reprodujo bare-metal, fallback prod verificado
- **Problemas conocidos:** ninguno — perf mejora 99/98 vs stale 95-96 confirma lazy command-palette sin regresión
- **Próxima tarea si completa:** WEB-06
