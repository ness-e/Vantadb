---
title: "TASK PROV-13: Providers OpenAI/Ollama/LiteLLM en Windows (compilar + CI)"
kind: task
description: "Windows job (matriz provider × ubuntu/windows) en providers-ci.yml + fix del crash de encoding en verify_pyi.py (UnicodeEncodeError con cp1252/ibm437) + cache de providers/*/target. Compilación y suites ya verdes en Windows HEAD (premisa 'no compilan' stale — resuelta por PROV-01/02/04)."
---

# TASK PROV-13: Providers OpenAI/Ollama/LiteLLM en Windows (compilar + CI)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 31, F1)
- **Fuente:** Backlog PROV-13 — decisión owner 2026-10-01 (Q5/B3): **arreglar, no declarar límite**; "los 3 providers no compilan en Windows; requiere fix + CI Windows"
- **Esfuerzo:** 🟡 2-4d | **Appetite:** max 4d | **Prioridad:** 🟠
- **Tipo:** CI/CD (gate script fix + workflow Windows; sin cambios de contrato de los providers)
- **Creado:** 2026-10-05 | **Estado:** ⏳ EN PROGRESO
- **Campaign ID:** master-plan-0.9.0-20261004 · **taskId server:** 31
- **Ejecutor:** vanta-worker
- **Incógnitas (uphill):** 0 abiertas (resueltas en DISCOVERY — ver Investigation Notes)
- **Pendientes (downhill):** 4 steps (2-5)

## Blast Radius

**Archivos exactos del cambio (modificados/nuevo):** `.github/scripts/verify_pyi.py` · `.github/workflows/providers-ci.yml` · `docs/dev/tasks/PROV-13.md` (nuevo)

| Dirección | Módulos |
|-----------|---------|
| Callers | `providers-ci.yml` es el único consumidor de `verify_pyi.py` (grep ✅ — `release-providers.yml` usa `provider_wheel_smoke.py`, sin cambios); los 3 READMEs de providers citan el CI |
| Callees | `providers/{openai,ollama,litellm}/` (Cargo.toml/src/tests/.pyi — **sin cambios**), actions pinneadas (checkout/setup-python/rust-toolchain/rust-cache), `.venv` local 3.11.9 (verificación) |
| Implicaciones | El cambio es de **infra CI únicamente**: no toca código Rust, firmas PyO3, stubs `.pyi` ni wheels. El leg ubuntu preserva sus comandos exactos (solo cambia la invocación del pyi-step a env var cross-shell y la config de cache). El leg Windows es nuevo (6 legs: 3 providers × 2 OS). El fix de `verify_pyi.py` elimina una dependencia de encoding del host (crash reproducido en Windows local). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `.github/scripts/verify_pyi.py` (167L), `.github/workflows/providers-ci.yml` (92L), `providers/openai/Cargo.toml` (23L), `providers/openai/pyproject.toml` (45L), `providers/openai/tests/test_openai.py` (217L), `providers/ollama/tests/test_ollama.py:115-174`, `docs/dev/tasks/PROV-01.md` (parcial), `docs/dev/tasks/PROV-12.md` (166L), `ci-rust.yml:255-309` (job `test-windows` — convención), `release-wheels.yml:100-139` (patrón venv Windows), `release-providers.yml` (grep), README rust-cache (docs oficiales v2).
- **Archivos referenciados hacia dentro (imports/deps):** `verify_pyi.py` → `ast/inspect/os/sys/pathlib` (stdlib; sin deps nuevas); `providers-ci.yml` → actions pinneadas (mismos SHA que el resto del repo) + `providers/*/Cargo.toml` + `.github/scripts/verify_pyi.py`.
- **Archivos que referencian a los editados (referencias entrantes):** grep `verify_pyi` → solo `providers-ci.yml:92`; grep `providers-ci` → `CI_POLICY.md`, `PUBLISH.md`, `TRIGGERS.md`, `README.md` (docs de inventario — sin cambios de contrato: el workflow mantiene nombre, triggers y cobertura, solo suma legs). `providers/{openai,ollama,litellm}/pyproject.toml` de PROV-12 **no se tocan**.
- **Veredicto impacto:** **BAJO** — infra CI acotada a 2 archivos; sin consumidores de código; el único riesgo es de sintaxis/shell del workflow (mitigado con `actionlint` + ejecución local de los comandos exactos del leg Windows en Windows real).

## Contrato

> Del plan (Task 31) — "los 3 providers compilan (`cargo check`/`clippy -D warnings` vía manifest-path) y sus tests pasan **en Windows**; job CI Windows verde (matriz provider); sin regresión en Linux/macOS (providers-ci.yml actual sigue verde); si el fix toca `shared_py.rs`, los 3 providers verificados."

