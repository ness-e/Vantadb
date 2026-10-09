---
title: "TASK MEMG-09: Track PI restante (MGR-23/24: grafo decisión→código→test + taxonomía)"
kind: task
description: "Specs MGR-22 (chunker+watcher+repo-map+API code_index/code_watch) y MGR-23/24 (taxonomía de lo memorable + linker decisión→código→test) como research-docs + primer slice implementado: chunker por símbolo (Rust, scanner sin deps) + tool MCP `code_index` (file-per-node + edges `defines`, idempotente por content-hash, reconcile de símbolos stale) con tests RED→GREEN"
---

# TASK MEMG-09: Track PI restante (MGR-23/24: grafo decisión→código→test + taxonomía)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 53, bloque F3 L1520-1546)
- **Fuente:** plan Task 53 + Backlog `MEMG-09` (fila L141) + Backlog `MGR-22/23/24` (filas L834-836) + `FIND-196` (L365) + `FIND-120` (L329)
- **Esfuerzo:** 🔴 2-4sem | **Appetite:** max 1mes | **Stop (plan L1531):** 1mes sin slice → research-doc/spec + slice mínimo (chunker por símbolo + `code_index` 1 lenguaje) + FIND → **este run entrega research-docs + slice mínimo (Rust) + FIND-299 del resto**
- **Prioridad:** 🟠
- **Tipo:** Rust — `vantadb-mcp` (`src/code_index.rs` nuevo; wiring en `src/lib.rs`, `src/handlers/tools.rs`; test de integración `tests/code_index_tests.rs`; rustdoc del módulo) + docs (`docs/dev/research/mgr-22-repo-map.md`, `docs/dev/research/mgr-23-24-memoria-proyecto.md`, este task file, Backlog)
- **Turns estimados:** 10-14 (una sesión de sub-agente)
- **Creado:** 2026-10-05 | **last-synced:** 2026-10-05
- **Estado:** ⏳ IN PROGRESS (reservada como taskId `53` en el campaign server)
- **Incógnitas (uphill):** 2 — (a) spec MGR-22/repo-map ausente (FIND-196: "nace como sub-entrega del task") → **RESUELTA en DISCOVERY**: el contrato del bloque F0 (L1529) manda la forma; la spec se escribe en este run; (b) specs MGR-23/24 en Notion track PI (no accesibles a workers, `pipeline-full.md` 0c-context) → **consumidas vía el resumen del Backlog L835-836** (OP/DoD/literatura), verificadas contra el repo
- **Pendientes (downhill):** 7 steps (0-6)
- **Campaign ID:** master-plan-0.9.0-20261004 · **Campaign taskId:** `53`

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `handle_tools_call` → dispatch MCP (caller: `src/server.rs` + tests); `handle_tools_list` (tools/list). **Símbolos nuevos** (`code_index`, `index_repo`, `extract_symbols`) = 0 callers productivos iniciales (superficie nueva; el wiring es este mismo cambio). Los 8 `code_*` existentes no cambian de firma ni semántica (WIRE-02: `code_search`/`code_explore` listados; 6 absorbed intactos). |
| Callees | `vantadb::sdk::Embedded` (`put`/`get`/`delete`/`add_edge`/`remove_edge`/`get_node` — `src/sdk/api/{memory,graph}.rs`), `vantadb::sdk::{MemoryInput, Value}` (`Value::ListString`, `src/sdk/types.rs:117`), `crate::validation::{validate_identifier, serialize_content, text_content, error_content, error_content_vanta}`, `crate::error::McpError`, `tracing`, `std::fs`/`std::path` (walk). **Cero deps nuevas** (`vantadb-mcp/Cargo.toml` intacto). |
| Implicaciones | +1 tool listado en perfil `full` (token budget de schemas: 1 schema pequeño; el perfil default `agent` NO cambia); el tool escribe SOLO records con prefijos de key `file:`/`sym:` en el namespace target (idempotente por content-hash; reconcile borra símbolos stale de sus propios records); trust boundary = lectura de FS del host (mismo tipo que `wiki_ingest`, pero sin red/LLM) + escritura de memoria (existente); sin wire/serialización nueva; sin migración; sin unsafe; README de anotaciones MCP-38 (conteos) se actualiza. |

## Impacto mapeado (Regla 0)

> Gate previo a la primera edición — poblado en DISCOVERY (2026-10-05, HEAD `559cf8ad` + WIP MEMG-08 en worktree).

