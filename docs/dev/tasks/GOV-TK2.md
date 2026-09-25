# GOV-TK2 — Re-verificar gap tools MCP (VERIFICACIÓN, 0 código)

## Metadata
- **Plan file:** docs/dev/plans/2026-09-07-cleanup-gates.md (Task 2, Wave0)
- **Fuente original:** Backlog GOV-TK line 456: "binario MCP expone 15 tools, skill documenta 33"
- **Re-scoping (plan 2026-09-07):** la fila tiene números falsos en ambos lados; primero medir realidad. Skill actual declara **79 tools** (`skills/vantadb-mcp/SKILL.md:116`).
- **Tipo:** research / verificación read-only — **PROHIBIDO implementar fix**
- **Esfuerzo:** 🟢 2-4h (real: ~30min, conteo estático suficiente, sin compilar)
- **Estado:** ✅ COMPLETED (2026-09-08)
- **Cambios fuente:** 0 (solo este task file)

## SDP — Skills cargadas
- **campaign-executor** (base) — orquestación
- **progreso** (base) — registro
- **source-driven-development** (base type MCP server) — conteo contra código real, no memoria del modelo
- **security-and-hardening** (base type, no aplicable — sin cambios de código)
- Descartadas por scoring-ruido (no aplican a verificación read-only): systematic-debugging, browser-testing-with-devtools, design-review, claude-api
- **SDP: campaign-executor, progreso, source-driven-development (+ ponytail full siempre activo)**

## Medición (conteo estático, comandos reproducibles)

```powershell
rg -c '"name": "[a-z_]+"' vantadb-mcp/src/handlers/tools.rs vantadb-mcp/src/code.rs vantadb-mcp/src/wiki.rs vantadb-mcp/src/threads.rs vantadb-mcp/src/context.rs vantadb-mcp/src/skills.rs vantadb-mcp/src/scenes.rs
# → tools.rs:49, code.rs:8, wiki.rs:6, threads.rs:6, context.rs:1, skills.rs:6, scenes.rs:3
rg -o '"name": "[a-z_]+"' vantadb-mcp/src/handlers/tools.rs | Sort-Object | Get-Unique
# → 49 nombres únicos, sin duplicados
```

### Tabla por prefijo (contrato: contar por prefijo, no total ciego)

| Grupo | Count | Fuente | Tools |
|-------|-------|--------|-------|
| Base `handlers/tools.rs` (memory_*, search_*, collection_*, graph_*, query_iql, axioms, maintenance, snapshot, import/export, capabilities, embed_texts) | **49** | `tools.rs:73-999` (49 `"name"` únicos, 0 dupes) | memory_put, memory_put_batch, memory_get, memory_delete, memory_delete_by_filter, memory_list, memory_list_namespaces, memory_versions, memory_supersede, memory_recall, memory_search, query_iql, search_semantic, search_memory, search_with_method, search_multi, get_node_neighbors, graph_page_rank, graph_degree_centrality, graph_traverse, graph_topological_sort, graph_is_dag, remove_edge, inject_context, read_axioms, write_axiom, delete_axiom, collection_stats, collection_list, collection_delete, audit_text_index, capabilities, generate_snippet, list_snapshots, export, import, bulk_import_file, bulk_import_stream, rehydrate, purge_expired, compact_wal, flush, compact_layout, vacuum, rebuild_index, repair_text_index, snapshot_create, snapshot_restore, embed_texts |
| Review-agent Skills (`skill_*`) | **6** | `skills.rs` + merge `tools.rs:1005` | skill_list, skill_view, skill_create, skill_update, skill_patch, skill_files_write |
| Code Intelligence (`code_*`) | **8** | `code.rs` + merge `tools.rs:1006` | code_search, code_explore, code_callers, code_callees, code_impact, code_node, code_status, code_files* (*stub not-supported documentado) |
| Wiki Knowledge (`wiki_*`) | **6** | `wiki.rs` + merge `tools.rs:1007` | wiki_search, wiki_read, wiki_list, wiki_graph, wiki_ingest, wiki_ingest_status |
| Context Engine | **1** | `context.rs` + merge `tools.rs:1008` | context_assemble |
| Scenes API (`scene_*`) | **3** | `scenes.rs` + merge `tools.rs:1009` | scene_read, scene_list, scene_query |
| Threads (`thread_*`) | **6** | `threads.rs` + merge `tools.rs:1010` | thread_create, thread_send, thread_get, thread_list, thread_delete, thread_purge_expired |
| **TOTAL perfil Full (default)** | **79** | 49 + 30 | — |

