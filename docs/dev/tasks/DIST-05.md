---
title: "TASK DIST-05: Assets del release + verificación post-release real (fix del 404)"
kind: task
description: "Flujo post-release con verificación ejecutable de artefactos (script + workflow + gate en PUBLISH.md) + install.ps1 parametrizado/verificado contra los assets reales (+ fix compat Windows PowerShell 5.1)"
---

# TASK DIST-05: Assets del release + verificación post-release real

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 18, F0)
- **Fuente:** Backlog `DIST-05` (H-011) — zip Windows de v0.7.0 → 404 live verificado
- **Esfuerzo:** 🟢 4h | **Appetite:** max 1d | **Prioridad:** 🟠
- **Tipo:** Release/CI (script + workflow + docs; sin lógica Rust)
- **Creado:** 2026-10-04 | **Estado:** ⏳ IN PROGRESS (steps 6/6 ejecutados; review P2-01 + commit en curso)
- **Campaign ID:** master-plan-0.9.0-20261004
- **Ejecutor:** vanta-lead (dominio release/CI)
- **Incógnitas (uphill):** 0 (resueltas en DISCOVERY — ver Investigation Notes)
- **Pendientes (downhill):** 1 (cierre: review P2-01 + commit)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | QUICKSTART §0 + README + DISTRIBUTION.md (one-liner `install.ps1`); inventario de workflows (`README.md`/`TRIGGERS.md`/`verified-numbers.md`) indexa el `release-verify.yml` nuevo; release drivers leen `PUBLISH.md` |
| Callees | GitHub REST API (`releases/latest`, `releases/tags/<tag>`, assets), registries (crates.io / PyPI / npm), raw.githubusercontent.com (wizard), assets reales del release (zip/tar.gz + `.sha256`) |
| Implicaciones | Sin cambios de código Rust, versión, changelog ni release-plz. `install.ps1`: +`-Version`, fallback stale eliminado (fail-loud), compat PS 5.1. Nuevo `scripts/verify-release.ps1` + `release-verify.yml` (schedule+dispatch) + gate en PUBLISH.md. `release-binaries.yml` NO se toca (operativo — Investigation Notes §2). Número versionado: workflows 37→38 (`verified-numbers.md` regla 2 → mismo commit). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `scripts/install.ps1` (133L), `scripts/install.sh` (182L — referencia de comportamiento fail-loud), `docs/dev/workflow/PUBLISH.md` (129L), `docs/dev/workflow/README.md` (91L), `docs/dev/workflow/TRIGGERS.md` (101L), `.github/workflows/release-binaries.yml` (156L), `docs/dev/references/verified-numbers.md` (45L), `docs/user/QUICKSTART.md:1-100`, `.opencode/rules/release-ci.md` (42L), `docs/dev/workflow/RULES.md` (218L), `docs/dev/avance/activo/ci-cd.md:480-560`, `docs/dev/tasks/DIST-01.md`, `docs/dev/tasks/DIST-04.md` (formato)
- **Referencias hacia dentro:** `install.ps1` ← 19 archivos (README.md, README_ES.md, SPEC.md, docs/user/{QUICKSTART,AI_IDES}.md, docs/index.md, DISTRIBUTION.md, FASE-A.md, Backlog, tasks FIND-105/115/176…) — ninguno consume el formato interno; `+ -Version` es aditivo · `PUBLISH.md` ← 13 (RULES.md, llms.txt, README.md, check-npm-versions.mjs, ci-cd.md, DIST-01, FIND-142/143/144/230…) — gana sección, links intactos · `release-binaries.yml` ← 26+ (Formula/vantadb.rb, CI_POLICY.md, DEPLOYMENT_GUIDE.md…) — sin cambios
- **Veredicto impacto:** **BAJO** — todo aditivo: param nuevo, sección nueva, 2 archivos nuevos. Sin consumidores rotos; sin cambios de comportamiento en lanes existentes.

## Contrato

> Del plan (Task 18) + instrucción del orquestador. **(c) es post-release → diferido documentado** (release 0.9.0 diferido por decisión owner 2026-10-04).

