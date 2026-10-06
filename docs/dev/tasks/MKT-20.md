---
title: MKT-20 (ex MKT-18f) — PyPI packaging for adapters + release workflow (F6/Task 71)
kind: task
description: "F6/Task 71 — preparación owner-assisted del publish de los adapters en PyPI: D1/D2 resueltas e implementadas, dry-run local verde y checklist final ejecutable"
---

# MKT-20 (ex MKT-18f) — PyPI packaging for adapters + release workflow (F6/Task 71)

- **Plan:** [master-plan-0.9.0](../plans/2026-10-04-master-plan-0.9.0.md) (Task 71, F6 — cierre owner-assisted)
- **Ruta:** vanta-lead (F6, 2026-10-06) | **Estado:** ⏳ EN PROGRESO (re-abierta como F6/Task 71)
- **Plan original:** docs/dev/plans/2026-09-03-quality-gtm-wave.md (Task 8, Wave 2) — ✅ COMPLETED 2026-09-03 (2 desviaciones documentadas abajo)

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `docs/dev/tasks/MKT-20.md`, [release-adapters.yml](../../../.github/workflows/release-adapters.yml), [PUBLISH.md](../workflow/PUBLISH.md), [integrations/README.md](../../../integrations/README.md), `release-plz.toml`, [PROV-12.md](PROV-12.md), `integrations/{openai,ollama}/{pyproject.toml,README.md}` (twins), `providers/openai/README.md`, `integrations/test_pins.py`, `.github/workflows/adapters-compat.yml`.
- **Archivos referenciados hacia dentro:** `integrations/*/pyproject.toml` → deps `vantadb-py>=0.5.0,<0.7.0` + framework pins; workflow → `integrations/<adapter>/{tests,dist}`; PUBLISH.md §Adapters ↔ `release-adapters.yml` + tabla de namespaces.
- **Archivos que referencian a los editados:** [FRAMEWORKS.md](../../user/FRAMEWORKS.md) (tabla adapters + caveats), [Backlog.md](../Backlog.md) (fila FIND-273), `docs/dev/operations/TEST_MAP.md`, `integrations/README.md` ↔ pyprojects, master plan (Task 71 — **NO se toca**: prohibido en esta iteración).
- **Veredicto impacto:** BAJO — esta iteración refresca docs/task file (sin cambios de workflow ni pyprojects). Las decisiones D1 (FIND-273) y D2 (pins) cambian la matriz del workflow/pyprojects en la iteración siguiente; su diff está desglosado en §F6-2/§F6-3.

## Estado de partida (hallazgo — 2026-09-03, histórico)

- `integrations/{langchain,llamaindex,mem0,crewai,dspy}/pyproject.toml` completos (hatchling, v0.5.0, pin `vantadb-py>=0.5.0,<0.6.0` entonces).
- `.github/workflows/release-adapters-62.yml` YA existía: matriz 9 adapters (incluye los 5), `python -m build`, publish gated por tag `adapters-v*.*.*`, OIDC trusted publishing (`environment: pypi`), lane TestPyPI por dispatch.
- Nombres PyPI verificados live (2026-09-03): los 5 → HTTP 404 = LIBRES. `vantadb-py` → EXISTE (0.5.0).

## Checklist de PUBLICACIÓN (2026-09-03 — histórico, superado por el checklist F6 abajo)

1. **Habilitar publicación:** GitHub environment `pypi` (+ `testpypi`) + pending publisher (OIDC) por nombre.
2. **Dry-run TestPyPI:** `gh workflow run release-adapters-62.yml -f publish_testpypi=true` → 5 dists.
3. **Tag release:** `git tag adapters-v0.5.0 && git push --tags` → publish-pypi; post: quitar "Not on PyPI yet" de los READMEs.

## Verificación (contrato MKT-18f — histórico)

