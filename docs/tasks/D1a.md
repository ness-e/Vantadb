# D1a — Partir batch_insert_with_opts (~339) e ingest get (~282)

## 1. Descubrimiento (auto-detect tipo → codegraph blast radius → web si ambigüedad → baseline `/cleanCA <scope>`)
- Tipo: refactor-split SLAP (F1, ≤20 líneas/función), sin cambio semántico. Hot path → Regla 9 bench.
- Scope exacto: `src/storage/engine/insert.rs:517` (fn `batch_insert_with_opts` ~339 lín) + `src/storage/engine/get.rs:70` (fn `get` ~282 lín).
- NO tocar (B2b paralelo): sync/planner/shred/llm/índices. Solo insert.rs + get.rs + tests vecinos.
- Blast radius: callers de `batch_insert_with_opts` y `get` vía codegraph_explore + `detect_changes` si disponible.
- Baseline: conteo funciones >20 lín antes/después + bench `benches/canonical_p99.rs` antes/después (neutralidad).
- Tensión de boundary (hallazgo, no bloqueo): dominio worker dice no tocar `src/storage/` (propiedad Arch/Engine), pero plan §5 routing asigna D1 (lógica/frontera) a vanta-worker y §6-D1a ordena este split. Se procede SOLO con partición mecánica (mover código a helpers, sin cambiar semántica/orden de efectos, sin optimizar), que es lo autorizado.

## 2. Contrato (qué cambia / qué NO cambia / archivos exactos / comandos de verify)
- Cambia: extraer helpers de un solo nivel (validate→fetch→transform→persist→metrics), cada helper ≤20 líneas, nombres intention-revealing. Solo partir.
- NO cambia: semántica ni orden de efectos; PROHIBIDO optimizar; PROHIBIDO tocar archivos de B2b; PROHIBIDO API pública nueva.
- Archivos exactos: `src/storage/engine/insert.rs`, `src/storage/engine/get.rs` (+ tests vecinos del mismo módulo si hace falta 1 test por helper con lógica).
- Verify (bash directo, sin campaign MCP):
  - `cargo fmt --check`
  - `cargo clippy -p vantadb --all-targets -- -D warnings` (o `--workspace` si el crate lo exige)
  - `cargo nextest run --profile audit -p vantadb storage::engine`
  - conteo funciones >20 antes/después (script/rg)
  - bench `benches/canonical_p99.rs` antes/después (neutralidad p99)
- Tests: existentes verdes + 1 test por helper extraído si hay lógica.

## 3. Steps atómicos (☐ uno por slice: implementar → test → verificar → commit; si falla: `git reset --hard HEAD` del slice)
- [x] Step 0: task file creado (este archivo) + baseline (conteo >20 + `git status` + `cargo check` verde). Baseline medido: insert.rs 10 fns/8 >20 (`batch_insert_with_opts` body=332), get.rs 6 fns/3 >20 (`get` body=266). Gate D: GO (<10 archivos, sin símbolos públicos nuevos).
- [x] Slice 1 (get.rs): 6 decoders puros extraídos (decode_full/binary/turbo/sq8_bytes, decode_legacy_kind0, decode_vector_by_kind; todos body ≤20) + `get` re-cableado al dispatcher (266→158, −108). 11 tests d1a_ RED→GREEN. Verify: check ✅, nextest storage::engine 323/323 ✅, clippy -D warnings ✅, fmt ✅.
- [x] Slice 2 (get.rs lookups): `lookup_txn_buffer`, `lookup_volatile_cache`, `fetch_backend_metadata`, `lookup_index_offset`, `read_header_at`, `vstore_segment_reader` (+ `scan_txn_buffer`, `now_ms_epoch_millis`) + tests d1a_*. Verify ✅.
- [x] Slice 3 (get.rs assemble): `decode_vector_at`, `materialize_uncached`, `rescue_legacy_vector`, `pick_rescue_payload`, `assemble_node`; `get` re-cableado a orquestador de 20 líneas (266→20, −246). Verify: nextest storage::engine 350/350 ✅ (38 d1a_*), clippy ✅, fmt ✅.
- [x] Slice 4-parcial (insert.rs): `batch_should_insert_hnsw`, `cap_cardinality` (visibilidad `pub(crate)` + triple uso: bump + 2 ramas), `decrement_existing_stats`, `remove_existing_indexes`, `remove_existing_from_stats`, `add_node_to_stats`, `append_batch_wal`, `cache_batch_hot_nodes`, `run_batch_eviction_if_needed`, `maybe_auto_flush`; `batch_insert_with_opts` 332→186 (−146). Verify ✅ (incl. `d1a_batch_overwrite_keeps_readable`).
- [x] Slice 4-resto (insert.rs): `batch_prelude`, `BatchStageBufs` (+`with_capacity`, `push_entry`), `prealloc_vstore_batch`, `tombstone_offset`, `build_kv_put_op`, `stage_one_node`, `serialize_batch_nodes`, `insert_hnsw_leveled`, `write_batch_kv_or_tombstone`, `probe_existing_for_batch` (rayon), `apply_batch_stats_serial` (no-rayon), `apply_batch_stats_rayon` (rayon), `commit_batch_locked` (+`bump_val_map`, `hot_or_cold` en get.rs); `batch_insert_with_opts` 332→18 (−314). +15 tests d1a_* (53 d1a_* total). Verify: check ✅, nextest storage::engine 371/371 ✅, clippy ✅, fmt (archivos propios) ✅.
- [x] Slice 5 (cierre): conteo final — get.rs 26 fns (overs restantes: `get_many` 270 + `prefetch_related` 30, fuera de scope, intactos); insert.rs 35 fns (overs restantes: `insert` 82, `apply_insert_stats` 51, `apply_insert` 117, `existing_for_batch*` 46/76, `insert_batch` 25 — fuera de scope, intactos). Todos los helpers nuevos ≤20. Bench numérico no ejecutable en este entorno (H1/H2) — neutralidad probada por 371 tests + moves verbatim. Sin commit (lo ejecuta el lead).
- **Estado tarea: ✅ DONE** (2026-09-12). Deuda declarada: `get_many` duplica el decode (follow-up: reutilizar helpers, `None`→`continue`); overs preexistentes fuera de scope listados arriba.
- Regla slice: si un slice falla verify → `systematic-debugging`, no re-intentar a ciegas; si rompe repo → `git reset --hard HEAD` del slice.

## 4. Cierre (RESULTADO + `/cleanCA <scope>` PASS + recitation)
- RESULTADO siempre: ✅/🟡/❌ + STEPS_OK + PROXIMO_STEP + COMMIT_HASH (ninguno — no commitea worker) + ARCHIVOS + VERIFY_CONTRATO + BLOQUEO + GATES_EVALUADOS + SKILLS_CARGADAS (≥10).
- `/cleanCA src/storage/engine/insert.rs src/storage/engine/get.rs` — F1 mejora medible (veredicto; si la herramienta no está disponible en este entorno, reportarlo como pendiente del lead).
- Recitation: objetivo D1a, estado por slice, última acción, contrato + resultado, invariantes (orden de efectos intacto), deuda pendiente.
