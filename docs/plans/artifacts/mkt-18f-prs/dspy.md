# Upstream PR draft — DSPy

**Prereq (bloquea merge upstream):** publicar `vantadb-dspy` en PyPI (tag `adapters-v0.5.0`).

**Target repo:** `stanfordnlp/dspy`
**Branch sugerida:** `patch-vantadb-retriever-example`

## Cambio propuesto

- DSPy acepta `retriever=` custom — lo canónico upstream es un ejemplo en `docs/` ("Retriever with external vector store"), no un core hook.
- Agregar página/entrada de ejemplo usando `vantadb-dspy.VantaDBRetriever`.

## PR title

`docs: add VantaDB embedded retriever example`

## PR body

```markdown
Adds an example of using [VantaDB](https://github.com/ness-e/Vantadb) as a DSPy retriever via `vantadb-dspy`.

- `VantaDBRetriever` satisfies the dspy retriever protocol; pass as `retriever=` to `dspy.Retrieve`
- Embedded local-first Rust engine (persistent hybrid BM25+HNSW); no server, works fully offline
- Source: https://github.com/ness-e/Vantadb/tree/main/integrations/dspy
- License: Apache-2.0
```
