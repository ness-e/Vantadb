---
title: "FIND-764 - el suite que nunca corrió: la concurrencia por ref deja de matar runs de `push`"
kind: task
status: completed
description: "`ci-rust.yml` cancelaba en vuelo también en `push`: `cd7d8c5f7` quedó en `develop` con el run `cancelled` — `Tests (Windows)` a los 35m23s por el push siguiente, `Tests (Linux)` a los 17m53s sin ejecutar un test (1 de 9 runs). Fix: `cancel-in-progress` solo en `pull_request`. Coste medido: 1 de 8 huecos espera; lista de SHAs publicada."
tags: [vantadb, ci, github-actions, concurrencia]
---

# FIND-764 - el suite que nunca corrió

## Contexto

`ci-rust.yml` declaraba, a nivel de workflow:

```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

El grupo es el **ref**, y el ref significa cosas distintas por evento. En `push`
es `refs/heads/develop`: el run que se cancela es el **único de ese workflow**
que va a reportar por ese SHA, y el SHA se queda en la historia de la rama.
Resultado: commits integrados cuyo suite completo nunca se ejecutó,
indistinguibles en el resumen de un run de "el diff rompía algo".

`FIND-771` (`edcd0fb65`) ya había resuelto exactamente esta pregunta para
`docs-gates.yml`, con el razonamiento por evento. Esta tarea **reusa** ese
criterio; no lo reinventa.

### Lo medido, con el SHA en la mano

| Dato | Valor | Fuente |
|---|---|---|
| SHA afectado | `cd7d8c5f78519eccec7255aea36851b642ecc8df` (en `develop`, ancestro del HEAD) | `git merge-base --is-ancestor` |
| El run | `45794331830`, **`cancelled`**, `09:41:37Z` → `10:31:14Z` = **49m37s** | `gh run view --json createdAt,updatedAt,conclusion` |
| `Tests (Windows)` | **cancelled a los 35m23s** (`09:55:50Z` → `10:31:13Z`), **atribuida**: la anotación del check-run dice `Canceling since a higher priority waiting request for CI: Rust — Build & Lint + Tests-refs/heads/develop exists` | `gh api .../check-runs/139828474272/annotations` |
| `Tests (Linux)` | **cancelled a los 17m53s** (`09:55:50Z` → `10:13:43Z`), murió **compilando** (`vantadb_py`), sin `error:` ni `test result` — **causa NO atribuida** | `gh api .../runs/45794331830/jobs` + log |
| Quién pudo matar a Linux | **nadie que exista**: entre `09:41:37Z` y `10:30:58Z` no hay **ningún** run de `ci-rust` en ninguna rama; su anotación es la genérica `The operation was canceled.` | `gh api "repos/…/actions/runs?created=2026-11-26T09:00:00Z..2026-11-26T11:30:00Z"` |
| Frecuencia del día | **1 de 9** runs de `develop` (los otros 8 `success`) | `gh run list --workflow ci-rust.yml --branch develop --limit 30` |

**Corrección de la premisa del ticket (el brief decía "lo canceló el push
siguiente, 49 min después").** El push siguiente (`337ee7963`, creado
`10:30:58Z`) **sí** mató un job de ese SHA — `Tests (Windows)`, 15 s después de
existir, y la anotación lo prueba nombrando el grupo. Pero **no** mató a
`Tests (Linux)`: ése murió **17m21s antes** de que ese run existiera, sin nada
en la ventana que lo explique. Son **dos** cancelaciones con **una** causa
conocida, y el resumen del run escribe la misma palabra para las dos.

Eso no debilita el hallazgo: lo **ensancha**. El fix cierra el mecanismo
demostrado (concurrencia por ref); el segundo modo de fallo —un `cancelled`
sin causa, que deja un SHA sin suite igual— es la razón por la que la lista de
§(C) se publica como **registro** y no como reemplazo del arreglo.

**Corrección de la segunda premisa del brief:** avisaba que un `rust-scope`
cancelado deja los contexts **ausentes**. En el SHA medido los **61** check-runs
existen y están todos `completed`. `rust-scope` nació en `FIND-770`
(`2b282eed7`), que todavía no está en `origin/develop`, así que su comportamiento
bajo cancelación **no se observó**: no se puede sin provocar una cancelación, y
este contrato lo prohíbe. Queda declarado, y deja de importar con este fix.

**Inventario del SHA `cd7d8c5f7`** (`gh api …/check-runs?per_page=100` → `total_count: 61`):
**53 `success` · 5 `skipped` · 2 `cancelled` · 1 `failure`**. El `failure` es
`Generated indexes are current`, de otro workflow y de las `09:41:50Z`; no es de
esta tarea, pero está en el mismo SHA y §Deuda explica por qué este commit
reproduce ese rojo.

## Impacto mapeado (Regla 0)

- **Leído completo:** `.github/workflows/ci-rust.yml` (1127 líneas antes del
  cambio: los 24 patrones de `on.push.paths` `:6-30`, el bloque `concurrency`
  `:59-61`, el job `rust-scope` `:64-160`, `clippy` con `needs: [rust-scope]`
  `:216-250`, `test` = `Tests (Linux)` `:401-445` con
  `needs: [fmt, clippy, secret-scan]`, y los 23 jobs restantes con sus
  `name:`/`if:`/`needs:`/`concurrency`).
- **Los 5 bloques `concurrency` de job** (`secret-scan` `:170`, `fmt` `:200`,
  `clippy` `:234`, `semver-checks` `:277`, `public-api-snapshot` `:320`) se
  leyeron pero **no se analizaron** en la primera versión de este fichero:
  llevaban `cancel-in-progress: true` y contradecían §12. Corregido (ver §(B)).
- **Referencias salientes:** ninguna ejecutable; ningún fichero importa este
  workflow. `concurrency` es directiva de plataforma, no contrato con código.
- **Referencias entrantes (las que deciden):**
  1. `ci-gate.yml` compara los `name:` de los jobs por **string** y falla en
     `failure|timed_out|cancelled|action_required`: este cambio **no toca ningún
     `name:`** ni ningún step.
  2. Los rulesets de `develop`/`main` declaran required checks por ese `name:`
     (`Tests (Linux)`, `Tests (Windows)`, `Tests (macOS)`, `Format Check`,
     `Clippy Lints` entre ellos): intactos.
  3. `docs-gates.yml` ya lleva la misma expresión desde `FIND-771` y **no tiene**
     bloques job-level con `true`: **no se toca**.
- **Veredicto:** una expresión en seis directivas `concurrency`. Cero símbolos,
  cero jobs, cero steps, cero cambios de nombre. El riesgo no es romper algo: es
  **elegir mal qué lado se serializa** (ver §(B)).

## (A) Medición de la ventana y de la cadencia

Población: los **9** runs de `ci-rust.yml` sobre `develop` del 2026-11-26
(`00:12:15Z` → `12:35:36Z`), con `gh run view <id> --json jobs,createdAt,updatedAt`.

| Run | SHA | Inicio | Duración del run | `Tests (Linux)` |
|---|---|---|---:|---:|
| `45727034307` | `4af909977` | 00:12:15Z | 31m39s | 10m06s |
| `45732896745` | `f5c027f0e` | 01:01:46Z | 30m22s | 9m10s |
| `45750538900` | `0094f0f6a` | 03:41:39Z | 39m32s | 9m50s |
| `45771789995` | `f2e968807` | 06:57:23Z | 57m08s | 12m51s |
| `45782445975` | `ebc83ffa9` | 08:21:17Z | 30m51s | 12m34s |
| `45794331830` | **`cd7d8c5f7`** | 09:41:37Z | **49m37s (cancelled)** | **17m53s — cancelled** |
| `45802050750` | `337ee7963` | 10:30:58Z | 58m52s | 12m27s |
| `45815405021` | `2f8ea4d76` | 11:58:56Z | 24m13s | 9m54s |
| `45821093876` | `3a7e225ad` | 12:35:36Z | 69m35s | 10m58s |

- **La ventana es el RUN, no el job:** `Tests (Linux)` tarda 9m10s–17m53s, pero
  el run lo marcan `Tests (Windows)` y `Tests (macOS)`. Duración medida del run:
  **24m13s–69m35s, mediana 39m32s**.
- **Cadencia:** 8 huecos entre los 9 pushes — 49m31s · 2h39m53s · 3h15m44s ·
  1h23m54s · 1h20m20s · **49m21s** · 1h27m58s · 36m40s. **Mediana 1h22m07s.**
- **Cuántos huecos caen dentro del run anterior** (comparando el `updatedAt` del
  run con el `createdAt` del push siguiente): **1 de 8** — el de 49m21s, que es
  justamente el que hoy mató `Tests (Windows)`. Los otros 7 llegaron con la cola
  libre: **12m27s–2h36m12s** de margen tras el `updatedAt` del anterior.
- **Cuántos pares consecutivos habrían descartado un pendiente** (`queue: single`
  cancela el pendiente anterior): **0 de 7** — **derivado**, no medido: requiere
  que dos huecos seguidos caigan dentro de la ventana, y con 8 huecos hay 7
  pares. Lo medido es el punto anterior.

## (B) Decisión — opción (c), la expresión por evento

```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: ${{ github.event_name == 'pull_request' }}
```

…en el bloque de workflow **y** en los 5 bloques de job.

| | (a) disciplina: no pushear con un run en vuelo | (b) marcar el SHA como no verificado | **(c) expresión por evento** |
|---|---|---|---|
| ¿Arregla el mecanismo? | no: depende de que alguien consulte antes de cada push | no: documenta el daño después | **sí: en `push` el run que reporta por el SHA ya no se cancela** |
| ¿Tiene gate? | **no** | no | **sí — la directiva; 0 líneas de código de usuario** |
| Coste medido | 0 | 0 | **1 de 8 pushes espera** ≤ el run anterior (≤69m35s, mediana 39m32s) |
| SHAs sin suite | 1 de 9 hoy | 1 de 9 documentado | **0 esperados a esta cadencia** |
| PRs afectados | no | no | **no**: `refs/pull/N/merge` es otro grupo |

**Por qué (c) y no (a):** (a) es el pre-mortem servido tal cual — "elegir (a) y
que nadie la cumpla porque no tiene gate". Un carril que empuja cada ~15 min no
va a consultar `gh run list` antes de cada push, y el fallo no se nota: el
resumen dice `cancelled`, que es exactamente lo que un humano no va a leer como
"tu suite no corrió". La disciplina se queda escrita igual (§RULES 12) como
verificación de comportamiento.

**Por qué (c) y no (b):** (b) produce una lista que envejece en el mismo commit
que la escribe y no impide la siguiente cancelación. Se hace **además** — la
lista de §(C) es el contrato mecánico de esta tarea — pero como registro. La
cancelación **sin causa** de `Tests (Linux)` en el mismo SHA demuestra por qué
hace falta el registro aparte: hay un segundo modo de fallo que este cambio no
toca.

**Por qué la expresión y no `false` a secas (opción (i) de `FIND-771`):**
`false` en todo serializa también los fix-up de un mismo PR, que comparten ref
(`refs/pull/N/merge`): un push de corrección esperaría **hasta 69m35s** para
reportar sobre un head que ya nadie va a mergear. Cancelar ahí es estrictamente
mejor, y es gratis.

**Por qué no la identidad del run en el grupo (opción (ii)):** deja correr N
runs sin límite. Un burst de 3 pushes a `develop` = 3 × (Windows ~57m + macOS
~26m + Linux ~11m) de runners para reportar SHAs ya superados.

**Los bloques de job (revisión B3).** Llevaban `cancel-in-progress: true` y
§12, tal como estaba escrita, los prohibía. Con el grupo de workflow serializando
el ref son **inertes hoy** (solo un run por ref ejecuta jobs, así que
`fmt-refs/heads/develop` no tiene dos contendientes), así que no eran un defecto
vivo — pero dejaban escrita una contradicción y una vía latente si el grupo de
workflow cambia. Se les puso la misma expresión: 5 líneas, cero cambio de
comportamiento, y la regla pasa a ser verificable con un `grep` sobre el fichero
entero.

**Techo declarado:** en `push` GitHub mantiene **uno en curso + uno pendiente**
por grupo (`queue: single`, default): un tercer push cancela el **pendiente**,
cuyo SHA ya está superado por el que lo cancela. Medido hoy: 1 de 8 huecos
espera; **derivado**: 0 de 7 pares habrían descartado un pendiente. Para que cada
push reciba su run existe `concurrency.queue: max` (hasta 100 pendientes), **no
usado**: con runs de 24–69 min, encolar más de uno es encolar SHAs muertos.
`workflow_dispatch` y `schedule` evalúan la expresión a **false** (no cancelan
un run en curso); eso **no** protege el slot pendiente — `queue: single` desplaza
un pendiente diga lo que diga el flag — y un `workflow_dispatch` sobre una rama
comparte `refs/heads/<rama>` con los push de esa rama.

## (C) La lista de SHAs, publicada

**Último SHA con `Tests (Linux)` ejecutado y verde: `3a7e225ad`** (run
`45821093876`, `10m58s`, `12:48:04Z` → `12:59:02Z`).

SHAs de la población medida, con su estado real (`gh run view <id> --json jobs`):

| SHA | Run | `Tests (Linux)` |
|---|---|---|
| `4af909977` | `45727034307` | ejecutado, success — 10m06s |
| `f5c027f0e` | `45732896745` | ejecutado, success — 9m10s |
| `0094f0f6a` | `45750538900` | ejecutado, success — 9m50s |
| `f2e968807` | `45771789995` | ejecutado, success — 12m51s |
| `ebc83ffa9` | `45782445975` | ejecutado, success — 12m34s |
| **`cd7d8c5f7`** | `45794331830` | **cancelado a los 17m53s — nunca ejecutó un test** (causa no atribuida) |
| `337ee7963` | `45802050750` | ejecutado, success — 12m27s |
| `2f8ea4d76` | `45815405021` | ejecutado, success — 9m54s |
| `3a7e225ad` | `45821093876` | ejecutado, success — 10m58s (**último verde**) |

SHAs en `origin/develop` **posteriores** al último verde, que no están en la
tabla porque **no tienen run**:

| SHA | ¿Run de `ci-rust`? | `Tests (Linux)` |
|---|---|---|
| `49ced681e` (`FIND-765`, `docs-gates.yml`) | **ninguno** | **no medido** — el fichero no está en `on.push.paths`, así que el workflow no se dispara |
| `c8e73e412` (docs: backlog/CI_POLICY/index) | **ninguno** | idem |

**Conclusión honesta:** hay **un** SHA sin suite verificada en la población
medida (`cd7d8c5f7`), y **dos** estados más que el resumen de runs tampoco
distingue y que se declaran en vez de contarse como verde: `cancelled` sin causa
(Linux) y **sin run por diseño** (path filter).

**No se puede provocar una cancelación** (el push es acto del owner, Regla 7):
la prueba viva de que la expresión ya no cancela la publicará el primer push
posterior a este commit.

## YAML final

```yaml
# nivel workflow
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: ${{ github.event_name == 'pull_request' }}

