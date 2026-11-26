---
title: "FIND-756 - los extremos del linaje son los nodos de la memoria: `memory_node_id(namespace, key)`, sin crear nodos"
kind: task
status: proposed
description: "`scene:`/`persona:`/`episode:`/`record:` son un espacio de ids fantasma que nada materializa; el fix los proyecta a `memory_node_id(ns, key)` — la dirección que `put` ya asigna a todo record — en 5 sitios, sin tocar `src/`, sin re-index y sin migrar aristas"
tags: [vantadb, vanta-memory, graph, contract-first]
---

# FIND-756 - los extremos del linaje son los nodos de la memoria

Decisión de arquitectura: [ADR-0066](../architecture/adr/ADR-0066-lineage-endpoints-are-memory-record-nodes.md).
Origen: `FIND-747` (2026-11-26). Este archivo es el **contrato**; la fila del
Backlog la cierra quien implemente, no el arquitecto.

## Contexto (verificado leyendo el código, 2026-11-26)

`FIND-747` dejó cuatro hechos medidos. Reconfirmados aquí:

1. `add_edge` exige **ambos** extremos como nodos vivos: `engine.get` +
   `Error::NodeNotFound` en el origen (`src/sdk/api/graph.rs:129-131`) y en el
   destino (`:141-143`).
2. Lo que falta es el **origen** (escena, persona, record), no el padre.
3. Esos ids salen de `entity_node_id`
   (`vanta-memory/src/core/ontology/mod.rs:45-52`): FNV-1a sobre un string en
   minúsculas. **Hashea, no crea.**
4. Las únicas `insert_node` de producción del crate materializan nodos-entidad
   de tripletas (`ontology/mod.rs:221,230`). `entity_node_id` fuera de
   `vanta-memory`: **0 hits** (grep `*.rs`, 25 coincidencias, todas dentro).

⇒ `scene:<s>:<n>`, `persona:<s>:<k>`, `episode:<s>:<id>`, `record:<s>:<id>`
nombran nodos que ningún código escribe. **No es un bug de orden**: es una
representación de identidad equivocada. Las entidades existen y ya tienen
dirección, porque todo record **es** un nodo:

- `put` → `put_one` (`src/sdk/api/memory.rs:971`) → `insert_one_internal`
  (`:980`) calcula `node_id = memory_node_id(namespace, key)` (`:1143`) y hace
  `engine.insert` (`:1234`).
- `memory_node_id = XxHash3_128(ns ++ "\0" ++ key)`
  (`src/sdk/serialization/mod.rs:113-124`), **ya exportado** a crates hermanos
  (`src/sdk/mod.rs:32-36`).

## Decisión, en una frase

**Los extremos del linaje se direccionan con `memory_node_id(namespace, key)`.
Nadie crea nodos: `put` ya los creó. La arista se crea después del `put`, en la
misma función. `entity_node_id` sobrevive sólo para las tripletas.**

Descartado, con una razón cada uno (detalle en el ADR):

| Opción | Por qué no |
|---|---|
| Materializar los ids fantasma con `insert_node` | dos direcciones para un mismo hecho; el nodo-entidad queda huérfano cuando `cascade_forget` borra el record |
| Arreglar sólo el extremo que falta | con un extremo roto no hay arista: `add_edge` sigue en `NodeNotFound` |
| Borrar las aristas y leer el linaje del metadata | `graph_bfs` no recorre metadata; borra una capacidad ya entregada |
| `derived_from` con padres cualificados y `put` como único dueño | cambio de campo persistido + formato de export + migración on-disk, para un bug de 5 líneas (es la vía de escape si el churn molesta) |
| Etiqueta distinta para el linaje cross-namespace | dos nombres para `LABEL_DERIVED_FROM`; los consumidores de `derived_from` pierden la mitad |
| API nueva en `src/`: `add_memory_edge(from_ns, from_key, to_ns, to_key, …)` | no quita el riesgo: el bug nunca fue el hash sino **qué par (ns,key)** se pasa, y el caller lo sigue pasando; añade superficie pública nueva y una segunda ortografía de `add_edge` |
| Cambiar tipo u orientación de arista | la orientación ya es la canónica (hijo → padre, `graph.rs:20-27`); lo roto era el id, no el ancla |

