# MOD-07 — Notifications JSON-RPC sin `id` rechazadas como -32700 espurio

> **Estado:** ⏳ IN PROGRESS · **Appetite:** max 2h · **Esfuerzo:** 🟢 · **Prioridad:** 🔴
> **Plan:** docs/plans/2026-08-23-backlog-triage.md (Task 2, Wave 1)
> **Workflow:** bug-fix (localizing→planning→implementing→testing→review→accept→close)

## Bug

Notifications JSON-RPC (requests SIN campo `id`) son rechazadas como error -32700
espurio → rompe handshake con clientes MCP estrictos. El spec MCP obliga al cliente
a enviar `notifications/initialized` (sin id) tras el initialize; el server responde
con parse error en vez de silencio.

## Root Cause (systematic-debugging Phase 1-3 — verificado en fuente)

1. `vantadb-mcp/src/protocol.rs:11` — `RpcRequest.id: Value` es campo **requerido**.
2. Línea de notificación (`{"jsonrpc":"2.0","method":"notifications/initialized"}`)
   falla `serde_json::from_str::<RpcRequest>` en `vantadb-mcp/src/server.rs:102`
   ("missing field `id`").
3. La rama Err (server.rs:104-117) emite `{"id":null,"error":{-32700 Parse error}}`
   — respuesta espuria a lo que no es un request.

**Spec (Regla 0, verificado via webfetch modelcontextprotocol.io lifecycle 2025-06-18):**
- `notifications/initialized`: client→server tras initialize; el server NO responde.
- `notifications/cancelled`: client→server para cancelar requests (este server no
  trackea requests in-flight cancelables — spawn_blocking sin registro por id).
- JSON-RPC 2.0 §4.1: notification = request sin `id`; responder está PROHIBIDO.
- `id: null` explícito sigue siendo un REQUEST (spec permite null id) — debe
  seguir respondiéndose.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `vantadb-mcp/src/protocol.rs` (24L, fuente verbatim
  codegraph), `vantadb-mcp/src/server.rs` (283L: run_stdio_server 27-176,
  write_json 179-194, dispatch_request 198-283), `vantadb-mcp/src/lib.rs` (46L),
  `vantadb-mcp/Cargo.toml` (25L), `tests/mcp_tests.rs` (patrones de test, setup_storage).
- **Referencias hacia dentro:** `RpcRequest` tiene exactamente 2 callers, ambos en
  server.rs (deser línea 102 + uso en dispatch_request:199) — codegraph blast radius
  confirmado. `run_stdio_server` exportado por lib.rs:40, llamado solo desde el binario.
- **Referencias salientes:** protocol.rs usa serde/serde_json únicamente. Ningún
  otro crate depende de RpcRequest (pub(crate)).
- **Tests existentes:** tests/mcp_tests.rs cubre handlers directamente; NINGÚN test
  ejercita el loop stdio ni la deserialización del wire format (gap que permitió el bug).
- **Veredicto:** cambio acotado a 2 archivos + tests unitarios en src/server.rs.
  Sin cambios de API pública. Riesgo bajo.

## Spec

### Cambios

1. **protocol.rs:** `id: Value` → `#[serde(default)] id: Option<Value>`.
   - Ausente ⇒ `None` ⇒ notification. Presente (incluido null) ⇒ `Some`.
2. **server.rs:** extraer loop de lectura (líneas 80-169) a fn genérica
   `serve_lines<R: AsyncRead+Unpin, W: AsyncWrite+Unpin>(storage, config, semaphore,
   metrics, running, reader: R, writer: W)` — refactor puro para poder testear el wire
   format con duplex pipes in-memory (run_stdio_server hardcodea stdin/stdout).
   - `run_stdio_server` delega pasando stdin/stdout.
   - Routing nuevo tras deser exitoso:
     - `req.id == None` ⇒ notification: log debug + `continue` SIN escribir nada.
       Conocidas (`notifications/initialized`, `notifications/cancelled`) y
       desconocidas se descartan silenciosamente (ninguna tiene side-effect en este
       server; documentado en comentario).
     - `Some(id)` ⇒ versión-check + dispatch + respuesta con ese id (comportamiento
       actual intacto). Version-error usa `req_id.clone()` (branch diverge).

### Invariantes (no romper)

1. NUNCA emitir respuesta a un mensaje sin `id` (JSON-RPC 2.0 §4.1 / regla tarea).
2. `-32700` SOLO para JSON malformado/no-deserializable (ej. falta `method` o
   `jsonrpc`), nunca para notificaciones válidas.
3. Requests CON id (string/número/null) siguen respondiéndose igual que hoy.
4. Sin unwrap nuevos; errores propagados como hoy.

## Steps

### Step 1 ✅ DONE — RED: extraer serve_lines + tests de contrato
- Refactor puro hecho: loop movido a `serve_lines<R, W>` genérica (+ `write_json`
  genérico); `run_stdio_server` delega con stdin/stdout.
- 4 tests agregados en `server::tests` vía `tokio::io::duplex` (sin spawn: el
  future no es Send por `EnteredSpan` cruzando await en dispatch_request).
- RED verificado: `notification_without_id_is_not_answered` FALLÓ reproduciendo
  el bug exacto (`got: {"error":{"code":-32700,"message":"Parse error: missing field \`id\`..."}}`).

### Step 2 ✅ DONE — GREEN: fix Option<Value> + routing notifications
- protocol.rs: `#[serde(default, deserialize_with = "keep_explicit_null")] id: Option<Value>`
  — helper necesario porque serde mapea `"id": null` explícito igual que ausencia;
  JSON-RPC 2.0 distingue ambos.
- server.rs: filtro `let Some(req_id) = &req.id else { debug!; continue }` antes del
  version-check; span `%req.id` → `?req.id`.
- `cargo nextest run -p vantadb-mcp`: 37/37 GREEN.

### Step 3 ✅ DONE — VERIFY full + commit + cierre
- `cargo fmt --check` ✅ · `cargo clippy -p vantadb-mcp --all-targets --all-features -- -D warnings` ✅
- FASE SECURITY: notificaciones desconocidas nunca llegan a handlers (solo log+drop,
  testeado); 0 bytes de respuesta por notification (antes -32700 = amplificación);
  sin unwrap nuevos en prod code; sin dependencias nuevas (cargo audit N/A).
- PERFORMANCE N/A: loop stdio I/O-bound, sin hot path vector/engine (Regla 9 no aplica).
- Commit: `fix(mcp): MOD-07 acepta notifications JSON-RPC sin id — handshake clientes estrictos`

## Context Save Point

- Tarea COMPLETA — sin trabajo pendiente.
- Nota para review: `dispatch_request` sostiene `EnteredSpan` (no-Send) a través de
  `.await` — el future no es `Send`; latente si alguien alguna vez hace tokio::spawn
  del server loop. Fuera de scope aquí; candidato a Backlog si molesta.
- Nota: `notifications/cancelled` se acepta pero no cancela nada (spawn_blocking sin
  registro por request-id) — decisión documentada en comentario del código.
