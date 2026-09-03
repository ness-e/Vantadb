# Review de Módulo — `vantadb-server/`

| Campo | Valor |
|---|---|
| **Fecha** | 2026-08-23 |
| **Revisor** | ox-alpha (segunda opinión, contexto fresco — P2-01) |
| **Alcance** | `vantadb-server/src/*`, `tests/*`, `Cargo.toml`; contraste estructural con `src/cli_server.rs` (core, 4.603 líneas) |
| **Skills cargadas** | code-review-and-quality, security-and-hardening, performance-optimization, systematic-debugging |
| **Veredicto global** | **7.5 / 10** — arquitectura limpia y auth sólida; 1 hallazgo bloqueante (clase MCP-01 en el path HTTP) |

---

## 1. Resumen ejecutivo

`vantadb-server` es un **crate binario thin**, no un servidor propio: `src/server.rs` (4 líneas) y `src/middleware.rs` (1 línea) son re-exports puros de `vantadb::cli_server`. Todo el server HTTP real (router, auth, rate-limit, handlers, TLS, shutdown) vive en el core. El crate aporta tres cosas: el binario dispatcher (MCP stdio <-> HTTP), las features de build, y la suite de tests e2e/integración a nivel socket.

**Hallazgo principal (🔴):** el fix MCP-01 (`ensure_indexes_current` al arranque) cubre el path MCP pero **no el path HTTP** — `cli_server::run()` abre el engine por una ruta (`StorageEngine::open_with_config` + `VantaEmbedded::from_engine`) que nunca ejecuta `ensure_indexes_current`. Búsqueda textual/híbrida vía HTTP sobre DB fresca presumiblemente reproduce el bug original ("text_index not found: bm25"). Los tests e2e no lo cubren porque solo ejercitan IQL INSERT/FETCH/DELETE.

---

## 2. Relación con `cli_server.rs` — duplicación / canonicalidad (verificada)

Verificación hecha leyendo ambos lados (estructura completa de cli_server.rs vía grep de símbolos/rutas, contenido íntegro de vantadb-server):

| Aspecto | Evidencia | Conclusión |
|---|---|---|
| Re-export server.rs | `vantadb-server/src/server.rs:1-4`: re-exporta `app, auth_middleware, init_telemetry, run, AuthIdentity, AuthState, NodeDTO, QueryRequest, QueryResponse, ServerState` desde `vantadb::cli_server` | Shim puro, cero lógica |
| Re-export middleware.rs | `vantadb-server/src/middleware.rs:1`: re-exporta `auth_middleware, AuthIdentity, AuthState` | Duplicado redundante del propio server.rs (hallazgo amarillo) |
| Router + handlers | `src/cli_server.rs:207-326` (`app_with_cors`), handlers desde linea ~776 | **Canonico en el core** |
| Auth middleware | `src/cli_server.rs:605-773` | Canonico en el core |
| Arranque/shutdown/TLS | `src/cli_server.rs:1749-1981` (`run`, `serve_http_or_tls`) | Canonico en el core |
| Dashboard embebido | `src/cli_server.rs:1702-1747` (`mount_dashboard`) | Confirma ADR-026/027: dashboard vive en el core, no aqui |

**Dictamen de canonicalidad:** NO hay duplicación. La división real de responsabilidades:

```
vantadb-server (binario)          src/cli_server.rs (core, canónico)
------------------------          ----------------------------------
main.rs: dispatcher               app/app_with_cors: Router + capas
  --mcp -> vantadb_mcp            auth_middleware (L1/L2/L3)
  default -> cli_server::run      handlers REST /api/v2/*
allocators (jemalloc/mimalloc)    governor rate-limit, circuit breaker
features (tls, otel, prometheus)  mount_dashboard, telemetría
tests e2e/integración             serve_http_or_tls + graceful shutdown
```

