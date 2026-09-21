# Upstream PR draft — LlamaIndex

**Prereq (bloquea merge upstream):** publicar `vantadb-llamaindex` en PyPI (tag `adapters-v0.5.0`).

**Target repo:** `run-llama/llama_index`
**Branch sugerida:** `patch-add-vantadb-vector-store`

## Cambio propuesto

- `docs/docs/api_reference/vector_stores/index.md`: agregar fila "VantaDB" → página de módulo.
- LlamaIndex también acepta paquetes `llama-index-vector-stores-*` en su registry propio; nuestro nombre es `vantadb-llamaindex` (no sigue el prefijo) — para el listing basta el doc link; si upstream exige el naming estándar, sería renombre de paquete (decisión humana, NO asumir).

## PR title

`feat: add VantaDB to vector stores docs listing`

## PR body

```markdown
Adds [VantaDB](https://github.com/ness-e/Vantadb) — embedded persistent vector memory — to the supported vector stores listing.

- Integration package: `vantadb-llamaindex` (implements `llama_index.core.vector_stores.types.VectorStore`)
- Local-first embedded Rust engine; hybrid BM25+HNSW; no server required
- Source: https://github.com/ness-e/Vantadb/tree/main/integrations/llamaindex
- License: Apache-2.0

Note: package name intentionally avoids the `llama-index-vector-stores-` prefix to keep the VantaDB brand family consistent (openai/mem0/crewai/dspy siblings). Happy to discuss if a naming exception is required.
```
