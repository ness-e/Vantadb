---
title: "MGR-22 — Repo-map + indexación incremental: spec (chunker, watcher, API code_index/code_watch) + slice v0"
kind: research
description: "Spec de MGR-22 con la industria validada (budgets Claude Code, ranking Aider, hooks OpenHands) y el slice v0 entregado por MEMG-09: chunker por símbolo Rust + tool code_index (file-per-node + defines, idempotente por content-hash); watcher/tree-sitter/scene → FIND-299"
---

# MGR-22 — Repo-map + indexación incremental: spec (chunker, watcher, API `code_index`/`code_watch`) + slice v0

- **Fecha:** 2026-10-05 · **Tipo:** research/spec (MEMG-09, Task 53) + slice v0 implementado (`vantadb-mcp/src/code_index.rs`)
- **Contrato (plan Task 53 L1529):** "spec (chunker + watcher Rust/Py/TS + repo-map + API `code_index`/`code_watch`) + primer slice implementado según spec + research-doc; la spec incorpora los detalles de industria validados (presupuestos de carga — Claude Code, ranking dependiente de tarea — Aider, hooks de verificación — OpenHands)"
- **Origen:** `../Backlog.md` fila MGR-22 (+ FIND-196/120); Notion track PI (hub + PI-1, no accesible a workers — el contrato del plan F0 manda)
- **Decisión owner (2026-09-14):** watcher para **Rust + Python + TypeScript desde el inicio**
- **Estado:** spec ✅ + slice v0 ✅ (Rust) · **Residual v1:** `FIND-299` · **Consume:** MGR-01, MGR-17 · **Destraba:** MGR-23/24 (doc hermano `mgr-23-24-memoria-proyecto.md`), FIND-120 (wizard), ICP-01 (repo-map)

## §0. Resumen ejecutivo

El sustrato existe y está probado: 8 tools `code_*` read-only sobre el graphrag propio (D28, sin codegraph externo), traversal BFS parametrizable, búsqueda seed→expand→retrieve→context ([`GraphRagPipeline::search`](../../../src/graphrag/pipeline.rs)) y hooks para 4 clientes AI-IDE. Lo que **no existía** era el productor del grafo de código: `code_files` es un stub ("no file-per-node concept"), `code_index`/`code_watch` no existían (0 hits) y el único watcher del repo es el hot-reload de `config.rs:1719`.

Este documento fija la **spec de MGR-22** (chunker, watcher, repo-map, API, ranking, presupuestos, seguridad) y documenta el **slice v0 implementado en MEMG-09**: chunker por símbolo para Rust (scanner sin deps) + tool `code_index` que escribe records **file-per-node** + edges `defines` en un namespace, idempotente por content-hash con reconcile de símbolos stale. El residual v1 (tree-sitter multi-idioma, watcher/`code_watch`, scene `repo-map`, ranking) queda en `FIND-299` con el roadmap por slices de §3.7.

## §1. Gap verificado (código real — HEAD 2026-10-05)

| # | Afirmación | Evidencia |
|---|-----------|-----------|
| 1 | 8 tools `code_*` existen, todos read-only y thin | `vantadb-mcp/src/code.rs:39-177` (defs), `:184-333` (dispatch), doc "query-only tools" `:1-20`; listed-vs-absorbed WIRE-02 `vantadb-mcp/src/handlers/tools.rs:1232` |
| 2 | `code_files` es un stub honesto ("no file-per-node concept (D28)") | `vantadb-mcp/src/code.rs:326-329` |
| 3 | `code_index`/`code_watch` NO existían | `rg "code_index\|code_watch" src vantadb-mcp skills` → 0 hits (2026-10-05) |
| 4 | Único watcher del repo = hot-reload de config (feature `hot-reload`, crate `notify`) | `src/config.rs:1713-1734` (`watch_config`) |
| 5 | Wiki ingesta solo `.md` (el código va por MGR-22, no por MGR-25) | `src/wiki/sources.rs` (filtro `.md`); Backlog MGR-25 ("el código rs/py/ts NO entra aquí") |
| 6 | GraphRAG siembra sobre **memory records** del namespace y expande por edges | `src/graphrag/seed.rs:5-32` (`MemorySearchRequest`), `src/graphrag/pipeline.rs:67-123` |
| 7 | Graph es nodos + edges bidireccionales; ids de record determinísticos por `(namespace,key)`; rewrite preserva edges | `src/sdk/api/graph.rs:101-145` (`add_edge`), `src/sdk/api/memory.rs:438` (`memory_node_id`), `:667-672` (`carry_graph_state`, MEMG-03) |
| 8 | Hooks por cliente ya existen (claude/codex/cursor/opencode + tests) con política de budget | `skills/vantadb-mcp/assets/hooks/` (README, VERSION, tests) + `TOKEN-BUDGET.md` |
| 9 | Budgets MCP del server: respuesta one-shot ≤40 KB default (configurable), cap cliente ~25k tokens (Claude Code) documentado en config | `vantadb-mcp/src/config.rs:104-114` (`byte_budget`); `TOKEN-BUDGET.md` |