---

## CONTRATO PARA EL IMPLEMENTADOR

### Archivos que se tocan (y ninguno más)

| Archivo | Qué cambia |
|---|---|
| `vanta-memory/src/core/scene/scene_index.rs` | proyección de `scene_node_id` y `ep_node_id` dentro de `write_scene_block` |
| `vanta-memory/src/core/persona/persona_generator.rs` | proyección de `persona_node_id`, `ep_node_id`, `scene_node_id` en `write_persona` |
| `vanta-memory/src/core/record/l1_writer.rs` | proyección de `node_id` y `parent_node_id` en `put_record_with_options` **y** el post-loop de `apply_dedup_batch` (**el path que usa el pipeline**) |
| `vanta-memory/src/core/profile/profile_sync.rs` | **añadir** la arista que hoy no existe en `sync_persona_to_scope` |
| `vanta-memory/src/core/ontology/mod.rs` | idempotencia en `ensure_lineage_edge`: no duplicar una arista que ya existe (D7 del ADR) |
| `vanta-memory/tests/lineage_graph.rs` | **nuevo**, las aserciones de abajo |
| comentarios `FIND-747` de los sitios | reescribirlos: describen un skip declarado que deja de ocurrir |

**No se toca `src/**`.** Si el implementador cree que necesita tocar `src/`,
para y vuelve al ADR.

### Los seis sitios, y por qué son seis

| # | Función | Camino |
|---|---|---|
| 1 | `scene_index::write_scene_block` | L2, escena |
| 2 | `persona_generator::write_persona` | L3, persona |
| 3 | `profile_sync::sync_persona_to_scope` | L3, persona con scope |
| 4 | `l1_writer::put_record_with_options` | L1 record por-record (`put_l3_axiom`, `write_memory`, `rollback`) |
| 5 | `l1_writer::apply_dedup_batch` | **L1 por lote — el que usa el pipeline** (`pipeline_worker.rs:614`, `l1_dedup.rs:236`, `approval.rs:137`) |
| 6 | `scene_index`/`persona` sobre `derived_from` **de campo** | el `put` ya la crea; no hay que añadir nada, sólo no duplicarla (D7) |

El sitio 5 es **bloqueante**: `apply_dedup_batch` construye `MemoryInput` con
`record_input` y llama `db.put_batch(inputs)` (`l1_writer.rs:510-521`); no pasa
por `put_record_with_options`, y `put_batch_inner` sólo reconcilia los padres
declarados en el **campo** del nodo (`memory.rs:1638-1705`). Un contrato que
cubra sólo el sitio 4 deja **todos** los L1 del pipeline sin arista y el test
pasa igual. Ya tiene `session_key` (`:459`), `ns` (`:466`) y los records
planeados: el post-loop es directo.

### La proyección exacta (copiable)

Reusa **las variables locales que ya existen** en cada función; no recalcules
la clave, porque recalcularla mal es de lo que trata este bug.

> **Trampa de ownership, medida:** en `write_scene_block`, `write_persona` y
> `sync_persona_to_scope` el `ns` se **mueve** al `MemoryInput`
> (`namespace: ns`, `scene_index.rs:219`, `persona_generator.rs:463`,
> `profile_sync.rs:165`). Por eso el id del hijo se calcula **antes** del
> `put`, y sólo la creación de la arista va después. `key` sí sobrevive en los
> dos primeros (`key: key.clone()`), pero no en `profile_sync` (`key,`), así que
> ahí también se calcula antes. En `l1_writer` `ns` es `&str` y no hay problema.
>
> **`session_key` no existe en el sitio 4.** `put_record_with_options` recibe
> `(db, ns, record, vector, confidence_override, dedup_policy_override)`
> (`l1_writer.rs:767-774`): el namespace de los padres sale de
> `record.session_key` (`MemoryRecord.session_key`,
> `core/abstractions/types.rs:108`). Pasar `l1_namespace(ns)` duplicaría el
> prefijo (`l1_reader.rs:25-27`) y el guard se saltaría la arista **en
> silencio**.

