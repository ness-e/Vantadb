# TDAM — 01: Core pipeline L0→L3 — Investigación profunda

> Fecha: 2026-08-18 · Agente: vanta-research · Fuente: `TencentCloud/TencentDB-Agent-Memory@97f9465` (branch `feat/server_team`, v2.0.0-beta.1, repo PÚBLICO)
> Nota de fuente: el clon local en `AppData\Local\Temp\opencode\tdam` es COMPLETO — checkout íntegro del commit exacto `97f94654280b2932c35ba4806a491999ed244cc9` (rama `feat/server_team`, v2.0.0-beta.1), verificado contra `.git/logs/HEAD`. Refs = `MemoryCore/src/<ruta>:<línea>` verificadas por lectura directa.

## 1. Resumen ejecutivo

TDAM implementa memoria persistente para agentes en 4 capas LLM-driven: **L0** (raw conversation, JSONL diario), **L1** (memorias estructuradas vía extracción LLM + dedup), **L2** (scene blocks Markdown narrativos), **L3** (persona/doctrina). El motor es un pipeline asíncrono por timers/colas con scheduling per-session, locks distribuidos y dual-write JSONL+vector. Todo depende de un LLM (L1/L2/L3), un `IMemoryStore` (SQLite con sqlite-vec+FTS5 o TCVDB) y un `StorageAdapter` (local fs o COS). Hay dos modos de host: OpenClaw (embedded agent) y Standalone (Vercel AI SDK) — ambos detrás de la misma interfaz `LLMRunner`, por lo que el pipeline es host-neutral.

## 2. Arquitectura — flujo

```
[mensajes] → L0 recorder (conversations/YYYY-MM-DD.jsonl) + L0 vector/FTS
   │  notifyConversation → MemoryPipelineManager (buffer + threshold + timer L1)
   ▼
L1 extractL1Memories: shouldExtractL1 → split bg/new → LLM (escenas+memorias JSON)
   → batchDedup (recall vector|FTS → LLM juicio store/update/merge/skip)
   → writeMemory: records/YYYY-MM-DD.jsonl (append-only) + upsertL1 vector + delete targets
   ▼  (onL1Complete → avanza timer L2)
L2 SceneExtractor (LLM con tools read/write/edit, sandbox scene_blocks/):
   backup → scene_index → prompt (maxScenes, warnings) → LLM escribe .md narrativos
   → cleanup [DELETED] → normalize filenames → syncSceneIndex → nav en persona.md
   ▼  (L2 done → enqueue L3)
L3 PersonaGenerator (LLM con tools, escribe persona.md, 2000/1200 chars):
   diff de escenas desde checkpoint → mode first|incremental → write/edit persona.md
   → strip nav + escapeXmlTags → append nav → markPersonaGenerated

[recall] performAutoRecall → hybrid RRF (FTS5+vector) | embedding | keyword
   → inyecta L3 persona + navegación L2 → tdai_memory_search / tdai_conversation_search
```

**Scheduling (servicios):** `IStateBackend` (buffer/session-state/timers/queue/locks) → `PipelineWorker` consume cola con locks distribuidos (L1 session-level, L2/L3 agent-level, hash tag `{inst:team:agent}`) + renew cada 30s + cascade L1→L2→L3 + dead-letter + XPENDING recovery; `TimerScanner` (16 shards ZSET, leaderless) convierte timers vencidos en tasks.

## 3. Lógica del pipeline (con refs)

**L0 — captura** (`core/conversation/l0-recorder.ts`): `recordConversation` escribe JSONL diario `conversations/YYYY-MM-DD.jsonl`, 1 mensaje/línea (`L0MessageRecord{sessionKey,sessionId,userId?,agentId?,recordedAt,id,role,content,timestamp}`); `sanitizeText`/`stripCodeBlocks`/`shouldCaptureL0`. `core/hooks/auto-capture.ts`: `performAutoCapture` → graba L0 local, escribe vectores si `VectorStore`+`EmbeddingService` disponibles, y notifica `MemoryPipelineManager` (L0 captura TODO; el filtro estricto ocurre en L1).

