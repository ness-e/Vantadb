# COMP-029: Node.js/TS bindings nativos vía napi-rs (backend adicional a WASM)

## Metadata
- **Plan file:** — (backlog directo)
- **Fuente:** docs/Backlog.md:293
- **Esfuerzo:** 🟡 2-3 sem
- **Prioridad:** 🟠
- **Tipo:** Rust core + Bindings (Mixto)
- **Turns estimados:** 15-45
- **Creado:** 2026-08-02
- **Estado:** ✅ COMPLETADO
- **Routing:** vanta-worker (implementación) + vanta-docs (docs/api/TS_SDK.md)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vantadb-ts/src/vantadb.ts` (envuelve `WasmVantaDB` con `_wasm<T>()` + `wrapWasmError`), `vantadb-ts/package.json` (name `vantadb` v0.5.0, ESM, dep `vantadb-wasm: file:../vantadb-wasm/pkg`), `vantadb-ts/vitest` tests, `docs/api/TS_SDK.md`, `web/` (consumidores futuros del wrapper TS) |
| Callees | `vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemoryListOptions, VantaMemorySearchRequest, VantaNodeInput, VantaValue}`, `vantadb::config::VantaConfig`, `vantadb::DistanceMetric` (mismo set que `vantadb-python/src/lib.rs`), features `fjall, memmap2, rayon` del crate `vantadb` |
| Implicaciones | Crate NUEVO `vantadb-node/` (no toca `vantadb-ts` ni `vantadb-wasm` existentes). **No rompe API pública:** el wrapper TS actual queda intacto; el binding nativo es un backend ADICIONAL. Browser se queda con WASM (`vantadb-wasm`). Compile-time: cdylib nativo por plataforma (napi `napi8`). Runtime: `.node` por arquitectura (napi-rs build genera `vantadb.<platform>.node`). No hay migración de datos ni schema bump. `cargo-deny` debe aceptar licencias de napi-rs (MIT/Apache-2.0). `vantadb-node` NO se agrega al workspace root si rompe build server — decidir en Step 1 (workspace member vs crate standalone). |

## Contrato
"`cd vantadb-node && npm run build` (o `napi build`) produce `vantadb.<platform>.node` y `index.js`/`index.d.ts` generados. Un test vitest en `vantadb-ts/tests/` (o `vantadb-node/tests/`) ejecuta: (1) `connect` a una DB temporal con persistencia real (fjall) → `put` → `get` devuelve el valor; (2) reiniciar la conexión a la misma ruta → el dato persiste (WAL/fsync, NO posible en WASM); (3) `search` devuelve resultados ordenados por score. La API del binding nativo es isomórfica con la del wrapper WASM actual (`vantadb-ts/src/vantadb.ts`) para los métodos expuestos. `cargo check -p vantadb-node`, `cargo fmt --check`, `cargo clippy` y `npx tsc --noEmit` pasan sin warnings."

## Herramientas necesarias
- cargo-mcp (check, nextest, fmt, clippy, add)
- codegraph_explore (blast radius del SDK surface) — parcialmente ejecutado
- rust-analyzer-mcp (diagnostics)
- Node/npm local (para `@napi-rs/cli`, tsc, vitest)

## Investigation Notes
- **Backlog.md:293** (estado ⚠️ Investigado, listo para implementar, sin dependencias): Investigación Jul 29 recomendó napi-rs como backend ADICIONAL a WASM (NO reemplazo). ~1500 líneas nuevas, ~2-3 sem. WASM no puede persistencia real; napi-rs sí (fjall/WAL/fsync). Browser se queda con WASM. 80% del patrón reutilizable de `vantadb-python`.
- **`vantadb-python/Cargo.toml`** (referencia de patrón): crate `vantadb_py`, lib `vantadb_native` (cdylib), pyo3 0.29 abi3-py311, dep `vantadb` in-process con features `fjall, memmap2, rayon`.
- **`vantadb-python/src/lib.rs`**: clase `VantaDB` con `engine: VantaEmbedded`, patrón `engine.clone()` para detach en hilos (open_in_background), imports de `vantadb::sdk::*` y `vantadb::config::VantaConfig`. Módulos helper: `convert.rs`, `types.rs`, `vector.rs`. 41 métodos públicos expuestos.
- **`vantadb-ts`**: WASM-powered. `package.json` name `vantadb` v0.5.0, ESM, dep `vantadb-wasm: file:../vantadb-wasm/pkg`, scripts `build=tsc`, `test=vitest`, `lint=eslint`. `src/vantadb.ts` envuelve `WasmVantaDB` con `_wasm<T>()` error boundary + `wrapWasmError` — patrón a replicar en el wrapper nativo (backend agnóstico: WASM o napi).
- **`vantadb-wasm`** (browser, NO se toca): `src/lib.rs`, `idb.rs`, `opfs.rs`, `worker.rs`, `opfs_bridge.js`.
- **Web research (metasearchmcp, Jul 2026)**: napi-rs vigente (napi-rs.github.io, github.com/napi-rs/napi-rs). `@napi-rs/cli` se instala como devDependency local para build → copia `.node` a `index.js`/`index.d.ts`. `napi build` soporta macOS/Linux/Windows x64 MSVC. napi-derive active.
- **Grep napi en repo**: solo `web/package-lock.json` (@napi-rs/wasm-runtime transitivo de rollup) y `VantaDB_Manual_Estrategico_Unificado.md` P9 (líneas 117, 227): decisión de estrategia multi-lenguaje — "TypeScript/WASM (parcial), Node nativo (napi-rs), Go (cuándo)". Ningún crate napi en Cargo.toml todavía.
- **Audit backlog-validation-2026-07-28.md:192** confirma: TS SDK usa WASM (`wasm-bindgen`), NO napi-rs.
- **`docs/Investigaciones/`** NO contiene la investigación Jul 29 de COMP-029 (no está como archivo) — la fuente es el backlog + Manual P9.
- ⚠️ **Disco C casi lleno (~45 MB libres; `target/` = 101.84 GB).** `cargo check -p vantadb` es el mínimo footprint. Si ENOSPC → `cargo clean` primero (libera ~100 GB).

## Steps

### Step 1: Crear crate `vantadb-node/` + scaffolding napi-rs
- **Archivos:** `vantadb-node/Cargo.toml`, `vantadb-node/src/lib.rs`, `vantadb-node/package.json`, `vantadb-node/build.rs`, `vantadb-node/npm/` (platform packages)
- **Acción:** Estructura mirror de `vantadb-python/`: crate name `vantadb_node`, lib `vantadb_native` (cdylib). Deps: `napi = { version = "3", default-features = false, features = ["napi8"] }`, `napi-derive = "3"`, `vantadb = { path = "..", features = ["fjall", "memmap2", "rayon"] }`, `serde`/`serde_json` si hace falta. devDeps: `@napi-rs/cli`. Scripts: `build = "napi build --platform --release"`, `test`, `lint`. Decidir: workspace member vs crate standalone (si workspace root build server se rompe, standalone con `[workspace]` vacío en su Cargo.toml).
- **Verify:** `cargo check -p vantadb-node` (o `cd vantadb-node && cargo check`)
- **Estado:** ✅ COMPLETADO

### Step 2: Exponer core class nativa (equivalente a clase `VantaDB` de python)
- **Archivos:** `vantadb-node/src/lib.rs` (+ módulos `convert.rs`, `types.rs`, `vector.rs`)
- **Acción:** `#[napi]` class `VantaDB` con `engine: VantaEmbedded`. Métodos de ciclo de vida: `connect(db_path, config?)`, `close()`. Patrón `engine.clone()` para operaciones async en threads (replicar `open_in_background`). Mapear error type a `napi::Error`.
- **Verify:** `cargo check -p vantadb-node`; smoke: `napi build` genera `.node` y `index.d.ts`
- **Estado:** ✅ COMPLETADO

