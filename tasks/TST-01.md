# TST-01: Tests for openai + ollama Python adapters

## Metadata
- **Plan file:** `docs/plans/2026-07-22-adapter-10of10-campaign.md`
- **Creado:** 2026-07-22T12:00
- **Estado:** ⬜ PENDING

## Steps

### Step 1: Create tests for openai Python adapter
- **Archivos:** Crear `integrations/openai/tests/`
- **Acción:** Tests que cubran:
  - Constructor con parámetros
  - add_texts (con y sin metadata)
  - similarity_search (con query)
  - delete
  - Edge cases: empty texts, None values
- **Verify:** `python -m pytest integrations/openai/tests/ -v`

### Step 2: Create tests for ollama Python adapter
- **Archivos:** Crear `integrations/ollama/tests/`
- **Acción:** Misma cobertura que openai
- **Verify:** `python -m pytest integrations/ollama/tests/ -v`