## §2. Industria validada (fuentes en §6)

| Práctica | Qué dice la fuente | Qué adopta MGR-22 |
|----------|--------------------|-------------------|
| **Load budgets (Claude Code)** | Respuestas MCP: warning >10k tokens; **límite default 25.000 tokens** por respuesta (`MAX_MCP_OUTPUT_TOKENS`); al exceder, el resultado se **persiste a archivo** y se reemplaza por una referencia (>50k chars, umbral independiente). El "~100KB" del plan es una estimación (≈4 chars/token), **no documentada**. | Presupuesto de respuesta `code_index`/`code_search` por clase ≤25k tokens (los caps v0 — `byte_budget` 40 KB, `max_files`, `MAX_SYMBOL_PAYLOAD` — atacan la misma cota); nunca devolver payloads gigantes inline. |
| **Tool-search 10% (Claude Code)** | El "10% del context window" oficial es el umbral `auto` de **tool search** (definiciones de tools <10% → upfront; ≥10% → diferidas). **No** está documentado como budget de recall. | El 10% del `TOKEN-BUDGET.md` project-local queda como **política propia** (recall/injection), no como estándar citado. La palanca de tools es la superficie listada por perfil (WIRE-02/MCP-37). |
| **Ranking dependiente de tarea (Aider repo-map)** | tree-sitter extrae **definiciones y referencias**; ranking sobre grafo de dependencias con **PageRank** (`nx.pagerank`); presupuesto `--map-tokens` default **1k**, ajuste dinámico; caché de tags SQLite (`.aider.tags.cache.v*`). | v1: ranking PageRank sobre el grafo `imports`/`calls` para el contexto inyectable; budget clase 1k tokens para el repo-map inyectado; caché = manifiesto por content-hash (ya en v0: el file record guarda `hash`). |
| **Hooks de verificación (OpenHands)** | `.openhands/microagents/repo.md` ("always loaded as part of the context") con el hábito de verificación ("make sure the tests are passing before committing"); estándar sucesor: `AGENTS.md` + CI. | La scene `repo-map` (v1) replica el patrón `repo.md`: estructura/stack/comandos/owners siempre-inyectable; el re-index se dispara desde git hooks (mismo canal que ya usan los 4 clientes). |
| **CodeRAG** | Recuperación repo-level con AST por definiciones + resumen NL por chunk + RRF + expansión por grafo + token budget. | v1: resumen NL por símbolo (docstring + firma) y RRF híbrido (ya existe en core); v0: chunk = texto fuente del símbolo. |
| **PROJECTMEM** | Log append-only + `PROJECT_MAP.md` proyectado. | La scene `repo-map` es una proyección regenerable del índice (no fuente de verdad): el grafo/records son el log. |

**No verificados (declarado):** `repo-rag` (sin paper canónico) y `cADR` (sin resultado; candidato cercano no equivalente: "Context Matters…" arXiv 2604.03826) — no se usan como fuente.

## §3. Spec MGR-22

### 3.1 Modelo de datos (file-per-node)

- **Namespaces dedicados por repo** (p. ej. `code`), keys determinísticas:
  - `file:{rel_path}` — payload: ruta + firmas (≤50); metadata: `kind=file`, `path`, `language`, `hash` (FNV-1a content hash), `symbol_count`, `symbols` (ListString con las keys de sus símbolos — manifiesto del reconcile).
  - `sym:{rel_path}#{kind}:{name}` — payload: texto fuente del símbolo (cap 4000 chars); metadata: `kind=symbol`, `symbol_kind`, `path`, `line_start`, `line_end`, `file_key`.
- **Edges:** `defines` (file → symbol, bidireccional vía `add_edge`). v1: `imports` (file→file, resolución de `use`/`mod`/`import`/`from`), `calls` (símbolo→símbolo, best-effort), `tested_by` (test→código) — alimentan traversal (`code_impact`) y ranking.
- **Ids:** los records de memoria tienen node id determinístico (`memory_node_id`) y el rewrite preserva edges (`carry_graph_state`) — base de la idempotencia.
- **Reconcile:** al re-indexar un archivo cambiado, sus símbolos viejos (manifiesto) que ya no existen: `remove_edge` + `delete` (sin ghosts de búsqueda, sin edges duplicados).

### 3.2 Chunker

