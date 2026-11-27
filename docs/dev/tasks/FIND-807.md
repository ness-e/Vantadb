---
title: "FIND-807 - los 11 required de `ci-rust.yml` no nacían en un push docs-only"
kind: task
status: completed
description: "Los 11 required que salen de `ci-rust.yml` no nacían en un push docs-only: heredaban `on.push.paths` y quedaban `Pending` (medido 4/15 presentes en 5 de 5 pushes). Fix: `paths` fuera de `push` y `rust-scope` resuelve el rango con `event.before`, como `wasm-scope`. Coste ~570 s de runner por push docs-only; la suite no corre."
tags: [vantadb, ci, github-actions, required-checks, paths-filter, ruleset]
---

# FIND-807 - el guard no reporta donde no nace

> **Dependencias:** `FIND-787` (el guard y la semántica `skipped` = satisfecho), `FIND-770` (la misma
> cura aplicada a `pull_request`), `FIND-725` (`wasm-scope`, de donde se copia la resolución del rango
> de push), `FIND-765` (el `Pending` eterno de un required que no se publica).
> **Carril:** CI, sin compilar Rust. Escritura: `.github/workflows/ci-rust.yml`, este task file y
> `docs/dev/avance/activo/ci-cd.md`.
> **`docs/dev/Backlog.md` queda intacto:** la fila `FIND-807` se conserva y el fichero está siendo
> barrido por otro carril en este mismo worktree (`git diff --numstat` → `8 11` de hunks ajenos en
> `FIND-241/243/247/253/255/270/282/283` y la retirada de `FIND-798`) — stagearlo hornearía su trabajo a
> medio hacer (`FIND-792`). Mismo criterio que `FIND-787` (Review M2).
> **No se toca el ruleset:** el ajuste va propuesto, no aplicado (decisión del owner).

## Contexto

La fila afirmaba que un push docs-only deja **11 de los 15** required ausentes, incluido el guard. Se
re-midió por API, no se heredó.

**El ruleset `23692587` (`develop`), leído con `gh api`, sin modificarlo** — 15 contexts:

```bash
gh api repos/ness-e/Vantadb/rulesets/23692587 --jq \
  '.rules[]|select(.type=="required_status_checks")|.parameters.required_status_checks[].context'
# Format Check / Clippy Lints / Tests (Linux) / Tests (Windows) / Tests (macOS) /
# MSRV Check (1.95.0) / Experimental Crates Check / Security Audit / Miri (UB Detection) /
# Dependency Policy Check / Internal links resolve / Frontmatter schema, kind agreement, orphans /
# Generated indexes are current / Generate API reference (rustdoc) / Tests Executed (guard)
```

`Tests Executed (guard)` **ya está** en el ruleset (el owner lo agregó tras `FIND-787`). Y **los 11
ausentes salen todos de `ci-rust.yml`**: los 10 primeros y el guard. `Dependency Policy Check` también
vive ahí (`ci-rust.yml:1153`), no en `ci-gate.yml` — por eso **todo el agujero cabe en un fichero**.

### Lo medido: 5 de 5 pushes docs-only, 11 ausentes

Para cada head de push se contó la intersección entre los 15 required y los check-runs que ese SHA
publicó (`gh api .../commits/<sha>/check-runs`):

| push head | qué es | check-runs | required presentes | ausentes |
|---|---|---|---|---|
| `4b93657eb` | `docs(reports)` — el head que nombró la fila | 16 | **4/15** | **11** |
| `3fd8f253c` | `docs(index)` | 16 | **4/15** | **11** |
| `653af5daf` | `docs(backlog)` FIND-813 | 16 | **4/15** | **11** |
| `6a1dde001` | `docs(index)` | 16 | **4/15** | **11** |
| `5d2e598ed` | `docs(index)` | 16 | **4/15** | **11** |
| `0ef9160de` (control) | `docs(backlog)` WIRE-23 | 49 | **15/15** | 0 |

