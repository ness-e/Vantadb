# FTR-01: Async methods for openai + ollama

## Metadata
- **Plan file:** `docs/plans/2026-07-22-adapter-10of10-campaign.md`
- **Creado:** 2026-07-22T12:00
- **Estado:** ⬜ PENDING

## Steps

### Step 1: Add async methods to openai adapter
- **Archivos:** `integrations/openai/vantadb_openai/vectorstore.py`
- **Acción:** Agregar:
  - `async aadd_texts(texts, metadatas=None, **kwargs)` — wrapper async de add_texts
  - `async asimilarity_search(query, k=4, **kwargs)` — wrapper async de similarity_search
  - `async adelete(ids=None, **kwargs)` — wrapper async de delete
- **Verify:** Python syntax check pasa

### Step 2: Add async methods to ollama adapter
- **Archivos:** `integrations/ollama/vantadb_ollama/vectorstore.py`
- **Acción:** Mismos métodos async que openai
- **Verify:** Python syntax check pasa
