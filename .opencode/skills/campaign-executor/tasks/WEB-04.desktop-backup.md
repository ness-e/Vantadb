# TASK WEB-04: HttpBackend real (fetch REST) + factory por entorno + vanta-http-map.ts

## Metadata
- **Plan file:** docs/plans/2026-08-18-vanta-studio-fase3.md (Task 4, Wave 1)
- **Creado:** 2026-08-19
- **last-synced:** 2026-08-19
- **Estado:** ✅ COMPLETED

## Blast Radius
**Callers:** `transport` singleton (transport.ts:40, fijado en load-time por `getTransport()`) · todas las wrappers de `desktop/src/vanta.ts` (~26 command wrappers, ~28 exports) · componentes que consumen las wrappers (33 imports de `../vanta`): ActivityPanel, ConnectionPanel, DataExplorer, ExportPanel, Timeline, ExportButtons, HomeOverview, IngestForm, CopyButton, GraphScene, IqlConsole, KpiCards, Inspector, WorkspaceShell, historial-tab, useConnectionState, ResultsList, RetrievalLens, SpaceLens.
**Callees:** `fetch` (REST `/api/v2/*` del server embebido — `src/cli_server.rs`) · adaptaciones wire espejo del bridge Tauri (`desktop/src-tauri/src/connections/native.rs`: `to_vanta_value`/`from_vanta_value`/`record_to_memory`/`search_request`/`ingest_to_input`/`gen_id`).
**Implicaciones:** NO se toca `vanta.ts` (firma intacta) · NO se toca `src/sdk/`, `src/cli_server.rs`, `desktop/src-tauri/` (solo lectura — divergencias documentadas, no parcheadas) · `desktop/src/transport.ts` gana HttpBackend real + factory por entorno · nuevo `desktop/src/vanta-http-map.ts` con la tabla cmd→REST + transformers · sin commit (lo commitea vanta-lead).

## Contrato
"`npm run build` verde en `desktop/`; `npx vitest run` verde; `node --test src/vanta-deep-link.test.ts` verde (test usa node:test, vitest lo reporta mal); cobertura: TODAS las wrappers de vanta.ts mapeadas a REST real O rechazo descriptivo documentado (divergencia wire, no inventar paths); sin commit."
Wire contract (verificado contra `src/cli_server.rs` + `src/sdk/types.rs` + bridge Tauri):
- Éxito = payload SDK serde directo; Error HTTP = `{"success":false,"error":"..."}` (404 get/delete = `"record not found: <key>"`). EXCEPCIÓN: `/api/v2/query` devuelve `{"success":false,"data":"..."}` — el mapper de errores debe leer `error ?? data`.
- Record SDK wire: `{namespace, key, payload, metadata(VantaValue tagged), created_at_ms, updated_at_ms, version, node_id(STRING u128), vector, sparse_vector, expires_at_ms}` → desktop DTO `{id, namespace, text, ...}` (bridge native.rs:227 `record_to_memory`).
- Search: `POST /api/v2/search` body `VantaMemorySearchRequest` → `Vec<VantaMemorySearchHit>`; hit = `{record, score, explanation}`.
- Put: `POST /api/v2/records` body `VantaMemoryInput` (201 record) | batch: `POST /api/v2/records/batch` (201 records → ids).
- List: `GET /api/v2/list?namespace=&limit=&cursor=&filter_ops=<JSON>` → `VantaMemoryListPage {records, next_cursor}` (400 si falta namespace).
- Get/Delete: `GET|DELETE /api/v2/records/{ns}/{key}`; versions: `GET /api/v2/records/{ns}/{key}/versions?version=N` (N presente = single, ausente = array).
- Export: `POST /api/v2/export` body `{path, namespace?, filter?}` → `VantaExportReport` (shape = desktop `ExportReport`).
- Delete by filter: `DELETE /api/v2/records?namespace=&filter=<JSON VantaMemoryFilter>` → `{"deleted":n}`.
- Audit: `GET /api/v2/audit?namespace=&op=&outcome=&limit=&cursor=` → `AuditPageV2 {events, next_cursor}` (404 si audit no configurado).
- Autocomplete: `GET /api/v2/autocomplete?prefix=` → `Vec<String>`.
- Health: `GET /api/v2/health` → `HealthReport` (shape idéntico al desktop).
- IQL: `POST /api/v2/query` body `{query}` → `QueryResponse {success, data, node_id?: u128 NUMERO, nodes?: NodeDTO[]}`; NodeDTO = `{id: u128 NUMERO, semantic_cluster, relational: BTreeMap<String,FieldValue>, hits, confidence_score}`. Adaptar a `VantaQueryResult` (Read: nodes→MemoryRecord con node_id/id string + text recuperado de relational `__vanta_payload`/`text`/`content`; Write: data empieza "Mutated N nodes: msg"; StaleContext: data empieza "STALE_CONTEXT").
- DIVERGENCIAS wire (NO inventar paths — rechazo descriptivo + Notas):
  - `vanta_connect|disconnect|list_connections|set_active`: multi-conexión es Tauri-only; web = una conexión implícita al server.
  - `vanta_metrics`: NO existe `/api/v2/metrics` JSON (solo `/metrics` Prometheus text, feature-gated) — requiere endpoint nuevo (follow-up).
  - `vanta_graph_bfs|dfs|degree`: REST devuelve arrays/mapas de u128 NUMERICOS sin labels/edges (`GraphTraversalRequest.roots: Vec<u128>`), incompatible con DTO desktop `VantaGraphTraversalResult` (ids string + labels) — requiere endpoint graph_v2 (follow-up).
  - `vanta_deep_link_take`: ya no-op para no-Tauri en vanta.ts (guard `transport instanceof TauriBackend`).
