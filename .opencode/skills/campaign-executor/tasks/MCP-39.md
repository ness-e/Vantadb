# MCP-39: Output budgeting (truncado explícito + next_cursor)

## Metadata

- **Plan file:** `docs/plans/2026-08-28-backlog-triage.md`
- **Creado:** 2026-08-29T00:00
- **last-synced:** 2026-08-29T01:30
- **Estado:** ✅ COMPLETED

## SDP

`campaign_discover_skills archivosClave="vantadb-mcp/src/handlers/tools.rs" phase="BUILD" contractKeywords=["output budgeting","next_cursor","truncated"]` →
`api-and-interface-design, codebase-memory, ponytail`.

Skills cargadas: **campaign-executor, api-and-interface-design, codebase-memory** + ponytail (full, persistente).

## Impacto mapeado (Regla 0)

### Archivos leídos completos

| Path | Lines | Notas |
|------|-------|-------|
| `vantadb-mcp/src/handlers/tools.rs` | 2799 (extractos: 1-200, 1112-1191, 1340-1439, 1535-1614, 2790-2799) | handlers `memory_list` (línea 1343) y `search_multi` (línea 1539) |
| `vantadb-mcp/src/validation.rs` | 557 | helpers `serialize_content`, `text_content`, `text_content_structured`, `error_content` |
| `vantadb-mcp/src/config.rs` | 114 | `McpConfig` (tuning knobs) + `McpProfile` |
| `docs/api/MCP.md` | 306 (extracto 160-219) | tabla de herramientas con descripciones de `memory_list` y `search_multi` |

### Referencias hacia dentro (lo que el archivo usa)

- `serde_json::{json, Value}` (línea 7)
- `crate::config::McpConfig` (línea 4)
- `crate::error::McpError` (línea 5)
- `crate::validation::*` (línea 6) — `validate_identifier`, `validate_payload`, `serialize_content`, `text_content`, `text_content_structured`, `error_content`, `parse_filter_ops`, `parse_search_request`, `index_vector_dim`
- `vantadb::executor::{ExecutionResult, Executor}` (línea 10)
- `vantadb::storage::StorageEngine` (línea 11)
- `vantadb::sdk::VantaMemoryListOptions`, `VantaMemoryMetadata`, `VantaMemoryFilter`, `VantaSearchRequest` (línea 1371, 1558)
- `vantadb::VantaEmbedded::from_engine` (línea 1380, 1564)

### Referencias hacia afuera (lo que usa el archivo)

- `crate::handlers::tools::handle_tools_list` (línea 70) — exportada vía `mod.rs`
- `crate::handlers::tools::handle_tools_call` (línea 1112) — exportada vía `mod.rs`
- `tests/mcp_tests.rs` (línea 1, 323, 517, 1559, 1649, 1713, 1751, 1821, 2118, 2184, 2252, 2279, 2433, 2466, 3118, 3236, 4341, 4361) — tests integración MCP (56 matches a `memory_list` o `search_multi`)

### Veredicto de impacto

- **blast radius:** MEDIO — tocar `memory_list` y `search_multi` afecta 56 tests de integración y 2 entradas en docs/api/MCP.md.
- **riesgo API pública:** MEDIO — cambiar la shape de la respuesta rompe consumidores que asumen `{records: [...], next_cursor: ...}` o `{hits: [...]}`. Mitigación: añadir campos (`truncated`, `byte_count`), NO renombrar/eliminar los existentes.
- **scope acotado a MCP** — no toca `vantadb`, `vantadb-python`, ni el core SDK. NO requiere rebuilds downstream.
- **pruebas:** tests integración en `tests/mcp_tests.rs` (no se requieren nuevos — los existentes validan shape; el contrato es el grep count).
- **docs:** actualizar `docs/api/MCP.md` (tabla de tools) con la nueva semántica de budgeting.

## Contrato

```powershell
# MCP-39 — Output budgeting (truncado + next_cursor)
Select-String -Path "vantadb-mcp/src/handlers/tools.rs" -Pattern "next_cursor|byte_budget|truncated" | Measure-Object | Select-Object Count
# >= 2
```

Estado actual (medido): `1` hit (`next_cursor` en línea 1383, `memory_list`).
Estado objetivo: `>=2` hits — añadir `byte_budget` en `config.rs` y `truncated`/`byte_count` en la respuesta de `search_multi` (y, opcionalmente, `memory_list`).

## Decisión de diseño (Spec)

### Defaults (pre-mortem: budget muy bajo trunca respuestas normales)

| Knob | Default | Razón |
|------|---------|-------|
| `byte_budget` | `40 * 1024` (40 KB) | 80% del cap de OpenCode (50 KB). Claude Code hard cap es 25k tokens (~100 KB) → 40 KB es seguro para ambos. Configurable via env `VANTADB_MCP_BYTE_BUDGET` para clientes con cap menor. |
| `min_byte_budget` | `1 * 1024` (1 KB) | Floor — por debajo de esto la respuesta no cabe. |
| `max_byte_budget` | `1024 * 1024` (1 MB) | Techo — por encima es pedirle a un cliente que renderice 1 MB de JSON. |

### Shape de respuesta

| Tool | Antes | Después |
|------|-------|---------|
| `memory_list` | `{records: [...], next_cursor: N}` | añadir: `byte_count: N, truncated: bool` (byte_count opcional, siempre presente) |
| `search_multi` | `{hits: [...]}` (vía `text_content_structured`) | envolver en `{hits: [...], byte_count: N, truncated: bool}` |

### Truncado

