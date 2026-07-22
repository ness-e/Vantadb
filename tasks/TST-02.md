# TST-02: Tests for langchain + llamaindex adapters

## Metadata
- **Plan file:** `docs/plans/2026-07-22-adapter-10of10-campaign.md`
- **Creado:** 2026-07-22T12:00
- **Estado:** ⬜ PENDING

## Steps

### Step 1: Expand tests for langchain adapter
- **Archivos:** `integrations/langchain/tests/`
- **Acción:** Agregar tests para:
  - add_texts con metadatos
  - delete con ids
  - similarity_search con filtros
  - max_marginal_relevance_search (MMR)
  - Edge cases
- **Verify:** `python -m pytest integrations/langchain/tests/ -v`

### Step 2: Expand tests for llamaindex adapter
- **Archivos:** `packages/llamaindex/tests/`
- **Acción:** Agregar tests para:
  - add y delete
  - query con vector
  - get_nodes
  - Edge cases
- **Verify:** `python -m pytest packages/llamaindex/tests/ -v`
