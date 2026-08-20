# Plan de Ejecución: Vanta Memory Engine — port de TDAM (F1–F7)

> **Campaign ID: 8f32192f-fd29-4608-823a-18a9fb74c1e6
> **Inicio:** 2026-08-18
> **Estado: completed
> **Fuente:** `docs/research/tdam/` (PLAN + 01..09 verificados + SYNTHESIS) + análisis multi-agente 2026-08-18 (3× vanta-research)
> **Catálogo:** `docs/Backlog.md` — filas MEM-01..38 (este plan es el estado de ejecución; el backlog es el catálogo)
> **Modo:** secuencial por fases — core LLM-free primero (F1–F3), crate LLM-driven después (F4–F5), opcionales (F6–F7) en segunda iteración.

## Resumen

| DO | DEFER | SKIP | BLOQUEADO |
|----|-------|------|-----------|
| F1 search profile, F2 entidades+ACL, F3 skills, F4 vanta-memory (L1/L2/L3/offload/recall), F5 Context Engine | F6 vanta-proxy + F7 wiki (segunda iteración) · billing/quota (server mode) · SDK sub-clientes (MEM-36) | split 4 servicios TDAM, Redis, SQLite/dual-write, store Mongo, @colbymchenry/codegraph, agent-adapters, 3 imágenes Docker, prompts Kenty chino | MEM-31 (callback destino), MEM-29 (dep red en WASM) |

## Decisiones del usuario (2026-08-18, resueltas)

| # | Decisión | Valor |
|---|----------|-------|
| D1 | Trait LLMRunner | **Ambos (sync + async)** — trait sync base + wrapper async opcional en server |
| D2 | Nodo escena MEM-12 | **Entra en F4** (S, barato — ancla LLM-free de L2) |
| D3 | Tokenizador offload | **tiktoken o200k_base** (⚠️ añade dependencia — validar tiktoken-rs en MEM-23; 3 chars/token como fallback) |
| D4 | Persistencia entity_* | **Nodos en partición InternalMetadata** (patrón thread.rs) |
| D5 | vanta-proxy | **Crate aparte** (`vanta-proxy/`, fuera de default-members) |
| D6 | Puertos propios | Fijar al implementar MEM-25 sin colisionar |
| D7 | Permission-checker | **Versión completa (cadena 7 eslabones, 96 líneas)** |
| D8 | Skill extracción | **Síncrona en v1** (cola local solo si latencia lo exige) |
| D9 | MMD formato | **Mermaid literal v1** (05); contrato META como mejora |
| D10 | Callback S2S | **Hook síncrono local + estado en store** (MEM-28); Vanta Studio lee el estado; S2S real diferido |
| D11 | Alcance F6/F7 | **Segunda iteración** de P27 (F1–F5 primero) |
| D12 | Publicación vanta-memory | **Interno del workspace**; publicar como `vantadb-memory` cuando F4-F5 estables |
| D13 | Exposición search profile (2026-08-20) | **IQL + API nativa + MCP** — sintaxis IQL opcional (Studio habla solo IQL vía `/api/v2/query`) |
| D14 | Nombre del perfil configurable (2026-08-20) | **`SearchProfileConfig`** — el `SearchProfile` existente (`src/index/search/profile.rs`, profiler I/O) conserva su nombre; cero riesgo de colisión |
| D15 | Audit log contrato 3 (2026-08-20) | **Crear audit log en server (`/api/v2/audit`, JSONL)** — Memory y Studio escriben/leen el mismo canal en server mode; modo nativo de Studio se mantiene |
| D16 | Alcance de campaña (2026-08-20) | **F1–F3 primero con checkpoint** (entrega LLM-free verificable + review humano), luego F4–F5 |
| D17 | Adelantar MEM-34 (2026-08-20) | **Telemetría L1/L2/L3/recall se ejecuta en F1** — extiende `operational_metrics_snapshot()` que Studio ya consume; contrato de datos probado temprano |
| D18 | MEM-35 data plane (2026-08-20) | **REST `/conversation/add` + `/skill/listing` — data plane orientado a AGENTES, no a Studio.** Razones: (a) `/conversation/add` es pipeline multi-paso (validar → ThreadStore → notify), forzarlo a IQL contaminaría el query language con side-effects; (b) Studio es consola/viewer — no ingesta conversaciones; (c) consistencia con `/api/v2/audit` (D15, también REST). Studio NO lo consume (no lo necesita); si algún día Studio lista skills → 1 wrapper en `server_client.rs` (documentado, trivial) |
| D19 | Tests en F1/F2 (2026-08-20) | **Tests dedicados por tarea** — contrato de tarea incluye tests propios (SearchProfileConfig, paridad IQL/API/MCP, snapshot metrics), no solo `cargo test` global |
| D20 | RRF_K en Studio (2026-08-20) | **Actualizar Studio en paralelo** — tras F1, `retrieval-core.ts` + `selfcheck-retrieval.ts` leen `rrf_k` dinámico del report en vez del literal 60 |

