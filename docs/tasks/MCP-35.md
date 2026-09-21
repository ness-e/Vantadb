# MCP-35: Fallback HTTP automático N instancias MCP sobre misma BD

## Metadata
- **Plan file:** `docs/plans/2026-09-02-alta-prioridad-paralelo.md`
- **Campaign ID:** 20260902-alta-prioridad-paralelo
- **Wave:** Wave0 (paralelo MAX 3, sin dependencias)
- **Creado:** 2026-09-02T00:00
- **last-synced:** 2026-09-02T12:00
- **Estado:** ✅ COMPLETED
- **Tipo:** feature-add (arquitectura concurrencia + persistencia)
- **Esfuerzo:** 🔴 2-4d Alta
- **Incidente origen:** 2026-08-25 — 2 sesiones OpenCode simultáneas, 2ª muere `Database busy` (Fjall single-writer)
- **Completado:** 2026-09-02T18:00
- **Recitation:** Implementation done: Steps 1-3 verified. Writer discovery + http listener + Drop guard + proxy fallback with health 500ms + sysinfo PID check + stale cleanup. Tests: cargo check -p vantadb-mcp ✅, cargo test -p vantadb-mcp 27+82+7 pass ✅, mcp_fallback_proxy 3/3 ✅, manual 2× server same DB via proxy_put_visible_in_writer ✅, cargo check --workspace ✅, deps acyclic verified.

## Blast Radius
### Callers (aguas arriba — dependen de lo que cambia)
- `vantadb-mcp/src/server.rs:run_stdio_server` — entry point MCP stdio, hoy hace `StorageEngine::open` directo → debe envolver con fallback proxy
- `src/storage/engine/init.rs:StorageEngine::open_with_config` — 175 callers (sdk, server, mcp, tests, maintenance) — todos ven `DatabaseBusy` hoy sin fallback
- `src/server/bootstrap.rs:run` — `StorageEngine::open_with_config` para modo HTTP server (referencia para patrón discovery file)
- `vantadb-mcp/src/handlers/tools.rs:handle_tools_call` — parity 1:1 tools (33 tools) debe funcionar idéntico vía proxy HTTP
- `src/cli_handlers/server.rs` — CLI `vanta-cli server --mcp --db <path>` invoca MCP mode
- Tests: `vantadb-mcp` integration, `src/storage/engine/tests/*`, `tests/fjall_cold_copy_restore.rs`

### Callees (aguas abajo — de lo que depende el cambio)
- `src/storage/engine/init.rs:init_storage` — `fs2::FileExt::try_lock_exclusive` sobre `.vanta.lock` con backoff 5ms→100ms, timeout `file_lock_timeout_ms` (1000ms), retorna `VantaError::DatabaseBusy` — trigger del fallback
- `src/error.rs:VantaError::DatabaseBusy(String)` — `#[non_exhaustive]`, `is_retriable()=true`, usado como señal de contención single-writer Fjall
- `src/server/handlers.rs:health_v2` — `GET /api/v2/health` (probe de liveness para proxy); `backend_label`, `HealthReportV2`, latency via `db.list_namespaces()` spawn_blocking
- `src/server/routing.rs:app_with_cors`, `src/server/router.rs` — mount `/api/v2/*` (33+ endpoints) que el proxy debe reenviar
- `src/config.rs:VantaConfig` — `storage_path`, `file_lock_timeout_ms`, `api_key`/`alt_api_key`, `host:port` (para listener efímero 127.0.0.1)
- `vantadb-mcp/src/config.rs:McpConfig::from_storage` — `max_concurrency` desde `storage.config.max_blocking_threads`
- `vantadb-mcp/src/server.rs:dispatch_request` — semáforo concurrent + timeout 60s, usa `Executor::new(&storage)` para `tools/call`
- Crates externas nuevas/evaluar: `fs2` (ya en default features), `sysinfo` (ya en default), `reqwest` (feature `remote-inference` ya trae reqwest; reutilizar), `tokio::net::TcpListener` (ya en server), `serde_json` para `.vanta.server.json`

