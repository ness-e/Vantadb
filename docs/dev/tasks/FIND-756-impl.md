---
title: "FIND-756-impl: contrato ejecutable — proyectar los extremos del linaje a memory_node_id y crear la arista tras el put"
kind: task
status: active
description: "Implementación de ADR-0066/FIND-756: 6 sitios, 6 trampas, 13 aserciones sobre el grafo, cero cambios en src/. Todo lo que el implementador necesita para no decidir nada."
tags: [vantadb, vanta-memory, graph, lineage, contract-first, executable]
---

# FIND-756-impl — el contrato, en pasos ejecutables

Decision: [ADR-0066](../architecture/adr/ADR-0066-lineage-endpoints-are-memory-record-nodes.md) (`status: accepted`).
Contract: [FIND-756](./FIND-756.md).
**Este fichero no re-decide nada.** Convierte el contrato en un orden de pasos con
criterio de parada. Si algo de aquí contradice a `FIND-756`, manda `FIND-756` salvo
que esté listado en §9 (discrepancias medidas contra `HEAD` el 2026-11-26).

---

## §0 — Anti-objetivo (leer antes de escribir una línea)

> **Si A11 falla y A7 pasa, el fix está a medias — cubriste el record por-record y
> dejaste el pipeline sin aristas.**

La trampa de esta tarea no es compilar: es **creer que está hecha**. Cuatro de los
seis sitios ya tienen un bucle de aristas que parece correcto y no crea ninguna. Un
diff que sólo cambie los ids de los sitios 1-4 compila, pasa `cargo test -p
vanta-memory`, y deja el pipeline exactamente igual de roto que hoy.

**Criterio de "hecho", en dos mitades que no se pueden fundir en una:**

| Verde | Significa | NO significa |
|---|---|---|
| **A11** | el lote (`apply_dedup_batch`) enlaza records **L1** cuando las fuentes son records L1 | que el pipeline de producción enlace |
| **A11b** | el lote **no** enlaza turnos L0 (0 aristas), porque el padre vive en `l0/<session>` y `l0_namespace` es privada | que el pipeline esté roto para siempre: cambia a 1 cuando se exporte `l0_namespace` |

Es decir: **A11 verde + A11b verde (0) es el estado completo de esta tarea**, y sigue
dejando el pipeline de producción sin aristas — gap medido y declarado en §9.4, no un
efecto secundario. Quien cierre la fila del Backlog cita **ambas**, nunca sólo A11.

Ninguna aserción puede basarse en `stderr` ni en la ausencia de `Node not found`: el
log lleva callado desde `FIND-747`, y eso es justo lo que hizo invisible el bug.

---

## §1 — Alcance de escritura (exacto, y por qué no toca `src/`)

### 1.1 Ficheros que se tocan

| Fichero | Qué cambia |
|---|---|
| `vanta-memory/src/core/scene/scene_index.rs` | Sitio 1: proyección de `scene_node_id` + `ep_node_id` en `write_scene_block` (y el comentario `Guarded:` de `:229-232`) |
| `vanta-memory/src/core/persona/persona_generator.rs` | Sitio 2: proyección de `persona_node_id`, `ep_node_id`, `scene_node_id` en `write_persona` (y `:473-475`) |
| `vanta-memory/src/core/record/l1_writer.rs` | Sitio 4 (`put_record_with_options`), sitio 5 (`apply_dedup_batch`), el helper privado de proveniencia, y el comentario `:789-791` |
| `vanta-memory/src/core/profile/profile_sync.rs` | Sitio 3: **añadir** la arista que hoy no existe en `sync_persona_to_scope` |
| `vanta-memory/src/core/ontology/mod.rs` | D7: idempotencia en `ensure_lineage_edge` + su test unitario (A4b) |
| `vanta-memory/tests/lineage_graph.rs` | **Nuevo.** A1-A3, A4, A5-A12, A11b |
| `docs/CHANGELOG.md` | **No.** Nada de esto es superficie pública. No lo toques |

### 1.2 Ficheros que NO se tocan

`src/**` completo · `vantadb-mcp/**` · `vantadb-server/**` · `docs/dev/Backlog.md` (la
fila `FIND-756` la cierra quien implemente, en su propio paso) · los dos ficheros de
test existentes que tocan el área (`ontology/mod.rs:340-354` y `:359-382` siguen
pasando sin cambios).

### 1.3 Por qué no hace falta `src/`

`memory_node_id` ya está exportado a crates hermanos (`src/sdk/mod.rs:33`), y los node
ids de los records **no cambian** — sólo cambia qué hash se usa para nombrar un
extremo. Consecuencia dura, y es la razón de la restricción:

- no hay re-index, ni rebuild de HNSW, ni rebuild del índice de texto;
- no hay migración de aristas: las viejas **no existen** (nunca se crearon), no están mal;
- no hay cambio de formato en disco: `MemoryRecord` no gana campos.

**Si crees que necesitas tocar `src/`, para y vuelve al ADR.** Las dos únicas cosas
tentadoras — añadir `add_memory_edge` al core, o materializar los ids fantasma con
`insert_node` — son O7 y O2 del ADR, y están rechazadas con razón escrita.

---

## §2 — Los seis sitios

Verificado contra `HEAD` el 2026-11-26. Los números de línea son de hoy; si el árbol
se movió, localiza por nombre de función, no por línea.

| # | Función | Camino | Estado hoy |
|---|---|---|---|
| 1 | `scene_index::write_scene_block` (`:188`) | L2, escena | arista con id fantasma `entity_node_id("scene:{s}:{key}")` (`:234-235`) |
| 2 | `persona_generator::write_persona` (fn privada, llamada desde `generate_persona` `:221`) | L3, persona | arista con id fantasma (`:477-478`, `:491-492`) |
| 3 | `profile_sync::sync_persona_to_scope` (`:116`) | L3, persona con scope | **ninguna arista** — sólo metadata (`:154-163`) |
| 4 | `l1_writer::put_record_with_options` (`:767`) | L1 record por-record (`put_record`, `put_l3_axiom`, `write_memory`, `rollback`) | arista con id fantasma (`:794-795`, `:814-815`) |
| 5 | `l1_writer::apply_dedup_batch` (`:457`) | **L1 por lote — el que usa el pipeline** | **ninguna arista**: hace `db.put_batch` (`:520`) y nunca pasa por `put_record_with_options` |
| 6 | `scene_index`/`persona` sobre `derived_from` de **campo** | el `put` ya la crea | no añadir nada; sólo no duplicar (D7) |

