# TDAM — 06: Metadata plane + ACL — Investigación profunda (REVISADO)

> **Fecha:** 2026-08-18 · **Agente:** vanta-research · **Scope:** `MemoryCore/src/metadata/` (router v3-meta, services, permission-checker, acl, schemas), auth layers, deployMode quota
> **Fuente:** clone local real `C:\Users\Eros\AppData\Local\Temp\opencode\tdam` (rama `feat/server_team` @ 97f9465). **REVISIÓN:** la versión previa afirmaba store MongoDB único, envelope `{code,msg,data}`, entidades y ACL con campos inexistentes y endpoints que no existen. Esta versión solo transcribe código leído (glob/grep/Read) con refs verificadas.

## 1. Resumen ejecutivo

1. **Store DUAL** — SQLite es el backend por defecto (`MemoryCore/src/metadata/store/factory.ts:95-100`); MongoDB solo si `TDAI_METADATA_MONGO_URI` está definido (`:85-92`). DB lógica `{prefix}_{instance}` = `tdai_metadata_{instance_id}` (`store/db-name.ts:7,43`). SQLite crea **12 tablas `meta_*`** (`store/sqlite-adapter.ts:128-312`). Multi-instancia: una DB por instancia + `MetadataStorePool` LRU (max 128, `factory.ts:181-293`) + purge (`dropDatabase` en Mongo, `rm` recursivo en SQLite, `factory.ts:262-280`); instanceId viene del header `x-tdai-service-id` (`router/instance.ts:28-31`).
2. **Envelope real** — `{ code, message, request_id, data }` (`gateway/v2-router.ts:332-338`): éxito `code:0, message:"ok"`; error `{code,message,request_id}` con `data` opcional. `request_id` lee `x-qcloud-transaction-id` ?? `x-request-id` (`v2-router.ts:322-330`).
3. **55 endpoints `/v3/meta/*`, solo POST** — `router/v3-meta-router.ts:2` ("55 接口") y gate `method !== "POST"` (`:367`). Router comparte el proceso HTTP del gateway (`gateway/server.ts:877-886,1055`). No existe `/auth/refresh` ni `team-member/update-role`.
4. **Permission checker allow-only** — solo evalúa `acl.effect === "allow"` (`service/permission-checker.ts:89,127`); `AclEffect` incluye `deny` "预留" (`types.ts:48-49`). Orden: resource→owner→member→visibility→role-default→ACL (`permission-checker.ts:47-138`); con matiz: `visibility=restricted` evalúa ACL inline antes de role-default para no-admin (`:83-100`). Anti-enumeración: checker devuelve `allowed:false, reason:"asset_not_available"` (`metadata-service.ts:1525-1527`); 404 real vía `orNotFound` (`v3-meta-router.ts:74-79`).

## 2. Arquitectura y flujo

```
HTTP (node:http puro, gateway/server.ts)
├─ v1 legacy (recall, capture, search/*, session/end, seed)
├─ data plane v2/v3 (v2-router.ts: L0-L3 memory + skill/knowledge/chat-memory extraRouteTable)
├─ metadata plane /v3/meta/* (55 endpoints, SOLO POST)
│    MemoryCore/src/metadata/router/v3-meta-router.ts → handlers por entidad
│    MemoryCore/src/metadata/service/{metadata-service,permission-checker,user-visibility,
│                                      resolve-user-id,config-param-service}.ts
│    MemoryCore/src/metadata/store/{sqlite-adapter,mongodb-adapter,factory,db-name}.ts
│    auth 3 capas (§3.3)
└─ /v3/internal/meta/* (ops, solo Bearer): user/init-admin, user/list-by-instance
```

**Flujo de una request a `/v3/meta/team/get`:** 1) L1 Bearer apiKey → `verifyAuth` + `timingSafeEqual` (`gateway/server.ts:1116-1129`); 2) gate `/v3/meta/*` (`server.ts:877-886`); 3) `extractInstanceId(x-tdai-service-id)` → store por instancia; 4) L3 `x-tdai-user-key` → `authenticateV3` (`router/auth.ts:47-69`); 5) handler zod → MetadataService → permission-checker; 6) envelope `successEnvelope`/`errorEnvelope`.

