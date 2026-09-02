# Changelog

All notable changes to the VantaDB engine will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [Unreleased]

### ✨ Features

- **Fase 4 Vanta Studio (2026-08-20, 18/18):** consola standalone 100% browser (WASM/OPFS con persistencia y reload), import drag&drop `.vdbdump`/JSONL/CSV, slider de pesos híbridos BM25/vector (RRF weighted client-side), superficie Índices/salud real, consolidación asistida con diff visible, y supersession durable en core (ADR-028): `VantaMemoryRecord.superseded_by`/`superseded_at_ms`, `supersede()`, filtro `exclude_superseded` en search/list (core + Python sync/async + export/import JSONL).

### Documentation

- **ERR-DOCS-01:** New [`docs/api/ERROR_HANDLING.md`](api/ERROR_HANDLING.md) — canonical reference for `VantaError` (Rust), 10-code `ERROR_CODES` contract (TypeScript/WASM), Python `VantaError` hierarchy (10 subclasses), and MCP JSON-RPC error mapping (5 std factories + 9 Vanta custom `-320xx` codes). Documents `is_retriable()` matrix, `recovery_hint()` guide, and `.to_dict()` cross-binding serialization. Provisional table pending `pub fn code()` from `ERR-CORE-01` (codes will gain `VANTADB_` prefix).
- **ERR-DOCS-01:** [`docs/api/TS_SDK.md`](api/TS_SDK.md) — expanded Error Handling section with full `ERROR_CODES` table, `VantaError` class shape, `wrapWasmError` semantics, and `CLOSED` lifecycle error.
- **ERR-DOCS-01:** [`docs/api/MCP.md`](api/MCP.md) — new JSON-RPC codes section (5 std factories + 9 Vanta `-320xx` codes + response envelope with `data.code`).
- **ERR-DOCS-01:** [`docs/api/EMBEDDED_SDK.md`](api/EMBEDDED_SDK.md) — expanded Rust `VantaError` variants with `is_retriable()` matrix and `recovery_hint()` guide.
- **ERR-DOCS-01:** [`docs/api/PYTHON_SDK.md`](api/PYTHON_SDK.md) — added `.code`, `.retriable`, `.details`, `.hint`, `.to_dict()` attributes to all 10 subclasses.

### Fixed

- **ts/wasm/node (ERR-TS-01):** error codes unified with the canonical `VANTADB_*` set from `VantaError::code()` (ERR-CORE-01). **BREAKING (0.x):** `vantadb-ts` `VantaError.code` wire values gained the `VANTADB_` prefix (keys unchanged: `ERROR_CODES.BUSY === "VANTADB_BUSY"`); `vantadb-wasm` emits core codes directly (duplicated 30→8 table removed); `vantadb-node` propagates the code as a `"{CODE}: {message}"` prefix recovered by `wrapNativeError` (invented `NATIVE_ERROR` code deleted); `validateVector` now throws `VantaError(VANTADB_VALIDATION_ERROR)` instead of `TypeError`/`RangeError`; `wrapWasmError`/`wrapNativeError` set `cause` (ERROR_HANDLING.md §4.3).
- **server:** cierre de deuda REST — `/api/v2/metrics` JSON operacional, graph_v2 con ids u128-safe (string wire), paginación cursor, IQL completo vía `/api/v2/query` (SELECT/INSERT + roundtrip graph).

### Other

- **core:** Bench(MKT-16): GraphRAG benchmark reproducible + metodologia (numeros indexacion reales; query PENDING por stack overflow engine)


### ♻️ Refactoring

- **AUD-004:** Refactor(AUD-004): renombrar tool MCP query_lisp a query_iql

- **INV-014-B:** Refactor(INV-014-B): eliminar plomería dark inerte (next-themes)

- **INV-016-B:** Refactor(INV-016-B): motion tokens (duration/ease) reemplazan cubic-bezier

- **GH-140:** Refactor(GH-140): eliminar selectores huérfanos probados de globals.css

- **index:** Refactor(index): replace MetricCache OnceLock with plain const (AUDREP-53)

- **frontend:** Refactor(frontend): remove dead hero variant toggle + unreachable cat branch (AUDREP-57)

- **frontend:** Refactor(frontend): extract shared tokenizer (AUDREP-58)


### ⚡ Performance

- Perf: add sparse hot-path benchmark and close AUDIT-02 as WONTFIX (measurement gate)

- **mcp:** Perf(mcp): bound memory in collection_stats/collection_list streaming aggregates

