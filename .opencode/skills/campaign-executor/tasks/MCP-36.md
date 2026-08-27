# MCP-36 — Protocolo moderno: negociación protocolVersion 2025-06-18 + structured output

## Metadata
- **Plan file:** docs/plans/2026-08-27-backlog-pipeline.md
- **Creado:** 2026-08-27
- **last-synced:** 2026-08-27
- **Estado:** ✅ COMPLETED
- **Fuente:** docs/Backlog.md fila MCP-36 (P0) — docs/reviews/archive/mcp-research-20260825.md §3 / §6 P0-A
- **Esfuerzo:** 🟢 (quick win <1d)
- **Prioridad:** 🔴 P0
- **Tipo:** Rust (MCP) — feature-add / protocolo
- **Turns estimados:** 4

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vantadb-mcp/src/server.rs:274` (`"initialize" => handle_initialize()`), `vantadb-mcp/src/lib.rs:29` re-export, tests `vantadb-mcp/tests/mcp_tests.rs:21` |
| Callees | `vantadb::metadata::{MCP_SERVER_INFO_NAME, reported_version}` (`src/metadata.rs`), `serde_json::json` |
| Implicaciones | Cambia firma `handle_initialize` (añade param), toca dispatch JSON-RPC, añade `structuredContent` a respuestas tools/call. No toca `src/wal.rs`, `src/vector/`, `src/storage/` (propiedad Arch/Engine). Blast radius pequeño, sin cambios en core. |

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:**
  - `vantadb-mcp/src/handlers/initialize.rs` (22 líneas — hardcode `2024-11-05`)
  - `vantadb-mcp/src/server.rs` (508 líneas — dispatch `"initialize"` sin params, `serve_lines`, `dispatch_request`)
  - `vantadb-mcp/src/handlers/tools.rs` (2236 líneas — `text_content`/`error_content`/`serialize_content`, 45 base tools + extend de skills/code/wiki/context/scenes/threads)
  - `vantadb-mcp/src/validation.rs` (537 líneas — helpers `text_content`, `serialize_content`, `error_content`)
  - `vantadb-mcp/src/protocol.rs` (40 líneas — `RpcRequest`/`RpcResponse`)
  - `vantadb-mcp/tests/mcp_tests.rs` (4032 líneas — `test_mcp_initialize` expects `2024-11-05`)
  - `vantadb-mcp/Cargo.toml`, `vantadb-mcp/src/lib.rs`, `.opencode/skills/vantadb-mcp/scripts/test-mcp.py` (171 líneas — envía `2024-11-05`)
  - `docs/reviews/archive/mcp-research-20260825.md` §3 (spec 2025-06-18) y Backlog fila MCP-36

- **Referenciados hacia dentro (imports/includes):** `metadata::MCP_SERVER_INFO_NAME`, `metadata::reported_version`, `serde_json::json`, `crate::handlers::initialize::*` en server.rs

- **Referencias entrantes (a los editados):** `handle_initialize` solo usado en `server.rs:274`; `text_content` usado en ~45 arms de `tools.rs` + `skills.rs`/`code.rs`/`wiki.rs`/etc.

- **Veredicto impacto:** bajo — 2 archivos core editados (initialize.rs + server.rs) + 1 helper (validation.rs) + 1 amplia (tools.rs) pero cambio reversible y sin firmas públicas expuestas fuera del crate (mcp server es binario, no lib pública). No cambia persistencia ni índices.

## Spec

| Decisión | Elección | Evidencia |
|----------|----------|-----------|
| Versiones soportadas | `["2025-06-18", "2024-11-05"]` con `LATEST = "2025-06-18"` | Spec estable actual 2025-06-18 (mcp-research §3); 2024-11-05 para compat hacia atrás. 2026-07-28 NO afecta stdio (research §3.2). |
| Negociación | Si `params.protocolVersion` ∈ soportadas → eco exacto; si ausente/null/no-string → `LATEST`; si desconocida → `LATEST` (forward-compatible, no error) | Quick win research §7.1 "eco de la versión pedida, anunciar 2025-06-18" — un server que rechace versión rompe clientes viejos. Spec MCP: server responde con versión que soporta. |
| Firma | `handle_initialize(params: Option<&Value>) -> Result<Value, Value>` | Pasar `req.params` desde dispatch; mantiene compat con tests (params None → default). |
| Capabilities anunciadas | `{"tools":{},"resources":{},"prompts":{}}` sin cambios (no Roots/Sampling/Logging) | Research §3.5: Roots/Sampling/Logging deprecados en 2026-07-28 — decisión explícita NO implementarlos (fila MCP-36: "NO implementar Roots/Sampling/Logging"). |
| Structured output | Cada éxito de `tools/call` devuelve `{"content":[{"type":"text","text": "...JSON..."}], "structuredContent": <Value>}` donde `structuredContent` es el Value original parseable | Spec 2025-06-18 structured tool output — `structuredContent` es hermano de `content` (modelcontextprotocol.io/spec 2025-06-18 — tool result). Ponytail: helper `structured_text_content(structured: &Value) -> Value` que serializa a text y duplica como structuredContent. Errores siguen `isError:true` sin structuredContent (MEM-32). |
| OutputSchema en tools/list | Añadir `outputSchema` opcional solo para 3 tools clave (`memory_put`, `memory_get`, `search_memory`) como prueba de concepto; resto sin schema (compat) | Backlog "evaluar structured output para tools clave" — no exigir outputSchema en toda la superficie en v1 (iterativo). |
| Tests | Actualizar `test_mcp_initialize` para 2025-06-18 + nuevo test de negociación (2024-11-05 eco) + test structuredContent | Contrato mecánico exige grep hit + cargo test ✅ |

## Contrato

```
cargo test -p vantadb-mcp --lib ✅
grep -n "2025-06-18" vantadb-mcp/src/handlers/initialize.rs → hit (≥1)
test-mcp.py conecta con protocolVersion=2025-06-18 y recibe eco correcto (protocolVersion == "2025-06-18" en result)
```

Evidencia adicional: `cargo test -p vantadb-mcp --test mcp_tests` incluye eco negociación; `grep -n structuredContent vantadb-mcp/src/validation.rs` → hit

## Herramientas

- cargo, cargo nextest, rust-analyzer
- codegraph_explore (si hace falta blast radius adicional)
- python (test-mcp.py manual)

## Steps

### Step 1: Negociación protocolVersion 2025-06-18 en initialize.rs + server.rs
- **Archivos:** `vantadb-mcp/src/handlers/initialize.rs`, `vantadb-mcp/src/server.rs`, `vantadb-mcp/tests/mcp_tests.rs`, `vantadb-server/tests/mcp_integration.rs`
- **Acción:** Definir `SUPPORTED_VERSIONS = ["2025-06-18","2024-11-05"]` y `LATEST = "2025-06-18"`; cambiar `handle_initialize(params: Option<&Value>)` para leer `params["protocolVersion"]` string y ecoar si está soportada, else default LATEST. Actualizar dispatch en server.rs `handle_initialize(req.params.as_ref())`. Actualizar tests `test_mcp_initialize` para esperar LATEST y añadir test negociación + server tests `initialize_negotiates_2025_06_18`.
- **Verify:** `cargo test -p vantadb-mcp --lib` ✅ 11/11 + `cargo test -p vantadb-mcp --test mcp_tests test_mcp_initialize` ✅ + `grep -n "2025-06-18" initialize.rs` → hit
- **Estado:** ✅ COMPLETED (2026-08-27 — verify: cargo test -p vantadb-mcp --lib 11 passed + mcp_tests 74 passed)

### Step 2: Structured output — helper + wiring en tools clave
- **Archivos:** `vantadb-mcp/src/validation.rs`, `vantadb-mcp/src/handlers/tools.rs`
- **Acción:** Añadidos `structured_text_content(structured: &Value)` y `text_content_structured(value: &impl Serialize)` en validation.rs. En tools.rs, 6 tools clave (`memory_put`, `memory_put_batch`, `memory_get`, `search_memory`, `search_with_method`, `search_multi`, `search_semantic`) ahora retornan `structuredContent`. Añadidos `outputSchema` en `handle_tools_list` para 5 tools (memory_put, memory_get, search_semantic, search_memory, search_with_method). Verificado `grep -n structuredContent` → hit + `outputSchema` → 5 hits.
- **Verify:** `cargo test -p vantadb-mcp --test mcp_tests` ✅ 75/75 + `cargo test -p vantadb-mcp --test mcp_tests test_mcp_structured_output_and_output_schema` ✅
- **Estado:** ✅ COMPLETED (2026-08-27)

### Step 3: Verificación integración — test-mcp.py con 2025-06-18 + verify full
- **Archivos:** `.opencode/skills/vantadb-mcp/scripts/test-mcp.py`, `skills/vantadb-mcp/scripts/test-mcp.py`, `vantadb-mcp/src/server.rs` (tests integración)
- **Acción:** Actualizados ambos test-mcp.py para usar `protocolVersion 2025-06-18`. Corridos verify full: `cargo fmt --check` ✅, `cargo clippy -p vantadb-mcp --all-features` ✅, `cargo nextest run -p vantadb-mcp --profile audit` ✅ 62 tests, `pwsh validate-docs-coverage.ps1` ✅ 0 gaps, `cargo check -p vantadb-server` ✅.
- **Verify:** `cargo test -p vantadb-mcp --lib` ✅ 11/11 + `grep -n "2025-06-18" initialize.rs` → 3 hits + `test_mcp_initialize_negotiation` + `initialize_negotiates_2025_06_18` ✅
- **Estado:** ✅ COMPLETED (2026-08-27)

## Dependencias
- Requiere: FIND-34..39, CORE-01/02 (Wave 1-2) no bloquean — es Wave 3 paralelo, pero validar que storage no esté roto
- Bloquea: MCP-37, MCP-38 (protocol moderno debe preceder profiles/annotations)

## Notas
- Ponytail: no abstraer negociación en trait; constante + match es suficiente. No implementar Elicitation/Annotations (son MCP-38). No tocar WAL/vector/storage.
- Security: `protocolVersion` es string plano, validar que sea string antes de ecoar — si no es string, default LATEST, no panic. No `unwrap`.
- No duplicar lógica: helper `structured_text_content` vive en validation.rs, usado por todos los handlers (thin wrapper).

## Context Save Point
- Trabajo previo: S1-S3 completados, verify full local ✅ (fmt, clippy mcp, nextest audit 62 passed, docs coverage 0 gaps, server initialize negotiation tests)
- Archivos tocados: vantadb-mcp/src/handlers/initialize.rs, vantadb-mcp/src/server.rs, vantadb-mcp/src/validation.rs, vantadb-mcp/src/handlers/tools.rs, vantadb-mcp/tests/mcp_tests.rs, vantadb-server/tests/mcp_integration.rs, .opencode/skills/vantadb-mcp/scripts/test-mcp.py, skills/vantadb-mcp/scripts/test-mcp.py
- Próximo step: ninguno — tarea COMPLETA, pendiente commit + campaign_update_task_state + progreso

## Verify (evidencia)
- `cargo test -p vantadb-mcp --lib` → 11/11 ✅
- `cargo test -p vantadb-mcp --test mcp_tests` → 75/75 ✅
- `grep -n "2025-06-18" vantadb-mcp/src/handlers/initialize.rs` → 3 hits (LATEST, SUPPORTED, doc)
- `grep -n "structuredContent" vantadb-mcp/src/validation.rs` → hit (helper)
- `grep -n "outputSchema" vantadb-mcp/src/handlers/tools.rs` → 5 hits
- `cargo fmt --check` → ✅
- `cargo clippy -p vantadb-mcp --all-targets --all-features -- -D warnings` → ✅
- `cargo nextest run -p vantadb-mcp --profile audit` → 62 passed
- `pwsh validate-docs-coverage.ps1` → 0 gaps
- `cargo check -p vantadb-server` → ✅ (mcp_integration fix)