| | v0 (entregado) | v1 (target) |
|---|---|---|
| Mecanismo | Scanner declarativo por líneas: code-mask (comentarios/strings/chars/raw strings) + profundidad de llaves; items top-level + `fn` en `impl`/`trait`; sin deps | **tree-sitter** (gramáticas Rust + Python + TypeScript), chunk por definición |
| Nombres | `fn`/`method` (`Type::method`)/`struct`/`enum`/`union`/`mod`/`const`/`static`/`type` (los bloques `impl`/`trait` aportan sus métodos; la declaración `trait` no emite símbolo contenedor); duplicados `~2`,`~3` | igual + resumen NL por chunk (CodeRAG) y doc-comments adjuntos |
| Límites | header en una línea; sin `mod` inline; Rust-only | multi-línea, multi-idioma, imports/calls |

El contrato del slice (plan L1530-1531) fija "chunker por símbolo + `code_index` 1 lenguaje"; el upgrade a tree-sitter queda agendado (FIND-299) con sus deps a triage.

### 3.3 Indexación incremental y watcher

- **Incremental v0 (entregado):** content-hash (FNV-1a) por archivo en el manifiesto; archivo sin cambios → **skip total** (sin re-put, sin version bump). Re-index de N archivos cuesta O(cambiados + walk).
- **Decisión Merkle vs content-hash (pre-mortem #2):** v0 = content-hash por archivo. Justificación: repo local de un solo writer; el skip ya da O(cambiados); Merkle aporta cuando hay ramas simultáneas/multi-writer o P2P — se reevalúa con `multi-escritor` (MEMG-05). Registrado como decisión §4.
- **Watcher (`code_watch`, v1):** sin daemon dentro del server MCP (R-2: no bloquear el event loop; el server es stdio). Modelo por eventos: **git hooks** (`post-commit`, `post-merge`, `post-checkout`) invocan `code_index` (idempotente → barato) + fs watcher opcional del host con debounce. Contrato propuesto:
  - `code_watch` (tool) = estado/sincronización: `{namespace, path}` → reporta frescura del manifiesto (cuántos archivos cambiaron por hash) y, con `sync: true`, dispara la pasada incremental (equivalente a `code_index` acotado a los cambiados).
  - Los hooks de cliente existentes (`skills/vantadb-mcp/assets/hooks/`) son el canal de integración preferente (patrón OpenHands/git hooks).

### 3.4 API (contrato)

**`code_index` (entregado)** — perfil `full` (writer; no entra al default `agent`):

| Arg | Requerido | Semántica | Default/límite |
|-----|-----------|-----------|----------------|
| `namespace` | sí | Namespace destino (dedicado; keys propias `file:`/`sym:`) | ≤256 bytes |
| `path` | sí | Root del repo a escanear | debe existir y ser directorio |
| `max_files` | no | Cap de archivos por pasada | 200; clamp 1..2000 |

Reporte: `{namespace, root, indexed_files, skipped_unchanged, symbols_indexed, stale_symbols_removed, files_skipped, truncated, symbols_truncated}`. Errores de dominio (`path` inexistente/no-dir) = `error_content` accionable; args inválidos = error JSON-RPC. Anotaciones MCP: `readOnly=false, destructive=false, idempotent=true, openWorld=true`. Caps: `.rs` only, skip dirs denylist, archivo ≤512 KB, snippet ≤4000 chars, ≤300 símbolos/archivo.

**`code_watch` (spec, v1)** — args `{namespace, path, sync?: bool}`; sin `sync` reporta frescura (`changed`, `unchanged`, `last_indexed`); con `sync=true` ejecuta la pasada incremental. Es un shim honesto: el daemon real son hooks del host. **No se expone hasta implementarlo** (superficie honesta; precedente: no repetir el patrón stub de `code_files`).

**`code_files` (v1):** listar archivos indexados del namespace (hoy absorbed → `code_search`); con file-per-node puede implementarse barato (filtro `kind=file`).

### 3.5 Ranking y presupuestos de carga

- **Recuperación:** BM25 + vector con fusión RRF (core existente) sobre los records del namespace; expansión por grafo (`defines`/`imports`/`calls`).
- **Ranking v1:** PageRank sobre el grafo de dependencias para seleccionar archivos representativos (patrón Aider) y boost por proximidad de hops (ya existe `hop_boost` en `retrieve`).
- **Budgets:** respuesta de tool ≤25k tokens clase Claude Code (caps v0 lo acotan); **contexto inyectable** (scene/context_text) por clase Aider ≈1k tokens con ajuste dinámico; el 10% del `TOKEN-BUDGET.md` es política project-local de recall (no estándar). Truncación declarada, nunca silenciosa.

### 3.6 Seguridad

- **Trust boundary:** `path` provisto por el cliente = input no confiable → canonicalización + walk contenido bajo el root + symlinks skipped + denylist de dirs + caps (v0, entregado).
- **Secretos:** v0 solo `.rs` (superficie mínima); el scan denylist/secretos del producto completo es de **FIND-120** (wizard), no se duplica acá.
- **Escritura:** confinada al namespace target con prefijos `file:`/`sym:`; nunca toca records ajenos.

### 3.7 Roadmap por slices (residual → FIND-299)

| Slice | Contenido | Aceptación |
|-------|-----------|------------|
| **v0 ✅ (MEMG-09)** | chunker Rust + `code_index` file-per-node + `defines` + idempotencia/reconcile | 13 tests integración + 11 unit; audit scoped 170/170 + `mcp_tests` unfiltered 115/115 |
| v1 | tree-sitter Rust/Py/TS + resumen NL + edges `imports`/`calls` | multi-idioma con tests por gramática |
| v2 | `code_watch` + git hooks (post-commit/merge/checkout) + fs debounce | hook re-indexa solo cambiados; E2E con hook de los 4 clientes |
| v3 | scene `repo-map` (estructura/stack/comandos/owners) + ranking PageRank + budget 1k | recall@k y tiempo-a-primer-recall-útil medidos (FIND-120) |
| v4 | `code_files` (listado) + foods para `code_search` (contenido no vacío — ver FIND-300) | paridad TDAM |

## §4. Decisiones registradas (candidatas ADR)

| # | Decisión | Alternativas descartadas | Razón |
|---|----------|--------------------------|-------|
| 1 | Incremental por **content-hash por archivo** | Merkle tree / full scan | single-writer local; O(cambiados); Merkle se reevalúa con MEMG-05 |
| 2 | Chunker v0 **line-scanner sin deps** | tree-sitter (+2 deps C), `syn` (dep + Rust-only) | slice mínimo; tree-sitter es el target v1 (multi-idioma); riesgo de build |
| 3 | Lógica en **`vantadb-mcp`** (own pipeline D28) | módulo core / script de skills | los `code_*` viven en el MCP; promoción a core solo con 2º consumidor |
| 4 | Watcher por **hooks/eventos**, no daemon in-server | watcher thread en el server MCP | R-2 (no bloquear stdio); hooks ya existen (4 clientes) |

## §5. Límites declarados / deuda

- Rust-only y scanner heurístico (v0) → tree-sitter v1 (FIND-299).
- `code_watch`/scene/ranking no entregados → FIND-299 (roadmap §3.7).
- `code_search` devuelve `content` vacío para memory records (graphrag `extract_content` no lee `__vanta_payload`; pre-existente, afecta todos los namespaces) → **FIND-300** (decidir fix 1-línea vs documentar).
- FIND-196 (repo-map/watcher sin research/spec) queda **superado**: spec entregada por este doc + slice v0; residual re-scopeado a FIND-299.
- FIND-120 (wizard de auto-index) consume esta spec; secretos/denylist de producto viven ahí.

## §6. Fuentes

**Industria (verificadas 2026-10-05):**
1. Claude Code MCP (límites 10k/25k tokens, 50k chars persist-to-file, tool-search 10%) — https://docs.claude.com/en/docs/claude-code/mcp
2. Anthropic — Effective context engineering for AI agents ("attention budget", tool-result clearing) — https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents
3. Aider repo-map (grafo, budget 1k, dinámico) — https://aider.chat/docs/repomap.html · https://aider.chat/2023/10/22/repomap.html
4. Aider `repomap.py` (`nx.pagerank`, caché `.aider.tags.cache.v*`) — https://github.com/Aider-AI/aider/blob/main/aider/repomap.py
5. OpenHands `repo.md` (microagente always-loaded, hábito de verificación; Wayback 2025-05-30) — https://web.archive.org/web/20250530181152/https://docs.all-hands.dev/modules/usage/prompting/microagents-repo
6. OpenHands skills/repo (sucesor: `AGENTS.md` + CI) — https://docs.openhands.dev/overview/skills/repo.md
7. CodeRAG (repo-level code retrieval) — https://arxiv.org/abs/2509.16112 · https://arxiv.org/abs/2406.14497
8. RepoFuse — https://arxiv.org/abs/2402.14323
9. PROJECTMEM — https://arxiv.org/abs/2606.12329

**Internas:** `../Backlog.md` (MGR-22, FIND-120/196), `../plans/2026-10-04-master-plan-0.9.0.md` Task 53, `../tasks/MEMG-09.md`, `vantadb-mcp/src/code.rs`, `vantadb-mcp/src/code_index.rs`, `src/graphrag/seed.rs`, `src/sdk/api/memory.rs`, `skills/vantadb-mcp/assets/hooks/TOKEN-BUDGET.md`.