**L1 — extracción** (`core/record/l1-extractor.ts`): `extractL1Memories` — quality gate `shouldExtractL1` (longitud, símbolos, prompt injection), split `new (max 10) + background (max 5)`, 1 call LLM JSON-mode con escenas (`SceneSegment{scene_name,message_ids,memories[]}`), `parseExtractionResult` con `sanitizeJsonForParse` + `repairExtractionJson` (repara `"priority": sheet` → 50), límite `maxMemoriesPerSession` (10). Tipos: `persona|episodic|instruction|work_fact|work_task|work_method|work_artifact` (legacy `episode/instruct/preference` mapeados). Prompts en `core/prompts/l1-extraction.ts`: modo `chat` (3 tipos, prioridad 0-100, -1 para instrucción global estricta) vs modo `code`/work (4 tipos work, con `owner/deadline/status`, `scope/method_type`, `artifact_type/artifact_ref`).

**L1 — dedup** (`core/record/l1-dedup.ts`): `batchDedup` 2 fases — (1) recall de candidatos: Tier 1 vector (batch-embed + `searchL1Vector`, excluye self-batch, topK 5) o Tier 2 FTS BM25; sin capacidad → skip dedup (store all, sin fallback Jaccard); (2) LLM batch único sobre "unified candidate pool" (`formatBatchConflictPrompt` en `core/prompts/l1-dedup.ts`), decisiones `store|update|merge|skip` con `target_ids[]`, `merged_content/type/priority/timestamps`; fallback store-all ante parse fail. `l1-writer.ts`: `writeMemory` — JSONL append-only (source of truth; guard CR-2 si falta storage → warn), update/merge borra targets en vector store en tiempo real, dual-write vector (`upsertL1`; embedding fail → metadata-only), `version` monotónico, IDs `m_<ts>_<hex>`.

**L2 — scene blocks** (`core/scene/scene-extractor.ts` + `core/prompts/scene-extraction.ts`): LLM actúa como agente con tools `read/write/edit` sandboxed a `scene_blocks/` (CleanContextRunner con allowlist). Prompt exige: máximo `maxScenes` (15), warning por niveles (≥max → merge obligatorio; =max-1 → solo update; cerca → preferir update), naming regex estricto (sin espacios/puntuación, `.md`), soft-delete vía marcador `[DELETED]`, heat management (create 1, update +1, merge sum+1), template META (`created/updated/summary/heat`) + secciones (chat: base/core traits/preferences/implicit signals/core narrative/evolución/contradicciones; work: SOP/judgment/taboos). Post-LLM: cleanup soft-deletes y META-only, `normalizeSceneFilenames`, `syncSceneIndex`, `updateSceneNavigation` (append nav a persona.md solo si hay cuerpo), señal out-of-band `[PERSONA_UPDATE_REQUEST]` → `cpManager.setPersonaUpdateRequest`.

**L3 — persona** (`core/persona/persona-generator.ts` + `core/prompts/persona-generation.ts`): diffs escenas vs `last_persona_time` del checkpoint; modo `first|incremental`; LLM escribe persona.md con tools (prohibido leer otros archivos), límite 2000 chars (chat) / 1200 (team doctrine); template 4-layer scan (base facts → interest graph → interaction protocol → cognitive core) con Chapters 1-4; post: strip nav, `escapeXmlTags` (inyección segura), append nav fresca, `markPersonaGenerated`. `persona-trigger.ts` (local): 5 condiciones de trigger para regenerar.

**Recall** (`core/hooks/auto-recall.ts`, `core/tools/memory-search.ts`, `conversation-search.ts`): estrategias hybrid (FTS5+vector en paralelo, RRF k=60) / embedding / fts, con degradación automática; short-circuit nativo en TCVDB; filtros type/scene (L1) y sessionKey (L0); inyección L3 + navegación L2 en el prompt de recall; `RECALL_TRUNCATION_SUFFIX`.

