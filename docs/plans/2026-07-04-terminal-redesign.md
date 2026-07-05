# Terminal Technical Redesign

**Goal:** Rediseño completo del homepage con aesthetic terminal (Code Brutalism) + corrección de violaciones vs DESIGN.md

**Architecture:** Capas: (1) Global CSS terminal → (2) DESIGN.md actualizado → (3) Secciones individuales

**Tech Stack:** React + Vite + CSS Modules + GSAP (existente) + Canvas (HeroBackground)

---

## Tasks

### Task 1: Actualizar DESIGN.md

**Files:**
- Modify: `docs/web/DESIGN.md`

**Cambios:**
- Nuevo estilo: **Terminal Technical** — Amber on Near-Black + Code Brutalism
- Agregar regla: scanline overlay, `$` prompts, ASCII decorations
- Eliminar Anti-Slop Rule #17 (partículas permitidas en hero)
- Actualizar section 7.2 Hero: agregar terminal window + particle background
- Actualizar section 7.4 Features: grid bento (reemplaza acordeón)
- Actualizar section 7.5 Quickstart: light background (corrige violación)
- Actualizar section 7.6 Architecture: pipeline horizontal 6 etapas
- Agregar sección 10: Terminal Design Tokens (scanline, prompt, ascii)

---

### Task 2: CSS Global — Terminal Overlay + Prompts

**Files:**
- Create: `web/src/styles/terminal.css`
- Modify: `web/src/styles/index.css` (importar terminal.css)

**Elementos:**
- `.terminal-scanline` — fixed overlay, repeating linear-gradient 2px, opacity 0.03, pointer-events none
- `.terminal-prompt` — `$` prefijo con color amber, font-mono
- `.terminal-ascii-border` — bordes con `┌─┐│└─┘` usando CSS pseudo-elements
- `.terminal-section-divider` — separador `──╌╌──` con amber
- Respetar `prefers-reduced-motion`: desactivar scanline

---

### Task 3: Nav — Terminal Prompt

**Files:**
- Modify: `web/src/components/SwissNav.tsx`
- Modify: `web/src/styles/nav.css`

**Cambios:**
- Agregar `$` prompt antes del logo o como decoración en la nav
- Estilo terminal en el label

---

### Task 4: Hero — Terminal Window + Polish

**Files:**
- Modify: `web/src/components/SwissHero.tsx`
- Modify: `web/src/styles/swiss-hero.css`

**Cambios:**
- Agregar terminal window decorativo en right side (inline CSS grid)
- Mostrar líneas de código VantaDB (estáticas, sin animación)
- Mantener particle background + SVG grid
- Ajustar layout responsive

---

### Task 5: CoreEngine — Bento Grid (reemplazar acordeón)

**Files:**
- Modify: `web/src/components/SwissCoreEngine.tsx`
- Modify: `web/src/styles/swiss-core-engine.css`

**Cambios:**
- Reemplazar acordeón por grid de 6 cards con tamaños variados
- Layout: Hybrid Search (2×1), Rust Core (1×1), WAL (1×1), Zero-Copy (1×1), PyO3 (1×1), No Server (3×1 bottom)
- Mantener GSAP scroll reveal pero sin accordion
- Cada card: icon SVG monoline + título + descripción corta + métrica

---

### Task 6: Quickstart — Light Background

**Files:**
- Modify: `web/src/styles/swiss-quickstart.css`
- Modify: `web/src/styles/quickstart.css` (unificar estilos light)

**Cambios:**
- Cambiar `--block-dark-*` tokens a `--bg-light` / `--text-light` / `--border-light`
- Eliminar archivo duplicado de estilos (unificar en un solo CSS)
- Mantener terminal visual con código y typing animation

---

### Task 7: Architecture — Pipeline Horizontal

**Files:**
- Modify: `web/src/components/SwissArchSection.tsx`
- Modify: `web/src/styles/swiss-architecture.css`

**Cambios:**
- Reemplazar stack vertical por pipeline horizontal de 6 etapas
- Etapas: Python App → PyO3 Bridge → Rust Engine → Query Engine → Storage Layer → Disk (mmap)
- Flechas SVG entre etapas
- Decoración terminal en labels
- Mantener hover interaction

---

### Task 8: Polish Sections — Benchmarks + Ecosystem + Monolith + Footer

**Files:**
- Modify: `web/src/styles/swiss-benchmark.css`
- Modify: `web/src/styles/swiss-ecosystem.css`
- Modify: `web/src/styles/swiss-monolith.css`
- Modify: `web/src/styles/footer.css`

**Cambios:**
- Agregar `$` prompts o `❯` en headers de sección
- Asegurar consistencia terminal en toda la página
- Footer: prompt decorativo

---

### Task 9: Verificar Build + Push

**Files:**
- Run: `npx tsc --noEmit && npx vite build`

**Steps:**
1. TypeScript check
2. Vite build
3. Commit
4. Push