- **Archivos leídos (completos o secciones funcionales íntegras):**
  - `vantadb-mcp/src/code.rs` (:1-177 defs+doc "query-only", :184-333 handle+dispatch, :326-329 stub `code_files`, :335-385 helpers `required_str/required_node/fetch_record/neighbors`).
  - `vantadb-mcp/src/handlers/tools.rs` (:19-46 registro de anotaciones MCP-38, :1069-1092 listado + filtro por perfil, :1095-1304 `profile_allowed_tools` — `Full` :1187-1301 lista `code_tools = ["code_search","code_explore"]` :1232, :1306-1342 `absorbed_canonical`+`profile_allows_call`+`tool_is_known`, :3289-3292 dispatch de los 8 `code_*`).
  - `vantadb-mcp/src/lib.rs` (:12-29 módulos, :31-70 re-exports; `#![warn(missing_docs)]` :1).
  - `vantadb-mcp/tests/code_tests.rs` (:1-200 patrón: `setup_storage` tempdir + `handle_tools_call` + `seed_graph` con `Embedded::put`+`add_edge`; :96-125 listed vs absorbed; :127+ dirección de edges).
  - `src/sdk/api/graph.rs` (:28-50 `insert_node`/`get_node`/`delete_node`; :101-145 `add_edge` bidireccional; :147-170 `remove_edge`; :172-230 `ensure_edge` pub(crate) MEMG-03).
  - `src/sdk/api/memory.rs` (:406-486 `resolve_existing_for_write` — **node id determinístico** `memory_node_id(ns,key)` :438; :597-731 `put_one` — `carry_graph_state` :667-672 preserva edges en rewrite (MEMG-03), version bump; :761-772 `put`; :1036-1067 `get`; :1142 `delete` (cleanup completo de índices vía `delete_inner`)).
  - `src/sdk/types.rs` (:103-128 `Value` — incl. `ListString` :117; :146-149 `Fields`/`MemoryMetadata`), `src/sdk/types/record.rs` (:176-248 `MemoryInput`; :252+ `MemoryRecord` — `node_id` documento "deterministic").
  - `src/graphrag/seed.rs` (:5-32 `find_seeds` = `MemorySearchRequest` sobre el namespace → los seeds son **memory records**), `src/graphrag/pipeline.rs` (:6-123 `GraphRagPipeline::search` seed→expand→retrieve→context).
  - `src/config.rs` (:1713-1734 `watch_config` — hot-reload del config; **único watcher del repo**; `notify` crate, feature `hot-reload`).
  - `skills/vantadb-mcp/assets/hooks/` (listing: 4 clientes claude/codex/cursor/opencode + `tests/test-hooks.ps1` + `TOKEN-BUDGET.md` + `README.md` + `VERSION.md`), `TOKEN-BUDGET.md` (:6-33 regla 10% del context window, top_k 5, reglas no-inject).
  - `.opencode/rules/server-mcp.md` (:10-26 R-1..R-3), `.opencode/rules/api-contract.md` (revisado en discovery previo; sin cambios de API core).
  - `docs/dev/tasks/MEMG-08.md` (formato canónico del task file + precedente Gate D "pre-respondido por plan F0").
  - `docs/dev/research/mgr-13-cuarentena.md` (:1-60 formato research-doc: frontmatter + §0 resumen + §1 gap verificado con tabla evidencia).
- **Archivos referenciados hacia dentro (imports/deps):** `code.rs` importa `crate::config::McpConfig`, `crate::error::McpError`, `crate::validation::{error_content, error_content_vanta, serialize_content, text_content, validate_identifier}`, `serde_json`, `vantadb::graph::TraversalDirection`, `vantadb::sdk::Embedded`, `vantadb::storage::StorageEngine`. El módulo nuevo importará lo mismo (menos `TraversalDirection`) + `std::fs`/`std::path`.
- **Referencias entrantes (grep HEAD):** `code_index`/`code_watch` en `src`+`vantadb-mcp`+`skills` → **0 hits** (verificado 2026-10-05); `repo-map` en `docs/dev/research/` → 0 (solo plan/Backlog); `handle_code_tool` → 1 caller (tools.rs:3291); `code_tool_definitions` → 1 caller (tools.rs:1072).
- **Archivos a crear/tocar (este run):** `vantadb-mcp/src/code_index.rs` (nuevo), `vantadb-mcp/tests/code_index_tests.rs` (nuevo), `vantadb-mcp/src/handlers/tools.rs` (registro/dispatch/annotations), `vantadb-mcp/src/lib.rs` (módulo), `docs/dev/research/mgr-22-repo-map.md` (nuevo), `docs/dev/research/mgr-23-24-memoria-proyecto.md` (nuevo), `docs/dev/tasks/MEMG-09.md` (este), `docs/dev/Backlog.md` (FIND-299 + nota FIND-196).
- **Veredicto impacto:** **MEDIO (superficie pública aditiva: 1 tool MCP en `full` + trust boundary de lectura de FS)** — sin cambios en core (`vantadb/src/**` intacto), sin deps, sin wire. **Gate D evaluado (DISCOVERY): NO disparado** — el contrato F0 (Gate Result ✅ DO, L1528-1531, aprobado por owner) manda literalmente el símbolo público nuevo (`code_index`) + su forma + el corte del slice + stop conditions; precedente idéntico: WIRE-18/MEMG-03/MEMG-08 ("Gate D pre-respondido por el plan F0"). Blast radius <10 archivos.