## Principios de adaptación (TDAM → VantaDB)

> Contrato transversal obligatorio para TODAS las tareas MEM-01..38. Los archivos TDAM son **referencia de algoritmos, modelos de datos y prompts** — NUNCA código a portar línea a línea. Cada tarea reimplementa en Rust sobre el stack VantaDB existente.

1. **Todo en Rust, workspace VantaDB.** Nada de TypeScript/Python/Node en runtime. Los crates se integran al workspace raíz (`Cargo.toml` inheritance, `default-members` coherente, features bien declaradas).
2. **Persistencia = VantaDB, siempre.** Todo dato vive en el store de VantaDB (nodos, particiones `InternalMetadata`, `text_index`, HNSW, grafo core). Prohibido: SQLite, Mongo, Redis, JSONL de TDAM, vectores propios fuera del store. El patrón `entity_*`/`skills`/MMD/escenas = nodos + metadata + índices VantaDB, no tablas.
3. **WASM-compatible y local-first.** El core (F1–F3, transversales) no rompe WASM ni añade deps de red. Lo LLM-driven vive en `vanta-memory` (crate LLM-free si no hay LLM); cualquier dep nueva (p.ej. tiktoken) se valida contra WASM antes de fijarse (D3).
4. **LLM opcional, no requerido.** VantaDB no tiene LLM generativo (`src/llm.rs` solo embeddings). El trait `LLMRunner` (MEM-08b) es host-neutral: en modo LLM-free, las rutas degradan (compresión local, store-all, dedup por heurística) — nunca bloquean ni pierden datos.
5. **Reusar antes que crear.** Si el grafo (`src/graph.rs`, graphrag), el planner, la auth, los metrics snapshot o los bindings existentes ya cubren una necesidad → se extienden, no se duplican (MEM-04 evalúa `src/rbac.rs`, MEM-32 expone grafo existente, MEM-34 extiende metrics existente).
6. **Bindings actualizados por contrato.** Python (`vantadb-python`) y TS (`vantadb-ts`) reflejan los nuevos endpoints/tools sin romper backward-compat (MEM-36); los MCP tools se añaden al servidor MCP existente.
7. **Sin deuda del stack TDAM.** No se copia: split 4 servicios, Redis/locks multi-nodo, dual-write, store Mongo, `@colbymchenry/codegraph`, agent-adapters, imágenes Docker, prompts Kenty en chino (reescribir principios, no traducir) — SYNTHESIS §2.4.

## Orden de ejecución (dependencias verificadas, actualizado 2026-08-20)

1. **F1 (MEM-01→02→34):** parametrizar planner (core) con `SearchProfileConfig` → exponer en **IQL + API + MCP** → telemetría adelantada (MEM-34). Sin dependencias previas. LLM-free. **Checkpoint tras F1+F2.**
2. **F2 (MEM-03→04→05):** entidades → checker → auth server + **audit log server `/api/v2/audit`** (D15). Core LLM-free. `src/rbac.rs` NO es dead code — es auth de capa transporte (token→role, `cli_server.rs:527` `auth.rbac.has_permission`, `cli_server.rs:198-205` `Rbac::new()`, `config.rs:114` `RbacConfig`); el checker (MEM-04) es autorización de recursos entity_* con ACL allow-only — coexisten (verificado en MEM-04, resolución del SYNTHESIS en task file MEM-04).
3. **F3 (MEM-06→07):** skills multi-versión core → tools MCP.
4. **F4 (MEM-08a→08b→09→10→11→12→13→14→15→16→17→18→19→20→21):** fundación crate → contratos+trait → L0 → L1 → L2 → L3 → triggers → skill extract → recall → cursor → MCP scenes. **Checkpoint tras F4.**
5. **F5 (MEM-22→23→24):** Context Engine cascade → emergency/tokens → MMD. **Checkpoint tras F5 (release candidate).**
6. **Segunda iteración (F6 MEM-25..27, F7 MEM-28..33):** proxy → wiki. Opcional.
7. **Transversales:** MEM-35 (data plane) tras F3; MEM-36 (SDK) tras F3; MEM-37 (integración) tras F4/F5; MEM-38 (ADR+docs) gate pre-release. MEM-34 ya NO es transversal — se adelantó a F1 (D17).

