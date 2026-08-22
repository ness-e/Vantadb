# Campaña P32 — Bindings SDK (sub-clientes TS/Python)

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

### Campaña P32 Bindings SDK completada 2026-08-22 - 4/4 tareas (plan `2026-08-22-vantadb-bindings-sdk.md`)

**MEM-36 pagada:** sub-clientes por dominio en TS y Python con backward-compat 100%. Suites: TS **246/246** (17 nuevos), Python **105 passed** (16 nuevos), docs coverage 0 gaps.

- **SDKB-01:** mapa canon namespace↔método por SDK — hallazgos: supersede SOLO Python; Python get/delete/insert node-level (graph); diferencias per-SDK documentadas (`dffc7419`).
- **SDKB-02:** TS sub-clientes lazy getters frozen (db.memory 12 / db.graph 10 / db.wiki vacío v1 / db.system 16) + test destructurado this-binding (`bf51f4cc`).
- **SDKB-03:** Python forward_to_db! delegantes espejo, __init__.pyi actualizado (`e4eb120e`).
- **SDKB-04:** Domain Sub-clients en READMEs + gate backward-compat final (`12d30257`).

**Decisiones:** D42 sub-clientes SOLO capa TS/Python (cero WASM — fricción wasm-pack eliminada); D43 capacidades vanta-memory vía bindings deferidas (requiere nuevo binding Rust); D44 TS primero.

**Deudas colaterales nuevas en Backlog:** BND-01 LinkError wasm pkg snippet idb.rs (pre-existente, dueño arch) · BND-02 drift types.ts↔pkg topological_sort.
