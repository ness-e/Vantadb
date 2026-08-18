# TDAM — 08: MemoryKnowledge + MemoryPanel + SDK — Investigación profunda (REVISADO)

> **Fecha:** 2026-08-18 · **Fuente:** clone `tdam` @ `97f9465` (rama `feat/server_team`) · **Verificación:** 100% contra código real (glob/grep/Read)
> **Alcance:** `MemoryKnowledge/`, `MemoryPanel/`, `sdk/` · **Estado previo:** el archivo anterior contenía rutas/endpoints/tools/código fabricados (`wiki/{fetcher,extractor,committer,state-machine,progress}.ts`, `/v3/knowledge/*`, `mint`, 4 SDK clients, `sdk/memory-consumer`). Esta versión solo transcribe lo leído.

## 1. Resumen ejecutivo

MemoryKnowledge (KS) es un servicio Hono que expone conocimiento wiki + code-graph a agentes LLM vía API HTTP y MCP stdio. MemoryPanel (Panel) es un control-stateless en `/api/v1` que gestiona asignación/visibilidad y hace de receptor de callbacks S2S de KS. El SDK oficial es SOLO `sdk/memory-core` (MemoryClient v2/v3, TS+Python); no existen KnowledgeClient/ProxyClient/PanelClient ni `sdk/memory-consumer`.

- **KS**: monta TODO bajo `/v3` (server.ts:62-96): `/wiki`, `/code-graph`, `/tools`, `/internal/llm-binding`, `/auto-sync`. Swagger en `/docs` + `/openapi.json` (server.ts:98-111). Health sin prefijo.
- **Panel**: `API_PREFIX = '/api/v1'` (app.ts:15), stateless, sirve `web/dist` vía serveStatic (app.ts:50-56). Conocimiento se inyecta a agentes con `injection_mode='tool'` (allocate-routes.ts:8,103).
- **SDK**: `sdk/memory-core` (TS `@tencentdb-agent-memory/memory-sdk-ts-v2` + Python `tencentdb_agent_memory`) — clientes conversation/atomic/scenario/core/offload/metadata/skill/memory-prompt/memory-generation-log + COS (index.ts:9-45; v3/client.ts).
- **Concurrencia real**: entre ARCHIVOS con `KNOWLEDGE_WIKI_INGEST_CONCURRENCY` default 3, clamp 1-10 (config.ts:95-98); global LLM `KNOWLEDGE_LLM_GLOBAL_CONCURRENCY` default 5, clamp 1-20 (config.ts:104-107), aplicado como `globalLlmLimit = pLimit(getGlobalLlmConcurrency())` (module.ts:35). Chunks en serie. Dedup por path canónico (type+title → `wiki/{dir}/{slug}.md`), no contentHash (ingest-v2/index.ts:392-410).

## 2. Arquitectura y flujo

```
MemoryPanel (stateless, /api/v1, React web/dist) ── control + callback receptor
   │ POST /knowledge/*  (allocate/unbind/agent-fixed/set-visibility/grant;
   │   wiki/* 14; code-graph/* 8; team-assets; status-callback)
   │ POST /chat-memory/{allocate,unbind,...}; /agent/delete-cascade; /agent-overview/bootstrap
   ▼
MemoryKnowledge (Hono, /v3, PORT local 8421 / global 8424)
   ├─ /wiki              (16 handlers POST: create/get/list/delete/ingest/update-meta + raw/* + page/* + graph/search)
   ├─ /code-graph        (13 según cabecera; con update-meta = 14: create/list/get/sync/delete/update-meta + 8 query)
   ├─ /tools             (list + call; 7 tools wiki + 9 code, read-only)
   ├─ /internal/llm-binding (set/status/list)
   └─ /auto-sync         (GET status, POST trigger)  ← mounts en server.ts:91
        ▼
   store/ (wiki-service, code-graph-service, llm-binding-store, auto-sync-scheduler, build-queue)
   engines/ (wiki: manager+graph-search+index-db+ingest-v2; code: bridge @colbymchenry/codegraph)
   source-fetcher/ (git-fetcher: https-only + SSRF blocklist)
   callback.ts → POST {TMC}/api/v1/knowledge/status-callback (callback.ts:60,92)
```
Callbacks: KS → Panel (TMC_CALLBACK_URL=Panel). Panel al recibir `ready` escribe entity_knowledge en el kernel vía `/v3/knowledge/create` (callback-routes.ts:202-242) y registra meta asset de code-graph con `owner_user_key` stashado (callback-routes.ts:67-112).