### 2.1 Las cinco proyecciones `(namespace, key)`

| Extremo | namespace | key |
|---|---|---|
| escena (L2) | `scene_namespace(session_key)` | el `key` local de `write_scene_block` (`:196`) |
| persona (L3) | `persona_namespace(session_key)` | el `key` local de `write_persona` (`:440`) |
| persona con scope | `profile_namespace(build_profile_isolation_scope(isolation))` | el `key` local de `sync_persona_to_scope` (`:136`) |
| record L1 / axioma L3 | el `ns` de la escritura | `sanitize_key(&record.id)` — lo que usa `record_input` (`:733`) |
| episodio / mensaje (padre) | `l1_namespace(<sesión>)` | `sanitize_key(src_id)` |

La regla que hace que esto sea correcto y no una segunda ortografía: **la key es la
que usó `put`**. Por eso el padre de un episodio se sanea igual que se sanea el key
del record del episodio — `record_input` guarda `key: sanitize_key(&record.id)`.

### 2.2 El código, por sitio

Copia esto; reusa **las variables locales que ya existen**, no recalcules claves.

**Sitio 1 — `scene_index.rs::write_scene_block`**

```rust
// ANTES del db.put (:218-227): ns se mueve al MemoryInput en :219, así que el id
// del hijo se calcula aquí. El hash no depende del put.
let scene_node_id = vantadb::sdk::memory_node_id(&ns, &key);

db.put(MemoryInput {
    namespace: ns,          // :219 — sin cambios, se sigue moviendo
    key: key.clone(),       // :220 — sin cambios
    ..                      // payload/metadata igual
})?;

// DESPUÉS del put: sustituye el bloque de :233-247. `now_ms` ya existe en :233:
// NO lo redeclares (shadowing = warning en clippy -D warnings).
let l1_ns = crate::core::record::l1_reader::l1_namespace(session_key);
for ep_id in &block.source_episode_ids {
    let ep_node_id = vantadb::sdk::memory_node_id(
        &l1_ns,
        &crate::core::conversation::sanitize_key(ep_id),
    );
    crate::core::ontology::ensure_lineage_edge(
        db, scene_node_id, ep_node_id, "derived_from", now_ms,
    );
}
```

**Sitio 2 — `persona_generator.rs::write_persona`**

```rust
// ANTES del db.put (:462-471): ns se mueve en :463.
let persona_node_id = vantadb::sdk::memory_node_id(&ns, &key);

db.put(MemoryInput { namespace: ns, key: key.clone(), .. })?;

// DESPUÉS del put: sustituye :476-500. `now_ms` ya existe en :476.
let l1_ns = crate::core::record::l1_reader::l1_namespace(session_key);
let scene_ns = crate::core::scene::scene_index::scene_namespace(session_key);
for ep_id in &record.source_episode_ids {
    let ep_node_id = vantadb::sdk::memory_node_id(
        &l1_ns,
        &crate::core::conversation::sanitize_key(ep_id),
    );
    crate::core::ontology::ensure_lineage_edge(
        db, persona_node_id, ep_node_id, "derived_from", now_ms,
    );
}
for scene_name in &record.source_scene_names {
    let scene_node_id = vantadb::sdk::memory_node_id(
        &scene_ns,
        &crate::core::conversation::sanitize_key(scene_name),
    );
    crate::core::ontology::ensure_lineage_edge(
        db, persona_node_id, scene_node_id, "derived_from", now_ms,
    );
}
```

**Sitio 3 — `profile_sync.rs::sync_persona_to_scope`** (hoy no hay arista: se añade)

```rust
// :135-136 ya calculan ns y key (ambos se MUEVEN en el put de :164-173).
let ns = profile_namespace(&build_profile_isolation_scope(isolation));
let key = crate::core::conversation::sanitize_key(PERSONA_KEY);

// NUEVO, justo después de :136 y antes de cualquier return temprano de :137-146:
let profile_node_id = vantadb::sdk::memory_node_id(&ns, &key);

// ... el db.put de :164 sin cambios ...

// DESPUÉS del put, antes del Ok(...) de :174:
let now_ms = now_ms();                      // ya importado en :13
let l1_ns = crate::core::record::l1_reader::l1_namespace(session_key); // parámetro de la fn
for ep_id in &persona.source_episode_ids {
    let ep_node_id = vantadb::sdk::memory_node_id(
        &l1_ns,
        &crate::core::conversation::sanitize_key(ep_id),
    );
    crate::core::ontology::ensure_lineage_edge(
        db, profile_node_id, ep_node_id, "derived_from", now_ms,
    );
}
```

> `profile_sync` **no** importa `sanitize_key` hoy (`:13` importa `now_ms` y
> `sanitize_component`). Usa la ruta completa `crate::core::conversation::sanitize_key`
> como arriba, o añade el import — pero no inventes un saneado local.

**Sitio 4 — `l1_writer.rs::put_record_with_options`** (`ns: &str`, no se mueve)

```rust
// :792-823, tras el `let stored = db.put(...)` de :781-787.
let now_ms = crate::core::conversation::now_ms();
// `session_key` YA existe en :793: `ns.strip_prefix("l1/").unwrap_or(ns)`.
// NO la re-derives de `record.session_key` — ver §9.1.
let l1_ns = crate::core::record::l1_reader::l1_namespace(session_key);
let node_id = vantadb::sdk::memory_node_id(ns, &crate::core::conversation::sanitize_key(&record.id));
for src_id in provenance_source_ids(record) {          // helper nuevo, §4.3
    let parent_node_id = vantadb::sdk::memory_node_id(
        &l1_ns,
        &crate::core::conversation::sanitize_key(&src_id),
    );
    crate::core::ontology::ensure_lineage_edge(
        db, node_id, parent_node_id, "derived_from", now_ms,
    );
}
```

