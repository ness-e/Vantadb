# WDA-04 — F4 UI (Toaster muerto · contrastes · touch targets · a11y fina · interactivos)

> Plan: `docs/plans/2026-08-19-web-design-audit.md` Task 5 §6. Tracking manual (MCP corrupto — no usar campaign_update_task_state).

## Estado: ✅ COMPLETED (2026-08-24)

## Impacto mapeado (Regla 0)

**Archivos leídos completos:** layout.tsx, site-navbar.tsx, back-to-top.tsx, easter-egg.tsx,
mark/mark-classic.tsx, command-palette.tsx, tutorial-modal.tsx, code-playground.tsx,
competitive-table.tsx, lang-toggle.tsx, page-header.tsx, home-view.tsx (imports),
about/team/page.tsx, footer.tsx, latency-comparator.tsx (190-320), ui/sonner.tsx,
rules/frontend-web.md.

**Referencias hacia dentro:** `ui/sonner.tsx` solo lo importa layout.tsx;
`ui/toaster.tsx` solo layout.tsx (quedó huérfano, se limpia en WDA-05 con ui/ muerta).
CommandKHint = dead export (0 imports). HomeView monta Hero→CtaFinal (11 secciones)
+ SiteShell(navbar/palette/backtotop/easteregg) + Footer.

**Veredicto:** fixes quirúrgicos en 17 archivos live de web/src. Sin API, sin deps.

## Steps

### Step 1 ✅ Toaster muerto
- `ui/sonner.tsx`: `export { Toaster as Sonner }`; layout importa `{ Sonner }`
- layout.tsx: import shadcn `<Toaster/>` eliminado
- **Verificado:** `rg -c "Toaster" web/src/app/layout.tsx` = 0

### Step 2 ✅ Contrastes (baseline home ×19 + team ×5)
- `text-black/40` (2.78:1) y `/50` (3.86:1) → `/70` (8.3:1) vía regex `(?<!:)text-black/(30|40|50)` — placeholders intactos (::placeholder no evaluado por axe).
- Archivos: hero, features, core-engine, latency-comparator, tutorials-section (+icon /30), faq-section, trust-section, site-navbar, command-palette, easter-egg, competitive-table, tutorial-modal, about/team/page.
- Team ×5 contabilizados: role labels neón 10px bold sobre paper (2.75:1) ×4 tarjetas → ink + card "No GitHub" /40 ×1 → /70.
- **Verificado:** grep residual `(30|40|50)` en archivos live de home/team/shell = 0.

### Step 3 ✅ Touch targets R-FE-5
- tutorial-modal step-progress buttons (6px alto) → hit-area ~26px vía `before:-inset-y-[10px]`.
- **Verificado:** `rg "w-\[14px\]|h-\[14px\]"` web/src = 0; `size-3\b` live = 0. Botones secundarios ≥28px (close h-7/h-8); navbar search/hamburger 36px (>24 absoluto).

### Step 4 ✅ A11y fina
- mark-classic: wrapper `role="button"` + tabIndex + Enter/Space → blink teclado-accesible; svg grafo y svg mark ahora `aria-hidden` (nombre en el wrapper).
- i18n aria-labels (solo claves existentes): back-to-top `t("backToTop")`, easter-egg/tutorial-modal close `tt("common.close")`, palette input `t("common.search")`. Sin clave → ES (coherente default lang).
- label-content-name-mismatch: logo navbar sin aria-label (nombre = texto visible "VantaDB…"); GitHub link icon-only sm–md con `aria-label="GitHub"`.
- heading-order home: footer h2→h4 salto → footer `h3`.

### Step 5 ✅ Interactivos
- Palette: `role="listbox"` + items `role="option"`/`aria-selected` (Escape/focus-trap/arrows ya existían).
- Tutorial copy button: `focus-visible:opacity-100` (antes invisible por teclado).
- Playground: comentario self-XSS ampliado (`new Function` self-XSS only; try/catch visible ya existía :327-331 — no sandbox iframe, YAGNI).
- Focus visible global ya cubierto (globals.css:327 outline 3px neon).

### Step 6 ✅ Responsive spot-check (solo código)
- competitive-table: `overflow-x-auto` + `min-w-[900px]` ya presente ✅ (vs-table dead).
- Navbar dropdowns desktop `hidden lg:flex` → inactivos a 390px; drawer mobile sin absolutos ✅. Sin fix necesario.

## Verify (contrato) — RESULTADOS
1. `rg -c "Toaster" web/src/app/layout.tsx` → **0** ✅
2. Icon-buttons <24px: greps `w-[14px]|h-[14px]` y `size-3\b` live → **0** ✅
3. `npm run build` → **exit 0**, 36 rutas, TS limpio ✅

## Context Save Point
- Tarea completa. Sin commit (instrucción del orquestador). Archivos tocados (17):
  app/layout.tsx, app/about/team/page.tsx, components/ui/sonner.tsx, components/vanta/
  {back-to-top, code-playground, command-palette, competitive-table, core-engine,
  easter-egg, faq-section, features, footer, hero, latency-comparator,
  mark/mark-classic, site-navbar, trust-section, tutorial-modal, tutorials-section}.
- Nota: ui/toaster.tsx queda huérfano → eliminar en WDA-05 (carpeta ui/ muerta).
- Preexistente NO tocar: cambios dirty de desktop/, docs/, completions/ de otras sesiones.