## Referencias de extracción (clon TDAM)

> Clon: `C:\Users\Eros\AppData\Local\Temp\opencode\tdam` @ `97f9465`. Prefijos: `MC` = `MemoryCore/src/`, `MP` = `MemoryProxy/src/`, `MK` = `MemoryKnowledge/src/`. Los archivos listados son la **referencia directa de port** — leer antes de implementar cada tarea, NO reimplementar desde 0. Verificados contra el clon (ronda 2026-08-20). Aviso SYNTHESIS §2.4: portar modelo/algoritmos/patrones, nunca el stack (SQLite/Mongo/Redis/Docker).

| Tarea | Archivo(s) fuente TDAM | Qué portar |
|---|---|---|
| MEM-01 | `MC/core/store/types.ts`, `MC/core/store/search-utils.ts`, `MC/core/tools/memory-search.ts` | SearchProfile (mode/rrf_k/candidate_k), search-utils |
| MEM-02 | `MC/core/tools/memory-search.ts`, `MC/core/tools/conversation-search.ts` | Passthrough profile en tools MCP |
| MEM-03 | `MC/metadata/types.ts`, `MC/metadata/store/interface.ts`, `MC/metadata/constants.ts`, `MC/metadata/utils/id-generator.ts` | Modelo entidades + CRUD (DDL SQLite es referencia, NO copiar — VantaDB es nodos InternalMetadata) |
| MEM-04 | `MC/metadata/service/permission-checker.ts` (172 líneas reales) | Cadena allow-only resource→owner→member→visibility→role-default→ACL |
| MEM-05 | `MC/metadata/router/auth.ts`, `MC/metadata/service/resolve-user-id.ts`, `MC/metadata/service/user-visibility.ts`, `MC/metadata/router/pagination.ts` | Auth 3 capas + resolución user |
| MEM-06 | `MC/core/skill/skill-store-ddl.ts` (104), `skill-store.ts` (733), `skill-versioning.ts` (435), `skill-format.ts`, `skill-config.ts`, `skill-core.ts` | Esquema multi-versión, optimistic lock, TTL, idempotencia |
| MEM-07 | `MC/core/skill/skill-tools.ts`, `skill-permission.ts`, `MC/gateway/skill-handlers.ts`, `skill-schemas.ts` | Tools skill_* (6), límites, owner check |
| MEM-08a | (scaffold — sin fuente TDAM directa; ver `MC/package.json`, `MC/index.ts` para layout) | Estructura crate |
| MEM-08b | `MC/core/abstractions/types.ts` (87), `MC/offload/types.ts`, `MC/adapters/standalone/llm-runner.ts` (467), `MC/adapters/openclaw/llm-runner.ts` | MemoryRecord/DedupDecision + trait LLMRunner host-neutral |
| MEM-09 | `MC/core/hooks/auto-capture.ts` (347), `MC/core/conversation/l0-recorder.ts` (607) | L0 capture idempotente |
| MEM-10 | `MC/core/record/l1-extractor.ts` (738), `MC/core/prompts/l1-extraction.ts` (417), `MC/offload/local-llm/parsers/l1-parser.ts`, `json-utils.ts`, `MC/offload/local-llm/prompts/l1-prompt.ts` | Split + 1 call LLM JSON + parse con reparación |
| MEM-11 | `MC/core/record/l1-dedup.ts` (408), `l1-reader.ts`, `l1-writer.ts`, `MC/core/prompts/l1-dedup.ts` (236) | Dedup 2 fases store/update/merge/skip |
| MEM-12 | `MC/core/scene/scene-format.ts` (75), `scene-index.ts` (137) | Contrato META + nodo escena |
| MEM-13 | `MC/core/scene/scene-extractor.ts` (604 — tools sandbox) | Tools read/write/edit sandboxed + store |
| MEM-14 | `MC/core/scene/scene-extractor.ts` (604), `MC/core/prompts/scene-extraction.ts` (572), `scene/filename-normalizer.ts`, `scene-format.ts` | Strategy UPDATE>MERGE>CREATE, heat, soft-delete, emptyExtraction |
| MEM-15 | `MC/core/persona/persona-generator.ts` (304), `persona-trigger.ts` (136), `MC/core/prompts/persona-generation.ts` (329), `MC/core/scene/scene-navigation.ts` (76) | Modos first/incremental, límites, escapeXml, triggers |
| MEM-16 | `MC/utils/stateful-pipeline-manager.ts` (500), `pipeline-manager.ts` (1218), `pipeline-factory.ts` (1231), `MC/services/pipeline-worker.ts` (843), `timer-scanner.ts`, `MC/utils/managed-timer.ts`, `checkpoint.ts` (745), `MC/core/state/types.ts`, `local-backend.ts` | Orquestación timers+locks, estado local sin Redis, reloj fake |
| MEM-17 | `MC/core/skill/skill-extractor.ts` (587), `MC/core/skill/conversation-add/{trigger-service,extract-worker,worker-pool,message-compressor,oversize-strategy,prepare-archive,skill-core-sink,wire,agent-task-queue,buffer-storage}.ts`, `MC/core/skill/prompts/skill-review-prompt.ts` (198), `skill-listing-prompt.ts` | Transcript marcadores, truncado, review taxonomía, sink idempotente |
| MEM-18 | `MC/core/hooks/auto-recall.ts` (999), `MC/core/memory-prompt/composer.ts` (41), `resolver.ts` (102), `types.ts` (142), `MC/core/profile/profile-sync.ts` (494) | prepend/append + 3 modos recall |
| MEM-19 | `MC/utils/sanitize.ts` (405), `MC/utils/text-utils.ts` (31) | sanitize_text + truncación code-point |
| MEM-20 | `MC/offload/state-manager.ts` (460 — lastOffloadedToolCallId), `MC/offload/storage.ts` (664), `MC/offload/hooks/after-tool-call.ts` (594) | Cursor persistente por sesión |
| MEM-21 | `MC/core/scene/scene-navigation.ts` (76), `scene-index.ts`, `MC/gateway/knowledge-handlers.ts` | Tools scene_read/list/query |
| MEM-22 | `MC/offload-client/context-engine.ts` (526), `MC/offload_server/compact/{compaction-handler.ts (328), compressor.ts (1194), fast-path.ts (189), helpers.ts, mmd-injector.ts}` | assemble + cascada mild/aggressive + revert si summary>original |
| MEM-23 | `MC/offload/fast-token-estimate.ts` (307), `l3-token-counter.ts` (35), `benchmark-token-estimate.ts` (89), `context-token-tracker.ts` (166) | Emergency + estimador tokens |
| MEM-24 | `MC/offload/mmd-injector.ts` (374), `mmd-meta.ts` (66), `MC/offload/pipelines/l2-mermaid.ts` | MMD persistente, fingerprint, marker _mmdContextMessage |
| MEM-25 | `MP/handler.ts`, `MP/server.ts`, `MP/auth.ts`, `MP/anthropicHandler.ts`, `MP/codexHandler.ts`, `MP/workbuddyHandler.ts` | 3 protocolos wire verbatim + rutas |
| MEM-26 | `MP/session/index.ts` (203), `MP/auth.ts`, `MP/identity.ts`, `MP/injection/` (8 archivos), `MP/mem-command/index.ts` (72) | Ciclo auth→session→injection local |
| MEM-27 | `MP/rate-limit/`, `MP/report/`, `MP/clickhouse.ts`, `MP/langfuse.ts`, `MP/opik.ts`, `MP/credit-reporter.ts`, `MP/mem-command/commands/` | Rate-limit sliding window, write-back, reporting (SIN Opik/Langfuse), mem: |
| MEM-28 | `MK/engines/wiki/index.ts` (23), `MK/engines/wiki/ingest-v2/{index.ts, cascade.ts, frontmatter.ts, slug.ts, template.ts}` | State machine pending→ready, locked:true, dedup |
| MEM-29 | `MK/engines/wiki/ingest-v2/chunker.ts`, `MK/source-fetcher/` (4 archivos), `file-protocol.ts` | SSRF blocklist + chunker 12k/400 |
| MEM-30 | `MK/engines/wiki/ingest-v2/{merge.ts, llm.ts, index-builder.ts, overview.ts, prompts.ts}` | Merge serial + pLimit + ensureSources |
| MEM-31 | `MK/engines/wiki/ingest-v2/index.ts` (progress/callback), `log-writer.ts` | Callback run_id + throttle |
| MEM-32 | `MK/engines/code/index.ts` (2), `MK/mcp/` (3), `MK/routes/` (6) | Tools code_* sobre graphrag EXISTENTE (patrón de rutas, no el grafo) |
| MEM-33 | `MK/mcp/` (3), `MK/routes/` (6), `MK/store/index.ts` (80) | Tools wiki_* sobre MEM-28 |
| MEM-34 | `MC/core/report/metric-tracking-{l1,l2,l3,recall}-latency.ts`, `metric-tracking-runner.ts`, `MC/offload/state-reporter.ts` (348), `MC/api-trace/*` | Latências por capa + envelope |
| MEM-35 | `MC/gateway/chat-memory-handlers.ts` (476), `knowledge-handlers.ts`, `memory-prompt-handlers.ts` | Data plane /conversation/add + /skill/listing (patrón, no endpoints TDAM) |
| MEM-36 | `sdk/memory-core/typescript/src/index.ts` (45), `sdk/memory-core/typescript/src/v3/index.ts` (177), `sdk/memory-core/python/` | Estructura sub-clientes por dominio |
| MEM-37 | `MC/core/hooks/auto-recall.ts` (999), `MC/offload-client/context-engine.ts` (526) | Integración offload↔recall |
| MEM-38 | (ADR/docs — sin fuente TDAM) | Documentación |