- **sdk:** Perf(sdk): stop materializing dead vector/sparse_vector in purge_expired (AUDREP-54)


### 🐛 Bug Fixes

- **gate:** Fix(gate): reusable workflow must not request contents: read — caller grants only checks: read; docs for ci-gate/chaos + gate sections

- Fix: put_batch rebuilds derived and text indexes after batch insert

- **index:** Fix(index): bound IVF deserialize counts against input length

- **ci:** Fix(ci): activate prometheus feature for server e2e metrics test

- **opencode:** Fix(opencode): add frontmatter to campaign-executor SKILL.md

- **web:** Fix(web) update product version to 0.5.0 (M6)

- **bindings:** Fix(bindings): harden native napi-rs backend + sync WASM temporal-edge types (COMP-029)

- **bindings:** Fix(bindings): drain in-flight ops on close for durable native backend (COMP-029)

- Fix: adapt graphrag test/example to 5-arg add_edge (COMP-021)

- **core:** Fix(core): enforce read_only on all storage mutations (COMP-029)

- **server:** Fix(server): adapt benchmarks/e2e/helpers tests to ENT-04 ServerState fields

- **task-system:** Fix(task-system): make harness parser tolerant to **Estado:** and ## Tarea N: formats

- **WEB-18:** Fix(WEB-18): align web pricing with GO_TO_MARKET (drop Team \ tier)

- **TECH-02:** Fix(TECH-02): wrapper reindexHnswFromText usa export real del pkg

- **AUD-001:** Fix(AUD-001): subir MSRV y eliminar COPY a crates inexistentes en Dockerfile

- **TECH-01:** Fix(TECH-01): setear VANTADB_STORAGE_PATH en child MCP (--db respetado)

- **AUDIT-01:** Fix(AUDIT-01): congelar buffer __array_interface__ ante drop en PyO3

- **AUD-011:** Fix(AUD-011): portar OpGate a bindings python/wasm

- **AUDIT-04:** Fix(AUDIT-04): acotar cache_warmer.co_access para evitar OOM en searches

- **INV-015-B:** Fix(INV-015-B): touch targets clear-search a 44px

- **AUDREP-01:** Fix(AUDREP-01): validate truncated vstore before compact copy

- **AUDIT-03:** Fix(AUDIT-03): Miri guard core - tree-borrows sobre 7 bloques UB (INV-024)

- **AUDREP-02:** Fix(AUDREP-02): replace inalcanzable expect con let-else en scan_nodes_page

- **AUDREP-13:** Fix(AUDREP-13): log warning por request no autenticada en dev mode

- **AUDREP-08:** Fix(AUDREP-08): evitar colision de timestamps y rename no atomico en archive_segment

- **AUDREP-03:** Fix(AUDREP-03): no tragar errores de tombstone write_header en ops.rs

- **index:** Fix(index): estabilizar invariante HNSW concurrente con flush del batch pendiente

- **web:** Fix(web): resolver colision de tipos WASM en code-playground (H03-WEB)

- **python:** Fix(python): quitar hang de CI en test_load y migrar put_batch a kwargs

- **node:** Fix(node): regenerar package-lock desync de @emnapi/runtime (H02-L2-002)

- **bench:** Fix(bench): resolver errores clippy en graphrag_bench example

- **core:** Fix(core): query_sparse en adapter openai y MemoryGovernor limit 0 bajo Miri

- **providers:** Fix(providers): query_sparse en adapters ollama y litellm

- **langchain:** Fix(langchain): MMR tie truncation y add_documents vacío

- **adapters:** Fix(adapters): openai pytest-asyncio, dspy .passages, letta fallback listing

- **engine:** Fix(engine): realinear tokenizer de frases con postings del indice (regresion 995258e9)

- **security:** Fix(security): limit HTTP request body to 1MB in /api/v2/query (AUDREP-12)

- **security:** Fix(security): key stretching for Cipher key derivation (AUDREP-10)

- **security:** Fix(security): only trust X-Forwarded-For from configured proxy (AUDREP-11)

- **index:** Fix(index): clamp sq8 remainder loop to min len to prevent OOB panic (NV-01)

- **index:** Fix(index): invalidate cached IVF when node count changes (AUDREP-09)

- **wal:** Fix(wal): hold shard lock across rotate sync+swap to avoid lost writes (AUDREP-15)

- **integrations:** Fix(integrations): bump 9 python adapters to 0.5.0 and pin vantadb-py>=0.5.0,<0.6.0