La decisión es correcta: un binario que envuelve la librería, sin copiar lógica. El costo es cosmético: quien revise este crate esperando el server encuentra solo re-exports (mitigado por doc comments correctos en `lib.rs`).

**Corrección al contexto de la tarea:** el fix MCP-01 **no vive en `vantadb-server/src/main.rs`**. `main.rs:53` abre `StorageEngine` directo (sin ensure) y delega en `vantadb_mcp::run_stdio_server`, donde el fix existe (`vantadb-mcp/src/server.rs:38`). El fix es correcto pero su domicilio es `vantadb-mcp`, no este crate. Ver hallazgo 8.1 para la consecuencia.

---

## 3. Arquitectura

### main.rs (148 líneas) — bien
- Arg parsing hand-rolled con comentario que justifica no usar clap en runtime (clap es dev-dep). Decisión correcta para exactamente 1 flag booleano.
- `validate_args` rechaza args desconocidos con exit code 2 y esta testeado (`main.rs:105-148`, 3 tests incl. first-unknown-wins).
- Allocators por cfg: jemalloc non-Windows / mimalloc Windows — correcto y documentado.
- Path MCP: flush explicito tras salir de `run_stdio_server` (`main.rs:66-71`). Path HTTP: el flush lo hace `cli_server::run` en shutdown. Ambos caminos cierran limpio.

### División core/binario
Punto menor: `ServerState` se construye campo a campo dos veces (en `run()`, cli_server.rs:1782-1791, y manualmente en cada test helper). Un constructor reduciría duplicación en tests — cosmético.

### Flujo real cliente -> server -> core -> respuesta (verificado)
1. Request -> `TraceLayer` -> `request_metrics_middleware` -> `circuit_breaker_middleware` -> CORS (outermost, preflight antes de auth) -> `GovernorLayer` (si rpm>0) -> `auth_middleware` (capa `protected`)
2. Handler -> `run_db_op` (permiso de `ConnectionPool` + `spawn_blocking`) -> executor/SDK sobre `state.db` (VantaEmbedded) o `state.storage`
3. Errores mapeados por `vanta_error_status` (cli_server.rs:902) -> `{success:false, error}` con status HTTP correcto; body limit global de 1MB (`DefaultBodyLimit::max(1_000_000)`, cli_server.rs:317)

Orden de capas sano: métricas y circuit breaker ven todo; auth solo protege rutas protegidas; `/health` publica queda fuera del auth pero bajo governor de IP cuando rpm>0.

---

## 4. Endpoints vs los ~27 de /api/v2 (ADR-026/027)

Conteo en `app_with_cors` (cli_server.rs:232-286):
- Publicas: `/health`
- Protegidas: 27 lineas `.route()` que expanden a **~37 operaciones HTTP** (rutas multi-metodo: `records` POST+DELETE, `records/{ns}/{key}` GET+DELETE, `threads` GET+POST, `threads/{id}` GET+POST+DELETE, `skills/{skill_id}` PUT+PATCH+DELETE)

Cobertura funcional: query IQL, records CRUD+batch+versions+list+search+autocomplete, audit paginado, export/import, graph bfs/dfs/degree/centrality/pagerank (+variantes v2), maintenance purge/compact/flush/rebuild-index, threads/conversacion, skills CRUD, snapshots, metrics (Prometheus opcional + v2 JSON). El set cubre y excede los ~27 declarados. Sin endpoints muertos evidentes.

### Streaming/SSE
No existe. Cero matches de SSE/EventStream/text-event-stream en cli_server.rs. Todo es JSON request/response sincrono. Para DB local embebido esto es razonable hoy (YAGNI), pero export de datasets grandes y graph traversal profundo responden en un solo cuerpo sin chunking. Nota informativa, no bloqueante.
<!-- section break -->
---

## 5. Auth / Middleware (fortaleza del modulo)

Revisado adversarialmente (`auth_middleware`, cli_server.rs:605-773):