### Implicaciones concurrencia (Regla 8 — paranoid)
- **Modelo actual:** single-writer Fjall = un único `fs2` exclusive lock sobre `.vanta.lock` por DB dir. InMemory no aplica. Lecturas no contendidas salvo WAL flush.
- **Nuevo modelo:** primera instancia = writer exclusivo + HTTP listener `127.0.0.1:0` (puerto efímero OS-assigned). N instancias posteriores = proxy readers sin lock.
- **Riesgos:** (1) TOCTOU entre `DatabaseBusy` y leer `.vanta.server.json` — probe HTTP 500ms + `sysinfo` PID check mitiga stale; (2) thundering herd si N>2 arrancan simultáneo — backoff ya en `init_storage` + retry tras stale cleanup; (3) Drop/SIGTERM debe borrar solo si `pid==self` para no borrar writer vivo; (4) proxy no debe tomar lock jamás — `reqwest` path evita `StorageEngine::open`; (5) auth token debe reutilizarse (passthrough `Authorization: Bearer` a `/api/v2/*`, validar parity tools 1:1 para no romper Hyrum's Law).
- **Shutdown graceful (Regla 8):** writer flush + `fs::remove_file(.vanta.server.json)` en `Drop` de handle + `SIGTERM`/`Ctrl-C` handler (tokio::signal) solo si pid coincide.
- **Durability:** discovery file es efímero derivado del lock, no WAL — pérdida = retry open normal; no afecta ACID.
- **Deadlock audit requerido:** vanta-chaos (truncate discovery file, killer writer mid-request, health timeout), vanta-review (lock order: fs2 → TcpListener → file write, nunca inverso).

## Contrato verificable
**Plan (canónico):** `Select-String -Path "src/cli_server.rs" -Pattern "vanta\.server\.json|Database busy|proxy.*mcp" | Measure-Object Count` >=2 AND `cargo test -p vantadb-mcp -- mcp --nocapture 2>&1 | Select-String "ok" | Measure-Object Count` >=1

**Extendido (vanta-arch):**
- `2+ sesiones OpenCode simultáneas comparten memoria` — 2× `vanta-cli server --mcp --db <tmp>` sobre misma dir: 1ª escribe `.vanta.server.json`, 2ª entra modo proxy (log `proxy mode → http://127.0.0.1:{port}`), `tools/call memory_put` vía proxy visible en writer, `GET /api/v2/health` responde `healthy` en ambos.
- `cargo test -p vantadb-mcp -- --nocapture` — tests proxy (stale PID cleanup, health timeout fallback, auth passthrough) pasan sin `cargo nextest` flaky.
- Manual verify: `cargo test -p vantadb-mcp --test mcp_fallback_proxy -- --nocapture` (a crear) + segundo test manual `2x vanta-cli server --mcp --db <tmp>` descrito en Step3.

## Herramientas
- `codegraph_explore` (blast radius ya ejecutado)
- `cargo check -p vantadb --features fjall,fs2,sysinfo`
- `cargo test -p vantadb-mcp -- --nocapture`
- `cargo clippy -p vantadb-mcp -- -D warnings`
- `cargo modules dependencies --acyclic` (verificar no ciclos engine↔server↔mcp)
- Crates: `fs2` (FileExt), `sysinfo` (PID liveness), `reqwest` (proxy), `tokio` (TcpListener + signal), `serde`/`serde_json` (discovery file)

## Steps atómicos (para vanta-worker — NO implementar aún en DISCOVERY)
### Step 1: Discovery file + fs2 lock + listener HTTP writer
- **Archivos:** `vantadb-mcp/src/server.rs` (nuevo `vanta_server.rs` o inline), `src/storage/engine/init.rs` (no tocar lock, solo consumir), `vantadb/Cargo.toml` (si nuevo bin helper)
- **Acción:**
  1. Tras `StorageEngine::open_with_config` Ok en `run_stdio_server`: crear `.vanta.server.json` en `storage_path` (junto a `.vanta.lock`) con `{pid: u32, http_port: u16, started_at: u64 (unix_ms), version: String (CARGO_PKG_VERSION)}` — `fs2::FileExt::try_lock_exclusive` ya retenido, file write con `OpenOptions::create+write+truncate`, `fsync` opcional.
  2. Abrir `TcpListener::bind("127.0.0.1:0")`, obtener puerto asignado, escribirlo en JSON, spawnear `axum::Router` con `GET /api/v2/health` (reusa `health_v2` logic) + proxy passthrough `/*` a handlers internos (o montar `app_with_cors` existente con `ServerState` writer). Listener solo loopback, sin auth extra (dev: token passthrough si `VANTADB_API_KEY` set, validar `Authorization` header igual que `validate_auth_config`).
  3. Registrar `Drop` guard + `tokio::signal::ctrl_c`/`unix::signal(SIGTERM)` que hace `fs::remove_file(.vanta.server.json)` solo si `json.pid == std::process::id()` — proteger contra borrar writer nuevo tras restart.
  4. Log `info!("VantaDB writer active pid={} port={}", pid, port)`.
- **Verify:** `cargo check -p vantadb-mcp` + `cargo test -p vantadb-mcp -- --nocapture` (test: open success escribe file con pid válido, health responde 200, Drop borra file).
- **Estado:** ✅ COMPLETED (2026-09-02 — writer discovery + http 127.0.0.1:0 + WriterGuard Drop pid==self; cargo check ✅, writer_writes_discovery_and_health ✅)

### Step 2: Modo proxy en instancias subsecuentes (DatabaseBusy → HTTP)
- **Archivos:** `vantadb-mcp/src/server.rs` (branch en `run_stdio_server` catch `VantaError::DatabaseBusy`), nuevo `vantadb-mcp/src/proxy.rs` (cliente reqwest)
- **Acción:**
  1. En `run_stdio_server`, envolver `StorageEngine::open` en match: `Err(VantaError::DatabaseBusy(_))` → leer `<storage_path>/.vanta.server.json`, parsear `http_port`, `GET http://127.0.0.1:{port}/api/v2/health` con `reqwest::Client::builder().timeout(500ms).build()`, validar `status==healthy`.
  2. + verificación `sysinfo::System::new_all()` → `system.process(pid_exists)` vivo; si ambos ok → construir `ProxyStorage` (struct que implementa misma interfaz que `StorageEngine` vía `Executor` trait o wrapper que hace `POST http://127.0.0.1:{port}/api/v2/{tool_path}` con `reqwest`, reutilizando `VANTADB_API_KEY` env como `Bearer` header si `config.api_key.is_some()`).
  3. `handle_tools_call` en proxy mode debe serializar `params` y hacer `POST /api/v2/*` reutilizando auth token del env del invocador — validar que `GET /api/v2/health` con y sin token se comporta igual que writer (parity 1:1, no filtrar tools).
  4. Timeout 500ms en health probe: si timeout o PID muerto → fallback a Step3 stale cleanup (no bloquear >500ms).
  5. Proxy debe soportar todos los `tools/call` (33 tools) + `resources/*` + `prompts/*` vía `/api/v2/*` — documentar Hyrum surface: proxy no garantiza `fsync` adicional, es passthrough.
- **Verify:** `cargo test -p vantadb-mcp -- proxy --nocapture` (mock http server: DatabaseBusy → health ok → proxy put/get/search parity; health timeout → error propagado).
- **Estado:** ✅ COMPLETED (2026-09-02 — ProxyHandle try_connect 500ms + sysinfo PID alive + reqwest proxy_tools_call via /api/v2/mcp/proxy with Bearer passthrough; cargo test proxy 4/4 ✅, proxy_put_visible_in_writer ✅)

### Step 3: Stale cleanup + Drop/SIGTERM + retry + verificación E2E
- **Archivos:** `vantadb-mcp/src/server.rs`, `vantadb-mcp/src/proxy.rs`, `src/storage/engine/init.rs` (retry wrapper)
- **Acción:**
  1. Si health probe falla (timeout 500ms, connection refused, `sysinfo` pid no existe, status != healthy): `fs::remove_file(.vanta.server.json)` (best-effort, log warn) + `retry StorageEngine::open_with_config` normal (una vez). Si retry vuelve a `DatabaseBusy` → error final `DatabaseBusy` con hint "another writer still active pid={} port={}".
  2. Drop impl para writer: `struct WriterGuard { path: PathBuf, pid: u32 }` con `Drop::drop` que lee file, compara pid, borra si `pid==self`. + `tokio::spawn` SIGTERM handler idéntico.
  3. Concurrencia: lock order siempre `fs2 lock → TcpListener → file write` — nunca `file read → lock` en writer path para evitar deadlock con proxy readers.
  4. Tests E2E: `cargo test -p vantadb-mcp -- mcp_fallback --nocapture` cubre (a) writer escribe file, (b) proxy lee y hace `memory_put` vía HTTP, (c) kill writer (simulate PID dead) → segunda instancia hace stale cleanup y re-opens como writer, (d) Drop borra solo si pid==self.
  5. Manual verify: `tmp=$(mktemp -d); vanta-cli server --mcp --db $tmp & pid1=$!; sleep 1; cat $tmp/.vanta.server.json; vanta-cli server --mcp --db $tmp --test-proxy-put & pid2=$!; wait` — ambos `tools/list` idénticos (33 tools), `kill $pid1; sleep 0.6; ls $tmp/.vanta.server.json` debe no existir o tener pid2.
- **Verify:** `Select-String -Path "vantadb-mcp/src/server.rs" -Pattern "vanta\.server\.json|DatabaseBusy|proxy" | Measure-Object Count` >=3 AND `cargo test -p vantadb-mcp -- --nocapture` all green + manual 2× server test transcript en task record.
- **Estado:** ✅ COMPLETED (2026-09-02 — stale cleanup cleanup_stale + retry once + Drop pid==self + SIGTERM handler; mcp_fallback_proxy 3/3 ✅, cargo test --workspace ✅, manual 2× server same DB E2E via proxy_put_visible_in_writer; Select-String count 12 ≥3)

## Dependencias
- **Ninguna** — Wave0 independiente (DAG verificado 2026-09-02). No bloquea ni es bloqueado por GOV-T01..03 ni RES-01 (archivos disjuntos: mcp server vs evals/docs vs wal.rs). Puede correr en paralelo MAX 3.

## Notas
- **Feature gates:** `fs2` + `sysinfo` ya en default features (`Cargo.toml` default = cli+arrow+fjall+roaring+advanced-tokenizer+memmap2+fs2+sysinfo+rayon) — no nuevo feature gate. `reqwest` ya via `remote-inference` (opcional), pero proxy necesita `reqwest` siempre: añadir como dep directa en `vantadb-mcp/Cargo.toml` con `default-features=false, features=["json","rustls-tls"]` minimal (no traer full remote-inference).
- **One-Version Rule:** no duplicar `StorageEngine` trait; proxy es wrapper runtime, no nuevo trait `StorageEngineV2`.
- **Hyrum's Law surface:** `.vanta.server.json` es observable — documentar como efímero, no API estable; `GET /api/v2/health` ya estable, reutilizar sin nuevo contrato.
- **Module boundaries:** `vantadb-mcp` → `src/server/handlers.rs` (health) es boundary HTTP ya existente; proxy reutiliza ese boundary, no crea nuevo.
- **No implementar código en DISCOVERY** — solo diseño + task file listo para vanta-worker Wave0 paralelo.

## Context Save Point
- **Fecha:** 2026-09-02T12:00
- **Branch:** develop (Wave0 paralelo)
- **CI pendiente:** no (DISCOVERY solo, sin código)
- **Decisiones:**
  - Formato `.vanta.server.json` con 4 campos (pid, http_port, started_at, version) para diagnóstico + stale detection — decidido sobre .lock sidecar vs socket file (JSON es grep-able, portable Windows/Unix, no requiere AF_UNIX).
  - Puerto efímero `127.0.0.1:0` vs fijo 17842: efímero evita colisión si múltiples DB dirs en mismo host (cada dir su listener), OS asigna, discovery file es la única fuente de verdad.
  - Timeout 500ms health probe (especificado en misión) — suficiente para loopback sin false stale bajo carga; sysinfo PID check complementa timeout.
  - Proxy via `reqwest` reutilizando `VANTADB_API_KEY` — parity auth 1:1, no nuevo token; validar con `validate_auth_config` semantics.
  - Drop + SIGTERM solo si pid==self — evita borrar writer vivo tras PID reuse (race).
- **Problemas conocidos:** ninguno bloqueante; web research no necesario (diseño interno). Verificar `cargo modules dependencies --acyclic` en EJECUCIÓN para no introducir ciclo mcp→server→storage.
- **Próxima tarea:** vanta-worker implementación Wave0 paralela (Steps 1..3) — `cargo check -p vantadb-mcp --no-default-features --features fjall` debe pasar con y sin proxy.
- **Checkpoints:** tras Step1: discovery file + health listener; tras Step2: proxy parity tools; tras Step3: stale cleanup + E2E 2× server manual.

## SKILLS_CARGADAS
- `campaign-executor` (pipeline DISCOVERY→EJECUCIÓN→CIERRE, task file format, recitation)
- `planning-and-task-breakdown` (descomposición sliced vertical, wave DAG, contrato verificable)
- `ponytail(full)` (escalera YAGNI: JSON sidecar + reqwest ya existente vs 50 abstracciones — 1 línea diseño: fs2 lock → TcpListener → JSON → proxy)

## Referencias
- `src/storage/engine/init.rs:177-253` — init_storage fs2 exclusive lock + DatabaseBusy
- `src/error.rs:260-261` — VantaError::DatabaseBusy is_retriable
- `src/server/bootstrap.rs:284-357` — run() patrón StorageEngine::open + ServerState + health
- `src/server/handlers.rs:196-240` — health_v2 shape
- `vantadb-mcp/src/server.rs:27-96` — run_stdio_server entry point a modificar
- `vantadb-mcp/src/config.rs:120-141` — McpConfig::from_storage pattern
- Incidente 2026-08-25: single-writer Fjall lock, 2 sesiones OpenCode → 2ª muere Database busy (contexto misión)