- **(a)** Flujo post-release con verificación **real** de artefactos (pip/npm/crates + assets de GitHub) + smoke del quickstart en máquina limpia → checklist **ejecutable** en `PUBLISH.md` (script `scripts/verify-release.ps1` + workflow `release-verify.yml` schedule/dispatch).
- **(b)** `install.ps1` parametrizado (`-Version`) y verificado contra los assets reales (descarga real + checksum + `vanta-cli --version`) + fix de compat Windows PowerShell 5.1 (hoy **no parsea**: ternary PS7 en `:31`) + eliminación del fallback stale `v0.4.0` (release inexistente → 404 garantizado).
- **(c)** Ejecutado y verde en el release 0.9.0 → **diferido** (release diferido por owner; el checkpoint F0→0.9.0 del plan lo retoma).

**Comandos de verificación (contrato mecánico):**

1. `pwsh scripts/verify-release.ps1 -Tag v0.8.0 -Smoke` → exit 0, ALL GREEN (14 assets + 4 registries + smoke del binario)
2. `pwsh scripts/verify-release.ps1 -Tag v0.7.0` → exit 1, falla **solo** en los 10 binarios (demuestra que detecta el 404)
3. `powershell -NoProfile -File scripts/install.ps1 -DryRun` (5.1) **y** `pwsh -NoProfile -File scripts/install.ps1 -DryRun` (7) → exit 0 ambos
4. `pwsh scripts/install.ps1 -Version v0.8.0 -NoWizard` con `$HOME` falso → instala + `vanta-cli.exe --version` reporta 0.8.0
5. `actionlint .github/workflows/release-verify.yml` → 0
6. `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs` → 0

## Spec (SDD — decisiones de release/CI, no feature-add)

> Phase 1b: no agrega símbolos públicos de producto (script interno + workflow + flag de instalador). Se llena igual: hay decisiones técnicas de release con evidencia.

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Dónde vive la verificación post-release | A) **script ejecutable + workflow (schedule semanal + dispatch) + gate en PUBLISH.md** (real, corre sin humano) / B) solo checklist manual en PUBLISH.md (nadie la corre — pre-mortem #2) / C) job dentro de `release-binaries.yml` (race con wheels/npm que publican en paralelo → falso rojo) | A | ✅ decidido-por-evidencia: pre-mortem #2 "checklist que nadie corre → automatizarlo"; C descartado por race (cascadas asíncronas del mismo tag) |
| 2 | Disposición de v0.7.0 (sin binarios) | A) **documentar la ausencia + no backfill** (los refs viejos no pueden producir binarios honestos: el workflow de la era no tiene input `release_tag`; dispatch desde main = binarios de otra versión) / B) backfill con branch temporal desde v0.7.0 (operación remota + build viejo con fixes post-0.8.0 ausentes → probable fallo) | A | ✅ decidido-por-evidencia: `release-binaries.yml@v0.7.0` sin input de upload; cascade suprimido pre-PAT (token creado 2026-10-02); `latest`=v0.8.0 con assets 5/5 → funnel no afectado |
| 3 | Fallback de `install.ps1` cuando la API falla | A) **fail-loud + `-Version` explícito** (consistente con `install.sh:97-101`) / B) pin conocido-bueno hardcodeado (se pudre; el actual `v0.4.0` ni existe) | A | ✅ decidido-por-evidencia: `gh release view v0.4.0` → "release not found"; install.sh ya es fail-loud |
| 4 | Compat de shell de `install.ps1` | A) **5.1 + 7** (one-liner `irm \| iex` corre en el shell del usuario; Windows 11 default = 5.1) / B) solo 7 (hoy: parse error en 5.1 → instalador muerto para el default) | A | ✅ decidido-por-evidencia: repro `powershell -NoProfile -File` → `UnexpectedToken '?'` en `:31` |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) **sin push** (política owner: push diferido al cierre del plan) y sin tags; (2) `release-plz.toml` / versiones / changelog intactos; (3) WIP ajeno intacto (`opencode.jsonc`, plan file, `docs/pipeline-state.json`, `vanta-memory/**`, `vantadb-python/**`, `vantadb-ts/**`); (4) `release-binaries.yml` sin cambios (no romper el lane que ya funciona); (5) el installer sigue siendo curl/irm-friendly (sin dependencias nuevas); (6) el script de verificación no requiere auth para el repo público (GH_TOKEN opcional = solo rate limit).
- **Comandos de verificación:** los 6 del Contrato.
- **Deuda pendiente:** verificación en 0.9.0 = diferida (post-release); backfill opcional de v0.7.0 = procedimiento documentado, no ejecutado (operación remota).

