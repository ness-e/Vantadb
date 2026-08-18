# TDAM — 07: MemoryProxy — Investigación profunda (REVISADO)

> **Fecha:** 2026-08-18 · **Agente:** vanta-research · **Scope:** `MemoryProxy/**` — proxy LLM transparente en TypeScript (~160 archivos, `MemoryProxy/src/`)
> **Rama / commit:** `feat/server_team` @ `97f9465` · **Fuente:** clone local `C:\Users\Eros\AppData\Local\Temp\opencode\tdam`
> ⚠️ **Nota metodológica:** la versión previa de este documento citaba rutas y fragmentos que **no existen** (`app.ts`, `session/manager.ts`, `context/engine.ts`, `middleware/rate-limit.ts`, `auth/api-key-bridge.ts`, `writeback/writeback.ts`, `reporting/report-usage.ts`, `quota-bridge.ts`, `routes/chat-completions.ts`, `agents/deepseek-harness.ts`). Todo lo siguiente fue verificado con glob/grep/Read contra el clone. Nada se inventa.

## 1. Resumen ejecutivo

MemoryProxy es un **proxy LLM de transporte transparente**: no cambia protocolo y reenvía `OpenAI /v1/chat/completions` y `Anthropic /v1/messages` tal cual, haciendo trabajo extra a la entrada/salida (README.md:5). Se ejecuta con **Node v22 estricto** (`src/index.ts:3-10`), **Hono + @hono/node-server** (`package.json:21,29`; `src/server.ts:3,19`; `src/index.ts:12`) en el **puerto 8096** (`config.example.yaml:26`).

Soporta **tres protocolos wire**: OpenAI Chat Completions, Anthropic Messages y **OpenAI Responses API** (Codex/WorkBuddy: `/v1/responses`, `/realtime/calls`, `/memories/trace_summarize` — `server.ts:221-245`), más auxiliares. Ciclo real por request: **auth → systemUser → sessionInit → injection → rateLimit → forward → extract → report** (README.md:44-53).

**No persiste memoria propia**: todos los reads/writes de Memory/Skill/Knowledge van al Gateway MemoryCore (default `:8420`) (`README.md:7,19`). Matiz: sí persiste *estado de sesión/caches* (inyección, skills) vía **ProxyStorage** con 5 backends — Redis, COS (kernel-sts), SQLite, FS, Memory (`README.md:33`; `storage/factory.ts`).

## 2. Arquitectura y flujo

```
Cliente (CC / CodeBuddy / Codex / WorkBuddy / dsh)   ← protocolo intacto
        │  POST /:agent/:spaceId/v1/messages | /v1/chat/completions | /v1/responses
        ▼
   MemoryProxy :8096  (Hono; createApp en src/server.ts)
   ┌─────────────────────────────────────────────────────────────────────┐
   │ src/index.ts            bootstrap, Node v22 gate, graceful shutdown │
   │ src/server.ts           fábrica de rutas Hono (331 líneas)          │
   │ src/handler.ts          OpenAI Chat Completions (2076 líneas)       │
   │ src/anthropicHandler.ts Anthropic Messages (+retry/fallback modelo) │
   │ src/codexHandler.ts / workbuddyHandler.ts  Responses API            │
   │ src/auxiliaryHandler.ts count_tokens|embeddings|completions|moderat.│
   │ src/auth.ts             verify user_key → user_id                   │
   │ src/systemUser.ts + systemUserPassthrough.ts  short-circuit interno │
   │ src/session/            session-key, store, form, extractor (+cc/cb)│
   │ src/meta/client.ts      MetadataClient /v3/meta/* (state machine)   │
   │ src/injection/          HookRegistry + InjectionPipeline + injectors│
   │ src/agent-adapters/     claude-code|codebuddy|codex|workbuddy|dsh   │
   │ src/tdai/               client, recorder, pending-writes            │
   │ src/skill/ core-client.ts, skill-bridge.ts, handler-glue.ts         │
   │ src/knowledge/ core-client.ts            src/memory/ memory-bridge  │
   │ src/mem-command/        mem:sync|create-skill|help                  │
   │ src/rate-limit/         guard + redis-store (Lua sliding window)    │
   │ src/report/ log; src/opik.ts langfuse.ts clickhouse.ts credit-…    │
   │ src/storage/            ProxyStorage 5 backends; src/db/ repos      │
   │ src/routes/             admin-auth, rate-limits, instance-destroy…  │
   └─────────────────────────────────────────────────────────────────────┘
        │  forward (timeout 600s)                │  HTTP API
        ▼                                        ▼
   Upstream LLM (TokenHub/OpenAI-compatible)   MemoryCore Gateway :8420
```