- **wal:** Fix(wal): persist shard count in metadata, reconcile on reopen

- **wal-shipping:** Fix(wal-shipping): add shutdown signal and per-failure backoff to run_loop

- **storage:** Fix(storage): drop mmap before rename in save_vector_index on Windows

- **ts-sdk:** Fix(ts-sdk): accept numeric version/node_id/timestamps in isMemoryRecord

- **server-tests:** Fix(server-tests): add trusted_proxies field to AppState in tests

- **repo:** Fix(repo): unignore .env.example to track env template (AUDREP-24)

- **storage:** Fix(storage): saturating math in write_node_to_vstore growth (AUDREP-33)

- **index:** Fix(index): clamp euclidean distance to non-negative (AUDREP-28)

- **crypto:** Fix(crypto): bound EncryptionStream frame_len to prevent OOM (AUDREP-31)

- **wal:** Fix(wal): quarantine corrupt tail to .corrupt before truncation (AUDREP-36)

- **mcp:** Fix(mcp): RAII guard sole decrement for active_requests (AUDREP-44)

- **index:** Fix(index): reject zero-norm cosine inserts (AUDREP-27) + total-order NaN eviction (AUDREP-29)

- **wal:** Fix(wal): fsync parent dir after durability-critical renames (AUDREP-35)

- **server:** Fix(server): generic panic message to client, detail logged (AUDREP-32)

- **sdk:** Fix(sdk): post-filter cursor detection prevents infinite pagination loop (AUDREP-30)

- **wal:** Fix(wal): require timestamp in archive names, drop mtime fallback (AUDREP-37)

- **storage:** Fix(storage): saturating 64-byte alignment, no overflow (AUDREP-34)

- **frontend:** Fix(frontend): derive html lang from i18n DEFAULT_LANG, drop suppressHydrationWarning (AUDREP-39)

- **storage:** Fix(storage): guard postcard deserialization at trust boundary (AUDREP-45)

- **index:** Fix(index): explicit warn + empty results for zero-norm cosine query (AUDREP-55)

- **wal:** Fix(wal): remove dead misleading last_offset field (AUDREP-56)

- **frontend:** Fix(frontend): i18n toast via t('terminal.codeCopied') (AUDREP-47)

- **build:** Fix(build): move exclude=["fuzz"] from [workspace.package] to [workspace] (AUDREP-23)

- **dx:** Fix(dx): cross-platform shell in Justfile (AUDREP-26)

- **frontend:** Fix(frontend): i18n skip-link via t('common.skipToContent') (AUDREP-42)

- **server:** Fix(server): remove expect in governor config, add wasm LICENSE, sync deny ignores (NV-02/03/05)

- **index:** Fix(index): canonical select_neighbors pruning + cap over-capacity at 2m, dedupe clippy forks (AUD-012/013/014)

- **index:** Fix(index): complete select_neighbors<F> refactor + overflow guards in search.rs (AUD-012/013/014, ERR-001)

- **mcp:** Fix(mcp): remove dead collect_stats (AUD-012 clippy), zero-norm-safe vectors in stats test

- **parser:** Fix(parser): reject reserved keywords as optional alias (WHERE/RANK data loss)

- Fix: SEC-01 UAF in VantaSearchHit __array_interface__ (vantadb-python)

- **bindings:** Fix(bindings): clamp top_k in python+wasm to avoid giant alloc (ERR-022)

- **mcp:** Fix(mcp): bounded collection stats/list/delete to prevent OOM (ERR-021)

- **storage:** Fix(storage): minimize HNSW insert_lock hold during queries (ERR-035)

- **storage:** Fix(storage): hold insert_lock across checkpoint/save race (ERR-010)

- **storage:** Fix(storage): SIGBUS handler sets flag, no re-execute loop (ERR-002)

- **storage:** Fix(storage): bounds-check vector_store indexing (ERR-003)

- **wal:** Fix(wal): surface truncated-shard replay instead of silent loss (ERR-011)

- **storage:** Fix(storage): revert inventory stats on txn abort (ERR-013)

- **index:** Fix(index): correct random_layer level distribution (ERR-018)

- **bench:** Fix(bench): force HNSW path in pure bench (ERR-019)

- **index:** Fix(index): decrement inbound counters on delete (ERR-012)

- **mcp:** Fix(mcp): preserve u128 neighbor ids as strings (ERR-025)

- **bindings:** Fix(bindings): preserve u128 node IDs in python (ERR-023)

- **bindings:** Fix(bindings): preserve u128 node IDs in wasm (ERR-024)

