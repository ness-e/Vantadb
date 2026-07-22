# ADP-04: letta — Fix dependency + validation + serialization

## Metadata
- **Plan file:** `docs/plans/2026-07-22-adapter-10of10-campaign.md`
- **Creado:** 2026-07-22T12:00
- **Estado:** ⬜ PENDING

## Blast Radius
- `integrations/letta/vantadb_letta/vectorstore.py`
- `integrations/letta/pyproject.toml`
- Tests en `integrations/letta/tests/`

## Contrato
Python syntax check pasa + pyproject.toml con letta-client correcto

## Steps

### Step 1: Fix dependency in pyproject.toml
- **Archivos:** `integrations/letta/pyproject.toml`
- **Acción:** Cambiar `letta>=0.1.0` a `letta-client>=1.0.0`. El paquete correcto para el SDK de Letta es `letta-client`, no `letta`.
- **Verify:** `pip install letta-client` funciona

### Step 2: Add to_dict/from_dict serialization
- **Archivos:** `integrations/letta/vantadb_letta/vectorstore.py`
- **Acción:** Implementar `to_dict()` y `from_dict()` para serializar/deserializar estado del vector store. Letta pipeline espera esto.
- **Verify:** `from_dict(to_dict())` es round-trip estable

### Step 3: Add input validation
- **Archivos:** `integrations/letta/vantadb_letta/vectorstore.py`
- **Acción:** Validar texto vacío en `insert()`, k<=0 en `search()`, limit inválido en `list()`. Levantar excepciones descriptivas.
- **Verify:** Insertar texto vacío da error claro

### Step 4: Add filters support to list()
- **Archivos:** `integrations/letta/vantadb_letta/vectorstore.py`
- **Acción:** Agregar parámetro `filters` a `list()` que filtre por source, agent_id, etc.
- **Verify:** `list(filters={"source": "test"})` filtra correctamente

## Dependencias
- Ninguna

## Context Save Point
- **Fecha:** 2026-07-22
- **Branch:** develop
