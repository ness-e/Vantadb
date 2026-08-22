# Tareas migradas desde Backlog — tabla histórica

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

## Tareas Completadas (Migradas desde Backlog)

These tasks reached 100% completion and were moved here from the active backlog.

| ID | Tarea | Prioridad | Estado |
|---|---|---|---|
| `AUD-044..051` | **Campaña fix audit 0.5.0** (plan `docs/plans/2026-08-17-fix-audit-bugs.md`, 8/8 ✅ 2026-08-18): AUD-044 CLI search en DB fresca corre `ensure_indexes_current` (commit `a1d92f03`); AUD-045 MCP `memory_put` acepta `expires_at_ms`+`sparse_vector` (commit `27f3770e`); AUD-046 MCP put valida dims antes de insertar (commit `4936418a`); AUD-047 binario release con feature `server` (commit `4ac3b9fa`); AUD-048 semántica filtros unificada CLI↔MCP plano/`$eq`/operadores (commits `8dbe07a8`,`e6f43f3b`); AUD-049 shim `import vantadb` re-exporta `vantadb_py` + fix `.gitignore` `*db/` (commits `9a5e5305`); AUD-050 `inject_context` error claro para thread_id tipo inválido (commit `2f82117c`~); AUD-051 CLI `put --metadata` + rechazo `__vanta_*` + docs filter scope (commit `626dcc00`). Verify: mcp_tests 41/41, cli_tests 79/79, nextest 1893. AUD-042 sigue BLOQUEADO upstream (tantivy ≥0.27) | 🟠 | ✅ 2026-08-18 |
- **Plan archivado:** `docs/plans/archive/2026-08-17-fix-audit-bugs.md` — 8/8 completadas
- **Retrospectiva:** Start: verificar contrato con comandos cortos por task antes de dar por cerrada una wave | Stop: el campaign-server MCP no trackea in-progress por tarea en FAIL_MODE=parallel (uno a la vez) | Continue: waves por DAG de archivos real (no por tabla del plan), sub-agente RESUME en misma sesión para resultado vacío (4/4 tareas recuperadas así) | Acción medida: sub-agentes que devuelven RESULTADO vacío → RESUME en misma sesión hasta bloque RESULTADO presente (baseline: 4 de 5 devolvieron vacío la 1ª vez — mejora: prompt exige bloque RESULTADO explícito como gate de cierre)
| `P22-MCP` | Certificación MCP server vs skill: Bloque 1 (código) MCP-01 text search fix (`ensure_indexes_current` en arranque), MCP-02 `distance_metric` per-request, MCP-03 `distance`=1−cosine, MCP-04 `DimensionMismatch` isError; Bloques 2-5 (docs) skill sync — IQL Syntax, Response Envelope, Error Channels, Behavior Notes, dead refs, contradicciones. **Completa 2026-08-17:** MCP-15 stack overflow resuelto (root cause: recursión infinita get→prefetch_related→get en pares co-accesados cache-miss; fix `PrefetchGuard` thread_local+RAII single-level, GATE vanta-audit aprobado 0 C/H/M, commit `cd8dd129`) y T15 explain shape (doc alineada a realidad, test `test_mcp_search_memory_explain_shape`, commit `a7c0a00c`). Commits previos `d8f720f9` `d24fb663` `04840079`; tests MCP 34/34; test-busqueda.py 20/20; hash SAME skills↔.opencode/skills | 🟠 | ✅ 2026-08-17 |
| `REVIEW-04` | Refactor 3 god modules: `node.rs` 2078L → `src/node/{bitset,vector_data,label,edge,field,flags,disk,unified}.rs` + mod.rs facade; `vfile.rs` 1309L → `vfile_mmap.rs` (mmap shim+AlignedBytes+SIGBUS) + VantaFile ~490L. Re-exports lib.rs:157-160 intactos, unsafe 30 preservados, tests 64+32 sin pérdida. `config.rs` excluido (ponytail assessment en header: cohesive leave-as-is). Commit `d5624082` | 🟡 | ✅ 2026-08-12 |
| `TIR-03` | Decisión "mitigar primero en incidentes" — gap real confirmado: `bug-workflow.md` no tenía fase de contención (arrancaba diagnosticando). Veredicto: IMPLEMENTAR docs mínimos — nueva **Fase 0.5 Contención/Estabilización** (revert/pausar + registrar ANTES del debug, no reemplaza el Iron Law). Fuente: gap-01 FALTA#15, REPORTE-FINAL §3.3-15. Doc: `docs/Investigaciones/2026-08-10-agent-engineering/TIR-03-decision.md`; review P2-01 vanta-review ✅ approve | 🔴 | ✅ 2026-08-12 |
| `REVIEW-05` | God files restantes: `serialize.rs` 1595L → `src/index/serialize/{mod,bytes,file}.rs`; `distance.rs` 1721L → `src/index/distance/{mod,kernels,metrics,mapper}.rs` (SIMD f32x8/f32x16 y métricas byte-idénticos, dispatch preservado); `physical_plan.rs` 1542L → `src/physical_plan/{mod,scan,filter,vector,project,sort,join}.rs` (10 operadores). Re-exports `index/mod.rs:22` y `lib.rs:110` intactos; API pública removed=[] added=[]; 1878/1878 tests + clippy -D warnings + fmt --check. P2-7 (zero-copy) diferida. Commit `92852f9f` | 🟡 | ✅ 2026-08-12 |
| `ERR-031` | Index-API: `VecIndex::add` traga rechazos (solo warn) → trait retorna `Result<()>`, 5 impls propagan rechazos (non-full DiskAnn/Scann, read-only IVF, zero-norm CPIndex); fix `339107b0` + colateral clippy `918e57b1`; 3 tests rechazo `f585e423` | 🟢 | ✅ 2026-08-12 |
| `AUDREP-14` | Seguridad-Network: sin CORS → middleware configurable off por defecto (`VantaConfig::allowed_origins`, env `VANTADB_ALLOWED_ORIGINS`), `app_with_cors()` capa más externa; 2 tests + docs; commit `74a2c050` | 🟠 | ✅ 2026-08-07 |
| `AUDREP-16` | WAL-Compatibilidad: shard count hardcodeado (4) → layout on-disk ground truth via sidecar `<base>.shards` + inferencia; reconcilia al abrir; 3 tests; commit `13da3d6d` | 🟠 | ✅ 2026-08-07 |
| `AUDREP-17` | WAL-Operaciones: `run_loop` sin shutdown ni backoff → flag `AtomicBool` + sleeps interrumpibles + backoff exponencial (2s→60s cap) + no-spam con `replica_url` vacía; 3 tests; commit `2422981d` | 🟠 | ✅ 2026-08-07 |
| `AUDREP-18` | Storage-Cross-platform: `save_vector_index` falla en Windows (mmap vivo en rename) → scoped drop del mapping antes del rename, espeja `CPIndex::sync_to_mmap`; 1822 tests (incl. Windows); commit `df235fdd` | 🟠 | ✅ 2026-08-07 |
| `AUDREP-20` | SDK-TS-Types: `isMemoryRecord` rechazaba version/node_id numéricos → acepta `string || number` alineado con `MemoryRecord`; 3 tests + `tsc --noEmit` 0 errores; commit `734e9e11` | 🟠 | ✅ 2026-08-07 |
| `AUDREP-21` | MCP-OOM: collection_stats/list materializaban todo → agregados streaming página a página (fold, pico 1 página); test bounded; commit `b5278799` | 🟠 | ✅ 2026-08-07 |
| `AUDREP-22` | Integraciones-Versiones: 9 adapters Python 0.3.0 vs core 0.5.0 → bump a 0.5.0 + pin `vantadb-py>=0.5.0,<0.6.0`; validado tomllib; commit `776f734c` | 🟠 | ✅ 2026-08-07 |
| `AUDREP-24` | Configuración: `.gitignore` ignoraba `.env.example` → negación `!.env.example` en L68; commit `535a3964` | 🟡 | ✅ 2026-08-07 |
| `AUDREP-28` | Index-Precisión: distancia euclídea negativa por FP rounding → `max(0.0, ...)` en `euclidean_distance_sq_with_norms` + test `test_euclidean_distance_sq_with_norms_never_negative`; commit `feeeb73f` | 🟡 | ✅ 2026-08-07 |
| `AUDREP-31` | Seguridad-OOM: frame_len sin límite en EncryptionStream → cap `MAX_FRAME_LEN` 512MiB validado antes de alloc, reusa `CryptoError::InvalidCiphertext`; test `test_encryption_stream_rejects_oversized_frame`; commit `e99d82b6` | 🟡 | ✅ 2026-08-07 |
| `AUDREP-33` | Storage-Overflow: `(vstore.size*2)` overflow → `saturating_mul(2)` + `saturating_add(4096)`; commit `e81f963f` | 🟡 | ✅ 2026-08-07 |
| `AUDREP-36` | WAL-Recuperación: WAL corrupto truncado sin backup → cuarentena `.corrupt`/`.corrupt.N` (fail-soft, recovery nunca depende del backup) + test `test_corrupt_wal_tail_is_quarantined`; commit `00080282` | 🟡 | ✅ 2026-08-07 |
| `AUDREP-40` | Frontend-Contenido: badge hero "v0.1 · MVP" obsoleto → "0.5.0 · MVP"; commit `af7d1655` | 🟡 | ✅ 2026-08-07 |
| `AUDREP-44` | MCP-Concurrency: `active_requests` leak → guard RAII (ya existía) como único decremento; eliminado `fetch_sub` manual que doble-decrementaba; test panic-safe `active_request_guard`; commit `489d9a88` | 🟡 | ✅ 2026-08-07 |
| `AUD-020` | Server-HTTPSec: tests auth/RBAC/rate-limit verdes — `cargo test -p vantadb-server --test server` 19/19; root cause: tests mandaban `{"query":"test"}`/`SELECT 1` (IQL inválido → 400 correcto post-ERR-027); fix: `SELECT * FROM Node`; RBAC HTTP ya conectado vía `token_role_map` | 🟡 | ✅ 2026-08-11 |
| `AUDREP-49` | Infraestructura: `version: "3.9"` obsoleto en compose → clave eliminada (sin warning); commit `af7d1655` | 🟢 | ✅ 2026-08-07 |
| `AUDREP-60` | SDK-TS-Código: `void ERROR_CODES` descartaba el const → eliminado el `void` (tipo `ErrorCode` sigue derivando); commit `b7f5a664` | 🟢 | ✅ 2026-08-07 |
| `AUDREP-62` | Server-DX: `--mcp` vía `args().any()` sin help → argv loop hand-rolled con `skip(1)` + `print_help()` documenta `--mcp` (sin clap runtime); commit `b7f5a664` | 🟢 | ✅ 2026-08-07 |
| `AUDREP-25` | Build-Release: fórmula Homebrew `0.2.0` con SHA placeholders → version 0.5.0; SHAs quedan placeholder (WONTFIX: no hay tarballs de release, documentado en comentario); commit `872c2a9b` | 🟡 | ✅ 2026-08-07 |
| `AUDREP-27` | Index-Lógica: zero-norm vectors silenciosamente descartados en Cosine → `add*` retorna `Result`; rechazo up-front; wrapper `VecIndex` loguea `tracing::warn`; 2 tests; commit `764ecc4b` | 🟡 | ✅ 2026-08-07 |
| `AUDREP-29` | Index-Algoritmo: NaN mapeado a `Equal` corrompía el heap HNSW → `total_cmp_sim` (orden total: NaN < finitos) evicción explícita; test; commit `764ecc4b` | 🟡 | ✅ 2026-08-07 |
| `AUDREP-30` | SDK-API: paginación con conteo pre-filtro → cursor post-filtro (`records.len()==limit`); test repro del loop infinito; commit `06916123` | 🟡 | ✅ 2026-08-07 |
| `AUDREP-32` | Seguridad: panic detail filtrado a clientes HTTP → mensaje genérico "Internal server error", detalle a `tracing::error!`; test; commit `a77905db` | 🟡 | ✅ 2026-08-07 |
| `AUDREP-34` | Storage-Overflow: `(total_needed + 63) & !63` overflow → `saturating_add(63) & !63`; commit `eb333794` | 🟡 | ✅ 2026-08-07 |
| `AUDREP-35` | WAL-Durabilidad: rename sin dir-fsync → helper `sync_parent_dir` (`utils/fs.rs`), aplicado en rotate/auto-rotate/compact_layout; no-op Windows (NTFS); 4 tests verdes; commit `8f3d65c0` | 🟡 | ✅ 2026-08-07 |
| `AUD-012` | Clippy gate: 5 errores pre-existentes (mixed_attributes_style, dead_code x2, approx_constant, items_after_test_module) arreglados en archive.rs, mcp lib.rs, parser, storage ops; `cargo clippy --workspace --all-targets` exit 0; commit `9d3c05a2` | 🔴 | ✅ 2026-08-08 |
| `AUD-013` | Tests invariante INV-024: unit tests de select_neighbors/neighbor_index + fix query zero-norm en flat threshold test; index:: suite 244 passed; commit `9d3c05a2` | 🔴 | ✅ 2026-08-08 |
| `AUD-014` | Prune duplicaba select_neighbors → canonicalizado como single source of truth (NodeSimMin::Ord tie-break); shrink delega; commit `9d3c05a2` | 🔴 | ✅ 2026-08-08 |
| `AUD-015` | Listas over-capacity sin techo (O(n²) build, hang en test 10k) → cap `2*m` en select_neighbors; test 10k: 6.46s vs hang previo; nextest 1844 passed; commit `9d3c05a2` | 🟠 | ✅ 2026-08-08 |
| `PERF-02` | Baseline riguroso post-publicación: `criterion` con perfiles fijos deterministas (warm-up 3s, measurement 5s, confidence 0.95, significance 0.05) + `critcmp` regression gate en workflow nightly (gated por `enable_critcmp`); dataset sintético determinístico persistido (`benches/data/synthetic_dataset.bin`, xorshift hash-verified). Sin cambios a benches principales. Commit `32462de6` | 🟡 | ✅ 2026-08-12 |
| `AUD-035` | Megafiles core (patrón REVIEW-05): **split 1** `src/sdk/search/mod.rs` 2521L → 8 submódulos (`lexical.rs`, `vector.rs`, `sparse.rs`, `hybrid.rs`, `explain.rs`, `audit.rs`, `debug_ops.rs`, `multi.rs`) + `tests.rs` (53 tests), mod.rs orquestador 330L — commit `5d96b536`. **Split 2** `src/storage/engine/ops.rs` 2131L → orquestador 331L + `delete.rs/get.rs/insert.rs/txn.rs` (mod.rs cableado). **Split 3** `src/index/search.rs` 2054L → `search/mod.rs` 52L + `pool.rs/profile.rs/layer.rs/neighbors.rs/nearest.rs/alternate.rs` + `tests.rs` 1379L. Signaturas públicas intactas (MCP/Python/WASM), visibilidad `pub(crate)`/`pub(super)` mínima. Nextest 1886 passed. Commits `5d96b536` + `552f08a8` | 🟡 | ✅ 2026-08-16 |
| `PERF-03` | Bench competitivo honesto de SDKs: harness `benchmarks/competitive_bench.py` extendido (Qdrant local + Milvus-lite, Chroma/Lance) mide en mismo HW; publica `docs/benchmarks/COMPETITIVE_SDK_BENCH.md` con números reales — hallazgo honesto: VantaDB Recall@10 59.2% vs Qdrant 100%/Chroma 97.6%/Milvus 100% (pierde en recall, gana en QPS). Commits `437a1125` + `9c1ec073` | 🟠 | ✅ 2026-08-12 |
| `PERF-05` | WAL async roadmap (ADR `DRV-015-wal-async-roadmap.md` — distinto del task DRV-015 de refactor WalWriter): documenta io_uring/aio + fsync group commit como siguiente paso tras DRV-014 (batch-append 3-5×). Sin código WAL nuevo; sin cambios a `src/`. Commit `9eef37c5` | 🔴 | ✅ 2026-08-12 |
| `PERF-08` | WASM serialización completa en hot path: `memory_record_to_js` emite `record.vector` como `Float32Array` zero-copy (js_sys) en vez de `serde_wasm_bindgen::to_value` por elemento; cierra P2-7. Persist-delta (H3-SER-001) diferido (requiere dirty-tracking en core, fuera de scope). Host compat: `vantadb-ts/src/types.ts` `vector?: Float32Array | number[]`. Commit `5105f22d` | 🟠 | ✅ 2026-08-12 |
| `COV-001` | Python: smoke test async de `AsyncVantaDB` — 3 tests (`test_async_smoke_crud_flush_purge`, `test_async_smoke_query_graph`, `test_async_smoke_export`) ejercitan `flush`/`purge_expired`/`query`/`graph_*`/`put`/`delete`/`export_*` (las ~37 líneas faltantes); pytest 3 passed; API pública intacta | 🟢 | ✅ 2026-08-12 (97a17828) |
| `COV-002` | TS: destrabar medición de coverage — `c8@^12` + `npm run coverage` envuelve `test-runner.mjs` y remapea V8 coverage a `src/*.ts` (vantadb.ts 86.56%, native.ts 53%, errors.ts 49.12%, guards.ts 47.95%); runner intacto 25 passed / 1 failed | 🟡 | ✅ 2026-08-12 (7419432c) |
| `TSYS-01` | Observabilidad de decisión — log estructurado de qué herramienta usó el agente y por qué cambió de estado; runtime: `decision_reason`/`pattern` + evento `plan.adjust` en campaign-server.mjs (gap-01 §3.3-17) | 🟡 | ✅ 2026-08-11 |
| `TSYS-02` | Handoff con invariantes — recitation exige invariantes + comandos de verificación + deuda (no solo lastAction/nextAction); task.md/pipeline-full.md (gap-01 §3.3-18) | 🟢 | ✅ 2026-08-11 (8f774c18) |
| `TSYS-03` | ADR gate mecánico — job `adr-gate` en ci-rust-10.yml:120-181 que falle si se toca API pública sin ADR (gap-01 §3.3-20) | 🟡 | ✅ 2026-08-11 (d9f2a4cb) |
| `TSYS-04` | Estimar con appetite (Shape Up) — "tiempo que VAMOS a invertir" como default en vez de effort vago; plan.md (gap-01 §3.3-21) | 🟢 | ✅ 2026-08-11 (8f774c18) |
| `TSYS-05` | SLA del pipeline — SLI/SLO/error budget; ADR-017 (gap-01 §3.3-23) | 🟡 | ✅ 2026-08-11 |
| `TSYS-07` | Recitation duplicado (3 definiciones) — unificado a 1 fuente, estructura §12 en pipeline-full.md/task.md (gap-01 §3.5-2) | 🟢 | ✅ 2026-08-11 (8f774c18) |
| `TSYS-08` | Triage "es ahora" (Shape Up) — triage DO/DEFER/SKIP/BLOQUEADO + pregunta "¿es el problema adecuado? ¿correcto el appetite? ¿es ahora?" + Cynefin en plan.md (gap-01 §3.5-8) | 🟢 | ✅ 2026-08-11 (8f774c18) |
| `TSYS-09` | Tracing de decisiones — `decision_reason`/`pattern` en `campaign_emit_event` + evento `plan.adjust` (por qué se reabrió/cerró, qué patrón) (agent-03 §5.2/§9; FALLA #6) | 🟢 | ✅ 2026-08-11 |
| `TSYS-10` | Human-in-the-loop: escalera a humano — HITL checkpoint: tareas 🔴 o ambiguas requieren confirmación humana antes de arrancar; §5 HITL en subagent-recovery.md (d9f2a4cb) | 🟢 | ✅ 2026-08-11 (d9f2a4cb) |
| `TSYS-11` | Límites de herramientas por rol — tabla de permisos por rol (worker = solo tools de su dominio; solo vanta-lead hace git push/commit/release) documentada en `.opencode/AGENTS.md` (agent-03 §9.2) | 🟡 | ✅ 2026-08-11 (d9f2a4cb) |
| `TSYS-13` | Validación de citas rotas por crawler — step de pipeline que extrae URLs de la evidencia, las resuelve (webfetch/HEAD; fallback manual sin red) y marca inválida la evidencia cuya URL no resuelve (agent-02 §7.8/§11.2) | 🟢 | ✅ 2026-08-11 (d9f2a4cb) |
| `TSYS-14` | Checklist anti-hábitos tóxicos como contrato — checklist conductual (agent-02 §12) referenciado desde prompts/task.md como guía obligatoria en fase de revisión | 🟢 | ✅ 2026-08-11 |
| `TSYS-15` | Memoria con esquema fijo y retrieval por tema — `- <fecha> \| <tema> \| <decisión\|lección> \| ref: <ruta:línea>`; read por tema vía `rg -n <tema>` (REPORTE-FINAL §3.4-2, FALLA #11) | 🟡 | ✅ 2026-08-11 |
| `TSYS-16` | Definir "qué es feature shippable" (trunk-based) — umbral formalizado en definition-of-done.md: (a) tests, (b) docs API en mismo PR, (c) observabilidad, (d) rollback viable, (e) sin caballos sueltos (REPORTE-FINAL §3.4-11) | 🟢 | ✅ 2026-08-11 (138d8735) |
| `COV-003` | Rust: tests del binario CLI — 7 tests nuevos en `tests/cli_tests.rs` (cmd_migrate/cmd_server + branches crud/data); `migrate.rs` 0%→51.17%, `server.rs` 0%→61.43%, `cli_handlers` ~0%→~76.5%; nextest 75 passed; clippy/fmt clean | 🟡 | ✅ 2026-08-12 (f9b93c75) |
| `COV-004` | ADR: política del gate de coverage en CI — ADR-018 decide root crate `vantadb` ≥80% (baseline 81.40%) como gate; workspace aggregate 72.76% se mide solo para visibilidad; bindings: Python pytest ≥85%, TS vía `c8` (COV-002), `cli_tests` incluido en root (COV-003), server/mcp/wasm excluidos; supersede ADR-015 §Decision #1 | 📖 | ✅ 2026-08-12 (8c631693) |
| `AUDREP-37` | WAL-PITR: fallback a mtime en `parse_segment_timestamp` → `Result<u64>`; nombre no parseable = `Err` (PITR falla loud en vez de reordenar silencioso); test; commit `ff82df8d` | 🟡 | ✅ 2026-08-07 |
| `AUDREP-38` | Parser: condiciones relacionales solo strings → RHS tipado (`parse_literal_field_value`): número → `Float`, quoted → `String` (backward compat); `edad > 18` funciona, ordering numérico; 4 tests; commit `e7214c00` | 🟡 | ✅ 2026-08-07 |
| `AUDREP-39` | Frontend-i18n: `lang="es"` hardcoded SSR → `lang={DEFAULT_LANG}` desde `dictionaries.ts`; quitado `suppressHydrationWarning` innecesario; commit `f49bbe10` | 🟡 | ✅ 2026-08-07 |
| `AUDREP-41` | Frontend-Dependencies: dead dep next-auth → `npm uninstall next-auth` (-13 pkgs); commit `e4cd7306` | 🟡 | ✅ 2026-08-07 |
| `AUDREP-43` | MCP-Rendimiento: collection_delete O(n) → transacción ya presente (`7d16a0b`); agregado test rollback sin deletes parciales; commit `e3c26287` | 🟡 | ✅ 2026-08-07 |
| `AUDREP-45` | Seguridad-Input: `postcard::from_bytes` sin bounds → `deserialize_node_payload` cap 128MiB en 8 paths storage (engine/ops, stats, maintenance); test malformed input; commit `eb879d84` | 🟡 | ✅ 2026-08-07 |
| `AUDREP-47` | Frontend-i18n: toast "quickstart.py copiado" hardcoded → `t("terminal.codeCopied")` (clave ES+EN existente); commit `668e191c` | 🟡 | ✅ 2026-08-07 |
| `AUDREP-48` | Configuración: raíz hardcodea edition/rust-version → `.workspace = true`; commit `1e2913a3` | 🟢 | ✅ 2026-08-07 |
| `AUDREP-50` | DX: faltaban targets macOS/WASM → añadidos `aarch64-apple-darwin`, `x86_64-apple-darwin`, `wasm32-unknown-unknown`; commit `1e2913a3` | 🟢 | ✅ 2026-08-07 |
| `AUDREP-52` | Configuración: tokio duplicado (deps vs dev-deps) → una entrada con features unificadas (rt/net/time); commit `e1c58b16` | 🟢 | ✅ 2026-08-07 |
| `AUDREP-53` | Código: OnceLock<MetricCache> para constante 2.0 → `const COSINE_TO_EUCLIDEAN_FACTOR` directo; tests distance 64 OK; commit `8dad81b1` | 🟢 | ✅ 2026-08-07 |
| `AUDREP-55` | Index-Lógica: Cosine→Euclidean fallback silencioso para zero queries → warn + resultados vacíos (sin rescore); 2 tests; commit `7864a50e` | 🟢 | ✅ 2026-08-07 |
| `AUDREP-56` | Código muerto: campo last_offset engañoso y sin lecturas → eliminado; serdefeg tolera markers viejos; commit `3469243b` | 🟢 | ✅ 2026-08-07 |
| `AUDREP-23` | Configuración: `exclude = ["fuzz"]` bajo `[workspace.package]` (tabla incorrecta) → movido a `[workspace]`; commit `d8e77741` | 🟡 | ✅ 2026-08-07 |
| `AUDREP-26` | DX-Cross-platform: Justfile solo con PowerShell → `set windows-shell` para Windows + shell POSIX default en Unix; commit `fec50757` | 🟡 | ✅ 2026-08-07 |
| `AUDREP-42` | Frontend-i18n: skip-link hardcoded → componente cliente `SkipLink` con `t("common.skipToContent")` (claves ES+EN); commit `6f0fdc4b` | 🟡 | ✅ 2026-08-07 |
| `AUDREP-46` | Frontend-TypeScript: `noImplicitAny: false` degradaba `strict: true` → `true`; 0 errores implícitos en el proyecto; `tsc --noEmit` clean; commit `d729fa66` | 🟡 | ✅ 2026-08-07 |
| `AUDREP-54` | Rendimiento: `purge_expired` clonaba vector + parseo JSON por registro (dead weight) → `vector: None, sparse_vector: None`; 353 tests SDK OK; commit `fe87f7ec` | 🟢 | ✅ 2026-08-07 |
| `AUDREP-57` | Frontend-Código muerto: `{false && ...}` + `setHeroVariant` + rama gato + `SfxLabel` inalcanzables → eliminados; hero renderiza `Mark` clásico; commit `8450a51c` | 🟢 | ✅ 2026-08-07 |
| `AUDREP-58` | Frontend-Duplicación: tokenizer Python copiado en 2 archivos → `lib/code-tokenizer.ts` compartido (`pythonTokenizer`/`jsTokenizer`); −90 líneas duplicadas; commit `3573c03f` | 🟢 | ✅ 2026-08-07 |
| `ERR-036` | Perf-Storage: write-lock en hot path de `get()` solo para `hits+=1` → `try_write()` + degradación a `read()` bajo contención (nunca bloquea a un writer); eviction/probes intactos; commit fix `e6cbc93f`; medición 1/4/8 threads: try_write ≈3% más rápido bajo lectores concurrentes; nextest 1898 passed; clippy -D warnings exit 0 | 🟠 | ✅ 2026-08-11 |
| `AUDREP-59` | Frontend-Configuración: nombre boilerplate `nextjs_tailwind_shadcn_ts` → `vantadb-web`; commit `3ca455ed` | 🟢 | ✅ 2026-08-07 |
| `AUDREP-61` | MCP-Seguridad: error interno filtrado a clientes → `error!()` con detalle server-side + JSON genérico al cliente; commit `bdf31c90` | 🟢 | ✅ 2026-08-07 |
| `AUDREP-02` | Engine-Panic: `.expect()` en deserialización de claves (guard previo ya lo protegía; fix defensivo `let-else`) | 🔴 | ✅ 2026-08-05 |
| `AUDREP-05` | Dockerfile: COPY 8 dirs inexistentes (resuelto por plan Task 16/AUD-001) | 🔴 | ✅ 2026-08-05 |
| `AUDREP-06` | Dockerfile: RUST_VERSION 1.94.0 → 1.94.1 (resuelto por plan Task 16/AUD-001) | 🔴 | ✅ 2026-08-05 |
| `AUDREP-03` | Storage-Consistencia: errores de tombstone tragados en ops.rs (3 sitios `let _ = write_header` → `tracing::error!`; commit `de83ebbf`; 345 tests storage OK) | 🔴 | ✅ 2026-08-05 |
| `AUDREP-08` | WAL-Race: colisión de timestamps en `archive_segment` (<1ms) → contador atómico `ARCHIVE_SEQ` + único rename atómico (commit `fe0dce6f`; 60 tests WAL OK) | 🔴 | ✅ 2026-08-05 |
| `AUDREP-13` | Seguridad-Auth: dev mode bypass silencioso → `tracing::warn!` por request no autenticada (commit `0f099822`; 4 tests auth OK) | 🟠 | ✅ 2026-08-05 |
| `AUDREP-12` | Seguridad-Network: sin límite de tamaño de body en `/api/v2/query` → `DefaultBodyLimit::max(1_000_000)` al router + test `body_limit_rejects_oversized` (413 para body > 1MB) | 🟠 | ✅ 2026-08-06 |
| `AUDREP-07` | Build-Código muerto: dep opcional `rayon` sin feature (paths paralelos inalcanzables) → `rayon = ["dep:rayon"]` en default features + limpieza docs (AGENTS.md:693, CONFIGURATION.md:229); commit `2c91d159` | 🔴 | ✅ 2026-08-06 |
| `AUDREP-19` | Frontend: `ignoreBuildErrors: true` en web/next.config.ts (bugs TS a producción) → eliminado (default false) + `reactStrictMode: true`; `tsc --noEmit` 0 errores; commit `1467a58e` | 🟠 | ✅ 2026-08-06 |
| `AUDREP-10` | Seguridad-Criptografía: key derivation débil (single SHA-256) → PBKDF2-HMAC-SHA256 (210k iter, salt 16B) + framing versionado; fallback legacy retrocompatible; 16 tests crypto OK; commit `a0ec48d3` | 🟠 | ✅ 2026-08-06 |
| `AUDREP-11` | Seguridad-Network: X-Forwarded-For confiado sin validación (IP spoofing) → `trusted_proxies` (env `VANTADB_TRUSTED_PROXIES`); XFF solo desde peers confiados, si no `ConnectInfo`; commit `008c9531` | 🟠 | ✅ 2026-08-06 |
| `NV-01` | Index-Panic: `sq8_similarity` indexa `rem_s[i]` con `rem_q.len()` (OOB) → clamp `min(rem_q.len(), rem_s.len())` en cosine+euclid; test `test_sq8_similarity_mismatched_dims_no_panic` OK; commit `555f3b70` | 🟠 | ✅ 2026-08-06 |
| `AUDREP-09` | Index-Datos obsoletos: IVF nunca se invalida tras inserts → `ivf_built_at_node_count: AtomicUsize`; rebuild cuando cambia node_count; test `test_ivf_rebuilds_when_nodes_added_after_build` OK; commit `c5b4967b` | 🟠 | ✅ 2026-08-06 |
| `AUDREP-15` | WAL-Concurrency: `rotate_all` libera el lock entre sync y swap (ventana race, writes perdidos) → lock sostenido a través de sync+open+swap; 26 tests wal OK; commit `f57bfa74` | 🟠 | ✅ 2026-08-06 |
| `NV-04` | Storage-UB: `AlignedBytes::grow_zeroed` sin garantía de alineación → ya implementado pre-existente (`Layout::from_size_align(len,4)` + alloc_zeroed, vfile.rs:394, AUDIT-03/INV-024); backlog desincronizado — sin commit | 🟠 | ✅ 2026-08-06 |
| `DEPS-01` | Investigación: 8 crates duplicadas (hashbrown/rand/rand_core/getrandom/reqwest/thiserror/lru/windows-sys) → reporte `docs/audit-reports/deps-01-duplicadas-2026-08-05.md`; causas=MSRV/API legítimas (thiserror y lru consolidadas); recomendación=NO unificar; Cargo.lock intacto | 🟡 | ✅ 2026-08-06 (investigación) |
| `TSK-56` | Fix Windows CI runner (windows-latest) | 🔴 | ✅ |
| `WEB-02` | Fase 2: Publish 3 Technical Blog Posts (Why I Built, SQLite for AI, Hybrid Search) | 🔴 | ✅ |
| `WEB-03` | Fase 2: Create real product pages (`/product/benchmarks`, `/security`, `/about/roadmap`, `/docs-api`) | 🔴 | ✅ |
| `DISC-05` | Fix telemetría de memoria (~225 GB falsos en 34 GB) | 🔴 | ✅ (pendiente reverificación formal) |
| `TSK-52` | SIGTERM shutdown handler (flush WAL + Fjall) | 🔴 | ✅ |
| `TSK-68` | Zero-copy FFI: NumPy arrays → 62ms→<20ms | 🔴 | ✅ |
| `TSK-73` | Async Python API (asyncio: `search_memory_async`) | ✅ Done | 2026-06-18 |
| `TSK-74` | Python type stubs (.pyi, mypy/pyright, autocomplete) | ✅ Done | 2026-06-18 |
| `TSK-69` | `put_batch()` con Rayon (5x speedup vs individual) | ✅ Done | 2026-06-18 |
| `TSK-46` | MMap-backed HNSW (1M vectores sin OOM en 8GB) | 🟠 | ✅ |
| `TSK-47` | Cuantización SQ8 (f32→i8, 4x RAM, <1% recall loss) | 🟠 | ✅ Done 2026-06-20 |
| `TSK-49` | Zero-copy deserialization con rkyv | 🟡 | ✅ Done 2026-06-20 |
| `TSK-50` | Backpressure al 80% RSS (rechazar con `MemoryPressure`) | 🟡 | ✅ |
| `TSK-75` | WAL compaction / vacuum (CLI + trigger 256MB) | 🟡 | ✅ |
| `TSK-76` | TTL en registros (`last_accessed`, `expires_at_ms`, `purge_expired`) | 🟠 | ✅ |
| `TSK-76b` | Memory eviction por importancia (score ponderado) | 🟡 | ✅ |
| `TSK-55` | Datasets reales en CI (GloVe-100, NQ 768d) | 🟠 | ✅ |
| `TSK-54` | Job CI nocturno de benchmarks (detección regresiones) | 🟡 | ✅ |
| `TSK-78` | Property-based testing expandido (proptest, boundaries) | 🟡 | ✅ |
| `TSK-79` | Benchmark regression alerts como gate de CI | 🟡 | ✅ |
| `TSK-37` | Benchmark calidad híbrida (NDCG/MRR/Recall@k) | 🟡 | ✅ |
| `TSK-97` | Hardening: eliminación de panics en runtime | 🟡 | ✅ |
| `DISC-02` | Test file locking con antivirus/backup en Windows | 🟡 | ✅ Simulación FILE_SHARE_READ/DELETE + stale lock 2026-06-20|
| `DISC-03` | Validar prefetch en SSDs rápidos (no degrade) | 🟢 | ✅ PrefetchMode config + env-var gating 2026-06-20|
| `TSK-93` | Prometheus completo (/metrics, histogramas p50/p95/p99) | 🟡 | ✅ |
| `TSK-94` | Logging estructurado JSON (tracing, levels) | 🟡 | ✅ |
| `ROAD-06` | Grafana Dashboard (plantilla oficial Prometheus) | 🟡 | ✅ Done 2026-06-20 |
| `TSK-67` | GraphRAG docs: ejemplo + benchmark reducción tokens | 🟠 | ✅ |
| `TSK-70` | Documento de garantías de durabilidad | 🟠 | ✅ |
| `TSK-80` | Migration guide ChromaDB y LanceDB | 🟠 | ✅ |
| `TSK-81` | README badges (CI, PyPI, Downloads, License) | 🟡 | ✅ |
| `AUD-05` | Reparar broken links en READMEs; → ✅ 18 links reparados en README.md + README_ES.md: CONTRIBUTING/SECURITY/SUPPORT → `.github/`, PYTHON_SDK.md → `docs/api/`, BENCHMARKS.md → `docs/operations/`, MEMORY_MVP_BASELINE.md removido (archivo eliminado). | 🔴 | ✅ |
| `AUD-06` | Fix referencia caída en DURABILITY_GUARANTEES.md; → ✅ `chaos_testing.rs` → `chaos_integrity.rs` en `DURABILITY_GUARANTEES.md:287` | 🔴 | ✅ |
| `AUD-07` | Fix `README.MD` uppercase en README_ES.md; → ✅ `README.MD` → `README.md` en `README_ES.md:24` | 🔴 | ✅ |
| `AUD-WORK` | Fix de CI y Auditoría de Workflows; → ✅ Corregidas exclusiones de nextest a nivel workspace, declaración de tests en Cargo.toml, clasificación de mcp_tests/tokenizer y features en CI. | 🔴 | ✅ |
| `DRV-001` | Refactor search.rs god file (1162L→845L, 5 sub-modules). phrase.rs + snippet.rs + debug.rs + text_index.rs. 22 unit tests nuevos. | 🟡 | ✅ |
| `AUD-08` | Auditar 33 bloques `unsafe`; Auditoría completada: 39 ítems unsafe (33 bloques, 4 impls, 1 pub fn, 1 extern fn). → ✅ 77% low-risk (mmap/FFI), 20.5% medium (from_raw_parts), 2.6% high (`pub unsafe fn release_mmap_vector`). Reporte completo en artifact del agente. | 🟡 | ✅ |
| `AUD-09` | Eliminar estado mutable global en tests; → ✅ `static TEST_RESULTS` eliminado, `static MULTI_PROGRESS` migrado a `thread_local!` + `RefCell`. Compilación limpia. | 🟡 | ✅ |
| `AUD-10` | Fix `set_var`/`remove_var` sin restore; → ✅ Variables de entorno se guardan/restauran en prefetch_benchmark.rs usando `var_os()` + `set_var`/`remove_var`. | 🟡 | ✅ |
| `AUD-11` | Agregar failure messages a ~50 bare assertions; → ✅ basic_node.rs (6), benchmark_internal.rs (1), test_sdk.py (~85), mcp_tests.rs (58), mcp_integration.rs (3). Total: ~153 assertions con mensajes descriptivos. | 🟡 | ✅ |
| `AUD-12` | Seedear generadores aleatorios en benchmarks; → ✅ hnsw_recall.rs + prefetch_benchmark.rs migrados a `StdRng::seed_from_u64(42)`. Benchmarks ahora reproducibles. | 🟡 | ✅ |
| `AUD-13` | Usar temp dirs en vez de paths hardcodeados; → ✅ `basic_node.rs` migrado a `TempDir`, `benchmark_internal.rs` usa `dir.path().join()`. `tempfile` ya era dev-dependency. | 🟡 | ✅ |
| `AUD-14` | Forwardear `ttl_ms` en Python wrapper; → ✅ `AsyncVantaDB.put()` ahora acepta `ttl_ms: int \| None = None` y lo forwardea al core Rust. Sin cambios del lado Rust (ya lo soportaba). | 🟡 | ✅ |
| `AUD-15` | Fix conflicto semver `tower 0.4` vs `0.5`; → ✅ Dev-dependency `tower` actualizado de `"0.4"` a `"0.5"` en Cargo.toml. `cargo tree -i tower` ahora muestra solo `tower v0.5.3`. | 🟡 | ✅ |
| `AUD-16` | Remover 3 stale advisory ignores en deny.toml; → ✅ `ignore` vaciado (RUSTSEC-2025-0119, 2026-0176, 2026-0177). `cargo deny check` → OK. | 🟡 | ✅ |
| `AUD-17` | Alinear rust-toolchain.toml con CI; → ✅ `channel = "1.94.1"` → `channel = "stable"`. Components/targets ya alineados. | 🟡 | ✅ |
| `AUD-18` | Agregar ejecución de tests en Windows CI; → ✅ Agregado step `Run tests (Windows)` con `cargo test --workspace` + timeout 30min en rust_ci.yml. | 🟡 | ✅ |
| `AUD-19` | Agregar `-L` a curl en install.sh; → ✅ `curl -s` → `curl -sL` en `scripts/install.sh:35`. El download binario ya tenía `-L`. | 🟡 | ✅ |
| `AUD-20` | Agregar detección `aarch64`/`arm64` en install.sh; → ✅ Detección en 2 etapas: normalize arch (`x86_64`→`amd64`, `aarch64`→`arm64`), luego compone suffix. Unknown arches hacen hard-fail. | 🟡 | ✅ |
| `AUD-21` | Crear o remover ref a `ROADMAP.md` en CHANGELOG; → ✅ Referencia removida de CHANGELOG.md:168, reemplazada con `<!-- TODO: create docs/operations/ROADMAP.md -->`. | 🟡 | ✅ |
| `AUD-22` | Manejar error de rate limiter en executor.rs; → ✅ `governor.request_allocation()` ahora propaga error via `?` en vez de `let _ =`. | 🔵 | ✅ |
| `AUD-23` | Manejar errores de flush/eviction en storage.rs + sdk.rs; → ✅ 4 sitios: flush/evict ahora logean warning con `tracing::warn!` en vez de `.ok()` silencioso. | 🔵 | ✅ |
| `AUD-24` | Refactorizar `compact_layout_bfs()` (247 líneas); → ✅ Dividida en 3 helpers: `traverse_graph()` (31L), `compact_layout()` (135L), `reindex_nodes()` (7L). Original: 249L → 53L orchestrator. | 🔵 | ✅ |
| `AUD-25` | Refactorizar `add()` (214 líneas); → ✅ Dividida: `validate_node()` (27L), `insert_hnsw()` (172L), `update_metadata()` (8L). `add()` ahora es dispatcher de 8 líneas. | 🔵 | ✅ |
| `AUD-26` | Refactorizar `open_with_config()` (266 líneas); → ✅ Dividida en 4 helpers: `init_storage`, `init_indexes`, `recover_state`, `init_wal`. Función original 271L → 59L de pipeline. | 🔵 | ✅ |
| `AUD-27` | Warnear backend string inválido en Python; → ✅ `_` arm dividido: `Some(other)` logea `tracing::warn!()`, `None` silencioso. | 🔵 | ✅ |
| `AUD-28` | Warnear `distance_metric` inválido en Python; → ✅ Misma división `Some(other)`→`tracing::warn!`, `None`→silencioso. | 🔵 | ✅ |
| `AUD-29` | Unificar repo URLs: `ness-e/Vantadb` vs `DevPness/Vantadb`; → ✅ 6 archivos migrados de `DevPness` a `ness-e`. Canonical: `ness-e/Vantadb`. | 🔵 | ✅ |
| `AUD-30` | Reemplazar `sleep(0.01)` por retry loop; → ✅ `_wait_until()` helper con timeout 5-10s. Eliminados 2 `time.sleep(0.01)` en test_lazy_eviction + test_purge_expired. 34 tests pasan. | 🔵 | ✅ |
| `AUD-31` | Feature-gate `arrow`, `rocksdb`, `fjall` opcionales; → ✅ 3 deps marcadas `optional = true`, features con `dep:` syntax, imports gated con `#[cfg(feature)]`. Default features incluyen las 3 (backward compatible). | 🔵 | ✅ |
| `AUD-32` | Fix `actions/checkout@v4` → `@v6` en nightly_bench.yml; → ✅ `@v4` → `@v6` en nightly_bench.yml:23. `upload-artifact@v4` ya era consistente. | 🔵 | ✅ |
| `AUD-33` | Fix `install-action@nextest` → `@v2`; → ✅ `taiki-e/install-action@nextest` → `@v2` con `tool: nextest` en heavy_certification.yml:274. | 🔵 | ✅ |
| `AUD-34` | Actualizar commit count en progreso docs; → ✅ `237 commits` → `460 commits` (git rev-list --count HEAD). | 🔵 | ✅ |
| `AUD-35` | Reemplazar 8 sleeps temporales con retry loops; → ✅ `e2e.rs:33` (wait_for_port), `e2e.rs:211` (JoinHandle::await), `server.rs:338` (wait_for_port), `e2e.rs:260` (justificado con comentario, rate limiter). 4 sleeps eliminados/reemplazados. | 🔵 | ✅ |
| `AUD-36` | Failure message + remover assertion temporal en basic_node.rs:189; → ✅ `assert!(true)` ya no existía. Agregado mensaje a `assert_eq!(engine.node_count(), 10_000, ...)`. | 🔵 | ✅ |
| `AUD-37` | Agregar ~15 edge case tests faltantes; → ✅ Archivo `tests/edge_cases.rs` creado con 25 tests cubriendo 17 categorías: NaN/Inf, empty key/batch/namespace, delete nonexistent, unicode metadata, zero-dim, all-zeros, WAL failure, concurrent, timeout, dim mismatch, large metadata, TTL, cross-namespace, duplicate ID, update nonexistent. Todos pasan. | 🔵 | ✅ |
| `AUD-38` | Feature flags granulares de tokio; → ✅ Root Cargo.toml: `"full"` → `["rt", "rt-multi-thread", "net", "sync", "signal", "macros"]`. vantadb-server dev-deps: `"full"` → `["rt", "rt-multi-thread", "net", "sync", "time", "macros"]`. | 🔵 | ✅ |
| `AUD-39` | Aflojar pin exacto `wide = "=1.2.0"`; → ✅ `=1.2.0` → `>=1.2, <2`. | 🔵 | ✅ |
| `AUD-40` | Workspace inheritance para version en Cargo.toml; → ✅ `[workspace.package]` creado con version/edition. 3 sub-crates migrados a `version.workspace = true`. | 🔵 | ✅ |
| `AUD-41` | Fix `pyo3/maturin-action@v1` pin vago en python_wheels.yml; → ✅ `@v1` → `@v2`. Nota: `maturin-action` actualmente no tiene tag `v2` — resuelve cuando el mantenedor lo publique. | 🟡 | ✅ |
| `AUD-42` | Agregar build de `vantadb-mcp` en release.yml; → ✅ `-p vantadb-mcp` agregado al build, rename+hash+attest+release glob incluido para las 3 plataformas. | 🟡 | ✅ |
| `AUD-43` | Agregar swap space en nightly_bench.yml; → ✅ Free disk space + 6GB swap agregados (mismo patrón que rust_ci.yml). | 🔵 | ✅ |
| `AUD-44` | Unificar `setup-python@v5` → `@v6` en nightly_bench.yml; → ✅ `@v5` → `@v6` en nightly_bench.yml:56. | 🔵 | ✅ |
| `TSK-45` | Publicar core en crates.io + docs.rs | 🔴 | ✅ |
| `TSK-106b` | SECURITY.md + vulnerability disclosure (90 días) | 🔴 | ✅ |
| `TSK-71` | WASM build (wasm32-wasi, re-priorizado desde ROAD-01) | 🔴 | ✅ |
| `TSK-112` | TS SDK vía WASM (core→wasm32-wasi, wrapper, npm) | 🔴 | ✅ |
| `TSK-113` | TS types + docs (intellisense, quickstart Node/Bun/Deno) | 🟠 | ✅ |
| `TSK-118` | Ejemplos TS con LangChain.js, LlamaIndex.TS, Vercel AI SDK | 🟠 | ✅ |
| `TSK-111` | Filtros metadata expandidos ($eq, $or, $in, $exists...) — ❌ Solo documentado, engine tiene operadores pero SDK no los expone | 🟡 | ❌ |
| `WASM-02` | OPFS persistence for WASM browser storage | 🔴 | ✅ |
| `WEB-07`  | Frontend test infra (Vitest + RTL + Playwright) | 🔴 | ✅ |
| `TEST-01` | WASM test suite (45 tests, wasm_tests.rs) | 🔴 | ✅ |
| `TEST-02` | Frontend component tests (23 tests, 3 files) | 🔴 | ✅ |
| `TEST-03` | Security test suite (30 tests: IQL injection, auth, fuzzing) | 🔴 | ✅ |
| `PERF-01` | Batch KV loader get_many + 5 N+1 refactors | 🔴 | ✅ |
| `SEC-03`  | Physical storage schema evolution + migration CLI | 🔴 | ✅ |
| `INV-005-A` | error.tsx App Router + drop dep muerta @mdxeditor/editor (Task 35, `6d0b84ec`) | 🟡 | ✅ 2026-08-05 |
| `INV-013-B` | JSON-LD schema.org/SoftwareApplication en layout root (Task 36, `1d072f4a`) | 🟢 | ✅ 2026-08-05 |
| `INV-015-B` | Touch targets clear-search 44px + iconos X h-5 (Task 38, `532788d2`) | 🟢 | ✅ 2026-08-05 |
| `INV-014-B` | Eliminar plomería dark inerte (theme-provider/theme-toggle/next-themes) (Task 37, `6e7b91b8`) | 🟢 | ✅ 2026-08-05 |
| `INV-016-B` | Motion tokens duration/ease reemplazan cubic-bezier (Task 39, `6afb37c3`) | 🟢 | ✅ 2026-08-05 |
| `GH-140` | Auditar + eliminar CSS no usado (−23.6%, 17 selectores + 3 keyframes) (Task 40, `21e6c58a`) | 🟢 | ✅ 2026-08-05 |
| `NUEVO-01` | README hero + benchmark graphic SIFT1M; GIF documentado (Task 41, `df1f84cc`) | 🟢 | ✅ 2026-08-05 |
| `GH-132` | Notebook Colab + badge Open in Colab (Task 42, `45c02e82`) | 🟢 | ✅ 2026-08-05 |
| `GH-131` | README integración mem0 (Task 43, `4ff2010a`) | 🟢 | ✅ 2026-08-05 |
| `GH-129` | README integración Semantic Kernel (Task 43, `4ff2010a`) | 🟢 | ✅ 2026-08-05 |
| `GH-128` | README integración DSPy (Task 43, `4ff2010a`) | 🟢 | ✅ 2026-08-05 |
| `INV-025` | Scoping Search Quality v2 (SEARCH_QUALITY_V2_SCOPING.md, contrato INV-009-B) (Task 44, `023d6e89`) | 🟡 | ✅ 2026-08-05 |
| `INV-009-B` | Phrase queries `Condition::TextMatch` + highlight contiguo (Task 45, `995258e9`) | 🟡 | ✅ 2026-08-05 |
| `INV-008-B` | `search_batch_requests` con SearchRequest completo (Task 46, `90fd3532`) | 🟡 | ✅ 2026-08-05 |
| `INV-007-B` | competitive_benchmark.json + competitive-table web (MKT-17) (Task 47, `58061ab8`) | 🟡 | ✅ 2026-08-05 |
| `NUEVO-16` | PQ viabilidad defer (REC-009 reafirmado, PQ_FEASIBILITY.md) (Task 48, `241a1d81`) | 🔵 | ✅ 2026-08-05 DEFER |
| `NUEVO-22` | Sparse indexed search (inverted index + posting lists) (Task 49, `5e71b5ff`) | 🔵 | ✅ 2026-08-05 |
| `ERR-042` | Perf-Search: `read_header` 2× por candidato en hot loop (+ entry points) → `node_header` leído 1× y reutilizado en distance + tombstone eligibility; fix `e95dd94a`; 2 tests paridad vfile vs in-memory + tombstone header excluido (commit `5a9eada1`); bench `vfile_search`: with_vfile 211→187ms (−11.4%), with_vfile_compacted 201→163ms (−19.0%); nextest 1902/1902 | 🟠 | ✅ 2026-08-11 |
| `ERR-043` | Perf-HNSW: `shrink_neighbors` clonaba vector del nodo (`as_f32_slice().map(to_vec)`) solo para usarlo como query → `compute_shrunk_neighbors` extraído, lee slice prestado (`as_f32_slice()`) sin alloc O(vec_len); fix `2a20b14a`; 3 tests shrink/paridad (INV-024 reachability, AUD-014 tie-break, search parity); nextest 1902/1902 | 🟡 | ✅ 2026-08-11 |
| `ERR-045` | Perf-HNSW: `get_neighbors` clonaba lista por nodo en BFS de compactación (`serialization_order`, usado por `serialize_to_bytes`) → `get_neighbors_ref` (borrow DashMap, sin clone) elimina O(N×M) allocs; fix `0b2e9d99`; paridad compactación: `serialization_order_preserves_search_results` + roundtrips 74/74; nextest 1902/1902, fmt + clippy -D warnings ✅ | 🟡 | ✅ 2026-08-11 |
| `ERR-044` | Perf-Tokenizer: `TextAnalyzer` reconstruido por llamada (batch N pagaba N setups stemmer/stopwords) → `build_advanced_analyzer` + `tokenize_with_analyzer` + `record_terms_with_analyzer`, hoisting 1 build por batch en `rebuild_text_index_with_report`; fix core `82ec9882`; paridad `test_analyzer_reuse_matches_fresh_build` + `test_record_terms_with_analyzer_matches_record_terms`; bench `batch_reuse_vs_fresh` (`6ebfe52c`, Cargo.toml [[bench]] harness=false): 917µs fresh → 641µs reuse (~30%); nextest 1902/1902, fmt + clippy -D warnings ✅ | 🟡 | ✅ 2026-08-11 |
| `ERR-015` | Desktop-Shutdown: `request_shutdown` mataba con `kill()` sin señal graciosa SIGINT (metadata loss Windows) → stdin EOF + timeout gracioso antes de kill en `child_process.rs`; fix `63e0f9ec` + `704f2a67`; docs `efdff368` | 🟢 | ✅ 2026-08-11 |
| `ERR-026` | MCP-Filtros: `parse_metadata` descartaba arrays/objetos/null → filtro no aplicado → resultados súper-conjunto → `Result<VantaMemoryMetadata, McpError>` con `invalid_params` nombrando key y tipos soportados; call sites (memory_put/list, search_memory) actualizados; fix `ce265569` + follow-up `aa1754d2` (delegar list/null a core); tests unit + integración | 🟢 | ✅ 2026-08-11 |
| `ERR-032` | Storage-Test: test de `deserialize_node_payload` removido → re-añadidos `_rejects_malformed_input` + `_cap_guard_rejects_and_ok_within_cap` en `src/storage/ops.rs:215,244` (cubre guard MAX_PERSISTED_NODE_BYTES) | 🟢 | ✅ 2026-08-12 |
| `ERR-033` | MCP/List: `memory_list(limit=0)` devolvía 1 por `max(1)` → short-circuit `limit==0` → página vacía (evita full-scan fallback); fix `fde27213`; test `test_list_zero_limit_returns_no_records` | 🟢 | ✅ 2026-08-11 |
| `ERR-047` | Perf-Search: copy inline en cada pop del hot loop (`take_l + extend`) → `Cow<NeighborVec>` Borrowed (inline cache, DashMap vivo durante loop) / Owned fallback; elimina 1 alloc+copy O(M) por candidato; fix `01630996`; search 128/128, index 251/251 | 🟢 | ✅ 2026-08-11 |
| `ERR-048` | Perf-Search: `visited.contains` + `visited.insert` (2 hash lookups) → `insert` solo (devuelve bool); aplicado en expansión principal + ACORN-1 second-hop; fix `a983d4e0`; search 128/128 | 🟢 | ✅ 2026-08-11 |
| `ERR-006` | Deps: deny.toml ignore RUSTSEC-2024-0436 stale ("advisory-not-detected") → removido + comentario "re-add only if it reappears"; RUSTSEC-2026-0002 documentado (lru 0.12.5 no en default resolve) | 🟢 | ✅ 2026-08-12 |
| `ERR-008` | Storage: `copy_unsafe` en vfile sin guard explícito de bounds → función ya no existe (removida; solo wrappers mmap con `// SAFETY` en vfile.rs) — obsoleta | 🟢 | ✅ 2026-08-12 |
| `ERR-009` | Tooling: correr `cargo miri test` (tree-borrows) sobre vfile/ops → job Miri (UB Detection) ya presente en CI `ci-rust-10.yml:457` + `ci-gate.yml:40` — cubierto | 🟢 | ✅ 2026-08-12 |
| `ERR-049` | Bench: sin bench dedicado a `ivf.rs` → `ivf_bench.rs` creado + registrado en Cargo.toml:247 `[[bench]]` | 🟠 | ✅ 2026-08-12 |
| `ERR-007` | Deps: `multiple-versions` "warn" activo (hashbrown ×4 majors, windows-sys ×4, syn 2/3, thiserror 1/2, rand 0.8/0.9/0.10) → splits de major version de deps transitivas, no unificables con `cargo update` (cada caller exige su major). Resolución: `[bans] skip` documentado en deny.toml con justificación por crate; `cargo deny check bans` → `bans ok`, exit 0 | 🟠 | ✅ 2026-08-12 |
| `PERF-01` | Bench-Honestidad: re-validar claims de rendimiento del README ("100k docs en 0.6s") contra benches reales → `30e90cd9` bench revalidado; `docs/benchmarks/` actualizado; claims soportadas por metodología+HW | 🟡 | ✅ 2026-08-12 |
| `PERF-04` | Index-Prefetch: flag config `prefetch` para HNSW, default OFF (ocultaba latencia real con `fnv1a` eager en put) → `152ddd26` perf(index) gate prefetch behind config flag, default off | 🟢 | ✅ 2026-08-12 |
| `PERF-06` | Config: `VANTADB_MEMORY_LIMIT` con parse humano KB/MB/GB → `914514bb` feat(config) suffixes en memory limit + `d9378656` (completions regeneradas) | 🟢 | ✅ 2026-08-12 |
| `PERF-07` | Perf-Serialization: JSON sparse parseado en cada read/write con `.ok()` tragando errores → parse explícito + log de corrupción en vez de `None` silencioso → `88b0f875` perf(PERF-07) explicit sparse parse | 🟢 | ✅ 2026-08-12 |
| `PERF-09` | Cold-start: `_force_copy` muerto (¿MmapFull real o corregir log?) → decisión: log honesto + nota legacy `force_copy` → `0be56cac` docs(PERF-09) honest cold-start log | 🟠 | ✅ 2026-08-12 |

### Julio 2026 — Auditoría de Código (2ª pasada)

| ID | Tarea | Prioridad | Estado |
|----|-------|-----------|--------|
| `AUD-01` | 🔴 OTel startup `expect()` panics if endpoint unreachable (`cli_server.rs:366`) | 🔴 | ✅ |
| `AUD-02` | 🔴 `unwrap()` on Option in mmap hot path (`storage.rs:572,629`) | 🔴 | ✅ |
| `AUD-03` | 🔴 `from_raw_parts` sin bounds check en hot path (`index.rs:1420,1701`) | 🔴 | ✅ |
| `AUD-04` | 🔴 Cast unsafe sin verificación de alineación (`rkyv_archives.rs:54-71`) | 🔴 | ✅ |
| `AUD-05` | 🔴 `.ok()` silencia errores UTF-8 en parsing de claves (`sdk.rs:1351-1362`) | 🔴 | ✅ |
| `AUD-06` | 🔴 N+1 query: `scan_nodes()` parsea metadata directo del scan, evita 1+N gets (`storage.rs:2271`) | 🔴 | ✅ |
| `AUD-07` | 🔴 `ensure_indexes_current` unifica 3 scans en 1 (`sdk.rs:1495`) | 🔴 | ✅ |
| `AUD-08` | 🔴 `memory_record_to_node_owned` reduce clones en `put()` (`sdk.rs:768`) | 🔴 | ✅ |
| `AUD-09` | 🟡 4 dead CLI handlers removidas + rustyline+strsim eliminados de Cargo.toml | 🟡 | ✅ |
| `AUD-10` | 🟡 `mapped_file_resident_bytes()` removida (`storage.rs:346`) | 🟡 | ✅ |
| `AUD-11` | 🟡 `wal_path` asignado pero nunca leído (`engine.rs:55`) | 🟡 | ✅ |
| `AUD-12` | 🟡 3 unused deps: `anyhow`, `num-traits`, `color-eyre` | 🟡 | ✅ |
| `AUD-13` | 🟡 Config parse falla silenciosamente con env vars inválidas (`config.rs:179-293`) | 🟡 | ✅ |
| `AUD-14` | 🟢 39 `pub fn` sin doc comments (74% de `sdk.rs`) | 🟢 | ✅ |
| `AUD-15` | 🟢 6 broken links en Backlog.md (apuntan a `docs/` raíz, deben ser `docs/VantaDB-MPTS/`) | 🟢 | ✅ |
| `AUD-16` | 🟢 15 módulos sin tests unitarios (añadidos tests a error.rs y binary_header.rs: +19 tests) | 🟢 | ✅ |
| `AUD-17` | 🟢 Dead code en `utils/` (`DuplicatePreventionFilter`, `OriginCollisionTracker` — removidos de re-exports públicos) | 🟢 | ✅ |
| `AUD-18` | 🟢 `#[allow(dead_code)]` obsoleto en `physical_plan.rs:query_vec_text` (falso positivo: condicionado a `remote-inference`) | 🟢 | ✅ |
| `TSK-119` | `delete_by_filter()` — eliminar por metadata — ❌ Era solo CLI handler, eliminado en AUD-09. Nunca fue SDK | 🟡 | ❌ |
| `TSK-86` | `similar_to_key()` — buscar similares a existente — ❌ Nunca implementado en ningún lenguaje | 🟡 | ❌ |
| `TSK-87` | `count()` con filtros — ❌ Era solo CLI handler, eliminado en AUD-09. Nunca fue SDK | 🟡 | ❌ |
| `TSK-88` | Multi-namespace search (buscar en N namespaces) — ❌ Nunca implementado. Siempre fue `namespace: &str` singular | 🟡 | ❌ |
| `COM-02` | CONTRIBUTING.md (entorno, tests, conventional commits) | 🔴 | ✅ (exists in `.github/`) |
| `COM-03` | Code of Conduct (Contributor Covenant) | 🔴 | ✅ (exists in `.github/`) |
| `CLI-EPIC` | CLI Polish completo | 🔴 | ✅ |
| `TSK-101` | ARM64 Linux wheels (experimental → estable) | 🟠 | ✅ |
| `TSK-102` | Python 3.13+ support en CI matrix | 🟡 | ✅ |
| `TSK-100` | Homebrew formula macOS (`brew install vantadb`) | 🟡 | ✅ |
| `TSK-35` | Suite de ejemplos Rust (basic, hybrid, graphrag, concurrent) | 🟡 | ✅ |
| `TSK-34` | Reorganización docs por audiencia (getting-started/guides/api) | 🟡 | ✅ |
<!-- dedup: filas DISC-01/04/06/07/08/09/10 cubiertas por tabla "DISC Discoveries Completed" -->
| `AUD-WORK` | CI fixes (nextest workspace exclusions, test declarations, heavy_cert classification, numpy venv, version extraction) | ✅ 8/9 hallazgos: 9/9 resueltos (último: test-threads Windows-específico ✅) |
| `TSK-126` | Agregar `impl Drop for StorageEngine` para liberación explícita del lock | 🟡 | ✅ |
| `TSK-128` | Hacer configurable el timeout de `insert_lock` | 🟡 | ✅ |
| `TSK-129` | Hacer configurable el timeout de `.vanta.lock` | 🟡 | ✅ |
| `TSK-130` | Agregar instrumentación de heap memory drift (jemalloc stats) | 🟡 | ✅ |
| `TSK-134` | Fix `release.yml:73` — swap validado, sin cambios | 🔴 | ✅ |
| `TSK-135` | Fix `python_wheels.yml:60` — `dtolnay/rust-toolchain@master` → `@stable` | 🟡 | ✅ |
| `TSK-136` | Fix `nightly_bench.yml:117` — `GITHUB_SHA` propagado a `github-script` | 🟡 | ✅ |
| `TSK-137` | Agregar swap en macOS/Windows para release builds | 🟡 | ✅ |
| `TSK-138` | Eliminar double checkout en `heavy_certification.yml` | 🟢 | ✅ |
| `TSK-139` | Eliminar stale path trigger `packages/**` en `rust_ci.yml` | 🟢 | ✅ |
| `TSK-140` | Eliminado job arm64 con `if: false` en `python_wheels.yml` | 🟢 | ✅ |

### Descubrimientos DISC Completados

| ID | Descubrimiento | Resolución |
|----|---------------|------------|
| `DISC-01` | Validar ExecutionResult consumers | ✅ Verificado: todos los match arms cubren Read/Write/StaleContext |
| `DISC-04` | Chaos testing kill -9 durante writes | ✅ AUD-02 (10 iters) + AUD-03 (20 iters tight loop) |
| `DISC-06` | MCP prompts/list handler | ✅ Implementado |
| `DISC-07` | MCP ArcSwap API (hnsw.read()→hnsw.load()) | ✅ Corregido |
| `DISC-08` | Server test suite expandido | ✅ 14 tests (auth, rate-limit, TLS, concurrent) |
| `DISC-09` | Skills Python dependencies | ✅ Scripts funcionales en Windows |
| `DISC-10` | CLI commands server/search/delete/namespace | ✅ Resuelto (TSK-24/25/26/27) |
| `DISC-11` | Unificar binarios CLI+MCP+Server | ⏸️ Postpuesto (dependencia circular) |
<!-- dedup: AUD-WORK cubierta por fila en tabla "July 2026 — Code Audit (2nd pass)" -->
