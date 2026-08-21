# Plan de Ejecución: Vanta Proxy + Knowledge (F6+F7) — proxy transparente + wiki/code-graph

> **Campaign ID:** e88295b0-974b-4009-b291-8c479d19b8a9
> **Inicio:** 2026-08-21
> **Estado:** ⏳ EN PROGRESO (4/9 tareas)
> **Fuente:** `docs/Backlog.md` filas MEM-25..33 + `docs/research/tdam/07-proxy.md` + `08-knowledge-panel-sdk.md` + `06-metadata-acl.md` (quota diferido) + SYNTHESIS §2.3/§3 + decisiones del usuario (2026-08-21)
> **Predecesores:** P27 F1-F4 ✅ 24/24 (`docs/plans/archive/2026-08-18-vanta-memory.md`) · P29 F5 ✅ 9/9 (`docs/plans/archive/2026-08-21-vanta-context-engine.md`) — crate vanta-memory completo (L0-L3/recall/context_engine/offload/gateway/seed/genlog), suite 430/430
> **Modo:** waves por dependencias — Wave 0 (fundaciones independientes) → Wave 1 (proxy wire + ingest) → Wave 2 (ciclo proxy + tools wiki + callback) → Wave 3 (rate-limit/write-back).

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 9 |
| 🟡 DEFER | 1 (MEM-36 SDK sub-clientes → campaña bindings propia; backward-compat 100% exige su propio appetite) |
| ❌ SKIP | 0 |
| 🔴 BLOQUEADO | 0 |

**Objetivo:** cerrar el roadmap TDAM — **F6** `vanta-proxy` (binario opcional Rust: proxy transparente de 3 protocolos wire con ciclo inject→forward→write-back) y **F7** knowledge (wiki store state machine + ingest concurrente + callback S2S + 12 tools MCP query-only sobre graphrag propio).

**Decisiones fijadas por este plan (no re-debatir en DISCOVERY):**
- **D24:** rate-limit = sliding window **in-process** por `spaceId×model`, fail-open consciente (TDAM guard.ts:40-51), SIN Redis. 429 con Retry-After + headers x-ratelimit-*.
- **D25:** auth del proxy = RBAC local entity_* con user_key (MEM-04/05 ya implementados) — NO Gateway remoto `/v3/meta/auth/verify`.
- **D26:** sessionKey desde headers (paridad TDAM session-key.ts:9-19: x-conversation-id → x-session-id → x-claude-code-session-id → x-chat-id → x-thread-id); state machine local team→agent→task contra entity_* locales, TTL 30 min SOLO estados pending.
- **D27:** F7 = **worker único** (research 08 §6: "1 servicio, no 2" — NO copiar KS+Panel separados). Wiki store state machine LLM-free en core `src/wiki/`; ingest con LLM en `vanta-memory/src/ingest/`.
- **D28:** tools code_* sobre graphrag PROPIO (`src/graph.rs` bfs/dfs/topological + graphrag existente) — NO `@colbymchenry/codegraph`.
- **D29:** inyección L2/L3 en system prompt; L0/L1 expuestos como tools (no invalidar KV-cache — TDAM README:28).
- **D30:** SSRF blocklist https-only **NO desactivable** cuando se implemente fetch remoto (research 08 §7 advierte no propagar el env-off de TDAM). **Con D36 (paths locales), el fetcher HTTPS queda FUERA de scope P30** — diferido documentado hasta que haya fuentes git.
- **D31:** config del proxy en **TOML** (upstream URL/apiKey, puerto, rate-limits, features).
- **D32:** callbacks de progreso del ingest = **canal interno (trait/callback) + polling `wiki_status(run_id)`** — sin HTTP. El desktop lo puentea a eventos Tauri; el CLI hace poll.
- **D33:** mem-command = **mismos 3 de TDAM** (`mem:sync | mem:create-skill | mem:help`), deshabilitado por defecto.
- **D34:** auth del proxy **obligatoria por defecto** (user_key contra RBAC entity_*; toda request sin key válida → 401).
- **D35:** rate-limit default **60 req/min** por spaceId×model, configurable.
- **D36:** fuentes del wiki v1 = **paths locales** de .md (sin red). Fetcher HTTPS/git diferido (solo tendría sentido para code-graph futuro).
- **D37:** riesgos aceptados conscientemente y confirmados por el usuario (2026-08-21): rate-limit multi-instancia, chars/3 subestima CJK, keyword-overlap sin vectores — los 3 con upgrade path documentado.
- **Estrategia sub-agentes:** ejecutar y monitorear — si primer-intento <50%, parar e investigar el patrón vanta-worker antes de seguir.
- **Principios heredados (vigentes):** P4 LLM opcional · P7 prompts inglés · sanitización namespace/keys · D19 tests por tarea · sin unwrap/expect en producción · errores tipados #[non_exhaustive] · conventional commits · verify mecánico del lead por tarea.

