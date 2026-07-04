---
title: "Grafo"
type: glossary-entry
status: stable
tags: [concept, graph, knowledge-graph, relaciones]
last_refined: 2026-06
links: "[[README.md]]"
aliases: [Graph, Knowledge Graph, Knowledge Graph, Property Graph]
description: "Data structure composed of nodes (entities) and edges (relationships), where both can have associated properties, modeling connectivity and explicit relationships"
---
# Grafo

##Definition

A **graph** (or **property graph**) is a data structure composed of **nodes** (entities) and **edges** (relationships), where both can have associated properties. Graphs model **connectivity and explicit relationships** between entities, allowing traversal queries and multi-hop reasoning.

## Mathematical Structure

$$
G = (V, E)
$$

Where:
- $V$ = set of vertices (nodes)
- $E \subseteq V \times V$ = set of edges

### Example: Knowledge Graph

```
[Alice] ──trabaja_en──▶ [Acme Corp]
   │                        │
   │                        ├──ubicada_en──▶ [Madrid]
   │
   └──amigo_de──▶ [Bob] ──usa──▶ [VantaDB]
```

## Property Graph Model

In a **property graph**, both nodes and edges have properties:

```
Nodo: Alice
├── label: "Person"
├── properties:
│   ├── name: "Alice"
│   ├── age: 30
│   └── email: "alice@example.com"
└── outgoing_edges: [trabaja_en, amigo_de]

Arista: works_at
├── source: Alice
├── target: Acme Corp
├── label: "WORKS_AT"
└── properties:
    ├── since: "2020-03-15"
    └── role: "Engineer"
```

##Why it Matters in VantaDB

VantaDB implements a **multi-model** that includes graphs natively:

### UnifiedNode = Document + Vector + Graph

```rust
struct UnifiedNode {
    key: String,                    // Identificador único
    vector: Vec<f32>,               // Embedding semántico
    text: String,                   // Contenido textual
    metadata: BTreeMap<String, Value>, // Propiedades
    edges: Vec<Edge>,               // Relaciones de grafo
}
```

### Use Case: GraphRAG

**GraphRAG** ​​combines vector-search with graph traversal:

```
Query: "¿Quién trabaja en Acme Corp y usa VantaDB?"

Step 1: vector-search → Relevant seed nodes
Step 2: Graph Traversal → Expand Relationships
Step 3: Filter by properties → Precise response
```

**Result:** Richer and more precise context than just vector-search.

## Types of Graph Queries

### 1. Traversal (Rourse)

```cypher
-- Cypher (Neo4j)
MATCH (p:Person)-[:WORKS_AT]->(c:Company)
WHERE c.name = "Acme Corp"
RETURN p.name
```

### 2. Path Finding

```cypher
-- Camino más corto entre Alice y Bob
MATCH path = shortestPath(
  (alice:Person {name: "Alice"})-[*]-(bob:Person {name: "Bob"})
)
RETURN path
```

### 3. Pattern Matching

```cypher
-- Encontrar triángulos (A→B→C→A)
MATCH (a)-[r1]->(b)-[r2]->(c)-[r3]->(a)
RETURN a, b, c
```

## Implementation in VantaDB

### Data Structure

```rust
struct Edge {
    target_key: String,      // Nodo destino
    edge_type: String,       // Tipo de relación
    properties: Metadata,    // Propiedades de la arista
    weight: f32,             // Peso opcional
}

// On UnifiedNode:
edges: Vec<Edge>
```

### Storage

- **Nodos:** Almacenados como `UnifiedNode` en el backend ([[fjall]]/[[rocksdb]])
- **Aristas:** Embebidas en el nodo fuente (edge list)
- **Índice de adyacencia:** Para lookup rápido de relaciones

### Traversal in VantaDB

```python
# Obtener nodo
alice = db.get("alice")

# Traversal 1-hop: Alice's friends
friends = [
    db.get(edge.target_key)
    for edge in alice.edges
    if edge.edge_type == "friend_of"
]

# Traversal 2-hop: friends of friends
friends_of_friends = []
for friend in friends:
    for edge in friend.edges:
        if edge.edge_type == "friend_of":
            friends_of_friends.append(db.get(edge.target_key))
```

## Advantages of the Graph in VantaDB

### 1. Rich Context for LLMs

Instead of retrieving only isolated documents, you retrieve **contextual subgraphs**:

```
Sin Grafo: "Alice es ingeniera"
Con Grafo: "Alice es ingeniera en Acme Corp (Madrid), 
            amiga de Bob (quien usa VantaDB), 
            reporta a Carlos (CTO)"
```

### 2. Reduction of Tokens in Prompts

GraphRAG reduces prompt tokens by **40-60%** vs traditional RAG:

| Enfoque | Tokens en Prompt | Precisión |
|---------|-----------------|-----------|
| Naïve RAG | 100% (baseline) | Media |
| Hybrid RAG | 92% | Alta |
| **GraphRAG (2 hops)** | **42%** | **Muy alta** |
| GraphRAG (TSV format) | 26% | Muy alta |

### 3. Multi-Hop Reasoning

Questions that require connecting scattered information:

```
Pregunta: "¿Qué tecnologías usan las personas que trabajan en startups de Madrid?"

Requiere:
1. Encontrar personas en Madrid
2. Filtrar por "trabaja_en" → startups
3. Para cada persona, encontrar "usa" → tecnologías
4. Agregar resultados
```

This is **natural in graphs**, but **very expensive in pure vector-search**.

## Comparison with Dedicated Graph Databases

| Sistema | Modelo | Integración con Vectores | Caso de Uso |
|---------|--------|-------------------------|-------------|
| **Neo4j** | Property Graph puro | Via plugin (vector search) | Knowledge graphs enterprise |
| **ArangoDB** | Multi-model (Doc+Graph+KV) | Nativo | Apps multi-modelo |
| **VantaDB** | Multi-model (Doc+Vector+Graph) | **Nativo y unificado** | Agentes de IA, GraphRAG |
| **TigerGraph** | Property Graph | No nativo | Analytics de grafos masivos |

### VantaDB Advantage

In VantaDB, **graph and vectors coexist in the same transaction**:

```python
# Actualizar documento, vector y grafo atómicamente
db.put(
    key="alice",
    vector=embed("Alice es ingeniera en Acme"),
    text="Alice es ingeniera en Acme",
    edges=[
        Edge(target="acme", type="trabaja_en"),
        Edge(target="bob", type="amigo_de")
    ]
)
```

On separate systems (Neo4j + Pinecone), you would need:
1. Update in Neo4j
2. Update in Pinecone
3. Handle inconsistency if one fails

## Trade-offs of the Embedded Graph

| Ventaja | Costo |
|---------|-------|
| Traversal sin red | Sin queries declarativas (Cypher/Gremlin) |
| Transaccional con vectores | Escalabilidad limitada vs grafos distribuidos |
| Zero-config | Sin optimizaciones avanzadas de grafos (partitioning) |

## See Also

- [[vectors]] — Complementary model (semantics vs relationships)
- [[rag]] — GraphRAG improves traditional RAG
- [[transactional]] — Atomicity between graph and vectors
- [[rrf]] — Vector-search fusion + graph traversal

---

*Los grafos modelan lo que los vectores no pueden: relaciones explícitas y razonamiento estructural.*

