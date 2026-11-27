---
title: "FIND-787 - un required check `skipped` cerraba el candado de `develop` sin ejecutar un test"
kind: task
status: completed
description: "En el push `43e41f055` los tres `Tests (Linux/Windows/macOS)` quedaron `skipped` (`steps: []`) porque `needs: secret-scan` cayó, y GitHub cuenta `skipped` como required satisfecho. Se cierra con un job guard (`if: always()`) que falla nombrando el job cuando una suite required no se ejecutó, y con el único `skipped` legítimo declarado: el que `rust-scope` midió sobre el diff."
tags: [vantadb, ci, github-actions, ruleset, required-checks, skipped]
---

# FIND-787 - `skipped` no es un test

> **Dependencias:** `FIND-765` / `FIND-770` (el `Expected` eterno de un workflow que no
> dispara) y `FIND-786` (los 9 FP de gitleaks que dispararon el `skipped` medido). `FIND-791`
> fija la restricción de diseño: un required que no corre en PR es el deadlock.
> **Carril:** CI, sin compilar Rust. Escritura: `.github/workflows/ci-rust.yml` (exclusivo),
> este task file y `docs/dev/avance/activo/ci-cd.md`. **`docs/dev/Backlog.md` queda intacto:** la fila
> `FIND-787` se **conserva** (ver Review M2) y el fichero, además, está siendo barrido por otro carril
> en este mismo worktree — tocarlo hornearía su trabajo (`FIND-792`).
> **No se toca el ruleset**: el cambio va propuesto, no aplicado (decisión del owner).

## Contexto

`43e41f055`, medido con `gh api`, no heredado de la fila:

```bash
gh api "repos/ness-e/Vantadb/commits/43e41f055/check-runs?per_page=100&filter=latest" \
  --jq '.total_count, ([.check_runs[]|select(.conclusion=="skipped")|.name]|join(" | "))'
# 61
# Tests (Windows) | Tests (Linux) | Tests (embeddings features) | Tests (macOS) |
# Code Coverage | WASM | Semver Checks | Public API Snapshot | ADR Gate | Adapter compat (...)

gh api "repos/ness-e/Vantadb/commits/43e41f055/check-runs?per_page=100&filter=latest" \
  --jq '.check_runs[]|select(.name=="Tests (Linux)")|.details_url'
# https://github.com/ness-e/Vantadb/actions/runs/45896745384/job/140178224761

gh api "repos/ness-e/Vantadb/actions/runs/45896745384/jobs?per_page=100" --jq \
  '.jobs[]|select(.name|startswith("Tests"))|
   "\(.name)\t\(.conclusion)\tsteps=\((.steps // [])|length)\tstarted=\(.started_at)\tcompleted=\(.completed_at)\trunner=\(.runner_name)"'
# Tests (macOS)               skipped  steps=0  started=21:29:32Z  completed=21:29:31Z  runner=null
# Tests (embeddings features) skipped  steps=0  started=21:29:32Z  completed=21:29:31Z  runner=null
# Tests (Linux)               skipped  steps=0  started=21:29:32Z  completed=21:29:31Z  runner=null
# Tests (Windows)             skipped  steps=0  started=21:29:32Z  completed=21:29:31Z  runner=null
```

Tres firmas a la vez: **`steps=0`** (no ejecutó ni el `checkout`), **`runner=null`** (ninguno asignado) y
`started_at` **posterior** a `completed_at` (el "started" es cuando GitHub lo resolvió, el "completed"
cuando lo descartó). Ninguno de los cuatro vio un runner. Y `skipped` **satisface** un required check,
textual y verificado hoy en `docs.github.com/en/pull-requests/reference/status-checks`: *"A job that is
skipped will report its status as 'Success'. It will not prevent a pull request from merging, even if
it is a required check."*

En ese SHA hay **dos** `failure` reales, no uno: `Secret Scan (gitleaks + secretlint)` y
`Linux Snapshot Certification` (runner real, 3m42s, job `140172408684`). Sólo el primero está en los
`needs` de los tests, así que el diagnóstico causal no cambia — pero son dos, y el segundo es la fila
de `FIND-791` (una lane que no es required, corriendo y fallando sin bloquear nada).

**Los 14 contexts required del ruleset `23692587` (`develop`)** — leído, no modificado:

```bash
gh api repos/:owner/:repo/rulesets/23692587 \
  --jq '.rules[]|select(.type=="required_status_checks")|.parameters.required_status_checks[].context'
# Format Check · Clippy Lints · Tests (Linux) · Tests (Windows) · Tests (macOS)
# MSRV Check (1.95.0) · Experimental Crates Check · Security Audit · Miri (UB Detection)
# Dependency Policy Check · Internal links resolve · Frontmatter schema, kind agreement, orphans
# Generated indexes are current · Generate API reference (rustdoc)
# strict_required_status_checks_policy: true
```

`Secret Scan` **no** está. O sea que el rojo que arrastró a los tres tests no bloqueaba nada por sí
mismo: su único efecto fue **apagar tres required checks** y que el candado se cerrara en verde.

### Las vías por las que un requerido `skipped` se cierra

| Vía | Por qué ocurre | ¿Bloqueaba algo? |
|---|---|---|
| `needs: [fmt, clippy, secret-scan]` | cualquiera de los tres falla u se omite | `fmt`/`clippy` son required y ya están rojos; **`secret-scan` no** → nada |
| `if: github.actor != 'dependabot[bot]'` | PR de dependabot | nada (diseño de `FIND-472`) |
| `needs: clippy` (cadena de `rust-scope`) | `rust-scope` midió que el PR no toca Rust | nada, y es la decisión que se quiere conservar |

La primera fila es la que se materializó. La tercera es una **decisión medida** (un `git diff`
real sobre el rango del PR): no es una omisión y no debe pagar 3×90 min de runner.

## Contrato

Los contexts required no pueden ser `skipped` sin que alguien lo declare. Dos caminos en la fila:

- **(A)** cada job que se saltaría publica un check que falla nombrando el motivo.
- **(B)** un check adicional que sólo se publique cuando la suite ejecutó de verdad.

**Elegido: (A).** (B) tal como está escrito es el deadlock que `FIND-765` midió: un context required
que no se publica queda `Pending` para siempre, y `FIND-791` lo confirma por el otro lado (la lane
`Linux Snapshot Certification` no corre en PR: por eso no es required). Un required check **tiene**
que reportar en cada run; lo único elegible es que el reporte del caso "no se ejecutó" sea **rojo**.
Eso es (A), y su forma mínima es **un** job guard, no un check por job.

## Impacto mapeado (Regla 0)

**Leídos completos:** `.github/workflows/ci-rust.yml` (1157 líneas), `.github/workflows/ci-gate.yml`
(el patrón de auditoría de checks ya existente en el repo), `.github/workflows/docs-gates.yml`
(§FIND-765, la cita de la doc de GitHub sobre `skipped`), `.opencode/rules/release-ci.md`.

**Referencias hacia dentro (qué consume lo que voy a tocar):**

| Consumidor | Efecto |
|---|---|
| ruleset `23692587` (`develop`) | matchea check-runs **por nombre**. Un job nuevo aporta un nombre nuevo; **no** altera los 14 existentes |
| `.github/workflows/ci-gate.yml` (`REQUIRED`, 19 nombres) | espeja el ruleset de **main**; un job nuevo en `ci-rust` que no esté en esa lista no lo rompe (sólo audita nombres listados) |
| `docs/dev/operations/CI_POLICY.md` | política de tiers; un guard de 5 min es Fast Gate |
| `.githooks/pre-push` | corre `actionlint` si cambian workflows — el gate que puede colgarse (`FIND-785`) |

**Referencias hacia afuera:** ninguna. El job no usa `needs` de otros workflows, no publica artifacts,
no escribe outputs que alguien consuma.

**Veredicto de impacto:** aditivo y aislado. Un job nuevo (26 líneas de `run:`: 18 sentencias, 4
comentarios y 4 líneas en blanco), cero cambios en los jobs existentes, cero cambios en nombres de
contexts existentes. El riesgo real no es el YAML: es que un required nuevo **no reporte**, y eso se
ataca garantizando que el guard corre siempre (ver Spec D3).

