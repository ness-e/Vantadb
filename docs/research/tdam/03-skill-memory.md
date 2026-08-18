# TDAM — 03: Skill Memory — Investigación profunda

> Fecha: 2026-08-18 · Agente: vanta-research · Scope: core/skill (+ conversation-add, prompts, queue) — handlers HTTP en `src/gateway/skill-handlers.ts` (no core/skill/) · Stack: **TypeScript/Node** (no Rust)
> Fuente: clone local completo `C:\Users\Eros\AppData\Local\Temp\opencode\tdam` (rama `feat/server_team`)

## 1. Resumen ejecutivo

TDAM implementa un sistema de "skills" (SKILL.md + recursos) con **snapshot inmutables multi-versión** en una sola tabla SQLite (`skills`), búsqueda **BM25 (FTS5 + jieba) con embedding vec0 opcional**, extracción automática desde conversaciones vía **LLM tool-calling review agent** (asíncrono, cola por agente en Redis), y un **fast-path** de coincidencia por nombre que está diseñado pero **NO activado**. El módulo es host-neutral (`types.ts:4`), con split de 4 servicios: buffer (COS) → trigger → worker pool → sink. El patrón más valioso para VantaDB: versión inmutable + optimistic lock + TTL de versiones viejas + dedup por nombre en el store.

## 2. Arquitectura y flujo

```
[Gateway /v3/skill/*] → SkillCore (facade, skill-core.ts)
   ├─ store: ISkillStore → SqliteSkillStore | TcvdbSkillStore (skill-store.interface.ts:57)
   ├─ resources: SkillResourceStore (blobs, skill-resource-store.ts)
   └─ versioning: SkillVersioning (transacción COS+DB+assets, skill-versioning.ts)

Extracción desde conversación (conversation-add/):
POST /v3/skill/conversation/add
  → SkillConversationAddHandler (buffer acumula msgs/sesión, add-handler.ts)
  → umbral (tool_call≥10 | bytes≥40KB | compressed | oversize)
  → SkillTriggerService.archive (trigger-service.ts)
      1. escribe archive JSON en COS  (PRIMERO, evita ghost tasks)
      2. mutex → append _tasks.json → Redis enqueue agent  (misma critical section)
  → SkillWorkerPool (N workers, worker-pool.ts)
      → peek agent (LMOVE at-least-once) → extract-lock (10min, renovable)
      → read archive → SkillExtractor (LLM review agent con tools)
      → SkillCoreSink.applyCandidates (solo re-registro de asset; skill ya lo creó el LLM)
  → DLQ _tasks_dlq.json tras 3 fallos permanentes (extract-worker.ts:573-643)
```

## 3. Lógica y algoritmos (refs)