| Cláusula | Comando | Resultado |
|---|---|---|
| C1 build ×5 | `python -m build` en cada uno de los 5 dirs | ✅ exit 0, wheel+sdist (10 artefactos) |
| C1 twine ×5 | `python -m twine check dist/*` | ✅ PASSED 10/10 |
| C2 nombres 404 | `GET https://pypi.org/pypi/<n>/json` ×5 | ✅ 5/5 → 404 LIBRE |
| C3 workflow | `actionlint .github/workflows/release-adapters-62.yml` | ✅ exit 0 — DESVIACIÓN: se reusó el existente (no se duplicó) |
| C4 README honestos | sección "Install from PyPI (after first release)" ×5 | ✅ 5/5 |
| C5 PRs upstream | artefactos locales | ✅ `docs/dev/plans/artifacts/MKT-20 (ex MKT-18f)-prs/*.md` ×5 |
| C6 NO tocar | `release-wheels-60.yml`, `release-plz.toml`, `docker*`, Formula, Backlog | ✅ intactos |

## Desviaciones documentadas (2026-09-03)

1. **Workflow nuevo → existente (C3):** re-uso, no duplicado (ponytail rung 2). OIDC (mejor que secret) y matriz de 9.
2. **Pre-mortem #2 (extras vs base deps) → NO aplicado:** mover la dep pesada a extras rompe el import base (los `__init__.py` re-exportan). Convención del repo/ecosistema: dep en base.

## Re-baseline 2026-09-29 (LEAD) — histórico

- **Estado live (verificado 2026-09-29):** los 9 nombres en PyPI = **404** (libres); `vantadb-py` 0.7.0 live (200). Pins: `vantadb-py>=0.5.0,<0.7.0` ×9 (excluyen el core vigente → decisión de bump pendiente del owner).
- **Workflow:** `release-adapters.yml` (matriz 9, OIDC, `fail-fast:false`, `skip-existing`, tag manual `adapters-v*`). Stale refs corregidos en `integrations/README.md` + docstring de `test_pins.py`.

---

## F6 / Task 71 — Re-baseline 2026-10-06 (vanta-lead) — preparación owner-assisted

> **Contrato (plan):** los 7 adapters (twins retirados — D1) publicados en PyPI (`GET /pypi/<nombre>/json` = 200) **o** dry-run TestPyPI verde (5+ dists) + checklist final ejecutado por el owner (environment `pypi` + tag `adapters-v0.5.0`); post-publish: quitar los avisos "Not yet published" de los READMEs.
> **Lane:** el agente prepara/verifica; el **tag/publish es del owner** (push diferido al cierre del plan). ⛔ Sin push/tag/publish real en esta iteración.

### F6-1. Estado re-verificado live (2026-10-06)

| Check | Evidencia | Resultado |
|---|---|---|
| Nombres PyPI (10 únicos = 9 adapters + `vantadb-litellm`) | `GET pypi.org/pypi/<n>/json` ×10 | ✅ 404 = LIBRES (10/10) |
| Nombres TestPyPI | `GET test.pypi.org/pypi/<n>/json` ×10 | ✅ 404 (10/10) |
| `vantadb-py` live | idem | ✅ **0.8.0** (200) — pins adapters `<0.7.0` → drift (D2) |
| Workflow vs origin/main | `git diff origin/main -- .github/workflows/release-adapters.yml` | ✅ al día (solo difiere el pin de `dtolnay/rust-toolchain` — bump Dependabot local) |
| Workflow en origin/main | `git ls-tree origin/main .github/workflows/` | ✅ `release-adapters.yml` presente; `release-providers.yml` + `release-verify.yml` local-only (sin push — diferido al cierre) |
| Environments | `gh api repos/ness-e/Vantadb/environments` | ✅ `pypi` + `testpypi` **YA existen** (required reviewer `ness-e`) — el paso "crear environment" del checklist 09-29 está cumplido |
| Pyprojects ×9 | scan name/version/pin | ✅ 9 × `0.5.0` + `vantadb-py>=0.5.0,<0.7.0` (como esperado; pin con drift → D2) |
| `release-plz.toml` | hold `vanta-memory` (DIST-01) | ✅ no afecta adapters (no son workspace members; ningún hold los toca) |

### F6-2. FIND-273 — twins `integrations/{openai,ollama}` (✅ RESUELTA 2026-10-06 — owner: RETIRAR)