## Checkpoints

- **Checkpoint 1 (tras F1+F2):** `cargo test -p vantadb` verde; search profile y entidades/checker con tests; review con humano antes de F3. → **✅ APROBADO 2026-08-20 por review humano (GO, sin push, orden secuencial 06→07→35)**
- **Checkpoint 2 (tras F4):** `cargo test -p vanta-memory` verde con LLM mock; pipeline L0→L3 end-to-end con mock; `cargo check -p vantadb` sin regresiones; review.
- **Checkpoint 3 (tras F5):** offload assemble/mild/aggressive/emergency verde; report correcto; decide D3 definitivamente.
- **Checkpoint 4 (release):** unified-review certify (Pre-Launch Gate, 8 capas) + semver-checks + ADR.

## Tasks (F1 — ejecutando)

### Task 1: MEM-01 — F1 Search profile por namespace en core
- **Archivos clave:** `src/planner.rs`, `src/sdk/serialization/vector_types.rs`, `src/sdk/types.rs`, `src/sdk/search/mod.rs`, `src/cli_server.rs` (parser IQL)
- **Gate Justificación:** F1 base — parametriza planner con `SearchProfileConfig`, expone en IQL/API/MCP (D13), report RRF incluye `rrf_k` (D20)
- **Contrato: cargo check -p vantadb-server + tests endpoints
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-01.md`
- **Estado:** ✅ COMPLETED (commit `6a50b8ee`, verify `cargo check -p vantadb` ✅ 2026-08-20)
- **last-synced:** 2026-08-20T08:00

### Task 2: MEM-02 — F1 Exponer search profile en MCP/search
- **Archivos clave:** `vantadb-mcp/src/handlers/tools.rs`, `vantadb-mcp/src/validation.rs`, `vantadb-mcp/src/config.rs`, `src/cli_server.rs`
- **Gate Justificación:** paridad IQL/API/MCP (D13) — depende de MEM-01 (mismo `SearchProfileConfig`)
- **Contrato:** `cargo check -p vantadb-mcp` pasa; test de paridad IQL/API/MCP (D19)
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-02.md`
- **Estado:** ✅ COMPLETED (commit `32b09daf`, verify `cargo check -p vantadb-mcp` ✅ + 44/44 MCP + 11/11 core 2026-08-20)

