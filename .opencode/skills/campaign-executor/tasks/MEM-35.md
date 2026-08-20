# MEM-35: F3 Core Data plane de referencia en server — REST /conversation/add + /skill/listing

## Metadata
- **Plan file:** docs/plans/2026-08-18-vanta-memory.md
- **Creado:** 2026-08-20T16:30
- **last-synced:** 2026-08-20T16:30
- **Estado:** ✅ COMPLETED — commit `9693d0ff` en `develop`

## Blast Radius

**Callers (aguas arriba — depende de lo que cambio):**
- `vantadb-server/src/server.rs` re-exporta `app`/`app_with_cors`/`auth_middleware`/`ServerState` desde `vantadb::cli_server` — los endpoints nuevos viven en el crate core `src/cli_server.rs`, el wrapper los hereda sin cambios.
- `vantadb-server/tests/e2e.rs` — tests TCP reales contra `app()`; los tests nuevos se agregan acá (D19).
- Router `protected` en `app_with_cors` (src/cli_server.rs:214-259) — añadir rutas ahí = auth 3 capas (MEM-05) + rate limit + CORS + body limit + audit auth automáticos.

**Callees (aguas abajo — de lo que depende el cambio):**
- `VantaEmbedded::create_thread/send_message/get_thread` (src/sdk/builder.rs:161-179) → `ThreadStore` (src/agentic/thread.rs) — **EXISTE** (la asunción del lead "ThreadStore NO existe" está desactualizada; no hace falta stub 501).
- `VantaEmbedded::engine_handle()` (pub(crate), src/sdk/builder.rs:110) + `SkillStore::list` (src/skills.rs:110) — mismo patrón que `vantadb-mcp/src/skills.rs:232` (skill_list).
- `AuditEvent::memory("conversation", ...)` (src/audit.rs:55) → op `memory_conversation`.

**Implicaciones:**
- NO romper endpoints existentes `/api/v2/*` (Studio) — las rutas nuevas van a root (`/conversation/add`, `/skill/listing`), sin tocar las v2.
- NO copiar `/v3/session/init` ni `/v3/knowledge/query` (TDAM no las tiene — SYNTHESIS.md:44).
- Sin deps nuevas. Sin WASM impact (los handlers viven bajo `#[cfg(feature="server")]` / binario, no en el core WASM).
- SkillRecord es Serialize; la response de listing es DTO lean (skill_id/version/name/description) — consistente con MCP skill_list y TDAM listing (inyección de prompt, no dump de content).

## Contrato
"`cargo check -p vantadb-server` pasa; tests dedicados de endpoints data plane (D19)"

## Herramientas
- cargo (terminal), codegraph, skill security-and-hardening, ponytail (full)

## Steps

### Step 1: DISCOVERY + task file MEM-35
- **Archivos:** `.opencode/skills/campaign-executor/tasks/MEM-35.md`
- **Acción:** verificar asunciones del lead con codegraph (ThreadStore existe; SkillStore::list; auth_middleware; AuditEvent::memory; e2e pattern), diseñar wire contract, crear este task file.
- **Verify:** archivo existe + plan file MEM-35 ⏳ EN PROGRESO
- **Estado:** ✅ (este archivo)

### Step 2: POST /conversation/add — handler + route en cli_server.rs
- **Archivos:** `src/cli_server.rs`
- **Acción:** struct `ConversationAddRequest { thread_id: Option<String>, title: Option<String>, role: String, content: String, ttl_secs: Option<u64> }`; handler `conversation_add` — si `thread_id` presente → `db.send_message(id, role, content)` (404 si no existe vía NodeNotFound→vanta_error_status); si ausente → `db.create_thread(title.unwrap_or_default(), ttl_secs)` + `send_message`; audit `AuditEvent::memory("conversation", "threads", id, "ok", None)`; response `(CREATED, {success:true, thread_id})`. Route `.route("/conversation/add", post(conversation_add))` en router `protected`.
- **Verify:** `cargo check -p vantadb --features server`
- **Estado:** ⬜ PENDING

### Step 3: GET /skill/listing — handler + route en cli_server.rs
- **Archivos:** `src/cli_server.rs`
- **Acción:** struct `SkillListingParams { owner_agent: Option<String>, name_prefix: Option<String>, limit: Option<usize>, offset: Option<usize> }`; handler `skill_listing` — `db.engine_handle()` → `SkillStore::new(&engine)` → `store.list(SkillListOptions{...})` (limit default 50, cap 200); response DTO lean `{items:[{skill_id,version,name,description}], total}`. Route `.route("/skill/listing", get(skill_listing))` en router `protected`.
- **Verify:** `cargo check -p vantadb --features server`
- **Estado:** ⬜ PENDING