Sin middleware global `app.use` (excepto dos gates 404 de markers, `server.ts:35-76`); todo el trabajo es in-handler.

## 3. Lógica y algoritmos (verificado)

- **Session — REAL:** NO hay hash(agent+spaceId), NO `POST /v3/session/init`, NO LRU. `sessionKey` sale de headers (`x-conversation-id`/`x-session-id`/`x-claude-code-session-id`/`x-deepseek-harness-session-id`/`x-chat-id`/`x-thread-id`, `session/session-key.ts:9-19`). El init es una **state machine local form team→agent→task** que consulta `/v3/meta/team/list`, `/v3/meta/agent/list`, `/v3/meta/task/list`, `agent/get`, `task/get`, `asset/list-accessible`, `participation-log`, `agent-fixed-asset` (`meta/client.ts:220-369`). TTL **30 min solo para estados pending** (`session/store.ts:31,116`). Persistencia multi-nodo L2a (SessionRepo) + L2b (BindingRepo) + history-scan fallback; cadena `getOrRecover` L1→L2a→L2b→reconstrucción (`store.ts:269-376`).
- **Rate limit — REAL:** sliding window **60s en Redis**, bucket = **spaceId × model** (`dimensionField` = `JSON.stringify([instanceId, modelId])`, `redis-store.ts:324-326`); Lua `CHECK_REQUEST_LUA`/`RECORD_TOKENS_LUA` (`redis-store.ts:37-121`). **Fail-open sin Redis** (degraded → allow, `guard.ts:40-51`). 429 con `Retry-After` + headers `x-ratelimit-*` (`guard.ts:111-121`). Overrides runtime por instancia×model vía `/v3/admin/rate-limits`.
- **Auth — REAL:** `POST {url}/v3/meta/auth/verify` con `x-tdai-service-id` derivado del spaceId de la ruta (`auth.ts:88`), sin caché, timeout configurable 5000ms (`config.example.yaml:324`). Cualquier resultado ≠ `code=0 && valid=true && user_id` → rechazo (`auth.ts:102-111`).
- **Inyección — REAL:** HookRegistry + InjectionPipeline con injectors `tdai-tools`, `tdai-profile-memory`, `tdai-l1-recall`, `tdai-fixed-asset`, `skill-tools`, `skill`, `knowledge-tools`, `asset-reflection` (`injection/injectors/*`). L2/L3 inyectados en system prompt; L0/L1 expuestos como **tools** para evitar invalidar KV-cache (README.md:28).
- **Write-back — REAL:** fire-and-forget con retry: `trackWrite` + `withL0Retry` (3 intentos, backoff 500ms→1s→2s) sobre `recordTdaiTurn` (L0) (`handler.ts:1986-1996`; `tdai/pending-writes.ts:38-108`); skill extract es `await` sincrónico (`handler.ts:2006-2009`); flush de pendientes en SIGTERM (`index.ts:154-169`). NO hay `/v3/session/heartbeat` ni `/v3/recall/trigger` ni `recall-feedback.ts`; recall L1 = `/v3/atomic/search` expuesto como tools (`tdai/client.ts:148-150`), sin feedback loop.
- **Reporting — REAL:** Opik (project = SHA-256(apiKey) 8 chars, `opik.ts:36-39`), Langfuse (**SDK oficial** `@langfuse/tracing` + `@langfuse/otel`, trace determinista `sessionKey+turnSeq`, `langfuse.ts:6-11`), ClickHouse (uso por turno + tabla raw para auditoría, `clickhouse.ts`), credit-reporter (CreditDelta según `creditPricing.models`, `pricing.ts`; **NO** hay `/v1/models`).
- **Mem-command — REAL:** comandos `mem:sync|create-skill|help` (`mem-command/index.ts:24`), prefijo `mem:`, **deshabilitado por defecto** (`isMemCommandAllowed` exige `config.enabled`).
- **Knowledge — REAL:** `/v3/knowledge/list` (`knowledge/core-client.ts:4,82`) + tools de self-discovery `tools/list|call` (`injection/injectors/knowledge-tools-injector.ts`). NO `/v3/knowledge/query`.
- **Credenciales — REAL:** sin minting de apiKey efímero; resolución por `upstream.agents[agent]`: server-key | passthrough de la key del cliente (`handler.ts:1095-1104`).
- **Kafka — NO existe** producer; OTLP solo como base de Langfuse (`langfuse.ts` usa OpenTelemetry NodeSDK + LangfuseSpanProcessor).
- **Extras reales:** system-user passthrough (`systemUserPassthrough.ts:1-42`), `ccRequestRouting` (default `false`, `config.ts:143`), costGuard router + retry modelo fallback (`anthropicHandler.ts:467-506`), `forwardTimeoutMs` 600s (`config.ts:10`), errores upstream 502, Dockerfile + `scripts/proxy.sh` + `scripts/setup-claude-code.sh`, tests vitest (`package.json:11`).