```rust
// scene_index.rs :: write_scene_block
// ANTES del db.put (ns se mueve al MemoryInput; el hash no depende del put):
let scene_node_id = vantadb::sdk::memory_node_id(&ns, &key);

db.put(MemoryInput { .. })?;

// DESPUÉS del put (D3: la arista va después, siempre):
let now_ms = crate::core::conversation::now_ms();
let l1_ns = crate::core::record::l1_reader::l1_namespace(session_key);
for ep_id in &block.source_episode_ids {
    let ep_node_id = vantadb::sdk::memory_node_id(
        &l1_ns,
        &crate::core::conversation::sanitize_key(ep_id),
    );
    crate::core::ontology::ensure_lineage_edge(db, scene_node_id, ep_node_id, "derived_from", now_ms);
}
```

```rust
// persona_generator.rs :: write_persona
// ANTES del db.put (ns se mueve):
let persona_node_id = vantadb::sdk::memory_node_id(&ns, &key);

db.put(MemoryInput { .. })?;

// DESPUÉS del put:
let now_ms = crate::core::conversation::now_ms();
let l1_ns = crate::core::record::l1_reader::l1_namespace(session_key);
let scene_ns = crate::core::scene::scene_index::scene_namespace(session_key);
for ep_id in &record.source_episode_ids {                 // padre: episode
    let ep_node_id = vantadb::sdk::memory_node_id(&l1_ns, &crate::core::conversation::sanitize_key(ep_id));
    crate::core::ontology::ensure_lineage_edge(db, persona_node_id, ep_node_id, "derived_from", now_ms);
}
for scene_name in &record.source_scene_names {            // padre: scene
    let scene_node_id = vantadb::sdk::memory_node_id(&scene_ns, &crate::core::conversation::sanitize_key(scene_name));
    crate::core::ontology::ensure_lineage_edge(db, persona_node_id, scene_node_id, "derived_from", now_ms);
}
```

```rust
// l1_writer.rs :: put_record_with_options   (ns es &str, no se mueve)
let node_id = vantadb::sdk::memory_node_id(ns, &crate::core::conversation::sanitize_key(&record.id));
// NO hay `session_key` en esta firma (:767-774): sale del record.
let l1_ns = crate::core::record::l1_reader::l1_namespace(&record.session_key);
for src_id in sources {                                   // source_episode_ids ∪ source_message_ids, sin separar
    let parent_node_id = vantadb::sdk::memory_node_id(&l1_ns, &crate::core::conversation::sanitize_key(&src_id));
    crate::core::ontology::ensure_lineage_edge(db, node_id, parent_node_id, "derived_from", now_ms);
}
```

```rust
// l1_writer.rs :: apply_dedup_batch   (SITIO 5 — el que usa el pipeline)
// Ya existen `ns` (:466) y `session_key` (:459). Va DESPUÉS de db.put_batch(:520),
// porque el put_batch reconcilia el campo y borraría la arista.
let now_ms = /* el mismo now_ms del flush */;
let l1_ns = crate::core::record::l1_reader::l1_namespace(session_key);
for planned in &planned_writes {
    let node_id = vantadb::sdk::memory_node_id(&ns, &crate::core::conversation::sanitize_key(&planned.record.id));
    for src_id in planned_provenance_ids(planned) {        // mismas fuentes que hoy arma el :796-812
        let parent_node_id = vantadb::sdk::memory_node_id(&l1_ns, &crate::core::conversation::sanitize_key(&src_id));
        crate::core::ontology::ensure_lineage_edge(db, node_id, parent_node_id, "derived_from", now_ms);
    }
}
```