**Verificado (release-blocking):** los twins declaran PyPI `vantadb-openai`/`vantadb-ollama` **y** módulo `vantadb_openai`/`vantadb_ollama` — los MISMOS que los providers Rust canónicos (decisión owner 2026-10-04, PROV-12 Gate D). `release-adapters.yml` los incluye (matriz 9). Si publicaran primero: reclaman los nombres (first-come en PyPI) y el módulo colisiona (shadowing) al instalar ambos. Son duplicados funcionales **superados** por los providers (misma clase `VantaDBOpenAI`/`VantaDBOllama`; el provider Rust tiene API más rica: `embed/store/search/get/list/delete/list_namespaces`).

**Opciones (la elección es del owner — FIND-273: "a decidir con el owner"):**

- **A) RETIRAR del release (recomendado):** quitar `openai,ollama` de las 2 matrices de `release-adapters.yml` (9→7; el contrato "5+ dists" se cumple con 7). Dirs permanecen en repo (source-only) con nota "superseded by providers" en sus READMEs; `FRAMEWORKS.md` re-apunta las 2 filas a providers; PUBLISH.md §Adapters 9→7; `integrations/README.md` 9→7. `test_pins.py` y `adapters-compat.yml` **no** cambian (los dirs permanecen). Diff ≈ 6 archivos, sin renames, sin nombre nuevo que mantener.
- **B) RENOMBRAR:** p. ej. `vantadb-openai-adapter` + módulo `vantadb_openai_adapter` (sin shadowing) — requiere decidir nombre nuevo (owner), tocar pyprojects + imports + tests + READMEs + FRAMEWORKS.md; publica un paquete duplicado del provider bajo nombre nuevo (superficie extra). `vantadb_shared` (force-include de ambos twins) queda atado a esta opción.

**✅ DECISIÓN OWNER 2026-10-06: opción A — RETIRAR del release. Implementado (esta iteración):** matrices del workflow 9→7 (×2, con comentario FIND-273) · nota "superseded by providers" en `integrations/{openai,ollama}/README.md` · `FRAMEWORKS.md` (filas OpenAI/Ollama → `providers/`, "seven" + install/limits) · `integrations/README.md` (7+2, FIND-273 resuelto) · `PUBLISH.md` §Adapters (7 + checklist). `test_pins.py`/`adapters-compat.yml` sin cambios (dirs permanecen).

### F6-3. Pins `vantadb-py` (✅ RESUELTA 2026-10-06 — owner: bump `>=0.6.1,<0.9.0`)

Evidencia de compat (probe representativo: suite completa del adapter langchain — el de mayor superficie — contra `vantadb-py` de PyPI; venv limpio; local Windows):

| core | suite langchain | Nota |
|---|---|---|
| 0.5.0 | ❌ 44 errores — `AttributeError: module 'vantadb_py' has no attribute 'Client'` | el piso declarado `>=0.5.0` **no es real** (Client no existe en 0.5.0) |
| 0.6.1 | ✅ 46 passed, 1 skipped | versión que resuelve el pin actual |
| 0.7.0 | ✅ 46 passed, 1 skipped | excluida por el pin (drift) |
| 0.8.0 | ✅ 46 passed, 1 skipped | core live actual — excluida por el pin (drift) |

+ **install smoke con core 0.8.0** (two-step: wheel → upgrade): ✅ 6/6 import OK (5 adapters + crewai vía py3.11 — crewai 1.x exige `<3.14`) **+ langchain** cubierto por la suite completa (46 passed, §F6-3) = 7/7.

**✅ DECISIÓN OWNER 2026-10-06: bump a `vantadb-py>=0.6.1,<0.9.0`** (el techo alternativo `<0.10.0` quedó descartado: el alias deprecado `vantadb_py` — que todos los adapters importan — está marcado para remoción en `vantadb-python/vantadb_py/__init__.py:1-21`; `<0.9.0` queda a salvo y se re-evalúa tras el 0.9.0). **Implementado:** 7 pyprojects + docstring de `test_pins.py` + `integrations/README.md` + `FRAMEWORKS.md`. Post-bump: correr `adapters-compat.yml`. Contexto del pin viejo: pip instalaba 0.6.1 y subir a 0.8.0 requería el two-step (arriba); con el pin viejo, `pip install <wheel> vantadb-py==0.8.0` en un solo paso da `ResolutionImpossible`.