- **index:** Fix(index): ACORN second-hop after repair_orphans (ERR-020)

- **bindings:** Fix(bindings): put_batch respects per-record namespace (ERR-030)

- **storage:** Fix(storage): immediate insert visibility for get (ERR-014)

- **storage:** Fix(storage): edge_count u16 overflow on persist (ERR-029)

- **http:** Fix(http): return 4xx/5xx on query error (ERR-027)

- **sdk:** Fix(sdk): reject zero-norm cosine query vectors (ERR-028)


### 👷 CI/CD

- **npm:** Ci(npm): migrate to trusted publishing (OIDC)

- **release:** Ci(release): enable release-plz with trusted publishing (OIDC) for crates.io

- **npm:** Ci(npm): publish wasm+ts automatically on v* release tags

- **npm:** Ci(npm): skip publish if version already exists (idempotent re-runs)

- **gate:** Ci(gate): skip heavy/fuzz runs when main CI is red

- **asan,heavy:** Ci(asan,heavy): fit heavy jobs in CI timeouts

- Ci: sccache + nextest install fix (GH-143)

- Ci: add example smoke tests to CI pipeline (GH-142)

- Ci: enable custom allocator in release binaries (mimalloc/jemalloc, INV-004)

- **task-system:** Ci(task-system): sincronizar reportes con backlog y endurecer flujo de pipeline

- **task-system:** Ci(task-system): registrar tareas campaña y validación backlog 08-05

- **task-system:** Ci(task-system): wire pre-call checks and add state tools

- **rust:** Ci(rust): habilitar -Zmiri-disable-isolation para Miri Test

- **rust:** Ci(rust): correr Miri sin feature roaring (fallback pure-Rust FilterBitset)

- Ci: resolve REVIEW-01/02/03/05 findings


### 💄 Style

- Style: rustfmt sparse search module


### 📖 Documentation

- **release:** Docs(release): npm triggers on v* release tags

- Docs: add doc-tests for public Rust API (GH-124)

- Docs: add complete docstrings to Python SDK public API (GH-122)

- Docs: add INV-017 sccache CI investigation + fix AGENTS.md drift (INV-017)

- Docs: migrate INV-017 + GH-143 to progress (sccache CI)

- Docs: close COMP-019 as WONTFIX - gRPC contradicts embedded-first (ADR)

- Docs: migrate COMP-028 to progreso (COMP-028)

- Docs: migrate NUEVO-10 to progreso (NUEVO-10)

- Docs: mark MKT-14 as completed (SKIP gate - already implemented)

- Docs: migrate COMP-021 to progreso (COMP-021)

- Docs: add blog series completion plan (INV-006)

- **tutorials:** Docs(tutorials): rewrite agent memory and RAG tutorials to real vantadb_py API (NUEVO-08)

- **tutorials:** Docs(tutorials): add hybrid search and embedding integrations tutorials (NUEVO-08)

- **tutorials:** Docs(tutorials): promote chromadb tutorial to active and add learning path index (NUEVO-08)

- **book:** Docs(book): sync mdBook tutorial stubs and summary (NUEVO-08)

- Docs: update tutorial references in master index and README (NUEVO-08)

- Docs: mark NUEVO-08 task as completed (NUEVO-08)

- Docs: migrate NUEVO-08 and INV-006 to progreso

- **strategy:** Docs(strategy) fix product version references to 0.5.0

- **blog:** Docs(blog): finalize blog series source (M1-M6), fix version to 0.5.0

- Docs: log M1-M6 resolution in INV-006 task

- Docs: mark ENT-04 as completed in all references (ENT-04)

- Docs: add strategic manual and archive NUEVO-17 task file

- **task-system:** Docs(task-system): document harness plan/state format requirements in troubleshooting

- **acid:** Docs(acid): INV-010 design multi-layer ACID rollback protocol

- **inv:** Docs(inv): complete INV-007/008/009 competitive benchmark, Python batch, phrase query designs

- **acid:** Docs(acid): INV-010 move design doc to docs/research

- **inv:** Docs(inv): complete web audits INV-013/014/015/016 — JSON-LD, light mode, touch targets, motion tokens

- **inv:** Docs(inv): extract INV-013/014/015/016 audit findings to Investigaciones docs

- **rules:** Docs(rules): add .opencode/rules area-specific agent rules

- **progreso:** Docs(progreso): archive no-progress history to ARCHIVO_HISTORICO, dedup, unify Spanish, fix commit hashes and tables

