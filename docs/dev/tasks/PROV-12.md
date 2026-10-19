---
title: "TASK PROV-12: Publicar wheels PyPI de providers (estrategia H-04)"
kind: task
description: "pyproject.toml + release-providers.yml (maturin; matriz 3 providers × 4 plataformas) + build/validación local + checklist owner para el publish real. Gate D: colisión de nombres PyPI resuelta por owner (providers canónicos)."
---

# TASK PROV-12: Publicar wheels PyPI de providers (estrategia H-04)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 30, F1)
- **Fuente:** Backlog P45 PROV-12 (INV-providers-01 H-04, estrategia aprobada: publicar); deps PROV-01/02/04 ✅
- **Esfuerzo:** 🟡 1sem | **Appetite:** max 1sem | **Prioridad:** 🟠
- **Tipo:** Release/Packaging (CI/CD + packaging; sin cambios de contrato de los providers)
- **Creado:** 2026-10-04 | **Estado:** ⏳ EN PROGRESO
- **Campaign ID:** master-plan-0.9.0-20261004 · **taskId server:** 30
- **Ejecutor:** vanta-lead (dominio release/packaging)
- **Incógnitas (uphill):** 0 abiertas (resueltas en DISCOVERY — ver Investigation Notes)
- **Pendientes (downhill):** 6 steps (2-7)
- **Nota de archivo:** este path contenía el task file de la campaña ARCHIVADA `2026-09-19-publicacion` (PROV-12 core-wheels dry-run, plan en `plans/archive/`, registro en `docs/dev/avance/meta.md:328`). El master plan 0.9.0 (Task 30) designa este path para el nuevo PROV-12 (providers) → el contenido previo quedó en git (`b764d703`..HEAD) y se registró `FIND-274` (higiene de IDs reusados entre campañas).

## Gate D — decisión owner (colisión de nombres PyPI)

**Descubrimiento (Paso 0 no lo vio):** `integrations/{openai,ollama}` (adapters Python, MKT-20/Task 71 F6) declaran en su `pyproject.toml` los nombres PyPI `vantadb-openai`/`vantadb-ollama` **y el mismo módulo** (`vantadb_openai`) que los providers Rust de esta tarea (H-04 + ADR-0033 + research: `pip install vantadb-openai` → `VantaDBOpenAI(path, key)`). Ambos 404 en PyPI/TestPyPI hoy → solo uno puede reclamarlos. Además el `vantadb-litellm` del trío GTM solo existe como provider.

**Question (2026-10-04) + respuesta owner:** **"Providers canónicos"** → los providers conservan `vantadb-openai`/`vantadb-ollama`/`vantadb-litellm`; los twins `integrations/openai`+`integrations/ollama` se renombran o retiran antes del publish de F6/MKT-20.

**Materialización:** fila `FIND-273` (Backlog) + ítem #0 del Checklist owner + nota en `PUBLISH.md` §Providers.

## Blast Radius

**Archivos exactos del cambio (nuevos/modificados):** `providers/openai/pyproject.toml` · `providers/ollama/pyproject.toml` · `providers/litellm/pyproject.toml` · `providers/openai/Cargo.toml` · `providers/ollama/Cargo.toml` · `providers/litellm/Cargo.toml` · `.github/workflows/release-providers.yml` · `.github/scripts/provider_wheel_smoke.py` · `docs/dev/operations/CI_POLICY.md` · `docs/dev/workflow/PUBLISH.md` · `docs/dev/workflow/TRIGGERS.md` · `docs/dev/workflow/README.md` · `docs/dev/Backlog.md` · `docs/dev/tasks/PROV-12.md` · `providers/openai/README.md` · `providers/ollama/README.md` · `providers/litellm/README.md`

