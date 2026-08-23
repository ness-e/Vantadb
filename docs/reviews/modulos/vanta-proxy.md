# Deep Module Review — `vanta-proxy`

> **Fecha:** 2026-08-22 · **Revisor:** ox-alpha (segunda opinión, contexto fresco — P2-01)
> **Alcance:** los ~20 archivos del crate (`src/` completo + `config.toml` + 3 suites de integración). Leídos en su totalidad: server, auth, session(+claude_code), inject, forward, rate_limit, sse_intercept, memory_tools, writeback, capture, mem_command, report, config, error, handlers×3, lib/main.
> **Evidencia de compilación/tests (ejecutada en esta sesión):**
> - `cargo check -p vanta-memory -p vanta-proxy --all-features` → ✅ Finished dev profile in 55.72s
> - `cargo test -p vanta-proxy` → ✅ todo verde: unitarios (auth 4, rate_limit 7, session 7, claude_code 12, sse_intercept 5, writeback 6, capture 4, mem_command 7, report 4, inject 5) + integración `pipeline.rs` 10, `tool_loop.rs` 5, `proxy_wire.rs` (~10, ver output parcial).

---

## 1. Veredicto ejecutivo

**Score: 8.0 / 10**

Crate sorprendentemente sólido para su edad. La separación de responsabilidades está bien trazada (handler → pipeline `process()` → forward), el fail-open/fail-closed de cada mecanismo está decidido y documentado (no accidental), y la cobertura de tests cubre los caminos que importan (wire e2e contra upstream mockeado, loop de tools, 502/504/429). Los hallazgos críticos son pocos y acotados; el más serio es de **honestidad funcional** (`mem:` commands que responden éxito sin hacer nada) y uno de **compatibilidad de producción** (body limit de axum de 2MB sobre payloads reales de Claude Code).

---

## 2. Superficie expuesta hoy (verificada por código, no por docs)

| Superficie | Estado | Evidencia |
|---|---|---|
| Crate API (lib.rs) | ✅ 14 módulos públicos (`auth`, `inject`, `server`, …) | `vanta-proxy/src/lib.rs:8-21` |
| `POST /{agent}/{spaceId}/v1/chat/completions` | ✅ expuesto, OpenAI Chat | `server.rs:343-346`, `handlers/openai.rs` |
| `POST /{agent}/{spaceId}/v1/messages` | ✅ expuesto, Anthropic Messages | `server.rs:347-350`, `handlers/anthropic.rs` |
| `POST /v1/responses` | ✅ expuesto, subset genérico sin `{spaceId}` (space_id = "") | `server.rs:351`, `handlers/responses.rs:26` |
| `GET /health` | ✅ sin auth (razonable para liveness) | `server.rs:342,355-357` |
| Tools inyectadas al modelo | ✅ `vanta_memory_capture`, `vanta_memory_search` (ejecutadas server-side en el loop SSE) | `inject.rs:26-35`, `memory_tools.rs` |
| `mem:` commands | ⚠️ parseo real, ejecución stub (ver 🔴 H-1) | `mem_command.rs:102-111` |
| MCP tools / Prometheus / billing | ❌ no existen en este crate — coherente con DEFERs del plan P30 (billing = server mode; métricas = single-process scope) | grep `rg "scene_query\|vanta_memory" src -g "*.rs"` del core: 0 hits |

**Nota de superficie:** el crate depende de `vanta-memory` y `vantadb` solo vía `default-features = false` (`Cargo.toml:26-27`) — grafo acíclico proxy → {memory, vantadb} respetado.

---

## 3. Arquitectura y patrones

**Pipeline único y explícito** (`server.rs:95-184`): `auth → rate-limit → mem-command → session → inject → [tool-loop] → forward`. Orden correcto y con justificación escrita: auth antes del limiter para que tráfico no autenticado no queme cuota ajena (`server.rs:134-136`). Todo handler es un wrapper fino sin lógica (`handlers/*.rs` ≤28 líneas).

