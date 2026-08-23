# Review de Módulo — `vantadb-mcp/`

**Fecha:** 2026-08-22 · **Revisor:** segunda opinión, contexto fresco (no participó en P22/P25) · **Alcance:** revisión profunda del servidor MCP stdio JSON-RPC 2.0

---

## 1. Resumen

`vantadb-mcp` es el canal agentic de VantaDB: un servidor **stdio JSON-RPC 2.0** que expone **59 tools** (36 core + 6 `skill_*` + 8 `code_*` + 6 `wiki_*`, contadas contra source hoy), 4 recursos (`metrics://`, `schema://` listados; `memory://{ns}/{key}` y `namespace://{ns}` servibles), y 4 prompts. La arquitectura es deliberadamente "thin wrapper": toda la semántica vive en el core (`VantaEmbedded`/SDK), la capa MCP solo valida en el trust boundary, despacha y serializa.

**Estado verificado hoy:** los bugs de P22 están presentes y correctos (`ensure_indexes_current` en arranque — server.rs:36-44; métrica per-request `distance_metric` con warn si falta — tools.rs:789-806; distancia real en `search_semantic` — tools.rs:913-921; `DimensionMismatch` en put/batch/search/search_semantic — tools.rs:455-467, 555-569, 769-781, 885-895; streaming OOM vigente vía `for_each_record` — validation.rs:372-407). De la lista P25 **MCP-16..23 y MCP-25..27 están implementados**; quedan abiertas **MCP-24** (search_with_method/multi), **MCP-28** (bulk import no direccionable) y **MCP-29** (IQL sobre memoria, diferido).

**Suite:** `cargo test -p vantadb-mcp --test mcp_tests` → **60 passed / 0 failed** (18.9 s, ejecutado hoy). Documentación sincronizada: `skills/vantadb-mcp/SKILL.md` y `.opencode/skills/vantadb-mcp/SKILL.md` → **hash SAME** (SHA256 `CF70E4F5…`), api-reference.md actualizado hoy.

**Veredicto general:** módulo maduro y bien disciplinado (8.3/10). Los hallazgos nuevos son de capa protocolo (notifications JSON-RPC, procesamiento serial del loop stdio) más que de dominio.

---

## 2. Arquitectura

```
stdin (línea JSON-RPC) ──▶ run_stdio_server (server.rs)
                            ├─ parse → RpcRequest {jsonrpc, id, method, params}
                            ├─ dispatch_request:
                            │    initialize / tools/list / prompts/*  → inline (sync, trivial)
                            │    tools/call / resources/read          → Semaphore + spawn_blocking
                            │                                            + timeout (McpConfig.request_timeout)
                            ├─ McpError → {code,message} | Ok(Value) → RpcResponse → stdout
                            └─ McpMetrics (requests/errors/active) log cada 30s

tools/call ──▶ handle_tools_call (handlers/tools.rs, dispatch gigante por nombre)
                ├─ memory_*   → VantaEmbedded (SDK) tras validate_identifier/payload/vector
                ├─ query_iql  → Executor::execute_hybrid
                ├─ skill_*    → skills.rs → SkillStore (core)
                ├─ code_*     → code.rs   → graphrag/BFS (core)
                └─ wiki_*     → wiki.rs   → WikiStore (core) + facade async ingest (MEM-52)

Validación (validation.rs): identifiers/payload/vector/sparse/filters($ops)/metadata(VantaValue)/
search_profile(serde del core + bounds), for_each_record (paginación OOM-fix),
error_content({isError:true}) vs JSON-RPC error.
```

Decisiones de diseño correctas y documentadas:

- **Dos canales de error (MEM-32):** errores de parámetros/protocolo → JSON-RPC `-32602/-32601`; errores de dominio → `Ok(error_content(...))` auto-corregible por el LLM. Aplicado consistentemente en tools.rs, skills.rs, code.rs y wiki.rs (wiki.rs:311-318 lo documenta explícitamente).
- **Trust boundary único:** toda validación ocurre al entrar (AUD-045/046/048/050 tienen comentarios con referencia cruzada a su origen).
- **Sin lógica duplicada:** skills/code/wiki delegan en el core; el wire shape de `search_profile` es el serde del core (MEM-01/D13/D19), con test de paridad nativo↔MCP (`test_search_profile_mcp_passthrough_parity_with_native`).
- **Ownership sin side-channel:** `skill_view/update/patch/files_write` responden idéntico a "no existe" y "no te pertenece" (skills.rs:191-207).
- **Acumuladores de grafo excluidos por decisión** (herramientas de paralelismo in-process sin estado del engine) — documentado en SKILL.md, no es gap.

---

## 3. Fortalezas