### Task 3: MEM-34 — F1 Core Telemetría por capa (adelantada, D17)
- **Archivos clave:** `src/metrics/core/mod.rs`, `src/metrics/core/snapshot.rs`, `src/sdk/types.rs`, `src/sdk/serialization/conversions.rs`, `src/audit.rs` (helper memory)
- **Gate Justificación:** extiende `operational_metrics_snapshot()` que Studio ya consume; audit log server `/api/v2/audit` (D15)
- **Contrato:** `cargo check -p vantadb` pasa; tests dedicados de snapshot metrics (D19)
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-34.md`
- **Estado:** ✅ COMPLETED (commit `84f28a18`, verify `cargo nextest run --profile audit --workspace` ✅ 2002/2002, clippy -D warnings ✅ 2026-08-20)
- **last-synced:** 2026-08-20T12:45
- **Nota:** `src/metrics/core/state.rs` no existe (estado vive en mod.rs); `vantadb-server/src/audit.rs` NO se creó — audit server ya existía en core (WEB-01, `src/audit.rs` + `/api/v2/audit` auth-protegido); MEM-34 aporta 13 campos de telemetría + `AuditEvent::memory` para F4

### Task 4: MEM-03 — F2 Entidades entity_* + CRUD en core
- **Archivos clave:** `src/entity.rs` (nuevo), `src/entity/tests.rs`, `src/lib.rs`
- **Gate Justificación:** F2 base — modelo teams/users/agents/tasks/assets (hoy NO existe; modelo actual namespace+key); D4: persistencia = nodos partición InternalMetadata (patrón thread.rs)
- **Contrato:** `cargo check -p vantadb` pasa; tests dedicados de CRUD entity (D19)
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-03.md`
- **Estado:** ✅ COMPLETED (commit `23719e23`, verify `cargo check -p vantadb` ✅ + 14/14 entity tests 2026-08-20)
- **last-synced:** 2026-08-20T13:00

