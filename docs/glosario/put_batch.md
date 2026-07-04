---
title: "put_batch"
type: glossary-entry
status: stable
tags: [glosario, api, escritura, batch, rendimiento]
last_reviewed: 2026-07-03
aliases: [put_batch, batch-insert, batch-write, insercion-por-lote]
description: "Inserción o actualización masiva de registros en paralelo, hasta 5x más rápida que inserciones individuales"
---

# put_batch

## Definición

**`put_batch`** es un método de la API de VantaDB que inserta o actualiza múltiples registros de memoria persistentes en una sola operación. Utiliza paralelismo vía **Rayon** para procesar los registros concurrentemente.

## Firma

```python
db.put_batch(
    entries: List[Tuple[str, str, str, Optional[dict], Optional[List[float]], Optional[int]]]
) -> List[dict]
```

Cada entrada es: `(namespace, key, payload, metadata, vector, ttl_ms)`.

## Implementación

```rust
// src/sdk/api.rs
pub fn put_batch(&self, inputs: Vec<VantaMemoryInput>) -> Result<Vec<VantaMemoryRecord>> {
    #[cfg(feature = "rayon")]
    use rayon::prelude::*;

    inputs.into_par_iter().map(|input| {
        validate_namespace(&input.namespace)?;
        validate_key(&input.key)?;
        validate_metadata(&input.metadata)?;
        let node_id = memory_node_id(&input.namespace, &input.key);
        // ... insert or update logic
    }).collect()
}
```

## Performance

| Estrategia | 10 registros | 100 registros | 1000 registros |
|-----------|-------------|---------------|----------------|
| `put()` individual | ~5 ms | ~50 ms | ~500 ms |
| `put_batch()` | ~2 ms | ~12 ms | ~100 ms |
| **Speedup** | **2.5x** | **4x** | **5x** |

## Validaciones

Todas las validaciones individuales se aplican a cada entrada del lote:
- Validación de `namespace`, `key` y `metadata`
- Colisión de `node_id` detectada y reportada por registro
- Fallo rápido: si un registro es inválido, el batch completo se rechaza

## Véase También

- [[similar_to_key]] — Búsqueda por clave existente
- [[../api/PYTHON_SDK.md|Python SDK Reference]]
- [[../api/EMBEDDED_SDK.md|Embedded SDK Reference]]