Status: ⬆️ uphill = 0 (todas las decisiones cerradas: D21-D37) · ⬇️ downhill = ~35 steps estimados

---

## Tasks

### Task 1: MEM-32 — MCP tools code_* query-only (8 tools sobre graphrag propio)
- **Appetite:** max 1d
- **Esfuerzo:** 🟢 | **Prioridad:** 🟠
- **Archivos clave:** `vantadb-mcp/src/code.rs` (crear), `vantadb-mcp/src/handlers/tools.rs` (editar wiring)
- **Verificación real:** ✅ CÓDIGO-REAL — `src/graph.rs` bfs_traverse:61 / dfs_traverse:234 / topological_sort:258 existen; graphrag existe; `vantadb-mcp/src/handlers/tools.rs` existe; costo = solo exposición (backlog row)
- **Gate Justificación:** barato y visible; D28 elimina la dependencia externa de TDAM
- **Gate Result:** ✅ DO
- **Contrato: cargo check -p vantadb pasa; tests D19 — evidencia: nextest wiki:: 24/24 exit 0, fmt exit 0, clippy -D warnings exit 0; commit e4767c0a
- **Pre-mortem:** (1) semántica de impact/callers difiere entre codegraph de TDAM y graphrag propio → mapear cada tool a la primitiva local equivalente y documentar el mapping; (2) tools sin grafo cargado → error claro, no panic
- **Stop conditions:** si impact requiere análisis que graphrag no soporta → exponer stub con error "not supported" documentado (no inventar semántica)
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟡×🟡 | mapping tool↔primitiva imperfecto | tabla de mapping en task file; docs en MEM-38-style gate final | DISCOVERY |
  | 🟢×🟡 | grafo vacío en tests | fixture seedeado reutilizable | primer test |
- **Cynefin:** 🟦 obvio
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 3 steps
- **Estado:** ✅ COMPLETED
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-32.md`
- **Branch:** | **Commit:**
- **Iteraciones:** | — | — | — | — |
- **Notas:** Ruta: vanta-worker. TDAM ref: `MemoryKnowledge/src/mcp/tools.ts:25-223` (12 tools; portar las 8 code_*).

### Task 2: MEM-28 — Wiki store + state machine pending→ready (core, LLM-free)
- **Appetite:** max 1d
- **Esfuerzo:** 🟡 | **Prioridad:** 🟠
- **Archivos clave:** `src/wiki/mod.rs` (crear), `src/wiki/store.rs` (crear), `src/wiki/state.rs` (crear), `src/lib.rs` (wiring)
- **Verificación real:** ✅ CÓDIGO-REAL — `src/wiki/` NO existe; TDAM refs verificadas: state machine `pending → processing(scanning/ingesting) → ready|failed(+sync_error ≤500 chars)` (wiki-service.ts:5-7), `run_id = randomUUID()` por build (:1026), re-ingest busy → 409 si pending/processing (:272-288), frontmatter `locked:true` inyectado (:1164-1183), cascade delete (:827-859), dedup por path canónico type+title (:392-410)
- **Gate Justificación:** fundación F7 LLM-free en core (patrón InternalMetadata/entity_* ya existe — MEM-03/12)
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vantadb` pasa; tests D19: (a) create → pending; (b) ingest en pending/processing → 409-equivalente; (c) transición completa pending→processing→ready con run_id; (d) fallo → failed con sync_error truncado 500; (e) dedup path canónico; (f) locked:true en páginas gestionadas + cascade delete"
- **Pre-mortem:** (1) estado en core vs vanta-memory → D27 lo fija: store LLM-free en core, persistencia vía patrón InternalMetadata; (2) state machine sin control de concurrencia → CAS/optimistic lock patrón MEM-06
- **Stop conditions:** si el store requiere SQL/schema nuevo en core → usar partición InternalMetadata (patrón scene MEM-12) y documentar
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟡×🟠 | carreras entre ingest calls | optimistic lock + test de carrera | test b |
  | 🟢×🟡 | sync_error sin truncar crece ilimitado | truncate 500 chars en setter | diseño |