**Aplicación del bump (post-decisión):** `integrations/{langchain,llamaindex,mem0,crewai,dspy,haystack,letta[,openai,ollama]}/pyproject.toml` + `integrations/README.md` (texto del pin) + `FRAMEWORKS.md` (caveats) + `test_pins.py` si cambia la lista.

### F6-4. Dry-run local (equivalente ejecutable — precedente PROV-12)

Ejecutado 2026-10-06 (local Windows; py3.14 + py3.11 para crewai; lock `dev-tools/heavy-test-lock.ps1`):

- **Build ×7:** exit 0 (7/7) — wheel + sdist (**14 artefactos**).
- **Twine check ×7:** exit 0 — **14/14 PASSED**.
- **Install+import smoke ×7** (venv limpio, resolución del pin = 0.6.1): **7/7 SMOKE OK**.
- **Compat probe (§F6-3):** matriz 4 versiones + smoke ×7 con 0.8.0.
- **Pendiente (owner/push):** dispatch TestPyPI real — requiere push (cierre del plan) + pending publishers + decisiones D1/D2.

Comandos re-ejecutables:

```powershell
# build + metadata (por adapter)
cd integrations/<adapter>; python -m build
python -m twine check (Get-ChildItem dist -File).FullName
# install smoke (venv limpio)
python -m venv $v; $v/Scripts/pip install dist/<wheel>.whl
$v/Scripts/python -c "import vantadb_<adapter>; print('SMOKE OK')"
# dry-run TestPyPI (OWNER, post-push):
gh workflow run release-adapters.yml -f publish_testpypi=true
```

### F6-5. Checklist owner FINAL (2026-10-06 — D1/D2 resueltas e implementadas)

**Precondición única pendiente:**

- [ ] **Push** (cierre del plan — instrucción owner 2026-10-04): publicar el estado local (workflow 7 + pins + docs) vía PR `develop → main`. *(D1 retirar twins ✅ + D2 bump pins ✅ — resueltas e implementadas 2026-10-06.)*

**Ejecución (7 adapters):**

1. **Pending publishers** (PyPI **y** TestPyPI, Trusted Publishing/OIDC) para los 7 nombres — owner `ness-e`, repo `ness-e/Vantadb`, workflow `release-adapters.yml`, environment `pypi`/`testpypi` — o token. *(Environments: ✅ existen — verificado 2026-10-06.)*
2. **Dry-run TestPyPI:** `gh workflow run release-adapters.yml -f publish_testpypi=true` → **aprobar el deployment del environment `testpypi`** → verificar ≥5 dists en `test.pypi.org` (esperado: 14 — 7 wheels + 7 sdists).
3. **Smoke (opcional):** `pip install --index-url https://test.pypi.org/simple/ --extra-index-url https://pypi.org/simple/ vantadb-langchain==0.5.0` → import + operación básica.
4. **Tag:** `git tag adapters-v0.5.0 && git push --tags` → **aprobar el deployment del environment `pypi`** → job `publish-pypi` (OIDC) verde.
5. **Post-publish (commit de limpieza):** `GET https://pypi.org/pypi/<name>/json` = 200 ×7; quitar los avisos "Not on PyPI yet" de los READMEs ×7; actualizar `FRAMEWORKS.md` (404 → publicado); re-correr `node scripts/docs/gen-index.mjs --check` (cualquier edición .md cambia el índice).

**Stop condition:** publish bloqueado por OIDC/credenciales → checklist al owner y cerrar la preparación con nota (no forzar).

### F6-6. Evidencia de esta iteración (2026-10-06)

**Pre-decisión (DISCOVERY/prep):**
- Live: 404 ×10 en ambos índices; `vantadb-py` 0.8.0; environments OK; workflow al día (diff = 1 pin).
- Dry-run local: builds 7/7 · twine 14/14 · smoke 7/7 · probe 4 versiones (0.5.0 ✗ / 0.6.1 ✓ / 0.7.0 ✓ / 0.8.0 ✓) · smoke con 0.8.0 (two-step) 6/6 + langchain por suite completa.
- Hallazgos colaterales: ENOSPC ambiental (clase FIND-269) durante los probes — los tests de langchain generan ~6.5 GB de tmp por corrida (Fjall preasigna segmentos de 64 MB); re-corridas con disco liberado = verdes (misma clase que PROV-12). También: exe lock de `vanta-cli.exe` por procesos stale (vanta-cli + vantadb-server de junio) → limpiados; `verify.ps1` verde tras la limpieza.

