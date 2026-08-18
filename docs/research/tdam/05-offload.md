# TDAM — 05: Context Window Management (offload) — Investigación profunda (REVISADO)

> **Fecha:** 2026-08-18 · **Agente:** vanta-research · **Scope:** `MemoryCore/src/offload/` (plugin), `MemoryCore/src/offload_server/` (servidor v2), `MemoryCore/src/offload-client/` (cliente), `MemoryCore/src/gateway/server.ts` + `services/pipeline-worker.ts` + `core/state/types.ts` (integración)
> **Fuente:** clone local `C:\Users\Eros\AppData\Local\Temp\opencode\tdam` rama `feat/server_team` @ `97f9465` (TencentCloud/TencentDB-Agent-Memory, MIT). Cada símbolo/ruta citado fue verificado con glob/grep/Read contra el clone. Commit: `97f9465`.

## 1. Resumen ejecutivo

Sistema **dual** de gestión de ventana de contexto: (a) **plugin OpenClaw** en `offload/` con hooks + compresión L3 local y (b) **servidor standalone v2** en `offload_server/` (PipelineWorker + locks distribuidos + tareas async) consumido por `offload-client/`. Cadena LLM **L1** (resume tool_pairs → `OffloadEntry`), **L1.5** (juicio de tarea → MMD activo), **L2** (genera/actualiza MMD Mermaid + `node_mapping`), **L4** (skills, solo v1); compresión local **mild/aggressive/emergency** sin LLM. El plugin ya **no genera L2 localmente**: "Local runL2Pipeline removed — all L2 processing goes through backend" (`offload/pipelines/l2-mermaid.ts:285`). **El backend Go v1 (internal/handler/store.go) NO existe en el repo: 0 archivos `.go` en todo el historial git.**

## 2. Arquitectura y flujo (diagrama ASCII real)

```
Plugin (MemoryCore/src/offload)                Servidor v2 (offload_server/)         Cliente (offload-client/)
┌──────────────────────────────────┐           ┌──────────────────────────────┐      ┌──────────────────────────┐
│ index.ts registerOffload()       │           │ gateway/server.ts            │      │ context-engine.ts         │
│ SessionRegistry (LRU 20) +       │           │  handleOffloadV2Route :1009  │      │ OffloadContextEngine :65  │
│ OffloadContextEngine             │           │  PipelineWorker :1844        │      │ assemble() :445           │
│ hooks: llm-output → shouldForceL1│           │  executors :2873-2929        │      │  ratio<0.5 skip :477      │
│        llm-input-l3 → L3 mild/   │  POST /v2/offload/ingest ──►            │      │  POST /v2/offload/compact │
│        aggressive/emergency      │  ──────► ingest-handler.ts (encola)     │      │  fallback localCompact    │
│        after-tool-call,          │           offload-task-executor.ts      │      │  :514                     │
│        before-prompt-build       │           (L1 rename-claim + dedup,     │      │ POST /v2/offload/query-mmd│
│ L2 scheduler poll 5s :946        │            L1.5 CAS, L2 con locks)      │      │  → {mmds[], currentMmd}   │
│ BackendClient /offload/v1/*      │           compact/compaction-handler.ts │      │ TOOL_RESULT_TRUNCATE=2000 │
└──────────────────────────────────┘           └──────────────────────────────┘      └──────────────────────────┘
```
Layout server: `offload/<sanitizedSessionId>/{pending.jsonl, pending-processing-<id>.jsonl, entries.jsonl, node-mapping.jsonl, recent-context.txt, state.json, compact-state.json, mmds/, refs/}` (`offload_server/session-utils.ts:17-26`). Layout plugin: `~/.openclaw/context-offload/<agent>/{offload-<session>.jsonl, refs/, mmds/, state.json, sessions-registry.json}` (`offload/storage.ts:38-53`).

## 3. Lógica y algoritmos (refs verificadas)