- **Cynefin:** 🟨 complicado — state machine conocida, persistencia a decidir detalle
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 4 steps
- **Estado:** ✅ COMPLETED
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-28.md`
- **Branch:** | **Commit:**
- **Iteraciones:** | — | — | — | — |
- **Notas:** Ruta: vanta-worker. Toca core `vantadb` — leer `.opencode/rules/core-engine.md` + api-contract.md antes de editar.

### Task 3: MEM-29 — Fuentes locales del wiki + chunker 12k/400
- **Appetite:** max 1d
- **Esfuerzo:** 🟢 | **Prioridad:** 🟠
- **Archivos clave:** `src/wiki/sources.rs` (crear — scanner de paths locales), `src/wiki/chunker.rs` (crear)
- **Verificación real:** ✅ CÓDIGO-REAL — no existen; TDAM refs: chunker defaults 12000/400 (chunker.ts:19-20), SOURCE_CHAR_BUDGET=28000 (ingest-v2/index.ts:78)
- **Gate Justificación:** fundación del ingest; D36 fija paths locales (sin red) — el fetcher HTTPS/SSRF queda FUERA de scope P30 (diferido documentado hasta que haya fuentes git; el riesgo 🔴×🔴 SSRF desaparece de esta campaña)
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vantadb` pasa; tests D19: (a) scanner descubre .md en path local recursivo; (b) chunker 12000/400 produce chunks esperados; (c) SOURCE_CHAR_BUDGET 28000 respeta; (d) boundaries sin corromper estructura; (e) path inexistente/fuera de raíz permitida → error claro (path traversal guard)"
- **Pre-mortem:** (1) path traversal (../ fuera de la raíz del wiki) → canonicalizar y validar prefijo; (2) archivos binarios/no-.md mezclados → filtrar por extensión + skip con log
- **Stop conditions:** ninguno previsto — tarea mecánica
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟡×🟠 | path traversal lee fuera de la raíz | canonicalize + starts_with(raíz) + test | test e |
  | 🟢×🟡 | symlinks escapando la raíz | seguir solo files regulares post-canonicalize | diseño |
