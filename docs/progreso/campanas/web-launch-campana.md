# Campaña WEB Launch (2026-08-04)

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

### 2026-08-04 — Campaña WEB Launch (5 tareas) ✅

**Fuente:** Backlog (plan `docs/plans/2026-08-04-launch-web-campaign.md`)

**Ejecutada por:** pipeline paralelo (vanta-docs/vanta-worker + orquestador vanta-lead)

- **MKT-05:** 5/5 blog posts técnicos — 5º añadido `docs/blog/benchmarks_vs_lancedb_chroma.md` (run real glove-100-angular: VantaDB 241.4 QPS/recall 100%/p50 4.124ms). Commit `bf5e6c1e`.
- **MKT-15:** tabla competitiva §03 en `/benchmarks` (cifras reales VantaDB/LanceDB/ChromaDB + filosofía Pinecone/Weaviate, sin cifras inventadas). Commit `68e18405`.
- **WEB-001:** WASM real en `/playground` — rebuild `wasm-pack build --target no-modules` (initSync), assets en `web/public/vanta-wasm/`, loader en `code-playground.tsx`. Commit `ee310422`.
- **WEB-18:** pricing alineado a GTM Phase 1 (opción b) — tier "Team $49" eliminado, planes Community + Enterprise. Commit `f90b4ec8`.
- **GH-119:** guía `docs/tutorials/migrate-from-vectara.md` (259 líneas), issue #119 cerrado. Commit `ebfb3363`.

**SKIP:** TSK-103 → cubierto por MKT-15/NUEVO-10, no implementado. OLD-01 → DEFER.

**Nota:** `docs/Blog=` no versionado; WASM `vantadb-wasm/pkg/` en .gitignore (se sirve copia desde `web/public/vanta-wasm/`).