1. **Cobertura de contrato real:** 60 pruebas integrais que ejercitan el contrato de verdad (round-trips export→import→get en DB fresca, delete_by_filter con `$gt`, batch all-or-nothing, page_rank convergencia A→B→C, ids u128 como strings, sanitización IQL/injection, shape de explain, límites oversized). No son humo: verifican payloads, no solo exit codes.
2. **Disciplina de validación ejemplar:** `parse_metadata` rechaza objetos/arrays mixtos explícitamente en vez de silenciar filtros (ERR-026, con test); `parse_filter_ops` unifica formato flat + operadores con el CLI (AUD-048); vectores con check de finitud y dim contra el índice vivo.
3. **Streaming anti-OOM sólido:** `collection_stats/list/delete/export` paginan vía `for_each_record` (máx `max_list_limit` en memoria), con test de namespace grande acotado (`test_collection_stats_large_namespace_bounded`). El fix ERR-021 sigue vigente tras P22/P25.
4. **Docs como contrato versionado:** SKILL.md duplicado con hash SAME verificado hoy; cuenta de tools declarada (59) coincide con source; behavior notes F4-F11 verificables contra tests; advertencia explícita sobre el shape de explain (T15) que previene aserciones rotas de agentes.
5. **Comentarios con genealogía:** cada fix trae su AUD-/MCP-/MEM-id y el porqué (p.ej. tools.rs:449-454 explica por qué un vector de dim equivocada corrompe HNSW silenciosamente). Esto hace el módulo auditable.
6. **Seguridad razonable para stdio local:** confirm destructiva en `collection_delete`, path traversal bloqueado en `skill_files_write` (absolutas, `..`, drive-letters, null bytes — skills.rs:599-623), base64 validado sin dependencia nueva, fail-closed en metadata corrupta de skills.

---

## 4. Hallazgos

| # | Severidad | Archivo:línea | Hallazgo |
|---|-----------|---------------|----------|
| H1 | 🔴 | `protocol.rs:8-14` + `server.rs:102` | **Notifications JSON-RPC se rechazan como parse error.** `RpcRequest.id` es `Value` (no `Option<Value>` ni `#[serde(default)]`): una notification válida sin `id` (p.ej. `notifications/initialized` que todo cliente MCP envía tras el handshake) falla la deserialización → el server escribe una respuesta `-32700` con `"id": null`. Viola JSON-RPC 2.0 §4.1 (a las notifications no se responde). Los clientes mayores toleran la línea espuria, pero cualquier cliente estricto o multiplexado puede tratarla como error de sesión. Fix mínimo: `#[serde(default)] pub id: Value` + skip-write cuando el request original no traía `id`. Sin test que cubra notifications (0 matches en mcp_tests.rs). |
| H2 | 🟠 | `server.rs:83-153` | **El loop stdio procesa serialmente; la maquinaria de concurrencia es dead-code efectivo.** `dispatch_request(...).await` corre inline en el loop: el `Semaphore(max_concurrency)` y `spawn_blocking` nunca llegan a superponer requests. Un `rebuild_index` de 50 s bloquea todo, incluso requests triviales. Además `test_mcp_concurrent_requests` prueba `dispatch_request` directamente, no el loop — da falsa confianza. Legal según spec pero contradice la configuración documentada (`max_concurrency: 32`) y degrada DX en agentes que hacen fan-out. |
| H3 | 🟠 | `server.rs:150-153` | **El graceful shutdown descarta la respuesta ya computada.** Cuando `running=false`, el `break` ocurre ANTES de serializar/escribir `response` (líneas 155+): el request in-flight completa trabajo pero el cliente nunca recibe resultado ni error (timeout del lado cliente). El log dice "Graceful shutdown after processing in-flight request" pero la respuesta se pierde. Fix: escribir la respuesta antes del break. |
| H4 | 🟡 | `tools.rs:879` | **`search_semantic` no acota `k`** (`args["k"].as_u64().unwrap_or(5) as usize`, sin `min(config.max_top_k)`). Inconsistente con `search_memory` (tools.rs:787 sí acota). Un `k` gigante materializa todo el HNSW en memoria. Una línea lo arregla. |
| H5 | 🟡 | `server.rs:224-234` | **Timeout no cancela el trabajo:** al vencer `request_timeout` la respuesta vuelve, pero el `spawn_blocking` sigue corriendo y retiene el permit hasta terminar (tokio no cancela blocking tasks). N operaciones colgadas saturan el pool permanentemente. Mitigación aceptable para v1, pero conviene documentarlo o instrumentar (log warn con duración excedida). |
| H6 | 🟡 | `tools.rs:1028-1029` | **`total_bytes` de `collection_stats` usa `format!("{:?}", v).len()`** para valores de metadata: longitud de Debug ≠ bytes reales (un Int `1` = 1 char, un String con escapes infla). Métrica aproximada publicada sin caveat en SKILL.md. O se documenta como estimación o se mide en serio. |
| H7 | 🟡 | `handlers/resources.rs:99` | **`namespace://{ns}` hardcodea `limit: 100` sin paginación expuesta**, inconsistente con `max_list_limit=10_000` del resto del server. Namespaces grandes quedan truncados silenciosamente en el recurso (con `next_cursor` presente, al menos). |
| H8 | 🟡 | `tools.rs:1389-1400` + `wiki.rs:223-246` | **Superficie LLM06 (excessive agency):** `bulk_import_file(path)` y `wiki_ingest(root)` aceptan rutas arbitrarias del host → un prompt-injectado puede hacer que el server lea archivos locales (.vdbdump cualquiera, árbol markdown completo) y los ingiera a la DB, donde el agente los lee vía wiki_read/memory paths. Para stdio local single-user es riesgo aceptado y está documentado como diseño ("host-side file"), pero no hay allowlist/root-cap ni nota de threat model. Recomendado: documentar el riesgo en SKILL.md § Security como mínimo. |
| H9 | 🟢 | `code.rs:268-271` | `code_files` como stub documentado que siempre devuelve error: honesto y barato; correcto mantenerlo para paridad de superficie TDAM. |
| H10 | 🟢 | `tools.rs:916-928` | En `search_semantic`, hits cuyo nodo falló al fetch se omiten silenciosamente (`if let Ok(Some(...))`). Con nodos vivos esto no debería pasar; un warn! ayudaría a detectar drift índice→store. |
| H11 | 🟢 | `handlers/prompts.rs:99` | Prompt desconocido devuelve `-32602`; aceptable, aunque `-32601`-style sería más preciso. No bloquea. |

