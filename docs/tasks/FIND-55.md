# FIND-55: Sanitizar body 500 — chain solo a logs, code al cliente

## Metadata
- **Plan file:** — (ejecución directa desde Backlog, orden del orquestador 2026-09-02)
- **Creado:** 2026-09-02T00:00
- **last-synced:** 2026-09-02T17:45
- **Estado:** ✅ COMPLETED
- **SDP:** campaign-executor (pipeline) + ponytail full (activo); TDD/incremental/context aplicados per base del agente — sin candidatos extra justificados para un cambio de 1 archivo con contrato mecánico.
- **Commits:** `fefdbc93` (fix) + `14ca886f` (docs closure)
- **Sub-agente:** vanta-worker
- **Área:** `src/server/errors.rs` (NO tocar `router.rs` — otro agente; NO tocar `vantadb-mcp/`)

## Blast Radius (Discovery)

- `query_error_response` / `vanta_error_response`: callers en `handlers.rs`, `routing.rs`, `errors.rs` — todos construyen Response; ninguno hace match sobre el contenido del body 500.
- `log_vanta_error`: 2 callers (ambas funciones de envelope) → cubre `query_error_response`, no solo `vanta_error_response` ✅. **GAP detectado:** el helper NO emite el Display completo (solo fields code/retriable/hint + mensaje estático). Sin fix, tras sanitizar, la chain desaparecería de logs y respuesta. Step 1 lo corrige.
- `vantadb-ts/src/errors.ts::classifyWasmError`: regex-mirrora prefijos Display del CORE en la frontera wasm/napi (`to_js_err`), NO del envelope HTTP → inafectado (verificado errors.ts:68-94).
- Consumidores del prefijo "Execution Error:" en envelope HTTP: `desktop/src-tauri/tests/server_client_mock.rs` (mock fixture propio, shape 4xx "node not found" → sigue fiel: los 4xx conservan el message), `desktop/src-tauri/src/connections/wire_types.rs` (test con string hardcodeado), `docs/api/openapi.yaml` + `docs/api/HTTP_API.md` (ejemplos 404 → siguen válidos). Ninguno depende del body 5xx → sin cambios, hallazgo anotado.
- Tests existentes en `src/server/errors.rs`: `error_envelopes_carry_canonical_code` usa NodeNotFound (4xx) → no rompe. Ningún test del repo aserta contenido de body 500.

## Contrato

`cargo test -p vantadb --lib --features server` 0 failed AND tests nuevos de sanitización green AND `cargo clippy --workspace --all-targets --all-features -- -D warnings` 0 AND `cargo fmt --all -- --check` 0 AND grep del body 500 genérico visible en `src/server/errors.rs`.

Commit: `fix(server): sanitiza body 500 — chain solo a logs, code al cliente (FIND-55)`

## Herramientas

- cargo test/clippy/fmt, codegraph_explore (blast radius hecho), grep (consumidores verificados)

## Steps

### Step 1: Log full chain en `log_vanta_error`
- **Archivos:** `src/server/errors.rs`
- **Acción:** añadir `error.display = %e` a ambos brazos del `tracing::event!` (field set idéntico entre brazos; el message del evento sigue estable para pipelines — el detail vive en un field, como `error.hint`). Ajustar doc-comment.
- **Verify:** cubierto por tests de Step 3 + suite server
- **Estado:** ✅ DONE

### Step 2: Sanitizar rama 5xx en ambos envelopes
- **Archivos:** `src/server/errors.rs`
- **Acción:** en `query_error_response` y `vanta_error_response`, si `status.is_server_error()` → message `"internal error"` (el JSON ya lleva `code`; NO remover code). 4xx mantienen Display descriptivo (dato del input del usuario).
- **Verify:** grep "internal error" → 4 hits (2 prod + 2 test)
- **Estado:** ✅ DONE

### Step 3: Tests TDD (RED antes del GREEN)
- **Archivos:** `src/server/errors.rs`
- **Acción:** (a) `VantaError::IoError` con marcador contrived → body `data`/`error` NO contiene el string io y SÍ `code == VANTADB_IO_ERROR`; (b) `ValidationError` (4xx) mantiene `reason` en ambos envelopes.
- **Verify:** RED confirmado (io leak) → GREEN 6/6
- **Estado:** ✅ DONE

### Step 4: Verificación completa del contrato
- **Verify:** 2041/2041 tests, clippy 0, fmt 0, grep OK
- **Estado:** ✅ DONE

## Dependencias
- ERR-OBS-01 (helper `log_vanta_error`), ERR-CORE-01 (campo `code` e1fe7ec2) — ambos ya landed.

## Notas
- **HALLAZGO (req. 3):** ningún consumidor del envelope HTTP depende del prefijo "Execution Error:" en 5xx; `classifyWasmError` mira Display del CORE (wasm boundary), no HTTP. Docs de 4xx siguen exactas.
- Premisa del task ("el helper ya emite Display completo") era falsa → Step 1 la hace verdadera (gap cerrado en el propio file, sin scope creep).
- NO stagear `completions/*` ni `.opencode`; NO tocar `stash@{0}`.

## Context Save Point
- **Fecha:** 2026-09-02
- **Branch:** (actual, sin cambiar)
- **CI pendiente:** no
- **Decisiones:** `"internal error"` literal (no "Internal server error") porque lo dice el requirement; display al log como field `error.display` (no en message) para no romper el mensaje estable del pipeline; field set idéntico en ambos brazos preserva el invariant del comentario ERR-OBS-01.
- **Problemas conocidos:** ninguno
- **Próxima tarea:** —
