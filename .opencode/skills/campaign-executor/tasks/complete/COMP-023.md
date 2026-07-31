# COMP-023: 3 Filtering Strategies (Pre/In/Post) with Selectivity-Based Optimizer

## Metadata
- **ID:** COMP-023
- **Priority:** 🟡 Media
- **Effort:** 1-2 sem (core: ~130 líneas en 2 archivos)
- **Dependencies:** COMP-003 ✅, COMP-012 ✅, COMP-028 (no estrictamente bloqueante — usamos `get_estimated_selectivity()` existente)
- **Tags:** filtering, performance, search, strategy

## Objective
Implementar 3 estrategias de filtrado para vector search controladas por un optimizador de selectividad. Reemplazar el post-filter rígido actual (`ALL_BITSET` + `matches_memory_filters`) con selección dinámica entre pre-filter, in-filter y post-filter según la estimación de joint selectivity.

## Background
Actualmente `vector_memory_search` en `src/sdk/search/mod.rs` hace:
1. `index.search()` con `ALL_BITSET` (no filtra en índex)
2. Itera candidatos, hace `get_many()` → `memory_record_from_node()` → `matches_memory_filters()`

Esto desperdicia trabajo cuando el filtro es muy selectivo (< 1% de datos) o moderadamente selectivo (< 10%). Con `FilterBitset` ahora sobre `croaring::Bitmap` (COMP-012) y el in-filter check en `search_layer` (COMP-003), tenemos la infraestructura para ser inteligentes.

## Archivos involucrados
- `src/sdk/search/mod.rs` — estrategia principal `vector_memory_search()`
- `src/node.rs` — `FilterBitset` (añadir helpers si aplica)
- `src/sdk/serialization/impl_export.rs` — `records_for_namespace()` existente (reutilizar para construir bitset)

## Plan de implementación

### Paso 1: Añadir `FilterStrategy` enum y selector (src/sdk/search/mod.rs)

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum FilterStrategy {
    /// Scan metadata → build bitset → vector search on filtered subset.
    /// Best when selectivity ≪ 1%: expensive bitset build, cheap vector search.
    PreFilter,
    /// Build bitset → pass as query_mask during HNSW walk.
    /// Best when selectivity < 10%: moderate bitset build, no post-filter needed.
    InFilter,
    /// Vector search with ALL_BITSET → filter results by metadata.
    /// Default. Best when selectivity ≥ 10%: no bitset build overhead.
    PostFilter,
}
```

Añadir función selectora:
```rust
fn select_filter_strategy(
    engine: &StorageEngine,
    filters: &VantaMemoryMetadata,
) -> Result<FilterStrategy> {
    if filters.is_empty() {
        return Ok(FilterStrategy::PostFilter);
    }
    let mut joint_selectivity = 1.0f32;
    for (field, value) in filters.iter() {
        // Convert VantaValue → RelOp::Eq + FieldValue
        let rel_op = RelOp::Eq;
        let fv = vanta_value_to_field_value(value);
        let sel = engine.get_estimated_selectivity(field, &rel_op, &fv);
        joint_selectivity *= sel;
    }
    if joint_selectivity < 0.01 {
        // Very selective: pre-filter is worth the full scan
        Ok(FilterStrategy::PreFilter)
    } else if joint_selectivity < HIGH_SELECTIVITY_THRESHOLD {
        // Moderately selective: in-filter during HNSW walk
        Ok(FilterStrategy::InFilter)
    } else {
        // Weak filter: post-filter is cheapest
        Ok(FilterStrategy::PostFilter)
    }
}
```

### Paso 2: Añadir función `bitset_from_filters` (src/sdk/search/mod.rs)

Reutilizar `records_for_namespace()` — itera registros, chequea namespace y filters, devuelve `FilterBitset` con todos los `node_id` que matchean.

```rust
fn bitset_from_filters(
    engine: &VantaEngine,
    namespace: &str,
    filters: &VantaMemoryMetadata,
) -> Result<FilterBitset> {
    // Usar records_for_namespace() que ya tiene index + fallback full scan.
    let records = engine.records_for_namespace(namespace, filters)?;
    let mut bitset = FilterBitset::new();
    for record in &records {
        bitset.set_bit(record.node_id);
    }
    Ok(bitset)
}
```

Consideración: `records_for_namespace()` devuelve `Vec<VantaMemoryRecord>` que ya viene filtrado. Extraer solo `node_id` es suficiente — no necesitamos duplicar la lógica de filtrado de metadata.

### Paso 3: Modificar `vector_memory_search` (src/sdk/search/mod.rs)

```rust
fn vector_memory_search(
    &self, namespace, query_vector, filters, top_k, distance_metric,
) -> Result<Vec<VantaMemorySearchHit>> {
    if query_vector.is_empty() || top_k == 0 { return Ok(Vec::new()); }
    let engine = self.engine_handle()?;

    // ── Select strategy ──
    let strategy = select_filter_strategy(&engine, filters)?;

    // ── Build query_mask según estrategia ──
    let query_mask = match strategy {
        FilterStrategy::PreFilter | FilterStrategy::InFilter => {
            let mask = bitset_from_filters(&engine, namespace, filters)?;
            mask
        }
        FilterStrategy::PostFilter => ALL_BITSET.clone(),
    };

    // ── Empty bitset → no results ──
    if query_mask.is_empty() {
        return Ok(Vec::new());
    }

    // ── Vector search ──
    let budget = (top_k.saturating_mul(10)).min(500).max(top_k);
    let candidates = {
        let index = engine.vec_index();
        let vs = engine.vector_store.read();
        index.search(query_vector, &query_mask, budget, Some(&*vs), distance_metric)
    };

    // ── Materialize hits ──
    let mut hits = Vec::with_capacity(top_k);
    // ... (código existente de materialize, adaptado: 
    //      para PreFilter/InFilter no se necesita re-chequear filters,
    //      solo namespace check)
    // ... (brute-force fallback igual que antes)
    Ok(hits)
}
```

Los detalles exactos de materialización dependen de la estrategia:
- **PreFilter/InFilter:** el `query_mask` ya filtró, solo chequear namespace y score (no re-aplicar `matches_memory_filters`)
- **PostFilter:** comportamiento actual (chequear namespace + metadata)

### Paso 4: Tests

Añadir tests en `src/sdk/search/mod.rs`:

```rust
#[test]
fn test_select_filter_strategy_empty_filters() {
    assert_eq!(select_filter_strategy(&engine, &VantaMemoryMetadata::new())?, FilterStrategy::PostFilter);
}