```rust
// profile_sync.rs :: sync_persona_to_scope
// ANTES del db.put (ns y key se mueven):
let profile_node_id = vantadb::sdk::memory_node_id(&ns, &key);

db.put(MemoryInput { .. })?;

// DESPUÉS del put:
let now_ms = crate::core::conversation::now_ms();
let l1_ns = crate::core::record::l1_reader::l1_namespace(session_key); // el parámetro de la fn, no un campo de PersonaRecord
for ep_id in &record.source_episode_ids {
    let ep_node_id = vantadb::sdk::memory_node_id(&l1_ns, &crate::core::conversation::sanitize_key(ep_id));
    crate::core::ontology::ensure_lineage_edge(db, profile_node_id, ep_node_id, "derived_from", now_ms);
}
```

Reglas que la copia debe respetar:

1. **`sanitize_key` en la clave del padre.** `record_input` guarda el record con
   `key: sanitize_key(&record.id)` (`l1_writer.rs:733`), así que los ids de
   episodio —que son ids de record— se sanean igual. Con el id crudo la arista
   se salta **en silencio**.
2. **`source_episode_ids` y `source_message_ids` se proyectan con la misma
   regla y sin separar** (`l1_writer.rs:796-812` tal cual). `put_l3_axiom` pone
   `source_message_ids = source_episode_ids` (`cascade_forget.rs:143`): los
   campos no son disjuntos. Un id que no sea record lo descarta el guard, que
   para eso está — hoy eso sólo pasa con turnos crudos, y los turnos L0 **sí**
   son records (`l0_recorder.rs:363`), así que apuntarlos es trabajo futuro, no
   una imposibilidad.
3. **La arista va después del `put`.** `insert_one_internal` corre
   `reconcile_lineage_edges` (`memory.rs:1245-1251`) y borra toda arista
   `derived_from` que el **campo del nodo** (`__vanta_derived_from`) no declare;
   en el camino primario estos records declaran por **metadata**, no por campo.
   Crear la arista antes del `put` = que el `put` la borre. (El **id** del hijo
   sí puede —y en tres de los cinco sitios debe— calcularse antes, porque el
   `ns` se mueve.)
3b. **No dupliques.** `ensure_lineage_edge` debe saltarse la creación cuando el
   nodo origen ya tiene la arista forward a ese target (D7 del ADR): `add_edge`
   **no** deduplica (`graph.rs:132-138,144-150`), y hay records que llevan la
   **misma** arista por dos productores (campo del nodo + metadata), además de
   los `put` que retornan temprano por dedup sin reconciliar.
4. **No añadir `insert_node`.** Si el test de abajo falla por `NodeNotFound`, la
   respuesta es la proyección, no crear el nodo.

### Aserciones (leyendo el **grafo**, no el log)

Archivo nuevo `vanta-memory/tests/lineage_graph.rs`. Patrón de apertura ya usado
en el crate: `Config { backend_kind: BackendKind::InMemory, .. }` +
`Embedded::open_with_config` (p. ej. `persona_generator.rs` tests). Las aristas
se leen de `db.get_node(id).unwrap().expect(..).edges`, que expone
`target: u128`, `label: String`, `reverse: bool`
(`src/sdk/serialization/graph_types.rs:9-26`).

Escenario base: sesión `"s1"`; dos records L1 con ids `m_a` y `m_b` escritos con
`put_record(db, &l1_namespace("s1"), ..)`; una escena `"alpha"` creada con
`upsert_scene_with_sources(db, "s1", "alpha", .., vec!["m_a".into(), "m_b".into()])`.

