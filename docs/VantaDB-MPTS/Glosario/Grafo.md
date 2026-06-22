---
type: glosario-entry
status: stable
tags: [concepto, graph, knowledge-graph, relaciones]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
aliases: [Graph, Knowledge Graph, Grafo de Conocimiento, Property Graph]
description: "Estructura de datos compuesta por nodos (entidades) y aristas (relaciones), donde ambos pueden tener propiedades asociadas, modelando conectividad y relaciones explícitas"
---

# Grafo

## Definición

Un **grafo** (o **grafo de propiedades**) es una estructura de datos compuesta por **nodos** (entidades) y **aristas** (relaciones), donde ambos pueden tener propiedades asociadas. Los grafos modelan **conectividad y relaciones explícitas** entre entidades, permitiendo consultas de traversal y razonamiento multi-hop.

## Estructura Matemática

$$
G = (V, E)
$$

Donde:
- $V$ = conjunto de vértices (nodos)
- $E \subseteq V \times V$ = conjunto de aristas (edges)

### Ejemplo: Knowledge Graph

```
[Alice] ──trabaja_en──▶ [Acme Corp]
   │                        │
   │                        ├──ubicada_en──▶ [Madrid]
   │
   └──amigo_de──▶ [Bob] ──usa──▶ [VantaDB]
```

## Grafo de Propiedades (Property Graph Model)

En un **property graph**, tanto nodos como aristas tienen propiedades:

```
Nodo: Alice
├── label: "Person"
├── properties:
│   ├── name: "Alice"
│   ├── age: 30
│   └── email: "alice@example.com"
└── outgoing_edges: [trabaja_en, amigo_de]

Arista: trabaja_en
├── source: Alice
├── target: Acme Corp
├── label: "WORKS_AT"
└── properties:
    ├── since: "2020-03-15"
    └── role: "Engineer"
```

## Por Qué Importa en VantaDB

VantaDB implementa un **modelo multimodelo** que incluye grafos nativamente:

### UnifiedNode = Documento + Vector + Grafo

```rust
struct UnifiedNode {
    key: String,                    // Identificador único
    vector: Vec<f32>,               // Embedding semántico
    text: String,                   // Contenido textual
    metadata: BTreeMap<String, Value>, // Propiedades
    edges: Vec<Edge>,               // Relaciones de grafo
}
```

### Caso de Uso: GraphRAG

El **GraphRAG** combina búsqueda vectorial con traversal de grafo:

```
Query: "¿Quién trabaja en Acme Corp y usa VantaDB?"

Paso 1: Búsqueda vectorial → Nodos semilla relevantes
Paso 2: Traversal de grafo → Expandir relaciones
Paso 3: Filtrar por propiedades → Respuesta precisa
```

**Resultado:** Contexto más rico y preciso que solo búsqueda vectorial.

## Tipos de Consultas de Grafo

### 1. Traversal (Recorrido)

```cypher
-- Cypher (Neo4j)
MATCH (p:Person)-[:WORKS_AT]->(c:Company)
WHERE c.name = "Acme Corp"
RETURN p.name
```

### 2. Path Finding (Encontrar Caminos)

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

## Implementación en VantaDB

### Estructura de Datos

```rust
struct Edge {
    target_key: String,      // Nodo destino
    edge_type: String,       // Tipo de relación
    properties: Metadata,    // Propiedades de la arista
    weight: f32,             // Peso opcional
}

// En UnifiedNode:
edges: Vec<Edge>
```

### Almacenamiento

- **Nodos:** Almacenados como `UnifiedNode` en el backend ([Fjall](Fjall.md)/[RocksDB](RocksDB.md))
- **Aristas:** Embebidas en el nodo fuente (edge list)
- **Índice de adyacencia:** Para lookup rápido de relaciones

### Traversal en VantaDB

```python
# Obtener nodo
alice = db.get("alice")

# Traversal 1-hop: amigos de Alice
friends = [
    db.get(edge.target_key)
    for edge in alice.edges
    if edge.edge_type == "amigo_de"
]

# Traversal 2-hop: amigos de amigos
friends_of_friends = []
for friend in friends:
    for edge in friend.edges:
        if edge.edge_type == "amigo_de":
            friends_of_friends.append(db.get(edge.target_key))
```

## Ventajas del Grafo en VantaDB

### 1. Contexto Enriquecido para LLMs

En lugar de recuperar solo documentos aislados, recuperas **subgrafos contextuales**:

```
Sin Grafo: "Alice es ingeniera"
Con Grafo: "Alice es ingeniera en Acme Corp (Madrid), 
            amiga de Bob (quien usa VantaDB), 
            reporta a Carlos (CTO)"
```

### 2. Reducción de Tokens en Prompts

GraphRAG reduce tokens del prompt en **40-60%** vs RAG tradicional:

| Enfoque | Tokens en Prompt | Precisión |
|---------|-----------------|-----------|
| Naïve RAG | 100% (baseline) | Media |
| Hybrid RAG | 92% | Alta |
| **GraphRAG (2 hops)** | **42%** | **Muy alta** |
| GraphRAG (TSV format) | 26% | Muy alta |

### 3. Razonamiento Multi-Hop

Preguntas que requieren conectar información dispersa:

```
Pregunta: "¿Qué tecnologías usan las personas que trabajan en startups de Madrid?"

Requiere:
1. Encontrar personas en Madrid
2. Filtrar por "trabaja_en" → startups
3. Para cada persona, encontrar "usa" → tecnologías
4. Agregar resultados
```

Esto es **natural en grafos**, pero **muy costoso en búsqueda vectorial pura**.

## Comparación con Bases de Datos de Grafo Dedicadas

| Sistema | Modelo | Integración con Vectores | Caso de Uso |
|---------|--------|-------------------------|-------------|
| **Neo4j** | Property Graph puro | Via plugin (vector search) | Knowledge graphs enterprise |
| **ArangoDB** | Multi-model (Doc+Graph+KV) | Nativo | Apps multi-modelo |
| **VantaDB** | Multi-model (Doc+Vector+Graph) | **Nativo y unificado** | Agentes de IA, GraphRAG |
| **TigerGraph** | Property Graph | No nativo | Analytics de grafos masivos |

### Ventaja de VantaDB

En VantaDB, **grafo y vectores coexisten en la misma transacción**:

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

En sistemas separados (Neo4j + Pinecone), necesitarías:
1. Actualizar en Neo4j
2. Actualizar en Pinecone
3. Manejar inconsistencia si uno falla

## Trade-offs del Grafo Embebido

| Ventaja | Costo |
|---------|-------|
| Traversal sin red | Sin queries declarativas (Cypher/Gremlin) |
| Transaccional con vectores | Escalabilidad limitada vs grafos distribuidos |
| Zero-config | Sin optimizaciones avanzadas de grafos (partitioning) |

## Véase También

- [Vectores](Vectores.md) — Modelo complementario (semántica vs relaciones)
- [RAG](RAG.md) — GraphRAG mejora RAG tradicional
- [Transaccional](Transaccional.md) — Atomicidad entre grafo y vectores
- [RRF](RRF.md) — Fusión de búsqueda vectorial + traversal de grafo

---

*Los grafos modelan lo que los vectores no pueden: relaciones explícitas y razonamiento estructural.*