| Dirección | Módulos |
|-----------|---------|
| Callers | `providers-ci.yml` (build/test ubuntu), `CI_POLICY` (circuit breaker experimental), `PUBLISH.md` §crates (canal), READMEs de providers; nadie los importa desde el workspace (standalone, `[workspace]` vacío) |
| Callees | `providers/{openai,ollama,litellm}/` (Cargo.toml + src + tests + .pyi + README), `vantadb` (path dep compilado dentro del wheel), SDKs Python (`openai`/`ollama`/`litellm` — duck-typing por reflexión), patrón: `release-wheels.yml` + `release-adapters.yml` |
| Implicaciones | Aditivo: 3 `pyproject.toml` nuevos + 1 workflow nuevo + 1 script smoke + docs. **Sin cambios de contrato Python** (módulos/clases/firmas intactos). El wheel embebe el core (path dep) → sin pin de `vantadb-py`. `publish = false` del crate se mantiene (no crates.io). Windows: `cargo check` ×3 ✅ verificado 2026-10-04 (premisa de PROV-13 stale — ver Notes). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `providers/{openai,ollama,litellm}/Cargo.toml` (23L c/u), `providers/openai/src/python.rs` (parcial 1-100), tests (`test_openai.py` 1-119; importorskips ×3), READMEs (openai/litellm completos), `.github/workflows/release-wheels.yml` (316L), `.github/workflows/release-adapters.yml` (161L), `.github/workflows/providers-ci.yml` (92L), `vantadb-python/pyproject.toml` (70L) + `Cargo.toml` (33L, `abi3-py311`), `docs/dev/operations/CI_POLICY.md:142-171`, `docs/dev/workflow/PUBLISH.md` (1-190), `integrations/{openai,ollama}/pyproject.toml` + `integrations/openai/vantadb_openai/{__init__,vectorstore}.py` (colisión), `docs/dev/reviews/archive/research-providers-20260825.md` (H-04/H-14), `ADR-0033` (276L), `docs/user/FRAMEWORKS.md`, `docs/dev/tasks/MKT-20.md`, `docs/dev/tasks/DIST-06.md`, plan Tasks 30/31/71.
- **Archivos referenciados hacia dentro (imports/deps):** los 3 crates dependen de `vantadb` (path `../..`, features `fjall,memmap2`) + `pyo3 0.29` opcional; el wheel compilado los embebe (self-contained). `providers/shared_py.rs` compartido vía `#[path]`.
- **Archivos que referencian a los editados (referencias entrantes):** `providers-ci.yml` (matrix de nombres — se mantiene), `CI_POLICY` (tabla experimental), `PUBLISH.md` (tabla §crates + namespace), Backlog P45 (fila PROV-12). Ningún consumidor parsea los pyproject (nuevos).
- **Veredicto impacto:** **BAJO-MEDIO** — aditivo; el único punto sensible es la **identidad PyPI** (resuelta por Gate D) y la estabilidad del build multiplataforma (verificada local en Windows; el resto de plataformas se valida en el primer run del workflow).

## Contrato

> Del plan (Task 30) — "wheels publicados **o** dry-run TestPyPI verde (5+ dists) + checklist owner para el publish real". El publish real es del owner (patrón MKT-20/Task 71; push diferido al final del plan).

1. **Distribución preparada y verificada**: 3 `pyproject.toml` maturin (nombres canónicos) + `release-providers.yml` (matriz 3×4: linux-x86_64/macos/windows/linux-aarch64; OIDC; input TestPyPI; skip-existing; tag `providers-v*.*.*`) — ⬜ Steps 2-4
2. **Dry-run local verde**: build de wheels ×3 + `twine check` + smoke en venv limpio por provider (`pip install <wheel>` → import → init → store/search con vector fake, sin red) — **equivalente ejecutable del dry-run** (el dispatch real a TestPyPI requiere push + environments del owner) — ⬜ Step 6
3. **Checklist owner** para el publish real (TestPyPI dispatch → smoke → tag → verificación post) — ⬜ Step 5
4. **CI multiplataforma**: el workflow construye los 4 targets (12 wheels) en PR/tag/dispatch; `actionlint` 0 — ⬜ Steps 4, 6
5. **CI_POLICY actualizado** (canal de release de providers) + `PUBLISH.md` (namespace + §crates + §Providers) — ⬜ Step 5
6. **Sin cambios de contrato de los providers** (PROV-04 cerrado): módulos, clases y firmas Python intactos; `publish = false` del crate se mantiene — invariante verificado en cierre — ⬜ Step 6
7. **Colisión de nombres**: `FIND-273` registrado (twins de integrations a renombrar/retirar en F6) — ⬜ Step 5

