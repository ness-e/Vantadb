---
title: "TASK-WEB-09+MKT-22: Densidad visual del home (WEB-09) + panel de la métrica principal (MKT-22)"
kind: task
description: "Task 36 del master plan 0.9.0 — WEB-09: recontar efectos en HEAD y diferir con criterio+fecha (sin input visual nuevo del owner); MKT-22: publicar el North Star en README con metodología + comando y estado 'primera medición pendiente' (sin datos reales de proxy)"
---

# TASK-WEB-09+MKT-22: Densidad visual del home + panel de la métrica principal

## Metadata
- **Plan file:** docs/dev/plans/2026-10-04-master-plan-0.9.0.md (Task 36 — WEB-09 + MKT-22)
- **Fuente:** Backlog `WEB-09` (`docs/dev/Backlog.md:704`, origen INV-web-01 H-07) + `MKT-22` (`docs/dev/Backlog.md:208`, origen análisis externo 2026-10-01 C5)
- **Esfuerzo:** 🟡 1d · **Prioridad:** 🟡
- **Tipo:** Docs (WEB-09: repo web **read-only** — recount + defer; MKT-22: README + gates docs)
- **Turns estimados:** 10-15
- **Creado:** 2026-10-05T01:33
- **last-synced:** 2026-10-05T02:10
- **Estado:** ✅ COMPLETED (2026-10-05 — review P2-01 approve; WEB-09 defer con criterio + MKT-22 North Star publicado sin números)
- **Incógnitas (uphill):** 0 — resueltas en DISCOVERY (conteo recontado en HEAD; ¿hay store de proxy con sesiones? → **no** → stop condition aplicada)
- **Pendientes (downhill):** 0 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `README.md` (raíz VantaDB) — superficie GitHub/landing; nadie lo importa |
| Callees | `scripts/north_star_metric.py` (referenciado por comando, no se modifica); `vanta-cli mcp-call` (tooling, no se modifica); `docs/api/PROXY.md` §North Star (link target, no se modifica) |
| Implicaciones | Docs-only. Sin cambios de código, contratos, APIs ni CI. Repo web: **0 archivos modificados** (worktree de reconocimiento, retirado al cierre) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `README.md` (§Product Boundary y estructura), `scripts/north_star_metric.py` (224L), `docs/api/PROXY.md:195-274` (§North Star + defaults), web repo (worktree `web-web09` @ main `fd7b41b`): `src/app/page.tsx`, `src/components/vanta/home-view.tsx`, `trust-bar.tsx`, `hero.tsx`, `mark/mark-classic.tsx` (extractos), `src/app/globals.css` (keyframes + reduced-motion), `e2e/web09-screenshots.spec.ts` (existencia), `visual-audit/` (138 archivos); `docs/dev/tasks/WEB-09.md` (iteración previa).
- **Referencias hacia dentro:** `README.md` es doc raíz sin imports. El link nuevo apunta a `docs/api/PROXY.md#north-star-metric-icp-01` (target existe — heading `## North Star metric (ICP-01)`).
- **Referencias salientes:** comando `python scripts/north_star_metric.py` (script existe; `--self-test` ✅ exit 0, Python 3.14.7) y `vanta-cli mcp-call` (subcomando existe en `target/release/vanta-cli.exe`, verificado).
- **Veredicto impacto:** **bajo** — aditivo, docs-only, revertible sin efectos colaterales.

## Contrato

> "(WEB-09) decisión visual del owner aplicada (densidad reducida) o diferida con criterio escrito y fecha; (MKT-22) número de sesiones visible en un panel/reporte actualizado (README o reporte generado por el script), con comando reproducible documentado; sin claims sin evidencia (Regla 11)."

Verificación exacta:
- **WEB-09:** task file con (a) recount en HEAD, (b) defer **con criterio escrito + opciones + fecha de revisión 2026-11-02**, (c) 0 cambios en el repo web. Evidencia: tabla de recount (abajo) + probes git.
- **MKT-22:** `README.md` con sección North Star: definición + comando reproducible + estado **"first measurement pending"** (sin números inventados). Gates: `node scripts/docs/check-links.mjs` ✅ · `node scripts/docs/check-docs.mjs` ✅ · `node scripts/docs/gen-index.mjs --check` ✅ · markdownlint (archivos tocados) ✅ · `pwsh scripts/validate-docs-coverage.ps1` ✅.

## Spec (SDD)

