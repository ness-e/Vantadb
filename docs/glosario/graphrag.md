---
title: "GraphRAG"
type: glossary-entry
status: implemented
tags: [vantadb, glosario, rag, grafo, ia]
last_refined: 2026-06
links: "[Glosario](./README.md)"
---

# GraphRAG

## Definición

**GraphRAG** (Graph-based Retrieval-Augmented Generation) es una técnica avanzada de recuperación de información que combina [RAG](RAG.md) tradicional con traversal de [Grafo](graph.md) de conocimiento para proporcionar contexto enriquecido y estructuralmente conectado a modelos de lenguaje.

## Cómo Funciona

A diferencia del RAG tradicional que recupera fragmentos de texto aislados basándose únicamente en similitud semántica, GraphRAG:

1. **Identifica nodos semilla** relevantes mediante busqueda-vectorial ([HNSW](HNSW.md))
2. **Expande el contexto** recorriendo aristas del grafo (1-3 hops)
3. **Recupera subgrafos** completos con relaciones explícitas
4. **Formatea el contexto** preservando la estructura relacional

## Fórmula de Reducción de Tokens

$$
\text{Token Reduction} = 1 - \frac{\text{Tokens}_{\text{GraphRAG}}}{\text{Tokens}_{\text{RAG}}}
$$

> ⚠️ **Estado de la métrica (2026-08-05, MKT-16):** el valor "40-60%" era un **claim sin run**. Ver [GraphRAG Benchmark Methodology](../blog/graphrag-benchmark.md) — el script reproducible `benchmarks/graphrag_bench.rs` mide esta métrica, pero a escala productiva (3000 nodos) la fase de query **sigue sin poder ejecutarse** (`stack overflow reproducible` del engine en Windows release, clase AUDIT-04). **No usar 40-60% como dato verificado** hasta que un run real a escala lo confirme.

## Implementación en VantaDB

```python
import vantadb_py as vantadb

db = vantadb.VantaDB("./data")

# Búsqueda vectorial: recupera nodos relevantes
results = db.search_memory(
    namespace="default",
    query_vector=embed("¿Quién trabaja en Acme?"),
    top_k=10,
)

# Traversal de grafo: expande vecinos de los nodos hit (via graph_bfs)
roots = [hit.node_id for hit in results]
neighbors = db.graph_bfs(roots, max_depth=2)  # Expandir 2 niveles de relaciones

# Resultado incluye:
# - alice (directamente relevante: "Alice trabaja en Acme")
# - bob (conectado: "Bob es amigo de Alice")
# - acme (conectado: nodo empresa)
```

## Ventajas sobre RAG Tradicional

| Aspecto | RAG Tradicional | GraphRAG |
|---------|-----------------|----------|
| **Contexto** | Fragmentos aislados | Subgrafos conectados |
| **Relaciones** | Implícitas (texto) | Explícitas (aristas) |
| **Tokens** | Alto (redundancia) | Bajo (pendiente de verificación — ver benchmark) |
| **Razonamiento** | Single-hop | Multi-hop |
| **Alucinaciones** | Mayor riesgo | Menor riesgo |

## Casos de Uso

### 1. Memoria de Agentes de IA

```python
import vantadb_py as vantadb

db = vantadb.VantaDB("./data")

# Agente recuerda conversaciones con contexto relacional
db.put("default", "user_pref_1",
       payload="Usuario prefiere respuestas concisas")
db.add_edge(1, 2, "preferencia_de")  # enlaza user_pref_1 con user_123

# Búsqueda recupera preferencia + usuario + conversaciones relacionadas
results = db.search_memory(
    namespace="default",
    query_vector=embed("preferencias usuario"),
    top_k=10,
)
neighbors = db.graph_bfs([hit.node_id for hit in results], max_depth=2)
```

### 2. Knowledge Base Empresarial

```python
import vantadb_py as vantadb

db = vantadb.VantaDB("./data")

# Documentos conectados por relaciones
db.put("default", "policy_security",
       payload="Política de seguridad...")
db.add_edge(1, 2, "aprobado_por")  # enlaza policy_security con dept_legal

# Búsqueda recupera política + departamento + responsables
```

### 3. Codebase Intelligence

```python
import vantadb_py as vantadb

db = vantadb.VantaDB("./data")

# Funciones conectadas por llamadas
db.put("default", "function_auth", payload="def authenticate()...")
db.add_edge(1, 2, "llama_a")  # enlaza function_auth con function_validate

# Búsqueda recupera función + dependencias + tests
```

## Métricas de VantaDB

| Métrica | Valor | Estado |
|---------|-------|--------|
| **Token Reduction** | — | ⏳ PENDIENTE de run — ver [benchmark](../blog/graphrag-benchmark.md) |
| **Latencia adicional por hop** | — | ⏳ PENDIENTE de run |
| **Max hops soportados** | 3 (configurable) | ✅ código: `expansion_hops: 2` default, configurable |
| **Recall improvement** | — | ⏳ Requiere ground truth etiquetado (no existe) — no-goal |

## Véase También

- [RAG](RAG.md) — Retrieval-Augmented Generation tradicional
- [Grafo](graph.md) — Estructura de datos subyacente
- [HNSW](HNSW.md) — busqueda-vectorial para nodos semilla
- [RRF](RRF.md) — Fusión de resultados híbridos

### Documentación de Implementación Relacionada
- [[../api/GRAPH_RAG|GraphRAG API]]

---

*GraphRAG es una capacidad diferenciadora de VantaDB que reduce costos de inferencia LLM y mejora la precisión de respuestas.*