## 3. Lógica y algoritmos (refs verificadas)

- **State machine wiki**: `pending → processing(scanning/ingesting) → ready / failed(+sync_error)` (wiki-service.ts:5-7). `runBuild` (wiki-service.ts:1014-1077): set `processing/scanning` → worker → `ready` (internal_status null, page_count, last_sync_at) o `failed` con `sync_error` truncado a 500. `run_id = randomUUID()` por build (l.1026), compartido con progress callbacks. Re-ingest resetea: `ingest()` (l.272-288) → busy si pending/processing (409), si no → `pending` + `enqueueBuild`. Checkpoints anti-delete en l.1016 y l.1039.
- **State machine code-graph**: `pending → processing(cloning/fetching/indexing) → ready / failed(+sync_error)` (code-graph-service.ts:7); `sync()` resetea a `pending` (l.162-169). Fases internas reales del worker: `fetching`/`indexing` (incremental) o `cloning`/`indexing` (fresh) (module.ts:129-159).
- **Engine manager wiki**: estados internos `scanning|ready|error` (types.ts:91); ante interrupción `state.status = "error"; state.error = "Restart"` (manager.ts:810); restore fallido → error (manager.ts:922-951).
- **Ingest wiki (ingest-v2/index.ts)**: `extractSource` (l.112) con `SOURCE_CHAR_BUDGET = 28_000` (l.78, usado en l.129-132; chunker defaults 12000/400 en chunker.ts:19-20 solo si se invoca con esos defaults) → `commitCandidates` SERIAL (l.211-283): agrega por relPath, `mergePage` bajo `globalLlmLimit`, write + `rebuildIndexFile` + `appendIngestLogBatch`; falla de merge por página no bloquea el resto. Modo `two-stage` (analysis→generate) o `single-stage` (l.80-91). `canonicalizePagePath` normaliza path por type+title (dedup estable, l.392-410); `STRUCTURAL_FILES` protegidos (l.69-75); `ensureSources` fuerza frontmatter `sources` (l.368-375).
- **Índice wiki**: por wiki un `index.db` SQLite con 4 tablas `wiki_fts` (FTS5 unicode61) / `page_meta` / `graph_edge` / `source` (index-db.ts:76-118); pool de lectura LRU `POOL_MAX=300` (l.33-37), WAL (l.64-69). `writeIndex` reconstruye en transacción las 3 tablas (manager.ts:394-412); FTS query con `bm25(wiki_fts, 5.0, 1.0)` — title×5 (manager.ts:381-391). Graph multi-hop BFS `graphMultiHopSearch` con `DEFAULT_MAX_NODES = 200` (graph-search.ts:38) — el cap 200 es del WIKI, no de codegraph.
- **Routing LLM real**: `resolveLlmConfig` (llm-binding-store.ts:146-185). mode=`proxy` → `baseUrl = {proxy_base_url}/proxy/{service_id}/v1` (l.167); mode=`byo` → `base_url` directo (l.173-184). Model SIEMPRE del global `LLM_MODEL` (l.135-136). Sin binding usable: `LLM_MODE=custom` → global directo; `LLM_MODE=proxy` (default) → baseUrl/apiKey vacíos para fallar loud (l.152-157). Rutas: `POST /internal/llm-binding/{set,status,list}` (llm-binding.ts:40,95,103).
- **CodeGraph real**: npm `@colbymchenry/codegraph` + `ToolHandler` (bridge.ts:8-78, incluye resolución de plataforma pkg `codegraph-${platform}`). Queries internos `codegraph_{search,explore,callers,callees,impact,node,status,files}` (tools.ts:385-395). `source-fetcher` usado SOLO por code-graph (module.ts:119-160): `GitSourceFetcher` = https-only (git-fetcher.ts:59-63) + SSRF blocklist `PRIVATE_ADDR_RE` 10./172.16-31./192.168./169.254./127./0./localhost/::1/fe80: (l.25-26), desactivable con `KNOWLEDGE_SSRF_CHECK=off` (l.32-37); `sync` incremental con `git clean -e .codegraph` (l.87-97).
- **Progress**: phases `extracting|merging|indexing` sobre archivos con `{total,completed,failed,skipped,percent}` y throttle `PROGRESS_THROTTLE_MS = 500` (manager.ts:110,121); fire-and-forget a `status-callback` (callback.ts:90-101); Panel valida y almacena en `ingestProgressStore` (callback-routes.ts:121-151).
- **Summary wiki**: LLM, prompt ≤100 chars, ≤20 páginas, resultado `slice(0, 256)` (callback.ts:134-173). Code-graph: plantilla sin LLM (callback.ts:179-187).

