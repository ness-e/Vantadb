# Changelog

All notable changes to the VantaDB engine will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [Unreleased]

## [0.6.0](https://github.com/ness-e/Vantadb/compare/v0.5.0...v0.6.0) - 2026-09-05

### Added

- *(cli)* restore --dry-run validacion sin mutar (GOV-TK1)
- *(integrations)* packaging PyPI 5 adapters + release workflow (MKT-18f)
- *(demo)* compose multi-servicio ollama+vantadb (MKT-18i; AnythingLLM re-escalado, sin soporte de backend aguas arriba)
- *(cli)* doctor --fix con dry-run seguro (GOV-TK1)
- *(metrics)* vantadb_errors_total por code — registry in-tree (FIND-53)
- *(observability)* Backtrace + tracing estructurado + docs OBSERVABILITY (ERR-OBS-01)
- *(memory)* MEM-12 re-verify Wave3 F4 META + nodo escena ponytail reuse a6526f70 (Task 15)
- *(mcp)* From<VantaError> for McpError + isError con code/retriable (ERR-MCP-01)
- *(search)* RES-06 docs/api/scores follow-up bench (ponytail reuse RES-04/05, vantadb-ts pass-through ERR-028) — Wave3 37/86
- *(memory)* MEM-10 L1 extractor — Wave3
- *(error)* VantaError::code() 10 códigos canónicos + tipa vector/edge overflow (ERR-CORE-01)
- *(memory)* MEM-08 F4 Fundacion crate vanta-memory scaffold Wave3 verify 328 tests (ponytail reuse, disjoint GOV-C5/RES-06)
- *(mcp)* MEM-02 search profile en MCP/search — paridad IQL+API+MCP (Wave2 F1)
- *(iql)* MEM-03 PROFILE clause — reuse SearchProfileConfig (Wave2 F1)
- *(search)* RES-04 semántica scores — docs/api/scores + src/api/scores helper (ponytail)
- *(iql)* RES-03 phrase TextMatch literal — tokenización sin stemming/stopwords + highlight frase completa
- *(mcp)* MCP-35 fallback HTTP N instancias misma BD — proxy .vanta.server.json + 2x DB E2E
- *(web)* WEB-09 refinamiento sutil+A11y home — 15 efectos atenuados con gating reduced-motion
- *(web)* WEB-08 E2E guard landing->docs->playground verde + WEB-06 quick-install fold
- *(web)* WEB-07 sandbox iframe para playground (new Function self-XSS) — allow-scripts
- *(web)* WEB-04 unifica idioma metadata 5 layouts ES — title===OG.title, build+lint verde
- floor-guard.ps1 + merge 8 skills + 6 agents (submodule aecc8dc)
- add CONSTRAINTS.md quality bar + update configOpencode submodule
- *(PY-01)* verify graph_bfs_filtered Python binding parity with Node/TS
- *(MCP-39)* add output budgeting with next_cursor for search_multi and memory_list
- *(MCP-37)* add tool surface profiles (memory/dev/full) for Cursor cap
- *(TBH-09)* add crash_recovery bench (open + WAL replay sweep)
- *(TBH-08)* add wal_throughput bench (sweep WAL x fsync x batch sizes)
- *(TBH-06)* add insta 1.48 snapshot testing (3 parser tests + .snap files)
- *(TBH-02)* initialize criterion regression baseline
- *(TBH-01)* verify_datasets.ps1 + heavy-certification pre-test gate
- *(desktop)* Settings dropdown for local embedding model (DESKTOP-EMBED-01+)
- *(desktop)* opencode MCP config + tsc removed from release build
- *(desktop)* local ONNX embeddings via Tauri IPC (DESKTOP-EMBED-01)
- *(embeddings)* sanity numérico con ort real (cosine self/paraphrase/cross-lang)
- MEM-64 — Skills versionadas + CompactionReport
- MEM-59 — Recall MCP público (memory_recall, memory_search)
- MEM-62 — Export markdown git-friendly (round-trip idempotente)
- MEM-61 — Dreaming consolidación idle (sleep-time tiering)
- MEM-60 — Lifecycle heat+decay L1 + contradicciones (con provenance)
- PROV-10 — store() con custom key/upsert determinista
- [**breaking**] PROV-04 — Canonical contract providers (text/next_cursor/limit usize)
- WSM-12 — Contador sanitizaciones NaN/Inf en operational_metrics
- WSM-06 — Batch paridad WASM (DISCOVERY-scoped subset)
- WSM-05 — Hand-written .d.ts para pkg standalone (reduce any)
- WSM-04 — Errores tipados {code,message} en WASM glue
- BND-10 — Node API parity (13 endpoints: lifecycle+maintenance+search)
- TS-04 — API parity remove_edge, count, supersede, sparse_vector, batch search
- SRV-05 — RBAC scoping por namespace (r/w per-collection)
- RES-01 — ACID Phase 4a (WAL v2 WalRecord::Prepare)
- WSM-07 — spawnOpfsWorker helper exported from glue
- SRV-02 — Tracing-id por request (x-request-id/traceparent)
- MCP-34 — snapshot_restore + anti path-traversal
- MCP-39 — Output budgeting (truncado + next_cursor)
- SRV-04 — Multi API keys + rotación sin downtime
- SRV-04 — Multi API keys + rotación sin downtime
- WSM-03 — Auto-save en visibilitychange/pagehide
- PY-01 — paridad graph_bfs_filtered en Python binding
- WSM-02 — Manejo cuotas storage browser (QuotaExceededError)
- MCP-37 — Perfiles de tool surface (cap Cursor 40 tools)
- CORE-001 — Scope Enforcement en ACT State
- pipeline run master-optimization 20/20 COMPLETED
- Master pipeline optimization - 20 items implemented
- *(CORE-01)* persistencia Binary en vstore, requiere ADR de formato
- *(EMB-07)* bench comparativo 9 modelos
- *(EMB-05)* MCP tool embed_texts con embed_batch
- *(EMB-02)* embed-local feature + LocalOnnxProvider (ort+tokenizers)
- *(EMB-01)* infra embeddings/ manifest + download/verify + gitignore
- *(test)* FIND-39 add test_scalar_remove for ScalarIndex.remove
- WSM-01 — fix silent OPFS fallback (no .ok(), persistence faithful)
- *(mcp)* MCP-38 tool annotations readOnly/destructive/idempotent/openWorld para 76 tools
- *(mcp)* MCP-36 protocolo moderno 2025-06-18 + structured output
- *(desktop)* DESKTOP-QW10 — E2E multi-perfil + proxy dashboard mock (H-07)
- *(integrations)* QW-1..9 quickwins — crewai/langchain/llamaindex/ollama-openai dedup, nits, letta docs, PyPI+READMEs+CI matrix (verify-only, fixes in 96e143ec)
- add automated maintenance ritual to codebase-memory skill and /codeGraph
- *(desktop)* DESKTOP-QW9 — baseline medido recursos app (H-15) §Desktop BENCHMARKS (bundle 2.71MB, build 24.59s)
- *(desktop)* DESKTOP-QW8 — excluir desktop de release-plz (H-11, manual 0.1.0 vs 0.5.0)
- *(desktop)* DESKTOP-QW7 — Rename namespace preserva sparse_vector (H-04)
- *(desktop)* DESKTOP-QW6 — CSP minima localhost+https remoto (H-01)
- *(desktop)* DESKTOP-QW4 — Botón FILTROS activo = reglas >0 (H-14, DAUD-02)
- *(desktop)* DESKTOP-QW3 — statusReport.ts markdown EN→ES (H-05)
- *(desktop)* DESKTOP-QW2 — Handler F1/F2 → HelpPanel con tabs contextuales (H-03)
- *(desktop)* DESKTOP-QW1 — CommandPalette union sync (H-02)
- update npm release workflow with matrix build for all targets (BND-08, BND-09)
- *(node)* add musl targets + npm release workflow (BND-08/09)
- *(server)* multi-api-key rotation + namespace RBAC + Docker + hardening guide (SRV-04/05/07/08)
- *(web)* quickwins INV - metadata i18n layouts, assets mascota, sandbox playground doc, E2E specs scaffold, install prominent (WEB-03..09)
- *(node)* index.d.ts tipado fuerte + 25 tests + NODE_SDK.md + bench A/B harness (node hardening w1: BND-11/12/13, PERF-BENCH-01)
- *(integrations)* bugs adapters (crewai/langchain/llamaindex/haystack/mem0) + shared module dedup ollama/openai + READMEs (integrations wins QW-1..8)
- *(server)* rotacion audit log + request-id tracing (SRV-01/02)
- *(desktop)* CSP minima Tauri (H-01) + sparse_vector end-to-end en rename (H-04) + F1/F2 handler + palette sync + DAUD rows sync (desktop quickwins)
- *(python)* deprecar alias import vantadb_py (PY-03) + README ingles + classifiers 3.14 + excludes maturin (py research quickwins)
- *(storage)* MCP-34b physical snapshot_restore - core + SDK + MCP tool with destructive confirmation
- *(commands)* /research <modulo> — investigacion INV-* con registro de modulos y decisiones HITL por hallazgo
- *(task-system)* Skill Discovery Protocol (SDP) — carga de skills auditada en tareas y planes
- *(node)* BND-05 expose graph + explain surface (12 napi methods) for wasm/ts parity
- *(mcp)* MCP-34a snapshot_create tool (wrapper, anti path-traversal)
- *(mcp)* MCP-33 write_axiom/delete_axiom tools (Iron Axioms intact)
- *(python)* MOD-20 typed VantaError hierarchy + query_structured()
- *(mcp)* MOD-10+MCP-24 expose versions/supersede/vacuum/remove_edge + search_with_method/search_multi tools
- *(server)* MOD-13 add request TimeoutLayer (30s interactive / 600s bulk)
- *(python)* MOD-19 expose count, delete_by_filter, similar_to_key in PyO3 binding
- *(desktop)* DESKTOP-38 — dashboard PROXY + endpoint /snapshot en vanta-proxy
- *(web)* WDA-07 F7 comercial — hero value prop + CTA primario, pricing honesto, demo funnel coherente
- *(desktop)* DESKTOP-23..39 — campaña Vanta Studio (16/17 tareas)
- *(web)* WDA-02 F2 estructura — not-found.tsx, sitemap +7, SITE_URL real, tt() a lib
- *(desktop)* FIND-19/23/25 native chrome, splash intro, user-language UI
- *(desktop)* FIND-22 follow-OS theme + FIND-26 design decisions doc
- *(web)* FIND-12 /docs/guides renders repo guides in-site
- *(ts)* FIND-01 accept flat metadata strings (Rust enum shape hidden)
- *(mcp)* MCP-32 tools thread_* CRUD — historial conversacional desde agentes
- *(mcp)* MCP-30 tools scene_read/list/query — navegación por escenas desde agentes
- *(server)* FIND-07 refuse-to-start auth gate for non-loopback HTTP binds
- *(mcp)* MCP-31 tool context_assemble — context engine accesible desde agentes
- *(task-system)* canonical findings routing with FIND-* backlog tickets
- *(task-system)* guided spec flow driven by question tool
- *(docs)* avance-canónico + unificación reviews
- *(task-system)* S1+D1-D4 spec-first, HITL routing y limpieza decidida
- *(task-system)* R1-R7 paralelismo real multi-instancia
- *(task-system)* H4-H9 question gates HITL, umbral único y memoria única
- *(vanta-proxy)* MEM-56 hook Langfuse/OTLP-JSON manual sobre ReportHook (off by default, P4, cierra Última Milla)
- *(desktop)* MEM-58 consolidación UI ↔ context engine real (vanta_context_assemble IPC + fallback heurístico)
- *(desktop)* MEM-53 comandos IPC pipeline vanta-memory + docs(api): MCP.md 56 tools documentadas (gate fix)
- *(server)* MEM-55 conversation/add dispara extracción L1 vía ConversationTrigger + HttpCaptureBridge (H6, P4 fallback)
- *(vanta-proxy)* MEM-57 parser claude-code (classify main/fork/sidequery + extract user text sin system-reminders)
- *(mcp)* MCP-20+26 recovery tools + capabilities/snippet/snapshots introspection
- *(vanta-memory)* BND-03 tiktoken-rs 0.12 feature-gate precise-tokens (golden tests cl100k, D21 enmienda ADR-029)
- *(mcp)* MCP-21+22 graph GDS (pagerank/centrality) + traversal tools
- *(server)* MEM-54 skills CRUD HTTP con optimistic lock + owner 404 anti-enumeración (H5)
- *(mcp)* MCP-18+19 memory_delete_by_filter + memory_put_batch tools
- *(vantadb-mcp)* MEM-52 fachada productiva ingest wiki (wiki_ingest/wiki_ingest_status, worker split begin/execute, H3)
- *(mcp)* MCP-17+25 export/import JSONL + bulk_import_file/stream via MCP
- *(vanta-proxy)* MEM-51 O2 interceptor stream + loop agéntico memory-tools (D46-D48, cap 3 iter)
- *(mcp)* MCP-16+23 maintenance tools purge_expired/compact_wal/flush/compact_layout
- *(mcp)* MCP-27 document query_iql graph-only semantics + round-trip test
- *(vanta-proxy)* MEM-50 wire WriteBack::track al request path (capture_turn post-forward, H1)
- *(vantadb-python)* SDKB-03 sub-clientes memory/graph/system/wiki delegantes + 16 tests (backward-compat 105)
- *(vantadb-ts)* SDKB-02 sub-clientes memory/graph/wiki/system lazy getters frozen + 17 tests (backward-compat 246)
- *(plan)* MEM-36 meta-tarea completada — plan campaña Bindings SDK creado (4 tareas, D42 sub-clientes capa TS/Python)
- *(vanta-memory)* MEM-48 compresión consume scores L1 reales (MemoryScoreMap + fallback heurístico)
- *(vanta-memory)* MEM-47 semantic recall dual-pool + RRF en recall/dedup/query (D38 fallback keyword)
- *(vanta-memory)* MEM-46 embeddings en L1 writer vía EmbeddingProvider core (feature opt-in, P4 best-effort)
- *(vanta-memory)* MEM-45 auto-sync scheduler re-ingest programado (ManagedTimer pull-based, busy guard, FNV-1a)
- *(vanta-memory)* MEM-43 wire context engine → pipeline worker (fase post-L3, flag config, budget compartido)
- *(vanta-proxy)* MEM-27 rate-limit sliding window + write-back retry + mem-command + reporting (cierra F6)
- *(vanta-memory)* MEM-31 progreso ingest canal interno + polling run_id (throttle 500ms, P4 best-effort)
- *(vantadb-mcp)* MEM-33 tools MCP wiki_* query-only (BM25 propio título×5, BFS cap 200, guard require_ready)
- *(vanta-proxy)* MEM-26 ciclo auth→session→inject (D34 fail-closed, 5 aliases sessionKey, D29 system-prompt-only)
- *(vanta-memory)* MEM-30 ingest worker serial + merge por página + fallback P4 (STRUCTURAL_FILES, ensureSources)
- *(vanta-proxy)* MEM-25 crate + 3 protocolos wire verbatim (OpenAI/Anthropic/Responses subset, SSE passthrough)
- *(vantadb)* MEM-29 fuentes locales wiki + chunker 12k/400 (path traversal guard, D36)
- *(vantadb)* MEM-28 wiki store + state machine pending→ready (InternalMetadata, CAS optimista)
- *(vantadb-mcp)* MEM-32 tools MCP code_* query-only sobre graphrag propio (D28)
- *(vanta-memory)* MEM-37 assemble_with_recall budget coordinator único (cursor boundary + e2e)
- *(vanta-memory)* MEM-24 MMD persistente (TaskMemory META, dedup fingerprint, injector pair-safe)
- *(vanta-memory)* MEM-42 reclaimer GC offload (retention_days, post-cursor estricto, idempotente)
- *(vanta-memory)* MEM-22 context engine assemble + cascada mild/aggressive (LLM-free, cursor-aware)
- *(vanta-memory)* MEM-39 seed/import CLI (vanta-seed bin, idempotencia content-hash)
- *(vanta-memory)* MEM-41 generation-log provenance best-effort L1/L2/L3 (genlog/<session>, cap 100)
- *(vanta-memory)* MEM-40 recall_scope híbrido session|agent|team (default agent) + primer test search_multi
- *(vanta-memory)* MEM-23 token estimator chars/3 + emergency truncate + CompactionReport
- *(vanta-memory)* MEM-21 tools MCP scene_read/list/query (cierra F4)
- *(vanta-memory)* MEM-20 cursor persistente por sesión (offload state/storage/hook)
- *(vanta-memory)* MEM-19 sanitize_text consolidado + truncación code-point
- *(vanta-memory)* MEM-18 recall prepend/append + 3 modos + profile sync
- *(vanta-memory)* MEM-17 skill extract transcript + sink idempotente
- *(vanta-memory)* MEM-16 orquestación timers+locks (estado local, reloj fake)
- *(vanta-memory)* MEM-15 persona first/incremental + triggers + scene navigation
- *(vanta-memory)* MEM-14 strategy UPDATE>MERGE>CREATE + heat + soft-delete
- *(vanta-memory)* MEM-13 tools read/write/edit sandboxed + store
- *(vanta-memory)* MEM-12 contrato META + nodo escena (ancla L2)
- *(vanta-memory)* MEM-11 L1 dedup 2 fases (store/update/merge/skip)
- *(vanta-memory)* MEM-10 L1 extractor (split + 1 call LLM JSON + parse reparación)
- *(vanta-memory)* MEM-09 L0 capture idempotente (LLM-free)
- *(vanta-memory)* MEM-08b F4 contracts + host-neutral LlmRunner
- *(memory)* scaffold crate vanta-memory (MEM-08a) — workspace member, 6 módulos esqueleto, features llm-driver/mock
- *(server)* data plane REST /conversation/add + /skill/listing (MEM-35)
- *(mcp)* tools skill_* sobre SkillStore con owner check (MEM-07)
- *(core)* esquema skills multi-versión con optimistic lock (MEM-06)
- *(server)* auth 3 capas L1/L2/L3 + audit auth events (MEM-05)
- *(core)* permission-checker allow-only sobre entity_* (MEM-04)
- *(core)* entidades entity_* + CRUD con partición InternalMetadata (MEM-03)
- *(metrics)* snapshot por capa L1/L2/L3/recall/offload + audit memory (MEM-34)
- *(mcp)* search_profile passthrough en tool de búsqueda (MEM-02)
- *(planner)* SearchProfileConfig per request + cláusula IQL PROFILE (MEM-01)
- VER-01 — E2E ampliado (REST-02/03/06) + ADR-027 + cierre Fase 4 (18/18)
- FEAT-03b — core decay como supersession durable (ADR-028)
- FEAT-03a — consolidación asistida UI (candidatos kNN + diff + superseded_by)
- FEAT-02 — superficie Índices/salud real (reemplaza placeholder VS-03)
- FEAT-01 — slider de pesos híbridos BM25/vector en RETRIEVAL
- WASM-04 — drag&drop .vdbdump/JSONL/CSV (import de archivos reales)
- WASM-03 — consola standalone 100% browser (modo wasm, sin server)
- WASM-02 — WasmBackend en factory getTransport (consola 100% browser, OPFS/IDB)
- WASM-01 — persistencia browser OPFS/IndexedDB probada E2E (fixes opfs NotFoundError + bridge IDB)
- REST-06 — ServerConnection.query via /api/v2/query (IQL en consola web)
- REST-05 — namespace_stats en bridge desktop (vanta_namespace_stats + sidebar/HOME reales, fallback solo Unsupported)
- REST-04 — cursor real en /api/v2/list + search paginado (next_cursor, sin duplicados)
- REST-04 — cursor real en /api/v2/list + search paginado (next_cursor, sin duplicados)
- REST-01..03 — rate limiter UI bursts + /api/v2/metrics JSON + graph_v2 u128-safe (8 rechazos vanta-http-map -> 4)
- *(web)* WEB-06 E2E Playwright + fix namespace default REST (bug cazado por E2E)
- *(web/remotion)* banner V3 + demo GIFs for README
- *(desktop)* WEB-05 build web consola (vite --mode web, base /dashboard/, guards runtime Tauri)
- *(desktop)* WEB-04 HttpBackend real (fetch REST) + vanta-http-map
- *(server)* WEB-03 servir estaticos /dashboard + SPA fallback + flag --dashboard-dir
- *(server)* WEB-02 REST v2 resto SDK (export/import/graph/maintenance/threads/snapshots)
- *(server)* WEB-01 REST superficie consola v2 (records/list/search/audit)
- OP-02 — batch ops en grid (selección múltiple + export/eliminar con undo)
- OP-01 — import CSV/JSON pegado (parser + preview + ingest en lote, máx 1000)
- *(web/remotion)* BannerV2 — banner mejorado con dot grid pattern, mark con orbit ring y ojos vivos
- ESPACIO-02 — batch ops sobre selección lasso (export/eliminar con undo) + fix timeout worker test
- ESPACIO-01 — scatterplot UMAP en worker (regl-scatterplot, lasso, color por namespace)
- GRAFO-03 — consola IQL embebida (CodeMirror + autocompletado + highlight subgrafo)
- GRAFO-02 — visor de grafos R3F con force-directed (d3-force, toon+outline, expand incremental)
- GRAFO-01 — bridge de grafos al desktop (bfs/dfs/degree, WASM, TS, Tauri)
- VS-CORE-05 — batch delete con filtro (WASM, TS, bridge Tauri)
- VS-CORE-04 — exportar selección/query con filtro (core, WASM, TS, bridge)
- VS-CORE-06 — IQL en desktop (bridge vanta_query + autocompletado)
- Historial+Diff entre versiones en Inspector (VS-14)
- favoritos/historial/copy-as + encoding redundante a11y (VS-17, VS-18)
- retención de versiones históricas core + bridge (VS-CORE-07, cap 32 aprobado)
- retención de versiones históricas core + bridge (VS-CORE-07, cap 32 aprobado)
- studio Fase 1 — lente RETRIEVAL, ACTIVITY+Timeline, deep links+export (VS-13, VS-15, VS-16)
- desktop bridge expone explain + audit log (VS-CORE-03, VS-12)
- VS-07/VS-08/VS-09 — filtros compuestos + undo/papelera + command palette
- VS-06 Inspector master-detail — 4 tabs + CodeMirror + commit explicito (P6)
- VS-05 grid virtualizado — TanStack Table v9 + Virtual con paginacion por cursor
- VS-CORE-01 cursor/paginacion en bridge desktop — vanta_list con cursor + listPage
- VS-04 HOME overview — 7 cards de resumen (P3 Shneiderman)
- VS-11 bridge DTO enriquecido — MemoryRecord con version/node_id/timestamps/TTL/vector
- VS-03 workspace shell — Sidebar + Topbar + superficie + Inspector
- VS-10 bridge vanta_put — comando Tauri + vantaPut tipado
- VS-CORE-02 namespace_stats single-pass + VantaNamespaceStats tipos
- VS-02 — MARK variante desktop (asistente de datos)
- VS-01 design system desktop — Tailwind v4 + tokens manga/linocut + dark toggle
- VS-00 prototipo Vanta Studio — HOME/MEMORIAS/Inspector HTML validado con Playwright
- AUD-048 unify filter semantics between CLI and MCP
- AUD-048 unify filter semantics between CLI and MCP
- *(cli)* AUD-051 add --metadata flag, block __vanta_* filters, doc filter scope
- *(python)* AUD-049 add vantadb shim module re-exporting vantadb_py
- *(FND-23-F1)* instrumenta vanta_graph_ops_total (traverse, edge queries, add/remove_edge) - deuda ADR-024 saldada
- *(FND-01)* regla memory-budget + bench OOM confirma RSS sin limite (guard subestima 6.5x)
- *(FND-07)* /metrics con feed real de latencia de queries (prometheus, R-3)
- *(task-system)* TSYS-01/05/06/14/15/16 — SLA ADR-017, chaos resilience, memoria esquema, feature shippable, checklist anti-tóxico
- *(task-system)* TSYS-09/10/11/12/13 — tracing de decisiones, HITL, límites de tools, waves paralelas, validación de citas
- *(task-system)* TSYS-02/03/04/07/08 — handoff invariantes, ADR gate, Shape Up appetite, recitation unificado, triage es-ahora
- *(harness)* P3 restante + fallas §3.6 — SARL trace, telemetría skill/tool, Regla 0 auditable, docs gate, reconciliación de memorias
- *(harness)* memory reconciliation trigger in progreso skill
- *(ci)* docs-coverage gate in verify + canonical script map
- *(harness)* skill/tool telemetry in verify-log
- *(pipeline)* SARL trace registry for sub-agent recovery
- *(pipeline)* auditable Regla 0 impact-mapping in task template and pipeline-full gate
- *(pipeline)* P2 calidad estructural + P3 — review ajeno, postmortem, traceId, guardrails de fases
- *(pipeline)* P1 disciplina de proceso — pre-mortem, risk register, retrospectiva medible, guardrails, gates
- *(harness)* P0 harness y higiene — evals, estados únicos, debugging unificado, path resolution
- *(config)* support KB/MB/GB suffixes in memory limit (PERF-06)
- *(sdk)* expose ivf/scann search method from python binding (FEAT-04)
- *(arrow)* export full vector column in export_arrow (FEAT-03)
- *(ADMIN-08)* processes and connections panel with kill/remove
- *(ADMIN-09)* snapshot export to JSON with last-snapshot persistence
- *(ADMIN-07)* data explorer with pagination for active connection
- *(ADMIN-06)* SOP operational panels (WAL replay, reindex, health)
- *(ADMIN-05)* derived KPI cards with CSS sparklines
- *(ADMIN-04)* metro-style metrics dashboard grid with live poll
- *(ADMIN-01)* expose operational metrics snapshot as vanta_metrics Tauri command
- *(DESKTOP-20)* connection manager shutdown_all lifecycle with kill timeout
- *(ADMIN-03)* migrate desktop UI to web design system light mode, drop dead ConnectionSelector
- *(parser)* typed numeric/string RHS for relational conditions (AUDREP-38)
- *(server)* register --mcp flag in --help via hand-rolled argv (AUDREP-62)
- *(DESKTOP-10)* wire server HTTP adapter via ConnectionSelector (loopback url/port/token)
- *(DESKTOP-07)* frontend MVP (health/ingest/search)
- *(DESKTOP-11)* spawn manager subproceso MCP (sidecar)
- *(DESKTOP-06)* CRUD commands async + ConnectionManager registry
- *(DESKTOP-09)* ServerConnection sobre cliente IQL
- *(server)* add configurable CORS middleware (default off)
- *(DESKTOP-05)* NativeConnection sobre VantaEmbedded con lock de path
- *(DESKTOP-03)* integrar crate vantadb + AppState managed + healthcheck
- *(DESKTOP-04)* contract VantaConnection trait + serde DTOs + task file
- *(DESKTOP-08)* cliente IQL tipado + tests mock server
- *(DESKTOP-02)* scaffold Tauri v2 desktop con workspace propio
- *(web)* add Remotion banner/favicon generator and switch site icon to favicon.png
- *(DESKTOP-02..05)* scaffold Tauri MVP with NativeConnection contract + ping
- *(NUEVO-22)* sparse indexed search (inverted index + posting lists)
- *(INV-009-B)* Condition::TextMatch con frases (snippet contiguo)
- *(INV-008-B)* search_batch_requests con SearchRequest completo (Python SDK)
- *(INV-007-B)* competitive_benchmark.json + tabla web (MKT-17)
- *(INV-005-A)* error.tsx boundary + eliminar dep muerta @mdxeditor
- *(INV-013-B)* JSON-LD structured data en layout
- *(TECH-05)* resource MCP schema://
- *(MKT-15)* add competitive benchmark table to /benchmarks
- *(WEB-001)* run real WASM in playground
- *(core)* formalize multi-index query routing with cost-based selection (OLD-21)
- native sparse vectors + sparse+dense hybrid search (NUEVO-18)
- segment tier policy hot/warm/cold + archive tier (NUEVO-17)
- *(server)* explicit connection pool + circuit breaker (ENT-04)
- *(bindings)* add napi-rs native Node bindings as additional backend (COMP-029)
- temporal edges with created_at_ms and time-window traversal (COMP-021)
- public reproducible benchmark suite (NUEVO-10)
- add unified semantic cost estimator module (COMP-028)
- add LangChain+Ollama RAG demo, remove legacy sketch (TSK-104)
- add enterprise audit logging (JSONL, timestamp + op) (TSK-107b)
- add Chroma/LanceDB migration scripts + fix tutorials API (NUEVO-07)

