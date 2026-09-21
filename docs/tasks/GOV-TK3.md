# GOV-TK3 — drift yaml↔real ×3 (Wave 0, Task 4)

> Plan: `docs/plans/2026-09-04-durability-release-readiness.md` (Task 4, Wave 0) · Backlog P38/GOV-TK · Ruta: vanta-docs
> Commit (solo si contrato pasa): `docs(api): drift yaml-real ×3 (GOV-TK3)`
> Estado: ✅ COMPLETO (2026-09-05, commit `b3be4176`)

## Contrato (ley)

Los 3 drifts resueltos uno por uno con evidencia (doc corregida o código alineado) + suite afectada verde.
**Dirección global: DOC-fix** (código verificado correcto por live-fire GOV-B5 + tests; cambiar el parser/handlers sería behavior-change fuera de appetite).

## DISCOVERY — evidencia por drift (2026-09-05, re-verificado contra HEAD 664fe472)

Contexto: GOV-TK3 se cerró el 2026-08-28 (`ad7a52af`) para `traverse→SIGUE`, `GraphTraversalBody start/mode/...` (forma MCP) y nota ensure-en-search. La auditoría GOV-B5 (live-fire curl real, `tasks/GOV-B5.md:40-82`, 2026-08-22, POSTERIOR al fix MCP-01 del 08-17) prueba que el yaml sigue difiriendo del HTTP real en 3 puntos. Esta tarea los cierra.

### Drift 1 — gramática IQL del yaml en minúsculas vs parser UPPERCASE

| Lado | Evidencia |
|---|---|
| Parser exige UPPERCASE case-sensitive | `src/parser/mod.rs:213` `tag("FROM")`, `:148` `tag("SIGUE")`, `:294` `tag("INSERT")`, etc. (nom `tag` = match exacto). Autocomplete sí es case-insensitive (`:582` `eq_ignore_ascii_case`, `:1847` test) pero **parsear no**. |
| Live-fire: minúsculas FALLAN | `tasks/GOV-B5.md:59`: `from agent/main` y `insert <id> as <type> fields k=v` → parse error. Keywords UPPERCASE obligatorias. |
| yaml documenta minúsculas + sintaxis inexistente | `docs/api/openapi.yaml:154` `from <entity> [where…] [rank by…]`, `:159` `from…where vec(<field>) <~>… [min_score <n>]`, `:171` `insert <id> as <type> [fields…] [with vector…]`, `:176` `update <id> [set…]`, `:181` `delete <id>`, `:186` `relate <source> -> <target> as <label>`, `:191` `message <role>…in thread…` — NINGUNA parsea. Formas reales: `INSERT NODE#… TYPE …`, `field ~ "text"[, min = n]`, `RELATE NODE#… --"…"--> NODE#…`, `INSERT MESSAGE … TO THREAD#…` (`src/parser/mod.rs:294-402`). |
| Docs correctas (testigos) | `docs/api/HTTP_API.md:84,170` "keywords are UPPERCASE / case-sensitive uppercase" ✅; `docs/api/IQL.md` usa UPPERCASE pero con 2 sub-drifts: `SIGUE <min>-<max>` (guión) vs parser `..` (`mod.rs:150`), y `TIPO` vs parser `TYPE` (`mod.rs:153`; `TIPO` solo existe como palabra reservada `:64`, no como keyword parseada). Afecta `IQL.md:34,47-48,148`. |
| Test actual NO lo cubre | `tests/api/openapi_yaml_parity.rs` solo chequea `textMatch`/`~`/`SIGUE` — no chequea case. |

**Decisión:** doc-fix. Reescribir `openapi.yaml:145-192` a gramática UPPERCASE real + corregir `IQL.md` (`..`, `TYPE`) + extender parity test (minúsculas deben FALLAR, formas canónicas deben parsear).

### Drift 2 — GraphTraversalBody del yaml ≠ HTTP real (roots numéricos + max_depth requerido)