- **progreso:** Docs(progreso): add BACKLOG_HISTORY archive and sdk-gap audit, DRV-014 ADR

- **plans:** Docs(plans): update PROMPT-MAESTRO-FREEZE recitation for INV-012 anti-locality re-evaluation

- Docs: DESKTOP-01 tauri platform research report

- **GH-119:** Docs(GH-119): add Vectara migration guide

- **MKT-05:** Docs(MKT-05): add 5th pre-launch blog post on benchmarks

- Docs: DESKTOP-01b 6-integrations research + multi-connection architecture

- Docs: sync investigations A/B/C/D — rules filled, backlog tasks, superseded fix

- Docs: close launch-web-campaign (MKT-05, MKT-15, WEB-001, WEB-18, GH-119)

- **rules:** Docs(rules): fill API/JS skeletons + harden core/release from multi-agent audit

- **backlog:** Docs(backlog): add AUD-001..AUD-011 findings from doc<->code audit

- **GH-123:** Docs(GH-123): fix typos, broken links, and stale version refs in docs/

- Docs: add TECH-01..08 tasks from DESKTOP-01b findings

- **Backlog-EDIT:** Docs(Backlog-EDIT): corregir 12 premisas stale en Backlog.md

- **backlog-validation:** Docs(backlog-validation): Fase 1 — cierres y consolidaciones (NUEVO-22, TSK-103, MKT-17, GH-144, LEG-01, COM-04, ADR-012)

- **backlog-validation:** Docs(backlog-validation): F2 wave0 — AUDIT-01, AUD-001, TECH-01, TECH-02

- **backlog-validation:** Docs(backlog-validation): F2 wave1 — AUD-004, AUD-011

- Docs: add backlog Phase 13 audit findings and competitor research

- **backlog-validation:** Docs(backlog-validation): F2 wave2 — AUDIT-04 root-cause cache_warmer

- **DEBT-01:** Docs(DEBT-01): reparar gate docs-coverage y documentar 13 gaps

- **TECH-06:** Docs(TECH-06): cerrar CORS — sin consumidor browser real

- **TECH-03:** Docs(TECH-03): corregir 3 stale-docs (MCP excluyente, API python real, query_iql)

- **backlog-validation:** Docs(backlog-validation): F3 wave2 — TECH-08 decision, AUDIT-05, AUDIT-08

- **AUD-007:** Docs(AUD-007): corregir drift de nombres y constantes en ARCHITECTURE.md

- **AUD-009:** Docs(AUD-009): corregir nota Vite->Next.js en DESKTOP-01b

- **AUD-003:** Docs(AUD-003): retractar verificación contra src/governance inexistente

- **AUD-003:** Docs(AUD-003): taskfile commit hash

- **AUD-002:** Docs(AUD-002): corregir API fantasma en GRAPH_RAG.md

- **AUD-005:** Docs(AUD-005): sincronizar openapi.yaml a 0.5.0 + gate CI check-api-version

- **AUD-008:** Docs(AUD-008): corregir drift de versiones en STORAGE_VERSIONING.md

- **AUD-006:** Docs(AUD-006): documentar 5 tools MCP reales (collection_*, rehydrate) + gate de paridad

- **GH-123:** Docs(GH-123): corregir 3 links rotos reales + documentar metodo de auditoria; issue cerrado

- **backlog-validation:** Docs(backlog-validation): F4 wave2 — AUD-005, AUD-006, AUD-008, GH-123

- **NUEVO-01:** Docs(NUEVO-01): README hero + benchmark graphic (GH-139)

- **GH-132:** Docs(GH-132): notebook Colab + badge Open in Colab

- **GH-131:** Docs(GH-131): README integraciones mem0, Semantic Kernel, DSPy

- **INV-025:** Docs(INV-025): scoping Search Quality v2 + dependencia con INV-009-B

- **NUEVO-16:** Docs(NUEVO-16): viabilidad Product Quantization (update REC-009)

- **GH-141:** Docs(GH-141): documentar webhook GitHub→Discord (push, PR, issues, release)

- **MKT-10:** Docs(MKT-10): reescribir campaña AI Agent Memory con checklist medible

- **DESKTOP-MVP-54:** Docs(DESKTOP-MVP-54): save point for Tauri MVP task 54

- **backlog-validation:** Docs(backlog-validation): plan 53/54 completado (resta Task 50 humana Discord)

- **progreso:** Docs(progreso): migrar 14 tareas F5/F6 completadas de Backlog a historial

