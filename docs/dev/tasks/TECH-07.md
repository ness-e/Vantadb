# TECH-07 — Publicar pkg WASM con feature `opfs`

- **Estado:** ✅ COMPLETED
- **Fecha:** 2026-08-05
- **Commit:** `566e9369`

## Qué se hizo
- Rebuild del pkg WASM con `wasm-pack build --features opfs` (wasm-pack 0.15.0) → `pkg/vantadb_wasm.d.ts` ahora incluye los 4 exports: `connect_worker`, `worker_read`, `worker_write`, `worker_delete` (líneas 73, 223, 230, 237).
- Documentación en `vantadb-wasm/src/lib.rs`: nota "Optional capability" en `connect_worker` (con ejemplo de `spawnOpfsWorker` desde `src/opfs_bridge.js`) y en worker_read/write/delete.
- Demo browser: `vantadb-wasm/demo/worker-test.html` (nuevo).
- Nota: el sub-agente original devolvió vacío sin commit; el lead verificó el pkg (d.ts con los 4 exports), commiteó el trabajo restante y escribió este save point.

## Verificación
- `wasm-pack --version` → 0.15.0 ✅
- `pkg/vantadb_wasm.d.ts`: grep connect_worker/worker_read/worker_write/worker_delete → 4 exports presentes ✅
- El pkg/ NO está trackeado en git (build local) — el commit contiene solo fuente + demo.

## Observaciones
- Browser test con worker pendiente de entorno real (se puede ejecutar `vantadb-wasm/demo/worker-test.html` en navegador).