## 4. Funcionalidades / Endpoints (SOLO reales)

**KS — todos los endpoints exigen header `x-tdai-service-id`** (multi-tenancy R1/R5, api-helpers.ts:12) y responden con `ApiResponseEnvelope {code,message,request_id,data}` (api-helpers.ts:73-78). Todo POST salvo health y auto-sync/status.

| Mount | Endpoints POST |
|---|---|
| `/v3/wiki` | get, ingest, delete, update-meta, create, list, raw/{ls,read,write,rm}, page/{ls,read,write,rm}, graph, search (routes/wiki.ts) — **ingest NO acepta repoUrl**: requiere wiki creada + `raw/*` subidos; sin fuentes → 400 "wiki has no source files, upload before ingest" (l.82-85) |
| `/v3/code-graph` | create, list, get, update-meta, sync, delete, search, explore, callers, callees, impact, node, status, files (routes/code-graph.ts) — `repo_url` en create; sync acepta repo, no ingest |
| `/v3/tools` | list (descubre 16 tools: 7 wiki + 9 code con `get_info`), call (whitelist read-only; 403 tool desconocido) (routes/tools.ts:48-178) |
| `/v3/internal/llm-binding` | set, status, list (sin header en list) (routes/llm-binding.ts) |
| `/v3/auto-sync` | GET status, POST trigger (routes/auto-sync.ts) |

**MCP (stdio, mcp/server.ts:1-11) — 12 tools query-only (mcp/tools.ts:25-223)**: 8 code_* (`code_search, code_explore, code_callers, code_callees, code_impact, code_node, code_status, code_files`) + 4 wiki_* (`wiki_search, wiki_read, wiki_list, wiki_graph`). NO hay `wiki_get`, `graph_*`, `memory_recall`, `memory_search`.

**Panel (`/api/v1`)** (app.ts:15; conocimiento en knowledge/index.ts): `/knowledge/{allocate,unbind,agent-fixed,set-visibility,grant}` (allocate-routes.ts:53-242); `/knowledge/wiki/*` 14 (list, create, ingest, get, delete, graph, page/{ls,read,rm}, search, raw/{ls,read,rm,write}); `/knowledge/code-graph/*` 8 (list, create, register-meta, get, sync, delete, search, explore); `/knowledge/{wiki,code-graph}/team-assets` (list-routes.ts:62-63); `/knowledge/status-callback` S2S sin user-key (callback-routes.ts:117). Chat-memory: `/chat-memory/{allocate,unbind,mine,create,import,team-assets,agent-fixed,my-agents,patch-scope,set-agent-fixed,layer,clear,layer-delete,layer-update,search}`. Otros: `/agent/delete-cascade` (agent-lifecycle.ts:92), `/agent-overview/bootstrap` (agent-overview.ts:159). El "15+13" corresponde al KS (wiki 16 handlers, code-graph 14), no al Panel.

**Llaves (NO "mint" en KS)**: Panel→Gateway `POST /v3/meta/user-key/create` (usuario fijo `knowledge-service`) y push a KS `POST /v3/internal/llm-binding/set` (ensure-knowledge-llm-binding.ts:141-158). Gateway es `/v3/meta/*`; KS es `/v3/internal/llm-binding/*`.

