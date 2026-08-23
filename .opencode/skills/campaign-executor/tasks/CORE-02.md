# CORE-02 — Bug IQL transporte WASM: graph-store vacío en modo standalone

## Estado: ⏳ IN PROGRESS

- **Prioridad:** 🟠 · **Esfuerzo:** 🟡 · **Appetite:** max 1d · **Cynefin:** 🟧 complejo
- **Plan:** docs/plans/2026-08-23-backlog-triage.md (Task 6)
- **Contrato mecánico:** test wasm roundtrip — insert edge (RELATE/add_edge) → query IQL (FROM) devuelve el edge, en el engine WASM (test cfg(target_arch="wasm32") vía wasm-pack --node o test capa bindgen equivalente).

## Root Cause Analysis (systematic-debugging Phase 1-3 completado)

### Hipótesis evaluadas (en orden)

| Hip | Veredicto | Evidencia |
|-----|-----------|-----------|
| **H1** graph-store no persistido/restaurado desde OPFS | ✅ **CONFIRMADA (root cause)** | `persist_payload()` (vantadb-wasm/src/lib.rs:661) serializa SOLO `Vec<VantaMemoryRecord>` vía `collect_all_deduped()` (list_namespaces+list). Nodos de grafo (insert_node/add_edge/IQL RELATE) NO tienen FIELD_NAMESPACE → invisibles al export → nunca llegan a `db_state.json`. En reopen, `load()` importa solo records de memoria → grafo vacío. |
| **H2** in-memory por diseño, cada query abre instancia nueva | ❌ Rechazada como "by design" | Persistencia SÍ existe para memory records (differential-persist PERF-08); el gap es cobertura del snapshot, no ausencia de diseño. Fix = extender snapshot (listado explícito en Regla 2 como fix acotado aceptable). NO requiere rediseñar OPFS. |
| **H3** dos instancias de engine | ❌ Rechazada | `WasmBackend.db()` cachea un solo `dbPromise` (desktop/src/transport.ts:86); binding usa un único `inner: VantaEmbedded`. |

### Hallazgo colateral (stale comment)

El comentario de `vanta_query: unsupported` en `desktop/src/vanta-wasm-map.ts:167-174`
("`SELECT * FROM ns` devuelve vacío con records presentes") fue escrito 2026-08-20
(`901a1c51`), ANTES de MCP-29 (`55fb1ccc`, 2026-08-23) que hizo los namespaces
visibles al Scan IQL (`src/physical_plan/scan.rs:81-92`). La parte "memory records
invisibles a FROM" está OBSOLETA in-session; lo que sigue roto es la PERSISTENCIA
del grafo (H1). El desbloqueo de `vanta_query` en el map es trabajo de UI/wire
separado — NO en este scope.

### Mecánica verificada

- Edges viven EN los nodos: RELATE → `node.add_edge` + `storage.insert` (src/executor.rs:302-343); add_edge SDK agrega forward+reverse (src/sdk/api.rs:1175-1214).
- `edge.reverse` es load-bearing para traversal direccional (src/graph.rs:35-36) → el formato de persistencia debe preservarlo (VantaEdgeRecord hoy lo pierde).
- Labels se internan String↔u32 por-engine (`label_intern`) → al restaurar hay que re-internar (export resuelve label a String vía `unified_to_record`; import re-internará).
- `From<VantaValue> for FieldValue` existe (conversions.rs:80) → restaurar fields es directo.
- WASM fuerza `BackendKind::InMemory` (build_config, lib.rs:72); tests nativos MCP-29 usan el mismo backend y pasan → in-session OK.
- Infra test disponible: wasm-pack 0.15.0 + node v24 + target wasm32 instalado → `wasm-pack test --node` corre tests bindgen headless (OPFS/IDB se auto-skip).

## Impacto mapeado (Regla 0)

**Archivos leídos completos:** vantadb-wasm/src/lib.rs (init/save/load/persist_payload/query/insert_node/get_node), desktop/src/vanta-wasm-map.ts (completo), src/sdk/api.rs (insert_node/add_edge/query), src/executor.rs (execute_statement/Relate), src/physical_plan/scan.rs (completo), src/storage/engine/mod.rs (intern_label/node_to_record), src/engine.rs (InMemoryEngine), src/sdk/serialization/graph_types.rs (completo), vantadb-wasm/tests/wasm_tests.rs (completo), desktop/src/transport.ts (completo), src/parser/mod.rs (parse_relate).