## 4. Funcionalidades / Endpoints (solo reales, `src/server.ts`)

| Ruta | Handler | Notas |
|---|---|---|
| `GET /health` (84) | inline | 503+degraded si storage COS pedido cae a local |
| `GET /whoami` (107) | inline | apiKey → keyId SHA-256 8 chars |
| `POST /skill-bridge/*` (127) | skill-bridge | reverse-proxy de core, inyecta serviceToken |
| `POST /memory-bridge/*` (132) | memory-bridge | ídem para tdai read-only (`/v3/atomic/search`…) |
| `POST /v3/instance/proxy-destroy` (139) | routes/instance-destroy | admin |
| `GET/PUT/DELETE /v3/admin/rate-limits` (142-144) | routes/rate-limits | admin |
| `POST /v3/session/refresh-cache` (147), `/v3/session/force-archive-skill` (152) | routes/ | base de mem: |
| `POST /v1/messages` (160) | anthropicHandler | |
| `POST /v1/{messages/count_tokens,embeddings,completions,moderations}` (165-168) | auxiliaryHandler | |
| `/:agent/:spaceId/cost-guard/v1/{messages,chat/completions}` (188-191) | gated `costGuard.markerOptIn` | marker 404 si off (`server.ts:35-49`) |
| `/:agent/:spaceId/analyse/v1/{messages,chat/completions}` (202-205) | gated `assetReflection.markerOptIn` | |
| `/codex/:spaceId/{v1/,}responses{/compact}` + `/v1/memories/trace_summarize` + `/v1/realtime/calls` (221-229) | codexHandler | Responses API, con/sin `/v1` |
| `/workbuddy/:spaceId/…` (237-245) | workbuddyHandler | Responses API |
| `/:agent/:spaceId/v1/messages` (307) y `/:agent/:spaceId/v1/chat/completions` (312) | **rutas primarias** | + aux (308-311) |
| `/:agent/v1/*` (315-316) | deprecated | sin credit reporting |
| `/proxy/:spaceId/*` (320-325) | legacy | default codebuddy |
| `POST /*` (328) | catch-all → chat completions | |

spaceId vía regex `/{agent}/{spaceId}/` + legacy `/proxy/{spaceId}/` (`credit-reporter.ts:66-80`).

## 5. Código clave (fragmentos leídos literalmente)

