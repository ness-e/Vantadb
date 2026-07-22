# ADP-02: haystack — Fix filter parsing + count limit + OVERWRITE + serialization

## Metadata
- **Plan file:** `docs/plans/2026-07-22-adapter-10of10-campaign.md`
- **Creado:** 2026-07-22T12:00
- **Estado:** ⬜ PENDING

## Blast Radius
- `integrations/haystack/vantadb_haystack/vectorstore.py`
- Tests en `integrations/haystack/tests/`

## Contrato
Python syntax check pasa + todos los tests existentes pasan

## Steps

### Step 1: Implement Haystack filter syntax parser
- **Archivos:** `integrations/haystack/vantadb_haystack/vectorstore.py`
- **Acción:** Haystack usa sintaxis anidada con `{operator, conditions}` y `{field, operator, value}`. Traducir esto a filtros planos de VantaDB. Por ejemplo:
  - `{"field": "meta.file", "operator": "==", "value": "doc.pdf"}` → filtro VantaDB
  - `{"operator": "AND", "conditions": [...]}` → múltiples filtros
  - `{"operator": "OR", "conditions": [...]}` → OR lógico
- **Verify:** `filter_documents({"field": "meta.test", "operator": "==", "value": "val"})` funciona

### Step 2: Fix `count_documents()` limit
- **Archivos:** `integrations/haystack/vantadb_haystack/vectorstore.py`
- **Acción:** Reemplazar `list_memory(namespace, limit=10000)` con un método que no tenga límite artificial o que use un límite mucho mayor.
- **Verify:** `count_documents()` retorna count correcto

### Step 3: Implement DuplicatePolicy.OVERWRITE
- **Archivos:** `integrations/haystack/vantadb_haystack/vectorstore.py`
- **Acción:** En `write_documents()`, cuando `policy == DuplicatePolicy.OVERWRITE`, detectar duplicados (por ID) y reemplazar en vez de skippear.
- **Verify:** Escribir doc con mismo ID con OVERWRITE lo reemplaza

### Step 4: Fix `to_dict()` to save all config
- **Archivos:** `integrations/haystack/vantadb_haystack/vectorstore.py`
- **Acción:** Guardar `memory_limit_bytes`, `read_only`, `backend` en `to_dict()`.
- **Verify:** `from_dict(to_dict())` preserva todas las configs

## Dependencias
- Ninguna

## Context Save Point
- **Fecha:** 2026-07-22
- **Branch:** develop