| Control | Estado | Evidencia |
|---|---|---|
| Comparacion de token constant-time | OK | `ct_eq` (linea 666) — sin timing attack |
| Rate limit de fallos de auth por IP | OK | `rate_limiter.record_failure` + 429 (lineas 641-654, 672) |
| Dev mode visible | OK | Sin API key permite todo PERO loguea warn por request (629-634); bypass hecho visible, correcto para local-first |
| Trusted proxies XFF | OK | `client_ip` solo honra `x-forwarded-for` si el peer esta en `trusted_proxies` (573-599) — no IP spoofing |
| Fail-closed identidad L2/L3 | OK | error de `resolve_identity` -> 401 sin leak interno (693-713) |
| RBAC por metodo (transport L1) | OK | POST/PUT/PATCH/DELETE -> Permission::Write (718-740) |
| Audit de eventos de auth | OK | fallos y exitos L2/L3 auditados; L1 success no (anti-flood, comentado en codigo) |
| Invariant violation defensivo | OK | AuthState ausente -> 401, no panic (611-623) |

Rate limiting de trafico: `GovernorLayer` con posture **fail-closed** (AUD-021, cli_server.rs:294-303) y burst diferenciado segun haya auth (REST-01).

Amarillo menor: `auth.token_role_map.get(token_val)` (linea 720) usa el token crudo como key de HashMap. No es un leak, pero si el map crece conviene hashear el token como key. No bloqueante.

---

## 6. Timeouts, concurrencia, graceful shutdown

| Mecanismo | Estado | Detalle |
|---|---|---|
| Body limit | OK | 1MB global, testeado (`body_limit_rejects_oversized`, cli_server.rs:2927) |
| Pool de concurrencia | OK | `ConnectionPool` con acquire timeout; `run_db_op` usa permit + spawn_blocking — handlers no bloquean el runtime |
| Circuit breaker | OK | Abierto -> 503 con Retry-After; half-open cierra — testeado (server.rs:343, 388) |
| **Request timeout** | **FALTA** | No hay `TimeoutLayer` (0 matches de TimeoutLayer/tower_http::timeout en cli_server.rs). Un handler atascado (ej. pagerank sobre DB grande) retiene la conexion indefinidamente; solo existe pool_acquire_timeout_ms para obtener permiso, no deadline de request. Recomendado: `tower_http::timeout::TimeoutLayer` configurable, excluyendo export/import si hace falta |
| Graceful shutdown HTTP | OK | SIGINT+SIGTERM (unix) -> `with_graceful_shutdown` -> flush en spawn_blocking (1960-1979) |
| Graceful shutdown TLS | OK | `axum_server::Handle::graceful_shutdown(10s)` + flush (1922-1943) |
| Telemetria shutdown | OK | `shutdown_telemetry()` tras flush (feature otel) |
| TLS | OK | rustls TLS1.2+1.3, ALPN h2/http1.1, valida exactamente 1 private key; testeado e2e con cert rcgen (server.rs:484) |

---

## 7. Tests e2e e integracion — que cubren y que no

### tests/e2e.rs (10 tests, socket real + reqwest)
Cubre: health/metrics publicos, roundtrip INSERT/FETCH/DELETE via `/api/v2/query` IQL, auth (401 / token valido / token incorrecto) sobre HTTP real, **persistencia across restart** (reabre StorageEngine y verifica dato), rate limit over socket, 400 por JSON invalido, threads de conversacion (creacion, append, thread_id invalido -> 400, requiere auth), skill listing vacio/filtrado/lean (sin leak de content).

Calidad: `wait_for_port` event-based (sin sleeps arbitrarios); helpers centralizados en `helpers/mod.rs`.

Amarillo: `test_e2e_rate_limit_over_http` acepta `200 || 429` — valida "el server responde", no el rate limit. Honestidad declarada en comentario, pero una regresion que desactive el governor pasaria desapercibida.