#[test]
fn test_pre_filter_highly_selective() {
    // Insert red/blue data, filter for a rare color
    let hits = engine.vector_memory_search("ns", &vec, &filters_red, 10, Cosine)?;
    // Verify only red items returned
    assert!(hits.iter().all(|h| h.record.metadata.get("color") == Some(&VantaValue::String("red".into()))));
}

#[test]
fn test_in_filter_works() {
    // Similar but for moderate selectivity
}

#[test]
fn test_post_filter_for_weak_selectors() {
    // When filter matches almost everything, post-filter should be used
}
```

## Criterios de éxito
1. `cargo check -p vantadb` pasa
2. Tests existentes pasan (especialmente tests de vector_memory_search y búsqueda con metadata)
3. Joint selectivity < 0.01 activa pre-filter
4. 0.01 ≤ selectivity < 0.1 activa in-filter
5. Selectivity ≥ 0.1 activa post-filter (comportamiento actual)
6. `FilterBitset` vacío cortocircuita a resultados vacíos (no hace búsqueda innecesaria)

## Notas
- `get_estimated_selectivity()` requiere `String` + `RelOp` + `FieldValue`. El SDK usa `VantaMemoryMetadata` (HashMap<String, VantaValue>). Puede ser necesaria una función de conversión `vanta_value_to_field_value()`.
- `records_for_namespace()` ya soporta filtros y fallback a full scan. No reinventar.
- Para pre-filter: si `query_mask` tiene < budget elementos, el vector search será naturalmente rápido.
- Para in-filter: el `search_layer` en HNSW ya checkea `matches_mask()` por nodo — usar `query_mask` directamente.
- `FilterBitset::is_empty()` debe existir (post-COMP-012). Verificar que funciona con `croaring::Bitmap`.

## Roadmap Dependencies
- COMP-003 ✅ (in-filter mechanism)
- COMP-012 ✅ (FilterBitset on croaring)
- COMP-028 ❌ (Semantic Cost Estimator — para selector más preciso en el futuro)