## Deuda técnica (Regla 6)

Sin deuda nueva. `verify-release.ps1` mantiene la lista de assets esperados hardcodeada (5 targets + 4 wheels): si el matrix de `release-binaries.yml` cambia, el script debe actualizarse — se documenta como invariante de mantenimiento en el header del script (no es deuda P2).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato (a)+(b) verificados con los 6 comandos; (c) diferido documentado. Gates docs (check-links/check-docs) + actionlint verdes. |
| **Commit** | Commit atómico `ci: DIST-05 — …` (conventional), verificación mecánica (nunca auto-reporte), sin archivos fuera del blast radius. |
| **Release** | n/a propio — el task ES parte del release (checkpoint F0→0.9.0: "verificación post-release (Task 18)"); changelog lo maneja release-plz. |

## Herramientas necesarias

- `pwsh` (7) + `powershell` (5.1) — verificación dual del installer
- `gh` CLI — inspección de releases/assets reales; `actionlint` — gate del workflow
- `node scripts/docs/*.mjs` — gates de docs
- `campaign_verify_cmd` / `campaign_update_task_state` — verificación y estado

**Skills cargadas (SDP):** ci-cd-and-automation (pinned policy CI/release) · git-workflow-and-versioning (pinned) · shipping-and-launch (post-release/rollout) · doubt-driven-development (verificación adversarial de decisiones) · documentation-skill (obligatoria: edita docs/) · source-driven-development (convenciones GH release assets) · incremental-implementation (slices). Base auto: campaign-executor · progreso.

## Investigation Notes (DISCOVERY — 2026-10-04)

