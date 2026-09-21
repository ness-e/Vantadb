# Upstream PR draft — LangChain

**Prereq (bloquea merge upstream):** publicar `vantadb-langchain` en PyPI (tag `adapters-v0.5.0`). Upstream no linkea paquetes 404.

**Target repo:** `langchain-ai/langchain`
**Branch sugerida:** `patch-add-vantadb-vectorstore-docs`

## Cambio propuesto

Página de integrações de vector stores (docs), estilo partner listing:

- `libs/docs/docs/docs/integrations/vectorstores/index.md` (o equivalente actual): agregar entrada "VantaDB" → link a nuestra doc.

## PR title

`docs(integrations): add VantaDB vector store provider`

## PR body

```markdown
Adds [VantaDB](https://github.com/ness-e/Vantadb) to the vector-stores integrations list.

- PyPI: `vantadb-langchain` (implements `langchain_core.vectorstores.VectorStore`)
- Embedded, local-first Rust engine (no server); hybrid BM25+HNSW search
- Adapter source: https://github.com/ness-e/Vantadb/tree/main/integrations/langchain
- License: Apache-2.0 | Maintainers: VantaDB team

Quickstart: `pip install vantadb-langchain` → `from vantadb_langchain import VantaDBVectorStore`
```
