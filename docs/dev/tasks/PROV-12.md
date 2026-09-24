# PROV-12 — publicar wheels PyPI vía CI (lista para publicar, sin publish local)

> **Plan:** `docs/dev/plans/2026-09-19-publicacion.md` · **Wave:** Wave2 primera en secuencia · **Ruta:** vanta-lead
> **Branch:** develop · **Commit:** `ci: PROV-12 — ...` · **Appetite:** 2d · **Esfuerzo:** 🟡 · **Prioridad:** 🔴 Alta
> **Estado:** ⏳ IN PROGRESS (DISCOVERY completo, Gate D GO)
> **SDP:** `campaign_discover_skills_v2` phase=BUILD keywords=[pypi, maturin, wheels, release, CI, secrets, testpypi] → base+keyword skills abajo

## TAREA

**Objetivo:** dejar el release de wheels PyPI listo y verificado en seco, SIN publish local (el token vive en GitHub Secrets / OIDC, inaccesible desde esta máquina).

**Contrato:**
1. Workflow release de wheels listo y verificado en seco (TestPyPI primero).
2. `pip install` en entorno limpio + smoke `Client` verde (con wheel construida localmente — el artefacto es el mismo que CI publica).
3. Docs con versión == código.
4. Si falta el secret / prereq → STOP con instrucción exacta al owner (no inventar, no publicar a ciegas).
5. La task cierra como **"lista para publicar"**, NO como "publicado".

**AC (acceptance criteria):**
- [ ] Prereqs PROV-01/02/04 verificados (o veredicto documentado de N/A con evidencia).
- [ ] `release-wheels-60.yml` leído entero; `actionlint` verde (o fix mínimo si está roto).
- [ ] `maturin build --release` OK sin publish + metadata del wheel verificada (nombre `vantadb-py`, versión == workspace).
- [ ] `pip install <wheel>` en venv limpio + `verify_published_wheel.py` verde (put/get/list/search/capabilities/durabilidad).
- [ ] Coherencia de versión: `[workspace.package] version` == `pyproject.toml version` == `__version__` del wheel instalado.
- [ ] Docs con versión == código (sin hardcodes divergentes en el scope tocado).
- [ ] STOP honesto: publish real (TestPyPI/PyPI) requiere al owner — instrucción exacta entregada, nada publicado desde esta máquina, ningún secret a disco/código/logs.
- [ ] Commit `ci: PROV-12 — ...` con staging SELECTIVO (solo paths propios), NO PUSH.

**Estrategia H-04 aprobada:** pyproject.toml + maturin, CI release multiplataforma.
**Gate V (resuelto por owner 2026-09-19 — NO re-preguntar):** (1) token en GitHub Secrets — SIN publish local; (2) TestPyPI primero; (3) versión derivada del tag (release-plz, sin edición manual).
**Pre-mortem:** (1) sin token visible no hay publish → vía CI; si el secret falta → STOP con instrucción exacta; (2) matriz multiplataforma rompe en un OS → ship con la matriz que pase + nota.

## ARCHIVOS

**Clave (leídos completos antes de actuar):**
- `vantadb-python/pyproject.toml:7` (version 0.5.0), `:40-57` ([tool.maturin], module-name `vantadb_py`, include/exclude H-05), `:63-67` (pytest slow-marks FX-3)
- `vantadb-python/Cargo.toml:3` (version.workspace), `:8` (publish=false), `:11` (lib name `vantadb_native`), `:15` (abi3-py311), `:24` (core dep path)
- `.github/workflows/release-wheels-60.yml:3-19` (triggers: dispatch+PR+tag), `:41-57` (matriz 4 targets), `:94-95` (version_coherence gate), `:97-106` (maturin build), `:107-129` (smoke unix/windows), `:139-163` (publish-testpypi dispatch+`TEST_PYPI_API_TOKEN`/OIDC env testpypi), `:164-202` (publish-pypi en tag + attestations + attach release), `:204-310` (verify TestPyPI/PyPI install + provenance)
- `docs/user/operations/CI_POLICY.md:424-431` (§9 workflow wheels: build+smoke, TestPyPI manual, PyPI diferido)
- `vantadb-python/verify_published_wheel.py:1-80` (smoke post-publish: version+put/get/list/search/caps/durabilidad)
- `Cargo.toml:727-730` ([workspace.package] version 0.5.0, edition 2021, rust 1.94.1)