Dos consumidores que el review P2-01 agregó a este mapa y que no estaban: **`Generated indexes are
current`** (context required #13, `docs-gates.yml:300` corre `gen-index --check`) queda **rojo por el
doc nuevo** y **no se puede arreglar desde este carril** — ver "Deuda" C1; y `on.push.paths` de este
mismo fichero, que en un push sólo-docs no produce run (L5, ver Deuda).

## Spec - decisiones

| # | Decisión | Elegido | Alternativa descartada | Por qué |
|---|---|---|---|---|
| D1 | Caminos de la fila | **(A) guard** | (B) check que sólo se publica al ejecutar | (B) deja `Pending` eterno = `FIND-765`; un required no puede elegir *si* reporta |
| D2 | Alcance del guard | **un job que mira los 3** required | un check propio por job | un solo `run:` de 6 sentencias top-level (18 contando el cuerpo de los compuestos) vs tres duplicados; un solo context nuevo para el owner |
| D3 | Cómo se garantiza que el guard reporte en PR | `if: always()` + `needs` explícito; y `ci-rust.yml` no lleva `paths:` en `pull_request:` | `if:` condicionado al evento | el 5º `if:` condicionado es exactamente cómo `FIND-765`/`FIND-791` fabrican un required que no corre |
| D4 | Cómo lee los resultados | `needs.<job>.result` | `gh api .../check-runs` del propio SHA | `needs` no necesita red, ni `actions: read`, ni jq, ni `run_id`: un fallo de API dentro de un merge gate es un modo de fallo nuevo y peor que el que se arregla. El techo: no puede probar `steps: []` — pero `skipped` ya lo cubre |
| D5 | ¿Se quita `needs: secret-scan` de los tres tests? | **No** | quitarlo (3 líneas menos) | medido: `Secret Scan` **no** es required, así que hoy no bloquea nada y su único efecto es apagar tres required. Quitando la arista, los tests corren y **el secret scan pierde el único efecto que tiene** sobre el merge. Conservándola + guard, un gitleaks rojo ⇒ tests `skipped` ⇒ guard rojo ⇒ **bloquea**, sin tocar el ruleset — en todo PR donde las suites estén en scope. Borde declarado: en un PR que `rust-scope` apaga (sólo docs), el guard pasa por D6 y el gitleaks rojo sigue sin bloquear, porque `Secret Scan` no es required; se arregla con la línea **opcional** 2 del ruleset, no con código |
| D6 | Único `skipped` legítimo | `rust-scope` dice `false` **y** terminó `success` | "nunca se salta" | sin esto, cada PR de docs pagaría 3×~45-90 min de runner por un diff que ya se midió. Es fail-**closed** en el resto: scope roto o ausente ⇒ se exige ejecución |
| D7 | Resultado `failure` en un test | el guard **pasa** | fallar también | ese required ya está rojo por sí mismo; el guard sólo responde "¿se ejecutó?", no duplica el veredicto |
| D8 | Actor dependabot | el guard **falla** (nombra al actor en el error) | exención por actor | una exención por actor es el mismo agujero una capa más arriba (pre-mortem b). Queda declarado como precondición del ruleset: ver "Cambio de ruleset propuesto" |

## Pasos

1. ✅ Medir la premisa y el ruleset (`gh api`, arriba).
2. ✅ Escribir el job `tests-guard` en `ci-rust.yml` (aditivo, después de `test-macos`).
3. ✅ Extraer el `run:` real del YAML y correrlo en bash con los `needs` stubbeados: 4 caras.
4. ✅ `actionlint` con y sin shellcheck, con tiempo medido (baseline y después).
5. ✅ Registrar en `docs/dev/avance/activo/ci-cd.md` y **dejar la fila del Backlog intacta**: el gate es
   **inerte** hasta que el ruleset la exija, y retirarla dejaría sin rastreador un agujero abierto
   (Review M2). El fichero, además, está siendo barrido por otro carril en este worktree.

## Verificación

### Las dos caras (más las dos del borde), contra el `run:` real del YAML

El `run:` se **extrae del fichero** (no se copia a mano) y se ejecuta con bash 5.3.9, con los
`needs.*` sustituidos por variables de entorno: es exactamente el script que corre en CI, con las
únicas entradas que GitHub le daría. Salidas completas en "Verificación ejecutada" abajo.

| Cara | Entradas | Esperado | Obtenido |
|---|---|---|---|
| (i) la suite ejecuta | `LINUX/WINDOWS/MACOS=success`, `SCOPE=true` | exit 0 | exit **0** |
| (ii) la suite se salta | los tres `skipped`, `SCOPE=true` | exit 1 nombrando los tres | exit **1**, tres `::error::` |
| (iii) path scope medido | los tres `skipped`, `SCOPE=false`, `SCOPE_RESULT=success` | exit 0, motivo impreso | exit **0** |
| (iv) scope roto (fail-open) | los tres `skipped`, `SCOPE_RESULT=failure` | exit 1 | exit **1** |
| (v) un test en `failure` | `LINUX=failure`, otros `success` | exit 0 (ese required ya está rojo) | exit **0** |
| (vi) PR de dependabot | los tres `skipped`, `ACTOR=dependabot[bot]` | exit 1 | exit **1** |

Además, seis caras adversariales corridas por el revisor (no por el autor), todas fail-closed: sin
ninguna env (`unknown`), `SCOPE=false` con `SCOPE_RESULT` vacío, `SCOPE=False` capitalizado, y
`cancelled` / `timed_out` / `neutral` ⇒ exit **1** en todos.

### `actionlint` — con y sin shellcheck, **con tiempo**

`FIND-785` midió que un `run:` que gana una sentencia puede colgar `actionlint` >100 s y bloquear
todos los carriles por el `pre-commit`. Medido antes de tocar el fichero y después:

| Momento | `actionlint` (shellcheck) | `actionlint -shellcheck=` |
|---|---|---|
| baseline, `HEAD` (1.7.12) | exit 0 — **0,34 s** | exit 0 — **0,06 s** |
| después del job nuevo (1.7.12) | exit 0 — **0,37-0,46 s** (3 corridas) | exit 0 — **0,08-0,09 s** |
| después del job nuevo (**1.7.7, el pineado por el gate**) | exit 0 — **0,465-0,472 s** | exit 0 — **0,30-1,10 s** |
| baseline `HEAD` (1.7.7) | exit 0 — **0,395-0,513 s** | exit 0 — **0,067-0,098 s** |

Las dos primeras filas son medición propia con el binario que hay en esta máquina (`1.7.12`); las dos
últimas las corrió el **revisor** con el tarball **pinned** que usa `.github/workflows/docs-gates.yml`
(`1.7.7`, shellcheck 0.11.0), porque `1.7.12` no es la base de evidencia que gatea. Con las dos
versiones: exit 0 y **sin cuelgue**.

El bloque nuevo son **26 líneas**: 18 sentencias, 4 comentarios y 4 líneas en blanco (contadas por el
extractor). Los 4 comentarios **están dentro** del `run:` a propósito: explicar cada rama donde la
lee quien depura el gate, como el resto del fichero. `shellcheck -S style` sobre el script extraído:
exit 0.

### El "antes" no tenía nada que ponerse rojo

Extraer el job del `HEAD` no falla por un caso del test: no existe.

```text
$ git show HEAD:.github/workflows/ci-rust.yml > ci-rust.HEAD.yml
$ python extract.py ci-rust.HEAD.yml tests-guard "Fail when a required test check was skipped..."
    start = next(i for i, l in enumerate(lines) if job_re.match(l))
StopIteration
```

O sea: en `43e41f055` no hubo un guard que se equivocara; **no había guard**. La prueba de dos caras
no se puede correr "contra el código viejo" porque el código viejo es la ausencia del gate.

## Cambio de ruleset propuesto (**NO aplicado**)

Una línea, `develop` (23692587), `required_status_checks`:

```text
+ "Tests Executed (guard)"
```

Opcional, sólo si el owner quiere cerrar el borde de D5 (un PR de sólo docs con `gitleaks` rojo hoy
puede mergear: las suites las apaga `rust-scope` y `Secret Scan` no es required):

```text
+ "Secret Scan (gitleaks + secretlint)"
```

Esa segunda línea **no** la necesita este hallazgo — con el guard, un gitleaks rojo ya bloquea todo PR
donde las suites estén en scope — y no se propone como parte del arreglo: se enumera porque es la
respuesta correcta al borde, y aplicarla sí exige medir su coste en los PRs que no tocan Rust.

- **Por qué no es el deadlock de `FIND-765`:** `ci-rust.yml` no lleva `paths:` en `pull_request:`
  (FIND-770), así que el workflow arranca en todo PR, y el job lleva `if: always()` sin condición de
  evento ni de actor: reporta en cada run. Un required que reporta siempre no puede quedar `Pending`.
- **Por qué `main` (16844590) no lo necesita para este hallazgo:** el candado roto era el de
  `develop`. Si se agrega en `develop` y no en `main`, la asimetría es la misma que ya existe con
  `Miri (UB Detection)` (required en `develop`, ausente en `main`) — declarado, no silencioso.
- **Precondición que el owner debe decidir antes de aplicarlo (D8):** con el guard required, un PR
  de **dependabot** contra `develop` queda rojo (los tres tests se saltan por
  `if: github.actor != 'dependabot[bot]'`). Tres salidas, en orden de honestidad:
  1. aceptarlo — el contrato es "un required mide"; con `FIND-472` el PR de dependabot ya no mide nada;
  2. quitar el guard de actor de los tres tests — cuesta 3×~45-90 min por PR de dependabot;
  3. eximir al actor en el guard — **reabre el agujero** para ese actor (pre-mortem b). No recomendado.
- **`Secret Scan (gitleaks + secretlint)` no necesita volverse required para este hallazgo:** en todo
  PR **donde las suites estén en scope**, su rojo bloquea por la vía del `skipped` que provoca. Lo que
  **no** cubre es el PR que `rust-scope` apaga (sólo docs): ahí el guard sale por D6 **antes** de leer
  los tres resultados (medido: cara (iii) ⇒ exit 0 con `gitleaks` rojo), así que ese borde sigue
  abierto y su arreglo es la línea opcional 2, no este job.
- **Lo que este carril no puede verificar:** que GitHub evalúe el ruleset así. Sólo se prueba en un
  push/PR real después de aplicarlo. Hasta entonces, el guard es **inerte** (publica un check que
  nadie exige).

## Verificación ejecutada

### Las caras, contra el `run:` extraído del YAML

```text
$ bash find787_harness.sh
=== run: block extracted from the YAML: 26 lines, sha256 d1e90f2778bbbd35 ===

--- (i) la suite ejecuta: los tres success, scope=true
  Tests (Linux) -> success (executed)
  Tests (Windows) -> success (executed)
  Tests (macOS) -> success (executed)
All three required test suites executed
exit=0

--- (ii) la suite se salta: los tres skipped, scope=true  [el caso de 43e41f055]
::error::Tests (Linux) reported 'skipped' (actor someone) - a required check satisfied by a skip measured nothing (FIND-787)
::error::Tests (Windows) reported 'skipped' (actor someone) - a required check satisfied by a skip measured nothing (FIND-787)
::error::Tests (macOS) reported 'skipped' (actor someone) - a required check satisfied by a skip measured nothing (FIND-787)
::error::The required test suite did not execute: Tests (Linux) (skipped); Tests (Windows) (skipped); Tests (macOS) (skipped); fix the upstream cause and re-run - do not merge on a skip
exit=1

--- (iii) path scope medido: los tres skipped, scope=false + success  [unico skipped legitimo]
rust-scope measured the diff: not a Rust change -> the test suites are out of scope
exit=0

--- (iv) scope roto (fail-open): los tres skipped, SCOPE_RESULT=failure
::error::Tests (Linux) reported 'skipped' (actor someone) - a required check satisfied by a skip measured nothing (FIND-787)
::error::Tests (Windows) reported 'skipped' (actor someone) - a required check satisfied by a skip measured nothing (FIND-787)
::error::Tests (macOS) reported 'skipped' (actor someone) - a required check satisfied by a skip measured nothing (FIND-787)
::error::The required test suite did not execute: Tests (Linux) (skipped); Tests (Windows) (skipped); Tests (macOS) (skipped); fix the upstream cause and re-run - do not merge on a skip
exit=1

--- (v) un test rojo, los otros ejecutados: el required ya esta rojo por si mismo
  Tests (Linux) -> failure (executed)
  Tests (Windows) -> success (executed)
  Tests (macOS) -> success (executed)
All three required test suites executed
exit=0

--- (vi) PR de dependabot: los tres skipped por actor  [D8, precondicion del ruleset]
::error::Tests (Linux) reported 'skipped' (actor dependabot[bot]) - a required check satisfied by a skip measured nothing (FIND-787)
::error::Tests (Windows) reported 'skipped' (actor dependabot[bot]) - a required check satisfied by a skip measured nothing (FIND-787)
::error::Tests (macOS) reported 'skipped' (actor dependabot[bot]) - a required check satisfied by a skip measured nothing (FIND-787)
::error::The required test suite did not execute: Tests (Linux) (skipped); Tests (Windows) (skipped); Tests (macOS) (skipped); fix the upstream cause and re-run - do not merge on a skip
exit=1
```

bash 5.3.9, sin `gh` ni red: las entradas de `needs.*` son variables de entorno. El `run:` no se
copia a mano: lo extrae un script del propio `.github/workflows/ci-rust.yml`.

### `actionlint`

```text
$ actionlint .github/workflows/ci-rust.yml             -> EXIT=0   0,37 s / 0,43 s / 0,46 s
$ actionlint -shellcheck= .github/workflows/ci-rust.yml -> EXIT=0   0,08 s / 0,09 s
```

### Lo que **no** se puede verificar acá

- **Que GitHub evalúe el ruleset así.** Un ruleset sólo se prueba en un push/PR real, y encima este
  cambio es **inerte** hasta que el owner agregue `Tests Executed (guard)` a los required: publicar
  el check no cambia ningún veredicto por sí solo. La prueba de campo es: un PR con `secret-scan` en
  rojo debe quedar en **`Tests Executed (guard): failure`** — no mergeable.
- **Que el job se programe de verdad con `if: always()` en un run real.** El razonamiento está en el
  Spec (D3) y es una lectura de la documentación de `jobs.<id>.if` + `always()`, no una medición
  local.


## Deuda

- El guard no puede distinguir `skipped` por **path scope** de `skipped` por cancelación de la
  `concurrency`: una cancelación del run entero lo cancela **a él también**, y `cancelled` no
  satisface un required (bloquea), así que el agujero no se abre por ahí (`FIND-764` es la fila de
  las suites canceladas, cerrada).
- El guard mide `needs.<job>.result`, no el check-run publicado. Si mañana un job cambia su `name:`
  y el ruleset no, el guard sigue verde mientras el required queda `Pending`: es el mismo defecto de
  `FIND-791` (nombre vs matrix), con otra causa. Detectarlo exige leer `check-runs` (D4, descartado).
- `Tests (embeddings features)` también quedó `skipped` en `43e41f055` y **no** es required: queda
  fuera del contrato de esta fila a propósito (el guard sólo cubre los tres required).
- **C1 (bloqueante, del review P2-01): `Generated indexes are current` queda rojo por el doc nuevo.**
  `node scripts/docs/gen-index.mjs --check` ⇒ exit 1 (`### Task files — 288` → `293`, `documents
  indexed 884`). Comprobado: **no es sólo mi fichero**. El worktree tiene **5** `docs/dev/tasks/*.md`
  sin trackear: `FIND-787.md` (mío) y `FIND-792/793/795/796.md` (el carril del helper de commit, que
  este contrato prohíbe tocar). Correr `gen-index --write` acá indexaría **cuatro ficheros ajenos**
  dentro de mi commit: la clase exacta de `FIND-792`. Por eso el paso **no se corre** en este carril
  (además está prohibido por el contrato) y la entrada la agrega quien cierre último, o una corrida
  en árbol limpio. `check-docs` y `check-links` sí dan 0: no tocan índices.
- **L5:** `on.push.paths` de `ci-rust.yml` sigue vigente, así que un push que sólo toque `docs/` **no
  produce run** y el guard, si algún día es required, quedaría `Expected` en ese push igual que los
  otros 10 contexts de este fichero (el bypass `RepositoryRole` del ruleset es lo que hoy lo tapa). No
  es nuevo; queda dicho para que la línea del ruleset no se lea como "cubre también los push de docs".
- **L6:** el guard prueba que el job **arrancó y midió** (conclusión `success`/`failure`), no cuántos
  tests corrieron. Un job que arranca y ejecuta 0 tests sigue verde: es `FIND-794`/`FIND-788` (el
  contador) y necesita su propio contrato.

## Review

**P2-01 — `vanta-review`, contexto fresco, una ronda. Veredicto: `changes-required` (1 Critical, 2 High,
5 Medium, 7 Low/nits).** El contrato de código pasó y el revisor lo reprodujo (extracción con PyYAML,
hash `d1e90f2778bbbd35` idéntico, 6 caras declaradas + 6 propias), y la premisa se sostuvo; lo que no
pasó fue el cierre documental.

| # | Hallazgo | Estado |
|---|---|---|
| C1 | `gen-index --check` rojo por el doc nuevo: deja roja una required context | **declarado, no arreglable acá** — `--write` indexaría 4 task files ajenos (`FIND-792/793/795/796`) y el contrato prohíbe `gen-index`. Medido y escrito en Deuda/§Impacto |
| H1 | "un solo `failure` real" falso: hay **dos** (`Secret Scan` + `Linux Snapshot Certification`) | **corregido** en el Contexto; re-verificado (`gh api .../check-runs --jq` de `failure`) |
| H2 | `status: completed` con `## Review` vacío | **corregido**: esta tabla |
| M1 | "el `run:` no lleva un solo `#`" falso: tiene 4 | **corregido**: se declaran 26 líneas (18 sentencias + 4 comentarios + 4 blancas) y el por qué |
| M2 | retirar la fila del Backlog con el gate inerte | **corregido**: la fila se **queda**, sin tocar el texto; y el fichero ni siquiera entra en el commit porque otro carril lo está barriendo en este worktree (`-Paths` no lo incluye) |
| M3 | "`Secret Scan` ya bloquea con el guard" sin el calificador | **corregido** en D5, en el bullet del ruleset y en el avance: sólo en PRs con las suites en scope |
| M4 | WIP ajeno en el árbol, hazard de commit | **corregido**: commit por path con las 5 rutas, nada de `git add -A` |
| M5 | `actionlint` medido en 1.7.12, el gate pinea 1.7.7 | **cerrado con la medición del revisor** en 1.7.7 (exit 0, 0,465-0,472 s): fila propia en la tabla |
| L1 | 2 `U+FFFD` en `ci-cd.md` | **corregido** |
| L2 | cita `ci-gate.yml:105` por `:96` | **corregido** |
| L3 | "12 líneas de `run:`" vs 26 | **corregido** |
| L4 | `CI_POLICY.md` no lista el job | **declarado**: la tabla ya está parcial (drift previo), fuera del write-scope |
| L5 | `on.push.paths` + required en push sólo-docs | **declarado** en Deuda |
| L6 | el guard prueba "arrancó", no "cuántos tests" | **declarado** en Deuda |
| L7 | la cita de la doc no estaba en la página citada | **corregido** con la frase real, verificada por `webfetch` hoy: *"A job that is skipped will report its status as 'Success'. It will not prevent a pull request from merging, even if it is a required check."* |
| nit | el `::error::` dice "satisfied by a skip" también para `cancelled` | **declarado**: cosmético, el `needs` global dice "did not execute" |

**El revisor resistió sin cambios** (re-ejecutado por él, no aceptado por mi palabra): el fail-closed
del guard con 6 caras adversariales, la extracción byte-idéntica del `run:`, el `--numstat` aditivo
(`+80/−0`), que el ruleset no se tocó (`updated_at` sin cambio, ningún `gh api -X`), y el razonamiento
(A) vs (B) con la precondición de dependabot.

**Sustitución de gate declarada:** el tier de estos paths (`docs/dev/**`, `.github/workflows/**`) es
**Fast** y su gate es `dev-tools/verify.ps1`; este carril tiene **prohibido compilar Rust** (disco
~7,7 GB), así que el gate mecánico aplicado es el del área: `actionlint` con y sin shellcheck (con
tiempo, contra la versión pineada), `check-docs` + `check-links`, y el harness de 6 caras sobre el
`run:` extraído.

## NOTICED BUT NOT TOUCHING

- `.github/workflows/ci-gate.yml:96` — `success|skipped|neutral) ;;` trata `skipped` como verde en
  el gate que decide si corren las lanes pesadas. Es la misma semántica, con blast radius distinto
  (aborta/no aborta horas de runner; no cierra el candado de una rama) y con otra política declarada
  (`EST-03`, owner 2026-09-24: "some guards skip by design"). Fuera del contrato: no lo toco.
- El `Adapter compat (${{ matrix.adapter }}/...)` sigue exportando el nombre sin expandir
  (`FIND-791`): no es mi fila.
- `docs/` entero en la allowlist de `.gitleaks.toml` (`FIND-786`, declarado allí).