- Serializar a `serde_json::Value` y medir `byte_size = serialized.to_string().len()`.
- Si `byte_size > byte_budget` → pop hits del final hasta caber, marcar `truncated: true`, devolver `{hits, byte_count, truncated: true}`.
- Si cabe → `truncated: false`, `byte_count = serialized.len()`.
- `next_cursor` para `search_multi`: NO aplica (search no es paginada por SDK — `search_multi` ya cap con `top_k`). El flag `truncated` es la señal.
- `next_cursor` para `memory_list`: YA EXISTE (línea 1383). Añadir `byte_count` y `truncated` opcionales (no rompe compat).

### Helper genérico (pre-mortem: shapes distintos)

Crear `fn budget_value<T: Serialize>(value: &T, byte_budget: usize) -> (Value, bool, usize)` en `validation.rs` que:
1. Serializa a `serde_json::Value` (no `String`, para preservar tipos).
2. Si `to_string().len() <= byte_budget` → devuelve `(value, false, size)`.
3. Si excede → reduce arrays al final (pop) hasta caber, marca `truncated: true`.

Para `search_multi` (que tiene `hits: Vec<VantaSearchHit>`): si truncado, pop del final del array de hits antes de envolver.

## Herramientas

- `Read`, `Edit`, `Glob`, `Grep`
- `Bash` (powershell + cargo)
- `campaign_update_task_state` (MCP)
- `campaign_verify_cmd` (MCP)

## Steps

### Step 1: Añadir `byte_budget` a `McpConfig`

- **Archivos:** `vantadb-mcp/src/config.rs`
- **Acción:** añadir campos `byte_budget`, `min_byte_budget`, `max_byte_budget` a `McpConfig` con defaults 40KB / 1KB / 1MB. Leer env `VANTADB_MCP_BYTE_BUDGET` en `from_storage`, clamp entre min/max.
- **Verify:** `cargo check -p vantadb-mcp` → exit 0. ✅
- **Estado:** ✅ COMPLETED

### Step 2: Helper genérico `budget_value` en `validation.rs`

- **Archivos:** `vantadb-mcp/src/validation.rs`
- **Acción:** añadir `pub(crate) fn budget_value<T: Serialize>(...) -> (Value, bool, usize)` que trunca arrays al final. Tests inline (mismo módulo) cubriendo: cabe sin truncar, pop reduce a cabe, valor trivial (objeto/array vacío).
- **Verify:** `cargo test -p vantadb-mcp --lib budget` → 5/5 PASS. ✅
- **Estado:** ✅ COMPLETED

### Step 3: `memory_list` reporta `byte_count` + `truncated`

- **Archivos:** `vantadb-mcp/src/handlers/tools.rs` (línea 1383)
- **Acción:** cambiar `let result = json!({"records": page.records, "next_cursor": page.next_cursor})` → usar `budget_value` y añadir `byte_count` y `truncated` (sin tocar `records`/`next_cursor` keys — back-compat).
- **Verify:** `cargo test -p vantadb-mcp --test mcp_tests` → 77/77 PASS. ✅
- **Estado:** ✅ COMPLETED

### Step 4: `search_multi` envuelve en `{hits, byte_count, truncated}`

- **Archivos:** `vantadb-mcp/src/handlers/tools.rs` (línea 1565)
- **Acción:** usar `text_content_hits_with_budget` para preservar `text` como array (back-compat) y poner metadata en `structuredContent` (MCP 2025-06-18 envelope).
- **Verify:** `cargo test -p vantadb-mcp --test mcp_tests test_mcp_search_multi_round_trip` → PASS. ✅
- **Estado:** ✅ COMPLETED

### Step 5: Verificar contrato + docs

- **Archivos:** `docs/api/MCP.md` (líneas 172, 184)
- **Acción:** añadir nota en descripción de `memory_list` y `search_multi` sobre `byte_count`/`truncated`. Documentar `VANTADB_MCP_BYTE_BUDGET` env var (con defaults y límites por cliente). Nueva sección "Output budgeting (`byte_budget`, MCP-39)".
- **Verify:** `Select-String ... | Measure-Object | Select-Object Count` = 7 ≥ 2. ✅
- **Estado:** ✅ COMPLETED

### Step 6: Verify full + commit

- **Acción:** `cargo fmt --check -p vantadb-mcp` ✅, `cargo test -p vantadb-mcp` ✅ (144/144), `scripts/validate-docs-coverage.ps1` → pre-existing failure in `vantadb/src/llm.rs:135` (out of MCP-39 scope; pre-existing debt). Reportar al orquestador (vanta-worker NO hace commit; vanta-lead ejecuta `git commit -m "feat: MCP-39 — Output budgeting (truncado + next_cursor)"`).
- **Verify:** todos ✅ (MCP scope). Pre-existing `llm.rs:135` clippy failure bloquea el full verify pero es independiente de MCP-39.
- **Estado:** ✅ COMPLETED

## Dependencias

- Task 5: MCP-37 — Perfiles de tool surface (completado en commit previo; MCP-39 hereda `McpConfig` con el campo `profile` ya integrado).

## Notas

- vanta-worker NO ejecuta `git commit` — deja archivos staged y reporta al orquestador (vanta-lead).
- Default `byte_budget = 40KB` chosen por el pre-mortem (80% de OpenCode cap 50KB, seguro para Claude Code 25k tokens).
- Helper `budget_value` es genérico → no depende de la shape de `memory_list` vs `search_multi` (Ponytail rung 1: una sola implementación, dos consumidores).

## Context Save Point

- **Fecha:** 2026-08-29
- **Branch:** develop
- **CI pendiente:** sí (verify full)
- **Decisiones:** byte_budget default 40KB (pre-mortem), helper genérico budget_value (Ponytail rung 1), no romper `next_cursor`/`records` keys (back-compat).
- **Problemas conocidos:** ninguno.
- **Próxima tarea:** FIND-24b — Fix docs drift MCP skill.