- **backlog:** Docs(backlog): tachar AUDREP-03/08/13 completados + actualizar críticos activos

- **review:** Docs(review): registrar reportes full/certify 2026-08-05 + manual de operacion

- **backlog:** Docs(backlog): validar reviews 07-27 y archivar duplicados

- **docs:** Docs(docs): limpiar y consolidar audit-reports y reviews

- **gate:** Docs(gate): fix markdownlint y frontmatter para GATE Docs

- Docs: add Open Core licensing model for VantaDB Pro

- Docs: revamp README with animated banner, integrations and translated content

- Docs: update licensing, backlog and plans; sync provider lockfiles to 0.5.0

- **audit:** Docs(audit): dependency duplication report with root-cause table (DEPS-01)

- **progreso:** Docs(progreso): close P13 audit findings (AUDREP-07/09/10/11/15/19, NV-01/04, DEPS-01)

- **DESKTOP-04:** Docs(DESKTOP-04): registrar commit hash en task file

- **DESKTOP-05:** Docs(DESKTOP-05): task tracking file

- **avance:** Docs(avance): add live progress mirror with domain files + coverage check script

- **tokenizer:** Docs(tokenizer): advanced-tokenizer default-enabled, auto-rebuild on v3->v4 schema change

- **audit:** Docs(audit): record full audit 2026-08-08 (0 critical) + ERR-001..020 review ledger

- **plans:** Docs(plans): mark ERR-029 task 26 DONE (ERR-029)

- **plans:** Docs(plans): mark ERR-027/028 tasks 24-25 DONE


### 🔒 Security

- Security(mcp): log internal error detail instead of leaking to client (AUDREP-61)


### 🚀 Features

- Feat: Vanta Studio Fase 3 — transporte pluggable (Tauri/HTTP) + REST completo del SDK (`/api/v2/*`, ~27 endpoints: health, records CRUD/batch/versions/delete_by_filter, list con cursor, search, autocomplete, query, audit, export/import, graph, maintenance, threads, snapshots) + dashboard web embebido `/dashboard` servido por `vanta-cli server --dashboard-dir <dir>` (WEB-00..06, ADR-026)

- Feat: add Chroma/LanceDB migration scripts + fix tutorials API (NUEVO-07)

- Feat: add enterprise audit logging (JSONL, timestamp + op) (TSK-107b)

- Feat: add LangChain+Ollama RAG demo, remove legacy sketch (TSK-104)

- Feat: add unified semantic cost estimator module (COMP-028)

- Feat: public reproducible benchmark suite (NUEVO-10)

- Feat: temporal edges with created_at_ms and time-window traversal (COMP-021)

- **bindings:** Feat(bindings): add napi-rs native Node bindings as additional backend (COMP-029)

- **server:** Feat(server): explicit connection pool + circuit breaker (ENT-04)

- Feat: segment tier policy hot/warm/cold + archive tier (NUEVO-17)

- Feat: native sparse vectors + sparse+dense hybrid search (NUEVO-18)

- **core:** Feat(core): formalize multi-index query routing with cost-based selection (OLD-21)

- **WEB-001:** Feat(WEB-001): run real WASM in playground

- **MKT-15:** Feat(MKT-15): add competitive benchmark table to /benchmarks

- **TECH-05:** Feat(TECH-05): resource MCP schema://

- **INV-013-B:** Feat(INV-013-B): JSON-LD structured data en layout

- **INV-005-A:** Feat(INV-005-A): error.tsx boundary + eliminar dep muerta @mdxeditor

- **INV-007-B:** Feat(INV-007-B): competitive_benchmark.json + tabla web (MKT-17)

- **INV-008-B:** Feat(INV-008-B): search_batch_requests con SearchRequest completo (Python SDK)

- **INV-009-B:** Feat(INV-009-B): Condition::TextMatch con frases (snippet contiguo)

- **NUEVO-22:** Feat(NUEVO-22): sparse indexed search (inverted index + posting lists)

- **DESKTOP-02..05:** Feat(DESKTOP-02..05): scaffold Tauri MVP with NativeConnection contract + ping

- **web:** Feat(web): add Remotion banner/favicon generator and switch site icon to favicon.png

- **DESKTOP-02:** Feat(DESKTOP-02): scaffold Tauri v2 desktop con workspace propio

- **DESKTOP-08:** Feat(DESKTOP-08): cliente IQL tipado + tests mock server

- **DESKTOP-04:** Feat(DESKTOP-04): contract VantaConnection trait + serde DTOs + task file

