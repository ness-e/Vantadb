# Fase 2: Index / Landing Page

> Extraído de: `strategy/implementation_plan.md` | Skills: `gpt-taste`, `industrial-brutalist-ui`, `high-end-visual-design`

---

## Overview

Reescribir completa la landing page (`index.tsx`) con 8 secciones Swiss.

## [MODIFY] [index.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/index.tsx)

**Orden de secciones:**
1. `<SwissHero />` — Hero tipográfico con fondo animado
2. `<SwissBenchmarkGrid />` — Comparativa Bento
3. ~~Estadísticas~~ — **ELIMINADAS**
4. `<SwissQuickstart />` — Terminal + pasos 01-04
5. `<SwissCoreEngine />` — Features del motor con scroll pin (sección oscura)
6. `<SwissArchSection />` — Diagrama blueprint
7. `<SwissEcosystem />` — Grid de integraciones
8. `<SwissUseCases />` — Tarjetas editoriales
9. `<SwissMonolith />` — CTA final (The Monolith)

**Eliminar imports obsoletos:**
- `SingularityHero`, `AmberParticles`, `CodeGridBackground`, `ComparisonTable`, `ScrollStory`

## [NEW] Swiss Hero Components

### SwissHero.tsx
**Hero 100% tipográfico con fondo animado:**
- Grid 12 cols: etiquetas/título/CTAs en cols 1-8, cols 9-12 vacío (asimetría)
- Fondo animado: grid lines se dibujan con stroke-dashoffset (~800ms staggered), luego ambient drift (opacidad 0.3→0.5→0.3, loop 8s)
- Entrada: labels flash → título clip-path mask → subtítulo/CTAs fade

### SwissBenchmarkGrid.tsx
**Grilla Bento asimétrica 6 métricas:**
```
┌──────────┬─────┬──────────┐
│  2×2     │ 1×1 │  2×1     │
│ LATENCY  │DEPS │ MEMORY   │
├──────────┼─────┼──────────┤
│  1×1     │  2×1          │
│ SETUP    │  SEARCH TYPE  │
├──────────┼───────────────┤
│  3×1  CRASH RECOVERY     │
└──────────────────────────┘
```
- Entrada: celdas expanden desde grid lines (stagger 60ms), count-up 200ms
- Hover: borde → `--amber`, label → `--amber`

### SwissQuickstart.tsx
**Terminal + pasos [01]-[04]:**
- Grid 2 col: 4col pasos + 8col terminal
- Auto-play secuencial, typewriter 30ms/char
- Click en paso: salta directamente

### SwissCoreEngine.tsx
**GSAP ScrollTrigger pin — 6 features del motor:**
- Sección OLED oscura con pin
- 6 features se revelan secuencialmente al scrollear
- Cada feature: icono monoline + título + descripción

### SwissArchSection.tsx
**Diagrama blueprint industrial SVG:**
- Capas apiladas con borde 1px
- Scroll: exploded view (capas se separan)
- Hover: capa → borde `--amber`, resto opacity 0.3

### SwissEcosystem.tsx
**Grid 4×3 de integraciones:**
- Frameworks, LLM Providers, Deployment
- Icono monoline + label ALL CAPS
- Hover: icono → `--amber`, fondo → `--amber-dim`

### SwissUseCases.tsx
**Tarjetas editoriales horizontales:**
- Grid `3fr 9fr`: número display + contenido
- Hover: número `--subtle` → `--amber`
- 4 casos: AI Agent Memory, Local RAG, IDE Intelligence, Offline KB

### SwissMonolith.tsx
**CTA final bloque OLED:**
- Fondo `#0a0a0a`, padding 160px
- `pip install vantadb` en hero, centrado
- Cursor parpadeante (CSS blink 500ms)

## Verification

### Automated
```powershell
npx tsc --noEmit
npx eslint .
npm run build
```

### Visual
- Hero tipográfico con fondo animado, benchmark grid con count-up
- Quickstart terminal funcional, core engine pin funcional
- Architecture exploded view, ecosystem grid, use cases cards, monolith CTA

### Git
```bash
git add -A && git commit -m "feat(index): phase 2 — complete landing page redesign"
```
