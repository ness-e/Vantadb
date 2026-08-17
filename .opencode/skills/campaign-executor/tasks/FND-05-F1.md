# FND-05-F1: Corregir 4 drifts de stub Python + limpiar maturin features redundante

## Metadata
- **Plan:** docs/plans/2026-08-16-wave-followups.md (W2)
- **Fuente:** FND-05 (análisis SDK idiomático, gaps documentados con archivo:línea)
- **Estado:** ⏳ IN PROGRESS · **Sub-agente:** vanta-worker
- **Prioridad:** 🔴

## Objetivo
Corregir los 4 drifts de stub de `vantadb-python` que rompen type-checking del usuario:
1. `connect` declarado en `vantadb_py/__init__.pyi:293` pero NO exportado en `__init__.py` → alinear (si el runtime tiene la función, exportarla en `__init__.py`; si no, corregir el stub)
2. `__version__` declarado como función (`def __version__() -> str`) pero runtime es atributo string (lib.rs:1962 `add("__version__")`) → corregir stub
3. Tipos de retorno de `get_memory` / `list_memory` / `put` desalineados con lib.rs real (lib.rs:793 get_memory → `VantaPyMemoryRecord | None`; lib.rs:880 list_memory → `VantaListResult`; put → `VantaPyMemoryRecord`)
4. Bonus: limpiar `[tool.maturin] features = ["pyo3/extension-module"]` redundante en `vantadb-python/pyproject.toml` (ya en `vantadb-python/Cargo.toml:15`) — SOLO si no rompe el build

## Archivos clave
- `vantadb-python/vantadb_py/vantadb_py.pyi`, `__init__.pyi`, `__init__.py`, `vantadb-python/pyproject.toml`, `vantadb-python/src/lib.rs` (runtime real)

## Steps
1. DISCOVERY: leer lib.rs (exports reales de PyO3: qué pyclass/functions se registran), stubs .pyi, __init__.py; confirmar los 4 drifts con archivo:línea
2. Corregir stubs/`__init__.py` para alinear con runtime real (fuente de verdad = lib.rs). Si `connect` no existe en runtime → quitarlo del stub (NO inventar API runtime)
3. Limpiar pyproject.toml (maturin features redundante) si el build sigue intacto
4. Verificar: `python -m py_compile` de los .py/.pyi + `cargo check -p vantadb_py` + (si disponible) mypy/pyright sobre los stubs + `maturin build --dry-run` o al menos confirmación de que pyproject sigue parseable
5. Task file + RESULTADO

## Contrato (verify mecánico)
- Stubs alineados con runtime (cada export del stub existe en lib.rs o __init__.py; tipos de retorno coinciden)
- `cargo check -p vantadb_py` pasa
- `python -m py_compile` pasa
- pyproject parseable (maturin features limpiado sin romper)

## Invariantes (handoff)
- NO tocar: docs/Backlog.md, AUD-024.md, verify-log.jsonl, _vanta-cli.ps1, plan file, .opencode/AGENTS.md, .opencode/agents/*
- NO git add/commit; NO campaign_update_task_state
- Runtime lib.rs NO cambia (solo stubs/__init__.py/pyproject) — el runtime ya es correcto
- No agregar API que no exista en runtime; no quitar exports reales

## Fases
- SECURITY: n/a
- PERFORMANCE: n/a

## Resultado
```
RESULTADO: ✅ COMPLETO | 🟡 INCOMPLETO | ❌ FALLIDO
STEPS_OK: <n>/<M>
PROXIMO_STEP: <...>
COMMIT_HASH: ninguno (lead commitea)
ARCHIVOS: <paths tocados>
VERIFY_CONTRATO: <pasa | no-corrido | falla>
BLOQUEO: <ninguno | ...>
```