**Zero-overhead gate bien puesto**: el loop de interceptación SSE solo se activa si el protocolo es OpenAI/Anthropic **y** el body anuncia nuestras tools (`server.rs:254-257`, gate en `memory_tools::announces`). Todo lo demás es passthrough byte-idéntico (testeado: `c_without_our_tools_passthrough_is_byte_identical`).

**Contrato D29 (inyección KV-cache safe)** implementado correctamente: solo posición system prompt (`system` Anthropic — string o array con insert en índice 0; primer mensaje `system` OpenAI; `instructions` Responses), nunca historia (`inject.rs:203-261`). Guard anti-doble-inyección via `starts_with(memory_block)`.

**Degradación consistente**: errores de storage en inyección → bloque vacío + debug log, nunca rompen el wire (`inject.rs:91-141`). Errores de tools del modelo → texto descriptivo que el modelo puede reaccionar, nunca 500 (`memory_tools.rs:67-77`).

**Puntos de diseño discutibles pero documentados** (aceptables):
- Loop agentic bufferiza rondas con nuestras tools: se pierde streaming incremental solo en esos turnos (D46 aceptado).
- Tool calls de cliente mezcladas con las nuestras en la misma ronda quedan sin respuesta upstream (`memory_tools.rs:146-148`, `ponytail:` documentado). Riesgo real pero raro; ver 🟡 M-4.

---

## 4. Seguridad (trust boundary crítica)

### ✅ Lo que está bien hecho
- **Auth fail-closed total (D34):** header ausente/vacío/desconocido → `Unauthorized`; no existe modo abierto (`auth.rs:70-79`). Testeado (`unknown_key_fails_closed_d34`).
- **Comparación constant-time** de user keys (`auth.rs:128-136`) — sin dependencias externas.
- **El user key NUNCA llega al upstream:** `x-vanta-user-key` en hop-by-hop strip list (`forward.rs:31`). Crítico y correcto.
- **SSRF acotado por construcción:** la ruta wire es constante por handler (`"/v1/messages"` etc.), nunca derivada del input del cliente; el destino es siempre `config.upstream.url` controlado por el operador. No hay fetcher remoto (fetch HTTPS quedó fuera de scope P30 según D30/D36 — coherente).
- **Rate limit antes del costo**: sliding window correcta bajo concurrencia (test de 8 hilos × 50 checks admite exactamente el límite), poison-recovery consciente (`rate_limit.rs:92-95`).
- **Secrets:** api_key solo desde config TOML; no hay logging del header de auth ni de bodies completos (solo `tracing::debug!` de agent/space_id). El persist file guarda labels, no contenido.

### Hallazgos de seguridad

**🔴 C-1 — `mem:` commands responden éxito simulado (honestidad funcional / abuso de confianza).**
`mem_command.rs:104-107`: `"sync"` responde `"✅ Session memory refreshed (skills / knowledge / tasks)."` **sin tocar ningún store**, y `"create-skill"` responde `"✅ Skill creation queued from prompt: ..."` **sin encolar nada**. Un usuario que confía en ese check verde cree que su memoria se refrescó o que un skill se creó cuando no pasó nada. Es un comando opt-in (`enabled=false` default), lo que baja la severidad operativa, pero el texto es una afirmación falsa de efecto secundario. Fix mínimo: cambiar la redacción a estado real ("mem:sync is not wired to a backend yet") o conectarlo a `flush_session` del pipeline (que ya existe y sería ~5 líneas).

**🟡 S-1 — Sin límite de tamaño de request visible y DefaultBodyLimit de axum en el camino.**
Los handlers extraen `bytes::Bytes` directamente; axum 0.8 aplica `DefaultBodyLimit` de **2 MB** por defecto si nadie lo cambia, y no hay `DefaultBodyLimit::disable/max` en `router()`. Conversaciones largas de Claude Code superan 2 MB con frecuencia → 413 silencioso en producción. Recomendación: fijar explícitamente `DefaultBodyLimit::max(N)` con N elegido (p. ej. 32 MB) y testearlo.