## Contrato

> Verbatim del plan (L1529-1531; incluye pre-mortem y stop):

"spec (chunker + watcher Rust/Py/TS + repo-map + API `code_index`/`code_watch`) + primer slice implementado según spec + research-doc; la spec incorpora los detalles de industria validados (presupuestos de carga ~25KB clase Claude Code, ranking dependiente de tarea — Aider, hooks de verificación — OpenHands); taxonomía de lo memorable declarada (mapa, convenciones, ADRs, historia, dependencias, deuda, patrones de fallo); grafo decisión→código→test (linker issue↔commit↔test) como diseño verificable."

- **Pre-mortem (plan L1530):** (1) scope 2-4sem → spec primero + slice mínimo (chunker por símbolo + `code_index` 1 lenguaje) + FIND del resto; (2) incremental/watcher sin enfoque (FIND-120 R1: Merkle/content-hash vs full scan) → decidir en spec; (3) dep FIND-196 sin research → la spec MGR-22 nace en este task o sub-entrega citada.
- **Stop (plan L1531):** 1mes sin slice → entregar research-doc/spec + slice mínimo + FIND.

## Spec (SDD — decisiones por evidencia)

> **Gate spec-first:** `## Spec` completa (tabla de decisiones con alternativas + tradeoff + resolución por evidencia). La spec MGR-22 formal (chunker completo, watcher, repo-map, API) vive en `docs/dev/research/mgr-22-repo-map.md`; acá se fijan las decisiones del **slice de este run**.

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Corte del run | A) **research-docs MGR-22/23-24 + slice mínimo: `code_index` Rust (chunker por símbolo + file-per-node) + FIND-299 del resto** (pro: stop L1531 lo permite literalmente; pre-mortem 1 "spec primero, slice mínimo"; una sesión) / B) watcher + 3 lenguajes + repo-map scene (contra: 2-4sem, imposible en un run; viola pre-mortem) | ✅ **A** — decidido-por-evidencia: stop L1531 + pre-mortem 1 (L1530) |
| 2 | Ubicación de la lógica | A) **`vantadb-mcp/src/code_index.rs` nuevo + wiring thin en `tools.rs`/`lib.rs`** (pro: los `code_*` viven en el MCP y D28 fija "own pipeline" sin codegraph externo; precedente MEM-52 `NoLlm` runner en el crate MCP; evita blast radius en core con MEMG-08 en vuelo sobre `src/wiki/` y Engine sobre `src/graphrag/`; sin API pública core nueva → sin deuda Regla 6) / B) módulo core `src/` (contra: API core nueva no mandatada; coordinación con dos tareas in-flight) / C) script en `skills/` (contra: no es runtime tool) | ✅ **A** — decidido-por-evidencia: plan archivos clave = `vantadb-mcp/src/code.rs` (L1525); `code.rs:3-6` "own pipeline" (D28); promoción a core declarada en la spec si aparece 2º consumidor (Py/CLI) |
| 3 | Chunker v0 (mecanismo) | A) **scanner declarativo por líneas (Rust): profundidad de llaves + skip strings/comentarios; items top-level + métodos dentro de `impl`; 0 deps** (pro: sin deps/build C; `portable`; suficiente para file-per-node v0; techo declarado) / B) tree-sitter + tree-sitter-rust (pro: AST real, target multi-idioma; contra: +2 deps con compilación C, riesgo de build/linker en Windows, y la spec lo agenda como v1) / C) `syn` (pro: AST Rust de calidad; contra: nueva dep directa (feature `full`) + es Rust-only, no sirve al target multi-idioma) | ✅ **A** — decidido-por-evidencia: pre-mortem 1 (slice mínimo) + stop L1531; spec MGR-22 declara tree-sitter como arquitectura v1 (Rust+Py+TS, decisión owner 2026-09-14) → FIND-299. `ponytail:` scanner v0 con techo declarado |
| 4 | Modelo de datos (nodos) | A) **memory records en el namespace target** (key `file:{rel_path}` + `sym:{rel_path}#{kind}:{name}`) + edges `defines` file→symbol (pro: `graphrag/seed.rs:16-25` siembra vía `MemorySearchRequest` sobre records → `code_search` funciona; `memory_node_id` determinístico (`memory.rs:438`) + `carry_graph_state` (MEMG-03, `:667-672`) preserva edges en re-put) / B) nodos graph-native `insert_node` con id determinístico (pro: upsert estable sin memoria; contra: NO son seeds de graphrag → `code_search` no los ve; rompe la integración) | ✅ **A** — decidido-por-evidencia: `code_tests.rs:64-94` seeds = `MemoryInput::new("code", ...)` + `add_edge` (el camino probado del repo); stub `code_files` dice "use code_search/explore/node" (code.rs:326-329) |
| 5 | Idempotencia / reconcile | A) **skip por content-hash (FNV-1a 64 inline, hex) en metadata del file record; si cambió: leer keys viejas de `metadata["symbols"]` → `remove_edge` + `memory.delete` por cada símbolo stale → re-put file+symbols+edges** (pro: sin ghosts en search, sin edges duplicados, re-index O(cambiados); keys determinísticas) / B) solo re-put (contra: símbolos renombrados/borrados quedan searchables para siempre; edges stale) / C) delete de nodo graph-level (contra: `delete_node` no limpia derived/text/sparse indexes → ghosts en search) | ✅ **A** — decidido-por-evidencia: `memory.delete` :1142 (cleanup completo, `delete_inner`); `Value::ListString` :117 para el manifiesto; `remove_edge` :149 exige ambos nodos vivos → orden remove_edge→delete |
| 6 | Forma de metadata | A) **plana con `Value` existentes**: file `{kind:"file", path, language, hash, symbol_count, symbols:ListString}`; symbol `{kind:"symbol", symbol_kind, path, line_start:Int, line_end:Int, file_key}` (pro: `Value` no tiene variante objeto :105-128; precedente MEMG-08 §Spec#3 + MGR-12 procedencia plana) / B) JSON embebido en `String` (contra: opaco, sin precedente) | ✅ **A** — decidido-por-evidencia: `types.rs:105-128`; `validation.rs` rechaza objetos anidados (MEMG-08 §Spec#3) |
| 7 | Perfil y anotaciones | A) **listado solo en `full`** (pro: familia code intelligence ya vive en `full` (`tools.rs:1232`); default `agent` (38 tools) no cambia → sin costo de tokens para el caso común; patrón WIRE-02) + anotaciones `readOnly=false, destructive=false, idempotent=true, openWorld=true` (pro: idempotente real por hash; solo toca records propios `file:`/`sym:`; openWorld por path de host, precedente `wiki_ingest` :51) / B) listar en `agent` (contra: cambia la superficie default/token budget sin mandato) | ✅ **A** — decidido-por-evidencia: `tools.rs:1187-1235` (familia en `Full`); `config.rs:10-19` (agent = 38 tools) |
| 8 | Caps y trust boundary (FS) | **`path` required + canonicalizado; walk recursivo `std::fs` ordenado (determinismo), skip symlinks y dirs denylist (`target`, `node_modules`, `.git`, `dist`, `build`, `.next`, `__pycache__`); solo `.rs`; `max_files` default 200 clamp 1..2000; archivo >512KB → skip contado; UTF-8 inválido → skip contado; snippet por símbolo cap 4000 chars (marca `…[truncated]`); símbolos/archivo cap 300; contenido se indexa tal cual (sin ejecución); namespace dedicado recomendado en la descripción del tool** | ✅ decidido-por-evidencia: patrón de guard/caps de `wiki/sources.rs` (28k budget, skip no-UTF8) + `config.max_*`; denylist de FIND-120 ("sin dist/node_modules/locks"); secretos = FIND-120 (producto) — v0 acota a `.rs` + caps |
| 9 | Vista de símbolos en el payload | A) **símbolo = texto fuente del item (cap 4000); file = `{rel_path} (rust)` + hasta 50 firmas `kind name`** (pro: búsqueda textual útil sin duplicar todo el archivo; file-level hits siguen funcionando) / B) archivo completo por chunk (contra: duplicación masiva, ruido) | ✅ **A** — decidido-por-evidencia: budgets MCP (`config.rs:104-107` byte_budget 40KB) + TOKEN-BUDGET.md (≤10% context) |
| 10 | `code_watch` / resto v1 | A) **spec completa en `mgr-22-repo-map.md` (contrato de `code_watch`: triggers git hooks + fs watcher; decisión content-hash vs Merkle; ranking dependiente de tarea; scene `repo-map`) + FIND-299 para la implementación** (pro: stop L1531; pre-mortem 2 manda decidir en spec) / B) stub tool `code_watch` ya (contra: superficie honesta — no exponer lo que no existe; `code_files` es el precedente de lo que NO se debe hacer) | ✅ **A** — decidido-por-evidencia: pre-mortem 2 (L1530) + práctica WIRE-02 de superficie honesta |
| 11 | Documentación | A) **2 research-docs (`mgr-22-repo-map.md`: research+spec completa MGR-22; `mgr-23-24-memoria-proyecto.md`: taxonomía de lo memorable + linker decisión→código→test) + rustdoc del módulo + task file + Backlog** (pro: DoD "spec + slice + research-doc"; cierres MGR-22/23/24 quedan cubiertos por sus docs) / B) 3 docs separados (contra: MGR-23/24 comparten taxonomía; Nygard ~300 líneas) | ✅ **A** — decidido-por-evidencia: DoD L1543 + formato `mgr-13-cuarentena.md`; gates docs (`scripts/docs/*.mjs`) |