| Lado | Evidencia |
|---|---|
| HTTP real bfs/dfs | `src/server/handlers.rs:792-801` `GraphTraversalRequest { roots: Vec<u128>, max_depth: usize, direction: Option<GraphDirection> }`; `GraphDirection` = `forward\|reverse\|both`, default `forward` (`:774-780`, `:840`). |
| Live-fire | `tasks/GOV-B5.md:61-63`: `{"roots":["7"]}` → 400 invalid number (¡strings rechazados!); `{"roots":[7]}` → 400 missing field `max_depth`; `{"roots":[7],"max_depth":2}` → `[7]` ✅. Respuesta legacy = array de ids numéricos (`:841` `Json(ids)`), NO `NodeDTO[]`. |
| HTTP real resto | `degree`/`centrality` → `GraphRootsRequest { roots: Vec<u128> }` SOLO (`:803-808`; `max_depth` extra se ignora — GOV-B5:64 ✅). `pagerank` → `{ roots, max_iterations?=100, damping?=0.85, tolerance?=1e-6 }` (`:820-831`). `v2/bfs|dfs` → `{ roots: Vec<String decimal-u128>, max_depth, direction?, limit?=50 }` (`:942-955`; strings SÍ porque el browser no parsea u128 — `:942-944`). `v2/degree` → `{ namespace: String, limit?=50 }` (`:957-964`). |
| yaml (mal) | `openapi.yaml:2033-2086` `GraphTraversalBody` con `start: string[]`, `mode: bfs\|dfs`, `direction: outgoing\|incoming\|both`, `filter` — forma del tool MCP `graph_traverse`, referenciada por los 8 paths HTTP `:676,:701,:726,:756,:786,:816,:841,:866` que NINGUNO acepta. |
| Test actual **fosiliza el drift** | `openapi_yaml_parity.rs:89-142` exige `start/mode/max_depth/direction` en el yaml — hay que reescribirlo a la forma HTTP real. |

**Decisión:** doc-fix. Bodies por endpoint en el yaml (bfs/dfs, roots-only, pagerank, v2-traversal, v2-degree) + respuestas 200 corregidas (legacy bfs/dfs → `integer[]`) + reescribir el test de paridad. NO tocar handlers (clientes reales + DTO desktop dependen de la forma actual).

### Drift 3 — search en DB fresca requiere rebuild-index previo

| Lado | Evidencia |
|---|---|
| `ensure` corre SOLO en startup | `src/server/bootstrap.rs:336-343`: `ensure_indexes_current()` en startup, skip si `read_only`. No hay ensure por-request en search. |
| Live-fire: PUT-record + search FALLA hasta rebuild | `tasks/GOV-B5.md:48-52`: fresh DB → PUT `/records` → `POST /search {text_query}` → `{"error":"text_index not found: bm25"}`; tras `POST /maintenance/rebuild-index` → resultados ✅ (score 0.57536423). Mecanismo: el PUT de records escribe nodos directo (el mantenimiento incremental del text index vive en el PUT de memoria — `src/sdk/api/memory.rs:128,460,507`; `put_batch` reconstruye en un pase `:334-336`), así que escrituras post-startup vía node/record API no alimentan el índice de texto. |
| yaml afirma lo contrario + puntero stale | `openapi.yaml:501-505`: "ensure…is called automatically…This prevents 'text_index not found' errors" (FALSO para escrituras post-startup) y "(see `src/cli_server.rs)" (STALE — desde `613da161` 09-01 es un shim; el código vive en `src/server/bootstrap.rs`). `docs/api/HTTP_API.md:77-78`: "(no manual rebuild needed on a fresh database)" (FALSO para el flujo records — mismo live-fire). |
| Test actual pasa pero es débil | `test_search_endpoint_documents_index_ensure` solo exige mencionar ensure/rebuild. |

**Decisión:** doc-fix (behavior-change — auto-rebuild en PUT/search — toca hot paths, exige benchmark Regla 9: fuera de appetite de un ticket docs). Reescribir nota del yaml (condición startup-only + síntoma + remedio + puntero correcto) + corregir `HTTP_API.md:77-78` + endurecer test (exige `rebuild-index`, síntoma `text_index not found`, y prohíbe el puntero `cli_server`).

## Impacto mapeado (Regla 0) — OBLIGATORIO antes del primer edit

