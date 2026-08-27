---
name: codebase-memory
description: >-
  codebase-memory-mcp — code intelligence MCP server (tree-sitter, 158 languages,
  semantic + BM25 search, Cypher queries, architecture overview, git change
  impact, dead-code detection, ADR management). Use when the agent needs to
  understand, navigate, or refactor a codebase beyond single-file reads:
  architecture overviews, call-path tracing (who calls / what calls a function),
  change blast-radius before a commit, structural/semantic search, cross-repo
  linking, architecture decision records. Installed at
  C:\Users\Eros\AppData\Local\Programs\codebase-memory-mcp\codebase-memory-mcp.exe
  (v0.10.8); configured as the `codebase-memory-mcp` MCP server in opencode.jsonc.
  Complements CodeGraph (project's own pre-indexed graph) — prefer CodeGraph for
  VantaDB Rust core symbol resolution; use codebase-memory-mcp for architecture,
  impact, semantic, and cross-cutting questions. Includes an automated maintenance
  ritual (health check + re-index if stale/broken) that runs on skill load.
---

# codebase-memory-mcp

**Fastest code intelligence engine for AI coding agents.** Builds a persistent
knowledge graph from your codebase via tree-sitter AST analysis (158 languages)
plus Hybrid LSP semantic type resolution for 12 languages (Python, TS/JS/JSX/TSX,
PHP, C#, Go, C, C++, Java, Kotlin, Rust, Perl). Ships as a native executable —
no language runtime, no API key, 100% local. Homepage:
<https://github.com/DeusData/codebase-memory-mcp>; research preprint:
<https://arxiv.org/abs/2603.27277>.

- **Binary:** `C:\Users\Eros\AppData\Local\Programs\codebase-memory-mcp\codebase-memory-mcp.exe`
- **MCP server name:** `codebase-memory-mcp` (enabled in `opencode.jsonc`)
- **Cache/store:** `%LOCALAPPDATA%\codebase-memory-mcp` (config + indexes)
- **Auto-index:** on (set via `codebase-memory-mcp config set auto_index true`)

## Relationship with CodeGraph (read this first)

VantaDB already runs **CodeGraph** (`codegraph serve --mcp`) — a pre-indexed graph
of the VantaDB core (authoritative for Rust symbols, call paths, blast radius,
wired into CI hooks). Both are graph tools; they **complement, not replace**:

| Need | Use |
|------|-----|
| Resolve a VantaDB Rust symbol, caller/callee inside the core, blast radius for a change touching `src/` | **CodeGraph** (`codegraph_explore`) — pre-indexed, tuned, CI-integrated |
| Architecture overview (languages, packages, routes, hotspots, clusters) of this or any repo | **codebase-memory-mcp** (`get_architecture`) |
| "Who calls `X`?" / "What does `X` call?" across the whole repo (BFS) | **codebase-memory-mcp** (`trace_path`) |
| Blast radius of uncommitted git changes + risk classification before commit/push | **codebase-memory-mcp** (`detect_changes`) |
| Semantic / natural-language code search, or full-text across the graph | **codebase-memory-mcp** (`semantic_query`, `search_code`, `search_graph`) |
| Dead-code detection, cross-service HTTP/gRPC/GraphQL linkage | **codebase-memory-mcp** (`search_graph` degree scans, edges) |
| Cypher-style graph queries, ADRs | **codebase-memory-mcp** (`query_graph`, `manage_adr`) |
| Cross-repo intelligence (multiple repos in one store) | **codebase-memory-mcp** (only tool that does this) |

Rule of thumb: **CodeGraph first for VantaDB-core symbols; codebase-memory-mcp for
architecture / impact / semantic / cross-cutting / cross-repo.** Avoid forcing a
re-index churn — both share the repo, and `auto_index` keeps the CBM graph fresh.

### VantaDB specifics (operational notes)

- **Project key:** `C-Users-Eros-VantaDB-Proyect-VantaDB`. The daemon derives the
  key from the repo path; the space and the non-ASCII `ó` in `VantaDB Proyect`
  get normalized to that form. Use it as the `project` arg (e.g.
  `search_graph --project C-Users-Eros-VantaDB-Proyect-VantaDB`).
- **Index mode:** `index_repository` in `full` mode **errors on this repo** (the
  "all files" pass hits a file that crashes the semantic stage). `moderate` works
  and adds similarity/semantic edges (28.3K nodes / ~113K edges). `fast` also works
  but skips similarity/semantic edges. **Prefer `moderate`** for VantaDB. Note:
  there is no config key to change `auto_index`'s mode, so if `auto_index` ever
  re-runs from a clean cache it may attempt `full` and fail — the cached `moderate`
  graph stays usable, just re-index `moderate` if needed.
- **Artifact:** a `.codebase-memory/graph.db.zst` snapshot was written; it's
  gitignored (regenerable) — teammates can reindex from scratch.

## Carga automática

Esta skill DEBE cargarse con el tool `skill` (o leyendo este SKILL.md) **siempre que**:
- el agente vaya a usar `codegraph_explore` / `codegraph_status` (CodeGraph), o
- el agente vaya a usar cualquier tool `codebase-memory-mcp_*`, o
- se invoque `/codeGraph`.

No depende de que el usuario lo pida: es parte del ritual de inicio de cualquier trabajo de
código en VantaDB. La skill trae la rutina de mantenimiento abajo, así que cargarla garantiza
índices frescos antes de consultar.

## Mantenimiento automático (ritual al cargar la skill)

Objetivo: que ambos grafos (CodeGraph + codebase-memory-mcp) estén frescos y sanos cada vez
que se usan, sin reindex manual. El agente corre ESTE ritual **antes de cualquier consulta de
negocio**:

1. **LEER estado** — `maintenance-state.json` en este directorio
   (`{"cbm_last_index": "<ISO8601|null>", "cbm_last_health": "<ISO8601|null>",
      "cg_last_init": "<ISO8601|null>"}`). Si no existe, crearlo con nulls.
2. **HEALTH CHECK** (rápido, ~1 llamada c/u):
   - CBM: `codebase-memory-mcp_list_projects` → ¿está `C-Users-Eros-VantaDB-Proyect-VantaDB`?
          + `codebase-memory-mcp_index_status(project:...)` → ¿error / reindex corriendo?
   - CodeGraph: ¿existe `.codegraph/` en la raíz del repo? + `codegraph_status` si disponible.
3. **EVALUAR antigüedad** (umbrales por defecto):
   - CBM: si `cbm_last_index` es null O > 7 días O health check falla → **requiere mantenimiento**.
   - CodeGraph: si no existe `.codegraph/` O `cg_last_init` > 30 días → **requiere mantenimiento**.
4. **MANTENIMIENTO** (solo si es necesario):
   - CBM: `codebase-memory-mcp_index_repository(repo_path:"C:\Users\Eros\VantaDB Proyect\VantaDB",
     mode:"moderate")` — **NUNCA `full`** (falla en este repo; ver VantaDB specifics).
     Esperar a que termine (chequear con `index_status`).
   - CodeGraph: `codegraph init` **solo si** no existe `.codegraph/`.
   - Equipo: `index_repository(..., persistence:true)` para regenerar `graph.db.zst`.
5. **ACTUALIZAR estado** — escribir `maintenance-state.json` con los timestamps nuevos.
6. **FALLO de mantenimiento** (CBM error, binario no encontrado en `opencode.jsonc`,
   CodeGraph init falla): avisar al usuario con el comando exacto y **no bloquear** la tarea
   de negocio — degradar a búsqueda manual (grep/Read) y continuar.

Umbrales ajustables: subir a 14/60 días si el repo cambia poco; bajar a 3/14 si hay refactors
frecuentes. El ritual es read-only salvo el paso 4 (re-index), que es el único efecto secundario.

## MCP Tools (15)

All tools are called by the agent as `codebase-memory-mcp.<tool>`. The agent
(translator) turns natural-language requests into these calls. The `project`
argument is the project key returned by `list_projects` (auto-derived from the
repo path on first index).

### Indexing

- **`index_repository`** — Index a repository into the graph. Args: `repo_path`
  (absolute path). Auto-sync (watcher) keeps it fresh afterwards. Run this once
  per repo before querying; with `auto_index` on, it happens automatically on
  first session connect.
- **`list_projects`** — List all indexed projects with node/edge counts. Use to
  discover the `project` key to pass to other tools.
- **`delete_project`** — Remove a project and all its graph data.
- **`index_status`** — Check indexing status of a project (e.g., is a reindex
  running). Args: `project`.

### Querying

- **`search_graph`** — Structured search by label, name pattern (regex), file
  pattern, and degree filters (min/max in/out degree). Pagination via
  `limit`/`offset`. Labels: `Project, Package, Folder, File, Module, Class,
  Function, Method, Interface, Enum, Type, Route, Resource`. Example: name
  pattern `.*Handler.*` with label `Function`.
- **`trace_path`** — BFS traversal of call graph. Args: `function_name` (or
  `qualified_name`), `direction` (`inbound` = who calls it, `outbound` = what it
  calls, `both`), `depth` (1–5, default 5), `project`. Alias: `trace_call_path`.
- **`detect_changes`** — Map git diff (uncommitted/working tree) to affected
  symbols + blast radius with risk classification. Run BEFORE a commit/push to
  know what a change touches. Args: `project`.
- **`query_graph`** — Execute Cypher-like read-only graph queries. Args:
  `query` (openCypher read subset), `project`. Example:
  `MATCH (f:Function)-[:CALLS]->(g) WHERE f.name = 'main' RETURN g.name`.
- **`get_graph_schema`** — Node/edge counts, relationship patterns, property
  defintions per label. **Run this first** to learn the graph shape of a project.
- **`get_code_snippet`** — Read source code for a function by qualified name
  `<project>.<path_parts>.<name>`. Use `search_graph` to discover qualified names.
- **`get_architecture`** — Codebase overview in one call: languages, packages,
  entry points, routes, hotspots, boundaries, layers, clusters, ADRs.
- **`search_code`** — Grep-like text search within indexed project files only
  (graph-augmented). Args: `pattern`, `project`, optional `file_pattern`.
- **`manage_adr`** — CRUD for Architecture Decision Records. Modes: `get`,
  `sections`, `create`, `update`. Query modes don't wait behind a same-project
  reindex. Use to persist architectural decisions across sessions.
- **`ingest_traces`** — Ingest runtime traces to validate `HTTP_CALLS` edges.

## Cypher Subset (`query_graph`)

Read-only openCypher subset. Anything outside fails with a clear `unsupported…`
error (never silently empty).

- **Clauses:** `MATCH`, `OPTIONAL MATCH`, multiple `MATCH`, `WHERE`, `WITH`
  (+`WITH … WHERE`), `RETURN`, `ORDER BY`, `SKIP`, `LIMIT`, `DISTINCT`, `UNWIND`,
  `UNION`/`UNION ALL`, `CASE`.
- **Patterns:** labelled nodes, label alternation `(n:A|B)`, typed/directed
  relationships, variable-length paths `[*1..3]`, inline property maps.
- **WHERE:** `= <> < <= > >=`, `AND/OR/XOR/NOT`, `IN`, `CONTAINS`,
  `STARTS WITH`, `ENDS WITH`, `IS [NOT] NULL`, regex `=~`, label test `n:Label`,
  existence `WHERE NOT EXISTS { (f)<-[:CALLS]-() }` (great for dead-code).
- **Aggregates:** `count`(+`DISTINCT`), `sum`, `avg`, `min`, `max`, `collect`.
- **Functions:** `labels`, `type`, `id`, `keys`, `properties`;
  `toLower/toUpper/toString/toInteger/toFloat/toBoolean`; `size`, `length`,
  `trim/ltrim/rtrim`, `reverse`; `coalesce`, `substring`, `replace`, `left`,
  `right`.

Dead-code example: `MATCH (f:Function) WHERE NOT EXISTS { (f)<-[:CALLS]-() } AND NOT f:EntryPoint RETURN f.name`.

## CLI Mode (one-shot, no daemon)

Every MCP tool can be invoked as a local command — handy for scripts or when the
MCP session isn't connected. CLI mode never starts the coordination daemon.

```bash
codebase-memory-mcp cli index_repository --repo-path "C:\Users\Eros\VantaDB Proyect\VantaDB"
codebase-memory-mcp cli list_projects
codebase-memory-mcp cli search_graph --project vantadb --name-pattern '.*Handler.*' --label Function
codebase-memory-mcp cli trace_path --project vantadb --function-name Search --direction both
codebase-memory-mcp cli query_graph --project vantadb --query 'MATCH (f:Function) RETURN f.name LIMIT 5'
```

Flags are generated from each tool's input schema (`cli <tool> --help`). JSON
args may be piped on stdin; `--json` returns the full MCP envelope; `--progress`
forces human-readable progress on non-interactive stderr.

## Graph Visualization UI

Built into the binary. Owned by the shared coordination daemon (no duplicate
servers across sessions).

```bash
codebase-memory-mcp --ui=true --port=9749
```

Open <http://localhost:9749>. Cross-repo multi-galaxy layout supported.

## Configuration

```bash
codebase-memory-mcp config list                          # show all settings
codebase-memory-mcp config set auto_index true           # auto-index on session start (ON)
codebase-memory-mcp config set auto_index_limit 50000    # max files for auto-index (ON)
codebase-memory-mcp config set auto_watch true           # background git watcher (default ON)
codebase-memory-mcp config reset auto_index              # reset to default
```

| Env var | Default | Purpose |
|---------|---------|---------|
| `CBM_CACHE_DIR` | `%LOCALAPPDATA%\codebase-memory-mcp` | Override DB/storage dir (one canonical root per account) |
| `CBM_ALLOWED_ROOT` | unset | Confine `index_repository` to paths within this dir (untrusted callers) |
| `CBM_LOG_LEVEL` | `info` | `debug`/`info`/`warn`/`error`/`none` |
| `CBM_WORKERS` | detected | Parallel-indexing worker count (useful in containers) |
| `CBM_DIAGNOSTICS` | `false` | `1`/`true` → daemon writes `trajectory.ndjson` for leak/perf reports |

## Ignoring Files

Layered, lowest-precedence to highest: hardcoded patterns (`.git`,
`node_modules`, …) → `.gitignore` hierarchy → `.cbmignore` (project-specific,
gitignore syntax). Symlinks always skipped. Create a `.cbmignore` in the repo
root to exclude generated/large files from indexing.

## Team-Shared Graph Artifact

Commit `.codebase-memory/graph.db.zst` (zstd-compressed graph snapshot) so
teammates skip the reindex — `index_repository` imports it and fills the local
diff. Optional; add `.codebase-memory/` to `.gitignore` if you prefer everyone
reindexes from scratch. A `.gitattributes` line with `merge=ours` is auto-created.

## Recommended Agent Workflow (3 tiers)

The upstream installer defines Scout / Verify / Auditor tiers. In practice, when
you call these tools:

1. **Discover** — `list_projects` → `get_graph_schema` → `get_architecture` to
   learn the shape before drilling in.
2. **Targeted evidence** — `search_graph` / `trace_path` / `query_graph` for the
   specific question; back claims with `get_code_snippet` (real source lines).
3. **Impact before change** — `detect_changes` on the working tree to state blast
   radius + risk before editing; `check_index_coverage` mentally = ensure the
   files you cite are indexed (re-index if stale).

Never assert "nothing calls X" or "X is dead" without a `trace_path`/`query_graph`
existence check. A clean result means "no recorded gap", not "proven complete".