## Spec (decisiones — SDD)

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Nombres PyPI | A) **canónicos `vantadb-{openai,ollama,litellm}`** (H-04/ADR-0033/research; trío GTM = familia providers) / B) nombres nuevos para providers (rompe contrato + research) | ✅ **Gate D owner 2026-10-04: A** (FIND-273 para twins) |
| 2 | Build backend | A) **maturin `>=1.15.0,<2.0`** (convención del repo: `vantadb-python/pyproject.toml:2`; providers ya construyen con maturin en providers-ci) / B) setuptools-rust (fuera de convención) | ✅ A (evidencia: repo + CI real) |
| 3 | Versión | A) **`dynamic = ["version"]`** (single source: Cargo.toml 0.5.0; convención `vantadb-python` + lección "PyPI 0.6.0 incident") / B) versión estática (drift) | ✅ A |
| 4 | Deps del wheel | A) **declarar SDK** (`openai>=1.0,<2` / `ollama>=0.4,<1` / `litellm>=1.0,<2`) — cierra H-10 "requisito pip oculto"; convención integrations / B) no declarar (el usuario descubre el ImportError) | ✅ A (evidencia: research H-10 + convención repo) |
| 5 | abi3 | A) **`abi3-py311`** (1 wheel por plataforma para ≥3.11; convención core `vantadb-python/Cargo.toml:15`) — verificación: compila + tests pasan / B) cp311-only (wheels inútiles en 3.12/3.13) | ✅ A condicionado a verificación (fallback B + FIND) |
| 6 | sdist | A) **no sdist** (path dep `vantadb = ../..` no resoluble por consumidores externos; wheels prebuilt self-contained) / B) sdist (roto para el consumidor) | ✅ A (evidencia: `providers/openai/Cargo.toml:19`) |
| 7 | Matriz de plataformas | A) **4 targets** = patrón release-wheels (ubuntu x86_64 + macos + windows + linux aarch64 cross manylinux_2_28) / B) menos (incumple contrato) | ✅ A |
| 8 | Tag namespace | A) **`providers-v*.*.*`** (propio; no dispara `v*`/`node-v*`/`adapters-v*`; patrón tabla `PUBLISH.md:107-115`) / B) reusar `adapters-v*` (dispara los 9 adapters) | ✅ A |
| 9 | Smoke | A) **script compartido `.github/scripts/provider_wheel_smoke.py`** (usado por build-smoke ×12, verify-testpypi y verify-pypi; 1 fuente) / B) inline ×3 shells (duplicación) | ✅ A |
| 10 | Dónde vive el checklist owner | A) **`PUBLISH.md` §Providers (durable) + task file** / B) solo task file (se archiva) | ✅ A |
| 11 | READMEs | A) **sección "Install from PyPI (after first release)"** con caveat (patrón MKT-20 C4) / B) dejarlos (quedan stale al publicar) | ✅ A (mínimo) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) **sin push, sin tags, sin publish real** (push diferido al final del plan — instrucción owner; publish = owner); (2) contrato Python de los providers intacto (módulos `vantadb_openai|ollama|litellm`, clases `VantaDB*`, firmas `#[pyo3(signature)]` — PROV-04); (3) `publish = false` del crate se mantiene (no crates.io); (4) WIP ajeno intacto (`opencode.jsonc`, master plan, `docs/pipeline-state.json`, `docs/dev/tasks/TS-*`, `..\web`, cambios en vuelo de TS-11/TS-13); (5) Cargo.lock de providers sin churn (lección PROV-openai: revertir, no commitear); (6) los workflows existentes (`providers-ci.yml`, `release-wheels.yml`, `release-adapters.yml`) NO se modifican.
- **Comandos de verificación:** ver Contrato (2/4) + `campaign_verify_cmd`.
- **Deuda pendiente:** ninguna nueva. Notas: PROV-13 (Windows CI en providers-ci) queda como está (premisa stale — check ✅ local); rename/retiro de twins = FIND-273 (F6).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato 1-7 ✅ (pyprojects + workflow + dry-run local verde + checklist + docs + FIND + invariante de contrato). |
| **Commit** | Commit local `ci(providers):` (conventional; SIN push), verificación mecánica (nunca auto-reporte), sin archivos fuera del blast radius. |
| **Release** | Wheels en PyPI — ejecutado por el owner vía checklist (dispatch TestPyPI → tag `providers-v0.5.0`). El cierre del task NO publica. |

## Steps

1. ✅ **Task file + Spec + Gate D** (este archivo; decisión owner registrada).
2. ✅ **pyproject.toml ×3** (maturin, nombres canónicos, dynamic version, deps SDK, module-name explícito).
3. ✅ **abi3-py311 ×3** (`pyo3` features) — verificado: `cargo check` ×3 + `maturin build --release` ×3 → wheels `cp311-abi3` ✅ + suites de providers contra los wheels (18/17/19 passed).
4. ✅ **`release-providers.yml`** (matriz 3×4 + smoke + publish-testpypi + publish-pypi + verify jobs) + `.github/scripts/provider_wheel_smoke.py` — `actionlint` 0.
5. ✅ **Docs**: `CI_POLICY` (canal), `PUBLISH.md` (namespace + §crates + §Providers + checklist), `TRIGGERS.md`/`README.md` (inventario 40), READMEs ×3 (Install PyPI), `FIND-273` (Backlog).
6. ✅ **Verificación local**: `maturin build` ×3 + `twine check` ×3 PASSED + smoke venv limpio ×3 (SMOKE OK) + pytest ×3 (18/17/19) + `actionlint` 0 + gates docs (check-links 0 · check-docs 0 · gen-index 0) + `dev-tools/verify_changed.ps1` ALL 3 PASS + `dev-tools/verify.ps1` **ALL 10 PASS**.
7. ⬜ **Cierre**: OCR delegation + review P2-01 (vanta-review) + commit local + campaign close (taskId 30) + progreso.