**Sitio 5 — `l1_writer.rs::apply_dedup_batch`** → §4 completo (es el que decide si
el fix sirve).

**Sitio 6 — idempotencia (D7)** → §3 trampa 6.

---

## §3 — Las seis trampas

Las seis están medidas contra `HEAD`. Cada una produce un fallo **silencioso**: el
test pasa, la suite pasa, y la arista no existe o se duplica.

1. **El `ns` se mueve al `MemoryInput`.** En los sitios 1, 2 y 3 el `namespace` es
   `ns` sin `&` (`scene_index.rs:219`, `persona_generator.rs:463`,
   `profile_sync.rs:165`). Por eso el id **del hijo** se calcula antes del `put`, y
   sólo la creación de la arista va después. En los sitios 1 y 2 `key` sobrevive
   porque se clona (`key.clone()`); en el sitio 3 se mueve también (`key,` en
   `:166`), así que ahí los **dos** ids se calculan antes. En el sitio 4 `ns` es
   `&str` y no hay problema; en el 5 `ns` es un `String` local que sólo se presta
   (`&ns` en `:511`).

2. **`session_key` no es lo que el contrato dice en el sitio 4.** `put_record_with_options`
   recibe `(db, ns, record, vector, confidence_override, dedup_policy_override)`
   (`:767-774`) — no hay `session_key` en la firma. Pero **sí hay una derivación
   local en `:793`**: `ns.strip_prefix("l1/").unwrap_or(ns)`. Reúsala. Ver §9.1:
   usar `record.session_key` compila y es peor.

3. **`sanitize_key` en la clave del padre.** `record_input` guarda con
   `key: sanitize_key(&record.id)` (`:733`), así que los ids de episodio —que son
   ids de record— se sanean igual. Con el id crudo, la arista se salta **en
   silencio** para cualquier id con un carácter fuera del set seguro.

4. **La arista va después del `put`, siempre.** `insert_one_internal` corre
   `reconcile_lineage_edges` (`src/sdk/api/memory.rs:1245-1251`) y borra toda arista
   `derived_from` que el **campo del nodo** (`__vanta_derived_from`) no declare; en
   el camino primario estos records declaran por **metadata**, no por campo. Crear la
   arista antes del `put` = que el `put` la borre. El **id** del hijo sí puede —y en
   tres sitios debe— calcularse antes.

5. **`source_episode_ids` ∪ `source_message_ids`, sin separar y sin ramificar.** El
   código de `:796-812` (sitio 4) es la regla: primero el array
   `metadata["source_episode_ids"]`, después `record.source_message_ids`, dedupe por
   `contains` al añadir. `put_l3_axiom` pone `source_message_ids =
   source_episode_ids` (`cascade_forget.rs:143`), así que los campos **no son
   disjuntos**: ramificar por campo crea duplicados y pierde ids.

6. **`ensure_lineage_edge` debe deduplicar (D7).** `add_edge` **no** deduplica
   (`src/sdk/api/graph.rs:132-138,144-150`), y hay records que llevan la **misma**
   arista por dos productores (campo del nodo + metadata), además de los `put` que
   retornan temprano por dedup sin reconciliar. El guard ya tiene los dos
   `NodeRecord` (`ontology/mod.rs:269-295`), así que el chequeo es gratis: si el nodo
   origen ya tiene una arista forward **con ese mismo `label`** hacia ese `to`,
   **retorna `true` sin llamar a `add_edge`** — ya existía, y así lo dice el doc vivo
   del guard (`ontology/mod.rs:261`: *"Returns `true` when the edge was created (or
   already existed)"*). El `false` se reserva **exclusivamente** al endpoint ausente y
   al error de `add_edge`; **la precondición, el `debug!` con `missing_node` y ese
   `false` no se tocan** — sólo se añade el chequeo de duplicado.

   > **El dedupe es por `(target, label)`, no sólo por `target`.** `label` es un
   > parámetro y el camino de tripletas pasa `&triplet.predicate`
   > (`ontology/mod.rs:238`): un sujeto con dos predicados distintos hacia el mismo
   > objeto es legítimo (`likes` y `works_at`). Deduplicar sólo por `target` rompería
   > las tripletas — que están en la lista de no-regresión de §8.

   > **No inviertas este `true` a `false`.** El doc de `:261` está en la lista de
   > regresión de §8, A4b exige `true` (§5.3), y ningún caller usa el `bool`
   > (`ontology/mod.rs:238` lo descarta como expresión suelta; no hay `#[must_use]`):
   > devolver `false` obligaría a cambiar A4b **y** el doc, sin comprar nada.

Y una séptima que no es de proyección sino de fixture:

7. **`sync_persona_to_scope` retorna temprano si el contenido no cambió**
   (`profile_sync.rs:137-146`: `record.content == body` ⇒ `Ok` sin `put`). En una
   base que ya tiene el scoped persona sincronizado, **re-ejecutar no crea la
   arista**. A8 usa una base limpia (primer sync). Es la misma clase de agujero que
   la deuda `FIND-772` (sin backfill), no un bug nuevo.

---

## §4 — El sitio 5: `apply_dedup_batch` (el que decide si el fix sirve)

El contrato exige cubrir este camino pero no dice **cómo**; aquí está la decisión,
con su razón. Sin esto la tarea no es ejecutable.

### 4.1 Qué hace hoy

`apply_dedup_batch(db, session_key, session_id, memories, decisions, now_ms, embed)`
(`:457-465`), en cuatro fases:

| Fase | Líneas | Qué hace |
|---|---|---|
| 1 | `:468-496` | `plan_write` por memoria ⇒ `planned_writes: Vec<PlannedWrite>` |
| 2 | `:498-521` | `record_input(&ns, &planned.record, …)` y **`db.put_batch(inputs)`** (`:520`) |
| 3 | `:523-550` | poda: `db.delete(&ns, key)` de los targets que el batch no reescribió |
| 4 | `:552-559` | `log_and_mark` (generation log + `mark_contradicted_targets`) y `Ok(written)` |