- **Cynefin:** 🟦 obvio
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 3 steps
- **Estado:** ✅ COMPLETED
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-29.md`
- **Branch:** | **Commit:**
- **Iteraciones:** | — | — | — | — |
- **Notas:** Ruta: vanta-worker. SECURITY phase (path traversal es trust boundary). Fetcher HTTPS+SSRF diferido — cuando se implemente, aplicar D30 (no desactivable).

### Task 4: MEM-25 — vanta-proxy crate + 3 protocolos wire verbatim
- **Appetite:** max 3d
- **Esfuerzo:** 🔴 | **Prioridad:** 🔴
- **Archivos clave:** `vanta-proxy/Cargo.toml` (nuevo crate workspace, fuera de default-members), `vanta-proxy/src/{main,server,handlers/openai,handlers/anthropic,handlers/responses}.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — `vanta-proxy/` NO existe; axum 0.8 disponible (vantadb-server ya lo usa); TDAM refs: rutas primarias server.ts:307,312; 3 protocolos OpenAI Chat / Anthropic Messages / Responses API; forward timeout 600s (config.ts:10); upstream caído → 502
- **Gate Justificación:** alta adopción coding agents (SYNTHESIS fila 8); D5 decisión previa: crate aparte
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vanta-proxy` pasa; tests D19 con upstream mockeado (axum test server): (a) /v1/chat/completions forward verbatim (body/headers intactos); (b) /v1/messages ídem; (c) /v1/responses ídem (subset mínimo documentado); (d) upstream timeout → 504/502 claro; (e) upstream caído → 502; (f) /health; (g) streaming passthrough (SSE intacto)"
- **Pre-mortem:** (1) Responses API de TDAM acoplada a Codex/WorkBuddy handlers → portar SOLO el subset genérico /v1/responses, documentar recorte; (2) SSE streaming por axum → usar body stream passthrough sin buffering; (3) ~~config model~~ → **D31: TOML** (`vanta-proxy/config.toml` con serde).
- **Stop conditions:** appetite 3d excedido → entregar OpenAI+Anthropic (Responses a tarea propia ⬛)
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟠×🔴 | proxy rompe streaming SSE (buffering) | test de streaming passthrough desde el día 1 | test g |
  | 🟡×🟠 | Responses API scope creep | subset mínimo documentado; adapters NO (research §7) | DISCOVERY |
  | 🟡×🟡 | headers hop-by-hop mal reenviados | whitelist de headers; test | diseño |
- **Cynefin:** 🟨 complicado — protocolos conocidos, detalles de streaming analizable
- **Top 3 riesgos:** SSE, scope Responses, headers
- **Uphill/Downhill:** ⬆️ 0 (D31 TOML) · ⬇️ 5 steps
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-25.md`
- **Branch:** | **Commit:**
- **Iteraciones:** | — | — | — | — |
- **Notas:** Ruta: vanta-worker. Cargo.toml raíz: agregar a members pero NO default-members (patrón vanta-memory/server). Sin deps nuevas más allá de axum/tokio ya en workspace.

### Task 5: MEM-30 — Ingest merge serial + límite concurrencia LLM global
- **Appetite:** max 2d
- **Esfuerzo:** 🟡 | **Prioridad:** 🟠
- **Archivos clave:** `vanta-memory/src/ingest/mod.rs` (crear), `merge.rs` (crear), `worker.rs` (crear)
- **Verificación real:** ✅ CÓDIGO-REAL — ingest/ NO existe; depende de Tasks 2+3; TDAM refs: commitCandidates SERIAL (ingest-v2/index.ts:211-283), mergePage bajo globalLlmLimit pLimit(5) clamp 1-20 (module.ts:35, config.ts:104-107), fallo por página no bloquea resto, ensureSources fuerza frontmatter sources (:368-375), STRUCTURAL_FILES protegidos (:69-75)
- **Gate Justificación:** núcleo del valor F7 (índice wiki); dependencias Wave 0 listas
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vanta-memory` pasa; tests D19: (a) chunks → candidates agregados por relPath; (b) merge serial por página bajo límite global configurable (default 5, clamp 1-20); (c) fallo de merge en página N no bloquea N+1..; (d) ensureSources inyecta frontmatter; (e) STRUCTURAL_FILES nunca sobrescritos; (f) LLM opcional (P4): sin runner → merge determinístico fallback o skip documentado"
- **Pre-mortem:** (1) concurrencia en Rust sin tokio → el crate es sync (D1); usar threads con semaphore o hacer merge secuencial puro (ponytail: serial ES el requisito del merge; el límite aplica al LLM call que sí puede ser thread pool pequeño); (2) prompts de merge en chino → reescribir inglés P7
- **Stop conditions:** appetite 2d excedido → entregar extract+commit serial sin límite concurrente (secuencial puro) ⬛
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟡×🟠 | merge LLM alucina contenido | prompt con cita de chunks + guard longitud | test c |
  | 🟡×🟡 | concurrencia mal medida bloquea | semaphore con cap; medir | benchmarks |
  | 🟢×🟠 | fallo parcial deja índice inconsistente | write por página atómico + log de fallos | test c |
- **Cynefin:** 🟧 complejo — el comportamiento emerge al probar con contenido real; steps cortos con verify frecuente
- **Top 3 riesgos:** alucinación merge, concurrencia, consistencia parcial
- **Uphill/Downhill:** ⬆️ 0 (binding = LlmRunner del crate, decisión fija) · ⬇️ 5 steps
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-30.md`
- **Branch:** | **Commit:**
- **Iteraciones:** | — | — | — | — |
- **Notas:** Ruta: vanta-worker. DEPENDE de Tasks 2+3. Prompts en `vanta-memory/src/ingest/prompts.rs` (inglés).