**🟡 S-2 — Bufferizado completo de streams SSE en memoria sin cap.**
`sse_intercept::drain` acumula la respuesta upstream completa en RAM (`StreamCapture.full` + chunks) para toda ronda del tool-loop (`server.rs:288-291`). Un upstream comprometido/malicioso (o simplemente un modelo desbocado con `max_tokens` enorme) puede crecer la memoria sin techo. Cap recomendado (p. ej. 64 MB) con replay parcial o error tipado.

**🟢 Nota N-1 —** `ct_eq` early-exit en length filtra longitud de key; estándar y aceptable dado el scan previo.
**🟢 Nota N-2 —** `upstream.url` default apunta a `http://127.0.0.1:8096` (self-loop) y no valida esquema; un operador descuidado mandaría Authorization Bearer por HTTP claro. Validar https:// en producción o warn al cargar config.

---

## 5. Lógica sospechosa (state machines, locks, retries)

**Session state machine (session.rs)** — correcta y bien testeada:
- Transiciones monotónicas `team→agent→task` validadas contra entity store **antes** de mutar (`advance`, `session.rs:122-159`); rechazo de skip/backwards con sesión intacta.
- TTL solo para estados pendientes; `Task` terminal nunca expira (test `pending_ttl_sweeps_but_task_persists`).
- Sweep lazy O(n) por request sobre el HashMap completo: aceptable ahora, pero combinado con la ausencia de cap es crecimiento sin techo (el propio código lo marca: `// ponytail: HashMap unbounded`, `session.rs:161`).
- **Observación:** la state machine existe y funciona pero **nadie la consume** — `ensure()` se llama en el pipeline (`server.rs:171`) pero ninguna ruta llama `advance()` fuera de tests. Hoy es infraestructura muerta en producción: la sesión solo sirve como clave para inyección/captura. No es bug, pero es superficie mantenida sin beneficio actual (flag ponytail `yagni` hasta que exista el caller).

**Rate limiter** — matemática correcta (retry_after = oldest + window − now, min 1s), buckets sin eviction: con espacio×modelo ilimitado el mapa crece monótonamente. 🟡 M-3.

**Write-back (writeback.rs)** — el diseño más fino del crate:
- Retry 500ms→1s→2s, dead-letter en cola persistida, flush graceful en shutdown con deadline (`main.rs:47`).
- Clave de idempotencia: los jobs capturan `key = {now_ms}-{seq}` al crear el job (`capture.rs:49-53`), así un flush que reintenta tras timeout hace upsert del MISMO record — no duplica. Bien pensado (probablemente no casual: `TURN_SEQ` está documentado).
- Edge case revisado y OK: `flush` con timeout global pierde el registro de qué jobs individuales terminaron OK → los reintenta todos; por el punto anterior es seguro.
- Persistencia full-rewrite por fallo marcada con techo documentado (`ponytail:`, `writeback.rs:105`).

**Tool loop (server.rs:241-336)** — cap duro de 3 iteraciones (D48) con salida correcta: al alcanzar el cap devuelve la última respuesta tal cual (que termina en tool_use de nuestras tools — el cliente verá tool calls que no conoce; caso borde aceptado y documentado). Reconstrucción de historia fiel (todos los tool calls acumulados van al assistant message, solo los nuestros reciben resultado sintetizado). Parsing OpenAI (merge por `index`) y Anthropic (`content_block_*`, `input_json_delta`) correcto y testeado contra shapes reales.

**Capture (capture.rs)** — extracción del último texto usuario delega al adaptador Claude Code que escanea backwards saltando `<system-reminder>` (MEM-57); testeado. Solo captura si el último mensaje es `user` y status 200.

---

## 6. Flujos end-to-end

