# Serie OLD — exploración y fundaciones (chaos, snapshots, WAL, GraphRAG)

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

### 2026-07-26 — OLD-03: Chaos Testing — Harness de Failpoints Formal ✅

**Fuente:** Backlog Phase 9 (Old Docs Rescue) `OLD-03`

**Problema original:** Test `chaos_integrity_failpoints_certification()` existía con failpoints inline. Sin harness reutilizable, sin documentación, sin CI workflow.

**Resuelto por (vanta-chaos, ponytail):**
- `ChaosTestHarness` en `src/testing/chaos.rs`: setup/teardown automático, enable/disable/assert_recovery/destroy
- 6 escenarios: wal_append, storage_insert, mmap_flush, hnsw_serialize, edge_write (nuevo), snapshot_serialize (nuevo)
- `docs/chaos-testing.md` con patrón de failpoints, cómo agregar nuevos, cómo correr localmente
- Feature-gated `failpoints` — 0 overhead en builds productivos

**Verificación:** `cargo nextest run --features failpoints -p vantadb -- test_chaos` ✅ | `cargo check --features failpoints -p vantadb` ✅

**Ids:** `OLD-03`

**2026-07-27 — Correcciones post-certificación:**
- **Bug:** Test binary no terminaba después de `ok` — proceso colgado en limpiza.
- **Root cause 1:** `ChaosTestHarness` declaraba `dir: TempDir` antes que `engine: Arc<StorageEngine>`. Rust dropea struct fields en orden de declaración → `dir` se dropeaba primero e intentaba borrar el directorio temporal mientras el engine aún tenía archivos abiertos (Windows: no puede borrar archivos abiertos).
- **Root cause 2:** `with_global_bar()` llamaba `pb.enable_steady_tick(100ms)` que spawnea un thread background con su propio `Arc` al estado del ProgressBar. Cuando el test terminaba y los thread-locals se limpiaban, el thread sobrevivía y prevenía la salida del proceso.
- **Fix 1:** Reordenar campos `engine` → `dir` en struct (cambia orden de Drop).
- **Fix 2:** Remover `enable_steady_tick` de `with_global_bar()` y `create_progress()`. Cambiar draw targets a `hidden()`.
- **Commit:** `16e19434` (hang fix), `2812f9eb` (field order)
- **Verificación:** 15 test binaries corridos (22 tests individuales). 0 hangs. 3 fallas pre-existentes no relacionadas. ✅

### 2026-07-26 — OLD-08: Snapshots mediante Hard Links ✅

**Fuente:** Backlog Phase 9 (Old Docs Rescue) `OLD-08`

**Problema original:** `snapshot_certification.rs` existía, hard-link pattern POSIX no implementado.

**Resuelto por (vanta-worker):**
- `FsSnapshot` + `SnapshotManager` con hard-link POSIX (O(1) instantáneo)
- `StorageEngine::create_snapshot(name)` / `list_snapshots()` 
- `VantaEmbedded::create_snapshot(name)` en SDK público
- CLI `vantadb snapshot create <name>` + `vantadb snapshot list`
- +failpoint `snapshot_create_fail` para chaos testing
- Tests: instant, multiple snapshots, independence

**Verificación:** `cargo check -p vantadb` ✅ | `cargo nextest run -p vantadb -- snapshot` ✅

**Ids:** `OLD-08`

### 2026-07-26 — OLD-09: Olvido Bayesiano (Bayesian Hit Decay) ✅

**Fuente:** Backlog Phase 9 (Old Docs Rescue) `OLD-09`

**Problema original:** `EvictionPolicy` tenía hit counts + recency weights pero sin modelo probabilístico formal para decidir qué nodos evictar.

**Resuelto por (vanta-worker, ponytail):**
- `BayesianDecay` struct con modelo Beta-Binomial: `score = α/(α+β)` donde α = prior_alpha + hits, β = prior_beta + seconds_since_last_hit
- `EvictionPolicy` enum que envuelve `Weighted` (legacy) o `Bayesian` (nuevo)
- Threshold configurable (default 0.3) — scores por debajo → eviction candidate
- Feature-gated `bayesian_decay`
- 31 tests (boundary, param clamping, enum round-trip, weighted compat)

