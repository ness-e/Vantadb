---
title: "ADR-032: Persistencia on-disk de vectores Binary/Turbo/SQ8 en vstore"
type: adr
status: accepted
tags: [vantadb, architecture, adr, storage, persistence, vstore, binary, quantization]
created: 2026-08-28
last_reviewed: 2026-08-28
related: [ADR-019-sparse-vector-persisted-format, ADR-020-storage-backend-default, ADR-022-wal-batch-async, DRV-014-wal-batch-tradeoff]
---

# ADR-032: Persistencia on-disk de vectores Binary/Turbo/SQ8 en vstore

## Context

`src/storage/ops.rs:59` (`write_node_to_vstore`) solo persistía `VectorRepresentations::Full` como `f32` LE (`vector_len = N, bytes = N*4`). Cualquier otra variante (`Binary(Box<[u64]>)`, `Turbo(Box<[u8]>)`, `SQ8(Box<[i8]>, f32)`, `MmapFull`, `None`) escribía `vector_len = 0` y **cero bytes de payload**. El header `DiskNodeHeader` (64 B, `src/node/disk.rs`) quedaba con `vector_len = 0` indistinguible de "sin vector".

`src/storage/engine/get.rs:206-211` rescataba el vector para `Binary`/`SQ8` desde el `HnswNode.vec_data` en memoria (fix `8c8eef23`): tras leer un `N=0` vacío desde vstore, sobre-escribía `node.vector` con la copia del HNSW. Esto hacía que `put(Binary) → get()` funcionara **en memoria**.

El gap es de durabilidad: `src/storage/archive.rs:237-284` (`rebuild_hnsw_from_vstore`) reconstruye el índice escaneando vstore. Con `vector_len = 0` lee `None` y **pierde** el vector cuantizado. Tras `flush + reopen` (índice HNSW vacío → rebuild) el `Indexed vectors` cuenta 0 y `search` no encuentra el registro. WAL replay (`src/storage/engine/init.rs:391` → `replay_write_node` → `write_node_to_vstore`) reproduce el mismo error, por lo que un crash con WAL aún no checkpointeado también deja el vector perdido si el HNSW file no estaba checkpointeado.

Las variantes cuantizadas existen porque `VectorRepresentations` declara 6 estados (vector_data.rs:81-97) y su serialización en el **fichero HNSW** (`src/index/serialize/bytes.rs:151-179`) ya persiste los 5 tipos (Full=1, Binary=2, Turbo=3, SQ8=4, None=0) de forma estable. El HNSW file es durable a través de `flush → save_vector_index`, por lo que hoy los vectores `Binary` sobreviven al reinicio vía el HNSW. Se pierden **solo** cuando el HNSW file falta/corrupto y se entra a rebuild, o cuando el operador fuerza `rebuild_vector_index()`.

El contrato CORE-01 exige que `insert Binary → flush → reopen → get/search` sea durable sin depender del HNSW file como única copia.

### Invariantes

1. API pública `VantaMemoryRecord.vector: Option<Vec<f32>>` no cambia — `Binary`/`Turbo`/`SQ8` son representaciones internas del nodo (`UnifiedNode.vector`) derivadas por cuantización/gobernador; persisten en vstore pero el SDK materializa `vector` como `Full` o `None` según corresponda (SQ8/Binary se exponen vía HNSW similarity, no como `VantaMemoryRecord.vector`).
2. No se rompe compatibilidad de lectura de ficheros existentes (VFILE v1/v2). Ficheros viejos con `flags[10:13]=0` y `vector_len>0` se interpretan como `Full` (comportamiento actual).
3. `DiskNodeHeader` mantiene 64 B — no se cambia su tamaño ni orden de campos (zero-copy `zerocopy`).

## Decision

Persistir **todas** las variantes de `VectorRepresentations` en `VantaFile` usando 4 bits del campo `flags` del `DiskNodeHeader` para codificar el **kind**, y reinterpretando `vector_len`.