> **Visibilidad:** `sanitize_key` es `pub(crate)` (`l0_recorder.rs:267`, re-export
> `pub(crate) use` en `conversation/mod.rs:12`) y el `PERSONA_KEY` de
> `profile_sync` (`:70`) es privado y vale `"persona"` (≠ `"persona.md"` de
> `persona_generator.rs:49`). Un test de `vanta-memory/tests/` es un crate
> externo: **no** puede llamar a ninguno de los dos. Escribe las claves como
> literales del fixture (`"alpha"`, `"persona"`, `"persona.md"`, `"m_a"`), que ya
> son sanas.

| # | Aserción | Cómo |
|---|---|---|
| A1 | La escena tiene arista forward a cada episodio | `get_node(memory_node_id(scene/"s1", "alpha")).edges` contiene, por cada `ep`, una `e.target == memory_node_id(l1/"s1", ep) && e.label == "derived_from" && !e.reverse` |
| A2 | El episodio tiene la mitad reverse | `get_node(memory_node_id(l1/"s1", "m_a")).edges` contiene `e.target == <id escena> && e.label == "derived_from" && e.reverse` |
| A3 | Conteo exacto **tras cada escritura** | exactamente **2** forward con label `derived_from` en la escena, y **1** reverse por episodio; ninguna duplicada |
| A4 | Idempotencia real | repetir `upsert_scene_with_sources` con las mismas fuentes ⇒ el conteo de A3 no cambia (ni 3 forward, ni 2 reverse) |
| A4b | Idempotencia **sin** el drop del reconcile | arista ya presente en el nodo origen antes de la llamada (insertarla con `add_edge`, luego `ensure_lineage_edge`) ⇒ sigue habiendo **una**, no dos. Es la aserción que prueba 3b/D7; A4 sola pasa aunque el guard no deduplique, porque el `put` previo borra |
| A5 | Los ids fantasma **no** existen | `get_node(entity_node_id("scene:s1:alpha")).unwrap().is_none()` y lo mismo para `entity_node_id("episode:s1:m_a")` — el fix no materializa nada |
| A6 | Persona → episode y persona → scene | `generate_persona` es la entrada pública (`persona_generator.rs:221`; `write_persona` es privada) y necesita un `LlmRunner` de juguete — el stub `Fixed`/`NeverCalled` ya existe en `vanta-memory/tests/persona.rs:37,46`. Encadena: records L1 → `upsert_scene_with_sources(.., vec!["m_a".into()])` → `generate_persona` ⇒ dos aristas forward distintas desde `memory_node_id(persona/"s1", "persona.md")`, con los targets `memory_node_id(l1/"s1","m_a")` y `memory_node_id(scene/"s1","alpha")` |
| A7 | L1 record → episode | un record L1 escrito con `source_episode_ids = ["m_a"]` vía `put_record` ⇒ arista desde `memory_node_id(l1/"s1", <id literal del fixture>)` hacia `memory_node_id(l1/"s1","m_a")` |
| A8 | Scoped persona → episode (D5) | `sync_persona_to_scope` ⇒ arista desde `memory_node_id(profile_namespace(scope), "persona")` hacia el episodio |
| A9 | Guard intacto: padre ausente | escena con `source_episode_ids = vec!["no_existe"]` ⇒ **0** aristas forward, sin panic, sin nodo inventado (`get_node(memory_node_id(l1/"s1","no_existe"))` es `None`) |
| A10 | Guard intacto: episodio **ausente** | escribir `m_a`, crear la escena (arista existe), `forget("s1", Key("m_a"), Hard)` ⇒ `get_node(memory_node_id(l1/"s1","m_a"))` es `None`, el guard salta y no se crea arista; sin panic. **`Soft` no sirve**: reinserte el nodo con `invalid_at_ms` (`memory.rs:2081-2106`) y el guard lo acepta |
| A11 | **El path del pipeline** | un flush por `apply_dedup_batch` (no `put_record`) con `source_episode_ids = ["m_a"]` ⇒ la arista existe. Sin esta aserción, el sitio que el pipeline usa de verdad queda sin cubrir y todo lo demás pasa igual |
| A12 | Episodio `Soft` sigue siendo destino válido | tras `forget(.., Soft)`, `get_node` devuelve `Some` y la arista se crea: la existencia en el grafo y la visibilidad en recall son cosas distintas (ADR D9) |