## 3. Lógica y algoritmos

**3.1 Entidades REALES** (`metadata/types.ts`): `UserEntity` con `user_type: "normal"|"system_admin"` (`:16,68`); `TeamEntity` con `owner_user_id` y roles `TeamRole = "admin"|"member"|"reviewer"` (`:18`) — **no existe rol "owner"**; `AgentEntity` (`:99-111`) **sin** `agent_type/owner_team_id/llm_binding/skills/knowledge_bindings`; `InjectionMode = "direct"|"summary"|"tool"|"reference"` (`:34`) vive en `FixedAssetBindingEntity` (`:199`); `TaskEntity` (`:113-127`) **sin** `task_uid/owner_agent_id/user_memory_tier`; `AssetVisibility = "private"|"team"|"restricted"|"agent"|"task"` (`:25`, no existe "public"); `AssetType = "skill"|"llm_wiki"|"code_graph"|"chat_memory"` (`:24`); `AssetStatus = draft|candidate|approved|deprecated|archived|failed` (`:26-32`); password scrypt+pepper `$scrypt$...` (`:59-60`, `utils/crypto.ts:157-159`).

**3.2 ACL real** — `AclEntity {id, asset_id, subject_type(user|team_role|agent), subject_id, permission(read|write|delete|assign|share|use), effect, granted_by, created_at, updated_at}` (`types.ts:275-285`; tabla `meta_asset_acl` `sqlite-adapter.ts:263-274`). Sin wildcard. Lazy-load real: primera pasada `aclRecords: []`, recarga solo si `reason === "no_permission"` y `roleDefaultCovers` no cubre (`metadata-service.ts:1536-1553`; `permission-checker.ts:145-148`). No existe anti self-escalation: `grantAclForCaller` exige ser asset owner (`assertCallerIsAssetOwner`) + `granted_by === callerId` (`metadata-service.ts:1976-1983`); `grantAcl` → 404 si asset no existe (`:1504-1508`).

**3.3 Auth 3 capas** — L1 `Bearer` con `timingSafeEqual` (`gateway/server.ts:268-273,1116-1129`; env `TDAI_GATEWAY_API_KEY`, `:732-734`); L2 `x-tdai-service-id` → **401 si falta** (`v2-router.ts:357-359`), se usa como instanceId para resolver store (`:373-383` y `metadata/router/instance.ts:28-31`) — no valida contra llm-binding; L3 `x-tdai-user-key` → `userId`/`isSystemAdmin`, excepción `/v3/meta/auth/verify` (`V3_NO_USER_KEY_ROUTES`, `auth.ts:33-35,64`); la key del "memory system user" es **rechazada** en L3 (`auth.ts:55-57`; `system-user.ts:122-129`).

**3.4 SYSTEM_ADMIN** — no hay bypass global en router; bypass por-método `canManageUsers = ctx.isSystemAdmin` (`service/user-visibility.ts:16-18`); el permission-checker **no bypassa** (system_admin no-owner/no-member → DENY, `permission-checker.ts` no tiene rama admin global); protección `last_system_admin` (`metadata-service.ts:522-525`, → 409).

**3.5 Quota** — NO está en el flujo metadata: `v3-meta-router.ts` no hace debit ni importa quota. Es del data plane (`gateway/quota-credit-policy.ts`, `core/quota/quota-manager.ts`). `CreditCalculator`: Input 1.0 / Cache 0.2 / Output 4.0 Credit por 1k tokens × multiplier por modelo (`core/quota/credit-calculator.ts:5-8,36-40`). QuotaManager solo en `deployMode=service` (`gateway/server.ts:1776-1778`; standalone usa `NoopQuotaReporter`, `:1759`, `quota-manager.ts:10-11,79-88`); `deployMode=service` exige MongoDB (`factory.ts:121-129`). Metadata solo expone límites vía `/v3/meta/instance-quota/get` (ConfigParamService).