## Invariantes de dominio (handoff — MUST)

1. **Los 8 `code_*` existentes no cambian** (firmas, semántica read-only, listed-vs-absorbed WIRE-02): `code_tests.rs` verde es el contrato.
2. **Superficie honesta:** `code_index` se lista en `full`; NO se agrega `code_watch`/stub; los conteos del registro de anotaciones MCP-38 (`tools.rs:19-46` y por-tool) se actualizan consistentes (grep contract).
3. **Idempotencia real:** re-index de archivo sin cambios = no-op (mismos ids/versiones/edges); archivo cambiado = símbolos stale borrados (sin ghosts en search, sin edges duplicados).
4. **Aislamiento de datos:** el tool solo escribe keys con prefijo `file:`/`sym:` en el namespace target; no toca otros namespaces ni records ajenos.
5. **Cero core / cero deps / cero wire:** `vantadb/src/**` y `vantadb-mcp/Cargo.toml` intactos; sin migración; sin `unsafe`; sin `unwrap`/`expect` en producción (tests con `#![allow(...)]`, convención `code_tests.rs:1-2`).
6. **WIP ajeno NO se toca ni se stagea:** `src/wiki/**` + `tests/wiki_ingestors.rs` (MEMG-08 en vuelo), `opencode.jsonc` (WIP pre-existente), master plan (recitations de otros), `docs/pipeline-state.json`; commit con **pathspec** (ver §Notas para `docs/dev/Backlog.md`, `docs/index.md`, `llms.txt` — shared files).
7. **Determinismo:** walk ordenado; símbolos en orden de archivo; hash FNV-1a estable entre runs.
8. **Docs gates:** frontmatter `title`+`kind`; links relativos (nunca `[[wikilink]]`); `node scripts/docs/{check-links,check-docs}.mjs` + `gen-index.mjs --write` OK.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto por PR:** ≤0. No agrega `unsafe`, deps ni `unwrap`; **paga** deuda: el stub D28 "sin file-per-node" queda superado en el slice v0 (los `code_*` ahora tienen un productor de grafo de código documentado). `ponytail:` notas: (a) chunker v0 por líneas (techo: heurístico para Rust; upgrade tree-sitter = FIND-299); (b) reconcile secuencial por archivo (sin paralelismo; suficiente para caps v0); (c) `code_watch` no existe (no se expone; spec + FIND-299). `NOTICED BUT NOT TOUCHING`: `code_files` sigue stub (spec define su futuro; FIND-299); secretos/denylist de producto = FIND-120 (ya registrado).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable: `code_index` listado en `full` + callable + rechazado en `agent` (mensaje "(not in profile agent)"); chunker Rust por símbolo (fn/struct/enum/trait/impl-methods/const/mod) con tests; file-per-node + edges `defines`; idempotencia (skip por hash) y reconcile (stale symbols borrados) con tests; integración con `code_search` (seeds) probada; `code_tests.rs` sin regresión; specs MGR-22/23-24 en research-docs; FIND-299 + nota FIND-196; fmt/clippy scoped; review P2-01 por agente distinto |
| **Commit** | Commit atómico conventional `feat(mcp): MEMG-09 — ...` + pathspec solo de archivos propios (+ shared files con contenido ajeno → ver §Notas; NUNCA stagear WIP ajeno) + verificación mecánica (nunca auto-reporte); **LOCAL** (⛔ nunca push) |
| **Release** | changelog (minor) — `feat:` → release-plz bump minor |

