---
title: "Fase 3: Subpáginas Técnicas"
status: active
tags: [vantadb, web, strategy]
last_reviewed: 2026-07-03
aliases: []
---

# Fase 3: Subpáginas Técnicas

> Extraído de: `strategy/implementation_plan.md` | Skills: `frontend-design`, `d3-visualization`, `minimalist-ui`

---

## Overview

Rediseñar 4 subpáginas técnicas bajo patrón SwissSubpageHero + secciones alternadas.

## SwissSubpageHero — Componente Compartido

### [NEW] [SwissSubpageHero.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/components/SwissSubpageHero.tsx)
**Props**: `label`, `title`, `description`, `breadcrumb`

**Layout**: grid 12 cols, título en cols 1-8 (asimetría)
- Label: `[ENGINE]` en `--text-label` naranja
- Breadcrumb: `Home / Engine` en `--text-label`, `--steel`
- Título: `--text-display`
- Descripción: `--text-body`, `--muted`
- Borde inferior: `1px solid var(--border)` full-width

### [NEW] [swiss-subpage.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/swiss-subpage.css)

---

## Subpáginas

### [MODIFY] [engine.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/engine.tsx)
- Hero: `<SwissSubpageHero label="ENGINE" title="The Rust Core" />`
- 6 secciones: HNSW, BM25, WAL, PyO3, Zero-Copy Serde, SIMD
- Layout: grid `5fr 7fr` (SVG + texto) o invertido
- Fondo alternado: warm paper ⇄ OLED negro

### [MODIFY] [engine.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/engine.css)
- Purgar, reescribir con tokens Swiss

### [MODIFY] [architecture.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/architecture.tsx)
- Hero: `<SwissSubpageHero label="ARCHITECTURE" />`
- Diagrama SVG interactivo con exploded view al scroll
- Labels con líneas de cota y coordenadas técnicas

### [MODIFY] [architecture.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/architecture.css)
- Reescribir

### [MODIFY] [integrations.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/integrations.tsx)
- Hero: `<SwissSubpageHero label="INTEGRATIONS" />`
- Grid matrix ampliado con iconos monoline + código ejemplo

### [MODIFY] [integrations.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/integrations.css)
- Purgar, reescribir

### [MODIFY] [use-cases.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/use-cases.tsx)
- Hero: `<SwissSubpageHero label="USE CASES" />`
- Versión expandida: cada caso = sección completa con diagrama + stack + código

### [MODIFY] [use-cases.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/use-cases.css)
- Reescribir

---

## Verification

### Automated
```powershell
npx tsc --noEmit
npx eslint .
npm run build
```

### Visual
- Engine/Architecture/Integrations/UseCases con Swiss style
- SVG diagramas funcionales

### Git
```bash
git add -A && git commit -m "feat(pages): phase 3 — technical subpages redesign"
```