```ts
// src/index.ts:3-10 — Node v22 estricto
if (!process.version.startsWith("v22.")) {
  console.error(`\x1b[31m[ERROR] Node.js version check failed!\x1b[0m`);
  ...
  process.exit(1);
}
```

```ts
// src/server.ts:307,312 — rutas primarias
app.post("/:agent/:spaceId/v1/messages", (c) => handleAnthropicMessages(c, config));
app.post("/:agent/:spaceId/v1/chat/completions", (c) => handleChatCompletions(c, config));
```

```ts
// src/auth.ts:88 — auth real
const url = config.url.replace(/\/+$/, "") + "/v3/meta/auth/verify";
```

```ts
// src/session/session-key.ts:9-19 — sessionKey desde headers
const id = c.req.header("x-conversation-id") ?? c.req.header("x-session-id") ??
  c.req.header("x-claude-code-session-id") ?? c.req.header("x-deepseek-harness-session-id") ??
  c.req.header("x-chat-id") ?? c.req.header("x-thread-id") ?? null;
```

```ts
// src/rate-limit/guard.ts:40-51 — fail-open sin Redis
if (decision.degraded) { ... log.warn("rate_limit.fail_open", ...); return; }
// guard.ts:111-121 — 429 Retry-After
return new Response(JSON.stringify(body), { status: 429, headers: { ..., "retry-after": ... } });
```

```ts
// src/mem-command/index.ts:24 — comandos reales
const KNOWN_COMMANDS = new Set(["sync", "create-skill", "help"]);
```

```ts
// src/tdai/client.ts:148-150 — recall L1
if (!this.isEnabled() || !this.config.recallL1 || !query.trim()) return [];
const data = await this.postForCtx<...>("/v3/atomic/search", ctx, {...});
```

```ts
// src/handler.ts:1987-1996 — write-back fire-and-forget con retry
trackWrite(
  withL0Retry(() => recordTdaiTurn(ctx.tdaiClient!, ctx.tdaiIdentity, ctx.tdaiUserMessage, ...))
    .catch((err) => pipe.error("TDAI_L0", err))
);
```

```ts
// src/handler.ts:1102-1104 — resolución de apiKey (sin minting)
const effectiveApiKey = agentUpstreamEntry ? (agentUpstreamEntry.apiKey ?? "") : config.upstream.apiKey;
```

## 6. Integración en VantaDB

Para un binario opcional `vanta-proxy`, portar de MemoryProxy: los **tres protocolos** (OpenAI Chat Completions, Anthropic Messages, Responses API), el **ciclo inject→forward→write-back** (pipeline de hooks + write-back fire-and-forget con retry y flush en shutdown), el **patrón mem-command** (prefijo + whitelist por config) y el **rate-limit Lua de ventana deslizante** (si se dispone de Redis; sino, fail-open consciente). El diseño de rutas `/:agent/:spaceId/...` con regex de spaceId y la separación auth→session→injection→forward→report son directos de replicar.

## 7. Riesgos / limitaciones / NO copiar

- **NO copiar**: adapters específicos por cliente (claude-code/codebuddy/codex/workbuddy/dsh — lógica de negocio ajena), bridges con minting de credenciales (no existen; la resolución server-key/passthrough es config-driven), producer Kafka (no existe), dependencias pesadas sin uso (`node-pty`, `better-sqlite3`, `cos-nodejs-sdk-v5`, `@langfuse/*`).
- **Limitaciones reales**: sin LRU de sesión ni hash de identidad; sin feedback loop de recall; sin `/v1/models` (modelos en tabla `creditPricing`); rate limit **fail-open** sin Redis (riesgo de abuso si cae Redis); sesión `pending` TTL 30 min; multi-nodo exige `storage.backend=cos` + `injection.externalGatewayUrl` (README.md:324).
- La versión previa con rutas/fragmentos fabricados **queda descartada**; este documento es la fuente verificada.

## RESULTADO
- Estado: ✅ CORREGIDO Y VERIFICADO
- Archivo: docs/research/tdam/07-proxy.md
- Hallazgo principal: MemoryProxy es un proxy transparente de **3 protocolos** (OpenAI Chat, Anthropic, Responses) con ciclo auth→sessionInit→injection→rateLimit→forward→extract→report; no persiste memoria propia (todo al Gateway :8420) y su única persistencia es estado de sesión/caches vía ProxyStorage (Redis/COS/SQLite/FS/Memory).
- Ref clave real: `src/server.ts:307,312` (rutas primarias); `src/session/session-key.ts:9-19` (sessionKey por headers); `src/rate-limit/guard.ts:40-51` (fail-open); `src/tdai/client.ts:148-150` (recall L1 /v3/atomic/search); `src/mem-command/index.ts:24` (mem:sync|create-skill|help); `src/auth.ts:88` (/v3/meta/auth/verify); `src/handler.ts:1987-1996` (write-back fire-and-forget)