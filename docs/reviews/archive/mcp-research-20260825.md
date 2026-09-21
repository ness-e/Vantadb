---
title: "Investigación profunda: vantadb-mcp vs usuarios objetivo y estado del arte MCP"
type: review
status: active
date: 2026-08-25
scope: vantadb-mcp (excluye core `src/` salvo contratos VantaEmbedded)
mode: read-only (hallazgos → Backlog, sin fixes)
---

# MCP Deep Research — `vantadb-mcp` vs competencia y usuarios reales

**SKILLS_CARGADAS:** `coordinated-web-search` (v2 router en cascada), `source-driven-development`
**GATES_EVALUADOS:** P/D gates evaluados sobre superficie pública del MCP (tools nuevas detectadas en findings); no se implementó nada — solo research.

---

## 1. Resumen ejecutivo

`vantadb-mcp` tiene la **superficie funcional más amplia de su categoría** (~75 tools contadas vs 9–19 de la competencia) y el único modelo **embedded Rust sin Docker, sin LLM key, sin DB externa**. Pero pierde puntos donde la competencia ya está: **protocolo 2024-11-05 hardcodeado** (spec estable actual: 2025-06-18; latest: 2026-07-28), **cero tool annotations**, y una superficie de tools que **excede el límite de ~40 tools de Cursor** — uno de los clientes objetivo declarados.

| Dimensión | Score | Evidencia clave |
|---|---|---|
| DX de onboarding | 7/10 | One-liner `vanta-cli server --mcp`, cero deps externas; docs solo cubren 3 clientes |
| Paridad de protocolo | **4/10** | `initialize.rs:11` → `"protocolVersion": "2024-11-05"` hardcodeado; sin annotations; sin structured tool output |
| Completitud funcional | 9/10 | ~75 tools (45 core + threads/code/wiki/scenes/skills/context) — ver §4.1 |
| Performance / overhead | 6/10 | Límites configurados (`config.rs:51-54`) pero **0 benchmarks publicados** — todo claim sin evidencia |
| Robustez | 8/10 | 72 tests async en `mcp_tests.rs` (4270 líneas); drain en shutdown; stdio concurrente MOD-08/09 |
| Seguridad | 6/10 | Validación en trust boundaries presente, pero `validate_identifier` NO bloquea separadores de path (lección 2026-08-25) |
| Docs & skills | 6/10 | Link roto MCP.md→`docs/skills/vantadb-mcp/SKILL.md` (no existe); hashes SKILL.md distintos entre sí; conteo de tools desactualizado |
| Observabilidad | 5/10 | tracing a stderr OK; sin timing por-request ni guía de logs por cliente |
| Testabilidad | 8/10 | Flujos reales cubiertos; gaps en scene tools (`SceneToolCall` sin covering tests) |
| **Diferenciación** | 8/10 | Único embedded local-first con híbrido BM25+HNSW+RRF + grafo analítico + IQL + wiki + skills |

**Veredicto:** ventaja competitiva real en arquitectura y completitud; deuda crítica en protocolo y disciplina de superficie.

---

## 2. Usuarios objetivo y fricciones reales del transporte stdio

Investigado con evidencia de issues reales (sub-agente 3, todas con URL):

### Caps de output que afectan directamente nuestras respuestas
| Cliente | Límite | Fuente |
|---|---|---|
| Claude Code | warning >10k tokens, **hard cap 25k tokens** (`MAX_MCP_OUTPUT_TOKENS`) | code.claude.com/docs/en/mcp |
| OpenCode | **2000 líneas / 50KB truncados** antes del LLM, no configurable | anomalyco/opencode#22565 |
| Codex CLI | truncamiento line-based, empeorado en v0.56 | openai/codex#6426 |
| Cursor | umbral indocumentado | forum.cursor.com/t/149292 |