### 1 — Nuevo campo in-header: `VECTOR_KIND` en `flags`

Bits `10–13` de `header.flags` (`u32`) codifican el kind (mask `0x3C00`, shift 10). Bits `0–9` siguen siendo los `NodeFlags` (`ACTIVE`→`CONFLICT_RESOLVED`). Bits `14–31` libres para futuro.

| Kind | Valor | Variante Rust | `vector_len` (u32) | Payload on-disk (bytes) | Descripción |
|------|-------|---------------|--------------------|--------------------------|-------------|
| NONE | 0 | `None` | 0 | 0 | sin vector |
| FULL | 1 | `Full(Vec<f32>)` | N = `v.len()` | `N*4` LE `f32` | denso |
| BINARY | 2 | `Binary(Box<[u64]>)` | M = `b.len()` | `M*8` LE `u64` | RaBitQ 1-bit, `dims = M*64` |
| TURBO | 3 | `Turbo(Box<[u8]>)` | K = `t.len()` | `K` bytes | PolarQuant 4-bit empaquetado, `dims = K*2`, padded a 8 en memoria pero en vstore se guarda exacto K |
| SQ8 | 4 | `SQ8(Box<[i8]>, f32)` | N = `d.len()` | `N` bytes `i8` + 4 bytes `scale` LE `f32` | dims = N |

`vector_offset` sigue siendo `offset + 64` (payload empieza tras el header). El bloque completo (header 64 + payload) se alinea a 64 B (`write_cursor = (offset+64+payload_len+63) & !63`), igual que antes — por lo que todo `vector_offset` es 64-alineado (satisface 4- y 8-alineado).

`VFILE_VERSION` permanece `2`. Archivos legacy tienen `kind=0`. Regla de lectura legacy: si `kind==0` y `vector_len>0` → tratar como `FULL` (compat). Si `kind==0` y `vector_len==0` → `None` (legacy `Binary` con `len=0` se vuelve `None` solo en el path de rebuild; para `get()` en ficheros legacy se mantiene el rescue desde HNSW — ver §3).

### 2 — Write path (`write_node_to_vstore`)

```rust
let (kind, vec_len, payload_bytes): (u32, usize, &[u8]) = match &node.vector {
  Full(v) => (FULL, v.len(), v.as_bytes()),
  Binary(b) => (BINARY, b.len(), bytemuck::cast_slice(b)),
  Turbo(t) => (TURBO, t.len(), t),
  SQ8(d, scale) => { /* payload = d bytes + scale LE */ },
  MmapFull(opt) => { // materializar como FULL vía as_f32_slice
  },
  None => (NONE, 0, &[]),
};
header.vector_len = vec_len as u32;
header.flags = (node.flags.0 & !MASK) | (kind << SHIFT);
vstore.write_header(offset, &header)?;
copy payload_bytes (+ scale si SQ8) al mmap
```

Se valida `vec_len <= u32::MAX` y `payload_len <= vstore.size`. Antes de crecer, se precalcula `total_needed = offset+64+payload_len`.

### 3 — Read paths

`get()` / `get_many()` / `get_with_snapshot()` / `rebuild_hnsw_from_vstore` / `compact_layout` / `search_layer` decodifican `kind = (header.flags & MASK) >> SHIFT`:

- `FULL`: `bytes = vector_len*4`, validar `vector_offset+bytes <= mmap.len()`, `align_to::<f32>()`, `Full(vec.to_vec())`.
- `BINARY`: `bytes = vector_len*8`, `align_to::<u64>()`, `Binary(b.to_vec().into_boxed_slice())`.
- `TURBO`: `bytes = vector_len`, `Turbo(bytes.to_vec().into_boxed_slice())`.
- `SQ8`: `bytes = vector_len + 4`, últimos 4 = scale `f32 LE`, primeros `vector_len` bytes = `i8`, `SQ8(data, scale)`.
- `NONE`: `None`.
- Legacy `kind==0`: fallback — si `vector_len>0` leer como `FULL` (read Full); si `0` leer como `None` y, en `get*`, rescatar `Binary`/`SQ8` desde `HnswNode.vec_data` si existe (mantiene compat de ficheros viejos en el path de lectura caliente; rebuild no rescata — ver §5).

