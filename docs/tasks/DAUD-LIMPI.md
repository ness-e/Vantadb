# DAUD-LIMPI — Grupo limpieza desktop post-fix (DAUD-03..08)

> **Plan:** `docs/plans/2026-08-25-batch-desktop-ux-core.md` (Task 3, Wave 2)
> **Estado:** ⏳ IN PROGRESS
> **Esfuerzo:** 🟢 · **Contrato:** `cd desktop && npm run build` exit 0 + `npx vitest run` pasa + grep utilidades muertas = 0 usos confirmado; stash@{0} dropeado (verify diff vs worktree = 0)

## Archivos clave

- `desktop/src/App.css`
- `desktop/src/index.css`
- `desktop/src/components/layout/WorkspaceShell.tsx`
- `desktop/src/components/mark/mark-studio.tsx`
- `desktop/src/components/activity/Timeline.tsx`
- `desktop/DESIGN_DECISIONS.md`

## Impacto mapeado (Regla 0)

**Archivos leídos completos:** App.css (60L), index.css (794L), WorkspaceShell.tsx (1132L), TitleBar.tsx (67L), mark-studio.tsx (116L), Timeline.tsx (81L), EventChip.tsx (42L), activity/logic.ts (139L), App.tsx (63L), DESIGN_DECISIONS.md (42L), package.json, READMEs, plan file.

**Referencias hacia dentro (de lo que se borra/modifica):**
- `button:hover/active` global (App.css:53-59): consumidores desnudos en ~25 inputs/buttons dentro de `<main>` + botones SIN `.press` en modales fuera de `<main>` (HelpPanel 1, Inspector 1, ImportDrop 1, ImportPaste 1 — verificados con rg).
- Utilidades `halftone/grid-tech/speed-lines/speed-lines-radial/halftone-fade/animate-rise`: **0 usos en desktop/src/** (grep 2026-08-25: solo index.css + prototype/index.html, que es referencia estática no-bundled). `@keyframes vanta-rise` SÍ se usa (`.stagger-children > *`, index.css:684-687).
- `:root body` / `.dark body` (index.css:279-288): App.css body unlayered (App.css:20-25) gana por cascade layers y ya voltea con tokens — bg/color redundantes; `font-family`/`font-feature-settings` NO están en App.css (única fuente = index.css).
- Glifo `✎` (WorkspaceShell.tsx:637, botón renombrar namespace): reemplazo por `<Pencil/>` Lucide — import lucide-react ya existe (línea 19).
- Glifos mark-studio.tsx: 5× `◆` (hints idle/loading/empty/error + annoyed): U+25C6 Emoji_Presentation=No → monócromo, identidad linocut → KEEP.
- Glifos Timeline vía logic.ts (OP_META: ✎ ▤ ✕ ▦ ⤓ ⤒ ◈ ✓): todos Emoji_Presentation=No → monócromos → KEEP. logic.ts es React-free a propósito (self-check tsc) — no puede usar componentes Lucide.

**Referencias entrantes:**
- App.css/index.css: importados por App.tsx (`import "./App.css"`) y main.tsx (index.css). Sin imports desde otros módulos.
- WorkspaceShell.tsx: importado por App.tsx. `Pencil` es un export de lucide-react (verificado node_modules:15305).
- DESIGN_DECISIONS.md: documento de decisiones (FIND-26), sin imports; referenciado por convención.

**Veredicto de impacto:** bajo. Cambios cosméticos de scope CSS + 1 reemplazo de glifo + doc. Ningún símbolo público nuevo (excepto §5 del doc de decisiones). Sin hot paths. Sin trust boundary.

## Steps

- [x] **1. DISCOVERY** — lectura directa de 10+ archivos (CodeGraph auto-sync off), greps de utilidades/glifos, verificación stash. Hallazgo DAUD-08: diff stash@{0} vs worktree = 242 archivos ≠ 0 → NO dropear.
- [x] **2. DAUD-03** — App.css: excluir TitleBar del press-effect global vía `[data-tauri-drag-region] button` (root de TitleBar ya tiene el atributo; scoping a `main button` rompería botones desnudos de modales fuera de main — verificado).
- [x] **3. DAUD-04** — index.css: consolidar `:root body` (solo font-family + font-feature-settings) y eliminar `.dark body` (redundante con App.css body vía tokens).
- [x] **4. DAUD-05** — index.css: borrar `.halftone`, `.halftone-fade`, `.speed-lines`, `.speed-lines-radial`, `.grid-tech` + 4 overrides `.dark` (0 usos TSX verificados) + clase `.animate-rise` (0 usos; conservar `@keyframes vanta-rise` usado por stagger-children).
- [x] **5. DAUD-06** — WorkspaceShell.tsx: `✎` → `<Pencil className="h-3.5 w-3.5" strokeWidth={2.5} />`. **Hallazgo extra:** ProxyDashboard.tsx:279 tenía otro `✎ cambiar URL` → también Pencil. Audit mark-studio (5× ◆ → KEEP) y Timeline/logic (glifos monócromos ✎▤✕▦⤓⤒◈✓ → KEEP, módulo React-free) documentado en DESIGN_DECISIONS §5.
- [x] **6. DAUD-07** — DESIGN_DECISIONS.md: agregar §5 Icon convention (Lucide strokeWidth 2.5 para UI funcional; glifos geométricos monocromos como identidad; prohibido emoji-presentation Windows; lucide-react ya instalado ^1.34.0).
- [x] **7. VERIFY** — `cd desktop && npm run build` exit 0 (2×, tras ediciones + ProxyDashboard) · `npx vitest run` 68/68 (2×) · grep utilidades: 0 definiciones/usos en src (solo 2 comentarios históricos) · `git stash list` final: stash@{0} INTACTO (DAUD-08 NO dropeado — reportado).

## Context Save Point

DISCOVERY completo (2026-08-25). DAUD-08: **stash@{0} NO dropeado** — `git diff stash@{0}` = 242 archivos; contiene contenido único no absorbido en HEAD (src/storage/*, wal_sharded.rs, vantadb-mcp/*, vantadb-node/*, vantadb-python/* — 1456+ líneas difieren de HEAD). Requiere decisión del lead (inspeccionar `git stash show -p stash@{0}`). Sin ediciones aún.

## Resultado

- ✅ **COMPLETO (5/6 DAUD)** — DAUD-03/04/05/06/07 implementados y verificados. DAUD-08 **NO ejecutado deliberadamente**: la regla crítica del propio task exige "diff vs worktree = 0" para dropear; el diff real es 242 archivos con contenido único (Rust core/MCP/node). El contrato "stash@{0} dropeado" NO se cumple → se reporta al lead, no se fuerza el drop.
- **Verify:** `cd desktop && npm run build` exit 0 (2×) · `npx vitest run` 11 files / 68 tests passed (2×) · grep `halftone|grid-tech|speed-lines|animate-rise` en src = 0 defs/usos · grep `✎` en TSX = 0 usos funcionales (solo 2 comentarios).
- **Archivos tocados (5):** `desktop/src/App.css` (+15/-0), `desktop/src/index.css` (-88 neto), `desktop/src/components/layout/WorkspaceShell.tsx` (+4/-2), `desktop/src/components/proxy/ProxyDashboard.tsx` (+4/-1), `desktop/DESIGN_DECISIONS.md` (+21).
- **NO COMMIT** (regla: el lead verifica mecánico y commitea por tarea). Worktree tiene cambios de otros agentes (AGT-04.md, lessons.md, assets borrados, docs/reviews) — el commit del lead debe incluir SOLO los 5 archivos + task file.