### Límites de número de tools — **hallazgo crítico**
- **Cursor: cap ~40 tools totales** ("Exceeding total tools limit") — forum.cursor.com/t/108637. VantaDB expone ~75 → **en Cursor las tools sobrantes ni siquiera se registran**.
- Windsurf: cap 100 tools documentado.
- Claude Code mitiga con "tool search"; los demás no.

### Timeouts y ciclo de vida
- Claude Code: timeout de conexión default 30s (`MCP_TIMEOUT`); **stdio NUNCA auto-reconecta** tras crash (#86227, #86349). Windows .exe nativo con timeout inexplicable #82791.
- Codex Windows: initialize no llega por stdin → todos los servers time out (community.openai.com/t/1363658).
- OpenCode: timeout option ignorada / deshabilita server a los 15s (#8121).
- Universal: `npx` requiere wrapper `cmd /c` en Windows (gist aruruka/8d52dcd…).

**Implicancia directa:** el binario nativo `vanta-cli` es una **ventaja DX real** frente a servers npx/Python (el dolor #1 multi-cliente es el wrapper npx en Windows) — hoy enterrado en docs, debe ser headline.

---

## 3. Protocolo y estándares (modelcontextprotocol.io)

Fuente: spec oficial + blog MCP (sub-agente 1, URLs citadas inline).

| Versión | Estado | Relevancia para nosotros |
|---|---|---|
| 2025-06-18 | Estable, ampliamente desplegada | Structured tool output, elicitation, annotations, header `MCP-Protocol-Version` |
| 2025-11-25 | Estable | Security best practices doc |
| **2026-07-28** | Latest | Stateless core, handshake retirado (solo golpea HTTP remoto), **Roots/Sampling/Logging DEPRECATED** |

Hallazgos:
1. **stdio sigue siendo el transporte recomendado** ("Clients SHOULD support stdio whenever possible") — nuestra apuesta embedded-first queda validada por la spec.
2. Las breaking changes de 2026-07-28 (stateless core, MRTR, headers de routing) afectan a servidores Streamable HTTP, **no a nosotros** mientras sigamos stdio-only.
3. **Tool annotations** (`readOnlyHint`, `destructiveHint`, `idempotentHint`, `openWorldHint`, 2025-06-18): los directorios oficiales (ChatGPT plugins, Claude Connectors) empiezan a chequearlas; "most servers don't use them" aún — ventana para destacar. Nosotros: **0 annotations** (grep `annotations` en `vantadb-mcp/src` = vacío).
4. Seguridad oficial (spec 2025-11-25): consentimiento, token passthrough prohibido, SSRF, session binding. Para un server stdio local aplica: stdout limpio de output no-MCP, logs solo a stderr (✅ cumplimos: `init_telemetry_fmt with_writer(stderr)`), sandboxing sugerido. Prompt injection/"tool poisoning" es terminología comunitaria (Invariant Labs), no sección oficial.
5. No implementar Roots/Sampling/Logging: **deprecados en 2026-07-28** — decisión correcta de no tenerlos.

---

## 4. Estado interno de `vantadb-mcp` (evidencia file:line)

### 4.1 Inventario real de tools (contado, no asumido)

El brief decía "72: 42 core + 30". Conteo real por grep de `"name":` en definiciones:

| Módulo | Tools | Count |
|---|---|---|
| `handlers/tools.rs` (core) | memory_put, memory_put_batch, memory_get, memory_delete, memory_delete_by_filter, memory_list, memory_list_namespaces, memory_versions, memory_supersede, query_iql, search_semantic, search_memory, search_with_method, search_multi, get_node_neighbors, graph_page_rank, graph_degree_centrality, graph_traverse, graph_topological_sort, graph_is_dag, remove_edge, inject_context, read_axioms, write_axiom, delete_axiom, collection_stats/list/delete, rehydrate, purge_expired, compact_wal, flush, compact_layout, vacuum, rebuild_index, audit_text_index, repair_text_index, capabilities, generate_snippet, list_snapshots, snapshot_create, export, import, bulk_import_file, bulk_import_stream | **45** |
| `threads.rs` | thread_create/send/get/list/delete/purge_expired | 6 |
| `code.rs` | code_search/explore/callers/callees/impact/node/status/files | 8 |
| `wiki.rs` | wiki_search/read/list/graph/ingest/ingest_status | 6 |
| `scenes.rs` | scene_read/list/query | 3 |
| `skills.rs` | skill_list/view/create/update/patch/files_write | 6 |
| `context.rs` | context_assemble | 1 |
| **Total** | | **~75** |

También expone resources (`resources.rs`: Operational Metrics, Database Schema) y prompts (`prompts.rs`: search_memory, analyze_namespace, summarize_context, query_builder).

### 4.2 Flujos y robustez
- stdio concurrente (MOD-08/09), drain en shutdown vía OpGate (`vantadb-node/src/lib.rs:76-80` patrón; equivalente MCP verificado en lessons).
- Validación trust-boundary: parse_metadata delega arrays/null al core (ERR-026); filtros aceptan formato plano Y operadores `$eq/$neq/$gt/...` normalizados (AUD-048, 2026-08-18).
- Axiomas: Iron Axioms hardcoded + records en `_axioms` (axioms.rs:15-29, MCP-33); cap MAX_AXIOM_RECORDS=10000.
- Limitaciones documentadas honestamente (lessons): search_semantic k clampeado a max_top_k; timeout de spawn_blocking NO cancela trabajo; total_bytes estimación deliberada.
- Config: `default_list_limit:100, max_list_limit:10_000, max_top_k:1000` (config.rs:51-54).

### 4.3 Persistencia
`VantaEmbedded` fjall (persistente) vs in-memory — ambos backends accesibles vía config; snapshots físicos Fjall listables pero **sin restore vía MCP** (MCP-34 pendiente; `snapshot_create` YA existe en tools.rs:466 — la fila MCP-34 está parcialmente stale).

### 4.4 Tests
`mcp_tests.rs`: **72 tests async**, 4270 líneas. Gaps: scene tools (`SceneToolCall/execute_scene_tool` en vanta-memory) sin covering tests detectados por codegraph.

### 4.5 Historial Backlog/tareas
- P25 cerró MCP-16..29 (core ~100% expuesto). P26 trackea capa cognitiva (MCP-30..34).
- **MCP-34** (snapshots create/restore): pendiente, pero `snapshot_create` ya existe → fila necesita split: lo que falta es `snapshot_restore` + validación anti path-traversal (lección: `validate_identifier` no bloquea `/ \ . ..`; `FsSnapshot` no deriva Serialize).
- **MCP-35**: fallback HTTP proxy para N instancias sobre la misma BD (incidente 2026-08-25: segunda sesión OpenCode sin tools por lock exclusivo) — pendiente, prioridad Alta.
- Deuda docs: `docs/api/MCP.md:12` enlaza `skills/vantadb-mcp/SKILL.md` relativo a docs/ → **el archivo no existe**; los dos SKILL.md reales (`.opencode/skills/vantadb/` y `.opencode/skills/vantadb-mcp/`) tienen **hashes distintos** (155E93… vs DF1A68…) — drift entre skill canónica y variante MCP.

---

## 5. Matriz competencia (features × productos)

| Feature | **vantadb-mcp** | mem0 MCP | basic-memory | official memory | Graphiti/Zep | Cognee | Letta |
|---|---|---|---|---|---|---|---|
| Backend | **Fjall embedded (Rust)** | Vector DB hosted/OSS | Markdown + SQLite | JSONL file | Neo4j/FalkorDB (Docker) | SQLite+LanceDB+Kuzu | git-backed MemFS |
| Deps externas | **Ninguna** | LLM+embedder obligatorios | ninguna (uv) | ninguna | Docker + LLM key + graph DB | LLM key | ninguna core |
| # Tools MCP | **~75** | 9 (OSS, repo archived) | ~19 | 9 | 13 | 3 pinned (+internas) | client-side |
| Híbrido BM25+vector+RRF | ✅ | ✅ (Abr 2026) | opcional (flags) | ❌ keyword only | ✅ | ✅ | ❌ |
| Grafo analítico (PageRank/traverse/DAG) | ✅ único | ❌ | wikilinks | ❌ | ✅ temporal | ✅ | ❌ |
| IQL / query language | ✅ único | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| TTL / versioning / supersession | ✅ | ❌ | ❌ | ❌ | bi-temporal facts | session cache | blocks |
| Wiki ingestion / skills store / threads | ✅ único | ❌ | ❌ | ❌ | ❌ | ❌ | skills (host) |
| Auto-extracción LLM de conversaciones | ❌ | ✅ core | ❌ | ❌ | ✅ core | ✅ core | ✅ core |
| Hosted/cloud option | ❌ | ✅ ($19–249/mo) | $15/mo sync | ❌ | Zep managed | Cloud | cloud |
| Annotations de tools | ❌ | ? | ✅ | ❌ | ? | ? | ? |
| Stars / actividad | privado | 62.6K ⭐ Apache-2.0 | activa, AGPL-3.0 | MIT ref-only | 30.3K ⭐ Apache-2.0 | 30.3K ⭐ Apache-2.0 | restructuring |
| Perf publicado | ❌ claim sin evidencia | LoCoMo 92.5 (plataforma, reproducible via memory-benchmarks) | claim sin evidencia | — | "<200ms" vendor claim, paper arXiv:2501.13956 | BEAM 0.79@100K self-reported, arXiv:2505.24478 | claim sin evidencia |

Lectura estratégica:
- **mem0 pivoteó a hosted** (mem0-mcp OSS archived) → dejó hueco de "memoria local seria sin LLM key" que **somos los únicos en llenar**.
- La competencia ganadora compite con **menos tools mejor descritas** (Cognee: 3 pinned; basic-memory: ~19 con schema_* helpers), no con más tools.
- Nadie tiene PageRank/IQL/wiki/skills — diferenciación real pero invisible si el cliente nunca ve las tools (cap de Cursor).

---

## 6. Gap analysis priorizado

### 🔴 P0 — Falta (bloquea adopción)
| ID | Gap | Evidencia |
|---|---|---|
| **P0-A** | `protocolVersion` hardcodeado `"2024-11-05"` — sin negociación, sin structured tool output, sin elicitation-capable | `vantadb-mcp/src/handlers/initialize.rs:11` |
| **P0-B** | 75 tools > cap ~40 de Cursor; presión de contexto en todos los clientes; sin perfiles/subconjuntos | forum.cursor.com/t/108637; inventario §4.1 |
| **P0-C** | Sin tool annotations — requisito emergente de directorios (ChatGPT/Claude Connectors) y señal de calidad | grep vacío; blog.modelcontextprotocol.io/posts/2026-03-16-tool-annotations |

### 🟡 P1 — Falta (compite mal sin esto)
| ID | Gap | Evidencia |
|---|---|---|
| P1-D | `snapshot_restore` ausente (backup físico incompleto) + `validate_identifier` sin anti path-traversal | Backlog MCP-34; lección 2026-08-25 snapshot tools |
| P1-E | Respuestas sin byte/token budget → riesgo de truncamiento silencioso en Claude Code (25k tok) / OpenCode (50KB) | §2 caps; search_multi puede retornar payloads grandes |
| P1-F | Sin presencia en registry.modelcontextprotocol.io / glama / smithery → invisibles al discovery | sub-agente 2: registry movió catálogo; no estamos |
| P1-G | Sin historia de memoria conversacional auto-consolidada (lo que hace memorable a mem0/graphiti/cognee) | matriz §5 |
| P1-H | Multi-instancia misma BD muere (lock exclusivo) | MCP-35, incidente 2026-08-25 |

### 🟢 P2 — Mejorable / optimizable
- **Mejorable:** docs drift (link roto + hash mismatch + conteo 72≠75); observabilidad por-request (timing en stderr); cobertura de tests de scene tools; docs de instalación por cliente (faltan Codex TOML, Gemini settings.json, Windsurf mcp_config.json).
- **Optimizable:** respuestas de `search_multi` paginables; consolidación de tools CRUD afines (memory_versions/supersede → parámetro de memory_put? decisión de diseño, no automática); benchmark reproducible propio para reemplazar claims sin evidencia.

---

## 7. Quick wins (<1 día) vs apuestas estratégicas (>1 semana)

**Quick wins:**
1. Negociar protocolVersion (eco de la versión pedida, anunciar 2025-06-18) — horas.
2. Annotations en las 75 tools (la mayoría readOnlyHint=true trivialmente determinable) — 1 día.
3. Fix docs: link roto MCP.md, sincronizar SKILL.md ×2 (hash), actualizar conteo de tools — medio día.
4. Anti path-traversal en validate_identifier (`/ \ ..`) — líneas.
5. server.json manifest + PR al registry.modelcontextprotocol.io.

**Apuestas estratégicas (>1 semana):**
1. **Perfiles de tool surface** (env `VANTADB_MCP_PROFILE=memory|dev|full`) para caber en Cursor y reducir contexto — diseño + tests + docs.
2. **Memoria conversacional auto-consolidada** feature-gated (extract→consolidate→recall estilo mem0-lite, reusando scenes/MEM-13/14) — la brecha competitiva más grande.
3. **MCP-35 HTTP proxy fallback** multi-sesión.
4. Benchmark público reproducible (vs official-memory y mem0-OSS en tareas de recall) para convertir diferenciación en evidencia.

---

## 8. Recomendaciones → filas Backlog

Nuevas filas añadidas a `docs/Backlog.md` § P26 (formato canónico, sin esquemas nuevos):

| ID | Resumen |
|---|---|
| `MCP-36` | P0-A: negociación protocolVersion + structured tool output |
| `MCP-37` | P0-B: perfiles de tool surface para caps de clientes (Cursor 40) |
| `MCP-38` | P0-C: tool annotations en toda la superficie |
| `MCP-39` | P1-E: output budgeting (byte/token budget en respuestas grandes) |
| `MCP-40` | P1-F: registro registry.modelcontextprotocol.io + server.json + directorios |
| `MCP-41` | P1-H: absorbe/complementa MCP-35 (multi-instancia) — ver fila existente |
| `MCP-42` | P1-G: memoria conversacional auto-consolidada (feature-gated llm) |
| `FIND-24b`* | Docs drift: link roto MCP.md→docs/skills/, hash mismatch SKILL×2, conteo 72 vs ~75 real (*extensión de FIND-24, misma área) |

MCP-34 se re-scoping inline en su fila existente (snapshot_create ya hecho; falta snapshot_restore + path validation).

### Claims de performance
Todos los números de competencia citados traen fuente o están marcados "claim sin evidencia" (§5). VantaDB **no publica ninguno** → hasta tener benchmark reproducible, cualquier claim nuestro debe tacharse igual.

---

## Fuentes principales
- Spec: modelcontextprotocol.io/specification/2025-06-18 (transports, lifecycle, security_best_practices); blog.modelcontextprotocol.io/posts/2026-07-28 y /posts/2026-03-16-tool-annotations
- Competidores: github.com/mem0ai/mem0 (+mem0-mcp archived), basicmachines-co/basic-memory, getzep/graphiti, topoteretes/cognee, letta-ai/letta-code, modelcontextprotocol/servers (src/memory); papers arXiv:2501.13956, arXiv:2505.24478
- Clientes: code.claude.com/docs/en/mcp; issues anthropics/claude-code#86227,#86349,#82791; opencode#8121,#16449,#22565; openai/codex#6020,#6426; foros cursor/windsurf; google-gemini.github.io/gemini-cli docs
- Interno: codegraph_explore (vantadb-mcp handlers, axioms, scene_tools), docs/api/MCP.md, mcp_tests.rs, docs/Backlog.md P25/P26, task-system memory lessons+decisions
