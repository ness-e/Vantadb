# MEM-26: vanta-proxy ciclo auth→session→injection

## Metadata
- **Plan file:** docs/plans/2026-08-21-vanta-proxy-knowledge.md (Task 6)
- **Creado:** 2026-08-21T00:00
- **last-synced:** 2026-08-21T00:00
- **Estado:** ✅ COMPLETED (verify mecánico: check exit 0 · nextest 26/26 · fmt exit 0 · clippy --no-deps -D warnings exit 0; SIN commit por regla de la invocación)

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:** `vanta-proxy/src/{server,forward,config,error,main,lib}.rs`, `vanta-proxy/src/handlers/{openai,anthropic,responses}.rs`, `vanta-proxy/tests/proxy_wire.rs`, `vanta-proxy/Cargo.toml`, `src/entity/mod.rs` (EntityStore), `src/sdk/builder.rs` (VantaEmbedded::from_engine), `src/storage/engine/init.rs` (open_with_config), `src/cli_server.rs` (patrón auth MEM-05 resolve_user_key), `vanta-memory/src/core/persona/persona_generator.rs` (get_persona/PersonaRecord/persona_namespace), `vanta-memory/src/core/scene/scene_index.rs` (list_scenes/current_scene/upsert_scene/open_db patrón), `vanta-memory/src/lib.rs`, `vanta-memory/src/core/{mod,persona/mod,scene/mod}.rs` (paths públicos), Cargo.toml raíz (members).
- **Referencias hacia dentro:** ninguna nueva — `auth.rs`/`session.rs`/`inject.rs` no existen; los handlers existentes se editan para llamar el pipeline.
- **Referencias entrantes:** handlers de MEM-25 (único punto de wiring); `tests/proxy_wire.rs` debe actualizarse (auth obligatoria D34 rompe los 7 tests previos sin user-key).
- **Veredicto de impacto:** aditivo + wiring localizado en vanta-proxy; NO toca core `vantadb` ni `vanta-memory` código (solo SDK público: EntityStore, StorageEngine, VantaEmbedded::from_engine, get_persona/list_scenes/current_scene). Facade pública NO requerida (los paths ya son pub).
- **Deps:** SIN deps nuevas de crates.io. Se agregan 2 deps de workspace dirigidas: `vantadb` (path "../", default-features off) y `vanta-memory` (path). Grafo esperado acíclico: proxy→{memory,vantadb}; memory→vantadb.

## Blast Radius
Callers: main.rs → AppState::new (firma estable). Callees nuevos: vantadb::entity::EntityStore (entity_list/entity_get), vantadb::storage::StorageEngine::open_with_config, vanta_memory::core::{persona,scene}. Implicaciones: los 7 tests wire previos requieren seeding de user entity + header x-vanta-user-key (D34: sin modo open). Header x-vanta-user-key se agrega a hop-by-hop (credencial interna no se filtra al upstream).

## Contrato
"`cargo check -p vanta-proxy` exit 0 · `cargo nextest run -p vanta-proxy` exit 0 (7 wire + tests D19 a-f) · `cargo fmt --check` exit 0 · `cargo clippy -p vanta-proxy --all-targets --no-deps -- -D warnings` exit 0"

Tests D19 (upstream axum mockeado + db InMemory):
(a) auth user_key válida/inválida/ausente → 200/401/401 (D34 sin modo open); (b) sessionKey por cada alias header con prioridad; (c) state machine team→agent→task + TTL 30min solo pending (sweep lazy) + entidad inexistente rechazada; (d) inyección persona/escenas SOLO en posición system prompt (historia intacta — assert posición); (e) L0/L1 como tools presentes en body; (f) sin sesión previa → init limpio (body verbatim sin header de sesión; con sesión fresca sin memoria → solo tools agregadas).

## Herramientas
- terminal cargo, codegraph, campaign MCP

## Steps
### Step 1: auth.rs (D34) + error variants + config auth.db_path
- **Archivos:** `src/auth.rs` (crear), `src/error.rs` (+Unauthorized 401/+InvalidRequest 400), `src/config.rs` (+AuthConfig), `src/lib.rs`
- **Acción:** port MEM-05 resolve_user_key (ct-compare manual, sin dep subtle), authenticate(headers) fail-closed, entity_exists para session
- **Verify:** `cargo check -p vanta-proxy`
- **Estado:** ✅ COMPLETED

### Step 2: session.rs (D26)
- **Archivos:** `src/session.rs` (crear)
- **Acción:** aliases headers en orden, Stage Team→Agent→Task, SessionStore Mutex<HashMap>, TTL 30min SOLO pending con sweep lazy, advance valida entidad contra EntityStore
- **Verify:** `cargo check -p vanta-proxy`
- **Estado:** ✅ COMPLETED

### Step 3: inject.rs (D29) + wiring pipeline en server/handlers
- **Archivos:** `src/inject.rs` (crear), `src/server.rs`, `src/forward.rs` (strip user-key header), `src/handlers/*.rs`, `src/main.rs` (sin cambio de firma), `Cargo.toml` (deps workspace)
- **Acción:** persona/escenas vía vanta-memory SOLO al system prompt; L0/L1 como tools; pipeline auth→session→inject antes del forward
- **Verify:** `cargo check -p vanta-proxy`
- **Estado:** ✅ COMPLETED

### Step 4: Tests D19 (a)-(f) + actualización proxy_wire.rs
- **Archivos:** `tests/pipeline.rs` (crear), `tests/proxy_wire.rs` (seed user + header)
- **Verify:** `cargo nextest run -p vanta-proxy`
- **Estado:** ✅ COMPLETED

### Step 5: Verify mecánico completo + cierre
- **Acción:** fmt/clippy --no-deps -D warnings/check/nextest; task file ✅; recitation; SIN commit (regla de la invocación)
- **Verify:** contrato completo
- **Estado:** ✅ COMPLETED

## Dependencias
- MEM-25 (Task 4) ✅ commit eb354c0d. Ninguna otra.

## Notas
- VantaEmbedded::from_engine(Arc<StorageEngine>) permite UNA sola apertura de storage sirviendo EntityStore (auth/session) y funciones vanta-memory (persona/escenas) sobre el mismo backend.
- D29 KV-cache: la inyección jamás toca mensajes de historia; solo posición system (messages[0].role=system / campo `system` Anthropic / `instructions` Responses) + array `tools`.
- Body no-JSON → forward verbatim sin tocar (fallback seguro para streams).

## Context Save Point
- **Fecha:** 2026-08-21
- **Branch:** develop (sin commit — regla de la invocación)
- **Decisiones:** un solo StorageEngine compartido auth+memory; TTL aplica a estados pending (Team|Agent), Task es terminal; herramientas L0/L1 siempre expuestas cuando hay sesión, bloque de memoria solo si hay contenido
- **Próxima tarea:** Task 7 MEM-33 (wiki_* MCP tools)
