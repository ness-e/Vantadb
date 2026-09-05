# MKT-18f — PyPI packaging for 5 adapters + release workflow

- **Plan:** docs/plans/2026-09-03-quality-gtm-wave.md (Task 8, Wave 2)
- **Ruta:** vanta-worker | **Fecha:** 2026-09-03 | **Estado:** ✅ COMPLETED (con 2 desviaciones documentadas)
- **Scope real ejecutado:** verificación packaging 5/5 + honestidad README + artefactos PR upstream. El release workflow y los pyproject **ya existían** (ver C2/C1 abajo) — no se duplicó.

## Estado de partida (hallazgo)

- `integrations/{langchain,llamaindex,mem0,crewai,dspy}/pyproject.toml` completos (hatchling, v0.5.0, pin `vantadb-py>=0.5.0,<0.6.0`).
- `.github/workflows/release-adapters-62.yml` YA existe: matriz 9 adapters (incluye los 5), `python -m build`, publish gated por tag `adapters-v*.*.*`, OIDC trusted publishing (`environment: pypi`), lane TestPyPI por dispatch.
- Nombres PyPI verificados live (2026-09-03): los 5 → HTTP 404 = LIBRES. `vantadb-py` → EXISTE (0.5.0) → el pin es válido.

## Checklist de PUBLICACIÓN (3 pasos — humano/lead con tokens, fuera del contrato de código)

1. **Habilitar publicación:** en el repo, crear el GitHub environment `pypi` (y opcionalmente `testpypi`) requerido por `release-adapters-62.yml`; en PyPI el primer publish desde Actions funciona vía *pending publisher* (OIDC) — o pre-crear los 5 proyectos con token si se prefiere `PYPI_API_TOKEN`.
2. **Dry-run TestPyPI:** `gh workflow run release-adapters-62.yml -f publish_testpypi=true` → verificar 5 dists en test.pypi.org/project/vantadb-langchain (×5).
3. **Tag release:** `git tag adapters-v0.5.0 && git push --tags` → job publish-pypi sube los 9; tras 200 OK, commit post-release: quitar el aviso "Not on PyPI yet" de los 5 READMEs y corregir claims de adapters en `docs/strategy/REDDIT_POSTS.md`.

## Verificación (contrato)

| Cláusula | Comando | Resultado |
|---|---|---|
| C1 build ×5 | `python -m build` en cada uno de los 5 dirs | ✅ exit 0, wheel+sdist (10 artefactos) |
| C1 twine ×5 | `python -m twine check dist/*` | ✅ PASSED 10/10 |
| C2 nombres 404 | `GET https://pypi.org/pypi/<n>/json` ×5 | ✅ 5/5 → 404 LIBRE |
| C3 workflow | `actionlint .github/workflows/release-adapters-62.yml` | ✅ exit 0 — DESVIACIÓN: no se creó `release-adapters.yml` nuevo porque 62 ya cumple la cláusula (tag-gate + matriz ⊇ 5 + build); duplicar = doble ruta de publish PyPI |
| C4 README honestos | sección "Install from PyPI (after first release)" ×5 | ✅ 5/5 |
| C5 PRs upstream | artefactos locales | ✅ `docs/plans/artifacts/mkt-18f-prs/*.md` ×5 |
| C6 NO tocar | `release-wheels-60.yml`, `release-plz.toml`, `docker*`, Formula, `docs/Backlog.md` (hasta cierre) | ✅ intactos |

## Desviaciones documentadas

1. **Workflow nuevo → existente (C3):** re-uso, no duplicado (ponytail rung 2). El `release-adapters-62.yml` usa OIDC (mejor que secret nombrado) y matrix de 9. Si se quisiera path-filter adicional sobre `integrations/**` para pre-build checks, es un `on.pull_request.paths` menor — no requerido por el contrato.
2. **Pre-mortem #2 (extras vs base deps) → NO aplicado:** `langchain`/`llamaindex`/`mem0` importan el framework top-level y `__init__.py` re-exporta → mover la dep pesada a extras rompe el install base (ImportError). Convención del repo (`vantadb-openai`: `openai>=1.0` en base) y del ecosistema (llama-index-vector-stores-*, langchain partners) mantienen la dep en base. Re-abrir solo con guards de import ×3 (tarea nueva).