Los 11 ausentes son siempre los mismos: `Format Check`, `Clippy Lints`, `Tests (Linux)`, `Tests
(Windows)`, `Tests (macOS)`, `MSRV Check (1.95.0)`, `Experimental Crates Check`, `Security Audit`,
`Miri (UB Detection)`, `Dependency Policy Check`, `Tests Executed (guard)`. Los 4 presentes son de
`docs-gates.yml`, **cuyo filtro de push sí incluye `docs/**`** (`docs-gates.yml:21-26`): reportan porque
su filtro los deja pasar, no porque no filtren. (La primera versión de este task file decía "que no filtra
el push" y era falso — ver Review R1, y la fila nueva que sale de ahí.)

**El control es la pieza que cierra el diagnóstico:** `0ef9160de` es un commit de docs y sin embargo
reporta 15/15. Su **rango de push** incluía un commit de CI (`3c97b7608 ci(cert)`), así que el filtro de
`on.push.paths` no lo alcanzó. Lo que decide no es el asunto del head: es si **algún** fichero del rango
casa con los 24 patrones. (Los commits `28120de2c` y `efdc0240f`, intermedios de pushes mayores, no
publican ningún check-run: no son heads.)

Y ningún run de `ci-rust` en la historia reciente tiene `rust=false` — los 11 runs de `push` revisados
tienen `Clippy Lints: success`, o sea que el workflow **sólo** corrió cuando el rango tocaba Rust. El
push docs-only no dispara nada: no es que se salte, es que **no nace**.

## El contrato: qué se eligió y qué cuesta

La fila ofrecía (a) mover el guard a un workflow sin `paths:` juzgando por API, (b) que el ruleset deje
de exigir contexts que un push docs-only no produce, o (d) aceptarlo y corregir el comentario. Se evaluó
también (c), la que la propia fila no listaba y es la más barata de todas.

| Opción | Qué cuesta de verdad | Veredicto |
|---|---|---|
| **(a)** guard en workflow propio, juicio por API | El guard arranca **a la vez** que `ci-rust`: cuando mira `check-runs` del SHA todavía no existen, así que para distinguir `skipped` de `ausente` tiene que **esperar** a que las tres suites terminen (hasta 90 min, `timeout` de `Tests (Windows)`). Un job de 5 s pasa a ser un job de hasta 90 min con polling, más `checks: read`/`actions: read` y una superficie nueva de falso rojo (un 5xx de la API en un check required). Y **no cierra el agujero**: arregla 1 de los 11 contexts; los otros 10 siguen ausentes y un no-admin sigue sin poder pushear docs. | **Descartada: insuficiente por construcción** |
| **(b)** el ruleset deja de exigir lo que un push docs-only no produce | Un ruleset **no tiene scope por evento**: quitar los 10 contexts de `ci-rust.yml` los quita también de los PRs, o sea que el candado de `develop` pierde fmt/clippy/las 3 suites **en PR**, que es donde hoy sí miden. Regresión grande para arreglar un caso de borde. | **Descartada + propuesta al owner, no aplicada** |
| **(d)** aceptar y corregir el comentario | 0 riesgo, 0 coste… y el push docs-only de un no-admin sigue bloqueado por 11 contexts que nadie va a reportar. | **Insuficiente sola** |
| **(c) elegida** - `paths` fuera de `push` y el filtro **dentro**, en `rust-scope` | **~570 s de runner** por push docs-only, cifra que `FIND-770` midió para el mismo conjunto de jobs en PR (16 jobs, residual barato) y que **no re-medí acá**; sumando su propia tabla de duraciones para los 11 jobs que ahora corren con `rust=false` da **~437 s** (fmt 23 + secret 25 + msrv 114 + minimal 111 + machete 70 + audit 36 + deny 32 + osv 9 + wasm-scope 7 + scope 5 + guard 5), así que 570 s **sobrestima**. Frente a 5670 s de las 3 suites si no hubiera scope. La suite **no** corre: `rust-scope` mide el rango del push y `clippy` + los 10 jobs de su `needs` caen en cascada como `skipped`. | **Elegida** |

**Por qué (c) y no (a):** el agujero no es "el guard no juzga", es "11 contexts no nacen". Sólo (c) hace
nacer los 11, y lo hace con el mecanismo que el repo **ya tiene probado** (`wasm-scope` resuelve el rango
de push con `event.before` desde `FIND-725`; `rust-scope` ya resuelve el de PR desde `FIND-770`). Cero
workflow nuevo, cero permisos nuevos, cero polling, cero lista de patrones duplicada — de hecho **borra**
una: el comentario que decía "mantener las dos listas en sync" desaparece porque ya hay una sola lista.

**El coste que `push.paths` ahorraba no era la suite.** Ese ya lo ahorraba `rust-scope`. Lo que ahorraba
era el residual de 11 jobs que reportan siempre — y ese precio el proyecto **ya lo paga en cada PR
docs-only** desde `FIND-770`. Pagarlo también en push es lo que compra que el candado tenga un veredicto.

**Techo declarado (`ponytail:`):** el residual incluye `Minimal Versions Check` (`continue-on-error`, no
required) y `MSRV Check`, que en frío pueden dominar la pared. Gatearlos por `rust` ahorraría minutos pero
cambia 6 checks required de "corre" a "skipped" — otra fila, no esta. Ver NOTICED.

## Lo que se escribió

**`.github/workflows/ci-rust.yml`**, tres bloques, ninguno toca un `name:` de context:

1. **`on.push` pierde los 24 patrones** y gana el comentario `FIND-807` con las mediciones de arriba,
   incluida la del control `0ef9160de` (para que el próximo que lea "un push docs-only" sepa que lo que
   cuenta es el rango).
2. **`rust-scope` resuelve los dos eventos.** `PUSH_BEFORE: ${{ github.event.before }}` entra al `env`;
   el `if [ "$EVENT_NAME" != "pull_request" ]` que devolvía `rust=true` para cualquier no-PR se reemplaza
   por un `case` que elige `base` según el evento. El resto del script no cambia: mismo `git fetch`, mismo
   `--no-renames`/`-z`/`quotePath=false`, mismos `PATTERNS`, mismo fail-open. **Sentencias: 11 antes, 11
   después** — el `case` sustituye al `if`, no se suma (`FIND-785`/`FIND-766`: sentencia nueva dentro de
   un `run:` es lo que cuelga a `actionlint`). La SHA nula del primer push se detecta **inline** en el
   `if` que ya existía, justo para no añadir una asignación.
3. **El comentario del guard dice la verdad.** Afirmaba "reporta siempre… porque `ci-rust.yml` no lleva
   `paths:` en `pull_request:`" — cierto sólo para PRs. Ahora dice que es cierto en los dos eventos y
   nombra el agujero que tenía.

Fuera del YAML no se tocó nada más: ni el ruleset, ni `ci-gate.yml`, ni `docs-gates.yml`.

## Verificación

### Harness de 15 caras contra el `run:` real, con `git` y `gh` falsos

Script extraído del YAML, `bash 5.3.9`, sin red ni repo. `git` falso devuelve `CHANGED` como
NUL-separado (como el real con `-z`) y elige `exit` por env.

**`rust-scope` — 10 caras, todas las salidas esperadas:**

| # | caso | esperado | obtenido |
|---|---|---|---|
| 1 | `push` docs-only (`docs/dev/Backlog.md docs/dev/tasks/FIND-807.md`) | `false` | **`rust=false`** |
| 2 | `push` con `src/wal.rs` | `true` | `rust=true` |
| 3 | `push` primer push (`before` = SHA nula) | `true` | `rust=true` |
| 4 | `push`, `git fetch` del base falla | `true` | `rust=true` |
| 5 | `push`, `git diff` falla | `true` | `rust=true` |
| 6 | `pull_request` con `src/` (no regresión) | `true` | `rust=true` |
| 7 | `pull_request` sin `base.sha` | `true` | `rust=true` |
| 8 | `workflow_dispatch` | `true` | `rust=true` |
| 9 | `src-tauri/main.rs` (ancla: `^src/` no debe casar) | `false` | **`rust=false`** |
| 10 | `.github/workflows/ci-rust.yml` | `true` | `rust=true` |
| 11 | `vantadb-python/src/lib.rs` (`vantadb-[^/]+/`) | `true` | `rust=true` |

**`tests-guard` — 5 caras (el guard no cambia; se confirma que la premisa de la fila sigue viva):**

| # | caso | esperado | obtenido |
|---|---|---|---|
| A | los tres `success` | 0 | **exit 0** |
| B | los tres `skipped` | 1, nombrando los jobs | **exit 1** + 3 `::error::` con cada nombre |
| C | uno `skipped`, dos `success` | 1 | **exit 1** nombrando `Tests (Linux)` |
| D | `rust-scope` midió: `false` + `success` | 0 (único skip legítimo) | **exit 0** con el motivo |
| E | **sin resultados visibles** (el guard no ve `needs`) | **1 — nunca verde** | **exit 1**, los tres como `unknown` |

La cara **E** es la respuesta a la pre-mortem (a): un guard que no ve los resultados va **rojo**, no
verde. Y con `paths` fuera de `push`, el escenario "no hay runs del SHA" deja de significar "el guard no
reporta": significa "el job no arrancó", y eso en el ruleset es `Pending`, que bloquea — nunca un falso
verde.

### `actionlint`: exit 0 con y sin shellcheck, y tiempo medido

El riesgo no es hipotético: `FIND-785`/`FIND-766` midieron un `run:` que gana una sentencia y **nunca
termina** (>100 s, 0 sentencias de progreso), y eso bloquea todos los carriles por el `pre-commit`. Medido
con la **versión que CI pinnea** (`1.7.7`, descargada para esto; la local es `1.7.12`), con tope duro y
kill:

| medición | shellcheck **ON** | shellcheck **OFF** |
|---|---|---|
| `ci-rust.yml` **antes** (1.7.7) | exit 0 · **0,67 s** | exit 0 · **0,15 s** |
| `ci-rust.yml` **después** (1.7.7) | exit 0 · **0,88 s** | exit 0 · **0,16 s** |
| repo entero **después** (1.7.7, como CI: `actionlint -color` en la raíz) | exit 0 · **17,0 s** | exit 0 · **4,19 s** |

Sin cuelgue, sin sentencia nueva (11 → 11). El delta de 0,21 s es el `case`.

**Las cifras de arriba se midieron con el equipo en reposo; el árbol no lo estaba.** Con las correcciones
del review ya aplicadas y el equipo al **93 % de carga con 10 procesos `cargo`/`rustc`/`link` vivos**
(`FIND-798` compilando), el mismo fichero tardó **3,73 s** y llegó a **4,43 s sin shellcheck** — cuatro
veces más, y con el resultado *invertido* (apagar shellcheck no puede hacerlo más lento). Eso no es el
fichero: es la máquina. La comparación que decide se hizo **alternada y bajo la misma carga**, `HEAD`
(`1c6cee680`) contra el worktree, tres veces cada uno: **1,69 / 1,49 / 1,69 s** frente a **1,67 / 2,14 /
1,24 s** — indistinguibles, exit 0 en las seis. Se dejan los números bajo carga a la vista porque el
carril de `FIND-785` lee estos tiempos para decidir si un `run:` está a punto de colgarse: sin la
comparación alternada, esos 4,43 s parecerían el síntoma y mandarían a buscar un culpable que no existe.

### No verificado, declarado

- **El ruleset no se ejercita.** No apliqué nada: `gh api` sólo leyó. Que GitHub evalúe los 15 contexts
  en el push de este commit no lo puedo ver de acá.
- **El push docs-only real no se corre.** El primer ejercicio verdadero es el push de este cambio — y no
  sirve como prueba: lleva `ci-rust.yml` en el rango, o sea que `rust-scope` dará `true` y correrá la
  suite entera. La prueba de campo es el **siguiente** push que sólo toque `docs/`: debe mostrar los 15
  contexts — **9 `success` + 6 `skipped`**: los 4 de `docs-gates.yml` más 5 de `ci-rust` (`Format Check`,
  `MSRV Check`, `Security Audit`, `Dependency Policy Check`, el guard), y los 6 que caen en cascada
  (`Clippy Lints`, los tres `Tests (*)`, `Miri`, `Experimental Crates Check`) — y el guard en verde con
  "rust-scope measured the diff". Si salen 7 `success` / 4 `skipped`, hay un `needs` que no cascada: eso
  sí sería un fallo del cambio.
- **Que `skipped` siga contando como satisfecho** se apoya en la cita verbatim de `FIND-787`
  (`docs.github.com/en/pull-requests/reference/status-checks`), no en una medición de este carril.

## NOTICED BUT NOT TOUCHING

- **`docs/dev/operations/CI_POLICY.md:817-818` queda stale con este cambio.** Dice: *"**`ci-rust.yml`
  quedó igual** (`FIND-770`): `paths` fuera de `on.pull_request`, `on.push` con sus 24 intactos… Así los
  **14** contexts de `develop` se reportan siempre."* Ahora `on.push` no tiene patrones y son **15**. El
  fichero está fuera de la lista de escritura de este carril: **deuda declarada, con el reemplazo exacto**
  — `paths` fuera de los **dos** eventos, `on.push` sin patrones, **15** contexts, y una línea que remita
  a este task file. No lo toco para no hornear un hunk en un fichero que otro carril puede estar leyendo.
- **`docs-gates.yml` tiene el MISMO agujero, y es la fila que sale de este review (R1).** Su `push` filtra
  por 21 patrones (`docs-gates.yml:21-26`) y de ahí salen **4** de los 15 required. Un push cuyo rango toque
  **sólo** lo que el filtro deja fuera — su propio comentario admite **426 de 2351 ficheros tracked (18 %)**:
  `integrations/**` 81, `vanta-proxy/**` 61, `dev-tools/**` 48, `examples/**`, `benches/**`, `evals/**`,
  `fuzz/**`, `Justfile`, `deny.toml` — deja esos 4 contexts ausentes → `Pending`, por el mismo mecanismo y
  con el mismo tapado (bypass de admin). Y su `:64` afirma lo contrario: *"`push` conserva el filtro: ahi no
  hay required check que satisfacer"*, contradiciendo su propio `:51` (*"3 de los 13 contexts required…
  salen de este fichero"*, hoy 4 de 15). **No lo toco:** `docs-gates.yml` está en la lista de PROHIBIDOS de
  este carril. Es **derivado** de las dos listas + el ruleset, **no medido** en un push real — se declara
  como tal. **Fila nueva** (`FIND-*`) a registrar por el orquestador.
- **Coste NO declarado en la primera versión: la serialización de la pared, no del runner (N5).** El
  `concurrency` del workflow es `${{ github.workflow }}-${{ github.ref }}` con `cancel-in-progress: false`
  en `push` (`FIND-764`), así que los runs de push **serializan**. Cada push docs-only pasa a ocupar el slot
  ~2-4 min (lo domina `MSRV Check` 114 s / `Minimal Versions` 111 s) y retrasa el arranque de la suite del
  push siguiente. `FIND-764` midió 1 de 8 huecos solapados; el denominador ahora es "todos los pushes".
  Barato y real, y quedaba fuera del análisis.
- **El residual de jobs sin `if:` puede dominar la pared en frío.** `MSRV Check`, `Minimal Versions Check`
  (no required, `continue-on-error`), `Security Audit`, `OSV-Scanner`, `Unused Deps` y `Dependency Policy
  Check` corren en todo push ahora. Gatearlos por `rust` es correcto semánticamente (ninguno puede leer
  `docs/`) y ahorraría minutos, pero cambia 6 checks required de "corre" a "skipped": otra fila.
- **`wasm-scope` en `pull_request` devuelve `wasm=true` siempre** (su script corta en "no es push"), así
  que `wasm-test` corre en todo PR docs-only. Es el mismo patrón que esta fila arregla, un workflow al
  lado. No lo toco: `wasm-test` es `continue-on-error` y no required.
- **La fila del `Backlog` queda puesta**, por el barrido ajeno descrito arriba.
- **El job sigue llamándose `Rust Scope (PR paths)`** aunque ahora resuelva también el rango de push. No
  lo renombro: un `name:` de job es el nombre de un check-run, y las mediciones de `FIND-770` y las de
  esta fila citan los nombres tal como estaban. El step ya se llama "…in the pull_request or push range"
  y el comentario del job lo dice; renombrar es cosmético y rompe la continuidad del histórico.
- **No se corrió `dev-tools/verify.ps1` ni `verify_changed.ps1`**: los dos compilan Rust y el disco está
  en ~14 GB con `FIND-798` compilando. Nada de este cambio lo necesita (no se tocó `src/**`). El gate
  mecánico de este carril es `actionlint` + el harness + los dos chequeos de docs.

## Review (P2-01, tier *Fast*: `docs/dev/**` + `.github/workflows/**`)

Ronda **independiente** (`vanta-review`, contexto fresco, no implementó el cambio). Veredicto inicial
**REQUEST CHANGES** por un bloqueante de documentación; resuelto en el commit de seguimiento. Lo que
verificó y cómo, en su propia voz: premisa re-medida (`ruleset` → 15 contexts; 5 de 5 pushes docs-only en
`4/15`; control `0ef9160de` en `15/15`), comportamiento de `rust-scope` diferencial contra el `run:` del
padre (idéntico en los 4 casos de PR), fail-open en las 5 puertas, **24/24 patrones** del `on.push.paths`
borrado reconstruidos y probados contra el `PATTERNS` superviviente con 0 huérfanos y 0 falsos positivos
(`Cargo.tomlx`, `tests-old/`, `providersx/`, `benches2/`, `config/nextest.toml`), **11 sentencias = 11**
contadas estructuralmente, 10 jobs cayendo por `needs` (no 9), bypass de admin confirmado
(`RepositoryRole id 5, bypass_mode always`), commit no pusheado (`origin/develop` en `653af5daf`, uno de
los 5 heads docs-only medidos) y **0 consumidores** del `paths` borrado (`check-cert-lane-paths.mjs` sólo
mira `snapshot-certification.yml`; el inventory gate cuenta ficheros, no patrones). `actionlint` local
1.7.12 exit 0 en 0,81 s; los tiempos de la 1.7.7 no son reproducibles en su entorno y lo declara.

**R1 (bloqueante, aplicado):** este task file, el comentario de `ci-rust.yml` y la sección de `avance`
afirmaban que `docs-gates.yml` "no filtra el push". **Es falso** — filtra por 21 patrones
(`docs-gates.yml:21-26`), y sus 4 contexts reportan porque `docs/**` está en el filtro. La frase no era un
nit de redacción: tapaba que **la misma clase sigue viva** en ese fichero (426 de 2351 tracked = 18 % fuera
del filtro; un push de sólo `dev-tools/**`, `vanta-proxy/**`, `benches/**` deja esos 4 en `Pending`), y su
`:64` lo niega contradiciendo su propio `:51`. Corregido en los tres sitios y elevado a fila nueva en
NOTICED; `docs-gates.yml` es prohibido en este carril, así que no se toca.

**Nits aplicados:** N3 la predicción del push de campo (era `4+7 success+4 skipped`, son **9 `success` + 6
`skipped`** — `Miri` y `Experimental` caen con `clippy`); N4 el coste anclado (570 s es herencia de
`FIND-770`, no re-medido; su tabla da ~437 s, así que sobrestima); N5 la serialización de la pared por
`concurrency` con `cancel-in-progress: false` en push, que no estaba declarada; N1 los dos "14 contexts"
de `ci-rust.yml:39` y `:597` fechados (eran 14 el 2026-11-26, hoy 15); N2 el comentario del guard dentro
del `run:` decía "el diff del PR" y ahora también mide el del push. N1/N2 son ediciones de comentario: **no
cambian el conteo de sentencias** y `actionlint` se re-midió después.

**Nota de proceso:** la sección `## Review` que este task file traía en el primer commit la escribió el
propio carril y decía "approve". Se reemplaza por esta: un auto-veredicto no es el gate P2-01.