## Evidencia de verificación (2026-10-04, local Windows)

| Gate | Comando | Resultado |
|------|---------|-----------|
| Compila (abi3) | `cargo check --manifest-path providers/{openai,ollama,litellm}/Cargo.toml` | ✅ ×3 |
| Build wheels | `maturin build --release --manifest-path ... --out dist` | ✅ `vantadb_{openai,ollama,litellm}-0.5.0-cp311-abi3-win_amd64.whl` |
| Metadata | `python -m twine check dist/vantadb_{openai,ollama,litellm}-*.whl` | ✅ PASSED ×3 |
| Smoke venv limpio | `.github/scripts/provider_wheel_smoke.py <provider>` (venv 3.11, `pip install <wheel>`) | ✅ SMOKE OK ×3 (import+init+store+search+list, sin red) |
| Suites providers | `pytest providers/<p>/tests/` contra el wheel instalado | ✅ openai 18 · ollama 17 (requiere `vantadb-py`, como CI) · litellm 19 |
| Workflow | `actionlint .github/workflows/release-providers.yml` (+ dir completo) | ✅ 0 |
| Verify rápido | `dev-tools/verify_changed.ps1` | ✅ ALL 3 PASS (fmt/check/clippy) |
| Verify full | `dev-tools/verify.ps1` | ✅ **ALL 10 PASS** (fmt/check/clippy/audit/deny/nextest 2518/docs-coverage/cli-probes/consumo/backup) |
| Docs gates | `check-links` · `check-docs` · `gen-index --check` | ✅ 0 · 0 · 0 (índice regenerado; no staged por entradas ajenas en vuelo) |

Nota ENOSPC (ambiental, clase FIND-269): la primera corrida de `verify.ps1` falló 2/2518 tests con `Os code 112 StorageFull` por presión de disco; ambos pasan en aislamiento y la re-corrida completa con disco liberado dio ALL 10 PASS.

## Checklist owner (publish real — ejecuta el owner)

> Referencia durable: `PUBLISH.md` §Providers. Precondición: **push** (al final del plan) + environments.

0. **Decisión de nombres** — ✅ RESUELTA 2026-10-04: providers canónicos; coordinar `FIND-273` (twins de integrations) antes del publish de F6.
1. **Habilitar publicación**: GitHub environments `pypi` + `testpypi` (si no existen); PyPI/TestPyPI → *pending publisher* (Trusted Publishing/OIDC) para `vantadb-openai`, `vantadb-ollama`, `vantadb-litellm` (o token).
2. **Dry-run TestPyPI**: `gh workflow run release-providers.yml -f publish_testpypi=true` → 12 wheels (3 proyectos × 4 plataformas) en test.pypi.org + job `verify-testpypi-install` verde.
3. **Smoke TestPyPI (manual, opcional)**: `pip install --index-url https://test.pypi.org/simple/ --extra-index-url https://pypi.org/simple/ vantadb-openai==0.5.0` → import + init + store/search.
4. **Tag release**: `git tag providers-v0.5.0 && git push --tags` → `publish-pypi` (OIDC) + `verify-pypi-install` verde.
5. **Post-publish**: `GET https://pypi.org/pypi/<nombre>/json` = 200 ×3; quitar el caveat "after first release" de los READMEs (commit de limpieza); opcional: `pypistats`.

## Investigation Notes (DISCOVERY — 2026-10-04)