---

## 5. Cobertura de tools vs SDK (verificada hoy contra `src/sdk/`)

### Expuestos vía MCP (verificado contra source, 2026-08-22)

| Primitiva SDK | Tool MCP | Nota |
|---|---|---|
| `put` / `put_batch` | `memory_put` / `memory_put_batch` (MCP-19) | batch con parity AUD-046 |
| `get` / `delete` / `list` / `list_namespaces` | `memory_get/delete/list/list_namespaces` | filters AUD-048 |
| `delete_by_filter` | `memory_delete_by_filter` (MCP-18) | guard ≥1 filtro |
| `search` (+SearchProfileConfig MEM-01) | `search_memory` | explain, profile passthrough |
| `search_vector` | `search_semantic` | distancia real (MCP-03) |
| `rebuild_index`, `audit_text_index(_deep)`, `repair_text_index` | homónimas (MCP-20) | reports serde verbatim |
| `purge_expired`, `compact_wal`, `flush`, `compact_layout` | homónimas (MCP-16/23) | ✅ implementadas |
| `capabilities`, `generate_snippet` | homónimas (MCP-26) | ✅ |
| `import_records` + `export_line_from_record` | `export` / `import` (MCP-17, cap 10 MB) | round-trip testeado |
| `bulk_import_stream` / `bulk_import_file` | homónimas (MCP-25) | deuda MCP-28 conocida |
| `graph_bfs/dfs(+filtered)`, `graph_topological_sort`, `graph_is_dag` | `graph_traverse` / homónimas (MCP-22) | accumulators excluidos por diseño |
| `graph_page_rank`, `graph_degree_centrality` | homónimas (MCP-21) | GDS desde el agente ✅ |
| `get_node`, insert_node vía IQL, `recover_archived_nodes` | `code_node`, `query_iql`, `rehydrate` | |
| `operational_metrics` | recurso `metrics://` + `code_status` | |

### Métodos SDK SIN tool (gap real, contrastado con backlog)