En ninguna fase se crea una arista. `put_batch_inner` reconcilia sólo los padres
declarados en el **campo** del nodo (`src/sdk/api/memory.rs:1638-1705`) y estos
records no declaran nada ahí.

### 4.2 Cómo se cubre: **no hay hook en `put_batch`** — Phase 5 explícita

Descartado el hook: `put_batch` vive en `src/` (fuera de alcance), es genérico sobre
`MemoryInput` (no conoce `MemoryRecord` ni la sesión), y un hook ahí obligaría a
cambiar el core para un caso de un solo crate.

La forma es **una quinta fase al final de `apply_dedup_batch`**, entre el bucle de la
fase 4 (`:558`) y el `Ok(written)` (`:559`):

```rust
    // Phase 5 - lineage edges (ADR-0066 D1b/FIND-756). After `put_batch` (D3: the
    // put would reconcile them away) AND after the phase-3 prune, so no edge points
    // at a node this same flush deletes. Only on the success path.
    let l1_ns = crate::core::record::l1_reader::l1_namespace(session_key);
    for planned in &planned_writes {
        let node_id = vantadb::sdk::memory_node_id(
            &ns,
            &crate::core::conversation::sanitize_key(&planned.record.id),
        );
        for src_id in provenance_source_ids(&planned.record) {
            let parent_node_id = vantadb::sdk::memory_node_id(
                &l1_ns,
                &crate::core::conversation::sanitize_key(&src_id),
            );
            crate::core::ontology::ensure_lineage_edge(
                db, node_id, parent_node_id, "derived_from", now_ms,
            );
        }
    }
    Ok(written)
```

Tres decisiones dentro de esa forma, y por qué:

- **Al final y no justo tras `put_batch`.** La fase 3 borra keys. Una arista creada
  antes de la poda puede apuntar a un nodo que el mismo flush elimina: arista
  colgante. El contrato sólo exige "después del `put_batch`"; al final cumple y
  además deja el grafo consistente con el estado final.
- **`now_ms` es parámetro** (`:463`), no hay que inventarlo ni releer el reloj: se
  usa el mismo timestamp del flush (que ya usan `plan_write` y `log_and_mark`).
- **`ns` y `session_key` ya existen** (`:466`, `:459`) — exactamente por eso este
  sitio es directo, como dice el contrato.

### 4.3 El helper compartido (una sola ortografía)

Sitios 4 y 5 necesitan la **misma** unión. En vez de dos copias de `:796-812`,
extrae a `l1_writer.rs` (privado) y llama desde los dos:

```rust
/// Provenance parents of a record: `metadata["source_episode_ids"]` ∪
/// `source_message_ids`, deduped, order-preserving. One rule, two call sites
/// (per-record + batch) - ADR-0066 D4.
fn provenance_source_ids(record: &MemoryRecord) -> Vec<String> {
    let mut sources: Vec<String> = Vec::new();
    if let Some(arr) = record
        .metadata
        .get("source_episode_ids")
        .and_then(|v| v.as_array())
    {
        for item in arr {
            if let Some(s) = item.as_str() {
                sources.push(s.to_string());
            }
        }
    }
    for src in &record.source_message_ids {
        if !sources.contains(src) {
            sources.push(src.clone());
        }
    }
    sources
}
```

Sustituye `:796-812` por `let sources = provenance_source_ids(record);` y deja el
bucle de `:813-823`. Mismo comportamiento que hoy, una definición.

### 4.4 Lo que A11 prueba y lo que **no** prueba

`ExtractedMemory` (`core/abstractions/types.rs:56-72`) **no tiene**
`source_episode_ids`; tiene `source_message_ids` y un `metadata: Value` libre.
`plan_write` copia ambos al `MemoryRecord` (`:213-214`, `:278-279`). Por eso:

- **A11 es construible**: un `ExtractedMemory` con
  `metadata: json!({"source_episode_ids": ["m_a"]})` llega al post-loop y produce la
  arista hacia el record L1 `m_a`. Eso prueba que el camino del lote crea aristas.
- **A11 NO prueba que el pipeline de producción enlace.** En producción,
  `ExtractedMemory.source_message_ids` son **turnos L0**, y un turno L0 vive en
  `l0/<session>`, no en `l1/<session>` (`l0_recorder.rs:363`; `l0_namespace` es
  privada, `:556`). Con la proyección de §2.1 el padre cae en `l1/<session>/<turno>`,
  el guard no lo encuentra y lo salta en `debug!`.

Que eso quede **fijado con una aserción** (A11b) en vez de descubierto en producción
es el punto: hoy da 0 aristas y ese es el estado honesto; cuando se exporte
`l0_namespace` (deuda declarada en ADR-0066 §Relation to FIND-296) A11b cambia a 1
**a propósito**, no se descubre por accidente.

**El fixture de A11 no reproduce un input de producción, y hay que decirlo.** Ningún
writer del árbol pone `source_episode_ids` en `ExtractedMemory.metadata`: se verificó
que la clave la emiten sólo `scene_index.rs:208-217`, `persona_generator.rs:452-461`,
`profile_sync.rs:154-163`, `cascade_forget.rs:145` y `ontology/mod.rs:197`, y
**ninguno pasa por `apply_dedup_batch`**. A11 inyecta esa clave a mano para ejercitar
el post-loop; por tanto **A11 prueba el mecanismo, no un caso real**. El caso real del
pipeline es A11b → 0.

---

## §5 — Las 13 aserciones

Ficheros: **`vanta-memory/tests/lineage_graph.rs`** (nuevo) y, sólo para A4b,
`vanta-memory/src/core/ontology/mod.rs` `mod tests` (razón en §5.3).

Patrón de apertura ya usado en el crate (`vanta-memory/tests/scene.rs:18-25`):

