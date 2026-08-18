# Research — Human-Facing DB UI (Vanta Studio)

Investigación sobre cómo representar VantaDB a humanos: cómo hacer **visible, comprensible,
administrable y editable** toda la información de la base (registros, metadatos, vectores,
grafos, TTL, audit) para que un usuario la gestione sin escribir código.

Fecha: 2026-08-18 · Estado: completo · Método: 5 sub-agentes de investigación en paralelo
(websearch + webfetch + fuentes oficiales 2024–2026) + síntesis integradora.

## Índice de documentos

| Carpeta | Documento | Tema |
|---------|-----------|------|
| `01-vector-db-consoles/` | `RESEARCH.md` | Consolas/paneles de 12 vector DBs (Pinecone, Weaviate, Qdrant, Milvus, Chroma, LanceDB, RedisInsight, pgvector, Atlas, Vespa, Kibana, Typesense) |
| `02-desktop-db-tools/` | `RESEARCH.md` | 19 herramientas desktop de administración de DBs (DBeaver, TablePlus, Compass, Redis Insight, Neo4j Bloom, Datasette, Surrealist, etc.) — 11 lecciones P0–P3 |
| `03-ai-memory-graphs/` | `RESEARCH.md` | Memoria de agentes de IA y knowledge graphs (Mem0, Letta, Zep/Graphiti, LangGraph, GraphRAG, Neo4j, Obsidian, LightRAG/Cognee) — observabilidad y explicabilidad |
| `04-embedding-visualization/` | `RESEARCH.md` | Técnicas de reducción de dimensionalidad (UMAP/t-SNE/PaCMAP) + stack browser (regl-scatterplot, UMAP-js, WebGL) |
| `05-data-editor-ux/` | `RESEARCH.md` | UX/UI de edición de datos: grids (TanStack/AG Grid), editores JSON (CodeMirror/Monaco), key-value schema-less, query builders, master-detail |
| `06-synthesis/` | `SYNTHESIS.md` | **Concepto integrador "Vanta Studio"**: workspace unificado (Home/Memorias + lentes Retrieval/Grafo/Espacio/Operaciones), stack recomendado, plan por fases, gaps del core |
| `07-cognitive-psychology/` | `RESEARCH.md` | **Corrección cognitiva**: 5 debilidades detectadas en la síntesis (overview, split-attention de 5 tabs, timeline+diff, undo/papelera/palette, encoding de scores) con fuentes científicas (Shneiderman, Ware, Cleveland–McGill, Sweller, Norman) |

## Lectura recomendada

1. Empieza por `06-synthesis/SYNTHESIS.md` — el concepto completo y el plan por fases.
2. Profundiza en los reportes 01–05 según el área que te interese (cada uno tiene URLs reales).

## Contexto del modelo de datos de VantaDB que guía toda la investigación

- Registros `VantaMemoryRecord`: namespace, key, payload (texto), metadata (mapa arbitrario de `VantaValue`: string/int/float/bool/datetime/listas/null), created_at/updated_at_ms, version, node_id (u128), vector `Vec<f32>` opcional, sparse_vector, expires_at_ms (TTL).
- Búsqueda híbrida BM25 + HNSW + RRF; capa de grafo (nodos, aristas dirigidas con peso, acumuladores, IQL).
- Audit log JSONL, métricas operacionales, import/export (JSONL, `.vdbdump`).
- Almacenamiento: LSM (keyspaces/shard levels), RocksDB, WASM→IndexedDB/OPFS.
- Módulo desktop: Tauri v2 + React + Vite (en desarrollo temprano).