### Step 3: Operaciones CRUD + memory + search
- **Archivos:** `vantadb-node/src/lib.rs`
- **Acción:** Métodos isomórficos con `vantadb-ts/src/vantadb.ts` (WASM wrapper): `put/insert_node`, `get`, `delete`, `search`, `list`/`list_memories`, `graph` ops si expuestas. Usar los types `VantaMemoryInput`, `VantaNodeInput`, `VantaValue`, `VantaMemorySearchRequest` de `vantadb::sdk`. Serde conversion para inputs/outputs.
- **Verify:** `cargo check -p vantadb-node`
- **Estado:** ✅ COMPLETADO

### Step 4: Wrapper TS del binding nativo (backend-agnóstico)
- **Archivos:** `vantadb-node/index.js` (generado), `vantadb-node/index.d.ts` (generado), `vantadb-ts/src/native.ts` (o similar)
- **Acción:** Wrapper TS que carga el `.node` nativo con fallback claro: si no hay `.node` para la plataforma, error explícito (browser usa WASM, Node usa napi). Reutilizar firma de `WasmVantaDB` + `wrapWasmError` para error handling consistente. NO romper la API pública actual de `vantadb-ts`.
- **Verify:** `cd vantadb-ts && npx tsc --noEmit`; vitest smoke test
- **Estado:** ✅ COMPLETADO