### tests/server.rs (22 tests, oneshot/router)
Auth en 5 modos, RBAC por rol (reader/writer/admin), rate limit on/off/burst, concurrencia paralela y semaforo chico, circuit breaker abierto y half-open, config TLS + server TLS real con query. Cobertura solida de middleware.

### tests/mcp_integration.rs (1 test compuesto)
Handshake MCP (protocolVersion 2024-11-05), tools_list incluye query_iql, dispatcher de tools (get_node_neighbors, query_iql INSERT). Ejercita `vantadb-mcp` directamente (no socket). Complementa, no duplica, la suite de mcp_tests del crate vantadb-mcp.

### Hueco de cobertura (conectado al hallazgo 8.1)
Ningun test e2e ejercita **busqueda textual/hibrida** por HTTP sobre DB fresca. El hueco es exactamente el que oculta la clase de bug MCP-01.

### Cargo.toml
publish=false correcto; dev-deps pesadas bien acotadas a test (reqwest rustls, rcgen, axum-server tls). Feature `sysinfo = []` vacia parece muerta — revisar si tiene consumidor o borrar.
<!-- section break -->
---

## 8. Hallazgos

### 8.1 ROJO (bloqueante de contrato): `ensure_indexes_current` no corre en el path HTTP — misma clase que MCP-01
- **Cadena de evidencia:**
  - `cli_server::run()` abre con `StorageEngine::open_with_config` (`src/cli_server.rs:1758`) y envuelve con `VantaEmbedded::from_engine` (1784).
  - `VantaEmbedded::from_engine` (`src/sdk/builder.rs:35-42`) **no** llama `ensure_indexes_current`; solo `open_with_config` del SDK lo hace condicional a `!read_only` (`builder.rs:104-106`).
  - Grep: cero matches de `ensure_indexes_current` en `src/cli_server.rs`. Ningun handler/middleware lo invoca.
  - Root cause documentado en MCP-01/AUD-044 (`docs/Backlog.md:578`): los puts escriben postings pero el estado del text index nunca se crea -> `ensure_text_index_query_ready` (`text_index.rs:18`) falla siempre en DB fresca.
- **Consecuencia esperada:** `/api/v2/query` con busqueda textual e hibrida, y `/api/v2/search`, fallan con "text_index not found: bm25" sobre DB fresca. El workaround manual existe (`/api/v2/maintenance/rebuild-index`) pero el contrato REST no deberia requerirlo.
- **Por que los tests no lo detectan:** e2e.rs solo usa IQL INSERT/FETCH/DELETE; ningun test hace search textual via HTTP.
- **Fix minimo:** llamar `db.ensure_indexes_current()` (idempotente) en `run()` tras abrir, o cambiar la apertura a `VantaEmbedded::open_with_config(config)` — un cambio de 1-3 lineas. Mas test e2e de regresion: put + text search por HTTP.
- **Verificacion sugerida antes de fixear:** levantar server contra DB vacia, POST put con texto, POST search textual -> confirmar el fallo (repro determinista).

### 8.2 AMARILLO: sin request timeout
Sin `TimeoutLayer`, una operacion lenta o colgada retiene conexiones indefinidamente (seccion 6). Para un server local el riesgo es bajo pero real con pagerank/centrality sobre keyspaces grandes. Fix estandar de tower-http, ~5 lineas + config.

### 8.3 AMARILLO: asercion debil en test de rate limit e2e
`200 || 429` (e2e.rs:296-300) no detectaria desactivacion accidental del governor. Sugerencia: con burst=1 conocido, forzar N requests rapidos y exigir al menos un 429.

### 8.4 AMARILLO menor: `middleware.rs` re-export redundante
`vantadb-server/src/middleware.rs` duplica re-exports ya presentes en `server.rs`. O se elimina, o se deja si hay consumidores externos del path `vantadb_server::middleware::...` — verificar antes de borrar.