**Relacionados (lectura según necesidad):**
- `Cargo.toml` workspace version (coherencia), `docs/CHANGELOG.md` (release-plz lo maneja — NO editar)
- `vantadb-python/tests/test_sdk.py` (smoke que CI corre contra el wheel)
- `.opencode/rules/release-ci.md` (Regla 4 version sync, Regla 5 continue-on-error+CATEGORY), `.opencode/rules/python-bindings.md`

**Prohibidos (WIP ajeno / datos vivos — NO tocar):**
`reparacion.bat`, `.opencode/`, `Justfile`, `ocr-*`, `completions/*`, `desktop/src-tauri/Cargo.lock`, stash@{0} GOV-C4, `docs/dev/Backlog.md`, plan file (solo recitation al cierre — ver nota CIERRE), `C:/Users/Eros/.vantadb*` (datos vivos), `src/` (cero cambios core), `web/`, `examples/` (SHOW-02/03 cerrados). Ningún token a disco/código/logs. Ningún publish a PyPI/TestPyPI desde esta máquina.

## DEPENDENCIAS

- **Wave:** Wave2 primera en secuencia (Wave0 ✅ FIND-98-retry + SHOW-02, Wave1 ✅ SHOW-03 + DIST-10/14).
- **Bloqueantes:** Gate V RESUELTO por owner 2026-09-19 (§ arriba). Prereqs nominales PROV-01/02/04 → veredicto en INVESTIGACIÓN (paso 1).
- **Si prereqs faltan en sustancia → STOP + re-triage** (no forzar, no ejecutar a ciegas).
- **NextTask tras cierre:** EXE-03-prep (la ejecuta el orquestador, no yo). **P2-01:** lo hace el orquestador (no yo).

## REFERENCIAS

- **Rules (lectura completa antes de actuar — HECHA):** `.opencode/rules/release-ci.md` (42L: allocators prod, sccache único, Dockerfile MSRV, version sync §4, continue-on-error+CATEGORY §5).
- **Refs:** `.opencode/references/definition-of-done.md` (DoD standing + DoD VantaDB comandos + shippable trunk-based), `docs/user/operations/CI_POLICY.md` §9, `dev-tools.md` (vía AGENTS).
- **Commands:** `pipeline.md`, `audit.md`. **SPEC:** `SPEC.md` raíz. **Tabla Spec:** N/A (release/packaging, 0 greenfield).

## SKILLS (SDP Paso 0b — ejecutado, scores en tool result)

- `ci-cd-and-automation` (sugerida plan — CARGADA: quality gates, staged rollout TestPyPI→PyPI, rollback plan, secrets en manager) — aplica a todo el workflow release.
- `git-workflow-and-versioning` (sugerida plan — CARGADA: semver desde tag, atomic commits, pre-commit hygiene sin secrets) — aplica a versionado + commit `ci:`.
- `security-and-hardening` (sugerida plan — CARGADA: threat-model secrets, never-commit-secrets, trusted publishing OIDC) — aplica a manejo de tokens/OIDC.
- `shipping-and-launch` (SDP keyword — CARGADA: pre-launch checklist, staged rollout, rollback strategy) — aplica a estrategia TestPyPI-primero + rollback plan.
- `source-driven-development` (SDP base) — NO cargada como skill (sin ambigüedad de APIs externas que lo exija; docs PyPI solo si gap — N/A).
- `incremental-implementation` / `test-driven-development` / `context-engineering` / `doubt-driven-development` / `frontend-ui-engineering` / `api-and-interface-design` (SDP lifecycle genéricos) — NO aplican (0 código nuevo, 0 UI, 0 API nueva); se declaran para trazabilidad.
- `SKILLS_CARGADAS:` ci-cd-and-automation, git-workflow-and-versioning, security-and-hardening, shipping-and-launch (+ SDP scoring registrado arriba).