- **OffloadEntry** `offload/types.ts:13-30` (plugin) / `offload_server/types.ts:20-29` (server, con `result_ref`). **PluginState** con cursor `lastOffloadedToolCallId` `offload/types.ts:44-57` (L54).
- **L1** (server): claim atómico `pending.jsonl` → rename a `pending-processing-{task.id}.jsonl` (`offload-task-executor.ts:80-90`); **dedup** contra `entries.jsonl` por `tool_call_id` (`:107-127`); fallback `"[L1 parse incomplete]"` score 2 (`:193-200`); **refs solo si result ≥20 chars** (`:204-219`); timestamps originales sobreescritos (`:185-189`).
- **L1.5** (server): fase snapshot sin lock + fase write con **lock corto** `acquireLock(lockKey, lockOwner=task.id, 10_000)` (`:380-391`); **CAS** `boundary.targetMmd !== "_pending"` (`:398-408`); backfill `state.boundaries[i].targetMmd = state.activeMmdFile` (`:414`); `handleTaskTransition` `task-transition.ts:12-35`; filenames `${seq}-${label}.mmd` (`:41-60`). JSON de salida `{taskCompleted, isLongTask, isContinuation, continuationMmdFile, newTaskLabel}` `offload_server/prompts/l15-prompt.ts:16-22`.
- **L2** (server): entradas con `node_id` null filtradas por boundary `targetMmd` (`offload-task-executor.ts:506-511`, `getEffectiveNodeId :742-744`); **guard: entrada >10 min → node_id fallback `${mmd}-orphan`** (`:515-533`); resultado `{file_action: write|replace, node_mapping}` aplicado a `mmds/*.mmd` (`:699-721`) y backfill a `node-mapping.jsonl` (`:615-623`). Prompt: `flowchart TD`, `node_mapping {"tool_call_id":"001-N1"}`, **presupuesto 4000 chars** (`offload_server/prompts/l2-prompt.ts:7,27,45-51,85-91`).
- **L3 plugin mild**: cascade MIN=10 / INITIAL=7 / FLOOR=1 (`offload/hooks/llm-input-l3.ts:113-115`), `compressByScoreCascade :402`, **skip si summaryLength > originalLength** (`:530-538`), `replaceAssistantToolUseWithSummary` (`offload/l3-helpers.ts:184`).
- **L3 aggressive (one-shot)**: `aggressiveCompressUntilBelowThreshold` (`llm-input-l3.ts:667-678`); boundary por fingerprint `_lastAggressiveBoundary {originalIndex, fingerprint, keptMsgCount, remainingTokens}` (`state-manager.ts:96-101`); fingerprint real `role + first 200 chars` (`index.ts:121-129`), re-aplicado en assemble (`index.ts:1484-1520`).
- **L3 emergency**: `emergencyCompress :755`, `_emergencyTailDelete :848`, `_emergencyTruncateOversized :968` (trunca a ~2000 chars, `:121`).
- **MMD injector**: ACTIVE solo; HISTORY solo post-aggressive (`offload/mmd-injector.ts:28-31`); `adjustForToolCallPair` sin partir pares tool_call/tool_result (`:231,:243`); marker `_mmdContextMessage` (`:20`); dedup fingerprint `${content.length}:${content.slice(0,64)}` (`:372-374`).
- **L2 plugin scheduler**: poll `setTimeout(tick, 5000)` (`index.ts:946`), `checkL2Trigger` (`pipelines/l2-mermaid.ts:96-218`: null_count threshold | timeout | retry-wait).
- **shouldForceL1**: pending ≥ 4 (`offload/hooks/llm-output.ts:10,16-22`).
- **Compaction server**: `compact-state.json` independiente (sin lock) (`compact/compaction-handler.ts:226-229`); **fast-path re-aplica confirmed/deletedOffloadIds** (`compact/fast-path.ts:28-39`); `resolveLevel` por ratio (`compact/compressor.ts:115`, `compaction-handler.ts:142-147`); cadena mild→aggressive→emergency (`:153-220`); inyección MMDs históricos = 10% del context window (`:193-208`); calibración tokens: drift >15% → factor 0.5–3.0 (`:267-270`).
- **Reclaimer**: 5 min inicial + 24 h (`offload/index.ts:1280-1304`); `retentionDays < 3` desactiva (`reclaimer.ts:75-78`); 5 pasos por mtime (jsonl/refs/mmds/logs/registry) (`:4-14`).

## 4. Funcionalidades/Endpoints (reales)

- **BackendClient v1** (`offload/backend-client.ts`): `POST /offload/v1/l1/summarize :140`, `/l15/judge :168`, `/l2/generate :196`, `/l4/generate :225`, `/store :256` (X-User-Id/X-Task-Id → Mongo; backend Go externo **no está en el repo**).
- **Server v2** (`offload_server/router.ts:35-80`, wire en `gateway/server.ts:1009`): auth Bearer → 401 (`router.ts:39-41`); `POST /v2/offload/ingest` — body `{session_id, tool_pairs[], prompt?, recent_messages?}` (`schemas.ts:29-41`) → **encola tareas async (NO pipeline síncrono)** y agenda L2 1s/30s (`offload-task-executor.ts:245-262,437-453`); filtros session `memory-.*-session-\d+`/`subagent` (`ingest-handler.ts:93-94`) y prompt `[Inter-session message]`/`Pre-compaction` (`:191-196`); `POST /v2/offload/query-mmd` — `{session_id, limit?}` → `{mmds[], currentMmd}` (`mmd-handler.ts:79-89`); `POST /v2/offload/compact` — `{session_id, messages, ratio, context_window, total_tokens, message_tokens?}` → `{messages, report}` (`schemas.ts:66-73`, `compaction-handler.ts:254`).
- **TaskPayload**: `offload-l1 | offload-l15 | offload-l2 | L1 | L2 | L3 | flush` (`core/state/types.ts:61`); locks por MMD `pipeline:{instanceId}:offload-l2:{mmdFile}` (`services/pipeline-worker.ts:688-697`); timers `setTimerIfEarlier` (`ingest-handler.ts:164`, `offload-task-executor.ts:248,439,603,632`); credit limits `quotaManager.checkCreditQuota` (`gateway/server.ts:2704-2711,2766+`).
- **Cliente**: timeouts ingest 5000 ms / compaction 30000 ms / health 5000 ms (`offload-client/types.ts:19-22,32-33`, `offload-api-client.ts:12-22`); header `X-TDAI-Service-Id` (`offload-api-client.ts:174`); tokens `o200k_base` (`token-estimator.ts:18`) — `cl100k_base` solo como default del plugin (`offload/types.ts:246-247`).