1. **Estado real**: providers sin `pyproject.toml` (`git ls-files` ✅), PyPI 404 ×3 (live: pypi.org + test.pypi.org), crate 0.5.0, `publish=false`, no workspace members (`CI_POLICY:151-153`).
2. **Windows**: `cargo check --manifest-path providers/{openai,ollama,litellm}/Cargo.toml` → **✅ ×3 en Windows local (2026-10-04)**; la premisa de PROV-13 ("no compilan en Windows") no reproduce en HEAD → anotado para Task 31 (verificar antes de ejecutarla).
3. **Colisión de nombres** (Gate D): integrations twins 2026-07-10; providers (restructure "Python for frameworks, Rust for providers") 2026-07-22; ambos reclaman los nombres; owner → providers canónicos (FIND-273).
4. **Patrón release-wheels** (316L): matriz 4 targets, maturin-action, smoke por plataforma, TestPyPI dispatch, publish por tag `v*`, verify installs, attestations. **Patrón release-adapters** (161L): matriz 9, OIDC, `skip-existing`, tag propio, upload/download-artifact `merge-multiple`. PROV-12 combina ambos (maturin del primero; namespace/OIDC/skip del segundo).
5. **Convención pyproject** (`vantadb-python`): maturin `>=1.15.0,<2.0`, `dynamic = ["version"]`, `[tool.maturin] module-name`, license `{text}`, classifiers "solo lo que CI verifica" (FIND-85).
6. **Signatures** (smoke): openai `(db_path, api_key, ...)`; ollama `(db_path, base_url=..., ...)`; litellm `(db_path, api_key=None, ...)`; namespaces default `openai_store`/`ollama_store`/`litellm_store`; clase `VantaDB{OpenAI,Ollama,LiteLLM}`.
7. **Lecciones aplicables**: PROV-openai (Cargo.lock churn → revertir, no commitear); PyO3 (no usar python 3.14 system para providers; `.venv` 3.11.9 del repo); MKT-20 (reuso > duplicado; dry-run real requiere owner; checklist 3 pasos).
8. **Tooling local verificado**: maturin 1.15.0, actionlint 1.7.12, twine 7.0.0, `.venv` Python 3.11.9.

## Review P2-01 (fresh — vanta-review)

- **Revisor:** vanta-review (P2-01, contexto fresco — no participó de la implementación; sesión `ses_ef5dd5fa7ffejGRYYZxxlz3hA7`)
- **Enfoque:** contrato 7/7 + red-team del lane de release (tag↔versión, permisos, pins, OIDC, skip-existing, failsafe) + re-ejecución mecánica en contexto fresco
- **Cómo se probó:** actionlint exit 0 · twine check ×3 PASSED · smoke ×3 en venv limpio (SMOKE OK) · maturin build con los args exactos del workflow (wheel abi3 + metadata) · gates docs 0/0/0 · wheel contents (.pyi/py.typed/`__init__.py`) y firmas pyo3 verificadas
- **Veredicto:** ✅ **APPROVE** — contrato 7/7; 0 Critical/High.
  - **M1 (Medium, tag↔versión):** RESUELTO post-review — ambos `Extract provider version` ahora derivan del tag cuando `GITHUB_REF` es `refs/tags/providers-v*` (espejo de `release-wheels.yml:226-234`); actionlint re-corrido ✅.
  - **L1 (sweep de commit vecino `d6d40461`):** `docs/index.md`/`llms.txt`/FIND-273 ya commiteados por el vecino; `Backlog.md` quedó con solo FIND-274 → se stagea en este commit.
  - **L2 (Blast Radius):** completado con `TRIGGERS.md` + `README.md`.
  - **L3 (Cargo.lock stale) / O1 (verify 3.11) / N1 (retry loop):** documentados, sin acción (L3: revert mantenido; O1/N1: alineados con los lanes del core).

## Context Save Point

- **Step actual:** 7 (cierre) — review P2-01 en curso (vanta-review, fork contexto fresco); commit pendiente del veredicto.
- **Worktree:** `develop` (commits locales; sin push)
- **Próxima acción:** recoger veredicto → RESULTADO §7 → commit local `ci(providers):` (solo archivos del blast radius) → campaign close (taskId 30) → progreso.

## RESULTADO (§7)

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 7/7
PROXIMO_STEP: ninguno
COMMIT_HASH: 04be0ec1 (changeset) + commit de cierre docs (task file)
ARCHIVOS: providers/{openai,ollama,litellm}/{pyproject.toml,Cargo.toml,README.md} · .github/workflows/release-providers.yml · .github/scripts/provider_wheel_smoke.py · docs/dev/operations/CI_POLICY.md · docs/dev/workflow/{PUBLISH,TRIGGERS,README}.md · docs/dev/Backlog.md (FIND-273/274) · docs/dev/tasks/PROV-12.md
VERIFY_CONTRATO: pasa
BLOQUEO: ninguno
GATES_EVALUADOS: P:no(familia del plan aprobada) D:si(colisión nombres → owner, resuelta) V:no C:si(sweep ajeno L1 documentado)
SKILLS_CARGADAS: ci-cd-and-automation · git-workflow-and-versioning · shipping-and-launch · source-driven-development · doubt-driven-development · documentation-skill (+ base auto: campaign-executor · progreso)
```