| Método | Ubicación | ¿Trackeado? | Evaluación |
|---|---|---|---|
| `versions` / `get_version` | `sdk/api.rs:451,469` | ❌ **No trackeado** | Gap nuevo propuesto: historial de versiones de un key inalcanzable vía MCP (el agente ve `version` en records pero no puede listarla). Barato de exponer. |
| `supersede` | `sdk/api.rs:840` | ❌ **No trackeado** | Cadena supersedence (degradación semántica) invisible para el agente. |
| `similar_to_key` | `sdk/api.rs:1520` | ❌ **No trackeado** | "Más como este" es una operación natural de agente; hoy requiere get→extraer vector→search_manual. |
| `count` | `sdk/api.rs:1412` | ❌ No trackeado | Aproximable con `collection_stats`/`memory_list`, prioridad baja. |
| `namespace_stats` | `sdk/api.rs:1474` | ❌ No trackeado | `collection_stats` lo aproxima parcialmente. |
| `vacuum` | `sdk/api.rs:81` | ❌ No trackeado | Único mantenimiento faltante del grupo MCP-16/23. Candidato natural a extender ese grupo. |
| `remove_edge` | `sdk/api.rs:1218` | ❌ No trackeado | **Los edges son inborrables vía MCP**: `RELATE` crea, pero no hay cláusula IQL ni tool para quitar un edge (solo `DELETE NODE`). Corrige con `graph_remove_edge` o cláusula IQL. |
| `search_with_method` / `search_multi` / `search_all` | `sdk/search/mod.rs:84`, `multi.rs:20,76` | ✅ MCP-24 pendiente | Correctamente trackeado. |
| `explain_memory_search` | `sdk/search/explain.rs:12` | Parcial | `explain:true` cubre per-hit; el reporte top-level `{route,fusion_report}` no tiene tool. Menor. |
| `export_namespace/all`, `import_file` (file-based) | `serialization/impl_export.rs` | Por diseño | CLI/SDK file paths documentados como escape hatch del cap stdio. |
| `pipeline` / optimizer config | `api.rs:90-106` | Por diseño | Operación admin fuera del alcance agentic; razonable. |
| `debug_*` | `sdk/search/debug_ops.rs` | Por diseño | Test-only, correcto no exponer. |
| `create_snapshot` / restore | `builder.rs:243` | Parcial (MCP-26) | Solo `list_snapshots` expuesto; creación/restore físico queda en CLI — aceptable, documentado. |

**Conclusión de cobertura:** P25 quedó bien ejecutado; los gaps restantes reales y NO trackeados son: **versiones, supersede, similar_to_key, vacuum, remove_edge** (propongo agruparlos como MCP-30..34 en backlog, ver §7).

---

## 6. Flujo de un agente real & DX

Handshake `initialize` → `notifications/initialized` (**falla con -32700 espuria**, H1) → `tools/list` (59 schemas completos con descripciones accionables) → uso normal excelente: errores de params son `-32602` con mensajes que distinguen ausencia de tipo incorrecto (AUD-050), errores de dominio vuelven como `isError` legible por el LLM, y las descripciones de tools ya documentan gotchas (u128 como strings, semántica IQL MCP-27, cap de export).

DX destacable: SKILL.md con quick start por editor, assets pre-configurados, script de smoke test Python, tabla de sintaxis IQL con ejemplos verificados, y notas de comportamiento adversarial (F7: trailing garbage aceptado; F10: rehydrate inalcanzable sin consolidación previa). Esta es la mejor doc de integración del repo.

DX mejorable: H2 (serialización bloquea fan-out de agentes paralelos) y la ausencia de un método `ping` (responde method_not_found; algunos clientes lo usan como healthcheck).

---

## 7. Propuestas (priorizadas)

1. **P1 — H1:** aceptar notifications (`#[serde(default)]` en `id` + no responder si vino sin id) + test en mcp_tests.rs. ~10 líneas.
2. **P1 — H3:** escribir la respuesta antes del `break` del shutdown. 3 líneas movidas.
3. **P2 — H4:** clamp de `k` en `search_semantic` contra `config.max_top_k`.
4. **P2 — H2:** decidir y documentar: o el loop pasa a `tokio::spawn(dispatch_request)` con write-back ordenado (canal mpsc por id), o se declara explícitamente "procesamiento serial" en config/SKILL y se simplifica el semaphore. Hoy el código sugiere una concurrencia que no existe.
5. **P3 — Backlog:** abrir filas MCP-30..34: `memory_versions`, `supersede`, `similar_to_key`, `vacuum`, `graph_remove_edge` (o cláusula IQL `UNRELATE`).
6. **P3 — H6/H7/H8:** documentar `total_bytes` como estimación, paginar `namespace://`, añadir nota de threat model LLM06 en SKILL.md § Security.

---

## 8. Score

| Eje | Puntaje | Justificación |
|---|---|---|
| Correctness | 8.5/10 | 60/60 tests; hallazgos H1-H3 son de protocolo/shutdown, no de dominio |
| Seguridad | 8/10 | Validación de boundary ejemplar; H8 es riesgo aceptado sin documentar |
| Arquitectura | 8.5/10 | Thin-wrapper disciplinado, cero lógica duplicada; H2 desajuste concurrencia-documentación |
| Performance | 8/10 | Streaming OOM-fix vigente; H4/H5 menores |
| Docs/Contrato | 9.5/10 | Hash SAME verificado, 59 tools contadas contra source, behavior notes testeables |
| **Global** | **8.3/10** | |

**Dictamen:** ✅ **aprobar con seguimiento** — H1/H3 son fixes pequeños de alto valor que deberían entrar antes de cerrar la campaña P25; nada bloquea el estado actual del módulo.
