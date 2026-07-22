# TST-04: Tests for letta + mem0 adapters

## Metadata
- **Plan file:** `docs/plans/2026-07-22-adapter-10of10-campaign.md`
- **Creado:** 2026-07-22T12:00
- **Estado:** ⬜ PENDING

## Steps

### Step 1: Expand tests for letta adapter
- **Archivos:** `integrations/letta/tests/`
- **Acción:** Tests para:
  - to_dict/from_dict roundtrip
  - insert con embedding mockeado
  - search con embedding mockeado
  - list con filters
  - Edge cases (texto vacío, k<=0)
- **Verify:** `python -m pytest integrations/letta/tests/ -v`

### Step 2: Verify tests for mem0 adapter
- **Archivos:** `integrations/mem0/tests/`
- **Acción:** Verificar que los 19 tests existentes (creados en ADP-01) pasan. Si falta algo, agregar tests adicionales.
- **Verify:** `python -m pytest integrations/mem0/tests/ -v`
