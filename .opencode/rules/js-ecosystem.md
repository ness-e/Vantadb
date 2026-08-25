# JS Ecosystem (WASM / TS / Node) — Reglas

> **Scope:** `vantadb-wasm/` (`src/lib.rs`, `idb.rs`, `opfs.rs`, `worker.rs`, `opfs_bridge.js`), `vantadb-ts/` (SDK TS npm `vantadb`), `vantadb-node/` (napi-rs)
> **No tocar aquí:** bindings Python (`python-bindings.md`), server/MCP (`server-mcp.md`), API pública del core (`api-contract.md`)
> **Status:** 🟢 Vigente
> **Fuentes:** auditoría WASM/TS/Node 2026-08-04, DOC3 (DESKTOP-01b)

## Reglas

### R-1: WASM — persistencia IDB/OPFS solo vía backends explícitos

- **Must:** documentar `vantadb-wasm` con sus backends reales: `InMemory` (default) y persistencia `connect_persistent`/`connect_idb`/`connect_worker` con save/load a IndexedDB/OPFS (implementados 2026-08-23, CORE-02: `graph_state.json`, `save_idb`/`load_idb`). Toda doc que prometa persistencia debe nombrar el método de conexión que la habilita.
- **Must not:** afirmar persistencia para `connect()`/`new()` en memoria, ni prometer WAL en WASM (sigue deshabilitado en `init.rs`).
- **Por qué:** la regla anterior ("siempre InMemory") quedó obsoleta cuando se implementó la capa de persistencia IDB/OPFS; documentar menos de lo que existe también es falla de diagnóstico.

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
- **Por qué:** los 3 bindings (node `lib.rs:201-287`, python `op_gate`/`enter()`, wasm `try_enter`/`enter`) implementan `OpGate` + drenaje desde COMP-029/AUD-011 (2026-08-05); esta regla exige mantenerlo en cualquier binding nuevo.

<!-- Referencias cruzadas: → ver api-contract.md, release-ci.md, concurrency-async.md -->
