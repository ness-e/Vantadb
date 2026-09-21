# ADR: Node.js/TS bindings nativos vía napi-rs (backend adicional a WASM)

## Context

`vantadb-ts` expone el SDK vía WASM (`vantadb-wasm`, `wasm-bindgen`). WASM cubre el
browser (IndexedDB/OPFS) pero no puede ofrecer persistencia real en archivos
(fjall/WAL/fsync) ni acceso directo al filesystem para aplicaciones Node.js
server-side. La investigación de Jul 29 (Backlog COMP-029) recomendó napi-rs como
**backend ADICIONAL** — no reemplazo — para dotar a Node.js de persistencia real,
reutilizando el 80% del patrón consolidado de `vantadb-python` (`VantaEmbedded` es
`Clone`; cada operación corre en un hilo con un handle clonado).

Decisión estratégica vinculada: `VantaDB_Manual_Estrategico_Unificado.md` P9
"TypeScript/WASM (parcial), Node nativo (napi-rs)" — napi-rs es el mecanismo de
bindings nativos estándar para Rust en Node.

## Decision

Agregar un crate **standalone** `vantadb-node/` (NO workspace member) cuya lib es
un `cdylib` `vantadb_native`, envuelto con `napi 3` (feature `napi8`) +
`napi-derive` sobre el SDK `vantadb` (features `fjall, memmap2, rayon`).

- API isomórfica con el wrapper WASM (`vantadb-ts/src/vantadb.ts`): `connect`,
  `flush`, `close`, `put`, `put_batch`, `get`, `delete`, `list`, `list_namespaces`,
  `search`, `capabilities`.
- Patrón async: toda operación de engine en `tokio::task::spawn_blocking` con
  `engine.clone()`, así el thread principal de JS nunca se bloquea.
- Boundary de I/O: `serde_json::Value` — inputs se parsean manualmente (los structs
  del SDK no tienen `#[serde(default)]`), outputs con los `Serialize` existentes.
- `[workspace]` vacío en su `Cargo.toml`: napi-rs emite un cdylib; los cdylib en
  workspace trigger el crash del linker MSVC en builds windows del workspace con
  `target/` compartido vía `.cargo/config.toml`.

### Decisión de aislamiento (standalone, no workspace member)

| Opción | Veredicto |
|--------|-----------|
| workspace member | ❌ — cdylib rompe build del workspace en Windows MSVC (crash linker) |
| standalone crate | ✅ — build independiente, comparte `target/` vía `.cargo/config.toml` |

## Consequences

- **Pros:** persistencia real en Node.js (fjall/WAL/fsync) no disponible en WASM;
  reutiliza patrón de `vantadb-python`; browser no se afecta (`vantadb-wasm` .
  intocado); + API aditiva sin breaking changes en `vantadb-ts`.
- **Cons:** requiere build nativa por plataforma (`.node` per target); napi-rs
  introduce un runtime binding (addon nativo) que crates.io/wasm no. Browser y
  WASM siguen siendo el único backend para browser.
- **Deuda (Regla 6):** cuota deuda paga por napi-rs FFI preexistente (bindings
  generados por macros, unsafe encapsulado); código propio permanece en safe Rust
  (`spawn_blocking` con `VantaEmbedded: Clone`).

## Verification

- `cd vantadb-node && npm test` → vitest pasa 3/3 (put/get, persistencia
  cross-reconnect, search ordenado por score).
- `cargo check -p vantadb-node` limpio.
- `npx tsc --noEmit` en `vantadb-ts` → error preexistente ajeno en
  `src/vantadb.ts:789` (temporal edges + WASM, HEAD sin modificar por COMP-029).

## Status

Accepted (2026-08-02). Routing: vanta-worker + vanta-docs.