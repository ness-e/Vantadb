# TASK WEB-01: REST — superficie de la consola (CRUD + search + list + IQL + health/metrics/audit)

## Metadata
- **Plan file:** docs/plans/2026-08-18-vanta-studio-fase3.md (Task 2, Wave 1)
- **Creado:** 2026-08-19T11:00
- **last-synced:** 2026-08-19T12:30
- **Estado:** ✅ COMPLETED

## Blast Radius
**Callers:** `app_with_cors` (3 callers en `src/cli_server.rs`) · `app` (27 callers) · `vantadb-server` crate re-exporta `vantadb::cli_server::{app, auth_middleware, init_telemetry, run, AuthState, NodeDTO, QueryRequest, QueryResponse, ServerState}` (vantadb-server/src/server.rs:1-4) · `ServerState` struct literal en 10 sitios (cli_server.rs:898 run / 1155 body_limit test / 1242 cors_test_state; helpers/mod.rs:17; server.rs:151/344/386/486/577; e2e.rs:232; benchmarks.rs:280).
**Callees:** `VantaEmbedded::from_engine` (builder.rs:35) · `put`/`put_batch`/`get`/`get_version`/`versions`/`delete`/`list`/`delete_by_filter`/`list_namespaces` (sdk/api.rs) · `search` (sdk/search/mod.rs:69) · `parser::autocomplete_prefix` (parser/mod.rs:572) · `AuditEvent` (audit.rs:16, Serialize) · `ConnectionPool::acquire` + `PoolError` · `VantaError` (error.rs:91).
**Implicaciones:** AGREGAR campo `db: VantaEmbedded` a `ServerState` → actualizar los 10 struct literals. NO se toca `src/sdk/` (tipos solo lectura), `desktop/` (patrones solo referencia), `vantadb-wasm/`, `web/`. NO duplicar `/api/v2/query` ni `/metrics`. Sin commit (lo commitea vanta-lead).

## Contrato
"`cargo check --features server` verde; `cargo test` verde (tests existentes no rotos); smoke: `vanta serve` con DB temp + cada endpoint probado con `Invoke-RestMethod` (status + shape JSON); task file WEB-01.md con pasos atómicos y checkboxes; sin commit."
Wire contract:
- Éxito = payload SDK directo (record, `Vec<VantaMemoryRecord>`, `VantaMemoryListPage`, `Vec<VantaMemorySearchHit>`, `Vec<String>`, `AuditPage`). Error = `{"success": false, "error": "<mensaje>"}` con status del mapeo `vanta_error_status` (400/404/409/422/500). `VantaValue` en JSON es externally-tagged (`{"String":"x"}`) — mismo wire del SDK serde.
- `GET /api/v2/health` → `{"status":"healthy|degraded","backend":"fjall|rocksdb|in-memory","latency_ms":u64,"checked_at_ms":u64,"message":?}` (espeja `HealthReport` desktop).
- `GET /api/v2/records/{ns}/{key}` → 200 record | 404 `{success:false,error:"record not found: <key>"}`.
- `GET /api/v2/records/{ns}/{key}/versions?version=N` → `version` presente = get_version (404 si no existe); ausente = `Vec<VantaMemoryRecord>`.
- `DELETE /api/v2/records/{ns}/{key}` → 200 `{"deleted":true}` | 404 si no existe.
- `DELETE /api/v2/records?namespace=&filter=<JSON array VantaMemoryFilterItem>` → 200 `{"deleted":count}` | 400 si filter inválido/vacío.
- `GET /api/v2/list?namespace=&limit=&cursor=&filter_ops=<JSON array>` → `VantaMemoryListPage` (cursor real) | 400 si falta namespace.
- `GET /api/v2/audit?namespace=&op=&outcome=&limit=&cursor=` → AuditPage newest-first (patrón desktop commands/audit.rs) | 404 `"audit log no configurado"` si `audit_log_path` es None.
- D12: rutas v2 dentro del router protegido (Bearer si `api_key` configurado); `/health` público se mantiene; `/metrics` y `/api/v2/query` intactos.

## Herramientas
- Bash (`cargo check --features server`, `cargo test -p vantadb`, smoke con `Invoke-RestMethod`), codegraph, read/edit