**Store contract** (`core/store/types.ts`): `IMemoryStore` (extends MemoryPromptStore + MemoryGenerationRefStore) — capabilities flags, L0/L1 upsert/delete/search/query, `reindexAll`, profiles L2/L3, entity metadata (team/user/agent/task), audit, clearMemoryContent; fault-tolerant (devuelve vacío/false, no throw).

**LLM runner contract** (`core/types.ts` local): `LLMRunner.run({prompt, systemPrompt, taskId, timeoutMs, workspaceDir, storage, ...})`. Implementaciones: `CleanContextRunner` (OpenClaw `runEmbeddedPiAgent`, plugins off, `systemPromptOverride`, allowlist `["read","write","edit"]`, `disableTools` sin tools) y `StandaloneLLMRunner` (Vercel AI SDK `generateText`, `compatibility:"compatible"`, tools sandboxed read/write/edit con path validation, storage-backed tools para COS, `AbortSignal.any` + `maxSteps=20`).

## 4. APIs y CLI

- **Facade:** `TdaiCore.initialize()`, `handleBeforeRecall`, `handleTurnCommitted` (`core/tdai-core.ts`); pipeline via `pipeline-factory.ts`: `initDataDirectories/initStores/resetStores/createPipelineManager/createL1Runner/createPersister/createL2Runner/createL3Runner`.
- **Tools del agente:** `tdai_memory_search(query, limit, type?, scene?)`, `tdai_conversation_search(query, limit, sessionKey?)`.
- **Servicios:** `PipelineWorker` (concurrency 60, lock TTL 10min, renew 30s, retries 3 con backoff 5/15/45s, MAX_LOCK_REQUEUE 15), `TimerScanner` (scan 2s, shards 16), `worker-permit-pool.ts`.
- **State backend:** `IStateBackend` local (LocalStateBackend) o Redis (sharded ZSETs + streams); `TaskPayload{type: L1|L2|L3|flush|offload-*}`, `PipelineSessionState{conversation_count, last_extraction_time, l2_pending_l1_count, ...}`.
- **CLI/seed:** `cli/index.ts`, `cli/commands/seed.ts`, `core/seed/*` (input/seed-runtime/types) — siembra de datos inicial.

## 5. Código clave citado

- `MemoryRecord` (l1-writer): `{id, content, type, priority, scene_name, source_message_ids, metadata, timestamps, createdAt, updatedAt, version, sessionKey, sessionId, taskId?, teamId?, userId?, agentId?}`
- `DedupDecision`: `{record_id, action, target_ids[], merged_content?, merged_type?, merged_priority?, merged_timestamps[]}`
- `L1ExtractionResult`: `{success, extractedCount, storedCount, records, sceneNames, lastSceneName?}`
- `extractL1Memories` — filtro L0→L1: "L0 captures everything; strict filtering happens here at L1" (`shouldExtractL1`)
- `writeMemory` — "JSONL is the append-only persistent store (source of truth for backup/recovery). VectorStore (SQLite) is the primary retrieval engine." + guard CR-2.
- `batchDedup` — "v4: Removed JSONL-based Jaccard fallback. Candidate recall relies exclusively on vector search (primary) and FTS5 BM25 (degraded). If neither available → skip."
- SceneExtractor `extract()` — backup → index → prompt → LLM sandbox → cleanup → normalize → sync → nav → persona signal; restore de backup ante fallo de LLM.
- `PersonaGenerator.generateLocalPersona()` — diff `updated > last_persona_time`, mode first/incremental, escribe persona.md via LLM, append nav.
- `PipelineWorker.getLockKey()` — "L1: session-level lock, L2/L3: agent-level lock (shared scene/profile dirs must be mutually exclusive)".
- `CleanContextRunner` — tools allowlist `["read","write","edit"]`; "prevents the LLM from accessing exec, sessions, browser, cron".

## 6. Integración VantaDB — implicaciones

