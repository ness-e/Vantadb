# JS Ecosystem (WASM / TS / Node) — Reglas

> **Scope:** `vantadb-wasm/` (`src/lib.rs`, `idb.rs`, `opfs.rs`, `worker.rs`, `opfs_bridge.js`), `vantadb-ts/` (SDK TS npm `vantadb`), `vantadb-node/` (napi-rs)
> **No tocar aquí:** bindings Python (`python-bindings.md`), server/MCP (`server-mcp.md`), API pública del core (`api-contract.md`)
> **Status:** 🟢 Vigente
> **Fuentes:** auditoría WASM/TS/Node 2026-08-04, DOC3 (DESKTOP-01b)

## Reglas

### R-1: WASM siempre InMemory — no prometer persistencia

- **Must:** documentar y mantener `vantadb-wasm` como backend `BackendKind::InMemory` (WAL deshabilitado en `init.rs`); si se añade persistencia (IDB/OPFS), hacerlo con feature gate explícito.
- **Must not:** afirmar en docs que WASM persiste datos (los handlers `connect_idb`/`save_idb`/`load_idb`/`delete_idb` existen pero el backend sigue InMemory).
- **Por qué:** `vantadb-wasm/src/lib.rs:60-66` fija `backend_kind: BackendKind::InMemory`; documentar persistencia donde no existe es una falla de diagnóstico (DOC3 §5.1).

### R-2: `pkg/` y artefactos de build no se commitean

- **Must:** considerar `vantadb-wasm/pkg/` (y `dist/` de `vantadb-ts`) artefactos de build regenerables; NO referenciarlos como fuente en docs de investigación.
- **Must not:** citar `vantadb-wasm/pkg/` como estructura existente del repo (no está commiteado) ni documentar contenido de `pkg/` que no se puede verificar.
- **Por qué:** la auditoría DOC3 citó `pkg/` con `connect_worker`/`opfs_bridge` como existente; el directorio no existe en el repo (artefacto no commiteado).

### R-3: Crates standalone (napi-rs) fuera del workspace son intencionales

- **Must:** mantener `vantadb-node` (y crates de providers standalone) con `[workspace]` vacío y el comentario de por qué (MSVC linker crash con cdylib); documentarlo como decisión, no como omisión.
- **Must not:** "corregir" un `[workspace]` vacío sin leer el comentario de exclusión intencional del root Cargo.toml.
- **Por qué:** DOC1 (VantaDB-28-07-2026) reportó como error lo que es una exclusión deliberada documentada (cdylib + workspace heredado rompe el linker MSVC).

### R-4: Lifecycle de bindings: op-gate + drenaje en close

- **Must:** todo binding (Python/WASM/Node) que exponga ops async sobre el engine: (1) enrutar la op con `spawn_blocking`, (2) guardar cada op con un op-gate que rechace `database is closing`, (3) en `close()` llamar `drain()` del gate antes de cerrar el engine.
- **Must not:** permitir un `put` fire-and-forget cuyo `spawn_blocking` no ha corrido cuando `close()` retorna — el write se pierde silenciosamente en exit.
- **Por qué:** `vantadb-node/src/lib.rs:75-92,277-287` implementa `OpGate` + `drain()` exactamente para esto; `vantadb-python` y `vantadb-wasm` no tienen el patrón → riesgo de write-after-close.

<!-- Referencias cruzadas: → ver api-contract.md, release-ci.md, concurrency-async.md -->