## HERRAMIENTAS+MCP

- `maturin build --release --out dist --manifest-path ./vantadb-python/Cargo.toml` (SIN publish) + inspección metadata wheel (`vantadb_py-*.whl`, `METADATA`, tags).
- `actionlint .github/workflows/release-wheels-60.yml` (disponible vía winget) si se toca el workflow; si no se toca → solo lectura + actionlint de verificación.
- `cargo test --test version_coherence` (gate que CI corre; local solo si el costo compila lo justifica — ver step 3).
- `python -m venv` limpio + `pip install <wheel>` + `python vantadb-python/verify_published_wheel.py` con `VANTADB_EXPECTED_VERSION`.
- `campaign_verify_cmd` para verify mecánico (BUG exit -1 conocido → bash directa + mención en RESULTADO).
- Cargo con `-j 2` si compila (OOM Windows). Internet SOLO para docs oficiales PyPI/TestPyPI si gap (URLs verificadas; sin red → `[cita NO VERIFICADA]` + deuda TSYS-13).

## INVESTIGACIÓN CÓDIGO (blast radius — generado en DISCOVERY)

- **Workflow `release-wheels-60.yml` (310L, leído entero):** completo y coherente — build 4 targets (linux-x86_64, macos, windows, linux-aarch64 cross), smoke install+pytest por plataforma nativa (aarch64 exento con justificación MKT-18h/BND-09), publish TestPyPI por `workflow_dispatch` (`publish_testpypi=true`, environment `testpypi`), publish PyPI en tag `v*` (environment `pypi` + attestations NON-CRITICAL + attach a GH Release), verify-install en ambos (TestPyPI con espera CDN 30s, PyPI con retry backoff) + verificación de provenance. Pins SHA en todas las actions ✅. `continue-on-error: true` solo en attest con `# CATEGORY: NON-CRITICAL` ✅ (Regla 5 OK).
- **Hallazgo clave (secret-state):** los jobs de publish usan `id-token: write` (Trusted Publishing OIDC) + `environment: testpypi/pypi` — NO referencian `${{ secrets.TEST_PYPI_API_TOKEN }}` ni `PYPI_API_TOKEN`. El owner debe configurar Trusted Publisher en PyPI/TestPyPI (o añadir API token como secret del environment) — la instrucción exacta va en el STOP/handoff.
- **pyproject/Cargo.toml:** versión 0.5.0 en ambos, `version.workspace=true`, `publish=false` (el crate Rust no se publica; solo el wheel), abi3-py311, classifiers 3.11+3.13 (FIND-85).
- **Prereqs PROV-01/02/04:** SIN filas en `docs/dev/Backlog.md` (solo mención P45 como quickwins aprobados → plan 2026-08-25 nunca ejecutado como filas). Sustancia: eran quickwins de `providers/*` (adapters externos). `providers/` están EXCLUIDOS del workspace (Backlog:495 CRIT) y el wheel `vantadb-py` depende solo del core (`vantadb` path dep) — los providers NO bloquean la publicación del wheel. **Veredicto: N/A en sustancia, documentado; no re-triage (no se ejecuta a ciegas: se verifica lo verificable).**
- **Impacto mapeado (Regla 0):** archivos leídos completos: pyproject.toml, vantadb-python/Cargo.toml, release-wheels-60.yml, CI_POLICY.md §9, verify_published_wheel.py, release-ci.md, definition-of-done.md. Referencias hacia dentro: workflow → `vantadb-python/tests/test_sdk.py`, `verify_published_wheel.py`, `Cargo.toml` workspace, environments `testpypi/pypi`. Referencias entrantes: CI_POLICY §9, Backlog PROV-12, plan Wave2. **Veredicto: impacto BAJO — tarea de verificación; cambios esperados: ninguno en workflow/código (o fix mínimo), task file nuevo + commit `ci:`.**

## INVESTIGACIÓN PROBLEMA

