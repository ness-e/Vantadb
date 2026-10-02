---
title: Payload Indexes
kind: glossary
status: stable
description: Los payload indexes son índices derivados sobre los campos de metadata de un
tags: [vantadb, glosario, indices, filtros]
links: "[Glosario](./README.md)"
---

# Payload Indexes

## Definición

Los **payload indexes** son índices *derivados* sobre los campos de metadata de un
memory record. No son un feature configurable: VantaDB los mantiene
automáticamente para **cada** campo de metadata que se escribe, y los usa para
resolver filtros de igualdad (`$eq`) y para localizar los registros de un
namespace por prefijo.

Están separados de los payloads canónicos en el storage: son
`BackendPartition::PayloadIndex` (`src/backend.rs:46`), una partición propia del
KV store que se reconstruye desde los registros canónicos
(`rebuild_derived_indexes`, `src/sdk/serialization/impl_rebuild.rs`).

## Cómo funcionan

Una clave de payload index es un prefijo composed de tres partes más el key del
registro (`payload_index_prefix` / `payload_index_key`,
`src/sdk/serialization/mod.rs:240-258`):

```
<namespace>\0<field>\0<valor codificado>\0<record key>
```

Ejemplo verificado por test (`src/sdk/serialization/mod.rs:1057`):

```rust
// Metadata: {"department": "engineering", "level": "senior"}
// Índice:   "ns\0color\0s:red\0" + record_key
let prefix = payload_index_prefix("ns", "color", &Value::String("red".into()))?;
assert_eq!(prefix, b"ns\0color\0s:red\0");
```

Como el prefijo es ordenado por bytes, un filtro de igualdad se resuelve con un
**prefix scan** (`scan_partition_prefix_iter`) en lugar de un escaneo completo:

```
Sin payload index:   Top 1000 candidatos → filtrar → Top 10
Con payload index:   prefijo → conjunto de candidatos → Top 10
```

## Índices derivados que existen

El conjunto real de índices derivados es **otro** — no hay un índice por tipo de
dato (`KeywordIndex`, `IntegerIndex`, `FloatIndex`, `BooleanIndex` no existen en
el código):

| Índice | Partición | Qué indexa | Notas |
|--------|-----------|------------|-------|
| **Namespace** | `NamespaceIndex` | `namespace\0` → node ids | Prefix scan para `list()` y export |
| **Payload (metadata)** | `PayloadIndex` | `ns\0field\0value\0key` | Un solo tipo de índice, agnóstico al tipo del valor |
| **Text / BM25** | `TextIndex` | Términos → postings con posiciones y TF | Auditable: `audit_text_index()` / `repair_text_index()` |
| **Sparse** | `SparseIndex` | `ns\0dim\0key` → posting `(node_id, weight)` | Búsqueda sparse por dimensión; scoring = dot product crudo |
| **Vector (HNSW/IVF/SCANN/Flat)** | — | embeddings | Índice de dossier, no prefijo |

Los valores de payload se **flattenan** antes de indexarse: un `ListInt([1,2,3])`
produce tres entradas `Int(1)`, `Int(2)`, `Int(3)` vía
`Value::to_index_values()` (`src/sdk/types.rs:133`). Por eso un prefix scan sobre
una lista completa no es posible — solo sobre valores escalares individuales.

## Operadores de filtro

El enum `FilterOp` (`src/sdk/types/record.rs:27`) define **seis** operadores:

| Operador | Significado |
|----------|-------------|
| `Eq` | `==` |
| `Neq` | `!=` |
| `Gt` | `>` |
| `Gte` | `>=` |
| `Lt` | `<` |
| `Lte` | `<=` |

No existe `$in`. Los operadores de rango (`Gt`/`Gte`/`Lt`/`Lte`) requieren
valores ordenables del mismo *variant* que el campo: las comparaciones
cross-variant caen al orden de declaración del enum
(`String < Int < Float < Bool < DateTime < … < Null`), así que un filtro
`$gte` sobre un campo string contra un valor int no significa lo que parece.

## Configuración

No hay configuración de payload indexes — no existen `PayloadIndexConfig` ni
`PayloadIndexType`. Se configuran los **filtros** en la query:

```python
import vantadb

db = vantadb.Client("./data")

# Igualdad
results = db.search("ns", query_vector=query_vector,
                    filters={"department": "engineering"})

# Rango
results = db.search("ns", query_vector=query_vector,
                    filters={"year": {"$gte": 2020, "$lte": 2024}})

# Combinación (AND implícito)
results = db.search("ns", query_vector=query_vector,
                    filters={"department": "engineering", "year": {"$gte": 2023}})
```

## Mantenimiento

Los payload indexes se actualizan en el camino de escritura
(`replace_derived_indexes`, `src/sdk/serialization/impl_index.rs:154`), que
borra las entradas del registro anterior y escribe las nuevas. El rebuild desde
almacenamiento canónico se dispara cuando el estado derivado guardado no cuadra
(`ensure_derived_indexes_current_with`, mismo archivo):

```python
# Reconstruye índices ANN, derivados y de texto desde el storage canónico
db.rebuild_index()
```

No existen `rebuild_payload_indexes()` ni `rebuild_payload_index("field")`.

## Métricas

| Métrica | Dónde |
|---------|-------|
| Entradas de índice de payload | `operational_metrics()["derived_prefix_scans"]` cuenta los prefix scans |
| Bytes / conteos por índice | `operational_metrics()["text_postings_written"]`, `derived_prefix_scans` |
| Consistencia del índice de texto | `audit_text_index()` / `repair_text_index()` |

## Véase También

- [HNSW](./hnsw.md) — Índice vectorial complementario
- [BM25](./bm25.md) — Índice léxico
- [RRF](./rrf.md) — Fusión de resultados filtrados
- [serialization](./serialization.md) — Formato de claves derivadas

---

*Payload indexes permiten filtrado por igualdad sin escanear el namespace completo.*