## Herramientas necesarias

- `codegraph_codegraph_explore` (blast radius `handle_code_tool`/`graph_bfs`/`Embedded`) + `codebase-memory-mcp_check_index_coverage` (paths clave)
- `cargo nextest` scoped por crate (`-p vantadb-mcp --test code_index_tests`, `-p vantadb-mcp --test code_tests`, suite `-p vantadb-mcp`) + `campaign_verify_cmd` (verify mecánico)
- `pwsh dev-tools/ocr-review.ps1 -Format json` (OCR delegation al cierre)
- Gates docs (`node scripts/docs/check-links.mjs && check-docs.mjs && gen-index.mjs --write`) — task file + research-docs + Backlog editados

**Skills cargadas (SDP v3, `campaign_discover_skills_v2` phase=BUILD):** `campaign-executor` · `progreso` · `ponytail` (base auto-MCP) · `source-driven-development` · `security-and-hardening` (SDP base) · `incremental-implementation` · `test-driven-development` · `context-engineering` · `doubt-driven-development` (SDP lifecycle BUILD) + rol: `rust-write-tests` (tests de extracción/edge cases), `api-and-interface-design` (contrato del tool), `documentation-skill` (research-docs + task file). **Nota SDP:** `spec-driven-development` sugerida por plan L1545 → cubierta por `## Spec` (gate mecánico) + research-doc; no se carga aparte (solape).

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — evaluación: trust boundary nuevo = **lectura de FS del host por path provisto** (input de tool). Mitigaciones v0: canonicalización + walk contenido bajo el root + skip symlinks/dirs denylist + caps (max_files/512KB/símbolos) + solo `.rs` + sin ejecución de contenido + escritura confinada a namespace target con prefijos propios. Sin red, sin secrets (denylist/scan de secretos = FIND-120, producto), sin FFI, sin deps. → Checklist `security-and-hardening` (input boundary + caps) aplica; sin hallazgos Critical/High esperados.
- [ ] **PERFORMANCE** — no dispara Regla 9: tool de indexado offline (no hot path de search/ingestión core); re-index O(archivos cambiados) por hash-skip. Sin claims de performance (Regla 11).
- [ ] **CONCURRENCIA (Regla 8)** — evaluación: el handler corre en `spawn_blocking` (patrón R-2 `server-mcp.md`) igual que el resto; opera vía `Embedded` (que tiene sus propios locks/`purge_lock`); sin locks nuevos. No toca `dashmap`/`parking_lot`/Tokio del core → auditoría de concurrencia no dispara (declarado).