| # | Decisión | Opciones (+tradeoff) | Default | Resuelto |
|---|----------|----------------------|---------|----------|
| 1 | Resultado WEB-09 | A) aplicar reducción adicional sin criterio owner (prohibido — inventaría el criterio) / B) diferir con criterio escrito + fecha (stop condition sancionada del plan) / C) cerrar como "ya aplicada" sin registrar el defer | B | ✅ decidido-por-evidencia: Backlog:704 "requiere input visual owner"; plan Task 36 stop condition "sin criterio del owner → diferir con nota"; atenuación owner 2026-09-02 ya mergeada (`c4e7c57`); rediseño `design/v2` activo (commits 2026-10-02) |
| 2 | Ubicación del panel MKT-22 | A) `README.md` (nombrado en plan y Backlog) / B) `PROXY.md` (ya tiene la metodología completa) / C) reporte generado por el script | A | ✅ decidido-por-evidencia: plan Task 36 "README o reporte"; Backlog:208 key file `README.md` |
| 3 | Número sin datos reales | A) publicar un número (prohibido — inventado) / B) publicar metodología + comando + "pendiente de primera corrida" (stop condition sancionada) | B | ✅ decidido-por-evidencia: 0 stores de proxy con sesiones (5 probes, abajo); plan stop condition "sin datos reales → metodología + comando, número pendiente" |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** sin números/claims sin evidencia reproducible (Regla 11); repo web sin cambios (no tocar `design/v2` ni branches `ts10/*`/`ts13/*`/`show02/*`); no tocar `opencode.jsonc`, master plan, `docs/pipeline-state.json`; commit **LOCAL** (nunca push); staged ajeno fuera del commit (pathspec).
- **Comandos de verificación:** los del Contrato (gates docs + `--self-test`).
- **Deuda pendiente:** WEB-09 → decisión de densidad del home (revisión 2026-11-02 o merge de `design/v2`); MKT-22 → número real pendiente de primera corrida con store de proxy.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda (docs-only; no introduce código ni dependencias).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificado: recount + defer (WEB-09) y README publicado + gates docs (MKT-22) |
| **Commit** | Atómico `docs:`, conventional, `git diff` limpio, pathspec explícito (staged ajeno excluido), verificación mecánica (gates docs) |
| **Release** | n/a — docs-only (justificado: no toca código, semver ni changelog) |

## Herramientas necesarias

- `rg` / `git` (recount web repo, read-only), `vanta-cli mcp-call` (probes de stores), `python` (self-test), `node scripts/docs/*.mjs` (gates), `pwsh dev-tools/ocr-review.ps1`, sub-agente `vanta-review` (P2-01).

**Skills cargadas (SDP):** `frontend-ui-engineering` + `design-taste-frontend` (dominio visual del recount) · `documentation-skill` (OBLIGATORIA: README + task file) · `source-driven-development` (verificación de comandos/API reales antes de documentarlos) · base auto (campaign-executor, progreso, ponytail) · pinned SDP v3: `documentation-and-adrs`, `api-and-interface-design` (registrados; sin delta aplicable en docs-only).

## Investigation Notes

### WEB-09 — Recount en HEAD (main `fd7b41b`, worktree `web-web09`, 2026-10-05)

El conteo del review INV-web-01 H-07 ("73 usos, trust-bar ×11, hero 5 capas") **no se reproduce** en HEAD:

| Elemento | Review (stale) | HEAD `fd7b41b` (medido) |
|----------|----------------|--------------------------|
| `trust-bar.tsx` | ×11 efectos | **1 token animado** (`animate-marquee`, 28s, pausa hover + `prefers-reduced-motion`) + **3 overlays estáticos** (halftone 0.02, speed-lines ×2 w-8/opacity-60) + hover border |
| `hero.tsx` | 5 capas fondo | **3 capas fondo** (grid-tech, halftone 260px @0.12, speed-lines h-20 @0.03) + **1 RegMark** (era 4) + 3 tokens animados (stamp, flicker/bounce con `motion-safe`/`motion-reduce`) + glitch-hover gated |
| Home (11 secciones) | — | **10 tokens `animate-*`** en 7 archivos (hero 3, trust-bar 1, features 1, core-engine 1, code-terminal 2, trust-section 1, home-view 1) + framer-motion en latency-comparator + animejs/SMIL en mark |
| Mark (classic) | — | 4 familias: hover pulse 1.8, ambient 3 nodos, SMIL glow ×1 (opacity 0.14), blink 8s — todas con gating `matchMedia(prefers-reduced-motion)` |
| `globals.css` | — | 10 `@keyframes` + **6 bloques `prefers-reduced-motion`** |
| Site-wide `animate-*` | "73 usos" | **81 tokens** en 49 archivos (incl. `globals.css`; todo el sitio, no solo home) |

