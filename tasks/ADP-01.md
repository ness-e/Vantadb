# ADP-01: mem0 — Rewrite to implement VectorStoreBase

## Metadata
- **Plan file:** `docs/plans/2026-07-22-adapter-10of10-campaign.md`
- **Creado:** 2026-07-22T12:00
- **Estado:** ⬜ PENDING

## Blast Radius
- `integrations/mem0/vantadb_mem0/vectorstore.py` — reescritura completa
- `integrations/mem0/pyproject.toml` — posible ajuste de dependencias
- Tests existentes en `integrations/mem0/tests/`

## Contrato
Python syntax check pasa + implementa los 11 métodos de `mem0.vector_stores.base.VectorStoreBase`

## Steps

### Step 1: Investigate mem0 VectorStoreBase interface
- **Archivos:** N/A (documentación externa)
- **Acción:** Investigar la interfaz `VectorStoreBase` de mem0ai: `create_col()`, `insert()`, `get()`, `update()`, `delete()`, `search()`, `list_cols()`, `delete_col()`, `col_info()`, `reset()`, `keyword_search()`.
- **Verify:** Lista completa de métodos y sus firmas

### Step 2: Rewrite class to inherit from VectorStoreBase
- **Archivos:** `integrations/mem0/vantadb_mem0/vectorstore.py`
- **Acción:** Cambiar `class VantaDBVectorStore:` a `class VantaDBVectorStore(VectorStoreBase)`. Los 11 métodos abstractos deben implementarse. Firmas deben coincidir con lo que mem0 espera:
  - `create_col(name, vector_size, distance)` — crear colección (namespace)
  - `insert(vectors, payloads=None, ids=None)` — batch insert
  - `get(vector_id)` — leer por ID
  - `update(vector_id, vector=None, payload=None)` — actualizar
  - `delete(vector_id)` — eliminar
  - `search(query, vectors, top_k=5, filters=None)` — buscar
  - `list_cols()` — listar colecciones
  - `delete_col(name)` — eliminar colección
  - `col_info(name)` — info de colección
  - `reset()` — reset completo
  - `keyword_search()` — búsqueda BM25
- **Verify:** `python -c "from vantadb_mem0.vectorstore import VantaDBVectorStore"` funciona

### Step 3: Fix score conversion for mem0 standard
- **Archivos:** `integrations/mem0/vantadb_mem0/vectorstore.py`
- **Acción:** Los scores deben estar en rango [0, 1] donde 1 = más similar. Convertir distancia coseno (0 = iguales) a score.
- **Verify:** score está en [0, 1]

### Step 4: Update pyproject.toml dependencies
- **Archivos:** `integrations/mem0/pyproject.toml`
- **Acción:** Verificar que `mem0ai>=0.1.0` es correcto y que hay import de VectorStoreBase.
- **Verify:** `pip install -e integrations/mem0` funciona

## Dependencias
- Ninguna

## Context Save Point
- **Fecha:** 2026-07-22
- **Branch:** develop
- **Decisiones:** mem0 VectorStoreBase es la interfaz correcta. Los 11 métodos deben implementarse para compatibilidad total.