### Task 6: MEM-26 — vanta-proxy ciclo auth→session→injection
- **Appetite:** max 2d
- **Esfuerzo:** 🟡 | **Prioridad:** 🔴
- **Archivos clave:** `vanta-proxy/src/{auth,session,inject}.rs` (crear)
- **Verificación real:** ✅ CÓDIGO-REAL — depende de Task 4; TDAM refs verificadas: sessionKey headers (session-key.ts:9-19), state machine local form team→agent→task TTL 30min solo pending (store.ts:31,116), inyección L2/L3 system prompt + L0/L1 como tools (README:28), resolución apiKey server-key|passthrough (handler.ts:1095-1104)
- **Gate Justificación:** el ciclo es el valor del proxy (sin él es reverse-proxy tonto); D25/D26/D29 fijan el diseño local-first
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vanta-proxy` pasa; tests D19 con upstream+db mockeados: (a) auth user_key válida/inválida contra RBAC local — **D34: toda request sin key válida → 401, sin modo open** contra RBAC local; (b) sessionKey extraído de cada header alias; (c) state machine team→agent→task con TTL pending; (d) inyección persona/escenas en system prompt (desde vanta-memory vía lib); (e) L0/L1 expuestos como tools en el request; (f) sin sesión previa → init limpio"
- **Pre-mortem:** (1) vanta-proxy depende de vanta-memory (inyección) + vantadb (auth) → deps de workspace dirigidas, cuidado ciclos (vanta-memory→vantadb ya existe; proxy→ambas OK); (2) inyección rompe KV-cache si va al history → D29: solo system prompt + tools
- **Stop conditions:** si la integración con vanta-memory exige cambios en el crate → exponer trait/facade mínima en vanta-memory (task aditiva pequeña) y documentar
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟡×🔴 | inyección en history invalida KV-cache | D29: solo system prompt + tools; test de posición | test d/e |
  | 🟡×🟠 | ciclo de dependencias workspace | grafo: proxy→{memory,vantadb}; verificar con cargo tree | DISCOVERY |
  | 🟢×🟡 | TTL pending leak | sweep en cada request (lazy) | diseño |
- **Cynefin:** 🟨 complicado
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 4 steps
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-26.md`
- **Branch:** | **Commit:**
- **Iteraciones:** | — | — | — | — |
- **Notas:** Ruta: vanta-worker. DEPENDE de Task 4.

### Task 7: MEM-33 — MCP tools wiki_* query-only (4 tools)
- **Appetite:** max 1d
- **Esfuerzo:** 🟢 | **Prioridad:** 🟡
- **Archivos clave:** `vantadb-mcp/src/wiki.rs` (crear), `vantadb-mcp/src/handlers/tools.rs` (wiring)
- **Verificación real:** ✅ CÓDIGO-REAL — wiki.rs NO existe; depende de MEM-28 (store) + MEM-30 (índice); TDAM ref: mcp/tools.ts 4 wiki_* (wiki_search/wiki_read/wiki_list/wiki_graph), graphMultiHopSearch DEFAULT_MAX_NODES=200 (graph-search.ts:38), bm25 title×5 (manager.ts:381-391)
- **Gate Justificación:** cierra las 12 tools del patrón MCP (8 code_ de Task 1 + 4 wiki_)
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vantadb-mcp` pasa; tests D19: (a) wiki_search con text index propio (BM25 propio, NO FTS5); (b) wiki_read respeta locked; (c) wiki_list; (d) wiki_graph BFS multi-hop cap 200; (e) wiki pending (no ready) → falla clara; (f) read-only"
- **Pre-mortem:** (1) BM25 propio vs FTS5 → VantaDB ya tiene text_index propio (MEM-06 lo reusó) — reusarlo, NO SQLite; (2) search antes de ingest → estado pending debe fallar claro
- **Stop conditions:** si el índice por-wiki exige infra nueva en core → index en la partición del store con rebuild transaccional (patrón manager.ts:394-412 adaptado)
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟡×🟡 | ranking BM25 propio peor que FTS5 | aceptable: mismo trade-off documentado; tuning k1/b si hace falta | benchmarks |
  | 🟢×🟢 | BFS sin cap explota | cap 200 hardcodeado + test | test d |
- **Cynefin:** 🟦 obvio
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 3 steps
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-33.md`
- **Branch:** | **Commit:**
- **Iteraciones:** | — | — | — | — |
- **Notas:** Ruta: vanta-worker. DEPENDE de Tasks 2+5.