- **DESKTOP-03:** Feat(DESKTOP-03): integrar crate vantadb + AppState managed + healthcheck

- **DESKTOP-05:** Feat(DESKTOP-05): NativeConnection sobre VantaEmbedded con lock de path

- **server:** Feat(server): add configurable CORS middleware (default off)

- **DESKTOP-09:** Feat(DESKTOP-09): ServerConnection sobre cliente IQL

- **DESKTOP-06:** Feat(DESKTOP-06): CRUD commands async + ConnectionManager registry

- **DESKTOP-11:** Feat(DESKTOP-11): spawn manager subproceso MCP (sidecar)

- **DESKTOP-07:** Feat(DESKTOP-07): frontend MVP (health/ingest/search)

- **DESKTOP-10:** Feat(DESKTOP-10): wire server HTTP adapter via ConnectionSelector (loopback url/port/token)

- **server:** Feat(server): register --mcp flag in --help via hand-rolled argv (AUDREP-62)

- **parser:** Feat(parser): typed numeric/string RHS for relational conditions (AUDREP-38)

- **ADMIN-03:** Feat(ADMIN-03): migrate desktop UI to web design system light mode, drop dead ConnectionSelector

- **DESKTOP-20:** Feat(DESKTOP-20): connection manager shutdown_all lifecycle with kill timeout

- **ADMIN-01:** Feat(ADMIN-01): expose operational metrics snapshot as vanta_metrics Tauri command

- **ADMIN-04:** Feat(ADMIN-04): metro-style metrics dashboard grid with live poll

- **ADMIN-05:** Feat(ADMIN-05): derived KPI cards with CSS sparklines

- **ADMIN-06:** Feat(ADMIN-06): SOP operational panels (WAL replay, reindex, health)

- **ADMIN-07:** Feat(ADMIN-07): data explorer with pagination for active connection

- **ADMIN-09:** Feat(ADMIN-09): snapshot export to JSON with last-snapshot persistence

- **ADMIN-08:** Feat(ADMIN-08): processes and connections panel with kill/remove


### 🧪 Testing

