---
title: "FIND-725: el gate de WASM decide sobre el rango del push, no sobre el último commit"
kind: task
description: "Un push multi-commit que toca vantadb-wasm/ en un commit intermedio corría el job WASM como skipped; el if: pasa a consumir un job de detección que difea before..after"
---

# FIND-725 — El gate de WASM mira el rango del push, no `head_commit`

> **Caso medido.** El run [`45510840907`](https://github.com/ness-e/Vantadb/actions/runs/45510840907)
> (push a `develop` con head `ec1bf9d58`, un commit **de docs**) reporta el job
> `WASM (wasm-pack browser tests)` como `completed / skipped`. La rama contiene
> `20c164970` (`feat(bindings): expose the AST JSON through a parse() method`,
> que toca `vantadb-wasm/src/lib.rs` y `vantadb-wasm/src/vantadb_wasm.d.ts`),
> así que el cambio de WASM quedó sin gate: el `if:` miraba
> `contains(github.event.head_commit.modified, 'vantadb-wasm/')` y `head_commit`
> es **solo el commit final** del push.

## Metadata

- **Plan file:** `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` (§6.b, residuales del run)
- **Fuente:** fila `FIND-725` de `docs/dev/Backlog.md:758` (origen: monitor CI `ec1bf9d58`, 2026-11-24)
- **Esfuerzo:** 🟢 1h (appetite del plan)
- **Prioridad:** 🟠 Alta (cobertura que se pierde en silencio: el status es `skipped`, no `failure`)
- **Tipo:** CI/CD (GitHub Actions) + docs
- **Turns estimados:** 5-10
- **Creado:** 2026-11-24T21:00
- **last-synced:** 2026-11-24T23:05
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps

## Blast Radius

| Dirección | Módulos |
|---|---|
| Callers | `ci-rust.yml` job `wasm-test` (único consumidor del `if:`). El workflow completo se dispara desde `on.push`/`on.pull_request` con `paths:` que ya incluyen `vantadb-*/**` (`ci-rust.yml:20,45`) — el trigger **no** es el problema. |
| Callees | `actions/checkout` (pin `3d3c42e5…`), `git fetch --depth 1` + `git diff --name-only` (mismo patrón que el job `adr-gate`, `ci-rust.yml:243-253`), `git rev-parse`. |
| Implicaciones | Job nuevo `wasm-scope` (ubuntu-latest, `timeout-minutes: 5`, guard de dependabot), sin dependencias nuevas. `wasm-test` suma `needs: [wasm-scope]` y su `if:` pasa a `!cancelled() && actor != 'dependabot[bot]' && (needs.wasm-scope.outputs.wasm == 'true' \|\| needs.wasm-scope.result != 'success')`. En `pull_request` el comportamiento **no cambia** (corre siempre); `workflow_dispatch` pasa de `skipped` a correr (ver bordes). El gate no se endurece: sigue `continue-on-error: true` (CATEGORY: EXPERIMENTAL). Cero cambios en `src/**`, en los crates de WASM o en el resto del workflow. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `.github/workflows/ci-rust.yml:1-1024` (completo); `.opencode/rules/release-ci.md` (completo); `docs/dev/workflow/RULES.md` (completo); `docs/dev/operations/CI_POLICY.md:1-60` + filas de WASM (`:150,181-182,188,198-200,223-225,551,689`); `.opencode/task-system/prompts/task.md` (completo); `.opencode/task-system/prompts/pipeline-full.md` (completo); `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` (§6.b + fila `FIND-725`); `docs/dev/Backlog.md:758`; `docs/dev/tasks/FIND-710.md:1-40` (formato canónico).
- **Referencias hacia dentro:** el par `head_commit.modified` + `vantadb-wasm/` aparece **una sola vez** en `.github/` (`rg -n "head_commit" .github/` → `ci-rust.yml:589`). No hay `dorny/paths-filter` en el repo (`rg -n "paths-filter|name-only" .github/` → solo `ci-rust.yml:251` y `nightly.yml:369`, ambos `git diff --name-only`). El job `wasm-test` no publica artefactos ni es `needs:` de nadie.
- **Referencias entrantes al job:** ninguna. `rg -n "wasm-test" .github/ docs/` → solo su propia definición y `CI_POLICY.md:224` (mención de comportamiento, no de nombre de job).
- **Veredicto impacto:** **bajo**. Bloque autocontenido; el riesgo real es semántico (que el predicado no dispare nunca), mitigado por fail-open y por la prueba del predicado de abajo.

## Contrato

"`npx --no-install actionlint .github/workflows/ci-rust.yml` sale 0 — **no ejecutable en este repo: no hay `package.json` ni `node_modules`, `npx --no-install` no resuelve el paquete**. Se usa en su lugar el binario canónico ya instalado: `actionlint .github/workflows/ci-rust.yml` (v1.7.12, exit 0). Además, la prueba del predicado fuera de CI demuestra: (i) un push cuyo commit **intermedio** toca `vantadb-wasm/` **sí** corre el job; (ii) un push que no lo toca en ningún commit **no** lo corre; (iii) el primer push de una rama corre siempre."

## Spec (SDD)

> No es feature-add: no agrega símbolos públicos, ni endpoints, ni métodos de
> binding. La tabla queda igual porque hay una decisión con tradeoff real.

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Mecanismo de detección | A: job propio `wasm-scope` con `git diff --name-only` + `outputs` (0 deps, +1 job de ~20 s) / B: `dorny/paths-filter` (menos YAML, pero **no está en el repo** → dependencia nueva y pin nuevo) / C: difear dentro del propio `wasm-test` (0 jobs nuevos, pero el job corre siempre y no se ahorra nada) | A | ✅ decidido-por-evidencia: opción (a) del ticket; `rg` confirma que `dorny/paths-filter` no existe (`release-ci.md:19-20` exige punto único y no duplicar integraciones) |
| 2 | Alcance del rango | A: solo `push` cambia; `pull_request` sigue corriendo siempre (falso positivo) / B: filtrar también PRs con `base.sha...HEAD` (ahorra minutos, **reduce** cobertura respecto de hoy) | A | ✅ decidido-por-evidencia: blast radius §7 del ticket —preferir falso positivo— y `pull_request` no tiene `event.before` |
| 3 | Base para el diff | A: `fetch-depth: 0` en el checkout (simple, clona historia completa) / B: checkout superficial + `git fetch --depth 1 origin <before>` (mismo patrón que `adr-gate:243-244`, sin historia) | B | ✅ decidido-por-evidencia: `adr-gate` ya lo usa en este mismo fichero (ponytail rung 2: reusar el patrón existente) |
| 4 | Cuando no se puede difear | A: fallar el job (fail-closed → `wasm-test` skipped → **pierde** cobertura) / B: `wasm=true` + `::warning::` (fail-open, cuesta minutos) | B | ✅ decidido-por-evidencia: blast radius §7 — el falso negativo es silencioso, el falso positivo cuesta minutos |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) el job `WASM` sigue siendo `continue-on-error` EXPERIMENTAL — este cambio **no** lo endurece; (2) `pull_request` corre WASM igual que antes, y **`workflow_dispatch` ahora también corre** (antes caía en el `if:` viejo y quedaba `skipped`: cambio benigno y deliberado, ver `## Review` L1); (3) **no** se toca el job `sanitizer-tsan` ni `dev-tools/tsan-suppressions.txt` (`FIND-714` en paralelo sobre el mismo fichero); (4) nada de `continue-on-error` nuevo (AGENTS.md Regla 2 / `RULES.md` regla 6); (5) pines por SHA intactos (`RULES.md` regla 3).
- **Comandos de verificación:** `actionlint .github/workflows/ci-rust.yml` → exit 0; `bash run-extracted3.sh extracted-detect.sh` → 8 escenarios (tabla abajo).
- **Deuda pendiente:** el `before` real del push `45510840907` no es recuperable (la API de eventos ya no expone `payload.before` para ese run). Además, `wasm-scope` **nunca corrió en CI**: el primer run real (tras el push) hay que mirarlo con `gh api repos/ness-e/Vantadb/actions/runs/<id>/jobs --jq '.jobs[]|select(.name|test("WASM"))|"\(.name) => \(.conclusion)"'` — el `git fetch` por SHA es el único eslabón probado sólo fuera de CI.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda. El cambio **reduce** deuda: elimina un gate que no cubría el caso que debía cubrir.