### Task 8: MEM-31 — Progreso de ingest: canal interno + polling con run_id
- **Appetite:** max 1d
- **Esfuerzo:** 🟢 | **Prioridad:** 🟡
- **Archivos clave:** `vanta-memory/src/ingest/callback.rs` (crear — canal interno + `wiki_status(run_id)` consultable)
- **Verificación real:** ✅ CÓDIGO-REAL — callback.rs NO existe; TDAM refs: run_id randomUUID compartido con progress (wiki-service.ts:1026), throttle PROGRESS_THROTTLE_MS=500 (manager.ts:110,121), fases extracting|merging|indexing con {total,completed,failed,skipped,percent}; **D32: canal interno + polling, sin HTTP**
- **Gate Justificación:** rechazo de paquetes tardíos (build anterior) es el patrón clave; D32 cierra el destino: canal interno + polling `wiki_status(run_id)` — sin HTTP ni auth S2S. El desktop lo puentea a eventos Tauri después.
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vanta-memory` pasa; tests D19: (a) consulta con run_id viejo es descartada; (b) throttle 500ms entre actualizaciones de progreso; (c) summary truncado a los límites; (d) el canal interno nunca bloquea el ingest; (e) fases extracting|merging|indexing con {total,completed,failed,skipped,percent}; (f) wiki_status(run_id) consultable desde otro handle"
- **Pre-mortem:** (1) progreso live en desktop → puente Tauri event sobre el canal interno (fuera de scope acá, solo el trait); (2) run_id sin persistir → guardar en el store del wiki (MEM-28)
- **Stop conditions:** ninguno previsto — canal interno es mecánico
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟡×🟡 | paquetes tardíos corrompen estado | filtro run_id estricto + test a | test a |
  | 🟢×🟢 | update flood | throttle 500ms | test b |
- **Cynefin:** 🟦 obvio
- **Uphill/Downhill:** ⬆️ 0 (D32) · ⬇️ 3 steps
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-31.md`
- **Branch:** | **Commit:**
- **Iteraciones:** | — | — | — | — |
- **Notas:** Ruta: vanta-worker. DEPENDE de Task 5.

