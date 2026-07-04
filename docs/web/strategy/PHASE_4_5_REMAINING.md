---
title: "Fase 4-5: Metrics, Solutions, Docs, Pricing, About"
status: active
tags: [vantadb, web, strategy]
last_reviewed: 2026-07-03
aliases: []
---

# Fase 4-5: Metrics, Solutions, Docs, Pricing, About

> Extraído de: `strategy/implementation_plan.md` | Skills: `frontend-design`, `d3-visualization`, `design-taste-frontend`

---

## FASE 4 — Subpáginas de Métricas + Changelog

### [MODIFY] [cost.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/cost.tsx)
- `<SwissSubpageHero label="COST" title="Total Cost of Ownership" />`
- Grid Bento comparativo: VantaDB vs Pinecone vs Weaviate vs Chroma
- Números gigantes Space Grotesk + barras SVG + tabla 1px

### [MODIFY] [latency.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/latency.tsx)
- `<SwissSubpageHero label="PERFORMANCE" title="Latency Benchmarks" />`
- Barras SVG p50/p95/p99, tabla monoespaciada JetBrains Mono

### [MODIFY] [storage.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/storage.tsx)
- `<SwissSubpageHero label="STORAGE" title="Storage Architecture" />`
- Diagramas SVG: WAL, HNSW graph, BM25 inverted index

### [MODIFY] [config.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/config.tsx)
- `<SwissSubpageHero label="CONFIG" title="Configuration Reference" />`
- Tabla de opciones: `name \| type \| default \| description` con bordes 1px
- Labels de sección: `[INDEXING]` `[STORAGE]` `[SEARCH]` `[RUNTIME]`

### [MODIFY] [maint.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/maint.tsx)
- `<SwissSubpageHero label="MAINTENANCE" title="Operations Guide" />`
- Pasos numerados `[01]`–`[N]` con borde izquierdo activo

### [MODIFY] [changelog.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/changelog.tsx)
- `<SwissSubpageHero label="CHANGELOG" title="Release Notes" />`
- Timeline vertical con línea 1px, releases con badges monoline

---

## FASE 5 — Solutions, Docs, Pricing, About, Blog, Community, Contact

### [MODIFY] [solutions/ai-agents.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/solutions/ai-agents.tsx)
- `[SOLUTION] AI Agent Memory`, diagrama agent→VantaDB→Memory

### [MODIFY] [solutions/local-rag.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/solutions/local-rag.tsx)
- `[SOLUTION] Local RAG Pipeline`, diagrama Documents→Embed→Query→LLM

### [MODIFY] [solutions/ai-ide-tooling.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/solutions/ai-ide-tooling.tsx)
- `[SOLUTION] AI IDE Tooling`, features para devs

### [MODIFY] [docs.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/docs.tsx)
- Layout 2 col: sidebar (3col) + contenido (9col)
- Link activo con borde izquierdo `--amber`

### [MODIFY] [pricing.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/pricing.tsx)
- `[PRICING] Simple, Transparent Pricing`
- Grid planes con borde 1px, destacado con borde `--amber`
- Feature table monoline ✓/✗

### [MODIFY] about/ pages
- `about/index.tsx`: Layout editorial + timeline hitos
- `about/company.tsx`: Swiss grid + tipografía correcta
- `about/community.tsx`: Grid canales con iconos monoline
- `about/contact.tsx`: Grid 2 col, inputs rectangulares radius 0px

### [DELETE] [about/roadmap.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/about/roadmap.tsx)
- **ELIMINAR** completamente + links en Nav y Footer

### [MODIFY] blog/ pages
- Blog list: grid 2 col desktop, cada card con borde 1px
- Post detail: layout editorial con tipografía Swiss

---

## Verification

### Automated
```powershell
npx tsc --noEmit
npx eslint .
npm run build
```

### Visual
- Fase 4: Cost/Latency/Storage/Config/Maint con métricas legibles, changelog timeline
- Fase 5: Solutions/Docs/Pricing/About/Blog/Community/Contact redesigned, roadmap 404

### Git
```bash
git add -A && git commit -m "feat(pages): phase 4 — metrics subpages and changelog"
git add -A && git commit -m "feat(pages): phase 5 — solutions, about, blog, pricing"
```