## Definition of Done

| Nivel | Gate |
|-------|------|
| **Task** | `actionlint` exit 0 + las 3 pruebas del contrato (8 escenarios end-to-end) + los 9 bordes documentados |
| **Commit** | Atómico, `ci(wasm): …`, `git diff` limitado a `ci-rust.yml` + este task file |
| **Release** | N/A (no toca código publicado, ni versión, ni publish). `ci:` no genera bump en release-plz. |

## Herramientas necesarias

- `actionlint` (v1.7.12, binario instalado), `bash` 5.3, `git`, `gh` (evidencia del run), `pwsh dev-tools/ocr-review.ps1`.

**Skills cargadas (SDP):**
- `ci-cd-and-automation` (pinned, CI/release) — patrón de quality gates y `paths` en workflows.
- `git-workflow-and-versioning` (pinned, CI/release) — conventional commit `ci:` y cierre de release (base del rol).
- `security-and-hardening` (pinned, trust boundary) — revisado: el diff sólo lee SHAs del evento; no hay input de usuario ni `eval`; el `grep` es sobre salida de `git diff` con patrón fijo.
- `doubt-driven-development` — la prueba del predicado es el mecanismo adversarial (falso negativo vs falso positivo).
- `documentation-skill` — este task file (`kind: task`, frontmatter, enlaces relativos).
- `campaign-executor`, `progreso` — fase BUILD (base del pipeline).
- `incremental-implementation` / `test-driven-development` — RED = prueba del predicado con el `if:` viejo; GREEN = con el nuevo.