1. **v0.7.0 = 4 wheels, sin zip (re-verificado live):** `gh release view v0.7.0 --json assets` → solo `vantadb_py-0.7.0-*.whl` (4). v0.6.1 igual. **Causa raíz histórica:** el cascade `release:published` estaba suprimido (release-plz creaba releases con `GITHUB_TOKEN`; el PAT `RELEASE_PLZ_TOKEN` recién se creó 2026-10-02 — `gh secret list` lo confirma) y `release-binaries` falló su primera ejecución real (FIND-229, v0.8.0) → backfill manual (`37082059543`, success, 33m44s) → **v0.8.0 = 5/5 binarios + 4 wheels**. El run normal (release event) nunca pasó verde; el dispatch sí. Por eso la verificación post-release es el gate que faltaba.
2. **`release-binaries.yml` hoy:** operativo (backfill verde). Upload gatea por `release:published` o `inputs.release_tag`; matrix 5 targets; Windows zip con layout `release\vanta-cli.exe` (consistente con el installer). **No requiere cambios** — el gap era de verificación, no de build.
3. **`install.ps1` (scripts/):** fallback `v0.4.0` → `gh release view v0.4.0` = **release not found** (404 garantizado). Mensaje menciona `-Version <tag>` que **no existe** en el param block. **No parsea en Windows PowerShell 5.1** (repro: `powershell -NoProfile -File scripts/install.ps1 -DryRun` → `UnexpectedToken '?'` en `:31`, ternary PS7-only) — el one-liner `irm | iex` corre en el shell del usuario (5.1 default en Windows 11) → instalador muerto para el default. `install.sh` es fail-loud (mejor comportamiento de referencia).
4. **Assets esperados (release completo):** 10 binarios (5 targets × archivo + `.sha256`) + 4 wheels (`vantadb_py-<V>-cp311-abi3-{macosx_11_0_arm64,manylinux_2_28_aarch64,manylinux_2_28_x86_64,win_amd64}.whl`). Registries: crates.io `vantadb` · PyPI `vantadb-py` · npm `vantadb` + `vantadb-wasm` (`vantadb-node` = 404 documentado; `vanta-memory` = hold DIST-01 → excluidos).
5. **`vanta-cli --version`** existe (`src/cli.rs:11` — `#[command(version = env!("CARGO_PKG_VERSION"))]`) → smoke del binario verificable.
6. **Decisión assets v0.7.0 (Spec #2):** documentar la ausencia; no backfill. Alternativa documentada como procedimiento manual opcional (owner), no ejecutada (operación remota diferida).
7. **Catch del VERIFY (el hallazgo más fuerte):** el zip real de v0.8.0 es **flat** (`vanta-cli.exe` en la raíz del archivo; verificado con `tar -tf` sobre el asset descargado) — `install.ps1` buscaba `release\vanta-cli.exe` → **el instalador estaba roto end-to-end contra el asset real** (el funnel seguía roto en v0.8.0 por layout, no por 404). Fix: resolución robusta (flat o anidado) en `install.ps1` + smoke de `verify-release.ps1`. Esto es exactamente lo que "verificado contra los assets reales" debía atrapar.
8. **Windows PowerShell 5.1 (dos capas):** (a) ternary PS7-only en `:31` → parse error en todo el script; (b) archivo UTF-8 **sin BOM** → 5.1 lo lee ANSI, los emojis se corrompen y rompen el parse de strings (`-File`). Fix: if/else + BOM UTF-8. Repro antes: `powershell -NoProfile -File scripts/install.ps1 -DryRun` → `UnexpectedToken '?'`; después: exit 0.
9. **Param `-InstallDir`:** PowerShell es case-insensitive (`$installDir` ≡ `$InstallDir`); la primera versión del param quedó clobbered por la asignación del default → fix `if (-not $installDir) { $installDir = "$HOME\.vanta\bin" }`. Verificado con installs reales aislados (7 y 5.1; doble corrida ejercita el backup; `vanta-cli --version` = 0.8.0).

## Evidencia (VERIFY)

| Comando | Resultado |
|---------|-----------|
| `pwsh scripts/verify-release.ps1 -Tag v0.8.0 -Smoke` | ✅ exit 0 — 14/14 assets + 4/4 registries + sha256 verificado + `vanta-cli 0.8.0` |
| `pwsh scripts/verify-release.ps1 -Tag v0.7.0` | ✅ exit 1 — 10 fallos, **solo** en binarios (el resto OK) — detecta el 404 |
| `powershell -NoProfile -File scripts/install.ps1 -DryRun` (5.1) | ✅ exit 0 (antes: parse error) |
| `pwsh -NoProfile -File scripts/install.ps1 -DryRun` (7) | ✅ exit 0 |
| `pwsh scripts/install.ps1 -Version v0.8.0 -NoWizard -InstallDir <tmp>` | ✅ install + backup (2ª corrida) + `vanta-cli 0.8.0` |
| `powershell -NoProfile -File scripts/install.ps1 … -InstallDir <tmp>` | ✅ install + `vanta-cli 0.8.0` |
| `pwsh scripts/install.ps1 -Version 0.8` | ✅ exit 1 (validación) |
| `actionlint .github/workflows/release-verify.yml` | ✅ exit 0 |
| `node scripts/docs/check-links.mjs` / `check-docs.mjs` | ✅ exit 0 |
| `(Get-ChildItem .github/workflows -Filter *.yml).Count` | 38 (= registries actualizados) |

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — resueltas en DISCOVERY (Notes 1-6) |
| Pendientes de ejecución (downhill) | 6 steps |
| % completado | 0% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- **SECURITY:** aplica parcialmente — el installer descarga+ejecuta payload remoto: se preserva el patrón existente (TLS 1.2 + repo oficial + sha256 verify; warn-and-continue si falta el `.sha256`, comportamiento existente documentado). Sin secretos nuevos; `GH_TOKEN` del workflow = `github.token` read-only. No se agrega superficie de red más allá de los checks públicos.
- **PERFORMANCE:** N/A — sin hot paths.

## Steps

### Step 1: `scripts/verify-release.ps1` (nuevo) — verificación real de artefactos

- **Archivos:** `scripts/verify-release.ps1`
- **Acción:** script PS 5.1/7 compatible: resuelve tag (o `-Tag`), valida `vX.Y.Z`; verifica 10 assets binarios + 4 wheels en el GitHub Release; verifica versión live en crates.io/PyPI/npm (4 checks); `-Smoke` descarga el asset de la plataforma actual, verifica sha256 y corre `vanta-cli --version`. Exit 0/1/2. `GH_TOKEN` opcional (rate limit; solo se envía a api.github.com). Known gaps documentados en header.
- **Verify:** `pwsh scripts/verify-release.ps1 -Tag v0.8.0 -Smoke` → exit 0 ALL GREEN · `pwsh scripts/verify-release.ps1 -Tag v0.7.0` → exit 1 (solo binarios)
- **Estado:** ✅ DONE — v0.8.0: 14/14 assets + 4/4 registries + sha256 + `vanta-cli 0.8.0` (exit 0); v0.7.0: exit 1, 10 fallos solo en binarios (detecta el 404). Bug de parse del `.sha256` (byte[] → UTF8) y layout flat corregidos durante VERIFY.

### Step 2: `scripts/install.ps1` — `-Version` + fail-loud + compat 5.1 + layout robusto

- **Archivos:** `scripts/install.ps1`
- **Acción:** `[string]$Version` (validado `^v\d+\.\d+\.\d+$`); `[string]$InstallDir` (override del default, testable); fallback `v0.4.0` eliminado → fail-loud con guía (`-Version`); mensaje de download-failure con URL del release; ternary PS7 (`:31`) → if/else; BOM UTF-8 (5.1 leía ANSI → mojibake rompía el parse); resolución robusta del binario extraído (flat `vanta-cli.exe` o anidado `release\`) — el zip real es flat y el script buscaba `release\` → **instalador roto contra el asset real**.
- **Verify:** `-DryRun` exit 0 en 5.1 y 7 · `-Version` inválido → exit 1 · installs reales aislados (7 y 5.1, doble corrida = backup) + `vanta-cli --version` = 0.8.0
- **Estado:** ✅ DONE — evidencia: dry-runs 5.1/7 exit 0; invalid `-Version 0.8` exit 1; install real `-InstallDir` temp (7: run1 install + run2 backup `.bak-*` + `vanta-cli 0.8.0`; 5.1: install + `vanta-cli 0.8.0`).

### Step 3: `PUBLISH.md` — §Post-release verification (gate de cierre del release)

- **Archivos:** `docs/dev/workflow/PUBLISH.md`
- **Acción:** sección nueva: gate de release (dueño: release driver), paso 1 mecánico (`verify-release.ps1 -Smoke` + alternativa `gh workflow run release-verify.yml`), paso 2 smoke manual en máquina limpia (Windows/Python/npm con comandos exactos), remediación (backfill/re-run), tabla de known gaps (≤v0.7.0 sin binarios; vanta-memory hold; vantadb-node nunca publicado). Chain overview + §Binaries apuntan al gate.
- **Verify:** `node scripts/docs/check-links.mjs` + `check-docs.mjs` → 0
- **Estado:** ✅ DONE — gates exit 0 (links y docs).

### Step 4: `.github/workflows/release-verify.yml` (nuevo) — verificación programada + dispatch

- **Archivos:** `.github/workflows/release-verify.yml`
- **Acción:** schedule `0 8 * * 1` (Mon 08:00, sin solape) + `workflow_dispatch` (input `tag`, vacío = latest); job ubuntu-latest `pwsh scripts/verify-release.ps1 -Smoke`; `permissions: contents: read`; timeout 15; checkout SHA-pinned (RULES.md §2-4, §6).
- **Verify:** `actionlint .github/workflows/release-verify.yml` → 0
- **Estado:** ✅ DONE — actionlint exit 0; 0 tags flotantes; timeout+permissions presentes.

### Step 5: Registries de inventario — README.md / TRIGGERS.md / verified-numbers.md

- **Archivos:** `docs/dev/workflow/README.md`, `docs/dev/workflow/TRIGGERS.md`, `docs/dev/references/verified-numbers.md`
- **Acción:** README: 37→38 + fila en Release (7→8); TRIGGERS: fila en tabla Release + slot Mon 08:00 en Notes + fecha de lectura; verified-numbers: 37→38 + nota "activos" (regla 2 del registry).
- **Verify:** gates docs → 0 · conteo real = 38
- **Estado:** ✅ DONE — `(Get-ChildItem .github/workflows -Filter *.yml).Count` = 38; gates exit 0.

### Step 6: Cierre — OCR + review P2-01 + commit local

- **Archivos:** (cierre)
- **Acción:** `pwsh dev-tools/ocr-review.ps1` (advisory) + review P2-01 por agente distinto (`vanta-review`) + commit local `ci: DIST-05 — …` (NUNCA push).
- **Verify:** OCR sin Critical/High · veredicto reviewer registrado · `git status` limpio post-commit
- **Estado:** ✅ DONE — OCR preview + delegation (rule groups; el hallazgo de inyección del workflow se corrigió pasando el input por `env`); review APPROVE `ses_ef929e5fbffeOOnNT06tUugRjw`; Medium+NIT aplicados y re-verificados; commit local (hash en RESULTADO §7; sin push).

## Dependencias

- F0 — sin dependencias. nextTask: DIST-06.

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED. Tier de paths: Fast (`.github/**`, `scripts/**`, `docs/dev/**`).

- **Revisor:** `vanta-review` (fresco, sesión `ses_ef929e5fbffeOOnNT06tUugRjw` — no participó de la implementación; read-only sobre el worktree sin commit).
- **Enfoque:** diseño de la verificación (¿cubre el gap real?), decisión v0.7.0, corrección de `install.ps1`, scope discipline, RULES.md §2-6.
- **Cómo se probó:** re-ejecutó los 6 comandos del Contrato (v0.8.0 -Smoke ALL GREEN; v0.7.0 solo binarios; dry-runs 5.1/7; installs reales aislados 5.1/7; actionlint 0; gates docs 0) + `tar -tf` del asset real + era v0.7.0 vía `git show`/API.
- **Veredicto:** ✅ **APPROVE** (0 Critical/High). Medium no bloqueante (no ejecutar binario con checksum mismatch + no heredar `GH_TOKEN` al hijo) + NIT (doc de mantenimiento en header) → **aplicados post-verdict** y re-verificado el happy path (`v0.8.0 -Smoke` ALL GREEN). OPTIONALs (fail-closed del `.sha256` faltante; tags pre-release en regex) → candidatos a FIND para el orquestador (fuera de este commit, por indicación del reviewer).
- **Checklist anti-hábitos:** ok (salidas reproducidas; v0.7.0 falla por diseño; seguridad no degradada; SDP declarado).

## Notas

- Plan file y Backlog: NO tocados por instrucción del orquestador (el orquestador sincroniza el plan; la fila Backlog `DIST-05` queda para el cierre de plan/progreso).
- `release-binaries.yml` no se modifica: el gap era de verificación (nadie chequeaba el release), no de build (backfill v0.8.0 verde 5/5).
- **Desviación de scope registrada (no silenciosa):** `campaign_validate_scope` (advisory) marcó `scripts/verify-release.ps1` y `.github/workflows/release-verify.yml` como fuera del blast radius declarado — la lista del plan es derivada del texto (solo archivos existentes). Expansión justificada: contrato (a) exige "checklist **ejecutable**" y el pre-mortem #2 pide "automatizarlo como step de workflow"; sin archivos nuevos la verificación queda como prosa no ejecutable. Decisión en Spec #1.
- **Incidente de verificación (transparencia):** una versión intermedia del param `-InstallDir` (clobbered, ver Notes #9) hizo que dos corridas de prueba instalaran el binario en el `$HOME\.vanta\bin` **real**. Restaurado a estado pre-test: el directorio existía vacío desde 2026-09-26 (sin `vanta-cli.exe`; la run 1 no reportó backup) y se removieron los 3 archivos creados por el test. El resto de los installs de prueba usaron `-InstallDir` aislado en `%TEMP%`.