- **Publish sin secret local (vía CI):** imposible publicar desde esta máquina (sin token, sin OIDC). Lo verificable aquí: build del wheel, metadata, install local en venv limpio, smoke funcional, coherencia de versión, lint del workflow. Lo que REQUIERE al owner: configurar Trusted Publishing (o API tokens) en GitHub environments + PyPI/TestPyPI, disparar `workflow_dispatch` con `publish_testpypi=true`, luego tag `v*` para PyPI.
- **TestPyPI-primero:** orden ya codificado en el workflow (dispatch manual → tag). Estrategia: 1) dispatch TestPyPI, 2) verify-install verde, 3) tag → PyPI + verify + provenance.
- **Versión-desde-tag:** release-plz deriva del tag; el workflow extrae versión del tag (`${GITHUB_REF#refs/tags/v}`) con fallback a `Cargo.toml` en TestPyPI. Sin edición manual ✅.

## INVESTIGACIÓN INTERNET

- N/A — el workflow existente + CI_POLICY §9 documentan el flujo completo; sin gaps de docs PyPI que lo exijan. (Si surge gap en EJECUCIÓN → solo docs oficiales, URLs verificadas.)

## STEPS (atómicos, ~100 líneas/step, cada uno reversible)

- [x] **Step 1 — DISCOVERY: prereqs + workflow + rules + Gate D** → veredicto PROV-01/02/04 N/A en sustancia; workflow leído entero; rules/refs leídas; Gate D GO. (este step)
- [x] **Step 2 — EJECUCIÓN: actionlint + coherencia versión + maturin build (dry-run sin publish)** → ✅ actionlint exit 0 · workspace 0.5.0 == pyproject 0.5.0 · `__version__`=reported_version()=CARGO_PKG_VERSION (`src/metadata.rs:28`, `vantadb-python/src/lib.rs:2414`) · `maturin build --release` OK (5m05s, `-j 2` vía CARGO_BUILD_JOBS) → `dist/vantadb_py-0.5.0-cp311-abi3-win_amd64.whl` · METADATA: Name vantadb-py / Version 0.5.0 / Requires-Python >=3.11. Nota: 15 warnings pre-existentes del core en release (no míos, fuera de scope); wheel cayó en `dist/` raíz (gitignored ✅).
- [x] **Step 3 — EJECUCIÓN: venv limpio + pip install wheel + smoke verify_published_wheel.py verde + docs versión==código** → ✅ venv fresco en `%TEMP%\prov12-smoke` · `pip install dist/vantadb_py-0.5.0-...whl` OK (vantadb-py-0.5.0) · `verify_published_wheel.py` PASSED con `VANTADB_EXPECTED_VERSION=0.5.0` (version match + put/get/list/search/caps/durabilidad) · docs: `pip install vantadb-py` sin pin (`PYTHON_SDK.md:29` ✅), sin hardcodes divergentes; menciones 0.5.0/0.6.0 son marcadores de disponibilidad, no pins. Observación (Gate C): `DeprecationWarning: 'vantadb_py' import name deprecated → use 'import vantadb'` (pre-existente, área python-bindings, fuera de scope).
- [x] **Step 4 — CIERRE: verify contrato + Gate C (colaterales) + commit selectivo `ci:` (NO PUSH) + STOP/handoff owner + RESULTADO §7** → ✅ contrato en seco cumplido (ver AC) · `campaign_verify_cmd` bug exit -1 confirmado (bash directa: actionlint exit 0) · Gate C vía question → "arregla todo": (3) wheels viejas 0.1.5/0.4.0 ELIMINADAS de `dist/` y `vantadb-python/dist/` (artefactos gitignored, queda solo 0.5.0 fresca); (1) DeprecationWarning `vantadb_py→vantadb` es deprecación INTENCIONAL (AST-010, removal en 0.6.0) — no se revierte, el smoke CI la usa por diseño; (2) 15 warnings core en release → `src/` PROHIBIDO en esta task, se deriva al orquestador (candidata FIND-*; ni Backlog ni src/ tocados). Commit selectivo solo `docs/dev/tasks/PROV-12.md`, NO PUSH. STOP publish: requiere owner (instrucción exacta abajo).