### Task 5: MEM-04 — F2 Permission-checker allow-only
- **Archivos clave:** `src/entity/checker.rs`, `src/entity/checker_tests.rs`, `src/entity/mod.rs`
- **Gate Justificación:** cadena 7 eslabones (D7); `src/rbac.rs` evaluado → **NO dead code** (coexistencia, ver línea 55); resuelve contradicción SYNTHESIS (172 líneas reales del clon TDAM @ `97f9465`)
- **Contrato:** `cargo check -p vantadb` pasa; tests dedicados del checker (D19)
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-04.md`
- **Estado:** ✅ COMPLETED (commit `9717bf03`, verify `cargo nextest run -p vantadb -- entity` ✅ 37/37 2026-08-20)
- **last-synced:** 2026-08-20T14:10

### Task 6: MEM-05 — F2 Auth 3 capas en server + audit log
- **Archivos clave:** `src/cli_server.rs`, `vantadb-server/src/middleware.rs`, `vantadb-server/src/server.rs`
- **Gate Justificación:** L1 Bearer timingSafeEqual, L2 service-id, L3 user-key → userId/isSystemAdmin desde entity_* (MEM-03); `/health` pública; **audit log server YA EXISTE (WEB-01, `src/audit.rs` + `/api/v2/audit`)** — extender, no duplicar (descubierto en MEM-34)
- **Contrato:** `cargo check -p vantadb-server` pasa; tests dedicados de auth 3 capas (D19)
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-05.md`
- **Estado:** ✅ COMPLETED (commit `01a5de66`, verify `cargo check -p vantadb-server` ✅ + tests auth 16/16 2026-08-20)
- **last-synced:** 2026-08-20T15:15

