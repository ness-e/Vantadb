# OLD-02: GraphRAG Pipeline Formal — seed → expand → retrieve → generate context

## Metadata
- **Plan file:** *(none — backlog task)*
- **Fuente:** `docs/Backlog.md:172` — Phase 9 (Old Docs Rescue)
- **Esfuerzo:** 🟡 1-2 sem
- **Prioridad:** 🗺️ Roadmap
- **Tipo:** Rust core + Python examples
- **Turns estimados:** 30-60
- **Creado:** 2026-07-26
- **Estado:** ✅ COMPLETED (2026-07-26, verificado batch 6: `src/graphrag/` pipeline formal completo — mod, pipeline, retrieve, expand — + `examples/rust/graphrag.rs`)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| **Callers** | `vantadb-mcp` (query routing), `vantadb-python` (Python SDK), `vantadb-server` (HTTP API) |
| **Callees** | `src/search/` (hybrid search, vector search, lexical search), `src/node.rs` (graph nodes), `src/graph/` (BFS, edge traversal), `src/llm/` (remote-inference feature for auto-embedding) |
| **Implicaciones** | Se agrega API pública `graphrag_search()` en SDK. No rompe contratos existentes — es aditiva. `search_memory` y `graph_bfs` existentes no cambian. DRV-123 auto-embedding es recomendado, no bloqueante. |

## Contrato
```bash
cargo nextest run --profile audit -p vantadb --test graphrag --build-jobs 2 && \
cargo check -p vantadb --features "remote-inference" && \
python -m pytest examples/python/test_graphrag_pipeline.py -v --exitfirst
```

## Estado actual
- `examples/rust/graphrag.rs` — solo demo de API de grafo (insertar nodos, edges, BFS). **No es un pipeline GraphRAG.**
- `docs/glosario/graphrag.md` — definición conceptual + fórmula de reducción de tokens + ejemplos idealizados de API
- VantaDB tiene los primitivos: graph BFS, hybrid search, RRF fusion, metadata filters, auto-embedding (remote-inference)
- Falta: pipeline orquestador formal seed→expand→retrieve→generate context

## Steps

### Step 1: Diseñar la API GraphRAG
- **Archivos:** `src/graphrag/mod.rs` (nuevo), `src/graphrag/pipeline.rs` (nuevo)
- **Acción:** Definir `GraphRagPipeline` struct con configuración:
  - `seed_k: usize` (top-K seeds de vector search)
  - `expansion_hops: usize` (cuántos hops BFS desde seeds)
  - `max_expansion_nodes: usize` (límite de nodos expandidos)
  - `retrieval_top_k: usize` (final ranking K)
  - `embedding_fn: Option<Arc<dyn Fn(&str) -> Vec<f32>>>` (opcional, DRV-123 bridge)
  - Modo de expansión: BFS simple, BFS con poda por relevancia, o random walk
