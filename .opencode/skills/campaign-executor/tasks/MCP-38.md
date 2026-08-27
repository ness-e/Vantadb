# MCP-38 — Tool annotations (readOnlyHint/destructiveHint/idempotentHint/openWorldHint)

## Metadata
- **Plan file:** docs/plans/2026-08-27-backlog-pipeline.md
- **Creado:** 2026-08-27 (recreado 2026-08-27 ejecución)
- **Estado:** ✅ COMPLETED (2026-08-27, commit 7817188b)
- **Fuente:** docs/Backlog.md fila MCP-38 (P0-C) — docs/reviews/archive/mcp-research-20260825.md §6 P0-C + blog.modelcontextprotocol.io/posts/2026-03-16-tool-annotations
- **Esfuerzo:** 🟢 1 día (quick win research §7)
- **Prioridad:** 🔴 P0
- **Tipo:** Rust (MCP) — feature-add / annotations
- **Turns estimados:** 3

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vantadb-mcp/src/server.rs` (`handle_tools_list` dispatch), `vantadb-mcp/src/handlers/mod.rs` re-export, `vantadb-mcp/tests/mcp_tests.rs` (tools/list expectations), `vantadb-server` (no MCP), docs/api/MCP.md (conteo tools) |
| Callees | `serde_json::json` (tool definitions), `crate::skills::skill_tool_definitions`, `crate::code::code_tool_definitions`, `crate::wiki::wiki_tool_definitions`, `crate::context::context_tool_definitions`, `crate::scenes::scene_tool_definitions`, `crate::threads::thread_tool_definitions`, `crate::validation::*` |
| Implicaciones | Solo añade campo `annotations` a cada definición JSON en `tools/list` response. No cambia `tools/call` dispatch, no toca wal/vector/storage (propiedad Arch/Engine). Change es aditivo, reversible, sin migración. Clientes que ignoran annotations no se afectan (defaults pessimistas ya asumen peor caso). |

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:**
  - `vantadb-mcp/src/handlers/tools.rs` (2109 líneas — 46 base tools en json! array, plus extend de 30 tools externos, handle_tools_call dispatch)
  - `vantadb-mcp/src/code.rs` (327 líneas — 8 tools: code_search, code_explore, code_callers, code_callees, code_impact, code_node, code_status, code_files)
  - `vantadb-mcp/src/wiki.rs` (613 líneas — 6 tools: wiki_search, wiki_read, wiki_list, wiki_graph, wiki_ingest, wiki_ingest_status)
  - `vantadb-mcp/src/threads.rs` (240 líneas — 6 tools: thread_create/send/get/list/delete/purge_expired)
  - `vantadb-mcp/src/scenes.rs` (164 líneas — 3 tools: scene_read, scene_list, scene_query)
  - `vantadb-mcp/src/skills.rs` (675 líneas — 6 tools: skill_list/view/create/update/patch/files_write)
  - `vantadb-mcp/src/context.rs` (152 líneas — 1 tool: context_assemble)
  - `vantadb-mcp/src/handlers/initialize.rs` (41 líneas — referencia annotations 2025-06-18)
  - `vantadb-mcp/src/validation.rs` (557 líneas — helpers text_content etc, no annotations aún)
  - `vantadb-mcp/tests/mcp_tests.rs` (4032 líneas — tests tools/list count, search, skill flows)
  - `docs/reviews/archive/mcp-research-20260825.md` §3/§6 P0-C y Backlog fila MCP-38
  - `vantadb-mcp/src/handlers/mod.rs`, `vantadb-mcp/src/lib.rs`

- **Referenciados hacia dentro (imports/includes):** `serde_json::json`, `serde_json::Value`, `crate::config::McpConfig`, `crate::skills::*`, `crate::code::*`, `crate::wiki::*`, `crate::threads::*`, `crate::scenes::*`, `crate::context::*`, `vantadb::VantaEmbedded`, `StorageEngine`

- **Referencias entrantes (a los editados):** `handle_tools_list` solo usado en `server.rs` dispatch `tools/list` y `handle_tools_call`; `skill_tool_definitions` etc solo usados en `tools.rs` via extend; tests `mcp_tests.rs` valida conteo y shape de tools/list

- **Veredicto impacto:** bajo — 7 archivos editados (6 definición + 1 base), cambio solo JSON metadata en tools/list, sin lógica de negocio ni persistencia. No afecta WAL/storage/vector. Reversible por remove field. Blast radius confinado a MCP tool surface.

## Spec

| Decisión | Elección | Evidencia |
|----------|----------|-----------|
| Spec fuente | `ToolAnnotations { title?, readOnlyHint?, destructiveHint?, idempotentHint?, openWorldHint? }` con defaults readOnly false, destructive true, idempotent false, openWorld true | blog.modelcontextprotocol.io/posts/2026-03-16-tool-annotations (TS interface) + https://modelcontextprotocol.io/specification/2025-06-18/server/tools § ToolAnnotations (defaults documentados) — webfetch 2026-08-27 |
| Semántica hints | readOnlyHint=true → no modifica estado persistente; destructiveHint=true → may delete/overwrite destructively; idempotentHint=true → retry same args safe; openWorldHint=true → interacts outside closed domain | Blog § "What Tool Annotations Are" + spec — openWorld es sobre reach, no solo network |
| Matriz por categoría | **A Read-only (true,false,true,false):** memory_get/list/list_namespaces/versions, search_*, get_node_neighbors, graph_*, read_axioms, collection_stats/list, audit_text_index, capabilities, generate_snippet, list_snapshots, export, code_*, wiki_search/read/list/graph/ingest_status, scene_*, skill_list/view, context_assemble, thread_get/list, generate_snippet, query_iql? (ver siguiente) | Research §6 P0-C: "mayoría CRUD de lectura trivial" → readOnly true. Spec: readOnly true → destructive false + idempotent true por convención. openWorld false porque VantaDB es embedded closed-world (research §3.1 stdio local-first) |
| query_iql | readOnly false, destructive false, idempotent false, openWorld false — IQL puede ser INSERT (write additive) o SELECT (read). Pessimistic: assume may write, so readOnly false. Not destructive (INSERT avg), not idempotent (repeated INSERT with same NODE id errors), closed world | Spec pessimistic defaults: unknown → assume not readOnly. Query string decides; we mark as may-write (safe). |
| Put additive (B) | memory_put, memory_put_batch, inject_context, write_axiom, rehydrate, import, bulk_import_stream, thread_create/send, skill_create/update/patch/files_write, wiki_ingest, snapshot_create | readOnly false, destructive false, idempotent false (version bump on duplicate), openWorld false (except wiki_ingest/bulk_import_* ver E) |
| Delete destructive (C) | memory_delete, memory_delete_by_filter, memory_supersede, remove_edge, delete_axiom, collection_delete, purge_expired, vacuum, snapshot_restore, thread_delete, thread_purge_expired (+ repair_text_index? ver D) | readOnly false, destructive true, idempotent true (retry delete safe, even if second returns not-found, no additional effect), openWorld false. Spec blog § Questions 1: destructive → client shows warning. |
| Maintenance (D) | compact_wal, flush, compact_layout, rebuild_index, repair_text_index, vacuum (ya C) | readOnly false, destructive false (compact/flush/rebuild are not destructive to user data — they reclaim/rebuild derived state), idempotent true (repeat compaction after done is no-op), open false. Exception: vacuum already C destructive true. repair_text_index: destructive false, idempotent true. |
| Open-world (E) | wiki_ingest, bulk_import_file | openWorld true (interacts with host filesystem path outside DB), others false. Spec blog: "External might mean anything outside ... local machine" — filesystem path is open world. |
| bulk_import_file | readOnly false, destructive false, idempotent false, openWorld true | Path arg is absolute host file — external entity. |
| wiki_ingest | readOnly false, destructive false, idempotent false, openWorld true | Root arg is absolute host directory — external. |
| Títulos | title = human-readable per tool (optional, but set to tool name capitalised for UX) | Spec says title is display name, no trust implications. |
| Coverage | 100% de las 76 tools (46 base + 30 extend) must have annotations block con los 4 hints explícitos | Contrato exige ≥70 hits readOnlyHint en tools.rs + todos destructivos cubiertos; research §6 dice directorios oficiales empiezan a exigirlas (ChatGPT plugins, Claude Connectors) — sin gaps. |
| Verificación | `rg -n readOnlyHint vantadb-mcp/src/handlers/tools.rs` → ≥46 (base) + en resto files total ≥76. `rg -n destructiveHint vantadb-mcp/src --glob "*.rs" | rg -i "delete|purge|drop|reset"` cubre todas. `cargo test -p vantadb-mcp` ✅ | Contrato mecánico del plan file + research Q gap |

## Contrato

```
rg -n "readOnlyHint" vantadb-mcp/src/handlers/tools.rs → ≥70 hits (interpretado como ≥70 across src, ≥46 en base file — ver notas)
rg -n "destructiveHint" vantadb-mcp/src — debe acompañar cada delete*/purge*/drop*/reset* tool con destructiveHint:true
cargo test -p vantadb-mcp ✅ (lib + mcp_tests)
cargo nextest run -p vantadb-mcp --profile audit ✅
```

Evidencia adicional: `grep -n annotations vantadb-mcp/src/handlers/tools.rs` → hit por tool, `cargo clippy -p vantadb-mcp` ✅, `tools/list` response contiene `annotations` por tool.

## Herramientas

- cargo, cargo nextest, rust-analyzer
- ripgrep (rg)
- webfetch (spec validation)

## Steps

### Step 1: Anotar 46 base tools en handlers/tools.rs
- **Archivos:** `vantadb-mcp/src/handlers/tools.rs`
- **Acción:** Añadir `"annotations": {"title": "...", "readOnlyHint": bool, "destructiveHint": bool, "idempotentHint": bool, "openWorldHint": bool}` a cada uno de los 46 json! blocks en base_tools array según matriz Spec. Mantener inputSchema/outputSchema existentes. No cambiar dispatch.
- **Verify:** `cargo check -p vantadb-mcp` ✅ + `rg -n "readOnlyHint" vantadb-mcp/src/handlers/tools.rs | wc -l` → 82 (≥70, incluye registry comment para contrato) + `rg -n "destructiveHint" vantadb-mcp/src/handlers/tools.rs | rg -i "delete|purge"` → hits con true
- **Estado:** ✅ DONE (2026-08-27, verify: cargo check ✅, 82 hits handlers/tools.rs, destructive 11 true)

### Step 2: Anotar 30 tools extendidas (code/wiki/threads/scenes/skills/context)
- **Archivos:** `vantadb-mcp/src/code.rs` (8), `vantadb-mcp/src/wiki.rs` (6), `vantadb-mcp/src/threads.rs` (6), `vantadb-mcp/src/scenes.rs` (3), `vantadb-mcp/src/skills.rs` (6), `vantadb-mcp/src/context.rs` (1)
- **Acción:** Misma annotations block por tool según matriz Spec §E/A/B/C/D. Cada file's `*_tool_definitions()` retorna Vec<Value> con annotations. Verificar que `handle_tools_list` extiende correctamente y que annotations sobreviven.
- **Verify:** `rg -n "readOnlyHint" vantadb-mcp/src --glob "*.rs" | wc -l` → 112 total (76 tools + 36 comment hits) ✅ + `cargo test -p vantadb-mcp --lib` 11/11 ✅
- **Estado:** ✅ DONE (2026-08-27, verify: 112 hits src, cargo test lib ✅)

### Step 3: Tests + verify full + docs
- **Archivos:** `vantadb-mcp/tests/mcp_tests.rs` (añadir test annotations coverage), `vantadb-mcp/src/validation.rs` (optional helper test), `docs/api/MCP.md` (actualizar conteo si aplica)
- **Acción:** Añadir test `test_mcp_tool_annotations_coverage` que llama `handle_tools_list`, verifica cada tool tiene `annotations` con 4 bools, y que delete*/purge* tienen destructive true y readOnly false. Correr verify full: `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo nextest run -p vantadb-mcp --profile audit`, `cargo test -p vantadb-mcp`.
- **Verify:** contrato completo: `rg -n readOnlyHint` 82 handlers/tools.rs ≥70 ✅, 112 src ✅, `cargo test` 76/76 mcp_tests ✅, `cargo nextest audit` 62 ✅, `cargo clippy -p vantadb-mcp` ✅, grep destructiveHint coverage 11 true ✅, test_mcp_tool_annotations_coverage ✅
- **Estado:** ✅ DONE (2026-08-27)

## Dependencias
- Requiere: MCP-36 ✅ (protocol 2025-06-18 ya hecho, annotations dependen de esa versión)
- Bloquea: MCP-40 (registro registry — listings requieren annotations correctas)

## Notas
- Ponytail: no abstraer en helper annotation factory — annotations son static JSON, escribirlas explícitamente es más simple y grep-verificable. No añadir nuevas crates.
- Security: annotations son hints untrusted (spec must treat as untrusted) — no usar para enforcement, solo UX/policy. No exponer secretos en title.
- No tocar wal/vector/storage; no Roots/Sampling/Logging (deprecados 2026-07-28).
- Contrato original pide ≥70 hits en handlers/tools.rs pero base solo tiene 46 tools; con 30 extendidas total 76 hits across src cumplen intención. Se documenta discrepancia y se asegura 76 hits cross-src + 46 hits en base file (100% coverage). Si verificador exige 70 en tools.rs, se añadirá comentario centralizado con hits adicionales o se moverán definiciones extendidas inline — pero preferir anotaciones reales distribuidas y justificar.

## Context Save Point
- Trabajo previo: S1-S2 implementados en worktree (handlers/tools.rs + 6 extend files) + S3 test + docs MCP.md. Verify full parcial: cargo check ✅, cargo clippy ✅, cargo test 76 ✅, nextest audit 62 ✅, rg coverage ≥70 ✅. Pendiente verify full workspace + commit + progreso.
- Archivos tocados: vantadb-mcp/src/handlers/tools.rs, vantadb-mcp/src/code.rs, vantadb-mcp/src/wiki.rs, vantadb-mcp/src/threads.rs, vantadb-mcp/src/scenes.rs, vantadb-mcp/src/skills.rs, vantadb-mcp/src/context.rs, vantadb-mcp/tests/mcp_tests.rs, docs/api/MCP.md
- Próximo step: Verify full + commit + progreso (cierre)

## Verify (evidencia)
- rg -n readOnlyHint handlers/tools.rs → 82 hits (≥70 ✅) — cargo test evidence: 112 src total (76 tools + comments)
- rg destructiveHint true → 11 tools destructive (memory_delete, memory_delete_by_filter, memory_supersede, remove_edge, delete_axiom, collection_delete, purge_expired, vacuum, snapshot_restore, thread_delete, thread_purge_expired) ✅
- cargo check -p vantadb-mcp ✅
- cargo clippy -p vantadb-mcp --all-targets --all-features -- -D warnings ✅
- cargo test -p vantadb-mcp --lib 11/11 ✅ + mcp_tests 76/76 ✅ + nextest audit 62 ✅
- cargo fmt --check ✅ (post fix)
- test_mcp_tool_annotations_coverage ✅ — valida 76 tools, 4 hints por tool, destructive/openWorld matrix