1. **Modelo**: `Skill` = fila (skill_id, version) inmutable + `is_head` (types.ts:264-286). Identidad 5-tupla: user/owner_agent/team/task/skill_id. DDL: `SKILLS_DDL` (skill-store-ddl.ts:21-65), índice único parcial `(team_id, owner_agent_id, name) WHERE is_head=1 AND status='active'` (línea 48-49).
2. **CRUD**: `SkillCore` (skill-core.ts:252-648) — create/update/patch/delete/writeFiles/removeFiles/get/list/search/listVersions/readFile/exportSkill. **delete = borrado físico** de todas las versiones (skill-core.ts:374-398, decisión 2026-07, no soft).
3. **Versionado**: `SkillVersioning.appendNextVersion` (skill-versioning.ts:212-302): verifica hash de contenido (idempotente si no cambia nada), copia árbol de recursos `copyTree` a `v<version+1>`, aplica cambios, escribe DB en transacción. Optimistic lock: `expected_version` obligatorio (skill-permission.ts:61-67). TTL: `cleanupExpiredVersionsForSkill` conserva head + 3 recientes (skill-versioning.ts:382-430). Nombre inmutable entre versiones.
4. **Blobs**: `SkillResourceStore` (skill-resource-store.ts): layout `skills/<skill_id>/v<N>/files/<path>`; límites 5MB/recurso y 50MB/skill total (líneas 14-15, assertTotalSize 82-104); anti path-traversal (205-216); mime por extensión; `is_executable` en metadata del objeto.
5. **RAG search**: `SqliteSkillStore.searchSkills` (skill-store.ts:557-641) — FTS5 con `unicode61 remove_diacritics` + pre-tokenización jieba para CJK (`tokenizeForFts`), snippet con `<mark>`, filtro por IDs en FTS, re-verificación `is_head=1 AND status='active'` en main table. `skill_vec` vec0 (DDL template skill-store-ddl.ts:92-97). **SQLite solo implementa BM25**: mode embedding/hybrid degrada a BM25 con warn (skill-store.ts:561-573). TCVDB sí tiene native hybridSearch (`getCapabilities().nativeHybridSearch` en tcvdb-skill-store.ts:164; método `hybridSearch` en :783).
6. **Routing**: no hay router separado — la selección = búsqueda (search/list) + inyección de `<available_skills>` en system prompt (`SKILL_LISTING_HEADER`, skill-listing-prompt.ts:11-26) limitada por `charBudgetPercent`/`searchTopK` (skill-config.ts:230-235).
7. **Extracción desde conversación**: `SkillExtractor.extract` (skill-extractor.ts:128-311) — transcript con marcadores `<<past-user>>` anti role-capture (líneas 424-427), truncado head 8k/tail 32k chars, pre-carga de skills propios (full/relevant/recent, líneas 157-195) para evitar duplicados, y prompt `SKILL_REVIEW_PROMPT` (skill-review-prompt.ts:40-198) con taxonomía SOP/Background/Preference y contrato de salida estricto ("Nothing to save." o tool calls).
8. **Fast-path**: `nameMatchFastPath` (skill-fast-path.ts:34-44) — substring match name∈query, min length 4. **DECISION explícita de NO integrarlo** (líneas 16-24): costo de traer 1000+ skills anula el beneficio.
9. **Permiso LLM**: `createSkillTools` (skill-tools.ts:54-217) expone SOLO skill_list/view/create/update/patch/files_write — **sin delete ni files_remove**. Todos los writes exigen `expected_version`; owner check `assertOwner` (team+agent); errores devueltos como JSON para que el LLM se auto-corrija.
10. **Cola**: `RedisSkillAgentTaskQueue` (agent-task-queue.ts:509-776) — List+Set por agente, `peekAgent` at-least-once con LMOVE/EVALSHA/fallback, `withTasksMutex` (SET NX PX) protege `_tasks.json`, `extract-lock` 10min renovable. Clasificación transient/permanent + DLQ (extract-worker.ts:653-696).

## 4. Funcionalidades / Endpoints

| Endpoint | Handler (gateway/skill-handlers.ts:1140-1156) | Acción |
|---|---|---|
| POST /v3/skill/create | handleCreate | create v1 (+ ensureSkillAsset) |
| POST /v3/skill/update | handleUpdate | reemplazo total SKILL.md |
| POST /v3/skill/patch | handlePatch | string replace único/all |
| POST /v3/skill/delete | handleDelete | borrado físico |
| POST /v3/skill/get / get-by-name | handleGet(ByName) | detalle (head o versión) |
| POST /v3/skill/list | handleList | head rows + filtros |
| POST /v3/skill/search | handleSearch | BM25/hybrid |
| POST /v3/skill/versions | handleVersions | historial + is_expired |
| POST /v3/skill/files/write,remove,read | handleFiles* | recursos blob |
| POST /v3/skill/export | handleExport | zip SKILL.md+files |
| POST /v3/skill/listing | handleListing | inyección prompt agentes |
| POST /v3/skill/extract | handleExtract | trigger directo |
| POST /v3/skill/conversation/add | handleConversationAdd | acumulador de sesión |
| POST /v3/skill/conversation/force-archive | handleForceArchive | archivo forzado |

## 5. Código clave