### Step 4: E2E tests data plane en vantadb-server/tests/e2e.rs
- **Archivos:** `vantadb-server/tests/e2e.rs`
- **Acción:** tests TCP reales: (a) conversation/add crea thread y devuelve thread_id; (b) conversation/add con thread_id existente appendea (get_thread verifica 2 mensajes); (c) conversation/add sin auth → 401 (context con api_key); (d) skill/listing vacío; (e) skill/listing con skills seedeadas vía `SkillStore::create` sobre `state.storage` + filtro owner_agent/name_prefix. Sin deps nuevas (reqwest/axum ya en dev-deps).
- **Verify:** `cargo nextest run -p vantadb-server --test e2e`
- **Estado:** ⬜ PENDING

### Step 5: Cierre — verify full + commit
- **Archivos:** — (solo archivos tocados)
- **Acción:** `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo nextest run --profile audit --workspace --build-jobs 2`; `git add` solo archivos de la tarea; commit convencional `feat(server): data plane REST /conversation/add + /skill/listing (MEM-35)` (lo ejecuta vanta-lead si aplica).
- **Verify:** todos los checks ✅
- **Estado:** ⬜ PENDING

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:** `src/cli_server.rs` (app_with_cors 198-299, threads_* 2417-2560, audit_events 1411-1455, vanta_error_status 875-892, auth_middleware 578-746), `src/sdk/builder.rs` (create_thread/send_message/engine_handle/audit), `src/agentic/thread.rs` (ThreadStore completo), `src/skills.rs` (SkillStore::list), `src/sdk/types.rs` (SkillListOptions/SkillRecord), `src/audit.rs` (AuditEvent::memory), `vantadb-server/src/server.rs`, `vantadb-server/tests/e2e.rs`, `vantadb-server/Cargo.toml`, `.opencode/rules/server-mcp.md`, TDAM 01 §10 + 03 §4 (docs/research/tdam/).
- **Referencias hacia dentro (quién depende de lo que toco):** `vantadb-server::server` re-export (server.rs:1-4) — sigue compilando; `tests/e2e.rs` — solo agrego tests.
- **Referencias hacia fuera (de lo que dependo):** `db.create_thread/send_message/engine_handle` (builder, pub), `crate::skills::SkillStore` (lib.rs:122 pub mod skills), `crate::audit::AuditEvent` (pub), `run_db_op`/`vanta_error_response` (locales al módulo).
- **Veredicto:** cambio aditivo de bajo impacto. Sin breaks de API pública del SDK (endpoints nuevos a root, no v2). No toco wal/vector/storage (propiedad Arch/Engine). Sin deps nuevas → sin cargo audit obligatorio.

## Dependencias
- MEM-05 ✅ (auth 3 capas, commit `01a5de66`) — protege ambos endpoints
- MEM-06 ✅ (SkillStore, commit `92cf709f`) — GET /skill/listing lo consume
- MEM-03 ✅ (EntityStore, commit `23719e23`) — base de ThreadStore/SkillStore
- MEM-07 ✅ (MCP skill_*, commit `4763bf44`) — mismo SkillStore, canal separado; sin conflicto

## Notas
- **Hallazgo DISCOVERY:** el lead asumió "ThreadStore NO existe" — codegraph prueba que SÍ existe (`src/agentic/thread.rs`, expuesto en `/api/v2/threads`). Approach lazy: wiring directo sobre `db.create_thread/send_message`, sin stub 501.
- **Decisión owner scoping:** `owner_agent` es query param opcional (no derivado de AuthIdentity) — no hay mapping user→agent en F3 (la identity L3 es user_key→user entity; skills se owner-izan por owner_agent). `ponytail:` documentar el techo: cuando F4 (pipeline LLM) aterrice, scoping a AuthIdentity si aplica.
- **TDAM referencia:** `POST /v2|v3/conversation/add` = L0 message → notifyPipeline async (01 §10:108-116); `POST /v3/skill/listing` = head rows con filtros para inyección de prompt (03 §4:59). VantaDB: GET (REST idempotente) sobre SkillStore::list (D13).

## Context Save Point
- **Fecha:** 2026-08-20T16:30
- **Branch:** develop
- **CI pendiente:** no (verify full en Step 5)
- **Decisiones:** (1) ThreadStore existe → wiring directo, no stub; (2) rutas a root (no /api/v2) porque son agent-facing (D18), protegidas igual por auth_middleware; (3) listing DTO lean sin content (prompt-injection use case); (4) owner_agent opcional, techo documentado.
- **Problemas conocidos:** ninguno
- **Próxima tarea:** checkpoint tras F1+F2+F3 (vanta-lead)