### Step 5: Test de persistencia (diferencial vs WASM)
- **Archivos:** `vantadb-node/tests/persistence.test.ts` (vitest)
- **Acción:** (1) `connect` a ruta temporal → `put` → `get` devuelve valor; (2) `close` → reconectar a misma ruta → dato persiste (fjall/WAL — lo que WASM no puede); (3) `search` retorna ordenado por score.
- **Verify:** `cd vantadb-node && npm test`
- **Estado:** ✅ COMPLETADO

### Step 6: Docs + ADR + registro en Backlog
- **Archivos:** `docs/api/TS_SDK.md` (o nuevo `docs/api/NODE_SDK.md`), `docs/architecture/adr/NNN_napi-rs-node-bindings.md`, `docs/Backlog.md` (COMP-029 → ✅), `VantaDB_Manual_Estrategico_Unificado.md` (P9: marcar "Node nativo (napi-rs)" como decidido/implementado)
- **Acción:** Doc-driven: escribir docs API del binding nativo. ADR con la decisión "napi-rs como backend adicional a WASM" (tradeoff: binario nativo vs portabilidad WASM). Actualizar backlog y progreso.
- **Verify:** `scripts/validate-docs-coverage.ps1` (si existe); revisión manual de referencias
- **Estado:** ✅ COMPLETADO

## Dependencias
- Ninguna bloqueante. (P9 del Manual Estratégico vence esta semana como decisión — confirmar antes de Step 1.)

## Notas
- NO tocar `vantadb-wasm/` (browser se queda con WASM) ni la API pública de `vantadb-ts` existente — el binding nativo es aditivo.
- NO introducir deuda técnica nueva sin pagar equivalente (Regla 6): napi-rs trae `unsafe` en el FFI generado, pero el código nuestro debe estar en safe Rust; los `napi` derive macros lo manejan.
- El patrón `engine.clone()` de `vantadb-python` (VantaEmbedded es cloneable) es la clave para operaciones async sin locks globales — replicar exacto.
- Regla 3 (docs sync): exponer métodos públicos en binding → actualizar `docs/api/` en el mismo PR.
- ⚠️ Disco: si `cargo check` falla por ENOSPC → `cargo clean` primero.
- Confirmar versión de napi-rs y plataformas target en `vantadb-node/.npmignore`/CI antes de build release (no hay proyecto previo en el repo como ejemplo).
