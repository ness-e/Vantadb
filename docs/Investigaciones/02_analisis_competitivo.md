# Investigación 2 — Análisis Competitivo Profundo

> VantaDB vs el ecosistema de bases de datos vectoriales en 2026.

> ⚠️ **Hallazgo crítico:** ChromaDB ha lanzado soporte nativo para BM25 y SPLADE, lo cual invalida claims anteriores de que ChromaDB carecía de búsqueda híbrida.

## Mapa del Ecosistema

| Segmento | Competidores | Relevancia |
|---|---|---|
| Embebido / Local-First | ChromaDB, LanceDB | Competencia directa |
| Servidor dedicado | Qdrant, Weaviate | Competencia en features |
| Cloud managed | Pinecone | Competencia de mindshare |
| Extensión relacional | pgvector | Competencia indirecta |

## Diferenciación Real de VantaDB

> "El único motor embebido que garantiza durabilidad a nivel WAL con CRC32C, unifica vectores + texto + grafo en transacciones atómicas, y ofrece GraphRAG nativo — sin servidor, sin configuración, y sin dependencias C++."

### Brechas urgentes (pre-Show HN)
1. **Latencia Python SDK**: 62ms → necesita estar <20ms
2. **TypeScript SDK**: Sin TS SDK el TAM es la mitad

Ver también: `docs/VantaDB-MPTS/Estrategia de Ecosistema y GTM.md`