### 8.5 VERDE: nota sobre feature `sysinfo = []`
Feature vacia en Cargo.toml:33-39 sin dependencias asociadas visibles. Verificar consumo o eliminar.

### 8.6 VERDE: main.rs MCP abre StorageEngine raw
Correcto hoy porque `vantadb_mcp::run_stdio_server` corre ensure internamente, pero el binario delega la garantia de indices a otro crate implicitamente. Si algun dia existe un segundo consumidor de ese handle raw, repetira MCP-01. Comentario en main.rs lo dejaria explicito.

---

## 9. Alternativas evaluadas (brainstorm)

1. **Mover cli_server.rs a este crate** vs status quo (canonico en core): mover rompe a todos los consumidores internos del core (`vanta-cli server`) y gana poco; el core ya depende de axum/tower para su binario CLI. Se descarta — el costo de migracion supera el beneficio cosmético.
2. **TimeoutLayer** vs pool timeout como unico control: el pool acota paralelismo, no latencia individual. Ambos son complementarios; se recomienda TimeoutLayer.
3. **ensure en run()** vs exponer rebuild-index como unico camino: igual decision que MCP-01 (opcion a elegida alli); consistencia manda — mismo punto de fix.

## 10. Recomendaciones priorizadas (iterate)

1. **P0** — Reproducir y fixear hallazgo 8.1 (ensure en path HTTP + test e2e de text search). Verificable con repro determinista.
2. **P1** — Agregar `TimeoutLayer` configurable (8.2).
3. **P2** — Endurecer test de rate limit (8.3); limpiar middleware.rs y feature sysinfo (8.4/8.5).
4. **P3** — Constructor `ServerState::new` para reducir duplicacion en helpers de test.

---

## 11. Scorecard

| Eje | Score | Justificacion |
|---|---|---|
| Arquitectura / canonicalidad | 9/10 | Binario thin sin duplicacion; division core/binario limpia |
| Seguridad (auth/middleware) | 9/10 | ct_eq, rate-limit fail-closed, trusted proxies, RBAC L1/L2/L3, audit |
| Contrato REST (/api/v2) | 6/10 | Superficie completa (~37 ops), pero busqueda textual presumiblemente rota en DB fresca (8.1) |
| Robustez HTTP (timeouts/shutdown) | 7/10 | Shutdown y TLS excelentes; falta request timeout |
| Tests | 7/10 | 33 tests bien disenados; hueco exacto donde vive el bug 8.1 |
| Higiene build/deps | 8/10 | Dev-deps acotadas, features claras; feature sysinfo muerta |

**Global: 7.5 / 10**

### Dictamen final
- **Veredicto: CAMBIOS REQUERIDOS** (hallazgo 8.1 bloquea el contrato REST de busqueda; resto aprobable tal cual)
- **Contrato:** no verificado end-to-end por el revisor (no se re-ejecuto la suite); verificado por lectura de codigo el gap 8.1. Comando de verificacion sugerido en 8.1.
- **DoD:** Task level — pendiente 8.1; Commit level — OK (convencionales, workspace lints); Release level — no aplica a este crate (publish=false, distribuido via workspace).

---

## Trazabilidad Backlog

Derivado a la fase **P32** de `docs/Backlog.md` (2026-08-23):

| Hallazgo | Tarea |
|---|---|
| 8.1 — `ensure_indexes_current` no corre en el path HTTP (misma clase que MCP-01) | **MOD-12** |
| 8.2 — Sin request timeout (`TimeoutLayer` ausente) | **MOD-13** |
| 8.3 — Aserción débil en test e2e de rate limit (acepta `200 \|\| 429`) | **MOD-14** |
| 8.4–8.6 — nits (`middleware.rs` re-export redundante, feature `sysinfo = []` vacía, StorageEngine raw en main.rs MCP) | **MOD-15** |
