# MMAP-02b: Eliminación de `sqrt()` Redundante en el Hot Path L2

**Objetivo:** Optimizar la ruta crítica de distancia Euclidiana (L2) eliminando la operación `sqrt()` del hot path del traversal HNSW y la conversión final de scores. La comparación de distancias cuadradas preserva el ordenamiento total, haciendo `sqrt()` innecesario durante la búsqueda y obligatorio únicamente en la presentación final al usuario.

---

## Contexto y Análisis del Problema

### Situación Actual

El benchmark `competitive_bench` con SIFT1M mostró una disparidad de rendimiento significativa entre Cosine y L2 (Euclidean). Tras auditar el código, se identificaron los siguientes puntos de ineficiencia:

1. **`search_nearest` → línea 1115**: Al construir los resultados finales, se aplica `-(-score).max(0.0).sqrt()` a **cada** resultado. Esto es correcto pero solo necesario una vez al final.

2. **`vector_memory_search` (sdk.rs:2210)**: Se aplica `sqrt()` en cada candidato del brute-force scan, incluyendo candidatos que serán descartados. Es un desperdicio de CPU.

3. **Falta de fast-path para Euclidean**: Mientras Cosine tiene un fast-path con `inv_cached_norm` pre-computado (ahorrando ~50% de trabajo SIMD), Euclidean no tiene optimización equivalente. El campo `inv_cached_norm` se fija a `0.0` para Euclidean (línea 876), desperdiciando la caché.

### Propiedad Matemática Clave

Para cualesquiera vectores `a, b, c`:
```
||a - b||² < ||a - c||²  ⟺  ||a - b|| < ||a - c||
```

La raíz cuadrada es una **función monótonamente creciente**. Esto significa que comparar distancias cuadradas produce exactamente el mismo ordenamiento que comparar distancias reales. El HNSW solo necesita ordenamientos relativos durante el traversal; el valor absoluto solo importa en el score devuelto al usuario.

### Impacto Esperado

- **Eliminación de `sqrt()` del hot path**: Cada búsqueda HNSW evalúa ~200-1000 nodos. Eliminar `sqrt()` de cada evaluación ahorra ~5-15ns × N nodos por query.
- **Coherencia**: El score devuelto al usuario seguirá siendo la distancia real (negada), no la cuadrática, preservando la semántica de la API.

---

## User Review Required

> [!IMPORTANT]
> **Cambio en la semántica del score devuelto:**
> Actualmente `search_nearest` devuelve `-sqrt(dist²)` para Euclidean. Tras la optimización, seguirá devolviendo exactamente el mismo valor. El cambio es **puramente interno**: la comparación durante el traversal usa distancias cuadradas, pero la conversión final `sqrt()` se aplica solo a los `top_k` resultados que se devuelven al usuario (máximo 10-100 elementos vs. 200-1000 evaluados internamente).

> [!WARNING]
> **Impacto en `select_neighbors` (heurística de poda):**
> La heurística de diversidad compara `sim_cand_sel > sim_q_cand`. Dado que ambas similitudes se calculan con `-dist²` (sin sqrt), la comparación sigue siendo correcta. Sin embargo, debemos verificar que no haya paths donde se mezclen scores con sqrt y sin sqrt en la misma comparación.

---

## Proposed Changes

### Componente: Motor de Búsqueda HNSW (`index.rs`)

#### [MODIFY] [index.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/index.rs)

**M1: Eliminar `sqrt()` del score final en `search_nearest` — solo aplicar a resultados devueltos**

La línea 1115 actualmente aplica `sqrt()` a todos los resultados del heap. Esto ya está en la sección post-búsqueda y solo procesa los `top_k` elementos, así que el overhead es mínimo aquí. **Este punto se mantiene correcto tal como está.**

**M2: Confirmar que `calculate_similarity`, `f32_slice_similarity`, `fast_similarity` usan `-dist²` consistentemente (sin sqrt)**

Verificación: todas estas funciones ya retornan `-euclidean_distance_squared_f32(a, b)` para Euclidean. No hay `sqrt()` en el hot path del traversal. **CONFIRMADO: No hay `sqrt()` redundante en el traversal HNSW.**