- **Método principal:** `search(&self, query: &str, vector: Option<&[f32]>) -> Result<GraphRagResult>`
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 2: Implementar Seed Phase
- **Archivos:** `src/graphrag/seed.rs` (nuevo)
- **Acción:** Desde un query string (con o sin vector explícito):
  - Si no hay vector y `embedding_fn` está configurado → auto-embed via embedding_fn
  - Si no hay vector ni embedding_fn → usar BM25/text search como fallback
  - Ejecutar vector search top-K → obtener seed nodes con sus scores
  - Si hay texto, también ejecutar hybrid search y fusionar con RRF
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 3: Implementar Expand Phase
- **Archivos:** `src/graphrag/expand.rs` (nuevo)
- **Acción:** Desde los seed nodes, expandir por aristas del grafo:
  - BFS configurable (1-3 hops default)
  - `edge_label_filter: Option<Vec<&str>>` — expandir solo por labels específicos
  - `expansion_mode`: Bfs, RelevanceGuided (poda nodos con score < threshold)
  - De-duplicar nodos ya visitados
  - Retornar lista plana de nodos expandidos con sus distancias desde seed
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 4: Implementar Retrieve + Re-rank Phase
- **Archivos:** `src/graphrag/retrieve.rs` (nuevo)
- **Acción:** Para cada nodo en seed ∪ expanded:
  - Fetch content completo + metadata
  - Calcular relevance score combinado:
    - `final_score = α * vector_score + β * (1 - expansion_distance / max_distance) + γ * degree_boost`
  - Top-K por `final_score`
  - Preservar edges entre nodos seleccionados para contexto estructural
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 5: Implementar Generate Context Phase
- **Archivos:** `src/graphrag/context.rs` (nuevo)
- **Acción:** Ensamblar el contexto estructurado para LLM:
  - `format_subgraph()` — serializar nodos + aristas a texto estructurado
  - `format_context(options)` — templates configurables:
    - `ContextFormat::PlainText` — solo contenidos concatenados con scores
    - `ContextFormat::Structured` — JSON/YAML con nodos, relaciones, scores
    - `ContextFormat::PromptReady` — texto formateado para inyectar en prompt LLM
  - Incluir metadatos de procedencia (query original, hops tomados, nodos totales)
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 6: Exponer API pública en SDK
- **Archivos:** `src/sdk/mod.rs`, `src/sdk/search/mod.rs`, `vantadb-python/src/lib.rs`
- **Acción:** Agregar `graphrag_search()` al SDK embebido y bindings Python:
  - `VantaEmbedded::graphrag_search(query, opts)` → devuelve `GraphRagSearchResult`
  - `client.graphrag_search(query)` en Python
  - `GraphRagSearchResult` contiene: `nodes`, `edges`, `context_text`, `pipeline_stats`
- **Verify:** `cargo check -p vantadb -p vantadb-python`
- **Estado:** ⬜ PENDING

### Step 7: Reemplazar ejemplo graphrag.rs con pipeline real
- **Archivos:** `examples/rust/graphrag.rs` (modificar), `examples/python/graphrag_pipeline.py` (nuevo)
- **Acción:** 
  - Actualizar `examples/rust/graphrag.rs` para usar el pipeline `GraphRagPipeline` en vez de la API raw de nodos/edges
  - Crear `examples/python/graphrag_pipeline.py` que demuestre seed→expand→retrieve→generate
  - Demostrar caso con graph_hops=2 y búsqueda multi-hop
- **Verify:** `cargo run --example graphrag && python examples/python/graphrag_pipeline.py`
- **Estado:** ⬜ PENDING

### Step 8: Tests de integración
- **Archivos:** `tests/graphrag/mod.rs` (nuevo)
- **Acción:** 
  - `test_simple_graphrag_search`: insertar 10 nodos con edges, buscar, verificar expansión
  - `test_hybrid_seed_fallback`: sin vector, ver que usa BM25 como seed
  - `test_pipeline_stats`: verificar que pipeline_stats contiene métricas correctas
  - `test_max_expansion_limit`: verificar que no se expande más allá de max_expansion_nodes
- **Verify:** `cargo nextest run --profile audit --test graphrag --build-jobs 2`
- **Estado:** ⬜ PENDING

### Step 9: Documentar API
- **Archivos:** `docs/api/GRAPH_RAG.md` (nuevo), actualizar `docs/glosario/graphrag.md`
- **Acción:** Documentar la API pública con ejemplos Rust y Python. Incluir guía de configuración pipeline.
- **Verify:** `scripts/validate-docs-coverage.ps1`
- **Estado:** ⬜ PENDING

## Dependencias
- DRV-123 (auto-embedding) — recomendado, **no bloqueante**. Sin DRV-123 el pipeline funciona con vectores pre-computados o BM25 fallback.
- COMP-004 (bitset filtering) — no necesario para v1
- `remote-inference` feature — ya existe, usar si está disponible

## Notas
- Microsoft GraphRAG es el reference implementation (arxiv 2404.16130). VantaDB implementa una versión simplificada: seed→expand local, sin community detection ni summarization global.
- SeedER (arxiv 2605.23753) describe seed-and-expand con RL policy — más avanzado que lo necesario aquí.
- El pipeline NO necesita LLM para generación — solo produce contexto estructurado. El LLM consumption queda del lado del usuario.
- Los nombres `graph_hops` y `graphrag_search` ya aparecen en el glosario idealizado — mantener consistencia.