**Estado real:** la atenuación decidida por el owner (2026-09-02, filosofía "sutil no cero + A11y primero + slices") **ya está aplicada y mergeada** en main del repo web — commit `c4e7c57 feat(web): WEB-09 refinamiento sutil+A11y home — 15 efectos atenuados con gating reduced-motion`. La iteración previa (`docs/dev/tasks/WEB-09.md`, "PROPUESTA-REFINADA / no commit") quedó superseded por ese commit.

**Criterio del defer (escrito):** (a) no hay criterio visual **nuevo** del owner registrado para una reducción adicional (búsqueda en Backlog/avance/plan: solo "requiere input visual owner; puede quedar diferida"); (b) el estado actual ES la última decisión visual del owner (aplicada); (c) `design/v2` está activo (commits 2026-10-02, sistema Codex) y reemplazará el home — decidir densidad fina del home actual tiene valor acotado; (d) no se inventa criterio.

**Opciones para el owner (cuando revise):**
- **A)** Aceptar el estado actual como decisión final de densidad (el home refleja "sutil no cero" 2026-09-02).
- **B)** Reducción adicional en el home actual — candidatos concretos: quitar halftone/speed-lines del TrustBar, bajar RegMark, quitar speed-lines del hero (requiere OK visual).
- **C) (recomendado)** Diferir al rediseño `design/v2`: la densidad se decide con el sistema Codex sobre el home nuevo.

**Fecha de revisión:** **2026-11-02** (≈4 semanas) o al merge de `design/v2` a `main`, lo que ocurra primero.

### MKT-22 — Store probes (2026-10-05)

La métrica se computa desde el proxy store (`[auth] db_path`). Probes con `vanta-cli mcp-call --tool memory_list --namespace proxy-turns`:

| Store | `proxy-turns` |
|-------|----------------|
| `db/` (raíz VantaDB) | vacío |
| `.vantadb-mcp/` | vacío |
| `~/.vantadb` | vacío |
| `desktop/src-tauri/vantadb_data` | vacío |
| `desktop/src-tauri/vantadb-local` | vacío |

→ **No existe store de proxy con sesiones** (el proxy está frozen, no publicado hasta 1.0.0). Stop condition del plan aplicada: publicar **metodología + comando + número "pendiente de primera corrida"**. `--self-test` ✅ (exit 0) valida la lógica de ventana offline. **Nunca un número inventado** (Regla 11).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | **0** — conteo recontado; datos del proxy verificados (no existen) |
| Pendientes de ejecución (downhill) | **0** — cerrado |
| % completado | 100% (Steps 1-4) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — **no aplica**: docs-only; sin input de usuario, auth, datos, dependencias ni FFI. El link nuevo es a un doc interno existente.
- [x] **PERFORMANCE** — **no aplica**: sin hot paths ni código. (WEB-09 mide efectos visuales; no se modifica ninguno.)

## Steps

### Step 1: MKT-22 — Publicar el panel North Star en `README.md` ✅ COMPLETED
- **Archivos:** `README.md`
- **Acción:** subsección `### North Star` añadida (definición + comando reproducible + estado "first measurement pending" + link a PROXY.md §North Star) y fila Agent (MCP) alineada ("sessions with put + search" + ancla `#north-star`).
- **Verify:** `rg -n "North Star" README.md` → líneas 230/240/242/248; **0 números publicados** (solo estado pendiente). ✅
- **Estado:** ✅ COMPLETED

### Step 2: Gates docs (links · schema · índices · markdownlint · coverage) ✅ COMPLETED
- **Archivos:** `docs/index.md`, `llms.txt` (regenerados)
- **Acción:** los 5 gates corridos.
- **Verify (resultados reales 2026-10-05):**
  - `node scripts/docs/check-links.mjs` → exit 0 (2957 links internos post-regen, 0 roto) ✅
  - `node scripts/docs/check-docs.mjs` → exit 0 (gating all clear) ✅
  - `node scripts/docs/gen-index.mjs --check` → exit 0 tras `--write` (1525 docs; mi task file indexado) ✅ — la regeneración incluye entradas pendientes de cierres previos (DESKTOP-44, PROV-13, SHOW-02) que estaban sin commitear
  - markdownlint (`npx --no-install markdownlint-cli2 README.md docs/dev/tasks/WEB-09-MKT-22.md`) → 0 issues ✅
  - `pwsh scripts/validate-docs-coverage.ps1` → 0 gaps ✅
- **Estado:** ✅ COMPLETED

### Step 3: OCR delegation + Review P2-01 (agente distinto) ✅ COMPLETED
- **Archivos:** —
- **Acción:** `pwsh dev-tools/ocr-review.ps1 -Format json` (0 reviewables — docs-only) + fork a `vanta-review` (sesión independiente `ses_ef56ca926ffee4BouJYX36kNuF`) con diff + evidencia; veredicto ✅ APPROVE (confianza alta), sin Critical/Required.
- **Verify:** OCR sin Critical/High (0 reviewables); veredicto registrado en §Review. ✅
- **Estado:** ✅ COMPLETED