```ts
// skill-store.ts:330-366 — escritura transaccional: head viejo → 0, insert nuevo, fts sync
this.db.exec("BEGIN IMMEDIATE");
if (head) this.db.prepare("UPDATE skills SET is_head=0 WHERE skill_id=? AND version=?").run(...);
// INSERT ... row_id=ulid(), version=head?head.version+1:1, is_head=1 ...
this.db.prepare("DELETE FROM skill_fts WHERE skill_id=?").run(input.skill_id);
// INSERT INTO skill_fts ... tokenizeForFts(name|description|content≤4000)
this.db.exec("COMMIT");
```

```ts
// skill-versioning.ts:222-228 — idempotencia sin escribir
if (noContentChange && noResourceChange) { return head; }
```

- **Frontmatter SKILL.md** — `skill-format.ts`: `name` regex `^[a-z0-9][a-z0-9-]*$` (1-64 chars), `description` ≤1024 chars, body ≤50k chars; violación → 400 sin escribir (contrato de formato estricto).

```ts
// skill-extractor.ts:425-426 — anti role-capture
const body = messages.map((m) => `<<past-${m.role}>>\n${m.content}`).join("\n\n");
return `${body}\n\n<<end-of-transcript>>\nAbove is the past conversation to review. ...`;
```

## 6. Integración en VantaDB

- **Core (LLM-free)** — copiar: tabla `skills` multi-versión con `(skill_id, version)` UNIQUE + `is_head`, índice único parcial por (owner, name), `expected_version` optimistic lock, TTL de versiones con keep-recent=3, hash de contenido para idempotencia, `SkillResourceStore` (path validation + límites), búsqueda FTS5. En VantaDB Rust core: reemplazar SQLite FTS5/jieba por el BM25+tokenizer existente y **HNSW embedding propio** (no vec0); esquema como namespace genérico `skills`.
- **vanta-memory (LLM-driven)** — copiar: `SkillExtractor` (transcript con marcadores anti role-capture + truncado head/tail + pre-carga de skills para dedup), `SKILL_REVIEW_PROMPT` (taxonomía SOP/Background/Preference, contrato de salida estricto), tool set del review agent (sin delete), sink idempotente.
- **MCP server** — exponer los 6 tools de `createSkillTools` (skill_list/view/create/update/patch/files_write) a agentes como tools MCP; NO copiar el split handler→trigger→queue→worker (Redis) de TDAM — en VantaDB el pipeline de extracción puede ser síncrono o una cola simple del core.

## 7. Riesgos / limitaciones / qué NO copiar

- **NO copiar**: los 4 servicios (buffer COS + trigger + Redis queue + worker pool + DLQ + mutex + extract-lock) — ~2.600 líneas de orquestación distribuida para un beneficio que en VantaDB logra un job síncrono o una cola local. También NO: `_tasks.json`/archives en object storage, ghost-task recovery, sink/asset registry (acopla a meta_assets), TCVDB split.
- **SQLite search está incompleto**: embedding/hybrid degrada a BM25 (skill-store.ts:561-573); el vec0 KNN real no está implementado — no asumir que copiando el DDL obtienes RAG híbrido.
- **jieba tokenization** es obligatoria para CJK en FTS5 (unicode61 no segmenta chino); si VantaDB no segmenta CJK, el BM25 en español/inglés funciona igual.
- **copyTree por cada versión** duplica todos los blobs del skill; con recursos grandes es caro — preferir content-addressed storage en VantaDB.
- **delete físico** con reporte -N a un counter (shark) es patrón TDAM; en VantaDB el contador/telemetry debe adaptarse.
- `fast-path` está desactivado por decisión de coste (skill-fast-path.ts:16-24) — no copiarlo como feature activa.
- Permisos: `assertTeamMatch` mapea a 404 para no filtrar existencia (skill-permission.ts:51-55) — buen patrón de seguridad a conservar.

## RESULTADO
- Estado: ✅ COMPLETO
- Archivo: docs/research/tdam/03-skill-memory.md
- Hallazgo principal: Skills = snapshots inmutables multi-versión en tabla única con optimistic lock + TTL + BM25/jieba, y extracción automática vía LLM review agent con tools (sin delete) — el resto (cola Redis, assets, fast-path) es deuda no portable.
- Ref clave: `MemoryCore/src/core/skill/skill-store.ts:330-366` (transacción appendVersion: head→0 + insert + FTS sync)