**3.6 Telemetría** — noop por defecto (`core/report/factory.ts:52,83-85,97,125-127`); `x-trace-id` solo en backend OTLP (`core/report/otlp-backend.ts:501`); **no existe** `x-request-duration-ms`.

## 4. Funcionalidades/Endpoints (solo reales, verif. en `v3-meta-router.ts:84-313`)

55 rutas POST: `user/*` (create, create-with-key, get, delete, list); `user-key/*` (create, list, get, revoke, update); `team/*` (create, get, update, delete, list); `team-member/*` (add, remove, list, **get** — no hay update-role); `agent/*` (create, get, update, delete, list, archive); `task/*` (create, get, update, delete, list, archive); `task-agent/*` (link, unlink, list); `participation-log/*` (append, list); `asset/*` (create, get, update, delete, list, **list-accessible**, **touch-usage**); `agent-fixed-asset/*` (set, list, **list-with-detail**, **summary-by-agents**); `acl/*` (grant, revoke, list, check); `auth/verify`; `instance-quota/get`; `config/user/get|set`. `/v3/knowledge/*` está en `gateway/knowledge-handlers.ts` (data plane, no metadata). `/v3/internal/meta/*` (init-admin, list-by-instance) solo-L1 en `router/internal-meta-router.ts:59-72`.

**Códigos** (`v3-meta-router.ts:319-353`): `*_not_found`→404; permission_denied/agent_team_mismatch/task_agent_not_linked→403; duplicate_*/last_system_admin/member_already_exists/already_initialized→409; invalid_credentials/invalid_password→401; missing_instance_id/missing_team_id→400.

## 5. Código clave (fragmentos literales)

```ts
// gateway/v2-router.ts:332-334
export function successEnvelope<T>(data: T, requestId: string): ApiResponseEnvelope<T> {
  return { code: 0, message: "ok", request_id: requestId, data };
}
// metadata/service/permission-checker.ts:89,127 — allow-only
acl.effect === "allow" && ((acl.subject_type === "user" && ...))
// metadata/service/metadata-service.ts:1536-1553 — lazy ACL
const fast = checkPermission({ ... aclRecords: [], ... });
if (fast.allowed) return fast;
if (fast.reason !== "no_permission") return fast;
if (membership && roleDefaultCovers(membership.role, action)) return fast;
const aclRecords = await this.allAclRecords(params.asset_id);
```

## 6. Integración en VantaDB

- **core:** entidades `entity_*` ya existentes + checker allow-only (~40 líneas, sin deny, sin lazy-load propio: VantaDB decide por rol simple).
- **server:** auth 3 capas (Bearer timingSafeEqual → service-id → user-key) + quota solo server mode.
- **NO copiar:** dual SQLite/Mongo con pool LRU, router monolítico de 55 endpoints, envelope con `message`/`request_id`, ACL con `granted_by`.

## 7. Riesgos/limitaciones / NO copiar

- 55 endpoints en un solo router = acoplamiento; preferir sub-routers por entidad.
- `deny` reservado pero nunca evaluado → no se puede expresar "bloquear a X" (solo ausencia de allow).
- system_admin no bypassa checker → operaciones cross-team requieren ACL explícita.
- Quota solo service-mode: standalone sin límites; `deployMode=service` obliga MongoDB.
- **Nota:** la versión previa tenía datos refutados (store único, envelope, ACL `meta_acls`, endpoints inventados); esta es la verificada contra el clone @ 97f9465.

## RESULTADO
- Estado: ✅ CORREGIDO Y VERIFICADO
- Archivo: docs/research/tdam/06-metadata-acl.md
- Hallazgo principal: el metadata plane es un sistema **dual SQLite/Mongo** con 55 endpoints POST allow-only; las afirmaciones previas (Mongo único, `{code,msg,data}`, campos inexistentes) quedaron refutadas contra el código real.
- Ref clave real: `MemoryCore/src/metadata/router/v3-meta-router.ts:84-313` (routeTable 55), `store/factory.ts:85-100` (dual), `service/permission-checker.ts:89,127` (allow-only), `gateway/v2-router.ts:332-338` (envelope)