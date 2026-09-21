# COMP-025: JSON Shredding — Dynamic Schema to Typed Columns

> **Ponytail scope:** Phase 1 = schema inference on insert + columnar storage + basic query integration.
> No intentar construir el sistema completo tipo ClickHouse (buckets, subcolumns, merges).
> ClickHouse y DuckDB references para inspiración, no para copiar.

## Metadata

- **Plan file:** `docs/plans/2026-07-28-compat-025-json-shredding.md`
- **Fuente:** `docs/Backlog.md:221`
- **Esfuerzo:** 🔴 2-3 sem (total) — Phase 1: 🟡 3-5 días
- **Prioridad:** 🟡 Medio
- **Tipo:** Rust Core + Storage
- **Turns estimados:** 30-60 (total) / 10-15 (Phase 1)
- **Estado:** ⬜ PENDING

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| **Callers** | `src/sdk/search/mod.rs` (filter queries), `src/engine.rs` (insert path) |
| **Callees** | `src/node.rs` (FieldValue, FilterBitset), `src/storage/mod.rs` (backend ops), `src/backend/mod.rs` (BackendPartition) |
| **Implicaciones** | Insert path se encarga de analizar metadata. No rompe queries existentes (shredding es opt-in vía FilterStrategy mejorada). Nuevo BackendPartition o reuse de payload_index. Sin migración de datos. |

## Contrato

```
cargo nextest run --profile audit --workspace --build-jobs 2 pasa
Y
ShreddedField::from_json_value() infiere tipos correctamente
Y
ShreddedRowStore::put()/get() persiste y recupera campos
```

## Investigation Notes

### Benchmarking approaches
- **ClickHouse v25.8**: Advanced shared data serialization con 6 archivos por bucket (structure, data, paths_marks, substreams, substreams_marks, substreams_metadata). Copia de datos en formato original. 58x más rápido, 3300x menos memoria para queries selectivas.
- **DuckDB**: Auto schema inference → STRUCT physical type (cada subcampo como columna tipada). Ideal para embedded analytics.
- **VantaDB context**: no necesitamos OLAP analytics sobre JSON. Necesitamos filtering eficiente sobre metadata de records. La integración natural es con `FilterStrategy::PreFilter` (selectividad < 1%).

### Rust crates for JSON
- `serde_json` — ya en dependencias (check Cargo.toml)
- `simd-json` — 2-4x faster parsing, compatible con serde
- `json_dotpath` — acceso a paths anidados tipo `user.address.city`

## Phase 1: Schema Inference + Columnar Storage

### Step 1: Add `ShreddedSchema` struct
- **Archivos:** Create `src/shred/mod.rs` (new module)
- **Acción:** Definir `ShreddedField` enum (I64, F64, Bool, String, Null) + `ShreddedSchema` (HashMap<String, ShreddedField>). Función `infer_field_type(value: &VantaValue) -> ShreddedField` que analiza el tipo del valor JSON.
- **Verify:** `cargo check -p vantadb`

### Step 2: Register module in lib.rs
- **Archivos:** `src/lib.rs` (o `src/vantadb.rs`)
- **Acción:** Agregar `pub mod shred;` y re-exportar tipos clave.
- **Verify:** `cargo check -p vantadb`

### Step 3: Add `ShreddedRowStore`
- **Archivos:** `src/shred/mod.rs`
- **Acción:** Struct `ShreddedRowStore` con métodos:
  - `put(node_id: u128, fields: &VantaMemoryMetadata, backend: &dyn StorageBackend)` — infiere schema, serializa cada campo como bytes tipados, guarda en backend
  - `get(node_id: u128, backend: &dyn StorageBackend) -> Option<ShreddedRow>` — recupera fields shreddeados
  - `delete(node_id: u128, backend: &dyn StorageBackend)` — limpia
  - Serialización: I64 → 8 bytes LE, F64 → 8 bytes LE, Bool → 1 byte, String → len-prefixed UTF-8
  - Key format: `shred::{node_id}` en BackendPartition::InternalMetadata (o nuevo partition)
- **Verify:** `cargo check -p vantadb`

### Step 4: Wire into insert path
- **Archivos:** `src/engine.rs` (método `put` o `add`)
- **Acción:** Cuando se inserta un record con metadata no vacía, llamar a `ShreddedRowStore::put()`. Usar `VantaMemoryMetadata` que ya existe. No bloquear si falla (best-effort).
- **Verify:** `cargo check -p vantadb`

### Step 5: Wire into filter path
- **Archivos:** `src/sdk/search/mod.rs` (en `bitset_from_filters` o `select_filter_strategy`)
- **Acción:** En `bitset_from_filters()`, si el field filter targeta un campo shreddeado, usar `ShreddedRowStore::get()` para match rápido en vez de escanear todos los records. Esto es un early-exit optimization.
- **Verify:** `cargo check -p vantadb`

### Step 6: Tests
- **Archivos:** `src/shred/mod.rs` (agregar `#[cfg(test)] mod tests`)
- **Acción:**
  1. `test_infer_field_types()` — verifica que `infer_field_type` detecta I64, F64, Bool, String correctamente
  2. `test_shredded_roundtrip()` — put → get → verify values
  3. `test_shredded_delete()` — put → delete → get → None
  4. `test_shredded_schema_evolution()` — put con field set A, put con field set B (nuevos campos), verifica schema merge
- **Verify:** `cargo test -p vantadb --lib -- shred::tests`

### Step 7: Module doc + README cross-ref
- **Archivos:** `src/shred/mod.rs` (doc comment), `docs/api/EMBEDDED_SDK.md` (cross-ref)
- **Acción:** Documentar formato de serialización y cuándo se activa el shredding. Agregar cross-ref en MPTS.
- **Verify:** `cargo doc --no-deps -p vantadb` + revisión visual

## Phase 1 — ✅ COMPLETED (2026-07-28)

8 tests. Schema inference + columnar storage + filter fast path (equality only). Verificado:

```
cargo check -p vantadb ✅
cargo test -p vantadb --lib -- shred::tests — 8/8 pass ✅
```

## Phase 2 — ✅ COMPLETED (2026-07-28)

Typed comparison filters already implemented — no code changes needed. `matches_shredded` ya aceptaba `op: &RelOp` y soportaba los 6 operadores. 13 tests en total (8 de Phase 1 + 5 de Phase 2). Verificado:

```
cargo test -p vantadb --lib -- shred::tests — 13/13 pass ✅
cargo check -p vantadb ✅
```

### Resumen de implementación existente
- `matches_shredded(field, op, expected)` en `src/shred/mod.rs:113-141` — match completo con 15 arms
- RelOp enum en `src/query.rs:130-143` — 6 variantes (Eq, Neq, Gt, Lt, Gte, Lte)
- `bitset_from_filters` en `src/sdk/search/mod.rs:383` pasa `&RelOp::Eq` (API pública usa equality)
- 4 tests unitarios de comparación: `test_matches_shredded_i64_comparisons`, `test_matches_shredded_f64_comparisons`, `test_matches_shredded_bool_eq_ne`, `test_matches_shredded_string_eq_ne`
- 1 test de integración: `test_shredded_comparison_filter_integration` (Gt/Lt/Gte con 3 nodes)

### Dependencias
- Phase 1 ✅
- COMP-023 ✅
- `query::RelOp` ✅
- Ninguna externa