**Capa file wiki**: raw/* en `raw/sources/` (no gatilla ingest); page/* en `wiki/` con frontmatter `locked: true` inyectado (wiki-service.ts:722-823, `injectLockedTrue` l.1164-1183) y borrado con cascada `cascadeDeleteWikiPagesWithRefs` (l.827-859).

**Telemetría**: middleware ClickHouse solo en `POST /tools/call` (server.ts:65; clickhouse-telemetry.ts:12-31), tabla `tool_call_logs` (config.ts:139).

## 5. Código clave (fragmentos leídos literalmente)

```ts
// module.ts:35 — límite global LLM (default 5, env KNOWLEDGE_LLM_GLOBAL_CONCURRENCY)
export const globalLlmLimit = pLimit(getGlobalLlmConcurrency());

// llm-binding-store.ts:159-171 — mode=proxy → baseUrl con /proxy/{service_id}/v1
if (binding.mode === "proxy") {
  if (!binding.proxy_base_url || !binding.api_key) return globalDefault();
  return { ... model: fallback.model, baseUrl: `${trimTrailingSlash(binding.proxy_base_url)}/proxy/${serviceId}/v1`, ... };
}

// wiki-service.ts:1020-1026 — estado processing + run_id (re-ingest resetea)
this.store.updateWikiStatus(serviceId, wikiId, { status: "processing", internal_status: "scanning", sync_error: null });
const ingestRunId = randomUUID();

// callback.ts:60 — URL del status callback S2S
const url = `${config.tmcCallbackUrl.replace(/\/$/, "")}/api/v1/knowledge/status-callback`;

// manager.ts:394-411 — transacción de reconstrucción de índices
db.prepare("DELETE FROM wiki_fts").run(); /* page_meta, graph_edge */
const insFts = db.prepare("INSERT INTO wiki_fts(page_id, title_tok, content_tok) VALUES (?,?,?)");

// git-fetcher.ts:59-63 — https-only
if (!sourceUrl.startsWith("https://")) { throw new Error("first version only supports public HTTPS repos; SSH/private repo support coming soon"); }
```

## 6. Integración en VantaDB

- **Sustrato graphrag ya existe** en VantaDB; este repo valida el patrón "index + graph + tools query-only". Exponer los 12 tools MCP (`code_*`/`wiki_*`) por MCP es el modelo correcto; no copiar KS+Panel separados.
- `deploy/panel-knowledge-combined` (imagen `agentmemory/memory-hub`, README.md:3-8) refuerza: Panel (8125) + KS (8424) en UN contenedor; `TMC_CALLBACK_URL=http://127.0.0.1:8125`; `LLM_MODE=proxy|custom` (README:91); puertos globales `MEMORY_CORE_PORT=8420, PANEL_PORT=8125, KNOWLEDGE_PORT=8424, PROXY_PORT=8096` (deploy/global-images/.env.example:48-51); KS local PORT=8421, Panel local 8123.
- **Ingest wiki = 1 servicio, no 2**: un solo worker (`realWikiWorker`, module.ts:174-202) orquesta extract+commit+index; no hay pipeline de servicios separados.
- Patrones a reutilizar: callback S2S con `run_id` para rechazar paquetes tardíos; `locked:true` para páginas gestionadas; SSRF blocklist en fetch de repos.

## 7. Riesgos / limitaciones / NO copiar

- **NO copiar** la versión previa de este doc: rutas `/v3/knowledge/*`, `wiki/{fetcher,extractor,committer,state-machine,progress}.ts`, "mint" como endpoint KS, 4 SDK clients, `sdk/memory-consumer` NO EXISTEN. Esta es la versión verificada.
- Dependencia externa `@colbymchenry/codegraph` con platform packages (bridge.ts:25-39) — requiere instalación con pnpm y falla si falta el pkg de plataforma.
- SSRF desactivable por env (`KNOWLEDGE_SSRF_CHECK=off`) — no propagar desactivado.
- `LLM_MODE=proxy` sin binding falla loud (por diseño); en VantaDB requiere el binding previo.
- KS confía en red interna (sin auth en rutas internas, llm-binding.ts:15-16); panel añade user-key.

## RESULTADO
- Estado: ✅ CORREGIDO Y VERIFICADO
- Archivo: docs/research/tdam/08-knowledge-panel-sdk.md
- Hallazgo principal: el valor está en el **patrón ingest (chunker → LLM concurrente entre archivos → merge serial → índice FTS5+graph_edge) + state machine + 12 tools MCP query-only**; VantaDB ya tiene el motor de grafo — falta la capa de ingest wiki y exponer los tools por MCP; un solo worker orquesta todo (1 servicio, no 2).
- Ref clave real: `MemoryKnowledge/src/engines/wiki/ingest-v2/index.ts:211-283` (commitCandidates serial), `engines/wiki/manager.ts:394-412` (writeIndex transaccional), `mcp/tools.ts:25-223` (12 tools), `store/llm-binding-store.ts:146-185` (routing proxy/byo), `source-fetcher/git-fetcher.ts:59-63` (https-only + SSRF)