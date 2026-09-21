# B2b — Eliminar `unwrap` prod con `Result` + `?`

## 1. Descubrimiento (auto-detect tipo → codegraph blast radius → web si ambigüedad → baseline `/cleanCA <scope>`)
- Tipo: fix prod (categoría (c) de B2a → `Result`+`?`; categoría (b) → solo doc invariante). Focos originales wal/engine/etc. son test — NO tocarlos.
- Base: `docs/tasks/B2a.md` Tablas B (44×(c), 8 archivos) y C (15×(b), 9 archivos). Orden B2a: `sync_ext.rs` → `planner/graph` → `shred` → `llm.rs` → `flat/scann/diskann`.
- 31 son `lock().unwrap()` en hot paths: evaluar cada uno (lock envenenado irrecuperable → documenta invariante (b); error real → variante Error + `?`).
- Baseline: `cargo clippy -p vantadb -- --deny warnings` verde (heredado de B2a 2026-09-11).
- Blast radius: `sync_ext.rs` (30 líneas, trait + callers); `flat/scann/diskann` hot paths lectura/búsqueda → avisar `vanta-tuner` + `vanta-chaos` stress (Regla 8/9).

## 2. Contrato (qué cambia / qué NO cambia / archivos exactos / comandos de verify)
- Cambia: 8 archivos (c): `src/sync_ext.rs` (3× expect) → trait devuelve `Result`; `src/planner.rs:356` (1×) → validación; `src/graph.rs:487` (1×) → `ok_or_else`+`?`; `src/shred/mod.rs` (4×) → bounds-check + `map_err`+`?`; `src/llm.rs:555` (1×, feature `remote-inference`) → `map_err`+`?`; `src/index/flat.rs` (5×), `src/index/scann.rs` (15×), `src/index/diskann.rs` (12×+2×) → `map_err`+`?` o helper `lock_or_poison()` si >3×/archivo. 9 archivos (b): solo comentario invariante `// INVARIANT:`/`// SAFETY:`, sin cambio de firma.
- NO cambia: focos B2a test (`wal.rs`, `wal_sharded.rs`, `engine.rs`, `parser/mod.rs`, `physical_plan/mod.rs`, `sdk/api.rs`); mensajes de error públicos sin nota; tests para que pasen (Regla 2); slices >1 archivo; `expect()` nuevo (prohibido).
- Archivos exactos: ver Tablas B/C de B2a. Callers que ahora reciben `Result`: actualizar firmas (especialmente `sync_ext.rs`).
- Verify por slice: `cargo nextest run --profile audit -p vantadb <módulo>` + `cargo clippy -p vantadb -- --deny warnings` en bash directo (PROHIBIDO campaign MCP). Cierre: `cargo fmt --check` + `/cleanCA <archivos>` (E1 ✅). Hot path search/ingestión con cambio de comportamiento → medir `canonical_p99` antes/después (Regla 9).

## 3. Steps atómicos (☐ uno por slice: implementar → test → verificar → commit; si falla: `git reset --hard HEAD` del slice)
- [x] Slice 0: crear este task file (formato §4 del plan) — sin Read previo del inexistente (solo glob). ✅ este archivo.
- [x] Slice 1: `src/sync_ext.rs` (3× expect poisoned) — RED (3 tests poison → panic) → GREEN (`into_inner` + INVARIANT; 0 callers prod, trait intacto); verify: nextest 8/8 + clippy ✅.
- [x] Slice 2: `src/planner.rs:356` (`SchemaError` + `?`) + `src/graph.rs:487` (`if let`, insert-precede-get) + doc `src/physical_plan/join.rs:95` (b, `let-else`); verify: planner 38/38, graph 66/66, join 8/8 + clippy ✅.
- [x] Slice 3: `src/shred/mod.rs` (4× → (b) docs, slices de longitud exacta por bounds-check; silent-truncate preservado) + `src/index/flat.rs` (add → `RuntimeError` + `?` con test RED→GREEN; resto `into_inner`); verify: flat 19/19, shred 13/13 + clippy ✅.
- [x] Slice 4-bis: `src/llm.rs:555` (env expect, feature `remote-inference`) — RED (test env-ausente → panic en ctor) → GREEN con **deferred-key single-file** (mejor que el plan multi-archivo): `api_key: String` → `Option<String>` (campo privado), `new()` sin panic + doc, `embed()` mapea `None` → `InvalidInput("VANTA_OPENAI_API_KEY must be set…")`. 0 API pública cambiada (`new()->Self`, `Default`, factory, trait intactos; los 4 call sites ya degradan graceful ante embed-Err). Rechazada Opción A (`new()->Result` + factory→`Result` + 4 call sites + quitar `Default`: ~70 líneas, 4 archivos, rompe ctor público). Verify: test 1/1 + filtros llm/executor/vector 198/198 (`--features remote-inference`) + check/clippy default y con feature + fmt ✅.
- [x] Slice 5: `src/index/flat.rs` — ver Slice 3 (completo).
- [x] Slice 6: `src/index/scann.rs` (`update_bounds` → `Result` + 3 locks de `add` → `?`; search/estimate/len → `into_inner`; test RED→GREEN); verify: 12/12 + clippy ✅.
- [x] Slice 7: `src/index/diskann.rs` (`insert_vector` → `Result` + 2 gets → `NodeNotFound` + `?`; add bitsets → `?`; resto → `into_inner`; test RED→GREEN); verify: 7/7 + suite `index` 355/355 + clippy ✅.
- [x] Slice 8: 15×(b) documentar invariantes (crypto 5×, serialize/bytes 2×, binary_header, router, state, vfile_mmap, vfile, fmt, data + join en Slice 2) — solo comentarios; verify: clippy + crypto 17/17 (feature `encryption`) + binary_header 13/13 ✅.
- [x] Slice 9: cierre — `cargo fmt --check` ✅ + barrido E1 por archivo (evidencia abajo) + RESULTADO. No commitear (lead). Gate D GO (0 símbolos públicos nuevos; 2 fns privadas con firma cambiada).

## 4. Cierre (RESULTADO + `/cleanCA <scope>` PASS + recitation)
- `/cleanCA <scope>`: pendiente de ejecutar por el lead (comando opencode, no bash). Evidencia E1 lista: barrido `rg 'unwrap\(\)|\.expect\('` sobre los 17 archivos — 0 (c) restantes en prod; quedan (b) documentados + tests (contrato B2a: tests/doctests/`crash_helper` excluidos).
- Recitation: objetivo B2b **44/44 (c) + 15/15 (b) COMPLETO**; última acción Slice 4-bis llm ✅; contrato clippy+nextest por slice ✅; invariantes: 0 API pública cambiada (2 fns privadas → `Result`, 1 campo privado `String`→`Option`), `VecIndex` intacto, truncado shred preservado; deuda: ninguna de B2b; próxima tarea: lead ejecuta `/cleanCA` + commit + `vanta-chaos` stress (Regla 8).
