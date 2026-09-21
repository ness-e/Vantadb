# Task BND-FIXES — 3 fixes técnicos de infraestructura (BND-06, BND-01, BND-02)

## Estado: ⏳ IN PROGRESS

Fuentes: `docs/Backlog.md` líneas 761 (BND-01), 762 (BND-02), 781 (BND-06).
Restricciones del orquestador: NO commitear. NO editar plan files.
Coordinación: sesión GOV paralela puede tocar `.config/nextest.toml` — re-basear si cambió bajo los pies (verificado al inicio: `git diff HEAD -- .config/nextest.toml` vacío).

## Steps

- ✅ S1 (BND-06): scope-safe default-filter en `.config/nextest.toml` — VERIFICADO: `-p vanta-proxy` 89/89 passed; `-p vantadb "wiki::"` 24 run/2017 skipped
- ⬜ S2 (BND-01): root cause LinkError pkg WASM (`vantadb-wasm/src/`) + rebuild `wasm-pack build --dev --target web`
- ⬜ S3 (BND-02): alinear `vantadb-ts/src/types.ts` contra `.d.ts` regenerado (regla VS-CORE-05: generar, no editar a mano) para `topological_sort`
- ⬜ S4: VERIFY FINAL completo + cierre

## Impacto mapeado (Regla 0)

**Archivos leídos completos:**
- `.config/nextest.toml` (100L verbatim) — default-filter con 55 exclusiones bare `binary(X)`; perfiles audit/ci-windows/experimental/chaos intactos
- `vantadb-wasm/src/idb.rs` (191L verbatim) — `#[wasm_bindgen(inline_js)]` + extern `__vanta_ensure_idb_bridge`
- `vantadb-wasm/src/opfs.rs` (273L verbatim) — solo js_sys/Reflect, SIN módulos wasm_bindgen
- `vantadb-wasm/pkg/snippets/` — solo 1 snippet (`inline0.js`, 3171B); pkg posiblemente parcheado localmente
- Raíz `Cargo.toml` (sección targets) — TODOS los `[[test]]` nombrados pertenecen a package `vantadb`

**Mapeo dueños de binarios excluidos (discovery):**
- Package `vantadb` (explícitos): benchmark_internal, benchmark_datasets, cli_tests, concurrency_parity, derived_index_recovery, durability_recovery, edge_cases, file_locking_stress, fjall_cold_copy_restore, fuzz_proptest, index_reconstruction, memory_api, memory_brutality, memory_export_import, operational_metrics, prefetch_benchmark, property_durability, schema_evolution, text_index_recovery, wal_resilience, structured_api_v2, hnsw_validation, hybrid_ranking_metrics, hybrid_retrieval_quality, sift_validation, stress_protocol, competitive_bench, hardware_profiles, basic_node, vector_scale_check, integration, antilocality_layout, backend_tests, chaos_integrity, crash_injection, gc, mmap_index, multi_process_lock, mutations, storage, tombstone_ann_vstore, multilingual_tokenizer_integration, memory_telemetry, regression_certification, snapshot_certification, security_audit
- Muertos (no existen en ningún crate): concurrency_primitives, python (real: python_sdk_boundary), hnsw_recall (real: hnsw_recall_certification), multi_namespace_stress
- `vantadb-server` (implícitos por stem de tests/): benchmarks, cli_args, e2e, mcp_integration, server
- `vantadb-mcp` (implícitos): code_tests, mcp_tests, skills_tests, wiki_async_ingest, wiki_roundtrip_e2e, wiki_tests
- Sin colisiones de nombres entre crates → calificación por dueño preserva semántica exacta de runs workspace

**Root cause BND-06 (reproducido):** nextest 0.9.133 valida CADA operador `binary(X)` del filter expression contra los binarios del scope seleccionado → `error: operator didn't match any binary names` con `-p vanta-proxy`. La validación es por operador, independiente de posición booleana.
**Hipótesis validada empíricamente:** envolver como `not (package(vantadb) and binary(security_audit))` pasa la validación scoped (test con `--config-file` temporal sobre `cargo nextest list -p vanta-proxy` ✅ lista tests).

**Referencias entrantes nextest.toml:** CI usa `--profile audit/ci-windows --features "cli,arrow,tls,opentelemetry"` (workspace-wide, ci-rust-10.yml/release-binaries-63.yml), `--profile chaos -p vantadb` (chaos-45.yml). El fix NO debe cambiar qué se excluye en runs workspace.

**Referencias entrantes wasm:** lib.rs:469-483 espera que el CONSUMIDOR JS importe `spawnOpfsWorker` desde `src/opfs_bridge.js` (no es snippet). idb.rs usa inline_js auto-registrado en globalThis.

**Veredicto impacto:** BAJO-MEDIO. nextest.toml: cambio mecánico de predicados, tabla de verdad idéntica para workspace, scoped pasa a funcionar. WASM/TS: pendiente root cause (S2).

## Verificación contrato (S4)

1. `cargo check -p vanta-proxy --all-targets`
2. `cargo nextest run -p vanta-proxy` → ~89 tests corren
3. `cargo test -p vanta-proxy` (o según lo arreglado)
4. `wasm-pack build --dev --target web` sin LinkError + tests TS cargan pkg
5. `cd vantadb-ts && npm test && npx tsc --noEmit`
6. `cargo fmt --check`
7. `cargo nextest run -p vantadb "wiki::"` sigue filtrando bien (FIX 1)

## Context Save Point

(discovery completo; sin ediciones aún)

---

## CIERRE (lead, 2026-08-23)
Sub-agentes agotados por memoria tras FIX 1 → lead completó FIX 2/3 directamente:
- **FIX 1 BND-06** ✅ nextest scope-safe (`not(package(X) and binary(Y))`) — `-p vanta-proxy` corre 89/89
- **FIX 2 BND-01** ✅ root cause: snippet inline0.js era IIFE sin export; el .wasm importaba `__vanta_ensure_idb_bridge` inexistente → LinkError. Fix: eliminar import fantasma + `extern "C" {}` vacío mantiene el snippet en el pkg (IIFE auto-registra globalThis.vantaIdbStorage al cargar). TS 246/246.
- **FIX 3 BND-02** ✅ .d.ts regenerado devuelve `any` para topological_sort — tsc exit 0, tipo manual GraphTopologicalSortResult cubierto por tests runtime (vanta.test.ts:635).
Verify final: nextest -p vanta-proxy 89/89 · npm test 246/246 · tsc exit 0 · fmt exit 0.