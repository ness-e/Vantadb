---
title: "ADR-019: Formato persistido del sparse vector sin full JSON"
type: adr
status: accepted
tags: [vantadb, architecture, adr, storage, serialization, performance]
created: 2026-08-12
last_reviewed: 2026-08-12
---

# ADR-019: Formato persistido del sparse vector sin full JSON

## Context

El sparse vector de un memory record (`SparseVector(pub BTreeMap<u32, f32>)`,
`src/node/vector_data.rs:30`) se persiste como `FieldValue::String(serde_json)`
bajo el key relacional `SPARSE_VECTOR_EXT_KEY` (`__vanta_sparse_vector`,
`src/sdk/serialization/mod.rs:30`):

- **Write:** `memory_record_to_node_owned` (mod.rs:369-373) hace
  `serde_json::to_string(sparse)` → `FieldValue::String(json)` →
  `node.set_field(...)`.
- **Read:** `memory_record_from_node` (mod.rs:296-313) hace
  `serde_json::from_str(json)`.

El parse de JSON representa ~1.49% del hot-path de búsqueda (AUDIT-02,
2026-08-06). PERF-07 ya evita el parse cuando la key falta; PERF-08 cerró el
lado WASM (`memory_record_to_js` → Float32Array). La deuda restante es el
parse/stringify por hit del sparse en el core persistido (P2-7).

**Invariantes (handoff):** (1) `VantaMemoryRecord.sparse_vector` público no
cambia; (2) compat de lectura del formato viejo garantizada (degrade a `None`
solo si corrupto, con `tracing::warn`); (3) `SPARSE_VECTOR_EXT_KEY` sigue
siendo el key relacional; (4) cero cambio de semántica/recall; (5) no tocar
WASM/`memory_record_to_js`.

## Decision

Persistir el sparse como **`FieldValue::ListFloat(Vec<f64>)` con pares
intercalados** `[dim_0, val_0, dim_1, val_1, ...]` bajo el MISMO key
`SPARSE_VECTOR_EXT_KEY`.

- **Write:** iterar `sparse.0` (BTreeMap, orden determinista) y aplanar cada
  `(dim, val)` a `[dim as f64, val as f64]` → `FieldValue::ListFloat`. Cero
  serde_json en el write path.
- **Read:** si el field es `ListFloat` → reconstruir `SparseVector` iterando
  los pares (`dim as u32`, `val as f32`). Si es `String` (formato viejo) →
  `serde_json::from_str` (compat backward). Si falta la key → `None` (PERF-07).
  Si está corrupto (ListFloat de largo impar, o JSON inválido) → `tracing::warn`
  + `None` (comportamiento PERF-07).

**Por qué `ListFloat` (no otras opciones):**

| Candidato | Veredicto |
|---|---|
| `ListFloat` con pares intercalados | ✅ Reutiliza variante existente de `FieldValue` (sin cambio de enum público, sin cambio de índices bincode), un solo key, cero deps, round-trip exacto |
| Claves por componente (ej. `ListInt` dims + `ListFloat` vals en keys separados) | ❌ Rompe el invariante "`SPARSE_VECTOR_EXT_KEY` sigue siendo el key relacional" y agrega keys internos |
| Variante nueva de `FieldValue` (ej. `Bytes(Vec<u8>)`) | ❌ Cambia enum público + índices bincode (rompe compat de TODOS los nodos), requiere migración global; api-contract R-6 (enum en crecimiento) |
| Campo nuevo en `UnifiedNode` | ❌ Cambia formato bincode del grafo, blast radius alto, innecesario: el key relacional ya survives KV round-trip |

**Exactitud numérica:** u32 (dims) es exacto en f64 (max 4.29e9 < 2^53); f32
(vals) → f64 es lossless (todo f32 es representable exactamente en f64);
f64→f32 devuelve el mismo bit pattern. El round-trip es idéntico al valor
original. El orden de `BTreeMap<u32, f32>` es determinista y monótono → el
flatten preserva el orden de `dot()` (merge lineal).

**Costo:** 16 bytes/entry (2×f64) vs ~12-18 bytes/entry del JSON — comparable;
el beneficio es eliminar parse + alloc del string JSON del hot path.

## Consequences

- **Pros:** elimina `serde_json::from_str` y `serde_json::to_string` del
  hot path del sparse por hit (AUDIT-02 1.49% → ~0); cero deps nuevas; cero
  `unwrap()`; round-trip exacto; key relacional único intacto; API pública
  intacta.
- **Cons:**
  - El field `__vanta_sparse_vector` en `cardinality_stats`/`scalar_index`
    (`ops.rs:725-751`) cambia su forma interna (String → ListFloat). No es
    visible al usuario: los keys `__vanta_*` se rechazan en input
    (`validate_metadata`, mod.rs:110) y se remueven del metadata expuesto
    (mod.rs:274).
  - `get_node`/`unified_to_record` (graph export, `graph_types.rs:93-97`)
    expone `relational` crudo: el valor del key `__vanta_sparse_vector` pasa de
    `VantaValue::String(json)` a `VantaValue::ListFloat(pares)`. Es un key
    reservado interno (rechazado en input, sin contrato público), pero
    consumidores que parseaban el JSON de ese key en graph export deberían
    migrar a la nueva forma.
  - Edge case NaN/Inf: serde_json rechazaba NaN (`to_string` falla → el key no
    se escribía → read devolvía `None`). Con `ListFloat`, NaN/Inf round-trip
    (bincode los acepta) → read devuelve `Some(map con NaN)`. Recall idéntico
    (los postings del sparse index se construyen desde el record en memoria,
    no desde el nodo persistido; el score NaN es igual); solo cambia el
    contenido de `sparse_vector` en el record devuelto para inputs degenerados.
    Aceptado.
- **Migración:** **no se requiere script one-shot.** El read path es dual
  (String + ListFloat) por diseño, así los nodos viejos se leen igual
  indefinidamente. Los nodos existentes migran al formato nuevo de forma
  perezosa en el próximo `put`/update (write path reescribe en nuevo formato).
  El shim de lectura del formato viejo (un match arm + `serde_json::from_str`)
  se mantiene hasta que exista un gate de versionado de storage (hoy no existe
  ninguno); no se deja escritura dual. Drop del shim viejo: diferido a cuando
  se introduzca versionado de formato de storage (fuera de alcance de P2-7).

## Referencias

- `docs/Backlog.md` § Phase 4 — P2-7
- AUDIT-02 (2026-08-06) — parse 1.49% del hot path
- PERF-07 (skip serde_json cuando falta key), PERF-08 (WASM, cerrado)
- ADR-011 (sparse vectors native) — decisión original de representación