Toda lectura valida `checked_mul` y `checked_add` y `end <= mmap.len()`. Corrupción → `None`/`0.0`/`tombstone skip` sin panic (existente `ERR-029`/`AUDREP-45` style).

### 4 — Compact / maintenance

`compact_layout` computa `payload_len` y `vec_size_aligned = (payload_len+63)&!63` a partir de `kind`, no asumiendo `*4`. Copia `header_size + aligned` bytes idéntico al layout de escritura, preservando `kind`.

`rebuild_hnsw_from_vstore` reconstruye `VectorRepresentations` nativa (no `Full` forzado) para que el HNSW rehidratado conserve la cuantización y las distancias (`SQ8`→`SQ8` no necesita `promotion`).

`search_layer` cuando `vector_store.is_some()` ahora despacha por `kind`:
- `FULL` → `metric_score(f32_vec, ...)` (actual)
- `BINARY` → `rabitq_similarity` (Hamming) mapeada a cosine
- `TURBO` → `turbo_quant_similarity`
- `SQ8` → `sq8_similarity`
- `NONE`/legacy empty → `0.0` (fallback a `fast_similarity` si no hay `vs` se mantiene).

Se mantiene `prefetch` tamaño por `payload_len`.

### 5 — Migración / versionado

- Escritura nueva: siempre escribe `kind` explícito (FULL incluso para f32) → ficheros nuevos son auto-descriptivos.
- Lectura vieja: `kind==0` + `vector_len>0` → FULL (correcto). `kind==0` + `vector_len==0` + `HnswNode` con `Binary` → `get()` rescata (compat caliente), pero `rebuild` producirá `None` (los `Binary` legacy no están en vstore). Se documenta como **limitación conocida**: un `rebuild_vector_index()` forzado sobre una DB creada antes de ADR-032 perderá los vectores `Binary` legacy no re-escritos. La mitigación es **lazy migration**: el próximo `put`/`update` de ese key re-escribe el vstore entry con `kind`. Operadores no deben forzar rebuild hasta que los keys críticos hayan sido tocados, o deben mantener el `vector_index.bin` como fuente primaria (flujo normal de `flush` lo mantiene). No se introduce migración one-shot ni bump de `VFILE_VERSION`; el write path dual no es necesario (solo existe vstore como fuente de verdad para nuevo).

- Future bump: si se requiere romper compat (e.g. nuevo kind >4), se bump `VFILE_VERSION` a 3 y se mantiene reader dual para v2 como ahora.

## Consequences

### Pros

- Elimina el durability gap de `Binary`/`Turbo`/`SQ8` con el mínimo cambio de formato (0 bytes extra de header, 4 bits en `flags`, `vector_len` reinterpretado). `rg -n "vector_len.*0" src/storage/ops.rs` pasa a 0 tras el fix (ya no hay `else {0}`).
- `rebuild_hnsw_from_vstore` y `WAL replay` se vuelven fieles al HNSW file para todos los tipos — el índice puede reconstruirse desde vstore sin pérdida.
- `cargo nextest run -p vantadb --profile audit -E 'test(persistence|vstore|rebuild)'` incluye roundtrip `Binary persist→flush→reopen→get/search`; pasa sin `#[ignore]`.
- Cero bump de versión de fichero, compat de lectura total (ficheros viejos leídos como `Full`). Coste: +`N+4` bytes para `SQ8` (scale) y `M*8` para `Binary` — despreciable frente a `M*4` previo no guardado.
- Aísla la decisión en 3 ficheros (`flags.rs`, `ops.rs`, `archive.rs`) + readers (`get.rs`, `txn.rs`, `search/layer.rs`) — sin tocar `DiskNodeHeader` tamaño.