**Verificación:** `cargo check --features bayesian_decay` ✅ | `cargo test --features bayesian_decay -- eviction` ✅ 31/31 | `cargo clippy` ✅

### 2026-07-26 — OLD-11: CLI/TUI Interactivo ✅

**Fuente:** Backlog Phase 9 (Old Docs Rescue) `OLD-11`

**Problema original:** CLI completo existía (46 tests), TUI con spec de 1106 líneas no implementado.

**Resuelto por (vanta-worker):**
- `vantadb tui` subcomando con ratatui + crossterm, feature-gated `tui`
- 3 modos: Dashboard (node count, memory %, cache, evictions, backend type), Monitor (live queries scaffold listo para hookear tracing), REPL (input con historial up/down, scroll, `.help`/`.clear`/`.stats`, ejecución IQL)
- 744 líneas en 5 archivos nuevos: `src/tui/mod.rs`, `src/tui/dashboard.rs`, `src/tui/monitor.rs`, `src/tui/repl.rs`
- Abre DB en read-only safe

**Verificación:** `cargo check --features tui` ✅ | `cargo test --test cli_tests --features cli` ✅ 46/46

**Ids:** `OLD-11`

### 2026-07-26 — OLD-12: Programa Piloto Formal ✅

**Fuente:** Backlog Phase 9 (Old Docs Rescue) `OLD-12`

**Problema original:** `docs/operations/PILOT_PROGRAM.md` existía como spec de 3 secciones, no como programa ejecutable.

**Resuelto por (vanta-docs):**
- `docs/operations/PILOT_PROGRAM.md` actualizado de 3→9 secciones: overview, early adopter profile, mutual commitments, timeline 8 semanas con milestones, KPI table (retention/NPS/benchmarks)
- +3 templates: `pilot-agreement-template.md` (10 secciones, NDA 2 años), `pilot-feedback-template.md` (7 secciones, severity P0-P2, NPS), `pilot-onboarding-checklist.md` (6 fases con verification commands)

**Verificación:** 4/4 archivos OK

**Ids:** `OLD-12`

### 2026-07-26 — OLD-19: Rehidratación desde Shadow Archive ✅

**Fuente:** Backlog Phase 9 (Old Docs Rescue) `OLD-19`

**Problema original:** `StorageEngine::recover_archived_nodes(summary_id)` existía en `maintenance.rs` con 6 tests, pero no estaba expuesto al SDK público, MCP, ni Python. El MCP ya retornaba `rehydration_available: true` en respuestas `StaleContext`, pero no había tool para ejecutar la rehidratación.

**Resuelto por (vanta-worker, ponytail):**
- `VantaEmbedded::recover_archived_nodes(summary_id: u128)` en `src/sdk/builder.rs` — delega al engine, convierte `UnifiedNode` → `VantaNodeRecord`
- MCP tool `rehydrate` en `vantadb-mcp/src/lib.rs` — toma `summary_id` string, retorna `recovered_count` + `rehydration_complete: true`
- Python binding `recover_archived_nodes(summary_id: &str)` en `vantadb-python/src/lib.rs` — parsea u128, llama con GIL detach, retorna lista de dicts
- 2 tests SDK adicionales

**Verificación:** `cargo check -p vantadb && cargo check -p vantadb-mcp && cargo check -p vantadb_py` ✅ | `cargo nextest run --profile audit -p vantadb -- test_recover_archived` ✅ 7/7 | `cargo clippy` todos ✅

**Ids:** `OLD-19`

### 2026-07-26 — OLD-16: Rotación WAL a 256MB ✅

**Fuente:** Backlog Phase 9 (Old Docs Rescue) `OLD-16`