# nivel job — los 5 bloques que ya existían, misma expresión
concurrency:
  group: fmt-${{ github.ref }}
  cancel-in-progress: ${{ github.event_name == 'pull_request' }}
```

## Steps

| # | Step | Estado |
|---|---|---|
| 1 | Medir la población (9 runs de hoy, jobs por run, `createdAt`/`updatedAt`) | ✅ |
| 2 | Confirmar qué deja una cancelación en los check-runs del SHA `cd7d8c5f7` (61: 53/5/2/1) | ✅ |
| 3 | Elegir (a)/(b)/(c) con números de la ventana y de la cadencia | ✅ (c) |
| 4 | `concurrency` por evento en workflow **y** en los 5 bloques de job | ✅ |
| 5 | Regla durable en `RULES.md` §12 (la doctrina, con su `Verify`) | ✅ |
| 6 | Contrato mecánico: `actionlint` 0 · YAML parseado · lista de SHAs publicada | ✅ |
| 7 | Review P2-01 (una ronda) + `ocr-review.ps1` | ✅ (3 Blocking + 5 Medium + 5 Low, cerrados) |
| 8 | Cierre: este task file, fila fuera del Backlog, entrada en `ci-cd.md`, commit | ✅ |

## Verificación (contrato mecánico)

| Comprobación | Comando | Resultado |
|---|---|---|
| lint de workflows | `actionlint .github/workflows/ci-rust.yml` | **exit 0** (baseline previo 0) |
| `concurrency` final | `js-yaml` sobre el fichero | workflow = `{"group":"${{ github.workflow }}-${{ github.ref }}","cancel-in-progress":"${{ github.event_name == 'pull_request' }}"}`; los 5 jobs (`secret-scan`, `fmt`, `clippy`, `semver-checks`, `public-api-snapshot`) con la misma expresión |
| ninguna ocurrencia en `true` | `grep -n 'cancel-in-progress' .github/workflows/ci-rust.yml` | **7** líneas con la cadena: **6 directivas** (workflow `:91` + los 5 jobs) y **1** mención en el comentario; **0** con el valor `true` (la `Verify` de §12 pasa) |
| jobs y `name:` intactos | `js-yaml` → `Object.keys(jobs).length` y `name:` vs `HEAD` | **24** jobs, **0** duplicados, **`sameNames=true`** |
| el resto del fichero no se movió | `git diff -U0 … \| grep '^[-+]'` | solo el bloque `concurrency` de workflow (comentario) y las 6 directivas |
| regla durable | `grep -n '^### 12' docs/dev/workflow/RULES.md` | presente, con `Must`/`Must not`/`Por qué`/`Verify`/`Techo` |
| SHAs publicados | `gh run list --workflow ci-rust.yml --branch develop --limit 30` + `gh run view <id> --json jobs` | tabla §(C) |
| atribución del caso | `gh api .../check-runs/<id>/annotations` | Windows: anotación con el grupo; Linux: `The operation was canceled.` |
| docs | `node scripts/docs/check-links.mjs` / `check-docs.mjs` | `check-links` **0 broken** (3602 links, 916 ficheros) · `check-docs` sin violación gating propia; `FIND-764.md` sale como *orphan* informativo, igual que los task files de otros carriles en vuelo |

**No ejecutado, declarado:** ningún `cargo` (carril CI, sin compilar: hay 2
carriles Rust en vuelo), ninguna cancelación provocada, y `gen-index.mjs` **no**
se corrió (prohibido por el contrato) — con su consecuencia escrita en §Deuda.

## Review (P2-01)

- **Revisor:** `vanta-review`, contexto fresco, report-only (no implementó).
- **Tier:** **fast** por glob (`.github/workflows/**` + `docs/dev/**`).
- **Cómo se probó:** re-corrió `actionlint` y el parseo `js-yaml`; re-derivó
  **todos** los números con `gh run list` / `gh run view --json jobs` /
  `gh api .../check-runs?...&per_page=100` y las anotaciones de los dos jobs
  cancelados; comprobó la ausencia de runs en la ventana 09:41:37Z–10:30:58Z
  (refuta la atribución original); cruzó los `name:` de los jobs contra `HEAD`
  (`sameNames`), `on.push.paths` = 24, `pull_request` sin `paths` (FIND-770) y
  la cadena `rust-scope → clippy`; y auditó la higiene de carril (7 paths ajenos
  sucios + `FIND-757/758.md` → 0 hits propios; `Backlog.md` `0 6`).
- **Veredicto: `changes-required`** (3 Blocking, 5 Medium, 5 Low) — **todos
  cerrados en esta misma ronda**:
  - **B-1 (Blocking, real y grave)** — el caso estaba mal atribuido: `Tests
    (Linux)` murió **17m21s antes** de que existiera el push que este fichero
    culpaba. Reescrito en los 4 ficheros: `Tests (Windows)` 35m23s con la
    anotación del grupo (la atribución buena) y `Tests (Linux)` 17m53s **sin
    causa atribuida**. El hallazgo queda **más** fuerte: dos cancelaciones, una
    con causa.
  - **B-2 (Blocking)** — 6 de las 9 duraciones de run estaban **+1:00** (bug de
    formato mío): máximo real **69m35s** (no 70m35s) y mediana **39m32s** (no
    40m32s). Corregidas la tabla, la prosa y el comentario del YAML. No cambia
    la decisión.
  - **B-3 (Blocking)** — §12 prohibía `cancel-in-progress: true` en un workflow
    con `push` y **el propio `ci-rust.yml` tenía 5** bloques de job así. Se les
    aplicó la misma expresión (inertes hoy, latentes mañana) y §12 ahora nombra
    el nivel job y pasa su `Verify` sobre el fichero entero.
  - **M-1** — el inventario de check-runs omitía el `failure` del SHA: ahora
    **61 = 53 success / 5 skipped / 2 cancelled / 1 failure**
    (`Generated indexes are current`), con la nota de que ese rojo lo reproduce
    este commit (§Deuda).
  - **M-2** — `gen-index --check` **exit 1** por el fichero de esta tarea: no se
    corrió (prohibido) pero la consecuencia ya no se esconde — §Deuda la declara
    bloqueante y cross-lane.
  - **M-3 / M-4** — `status: completed` y "**Commit:** este commit" sin commit, y
    §Review sin "Cómo se probó" ni Veredicto: esta sección es el cierre, y el
    commit existe al final de la misma ronda.
  - **M-5** — "0 de 7 pares" era una **inferencia** presentada como medición:
    ahora va etiquetada como derivada, con el denominador explicado.
  - **L-1** — `queue: single` desplaza un **pendiente** diga lo que diga el
    flag: corregido (`schedule`/`dispatch` no cancelan un run **en curso**, pero
    no protegen el slot pendiente).
  - **L-2** — el `Verify` de comportamiento no era decidible con
    `gh run list` (no trae jobs): ahora es `gh run view <id> --json jobs`.
  - **L-3** — la enmienda de §1 (`:50`) describía la concurrencia por ref sin el
    matiz nuevo: se le añadió "en `pull_request`; en `push` ya no se cancela
    (§12)".
  - **L-5** — "el único run que reporta por ese SHA" → "el único run **de este
    workflow**".
  - **L-4 (refutado, con medición)** — la review dijo que `ci-cd.md` perdió el
    newline final; `[IO.File]::ReadAllText(...).EndsWith("\n")` → **`True`**.
    No se cambió nada.

## Deuda / pendiente

- **Bloqueante, cross-lane: `gen-index --check` está rojo**, y lo estará mientras
  haya carriles en vuelo. `node scripts/docs/gen-index.mjs --check` → `[docs/index.md]
  "### Task files — 272" -> expected 275` (leído antes de este cierre). El drift
  es de **task files sin commitear** de otros carriles (`FIND-757.md`,
  `FIND-758.md`, `FIND-769.md` al cerrar: la lista cambia con cada carril), así
  que regenerar acá hornearía trabajo ajeno — y el contrato de esta tarea
  **prohíbe** `gen-index`. `Generated indexes are current` es un check
  **required**: el primer push de este commit saldrá rojo ahí hasta que el
  orquestador corra `gen-index.mjs --write` al cierre del wave. Es exactamente el
  `failure` que ya arrastra `cd7d8c5f7`.
- **`FAQ.md:26-30`** describe `cancel-in-progress` en dos formas (`true` en
  CI/PR, `false` en release) y **falta la tercera** (expresión por evento), que
  es hoy la forma de `docs-gates.yml` y `ci-rust.yml`. Fuera de la lista de
  escritura → **NOTICED BUT NOT TOUCHING**. Texto de reemplazo: *"`<expresión>`
  (gates con `push`): cancela en `pull_request` —el ref `refs/pull/N/merge` se
  reconstruye en cada push— y no cancela en `push`, donde el SHA no tiene otro
  run de ese workflow que reporte por él."*
- **`CI_POLICY.md`** no menciona la doctrina (solo casos de `concurrency`
  job-level, `:526-538`, `:870`). Dueño: el carril de `CI_POLICY`.
- **El segundo modo de fallo no está resuelto:** un `cancelled` sin causa (como
  el de `Tests (Linux)` en `cd7d8c5f7`) sigue dejando un SHA sin suite y este
  cambio no lo toca. Se detecta con la lista de §(C) y se mitiga en el release
  re-corriendo `Tests (Linux)` y **confirmando que ejecutó** (no sólo que está
  verde). Candidato a `FIND-*` si se repite.

- **Incidente de árbol compartido (declarado, reparado):** `9f8bc0ffd` se llevó
  **9 líneas de `docs/dev/tasks/FIND-768.md`**: `git read-tree HEAD` y `git commit`
  no son atómicos y el carril de `FIND-768` commiteó `4b3db3560` en medio (clase
  `FIND-753`). El helper verificó por contenido lo que le pedí y no podía ver el
  fichero ajeno; se detectó leyendo `git show --stat`. Restaurado en `4666b6467`
  (`+9/−0`).
- **Una exención de hook, declarada:** el commit se hizo con
  `SKIP=markdownlint-cli2`: el hook del env de `pre-commit` (v0.23.2) marca
  `Failed` con `Linting: 0 files` porque los `.md` están bajo `docs/dev/**`, que
  la config excluye; la **misma** invocación da **exit 0 / 0 issues** con v0.24.2.

## Alcance (escritura)

`.github/workflows/ci-rust.yml` · `docs/dev/workflow/RULES.md` ·
`docs/dev/tasks/FIND-764.md` · `docs/dev/Backlog.md` (solo la fila `FIND-764`) ·
`docs/dev/avance/activo/ci-cd.md`.

**Prohibido y respetado:** `snapshot-certification.yml` (`FIND-768` en vuelo) ·
`certification.yml` · `docs-gates.yml` · `src/**` (`FIND-758`) ·
`vantadb-mcp/**` (`FIND-757`) · `.opencode/**` · `dev-tools/**` ·
`docs/dev/plans/**`.
