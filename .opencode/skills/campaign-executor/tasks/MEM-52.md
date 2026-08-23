# MEM-52 — Fachada productiva de ingest wiki (H3)

Plan: docs/plans/2026-08-22-vanta-ultima-milla.md · Task 3 · Cynefin 🟦 obvio · 🟢×🟡

## Contrato

tests D19: POST/tool dispara worker::run async → estado pending→processing→ready
consultable por run_id (MEM-31) → páginas disponibles para wiki_read.

## Decisión de ubicación (documentada)

**Elegido: `vantadb-mcp/src/wiki.rs` (tools `wiki_ingest` + `wiki_ingest_status`).**

Por qué NO `src/cli_server.rs`:
- El worker vive en `vanta-memory`, que depende de `vantadb`. Una ruta HTTP en el
  crate core exigiría que `vantadb` dependa de `vanta-memory` → ciclo de paquetes
  prohibido por Cargo (verificado: core Cargo.toml no referencia vanta-memory).
- El consumidor natural del wiki vía MCP son agentes (ya existen wiki_search/
  read/list/graph en ese módulo con convenciones de error establecidas MEM-32/33).
- `vantadb-mcp` ya tiene vanta-memory como dev-dep (MEM-44, ciclo-free verificado
  vía cargo tree) → se promueve a dependencia regular. Sin crates externos nuevos.

## Diseño

1. Split del worker (vanta-memory/src/ingest/worker.rs) en dos fases públicas:
   - `begin(store, ns, slug) -> Result<String>` — request_ingest si !busy +
     begin_processing → run_id (sync, 2 writes baratos).
   - `execute(store, ns, slug, root, runner, config, progress, run_id)` — cuerpo +
     complete/fail + emits. `run_with_progress` = begin + execute (comportamiento
     idéntico; 472 tests deben seguir verdes).
2. Fachada MCP: registro global `INGEST_RUNS: OnceLock<Mutex<HashMap<run_id,
   {tracker, ns, slug}>>>`; `start_ingest` hace begin sync → spawn de std::thread
   con execute → retorna run_id inmediato (pre-mortem del plan). Thread mueve
   `Option<R>` owned (trait no dyn-compatible, MEM-30).
3. `wiki_ingest(ns, slug, root)` → {run_id, state:"pending"}; runner=None
   (modo LLM-free P4: fuentes skipped documentado). `wiki_ingest_status(run_id)`
   → estado del wiki + último snapshot del tracker (MEM-31).
4. Test D19 usa la misma fachada (`start_ingest`) con ScriptedRunner + poll hasta
   ready + wiki_read vía handle_tools_call (la maquinaria async es lo riesgoso;
   el wrapper del tool es glue trivial cubierto por test de registro).

## Impacto mapeado (Regla 0)

Archivos leídos completos:
- src/wiki/store.rs (codegraph, transiciones + guards busy/run_id)
- vanta-memory/src/ingest/worker.rs (completo, 312L)
- vanta-memory/src/ingest/mod.rs (1-110: IngestConfig, errores)
- vanta-memory/src/ingest/callback.rs (ProgressTracker, wiki_status)
- vantadb-mcp/src/wiki.rs (425L completo)
- vantadb-mcp/src/server.rs (dispatch, spawn_blocking, timeout)
- vantadb-mcp/src/handlers/tools.rs (1090L, rutas)
- vantadb-mcp/tests/wiki_roundtrip_e2e.rs (patrón ScriptedRunner)
- vantadb-mcp/Cargo.toml

Referencias hacia dentro (lo que llamo):
- worker::run_with_progress (1 caller interno: run; tests ingest.rs)
- WikiStore::{get, request_ingest, begin_processing, complete, fail}
- ProgressTracker::{begin_run, update_progress, wiki_status}

Referencias entrantes (lo que depende de lo que cambio):
- worker::run/run_with_progress: tests vanta-memory (472) — refactor debe ser
  transparente (mismo orden de operaciones y errores).
- handle_wiki_tool: routed desde handlers/tools.rs:1071.
- wiki_tool_definitions: extendida en handle_tools_list (tools.rs:232).

Veredicto: cambio aditivo; refactor de worker preserva contrato observable;
riesgo principal = polling flaky en tests (usar poll-loop con timeout, no sleep fijo — lección MEM-50).

## Steps

### ✅ Step 1 — Split begin/execute en worker (vanta-memory)
Refactor run_with_progress → begin() + execute(); cargo test -p vanta-memory completo: 295 lib + todos los bins de integración 0 failed.

### ✅ Step 2 — Fachada MCP (tools + registro + thread)
Cargo.toml dep promovida, wiki.rs (2 tool defs, start_ingest genérico, NoLlm, registro OnceLock), ruta en tools.rs; re-exports en lib.rs.

### ✅ Step 3 — Test D19 contrato async + verify mecánico completo
tests/wiki_async_ingest.rs: 3/3 (registro tools/list · D19 run_id→ready→wiki_read · degradado LLM-free ready).

## Verify mecánico (RESULTADO FINAL)

- `cargo check -p vantadb -p vanta-memory -p vantadb-mcp --all-targets` ✅ exit 0
- `cargo test -p vanta-memory "wiki::"` ✅ exit 0 · `cargo test -p vanta-memory ingest` ✅ 13+passed, suite completa 472/472 0 failed
- `cargo test -p vantadb-mcp` ✅ exit 0 (incluye 3 nuevos + e2e MEM-44)
- `cargo fmt --check` ✅ · `cargo clippy -p vanta-memory -p vantadb-mcp --all-targets --no-deps -- -D warnings` ✅

## Context Save Point

Tarea COMPLETA. Notas para el orquestador:
- MCP campaign_update_task_state(in-progress) fue rechazado por WIP guard (GOV-B2/GOV-E1 de otra sesión); intento de cierre igualmente.
- NO commiteado por orden explícita del usuario (el lead hace el commit).
- Runner del tool en producción = None (degradado P4): completa ready con fuentes skipped; páginas requieren runner LLM configurado. Inyección de runner para tests vía start_ingest genérico público.
- Registro global de runs nunca se poda (ponytail comment): bounded por cantidad de ingests por proceso.

## Verify mecánico

- cargo check -p vantadb -p vanta-memory -p vantadb-mcp --all-targets
- cargo test -p vanta-memory "wiki::" && cargo test -p vanta-memory ingest && cargo test -p vantadb-mcp
- cargo fmt --check
- cargo clippy -p vanta-memory -p vantadb-mcp --all-targets --no-deps -- -D warnings

## Context Save Point

(vacío — tarea inicia)