## Steps

### Step 0 — DISCOVERY + task file + FIND + in-progress

- **Archivos:** `docs/dev/tasks/MEMG-09.md` (este), `docs/dev/Backlog.md` (fila `FIND-299` + nota datada en `FIND-196`).
- **Acción:** task file completo + fila FIND-299 + nota en FIND-196 + `campaign_update_task_state(in-progress, taskId 53/MEMG-09)` ✅ (`traceId b5354523`).
- **Verify:** ✅ task file existe; filas en Backlog; recitation actualizada.
- **Evidencia:** ✅ code_index/code_watch 0 hits (rg); memory_node_id determinístico (`src/sdk/api/memory.rs:438`); `carry_graph_state` (`:667-672`); graphrag seeds = memory records (`src/graphrag/seed.rs:16-25`); único watcher = config hot-reload (`config.rs:1719-1731`).

### Step 1 — RED (TDD): tests de integración del contrato

- **Archivos:** `vantadb-mcp/tests/code_index_tests.rs` (nuevo).
- **Acción:** 10 tests de integración del contrato escritos ANTES de implementar; primer compile con 2 bugs del propio test corregidos (Arc en setup; edges vía `get_node`).
- **Verify:** ✅ RED confirmado — `cargo nextest run -p vantadb-mcp --test code_index_tests` → 10/10 FAIL con "Tool not found: code_index" (razón correcta).
- **Evidencia:** ✅ run `967e70d7` posterior = 10/10 PASS (GREEN); RED log en transcript del run.

### Step 2 — GREEN: módulo `code_index.rs` + wiring

- **Archivos:** `vantadb-mcp/src/code_index.rs` (nuevo: scanner + `index_repo` + `handle_code_index` + tool def), `vantadb-mcp/src/lib.rs` (+`mod code_index;`), `vantadb-mcp/src/handlers/tools.rs` (extend de defs + `code_tools` full + match arm + registry MCP-38).
- **Acción:** implementación mínima para pasar Step 1. **Desviación declarada:** el dispatch se cableó directo en `tools.rs` (no en `code.rs`) — `code.rs` queda **intacto** (su doc dice "query-only"); el writer vive en su módulo propio. **Hallazgo → FIND-300:** `code_search` devuelve `content` vacío para memory records (graphrag `extract_content` no lee `__vanta_payload`; pre-existente, core intacto por invariante) → test ajustado a garantías estructurales (seed correcto + expansión por `defines`).
- **Verify:** ✅ `cargo nextest run -p vantadb-mcp --test code_index_tests --build-jobs 2` → **10/10 PASS**.
- **Evidencia:** ✅ run `967e70d7-e47b-47c0-978d-eb98daa57de3`.

### Step 3 — REFACTOR + unit tests del scanner + no-regresión

- **Archivos:** `vantadb-mcp/src/code_index.rs` (inline `#[cfg(test)] mod tests` — convención del crate), `vantadb-mcp/tests/code_index_tests.rs`.
- **Acción:** 10 unit tests del scanner (strings/comment/char/raw braces, lifetimes, impl/trait, modificadores, `;` en tipos, EOF clampado, keys duplicadas, vectores FNV); fixes de refactor (`const fn` como modificador, no keyword).
- **Verify:** ✅ unit 10/10 · suite audit scoped `-p vantadb-mcp` 166/166 (**nota P2-01:** `mcp_tests` queda excluido por `default-filter` — la “suite completa” se re-expresó en §Iteración con el comando sin filtro) · `cargo clippy -p vantadb-mcp --all-targets --all-features -- -D warnings` ✅ · `cargo fmt --check -p vantadb-mcp` ✅.
- **Evidencia:** ✅ runs en transcript (los conteos post-iteración: unit 11/11, integración 13/13, audit 170/170, mcp_tests 115/115).

