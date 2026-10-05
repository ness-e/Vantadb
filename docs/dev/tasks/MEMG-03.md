---
title: "TASK MEMG-03: Grafo ↔ memoria (L1–L3 como nodos/aristas)"
kind: task
description: "Integración memoria↔grafo en el core SDK: aristas de linaje `superseded_by` (supersede) y `derived_from` (put derivado) con labels canónicos + fix del wipe de aristas en rewrites de record (`memory_record_to_node_owned` no copia edges) + query de linaje vía BFS + provenance con test dedicado + DX-04 documentado"
---

# TASK MEMG-03: Grafo ↔ memoria (L1–L3 como nodos/aristas)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 50, bloque F0-expandido L1436-1462 — primera tarea de F3)
- **Fuente:** plan Task 50 + Backlog MEMG-03 (L-s/ver) + Backlog `DX-04` (:158, dueño vanta-worker/docs · MEMG-03) + H-006
- **Esfuerzo:** 🔴 1-2sem | **Appetite:** max 2sem | **Stop (plan L1447):** 2sem sin contrato → entregar (a) aristas de linaje (supersede + derived_from) + query de linaje con test + (b) FIND del resto (escenas/persona como nodos enlazados)
- **Prioridad:** 🔴
- **Tipo:** Rust — core SDK (`src/sdk/api/memory.rs`, `src/sdk/api/graph.rs`) + test de integración nuevo + doc de contrato (`docs/api/EMBEDDED_SDK.md`)
- **Turns estimados:** 7-10 (una sesión de sub-agente)
- **Creado:** 2026-10-05 | **last-synced:** 2026-10-05
- **Estado:** ⏳ IN PROGRESS (reservada como taskId `50` en el campaign server)
- **Incógnitas (uphill):** 1 — ¿sobreviven las aristas al `put`? → **RESUELTA en DISCOVERY (mecanismo + test RED): NO sobreviven** (wipe confirmado; ver §Spec #2)
- **Pendientes (downhill):** 7 steps (1-7)
- **Campaign ID:** master-plan-0.9.0-20261004 · **Campaign taskId:** `50`

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `Embedded::put`/`put_batch`/`supersede`/`put_record_exact`/`reinforce`/`quarantine_apply`/`quarantine_promote` (bindings PyO3/WASM/Node, MCP, HTTP, CLI — vía SDK); `add_edge`/`remove_edge` (MCP `remove_edge` :636, bindings). **Sin cambios de firma** (helper privado `carry_graph_state`, `ensure_edge` `pub(crate)`); `supersede` gana un efecto aditivo (arista `superseded_by`) — comportamiento público extendido, no roto. |
| Callees | `StorageEngine::{insert, get, intern_label, batch_insert_with_opts}`, `UnifiedNode::{edges, label_index}`, `GraphTraverser::bfs_traverse` (src/graph.rs:66), `memory_node_id`/`memory_record_to_node_owned` (serialization/mod.rs:77,:511), `Edge` (src/node). |
| Implicaciones | 0 cambios de wire/serialización (las aristas ya se serializan en `NodeMetadata`, insert.rs:619-624); 0 migración; rewrites de record preservan aristas (fix); `supersede` +2 inserts (cold path); `put`/`put_batch` +1 `engine.get` cache-hot por record (medición Regla 9 en Step 6); recall/search NO afectados (aristas no participan del índice vectorial/textual). Riesgo de concurrencia aceptado y documentado (get→insert no atómico entre SDK y `add_edge`, misma clase que REVIEW-13/supersede). |

## Impacto mapeado (Regla 0)

> Gate previo a la primera edición — poblado en DISCOVERY (2026-10-05, HEAD `b66c4703`).

- **Archivos leídos (completos o secciones funcionales íntegras):**
  - `src/sdk/serialization/mod.rs` (:77-83 `memory_node_id`; :511-619 `memory_record_to_node_owned` — construye `UnifiedNode::new(record.node_id)` **sin copiar edges** → wipe).
  - `src/sdk/api/memory.rs` (:200-274 `materialize_confidence` V1/V3 — parents deben existir; :343-439 `resolve_existing_for_write[_locked]`; :509-615 `put_one`; :617-656 `put`; :660-879 `put_batch_inner`; :1098-1153 `put_record_exact`; :1164-1243 `supersede`; :1300-1361 `reinforce`; :1373-1411 `quarantine_apply`; :1422-1455 `quarantine_promote`) — **7 sitios de rewrite** `memory_record_to_node_owned` + `engine.insert`/`batch_insert_with_opts`.
  - `src/sdk/api/graph.rs` (:20-162 `insert_node`/`get_node`/`add_edge`/`remove_edge`; :164-240 `collect_graph_nodes`/`restore_graph_nodes`).
  - `src/sdk/graph.rs` (:50-101 `graph_bfs`/`graph_bfs_filtered` — API pública existente para la query).
  - `src/sdk/serialization/graph_types.rs` (:9-26 `EdgeRecord` — expone `label: String`, `reverse`, `created_at_ms`; :56+ `NodeRecord.edges`).
  - `src/graph.rs` (:63-112 `bfs_traverse` — usa `node.edges`; :114-160 filtrado por label).
  - `src/storage/engine/insert.rs` (:152-225 `insert`; :234-284 `apply_insert_stats` — el storage YA hace `get(node.id)` para limpiar edge_index del nodo viejo y añadir el nuevo: la semántica de insert es **replace**, no merge); `src/storage/engine/get.rs` (:373-392 `get` — volatile cache primero).
  - `src/node/unified.rs` (:12-47 struct; :109-134 `label_index` helpers).
  - `src/sdk/types/record.rs` (:170-349 `MemoryInput`/`MemoryRecord` — `derived_from` :215/:310, `node_id` :269, `superseded_by` :283).
  - `vanta-memory/src/core/record/lifecycle.rs` (:56-118 `ContradictionProvenance`/`mark_contradiction` — record propio de vanta-memory, caller-persisted), `vanta-memory/src/core/scene/scene_index.rs` (:40-74 — usa `vantadb::sdk::Embedded` :49), `vanta-memory/src/core/persona/persona_generator.rs` (:107-110 `persona/<session>`).
  - `tests/memory_api.rs` (:1-90 — patrón de test de integración: tempdir + `Embedded::open`), `docs/api/EMBEDDED_SDK.md` (§Memory L91, §Node/Graph L374-386, §MemoryRecord L193).
  - `docs/dev/tasks/WIRE-18.md` (formato canónico de task file).
- **Archivos referenciados hacia dentro (imports/deps):** `memory.rs` importa `memory_node_id`, `memory_record_to_node_owned`, `record_from_node`, `now_ms`, `UnifiedNode`, `HashSet` (:13-27); `graph.rs` importa `now_ms`, `Error`, `Edge`; el test nuevo importará `vantadb::{Embedded, MemoryInput, ConfidenceClass, Value}` + `vantadb::graph::TraversalDirection` (público) + `tempfile`.
- **Referencias entrantes (grep HEAD):** `memory_record_to_node_owned` = 7 sitios en `memory.rs` + tests inline (serialization/mod.rs); `add_edge` = bindings/MCP/tests (firma intacta); `supersede` = bindings PyO3/WASM/Node + MCP `memory_supersede` + tests (firma intacta); `carry_graph_state`/`ensure_edge` = **0 hits** (nuevos, privado/`pub(crate)`); `memory_graph_lineage` = **0 hits** (test nuevo).
- **Veredicto impacto:** **MEDIO (correctness fix en write path + efectos aditivos)** — sin API pública nueva ni wire change; hot path `put` recibe +1 `get` cache-hot (medir, Regla 9). Pre-mortems del plan mitigados: (1) wipe confirmado por mecanismo (:511 sin edges) + test RED antes de tocar nada → primer fix del contrato; (2) doble fuente de verdad → regla declarada §Spec #4/#5 (campo = dato canónico, arista = navegabilidad derivada en la op); (3) scope creep → solo linaje L1 core; L2/L3 + contradicción vanta-memory + MCP `add_edge` → FINDs (stop L1447).

## Contrato

"L1–L3 como nodos/aristas reales: (a) aristas de linaje entre registros creadas en las ops correspondientes (supersede/derived_from) con labels canónicos fijados en DISCOVERY (`superseded_by`, `derived_from`); (b) query '¿quién cambió la fuente de esta decisión y por qué?' respondible vía BFS + provenance con test dedicado; (c) sin regresión de recall (suite `vanta-memory` verde) ni de la suite core scoped; (d) decisión verificada sobre edges en updates de record (**preservar** — fix del wipe confirmado con test RED→GREEN) + DX-04 documentado (puente `ns+key ↔ node_id` en `docs/api/`)."

## Spec (SDD — decisiones por evidencia)

> **Gate D evaluado (DISCOVERY): NO disparado** — la solución **no agrega símbolos públicos nuevos** (`carry_graph_state` privado, `ensure_edge` `pub(crate)`, query con `graph_bfs`/`get_node` existentes) y el contrato + pre-mortem + stop ya están fijados por el plan F0 (Gate Result ✅ DO, L1444-1447). Precedente idéntico: WIRE-18 ("Gate D pre-respondido por el plan F0"). Blast radius <10 archivos.

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Labels canónicos de linaje | A) **`superseded_by` (old→new) y `derived_from` (child→parent)** — espejo exacto de los campos del record (pro: trazabilidad campo↔arista, regla "campo = dato" auto-explicativa; contra: —) / B) verbos (`supersedes`/`derives`) (contra: divergen del campo, ambigüedad de dirección) | ✅ **A** — decidido-por-evidencia: convención existente lowercase-snake (`knows`, `belonged_to`, `referenced_by`, `belongs_to_thread` — executor.rs:484, builder.rs:412); los campos `superseded_by`/`derived_from` (record.rs:283/:310) son la fuente canónica |
| 2 | Fix del wipe de aristas en rewrites | A) **helper uniforme `carry_graph_state`** (`engine.get(node.id)` → copiar `edges`+`label_index`) aplicado a los 7 sitios de rewrite (pro: simple, cubre put/put_batch/cold ops; contra: +1 `get` cache-hot por write) / B) threading por `resolve_existing_for_write` (pro: sin get extra en hot; contra: cambia 3 firmas + 7 sitios, mayor superficie de bug) / C) merge en storage (pro: raíz; **contra: PROHIBIDO — dominio Arch/Engine**) | ✅ **A** — decidido-por-evidencia: mecanismo confirmado (insert=replace, insert.rs:619-624 serializa `node.edges` del nodo nuevo); `get` consulta volatile cache primero (get.rs:381) → nodo recién leído/escrito = cache-hit; **medición before/after en Step 6 (Regla 9)**; `ponytail:` upgrade path = B si la medición muestra regresión |
| 3 | Idempotencia de aristas de linaje | A) **`ensure_edge` chequea forward (`!reverse`) en source + reverse (`reverse`) en target; solo inserta lo que falta** (pro: re-put/re-run sin duplicar; tolera estado parcial; contra: 2 gets) / B) push ciego tipo `add_edge` (contra: duplica en cada re-put) | ✅ **A** — decidido-por-evidencia: `add_edge` no deduplica (graph.rs:117-136); el contrato exige re-idempotencia ("aristas re-idempotentes por op", plan L1446); `reverse` es load-bearing para dirección (EdgeRecord doc, graph_types.rs:17-20) |
| 4 | Reconciliación de `derived_from` al re-put | A) **sync en la op**: drop de forward-stale (`!reverse`, label `derived_from`, target ∉ declared) en el child + cleanup best-effort del reverse en el parent + `ensure_edge` de los declared (pro: campo = canónico, arista = derivada; contra: ~20 líneas) / B) ensure-only (contra: arista stale contradice la regla declarada del pre-mortem 2) | ✅ **A** — decidido-por-evidencia: pre-mortem 2 del plan (L1446) fija "campo = dato canónico, arista = navegabilidad derivada en la op"; test de reconciliación en Step 4 |
| 5 | Forma de la query de linaje (contrato b) | A) **test dedicado con `graph_bfs` (Both) + `get_node`/`get`** — API pública existente (pro: sin símbolos nuevos → Gate D no dispara; demuestra "quién cambió (superseder) + por qué (payload del nuevo)" end-to-end; contra: —) / B) nuevo método público `memory_lineage` (contra: símbolo público nuevo → Gate D; no pedido por el contrato) | ✅ **A** — decidido-por-evidencia: contrato (b) dice "respondible **vía BFS** + provenance" — `graph_bfs` (sdk/graph.rs:50) + `EdgeRecord.label` (graph_types.rs:14) ya exponen todo |
| 6 | L2/L3 (escena/persona) y contradicción vanta-memory | A) **FIND** (pro: stop condition L1447 lo permite explícitamente — "FIND del resto (escenas/persona como nodos enlazados)"; contra: —) / B) incluirlos en este run (contra: scope creep a linkage vanta-memory, budget) | ✅ **A** — decidido-por-evidencia: escenas/persona usan el SDK (`scene_index.rs:49`) pero su linkage (session→scene→persona) es un slice posterior; `mark_contradiction` es del record propio de vanta-memory (lifecycle.rs:96), no del core |
| 7 | MCP `add_edge` | A) **fuera de scope + nota FIND** (pro: contrato no lo pide; `graph_traverse` MCP ya expone BFS :3140-3206; las aristas de linaje las crean las ops internas; contra: —) / B) agregar tool MCP (contra: símbolo público nuevo → Gate D; no pedido) | ✅ **A** — decidido-por-evidencia: contrato (a)-(d) L1445 no menciona surface MCP; el gap del plan ("MCP solo expone remove_edge") es informativo, no parte del contrato |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. **Record = canónico, nodo = proyección** (ADR-046 §D2): los campos del record mandan; las aristas son estado del nodo que el rewrite NO puede destruir (fix #2) y que se derivan en la op (regla #4).
  2. **Cero cambios de wire:** `NodeMetadata.edges` ya serializa (insert.rs:619-624); ninguna arista se persiste fuera del nodo; sin migración.
  3. **API pública intacta:** `carry_graph_state` privado; `ensure_edge` `pub(crate)`; `supersede`/`put` conservan firma y semántica (aditivo: aristas de linaje).
  4. **No tocar** `src/wal.rs`, `src/vector/`, `src/storage/` (dominio Arch/Engine) — el fix vive en `src/sdk/`.
  5. **Sin `unwrap`/`expect`/`unsafe` en producción**; errores con `Result`; los tests usan `#![allow(clippy::expect_used, clippy::unwrap_used)]` (convención repo: memory_api.rs:1).
  6. **WIP ajeno** (`opencode.jsonc`) NO se stagea; PROHIBIDO tocar master plan / `docs/pipeline-state.json`; commit con **pathspec**.
  7. **Concurrencia:** el fix no agrega locks; la carrera get→insert del SDK (misma clase que REVIEW-13) queda documentada en el task file, no se promete atomicidad.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** ≤0 — sin `unsafe`, sin deps nuevas. El cambio **elimina** la deuda "memoria y grafo viven separados" (MEMG-03) + el wipe de aristas (riesgo #1 del plan) + cierra DX-04; agrega 1 `get` cache-hot por record write (medido; upgrade path documentado en `ponytail:`). `NOTICED BUT NOT TOUCHING`: linkage escena/persona → FIND; contradicción vanta-memory → FIND; MCP `add_edge` → FIND/nota; edges dangling tras delete → nota (comportamiento preexistente).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable: aristas `superseded_by`/`derived_from` creadas e idempotentes + reconciliación de stale + preservación de aristas en los 7 rewrites (RED→GREEN) + query de linaje (BFS + provenance) con test dedicado + suite `vanta-memory` verde + suite core scoped verde + fmt/clippy + DX-04 doc + review P2-01 por agente distinto |
| **Commit** | Commit atómico conventional `feat(memory):` + pathspec solo de archivos propios (sin WIP ajeno) + verificación mecánica (nunca auto-reporte); **LOCAL** (⛔ nunca push) |
| **Release** | changelog (minor) — `feat:` → release-plz bump minor; doc `docs/api/EMBEDDED_SDK.md` en el mismo commit (Regla 3) |

## Herramientas necesarias

- `codegraph_codegraph_explore` (blast radius: `memory_record_to_node_owned`/`add_edge`/`bfs_traverse`) + `codebase-memory-mcp_check_index_coverage` (paths clave: `no_recorded_issue` ✅, freshness `metadata_changed` → releer fuente)
- `cargo nextest` scoped por crate (`-p vantadb --test memory_graph_lineage`, `-p vantadb` suite) + `-p vanta-memory` suite + `campaign_verify_cmd` (verify mecánico)
- `pwsh dev-tools/ocr-review.ps1 -Format json` (OCR delegation al cierre)
- Gates docs (`node scripts/docs/check-links.mjs && check-docs.mjs && gen-index.mjs`) — task file + `docs/api/EMBEDDED_SDK.md` editados

**Skills cargadas (SDP v3):** `campaign-executor` · `progreso` · `ponytail` (base auto) · `source-driven-development` · `doubt-driven-development` · `incremental-implementation` · `test-driven-development` · `context-engineering` (SDP phase=BUILD) + rol: `rust-write-tests` (tests de preservación/linaje), `api-and-interface-design` (contrato campo↔arista), `documentation-skill` (task file + docs/api bajo `docs/`). `security-and-hardening` evaluada (ver Fase SECURITY); `performance-optimization` evaluada (ver Fase PERFORMANCE).

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — evaluación: sin trust boundary nuevo. Los rewrites ya validan namespace/key/metadata/confidence; las aristas usan ids internos derivados (no input libre: labels fijos `superseded_by`/`derived_from`; targets = `memory_node_id` de parents ya validados como existentes). Sin red, sin secrets, sin deps. Sin hallazgos que requieran `security-and-hardening`.
- [ ] **PERFORMANCE** — dispara (write path `put`): +1 `engine.get` cache-hot por record en 7 sitios; `supersede` +2 inserts. Medición before/after con timing simple (10k puts, mismo dataset, misma máquina) en Step 6; sin regresión o regresión documentada con upgrade path (`ponytail:` → threading por resolve). Sin benchmark Criterion obligatorio (no hay claim de optimización; es correctness fix — Regla 9 aplica a "optimizar", la medición acá documenta el costo).

## Steps

### Step 1 — RED: test de preservación de aristas en rewrites (put + supersede)

- **Archivos:** `tests/memory_graph_lineage.rs` (nuevo)
- **Acción:** test de integración con `tempdir` + `Embedded::open`: (1) `record_update_preserves_existing_edges`: put A + put B → `add_edge(A.node_id, B.node_id, "related")` → re-put A (payload nuevo) → `get_node(A.node_id).edges` DEBE contener B (hoy: wipe). (2) `supersede_preserves_existing_edges`: put A + put C + add_edge(A,C,"related") → `supersede(ns,"a","c")` → `get_node(A.node_id)` conserva la arista + marca `superseded_by`.
- **Verify:** `cargo nextest run --profile audit -p vantadb --test memory_graph_lineage --build-jobs 2` → **RED esperado** (edges vacías en A tras el rewrite) → confirma el mecanismo leído.
- **Evidencia:** ✅ **RED EJECUTADO** — `record_update_preserves_existing_edges`: `left: 0, right: 1` ("record update must not wipe the node's edges"); `supersede_preserves_existing_edges`: `pre-existing edges must survive the supersede rewrite: []`. El wipe del plan (riesgo #1) es **REAL** (mecanismo + test ejecutado).
- **Estado:** ✅ COMPLETED

### Step 2 — GREEN: `carry_graph_state` en los 7 sitios de rewrite

- **Archivos:** `src/sdk/api/memory.rs`
- **Acción:** helper privado `carry_graph_state(engine, node)` (copia `edges`+`label_index` del nodo existente si hay) aplicado en `put_one`, `put_batch_inner` (por nodo del chunk, antes de `batch_insert_with_opts`), `put_record_exact`, `supersede`, `reinforce`, `quarantine_apply`, `quarantine_promote`. `ponytail:` note (get extra cache-hot; upgrade path = threading por resolve).
- **Verify:** `cargo nextest run --profile audit -p vantadb --test memory_graph_lineage --build-jobs 2` → Step 1 GREEN. + `cargo check -p vantadb`.
- **Evidencia:** ✅ GREEN 2/2 (1.58s). +7 sitios cubiertos (grep: `carry_graph_state` en :93 def + 7 call sites). Optimización posterior: skip cuando `resolve_*` probó que no hay nodo existente (fresh insert) — el costo queda en 1 `get` cache-hot por *update*.
- **Estado:** ✅ COMPLETED

### Step 3 — `ensure_edge` + arista `superseded_by` en `supersede`

- **Archivos:** `src/sdk/api/graph.rs` (`ensure_edge` `pub(crate)` idempotente + consts `LABEL_SUPERSEDED_BY`/`LABEL_DERIVED_FROM`), `src/sdk/api/memory.rs` (`supersede` la invoca post-insert), `tests/memory_graph_lineage.rs`
- **Acción:** `ensure_edge(source,target,label)` → forward `!reverse` en source + reverse `reverse` en target; solo inserta lo faltante; retorna si creó algo. `supersede` → `ensure_edge(old, new, "superseded_by")`. Tests: `supersede_creates_superseded_by_edge` (label visible en `get_node(old).edges`, `reverse` correcto, BFS Both desde old encuentra new) + idempotencia.
- **Verify:** `cargo nextest run --profile audit -p vantadb --test memory_graph_lineage --build-jobs 2` → GREEN.
- **Evidencia:** ✅ RED primero (5 tests de linaje fallando por edge inexistente) → GREEN 7/7 tras implementar. `supersede_creates_superseded_by_edge` verde (forward/reverse/timestamp/BFS).
- **Estado:** ✅ COMPLETED

### Step 4 — Aristas `derived_from` en put/put_batch (+ reconciliación + idempotencia)

- **Archivos:** `src/sdk/api/memory.rs`, `tests/memory_graph_lineage.rs`
- **Acción:** helper de sync (drop forward-stale + collect stale targets; post-insert: cleanup best-effort del reverse en parents stale + `ensure_edge(child, parent, "derived_from")` por cada parent declarado) aplicado en `put_one`, `put_batch_inner` (con last-occurrence para duplicados in-batch) y `put_record_exact` (best-effort: el import no valida existencia de parents). Tests: creación, idempotencia, reconciliación.
- **Verify:** `cargo nextest run --profile audit -p vantadb --test memory_graph_lineage --build-jobs 2` → GREEN.
- **Evidencia:** ✅ GREEN 9/9 (incluye `derived_put_creates_derived_from_edge`, `repeated_derived_put_does_not_duplicate_edges`, `changing_derived_parents_drops_stale_edge`, `put_batch_derived_creates_edges_and_preserves_existing`, `reinforce_preserves_existing_edges`).
- **Estado:** ✅ COMPLETED

### Step 5 — Query de linaje: "¿quién cambió la fuente y por qué?" (BFS + provenance)

- **Archivos:** `tests/memory_graph_lineage.rs`
- **Acción:** test `lineage_query_answers_who_changed_source_and_why`: put A ("decision v1") + put C ("decision v2") + `supersede(a→c)` + put B derivado de C → `graph_bfs(&[A.node_id], 2, Both)` DEBE encontrar C (superseder) y B (derivación de la nueva fuente); `get_node(A)` expone arista `superseded_by` con `created_at_ms`; `get(ns,"c").payload` = el "por qué". Asertar provenance completa (label + dirección + timestamp > 0).
- **Verify:** `cargo nextest run --profile audit -p vantadb --test memory_graph_lineage --build-jobs 2` → GREEN.
- **Evidencia:** ✅ GREEN — BFS desde el node del record original alcanza superseder (depth 1) y derivación de la nueva fuente (depth 2); provenance (label+timestamp) y payload del superseder verificados. Sin API pública nueva (Gate D no dispara).
- **Estado:** ✅ COMPLETED

### Step 6 — DX-04 doc + medición Regla 9 + gates docs

- **Archivos:** `docs/api/EMBEDDED_SDK.md` (subsección "Memory ↔ graph bridge (DX-04)"), `docs/dev/tasks/MEMG-03.md`
- **Acción:** documentar el puente `ns+key ↔ node_id` + medición timing simple 10k puts before/after (script temporal, no se commitea) + gates docs.
- **Verify:** gates docs verdes + medición registrada (sin regresión o regresión documentada).
- **Evidencia:** ✅ Doc DX-04 (43 líneas: bridge determinístico, labels de linaje, tabla de ops, ejemplo BFS "who changed & why", preservación en rewrites). ✅ Gates docs: check-links `0 (budget 0)` · check-docs exit 0 · gen-index `--check` exit 0. ✅ **Medición Regla 9** (InMemory, n=5000 fresh + 5000 updates, misma máquina, entorno compartido con MEMG-06): before=38.8s · after(1ª)=47.8s · after(2ª)=36.0s (varianza run-to-run ±33% con MISMO código → señal noise-dominated) · after+skip-fresh=**32.8s**. Conclusión: sin regresión medible; costo estructural final = 0 gets extra en fresh inserts, 1 `get` cache-hot por update (nodos con aristas son raros); upgrade path documentado en `ponytail:` (`carry_graph_state`). Instrumento temporal borrado.
- **Estado:** ✅ COMPLETED

### Step 7 — Verificación de cierre + review P2-01 + commit + campaign

- **Archivos:** todos los tocados
- **Acción:** `campaign_verify_cmd`: fmt + clippy + nextest scoped (`-p vantadb`, `-p vanta-memory`) + OCR delegation + FINDs (L2/L3, contradicción, MCP) + review P2-01 (`vanta-review` fork o degradado) + commit LOCAL `feat(memory):` con pathspec + `campaign_update_task_state(completed, taskId "50")` + `skill progreso`.
- **Verify:** contrato completo verde; RESULTADO §7.
- **Evidencia:** ✅ fmt+clippy scoped limpios · scoped core 2358 (2357+1 timeout de carga documentado) · vanta-memory 684/684 · gates docs ✅ · OCR Rules ✅ · **review P2-01 APPROVE (ronda 3)** · commit LOCAL `feat(memory):` con pathspec (5 archivos propios, sin WIP ajeno) · campaign `50` completed.
- **Estado:** ✅ COMPLETED

## Verificación

- `cargo fmt --check` — ✅ (`cargo fmt -p vantadb -- --check` exit 0; fix aplicado antes)
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — ✅ scoped `cargo clippy -p vantadb --all-targets -- -D warnings` (default features) **Finished sin warnings**. (Workspace-wide diferido: el árbol compartido tiene WIP de MEMG-06 en `vanta-memory`/`vantadb-mcp`/`desktop`; el gate de integración lo corre el lead.) Nota: clippy con `--no-default-features` (sin `cli`) NO es limpio por deuda pre-existente (dead-code de módulos gated) — se usó default features.
- `cargo nextest run --profile audit -p vantadb --test memory_graph_lineage --build-jobs 2` — ✅ **12/12 passed** (3.79s; +3 tests post-review H1/M1/F1)
- `cargo nextest run --profile audit -p vantadb --build-jobs 2 --lib --test memory_graph_lineage --test memory_api --test memory_export_import --test memory_telemetry --test quarantine_containment --test sdk_serialization --test derived_indexes --test version_coherence` — ✅ **2358 tests: 2357 passed + 1 timeout de carga** (`index::core::tests::concurrent_insert_preserves_hnsw_invariants`, HNSW concurrency — **pasa aislado en 47s**; timeout de 180s del perfil audit bajo carga concurrente; no relacionado al cambio). Re-corrido post-fixes de review (2358 incluye los 3 tests nuevos). Suite completa `-p vantadb` cli-off: el build de TODOS los test targets agotó page-file (OOM/LNK1171, entorno); scope curado + lib = cobertura del surface tocado.
- `cargo nextest run --profile audit -p vanta-memory --build-jobs 2` — ✅ **684/684 passed** (2 skipped) — sin regresión de recall. Re-corrido post-fixes de review.
- Gates docs — ✅ check-links `0 (budget 0)` · check-docs exit 0 · gen-index `--check` exit 0
- OCR delegation (`pwsh dev-tools/ocr-review.ps1 -Format json`) — ✅ Rule Groups extraídos (group 1 project `src/sdk/**` → memory.rs/graph.rs; group 2 system `**/*.rs` → test); self-review: 0 Critical/High; los `let _ =` de cleanup son best-effort documentados (convención `version_history`); sin unwrap/unsafe nuevos; carrera check-then-act de edges = clase pre-existente de `add_edge` (documentada).
- Review P2-01 por agente distinto — ✅ **APPROVE (ronda 3)** — ver §Review P2-01 abajo (ronda 1: CHANGES-REQUIRED H1+M1 → fixes; ronda 2: CHANGES-REQUIRED F1 → fix; ronda 3: APPROVE).

## Review P2-01 (ronda 1 — vanta-review, agente distinto) — respuesta del autor

**H1 (High) — arista `superseded_by` stale al re-putear un record superseded. FIX APLICADO.** `put_one` reconstruye el record con `superseded_by: None` (revive el record) pero `carry_graph_state` preservaba la arista vieja → campo y arista divergían, y un `supersede` posterior (guard pasa porque el campo volvió a None) dejaba DOS aristas contradictorias. Fix: `reconcile_derived_from_edges` generalizado a `reconcile_lineage_edges` (reconcilia `derived_from` **y** `superseded_by`: drop de forward halves cuyo target ∉ declarado; `None` ⇒ drop de todas) aplicado en `put_one`, `put_batch`, `put_record_exact` y `supersede`; cleanup counterpart best-effort. Test: `reput_of_superseded_record_drops_stale_superseded_by_edge` (re-put → sin arista ni reverse; re-supersede → exactamente 1).

**M1 (Medium) — doc DX-04 sobre-clamaba "import" preserva aristas (bulk import = 8º path de wipe). FIX APLICADO.** `bulk_import_stream` construye el nodo a mano y `engine.insert` reemplazaba → wipe. Fix: `carry_graph_state` antes del insert por record (+1 `get` por record en el path bulk, que ya hace lecturas por record para quarantine). Test: `bulk_import_overwrite_preserves_existing_edges` (header VDBJSON + overwrite de key existente → arista sobrevive). El claim del doc queda **verdadero** para ambos paths de import (JSONL vía `put_record_exact` + bulk).

**L1 (Low) — `ensure_edge` no actualiza `label_index`:** mismo patrón pre-existente de `add_edge` (`graph.rs:117-136`); `graph_bfs_filtered` escanea `edges` cuando `label_index` está vacío. Aceptado (paridad con `add_edge`; unificar en tarea aparte si el filtered-BFS con label_index se usa en producción).
**L2 (Low) — self-edge vía `ensure_edge` (solo import raw con `derived_from` propio):** degenerado (V3 lo bloquea en `put`); segunda mitad pisa la primera. Documentado, no bloquea.
**L3 (Low) — `remove_edge` borra ambas direcciones del label; en derivación mutua (solo import raw) cortaría la mitad legítima del otro record:** degenerado (V3 bloquea mutua en `put`); documentado.

**Riesgo aceptado (reviewer):** carrera get→insert de `ensure_edge` (clase REVIEW-13, ya documentada) y `ensure_edge` post-insert con `?` (patrón pre-existente de `replace_derived_indexes`).

**F1 (High, ronda 2) — bulk import preservaba lineage edges sin reconciliar. FIX APLICADO.** Tras el fix M1, `bulk_import_stream` copiaba TODAS las aristas (carry) pero el nodo bulk no setea `__vanta_derived_from`/`__vanta_superseded_by` (record resultante con campos None) → aristas stale divergentes (misma regla del pre-mortem 2). Fix: `reconcile_lineage_edges(&engine, &mut node, &input.namespace, &[], None)` post-carry + cleanup counterpart post-insert (el bulk declara lineage vacío — derived se rechaza arriba). Test: `bulk_import_overwrite_drops_stale_lineage_edges` (supersede → bulk overwrite → campo None, sin forward ni reverse).

**Veredictos:** ronda 1 `vanta-review` (agente distinto): CHANGES-REQUIRED (H1 High + M1 Medium) → ronda 2: CHANGES-REQUIRED (F1 High nuevo) → ronda 3: **APPROVE** (F1 verificado por lectura: orden carry→reconcile→insert→cleanup; retain preserva reverse halves y otros labels; tests discriminan; 12/12 re-ejecutado por el reviewer). Sin hallazgos abiertos.

## Notas de entorno (2026-10-05)

- **Bin lock:** `target/debug/vanta-cli.exe` está lockeado por el MCP server de VantaDB en ejecución (`vanta-mcp-local.ps1` → `target/debug/vanta-cli.exe server --mcp --db ~/.vantadb`). Cualquier build que incluya el bin falla ("Acceso denegado" al reemplazarlo). Workaround usado: `--no-default-features` sin `cli` (el bin tiene `required-features = ["cli"]` → no se construye) para los comandos de test; clippy/fmt no linkean. **No se mató el proceso** (es el server MCP de la sesión).
- **Disco:** C: estaba al 100% (0 GB libres); se borraron artefactos regenerables stale de `target/` (semver-checks, dur03, find237, x86_64-pc-windows-msvc, wasm32-unknown-unknown ≈ 21.6 GB).
- **Árbol compartido con MEMG-06** (misma working tree): sus archivos (`vanta-memory/src/context_engine/**`, `offload/reclaimer.rs`, `pipeline_worker.rs`, `vantadb-mcp/src/context.rs`, `desktop/...`, `docs/dev/Backlog.md`, `docs/index.md`, `llms.txt`) NO se tocan ni se stagean. FINDs propuestas abajo quedan pendientes de fold (Backlog.md tiene filas sin commitear de MEMG-06 — no se puede stagear sin mezclar).
- **Page-file:** builds masivos paralelos agotan memoria (OOM + LNK1171 "archivo de paginación demasiado pequeño"); usar `--build-jobs 2` y scopes curados.

## FINDs propuestas (fold pendiente — Backlog.md en vuelo por MEMG-06)

- **FIND-296 (propuesta) · 🟢 Baja · L2/L3 (escena/persona) como nodos enlazados sin linkage:** `vanta-memory/src/core/scene/scene_index.rs:40` + `persona_generator.rs:108` (ambos usan `vantadb::sdk::Embedded` — los records L2/L3 ya SON nodos) — falta el linkage session→scene→persona con aristas de linaje (stop condition del plan L1447). Origen: MEMG-03 (2026-10-05) · stop L1447.
- **FIND-297 (propuesta) · 🟢 Baja · Contradicción vanta-memory sin arista:** `vanta-memory/src/core/record/lifecycle.rs:96` (`mark_contradiction` setea `superseded_by` en el record propio de vanta-memory, caller-persisted) — no crea arista `superseded_by` en el grafo del engine (el core sí lo hace vía `supersede`). Origen: MEMG-03 · contrato (a).
- **Nota (no FIND) · MCP `add_edge`:** el contrato no lo pide; `graph_traverse` MCP ya expone BFS; las aristas de linaje las crean las ops internas. Si se quiere surface manual, es una tarea MCP nueva (Gate D).

## Resultado §7

RESULTADO: ✅ COMPLETO
STEPS_OK: 7/7
PROXIMO_STEP: ninguno
COMMIT_HASH: e41fef7d
ARCHIVOS: `src/sdk/api/memory.rs` · `src/sdk/api/graph.rs` · `tests/memory_graph_lineage.rs` · `docs/api/EMBEDDED_SDK.md` · `docs/dev/tasks/MEMG-03.md`
VERIFY_CONTRATO: **pasa** — 12/12 `memory_graph_lineage`; scoped core 2357/2358 (1 timeout de carga HNSW, pasa aislado en 47s — no relacionado); `vanta-memory` 684/684; fmt + clippy scoped limpios; gates docs ✅; review P2-01 APPROVE (ronda 3)
BLOQUEO: ninguno
GATES_EVALUADOS: P:no D:no V:no C:no | P/D: plan F0 pre-responde, sin símbolos públicos nuevos · V: sin fallas repetidas · C: sin colaterales
SKILLS_CARGADAS: campaign-executor · progreso · ponytail (base auto) · source-driven-development · doubt-driven-development · incremental-implementation · test-driven-development · context-engineering (SDP BUILD) + rust-write-tests · api-and-interface-design · documentation-skill

**Notas:** commit LOCAL (⛔ sin push). FIND-296/297 propuestas + nota MCP quedan pendientes de fold en Backlog (archivo con cambios ajenos sin commitear al momento del cierre). Cierre campaign taskId `50`. Review P2-01: 3 rondas vanta-review (H1/M1 → F1 → APPROVE).
