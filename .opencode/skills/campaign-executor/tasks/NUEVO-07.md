# NUEVO-07: Migration tools — Chroma→Vanta, LanceDB→Vanta

## Metadata
- **Plan file:** ninguno activo (backlog directo)
- **Fuente:** docs/Backlog.md línea 117 — "⚠️ Tutoriales OK, scripts ejecutables faltan"
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠
- **Tipo:** Mixto (Python SDK + Docs)
- **Turns estimados:** 20-30
- **Creado:** 2026-08-02
- **Estado:** ⬜ PENDING

## Contexto verificado (2026-08-02)
- `vantadb_py/migrate/` **NO existe** — el audit 2026-07-28 (backlog-validation) reportó un falso positivo ("chroma.py + lancedb.py existen"). Los scripts nunca existieron.
- Tutoriales SÍ existen: `docs/tutorials/03-migrating-from-chromadb.md` (331 líneas, status: draft) y `docs/tutorials/migration-from-lancedb.md` (483 líneas, status: active). Ambos prometen "a migration script you can run" — el script no existe.
- **API real del SDK** (`vantadb-python/src/lib.rs:589`): `VantaDB(db_path, memory_limit_bytes=None, read_only=False, backend=None)`; `put(namespace, key, payload, metadata=None, vector=None, ttl_ms=None)` → `VantaMemoryRecord` (fields: namespace, key, payload, metadata, vector, created_at_ms, updated_at_ms, version, node_id, expires_at_ms); `get_memory(namespace, key)`; `search_memory(...)`; `export_namespace(path, namespace)`; `export_all(path)`; `import_file(path)`. Ejemplo oficial en lib.rs:575-585: `db = VantaDB(":memory:", backend="memory")`.
- **Los tutoriales usan API desactualizada**: `vantadb.connect(path)`, `db.space(name)`, `space.put(...)` — NO existe en el SDK. Los scripts DEBEN usar `vantadb_py.VantaDB(...)`; los tutoriales DEBEN corregirse para referenciar los scripts reales.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | docs/tutorials/03-migrating-from-chromadb.md, docs/tutorials/migration-from-lancedb.md, README (potencial) |
| Callees | `vantadb_py` SDK (VantaDB::put, get_memory, search_memory, export_namespace, import_file), chromadb (fuente), lancedb (fuente) |
| Implicaciones | Scripts Python standalone nuevos — NO tocan Rust core, NO rompen contratos. Corrige deuda: tutoriales con API inventada |

**RIESGO:** bajo. Archivos Python nuevos + edición de 2 tutoriales. Sin cambios en `src/`, bindings ni CI.

## Contrato
"`python -m vantadb_py.migrate.chroma --help` y `python -m vantadb_py.migrate.lancedb --help` funcionan; un smoke test migra 3 registros demo desde cada fuente a VantaDB y `get_memory`/`search_memory` los recupera; los tutoriales referencian los scripts con path exacto y usan la API `vantadb_py.VantaDB` real."

## Herramientas necesarias
- Python (venv `target/audit-venv` con `vantadb_py` instalado — maturin build)
- `vantadb-python/tests/test_sdk.py` como referencia de uso del SDK
- No cargo-mcp (no se toca Rust)

## Investigation Notes
- ChromaDB export: `chromadb.PersistentClient(path)` → `collection.get(include=["documents","metadatas","embeddings"])` devuelve dict con ids/documents/metadatas/embeddings.
- LanceDB export: `lancedb.connect(path)` → `table.to_pandas()` o `table.search()`; schema Arrow → mapear columnas a payload/metadata/vector.
- TTL/metadata: mapear metadata Chroma/Lance a `metadata` de VantaDB (tipos escalares: str/int/float/bool/datetime/list/None).

## Steps

### Step 1: Crear paquete `vantadb_py/migrate/`
- **Archivos:** `vantadb_py/migrate/__init__.py` (exporta `migrate_from_chroma`, `migrate_from_lancedb`)
- **Acción:** crear directorio y `__init__.py` con docstring del paquete y funciones públicas
- **Verify:** `python -c "import vantadb_py.migrate"`
- **Estado:** ⬜ PENDING

### Step 2: Script `chroma.py`
- **Archivos:** `vantadb_py/migrate/chroma.py`
- **Acción:** CLI + función `migrate_from_chroma(source_path, dest_path, collection_name=None, namespace=None, batch_size=500)`. Lee ChromaDB, escribe con `VantaDB.put()` (API real, no `space.put`). Soporta `python -m vantadb_py.migrate.chroma --source ... --dest ...`
- **Verify:** `python -m vantadb_py.migrate.chroma --help` exit 0
- **Estado:** ⬜ PENDING

### Step 3: Script `lancedb.py`
- **Archivos:** `vantadb_py/migrate/lancedb.py`
- **Acción:** CLI + función `migrate_from_lancedb(source_path, dest_path, table_name=None, namespace=None, batch_size=500)`. Lee LanceDB (to_pandas), mapea columnas → payload/metadata/vector, escribe con `VantaDB.put()`
- **Verify:** `python -m vantadb_py.migrate.lancedb --help` exit 0
- **Estado:** ⬜ PENDING

### Step 4: Smoke test end-to-end
- **Archivos:** `vantadb_py/tests/test_migration.py` (o script demo)
- **Acción:** crear demo con datos ChromaDB/LanceDB de ejemplo (3 registros c/u), migrar, verificar con `get_memory` + `search_memory`
- **Verify:** pytest pasa; registros recuperados
- **Estado:** ⬜ PENDING

### Step 5: Actualizar tutoriales
- **Archivos:** `docs/tutorials/03-migrating-from-chromadb.md`, `docs/tutorials/migration-from-lancedb.md`
- **Acción:** reemplazar API inventada (`vantadb.connect`/`db.space`) por la API real `vantadb_py.VantaDB`; agregar sección "Migration script" con comando exacto `python -m vantadb_py.migrate.chroma ...`
- **Verify:** grep confirma 0 ocurrencias de `vantadb.connect(`/`space.put(` en los tutoriales
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna (task independiente)

## Notas
- El audit 2026-07-28 falló aquí: dijo scripts ✅ cuando no existen. Este task file documenta el estado real (2026-08-02).
- No usar `vantadb.connect` — verificar siempre contra lib.rs docstrings (API: `vantadb_py.VantaDB`).

## Context Save Point
- **Fecha:** 2026-08-02
- **Branch:** (según estado actual del repo)
- **CI pendiente:** no
- **Decisiones:** scripts en `vantadb_py/migrate/` (paquete Python, no binarios separados) — reutiliza venv existente; API real del SDK, no la de los tutoriales
- **Problemas conocidos:** tutoriales con API desactualizada (se corrigen en Step 5)
- **Próxima tarea:** — (una sola task)
