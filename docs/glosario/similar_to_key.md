---
title: "similar_to_key"
type: glossary-entry
status: stable
tags: [glosario, api, busqueda, similaridad]
last_reviewed: 2026-07-03
aliases: [similar_to_key, search-by-key, buscar-por-clave]
description: "Método de búsqueda por similitud que extrae el vector de un registro existente (clave) y ejecuta búsqueda vectorial contra él"
---

# similar_to_key

## Definición

**`similar_to_key`** es un método de la API de VantaDB que permite buscar registros similares a uno existente, identificado por su clave (`namespace` + `key`). Internamente obtiene el vector del registro origen y ejecuta una búsqueda **[[vector-search]]** con ese vector como query.

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

- [[vector-search]] — Búsqueda por similitud vectorial
- [[hnsw]] — Índice subyacente
- [[put_batch]] — Inserción por lote
- [[python-sdk]] — SDK de Python
- [[../api/PYTHON_SDK.md|Python SDK Reference]]