```rust
fn open_db() -> Embedded {
    let config = Config { backend_kind: BackendKind::InMemory, read_only: false, ..Config::default() };
    Embedded::open_with_config(config).expect("open in-memory db")
}
// Allow de tests: `#![allow(clippy::expect_used, clippy::unwrap_used)]` como en scene.rs:1-2.
```

Las aristas se leen del **grafo**, no del log:
`db.get_node(id).unwrap().expect("node").edges` → `EdgeRecord { target: u128,
label: String, reverse: bool, .. }` (`src/sdk/serialization/graph_types.rs:9-26`;
`NodeRecord.edges` en `:67`; `get_node` en `src/sdk/api/graph.rs:61`).

Escenario base: sesión `"s1"`; dos records L1 `m_a` y `m_b` escritos con
`put_record(db, &l1_namespace("s1"), &record)`; una escena `"alpha"` creada con
`upsert_scene_with_sources(db, "s1", "alpha", SUMMARY, CONTENT, vec!["m_a".into(), "m_b".into()])`
(`scene_index.rs:59-66`).

| # | Aserción | Cómo, exactamente | Fichero |
|---|---|---|---|
| **A1** | La escena tiene arista forward a cada episodio | `get_node(memory_node_id(&scene_namespace("s1"), "alpha"))` → `edges` contiene, por cada `ep` en `["m_a","m_b"]`, una `e.target == memory_node_id(&l1_namespace("s1"), ep) && e.label == "derived_from" && !e.reverse` | `tests/lineage_graph.rs` |
| **A2** | El episodio tiene la mitad reverse | `get_node(memory_node_id(&l1_namespace("s1"), "m_a"))` → `edges` contiene `e.target == <id de la escena> && e.label == "derived_from" && e.reverse` | idem |
| **A3** | Conteo exacto **tras cada escritura** | exactamente **2** forward `derived_from` en la escena, y **1** reverse por episodio; ninguna duplicada. Cuenta también `superseded_by`/otros labels para probar que no interfieren: filtra por `label == "derived_from"` | idem |
| **A4** | Idempotencia real | repetir `upsert_scene_with_sources` con las mismas fuentes ⇒ el conteo de A3 **no cambia** (ni 3 forward, ni 2 reverse) | idem |
| **A4b** | Idempotencia **sin** el drop del reconcile | insertar la arista forward con `db.add_edge` **antes** de llamar a `ensure_lineage_edge` ⇒ sigue habiendo **una**, no dos. Es la aserción que prueba D7/§3.6; A4 sola pasa aunque el guard no deduplique, porque el `put` previo borra | **`src/core/ontology/mod.rs` `mod tests`** — ver §5.3 |
| **A5** | Los ids fantasma **no** existen | `get_node(entity_node_id("scene:s1:alpha")).unwrap().is_none()` y lo mismo para `entity_node_id("episode:s1:m_a")`: el fix no materializa nada | `tests/lineage_graph.rs` |
| **A6** | Persona → episode **y** persona → scene | `generate_persona` (`persona_generator.rs:221`) es la entrada pública. Encadena: records L1 ⇒ `upsert_scene_with_sources(db, "s1", "alpha", .., vec!["m_a".into()])` ⇒ `generate_persona(&db, &Fixed(good_output("body")), &params())`. `source_scene_names`/`source_episode_ids` los deriva el generador de las escenas de la ventana (`:386-401`), así que salen `["alpha"]` y `["m_a"]`. Aserta **dos** forward distintas desde `memory_node_id(&persona_namespace("s1"), "persona.md")`: targets `memory_node_id(&l1_namespace("s1"), "m_a")` y `memory_node_id(&scene_namespace("s1"), "alpha")` | idem |
| **A7** | L1 record → episode | record L1 con `metadata = json!({"source_episode_ids": ["m_a"]})` escrito vía `put_record` ⇒ arista desde `memory_node_id(&l1_namespace("s1"), <id del fixture>)` hacia `memory_node_id(&l1_namespace("s1"), "m_a")` | idem |
| **A8** | Scoped persona → episode (D5) | base limpia ⇒ `sync_persona_to_scope(&db, "s1", &ProfileIsolation::default())` ⇒ arista desde `memory_node_id(&profile_namespace(&build_profile_isolation_scope(&ProfileIsolation::default())), "persona")` hacia `memory_node_id(&l1_namespace("s1"), "m_a")`. Las dos funciones son `pub` (`profile_sync.rs:41,65`), así que **no escribas el scope a mano**: `"team:default\|agent:default"` funciona sólo porque `sanitize_component` mapea `:` y `\|` a `_`, y eso es un detalle que no quieres fijar en un literal. **Requiere** una persona previa con `source_episode_ids` (genera con A6 o escribe la persona directamente) | idem |
| **A9** | Guard intacto: padre ausente | escena con `source_episode_ids = vec!["no_existe"]` ⇒ **0** aristas forward desde la escena, sin panic, sin nodo inventado (`get_node(memory_node_id(&l1_namespace("s1"), "no_existe"))` es `None`) | idem |
| **A10** | Guard intacto: episodio **ausente** | escribir `m_a`; crear la escena (arista existe); `db.forget(&l1_namespace("s1"), ForgetTarget::Key("m_a".into()), ForgetMode::Hard)` ⇒ `get_node(memory_node_id(&l1_namespace("s1"), "m_a"))` es `None`, el guard salta y no se crea arista; sin panic. **`Soft` no sirve**: reinserte el nodo con `invalid_at_ms` (`src/sdk/api/memory.rs:2081-2106`) y el guard lo acepta | idem |
| **A11** | **El path del pipeline** | un flush por `apply_dedup_batch` (**no** `put_record`): `ExtractedMemory` con `metadata: json!({"source_episode_ids": ["m_a"]})` y `decision("m_nuevo", DedupAction::Store)` ⇒ la arista existe desde `memory_node_id(&l1_namespace("s1"), "m_nuevo")` hacia `memory_node_id(&l1_namespace("s1"), "m_a")`. Firma del helper: `apply_dedup_batch(&db, "s1", "sess-id", &memories, &decisions, now_ms, None)` (patrón real en `tests/l1_dedup.rs:298-299`) | idem |
| **A11b** | El hueco que A11 no cierra: un turno L0 no se enlaza **todavía** | un flush por `apply_dedup_batch` con `source_message_ids = vec!["turn_1"]` y `metadata: Null` ⇒ **0** aristas forward desde el record. Fija el estado honesto de producción (el padre vive en `l0/<session>` y `l0_namespace` es privada). `// A11b flips to 1 when l0_namespace is exported (ADR-0066 §Relation to FIND-296)` | idem |
| **A12** | Episodio `Soft` sigue siendo destino válido | tras `forget(.., ForgetMode::Soft)`, `get_node` devuelve `Some` y la arista **se crea**: existencia en el grafo ≠ visibilidad en recall (D9) | idem |