- **Modelo mental:** el patrón "L0 raw → L1 estructurado → L2 narrativo → L3 doctrina" es valioso conceptualmente, pero la implementación TDAM es 100% LLM-driven en hot path. **VantaDB core puro (Rust, WASM) debe mantener la pipeline sin LLM** (`agentic::thread::ThreadStore` + `run_pipeline` ya existentes). Lo LLM-driven (extracción/dedup/escenas/persona) pertenece al crate nuevo **`vanta-memory`**, no al core.
- **Contratos reutilizables en vanta-memory:** `MemoryRecord`/`DedupDecision`/`L1ExtractionResult` como tipos de datos; patrón dual-write "append-only JSONL como source of truth + store vectorial como motor de búsqueda"; degradación por capabilities (vector → FTS → skip); RRF k=60 client-side; límites por sesión (maxMemoriesPerSession, maxScenes, persona ≤2000 chars).
- **Recall design:** estrategia hybrid con inyección de L2/L3 (persona + navegación) es un patrón directo para `vanta-memory`'s recall; en core Rust, equivalente puro = top-k híbrido existente sin LLM.
- **Concurrencia/scheduling:** para VantaDB, timers per-session + cola con locks por grano (session vs agent) es pertinente en el server, NO en el core WASM (el core expone `run_pipeline` síncrono; el orquestado asíncrono vive en el server/integrations que ya existen).

## 7. Riesgos y QUÉ NO copiar

- **NO copiar el split de 4 servicios** (pipeline-worker / timer-scanner / worker-permit-pool / state backend Redis+shards): es complejidad operativa para multi-tenant en la nube de Tencent; VantaDB local-first no lo necesita.
- **No copiar** la dependencia de JSONL como source of truth con limpieza diferida (memory-cleaner reconcilia contra vector store) — en VantaDB la fuente única debe ser el store (SQLite/indizado), sin doble verdad temporal.
- **Riesgo:** prompts en chino con reglas de negocio muy específicas (Kenty) — no traducir literalmente; extraer los principios (3-4 tipos, prioridad numérica, escenas narrativas ≤1500 chars, heat) y reescribir en el idioma del producto.
- **Riesgo:** `runEmbeddedPiAgent` acopla TDAM a OpenClaw; su lección es el `LLMRunner` host-neutral — VantaDB debe mantener esa separación (runtime ≠ pipeline).
- **Incógnitas (confianza media):** `store/sqlite.ts` (esquema FTS5/vec0, ~30KB, no leído completo) y `stateful-pipeline-manager.ts` (20KB) son refinamientos de lo ya capturado; `seed`/`cli` menores. El split `profile:team:T|agent:A` (L2/L3 scoped por team+agent) sugiere multi-tenancy — irrelevante para VantaDB single-tenant, validar si el core llega a exponerlo.

## 8. StatefulPipelineManager — pipeline-v2 en modo servicio