1. **Compilación Windows ×3** (reproducida): `cargo check` + `cargo clippy -D warnings` vía `--manifest-path` exit 0 — ✅ verificado en DISCOVERY (HEAD 2026-10-05)
2. **Tests Windows ×3**: suites de providers verdes en Windows (`maturin develop --release` + pytest) — ✅ verificado en DISCOVERY (18/17/19)
3. **Job CI Windows** en `providers-ci.yml` (matriz provider × os) con los comandos equivalentes a la verificación local — ⬜ Step 3
4. **Fix del bloqueante real de Windows**: `verify_pyi.py` crashea con `UnicodeEncodeError` (cp1252/ibm437) → exit 0 en cualquier consola — ⬜ Step 2
5. **Sin regresión Linux/macOS**: leg ubuntu preserva comandos/cobertura; `actionlint` 0 — ⬜ Steps 3-4
6. **Costo acotado** (pre-mortem #3): cache de `providers/*/target` (rust-cache `workspaces`) + timeout acotado — ⬜ Step 3
7. **Sin cambios de contrato de providers**: módulos/clases/firmas `.pyi` intactos; `shared_py.rs` NO se toca — invariante verificado en cierre — ⬜ Step 4

**Nota de cierre honesta (Regla 7 — push prohibido):** el "job CI Windows verde" **real** requiere push (diferido al owner). El cierre verifica: (a) los comandos exactos del leg Windows ejecutados en Windows local (proxy fuerte), (b) `actionlint` 0, (c) `verify_pyi.py` con stdout cp1252/ibm437 → exit 0. El verde del run de GitHub queda en `deuda` como verificación post-push del owner (patrón PROV-12).

## Spec (decisiones — SDD)

No aplica Spec formal — fix de infra CI sin símbolos públicos nuevos (no agrega `pub fn`/endpoint/binding). Decisiones menores registradas:

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Estructura del job Windows | A) **matriz `provider × os` en el job `check` existente** (6 legs; 1 definición; `if: runner.os` para el venv) / B) job `check-windows` separado (duplica 40L; drift) | ✅ A (convención repo: `release-wheels.yml`/`release-providers.yml` usan matriz con `if: matrix.os`) |
| 2 | Invocación del pyi-step | A) **`PROVIDER` env + script directo** (cross-shell bash/pwsh; modo local ya documentado en el script) / B) mantener exec/replace (solo bash: pwsh expande `${PROVIDER}`→vacío → SyntaxError) | ✅ A (verificado local) |
| 3 | Fix de encoding | A) **`sys.stdout.reconfigure(utf-8)` en el script** (raíz: no depender del locale del host) / B) `PYTHONIOENCODING` en el step (parche CI; el crash sigue en local) / C) quitar `✓` (pierde señal) | ✅ A (+`errors="replace"` defensivo) |
| 4 | Cache | A) **rust-cache `workspaces` = los 3 provider dirs** (default `. -> target` NO cachea providers — docs oficiales) / B) dejarlo (rebuild total por run; Windows cold ≈ riesgo de timeout) | ✅ A |
| 5 | Timeout | A) **30 min** (ubuntu cold 7min medido; Windows cold ≈10-13min estimado; 15 actual riesgo) / B) 15 (riesgo de timeout en el primer run cold) | ✅ A (acotado; convención repo: `test-windows` usa 60) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) **sin push, sin tags** (commit local — Regla 7); (2) contrato Python de los providers intacto (módulos/clases/firmas — PROV-04); (3) `providers/{openai,ollama,litellm}/pyproject.toml`, READMEs y `release-providers.yml` de PROV-12 **no se tocan**; (4) el leg ubuntu del workflow preserva sus comandos y cobertura (solo cambia la invocación del pyi-step y la config de cache — sin cambio funcional); (5) WIP ajeno intacto (`opencode.jsonc`, master plan, `docs/pipeline-state.json`, cambios en vuelo de otras sesiones); (6) `shared_py.rs` NO se modifica (no hay fix de compilación: la premisa era stale).
- **Comandos de verificación:** ver Contrato + `campaign_verify_cmd`.
- **Deuda pendiente:** verde real del job Windows en GitHub = post-push (owner) — ver nota de cierre.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato 1-7 ✅ (compilación+tests Windows reproducidos; job Windows agregado; fix encoding; actionlint 0; cache+timeout; contrato intacto). |
| **Commit** | Commit local `fix(providers):` (conventional; SIN push), verificación mecánica, sin archivos fuera del blast radius. |
| **Release** | n/a (infra CI; sin entrada de changelog — `ci:`-clase). |

## Steps