### Fixed

- *(mcp)* drenar stderr en test-mcp.py para teardown determinista (STABLE-04)
- *(api)* put_batch metadata coercion alineada (GOV-TK7)
- *(storage)* commit_transaction bajo insert_lock + test interleaving (FIND-62)
- *(wal)* rama explicita SyncMode::Never (FIND-63)
- *(tests)* ollama provider record access via index (no mapping contains) + self-contained namespaces test
- *(ci)* escape PROVIDER placeholder for bash + migrate ollama tests to get_memory/delete_memory/list_memory API
- *(ci)* providers venv+client libs, pyi path, ollama vector kwargs
- *(ci)* install provider client libs in Providers CI (importorskip skipped all tests)
- *(ci)* desktop artifact globs for tauri bundle layout + scope semver job to main
- *(ci)* set VANTADB_CLI_BIN in Desktop CI so MCP sidecar tests spawn vanta-cli (not legacy server bin)
- *(ci)* force PYTHONUTF8 in Python examples (emoji output breaks Windows cp1252)
- *(ci)* run examples install step with bash on all OSes (Windows defaults to PowerShell)
- *(clippy)* doc list-item continuation in snapshot docs, match-to-? in search_multi (new stable lints)
- *(ci)* install in-repo vantadb-langchain for examples smoke (unpublished on PyPI)
- *(ci)* install system deps in rustdoc artifact job (libclang for bindgen, same as rustdoc-70)
- *(wasm)* gate wasm32 crudo verde tras 175790a9 parcial (FIND-58)
- *(ci)* build vanta-cli sidecar in Desktop CI (tauri bundle requires binaries/vanta-cli.exe)
- *(ci)* Miri DistanceMetric import, openapi parity filename case, rustdoc nightly override, examples langchain filename
- *(release)* drop pinned versions on intra-workspace path deps - unblock release-plz minor bumps
- *(server)* Dockerfile roto - deprecado a imagen raiz con hardening.md honesto (FIND-56)
- *(ci)* un-ignore data/README.md and datasets/README.md - release-plz rejects tracked-but-ignored files
- *(ci)* green GATE Docs/Rustdoc/RELEASE/Providers/Chaos - version header, lint scope, libclang, plz override, venv, E0133
- *(storage)* wrap libc::_exit in unsafe block (E0133 on new toolchain, unbreak Chaos/PERF on Linux)
- *(ingestion)* default worker_count 1, w=4 medido -43% (FIND-57)
- *(docs)* green GATE Docs - MCP version header, lint excludes for frozen history, live-file MD fixes
- *(web)* clear eslint warnings - next/image for local avatars, OG disable note, navbar deps
- *(certify)* close L5/L9 findings - put_batch docs, wasm backend note, async required args, OsRng doc
- *(wasm)* enable getrandom wasm_js backend for wasm32 (config + 0.3 alias) - unbreak wasm-pack
- *(python)* align put_batch call sites, async wrapper and stubs with post-PY-QW2 API (drop legacy entries)
- *(web)* touch targets ≥44px con hit-area en 5 componentes (RES-12)
- *(memory-budget)* calibrar rss_threshold con bench F2 (RES-07)
- *(wasm)* web_time + cfg-out thread::sleep — 30 panics vitest (FIND-52)
- *(server)* sanitiza body 500 — chain solo a logs, code al cliente (FIND-55)
- *(server)* descarta origins CORS vacíos — flake cors_layer_none_when_empty (FIND-54)
- *(desktop)* preserva HttpErrorKind en memory commands (ERR-DESK-01)
- *(web)* toast por código de error + catch sin silenciar (ERR-WEB-01)
- *(providers)* err_to_py jerarquía MOD-20 con code/retriable (ERR-PY-01)
- *(ts)* alinea codes VANTADB_* TS/WASM/Node + guards VantaError (ERR-TS-01)
- *(server)* completa re-exports workspace — cli_server split verde total
- verde total verify sdk api NotInitialized + export_md frontmatter + clippy
- *(vantadb-mcp)* align test_embed_texts with McpConfig (FIND-036)
- *(lint)* drop unused imports post-REVIEW-10 split + const assert in config (FIND-035)
- update configOpencode to 8167a92 (revert MCP v2 to restore campaign)
- *(TBH-14)* enable conventional_commits in cliff.toml (matches commit_parsers config)
- *(desktop)* MCP sidecar spawns canonical vanta-cli (not vantadb-server)
- FIND-33 — Snapshot captura backend KV (consistency reopen)
- src/llm.rs:135 redundant_closure (clippy STABLE-01 gate 2)
- *(embeddings)* lock acumulativo + fallback de rev stale + refresh 9 revs
- PROV-07 — Validación explícita inputs providers (distance_metric, metadata)
- [**breaking**] TS-01 — Corregir tipos grafo ficticios (GraphBfsResult vs wire real)
- GOV-TK3 — Drift yaml↔real: IQL case, GraphTraversalBody, search fresh DB
- RES-05 — Synchronous context manager __enter__/__exit__ in Py binding
- Gap A+B inmediatos (Appetite vs Effort + warmer duplicación)
- MED-018 cache, MED-020 MoM validation, template 20 secciones
- campaign-server duplicate classifyBashWrite syntax error (CORE-001/HIGH-013)
- *(vanta-memory)* add sparse_vector None for VantaMemoryInput compat with develop 792
- *(embeddings)* completar EMB-03,04,06 en develop — download dry-run, vanta-memory L1, SQL vector
- FIND-36 — Cross-crate NativeConnection↔RocksDbBackend frontier doc (false-positive Leiden, DAG justified)
- FIND-35 — StorageEngine get/prefetch intentional SCC justification + PrefetchGuard doc
- FIND-34 — WAL writer DAG justification + recovery/quarantine edge tests
- *(search)* FIND-37 eliminate query_sparse.unwrap panics in hybrid dispatcher
- *(ci)* check-agents-refs honors prompts/ Path Resolution alias
- *(vantadb-ts)* TS-02 wrap async rejections in _native
- add missing alt_api_key field to ServerState test constructors (MOD-15)
- remove needless borrow in cli_server.rs:870 (clippy::needless_borrow)
- *(integrations)* integrations quick wins QW-1..QW-9
- *(server)* add alt_api_key to all ServerState test initializers (SRV-04/05)
- *(wasm)* OPFS append offset + cursor string-u64 + flush honesto + CRC estricto + worker retry/close (wasm quickwins QW-1..5)
- *(providers)* quickwins INV - compila openai (PROV-01 E0063 exclude_superseded), stubs .pyi alineados, timeout wireado litellm, validaciones explicitas, tests firma nueva
- *(storage)* FIND-25 create_snapshot quiesce via flush + recursive mirror excluding snapshots/
- *(server)* AUD-046 expose truncated_namespaces in all-namespaces list fan-out
- *(storage)* AUD-044 shim MmapMut flush writes buffer to disk (was no-op - compact_layout data loss)
- *(desktop)* FIND-23 HTTP map defaults namespace to DEFAULT_NS in ingest/get (was empty)
- *(build)* Justfile run-server/run-mcp + Dockerfile workspace completo
- *(docs)* corregir errores factuales en docs de raíz + des-trackear sesiones
- *(desktop)* recover DAUD-11 window defaults + DAUD-06 Mark neon tokens from stash@{0}
- *(dev-tools)* real exit code in nocturnal suite + unified core gate definition
- *(desktop)* DAUD-LIMPI - scope press-effect, consolidate CSS, remove dead utilities, glyphs, icon convention
- *(desktop)* UX-POLISH - unified confirm, grid density, empty states, onRefresh, fmtBytes, a11y misc
- *(desktop)* UX-A11Y - keyboard grid nav, focus trap, labels, accent-text contrast, ARIA tabs, canvas fallback
- *(python)* MOD-21 nits - graph_bfs direction, clamp_top_k warn, connect read_only/backend
- *(mcp)* MOD-11 nits H4-H8 - clamp k, config-driven list limit, threat model docs
- *(storage)* FIND-31 purge_expired after reopen - text index uses include_expired view
- *(desktop)* UX-16 declare lucide-react dependency (was phantom via hoisting)
- *(ts)* FIND-10 require(esm) support + distinguishable VantaError codes
- *(python)* MOD-18 consolidate vantadb_py stubs + anti-drift test
- *(storage)* REVIEW-17 remove 7 unnecessary unsafe under wasm32 (cfg memmap2 helpers)
- *(index)* FIND-29 replace last manual u8-to-f32 cast in layer.rs with align_to
- REVIEW-13 serialize supersede() read-modify-write (TOCTOU)
- *(mcp)* MOD-08+MOD-09 concurrent stdio loop + flush in-flight response on shutdown
- *(index)* FIND-28 replace unaligned u8-to-f32 casts with as_f32_slice (align_to)
- *(storage)* MOD-02 crash-atomic txn recovery bounds skip to txn extent
- *(desktop)* FIX-D3a reemplaza glifos presentacion-EMOJI por iconos lucide-react
- *(llm)* FIND-27 use current Ollama /api/embed endpoint
- *(desktop)* dark mode del grid + tokens de sombra/labels + errores destructive
- *(desktop)* TitleBar crasheaba la build web + atajos Alt fuera de contexto
- *(server)* list/search sin namespace agregan todos los namespaces
- *(desktop)* WorkspaceShell root fixed inset-0 covered the TitleBar
- *(web)* WDA-06 F6 escritura — literales sin i18n migrados a tt(), 6 claves ES/EN nuevas
- *(desktop)* window control permissions + splash choreography rework
- *(web)* WDA-04 F4 UI — contrastes a11y, touch targets, Toaster muerto fuera, roles ARIA
- *(web)* WDA-03 F3 información — 9 claims falsos corregidos contra BENCHMARKS/README reales
- *(web)* WDA-01 F1 diseño — limpia huérfanas dark mode + fix leak animejs/timers
- *(desktop)* drop unused useEffect import (tsc clean after FIND-23 cleanup)
- *(desktop)* FIND-24 WCAG AA contrast calibration for both themes
- *(core)* MOD-03 trigger_compaction deja de ser stub
- *(sdk)* REVIEW-14 keys cortas devuelven error de corrupcion sin panic — unwraps fragiles eliminados
- *(web)* REVIEW-18 fija turbopack root — warning package-lock stray eliminado
- *(core)* REVIEW-15 alineación garantizada en cast f32 vector_data — sin UB potencial
- *(deps)* REVIEW-08 actualiza h2 0.4.15->0.4.18 — RUSTSEC-2026-0258, deny advisories verde
- *(core)* MOD-01 valida antes de escribir WAL — insert/update rechazado no resucita datos
- *(python)* MOD-17 drain del OpGate fuera del GIL — close() concurrente sin deadlock
- *(core)* REVIEW-09 resetea latch saturated tras decay — warming vuelve a aprender en long-running
- *(wasm)* CORE-02 graph-store visible para query_iql en modo standalone
- *(server)* MOD-12 construye índices al arrancar server HTTP — text search funcional en DB fresca
- *(mcp)* MOD-07 acepta notifications JSON-RPC sin id — handshake clientes estrictos
- *(task-system)* H1-H3 hardening del campaign server
- *(build)* BND-06 nextest scope-safe + BND-01 LinkError pkg WASM + BND-03 tipos verificados (3 fixes técnicos)
- *(sdk)* MCP-28 bulk import writes __vanta_* fields (records now addressable) + payload key fix
- *(gov)* review P2-01 — master-index Investigaciones->research residual, skill configuration rate_limit 600 re-sync hash-SAME, addendum post-ejecucion al informe
- *(ci)* nextest default profile filtra binarios reales python/hnsw_recall (antes python_sdk_boundary/hnsw_recall_certification inexistentes) + TEST_MAP alineado (GOV-C1)
- callers export_namespace con filtro (vantadb-python) + clippy mcp_tests (VS-CORE-04)
- vectores Binary insertables y recuperables por get() (HNSW sin grafo)
- *(cli)* AUD-051 update cmd_put call in search.rs test for metadata param
- *(mcp)* AUD-050 inject_context clear error for invalid thread_id type
- *(mcp)* AUD-046 memory_put validates vector dims before insert
- *(mcp)* AUD-045 memory_put accepts expires_at_ms + sparse_vector
- *(cli)* AUD-044 ensure text indexes current on fresh DB search
- MCP-15 stack overflow — single-level prefetch guard in get() (prefetch re-entrancy)
- MCP-01..04 server MCP — text search, distance_metric, distance semantics, dim validation
- *(FND-13-F1)* re-sync numeros con BENCHMARKS §2 vigente (changes-required P2-01) + comentario volatile_cache en test FND-02
- *(FND-13-F1)* elimina claims fantasma de performance en web (5,400 vec/s etc) - Regla 11, numeros citados de BENCHMARKS
- *(TSYS-06-F1)* corrupcion visible + writes atomicos con checksum + WIP check-and-set + parsers.mjs (29/29 node --test)
- *(FND-05-F1)* stubs Python alineados con runtime (connect, __version__, retornos get_memory/list_memory/put) + limpieza maturin features
- *(FND-06-F1)* zero-norm cosine propaga error en TS (elimina fallback silencioso euclidean)
- *(FND-02-M3)* cierra race delete-vs-consolidate con lock completo + version check
- *(FND-01-F1)* guard memory usa RSS real del proceso (cierra subestimacion 6.5x OOM)
- *(FND-02-followup)* drop vstore guard en insert_to_cf antes de refresh_index (Regla 8 lock order, audit P2-01)
- *(FND-02)* deadlocks eviccion multi-indice (lock no reentrante + write guard) + regla concurrency-async
- *(python)* explicit backend error + unify new/connect config (AUD-037)
- *(server)* reject unknown CLI flags with exit 2 + tests (AUD-033)
- validate sparse vector dims on decode (AUD-023)
- propagate active-txn corruption as error instead of panic (AUD-031)
- *(ERR-015)* graceful shutdown via stdin EOF before kill in request_shutdown
- *(ERR-010)* snapshot se abre como DB reabrible (layout data/ simetrico)
- *(ERR-010)* scan_nodes_page no excluye el nodo id 0 (cursor vacio)
- *(ERR-026)* delegate list/null metadata filters to core instead of rejecting
- *(ci)* COV-004 — ref ADR-015 en job coverage + step renombrado >=80%
- repair maintenance deadlock, zero-norm search, putBatch versioning
- *(wal)* persist round-robin position across opens (ERR-050)
- *(AUD-021)* rate limiter fails closed instead of serving unthrottled
- *(ERR-033)* list(limit=0) returns no records instead of clamping to 1
- *(ERR-031)* VecIndex::add propagates rejections as Result
- *(task-system)* reject placeholder campaign ids
- *(harness)* cerrar 5 gaps familia A de investigación agent-engineering
- *(ERR-015)* graceful shutdown with timeout before kill in child_process
- *(ERR-026)* explicit parse_metadata for non-scalar filters
- *(integrations)* remove empty stubs from public surface (FEAT-07)
- *(sdk)* reject zero-norm cosine query vectors (ERR-028)
- *(http)* return 4xx/5xx on query error (ERR-027)
- *(storage)* edge_count u16 overflow on persist (ERR-029)
- *(storage)* immediate insert visibility for get (ERR-014)
- *(bindings)* put_batch respects per-record namespace (ERR-030)
- *(index)* ACORN second-hop after repair_orphans (ERR-020)
- *(bindings)* preserve u128 node IDs in wasm (ERR-024)
- *(bindings)* preserve u128 node IDs in python (ERR-023)
- *(mcp)* preserve u128 neighbor ids as strings (ERR-025)
- *(index)* decrement inbound counters on delete (ERR-012)
- *(bench)* force HNSW path in pure bench (ERR-019)
- *(index)* correct random_layer level distribution (ERR-018)
- *(storage)* revert inventory stats on txn abort (ERR-013)
- *(wal)* surface truncated-shard replay instead of silent loss (ERR-011)
- *(storage)* bounds-check vector_store indexing (ERR-003)
- *(storage)* SIGBUS handler sets flag, no re-execute loop (ERR-002)
- *(storage)* hold insert_lock across checkpoint/save race (ERR-010)
- *(storage)* minimize HNSW insert_lock hold during queries (ERR-035)
- *(mcp)* bounded collection stats/list/delete to prevent OOM (ERR-021)
- *(bindings)* clamp top_k in python+wasm to avoid giant alloc (ERR-022)
- SEC-01 UAF in VantaSearchHit __array_interface__ (vantadb-python)
- *(parser)* reject reserved keywords as optional alias (WHERE/RANK data loss)
- *(mcp)* remove dead collect_stats (AUD-012 clippy), zero-norm-safe vectors in stats test
- *(index)* complete select_neighbors<F> refactor + overflow guards in search.rs (AUD-012/013/014, ERR-001)
- *(index)* canonical select_neighbors pruning + cap over-capacity at 2m, dedupe clippy forks (AUD-012/013/014)
- *(server)* remove expect in governor config, add wasm LICENSE, sync deny ignores (NV-02/03/05)
- *(frontend)* i18n skip-link via t('common.skipToContent') (AUDREP-42)
- *(dx)* cross-platform shell in Justfile (AUDREP-26)
- *(build)* move exclude=["fuzz"] from [workspace.package] to [workspace] (AUDREP-23)
- *(frontend)* i18n toast via t('terminal.codeCopied') (AUDREP-47)
- *(wal)* remove dead misleading last_offset field (AUDREP-56)
- *(index)* explicit warn + empty results for zero-norm cosine query (AUDREP-55)
- *(storage)* guard postcard deserialization at trust boundary (AUDREP-45)
- *(frontend)* derive html lang from i18n DEFAULT_LANG, drop suppressHydrationWarning (AUDREP-39)
- *(storage)* saturating 64-byte alignment, no overflow (AUDREP-34)
- *(wal)* require timestamp in archive names, drop mtime fallback (AUDREP-37)
- *(sdk)* post-filter cursor detection prevents infinite pagination loop (AUDREP-30)
- *(server)* generic panic message to client, detail logged (AUDREP-32)
- *(wal)* fsync parent dir after durability-critical renames (AUDREP-35)
- *(index)* reject zero-norm cosine inserts (AUDREP-27) + total-order NaN eviction (AUDREP-29)
- *(mcp)* RAII guard sole decrement for active_requests (AUDREP-44)
- *(wal)* quarantine corrupt tail to .corrupt before truncation (AUDREP-36)
- *(crypto)* bound EncryptionStream frame_len to prevent OOM (AUDREP-31)
- *(index)* clamp euclidean distance to non-negative (AUDREP-28)
- *(storage)* saturating math in write_node_to_vstore growth (AUDREP-33)
- *(repo)* unignore .env.example to track env template (AUDREP-24)
- *(server-tests)* add trusted_proxies field to AppState in tests
- *(ts-sdk)* accept numeric version/node_id/timestamps in isMemoryRecord
- *(storage)* drop mmap before rename in save_vector_index on Windows
- *(wal-shipping)* add shutdown signal and per-failure backoff to run_loop
- *(wal)* persist shard count in metadata, reconcile on reopen
- *(integrations)* bump 9 python adapters to 0.5.0 and pin vantadb-py>=0.5.0,<0.6.0
- *(wal)* hold shard lock across rotate sync+swap to avoid lost writes (AUDREP-15)
- *(index)* invalidate cached IVF when node count changes (AUDREP-09)
- *(index)* clamp sq8 remainder loop to min len to prevent OOB panic (NV-01)
- *(security)* only trust X-Forwarded-For from configured proxy (AUDREP-11)
- *(security)* key stretching for Cipher key derivation (AUDREP-10)
- *(security)* limit HTTP request body to 1MB in /api/v2/query (AUDREP-12)
- *(engine)* realinear tokenizer de frases con postings del indice (regresion 995258e9)
- *(adapters)* openai pytest-asyncio, dspy .passages, letta fallback listing
- *(langchain)* MMR tie truncation y add_documents vacío
- *(providers)* query_sparse en adapters ollama y litellm
- *(core)* query_sparse en adapter openai y MemoryGovernor limit 0 bajo Miri
- *(bench)* resolver errores clippy en graphrag_bench example
- *(node)* regenerar package-lock desync de @emnapi/runtime (H02-L2-002)
- *(python)* quitar hang de CI en test_load y migrar put_batch a kwargs
- *(web)* resolver colision de tipos WASM en code-playground (H03-WEB)
- *(index)* estabilizar invariante HNSW concurrente con flush del batch pendiente
- *(AUDREP-03)* no tragar errores de tombstone write_header en ops.rs
- *(AUDREP-08)* evitar colision de timestamps y rename no atomico en archive_segment
- *(AUDREP-13)* log warning por request no autenticada en dev mode
- *(AUDREP-02)* replace inalcanzable expect con let-else en scan_nodes_page
- *(AUDIT-03)* Miri guard core - tree-borrows sobre 7 bloques UB (INV-024)
- *(AUDREP-01)* validate truncated vstore before compact copy
- *(INV-015-B)* touch targets clear-search a 44px
- *(AUDIT-04)* acotar cache_warmer.co_access para evitar OOM en searches
- *(AUD-011)* portar OpGate a bindings python/wasm
- *(AUDIT-01)* congelar buffer __array_interface__ ante drop en PyO3
- *(TECH-01)* setear VANTADB_STORAGE_PATH en child MCP (--db respetado)
- *(AUD-001)* subir MSRV y eliminar COPY a crates inexistentes en Dockerfile
- *(TECH-02)* wrapper reindexHnswFromText usa export real del pkg
- *(WEB-18)* align web pricing with GO_TO_MARKET (drop Team \ tier)
- *(task-system)* make harness parser tolerant to **Estado:** and ## Tarea N: formats
- *(server)* adapt benchmarks/e2e/helpers tests to ENT-04 ServerState fields
- *(core)* enforce read_only on all storage mutations (COMP-029)
- adapt graphrag test/example to 5-arg add_edge (COMP-021)
- *(bindings)* drain in-flight ops on close for durable native backend (COMP-029)
- *(bindings)* harden native napi-rs backend + sync WASM temporal-edge types (COMP-029)
- fix(web) update product version to 0.5.0 (M6)
- *(opencode)* add frontmatter to campaign-executor SKILL.md
- *(ci)* activate prometheus feature for server e2e metrics test
- *(index)* bound IVF deserialize counts against input length
- put_batch rebuilds derived and text indexes after batch insert
- *(gate)* reusable workflow must not request contents: read — caller grants only checks: read; docs for ci-gate/chaos + gate sections

