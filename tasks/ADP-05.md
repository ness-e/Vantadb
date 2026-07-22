# ADP-05: crewai — Fix error path + categorize + validation

## Metadata
- **Plan file:** `docs/plans/2026-07-22-adapter-10of10-campaign.md`
- **Creado:** 2026-07-22T12:00
- **Estado:** ⬜ PENDING

## Blast Radius
- `integrations/crewai/vantadb_crewai/vectorstore.py`
- Tests en `integrations/crewai/tests/`

## Contrato
Python syntax check pasa + tests existentes pasan

## Steps

### Step 1: Fix type inconsistency in _run() fallback path
- **Archivos:** `integrations/crewai/vantadb_crewai/vectorstore.py`
- **Acción:** En el fallback (sin embedding), `list_memory()` retorna `VantaMemoryListResult` con campo `.records`. Asegurar que el acceso a `.records` es consistente. También en el path de búsqueda, tratar `results` como objeto estructurado en vez de iterable directo.
- **Verify:** `_run("test")` funciona con y sin embedding

### Step 2: Implement categorize() with real logic
- **Archivos:** `integrations/crewai/vantadb_crewai/vectorstore.py`
- **Acción:** `categorize()` actualmente es stub que siempre devuelve "informational". Implementar lógica usando embedding para categorizar texto (ej: pregunta técnica → "technical", saludo → "greeting", etc.) o al menos basar en keywords.
- **Verify:** `categorize("hello")` ≠ "informational"

### Step 3: Guard embedding=None in _put()
- **Archivos:** `integrations/crewai/vantadb_crewai/vectorstore.py`
- **Acción:** Si `self.embedding is None`, no llamar a `self.embedding(text)`. Insertar sin vector.
- **Verify:** `_put("text", None)` no tira error

### Step 4: Accept k from kwargs in _run()
- **Archivos:** `integrations/crewai/vantadb_crewai/vectorstore.py`
- **Acción:** Si `kwargs` contiene `k` (top_k), usarlo en vez del default de 4.
- **Verify:** `_run("query", k=10)` usa 10 resultados

### Step 5: Add input validation
- **Archivos:** `integrations/crewai/vantadb_crewai/vectorstore.py`
- **Acción:** Validar que `query` no sea None/vacío en `_run()`. Validar que `text` no sea None en `_put()`.
- **Verify:** Query vacío da error descriptivo

## Dependencias
- Ninguna

## Context Save Point
- **Fecha:** 2026-07-22
- **Branch:** develop
