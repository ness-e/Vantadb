# Fase 0-1: Foundations, Tokens, Nav & Footer

> Extraído de: `strategy/implementation_plan.md` | Documento maestro: `design/DiseñoNuevo.md`
> Skills: `color-expert`, `frontend-design`, `emil-design-eng`, `high-end-visual-design`

---

## FASE 0 — Fundaciones y Tokens

Actualización del sistema de diseño base. Todo lo demás depende de esta fase.

### [MODIFY] [tokens.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/tokens.css)
**Cambios específicos:**
- Añadir variables de bloques oscuros invertidos:
  ```css
  --block-dark-bg: #0a0a0a;
  --block-dark-text: #f0f0f0;
  --block-dark-muted: #808080;
  --block-dark-border: rgba(255,255,255,0.08);
  ```
- Añadir easing cortante: `--ease-cut: cubic-bezier(0.25, 1, 0.5, 1)`
- Añadir macro-spacing: `--section-gap: 96px; --section-gap-lg: 160px`
- Verificar sincronización con DESIGN.md

### [NEW] [swiss-grid.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/swiss-grid.css)
**Sistema de grid visible de 12 columnas:**
```css
.swiss-grid {
  display: grid;
  grid-template-columns: repeat(12, 1fr);
  gap: 1px;
  max-width: 1200px;
  margin: 0 auto;
}
```
- Clases utilitarias: `.swiss-section`, `.swiss-section--dark` (fondo `#0a0a0a` con texto invertido), `.swiss-section--inverted`
- Clases de span: `.col-span-1` a `.col-span-12`, `.col-start-*`
- Asimetría: `.asymmetric-left` (cols 1-8 contenido, 9-12 vacío)
- `.grid-line-v`, `.grid-line-h` para hairlines decorativas visibles
- `.section-divider` — línea horizontal de 1px full-width entre secciones

### [MODIFY] [layout.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/layout.css)
- `.section`: padding `var(--section-gap) 0`, max-width `1200px`
- `.section-dark`: fondo `#0a0a0a`, `color: var(--block-dark-text)`
- Purgar estilos del diseño anterior (gradientes, sombras difusas)

### [MODIFY] [buttons.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/buttons.css)
- `border-radius: 0px` forzado en todos los botones
- Primary: fondo `--amber` → hover `#000000`
- Ghost: borde `1px solid var(--border)` → hover fondo `--border` + texto blanco
- Ghost invertido (para secciones oscuras): borde blanco, texto blanco → hover fondo blanco + texto negro
- Eliminar todo `box-shadow`, todo gradiente
- Transición: `150ms var(--ease-cut)`

### [MODIFY] [animations.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/animations.css)
- Eliminar animaciones de bounce, elastic, spring
- Nuevas clases:
  - `.reveal-mask` — clip-path reveal desde abajo
  - `.reveal-expand` — expand desde líneas del grid
  - `.reveal-draw` — stroke-dashoffset animado para líneas SVG
- Todas las duraciones ≤ 250ms
- `@media (prefers-reduced-motion: reduce) { ... animaciones OFF }`

---

## FASE 1 — Nav + Footer (Componentes Globales)

### [MODIFY] [Nav.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/components/Nav.tsx)
- Barra fija 64px, `--surface-glass` + `backdrop-filter: blur(12px)`
- Borde inferior `1px solid var(--border)`
- Links en JetBrains Mono ALL CAPS, 0.72rem, tracking 0.14em
- Hover 100ms: `--steel` → `--foreground`
- Activo: `--amber`
- Eliminar link a `/about/roadmap`
- Mobile: hamburger → panel lateral

### [MODIFY] [nav.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/nav.css)
- Reescribir: `border-radius: 0px`, `box-shadow: none`

### [NEW] [SwissFooter.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/components/SwissFooter.tsx)
- Fondo `#0a0a0a`, grid 5 columnas
- Links `--block-dark-muted`, hover `#ffffff`
- Columnas: Product | Solutions | Developers | Resources | Company
- Bottom bar: logo + copyright + GitHub

### [NEW] [footer.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/footer.css)

### [MODIFY] [__root.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/__root.tsx)
- Reemplazar footer inline por `<SwissFooter />`

---

## Verification

### Automated
```powershell
npx tsc --noEmit
npx eslint .
npm run build
```

### Visual
- Fase 0: Tokens correctos, sin errores de compilación
- Fase 1: Nav funcional desktop/mobile, footer con todas las páginas

### Git
```bash
git add -A && git commit -m "feat(design): phase 0 — swiss design tokens and grid system"
git add -A && git commit -m "feat(nav): phase 1 — swiss nav and OLED footer"
```
