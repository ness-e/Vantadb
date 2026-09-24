# MOD-16 — Suite pytest default rota (66 failed: RSS acumulado sin teardown)

## Objetivo
`pytest -q` en `vantadb-python/tests/` exit 0 (suite default completa verde), con
fixture autouse en conftest que cierre TODAS las DBs abiertas por test.

## Impacto mapeado (Regla 0)
**Archivos leídos completos:**
- `vantadb-python/tests/test_async_smoke.py` (134L) — usa `async with`, cierra bien; NO es fuente de leak.
- `vantadb-python/vantadb_py/__init__.py` (406L) — `AsyncVantaDB` envuelve `_sync: VantaDB`; `close()` async delega al sync vía to_thread.
- `vantadb-python/pyproject.toml` — marker `slow` excluido del run default (`addopts = "-m 'not slow'"`); sin conftest previo.
- `src/storage/engine/stats.rs:88-170` (`check_memory_pressure`) — mide **RSS del proceso completo** (`_get_rss_virt()`) contra el `memory_limit` **por-DB** (tests fijan 128MB) → cualquier acumulación cruza el umbral y rechaza writes con `VantaError::ResourceLimit("Memory pressure: …")`.

**Referencias hacia dentro:** ningún test importa un conftest (no existe hoy); pytest lo carga automáticamente antes de los módulos de test.
**Referencias entrantes:** ninguna (archivo nuevo `tests/conftest.py`); no se modifica código Rust ni la API pública.
**Veredicto:** cambio acotado a 1 archivo nuevo de test-infra. Riesgo bajo. Sin FFI/security/performance gates aplicables (no toca trust boundaries ni hot paths).

## Discovery (evidencia)
- Repro estado actual: `pytest -q` = **66 failed, 43 passed, 4 deselected** (~74s). Clasificación por archivo:
  - `test_perf_15_16.py`: 8 failed · `test_sdk.py`: 58 failed.
  - `test_async_smoke.py`, `test_load.py`, `test_migration.py`, `test_subclients.py`: pasan PERO envenenan el proceso — `test_load.py` crea DBs con bulk inserts (límites hasta 1024MB) y tiene **cero** `.close()`; `test_perf` deja 1 solo close para ~7 instancias.
- Claim histórico verificado: `pytest tests/test_sdk.py` solo = **70 passed exactos** → corrida parcial confirmada.
- Test individual post-guard pasa (`TestTTL::test_put_with_ttl` verde aislada) → failures son interferencia entre tests, no bugs unitarios.
- Instancias PyO3 **NO** son visibles vía `gc.get_objects()` ni weakref-ables (verificado empíricamente) → patrón registry por wrapper de constructor es el único camino.
- `close()` Rust es **idempotente** (segundo close OK, verificado) → teardown tolera dobles cierres.
- Puntos de construcción en tests: `vanta.VantaDB(...)` (5 archivos), `from vantadb_py import VantaDB` (test_migration), `vanta.AsyncVantaDB(...)` (test_sdk/test_async_smoke). `connect()` de lancedb es librería externa, no nuestro binding.
- Rebinding del global del paquete `vantadb_py.VantaDB` en conftest cubre TODAS las rutas: los tests importan el paquete después de conftest, y `AsyncVantaDB.__init__` resuelve `VantaDB` como global de módulo → el `_sync` interno queda registrado también.

## Spec
Fixture autouse function-scoped en `tests/conftest.py` nuevo:
1. Al import: guardar clase original y rebind `vantadb_py.VantaDB` a factory que registra la instancia en `_REGISTRY` y devuelve el objeto real (isinstance/repr intactos).
2. Fixture autouse: limpia registro → `yield` → cierra todas las instancias pendientes (`try/except` — teardown nunca enmascara el resultado del test).

## Steps
- ⬜ Step 1: crear `tests/conftest.py` con registry + fixture autouse.
- ⬜ Step 2: VERIFY — `pytest -q` completo exit 0; clasificar failures residuales (REGLA: >3 persistentes → documentar como bugs reales y RESULTADO INCOMPLETO).
- ⬜ Step 3: commit conventional + cierre.

## Contrato
`pytest -q` en `vantadb-python/tests/` exit 0 (suite default completa), fixture autouse cierra todas las DBs abiertas.

## Hallazgos (bugs reales post-fixture)
(ninguno todavía — llenar en Step 2 si aplica)

## Context Save Point
- Estado: DISCOVERY completo, implementación no iniciada.
- Comando repro: `& ".venv\Scripts\python.exe" -m pytest -q` desde `vantadb-python/`.
