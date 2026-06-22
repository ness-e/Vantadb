---
type: glossary-entry
status: stable
tags: [vantadb, glosario, rag, grafo, ia]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
---

# GraphRAG

## Definición

**GraphRAG** (Graph-based Retrieval-Augmented Generation) es una técnica avanzada de recuperación de información que combina [RAG](RAG.md) tradicional con traversal de [Grafo](Grafo.md) de conocimiento para proporcionar contexto enriquecido y estructuralmente conectado a modelos de lenguaje.

## Cómo Funciona

A diferencia del RAG tradicional que recupera fragmentos de texto aislados basándose únicamente en similitud semántica, GraphRAG:

1. **Identifica nodos semilla** relevantes mediante búsqueda vectorial ([HNSW](HNSW.md))
2. **Expande el contexto** recorriendo aristas del grafo (1-3 hops)
3. **Recupera subgrafos** completos con relaciones explícitas
4. **Formatea el contexto** preservando la estructura relacional

## Fórmula de Reducción de Tokens

$$
\text{Token Reduction} = 1 - \frac{\text{Tokens}_{\text{GraphRAG}}}{\text{Tokens}_{\text{RAG}}}
$$

**Resultado típico:** 40-60% de reducción vs RAG tradicional.

## Implementación en VantaDB

```python
# Búsqueda con traversal de grafo
results = db.search(
    vector=embed("¿Quién trabaja en Acme?"),
    top_k=10,
    graph_hops=2  # Expandir 2 niveles de relaciones
)

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
| **Tokens** | Alto (redundancia) | Bajo (40-60% menos) |
| **Razonamiento** | Single-hop | Multi-hop |
| **Alucinaciones** | Mayor riesgo | Menor riesgo |

## Casos de Uso

### 1. Memoria de Agentes de IA

```python
# Agente recuerda conversaciones con contexto relacional
db.put("user_pref_1", 
       text="Usuario prefiere respuestas concisas",
       edges=[{"target": "user_123", "type": "preferencia_de"}])

# Búsqueda recupera preferencia + usuario + conversaciones relacionadas
context = db.search(vector=embed("preferencias usuario"), graph_hops=2)
```

### 2. Knowledge Base Empresarial

```python
# Documentos conectados por relaciones
db.put("policy_security", text="Política de seguridad...",
       edges=[{"target": "dept_legal", "type": "aprobado_por"}])

# Búsqueda recupera política + departamento + responsables
```

### 3. Codebase Intelligence

```python
# Funciones conectadas por llamadas
db.put("function_auth", text="def authenticate()...",
       edges=[{"target": "function_validate", "type": "llama_a"}])

# Búsqueda recupera función + dependencias + tests
```

## Métricas de VantaDB

| Métrica | Valor |
|---------|-------|
| **Token Reduction** | 40-60% vs RAG tradicional |
| **Latencia adicional** | ~25-50ms por hop |
| **Max hops soportados** | 3 (configurable) |
| **Recall improvement** | +15-20% en queries relacionales |

## Véase También

- [RAG](RAG.md) — Retrieval-Augmented Generation tradicional
- [Grafo](Grafo.md) — Estructura de datos subyacente
- [HNSW](HNSW.md) — Búsqueda vectorial para nodos semilla
- [RRF](RRF.md) — Fusión de resultados híbridos

---

*GraphRAG es una capacidad diferenciadora de VantaDB que reduce costos de inferencia LLM y mejora la precisión de respuestas.*