### Step 4: Commit LOCAL + cierre campaign (taskId `36`) + progreso ✅ COMPLETED
- **Archivos:** `README.md`, `docs/dev/tasks/WEB-09-MKT-22.md`, `docs/index.md`, `llms.txt` (commit 1); `docs/dev/Backlog.md`, `docs/dev/avance/activo/web-frontend.md`, `docs/dev/avance/activo/vanta-proxy.md`, `docs/dev/avance/decisiones/wontfix.md` (commit 2)
- **Acción:** `git add` pathspec explícito (staged ajeno `opencode.jsonc` excluido) → 2 commits locales `docs:`; cierre campaign taskId `36` con payload review (P2-01 fresh, reviewer `ses_ef56ca926ffee4BouJYX36kNuF`); progreso (Backlog −2 filas + avance ×2 dominios + wontfix §DEFER).
- **Verify:** commits locales (nunca push); `check-avance-coverage` 1034/1034 (100%) ✅; campaign acepta (reviewBlocked:false).
- **Estado:** ✅ COMPLETED

## Dependencias

- Ninguna bloqueante. Repo web `ness-e/Vantadb-web` accesible (worktree `web-web09` creado y retirado sin cambios).

## Review (GATE — agente distinto, P2-01)

> Tier Fast (docs: `README.md` + `docs/dev/**`) → verify fast mecánico + veredicto registrado.

- **Revisor:** vanta-review (P2-01) — sesión independiente (`ses_ef56ca926ffee4BouJYX36kNuF`), sin participación en la implementación · 2026-10-05
- **Enfoque:** ¿el defer WEB-09 es honesto (criterio+fecha, sin inventar input del owner)?; ¿MKT-22 publica sin números inventados (R11) con comando reproducible?; consistencia README ↔ PROXY.md ↔ script; DoD multi-nivel.
- **Cómo se probó:** diff real leído (README/index/llms/task file). Recount independiente en `..\web-web09` HEAD=main=origin/main=`fd7b41b`: 81 tokens `animate-*` site-wide; trust-bar 1 (`animate-marquee` 28s+hover+reduced-motion); hero 3+RegMark+capas fondo; `globals.css` 10 keyframes/6 reduced-motion; `c4e7c57` ancestro de main (5 archivos) y `git status` web limpio. Probes re-corridos con `vanta-cli mcp-call`: `db/` y `~/.vantadb` → `proxy-turns` vacío. `north_star_metric.py --self-test` → OK (Py 3.14.7). Gates re-corridos: check-links 0 roto · check-docs all clear · gen-index --check 0 · markdownlint 0 · validate-docs-coverage 0 gaps · ocr-review 0 reviewables. Ancla `docs/api/PROXY.md#north-star-metric-icp-01` verificada a mano contra heading PROXY.md:219 (check-links solo advisory para fragmentos).
- **Checklist anti-hábitos tóxicos:** ✓ sin salidas inventadas (reproducidas) · ✓ stop conditions en vez de clarificación ausente · ✓ no declara done sin verificar (IN PROGRESS; Step 4 pendiente) · ✓ fallos no ignorados (gates verdes re-verificados) · ✓ 5 probes + alternativas A/B/C · ✓ citas verificadas (plan:1040/1042, Backlog:208/704, `c4e7c57`) · ✓ sin reintentos en bucle · ✓ sin huérfanos (task file indexado) · ✓ SECURITY n/a justificado (docs-only) · ✓ presupuesto acotado.
- **Veredicto:** ✅ **APPROVE** (confianza alta). Sin Critical/Required. Optional aplicados: (1) pathspec del commit incluye `docs/index.md`+`llms.txt`; (2) caveat CLI `mcp-call` cubierto por el link a PROXY.md. Nits corregidos: "30 días"→≈4 semanas; recount 49 archivos (incl. `globals.css`); check-links 2957 (post-regen).

## Notas

- Este task file es el canónico del Task 36 (2026-10-04). La iteración previa vive en `docs/dev/tasks/WEB-09.md` (histórica; su estado "no commit" quedó superseded por `c4e7c57` del repo web).
- Dos partes en un bloque: **no se separó** `MKT-22.md` — ambas cierran bajo Task 36 sin divergencia de estado (WEB-09 = defer sancionado; MKT-22 = publicación con stop condition).
- Prohibiciones respetadas: `opencode.jsonc`, master plan y `docs/pipeline-state.json` sin tocar; desktop (DESKTOP-44) sin tocar; repo web sin cambios.