### Merge y perfiles verificados en código
- `handle_tools_list` (`tools.rs:1003-1025`): base + 6 `extend` explícitos (skills, code, wiki, context, scenes, threads), luego filtro por perfil (`tools.rs:1013-1022`).
- Perfil **Full (default)**: permite las 79 (`tools.rs:1089-1134` + resto). Perfil **Dev**: ~38 (memory 22 + dev 16, sin code/wiki/skill/thread/scene). Perfil **Memory**: 22.
- **Sin compilación del binario** (stop condition del plan lo permite): el merge es explícito y estático; 1 corrida innecesaria. Nota registrada, no bloqueo.

## Comparativa de números

| Fuente | Número | Estado |
|--------|--------|--------|
| Fila backlog ("binario 15 / skill 33") | 15 vs 33 | ❌ **STALE en ambos lados** (versión vieja del skill y del binario) |
| Skill actual (`SKILL.md:116`: "Available MCP Tools (79)", 49 core + 30 resumidas) | 79 | ✅ **EXACTO** — coincide con 49+30 del código |
| Código real (perfil Full) | **79** | ✅ ground truth |
| Comentario interno `tools.rs:25` ("76 tools total, 46 base + 30 extend") | 76 | ⚠️ **STALE leve** — base creció 46→49 (memory_recall, memory_search, embed_texts); extend sigue en 30. Solo comentario, sin efecto funcional |

## VEREDICTO: 🔴 GAP MUERTO

- **No hay gap real.** La skill declara 79 y el binario (perfil Full, default) expone exactamente esas 79. Las 18 tools skill_/code_/wiki_ que motivaron la fila original **ya están mergeadas** en `handle_tools_list`.
- **Acción requerida:** fila GOV-TK del backlog → **historial con esta evidencia** (doc-only, lo hace el orquestador/DOC-SYNC-01 o el owner; fuera de scope de esta tarea).
- **NO crear tarea de fix.** El único drift (comentario "76 tools" en `tools.rs:25,35,1090`) es cosmético de 1 línea; si se quiere corregir, es un `chore:` trivial fuera de esta tarea — **NOTICED BUT NOT TOUCHING** (scope discipline).
- **Incógnitas uphill del task file viejo (D5, CHANGELOG, secretos CI)**: obsoletas — pertenecían al scoping de release, no a la verificación. Se archivan con la fila.

## Steps
### Step 1: Conteo estático por prefijo — ✅ DONE
- **Verify:** `rg -c` por archivo + lista única de 49 nombres base + tabla por prefijo arriba
### Step 2: Verificar merge + perfiles en `handle_tools_list` — ✅ DONE
- **Verify:** `tools.rs:1003-1011` (6 extends) + `tools.rs:1028-1134` (Memory 22 / Dev ~38 / Full 79)
### Step 3: Comparar contra skill y fila, emitir veredicto — ✅ DONE
- **Verify:** skill 79 == código 79 → gap muerto; fila 15-vs-33 stale en ambos lados

## Context Save Point
- **Fecha:** 2026-09-08
- **Branch:** develop (sin cambios — `git status` limpio salvo este task file)
- **Decisiones:** conteo estático como proxy (pre-mortem #1 del plan); sin corrida del binario por merge explícito
- **Próxima acción (orquestador):** mover fila GOV-TK a historial con evidencia de este reporte; no abrir tarea de fix