### 5.1 Lo que ya existe y se reutiliza (no lo reescribas)

- `vanta-memory/tests/persona.rs:34-68` — los stubs `Fixed(String)` (`:35-41`) y
  `NeverCalled` (`:44-50`) para el `LlmRunner` de A6, `params()` (`:52-59`) y
  `good_output(...)` (`:66-68`) para el payload. Cópialos al nuevo fichero (son
  locales al test, no se exportan).
- `vanta-memory/tests/l1_dedup.rs:35-44` — `memory(content) -> ExtractedMemory` y
  `:46-...` `record(id, content) -> MemoryRecord`. Copia el patrón, no lo importes.
- `MemoryRecord` es un struct literal sin constructor: copia los campos de
  `l1_dedup.rs:46-...`. `session_key` **debe** ir puesto (`"s1"`), y
  `metadata: json!({"source_episode_ids": ["m_a"]})` para A7/A11.

### 5.2 Visibilidad: qué **no** puede llamar un test de `tests/`

Un fichero en `vanta-memory/tests/` es un crate externo. No alcanza:

- `sanitize_key` (`pub(crate)`, `l0_recorder.rs:267`, re-export en
  `conversation/mod.rs:12`) ⇒ **escribe las claves como literales del fixture**
  (`"alpha"`, `"persona"`, `"persona.md"`, `"m_a"`, `"turn_1"`), que ya son sanas
  (`sanitize_key` = `sanitize_component(s, 512, false)`, y esos no llevan caracteres
  fuera del set).
- El `PERSONA_KEY` de `profile_sync` (`:70`, privado, vale `"persona"`) — **distinto**
  del `PERSONA_KEY` de `persona_generator` (`:49`, `pub`, vale `"persona.md"`). Es la
  confusión más fácil de cometer en A6 vs A8.
- `put_record_with_options` (`pub(crate)`, `:767`) ⇒ usa `put_record` (`pub`, `:748`).
- `write_persona` (privada) ⇒ usa `generate_persona` (`pub`, `:221`).
- `ensure_lineage_edge` (`pub(crate)`, `ontology/mod.rs:262`) ⇒ ver §5.3.

### 5.3 A4b va en el módulo, no en el crate de test

A4b necesita llamar a `ensure_lineage_edge` **directamente** (para probar el guard sin
el `put` de por medio), y `ensure_lineage_edge` es `pub(crate)`. Por eso A4b es un
**test unitario** en `vanta-memory/src/core/ontology/mod.rs` `mod tests`, junto a los
dos que ya existen (`:340-354`, `:359-382`), con el mismo estilo:

```rust
#[test]
fn lineage_edge_is_idempotent_when_forward_edge_exists() {
    // ambos extremos materializados (como en :359-382)
    // 1) db.add_edge(from, to, "derived_from", None, Some(0))
    // 2) assert!(ensure_lineage_edge(&db, from, to, "derived_from", 1))
    // 3) exactamente UNA arista forward from->to (no dos)
}
```

Nombre exacto sugerido: `lineage_edge_is_idempotent_when_forward_edge_exists`. No
renombres ni borres los dos tests existentes.

---

## §6 — Orden de los pasos (atómico y verificable cada uno)

Cada paso deja el árbol compilando y la suite verde. Esfuerzo en horas de una persona
que ya leyó este fichero.

| # | Paso | Fichero | Effort | Verificación de parada |
|---|---|---|---|---|
| **1** | **D7 primero**: deduplicado en `ensure_lineage_edge` + A4b como test unitario | `ontology/mod.rs` | **0.5 h** | `cargo test -p vanta-memory --lib ontology` — el nuevo test pasa, los dos viejos siguen pasando |
| **2** | **El helper** `provenance_source_ids` + sustituir el inline de `:796-812` (sin cambiar ningún id todavía) | `l1_writer.rs` | **0.25 h** | `cargo test -p vanta-memory --test l1_dedup --test l1_batch` verde (comportamiento idéntico) |
| **3** | **Sitio 1** escena | `scene_index.rs` | **0.5 h** | A1+A2+A3 escritas y verdes |
| **4** | **Sitio 5** `apply_dedup_batch` (Phase 5) + A11 + A11b | `l1_writer.rs` | **1.5 h** | A11 verde. **A11b verde con 0 aristas** (documenta el hueco, no lo esconde) |
| **5** | **Sitio 4** `put_record_with_options` + A7 | `l1_writer.rs` | **0.5 h** | A7 verde; A3/A11 no cambian |
| **6** | **Sitio 2** persona + A6 | `persona_generator.rs` | **1 h** | A6 verde con dos targets distintos |
| **7** | **Sitio 3** `sync_persona_to_scope` (arista nueva) + A8 | `profile_sync.rs` | **0.5 h** | A8 verde en base limpia |
| **8** | **Los guards**: A9, A10, A12, A4, A5 | `tests/lineage_graph.rs` | **1.5 h** | A9/A10/A12 verdes sin panic; A4 conteo estable; A5 `None` |
| **9** | **Comentarios**: reescribir los `Guarded: … declared skip` de `scene_index.rs:229-232`, `persona_generator.rs:473-475`, `l1_writer.rs:789-791` — dejan de describir el camino normal. En `profile_sync.rs`: comentario nuevo explicando que la arista va tras el `put` y por qué | los 4 | **0.5 h** | `rg -n 'declared skip' vanta-memory/src` → **0 hits** |
| **10** | **Cierre**: suite completa del crate + clippy + `cargo check -p vantadb` | — | **1 h** | §7 entero |