### Task 7: MEM-06 — F3 Esquema skills multi-versión en core
- **Archivos clave:** `src/skills.rs` (crear), `src/sdk/types.rs`, `src/lib.rs`, `tests/`
- **Gate Justificación:** F3 base — VantaDB no es SQL: namespace `skills` + nodos por versión con metadata (`version`, `is_head`, `content_hash`, `expires_at`, owner_agent), índice único parcial por `(owner,name)` donde is_head, optimistic lock `expected_version`, TTL keep-recent=3, idempotencia por content-hash; CRUD create/update/patch/delete/versions. Reusar text_index + HNSW propios (NO vec0). Consumido por MEM-07 (MCP skill_*) y MEM-35 (`GET /skill/listing`). Reusa EntityStore de MEM-03 como base de persistencia.
- **Contrato:** `cargo check -p vantadb` pasa; tests dedicados de skills multi-versión (D19)
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-06.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-08-20T16:00

### Task 8: MEM-07 — F3 MCP tools skill_*
- **Archivos clave:** `vantadb-mcp/src/handlers/tools.rs`, `vantadb-mcp/src/skills.rs` (crear)
- **Gate Justificación:** 6 tools del review agent (`skill_list/view/create/update/patch/files_write`) sobre MEM-06; writes exigen `expected_version`; owner check → 404 sin filtrar existencia; límites 5MB/recurso, 50MB/skill. Paridad con API nativa (D13).
- **Contrato:** `cargo check -p vantadb-mcp` pasa; tests dedicados de skills tools (D19)
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-07.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-08-20T17:00

### Task 9: MEM-35 — F3 Core Data plane de referencia en server
- **Archivos clave:** `src/cli_server.rs`, `vantadb-server/src/server.rs`, `vantadb-server/tests/e2e.rs`
- **Gate Justificación:** rutas **REST** `POST /conversation/add` (01 §10, vía ThreadStore) y `GET /skill/listing` (03 §4, sobre MEM-06) protegidas por auth existente; **orientado a AGENTES, no a Studio (D18)** — `/conversation/add` es pipeline multi-paso, forzarlo a IQL contaminaría el query language; Studio es viewer y no ingesta; si Studio lista skills algún día → 1 wrapper en `server_client.rs`. NO copiar `/v3/session/init` ni `/v3/knowledge/query` (no existen en TDAM — endpoints propios VantaDB). Auth 3 capas de MEM-05 protege.
- **Contrato:** `cargo check -p vantadb-server` pasa; tests dedicados de endpoints data plane (D19)
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-35.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-08-20T18:00

## Riesgos

| Riesgo | Impacto | Mitigación |
|--------|---------|------------|
| Coste LLM por flush (3 llamadas L1/L1.5/L2) | Alto | Modo LLM-free + control triggers (SYNTHESIS §4); defaults configurables |
| Compresión pierde detalle (refs solo a demanda) | Medio | Documentar trade-off en report; cursor idempotente (MEM-20) |
| Heat lo mantiene el LLM (no contador real) | Medio | Documentar; MCP scene_* depende de confiabilidad (MEM-21) |
| `src/rbac.rs` dead code ↔ checker nuevo | Bajo | Decisión explícita en MEM-04 (reemplazo vs coexistencia) → **RESUELTO: coexistencia** — rbac.rs es auth transporte (token→role, cli_server.rs:527), checker es auth de recursos entity_* (ACL allow-only); ambos necesarios (verificado en MEM-04) |
| CreditCalculator ÷1000 vs ÷10000 TDAM | Bajo (diferido) | Elegir UNA al portar billing (post-F7) |
| Prompts Kenty en chino | Medio | Reescribir principios, no traducir (MEM-10) |
| **RRF_K=60 hardcodeado en frontend Studio** (`retrieval-core.ts`) | Medio (resuelto D20) | MEM-01: report RRF incluye `rrf_k` usado; Studio actualizado en paralelo para leerlo dinámicamente |
| **`SearchProfile` existente es profiler I/O, no perfil configurable** | Bajo (resuelto D14) | `SearchProfileConfig` como nombre nuevo; profiler conserva su nombre |
| **Audit log vive hoy en desktop nativo, no en server** | Medio (resuelto D15) | MEM-05/MEM-34 crean `/api/v2/audit` en server; Studio lo lee por contrato en server mode |
| **`operational_metrics_snapshot` y `SearchProfile` sin tests covering** | Medio (resuelto D19) | Tests dedicados por tarea en F1/F2 (no solo `cargo test` global) |
| **MEM-35 REST invisible para Studio** | Bajo (resuelto D18) | Data plane agent-facing; Studio es viewer, no lo necesita; wrapper documentado si algún día lista skills |