- D12: base URL = `""` (mismo origin, dev server embebido) o `VITE_VANTA_API_BASE` para server remoto.

## Herramientas
- Bash (`npm run build`, `npx vitest run`, `node --test src/vanta-deep-link.test.ts` en `desktop/`), read/edit, codegraph

## Steps
### Step 1: Crear `desktop/src/vanta-http-map.ts`
- **Archivos:** `desktop/src/vanta-http-map.ts` (nuevo)
- **Acción:** tabla cmd→`{method, path, query?, body?, transform?}` + helpers de adaptación wire (espejo native.rs): `toVantaValue` (plain JSON→tagged), `fromVantaValue` (tagged→plain), `recordFromSdk` (key→id, payload→text, node_id→string, metadata untagged), `searchHitFromSdk`, `ingestToInput` (key = id ?? `rec_{now}_{seq}`, namespace ?? ""), `searchToRequest` (namespace ?? "", embedding ?? [], text_query trimmed o null, top_k ?? 10, "Cosine", explain ?? false), `exportFilterToWire`, `queryResultFromResponse`. Entradas "unsupported" con razón (connect/disconnect/list_connections/set_active/metrics/graph_*). Importar tipos desde `./vanta` (solo tipos, sin side effects).
- **Verify:** `npm run build` (tsc) en `desktop/`
- **Estado:** ✅ (16 comandos mapeados a REST real + 8 unsupported documentados; 23 comandos cubiertos en total — el recuento real del repo es ~23, no ~55 como decía el prompt)

### Step 2: HttpBackend real + factory en `desktop/src/transport.ts`
- **Archivos:** `desktop/src/transport.ts`
- **Acción:** reemplazar stub: `HttpBackend(base)` con `call<T>` que resuelve mapping (vanta-http-map), arma fetch (path + query + body JSON, `content-type` solo si body), `!res.ok` → error parseando `{success:false,error|data}` (con fallback `HTTP <status>`), `transform` sobre JSON. `getTransport()`: Tauri (`__TAURI_INTERNALS__`) → TauriBackend; browser → `HttpBackend(VITE_VANTA_API_BASE ?? "")` (`import.meta` con env opcional tipado — sin depender de vite/client). Singletons sin cambios.
- **Verify:** `npm run build`
- **Estado:** ✅ (build verde; fix: parameter property `private readonly base` rompía el strip-only mode de node --test — se usa field explícito)

### Step 3: Test unitario del mapper (node:test)
- **Archivos:** `desktop/src/vanta-http-map.test.ts` (nuevo)
- **Acción:** tests del mapper puro (sin fetch): `recordFromSdk` key→id/payload→text; `searchHitFromSdk`; `queryResultFromResponse` Read (node_id string + text de `__vanta_payload`) / Write / StaleContext; `toVantaValue` roundtrip tagged; mapeo completo: para cada wrapper export de `./vanta` que use `transport.call`, existe entrada en el map con endpoint real o `unsupported` documentado (assert por lista fija).
- **Verify:** `node --test src/vanta-http-map.test.ts`
- **Estado:** ✅ (14/14 tests; runner real del repo es node:test — vitest NO está instalado en desktop/, las suites vitest-style existentes tampoco corren)

### Step 4: Verify full + cierre
- **Acción:** `npm run build`, `npx vitest run`, `node --test src/vanta-deep-link.test.ts`. Smoke opcional contra server real (`vanta serve` con DB temp + `VITE_VANTA_API_BASE`). Reportar RESULTADO sin commit.
- **Verify:** todos los gates verdes
- **Estado:** ✅ (build ✓ · node:test 28/28 en 4 suites incl. vanta-deep-link pre-existente ✓ · vitest no instalado — documentado; smoke real diferido al lead: mapeo validado contra rutas exactas de cli_server.rs)

## Dependencias
- WEB-01 ✅ (endpoints `/api/v2/*` reales — este task los consume)
- Plan D10/D12 ✅ (server-embebido; una conexión implícita en web, sin auth en loopback)

## Notas
- **Adaptación wire es espejo del bridge Tauri** (native.rs) — si el bridge cambia, el map debe seguirle; ambas direcciones documentadas en el map.
- **Node ids como strings** en el DTO desktop (u128 excede safe integer); NodeDTO REST los manda como NUMBER — `String(n.id)` en la adaptación; para roots de graph NO es convertible sin pérdida → rechazo.
- **Ponytail:** transform de `QueryResponse` Read produce solo los campos que consume IqlConsole (node_id, text, namespace, metadata sin `__vanta_*`) — no replica el recovery completo de `node_record_to_memory`.