## Steps
### Step 1: ServerState + rutas + handlers en `src/cli_server.rs`
- **Archivos:** `src/cli_server.rs`
- **Acción:** campo `db: VantaEmbedded` en `ServerState` + `VantaEmbedded::from_engine(storage.clone())` en `run()`. Rutas nuevas en el router protected de `app_with_cors`: health_v2, records_put (POST), records_put_batch (POST /batch), records_get (GET {ns}/{key}), records_versions (GET {ns}/{key}/versions), records_delete (DELETE {ns}/{key}), records_delete_by_filter (DELETE), records_list (GET /list), records_search (POST /search), iql_autocomplete (GET /autocomplete), audit_events (GET /audit). Helpers: `vanta_error_status` (extraído de query_error_response), `vanta_error_response`, `pool_error_response`, `run_db_op` (pool permit + spawn_blocking, patrón execute_query — R-2 server-mcp), `read_audit_page` (copia local del patrón desktop, `read_to_string` + filtros + newest-first; missing file → io error → 500). Blocking ops SIEMPRE vía spawn_blocking (Regla 8).
- **Verify:** `cargo check --features server`
- **Estado:** ✅ (check verde; fix extra: `AuditEvent` ganó `Deserialize` en `src/audit.rs` para `read_audit_page`; `run_db_op` usa `std::result::Result` por colisión con el alias `error::Result`)

### Step 2: Actualizar literales `ServerState` (10 sitios)
- **Archivos:** `src/cli_server.rs` (tests: 1155, 1242), `vantadb-server/tests/helpers/mod.rs`, `vantadb-server/tests/server.rs` (151/344/386/486/577), `vantadb-server/tests/e2e.rs` (232), `vantadb-server/tests/benchmarks.rs` (280)
- **Acción:** insertar `let db = VantaEmbedded::from_engine(storage.clone());` antes del literal (sitios con `storage,` bare — evita use-after-move) o `db: VantaEmbedded::from_engine(storage.clone())` inline (run() con `storage: storage.clone()`, e2e con `storage: storage2,`). Sin import nuevo: path fully-qualified `vantadb::VantaEmbedded` en tests externos; `crate::sdk::VantaEmbedded` en-crate.
- **Verify:** `cargo check --features server` + `cargo check -p vantadb-server --tests`
- **Estado:** ✅ (11 sitios actualizados — la cuenta real es 11 literales, no 10; e2e.rs usa `storage: storage2.clone()` antes de `from_engine(storage2.clone())` para evitar move)

### Step 3: Tests en-crate (roundtrip + errores)
- **Archivos:** `src/cli_server.rs` (mod tests)
- **Acción:** helpers `json_request`/`raw_request` (patrón spawn_app + TCP raw existente en el archivo). Tests: `v2_records_roundtrip` (put 201 → get 200 → versions → list con cursor → delete_by_filter → audit events con audit_log_path en tempdir → health healthy → autocomplete) y `v2_errors_map_status` (get/delete missing → 404; filter inválido → 400; delete_by_filter sin filter → 400; audit sin configurar → 404).
- **Verify:** `cargo test -p vantadb --features server -- cli_server`
- **Estado:** ✅ (17/17 tests cli_server pasan; helpers finales: `raw_request`/`json_request`/`parse_response`/`urlencode`/`raw_get`/`raw_delete` — los closures con `&str` + async block fallaban lifetimes)

### Step 4: Verify full + smoke + cierre
- **Acción:** `cargo check --features server`, `cargo test` (suite server), smoke real: `vanta serve` con DB temp + `VANTADB_AUDIT_LOG_PATH`, probar cada endpoint con `Invoke-RestMethod` (status + shape). Reportar bloque RESULTADO sin commit.
- **Verify:** todos los gates verdes
- **Estado:** ✅ (1820 unit tests lib OK, 19 tests vantadb-server OK; smoke real OK — ver RESULTADO; nota: crash pre-existente del toolchain 1.95.0 compilando harnesses de integración ajenos — `cargo test` completo requiere `--lib` o `-j 1`)

## Dependencias
- VS-CORE-01 ✅ (cursor real en list — este task lo expone vía REST)
- Plan D10/D11/D12 ✅ (server-embebido primero; REST completo del SDK; sin auth en loopback, Bearer si api_key)

## Notas
- **Decisión db compartida:** `VantaEmbedded` vive en `ServerState` (no per-request) porque `init_audit` abre el JSONL — por-request abriría N handles sin Mutex compartido → appends concurrentes corruptos en el audit log (ACTIVITY).
- **Decisión error shape:** `{success:false,error}` — consistente con auth_middleware y circuit_breaker; el endpoint IQL existente conserva su shape `QueryResponse` (tests dependen).
- **Decisión audit default:** espeja desktop — si `audit_log_path` es None → 404 "audit log no configurado" (honesto: no hay audit corriendo). El fallback `<storage>/audit.jsonl` NO se usa (archivo nunca escrito → UI vacía engañosa).
- **ponytail:** `read_audit_page` lee el archivo completo (fine para logs de consola); tail read por byte-offset es el upgrade.
- **axum 0.8:** params de ruta con sintaxis `{ns}`/`{key}`; axum decodea percent-encoding automáticamente.