# FTR-03: LlamaIndex hybrid mode

## Metadata
- **Plan file:** `docs/plans/2026-07-22-adapter-10of10-campaign.md`
- **Creado:** 2026-07-22T12:00
- **Estado:** ⬜ PENDING

## Steps

### Step 1: Implement hybrid_mode for llamaindex adapter
- **Archivos:** `packages/llamaindex/vantadb_llamaindex/vectorstore.py`
- **Acción:** Actualmente `hybrid_mode` existe como flag pero no hace nada. Implementarlo:
  - Cuando `hybrid_mode=True`, la búsqueda usa tanto el embedding (vector search) como el texto (BM25/full-text search)
  - Combinar resultados con `Reciprocal Rank Fusion (RRF)` o weighted sum
  - La API de VantaDB soporta `text_query` en search_memory para hybrid search
- **Verify:** `query("text", hybrid_mode=True)` devuelve resultados relevantes
