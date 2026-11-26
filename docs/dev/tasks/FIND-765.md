---
title: "FIND-765 - el workflow que no dispara deja sus required checks en Pending"
kind: task
status: completed
description: "Un workflow filtrado por `paths` que no dispara no reporta ningún check-run: sus contexts required quedan en `Pending` para siempre. 426 de 2351 ficheros tracked (18%) quedaban fuera de `docs-gates.yml`, y 3 de los 13 contexts required de `develop` salen de él. Fix: `paths` fuera de `pull_request`, con el coste medido."
tags: [vantadb, ci, github-actions, required-checks, paths-filter]
---

# FIND-765 — el workflow que no dispara deja sus required checks en `Pending`

## Contexto

El ruleset `develop` (`23692587`, leído por API el 2026-11-26) tiene **13**
contexts required, **3** de ellos de `docs-gates.yml`:

```
Internal links resolve
Frontmatter schema, kind agreement, orphans
Generated indexes are current
```

`docs-gates.yml` disparaba por `on.pull_request.paths`, y esa lista no cubría
todo el árbol. Si ningún fichero del PR casaba, el workflow **no arrancaba**: no
había check-run, y los tres contexts se quedaban en `Pending` **para siempre**.
Es la clase `FIND-714`/`FIND-715` (un required check que nadie reporta), pero en
la capa del **disparador** en vez de la del **nombre**.

**"Inmergeable" quiere decir para quien no puede saltarse el ruleset.** El
ruleset `23692587` es `enforcement: active` con
`bypass_actors: [{actor_id: 5, actor_type: RepositoryRole, bypass_mode: always}]`
— o sea, **admin del repo sí puede mergear** y de hecho lo ha hecho: 4 PRs
sólo-docs fusionados **sin ningún run** (`#223` `docs/CHANGELOG.md`, `#229`
workflow+`PUBLISH.md`, `#264` `Backlog.md`, `#262` 24 docs, todos 2026-09/10).
El bug es real y **vivo** para quien **no** tiene bypass —Dependabot,
contribuidores— y ahí está la evidencia: **PR `#285`, hoy, con
`no checks reported on the branch`**. Por eso el bug se cuela durante meses: el
único actor que abre PRs en este repo casi siempre es el admin.

## El hallazgo que corrige la premisa de la fila

La fila del Backlog decía `web/**`. **`web/` no existe en ninguna rama**: ni en
`develop` ni en `origin/main` ni en ninguna de las 18 `origin/*`
(`git ls-tree -d <ref> -- web` → vacío; el `web/` del working tree es `?? web/`,
sin trackear). Sólo sobrevive en `refs/stash` y en los tags `v0.5.0`/`v0.6.0`/
`v0.6.1` — historia, no ramas vivas. El bug es real pero el path que nombra no.

Lo que sí queda fuera, medido sobre los patrones **tal como GitHub los evalúa**
(minimatch: `*` no cruza `/`, `**` sí) contra los 2351 ficheros de `HEAD`:
**426 de 2351 (18,1 %)** en ~30 entradas de primer nivel:

| Path | Ficheros | ¿Por qué importa? |
|---|---:|---|
| `integrations/**` | 81 | paquetes de adaptadores |
| `vanta-proxy/**` | 61 | **crate del workspace** (`vanta-proxy/Cargo.toml`) |
| `dev-tools/**` | 48 | se edita en casi todas las sesiones de agentes |
| `examples/**` | 38 | |
| `benches/**` | 27 | |
| `evals/**` | 25 | |
| `.github/**` | 11 | `dependabot.yml`, `pull_request_template.md`, `CONTRIBUTING.md` |
| `fuzz/**` `assets/**` `embeddings/**` … | 27 | |
| `Formula/**` `Justfile` `deny.toml` `rust-toolchain.toml` `CHANGELOG.md` … | 11 | raíz |

`vanta-proxy/**` es el caso más creíble: es un miembro del workspace y **no** lo
cubre `vantadb-*/**` (prefijo distinto).

## Por qué (a) no sirve y (b) sí — la distinción que decide

La documentación oficial separa dos cosas y sólo una es benigna:

| Causa | Resultado | Fuente |
|---|---|---|
| El **workflow** se salta por `paths` / `branches` / mensaje de commit | "Associated checks stay in a **Pending** state and block merging" · **"Avoid requiring workflows that can be skipped"** | [Troubleshooting required status checks](https://docs.github.com/en/pull-requests/how-tos/merge-and-close-pull-requests/troubleshooting-required-status-checks) |
| Un **job** se salta por `if:` | "The job reports **Success**" | idem |
| Un **job** se salta | "A job that is skipped will report its status as 'Success'. It will not prevent a pull request from merging, even if it is a required check." | [Status checks](https://docs.github.com/en/pull-requests/reference/status-checks) |
| Conclusión `skipped` | "treated as a success"; los estados aceptados son `success`, `skipped`, `neutral` | idem + Troubleshooting §"Required check needs to succeed against the latest commit SHA" |
| Workflow skipped → checks "remain in a Pending state" | | [Triggering a workflow](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/trigger-a-workflow) |

O sea: **el workflow que no corre no reporta nada; el job que se salta sí.**
Por eso la doc recomienda *no requerir workflows que puedan saltarse* — no
"ampliar un poco más la lista de paths".

## Opciones, con números

**(a) ampliar `paths`** (añadir `web/**` y compañía). Descartada con evidencia:
`web/**` **no existe** como path trackeado, y hay 426 ficheros más fuera. Cerrar
el hueco enumerando exige cubrir 30 entradas de primer nivel y **volver a
enumerar con cada directorio nuevo**: el fallo se reabre solo. Es un treadmill,
no un fix.

**(b) quitar el filtro en `pull_request`** ← **elegida**. Los 13 contexts se
reportan siempre, sea cual sea el path, incluidos los que aún no existen. Es la
forma que la doc llama correcta ("avoid requiring workflows that can be
skipped"): el workflow ya no puede saltarse.

**(c) job "no-op" que reporte los contexts cuando el resto no corre**.
Descartada: un job = **un** check-run, así que hacen falta **tres** jobs con el
`name:` duplicado de los reales; si el workflow algún día sí dispara, hay dos
check-runs con el mismo nombre y la conclusión que cuenta es la última. Duplica
la superficie de la clase `FIND-714` (nombres que deben coincidir a mano) en vez
de eliminarla.

### Coste medido de (b) — no es "16 jobs caros en cada PR"

Run **45771790202** (último verde completo de `docs-gates`, `2026-11-26T06:57:23Z`,
`head_sha f2e9688`): 16 jobs, **967 s de runner** (16,1 min), **10m38s de pared**
(arranque → último job).

| Job | s | % del total |
|---|---:|---:|
| Python examples match the real SDK | 466 | 48 % |
| Doc-tests (RUSTDOCFLAGS=-D warnings) | 270 | 28 % |
| Generate API reference (rustdoc) | 73 | 7,5 % |
| *los otros 13 jobs, juntos* | *158* | *16 %* |

**Los 3 jobs de build son el 84 % de los 967 s de runner** (no del tiempo de
pared: 809 s > 638 s, así que como porcentaje de pared no tiene sentido), y
ninguno de los tres puede estar afectado por un PR que sólo toque `Formula/`,
`dev-tools/` o un JSON. Los 3 required (`links` 7 s, `schema` 15 s, `index` 15 s)
son **37 s**.

**Paralelismo real: 967 s de runner / 638 s de pared = 1,52×**, y los 16 jobs no
arrancan juntos (`06:57:34` el primero, `07:00:52` el último: **198 s de
arranque escalonado** dentro de un solo run). **`Python examples` manda la
pared: 466 de los 638 s (73 %).** Abrir los 16 jobs a cada PR sube la presión de
cola (no hay *race* nueva: el `group` de `concurrency` y su
`cancel-in-progress` no cambian, y sólo cuenta el SHA último — ver `FIND-764`).
El límite de jobs concurrentes de la cuenta no lo expone la API: **no medido**,
se declara como riesgo.

Alcance del cambio de coste: **sólo** los PRs que hoy no disparan nada (los 426
ficheros de arriba). Un PR de `docs/`, `src/` o `vantadb-*` ya disparaba todos
los jobs: para ésos el coste es **idéntico**. Es decir: el subconjunto que hoy
paga ∞ pasa a pagar ~10 min de CI, y sólo **178** de esos 426 ficheros ganan
algo (los que `ci-rust` sí cubre); los **248** que tampoco ve `ci-rust.yml`
siguen inmergeables y ahora además queman 16 jobs (§"El otro tercio").

El repo es **público** (`gh api repos/ness-e/Vantadb → private: false`): los
minutos de Actions no se facturan; el coste real es latencia y ocupación de
runner. **Techo declarado:** si esa latencia molesta, el filtrado fino va
**dentro** de los 3 jobs de build con su propio `if:` + un job de diff de paths
(fail-closed), nunca en el disparador. No se hace ahora: añade un job, un
`checkout` con `fetch-depth: 0` y una superficie nueva donde un fallo del diff
dejaría el gate de rustdoc/doctests apagado en silencio.

## Impacto mapeado (Regla 0)

- **Leídos completos:** `.github/workflows/docs-gates.yml` (694 líneas en `HEAD`:
  `on:` `:17-68`, los 16 jobs con sus `name:`/`if:`) ·
  `.github/workflows/ci-gate.yml` (105 líneas: `REQUIRED` `:40-60`, política
  EST-03 `:61-99`) · `docs/dev/tasks/FIND-763.md` (208 líneas) ·
  `docs/dev/avance/activo/ci-cd.md` §`FIND-763` `:1075-1098` y §Rulesets
  `:958-966` · `docs/dev/Backlog.md:833`.
- **Referencias salientes:** ninguna ejecutable. `docs-gates.yml` no se importa
  desde ningún sitio; `ci-gate.yml` es `workflow_call` consumido por los
  workflows heavy (`needs: ci-gate`).
- **Referencias entrantes:** (a) los `name:` de los jobs son los check-runs que
  el ruleset `develop` (`23692587`) y `main` (`16844590`) exigen por string;
  (b) el job `actionlint` de este mismo fichero cuenta **ficheros** de workflow
  contra `docs/dev/operations/CI_POLICY.md`, no jobs → este cambio no lo mueve.
- **Veredicto:** se borra un bloque `paths:` de 21 líneas y se añade un
  comentario. Cero jobs nuevos, cero `name:` tocados, cero cambios en `ci-gate.yml`
  (el `REQUIRED` no hay que ajustarlo: sigue siendo 18 = ruleset de `main` + 1 de
  sanidad, y los 19 contexts ya se reportan en su propia rama).

## Steps

| # | Step | Estado |
|---|---|---|
| 1 | Medir el hueco real (426/2351, por directorio) y el coste de los 16 jobs | ✅ |
| 2 | Verificar la distinción workflow-no-corre vs job-saltado contra la doc oficial | ✅ |
| 3 | `docs-gates.yml`: quitar `paths` de `pull_request` (+ comentario con la medición y el techo) | ✅ |
| 4 | `actionlint` en los dos ficheros + coincidencia literal de los 3 `name:` contra el ruleset | ✅ |
| 5 | Cierre: task file, fila del Backlog, entrada de avance, commit local | ✅ |

## YAML final

```yaml
on:
  push:
    branches: [main, develop]
    paths:                      # sin cambios: en push no hay required check
      - 'docs/**'
      - … (21 patrones, intactos)
  pull_request:
    branches: [main, develop]
    # SIN `paths:` — ver el comentario de cabecera del bloque
  schedule:
    - cron: '23 7 * * 1'
  workflow_dispatch:
```

## Verificación

| Comprobación | Comando | Resultado |
|---|---|---|
| lint de workflows | `actionlint .github/workflows/docs-gates.yml` | **exit 0** (baseline previo: 0) |
| lint de workflows | `actionlint .github/workflows/ci-gate.yml` | **exit 0** (baseline previo: 0) |
| `pull_request` sin `paths` | parseo YAML (`js-yaml`) | `on.pull_request` = `{branches: [main, develop]}` — **sin** clave `paths` |
| `push` conserva el filtro | parseo YAML | `on.push.paths` = 21 patrones (idénticos a `HEAD`) |
| el número de jobs no cambia | parseo YAML | **16** antes y después |
| los 3 `name:` coinciden con el ruleset | comparación por code points + longitud (`len()`, Python) | `22 == 22`, `43 == 43`, `29 == 29`, code points idénticos |
| cobertura de paths | `git ls-files -- ':(exclude)…'` | el filtro de `push` deja 426 fuera; `pull_request` ya no filtra |
| enlaces / schema de docs | `node scripts/docs/check-links.mjs` · `check-docs.mjs` | ver §Cierre |

### Coincidencia literal de los 3 contexts

```
docs-gates.yml jobs.docs-links.name    -> "Internal links resolve"                     22 chars
ruleset 23692587 required_status_checks -> "Internal links resolve"                    22 chars

docs-gates.yml jobs.docs-schema.name   -> "Frontmatter schema, kind agreement, orphans" 43 chars
ruleset 23692587 required_status_checks -> "Frontmatter schema, kind agreement, orphans" 43 chars

docs-gates.yml jobs.docs-index.name    -> "Generated indexes are current"              29 chars
ruleset 23692587 required_status_checks -> "Generated indexes are current"              29 chars
```

Comparado por `len()` y por lista de `ord()` por carácter sobre los dos lados
(parseo de `docs-gates.yml` + `gh api .../rulesets/23692587`), no a ojo.

## El otro tercio: los 10 contexts que salen de `ci-rust.yml` (prohibido acá)

De los 13 contexts required de `develop`, **10 salen de `ci-rust.yml`** y ese
fichero **tiene el mismo filtro** (`pull_request.paths`, 22 patrones). Ese
fichero no es escribible en el alcance de esta fila, así que el hueco se
**reduce**, no se cierra — y el residual es **mucho mayor que el hueco que esta
fila cierra**:

| Conjunto de ficheros | Ficheros | Contexts que quedan en `Pending` |
|---|---:|---|
| fuera de `docs-gates.paths` (**antes** del fix) | 426 (18,1 %) | 3 (los de `docs-gates`) |
| fuera de `ci-rust.paths` | **1281 (54,5 %)** — **957 de ellos bajo `docs/`** | 10 (los de `ci-rust`) |
| **fuera de la unión de los dos** (ni un workflow dispara) | **248 (10,5 %)** — `dev-tools/**` 48, `examples/**` 38, `skills/**` 26, `evals/**` 25, `benchmarks/**` 21, `assets/**` 16, `fuzz/**` 12, `.github/**` fuera de `workflows|actions` 11, `embeddings/**` 8, `vhs/` 7 … | **13** (3 + 10) |

(GitHub evalúa `paths` con minimatch —`*` no cruza `/`, `**` sí—, no con
pathspec de git: implementado sobre esa semántica y re-verificado con una
segunda implementación independiente, ambos dan 426 / 1281 / 248 / 178. Un
cálculo previo con `git ls-files -- ':(exclude)…'` daba 246 y 180: **estaba mal**,
porque `git` y minimatch no coinciden en dos ficheros.)

**Consecuencia que hay que leer entera:** `docs/**` **no** está en los 22
patrones de `ci-rust.yml`, así que **un PR que sólo toque `docs/` reporta los 3
contexts de este fichero y sigue dejando los 10 de `ci-rust` en `Pending`**. El
caso docs-only —el más frecuente del repo— **no queda arreglado por esta fila**;
queda mejor, no cerrado.

Ganancia neta de este cambio: **178 ficheros** que sólo `ci-rust` cubría
(`integrations/**` 81, `vanta-proxy/**` 61, `benches/**` 27, y 9 ficheros de
raíz: `build.rs`, `deny.toml`, `osv-scanner.toml`, `rust-toolchain.toml`,
`release-plz.toml`, `.gitleaks.toml`, `.secretlintrc.json`,
`.secretlintignore`, `.config/nextest.toml`) pasan a reportar los 13 contexts.
Los **248** restantes siguen topando con **13**: **mismo fix, otro fichero** —
`ci-rust.yml`, fuera de alcance acá, y **sin ticket propio** (§Hallazgo sin
ticket).

### Hallazgo sin ticket (corrige la fila del plan)

El alcance de esta tarea declaró `ci-rust.yml` prohibido *"porque `FIND-762` lo
mide"*. **Verificado y falso**: `FIND-762` es *"el `pre-push` formatea con el
toolchain stable 1.95.0 y `ci-rust.yml:117` lo hace con nightly"* — un problema
de **toolchain de `fmt`**, no de `pull_request.paths`; y **no tiene task file**
(`docs/dev/tasks/FIND-762.md` no existe). La razón real de la prohibición es que
`FIND-762` está **editando ese fichero** en paralelo (conflicto de carril), que
es motivo suficiente pero **no mide nada de esto**.

Los 10 contexts filtrados de `ci-rust.yml` (1281 ficheros, 957 de ellos `docs/`)
**no los está midiendo nadie**: necesitan fila propia. No la abro acá porque el
alcance de esta tarea dice "sólo tu fila" en `Backlog.md` y otra lane está
creando filas en el mismo fichero (colisión de ID); queda en
`queda_pendiente` del RESULTADO para el orquestador.

## Lo que NO se puede verificar desde acá

- **No se puede abrir el PR de prueba** (ni disparar un `pull_request`): el
  contrato real es `gh pr checks` sin ningún `Expected` a los 10 min, y eso exige
  un PR real. El trabajo cierra en **commit local**, sin push.
- **Los 13/13 no quedan cubiertos por este cambio**: 10 contexts salen de
  `ci-rust.yml`, que conserva su `pull_request.paths`. Quedan **1281 ficheros
  (54,5 %)** —incluidos **todos los de `docs/`**— cuya PR sigue sin reportar
  esos 10, y **248 (10,5 %)** que no disparan **ninguno** de los dos workflows y
  dejan los 13 en `Pending`. Ver §"El otro tercio".
- Por lo mismo no se observa el coste de la primera corrida con el filtro
  quitado: el número de 967 s / 10m38s es del run `45771790202`, que **ya**
  corría los 16 jobs (disparado por `docs/**`), así que es el coste correcto de
  la configuración nueva, pero medido en un run que habría corrido igual.
- `actionlint` local es **v1.7.12**; CI pinea **1.7.7**. La sintaxis usada
  (borrar un filtro) existe en ambas.

## NOTICED BUT NOT TOUCHING

- **`docs/dev/operations/CI_POLICY.md:799-800`** dice que `docs-gates` corre en
  "push/PR a `main`+`develop` **que toquen docs, scripts o workflows**". Tras
  este cambio eso sólo es cierto en `push`. El fichero está **prohibido** por el
  alcance de esta tarea (hard scope, `FIND-752` en vuelo) → sync pendiente. Es
  deuda **creada por este cambio**, no preexistente: editarlo es seguro (el job
  `actionlint` sólo compara el **conteo de ficheros** de `:16`, 20 == 20, contra
  `find .github/workflows -name '*.yml'`; no lee triggers), así que sólo lo
  bloquea el alcance, no el riesgo.
- **Red de `docs-gates` en `develop`, cronología** (para que nadie lea una foto
  vieja): run `45804272736` (`10:45`, `ab3dae3`) → rojo en 3 jobs: el gate de
  `FIND-763` cazó `public documentation for \`create_snapshot\` links to private
  item \`is_mutable_after_snapshot\`` (exit 101 en 57 s), más
  `Internal links resolve` y `Generated indexes are current`. Run `45821094074`
  (`12:35`, `3a7e225` = HEAD de `develop`) → **success en los 16 jobs**. O sea:
  el árbol **volvió a verde** entre las dos fotos y el rojo **no** es una
  condición vigente del repo. Lo que sí queda: con el filtro quitado, cualquier
  rojo futuro de `docs-gates` aparecerá en **todos** los PRs en vez de sólo en
  los de docs.

## Review (P2-01)

Tier **fast** por glob (`.github/workflows/**`, `docs/dev/**` → "cualquier otro
path"), pero el contrato es de nombres de check: **`vanta-review`, contexto
fresco, report-only, no implementó nada**.

- **Enfoque:** re-corrió `actionlint` en los dos ficheros (0, y 0 también en
  `HEAD`), parseó los dos YAML por su cuenta (`on.pull_request` sin `paths`,
  `push.paths` 21 e idénticos, 16 jobs, `name:`/`if:`/`needs:`/`concurrency`
  idénticos), comparó los 3 contexts por `len()` y `ord()`, re-midió el run
  `45771790202` (16 / 967 s / 638 s / 466-270-73 / otros 13 = 158 s / required
  37 s), **recalculó las cifras de paths con una segunda implementación
  independiente de minimatch**, verificó las citas de la doc de GitHub verbatim,
  y escaneó los 20 workflows buscando `name:` duplicados entre ficheros.
- **Veredicto: `changes-required` → `approve` tras corrección.** 4 Blocking, y
  **ninguno en el YAML**: el artefacto pasó intacto ("correcto, mínimo, sin jobs
  nuevos, sin `name:` tocados, `push` intacto, y su comentario es exacto en todo
  lo verificable"). Los 4 Blocking eran del **registro** y se corrigieron todos:
  - **B-1** — `246`/`180` eran **248**/`178` (mi cálculo con `git ls-files
    --exclude` no coincide con minimatch en 2 ficheros; la ganancia real es
    `integrations 81 + vanta-proxy 61 + benches 27 + 9 de raíz`). Corregido en los
    5 sitios, con el método de medición declarado.
  - **B-2** — "fuera de `ci-rust.paths` = 426" era **1281 (54,5 %)**, y **957 de
    ellos bajo `docs/`**: como `ci-rust.yml` no lleva `docs/**`, **un PR sólo-docs
    sigue dejando los 10 contexts de `ci-rust` en `Pending`**. Subestimaba el
    residual 5,2× y la conclusión cambia: esta fila **no** arregla el caso
    docs-only. Corregido, con la tabla completa.
  - **B-3** — la delegación a `FIND-762` era falsa: `FIND-762` es *"el
    `pre-push` formatea con stable y `ci-rust.yml:117` con nightly"* (toolchain
    de `fmt`) y **no tiene task file**; no mide `paths`. La prohibición de
    `ci-rust.yml` es **conflicto de carril**, no medición. Reescrito y elevado a
    §"Hallazgo sin ticket".
  - **B-4** — faltaba esta sección antes de `status: completed`. Añadida.
  - **Medium corregidos:** M-1 (`inmergeable` = para quien no tiene bypass; el
    ruleset tiene `bypass_mode: always` para admin, y 4 PRs sólo-docs entraron sin
    run), M-3 (paralelismo 1,52×, `Python examples` = 73 % de la pared, presión de
    cola declarada como riesgo no medido), M-5 (la foto "rustdoc ROJO ahora mismo"
    estaba **caducada**: `45821094074`, `3a7e225` = HEAD, verde en los 16),
    L-1 (84 % de los 967 s **de runner**, no de pared), L-4 (`web/` sobrevive en
    `refs/stash` + 3 tags).
  - **Medium no aplicado, con razón:** **M-2** pide sincronizar
    `CI_POLICY.md:799-800`; el reviewer confirma que editar ese fichero es seguro
    (el gate de inventario sólo lee el conteo de `:16`), pero está **prohibido
    por el alcance duro de esta tarea** → queda como deuda declarada, no como
    omisión.
- **Lo que el reviewer confirmó que NO es problema:** dependabot **mejora** (los
  3 required no tienen guard de actor; 10 de 16 jobs se saltan), el gate de
  inventario no se mueve (cuenta ficheros), no hay *race* nueva de
  `cancel-in-progress` (mismo `group`), "el coste sólo cambia para los PRs que hoy
  no disparan" es cierto, y **0 `name:` duplicados** entre workflows para los 3
  contexts. También verificó que **no hace falta `merge_group`** (ningún ruleset
  tiene regla de merge queue y ningún workflow lo usa).

## Alcance (escritura)

`.github/workflows/docs-gates.yml` · `docs/dev/tasks/FIND-765.md` ·
`docs/dev/Backlog.md` (sólo la fila `FIND-765`) · `docs/dev/avance/activo/ci-cd.md`.

`ci-gate.yml` **no** se toca: el `REQUIRED` no hay que ajustarlo.

**Prohibido y respetado:** `.githooks/**` (`FIND-762` en vuelo) ·
`.github/workflows/ci-rust.yml` (`FIND-762` está editando ese fichero en
paralelo — **no** porque mida sus `paths`: ver §"Hallazgo sin ticket") ·
`src/**` · `vanta-memory/**` · `vantadb-mcp/**` · `dev-tools/**` (se
**ejecuta**, no se edita) · `docs/dev/operations/CI_POLICY.md` ·
`docs/dev/workflow/**` · `docs/dev/plans/**` · `.opencode/**`.