1. ✅ **DISCOVERY** (este archivo): repro por provider (check/clippy/tests en Windows), aislamiento del bloqueante real (encoding de `verify_pyi.py`), blast radius + Regla 0.
2. ✅ **Fix `verify_pyi.py`**: `sys.stdout.reconfigure(encoding="utf-8", errors="replace")` + docstring de invocación — verificado ×3 con `PYTHONIOENCODING=cp1252` y consola ibm437 → exit 0 + `✓` (campaign_verify_cmd pyi matrix ✅).
3. ✅ **`providers-ci.yml`**: matriz `provider × [ubuntu-latest, windows-latest]`, venv condicional por OS (POSIX `bin/` vs Windows `Scripts/` + `shell: pwsh`), pyi-step con `PROVIDER` env, rust-cache `workspaces` (3 providers), timeout 30, paths += `.github/scripts/verify_pyi.py` — `actionlint` 0 (campaign_verify_cmd ✅).
4. ✅ **Verificación local Windows (simulación del leg)**: clippy ×3 (campaign ✅) + `maturin develop --release` ×3 ✅ + pytest ×3 54/54 (campaign ✅; receta ENOSPC: limpiar `pytest-of-Eros` por suite) + pyi ×3 ✅ + `verify_changed.ps1` ALL 4 PASS + `verify.ps1` ALL 10 PASS.
5. ⬜ **Cierre**: OCR delegation + review P2-01 (vanta-review) + commit local + campaign close (taskId 31) + RESULTADO §7.

## Evidencia de verificación (DISCOVERY — 2026-10-05, Windows local)

| Gate | Comando | Resultado |
|------|---------|-----------|
| Compila ×3 | `cargo check --manifest-path providers/{openai,ollama,litellm}/Cargo.toml` | ✅ exit 0 ×3 (38s/33s/38s; fresh compile units) |
| Clippy ×3 | `cargo clippy --manifest-path ... -- -D warnings` | ✅ exit 0 ×3 |
| Build CI ×3 | `maturin develop --release` (venv 3.11.9, VIRTUAL_ENV) | ✅ openai `cp311-abi3-win_amd64` instalado (1m06s) |
| Tests ×3 | `pytest providers/<p>/tests/` | ✅ openai 18 · ollama 17 · litellm 19 (54 total) |
| **Bug Windows** | `PROVIDER=openai python verify_pyi.py` (stdout cp1252/ibm437) | ❌ **`UnicodeEncodeError: 'charmap' codec can't encode character '\u2713'`** → exit 1 (reproducido con cp1252 e ibm437) |
| Soporte | `vantadb-py 0.5.0` en PyPI | ✅ wheel `cp311-abi3-win_amd64` existe (pip install OK en Windows) |
| Tooling | `actionlint 1.7.12` · `maturin 1.15.0/1.13.3` · `.venv` 3.11.9 | ✅ |
| Gates finales | `dev-tools/verify_changed.ps1` | ✅ ALL 4 PASS (fmt/check/clippy/docs-coverage) |
| Verify full | `dev-tools/verify.ps1` | ✅ **ALL 10 PASS** (fmt/check/clippy/audit/deny/nextest/docs-coverage/cli-probes/consumo/backup) |
| Mechanical records | `campaign_verify_cmd`: actionlint · clippy matrix · pytest matrix · pyi matrix | ✅ ×4 exit 0 (actionlint 3.6s · clippy 15.0s · pytest 50.8s 54/54 · pyi 1.8s) |
| OCR delegation | `pwsh dev-tools/ocr-review.ps1 -Format json` (2 reviewables) | ✅ 0 Critical/High — Group 1 (`verify_pyi.py`) 0 hallazgos; Group 2 (`providers-ci.yml`) 0 Critical/High + 1 nit aplicado (`shell: pwsh` explícito en el step Windows, convención `release-wheels.yml`) |
| Nota ENOSPC | C: fluctuó a 0GB durante la ventana (sesiones paralelas + DBs de test ~1.2GB/suite) | 3 corridas de suite fallaron con `StorageFull (os 112)`; pasan con `pytest-of-Eros` limpio por suite (receta PROV-12). No es falla de código; el CI usa runners limpios. |

## Investigation Notes (DISCOVERY — 2026-10-05)

