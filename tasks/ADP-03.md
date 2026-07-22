# ADP-03: dspy — Fix forward return type + dump_state + metadata

## Metadata
- **Plan file:** `docs/plans/2026-07-22-adapter-10of10-campaign.md`
- **Creado:** 2026-07-22T12:00
- **Estado:** ⬜ PENDING

## Blast Radius
- `integrations/dspy/vantadb_dspy/vectorstore.py`
- Tests en `integrations/dspy/tests/`

## Contrato
Python syntax check pasa + tests existentes pasan

## Steps

### Step 1: Fix `forward()` return type
- **Archivos:** `integrations/dspy/vantadb_dspy/vectorstore.py`
- **Acción:** Cambiar retorno de `return passages` a `return dspy.Prediction(passages=passages)`. El protocolo DSPy Retrieve espera un objeto Prediction con atributo `.passages`.
- **Verify:** `python -c "from dspy import Prediction; isinstance(ret, Prediction)"` funciona

### Step 2: Implement `dump_state()` / `load_state()` with full state
- **Archivos:** `integrations/dspy/vantadb_dspy/vectorstore.py`
- **Acción:** Sobreescribir `dump_state()` para incluir `namespace`, `db_path`, `backend`, `memory_limit`, etc. DSPy optimizadores como MIPROv2 llaman estos métodos.
- **Verify:** `dump_state()` serializa todo el estado relevante

### Step 3: Add metadata support to `_add()`
- **Archivos:** `integrations/dspy/vantadb_dspy/vectorstore.py`
- **Acción:** Cambiar firma a `_add(text, key, metadata=None)` y pasar metadata a `self._db.put()`.
- **Verify:** Documentos insertados tienen metadata

### Step 4: Add k passthrough in forward()
- **Archivos:** `integrations/dspy/vantadb_dspy/vectorstore.py`
- **Acción:** Aceptar `k` como kwarg en `forward()`. Si se pasa, sobreescribe `self.k`.
- **Verify:** `forward(query, k=20)` usa k=20

## Dependencias
- Ninguna

## Context Save Point
- **Fecha:** 2026-07-22
- **Branch:** develop
