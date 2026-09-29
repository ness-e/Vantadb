---
title: similar_to_key
kind: glossary
status: stable
description: Método de búsqueda por similitud que extrae el vector de un registro existente (clave) y ejecuta búsqueda vectorial contra él
aliases: [similar_to_key, search-by-key, buscar-por-clave]
tags: [glosario, api, busqueda, similaridad]
type: glossary-entry
last_reviewed: "2026-09-15"
---


# similar_to_key

## Definición

**`similar_to_key`** es un método de la API de VantaDB que permite buscar registros similares a uno existente, identificado por su clave (`namespace` + `key`). Internamente obtiene el vector del registro origen y ejecuta una búsqueda **[vector-search](./vector-search.md)** con ese vector como query.

## Firma

```python
db.similar_to_key(
    namespace: str,
    key: str,
    top_k: int = 10,
) -> List[dict]
```

## Flujo de Ejecución

```
1. Validate(namespace, key)
2. get(namespace, key) → extrae el vector almacenado
3. search(vector=record.vector, top_k=top_k) → HNSW traversal
4. Return hits enriquecidos con metadata del registro
```

## Casos de Uso

- **Sistemas RAG**: "encuentra documentos similares a este"
- **Recomendación**: "más como este producto"
- **Agentes contextuales**: recuperar memorias relacionadas a una interacción previa

## Véase También

- [vector-search](./vector-search.md) — Búsqueda por similitud vectorial
- [hnsw](./hnsw.md) — Índice subyacente
- [put_batch](./put_batch.md) — Inserción por lote
- [python-sdk](./python-sdk.md) — SDK de Python
- [Python SDK Reference](../../api/PYTHON_SDK.md)