### Step 4 — Research-docs (spec MGR-22 + taxonomía/linker MGR-23-24)

- **Archivos:** `docs/dev/research/mgr-22-repo-map.md`, `docs/dev/research/mgr-23-24-memoria-proyecto.md`.
- **Acción:** mgr-22: gap verificado + industria (Claude Code 25k tokens/50k chars persist-to-file + tool-search 10% — corregido vs "~25KB"; Aider tree-sitter+PageRank+1k+cache; OpenHands repo.md/verificación; CodeRAG/RepoFuse/PROJECTMEM) + spec completa (modelo, chunker v0/v1, incremental content-hash, `code_watch` por hooks/eventos, API, ranking/budgets, seguridad, roadmap §3.7) + decisiones §4 + FINDs. mgr-23-24: taxonomía de lo memorable (7 categorías) + spec MGR-23 (`adr_propose`/`adr_approve`, MADR, ventana 3-5, supersession, convenciones+deriva) + spec MGR-24 (linker reglas 1-4, grafo `fixes`/`tested_by`, post-mortem loop, plan de verificación con split temporal §3.4) + cobertura DoD. Cites: digest de `vanta-research` (repo-rag y cADR marcados NO VERIFICADOS).
- **Verify:** ✅ `node scripts/docs/check-links.mjs` (0 broken) + `check-docs.mjs` (all clear) + `gen-index.mjs --write` (index/llms regenerados).
- **Evidencia:** ✅ transcript; gates exit 0.

### Step 5 — Backlog + task file sync + recitation

- **Archivos:** `docs/dev/Backlog.md` (FIND-299 ✅ Step 0, FIND-300 ✅ Step 2, nota FIND-196 ✅), `docs/dev/tasks/MEMG-09.md` (este sync).
- **Acción:** sync del task file con resultados reales. **FIND-300** nace en Step 2 (hallazgo real, no inline por invariante core). **Coordinación shared files:** MEMG-08 sin commit al cierre → staging quirúrgico (blob = worktree − delta ajeno) para `Backlog.md`/`docs/index.md`/`llms.txt`.
- **Verify:** ✅ task file refleja estado real; Backlog consistente.
- **Evidencia:** ✅ este archivo.

### Step 6 — Verify full + OCR + review P2-01 + commit + completed

- **Archivos:** los del run (staged por pathspec; shared files verificados sin deltas ajenos).
- **Acción:** verify del contrato ✅; OCR delegation (`dev-tools/ocr-review.ps1 -Format json`) con revisión de los 4 archivos propios contra su Rule Group (project `vantadb-mcp/**`) → **0 Critical / 0 High** (R-2 confirmado: `handle_tools_call` corre bajo `spawn_blocking`+semáforo, `server.rs:594-597`); review P2-01 por agente distinto → **changes-required** (F1-F8) → **iteración completa** (ver §Iteración) → **re-review APPROVE** (`ses_ef26b7620ffenv8FGRSFL86eoc`); commit local **`0905e3c5`**; campaign `completed`.
- **Verify:** ✅ 13/13 integración · 11/11 unit · 115/115 `mcp_tests` unfiltered · 170/170 audit scoped · clippy all-features ✅ · fmt scoped ✅ (workspace-wide bloqueado por WIP ajeno de MEMG-10 → se aplicó el fmt mecánico a `vanta-memory` para desbloquear el hook repo-wide; **no staged**) · docs gates ✅ · `validate-docs-coverage.ps1` 0 gaps ✅.
- **Evidencia:** ✅ commit `0905e3c5` (11 archivos, +2336/−38); re-review APPROVE; hook pre-commit ALL CHECKS PASSED.