**Referencias hacia dentro (lo que toco):**
- `VantaEdgeRecord` (graph_types.rs:9): usado por JsNodeRecord→JS wire (lib.rs:148-165), desktop DTOs leen `.edges`.
- `unified_to_record`: callers = node_to_record (4 callers: sdk/builder, sdk/api, mcp tools, storage mod).
- `save/load/save_idb/load_idb/connect_persistent/connect_idb/connect_worker` (wasm lib.rs): únicos consumers del snapshot.
- Tests existentes que construyen `VantaEdgeRecord{}` literal: graph_types.rs tests (2 literals a actualizar).

**Referencias entrantes (qué depende):**
- `vantadb-ts` y `web/` consumen el JS wire de JsNodeRecord.edges ({target,label,weight}) → campos NUEVOS son aditivos (no rompen consumidores existentes; serde default mantiene JSON viejo parseable).
- Desktop standalone usa `connect_persistent`/`save()` (transport.ts:91-106) — formato nuevo archivo separado `graph_state.json` = cero migración de `db_state.json`.

**Veredicto:** cambio acotado — 1 tipo SDK extendido aditivamente, 2 funciones core nuevas privadas+test, wiring snapshot en binding wasm. Sin cambios de API pública breaking. Blast radius < 10 archivos.

## Spec (mecánica)

1. **Core (src/sdk/serialization/graph_types.rs):** `VantaEdgeRecord` += `#[serde(default)] reverse: bool`, `#[serde(default)] created_at_ms: u64`; `unified_to_record` los llena desde Edge. Actualizar 2 test literals + test backward-compat (JSON viejo sin campos nuevos deserializea).
2. **Core (src/sdk/api.rs):** `pub(crate) fn collect_graph_nodes()` — scan(Default)→get_many→filter vivos sin FIELD_NAMESPACE→node_to_record. `pub fn restore_graph_nodes(Vec<VantaNodeRecord>) -> Result<usize>` — reconstruye UnifiedNode (fields/vector/tier/telemetry/edges re-internados) + engine.insert.
3. **WASM (vantadb-wasm/src/lib.rs):** `graph_payload()` serializa collect_graph_nodes a JSON; `save()`/`save_idb()` escriben `graph_state.json` SIEMPRE (maneja delete-all); `load()`/`load_idb()`/`connect_worker` lo restauran si existe. ponytail: rewrite completo por save (sin differential para grafo), diferir hasta medirlo caliente.
4. **Tests:**
   - Core: roundtrip collect→nuevo engine→restore→query IQL ve nodos+edges (labels/reverse/weights fieles); filtro excluye memory records; serde back-compat.
   - Wasm (cfg target_arch=wasm32, unit test interno accediendo helpers privados): roundtrip completo binding-level: insert_node×2 + add_edge + query("SELECT * FROM *") in-session + graph_payload→restore en segunda instancia→query devuelve edges. Corre con `wasm-pack test --node`.

## Stop conditions

Si restaurar fielmente requiriera rediseño del formato OPFS o tocar wal/storage → ABORTAR con diagnóstico. NO aplica: el diseño elegido es archivo lateral aditivo.

## Steps

- ✅ Step 1: VantaEdgeRecord + unified_to_record (reverse/created_at_ms) + tests serde — 9/9 graph_types PASS
- ✅ Step 2: collect_graph_nodes + restore_graph_nodes + tests nativos core02 — 2/2 PASS
- ✅ Step 3: wiring wasm (graph_payload/save/load opfs+idb+worker) + test wasm --node — `core02_graph_persist_tests::graph_roundtrip_through_snapshot_payload ... ok` (contrato mecánico CUMPLIDO)
- ⬜ Step 4: verify full + commit

## Hallazgos colaterales (candidatos FIND-* Backlog)

1. **FIND-CORE02a:** los unit tests de lib.rs de vantadb-wasm fallan bajo `wasm-pack
   test --node` (11/18, "missing field namespace") — pre-existentes en develop
   verificado por stash-run (7 pass/11 fail sin cambios). Escritos para browser;
   serde_wasm_bindgen no deserializa los objetos JS de make_put igual que
   serde_json. Requieren browser runner o fix de formato.
2. **FIND-CORE02b:** `VantaFields` (HashMap<String,VantaValue>) es difícil de
   construir desde JS puro: enums externally-tagged requieren {"String": "..."} y
   serde_wasm_bindgen tiene gaps ("invalid length 0"). El map desktop nunca pasa
   fields a insert_node hoy. Candidato: helper JS-side o untagged+helper.

## Context Save Point

Contrato wasm verde ANTES de verify full. No tocar más código salvo fallos de verify.
