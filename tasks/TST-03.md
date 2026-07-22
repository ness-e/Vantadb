# TST-03: Tests for crewai + dspy + haystack adapters

## Metadata
- **Plan file:** `docs/plans/2026-07-22-adapter-10of10-campaign.md`
- **Creado:** 2026-07-22T12:00
- **Estado:** ⬜ PENDING

## Steps

### Step 1: Expand tests for crewai adapter
- **Archivos:** `integrations/crewai/tests/`
- **Acción:** Tests para embedding mockeado, _put con metadata, categorize con diferentes inputs
- **Verify:** `python -m pytest integrations/crewai/tests/ -v`

### Step 2: Expand tests for dspy adapter
- **Archivos:** `integrations/dspy/tests/`
- **Acción:** Tests para Prediction return type, dump_state/load_state, metadata en _add
- **Verify:** `python -m pytest integrations/dspy/tests/ -v`

### Step 3: Expand tests for haystack adapter
- **Archivos:** `integrations/haystack/tests/`
- **Acción:** Tests para DuplicatePolicy (SKIP/FAIL/OVERWRITE), filter con operadores, search con embedding, to_dict/from_dict roundtrip
- **Verify:** `python -m pytest integrations/haystack/tests/ -v`