1. **Premisa stale confirmada**: "los 3 providers no compilan en Windows" **no reproduce en HEAD** — los fixes de PROV-01/02/04 (errores de campos faltantes, no Windows-específicos) la resolvieron; PROV-12 ya lo había anotado (`PROV-12.md:38,129`). Repro fresco: `cargo check` + `clippy -D warnings` ×3 ✅ en Windows.
2. **Bloqueante REAL de Windows encontrado (aislado)**: `.github/scripts/verify_pyi.py:161` imprime `✓` (U+2713) sin proteger el encoding → con stdout cp1252/ibm437 (default de consolas/pipes Windows) crashea con `UnicodeEncodeError` → exit 1. El futuro leg Windows habría estado ROJO en el step `.pyi` (falso rojo del gate). Fix raíz en el script (no parche en el workflow).
3. **Gaps del workflow para Windows**: (a) venv POSIX (`$PWD/.venv/bin`) no aplica a Windows (`Scripts/`); (b) el pyi-step usa `exec(...replace('\${PROVIDER}',...))` que solo funciona en bash — en pwsh `${PROVIDER}` se expande a vacío → `SyntaxError`; (c) rust-cache default (`workspaces: ". -> target"`) no cachea `providers/*/target` (crats standalone con `[workspace]` vacío) → cada run rebuild completo; (d) `paths:` no incluye `.github/scripts/verify_pyi.py` (cambios del gate no disparan el workflow).
4. **Evidencia de costo**: runs ubuntu reales (`gh run list`): warm ~3m12s-3m39s/job, cold (PR) ~7m02s. Windows estimado ×1.5-2 → timeout 15 actual es riesgo en el primer run cold → 30 acotado.
5. **Convenciones repo usadas**: `windows-latest` + `if: matrix.os == 'windows-latest'` (release-wheels/release-providers), `shell: pwsh` para pasos Windows, actions pinneadas por SHA (mismos del archivo), `test-windows` de ci-rust usa timeout 60.
6. **Soporte pip**: `vantadb-py==0.5.0` tiene wheel win_amd64 (PyPI JSON verificado) → `pip install vantadb-py` del CI funciona en Windows; `test_ollama.py:136` lo usa con `importorskip`.
7. **Alcance descartado**: `provider_wheel_smoke.py` (usado por release-providers en Windows) es ASCII puro → sin el bug; `shared_py.rs` compila ×3 → sin cambios; no se tocó `pyproject.toml` ni READMEs de PROV-12.

## Review P2-01 (fresh — vanta-review)

- **Revisor:** vanta-review (contexto fresco, P2-01 — no participó de la implementación; sesión `ses_ef58a46c8ffewaxi0r81ecwmvu`)
- **Enfoque:** auditoría del diff completo (2 archivos + task file) con re-ejecución del contrato en Windows local, actionlint, red-team de shell/cache/CI y verificación de scope contra los rulesets vivos.
- **Cómo se probó:** `actionlint providers-ci.yml` = 0 (dir completo 0) · pyi ×3 exit 0 con `✓` bajo cp1252/ibm437/default y con `GITHUB_WORKSPACE` seteado (CI-fidelity); versión HEAD reproduce `UnicodeEncodeError` exit 1 · pytest 18/17/19 = 54/54 exit 0 · clippy ×3 exit 0 (14.7s) · simulación pwsh del step venv Windows (GITHUB_ENV/GITHUB_PATH correctos) · `config.ts` de rust-cache (workspaces repo-relative + key por OS) · `gh api` rulesets: ninguna required check `Check provider (*)` · PyPI: `vantadb-py` 0.8.0 tiene win_amd64 · 0 skips de plataforma en tests.
- **Veredicto:** ✅ APPROVE — Critical 0 / High 0; Low: commit sugerido `ci(providers):` (convención) y limitar commit a los 3 archivos del blast radius (revertir Cargo.lock; no arrastrar índices generados). ENOSPC (os 112) documentado como ambiental (54/54 verde con basetemp fresco).
- **No verificado:** run real de GitHub (post-push, Regla 7) — deuda declarada; leg ubuntu re-ejecutado (inferido de diff + actionlint + run previo verde).

### Disposición de hallazgos

| Hallazgo | Disposición |
|----------|-------------|
| L-1 commit `ci(providers):` | ✅ aplicado (commit de cierre) |
| L-2 commit limitado a los 3 archivos (revertir Cargo.lock; no arrastrar `docs/index.md`/`llms.txt`) | ✅ aplicado (staging por pathspec; locks revertidos) |
| Nit comentario `0.5.0` stale en `providers-ci.yml` | ✅ aplicado (comentario corregido: `vantadb-py` unpinned → pip resuelve latest) |
| Nit `stderr` sin reconfigure en `verify_pyi.py` | Diferido — sin impresión Unicode por stderr hoy (documentado, sin acción) |
| M-1 verde real del run GitHub | Deuda declarada post-push (owner, Regla 7) |

## Context Save Point

- **Step actual:** 5 (cierre) — review P2-01 APPROVE (vanta-review); commit local en curso.
- **Worktree:** `develop` (commits locales; sin push)
- **Próxima acción:** revertir Cargo.lock ×3 → `git add` solo los 3 archivos del blast radius → commit `ci(providers): PROV-13` → RESULTADO §7 (commit docs) → campaign close (taskId 31).

## RESULTADO (§7)

```
RESULTADO: (pendiente de cierre)
```