**Ninguna aserción puede basarse en capturar `stderr` o en la ausencia de
`Node not found`.** El log lleva callado desde `FIND-747`: eso es precisamente
lo que hizo invisible este bug.

### Lo que NO debe cambiar (lista de regresión)

- `src/sdk/serialization/mod.rs:113-124` — la definición de `memory_node_id`.
  Tocarla invalida **todos** los node ids de **todas** las bases existentes.
- `src/sdk/api/graph.rs` — `add_edge`, `ensure_edge`, `LABEL_DERIVED_FROM`. Sin
  firma nueva, sin label nuevo, sin cambio de dirección.
- `vanta-memory/src/core/ontology/mod.rs:45-52` y `:216-238` — `entity_node_id`
  y las tripletas: sus nodos sí se materializan y su espacio FNV es coherente.
- `vanta-memory/src/core/ontology/mod.rs:262-302` — el guard de `FIND-747`
  conserva su precondición, su `debug!` con `missing_node` y su `false`; **sí**
  puede (y debe) saltarse la creación si la arista forward ya existe (3b/D7).
  Lo que no se toca es la comprobación de endpoints ni el logging.
- Payload y metadata de los records: sin campo nuevo en `MemoryRecord`, sin
  cambio en el formato de export/import.
- Node ids de los records: sin re-index, sin rebuild de HNSW, sin rebuild del
  índice de texto.
- `cargo check -p vantadb` sin diff en `src/`.

### Comandos de verificación

```bash
cargo test -p vanta-memory --test lineage_graph          # A1..A12 (A11 es el path del pipeline)
cargo test -p vanta-memory --test scene_strategy --test persona --test e2e_flow
cargo test -p vanta-memory                                # suite del crate
cargo clippy -p vanta-memory --all-targets -- -D warnings
cargo check -p vantadb                                    # `src/` sin diff
```

Hay tests existentes que tocan el área. Ninguno de estos dos **afirma el bug** y
ninguno cambia:

- `ontology/mod.rs:340-354` (`lineage_edge_skips_when_endpoints_are_not_nodes`) —
  usa ids que siguen sin ser nodos: pasa igual.
- `ontology/mod.rs:358-382` (`lineage_edge_created_when_both_endpoints_exist`) —
  movimiento directo de `ensure_lineage_edge`: pasa igual.

Lo que **sí** hay que reescribir son los comentarios "Guarded: … declared skip"
en `scene_index.rs:229-232`, `persona_generator.rs:473-475`,
`l1_writer.rs:789-791`: dejan de describir el camino normal.

### Deuda que este contrato deja abierta (declarada, no resuelta)

- **Sin backfill.** Los records escritos antes del fix sólo ganan sus aristas
  cuando se reescriben. No existe pase de backfill y su coste **no se midió**.
  Es la fila 1 de §Fila de Backlog propuesta.
- **Churn en cada rewrite.** `insert_one_internal` reconcilia `derived_from`
  contra el campo del nodo y borra las aristas cross-namespace; el `ensure`
  posterior las recrea. Estado final correcto, escrituras de más. Vía de escape:
  opción O5 del ADR.
- **`add_edge` es read-modify-write sin lock** (`src/sdk/api/graph.rs:129-151`).
  Hasta hoy no existía ninguna arista de linaje, así que el lost-update estaba
  dormido; este cambio lo deja vivo (dos `add_edge` por arista y por rewrite).
  Que dos pases puedan correr a la vez en un proceso **no se midió**: hay un
  solo `tokio::spawn` en `services/scheduler.rs:314` y `.vanta.lock` deja un
  escritor por base, pero el path on-demand de ADR-0065 puede llamar a un pase
  fuera del bucle. Es la fila 2.
