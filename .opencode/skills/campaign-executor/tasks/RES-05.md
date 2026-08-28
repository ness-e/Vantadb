# RES-05: Context manager síncrono __enter__/__exit__ en Py binding

## Metadata
- **Plan file:** docs/plans/2026-08-28-backlog-triage.md
- **Creado:** 2026-08-28T14:30:00
- **last-synced:** 2026-08-28T15:15:00
- **Estado:** ✅ COMPLETED

## Blast Radius
**Archivos modificados:**
- `vantadb-python/src/lib.rs` — Agregar métodos `__enter__` y `__exit__` al impl `VantaDB`

**Referencias entrantes:** 
- `vantadb-python/vantadb_py/__init__.py` — Re-exporta `VantaDB`, no necesita cambios
- Tests Python (si existen) — Verificar que el context manager funciona

**Implicaciones:**
- Permite uso idiomático de Python: `with VantaDB(...) as db:`
- El WAL se flushea automáticamente al salir del bloque `with`
- Complementa el `AsyncVantaDB` existente que ya tiene `__aenter__`/`__aexit__`

## Contrato
```powershell
Select-String -Path "vantadb-python/src/lib.rs" -Pattern "__enter__|__exit__" | Measure-Object | Select-Object Count
```
Debe retornar `>=2`

Verificación completa:
- `cargo check -p vantadb-python` — compila sin errores
- `cargo nextest run -p vantadb-python` — tests pasan (si existen)
- Test manual: `python -c "from vantadb_py import VantaDB; with VantaDB(':memory:', backend='memory') as db: db.put('ns', 'k', 'v')"`

## Herramientas
- `cargo check`, `cargo nextest`, `codegraph_explore` para blast radius
- `python` para verificación manual

## Steps

### Step 1: Agregar métodos __enter__ y __exit__ a VantaDB en lib.rs
- **Archivos:** `vantadb-python/src/lib.rs`
- **Acción:** Implementar `__enter__` (retorna `self`) y `__exit__` (llama `close()` para durabilidad completa)
- **Verify:** `cargo check -p vantadb-python` ✅
- **Estado:** ✅ COMPLETED

### Step 2: Verificación completa y test manual
- **Archivos:** `vantadb-python/src/lib.rs`
- **Acción:** Compilar, testear y verificar uso con `with` statement
- **Verify:** `cargo check -p vantadb-python && cargo nextest run -p vantadb-python` + test manual Python ✅
- **Estado:** ✅ COMPLETED

## Dependencias
- WSM-03 (auto-save WASM) — parallel wave, sin dependencia directa
- PY-03 (import alias) — completado, no afecta

## Notas
- `__enter__` retorna `PyRef<'_, Self>` — PyO3 permite ambos
- `__exit__` signature: `fn __exit__(&self, py: Python, exc_type: Option<&Bound<'_, PyAny>>, exc_val: Option<&Bound<'_, PyAny>>, exc_tb: Option<&Bound<'_, PyAny>>) -> PyResult<()>`
- **Decisión final:** `__exit__` llama `close()` (no `flush()`) para paridad con `AsyncVantaDB.__aexit__` y durabilidad total. El handle no se reutiliza tras `with` — patrón estándar Python.
- La implementación de `close()` ya tiene barrera de durabilidad (OpGate.drain)

## Context Save Point
- **Fecha:** 2026-08-28T15:15:00
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** Usar `close()` en `__exit__` para paridad con async y durabilidad total
- **Problemas conocidos:** Ninguno
- **Próxima tarea:** PY-03 (si pendiente) o siguiente en wave