**Request → inyección → loop SSE → write-back** (verificado leyendo el path completo + tests de integración):

```
POST /{agent}/{spaceId}/v1/messages
 → authenticate(headers)            [fail-closed]
 → limiter.check(spaceId, model)    [429 + Retry-After]
 → mem_command.parse (opt-in)
 → session_key_from_headers         [sin header → forward_raw verbatim]
 → sessions.ensure(key)
 → build_memory_block(persona+scene) [best-effort, "" si vacío]
 → inject_into(body)                [system-only + merge_tools]
 → announces(tools)? ──no──→ forward_raw (streaming puro)
        │yes
 → loop≤3: forward → drain(SSE) → reconstruct message
        ├─ sin memory tools → replay chunks verbatim (streaming intacto)
        └─ con tools → execute (capture→writeback.track fire-and-forget;
                        search→perform_auto_recall sync) → append_exchange → re-forward
 → status 200 → capture_turn → writeback.track("turn:{session}") [L0]
 → reporter.emit(JSON line)
```

Sin huecos encontrados en el flujo: cada rama de error tiene salida tipada (502/504/400/401/429) y el write-back jamás bloquea la respuesta. Los tests `proxy_wire.rs` y `tool_loop.rs` ejercitan exactamente estas ramas contra upstream mockeado real.

---

## 7. Completudes vs plan P27 F6 (segunda iteración)

Verificado contra `docs/plans/archive/2026-08-21-vanta-proxy-knowledge.md`:

| Ítem | Estado plan | Estado código | ¿Falta legítima? |
|---|---|---|---|
| Auth RBAC local (D25/D34) | DO | ✅ implementado + tests | — |
| Rate-limit in-process sliding window (D24/D35) | DO | ✅ implementado + tests | — |
| Session state machine (D26) | DO | ✅ implementada (⚠️ sin caller de `advance`) | — |
| Inyección system-prompt KV-safe (D29) | DO | ✅ + tools L0/L1 | — |
| Agentic tool loop + cap (D46/D48) | DO | ✅ + tests e2e | — |
| Write-back crash-safe + flush SIGTERM (MEM-27) | DO | ✅ + persist audit trail | — |
| `mem:` commands (D33) | DO, disabled default | ⚠️ parseo OK, **ejecución stub** | 🔴 C-1 |
| Billing/quota | **DEFER** (server mode, plan P27) | ❌ ausente | ✅ DEFER legítimo — no marcar como falta |
| Métricas/Prometheus | **No portado** (single-process scope, documentado en TDAM-parity del worker) | ❌ solo JSON log lines + hooks no conectados | ✅ decisión documentada |
| Fetcher HTTPS wiki (SSRF blocklist D30) | **Fuera de scope P30** (D36: paths locales) | ❌ ausente | ✅ DEFER legítimo |
| SDK sub-clientes (MEM-36) | **DEFER** a campaña bindings | ❌ ausente | ✅ DEFER legítimo |

Conclusión: las únicas ausencias son DEFERs explícitos del plan. No hay ítems DO del plan sin implementar.

---

## 8. Tests

**Presentes y sustanciales** (raridad en crates jóvenes): ~60 unitarios + 3 suites de integración con upstream mockeado real (axum-on-TCP, no mocks de trait): wire e2e por protocolo, 502/504 mapeados, streaming passthrough sin buffering, byte-identidad sin nuestras tools, cap del loop, búsqueda síncrona Anthropic. Unit tests de concurrencia real (hilos compartiendo el limiter). Tests de retry usan polling en vez de sleeps fijos donde importa (`capture.rs:167-174`).

**Huecos de cobertura detectados:**
1. No hay test del body >2 MB / `DefaultBodyLimit` (relacionado a S-1).
2. No hay test de drain con stream que crece sin límite (S-2).
3. No hay test de que `x-vanta-user-key` no llegue al upstream (la garantía vive solo en `forward.rs:31`; un test de wire que afirme la ausencia del header en el upstream capturado cerraría el contrato con evidencia).

