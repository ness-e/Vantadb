# REVIEW-17 — unsafe wasm32 innecesarios

- **Estado:** ⬜ PENDING → ✅ IN PROGRESS → ✅ COMPLETED
- **Plan:** `docs/plans/2026-08-25-batch-core-server-mcp.md` (Task 10)
- **Esfuerzo:** 🟢 | **Prioridad:** 🟢 | **Appetite:** max 2h

## Contrato

- `cargo check --target wasm32-unknown-unknown -p vantadb-wasm` sin errores ni warnings nuevos (baseline: 7 `unnecessary unsafe`)
- unsafe removidos o `// SAFETY:` por-plataforma (los 7 bloques citados)
- `cargo clippy --workspace --all-targets -- -D warnings` pasa

## Discovery (RESULTADO)

**Diagnóstico:** el gate real es FEATURE (`memmap2`), no arch. `vantadb-wasm` usa `default-features = false, features = ["wasm"]` (`wasm = []` vacío) → memmap2 OFF → shim activo bajo wasm32. En el shim, `MmapOptions::map/map_mut` son SAFE (lectura a buffer alineado); los 7 bloques `unsafe` que los envuelven son redundantes → `unused_unsafe` (confirmado por baseline `cargo check --target wasm32-unknown-unknown -p vantadb-wasm`: 7 warnings en las líneas exactas citadas).

**Callers directos de `Mmap::map`/`MmapMut::map_mut`** (asociados, API parity): `src/node/vector_data.rs:442`, `src/index/graph.rs:209`, `src/index/serialize/file.rs:62,132,147`, `src/storage/engine/maintenance.rs:180,200` — el shim DEBE conservar `unsafe fn` para parity (esos callers compilan bajo wasm32 también y dependen de la firma). Solo se remueven los bloques internos redundantes del shim.

**Estrategia:** 2 helpers safe `map_readonly`/`map_readwrite` en vfile_mmap.rs que concentran el `unsafe` real de memmap2 (cfg-gated, con `// SAFETY:`); los 5 callers (vfile.rs:202,206,277; archive.rs:74,105) usan los helpers → 7 bloques eliminados. La operación OS-level se preserva bajo memmap2 (x86_64 intacto); bajo shim no había unsafe que preservar.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `src/storage/vfile.rs` (843L, regiones 120-299 + codegraph), `src/storage/vfile_mmap.rs` (492L completo), `src/storage/archive.rs` (801L completo), `Cargo.toml` (feature `memmap2`/`wasm`), `vantadb-wasm/Cargo.toml`
- **Referencias hacia dentro (entrantes):**
  - `vfile.rs` re-exportado vía `crate::storage::vfile::*` → engine, archive.rs, index/graph.rs, index/serialize/file.rs, node/vector_data.rs, storage/engine/maintenance.rs, lsm.rs
  - `vfile_mmap.rs` re-exportado vía `crate::storage::vfile::{AlignedBytes, Mmap, MmapMut, MmapOptions}`, `install_sigbus_handler` (unix), `get_resident_bytes`
  - `archive.rs` → `compact_layout` (MCP `compact_layout`), `traverse_graph`, `rebuild_hnsw_from_vstore` (engine/storage)
- **Referencias hacia fuera (salientes):** vfile.rs → binary_header, crypto, error, node, engine; vfile_mmap.rs → error, memmap2, libc; archive.rs → error, index::CPIndex, node, vfile, engine, web_time, zerocopy
- **Veredicto:** los 7 bloques `unsafe` se eliminan; firmas públicas de `Mmap::map`/`MmapMut::map_mut`/`MmapOptions` NO cambian; helpers nuevos `pub(crate)` (no son API pública). Native (memmap2 ON) ejecuta la MISMA llamada unsafe, ahora en 2 helpers con `// SAFETY:`. Sin impacto semántico, sin cambios de tests.

## Steps