- **Proveniencia a nivel mensaje sin arista.** Un turno L0 **es** un record
  (`l0_recorder.rs:363`), así que es direccionable con la misma regla, pero
  `l0_namespace` es privada (`:556`) y encadenarlos multiplica aristas. Fuera de
  alcance: trabajo futuro, no imposibilidad.

### Filas de Backlog propuestas (listas para pegar; el número lo asigna el lead)

No se editó `docs/dev/Backlog.md` en este entregable a propósito: el commit de
esta tarea es sólo el ADR y este archivo. Pegar bajo la sección
`## Media 2026-11-26`:

**Fila 1 — el backfill (la deuda visible):**

```markdown
| `FIND-___` | 🟡 Media | **Sin backfill: los records escritos antes de `FIND-756` no ganan sus aristas de linaje hasta que se reescriben** | `vanta-memory/src/core/{scene/scene_index,persona/persona_generator,record/l1_writer,profile/profile_sync}.rs` | 🟡 4-8h | 🟢 Baja | 🟡 Pendiente (2026-11-26) | `FIND-756` (ADR-0066) corrige la proyección de ids y las aristas se crean a partir de ahí, pero sólo en el camino de escritura: una base que ya pasó por L2/L3 tiene los records y no las aristas, y no hay pase que las reconstruya. **Coste no medido** (ni tiempo ni escrituras): no se escribió ni cronometró ningún backfill. Qué: un pase que recorra `l1/<s>`, `scene/<s>`, `persona/<s>`, `profile/*` y aplique `ensure_lineage_edge(db, memory_node_id(ns,key), memory_node_id(l1_namespace(s), sanitize_key(id)), "derived_from", …)` sobre las fuentes declaradas en metadata, idempotente y reanudable. · Por qué: sin él, el grafo de una base existente queda permanentemente a medias, y la diferencia entre "base nueva" y "base migrada" es invisible. · Nota: es el único ítem de `FIND-756` que necesita iterar datos en vez de tocarlos en el write path. | Origen: `FIND-756` (2026-11-26) | Ver: `FIND-756`, `ADR-0066` |
```

**Fila 2 — `add_edge` no atómico (deuda que este cambio activa):**

```markdown
| `FIND-___` | 🟡 Media | **`add_edge` es get-modify-insert sin lock: con el linaje ya creado, dos escritores concurrentes pierden aristas en silencio** | `src/sdk/api/graph.rs:129-151,186-…` | 🟡 1d | 🟡 Media | 🟡 Pendiente (2026-11-26) | Hasta `FIND-756`/ADR-0066 ninguna arista de linaje llegaba a crearse (el guard de `FIND-747` las saltaba todas), así que el lost-update de `add_edge` —`engine.get`, `push`, `engine.insert`, dos veces, sin candado— estaba dormido. Ahora se crean dos `add_edge` por arista y por rewrite, y el fallo pasa a ser posible: el segundo escritor reinserta un nodo leído antes del primero y borra la arista recién puesta. **No medido**: no se demostró que dos pases puedan solaparse en un proceso (hay un solo `tokio::spawn` en `services/scheduler.rs:314` y `.vanta.lock` deja un escritor por base, pero el path on-demand de ADR-0065 puede llamar a `run_pass` fuera del bucle). Qué: decidir entre un candado por nodo en `add_edge`/`ensure_edge`, el patrón `crossbeam`/CAS sobre `edges`, o serializar los pases; y demostrar el solape antes de elegir. · Por qué: una arista perdida no deja rastro — es exactamente el modo de fallo que `FIND-747` acaba de hacer visible. | Origen: `FIND-756` (2026-11-26) | Ver: `FIND-756`, `ADR-0066`, `ADR-0065` |
```