### Cons

- Dos fuentes de verdad para la escala `SQ8` (header vs payload tail). Elegimos payload tail para no consumir `confidence`/`importance` ni expandir header, pero obliga a parsear tail en cada read.
- Legacy `Binary` (`vector_len=0, kind=0`) permanece irrecuperable por rebuild hasta reescritura lazy — aceptado porque esa data nunca fue durable en vstore y el HNSW file sí la tiene (backup vía `vector_index.bin`). Se documenta como riesgo en `Risk Register`.
- `compact_layout` y `rebuild` ahora tienen `match kind` (+~30 líneas) — complejidad local aceptable.

### Alternativas consideradas

| Opción | Veredicto |
|--------|-----------|
| Nuevo campo `u8 vector_kind` en `DiskNodeHeader` (`_pad: [u8;1]` → `kind: u8`) | ❌ Cambia tamaño/layout de header si `_pad` se interpreta; ficheros legacy con `_pad=0` se leerían como `NONE` incluso para `Full` (precisa migración). Requiere bump `VFILE_VERSION` y reader dual más intrusivo que los 4 bits en `flags` |
| Usar solo `vector_len == 0` como centinela + key relacional `__vanta_vector_kind` (pattern ADR-019 sparse) | ❌ Mueve el tipo al KV backend (`NodeMetadata`) — rebuild escanea solo vstore, no backend, por lo que no lo vería; además añade un `get` por nodo en el scan |
| No tocar vstore, confiar solo en `vector_index.bin` (status quo) | ❌ Deja el gap de durability abierto para el path `hnsw.nodes.is_empty() → rebuild` y documenta que la única copia es el índice — viola el contrato de que vstore es la fuente de reconstrucción |
| Variable `header.relational_len` o `tier` para kind | ❌ `relational_len` reserve para metadata relacional, `tier` domina cold/hot — sobrecarga semántica |

## Riesgos

- **R1 — Flags collision:** `node.flags` no tenía kind; escribir `kind` en header y rehidratar `node.flags = header.flags & !MASK` evita leakeo a `NodeFlags`. Test debe verificar que `HAS_VECTOR` y `TOMBSTONE` sobreviven al roundtrip con kind.
- **R2 — Misaligned `vector_offset`:** `read_header` rechaza `vector_offset % 4 != 0` (INV-024). Nuevo payload para `Binary` necesita 8-aligned; el write path garantiza `offset+64` (64-aligned) → satisface. `TURBO` payload de 1 byte sigue 64-aligned por header, ok.
- **R3 — `u32::MAX` `vector_len` overflow:** `checked_mul` ya usado para `*4`; extender a `*8` y `+4` con `checked_*`. `MAX_PERSISTED_NODE_BYTES` (128 MiB) limita el desastroso.
- **R4 — `SQ8` scale NaN/Inf:** scale `f32` es siempre `max_abs` finito (guard `< EPSILON → 1.0`). No se persiste NaN; lectura de scale NaN → tratar como `1.0` y log warn, no panic.

## Referencias

- `src/storage/ops.rs:59-109` write path previo (gap `vector_len=0` para Binary/Turbo/SQ8)
- `src/node/disk.rs:11-34` `DiskNodeHeader` 64 B
- `src/node/flags.rs:21-42` `NodeFlags` (bits 0-9 usados)
- `src/node/vector_data.rs:81-97` `VectorRepresentations` enum
- `src/index/serialize/bytes.rs:151-179` serialización HNSW ya multi-typed
- `src/storage/archive.rs:237-284` rebuild previo solo `Full`
- `src/storage/engine/get.rs:167-227` get + rescue Binary
- Plan `docs/plans/2026-08-27-backlog-v2.md` Task 4 CORE-01 (contrato: `rg vector_len.*0` 0 + nextest + ADR)
- ADR-019 sparse persisted format (precedente: `ListFloat` sin bump, legacy compat con String)