## Relación con P26 (Vanta Studio)

Integración **por contratos, no por ejecución** — campañas independientes (velocidades distintas: Studio Fase 0 en curso; este plan draft). Ningún contrato es bloqueante; la integración real se toca cuando F4/F5 existan (2ª iteración, D11). D10 ya decide el punto de unión principal: *"Hook síncrono local + estado en store (MEM-28); Vanta Studio lee el estado"*.

| # | Contrato | Lado Studio (P26) | Lado Memory (P27) | Estado |
|---|----------|-------------------|-------------------|--------|
| 1 | `explain_memory_search` (VS-CORE-03, ya existe en core) | Lente RETRIEVAL (Fase 1) muestra por qué | Recall (F4) usa el mismo search | Un contrato, dos consumidores — ya resuelto en core |
| 2 | Nodos escena + META `{created,updated,summary,heat}` | Grafo/IQL (Fase 2) + Inspector renderizan escenas/skills/entities | F4 añade nodo escena al grafo core (L2, MEM-12) | Inspector KV genérico ya los cubre — sin código ahora |
| 3 | Audit log JSONL compartido | ACTIVITY + Timeline (Fase 1) | Telemetría por capa (MEM-34, adelantada a F1): eventos L1/L2/L3/offload | **Resuelto 2026-08-20 (D15):** `vantadb-server` gana `/api/v2/audit` (MEM-05); Memory escribe, Studio lee en server mode. Modo nativo de Studio (`commands/audit.rs`) se mantiene |
| 4 | DTO estado (MEM-28) | Studio lee estado vía bridge Tauri | State store (pending→ready, run_id) | Mismo patrón que VS-11 (DTO enriquecido); definir cuando exista F7 |

**Punto de diseño compartido (no bloqueante):** VS-CORE-07 (retención de versiones) lo necesitan ambos — Studio para Historial+Diff, memory para offload/skills versionadas. Acordar el diseño una sola vez cuando VS-CORE-07 se ejecute (task file con cláusula de doble consumidor). Ver también: MEM-01 debe exponer el search profile en las mismas estructuras que `explain` (consumible por la lente RETRIEVAL) — y **el report RRF debe incluir `rrf_k` usado**, porque el frontend Studio (`retrieval-core.ts`, `selfcheck-retrieval.ts`) hardcodea RRF_K=60; con profiles rrf_k≠60 la lente mostraría números incorrectos sin ese campo (D13 + cambio pequeño en P26, no bloqueante).

## Open Questions

1. ✅ Orden F1–F7 y decisiones D1–D12 **confirmadas por el usuario 2026-08-18** (ver tabla arriba).
2. ✅ F6/F7 → **segunda iteración** de P27 (D11).
3. ✅ Publicación → **interno del workspace** (D12).
4. ⚠️ D3 (tiktoken): validar que `tiktoken-rs` compile en WASM antes de fijar MEM-23; si no, fallback 3 chars/token documentado.
5. ✅ D13–D17 **confirmadas por el usuario 2026-08-20** (IQL+API+MCP, SearchProfileConfig, audit log server, F1–F3 primero, MEM-34 adelantada).
6. ✅ D18–D20 **resueltas 2026-08-20** — MEM-35 = REST agent-facing (D18, no bloquea F1/F2); tests dedicados por tarea (D19); Studio lee rrf_k dinámico en paralelo (D20).
7. ⏳ Contrato 1 (explain): el `explain` core debe exponer el `rrf_k` usado en su report para que Studio lo consuma — se resuelve junto a MEM-01/D20 (no bloqueante).

=== RECITATION ===
Campaign ID: 2e7f046b-34d3-4d60-9b11-88d3c5f910a7
Objetivo activo: MEM-35 Data plane REST server
Estado: completed ✅
Última acción: MEM-35 verificado (9693d0ff, 11/11 e2e + 2077/2077)
Resultado: ✅
Próxima acción: Checkpoint tras F1+F2+F3 — reportar al usuario
Contrato: cargo check -p vantadb + tests dedicados del checker (D19) — `cargo nextest run -p vantadb -- entity` ✅ 37/37
Próxima tarea si completa: F4 (MEM-08a..21) tras aprobación
=== END RECITATION ===