**Post-decisión (D1/D2 implementadas):**
- Build ×7 (metadata nueva) = exit 0 · twine ×7 = 14/14 PASSED · smoke ×7 en venv limpio = **install=0, core resuelto 0.8.0 en un paso, import OK** (`target/mkt20/dryrun2-{builds,smoke}.log`).
- `python -m pytest integrations/test_pins.py` = 10 passed · `actionlint release-adapters.yml` = 0 · docs gates = 0/0/0 · `dev-tools/verify.ps1` = **ALL 10 PASS** (árbol final).
- Artefactos: `target/mkt20/dryrun2-builds.log` · `target/mkt20/dryrun2-smoke.log` (locales, regenerables).

## Review P2-01 (fresh — vanta-review, 2 rondas)

- **Ronda 1 (prep — `ses_eee23c8f6ffeF2fvvmPVeQIE5u`):** ✅ APPROVE — re-ejecutó 404 ×10 (ambos índices), environments (`gh api`), diff del workflow, probe de pin en venv (0.5.0 `Client: False` / 0.8.0 `True`), twine spot, gates. Findings Medium/Low → todos aplicados en la ronda 2.
- **Ronda 2 (implementación D1/D2 — `ses_eedfe4bf1ffe9AjydrN0g57x9f`):** ⚠️ changes-required → fix R1 (frase "shim" contradictoria en `integrations/README.md:59`) + optinals (contrato §F6 → 7; description + índice; `workflow/README.md` + `RUNBOOK.md`; runbook `release-adapters-62.md` ×4) → **ronda 2.1: ✅ APPROVE** (fixes re-verificados; wheel metadata `Requires-Dist: vantadb-py<0.9.0,>=0.6.1`; `test_pins.py` 10/10; gates 0/0/0).
- **Verificación mecánica final:** `dev-tools/verify.ps1` = **ALL 10 PASS** · build ×7 + twine 14/14 + smoke ×7 (core 0.8.0 en un paso) · `check-links`/`check-docs`/`gen-index --check` = 0 · `actionlint` = 0.
- **OCR delegation:** 0 reviewables propios (`.md`/`.yml`/`.toml`/`.py` → unsupported_ext; el único reviewable del preview — `dev-tools/heavy-test-lock.ps1` — es ajeno y no se commitea).
- **Residuales (fuera del changeset):** `docs/user/COMPARISON.md:209` "Nine Python adapters" (literalmente cierto — 9 dirs; cosmético eventual); fila Backlog `FIND-273` → remover vía `progreso` (Trigger 1) al cierre del orquestador.

## Context Save Point

- **Step:** cierre — review P2-01 ✅ (ronda 2.1 approve), gates verdes, commit local en curso.
- **Worktree:** `develop` (commits locales; ⛔ sin push).
- **Próxima acción:** commit local `ci(release):` → backfill de hash → campaign close (taskId 71).

## RESULTADO (§7)

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 6/6
PROXIMO_STEP: ninguno (owner: push + pending publishers + dry-run dispatch + tag `adapters-v0.5.0` — checklist §F6-5)
COMMIT_HASH: <pendiente — backfill>
ARCHIVOS: .github/workflows/release-adapters.yml · integrations/{langchain,llamaindex,mem0,crewai,dspy,haystack,letta}/pyproject.toml · integrations/test_pins.py · integrations/{openai,ollama}/README.md · integrations/README.md · docs/user/FRAMEWORKS.md · docs/dev/workflow/{PUBLISH,README,RUNBOOK,release-adapters-62}.md · docs/dev/tasks/MKT-20.md · docs/index.md · llms.txt
VERIFY_CONTRATO: pasa
BLOQUEO: ninguno
GATES_EVALUADOS: P:no(plan intacto) D:si(D1/D2 → owner, resueltas) V:no(sin stalls) C:si(colaterales ambient documentados)
SKILLS_CARGADAS: ci-cd-and-automation · git-workflow-and-versioning · shipping-and-launch · documentation-skill (+ base auto: campaign-executor · progreso)
```