**Total: ~8.25 h ≈ 1 día.** Si el paso 4 se está llevando más de 3 h, para: lo más
probable es que estés intentando enlazar turnos L0 (A11b) en vez de aceptarlo como
deuda declarada.

**Por qué el paso 1 antes de todo:** sin D7, A4b falla y cada paso posterior añade
duplicados que hay que diagnosticar después, con cinco sitios ya cambiados.

**Por qué el paso 4 antes del 5:** el sitio 5 es el que decide si el fix sirve
(§0). Si tiene un problema estructural, mejor descubrirlo con un solo sitio ya
tocado que con cuatro.

---

## §7 — Verificación

```bash
cargo test -p vanta-memory --test lineage_graph          # A1..A12 + A11b
cargo test -p vanta-memory --lib ontology                # A4b + los dos existentes
cargo test -p vanta-memory --test scene_strategy --test persona --test e2e_flow
cargo test -p vanta-memory                                # suite del crate
cargo clippy -p vanta-memory --all-targets -- -D warnings
cargo check -p vantadb                                    # `src/` sin diff
```

Y los checks de no-regresión que se pueden correr desde la raíz:

```bash
git status --short src/                                   # debe estar vacío
rg -n 'declared skip' vanta-memory/src                    # 0 hits
rg -c 'entity_node_id' vanta-memory/src/core/scene/scene_index.rs \
   vanta-memory/src/core/persona/persona_generator.rs \
   vanta-memory/src/core/record/l1_writer.rs               # 0 en los tres
rg -l 'memory_node_id' vanta-memory/src --glob '*.rs'      # exactamente 4 ficheros:
                                                           # scene_index, persona_generator,
                                                           # l1_writer, profile_sync
```

> `rg -l` (ficheros, no líneas) es lo que verifica "ningún hit fuera": un `| wc -l`
> cuenta líneas y no prueba nada — y `wc` no existe en el shell por defecto de este
> repo (Windows/pwsh).

Tests existentes que **no** cambian y deben seguir pasando (verificado que no afirman
el bug): `ontology/mod.rs:340-354` (`lineage_edge_skips_when_endpoints_are_not_nodes`
— usa ids que siguen sin ser nodos) y `:359-382`
(`lineage_edge_created_when_both_endpoints_exist` — movimiento directo, sin
`metadata`).

---

## §8 — Lo que NO cambia (lista de regresión)

- `src/sdk/serialization/mod.rs` — `memory_node_id` (`:118`). Tocarla invalida **todos**
  los node ids de **todas** las bases existentes.
- `src/sdk/api/graph.rs` — `add_edge` (`:113`), `ensure_edge` (`:186`),
  `LABEL_DERIVED_FROM` (const + doc en `:20-27`). Sin firma nueva, sin label nuevo,
  sin cambio de dirección.
- `vanta-memory/src/core/ontology/mod.rs:45-52` y `:216-238` — `entity_node_id` y las
  tripletas: sus nodos **sí** se materializan y su espacio FNV es coherente. Sigue
  `pub`.
- `vanta-memory/src/core/ontology/mod.rs:262-302` — el guard conserva precondición,
  `debug!` con `missing_node` y `false`. Sólo gana el chequeo de duplicado.
- Payload y metadata de los records: sin campo nuevo en `MemoryRecord`, sin cambio en
  el formato de export/import.
- Node ids de los records: sin re-index, sin rebuild de HNSW, sin rebuild del índice
  de texto.

---

## §9 — Discrepancias medidas contra `HEAD` (2026-11-26)

Dos cosas del contrato `FIND-756` / ADR-0066 no coinciden con el código de hoy. No
invalidan la decisión — la proyección y el orden son los del ADR — pero el
implementador no debe decidirlas solo:

### 9.1 `session_key` en el sitio 4 sale de `ns`, no de `record.session_key`

El ADR (D1) dice: *"The session comes from `record.session_key`
(`MemoryRecord.session_key`, `core/abstractions/types.rs:108`)"*. El campo **existe**
(`types.rs:108`), pero `put_record_with_options` **no lo usa**: ya deriva la sesión en
`:793` con `ns.strip_prefix("l1/").unwrap_or(ns)`.

**Resolución (obligatoria):** reusa la local de `:793`. Razones:

1. es lo que el código ya hace hoy para construir el id fantasma — el fix cambia el
   **input del hash**, no la derivación de la sesión (diff mínimo);
2. `l1_namespace` es `format!("l1/{}", sanitize_component(session_key, 128, false))`
   (`l1_reader.rs:25-27`), así que `l1_namespace(ns.strip_prefix("l1/")) == ns` para
   cualquier `ns` que ya sea un namespace L1 — el valor es el mismo, y no depende de
   que el caller haya rellenado `record.session_key`;
3. `record.session_key` es controlado por el caller: un `MemoryRecord` con
   `session_key` vacío produciría `l1_namespace("")` = `"l1/"`, el padre no se
   encontraría, y el guard se saltaría la arista **en silencio** — el modo de fallo
   exacto que esta tarea existe para eliminar.

No cambies la derivación. No añadas `record.session_key` como segunda fuente.

### 9.2 El ancla de `memory_node_id` y `ensure_lineage_edge` se movió unas líneas

`memory_node_id` está en `src/sdk/serialization/mod.rs:118` (el ADR cita `113-124`,
que es el rango del bloque) y exportado en `src/sdk/mod.rs:33` (`32-36`).
`ensure_lineage_edge` sigue exactamente en `ontology/mod.rs:262-302`. El contrato
`FIND-756` cita `l1_writer.rs:510-521` para el `put_batch` (`:507-521` hoy) y
`:459/:466` para `session_key`/`ns` (`:459/:466` exactos). **Nada material cambió**;
los rangos del contrato son correctos al ±2 líneas.

### 9.3 Lo que **sí** cambió respecto del contrato: hay un `session_key` más

El contrato dice "`session_key` no existe en el sitio 4" — cierto en la **firma**, pero
**sí** existe como local en `:793`. Ver §9.1. Es la única corrección que cambia una
línea de código respecto del snippet del contrato.