### Step 1 — Helpers safe en vfile_mmap.rs
- [x] `use std::fs::File;` sin cfg-gate (los helpers lo usan bajo memmap2 también)
- [x] `map_readonly(file)` / `map_readwrite(file)` con cfg-gate interno (memmap2: `unsafe` + `// SAFETY:`; shim: llamada safe directa)
- [x] Remover bloques `unsafe {}` internos de `Mmap::map`/`MmapMut::map_mut` del shim (73, 112) — fns siguen `unsafe fn` por parity, body safe documentado

### Step 2 — Callers en vfile.rs
- [x] :202 `ReadOnly(map_readonly(&file)...)`
- [x] :206 `ReadWrite(map_readwrite(&file)...)`
- [x] :277 `ReadWrite(map_readwrite(file)...)`
- [x] Imports + comentarios SAFETY stale actualizados

### Step 3 — Callers en archive.rs
- [x] :74 y :105 → `map_readwrite(&tmp_file)`
- [x] Import: drop `MmapOptions`, add `map_readwrite`

### Step 4 — Verify
- [x] `cargo check -p vantadb` (native, memmap2 ON) ✅ 0 warnings
- [x] `cargo check --target wasm32-unknown-unknown -p vantadb-wasm` → 0 warnings (baseline 7)
- [x] `cargo clippy -p vantadb --lib -- -D warnings` (memmap2) ✅
- [x] `cargo clippy -p vantadb --no-default-features --features wasm --lib -- -D warnings` (shim/wasm) ✅
- [x] `cargo nextest run -p vantadb storage::` → 363/363 ✅
- [x] `cargo fmt --check` ✅
- [x] `cargo clippy --workspace --all-targets -- -D warnings` → ⚠️ falla SOLO por FIND-30 pre-existente (`cli_server.rs:1302` unused `ns`, feature `server`, ajeno a REVIEW-17; fuera de blast radius, dueño MOD-13)

## Context Save Point

- Baseline wasm32: 7 warnings exactos en vfile.rs:202,206,277; archive.rs:74,105; vfile_mmap.rs:73,112 — 0 errores.
- Los módulos con callers directos (graph.rs, serialize/file.rs, maintenance.rs, vector_data.rs) NO tienen cfg-gate y compilan bajo wasm32 → el shim mantiene `unsafe fn` parity; NO tocar su unsafe.
- Verify wasm en Windows: target `wasm32-unknown-unknown` SÍ está instalado (rustup) → verify wasm COMPLETO, no parcial.
- NO commitees — el lead verifica y commitea por tarea.

## Context Save Point (FINAL)

**Todos los steps ✅.** Resultado:
- 7 bloques `unsafe` eliminados (vfile.rs:202,206,277; archive.rs:74,105; vfile_mmap.rs:73,112). El `unsafe` real de memmap2 ahora vive en 2 helpers `map_readonly`/`map_readwrite` (cfg-gated, con `// SAFETY:` + invariante — R-4 core-engine). FNs shim `Mmap::map`/`MmapMut::map_mut` siguen `unsafe fn` por parity (callers directos graph.rs, serialize/file.rs, maintenance.rs, vector_data.rs intactos).
- wasm32: 7 warnings → 0. Native (memmap2): comportamiento idéntico.
- Clippy workspace falla SOLO por FIND-30 pre-existente (`cli_server.rs:1302` unused `ns`, feature `server`), fuera de blast radius → registrado como FIND-30 en `docs/Backlog.md` (tabla P33). MOD-13 (dueño de cli_server.rs) puede absorberlo.
- Archivos tocados (NO commiteados — lead commitea): `src/storage/vfile_mmap.rs`, `src/storage/vfile.rs`, `src/storage/archive.rs`, `docs/Backlog.md` (FIND-30), task file.
- Learnings: feature `wasm = []` vacío + shim activo ⇒ los `unsafe` que envuelven `MmapOptions` (safe en shim) se vuelven unused_unsafe bajo wasm32; gate es FEATURE no arch.