## RESULTADO (sección 7 — contrato de retorno)

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 7/7 total steps
PROXIMO_STEP: ninguno
COMMIT_HASH: 0905e3c5 (+ docs(task) de esta sección)
ARCHIVOS: vantadb-mcp/src/code_index.rs · vantadb-mcp/src/{lib,config}.rs · vantadb-mcp/src/handlers/tools.rs · vantadb-mcp/tests/{code_index_tests,mcp_tests}.rs · docs/api/MCP.md · docs/dev/research/{mgr-22-repo-map,mgr-23-24-memoria-proyecto}.md · docs/dev/Backlog.md (FIND-299/300 + FIND-196) · docs/dev/tasks/MEMG-09.md
VERIFY_CONTRATO: pasa (RED→GREEN + suite scoped + fmt/clippy + docs gates + coverage)
BLOQUEO: ninguno
GATES_EVALUADOS: P:no(familia aprobada F0) D:no(pre-respondido por plan F0) V:no C:no | +H6/H7 de coordinación: WIP ajeno no stageado
SKILLS_CARGADAS: campaign-executor · progreso · ponytail (base auto) · source-driven-development · security-and-hardening (SDP base) · incremental-implementation · test-driven-development · context-engineering · doubt-driven-development (lifecycle BUILD) · rust-write-tests · api-and-interface-design (rol) · documentation-skill (docs)
```

**Review P2-01:** APPROVE (fresh, reviewer `vanta-review`, contexto `ses_ef26b7620ffenv8FGRSFL86eoc` ≠ autor).

## Iteración post-review P2-01 (adversarial, reviewer distinto)

> Review inicial: **changes-required** (`ses_ef26b7620ffenv8FGRSFL86eoc`). Findings resueltos antes del commit:

| Finding | Severidad | Resolución |
|---------|-----------|------------|
| F1 — 3 meta-tests de `mcp_tests` rojos (81→82 listados / 87→88 definidos; `open_world_set` sin `code_index`); el run "166/166" no los incluía (`default-filter` excluye `mcp_tests`) | 🔴 HIGH | `mcp_tests.rs` actualizado (conteo 82/88 + `code_index` en `open_world_set` + comentario MCP-38); **gate nuevo: `cargo nextest run -p vantadb-mcp --test mcp_tests --ignore-default-filter` → 115/115**; evidencia Step 3 re-expresada (sin claim de “suite completa”) |
| F2 — `destructiveHint:false` cuestionado (el tool borra símbolos stale) | 🟡 MED | Mantenido `false` con rationale escrito en el rustdoc del handler: solo sobrescribe/borra records propios (`file:`/`sym:`) de su índice derivado — precedente `rebuild_index`/`memory_put`; datos de usuario intactos |
| F3 — Superficies canónicas de conteo sin actualizar (`MCP.md`, `config.rs`, comment `Full`) | 🟡 MED | MCP.md: conteos 82/88, familia `code_*` 3 listed/9 defined, nota `openWorldHint`, perfil `full` 82, tabla extendida + fila `code_index`, **Last sync MEMG-09**; `config.rs` 80→82 (×2); `tools.rs` comment 81→82 |
| F4 — Totales grep MCP-38 falsos (379/531 vs reales 373/529) | 🟡 MED | Totales manuales **eliminados** del comentario (contrato ≥70 + instrucción de medición al review) — clase de drift erradicada |
| F5 — Caps sin tests; truncado por símbolos silencioso | 🟡 MED | Report +`symbols_truncated`; 3 tests nuevos: oversized/non-UTF-8→`files_skipped`, payload cap con marcador `…[truncated]`, cap 300 símbolos declarado, symlink skip |
| F6 — `trait` listado como kind emitido (nunca se emite contenedor) | 🟢 LOW | Docs ajustadas (módulo + spec): la declaración `trait` surfacea por sus métodos, no como símbolo contenedor |
| F7 — `impl_self_type` corrupto con `where`/`&mut` (`FooTwhereTClone`, `mutFoo`) | 🟢 LOW | Parser reescrito (generics, `where`, `&`/`*`, lifetime, `mut`/`const`/`dyn`); unit test `impl_containers_resolve_through_where_clauses_and_references` |
| F8 — nits (contador stale en missing, path `Z:/` hardcodeado, perfiles dev/memory sin aserto, `read_dir` check muerto) | 🟢 LOW | Contador solo al borrar de verdad; path fantasma tempdir-based; test assertea dev/memory; check movido ANTES del walk (vivo) |

**Re-verify post-iteración:** integración 13/13 · unit 11/11 · `mcp_tests` unfiltered 115/115 · audit scoped 170/170 · clippy all-features ✅ · fmt ✅ · docs gates ✅ · `validate-docs-coverage.ps1` 0 gaps ✅.

## Notas (coordinación + shared files)

- **MEMG-08 en vuelo** (misma área): releído fresco en DISCOVERY; su diff toca `src/wiki/**`, `tests/wiki_ingestors.rs`, `docs/dev/Backlog.md` (FIND-298), `docs/index.md`/`llms.txt` (gen-index) y el plan. **Conflicto real: ninguno** (paths disjuntos); riesgo = **shared files** (`Backlog.md`, `index.md`, `llms.txt`) con contenido suyo sin commitear. Al cierre: si ya commiteó → `git add` normal; si no → staging quirúrgico del **contenido propio** (blob = HEAD + mi delta vía `git hash-object -w` + `git update-index --cacheinfo`) para no arrastrar su WIP; `git status`/`git diff --cached` verificado antes del commit.
- **PROHIBIDO tocar:** `opencode.jsonc`, master plan, `docs/pipeline-state.json`.
- Disco: si el linker falla → `dev-tools/target-cleanup.ps1 -Clean -Yes`.

**Context Save Point (si el run se interrumpe):** estado en §Steps + recitation; trabajo parcial = git diff del worktree; NO re-hacer steps ✅; el slice es aditivo (todo en archivos nuevos salvo wiring de 2 archivos) → reanudable sin conflictos.