## Context Save Point

- Rama: develop (verificado). Versión: 0.5.0 workspace == pyproject (verificado). Maturin 1.15.0 disponible. actionlint disponible (winget). Python local 3.14.7 (classifiers declaran 3.11/3.13 — wheel abi3-py311 instalable en >=3.11; notar en step 3 si hay fricción).
- WIP ajeno en `git status` (`.opencode`, `completions/*`, `docs/dev/Backlog.md`, plan file, `skills/*`, `reparacion.bat` untracked) → staging SELECTIVO: solo `docs/dev/tasks/PROV-12.md` (+ workflow si hay fix mínimo, no esperado).
- Nada publicado, ningún secret tocado. Secret-state: workflow usa OIDC trusted publishing (sin secrets.* en el YAML).

## STOP / HANDOFF AL OWNER (publish real — no verificable ni ejecutable desde aquí)

**Estado:** "lista para publicar", NO "publicado". Nada se publicó; ningún secret existe en esta máquina.

**Hallazgo normativo:** `release-wheels-60.yml` NO usa `secrets.TEST_PYPI_API_TOKEN` / `secrets.PYPI_API_TOKEN` — los jobs `publish-testpypi` (`:139-163`) y `publish-pypi` (`:164-202`) usan **Trusted Publishing OIDC** (`permissions: id-token: write` + `environment: testpypi/pypi`). El owner debe configurar UNA de las dos vías:

**Vía A (recomendada, sin tokens): Trusted Publishing**
1. TestPyPI: https://test.pypi.org/manage/account/publishing/ → Add publisher → owner `ness-e`, repo `Vantadb`, workflow `release-wheels-60.yml`, environment `testpypi`.
2. PyPI: https://pypi.org/manage/account/publishing/ → igual con environment `pypi`.
3. GitHub repo → Settings → Environments → crear `testpypi` y `pypi` (protection rules a criterio; `pypi` con required reviewers recomendado).

**Vía B (tokens):** TestPyPI/PyPI → API token (scope proyecto `vantadb-py`) → GitHub repo → Settings → Secrets and variables → Actions → **Environment secrets** (NO repository secrets): `TEST_PYPI_API_TOKEN` en environment `testpypi`, `PYPI_API_TOKEN` en environment `pypi` — y añadir `password: ${{ secrets... }}`... NOTA: el workflow actual NO tiene campo `password:` (asume OIDC); si se usa Vía B hay que editar el workflow (task follow-up, no esta).

**Orden de disparo (TestPyPI primero, Gate V):**
1. Actions → `RELEASE: Wheels — Build & Publish` → Run workflow (branch main) → `publish_testpypi: true` → verde + job `verify-testpypi-install` verde.
2. `pip install -i https://test.pypi.org/simple/ --extra-index-url https://pypi.org/simple/ "vantadb-py==<tag>"` en máquina limpia + smoke.
3. Tag `git tag -a v<ver> && git push origin v<ver>` (release-plz coordina versión) → job `publish-pypi` + `verify-pypi-install` verdes.
4. Rollback: yank release en PyPI + `git revert` del tag-commit si el wheel está roto; matriz parcial por OS con nota (pre-mortem §2).

## VALIDACIÓN+CIERRE

- Verify contrato (steps 2-3) + `campaign_verify_cmd` (bug exit -1 → bash directa + mención).
- OCR delegation: N/A-justificado (CI/config, 0 código tocado — sin superficie para OCR; se corre solo si hay fix en workflow).
- DoD 3 niveles: (v1) determinista lo aplicable (fmt N/A sin código; clippy N/A; nextest N/A — se declara); standing checklist (correctness del dry-run, quality scope-discipline, integration con CI existente, docs); shippable trunk-based (rollback = revert del commit `ci:` / no hay migración).
- P2-01: lo hace el orquestador (no yo). Plan file: NO lo edito (WIP ajeno en working tree) — recitation para el orquestador en el mensaje de cierre.
- Gates: D ✅ GO (question 2026-09-19) · V resuelto por owner (no re-preguntar) · C vía `question` al cierre (colaterales).