- **Qué es:** orquestador central del pipeline-v2 que reemplaza a `MemoryPipelineManager` manteniendo la misma interfaz externa (L1/L2/L3 runners, `notifyConversation`, `flushSession`, `destroy`) pero **sin estado en proceso**: todo el estado vive en `IStateBackend` (doc header, `utils/stateful-pipeline-manager.ts:1-18`). 500 líneas reales. Con `LocalStateBackend` el comportamiento es idéntico al single-process; con backend remoto el core queda stateless y soporta multi-replica (`:10-11`).
- **Estado que mantiene:** per-session `PipelineSessionState` en el backend — `conversation_count`, `last_extraction_time`, `l2_pending_l1_count`, `warmup_threshold`, `l2_last_extraction_time` (`:154-162`, `:427-439`); set de `_activeInstances` para el TimerScanner (`:106`, `:423-425`); callbacks de runners/persister (`:97-101`, `:137-140`); timers config `l1.idleTimeout` / `l2.delayAfterL1|minInterval|maxInterval|sessionActiveWindow` (`:45-55`, `:83-89`).
- **Encadenado L1→L2→L3:** `notifyConversation` → `stateBackend.captureAtomic` (incremento atómico + umbral + enqueue task `type:"L1"` + timer `L1_idle`) (`:175-240`); al completar L1 el Worker llama `onL1Complete → advanceL2TimerAfterL1` = `max(now+delayAfterL1, lastL2+minInterval)` vía `setTimerIfEarlier("L2_schedule")` (`:301-328`); al completar L2, `onL2Complete → armL2MaxInterval` (timer de tope `now+maxInterval`) (`:333-346`); L3 se dispara desde el worker del server (server.ts:1848-1850). El manager **solo encola**; la ejecución la hace `PipelineWorker` externo y los timers vencidos los convierte en tasks el TimerScanner (`:15-16`).
- **Backlog/checkpoint:** `enqueueL1Drain` (backlog completo, `hasFullBacklog`) (`:363-385`) y `armL1IdleAfterDrain` (cola residual, `hasMore`) (`:398-416`); `flushSession` → cancela timer y encola task `type:"flush"` (`:246-278`); `start(restoredStates)` restaura checkpoint al backend (`:146-169`) y `destroy → persistCurrentStates` vuelca a `PipelineStatePersister` (`:284-292`, `:472-499`); warmup: umbral se duplica 1→2→4… hasta `everyNConversations`, luego 0=graduado (`:451-470`).
- **Relación con runners/factory:** runners inyectados vía `setL1Runner/setL2Runner/setL3Runner` (`:137-140`); `createStatefulPipelineManager` en `utils/pipeline-factory.ts:1207-1231` es drop-in de `createPipelineManager` (`:1110`) tomando `cfg.pipeline.*`; los runners reales se crean con `createL1Runner/createL2Runner/createL3Runner` (`pipeline-factory.ts:371,693,939`).
- **Rol plugin (OpenClaw) vs server:** el server lo inyecta al core vía `setStatefulPipelineManager` (`gateway/server.ts:1811-1826`) y construye el `PipelineWorker` con `TracedTaskExecutor` + callbacks `onL1Complete/onL2Complete` (`server.ts:1841-1854`). En modo servicio `instanceId` es obligatorio por request (header `x-tdai-service-id`); `"__unset__"` → throw (`:187-189`). El plugin standalone usa el manager legacy (`pipeline-manager.ts:300-308`); este es su equivalente stateless para el server.

## 9. Telemetría del pipeline (`core/report/`)

- **Métricas por capa (nombres reales):**
  - L1: `l1_extraction_latency_ms`, `l1_dedup_latency_ms` (`metric-tracking-l1-latency.ts:53,66`); hook en `core/record/l1-extractor.ts:424`.
  - L2: `l2_extraction_latency_ms`, `l2_llm_duration_ms`, `l2_scene_count_before/after`, `l2_scenes_created/updated/deleted` (`metric-tracking-l2-latency.ts:77-155`); hook en `core/scene/scene-extractor.ts:493`.
  - L3: `l3_generation_latency_ms`, `persona_length_before/after`, `persona_drift_ratio` (drift por líneas, `computeLineDriftRatio` `:142`); hook en `core/persona/persona-generator.ts:276`.
  - Recall: `recall_hit_count`, `recall_top_score`, `recall_latency_ms`, `recall_strategy` (codificado skipped=0/keyword=1/embedding=2/hybrid=3) (`metric-tracking-recall.ts:23-28,72-112`); hooks en `core/tdai-core.ts:394` y `gateway/v2-router.ts:946,1221`.
  - Runner (credits): `l1_extraction_credit_rate`, `l1_dedup_credit_rate`, `l2_extraction_credit_rate`, `l3_generation_credit_rate` (`metric-tracking-runner.ts:61-67`); tokens crudos `llm_input_tokens`/`llm_output_tokens` (`:276-289`) y por etapa `{l1_extraction|l1_dedup|l2_extraction|l3_generation}_{input,output}_tokens` (`:295-315`). `MetricTrackingRunner` (`:179`) decora `LLMRunner`: usa `lastUsage` side-channel o estima 3 chars/token; `TOKENS_PER_CREDIT=10000`, `INPUT_RATE=1.0`, `CACHE_RATE=0.2`, `OUTPUT_RATE=4.0` (`:92-100`); `MetricTrackingRunnerFactory` (`:340`) se inyecta en `wirePipelineRunners`.