- Test: add concurrent multi-namespace stress test (#134)

- Test: add proptest coverage for WAL record roundtrip (GH-127)

- **server:** Test(server): make metrics assertions conditional on prometheus feature

- **crewai:** Test(crewai): apuntar tests categorize a la funcion de modulo (stale)

- **mcp:** Test(mcp): renombrar query_lisp a query_iql (stale tras CUARENTENA-01)

- **mcp:** Test(mcp): collection_delete rollback leaves no partial deletes (AUDREP-43)

- **storage:** Test(storage): restore oversized-write guard test (ERR-005)


### 🧹 Chores

- Chore: mark INV-019 SKIP - advanced tokenizer already implemented

- **task-system:** Chore(task-system): fix audit findings in .opencode orchestrators and remove legacy .antigravity

- **task-system:** Chore(task-system): archive 16 completed task files to tasks/complete/ and mark ENT-04, COMP-029, TSK-107b done in Backlog

- Chore: INV-011 INV-012 — core-server separation audit + anti-locality re-evaluation

- **opencode:** Chore(opencode): fix stale skill counts, remove double skill loading, normalize skill references

- Chore: remove obsolete launch-web campaign task views

- **TECH-07:** Chore(TECH-07): documentar API worker opfs + demo browser

- **AUDIT-03:** Chore(AUDIT-03): fmt vfile shim + tachar AUDREP-01/04, AUDIT-03 en backlog y progreso

- Chore: pipeline-state 53/54 completo (resta Task 50 humana)

- **build:** Chore(build): wire rayon feature and fix doc drift (AUDREP-07)

- **web:** Chore(web): enable strict TS build checks and react strict mode (AUDREP-19)

- **backlog:** Chore(backlog): move AUDREP-14,16,17,18,20,21,22 to progreso

- **DESKTOP-05-09:** Chore(DESKTOP-05-09): wire native+server adapters into connections mod (parallel merge)

- **backlog:** Chore(backlog): desktop-MVP waves 0-3 completas (DESKTOP-02..11) -> progreso

- Chore: remove superseded root src-tauri scaffold (superseded by desktop/)

- **infra:** Chore(infra): drop obsolete compose version key (AUDREP-49)

- **backlog:** Chore(backlog): mark 10 AUDREP tasks complete (2026-08-07) -> progreso

- **release:** Chore(release): bump homebrew formula version 0.2.0 -> 0.5.0 (AUDREP-25)

- **backlog:** Chore(backlog): mark 10 AUDREP tasks complete (2026-08-07) -> progreso

- **web:** Chore(web): remove dead next-auth dependency (AUDREP-41)

- **release:** Chore(release): workspace-inherit edition/rust-version in root crate + add macOS/WASM toolchain targets (AUDREP-48, AUDREP-50)

- **server:** Chore(server): consolidate duplicated tokio dependency (AUDREP-52)

- **backlog:** Chore(backlog): mark 10 AUDREP tasks complete (2026-08-07) -> progreso

- **web:** Chore(web): enable noImplicitAny strictness (AUDREP-46)

- **web:** Chore(web): rename package to vantadb-web (AUDREP-59)

- **backlog:** Chore(backlog): mark 9 AUDREP tasks complete, migrate to progreso (2026-08-07)

- **backlog:** Chore(backlog): admin-console plan 10/10 complete (ADMIN-01..09 + DESKTOP-20) -> progreso

- **ts:** Chore(ts): add @vitest/coverage-v8 devDependency

- **workflow:** Chore(workflow): progreso deletes completed rows; add docs/avance mirror + plan reality gate in prompts

- **plans:** Chore(plans): close admin-console + desktop-mvp campaign records (task files, session, budgets, commit refs)

- **backlog:** Chore(backlog): migrate AUD-012..015 to progreso, record 225-row 2026-08-07 cleanup

- **backlog:** Chore(backlog): mark ERR-016 resolved, task record + parser doc-comment

- **docs:** Chore(docs): archive 5 completed plans + fix admin-console recitation, update progreso

- **pipeline:** Chore(pipeline): archive completed audit reports, create backlog-2026-08-09 plan

- **pipeline:** Chore(pipeline): move legacy plans to archive

- **ci:** Chore(ci): add cargo-semver-checks gate (RELEASE-01)

- Chore: mark SEC-01 completed in backlog plan

- **deps:** Chore(deps): handle RUSTSEC-2026-0002 (lru) (ERR-004)

## [0.5.0] - 2026-07-31

### Added

- IVF Flat index — inverted file with k-means clustering (no external deps). New `IndexType::Ivf` on `HnswConfig`. Lazy-built on first search, serialized in v8 format. ~50× faster than brute-force Flat on 1M vectors at ~90% recall.
- Multi-level LSM compaction (L0→L1→L2→L3) — `StorageEngine.vector_store` splits into per-level VantaFiles. `SegmentRegistry` handles legacy migration. `compact_level()` promotes live nodes between tiers. New `PipelineMode::CompactOnly`/`CompactL0Only` variants. Write amplification reduced from O(all data) to O(L0 size).

## [0.4.0] - 2026-07-20

### Added

- Initial public release of VantaDB.
- Embedded persistent vector/graph database engine.
- HNSW vector index with configurable distance metrics.
- WAL (Write-Ahead Log) with automatic crash recovery.
- Arrow IPC integration for zero-copy data interchange.
- CLI tool (`vanta`) for database operations.
- HTTP server with rate limiting and TLS support.
- Python SDK (`vantadb_py`) with PyO3 bindings.
- WASM build for browser-based querying.
- TypeScript SDK (`vantadb-ts`).
- AI framework adapters: LangChain, LlamaIndex, Haystack, CrewAI, DSPy, Litellm, OpenAI, Ollama, Mem0, Letta.
- MCP server (`vantadb-mcp`) for Model Context Protocol.
- Prometheus metrics and OpenTelemetry tracing.
- Encryption at rest (AES-GCM).
- PITR (Point-in-Time Recovery).
- WAL shipping for replication.
- Hot-reload of configuration.
- Failpoints for fault injection testing.
- Custom allocator support (mimalloc, jemalloc).

### Fixed

- CI/CD pipelines: FUZZ, release binaries, adapters, SBOM, wheels, code coverage — all green.
- Serialization bounds check overflow in `src/index/serialize.rs`.
- `vantadb-mcp` excluded from binary releases (library-only crate).
- Conditional `Attach to Release` step in adapter release workflow.
- Code coverage runner RAM increase (6GB → 8GB) to prevent LLD SIGBUS.

### Changed

- Workspace version reset to v0.4.0 — clean semantic versioning start.
- All previous tags (v0.1.0 through v0.3.0-stable, wasm-*, ts-*, adapters-*) removed.
- Root crate version inherits from `[workspace.package]`.

### Removed

- All pre-release tags and orphan GitHub Releases from v0.1.x era.