### Task 9: MEM-27 — vanta-proxy rate-limit + write-back + reporting + mem-command
- **Appetite:** max 2d
- **Esfuerzo:** 🟡 | **Prioridad:** 🟠
- **Archivos clave:** `vanta-proxy/src/{rate_limit,writeback,mem_command,report}.rs` (crear)
- **Verificación real:** ✅ CÓDIGO-REAL — depende de Tasks 4+6; TDAM refs: sliding window 60s bucket spaceId×model (redis-store.ts:324-326), fail-open (guard.ts:40-51), 429 Retry-After (:111-121), write-back withL0Retry 3 intentos backoff 500ms→1s→2s (handler.ts:1986-1996, pending-writes.ts:38-108), flush SIGTERM (index.ts:154-169), mem-command mem:sync|create-skill|help deshabilitado por defecto (index.ts:24)
- **Gate Justificación:** completa el ciclo del proxy; D24 fija in-process sin Redis
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vanta-proxy` pasa; tests D19: (a) sliding window 60s por spaceId×model con default **60 req/min (D35)** bloquea el exceso con 429+Retry-After; (b) fail-open consciente (degraded → allow + log warn); (c) write-back L0 fire-and-forget retry 3 backoff exponencial; (d) flush de pendientes en shutdown graceful; (e) mem-command con los **3 comandos de TDAM (D33)**: mem:sync | mem:create-skill | mem:help — deshabilitado por defecto, habilitado por config responde sync/help; (f) reporting log por turno (sin Opik/Langfuse/ClickHouse — log estructurado local); (g) toda request sin user_key válida → 401 (**D34 auth obligatoria**)"
- **Pre-mortem:** (1) sliding window in-process pierde estado multi-instancia → **D37 aceptado**: single-instance por diseño (local-first); (2) reporting externo (Opik/Langfuse/ClickHouse) es scope creep → log estructurado JSON local, hooks para backends futuros
- **Stop conditions:** appetite 2d excedido → entregar rate-limit + write-back (reporting/mem-command a tarea propia ⬛)
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟡×🟠 | write-back pierde turns en crash | pending queue persistida + flush SIGTERM/SIGINT | test d |
  | 🟡×🟡 | rate limit in-process incorrecto bajo concurrencia | Mutex/atomic window + test concurrente | test a |
  | 🟢×🟢 | mem-command abuso | disabled by default (TDAM parity) | diseño |
- **Cynefin:** 🟨 complicado
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 4 steps
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-27.md`
- **Branch:** | **Commit:**
- **Iteraciones:** | — | — | — | — |
- **Notas:** Ruta: vanta-worker. DEPENDE de Tasks 4+6.

---

## DEFERIDOS

| IDs | Destino | Motivo |
|---|---|---|
| MEM-36 | Campaña bindings SDK | sub-clientes TS/Python con backward-compat 100% — appetite propio |
| Quota/billing | Server mode posterior | SYNTHESIS §3 nota: elegir UNA calculadora (TDAM tiene 2 inconsistentes ÷1000 vs ÷10000) |

## Checkpoints

| # | Después de | Verificación |
|---|---|---|
| CP1 | Tasks 1-3 (Wave 0) | suite completa + tools code_* responden sobre grafo seedeado + wiki store transitions green |
| CP2 | Tasks 4-5 (Wave 1) | proxy forwarda 3 protocolos contra upstream mockeado (incluye SSE) + ingest serial green |
| CP3 | Tasks 6-8 (Wave 2) | ciclo proxy completo con inyección + wiki tools + callback run_id |
| CP4 | Task 9 + cierre | rate-limit/write-back green + docs coverage 0 gaps + ADR proxy/knowledge borrador |

## Lecciones aplicadas de P27/P29 (obligatorias)

1. Verify mecánico del lead tras CADA sub-agente (atrapó corrupción de plan file en ~todas las tareas).
2. SARL: RESUME con feedback exacto del fallo mecánico funciona mejor que RESUME genérico.
3. **Decisiones cerradas antes de delegar** (lección MEM-24: D23 abierta → 2 fallos; cerrada → 1er intento OK). Este plan cierra D24-D30 upfront.
4. Corregir header del plan file tras cada update_task_state (bug server MCP conocido).
5. Task file obligatorio en DISCOVERY; cierre estándar Backlog→progreso + learnings.
6. Métrica P29: primer-intento 33% — si esta campaña repite <50%, escalar el problema de contexto de vanta-worker al usuario antes de seguir.

---

=== RECITATION ===
Campaign ID: (pendiente MCP)
Objetivo activo: F6 vanta-proxy + F7 knowledge — cierre roadmap TDAM
Estado: pending ⏳
Última acción: MEM-29 fuentes locales + chunker: scan_local_sources recursivo orden lexicográfico + SOURCE_CHAR_BUDGET 28000, chunk_text 12000/400 split headings→párrafos→hard-cut con overlap tail; guard path traversal canonicalize+starts_with (ensure_within_root); skip no-.md/binarios/symlinks con tracing. Wave 0 completa (3/9)
Resultado: OK
Próxima acción: Wave 1: delegar MEM-25 (Task 4, vanta-proxy crate + 3 wire protocols, appetite 3d) a vanta-worker
Contrato: por tarea — cargo check/nextest/fmt/clippy del crate tocado exit 0 + tests D19
Próxima tarea si completa: 4
