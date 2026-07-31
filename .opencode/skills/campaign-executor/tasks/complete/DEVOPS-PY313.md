# DEVOPS-PY313: Python 3.13 wheels en CI matrix

## Metadata
- **Plan file:** P8 Post-Launch & Enterprise
- **Fuente:** `docs/Backlog.md:198`
- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🟡
- **Tipo:** CI/CD
- **Turns estimados:** 3-5
- **Estado:** ⬜ PENDING

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | `python_wheels.yml` CI workflow |
| Callees | PyPI publishing. Maturin build matrix. |
| Implicaciones | Solo CI config. Nuevo target Python 3.13 en la matrix de builds. Sin cambios de código. |

## Contrato
"`.github/workflows/python_wheels.yml` incluye `3.13` en la matrix de Python. `cibuildwheel` no falla en dry-run."

## Pasos
1. Leer `python_wheels.yml` — identificar la matrix actual de Python versions
2. Agregar `3.13` a la matrix
3. Verificar que `pyproject.toml` en `vantadb-python/` tiene `requires-python = ">=3.8"` (ya compatible)
4. `actionlint` check
