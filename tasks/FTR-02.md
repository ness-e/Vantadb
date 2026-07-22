# FTR-02: MMR for langchain + llamaindex

## Metadata
- **Plan file:** `docs/plans/2026-07-22-adapter-10of10-campaign.md`
- **Creado:** 2026-07-22T12:00
- **Estado:** ⬜ PENDING

## Steps

### Step 1: Implement MMR for langchain adapter
- **Archivos:** `integrations/langchain/vantadb_langchain/vectorstore.py`
- **Acción:** Implementar `max_marginal_relevance_search(query, k=4, fetch_k=20, lambda_mult=0.5)`.
  - fetch_k: cuántos resultados traer inicialmente
  - lambda_mult: balance entre relevancia y diversidad (0 = solo diversidad, 1 = solo relevancia)
  - Algoritmo: traer fetch_k docs → embedding de query + de los docs → MMR score → seleccionar top_k diversos
- **Verify:** Python syntax check + `max_marginal_relevance_search` funciona

### Step 2: Implement MMR for llamaindex adapter
- **Archivos:** `packages/llamaindex/vantadb_llamaindex/vectorstore.py`
- **Acción:** Implementar MMR similar. LlamaIndex tiene `query` que acepta parámetro de modo.
- **Verify:** Python syntax check