### 9.4 ADR-0066 D1b promete más de lo que esta tarea entrega (divergencia material)

El ADR justifica el sitio 5 así (`D1b`, §Decision outcome):

> *"A contract that covers only `put_record_with_options` therefore leaves every L1
> record written by the pipeline unlinked while the tests pass — the exact drift this
> decision exists to end."*

Esa frase es **correcta sobre el bug y optimista sobre el fix**. Medido hoy:

- el pipeline escribe `ExtractedMemory.source_message_ids` = **turnos L0**
  (`l1_writer.rs:213`);
- un turno L0 vive en `l0/<session>` (`l0_recorder.rs:363`) y `l0_namespace` es
  privada (`:556`);
- la proyección de §2.1 manda el padre a `l1/<session>/<turno>` ⇒ el guard no lo
  encuentra ⇒ **0 aristas**.

Conclusión: **implementar el sitio 5 no enlaza el pipeline de producción.** Cierra el
*drift estructural* (el lote ya intenta crear aristas, igual que el camino por-record)
pero no el *outcome*. A11b (0 aristas) es la aserción que lo dice en voz alta.

Esto no invalida la decisión de ADR-0066 — sigue siendo la proyección correcta, y el
sitio 5 es imprescindible para que el día que se exporte `l0_namespace` el pipeline
funcione sin un segundo fix. Lo que invalida es **cerrar la fila creyendo que el
pipeline quedó enlazado**. El ADR necesita una línea en D1b reconociéndolo; este
fichero no puede editar el ADR.

---

## §10 — Deuda declarada (no la resuelvas acá)

Ya tiene fila en el Backlog; **no** edites `docs/dev/Backlog.md`.

| Deuda | Dónde |
|---|---|
| Sin backfill: los records escritos antes del fix no ganan aristas hasta reescribirse. Coste no medido | `FIND-772` (`docs/dev/Backlog.md`, "Media 2026-11-26") |
| `add_edge` es get-modify-insert sin lock; este fix lo deja vivo (dos `add_edge` por arista y por rewrite) | `FIND-773` (`docs/dev/Backlog.md`, "Alta 2026-11-26") |
| Proveniencia a nivel mensaje: un turno L0 **es** un record, pero `l0_namespace` es privada (`l0_recorder.rs:556`) ⇒ A11b sigue en 0 | ADR-0066 §Relation to FIND-296; registrado aquí en A11b |
| Churn: cada rewrite borra y recrea sus aristas (D3). Vía de escape = O5 del ADR | ADR-0066 §Negative consequences |
| `sync_persona_to_scope` no recrea la arista si el body no cambió (§3 trampa 7) | Misma clase que `FIND-772` |

Cuando cierres la fila `FIND-756` del Backlog, cita las aserciones del grafo, **no** la
ausencia de `Node not found` en el log.

---

## §11 — ORDEN RECOMENDADO entre los dos contratos

Los dos task files de esta entrega: **`FIND-756-impl.md` va primero.**

> **Esto invierte la recomendación que este fichero traía antes de la review P2-01.** El
> motivo está medido, no es una preferencia: `MCP-43` **ya aterrizó**
> (`vantadb-mcp/src/memory_scheduler.rs`, 664 líneas, en `HEAD`), así que `WIRE-19` ya
> no desbloquea nada — lo único que le queda es **reparar el drift que `MCP-43` dejó**
> (tres copias de la resolución del TOML en vez de dos). Detalle completo en
> `WIRE-19-impl.md` §8.0.

| Criterio | FIND-756 | WIRE-19 |
|---|---|---|
| Ficheros en común con el otro | **0** | **0** |
| Decisiones abiertas | **0** | **1** (el default de intervalo del `--mcp`: 5 s enviado vs "off" del ADR D4 — ver `WIRE-19-impl` §8.0) |
| Tamaño | ~8.25 h, 6 sitios, 13 aserciones | ~1.5 d, **9** filas |
| Desbloquea | `FIND-296`/`MEMG-03` parcial | **nada**: `MCP-43` ya está hecho |
| Riesgo si espera | bajo | bajo (el drift ya existe y ya está en producción) |

**Razón 1 — no hay conflicto que ordene.** Cero ficheros compartidos: `FIND-756` toca
`src/core/{scene,persona,record,profile,ontology}` + `tests/lineage_graph.rs`;
`WIRE-19` toca `src/ingest/`, `src/services/` y cuatro ficheros de `vantadb-server` /
`vantadb-mcp`. El orden lo decide el valor, no el *merge*.

**Razón 2 — uno está listo y el otro no.** `FIND-756-impl` no tiene ninguna decisión
pendiente: cada paso es mecánico y cada aserción es falsable. `WIRE-19-impl` tiene una
—qué pasa cuando `VANTADB_SCHEDULER_INTERVAL_MS` no está definida en el `--mcp`— que
cambia el comportamiento de un host ya enviado y que **no me corresponde decidir en un
task file**. Empezar por el que no bloquea en una decisión es el uso correcto del
tiempo; y no hay penalización por dejarlo para después.

**Razón 3 — el valor del segundo ya no es el que lo justificaba.** `WIRE-19` se escribió
para habilitar `MCP-43`. `MCP-43` se construyó su propia copia y ya está en `HEAD`: hoy
`WIRE-19` vale por *higiene* (una regla en lugar de tres) y por cerrar la divergencia
`InMemory`/CWD en el camino de producción del `--mcp` (`threads.rs:367`). Eso es real y
urgente-como-deuda, pero no compite con el bug visible del linaje.

**Contra-argumento, para que el owner pueda invertirlo con conocimiento:** el drift de
`WIRE-19` **está vivo ahora** y el de `FIND-756` también (el linaje falta desde
`FIND-747`); si lo que pesa es "el que más se degrada si espera", `WIRE-19` gana porque
cada host nuevo copia otra vez (ya pasó con `MCP-43`). No hay dependencia técnica entre
ambos, así que invertir el orden no rompe nada — sólo cambia qué se arregla primero.