**Conclusión del análisis del hot path de index.rs:** El motor HNSW **ya opera correctamente** sin `sqrt()` durante el traversal. El único `sqrt()` está en la conversión final (línea 1115), que solo procesa los resultados devueltos.

---

### Componente: SDK Brute-Force Search (`sdk.rs`)

#### [MODIFY] [sdk.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/sdk.rs)

**M3: Eliminar `sqrt()` del brute-force en `vector_memory_search`**

La línea 2210 aplica `.sqrt()` a **cada** candidato evaluado durante el brute-force scan:
```rust
DistanceMetric::Euclidean => {
    -crate::index::euclidean_distance_squared_f32(query_vector, vector).sqrt()
}
```

Este `sqrt()` se evalúa para **todos** los registros del namespace, pero solo los `top_k` son devueltos. Optimización: usar `-dist²` para comparación/ranking, y solo aplicar `sqrt()` a los `top_k` finales.

**Cambio propuesto:**
```rust
DistanceMetric::Euclidean => {
    -crate::index::euclidean_distance_squared_f32(query_vector, vector)
}
```

Y tras `sort + truncate`, aplicar `sqrt()` solo a los hits supervivientes:
```rust
if distance_metric == DistanceMetric::Euclidean {
    for hit in hits.iter_mut() {
        hit.score = -(-hit.score).max(0.0).sqrt();
    }
}
```

---

### Componente: Función `cosine_sim_with_query_norm` — Reducción de `sqrt` para Cosine

#### [MODIFY] [index.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/index.rs)

**M4: Optimización de la ruta MMap de Cosine en `search_layer`**

En el zero-copy path de `search_layer` (líneas 566-568, 668-674), el match genérico `DistanceMetric::Cosine | DistanceMetric::Euclidean` no aprovecha el `query_inv_norm` pre-computado ni el `inv_cached_norm` del nodo para Cosine mmap. El path MMap siempre recalcula la norma del candidato.

**Cambio propuesto:** Para vecinos en mmap con métrica Cosine, usar el `fast_similarity` path cuando `query_inv_norm` está disponible, o al menos usar `cosine_sim_with_query_norm` con la norma del query pre-computada.

Para Euclidean mmap, ya está óptimo (`-euclidean_distance_squared_f32` sin sqrt).

---

## Resumen de Cambios Concretos

| Archivo | Línea | Cambio | Impacto |
|---------|-------|--------|---------|
| `sdk.rs` | 2210 | Eliminar `.sqrt()` del loop y aplicarlo post-truncate | Ahorra `sqrt()` × (N - top_k) candidatos |
| `index.rs` | 566-568, 668-674 | Split del match `Cosine|Euclidean` en MMap path para usar fast_similarity | Elimina ~50% SIMD en Cosine MMap |

> [!NOTE]
> **Hallazgo importante:** El hot path HNSW del traversal (`search_layer`) **ya está optimizado** para Euclidean — usa `-dist²` sin sqrt. El único `sqrt()` está en la conversión final de `search_nearest` (solo top_k resultados). La optimización real pendiente está en:
> 1. El brute-force scan de `sdk.rs` (eliminar sqrt del loop)
> 2. El fast-path de Cosine en MMap (aprovechar normas pre-computadas)

---

## Verification Plan

### Automated Tests

1. **Test de regresión de scores:**
   ```powershell
   cargo test --workspace --release
   ```
   Todos los tests existentes deben pasar sin cambios.

2. **Benchmark competitivo:**
   ```powershell
   cargo test --test competitive_bench --release -- --nocapture
   ```
   - Recall@10 para L2 debe **mantener o mejorar** los valores previos.
   - QPS para L2 debe **mejorar** respecto al baseline previo.
   - QPS para Cosine MMap debe mejorar si se implementa M4.

### Manual Verification

1. Comparar la tabla de resultados del benchmark con los valores del baseline anterior.
2. Verificar que `cargo clippy --all-targets --all-features -- -D warnings` pase limpio.
