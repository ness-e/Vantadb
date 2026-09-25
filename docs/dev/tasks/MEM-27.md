# Task: MEM-27 — vanta-proxy rate-limit + write-back + mem-command + reporting (P30 / Task 9)

## Estado
✅ COMPLETED — todos los steps verificados; suite vanta-proxy 52/52 (26 previos + 26 nuevos).

## Steps
1. ✅ DISCOVERY + Regla 0 (impacto mapeado abajo)
2. ✅ rate_limit.rs (D24+D35) + tests a/b
3. ✅ writeback.rs + tests c/d
4. ✅ mem_command.rs (D33) + test e
5. ✅ report.rs + test f
6. ✅ Wiring aditivo: server.rs pipeline (auth→rate-limit→mem-command→session→inject→forward), config.rs (`[mem_command] enabled`, `[writeback] persist_path`), main.rs flush SIGTERM/SIGINT, handlers pasan space_id, tests/proxy_wire.rs actualizado + 3 tests de integración nuevos
7. ✅ Verify mecánico completo exit 0:
   - `cargo check -p vanta-proxy` → exit 0
   - `cargo nextest run -p vanta-proxy` → 52/52 passed
   - `cargo fmt --check` → exit 0
   - `cargo clippy -p vanta-proxy --all-targets --no-deps -- -D warnings` → exit 0
8. ✅ Cierre: campaign_update_task_state completed (sin commit — regla del orquestador)

## Tests D19 (contrato)
- (a) sliding window: unit concurrente 8 threads×50 checks admiten exactamente `limit` (5) + integración wire con burst concurrente → 429 + Retry-After(1..=60) + x-ratelimit-{limit,remaining} + error type `rate_limit_error`; buckets independientes por spaceId×model; expiración de ventana probada con window corto.
- (b) fail-open degraded: flag → allow over-limit + warn log; recuperación del flag restaura enforcement.
- (c) retry backoff: with_l0_retry 3 intentos, base inyectable (10ms), recupera en 3er intento y mide espera ≥2 backoffs; exhaustión devuelve último error tras exactamente 3 intentos; schedule default [500,1000,2000] asertado.
- (d) flush: track que agota retries → enqueued + persistido a disco (label en JSON); flush(deadline) drena jobs sanos y conserva los que siguen fallando; main.rs llama flush(10s) tras shutdown graceful (SIGTERM unix + Ctrl+C).
- (e) mem-command: disabled by default (config default + test wire disabled→forwarded); enabled intercepta sync/help localmente (test wire); parser TDAM-fiel: case-insensitive, strict-args para help/sync, typo fallback, content array blocks conservativo, última user message.
- (f) reporting: TurnReport serializa a JSON con campos requeridos; hooks reciben cada emisión; model_from_body defaults "_"; emit bajo target `vanta_proxy::report`.

## Impacto mapeado (Regla 0)

### Archivos leídos completos
- `vanta-proxy/src/{lib,config,error,server,main,session}.rs`
- `vanta-proxy/src/handlers/{mod,openai,responses}.rs` (anthropic análogo)
- `vanta-proxy/Cargo.toml`, `tests/proxy_wire.rs`

### Referencias entrantes / salientes
- `server::AppState::process()` único punto de pipeline; tests construyen `ProxyConfig` con struct literal → actualizados aditivamente.
- TDAM refs verificadas en clon `97f9465`: parser.ts (strict args, KNOWN_COMMANDS), pending-writes.ts (trackWrite/withL0Retry/flush), guard.ts (fail-open).

### Veredicto
Cambios aditivos completados sin deps nuevas ni Redis. Sin unwrap/expect en código nuevo.

## Context Save Point
(ninguno — tarea completa)

## Notas de diseño
- Rate limit corre DESPUÉS de auth: tráfico no autenticado nunca quema cuota (401 tiene precedencia sobre 429).
- mem-command intercept corre ANTES de la resolución de sesión (los comandos funcionan sin header de sesión).
- Respuesta mem-command es envelope JSON plano (`{"object":"mem.command","message":...}`) en todos los protocolos — divergencia documentada vs builders específicos de TDAM.
- Persistencia de pendientes guarda labels (audit trail), no closures; flush replayea los jobs aún vivos en memoria.