## 5. Código clave (fragmentos literales verificados)

```ts
// offload_server/session-utils.ts:17-19
export function sanitizeSessionId(sessionId: string): string {
  return sessionId.replace(/[^a-zA-Z0-9._\-]/g, "_");
}
```
```ts
// offload_server/offload-task-executor.ts:397-408 (CAS L1.5)
const boundaryIdx = state.boundaries.findIndex((b) => b.timestamp === boundaryTimestamp);
if (boundaryIdx < 0) { /* skip */ }
if (state.boundaries[boundaryIdx].targetMmd !== "_pending") { /* already backfilled, skip */ }
```
```ts
// offload_server/offload-task-executor.ts:517-533 (guard L2)
const L2_MAX_AGE_MS = 10 * 60 * 1000;
if (Date.now() - oldestTs > L2_MAX_AGE_MS) { /* fallback node_id: `${targetMmdFile!.replace(/\.mmd$/, "")}-orphan` */ }
```
```ts
// offload/mmd-injector.ts:372-374 (dedup fingerprint)
function computeFingerprint(content: string): string {
  return `${content.length}:${content.slice(0, 64)}`;
}
```
```ts
// offload/hooks/llm-output.ts:16-22 (shouldForceL1)
return stateManager.getPendingCount() >= (pluginConfig?.forceTriggerThreshold ?? 4);
```

## 6. Integración en VantaDB (vanta-memory)

- **ContextEngine.assemble + MMD**: `offload-client/context-engine.ts:445-520` (estimar ratio → POST compact → fallback local) + inyección `<current_task_context>` con el MMD activo (`offload/mmd-injector.ts:348-361`).
- **Cursor core**: `PluginState.lastOffloadedToolCallId` (`offload/types.ts:54`) como marca de hasta dónde L3 ya resumió.
- **NO copiar la doble implementación**: elegir **un** modo — plugin (hooks + L3 local, L1–L2 vía backend v1) **o** servidor v2 (ingest async + PipelineWorker + compact). El plugin hoy es backend-only para L2 (no genera MMD local).

## 7. Riesgos / limitaciones / NO copiar

- **La versión previa de este doc citaba símbolos fabricados** (`boundedCompress`, `TaskStatus` en offload, `mergeNodes`, `withLock`, `resetOnNewSession`, `resultAlreadyComputed`, `offload-server/` con guion, `tasks/l15|l2|l4-task-executor.ts`, `offload-mmd.ts`): verificado — **no existen** en `src/` (grep global). `TaskStatus` solo existe en `metadata/types.ts` (no offload). Esta es la versión verificada.
- **Backend Go v1 ausente**: `POST /offload/v1/*` requiere un servicio Go externo no incluido en el repo (0 `.go`); no implementable desde este código.
- **`offload_server/state/` no existe**; el estado vive en `state.json` + `compact-state.json` dentro del layout por sesión.
- Confianza de refs: alta (leídas literalmente del clone); cambios de línea posibles si el commit difiere.

## RESULTADO
- Estado: ✅ CORREGIDO Y VERIFICADO
- Archivo: docs/research/tdam/05-offload.md
- Hallazgo principal: la "killer feature" es el **swap compresión + re-inyección de MMDs** con compresión local LLM-free (mild/aggressive/emergency) — patrón portátil `ContextEngine.assemble(msgs, ratio) → {messages, report}`; L1/L1.5/L2 (LLM) generan las entradas que alimentan el swap; el plugin es backend-only para L2.
- Ref clave real: `offload/hooks/llm-input-l3.ts:402` (mild cascade), `:667` (aggressive one-shot), `offload-client/context-engine.ts:445` (assemble), `offload_server/offload-task-executor.ts:397-408` (CAS L1.5)