### Other

- *(campaign)* cierra STABLE-04 (gates 1-6 + test-mcp.py 4/4) + sync plan/backlog/avance
- *(plans)* sync BND-08 + FUT-12-spec completados + recitations
- bump opencode submodule (0b39520 regex-escape + file-scope persist + refs docs/tasks)
- bump opencode submodule + docs/tasks canónico (D1-D16)
- *(adr)* spec WAL fsync-batching opt-in (FUT-12-spec)
- *(node)* prepublish verificado dry-run + checklist (BND-08)
- *(campaign)* Wave 1 3/3 - FIND-62 + GOV-TK7 + STABLE-06 (plan sync + backlog + avance)
- *(ts)* gate npm Fast Gate medido (STABLE-06)
- *(campaign)* Wave 0 3/3 - FIND-63 + GOV-TK3 + MEM-63 (plan sync + backlog + avance)
- *(web)* ojos del logo-mark alineados al hero mark + favicon regenerado
- *(assets)* avatares org PNG + baja de banner.gif y vantadb-avatar.png
- *(web)* composiciones OrgMark/OrgBadge + scripts de render GIF
- *(assets)* org-mark GIFs dark/light single-play + variantes org-badge
- *(api)* drift yaml-real ×3 (GOV-TK3)
- *(cli)* regenerate completions for restore --dry-run flag
- *(backlog)* FIND-62/63 del spike FIND-61 (ERR-010 txn + Never muerto)
- *(spike)* desglose insert_lock vs fsync + decisión batch (FIND-61)
- FIND-59 insert_lock decision (d) + ADR-037 + FIND-61 spike
- *(backlog)* mark FIND-53 resolved (embed_texts documented)
- *(mcp)* document embed_texts tool (close FIND-53, unblock docs-coverage gate)
- *(web)* visual evidence WEB-09 after state
- *(plan)* quality-gtm-wave budget ledger
- *(node)* share workspace target dir for standalone crate (disk space)
- *(deps)* sync Cargo.lock tracing edge (follow-up b778654f)
- *(cli)* regenerate shell completions for doctor --fix/--force (GOV-TK1)
- *(review)* add certify HTML report + fix eslint warnings (next/image, navbar deps)
- *(review)* certify report 20260903-171641 (8.7/10 B) + FIND-60 + index sync
- *(deps)* drop unused tracing from vantadb-server (cargo-machete certify gate)
- *(backlog)* FIND-59 serializacion insert_lock + FIND-58 re-validado (108 activas)
- *(state)* quality-gtm-wave completada 12/12 - checkpoints y handoffs humanos
- *(campaign)* cierre quality-gtm-wave 12/12 + archivo plan con retrospectiva
- *(bench)* AUD-045 medido — no aplica, premisa muerta (evidencia)
- *(ci)* track .cargo/config.toml (wasm32 getrandom backend) - keep rest of .cargo/ ignored
- *(bench)* A/B canal ingesta multi-consumidor — RES-03 medido, no aplica refactor
- *(avance)* RES-15-C registro regla dos backlogs con commit (progreso)
- *(process)* split backlog negocio/técnico (RES-15-C)
- *(deps)* bump nanoid 3.3.16 to 3.3.18 (fix CVE-2026-67213 HIGH)
- *(backlog)* RES-12 cierre - fila eliminada + avance web + recitation plan (wave2)
- *(deps)* unify vantadb-python lru 0.16 to 0.18 with core
- *(deps)* bump chacha20 0.10.1 to 0.10.2 (clear yanked warning)
- *(security)* redact expired Perplexity pre-signed S3 URLs from research archive
- *(backlog)* FUT-12/13/14 roadmap huérfano (RES-09)
- *(deps)* bump actions/setup-python 5.6.0 to 7.0.0 (SHA-pinned)
- *(deps)* bump pypa/gh-action-pypi-publish 1.14.0 to 1.14.1 (SHA-pinned)
- *(backlog)* FIND-56 - vantadb-server/Dockerfile roto, hardening.md lo documenta funcional
- *(backlog)* FIND-56 server Dockerfile roto (hallazgo SRV-07) + wave1 recitations en plan
- *(deps)* bump actions/cache 4.3.0 to 6.1.0 (SHA-pinned)
- *(wheels)* aarch64 linux + SHA real formula (MKT-18h)
- *(deps)* bump github/codeql-action 4.37.0 to 4.37.1 (init + analyze)
- *(backlog)* MKT-18i cerrada - compose demo shipped; fila re-escalada a AnythingLLM upstream (bloqueado, evidencia .env.example)
- *(deps)* bump actions/checkout 4.4.0 to 7.0.1 (SHA-pinned)
- *(docker)* unprivileged + release build wiring (SRV-07)
- *(backlog)* GOV-TK1 re-escalada - falta solo verificador de restore
- *(deps)* bump aes-gcm 0.10 to 0.11 and sha2 0.10 to 0.11 - migrate to aead 0.6 API
- *(deps)* bump sysinfo from 0.38.4 to 0.39.6
- *(deps)* bump ratatui 0.28 to 0.30.2 and crossterm 0.28 to 0.29.0 (tui feature)
- *(deps)* bump serial_test from 3.2 to 4.0.1
- *(deps)* bump lru from 0.16.4 to 0.18.4
- *(deps)* bump rust-minor group - portable-atomic 1.15.0, arrow 59.3.0, wide 1.7.0
- *(deps)* bump rust-patch group - thiserror 2.0.20, zerocopy 0.8.56, fjall 3.1.10, pyo3 0.29.2, clap 4.6.6, twox-hash 2.1.4
- *(gov)* verificar URL vantadb-examples (GOV-TK9)
- *(plan)* quality-gtm-wave 12 DO + purga 3 filas drift paso0
- *(backlog)* veredicto final alta-prioridad - 2 obsoletas, 3 re-escalas, DEC-02 ICEBOX
- *(limpieza-2)* WEB-05/WSM-09 removidas con evidencia + RES-07 task file reabierto
- *(limpieza)* auditoria plan alta-prioridad - 5 filas con evidencia removidas, 2 restauradas, 6 reaperturas
- *(backlog)* counter 125->121 — pipeline task FIND-52..55 4/4 cerradas
- *(progreso)* FIND-52 cerrada — fila Backlog removida + registro en bindings y core-engine
- *(progreso)* FIND-53 cerrada — fila Backlog removida + registro en core-engine
- *(progreso)* FIND-55 cerrada — fila Backlog removida + registro en core-engine
- *(progreso)* FIND-54 cerrada — fila Backlog removida + registro en core-engine
- *(backlog)* registro FIND-52..55 colaterales campana error-observability
- *(campaign)* cierre error-observability 9/9 + archivo plan + checkpoint
- *(avance)* ERR-DESK-01 completado — plan Task 6 + registro desktop
- *(plan)* ERR-OBS-01 Task 9 ✅ con evidencia de verificación + hash 817b83e5
- *(avance)* ERR-WEB-01 completado — plan Task 7 + registro web-frontend
- *(backlog)* FIND-52 — colateral d.ts regeneration drift (BND-10) documentado en cierre ERR-TS-01
- *(node)* closure ERR-TS-01 — map_err con contrato code-prefijo documentado + crate fmt-clean (drifts WSM-10 preexistentes)
- *(gov)* GOV-C7 Backlog counter + ROADMAP banner sin cifra — Wave3
- *(gov)* GOV-C6 CONFIGURATION.md sync — last_reviewed 2026-09-02 + sweep 44 vars 0 drift (Wave3)
- *(closure)* ERR-CORE-01 plan+avance+EMBEDDED_SDK — Wave 1 done, code() canónico listo (desbloquea Wave 2)
- *(plan)* MEM-09 L0 capture idempotente re-verify Wave3 33/86 — ponytail reuse 0 lineas, 5+5 l0_capture tests pass, capture --lib 5 pass
- *(gov)* GOV-C5 operations/master-index taxonomia 26->35 (6 categorias, 35 md parity)
- *(plan)* ERR-CORE-02 complete -- af0bb8b8 + 5/5 contracts pass (Wave 0 done)
- *(clippy)* deny unwrap/expect en prod + anyhow bins (ERR-CORE-02)
- *(gov)* GOV-C4 operations/master-index taxonomia 35/35 + last_reviewed 2026-09-02
- *(gov)* GOV-C3 verify Daily Backup Verification (§3.1 + verify.ps1 guard)
- *(gov)* GOV-C1 nextest audit verification scope-safe long names (ponytail 0 líneas)
- *(gov)* GOV-B6 79 tools — SKILL.md/api-reference/MCP.md sync (49 core +30 ext), plan contrato 33->79, hash-SAME
- bump opencode submodule GOV-C2 task file
- *(gov)* GOV-C2 master-index taxonomia — frontmatter 2026-09-02 + api 12→18 files (GOV-C2 Wave2 paralelo, disjoint GOV-B6/C1)
- *(gov)* GOV-B4 openapi parity 37/44 — router.rs gate fix (Wave2 SHIP)
- *(plan)* ERR-DOCS-01 complete -- 962831ae + 6/6 contracts pass
- *(gov)* GOV-B3 consumo guard anti-regresion — BENCHMARKS §11 + canonical_p99 anchor + verify.ps1 compile-gate (Wave2 batch2, disjoint MEM-02/03)
- *(gov)* GOV-B2 runbook DR sin comandos fantasma — rephrase 2 notas ghost, restore --input/--force/--rebuild + §3.1 intacto (Wave2 SHIP, disjoint MEM-01)
- *(gov)* GOV-B1 case_studies → archive interno — verify guard + task file (Wave2 docs-only)
- *(plan)* MEM-01 F1 search profile recitation Wave2 — verify planner 38 + search_profile 11 + parser 117 (6a50b8ee landed, ponytail reuse)
- *(bench)* RES-05 benchmark semántica scores — bench criterion minimal + BENCHMARKS §9 (Wave1c P38)
- *(gov)* GOV-A5 registros live — 3 captures crates.io/PyPI/npm 0.5.0 live 2026-08-01 + verify-log
- ERROR_HANDLING.md + code tables (ERR-DOCS-01)
- *(gov)* GOV-A3 probes CLI reales doctor/backup/restore probes
- *(gov)* GOV-A1 openapi parity 39->37 docs-only
- *(gov)* GOV-A4 snippets parity — validate_doc_snippets 0 FAIL (27 PASS 27 SKIP)
- *(plan)* añade bloque Question Gates a error-observability-excellence
- *(backlog)* trackea 4 god-files para split — FIND-48..51
- *(plans)* archivar 2026-08-25-research-web-quickwins 7/7 Done (WEB-09 sutil+A11y)
- *(avance)* slice2 Phase11 9/9 + AUD-043/044/047 + FIND-23 -> activo/*.md, Backlog 130->117
- *(plan)* WEB-09 plan Done — refinamiento sutil+A11y 3 slices completado
- *(plan)* alta prioridad paralelo GOV(30)+P27(38)+P38(17)+MCP-35 — 86 tareas en 29 sub-waves MAX_CONCURRENT=3
- *(backlog)* normalize BND-08/09/12 split rows + drop orphan BND-10/11 fragments + fix MD028 blockquotes
- *(server)* split cli_server god-file (4919L) by concern under src/server
- add MCP-35 HTTP fallback discovery design (ADR-036 + spec)
- *(mkt-04)* fix unverified claims in Reddit launch posts, mark ready-to-publish
- *(ci)* formaliza 3 exclusiones fast gate (FIND-22)
- *(plans)* archiva 3 planes completados (fast-gate-green, backlog-triage, master-pipeline)
- bump opencode submodule — FIND-035 task file
- *(progreso)* sync 43 drift backlog → avance (cargo check providers OK)
- update configOpencode to 8816df4 (ignore bun.lock)
- update configOpencode to 87dae19 (re-add MCP v2 + package.json ignore)
- restore tui completions (reverted by 7a17bc2d via stale binary regen) + backlog-triage plan state updates from sub-agents
- *(plan)* add 2026-09-01-fast-gate-green — triage 110 items, 2 DO (FIND-035 lint cascade, FIND-036 mcp test compile), Gate P approved; 3 premise-falsa SKIPs verified (FIND-44: 22 ADRs existen, TS-01 completada, TS-02 _native ya async)
- update configOpencode submodule + SKILLS-MANIFEST (194 skills, SDP v2)
- *(FIND-40)* fix drift in 3 core API docs + TODO for rest
- update .opencode submodule with PY-01 task file
- update .opencode submodule to latest (docs updates for configOpencode)
- *(progreso)* archive fast-gate-residues plan (4/4 tasks completed)
- add rustdoc workflow generating API reference (RES-11)
- remove .agents folder (skills moved to .opencode/skills submodule)
- extract .opencode to configOpencode submodule (private repo)
- *(insta)* add 2 query_result snapshot tests closing TBH-06
- *(progreso)* FIND-MCP-001 arqueológica + FIND-036 test_embed_texts.rs
- regenerate shell completions for tui subcommand + ignore runtime artifacts
- *(progreso)* AUD-043 arqueológico + FIND-035 lint cascade nuevo + stale cleanup
- *(cleanup)* post-P48 residues - remove legacy physical_plan monolith, broken opencode.json, complete MCP context test, add fast-gate plan + curriculum
- *(progreso)* P48 cierre - 22/23 TBH-* completados, plan archivado, pipeline-state actualizado
- *(TBH-18)* record dhat evaluation decision (not introduced; no alloc regressions documented)
- *(TBH-16)* record divan evaluation decision (not introduced; criterion sufficient per D1+D5)
- *(TBH-10)* add criterion conversion pattern lesson to lessons.md
- *(TBH-10)* convert bench_concurrent to criterion_main (generates estimates.json for nightly regression)
- *(TBH-21)* mark TASK-21 ✅ + add RECITATION block to plan file
- *(TBH-21)* document CoverageThreshold=60 review cadence in CI_POLICY.md (quarterly)
- *(TBH-07)* add recitation block to plan file
- *(TBH-07)* add cargo-mutants weekly mutation-test job to heavy-certification workflow
- *(TBH-11)* extend heavy-bench-nightly to 8 benches (add canonical_p99, memory_budget, incremental_bench, ivf_bench)
- *(TBH-20)* extend ci-examples to 3-OS matrix (ubuntu+windows+macos)
- *(TBH-17)* record loom evaluation decision (not introduced; current concurrency_parity.rs sufficient)
- *(TBH-15)* consolidate audit-tokens to ps1 only (delete .sh duplicate)
- *(TBH-23)* unify cargo fmt --check scope across Justfile/verify.ps1/audit-all.ps1 (target: --all)
- *(TBH-12)* add data/README.md + datasets/README.md (consolidated dataset registry)
- *(TBH-19)* add markdownlint-cli2 to pre-commit (mirror gate-docs-21)
- *(TBH-22)* trigger release-binaries on push tags v* (alongside release published)
- *(progreso)* TBH-13 ✅ — 6 SHA-pins (5 audit + 1 upload-artifact), commit 42141706
- *(TBH-13)* SHA-pin 6 third-party action refs in desktop.yml + opencode.yml (supply chain hardening)
- *(TBH-04)* add develop branch to 6 workflows + release.yml (D6 Dependabot alignment)
- *(TBH-05)* untrack benchmark artifacts (data_comp_bench, data_bench_db) - gitignored
- *(TBH-03)* remove schedule-only condition from ci-gate (universal gate per D3)
- completions update --format md (MEM-62) + lessons + Backlog note
- *(progreso)* TBH-01 ✅ — verify_datasets gate done, backfill to ci-cd domain
- progreso — cerrar campaign 2026-08-29-full-backlog-parallel
- *(desktop)* IPC vs MCP transport guide + embedding setup
- desktop/src-tauri — embed.rs cfg guards + Cargo.lock (MEM-58 follow-up)
- web/* — playground executor + lighthouse + visual inspection
- WIP cleanup — task files + docs + plan sync + Cargo.lock
- *(operations)* MCP snippets use canonical vanta-cli command
- FIND-41 — ADR clusters Leiden fragmentados (cohesion 0.59-0.71)
- FIND-24 — Cursor cross-namespace + perf list_window
- FIND-42 - ADR boundary src->skills (inversion de dependencia)
- GOV-TK4 — Re-medición coverage local (llvm-cov + CI artifact)
- REVIEW-10 - Split cli_server.rs por concern (state/routing)
- REVIEW-12 — Split api.rs por dominio (memory/search/namespaces/admin/graph)
- REVIEW-12 — Split api.rs por dominio (memory/search/namespaces/admin)
- STABLE-02/03 — Validar vanta-proxy y vantadb-server (gates 1-6)
- MEM-67 — TokenEstimator auto-detección tiktoken (idempotente)
- *(plan)* MEM-62 marcado ✅ COMPLETED (commit fc0bf52d)
- WSM-13 — Estrategia de bundle documentada (lazy-load + comparativa)
- PROV-05 — Extract shared helpers providers (record_to_pydict, err_to_py, metadata)
- PERF-BENCH-01 — A/B benchmark node native vs TS WASM (canonical)
- BND-08 — Pipeline npm release napi-rs (5 targets + musl)
- WSM-10 — Score/distance semantics unificada (3 transports)
- TS-03 — Documentar semántica score/distance (zero-norm cosine resuelto)
- SRV-08 — Hardening guide + competitive positioning
- FIND-43 — Aplanar builder CacheWarmer (no recursivo)
- FIND-38 — Consolidar helpers serialization cycle
- CORE-01 — sincronización cierre W4-SOLO + nota ADR-032 owner
- SRV-03 — Corregir links distribución (GitHub Release, no crates.io)
- WSM-08 — Corregir docs TS contradictorias (WASM persistence)
- FIND-46 — Documentar cargo semver-checks en pre-release gate
- MCP-40 — Registry manifest + ecosystem listings
- PY-03 — Sección 'Import name' en PYTHON_SDK.md + registro bindings
- FIND-24b — Fix drift MCP skill (links + tool count)
- FIND-40 — Fix drift in 3 core API docs (EMBEDDED_SDK, PYTHON_SDK, HTTP_API)
- FIND-44 — verify ADRs exist, contract satisfied
- HIGH-009 — Session Cleanup al Cerrar Campaña — verificación idempotente
- HIGH-008 — Autonomous Flag en Plan File — verificación idempotente
- HIGH-007 — Re-validar Skills tras Discovery — verificación idempotente
- HIGH-006 — detect_changes en plan.md Paso 0 — verificacion idempotente
- CORE-005 — SDP Unificado — verificación idempotente
- CORE-004 — Task File Template Completo — verificación idempotente 20/20
- CORE-003 — Question Gates Enforcement Automático — verificación idempotente
- CORE-002 — campaign_validate_output (LLM05) Enforzado en ACT — verificación idempotente
- CORE-001 — plan recitation COMPLETED
- plan reset 19 DO (PENDING) + 1 DEFER para re-ejecución trazable
- *(plans)* archive 2026-08-27-backlog-v2 7/7 COMPLETED — FIND-34/35/36 + CORE-01 + STABLE-01/03/08
- STABLE-08 — Medición Fast Gate default ampliado (test/default-all + just verify, 3 corridas cold, Heavy justificado)
- add comprehensive visual audit script and production audit results
- *(EMB)* Phase 11 embeddings local-first 9/9 COMPLETED — backlog + progreso + archive plan
- recitation EMB-09 — Qwen3 >3GB wiring + doc
- *(EMB-09)* Qwen3 >3GB excepcion + README
- *(EMB-08)* embed-local docs + QUICKSTART + EMBEDDINGS
- recitation EMB-07 — bench comparativo 9 modelos
- recitation EMB-05 — MCP tool embed_texts con embed_batch
- close WEB-07 stale WIP (was blocking EMB-01)
- STABLE-03 validate vantadb-server gates 1-6 (fix Cargo.toml path dep version for cargo package)
- archive research-web-quickwins plan (7/7 QW completed, WEB-09 gate visual pending)
- STABLE-01 validate vanta-memory gates 1-6 (fix Cargo.toml path dep version for cargo package)
- archive 2026-08-27 backlog-pipeline 7/7 — bindings + ci-cd avance + Backlog 109→102 + retrospective
- STABLE-00 ADR-031 default-members promotion DoD + CI_POLICY gate
- *(avance)* FIND-39 record in core-engine + remove from Backlog
- *(task)* MCP-38 mark COMPLETED + verify evidencia
- *(avance)* FIND-37 completed — update Backlog, core-engine, history, lessons, task file
- archive desktop quickwins plan + retrospective 10/10
- *(learnings)* DESKTOP-QW8..QW10 + WASM + QW-1 lecciones E2E TTL drift + server feature
- *(avance)* DESKTOP-QW8..QW10 — desktop quickwins Wave3 (H-11, H-15, H-07) en dominio desktop
- mark WEB-03 as verify-only done (0 refs, assets in public/assets/) - research-web-quickwins
- archive integrations-research-wins plan (9/9 QW completed, 4 waves, fixes in 96e143ec)
- archive research-providers-quickwins plan (7/7 tasks completed, fixes in 2754c783) with updated recitations
- add P47 promotion criteria STABLE-00..09 for default-members
- PROV quickwins task files WASM-QW + PROV-01..09 (verify-only, fixes in 2754c783) + archive WASM recitation
- archive WASM quickwins plan + WASM-QW1..5 task files (verify-only, fixes in 53f080e5)
- integrate codebase-memory-mcp code intelligence MCP
- *(avance)* DESKTOP-QW7 recitation sync + avance desktop + lessons H-04
- archive research-vantadb-ts-quickwins plan + register TS-02/05/06/07/08 in bindings
- TS-07 smoke-test tarball pack→install→quickstart (TS-05 engines) wired in release-npm-61.yml
- TS-08 verify CDN ESM jsDelivr vs esm.sh (Rollup failure reason) + WASM_PERSISTENCE hardening
- *(backlog)* DESKTOP-QW5 — limpiar filas DAUD-01..09 stale (H-13) + historial desktop
- lessons TS-06 + desktop quickwins
- TS-06 gate Fast Gate for TS tests (<5min, PR+push, no continue-on-error)
- *(ci)* TS-05 preserve engines in npm tarball
- archive completed vantadb-node hardening plan
- archive completed SRV quick wins plan
- mark SRV quick wins plan tasks as completed
- add alt_api_key config to CONFIGURATION.md (SRV-03, docs-coverage)
- archive plans + update avance (Providers + Integrations quick wins)
- add providers CI workflow (PROV-09)
- register Python SDK quick wins completion (PY-QW1..5)
- *(python)* remove legacy tuple API from put_batch (PY-QW2)
- remove obsolete asset PNGs (demo_terminal, social-preview)
- archive research/review docs and add plan + validation notes
- *(web)* Lighthouse re-measure command + workaround EPERM (WEB-05)
- cierre ejecucion INV-DECIDE waves quick-wins (9 commits, bloqueos y incidentes registrados)
- *(ts)* gate vitest Fast Gate + smoke-pack script + fix async _native envuelve rechazos (TS-02/05/06/07/08)
- *(node)* metadata npm engines/os/cpu + README (node research H-02/H-03)
- investigacion INV-* completa (9 modulos, sintesis global INV-DECIDE, planes quick-wins)
- *(backlog)* P42 backup-restore-chain completado (3/3) - plan archivado + retrospectiva
- *(storage)* FIND-26 remove dead PITR code (wal_archiver.rs, feature pitr) - ADR-014 superseded
- *(backlog)* new plan batch-backup-restore-chain (FIND-25 -> MCP-34b -> FIND-26)
- *(workflows)* trigger /oc en comentarios para OpenCode
- *(mcp-research)* informe INV vantadb-mcp + cierre bookkeeping asociado
- *(troubleshooting)* errores 500 en /api/v2/* de Vanta Studio dev
- *(mcp)* habilita playwright y agent-search en config del proyecto
- *(backlog)* P41 batch core-fixes-research completado (9/9) - plan archivado + retrospectiva
- *(research)* RES-01 ACID WAL v2 (GO condicional) + RES-02 backup/restore + RES-03 session layer no-go - hallazgos ruteados
- *(ci)* FIND-22 document fast gate test exclusions in CI_POLICY
- *(index)* AUD-045 replace IVF vector clones with f32_slice_similarity (bench -59%)
- *(backlog)* P39 +3 tareas PRX-11..13 tras auditoria de cobertura del research
- *(backlog)* P39 vanta-proxy gateway completo — 10 tareas PRX (investigacion 3 frentes)
- *(backlog)* P27 Wave 2 memoria agentica — 12 tareas MEM-59..70 (investigacion 3 frentes)
- *(backlog)* P41 batch core-fixes-research - Wave 0 complete (3/9), paused by user
- *(index)* AUD-047 extract metric_score closure, dedup ~35 lines in layer.rs
- *(experimental)* sincronizar taxonomia experimental al estado 0.5.0 + promociones
- *(backlog)* MCP-35 fallback HTTP automatico para N instancias MCP sobre la misma BD
- *(bookkeeping)* cierra AGT-04 huerfano + limpia assets de staging obsoletos
- *(audit)* reportes de auditoria 2026-08-25 pendientes de commitear
- *(backlog)* new plan batch-core-fixes-research (9 tasks: 5 core fixes + 3 research + 1 docs)
- *(backlog)* P40 batch desktop-ux-core completado (8/8) - plan archivado + avance
- *(desktop)* E2E-VISUAL - Playwright smoke guard (UX-19) + theme visual check (DAUD-01) + FIND-23
- TIR-08 verified - research criteria already in research-agent.md (1c7660dc)
- FIND-11 desktop README + lazy-load wasm note + npm naming clarification
- *(agents)* skills count 163 -> 162 (a11y-shared es carpeta de recursos, no skill)
- remove dead ui prototype and stale benchmark results
- FIND-17 brand identity naming convention (ADR-030) + README note
- *(server)* MOD-15 nits - drop middleware.rs re-export, empty sysinfo feature, main.rs comment, ServerState helper
- *(commands)* consolidate task/audit/finding flows — one canonical path per intent
- *(task-system)* podar memoria obsoleta + archivado de budgets de planes
- *(rules)* actualizar reglas obsoletas al estado real del código
- *(agents)* corregir referencias muertas y datos stale en AGENTS.md
- *(backlog)* gran limpieza — 100 filas completadas removidas, secciones cerradas colapsadas, GOV-C7 recount 84
- *(backlog)* new plan batch-desktop-ux-core (8 grouped tasks, ~20 backlog rows)
- *(task-system)* enforce SDD question-gates — spec-first, public-symbol detection, auditable gates
- *(backlog)* cleanup - remove 31 completed task rows from P35/P38/P39 batches, update open count to 77
- *(backlog)* P39 batch colaterales-deuda-desktop completado (14/14) - plan archivado + avance + MCP.md snapshot_create doc
- MEM-51 verified as already resolved (a9b65224, Última Milla batch) - 0 diff
- *(agents)* AGT-03 verify P2 debt refs (4 resolved, 2 updated) + AGT-06 anti-drift refs script
- *(agents)* AGT-04 clean-opencode-loop script + GC for corrupt/tmp files
- AGT-02 update CodeGraph stats to real index (20.5K symbols / 71.4K edges)
- *(storage)* MOD-06 WAL nits - sequential flush_all, batch_append moves, atomic shard meta
- *(tasks)* sync stale task file headers to actual commit state
- *(server)* FIND-32 align rate-limit unit tests to burst=rpm (5)
- *(backlog)* remove stale collision records (P34 adenda/H2/AGT-05) + new plan colaterales-deuda-desktop
- *(agents)* AGT-01 checkpoint - AGENTS.md metadata + DESKTOP-24/28 marked completed
- *(backlog)* P38 batch core/server/mcp/python/ts completado (14/15) - plan archivado + avance
- FIND-06 migrate Python SDK README to English + document MOD-10 MCP tools
- FIND-18 add NOTICE file for third-party attribution (Apache-2.0)
- *(storage)* MOD-04 selective TTL purge via scalar index range lookup (bench -24%/-9%)
- *(server)* MOD-14 harden rate-limit e2e to require 429 on burst
- *(backlog)* P35 archive plan file + register decision memory
- *(backlog)* P35 batch REVIEW/MOD/FIND completado (10/10) - plan archivado + avance
- *(desktop)* UX-01+UX-05 - LensShell compartido + token .label-tech
- FIND-04 cross-SDK search() parity table in Py+TS READMEs
- *(backlog)* P35 registra batch REVIEW/MOD/FIND pausado + hallazgos sesion
- *(backlog)* P34 adenda - smoke E2E PASS, conflicto multi-instancia, UX-16..19
- *(deps)* REVIEW-11 add pip ecosystem for vantadb-python to dependabot
- *(desktop)* checkpoint 2 fixes P34 (FIX-D4 topbar)
- *(desktop)* checkpoint fixes P34 en progreso (pre-batch pipeline)
- *(backlog)* P34 revision diseno/UX Vanta Studio - 15 hallazgos restantes
- *(task-system)* memoria campañas DESKTOP/WDA + recitation FIND-01, archiva plan web-design-audit
- *(glosario)* normalizar formato markdown en fjall.md
- *(cli)* sync shell completions con flag --allow-insecure
- *(desktop)* gitignore vantadb-local (artefactos de ejecución del engine embebido en tests)
- *(audit)* cierre campaña WDA — registro avance, checkpoint, retrospectiva
- *(web)* WDA-08 cierre auditoría — reporte consolidado, AGENTS/AUDIT actualizados, plan archivado
- *(web)* WDA-05 F5 performance — 13 deps zombies + ui/ muerta fuera, command-palette lazy, eslint reactivo (58 errores→0)
- *(desktop)* remove dead document.title effect from FIND-23 (self-review)
- *(web-audit)* WDA-00 F0 baseline — build/lighthouse/a11y/screenshots medidos
- FIND-15 honest comparison page + practical limits table
- FIND-02 embeddings section (Ollama/OpenAI manual pattern)
- *(plans)* archive campaña backlog-triage — puntero y retrospectiva en avance/historial
- *(backlog)* sync filas campaña backlog-triage — REVIEW/MOD/MCP marcadas con commits, FIND-28 registrado
- *(task-system)* cierra headers IN PROGRESS de task files campaña backlog-triage + sync avance/backlog
- *(plans)* cierre campaña backlog-triage — 16/17 ✅ + 1 SKIP, retrospectiva y lecciones
- *(backlog)* close FIND-03/05/07/08/09/13/14/16/24 + MKT-18g — counter 69
- *(marketing)* MKT-18g technical claims corrected against code
- FIND-08/09/13/14/16 pre-launch user-facing surface
- *(sdk)* FIND-03/FIND-05 python sdk doc drift + ghost APIs
- REVIEW-14 migrada a avance/core-engine — fila Backlog removida, FIND-27 registrada
- *(backlog)* FIND-26 design system duplication governance + web dark-theme decision
- *(backlog)* REVIEW-18 completado — fila migrada a avance/web-frontend
- *(backlog)* contador GOV-C7 77 filas activas post FIND-19..25
- *(backlog)* P33 Studio desktop-native findings (FIND-19..25) + sync impeccable 4.1.1
- *(chore)* REVIEW-19 stub CHANGELOG.md en raíz apuntando a docs/CHANGELOG.md
- *(task-system)* MOD-01 cierra task file con evidencia de verificacion y learnings WAL
- *(backlog)* MOD-01 migrada a docs/avance/core-engine al completarse
- *(backlog)* MOD-17 migrada a docs/avance/bindings + learnings GIL/pyd del cierre
- *(backlog)* P33 pre-launch readiness findings (FIND-12..18)
- *(backlog)* REVIEW-09 migrada a docs/avance/core-engine al completarse
- *(task-system)* REVIEW-09 cierra task file con gate P2-01 y evidencia de verificacion
- CORE-02 close-out — graph persistence API documentada + registro avance
- *(python)* MOD-16 fixture autouse cierra DBs — suite default pytest verde
- *(backlog)* P33 SDK/UX developer-experience findings (FIND-01..11)
- MOD-12 migra a avance/operaciones y remueve fila del Backlog — HTTP_API sin advertencias de rebuild obsoletas
- migra MOD-07 a avance/bindings y remueve fila del Backlog
- *(plans)* archive completed campaigns and orphaned budget sidecars
- cierre campaña avance-canónico — ajustes de wording del owner + estados de plan
- *(skills)* versiona skills core local-only y elimina clones restantes
- *(docs)* script cobertura a rutas post-migración + plan avance-canónico
- *(avance)* refs vivas a avance + skill progreso reescrita (avance-canónico)
- *(task-system)* cierre hardening I+II — planes archivados + retrospectiva en mirror meta
- *(task-system)* pipeline run usa claim:true; sync memoria y estados plan hardening-2
- *(backlog)* sync — remueve P32 completada, agrega MEM-51 (auditoría integración H2)
- *(plans)* cierre hardening-2 — retrospectiva + fix persistencia Campaign ID en updateTaskStateCore
- *(plans)* cierre campaña task-system-hardening — recitation final + retrospectiva
- *(reviews)* trazabilidad P32 en los 14 reportes de modulos — cada hallazgo mapeado a su MOD-XX
- *(gov)* Campaign ID actualizado en plan doc-governance (sesión paralela)
- *(backlog)* P32 completo — MOD-41..62 (providers/integrations/benches/benchmarks/cross) + fila Exec Summary
- *(backlog)* remover fila stale MEM-51 (implementada P33 interceptor O2)
- *(backlog)* BND-01/02/06 marcadas resueltas
- *(reports)* registrar reviews modulos en INDEX
- *(reviews)* 14 reviews profundos por módulo (sesión GOV/paralela) + task file GOV-A4 + harness snippets
- *(backlog)* TK6 inspeccionado y limpiado (0.94GB reales, no 68GB) + BND-07 ya registrado
- *(backlog)* TK6 inspeccionado y limpiado (0.94GB reales, no 68GB) + BND-07 Discord/DNS pendiente externo
- *(backlog)* P26 cognitive layer exposure via MCP (scenes/context-engine/threads/axioms/snapshots)
- *(backlog)* limpiar filas completadas P33 (MEM-50/51/52/55/56/57/58, BND-03) + archivar plan Última Milla
- *(progreso)* cierre campaña P33 Última Milla 10/10 — milestone índice + archivo campana + plan archivado
- *(mcp)* MCP-29 close-out — namespaces queryable via IQL (sanitized PhysicalScan match)
- *(backlog)* register campaign debts MCP-28 (bulk path addressing) + MCP-29 (namespaces as IQL tables)
- *(plan)* restaurar contrato Task 1 (corrupción server MCP)
- *(backlog)* análisis integral desktop — +DESKTOP-28..39 (deudas, gaps UX, lente MEMORIA, dashboard PROXY) + CORE-02 (bug IQL WASM)
- *(plan)* P31 Task 4 ✅
- *(reports)* INDEX — marcar backlog sync completo (REVIEW-06..20)
- *(backlog)* derivar REVIEW-13..20 (hallazgos low accionables de review-full 2026-08-22)
- *(gov)* bitacora agosto narrativa completa, tickets GOV-TK1..9 al backlog, last-audit-state linea base GOV, DESIGN_RULES archivado + TDAM-VANTADB fuera (cierre decision owner)
- *(backlog)* backlog sync review-full 2026-08-22 — derivar REVIEW-06..12 (hallazgos >= medium) + registrar reporte en INDEX
- *(plan)* restaurar contrato Task 1 (corrupción server MCP)
- *(backlog)* P25 MCP/HTTP exposure gap analysis + task files MCP-16..27
- *(gov)* cierre campana GOV 29/30 + reporte F2 zonas intocadas + backlog colapsado a registro; checkpoint COMPLETA
- *(gov)* auditoria raiz publica — 2🔴 (invite Discord invalido, vantadb.dev sin DNS) + 8 fixes inline (GOV-F1)
- *(gov)* propuesta limpieza artefactos con Regla 0 por item — corrige premisas (book/book nunca commiteado) (GOV-E1)
- *(plan)* restaurar contrato Task 1 (corrupción server MCP)
- *(gov)* migracion fisica Investigaciones/ -> research/ (49 items git mv) + sweep citas 64 archivos + convencion documentada + INV-019 colision resuelta (GOV-D4)
- *(gov)* split progreso README 372KB -> campanas/ (37 archivos) + indice <=50KB, dedup evento x3, 0 lineas perdidas (GOV-D2)
- *(plan)* P33 Última Milla — 10 tareas (H1-H6 críticos+medios, O2 interceptor agéntico D46-D48, tiktoken, Langfuse, claude-code)
- *(gov)* bitacora entrada agosto + regla GOV-D3 (amendment — bloque faltaba por ancla incorrecta)
- *(gov)* bitacora revivida — regla de uso + draft entrada 2026-08-22 para articulacion del autor (GOV-D3)
- *(gov)* ADR-026 reubicado en adr/ junto a sus pares (GOV-D5)
- *(gov)* mirror avance catch-up — dominios vanta-memory/vanta-proxy/context-engine nuevos + contrato meta.md (GOV-D1); CRASH_MODEL sync PERF-08 (GOV-D6)
- *(reviews)* auditoría final de integración del producto — H1-H9 + re-evaluación descartes → Backlog MEM-50..58/BND-05
- *(gov)* GOV-B1 residuales — refs case_studies archivado en README/master-index/graphrag/skill vantadb + stubs book
- *(adr)* ADR-029 ACEPTADO — articulación humana Regla 5 completa (D21-D37) + enmienda D21 tiktoken feature-gate
- *(gov)* checkpoint wave C completa 7/7 — 20/30 tareas GOV cerradas
- *(gov)* Backlog sync P29/P30/P31 + nota refs archivadas + contador real ~45 + regla; ROADMAP banner alineado; ops master-index +7 (GOV-C2/C3/C7/C5)
- *(ops)* CONFIGURATION sync sweep 44 env vars (+5), rate_limit_rpm 600, flush_threshold None (GOV-C6)
- *(gov)* master-index regenerado — 136 links 0 rotos, 15+ carpetas indexadas, api nuevos + stub MCP (GOV-C4)
- *(gov)* checkpoint wave B completa 6/6 (GOV run)
- *(api)* HTTP_API.md completo 35 paths con curl real verificado, regla yaml-spec/md-guia, experimentos agrupados (GOV-B5)
- *(mcp)* skill fuente unica con 33 tools documentadas (15 core+6 skill+8 code+4 wiki), hash-SAME, MCP.md stub (GOV-B6)
- *(api)* openapi.yaml completo 35 paths/40 ops con paridad exacta vs router + gate CI check_openapi_parity (GOV-B4)
- *(user)* fix snippets rotos — 31 FAIL->0 FAIL en harness, graph_bfs x2, ef_search fuera de glosario, FAQ fsync real, URL canonica ness-e/Vantadb (GOV-B3)
- *(ops)* DR runbook sin comandos fantasma — restore --dry-run/doctor --fix eliminados, verificacion diaria validada end-to-end (GOV-B2)
- [**breaking**] retire unverified case studies to internal archive (GOV-B1, D6) — fictitious clients EdgeSense/CodexAgent presented as real deployments
- *(gov)* cifra canonica de tests 2034 passed (nextest default, 2026-08-22) en TEST_MAP (GOV-A2)
- *(gov)* cifra coverage canonica ADR-018 (root >=80%, baseline 81.40%) en TEST_MAP/CI_POLICY/progreso — re-medicion pendiente llvm-cov ICE (GOV-A1 stop condition)
- *(docs)* harness validate_doc_snippets.py — 21 PASS/31 FAIL/6 SKIP inicial, detecta graph_bfs roto x2 (GOV-A4)
- *(backlog)* limpiar filas stale MEM-36/43/44/45 (pagadas en P31/P32)
- *(progreso)* cierre campaña P32 Bindings SDK 4/4 — MEM-36 pagada, archivo de plan
- *(bindings)* SDKB-04 Domain Sub-clients en READMEs TS/Python + gate backward-compat final (246+105, 0 gaps)
- *(plan)* restaurar contrato Task 3 (corrupción server MCP)
- *(task-system)* TIR-02a dora recovery time + TIR-04b contenedor tasks/closed + TIR-08c criterios research (GOV-T01..03)
- *(gov)* auditoria documental V1+II (salud 6.5/10) + plan campana GOV 30 tareas + fase GOV en backlog
- *(api)* SDKB-01 mapa canon namespace↔método por SDK (43 wasm/38 TS/45 python, diferencias documentadas)
- *(progreso)* cierre campaña P31 Cierre Final 8/8 — port TDAM al 100%, archivo de plan
- *(plan)* P31 Task 7 ✅ (marcado manual — MCP lo escribió en plan equivocado) + reubicar plan stray a archive
- *(adr)* MEM-49 guía socrática revisión ADR-029 + D21-D37 (prep articulación humana, Regla 5)
- *(plan)* restaurar contrato Task 1 (corrupción server MCP)
- *(vantadb-mcp)* MEM-44 e2e ingest→wiki_* roundtrip cross-crate (dev-dep vanta-memory, sin ciclo)
- *(plan)* P31 cierre final — 8 tareas (wiring, roundtrip, auto-sync, embeddings semánticos, scoring real, ADR humano, meta-plan bindings)
- *(api)* VANTA_MEMORY.md canónico (cierra cita colgante ADR-029) + hallazgos auditoría final MEM-43..45
- *(progreso)* cierre campaña P30 Proxy+Knowledge 9/9 — roadmap TDAM F1-F7 completo, archivo de plan
- lockfile tras MEM-26
- *(plan)* restaurar contrato Task 2 (corrupción server MCP)
- *(plan)* restaurar contrato Task 1 (corrupción server MCP)
- *(plan)* P30 decisiones D31-D37 cerradas por el usuario (TOML, canal interno+polling, mem-command TDAM, auth obligatoria, 60 req/min, paths locales, riesgos aceptados) — uphill 0
- *(plan)* P30 Vanta Proxy + Knowledge (F6+F7) — 9 tareas DO, decisiones D24-D30 cerradas upfront
- *(progreso)* cierre campaña P29 Context Engine 9/9 — migración a progreso, archivo de plan
- *(vanta-memory)* MEM-38 ADR-029 borrador + superficies F5 en EMBEDDED_SDK (gate cierre F5)
- *(plan)* P29 Vanta Context Engine (F5) — 9 tareas DO, triage con Paso 0 + pre-mortem + risk register
- *(tasks)* MEM-02.md retroactivo + decisiones auditoría MEM-39..42 en backlog
- *(vanta-memory)* e2e flow L0→L1→L2→L3→recall + huecos triage P27 al backlog (MEM-39..42)
- *(avance)* mirror MEM-21 en bindings con commit hash
- *(progreso)* cierre campaña P27 Vanta Memory Engine 24/24 — migración a progreso, archivo de plan, docs supersede
- lockfile + completions tras scaffold vanta-memory (MEM-08a)
- *(plan)* materializar F4 (Tasks 10-24, MEM-08a..21) + fix header (Campaign ID 97e683bd, Estado EN PROGRESO)
- *(plan)* MEM-35 ✅ COMPLETED (data plane REST, 11/11 e2e) — F1+F2+F3 completas (9/9)
- *(backlog)* sync estado MEM-01..07/34/35 con plan (8 completadas) + link catálogo en plan
- *(tasks)* mark MEM-35 completed (9693d0ff)
- *(plan)* MEM-07 ✅ COMPLETED (MCP skill_* tools, verify 13/13 + 44/44)
- *(task)* MEM-07 cerrado (commit 4763bf44)
- *(plan)* MEM-06 ✅ COMPLETED (skills multi-versión, verify 14/14) + restaurar contrato MEM-01
- *(task)* MEM-06 cerrado como completed
- *(plan)* checkpoint 1 F1+F2 aprobado — materializar F3 (MEM-06/07/35)
- *(plan)* checkpoint 1 F1+F2 aprobado por review humano — F3 pendiente (MEM-06→07→35)
- *(tasks)* cierre MEM-05 task file
- *(plan)* MEM-05 ✅ COMPLETED (auth 3 capas, verify 16/16)
- *(tasks)* cierre MEM-34 task file + completions CLI regenerados
- *(plan)* MEM-04 ✅ COMPLETED (permission-checker allow-only, verify 37/37)
- *(plan)* MEM-03 ✅ COMPLETED (entidades entity_* + CRUD, verify 14/14)
- *(plan)* materializar F2 (MEM-03/04/05) en formato ejecutable
- *(plan)* F1 ✅ (MEM-01/02/34) + completions CLI regenerados (subcomando tui)
- *(tasks)* cerrar 16 task files stale WIP de campañas previas (one-task-at-a-time gate)
- *(plan)* MEM-01 ✅ COMPLETED (fuse_rrf parametrizado, cláusula IQL PROFILE, report rrf_k real)
- *(plan)* MEM-01 completado (commit 6a50b8ee)
- *(plan)* P27 campaign ID + formato ejecutable F1 (MEM-01/02/34)
- plan auditoría diseño web v2 corregida (estado real del código)
- *(plan)* P27 D18-D20 cierre de hallazgos 4/6/7 (MEM-35 REST agent-facing, tests por tarea, Studio rrf_k dinámico)
- *(plan)* P27 D13-D17 + hallazgos blast radius (IQL profile, SearchProfileConfig, audit server, MEM-34 a F1)
- *(skills-manifest)* add Accessibility/Inclusive Design section (63 skills), mark remotion-best-practices canonical
- *(backlog)* archive DESKTOP-12..22 as obsolete by P26 direction, re-scope 23/26/27, prioritize 24/25
- plan Fase 4 — DEFER table: decay ya implementado como supersession durable (ADR-028)
- plan Fase 4 — W3 (FEAT-01..03) completa, 16/18; task FEAT-03b-impl verify real del lead
- FEAT-03b — ADR-028 (accepted) + contrato core decay supersession
- plan Fase 4 — W2 (WASM-01..04) completa, 12/18
- WASM-04 task file — verify real del lead (53/53 tests, E2E PASS, commit 380bb6eb)
- cargo fmt en desktop connections (colateral WASM-01)
- marcar Wave 1 REST completa en plan Fase 4
- DOC-04 — cabeceras/estados stale: progreso 2026-08-19 Fase 4, WEB-06 completo (583dad9a), conteo commits F3, header desktop.md F1/F2 separados, comentario muerto eliminado
- DOC-03 — registrar Fase 0 en progreso/README, archivar plan F0, corregir hash cierre F1 (9c27f5e9 -> 4c26b285)
- cerrar task files Vanta Studio F0-F3 (DOC-02) — estados COMPLETO con commits reales; WEB-001 renombrado a WASM-PLAYGROUND-001 (legacy)
- reconciliar P26 Vanta Studio (DOC-01) — Fases 0-3 completadas, commits reales, Fase 4 en ejecución
- plan Fase 4 Vanta Studio — cierre deuda REST + WASM/OPFS + diferenciadores (18 tareas)
- cerrar gaps de cobertura (get_version/versions, dashboard_dir, version_history_limit) + fix crash MCP en validate-docs-coverage
- cierre Fase 3 Vanta Studio — REST completo + dashboard embebido (WEB-00..06, ADR-026)
- regenerar completions + Cargo.lock para flag --dashboard-dir (WEB-03)
- *(desktop)* WEB-00 transporte pluggable vanta.ts (TauriBackend/HttpBackend stub)
- plan Fase 3 (7 tareas web/embebido) + archivar plan Fase 2 (deletion tras mover a archive)
- cierre plan Fase 2 (10/10) — archivo + retrospectiva + migración a progreso/avance
- *(web)* remove logo-reveal experiment artifacts and legacy gato assets
- re-exports filtros root + clippy tests (VS-CORE-04) + learnings VS-CORE-05
- plan Vanta Studio Fase 2 — grafo R3F, espacio, IQL, import y batch ops
- backlog — P28 CORE-01 persistencia on-disk de vectores Binary (follow-up 8c8eef23)
- cierre P26 Fase 1 — progreso 9/9 + archive plan + mirrors avance
- plan P26 Fase 1 COMPLETA — marcar 9/9 tareas HECHO
- task files VS-13/15/16 (recitation Wave 1)
- plan P26 Fase 1 — explicabilidad y tiempo (9 tareas, contratos verificados vs core/bridge)
- recitation plan P26 Fase 0
- plan P26 Fase 0 COMPLETA — marcar VS-04/05/06/07/08/10/11/VS-CORE-01 HECHO
- VS-07/VS-08/VS-09 HECHO en plan + task files
- task file VS-06
- task file VS-05
- task files VS-04 + VS-CORE-01
- task file VS-11
- VS-03/VS-10 HECHO en plan + task files
- fix plan file corrupto (Campaign ID, estado, contrato VS-00, VS-01/VS-CORE-02 HECHO)
- VS-02 migrada a progreso + avance desktop + task file
- sync vanta-lead permission + wave-r2-r7-fnd campaign IDs/recitation
- *(backlog)* P27 Vanta Memory Engine (38 tasks) + vanta-memory plan F1-F7
- *(research)* TDAM investigation — vanta memory engine source (11 reports)
- AUD-044..051 migradas a progreso — campaña fix audit 0.5.0 completa (plan archivado)
- fmt pre-existing mcp_tests.rs diff
- AUD-047 build released vanta-cli with server feature
- P22-MCP cierre — MCP-15 y T15 migradas a progreso (20/20 batería, 34/34 tests)
- T15 search_memory explain shape — document real contract + regression test
- sync strategy 2026-08-17 — ROADMAP 0.5.0/24 items, claims Show HN corregidos, GTM/blog/pro markados
- P23 VantaDB Pro + P24 I+D + P6 ampliada — verificación strategy 2026-08-17
- P18 header — investigación TIR completada + follow-ups de decisiones
- P18 TIR-01..08 investigadas — decisiones task-system (7 docs)
- AUD-024 completed + wave R2-R7-FND recitation sync (MCP-01 done)
- register VantaDB MCP server (vanta-cli --mcp) in opencode.jsonc
- P22 progreso — migración MCP-01..14 a registry, hallazgos MCP-15/T15 activos
- skill sync — MCP-05..14 content + backlog P22 findings (MCP-15, T15)
- MCP-05..14 skill vantadb-mcp sync — IQL syntax, envelope, behavior notes, dead refs, contradictions
- *(followups)* wave 15/15 completa - archiva plan + sync metadata stale (FND-01/11/12/14/16)
- *(FND-04-F1)* ADR-025 zero-copy Arrow diferido con señal de reapertura medible (cierra pendiente ADR-021)
- *(P2R-01)* gates P2-01 retroactivos TSYS-06/FND-07 + veredictos post-fixes wave (10/11 approve, 1 changes-required FND-13-F1)
- *(FND-13-F2)* Regla 11 en BENCHMARKS/PERFORMANCE_TUNING - provenance commands + marcar claims sin fuente
- sync vantadb skills with real API and MCP contract (SKL wave)
- *(FND-02-M2)* stress eviccion *_locked bajo contencion (256 candidatos, 7 threads, watchdog 60s)
- *(followups)* plan wave follow-ups (15 items, 5 waves) + 11 task files
- *(progreso)* migra 25 tareas wave p20-tsys (R1-10, TSYS-06, FND-01..24) a progreso + plan archivado
- *(FND-16-followup)* gate wasm/TS por PR con paths filter + fix path CONTRIBUTING + dictamen P2-01 en FND-02
- *(FND-24)* ICP + JTBD con evidencia honesta (0 usuarios reales - hipotesis + plan validacion)
- *(FND-23)* ADR-024 motor de grafos default-on hasta telemetria (senal vanta_graph_ops_total)
- *(FND-03)* feature set minimo compila + wheels empaquetan set minimo (estado OK)
- *(FND-22)* CONTRIBUTING con commit convention, PR flow, gates y triage
- *(P20a)* FND-06 - regla R-8 core-bindings + TODO(core) + drift ERR-028 documentado
- *(P20a)* FND-08 - ADR-023 backend compaction (diferir fjall/rocksdb marginal) + regla durability
- *(FND-16)* analisis multi-target CI - plan job wasm/TS por PR con paths filter
- *(P20b)* FND-13 - Regla 11 benchmarks honestos + claims README alineados
- *(FND-04)* zero-copy Arrow bindings - diferir con ADR + senal de reapertura
- *(P20b)* FND-10 - Regla 9 no-optimizar-sin-medir + benchmark canonico P99 (baseline 3.07ms p99)
- *(P20b)* FND-11+12+14 - AI Guardian (Regla 10), ADR forcing function, ritual feature stack
- *(FND-05)* SDK idiomatico - gaps + prototipos (no rewrite)
- *(P19)* R6 - routing table vanta-research + operating manual
- *(TSYS-06)* decision chaos runner task-system - tests puntuales, runner deferido
- *(P19)* R3 - delegar DISCOVERY pesado a vanta-research (pipeline.md + task.md)
- *(P19)* R1+R5+R8+R9+R10 - skills obligatorias, sync, dangling ref, permissions, seccion 7 consolidada
- ignore agent memory DB + enforcement runtime log
- migrar 10 tareas completadas (wave R2-R7-FND) a progreso/avance
- *(FND-21)* ADRs retroactivos - backend default, zero-copy Arrow, WAL batch/async
- *(FND-20)* nota trade-off HNSW (M=32 ef=100) vs IVF/FAISS
- *(FND-19)* inventario Arc<Mutex> core - 2 instancias, 1 accion recomendada (ingestion canal)
- *(FND-17)* analisis API reference docs-as-code (plan fase 1 cargo doc, defer typedoc)
- *(FND-18)* quickstart SDKs <5 min - fix metadata shape, PyPI primario, metricas
- *(FND-15)* verificación crash recovery/WAL (sin gap de producto, gap de infra de tests)
- *(FND-09)* Regla 8 - concurrencia paranoica en PRs
- *(R7)* corregir comandos de verificación rotos en Output Templates
- *(R2)* crear agente vanta-research (read-only research subagent)
- *(wasm)* dedup collect_all_deduped by u128 node_id (AUD-043)
- migrate AUD-034/037 to progreso, mirror bindings
- migrate AUD-038/040/041 to progreso, mirror core-engine/seguridad
- *(core)* add ListFloat arms to sparse_hot_path (AUD-041)
- *(wal)* reuse buffer in batch_append, drop per-record alloc (AUD-040)
- *(core)* remove obsolete unused_unsafe allow (AUD-038)
- correct AUD-035 progreso/avance registry with full 3-split scope
- *(core)* split storage/engine ops + index/search megafiles (AUD-035)
- *(wasm)* dedupe IDB transaction into single helper (AUD-034)
- AUD-035 — split search/mod.rs into lexical/vector/sparse/hybrid/explain submodules
- move reference sections out of AGENTS.md
- disable Rust MCPs by default and prune AGENTS.md
- migrate AUD-032/AUD-033 to progreso, mirror bindings/operaciones
- *(ci)* append CI-05 passed entry to verify-log (AUD-029)
- *(mcp)* split monolith into 12 modules (AUD-032)
- AUD-029 mark task file completed
- AUD-029 re-run CI-05 contract from root (verify-log aligned)
- migrar CI-01 (pre-commit config) de backlog a progreso (CI-01)
- migrate AUD-025/026/027 to progreso registry (Trigger 1)
- least-privilege per-job permissions in release workflow (AUD-027)
- drop cli/arrow/tantivy from native DLL default features (AUD-026)
- eliminate per-posting allocations in BM25 hot path (AUD-025)
- migrate AUD-028 to progreso (AUD-028)
- annotate pinned actions with version tags (AUD-028)
- *(progreso)* migrar AUD-022 y AUD-030 (CI supply-chain + bench gate)
- run bench regression gate on PRs + auto-commit baseline (AUD-030)
- pin sccache-action to verified SHA (AUD-022)
- migrate AUD-039 to progreso and record learnings
- swap LRU eviction to O(1) lru crate in python bindings (AUD-039)
- *(progreso)* migrar AUD-024 (perf drain_hnsw_batch_locked)
- avoid per-op heap clones in drain_hnsw_batch_locked (AUD-024)
- *(progreso)* migrar AUD-031 y AUD-036 (panic-hardening + finding stale)
- *(security)* document RUSTSEC-2026-0253 upstream blockage (AUD-042)
- *(backlog)* derivar AUD-032..043 del audit full 2026-08-12
- *(avance)* mirror P2-7 en core-engine + verify-log y sesiones task-system
- *(audit)* audit full 2026-08-12 — PASS, hallazgos AUD-022..031
- *(progreso)* migrar TSYS-01..16 implementados de Backlog a progreso
- TIR-03 fase de contención en bug-workflow
- *(progreso)* plan CI batch (CI-02..07) archivado — 6/6 migradas a progreso + retrospectiva
- *(task)* CI-07 — task file y sync plan (SHA pinning completado)
- *(pinning)* CI-07 — pin actions SHA (security + fuzz + chaos workflows)
- *(pinning)* CI-07 — pin actions SHA (release workflows) + fix ref muerta release-plz
- *(pinning)* CI-07 — pin actions SHA (heavy certification + perf workflows)
- *(pinning)* CI-07 — pin actions SHA (core fast-gate workflows)
- P2-7 — persistir sparse vector como ListFloat pairs (ADR-019)
- *(task)* CI-05 — task file sync estado final (completed + review verdict)
- *(plans)* CI-05 completado — sync plan (Task 4 benchmark baseline)
- *(bench)* CI-05 — baseline fijo y umbral de regresión en perf-bench
- *(plans)* CI-06 completado — sync plan (tests gate en release binaries + npm)
- *(release)* CI-06 — tests gate en release binaries + npm
- *(ci)* CI-03 — SBOM multi-ecosistema (rust + npm + python)
- *(progreso)* CI-04 migrado de Backlog a progreso (CodeQL multi-lenguaje)
- *(plans)* CI-04 completado — CodeQL multi-lenguaje (sync task file + plan)
- *(codeql)* CI-04 — CodeQL multi-lenguaje (rust+python+js/ts)
- *(fuzz)* CI-02 — fuzz gate en PRs acotado (paths-filter + timeout 15m)
- *(plans)* plan deuda CI batch (CI-02..07) verificado contra código real
- *(backlog)* indexar deuda P2-7 serialización zero-copy del sparse (core persistido)
- eliminar rutas viejas de docs movidos a subdirs (b173a626)
- *(REVIEW-04/05)* migrar god-files splits a progreso + marcar REVIEW-04 COMPLETED
- REVIEW-05 — split serialize/distance/physical_plan into submodules
- split god modules node.rs y vfile.rs en submódulos (REVIEW-04)
- *(backlog)* indexar 8 gaps residuales del task-system (P18) para investigación y decisión
- *(harness)* registrar verificaciones ERR-036/043 en verify-log
- *(ERR-026)* actualizar task file con recitacion y evidencias de delegation fix (aa1754d2)
- *(plans)* actualizar recitaciones archivadas + retrospectiva perf-bench-wasm
- *(deps)* mecanizar ignore RUSTSEC-2026-0253 + skip multiple-version documentado (AUD-016/ERR-007)
- *(progreso)* sincronizar Backlog COV/AUD + mirrors avance (COV-001..004, AUD-016..021, ERR-006/007/015)
- ignorar salida de c8 coverage en vantadb-ts
- *(AUD-018)* extender gate clippy -D warnings a wasm/server/mcp (verificado 2026-08-12)
- *(wasm)* differential persist cache for save/save_idb (delta writes, skip unchanged)
- *(progreso)* migrate COV-001..004 to progreso, remove from backlog, archive plan
- add CLI subcommand integration tests (migrate/server/crud)
- ADR-018 coverage gate target = root crate (supersede ADR-015)
- add c8 coverage script for vantadb-ts src/
- add Python AsyncVantaDB async smoke tests (flush/purge/query/graph/export)
- *(progreso)* PERF-03 Milvus medido — cerrar nota pendiente
- *(perf)* close PERF-03 — Milvus-frugal measured + honest competitive table (PERF-03)
- *(progreso)* migrar PERF-02/03/05/08 a progreso + mirror avance + archivar plan
- *(campaign)* PERF plan file (PERF-02..08)
- *(wasm)* zero-copy Float32Array vector serialization (PERF-08)
- *(adr)* WAL async roadmap io_uring/aio + fsync group commit (PERF-05)
- *(perf)* honest competitive SDK benchmark vs Qdrant/Chroma/Lance (PERF-03)
- *(benches)* deterministic criterion profiles + critcmp regression gate (PERF-02)
- record ERR-031 learning (verify git history + per-backend rejection tests)
- *(ERR-031)* migrate completed task from backlog to progreso
- *(ERR-031)* verify VecIndex::add rejections surface as Err
- *(ERR-015)* registrar completación (stdin EOF graceful shutdown)
- *(ERR-044)* migrar tarea completada de Backlog a progreso
- *(ERR-044)* add batch analyzer reuse benchmark + register bench target
- *(ERR-045)* marcar task file completado
- *(ERR-045)* registrar completación en progreso
- *(ERR-045)* avoid neighbor list clone in compaction BFS
- *(ERR-043)* register learning - verify git history before implementing fix
- *(ERR-043)* move completion from backlog to progress registry
- *(ERR-043)* verificar fix shrink_neighbors ya commiteado + task file
- *(ERR-042)* move completion from backlog to progress registry
- *(ERR-042)* parity search vfile vs in-memory + tombstone header excluded
- *(ERR-010)* cerrar reapertura 2026-08-11 con evidencia de fix
- *(ERR-010)* alinear 13 tests de certificacion con el core actual
- *(ERR-037)* perf evidence, benchmark results and backlog retirement
- *(ERR-037)* batch_existing_check bench with hot-tier cache-hit scenarios
- *(ERR-037)* batch_insert probe returns bookkeeping-only meta with chunked locks
- *(progreso)* migrar ERR-036 de Backlog a progreso
- *(task)* ERR-036 cierre — evidencia verify full y medicion perf
- fix clippy -D warnings en tests de index (colateral ERR-031)
- *(reviews)* validar investigaciones agent-engineering + sincronizar estado Backlog §P17
- *(plans)* archivar los 3 planes activos completados a docs/plans/archive/
- *(progreso)* registrar cierre del plan residuo-consolidado (Trigger 1)
- *(plans)* cierre residuo-consolidado — 24/24 DO ejecutados (T6 cosmético), estados por commit
- *(task)* task files COV-002/COV-003/AUD-020 y budget del plan residuo
- *(plans)* cerrar estado residuo-consolidado, corregir headers y reabrir ERR-010
- *(harness)* poblar verify-log y regenerar reports con datos reales
- *(server)* align HTTP tests with ERR-027 4xx semantics and cover RBAC
- cargo fmt drift in src/sdk/api.rs (pre-existing from c9188639)
- *(cli)* fix seeds flush + crc32c empty guard, COV-003 67/68 (ERR-050b)
- WIP residual-hardening + task files + plan residuo consolidado
- *(plans)* marcar COMPLETED tasks 3-16 del plan de consolidación
- *(audit)* lectores apuntan a docs/reviews tras unificación T7
- *(python)* use non-zero vectors in search tests (COV-001)
- *(docs)* eliminar duplicado de investigación en .opencode (T3)
- *(audit)* unified-review escribe en docs/reviews — escritores (T7)
- *(investigacion)* restaurar ACID_TRANSACTIONS.md desde git (T5)
- *(backlog)* incorporar items §3.3 del REPORTE-FINAL (T14)
- *(reports)* corregir rutas rotas en INDEX.md (T6)
- *(workflow)* redirect research.json output a docs/Investigaciones (T4)
- *(docs)* unificar investigación en docs/Investigaciones (T3)
- *(avance)* eliminar backup huérfano y espejos duplicados → links (T15)
- *(adr)* resolver archived-decisions → adr/ y reviews/ (T8)
- *(agents)* crear persona vanta-review (revisor de segunda opinión) (T10)
- *(PERF-09)* honest cold-start log and legacy force_copy note
- *(ERR-048)* single hash lookup for visited set in search_layer
- *(ERR-047)* borrow inline neighbor cache instead of copying into pool
- *(PERF-07)* explicit sparse parse with corruption log instead of silent .ok()
- *(ERR-045)* get_neighbors_ref avoids clone in read-only BFS
- *(ERR-043)* borrow neighbor vector in shrink instead of cloning
- *(plans)* mark Task 9 completed (orphans removed)
- *(task-system)* annotate pre-call-checks as legacy spec
- *(task-system)* remove orphaned pipeline scripts
- *(refs)* corregir referencias rotas en AGENTS.md, cliff.toml, skills, master-index
- *(manifest)* sincronizar conteo de skills (111 = 29 + 82) y marcar deprecadas
- *(refactor)* mover backlog-futuro R5 de docs/archive → docs/ (separar operativo de archivo muerto)
- *(plan)* sincronizar estados p3-remaining-fallas (6 tasks → COMPLETED)
- *(plan)* consolidación de carpetas docs + task-system (16 tasks)
- *(reports)* regenerar northstar + pipeline-evals (evals P1-06/EVAL-01)
- CI-01 pre-commit hooks config (rustfmt, ruff, prettier scoped)
- *(adr)* COV-004 ADR-015 coverage policy decision + CI_POLICY sync
- *(ts)* COV-002 fix vitest wasm external + v8 coverage config
- *(python)* COV-001 bulk import async wrappers (.vdbdump bytes + file)
- *(harness)* fix stale skill counts + document Windows glob workaround
- *(ERR-042)* cache read_header per candidate in search loop
- *(ERR-036)* atomic hit counter removes write-lock in get()
- *(manual)* drop vantadb-certify + vantadb-audit from skills tree
- *(skills)* remove deprecated vantadb-certify + vantadb-audit (unified-review los reemplaza)
- *(skills)* remove deprecated vantadb-full-review (unified-review lo reemplaza)
- *(campaign-executor)* remove PowerShell harness (loop unificado)
- *(ERR-037)* batch_insert single exists-check
- *(campaign-executor)* unify task execution depth + SARL sub-agent recovery
- *(ERR-044)* reuse TextAnalyzer across batch
- *(audit)* archive vantadb-audit-report as audit-full-2025-07-27 + update fuentes-vivas/INDEX references
- *(api)* document search_with_method in embedded SDK (FEAT-04 gap, validate-docs-coverage 0 gaps)
- *(backlog)* close P15/P16 pipeline - archive plan, remove completed rows, update counters (progreso)
- *(plans)* mark Task 49 COMPLETED via MCP state sync + final recitation
- *(plans)* close backlog pipeline - all 49 tasks complete (RELEASE-02 verified live)
- mark DOC-06 done in plan file (Task 46)
- document node ID limits in python SDK and quickstart (DOC-06)
- *(plans)* mark DOC-08 done with real commit hash (DOC-08)
- honest single-node and wal-shipping claims in README (DOC-08)
- *(plans)* mark DOC-07 task 47 done (DOC-07)
- *(config)* add configuration examples section (DOC-07)
- *(plans)* mark DOC-04 done with real commit hash (DOC-04)
- fix invented API in llms.txt, use real vantadb_py.VantaDB (DOC-04)
- fix invented API in llms.txt, use real vantadb_py.VantaDB (DOC-04)
- fix invented API in llms.txt, use real vantadb_py.VantaDB (DOC-04)
- *(plans)* record DOC-03 commit hash in Ejecutado
- fix UTF-8 mojibake in python README and design rules (DOC-03)
- mark DOC-05 done in plan file
- replace Obsidian wikilinks with relative md links (DOC-05)
- *(cli)* regenerate completions with --memory-limit (PERF-06)
- *(plans)* mark PERF-06 task 41 DONE (PERF-06)
- *(plan)* mark Task 40 PERF-04 DONE with real commit hash (PERF-04)
- *(index)* gate HNSW prefetch behind config flag, default off (PERF-04)
- mark DOC-02 done in plan (ref 6723cb3f)
- fix version drift 0.4 → 0.5.0 in QUICKSTART/README (DOC-02)
- *(plans)* mark COV-003 task 37 DONE
- *(cli)* cover vanta-cli subcommands (COV-003)
- *(plan)* mark Task 39 PERF-01 DONE with real commit hash (PERF-01)
- *(benchmarks)* revalidate README perf claims against real benches (PERF-01)
- *(plan)* mark Task 38 COV-004 DONE (COV-004)
- *(adr)* coverage policy gate for root vs workspace (COV-004)
- *(plan)* mark Task 35 REVISAR-01 DONE (REVISAR-01)
- *(ivf)* dedicated IVF build/query bench (REVISAR-01)
- *(plans)* mark Task 36 COV-001 as done (COV-001)
- *(python)* cover async flush/purge/query/graph (COV-001)
- *(plan)* Task 34 FEAT-07 DONE (1b2a39d3)
- *(plan)* mark Task 31 FEAT-04 DONE
- *(plan)* mark Task 33 (FEAT-06) DONE (FEAT-06)
- *(config)* document real config format (builder/env, no config.toml) (FEAT-06)
- *(plan)* mark Task 32 (FEAT-05) DONE with feature status doc
- *(features)* document per-feature status and EXPERIMENTAL flags (FEAT-05)
- *(plan)* mark FEAT-03 arrow export done
- *(plans)* mark FEAT-01 task 28 DONE
- *(plans)* mark Task 29 FEAT-02 DONE
- *(adr)* PITR/WAL-shipping decision and feature docs (FEAT-01)
- *(index)* clarify DiskANN is in-memory, not disk-backed (FEAT-02)
- *(plans)* mark ERR-050 task 27 DONE
- *(changelog)* generate [Unreleased] via git-cliff (ERR-050)
- *(plans)* mark ERR-027/028 tasks 24-25 DONE
- *(plans)* mark ERR-029 task 26 DONE (ERR-029)
- *(storage)* restore oversized-write guard test (ERR-005)
- *(deps)* handle RUSTSEC-2026-0002 (lru) (ERR-004)
- mark SEC-01 completed in backlog plan
- *(ci)* add cargo-semver-checks gate (RELEASE-01)
- *(pipeline)* move legacy plans to archive
- *(pipeline)* archive completed audit reports, create backlog-2026-08-09 plan
- *(docs)* archive 5 completed plans + fix admin-console recitation, update progreso
- *(backlog)* mark ERR-016 resolved, task record + parser doc-comment
- *(audit)* record full audit 2026-08-08 (0 critical) + ERR-001..020 review ledger
- *(tokenizer)* advanced-tokenizer default-enabled, auto-rebuild on v3->v4 schema change
- *(backlog)* migrate AUD-012..015 to progreso, record 225-row 2026-08-07 cleanup
- *(plans)* close admin-console + desktop-mvp campaign records (task files, session, budgets, commit refs)
- *(avance)* add live progress mirror with domain files + coverage check script
- *(workflow)* progreso deletes completed rows; add docs/avance mirror + plan reality gate in prompts
- *(ts)* add @vitest/coverage-v8 devDependency
- *(backlog)* admin-console plan 10/10 complete (ADMIN-01..09 + DESKTOP-20) -> progreso
- *(backlog)* mark 9 AUDREP tasks complete, migrate to progreso (2026-08-07)
- *(web)* rename package to vantadb-web (AUDREP-59)
- *(frontend)* extract shared tokenizer (AUDREP-58)
- *(frontend)* remove dead hero variant toggle + unreachable cat branch (AUDREP-57)
- *(web)* enable noImplicitAny strictness (AUDREP-46)
- *(sdk)* stop materializing dead vector/sparse_vector in purge_expired (AUDREP-54)
- *(backlog)* mark 10 AUDREP tasks complete (2026-08-07) -> progreso
- *(server)* consolidate duplicated tokio dependency (AUDREP-52)
- *(release)* workspace-inherit edition/rust-version in root crate + add macOS/WASM toolchain targets (AUDREP-48, AUDREP-50)
- *(web)* remove dead next-auth dependency (AUDREP-41)
- *(mcp)* collection_delete rollback leaves no partial deletes (AUDREP-43)
- *(index)* replace MetricCache OnceLock with plain const (AUDREP-53)
- *(backlog)* mark 10 AUDREP tasks complete (2026-08-07) -> progreso
- *(release)* bump homebrew formula version 0.2.0 -> 0.5.0 (AUDREP-25)
- *(backlog)* mark 10 AUDREP tasks complete (2026-08-07) -> progreso
- *(infra)* drop obsolete compose version key (AUDREP-49)
- remove superseded root src-tauri scaffold (superseded by desktop/)
- *(backlog)* desktop-MVP waves 0-3 completas (DESKTOP-02..11) -> progreso
- *(DESKTOP-05-09)* wire native+server adapters into connections mod (parallel merge)
- *(backlog)* move AUDREP-14,16,17,18,20,21,22 to progreso
- *(mcp)* bound memory in collection_stats/collection_list streaming aggregates
- *(DESKTOP-05)* task tracking file
- *(DESKTOP-04)* registrar commit hash en task file
- *(progreso)* close P13 audit findings (AUDREP-07/09/10/11/15/19, NV-01/04, DEPS-01)
- *(audit)* dependency duplication report with root-cause table (DEPS-01)
- resolve REVIEW-01/02/03/05 findings
- *(web)* enable strict TS build checks and react strict mode (AUDREP-19)
- *(build)* wire rayon feature and fix doc drift (AUDREP-07)
- update licensing, backlog and plans; sync provider lockfiles to 0.5.0
- revamp README with animated banner, integrations and translated content
- add Open Core licensing model for VantaDB Pro
- add sparse hot-path benchmark and close AUDIT-02 as WONTFIX (measurement gate)
- *(mcp)* renombrar query_lisp a query_iql (stale tras CUARENTENA-01)
- *(crewai)* apuntar tests categorize a la funcion de modulo (stale)
- *(gate)* fix markdownlint y frontmatter para GATE Docs
- *(rust)* correr Miri sin feature roaring (fallback pure-Rust FilterBitset)
- *(rust)* habilitar -Zmiri-disable-isolation para Miri Test
- *(task-system)* wire pre-call checks and add state tools
- *(task-system)* registrar tareas campaña y validación backlog 08-05
- *(docs)* limpiar y consolidar audit-reports y reviews
- *(task-system)* sincronizar reportes con backlog y endurecer flujo de pipeline
- *(backlog)* validar reviews 07-27 y archivar duplicados
- *(review)* registrar reportes full/certify 2026-08-05 + manual de operacion
- *(backlog)* tachar AUDREP-03/08/13 completados + actualizar críticos activos
- *(progreso)* migrar 14 tareas F5/F6 completadas de Backlog a historial
- pipeline-state 53/54 completo (resta Task 50 humana)
- *(backlog-validation)* plan 53/54 completado (resta Task 50 humana Discord)
- *(DESKTOP-MVP-54)* save point for Tauri MVP task 54
- *(MKT-16)* GraphRAG benchmark reproducible + metodologia (numeros indexacion reales; query PENDING por stack overflow engine)
- *(AUDIT-03)* fmt vfile shim + tachar AUDREP-01/04, AUDIT-03 en backlog y progreso
- *(MKT-10)* reescribir campaña AI Agent Memory con checklist medible
- *(GH-141)* documentar webhook GitHub→Discord (push, PR, issues, release)
- *(NUEVO-16)* viabilidad Product Quantization (update REC-009)
- *(INV-025)* scoping Search Quality v2 + dependencia con INV-009-B
- *(GH-131)* README integraciones mem0, Semantic Kernel, DSPy
- *(GH-140)* eliminar selectores huérfanos probados de globals.css
- *(GH-132)* notebook Colab + badge Open in Colab
- *(NUEVO-01)* README hero + benchmark graphic (GH-139)
- *(INV-016-B)* motion tokens (duration/ease) reemplazan cubic-bezier
- *(INV-014-B)* eliminar plomería dark inerte (next-themes)
- *(backlog-validation)* F4 wave2 — AUD-005, AUD-006, AUD-008, GH-123
- *(GH-123)* corregir 3 links rotos reales + documentar metodo de auditoria; issue cerrado
- *(AUD-006)* documentar 5 tools MCP reales (collection_*, rehydrate) + gate de paridad
- *(AUD-008)* corregir drift de versiones en STORAGE_VERSIONING.md
- *(AUD-005)* sincronizar openapi.yaml a 0.5.0 + gate CI check-api-version
- *(AUD-002)* corregir API fantasma en GRAPH_RAG.md
- *(AUD-003)* taskfile commit hash
- *(AUD-003)* retractar verificación contra src/governance inexistente
- *(AUD-009)* corregir nota Vite->Next.js en DESKTOP-01b
- *(AUD-007)* corregir drift de nombres y constantes en ARCHITECTURE.md
- *(backlog-validation)* F3 wave2 — TECH-08 decision, AUDIT-05, AUDIT-08
- *(TECH-03)* corregir 3 stale-docs (MCP excluyente, API python real, query_iql)
- *(TECH-06)* cerrar CORS — sin consumidor browser real
- *(TECH-07)* documentar API worker opfs + demo browser
- *(DEBT-01)* reparar gate docs-coverage y documentar 13 gaps
- *(backlog-validation)* F2 wave2 — AUDIT-04 root-cause cache_warmer
- add backlog Phase 13 audit findings and competitor research
- *(backlog-validation)* F2 wave1 — AUD-004, AUD-011
- *(AUD-004)* renombrar tool MCP query_lisp a query_iql
- *(backlog-validation)* F2 wave0 — AUDIT-01, AUD-001, TECH-01, TECH-02
- *(backlog-validation)* Fase 1 — cierres y consolidaciones (NUEVO-22, TSK-103, MKT-17, GH-144, LEG-01, COM-04, ADR-012)
- *(Backlog-EDIT)* corregir 12 premisas stale en Backlog.md
- remove obsolete launch-web campaign task views
- add TECH-01..08 tasks from DESKTOP-01b findings
- *(GH-123)* fix typos, broken links, and stale version refs in docs/
- *(backlog)* add AUD-001..AUD-011 findings from doc<->code audit
- *(rules)* fill API/JS skeletons + harden core/release from multi-agent audit
- close launch-web-campaign (MKT-05, MKT-15, WEB-001, WEB-18, GH-119)
- sync investigations A/B/C/D — rules filled, backlog tasks, superseded fix
- DESKTOP-01b 6-integrations research + multi-connection architecture
- *(MKT-05)* add 5th pre-launch blog post on benchmarks
- *(GH-119)* add Vectara migration guide
- DESKTOP-01 tauri platform research report
- enable custom allocator in release binaries (mimalloc/jemalloc, INV-004)
- *(plans)* update PROMPT-MAESTRO-FREEZE recitation for INV-012 anti-locality re-evaluation
- *(opencode)* fix stale skill counts, remove double skill loading, normalize skill references
- *(progreso)* add BACKLOG_HISTORY archive and sdk-gap audit, DRV-014 ADR
- *(progreso)* archive no-progress history to ARCHIVO_HISTORICO, dedup, unify Spanish, fix commit hashes and tables
- *(rules)* add .opencode/rules area-specific agent rules
- *(inv)* extract INV-013/014/015/016 audit findings to Investigaciones docs
- INV-011 INV-012 — core-server separation audit + anti-locality re-evaluation
- *(inv)* complete web audits INV-013/014/015/016 — JSON-LD, light mode, touch targets, motion tokens
- *(acid)* INV-010 move design doc to docs/Investigaciones
- *(inv)* complete INV-007/008/009 competitive benchmark, Python batch, phrase query designs
- *(acid)* INV-010 design multi-layer ACID rollback protocol
- *(task-system)* document harness plan/state format requirements in troubleshooting
- add strategic manual and archive NUEVO-17 task file
- *(task-system)* archive 16 completed task files to tasks/complete/ and mark ENT-04, COMP-029, TSK-107b done in Backlog
- *(task-system)* fix audit findings in .opencode orchestrators and remove legacy .antigravity
- *(server)* make metrics assertions conditional on prometheus feature
- rustfmt sparse search module
- mark ENT-04 as completed in all references (ENT-04)
- log M1-M6 resolution in INV-006 task
- *(blog)* finalize blog series source (M1-M6), fix version to 0.5.0
- docs(strategy) fix product version references to 0.5.0
- migrate NUEVO-08 and INV-006 to progreso
- mark NUEVO-08 task as completed (NUEVO-08)
- update tutorial references in master index and README (NUEVO-08)
- *(book)* sync mdBook tutorial stubs and summary (NUEVO-08)
- *(tutorials)* promote chromadb tutorial to active and add learning path index (NUEVO-08)
- *(tutorials)* add hybrid search and embedding integrations tutorials (NUEVO-08)
- *(tutorials)* rewrite agent memory and RAG tutorials to real vantadb_py API (NUEVO-08)
- add blog series completion plan (INV-006)
- migrate COMP-021 to progreso (COMP-021)
- mark MKT-14 as completed (SKIP gate - already implemented)
- migrate NUEVO-10 to progreso (NUEVO-10)
- migrate COMP-028 to progreso (COMP-028)
- close COMP-019 as WONTFIX - gRPC contradicts embedded-first (ADR)
- mark INV-019 SKIP - advanced tokenizer already implemented
- migrate INV-017 + GH-143 to progress (sccache CI)
- add example smoke tests to CI pipeline (GH-142)
- sccache + nextest install fix (GH-143)
- add INV-017 sccache CI investigation + fix AGENTS.md drift (INV-017)
- add complete docstrings to Python SDK public API (GH-122)
- add doc-tests for public Rust API (GH-124)
- add proptest coverage for WAL record roundtrip (GH-127)
- *(asan,heavy)* fit heavy jobs in CI timeouts
- add concurrent multi-namespace stress test ([#134](https://github.com/ness-e/Vantadb/pull/134))
- *(gate)* skip heavy/fuzz runs when main CI is red
- *(npm)* skip publish if version already exists (idempotent re-runs)
- *(release)* npm triggers on v* release tags
- *(npm)* publish wasm+ts automatically on v* release tags
- *(release)* enable release-plz with trusted publishing (OIDC) for crates.io
- *(npm)* migrate to trusted publishing (OIDC)

### Security

- *(mcp)* log internal error detail instead of leaking to client (AUDREP-61)

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
