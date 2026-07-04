---
title: "busqueda-lexica"
type: glossary-entry
status: stable
tags: [glosario, búsqueda, léxica, bm25, text]
last_reviewed: 2026-07-03
aliases: [lexical search, text search, keyword search, BM25]
---

# busqueda-lexica

## Definición

La **busqueda-lexica** es una técnica de recuperación de información basada en la coincidencia exacta de términos (keywords) entre la consulta y los documentos, utilizando modelos estadísticos como [BM25](BM25.md) para calcular la relevancia.

## Diferencias con busqueda-vectorial

| Característica | busqueda-lexica | busqueda-vectorial |
|----------------|-----------------|-------------------|
| **Tipo** | Coincidencia exacta | Similitud semántica |
| **Modelo** | [BM25](BM25.md), TF-IDF | Embeddings neuronales |
| **Fuerza** | Keywords específicos | Significado contextual |
| **Debilidad** | Sinónimos, polisemia | Términos exactos |
| **Velocidad** | Rápida (índice invertido) | Media (ANN) |

## BM25 (Best Matching 25)

### Fórmula

$$\text{score}(D, Q) = \sum_{i=1}^{n} \text{IDF}(q_i) \cdot \frac{f(q_i, D) \cdot (k_1 + 1)}{f(q_i, D) + k_1 \cdot (1 - b + b \cdot \frac{|D|}{\text{avgdl}})}$$

Donde:
- $f(q_i, D)$ = frecuencia del término $q_i$ en documento $D$
- $|D|$ = longitud del documento
- $\text{avgdl}$ = longitud promedio de documentos
- $k_1$ = parámetro de saturación de TF (default: 1.2)
- $b$ = parámetro de normalización de longitud (default: 0.75)
- $\text{IDF}(q_i)$ = inverse document frequency

### IDF (Inverse Document Frequency)

$$\text{IDF}(q_i) = \log \frac{N - n(q_i) + 0.5}{n(q_i) + 0.5}$$

Donde:
- $N$ = número total de documentos
- $n(q_i)$ = número de documentos que contienen $q_i$

## En VantaDB

### Estructura del Índice

```rust
// src/text_index.rs
pub struct TextIndex {
    // Término → Lista de postings
    postings: HashMap<String, Vec<Posting>>,
    // Estadísticas globales
    stats: TextTermStats,
    // Estadísticas por namespace
    namespace_stats: HashMap<String, TextNamespaceStats>,
}

pub struct Posting {
    pub node_id: u64,
    pub tf: u32,              // Frecuencia del término
    pub positions: Vec<u32>,  // Posiciones (para phrase queries)
}
```

### Tokenización

**Tokenizador Básico (default):**
- Lowercase ASCII
- Split en caracteres no alfanuméricos
- Sin stemming ni stopwords

**Tokenizador Avanzado (feature `advanced-tokenizer`):**
- Integración con `tantivy-tokenizer`
- Stemming multilingüe
- Stopwords removal
- Unicode folding

### Configuración

```python
db = VantaEmbedded("./data", config={
    "bm25": {
        "k1": 1.2,    # Saturación de TF
        "b": 0.75     # Normalización de longitud
    }
})
```

### Uso

```python
# busqueda-lexica pura
results = db.search(
    text="base de datos embebida",
    top_k=10,
    mode="text"  # Solo BM25
)

# Phrase query (comillas)
results = db.search(
    text='"base de datos"',
    top_k=10,
    mode="text"
)
```

## Casos de Uso

### Cuándo usar busqueda-lexica

✅ **Ideal para:**
- Nombres propios, IDs, códigos
- Terminología técnica específica
- Búsquedas con keywords exactos
- Queries cortas y precisas

❌ **No ideal para:**
- Búsqueda por significado
- Sinónimos y variaciones
- Queries en lenguaje natural

### Ejemplo: Híbrido vs Solo Léxica

```python
# Query: "¿Cómo funciona la persistencia WAL?"

# Solo léxica (BM25)
results_lexical = db.search(text="persistencia WAL", mode="text")
# Encuentra documentos con esas palabras exactas

# Solo vectorial
results_vector = db.search(vector=embed(query), mode="vector")
# Encuentra documentos semánticamente similares

# Híbrida ([RRF](RRF.md))
results_hybrid = db.search(
    vector=embed(query),
    text="persistencia WAL",
    mode="hybrid"
)
# Combina ambos: mejor recall
```

## Performance

| Métrica | Valor | Dataset |
|---------|-------|---------|
| Indexación | ~1000 docs/sec | 100K documentos |
| Query p50 | 115.3 ms | Python SDK |
| Query p99 | 137.5 ms | Python SDK |

## Véase También

- [BM25](BM25.md) - Algoritmo de scoring
- [busqueda-hibrida](busqueda-hibrida.md) - Combinación con vectorial
- [RRF](RRF.md) - Fusión de rankings
- [HNSW](HNSW.md) - Índice vectorial