**Problema original:** `WalWriter` tenía `rotate()` consumidor (toma `self`), pero no había auto-rotación por tamaño. ShardedWal heredaba el problema — los segmentos WAL no tenían límite.

**Resuelto por (vanta-worker, ponytail):**
- `WalWriter` ahora tiene `max_segment_size: u64` (hard-coded 256MB)
- `try_auto_rotate(&mut self)` — flush → rename `vanta.wal` → `vanta.wal.<timestamp>` → fresh WAL con header → resetea contadores
- Llamado al final de `append()` y `batch_append()` después de `maybe_sync()`
- ShardedWal hereda gratis (sus métodos delegan a WalWriter)
- 3 tests: trigger (archive existe + bytes_written reset), not-before-limit, data preservation (records verificables via WalReader)

**Verificación:** `cargo nextest run --profile audit -p vantadb -- wal` ✅ 52/52 passed | `cargo check -p vantadb` ✅ | `cargo clippy -p vantadb -- -D warnings` ✅ | `cargo fmt --check` ✅

**Ids:** `OLD-16`

### 2026-07-26 — OLD-14: MessageThread / GcWorker para Chat Agentico ✅

**Fuente:** Backlog Phase 9 (Old Docs Rescue) `OLD-14`

**Problema original:** No existía una abstracción `MessageThread` para chats agentic. `GcWorker` existía en `src/gc.rs` (TTL GC) pero no se usaba para ciclos de vida de conversaciones.

**Resuelto por (vanta-worker, ponytail):**
- `src/agentic/` nuevo módulo con `mod.rs` + `thread.rs`
- `MessageThread` struct: `thread_id`, `title`, `messages: Vec<Message>`, `created_at`, `updated_at`, `metadata`
- `Message` struct: `role` (system/user/assistant/tool), `content`, `timestamp`, `metadata`
- `ThreadStore` con CRUD completo sobre `StorageEngine` + opcional TTL via `GcWorker`
- 6 métodos expuestos vía `VantaEmbedded`: `create_thread`, `send_message`, `get_thread`, `list_threads`, `delete_thread`, `purge_expired_threads`
- 6 tests en `tests/message_thread_test.rs` incluyendo TTL expiry

**Verificación:** `cargo nextest run --test message_thread_test` ✅ 6/6 pass | `cargo check -p vantadb` ✅ | `cargo clippy -p vantadb -- -D warnings` ✅

**Ids:** `OLD-14`

### 2026-07-26 — OLD-02: Pipeline Formal de GraphRAG — seed → expand → retrieve → generate context ✅

**Fuente:** Backlog Phase 9 (Old Docs Rescue) `OLD-02`

**Problema original:** Existía un `examples/rust/graphrag.rs` que usaba la API raw de Node/Graph (insert manual → BFS), sin un pipeline formal con seed → expand → retrieve → context generation.

**Resuelto por (vanta-lead, vanta-worker, ponytail):**
- Pipeline completo en `src/graphrag/`: `mod.rs`, `pipeline.rs`, `seed.rs`, `expand.rs`, `retrieve.rs`, `context.rs`
- `GraphRagPipeline` struct con defaults: `seed_k=10`, `hops=2`, `max_expansion_nodes=100`, `top_k=20`
- `GraphRagResult` con campos: `nodes`, `edges`, `context_text`, `stats`
- Método SDK `VantaEmbedded::graphrag_search(namespace, query, query_vector)` agregado
- 4 tests en `tests/graphrag_test.rs` (simple_search, empty_result, hybrid_fallback, max_expansion) — **4/4 pass**
- `docs/api/GRAPH_RAG.md` — API reference completa con Rust/Python usage
- Task file creado en `.opencode/skills/campaign-executor/tasks/OLD-02.md`

**Pendiente:** `examples/rust/graphrag.rs` aún usa API raw (no pipeline), falta `examples/python/graphrag_pipeline.py`

**Verificación:** `cargo nextest run --test graphrag_test` ✅ 4/4 pass | `cargo check -p vantadb` ✅

**Ids:** `OLD-02`