- **Archivos a leer completos (regiones de edición):** `docs/api/openapi.yaml:142-192` (query desc), `:492-505` (search desc), `:666-880` (8 paths grafo), `:856-968` (v2/degree resp + pagerank path), `:2030-2087` (requestBodies); `docs/api/IQL.md:1-60,140-212`; `docs/api/HTTP_API.md:70-95`; `tests/api/openapi_yaml_parity.rs` (203L, leído ✅); `src/parser/mod.rs:140-160,211-250,290-410,475-500` (verificar formas exactas a documentar); `src/server/handlers.rs:906-940` (GraphTraversalDTO/GraphNodeDTO para resp v2), `:1085-1115` (resp v2/degree).
- **Referencias hacia dentro (quién consume lo que edito):** yaml consumido por `tests/api/openapi_yaml_parity.rs`, `scripts/validate-docs-coverage.ps1` (posible), lectores humanos, GOV-B5 (evidencia). IQL.md/HTTP_API.md solo humanos. Test consumido por `cargo test -p vantadb --test openapi_yaml_parity` + CI Fast Gate.
- **Referencias entrantes (quién depende):** `src/server/handlers.rs`, `src/parser/mod.rs`, `vantadb-mcp/.../tools.rs` NO dependen de los docs (una dirección: código → doc). Cero riesgo de rotura productiva.
- **Veredicto:** BLAST RADIUS = 4 archivos (3 docs + 1 test), 0 código productivo, 0 símbolos públicos nuevos, 0 cambios de comportamiento. Gate D (question-gates): NO disparado — sin símbolos nuevos, contrato no ambiguo (evidencia live-fire + código), feature-add n/a. Gate P: n/a (tarea asignada). Workflow MCP: n/a (tipo `docs`, sin workflow aplicable — steps atómicos vía writing-plans).

## SDP

`campaign_discover_skills phase=BUILD` → base + lifecycle (8 devueltas). Cargadas: writing-guidelines, writing-plans, documentation-and-adrs, spec-driven-development. Descubiertas no cargadas: incremental-implementation, test-driven-development (sin lógica nueva — solo asserts en test existente; Red-Green n/a), context-engineering (contexto ya empaquetado en este file). Excluidas con motivo: ai-seo (contrato interno yaml↔código, no contenido público indexable), release-notes-one-pager (sin bump de versión en este ticket). Code-intel grafos: omitidos con motivo — ground truth vía Read directo de regiones <150L (cero staleness, doc-parity no necesita call-graphs).

## Steps atómicos (~100L c/u, un step por turno, cada uno reversible)

- [x] **S1 — Drift 1a** ✅ (openapi.yaml query desc → UPPERCASE real)
- [x] **S2 — Drift 1b** ✅ (IQL.md `..`+`TYPE`; `rg TIPO` → 0)
- [x] **S3 — Drift 1c** ✅ (nuevo test case-sensitivity)
- [x] **S4 — Drift 2a** ✅ (5 bodies por endpoint; `GraphTraversalBody`/`GraphNodeList` eliminados; 0 refs)
- [x] **S5 — Drift 2b** ✅ (test reescrito a formas HTTP reales)
- [x] **S6 — Drift 3** ✅ (notas fresh-DB + test endurecido anti-`cli_server`)
- [x] **S7 — Gate + commit + cierre** ✅ (commit `b3be4176`, 4 archivos, hooks pre-commit ok)

## Context Save Point

- ✅ COMPLETO 2026-09-05, commit `b3be4176` (4 archivos, hooks pre-commit fmt+clippy+actionlint ok).
- Verify: parity 5/5 · parser lib 117/117 · `cargo fmt --check` limpio · `clippy -p vantadb --all-targets -D warnings` limpio · docs-coverage 0 gaps (pwsh7; powershell5.1 falla parseando el script — quirk entorno, script intacto en HEAD).
- Incidentes: build transitorio `tantivy rlib` + lock por sesión paralela (retry ok); targets `hardware_profiles|property_durability|fuzz_proptest|integration` no compilan en este entorno (pre-existente, fuera de scope — se usó `--lib`/`--test` acotado); full-workspace nextest excluido con motivo (0 código productivo).
- Deuda relacionada (NO scope, no tocar): `OPENAPI.yaml` mayúsculas (GOV-TK3-Ago usó `docs/api/OPENAPI.yaml` vs real `openapi.yaml` — el test ya fija lowercase `:16-17` ✅); claims numéricos Regla 11 intactos; MCP `graph_traverse` start/mode (forma propia, fuera del yaml HTTP).