- **Canales reales:** todo emite por `metricProducer.send()` — fachada que inyecta `traceId` del active span (`kafka-metric-producer.ts:60-80`; `MetricMessage{metric,instanceId,value,traceId}` en `types.ts:135-152`). Backend resuelto por `createObservabilityBackend` (`factory.ts:49-90`): **noop** (default, cero overhead) / **console** / **otlp** (endpoint único, recomendado open-source) / **internal** (módulo privado: Kafka → memory-monitor → Barad + ClickHouse + Langfuse); singleton global `getObservabilityBackend` (`:106`), init idempotente (`:116`). ClickHouse se escribe directo con `@clickhouse/client` (gzip, async_insert, batch 100 / flush 5s, tablas `otel_traces|logs|metrics_*`, env `CLICKHOUSE_*`) (`clickhouse-exporter.ts:1-60`); Langfuse solo filtra spans `ai.*`/`gen_ai.*` (`langfuse-span-processor.ts:1-14`, `parseLangfuseConfig:51`).
- **Traces:** `TracedTaskExecutor` decora el `TaskExecutor` con spans `core.l1.extraction` / `core.l2.extraction` / `core.l3.generation` / `core.flush` (+ offload-l1/l15/l2), restaura trace context desde `TaskPayload.data` y registra `instance_id/session_id/task_type` (`traced-task-executor.ts:22-27,92-150`). Contrato `IObservabilityBackend {trace, log, metric, llmTrace, traceMiddleware, tracePropagation}` (`types.ts:308-335`); todo es error-silent (try/catch, nunca afecta negocio).

## 10. HTTP data plane / gateway

- **Listener y enrutado:** `gateway/server.ts` (3062 líneas) maneja el ciclo de request; `gateway/v2-router.ts` (2295 líneas) implementa el data plane v2/v3 (`V2_PREFIX="/v2"` `:99`, `V3_PREFIX="/v3"` `:116`).
- **Endpoints data plane L0–L3 (18 subpaths, `DATAPLANE_HANDLERS` `v2-router.ts:413-432`):** `conversation/{add,query,search,delete,count}` (L0), `atomic/{update,query,search,delete,count}` (L1), `scenario/{ls,read,write,rm,count}` (L2), `core/{read,write,count}` (L3); montados en `/v2/*` y `/v3/*` (count solo `/v3`) (`:434-442`). `/v3` es la variante de **strict isolation**: `team_id + agent_id + user_id` obligatorios (body o headers `x-tdai-*`), `session_id` opcional (agregación cross-session) (`:102-150`, `V3_ALLOWED_SUBPATHS` `:153-172`). Plus **management plane**: `/v2/team|user|agent|task/*` deprecated (`:451-466`), `/v3/meta/*` (metadata, `server.ts:877-886`), `/v2/pipeline/status` (`v2-router.ts:468,2156`), `/v2|v3/instance/destroy` admin (`server.ts:843-862`).
- **extraRouteTable (módulos):** `/v3/skill/*` (`skill-handlers.ts`), `/v3/knowledge/{create,get,update,delete,list}` (`knowledge-handlers.ts:144-153`), `/v3/chat-memory/clear` (content-clear de chat memory, `chat-memory-handlers.ts:470-472`); unión en `server.ts:1013-1021`. Skill/knowledge/chat-memory se registran como tabla extra y **eluden** la validación estricta de tripleta (su scope viene de `memory_ids`/espacio, `chat-memory-handlers.ts:19-21`).
- **Envelope real:** `ApiResponseEnvelope { code, message, request_id, data? }` (`v2-schemas.ts:399-404`); `successEnvelope` = `{code:0, message:"ok", request_id, data}` (`v2-router.ts:332-334`); `errorEnvelope` (`:336-338`); HTTP status = `code===0 ? 200 : (400≤code<600 ? code : 200)` (`:637-639`). Errores de skill usan códigos modulares tipo `40001/40301/40401/...` (`skill-handlers.ts:6-21`).
- **Auth por capas:** L1 = `checkAuthForV2` (Bearer `server.apiKey`, opcional — no-op si no configurado, `server.ts:893-901,1099-1120`); L2 = `parseV2Auth` exige `Authorization: Bearer {api_key}` **+** `x-tdai-service-id` → `V2AuthContext {apiKey, serviceId}` (`v2-router.ts:344-366`, `v2-schemas.ts:423-426`); L3 = `x-tdai-user-key` solo en rutas v3 meta (`server.ts:878`). El kernel trata Bearer+service-id como **credencial de nivel admin**: no parsea `x-tdai-user-key` en el data plane; la autorización de owner se delega al panel backend (`chat-memory-handlers.ts:13-17`).
- **Service vs standalone:** `deployMode` controla divergencias (ej. mirror JSONL de L0 solo en standalone) (`v2-router.ts:233-240`); en service mode cada request resuelve store/embedding/storage **per-instance** por `serviceId` (`resolveStoreForRequest`/`resolveStorageForRequest` `:373-396`, resolvers inyectados en `server.ts:948-956`); standalone usa los singletons del core.
- **Gateway → pipeline:** el punto de disparo es `POST /v2|v3/conversation/add` → `notifyPipeline` → `statefulPipelineManager.notifyConversation(sessionId, [], instanceId, rounds, teamId, agentId)` (`server.ts:958-968`) → `captureAtomic` encola L1 → `PipelineWorker` consume. Worker construido con `TracedTaskExecutor` + `onL1Complete→advanceL2TimerAfterL1`, `onL2Complete→armL2MaxInterval` (`server.ts:1841-1854`); `buildTaskExecutor` (`:2574`) implementa `executeL1` (quota check → `core.runL1WithStore` → drain backlog) (`:2685-2753`), `executeL2` (`:2776`) y `executeOffloadL1/L15/L2` (`:2872-2909`).

