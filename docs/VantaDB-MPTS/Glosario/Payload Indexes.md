---
type: glossary-entry
status: stable
tags: [vantadb, glosario, índices, filtros]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
---

# Payload Indexes

## Definición

**Payload Indexes** son índices secundarios sobre campos de metadata que permiten filtrado eficiente durante búsquedas vectoriales, evitando el escaneo completo del dataset.

## Propósito

Sin payload indexes:
```
Búsqueda vectorial → Top 1000 candidatos → Filtrar por metadata → Top 10 resultados
(99% de trabajo desperdiciado)
```

Con payload indexes:
```
Filtro por metadata → Top 100 candidatos → Búsqueda vectorial → Top 10 resultados
(90% de reducción en trabajo)
```

## Tipos de Índices Soportados

### 1. Keyword Index

Para campos string con valores discretos:

```rust
// Metadata: {"department": "engineering", "level": "senior"}
// Índice: department → [doc1, doc5, doc12, ...]

pub struct KeywordIndex {
    postings: HashMap<String, HashSet<u64>>,  // value → doc_ids
}
```

### 2. Integer Index

Para campos numéricos con rangos:

```rust
// Metadata: {"year": 2024, "priority": 3}
// Índice: BTreeMap<i64, HashSet<u64>>

pub struct IntegerIndex {
    tree: BTreeMap<i64, HashSet<u64>>,  // value → doc_ids
}
```

### 3. Float Index

Para campos de punto flotante:

```rust
pub struct FloatIndex {
    tree: BTreeMap<OrderedFloat<f64>, HashSet<u64>>,
}
```

### 4. Boolean Index

Para campos booleanos:

```rust
pub struct BooleanIndex {
    true_docs: HashSet<u64>,
    false_docs: HashSet<u64>,
}
```

## Configuración

### Python

```python
db = VantaEmbedded(
    "./data",
    config={
        "payload_indexes": [
            {"field": "department", "type": "keyword"},
            {"field": "year", "type": "integer"},
            {"field": "score", "type": "float"},
            {"field": "published", "type": "boolean"}
        ]
    }
)
```

### Rust

```rust
let config = Config {
    payload_indexes: vec![
        PayloadIndexConfig {
            field: "department".into(),
            index_type: PayloadIndexType::Keyword,
        },
        PayloadIndexConfig {
            field: "year".into(),
            index_type: PayloadIndexType::Integer,
        },
    ],
    ..Default::default()
};
```

## Operadores de Filtro

### Igualdad

```python
results = db.search(
    vector=query_vector,
    filter={"department": "engineering"}
)
```

### Rango

```python
results = db.search(
    vector=query_vector,
    filter={"year": {"$gte": 2020, "$lte": 2024}}
)
```

### IN

```python
results = db.search(
    vector=query_vector,
    filter={"department": {"$in": ["engineering", "product"]}}
)
```

### Combinación (AND)

```python
results = db.search(
    vector=query_vector,
    filter={
        "department": "engineering",
        "year": {"$gte": 2023},
        "published": True
    }
)
```

## Selectividad y Optimización

### Estimación de Selectividad

```rust
fn estimate_selectivity(filter: &Filter) -> f64 {
    match filter {
        Filter::Eq(field, value) => {
            let index = payload_indexes.get(field);
            let matching = index.count(value);
            matching as f64 / total_docs as f64
        }
        Filter::Range(field, range) => {
            let index = payload_indexes.get(field);
            let matching = index.count_range(range);
            matching as f64 / total_docs as f64
        }
    }
}
```

### Estrategia de Ejecución

| Selectividad | Estrategia |
|--------------|------------|
| <10% | Pre-filter: filtrar antes de búsqueda vectorial |
| 10-50% | Post-filter: buscar y filtrar resultados |
| >50% | Full scan: búsqueda vectorial sin filtro |

## Mantenimiento

### Actualización Automática

```rust
fn put(&self, node: UnifiedNode) -> Result<()> {
    // 1. Insertar en storage
    self.storage.put(&node)?;
    
    // 2. Actualizar índices payload
    for (field, value) in &node.metadata {
        if let Some(index) = self.payload_indexes.get(field) {
            index.add(value, node.id)?;
        }
    }
    
    // 3. Actualizar índices vectoriales
    self.hnsw.add(node.id, &node.vector)?;
    
    Ok(())
}
```

### Rebuild

```python
# Reconstruir todos los índices payload
db.rebuild_payload_indexes()

# Reconstruir índice específico
db.rebuild_payload_index("department")
```

## Métricas

| Métrica | Descripción |
|---------|-------------|
| **Index Size** | Bytes en disco por índice |
| **Lookup Time** | Latencia de consulta al índice |
| **Selectivity** | % de documentos que matchean |
| **Build Time** | Tiempo de construcción |

## Véase También

- [HNSW](HNSW.md) — Índice vectorial complementario
- [BM25](BM25.md) — Índice léxico
- [RRF](RRF.md) — Fusión de resultados filtrados

---

*Payload indexes permiten filtrado eficiente sin sacrificar performance de búsqueda vectorial.*