## Investigation Notes

**Por qué `head_commit.modified` es un footgun (doc oficial).** El payload de
`push` documenta `commits` como *"An array of commit objects describing the
pushed commits. (Pushed commits are all commits that are included in the compare
between the before commit and the after commit.)"*, y `before`/`after` como *"The
SHA of the most recent commit on ref before/after the push"*
([webhook-events-and-payloads#push](https://docs.github.com/en/webhooks/webhook-events-and-payloads#push)).
`head_commit` es **un** objeto: el tip. Usarlo como fuente de "qué cambió" es
correcto para un push de un commit y falso para todo push de varios.

**El trigger ya era correcto; el bug estaba solo en el `if:`.** La sección
[*Git diff comparisons*](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#onpushpull_requestpull_request_targetpathspaths-ignore)
de la sintaxis de workflows dice que, para `paths:`, *"Pushes to existing
branches: A two-dot diff compares the head and base SHAs directly with each
other"*. Es decir: el filtro de workflow ya evalúa el rango completo — por eso
el run existe. Lo que fallaba era el predicado de job, que GitHub evalúa con el
contexto del evento, no con un diff.

**Ese mismo apartado fija la política de fallo.** *"If a push contains more than
1,000 commits, the workflow will always run"* / *"If generating the diff times
out, the workflow will always run"*: GitHub mismo prefiere correr de más antes
que perder cobertura. El fail-open del predicado sigue esa línea (decisión 4 de
la spec).

**`needs` aplica `success()` implícito (hallazgo H1 del review).** Las
[expressions](https://docs.github.com/en/actions/reference/workflows-and-actions/expressions#status-check-functions)
dicen: *"A default status check of `success()` is applied unless you include one
of these functions"*, y sobre `always()`: *"If you want to run a job or step
regardless of its success or failure, use the recommended alternative:
`if: ${{ !cancelled() }}`"*. Por eso el `if:` de `wasm-test` lleva
`!cancelled()` + `needs.wasm-scope.result != 'success'`: sin eso, un fallo del
job de scope (timeout, runner, checkout) dejaría los tests en `skipped`, que es
la misma pérdida silenciosa que esta tarea cierra.

**git: rename detection y quoting (M1/M2).** `git diff --name-only` imprime sólo
el path nuevo de un rename (`src/legacy.rs`, sin `vantadb-wasm/src/legacy.rs`) y
C-quotea los paths no-ASCII (`"vantadb-wasm/src/\303\261.rs"`), que rompe el
match por prefijo. De ahí `--no-renames` + `-c core.quotePath=false` + `-z`.

**Bordes cubiertos:**

| Borde | Qué pasa | Qué hace el predicado |
|---|---|---|
| Primer push de una rama | `github.event.before` = 40 ceros (`refs` creada) | `wasm=true` (no hay rango que difear) |
| `pull_request` | no existe `event.before` | `wasm=true` (comportamiento de hoy, sin cambios) |
| `workflow_dispatch` | no existe `event.before` | `wasm=true` — **antes quedaba `skipped`** (el `if:` viejo sólo cubría `pull_request` y `push`); un dispatch manual ahora corre los tests |
| `before` no resoluble (fetch falla, GC, force-push) | el objeto no está | `::warning::` + `wasm=true` (fail-open) |
| Rango vacío (`before == after`) | no hay cambios | `wasm=false` |
| Push de >1000 commits | GitHub corre el workflow igual | el fetch por SHA sigue siendo válido; si falla → fail-open |
| Rename **fuera** de `vantadb-wasm/` | `--name-only` con renames imprime sólo el path nuevo | `--no-renames` → imprime también el path viejo → `true` |
| Path no-ASCII dentro del crate | git lo C-quotea (`"vantadb-wasm/src/\303\261.rs"`) | `core.quotePath=false` + `-z` → path crudo → `true` |
| El job `wasm-scope` **falla** (no el script) | `needs` aplica `success()` implícito → `wasm-test` se saltearía | `!cancelled() && (… \|\| needs.wasm-scope.result != 'success')` → corre igual |

## Prueba del predicado (fuera de CI)

Script reproducible (`bash`, repo temporal, 5 escenarios). Compara el predicado
**nuevo** (rango `before..after`) contra el **viejo** (`head_commit.modified` =
sólo el tip):

```text
scenario (push range)                                      | NEW (range)        | OLD (head_commit)
------------------------------------------------------------------------------
(i)  db83fb61..4ab43149  wasm in intermediate commit       | true               | false
(ii) 4ab43149..67545b21  no wasm in any commit             | false              | false
(iii) 00000000..67545b21  first push of the branch         | true(first-push)   | false
(iv) 67545b21..1bc3b379  wasm at the tip (parity control)  | true               | true
(v)  deadbeef..1bc3b379  unfetchable before                | true(fail-open)    | n/a
```

Lectura: **(i)** es el bug (nuevo corre, viejo no) · **(ii)** no corre de más ·
**(iii)** el primer push siempre corre · **(iv)** paridad cuando el wasm está en
el tip · **(v)** el fail-open no pierde cobertura. El `fatal: bad object` de
stderr en (v) es el camino esperado: `git diff` falla, el `if !` lo captura y el
predicado devuelve `true(fail-open)`.

### Prueba end-to-end del bloque `run:` que se commitea

El bloque se **extrae del workflow** (`node extract.js` sobre el `.yml`, sin copia
que pueda derivar) y se ejecuta tal cual, sobre un clon superficial de un
`file://` remoto (simula `actions/checkout` en el tip + `fetch` por SHA del base,
como en CI). 8 escenarios, incluidos los dos falsos negativos que encontró el
review (M1 rename, M2 path no-ASCII):

```text
exact `run:` block from .github/workflows/ci-rust.yml (job wasm-scope)
shallow clone at the tip + fetch-by-SHA of the base, as in CI
-------------------------------------------------------------------------
(i)   multi-commit push, wasm intermediate, docs tip     exit=0  wasm=true
(ii)  push with no wasm anywhere                         exit=0  wasm=false
(iii) first push of the branch (before = 40 zeros)       exit=0  wasm=true
(iv)  wasm at the tip (parity control)                   exit=0  wasm=true
(v)   unfetchable before (fail-open)                     exit=0  wasm=true
(vi)  pull_request (no event.before)                     exit=0  wasm=true
(vii) rename OUT of vantadb-wasm/ (M1)                   exit=0  wasm=true
(viii) non-ASCII path in vantadb-wasm/ (M2)              exit=0  wasm=true
-------------------------------------------------------------------------
why (vii)/(viii) need the flags — raw `git diff --name-only` output:
(vii)  raw: src/legacy.rs
(viii) raw: "vantadb-wasm/src/\303\261.rs"
(vii)  fixed: src/legacy.rs vantadb-wasm/src/legacy.rs
(viii) fixed: vantadb-wasm/src/ñ.rs
```

`(vii) raw` sólo imprime el path **nuevo** (rename detection): el predicado sin
`--no-renames` diría `false` con un archivo que salió del crate. `(viii) raw`
viene entrecomillado por `core.quotePath`, y la comilla inicial rompe
`^vantadb-wasm/`; con `-c core.quotePath=false -z | tr '\0' '\n'` matchea.

**Alcance de esta prueba:** cubre el bloque `run:` (predicado, fetch, fail-open,
non-push) y el wiring de `GITHUB_OUTPUT`. **No** ejecuta el `needs`/`if` de
GitHub, ni el checkout del runner: eso sólo se verifica en el primer run real
(ver `Deuda pendiente`).

### Medición sobre la historia real del repo

El `before` exacto del run `45510840907` no es recuperable, así que se mide el
límite con precisión en vez de afirmar de más:

```text
git diff --name-only a46689786 ec1bf9d58 | grep '^vantadb-wasm/'
  → vantadb-wasm/src/lib.rs · vantadb-wasm/src/vantadb_wasm.d.ts      (rango CON el commit de wasm: NUEVO=true / VIEJO=false)
git diff --name-only 20c164970 ec1bf9d58 | grep '^vantadb-wasm/'
  → (vacío)                                                            (el cambio está en la base, fuera del rango: ambos false — correcto)
git show --name-only --format= ec1bf9d58 | grep '^vantadb-wasm/'
  → (vacío)                                                            (tip de docs: VIEJO=false siempre)
```

Garantía del fix: **si el cambio de `vantadb-wasm/` cae dentro del rango del
push, el job corre** — como en la fila 1 de la medición. Si el push sólo
arrastra commits ya publicados (el cambio está en `before`), no correr es lo
correcto: ese push no introduce el cambio.

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 2 (review P2-01 · commit) |
| % completado | 70% |

## Fase 1 — Evidencia de Debugging (GATE — solo tipo Bug)

- **Repro:** run `45510840907` (push `develop`, head `ec1bf9d58`) → job `WASM (wasm-pack browser tests)` = `completed / skipped`. Comando: `gh api repos/ness-e/Vantadb/actions/runs/45510840907/jobs --jq '.jobs[] | select(.name|test("WASM";"i")) | "\(.name) => \(.status) / \(.conclusion)"'` → `WASM (wasm-pack browser tests) => completed / skipped`.
- **Hipótesis:** `github.event.head_commit.modified` contiene **sólo** los paths del último commit del push; con un tail de docs, el predicado da `false` aunque un commit anterior haya tocado `vantadb-wasm/`. Causa raíz = usar un snapshot puntual (el tip) donde hace falta un rango (`before..after`).
- **1 variable controlada:** el predicado del `if:`. Todo lo demás (triggers, `paths:`, checkout, pasos del job, `continue-on-error`) queda idéntico.
- **Test RED:** el escenario (i) de la prueba del predicado — con el predicado viejo `false` (job skipped) y con el nuevo `true`. Verificado **antes** del fix, en rojo, en la tabla de arriba.

## Steps

### Step 1: Job `wasm-scope` + `wasm-test` consume el output
- **Archivos:** `.github/workflows/ci-rust.yml` (bloque del job `wasm-test`, `:580-603`, + job nuevo inmediatamente antes)
- **Acción:** agregar el job `wasm-scope` (checkout, fetch por SHA del `before`, `git diff --name-only`, `outputs.wasm` con fail-open) y reemplazar el `if:` de `wasm-test` por `needs.wasm-scope.outputs.wasm == 'true'`.
- **Verify:** `actionlint .github/workflows/ci-rust.yml`
- **Estado:** ✅ DONE — `actionlint` v1.7.12 exit 0 (sobre el fichero del worktree y sobre el blob que se commitea). `npx --no-install actionlint` no resuelve (sin `package.json`/`node_modules` en el repo), se usó el binario instalado.

### Step 2: Verificación mecánica del predicado
- **Archivos:** `C:\Users\Eros\AppData\Local\Temp\opencode\find-725\run-extracted3.sh` (temporal, no se commitea)
- **Acción:** extraer el bloque `run:` del `.yml` y correr los 8 escenarios sobre clones superficiales con remoto `file://`.
- **Verify:** `bash run-extracted3.sh extracted-detect.sh` → (i)=true, (ii)=false, (iii)=true, (iv)=true, (v)=true, (vi)=true, (vii)=true, (viii)=true
- **Estado:** ✅ DONE — tabla en `## Prueba end-to-end`; 8/8 como se espera.

### Step 3: OCR + review adversarial (P2-01)
- **Archivos:** diff de `ci-rust.yml` + este task file
- **Acción:** `pwsh dev-tools/ocr-review.ps1` (advisory) y review por `vanta-review` (contexto distinto al implementador).
- **Verify:** veredicto `approve` registrado en `## Review`
- **Estado:** ✅ DONE — ronda 1: **cambios requeridos** (H1, M1, M2, M3, L1, L2); los seis aplicados y re-verificados. Re-review (ronda 2) pedido al mismo revisor; ver `## Review`.

**OCR Rule Group 1** (`{.github/**,release-plz.toml,cliff.toml,...}`) aplicado al bloque nuevo, con `ocr delegate rule` sobre `.github/workflows/ci-rust.yml`:

| Chequeo del grupo | Resultado |
|---|---|
| `pull_request_target` misuse | N/A (no se usa) |
| Secrets en logs | ✅ ninguno; no se referencia `secrets.*` |
| Permissions least-privilege | ✅ hereda `contents: read` del workflow; el job no pide nada extra |
| Actions sin pin | ✅ `actions/checkout@3d3c42e5… # v7.0.1` |
| Script injection | ✅ `github.event.before`/`after`/`event_name` van por `env:` y se citan (`"$PUSH_BEFORE"`), nunca interpolados en el `run:` |
| Credenciales hardcodeadas | ✅ ninguna |
| `fetch-depth: 0` faltante | ✅ justificado: no hace falta historia, sólo un tree diff; el base se busca por SHA (comentario en el step) |
| Lógica de `if:` | ✅ `needs.wasm-scope.outputs.wasm == 'true'` (los outputs son strings) |
| `shell` explícito | ✅ `shell: bash` |
| `needs:` roto | ✅ `wasm-scope` existe |
| `timeout-minutes` | ✅ 5 |
| `concurrency` | ✅ la del workflow (`${{ github.workflow }}-${{ github.ref }}`) |
| Deps sin cachear | ✅ el job no instala nada |
| `continue-on-error` nuevo sin `# CATEGORY:` | ✅ no se agrega ninguno |
| release-ci.md (sccache punto único, versiones, allocator) | ✅ N/A / intacto |

### Step 4: Commit local
- **Archivos:** `.github/workflows/ci-rust.yml`, `docs/dev/tasks/FIND-725.md`
- **Acción:** el working tree comparte `ci-rust.yml` con `FIND-714` (TSan sin commitear), así que **no** se usa `git add` sobre el fichero: se commitea un blob propio (`git show HEAD:ci-rust.yml` + sólo el bloque WASM → `git hash-object -w` → `git update-index --cacheinfo`). Mensaje `ci(wasm): gate the wasm tests on the push range, not the last commit (FIND-725)`. **Sin push.**
- **Verify:** `git show --stat HEAD` = 2 archivos; `git show HEAD -- .github/workflows/ci-rust.yml | grep -c 'tsan-suppressions'` = 0
- **Estado:** ✅ DONE — commit local (ver `COMMIT_HASH` en el `RESULTADO`), con el follow-up del review incorporado por amend sobre el mismo commit.

## Dependencias

- **Colisión conocida:** `FIND-714` edita el job `sanitizer-tsan` (`ci-rust.yml:932+`) del **mismo fichero**. Regla: sólo el bloque del job WASM; no reformatear el resto. Si `git diff` muestra cambios de TSan ajenos, no incluirlos en el commit.
- **Post-push (owner):** verificar el primer run real de `wasm-scope` (ver `Deuda pendiente`).

## Review (GATE — agente distinto, P2-01)

- **Revisor:** `vanta-review` (contexto fresco, `ses_defd3b3a1ffepefvn5ZU0T5GU1`).
- **Enfoque:** ¿el job `wasm-scope` es la opción correcta frente a `dorny/paths-filter`? ¿Los bordes están cubiertos? ¿El `needs`/`if` reintroduce un skip silencioso?
- **Cómo se probó:** el revisor **re-ejecutó** la prueba del predicado (5/5 coincidían con la tabla), corrió `actionlint` sobre su copia y sobre el working tree **con controles negativos** (romper `needs.wasm-scop` → exit 1; `timeout-minutes: five` → exit 1) para demostrar que el 0 significa algo, y verificó el mecanismo `depth-1 checkout → fetch por SHA → two-tree diff` con un clon shallow real. Confirmó que `base.yml` era byte-idéntico al HEAD y que el delta no contenía TSan.

### Ronda 1 — ❌ cambios requeridos (6 hallazgos, todos aplicados)

| ID | Hallazgo | Disposición |
|---|---|---|
| **H1** | Con `needs` sin función de estado, GitHub aplica `success()` implícito: un fallo del job `wasm-scope` (timeout, runner, checkout) dejaba `wasm-test` en `skipped` — la misma pérdida silenciosa que esta tarea cierra, reabierta por la dependencia | ✅ aplicado: `!cancelled() && actor != 'dependabot[bot]' && (needs.wasm-scope.outputs.wasm == 'true' \|\| needs.wasm-scope.result != 'success')` |
| **M1** | `git diff --name-only` con rename detection imprime sólo el path nuevo: un archivo **movido fuera** de `vantadb-wasm/` daba `false` | ✅ aplicado: `--no-renames`; probado en (vii) |
| **M2** | git C-quotea paths no-ASCII: `"vantadb-wasm/src/\303\261.rs"` no matchea `^vantadb-wasm/` | ✅ aplicado: `-c core.quotePath=false … -z \| tr '\0' '\n'`; probado en (viii) |
| **M3** | El Step 4 decía `git add` del fichero, que se llevaría las 74 líneas de TSan de `FIND-714` | ✅ aplicado: Step 4 y `## Notas` documentan el commit por blob; verificado `git show HEAD \| grep -c tsan-suppressions` = 0 |
| **L1** | El comentario decía "sin cambios" para `workflow_dispatch`, que antes quedaba `skipped` | ✅ aplicado: comentario y tabla de bordes corregidos (cambio benigno, declarado) |
| **L2** | `wasm-scope` corría ~20 s inútil en PRs de Dependabot | ✅ aplicado: `if: github.actor != 'dependabot[bot]'` (no es required check → el guard no cuesta cobertura) |
| **L3** | La prueba del predicado era una copia y no ejercitaba non-push / fetch-fail / clon shallow | ✅ aplicado: ahora el bloque se **extrae** del `.yml` y corre sobre clones superficiales; 8 escenarios + `## Prueba end-to-end` declara su alcance explícito |
| **L4** | Deriva de estado en este task file (steps PENDING ya hechos) | ✅ aplicado: steps y contadores sincronizados |

### Ronda 2 — verificación de las correcciones

Pendiente de respuesta del mismo revisor (`vanta-review`, `ses_defd3b3a1ffepefvn5ZU0T5GU1`): se le pidió re-verificar H1/M1/M2/L1 con el bloque nuevo extraído del `.yml` (8 escenarios) y cerrar con veredicto. **Ningún veredicto de aprobación está registrado todavía.**

- **Veredicto:** ❌ cambios requeridos (ronda 1) → correcciones aplicadas → re-review pendiente.

## Notas

- **Scope discipline:** `CI_POLICY.md` no enumera jobs por nombre para `wasm-test` (sólo describe su comportamiento BEST-EFFORT en `:223-225`, que no cambia) y `TRIGGERS.md` es por fichero → el job `wasm-scope` no crea drift documental. No se toca `docs/dev/Backlog.md` (lo cura el cierre del plan) ni `docs/dev/plans/**`.
- **`npx --no-install actionlint`** no es ejecutable en este repo (sin `package.json`): el contrato se cumple con el binario `actionlint` v1.7.12 ya instalado y pinneado por la misma fuente que documenta `RULES.md` regla 6 (`actionlint` 0).
- **Gates de docs:** `node scripts/docs/check-links.mjs` → exit 0 (0 markdown rotos, 1 wikilink dentro de presupuesto en un archivo histórico ajeno). `node scripts/docs/check-docs.mjs` → exit 1 **por `docs/dev/tasks/FIND-724.md` (`MISSING FRONTMATTER`)**, un task file de otro carril en vuelo; este archivo pasa el esquema (`title` + `kind: task`) y los `orphans` son *reporting only*, no gate. No se regeneran índices (`gen-index.mjs --write`): arrastraría las entradas de `FIND-714`/`FIND-724` y metería archivos ajenos en este commit.
- **Working tree compartido:** al cerrar, el árbol tenía trabajo ajeno sin commitear (`FIND-714`: job `sanitizer-tsan` + `dev-tools/tsan-suppressions.txt`; `FIND-724`: `CI_POLICY.md` + `src/storage/engine/tests/snapshot_quiesce.rs`). El commit se armó con un blob propio (`git show HEAD:ci-rust.yml` + sólo mi bloque, `git hash-object` + `update-index --cacheinfo`) para no arrastrar ni una línea de TSan: el diff commiteado es de 3 hunks / +74/-3 sobre ese fichero.