## 11. Offload (borde del scope — resumen)

El offload de TDAM es la **gestión de ventana de contexto** del agente: cadena LLM **L1** (resume `tool_call+tool_result` → `OffloadEntry` con score y `result_ref`) → **L1.5** (task judgment: boundary MMD/continuación, fail-safe con retry) → **L2** (update del MMD activo) → **L4** (skill generation, solo `before_agent_start`) + **compresión local sin LLM** en 3 niveles `mild` (`mildOffloadRatio`) / `aggressive` (`aggressiveCompressUntilBelowThreshold`) / `emergency` (`emergencyCompress`) — `offload/index.ts:43-46,267,297-298,1805,1931-1946`. L1/L1.5/L2/L4 requieren `backendUrl` (`/offload/v1/l15/judge`, `/offload/v1/l4/generate`, `offload/backend-client.ts:168,225`; router server `/v2/offload/*`, `offload_server/router.ts:35`; modo local: L4 no soportado, `offload/local-llm/index.ts:166-167`); hay modo `collect` (solo L1/L1.5, L3 desactivado) (`index.ts:1146-1198`) y hooks `before-prompt-build`/`llm-input-l3`/`before-agent-start`/`after-tool-call` (`offload/hooks/`). El cliente `OffloadContextEngine` (`offload-client/context-engine.ts:65`) calibra tokens y difiere la compactación a `assemble()`: primero server-compaction, fallback `localCompact` (`:173-206,442-519`). Detalle completo (no duplicado aquí): `docs/research/tdam/05-offload.md`.

## RESULTADO
- Estado: ✅ COMPLETO
- Archivo: docs/research/tdam/01-core-pipeline.md
- Hallazgo principal: TDAM = pipeline L0→L3 LLM-driven con dual-write JSONL+vector, scheduling por timers/locks distribuidos y contratos host-neutrales (`LLMRunner`/`IMemoryStore`/`StorageAdapter`) — arquitectura a estudiar para `vanta-memory`, no para el core Rust.
- Ref clave: `core/record/l1-dedup.ts` `batchDedup` + `core/hooks/auto-recall.ts` (hybrid RRF)