---

## 9. Ponytail-audit (¿sobre-ingeniería?)

Crate contenido. Hallazgos menores:

- `yagni:` Session state machine (`session.rs` Stage/advance/TTL) mantenida sin ningún caller de `advance()` en producción — 330 líneas + tests sirviendo solo a tests. Reemplazo: nada (dejarla dormida está OK, pero decidirlo explícitamente). [`session.rs`]
- `delete:` `Reporter::add_hook`/`ReportHook` — mecanismo de fan-out sin un solo hook registrado en el binario. YAGNI hasta que exista backend. [`report.rs:31-50`]
- `shrink:` `handlers/{openai,anthropic,responses}.rs` son 3 wrappers idénticos salvo enum+path — podría ser una función parametrizada; a 27 líneas cada uno, no urge. [`handlers/`]
- `stdlib/native:` nada reinventado de más — ct_eq a mano evita una dep (justificado), SSE parser manual documentado como deliberado sobre streams propios.

Net: ~-80 líneas posibles, 0 deps. El crate NO está sobre-ingenierado; los `ponytail:` comments marcan techos reales con upgrade paths (buena higiene).

---

## 10. Hallazgos consolidados

| # | Sev | Hallazgo | Evidencia |
|---|---|---|---|
| C-1 | 🔴 | `mem:sync`/`mem:create-skill` responden éxito simulado sin efecto | `mem_command.rs:104-107` |
| S-1 | 🟡 | Body limit implícito de axum (2 MB default) sin configuración explícita → riesgo 413 en CC largo | `server.rs:340-353` (sin `DefaultBodyLimit`) |
| S-2 | 🟡 | Drain SSE sin cap de memoria en tool-loop | `sse_intercept.rs:60-69`, `server.rs:288` |
| M-3 | 🟡 | Buckets del rate limiter y sesiones sin eviction/cap | `rate_limit.rs:39`, `session.rs:161` (este último ya auto-documentado) |
| M-4 | 🟡 | Mezcla our-tool + client-tool en una ronda deja tool_call sin responder → 400 upstream probable; techo documentado pero sin mitigación (p. ej. responder client-tools con "handled client-side") | `memory_tools.rs:146-148` |
| M-5 | 🟡 | Falta test de wire que afirme ausencia de `x-vanta-user-key` hacia upstream | `forward.rs:29-31` |
| N-2 | 🟢 | `upstream.url` sin validación de esquema; default self-loop http | `config.rs:120` |
| N-3 | 🟢 | `/v1/responses` comparte bucket de rate-limit con space_id="" | `handlers/responses.rs:17-18` |
| N-4 | 🟢 | Timeout cliente-level corta streams >600s (documentado con upgrade path) | `forward.rs:81-82` |

## 11. Score y desglose

| Eje | Puntaje | Comentario |
|---|---|---|
| Correctez | 8.5 | State machines, retries y parsing SSE correctos; edge cases pensados |
| Seguridad | 7.5 | Fail-closed, constant-time, secret hygiene buenos; C-1 y falta de body cap restan |
| Arquitectura | 9 | Pipeline único, gates zero-overhead, contratos documentados |
| Readability | 8.5 | Doc-comments con decisiones (D24-D48) trazables al plan |
| Performance | 7.5 | Scan O(10k) usuarios por request (ver abajo*), drains sin cap |
| Tests | 9 | Integración con wire real; faltan 3 tests de contrato señalados |
| **Global** | **8.0** | |

\* *Nota performance heredada de MEM-05:* `resolve_user_key` escanea hasta 10.000 entidades usuario por request (`auth.rs:23,88-90`) — lineal en el hot path. Con el tamaño actual es irrelevante; con miles de usuarios reales será el primer cuello de botella. Índice por `user_key` o cache en memoria cuando toque.
