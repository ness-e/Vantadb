# MOD-12 — `ensure_indexes_current` ausente en path HTTP (text search rota DB fresca)

> **Estado:** ✅ COMPLETED · **Appetite:** max 2h · **Esfuerzo:** 🟢 · **Prioridad:** 🔴
> **Plan:** docs/plans/2026-08-23-backlog-triage.md (Task 3, Wave 1)
> **Workflow:** bug-fix · **Nota:** retry fresco — intento previo murió sin dejar trabajo (worktree limpio verificado)

## Bug

Búsqueda textual/híbrida vía HTTP falla en DB fresca (`text_index not found: bm25`).
El fix gemelo MCP-01 (`vantadb-mcp/src/server.rs:30-44`) cubrió SOLO el canal stdio;
el binario `vantadb-server` en modo HTTP llama `vantadb::cli_server::run(config)`
(`vantadb-server/src/main.rs:72-77`) que abre el engine por otra ruta sin garantizar índices.

## Root Cause (verificado en fuente hoy)

1. `src/cli_server.rs:1749 run()` abre `StorageEngine::open_with_config` (:1758) y
   construye `ServerState { db: VantaEmbedded::from_engine(storage) }` (:1784).
2. `VantaEmbedded::from_engine` (`src/sdk/builder.rs:35-42`) **NO** llama
   `ensure_indexes_current`; solo `open_with_config` lo hace (builder.rs:104-106,
   guard `read_only`) — y `run()` no usa ese constructor.
3. Grep: **0 matches** de `ensure_indexes_current` en `cli_server.rs` (hoy).
   Ningún handler/middleware lo invoca.

## Discovery crítico (pre-mortem): ¿otro constructor ya garantiza índices?

**NO — el fix es necesario, no SKIP:**
- `StorageEngine::open_with_config` → `init_storage`/`init_indexes` crean estructuras
  de storage, no el *estado current* del text index (evidencia MCP-01: puts escribieron
  26 postings pero el estado nunca se creó → query falla igual).
- `from_engine` leído completo (builder.rs) — sin ensure.
- Cadena completa confirmada por `docs/reviews/modulos/vantadb-server.md` §8.1 y
  `cross-modulos.md` F-2: `cli_server.rs:1758 → builder.rs:35-42`.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `src/cli_server.rs` (run() 1749-1805, ServerState
  126-148, routes 235-285, records_put/records_search 1122-1360, SearchPageRequest
  1305-1327, app() 207), `src/sdk/builder.rs` (385L), `vantadb-mcp/src/server.rs`
  (patrón MCP-01 27-74), `vantadb-server/tests/e2e.rs` (530L),
  `vantadb-server/tests/helpers/mod.rs` (32L), `vantadb-server/src/main.rs` (148L),
  `vantadb-server/src/{lib,server}.rs`, `src/sdk/serialization/vector_types.rs`
  (VantaMemorySearchRequest 12-57), `src/sdk/types.rs` (VantaMemoryInput 131-170),
  `vantadb-server/Cargo.toml`.
- **Referencias entrantes:** `run()` ← `vantadb-server/src/main.rs:73` (única llamada
  prod); `ServerState`/`app` re-exportados por `vantadb-server/src/server.rs:1-4`;
  `from_engine` tiene 14 callers (codegraph) — **NO se toca**.
- **Referencias salientes:** `run()` → open_with_config/from_engine/app_with_cors/
  serve_http_or_tls. Tipos wire exportados: `vantadb::sdk::{VantaMemoryInput,
  VantaMemorySearchRequest}` (lib.rs:171-172) — test puede construir payloads tipados.
- **Tests existentes:** e2e.rs ejercita IQL INSERT/FETCH/DELETE — NINGÚN test cubre
  text-search por HTTP (gap que ocultó el bug, igual que en MCP-01).
- **Veredicto:** 3 archivos (1 prod + 2 test), ~10 líneas netas. Sin cambios de API
  pública. Riesgo bajo. Fix en `run()` (no en `from_engine`: 14 callers incluirían
  engines read-only/hosts ajenos — blast radius innecesario).

## Spec

### Cambios

1. **`src/cli_server.rs` `run()`:** tras construir `state`, espejo exacto de MCP-01
   (server.rs:36-44): `if !config.read_only { if let Err(e) = state.db.ensure_indexes_current()
   { console::error(...) } }` — idempotente; log-and-continue (consistencia MCP-01).
2. **`vantadb-server/tests/helpers/mod.rs`:** `build_server_state` espeja la
   construcción de producción post-fix: `db.ensure_indexes_current()` tras
   `from_engine` — todos los e2e heredan el arranque canónico.
3. **`vantadb-server/tests/e2e.rs`:** `test_e2e_text_search_fresh_db` — put texto
   vía `POST /api/v2/records` → search textual vía `POST /api/v2/search`
   (payload tipado serializado, sin adivinar shapes serde) → `records.len() > 0`.

### Invariantes (no romper)

1. Semántica de `ensure_indexes_current` e índices intacta — solo se agrega invocación.
2. Guard `read_only`: engines read-only NO ejecutan ensure (igual builder.rs:104/MCP-01).
3. Fallo de ensure NO aborta el arranque (log-and-continue, twin MCP-01).
4. Sin unwrap nuevos en código prod (tests pueden, patrón existente).

## Steps

### Step 1 ✅ DONE — RED: e2e reproduce el bug vía HTTP
- Test contra helper SIN ensure: put 201 ✅ → search **404** (`VantaError::NotFound`
  = "text_index not found" mapeado por vanta_error_response) — bug reproducido
  exactamente sobre HTTP. Premisa validada, NO SKIP.
- Evidencia: fallo en e2e.rs:567 `left: 404 right: 200`.

### Step 2 ✅ DONE — GREEN: fix run() + helper espejo
- `src/cli_server.rs` (~1793): guard `!config.read_only` +
  `state.db.ensure_indexes_current()` log-and-continue vía console::error
  (espejo MCP-01 server.rs:36-44).
- `tests/helpers/mod.rs`: build_server_state espeja producción post-fix
  (+ensure con expect).
- GREEN: `cargo test -p vantadb-server --test e2e test_e2e_text_search_fresh_db`
  = 1 passed.

### Step 3 ✅ DONE — VERIFY full + commit + cierre
- `cargo fmt --check` ✅ · `cargo clippy -p vantadb -p vantadb-server --all-targets -- -D warnings` ✅
- Contrato mecánico: `rg -c "ensure_indexes_current" src/cli_server.rs` = **1** ≥ 1 ✅
- Tests: binario e2e **12/12** ✅ (incluye regresión nueva) · nextest -p
  vantadb-server **5/5** ✅ · nextest -p vantadb --features server módulo
  cli_server **49/49** ✅
- SECURITY: startup interno sin input de usuario; guard read_only preserva
  deployments read-only; fallo loggeado sin panic; 0 dependencias nuevas
  (cargo audit N/A).
- PERFORMANCE N/A: scan idempotente una vez al arranque, no hot path de requests.
- Commit: `fix(server): MOD-12 construye índices al arrancar server HTTP — text search funcional en DB fresca`

## Context Save Point

- Tarea COMPLETA — sin trabajo pendiente.
- Nota: el test de regresión vive en el binario `e2e`, excluido del perfil
  default de nextest (heavy certification lo corre semanalmente); verificarlo
  explícitamente con `cargo test -p vantadb-server --test e2e`.
- Nota para orquestador: durante la sesión aparecieron artefactos ajenos en el
  worktree (`tasks/MOD-16.md`, `vantadb-python/tests/conftest.py`) — agente
  paralelo, NO incluidos en este commit.
