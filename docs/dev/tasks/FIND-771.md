---
title: "FIND-771 - el gate required deja de morir en vuelo, y el step publica cuánto midió"
kind: task
status: completed
description: "`docs-gates.yml` cancelaba en vuelo sobre una ventana medida de 13m08s y desde FIND-763 hospeda un check required: un push dentro de la ventana dejaba el gate en `cancelled` sobre un commit cuyo doc nunca se midió. Fix: `cancel-in-progress` solo en `pull_request` (en push serializa) + step summary con el tamaño del árbol de doc y aserción de N>0."
tags: [vantadb, ci, github-actions, concurrencia, rustdoc]
---

# FIND-771 - el gate required deja de morir en vuelo

## Contexto

`FIND-763` (`1a318fa91`) cableó el gate de rustdoc y `FIND-765` (`49ced681e`)
hizo que `docs-gates.yml` reporte sus contexts en **todos** los PRs. El
resultado es que `Generate API reference (rustdoc)` es hoy **required** en
`develop` (14 contexts) y en `main` (19) — verificado por API el 2026-11-26.

El workflow que lo produce seguía con la directiva heredada de `FIND-764`:
`cancel-in-progress: true` a nivel workflow.

### Corrección de la premisa (la primera versión de este ticket estaba mal)

La fila del Backlog, el brief de la tarea y la primera versión de este fichero
decían que un required cancelado *"queda ausente del check set"* y que el PR se
queda en `Expected` para siempre. **Es falso, y se comprobó con la API sobre el
mismo run que el ticket cita** (`45802050777`, SHA
`337ee79636e21acca77b8d6fe16eb8060ecd1374`):

```
gh api repos/ness-e/Vantadb/commits/337ee796.../check-runs
→ "Generate API reference (rustdoc)" | completed | cancelled | 10:39:45 → 10:45:46
```

El check-run **existe y se reporta**: lo que queda es un `completed` con
conclusión `cancelled`. Un `cancelled` no es `skipped` (que GitHub cuenta como
Success) ni un `Expected` eterno — es un **no-success** que bloquea el merge
hasta que alguien pulse *Re-run*. En ese SHA, los 61 check-runs están todos
`completed`; ninguno falta.

El daño real, entonces, es distinto y sigue siendo real:

1. **6m01s de rustdoc tirados** (arrancó 10:39:45, murió 10:45:46) para no
   medir nada.
2. El commit queda con un required en `cancelled` → **merge bloqueado con un
   rojo que no habla del código**, y en `develop` la historia acumula commits
   cuyo gate de doc **nunca midió** — la clase de `FIND-764` (SHA sin suite
   verificada) encima de un gate **required**.

Se corrige el mecanismo en los tres sitios donde estaba escrito (este fichero,
el comentario del YAML y `ci-cd.md`): **no** hay "context ausente" ni deadlock
`Expected`; hay un `cancelled` reportado que exige re-run manual.

## Impacto mapeado (Regla 0)

- **Leído completo:** `.github/workflows/docs-gates.yml` (730 líneas en `HEAD`
  antes de este cambio: los 16 jobs con sus `name:`/`if:`, el bloque
  `concurrency` `:71-73`, el job `rustdoc` `:528-605` — `timeout-minutes: 30`
  `:535`, el step del gate `:569-572`, el artefacto `:580-594`, el `Summary`
  final `:596-605`) · `.github/workflows/ci-gate.yml` (el array `REQUIRED`
  `:36-58`, el consumidor que falla en `failure|timed_out|cancelled|
  action_required` `:66`) · `Cargo.toml:793-802` (`members`).
- **Referencias salientes:** ninguna ejecutable; ningún fichero importa este
  workflow.
- **Referencias entrantes (las que deciden):**
  1. Ruleset `develop` `23692587` → **14 contexts**, incluye
     `Generate API reference (rustdoc)`.
  2. Ruleset `main` `16844590` → **19 contexts**, ídem.
  3. `ci-gate.yml` compara por **string** con el `name:` del job (`FIND-763`):
     este cambio **no toca** el `name:` ni los steps del artefacto.
- **Veredicto:** una expresión en `concurrency` (nivel workflow) y el `run:` del
  step del gate. Cero símbolos, cero cambios en el artefacto, cero cambios en el
  nombre del check. El riesgo es el **opuesto** al habitual: no romper algo,
  sino elegir mal **qué lado** se serializa (ver (A)).

## Medición (la ventana, y cuántos pushes caben)

| Dato | Valor | Fuente |
|---|---|---|
| Ventana del run citado | **13m08s** (`12:35:36Z` → `12:48:44Z`) | `gh run view 45821094074` |
| Job `Generate API reference (rustdoc)` | **13m05s** (`12:35:38Z` → `12:48:43Z`) = **98 %** del run | idem |
| Resto de jobs (16 en total) | todos **≤ 4m10s** (`Doc-tests`, `12:35:39`→`12:39:49`) | idem |
| Mismo job, cache caliente | **1m13s** / **1m29s** / 11m19s en otros runs de hoy | `45771790202`, `45832678767`, `45732896798` |

La ventana del run **es** la del job rustdoc: 13m08s en frío, ~4 min con
sccache caliente. Se cita el peor del día, que es el que importa para el riesgo.

**Frecuencia real, medida y sin adornos.** Los **12** pushes a `develop` de hoy
(`createdAt` de cada run de `docs-gates`, 00:12:15 → 13:48:08) dejan **11
huecos**: 12m29s · 14m29s · 36m40s · 49m21s · 49m31s · **1h07m51s** ·
1h12m32s · 1h13m29s · 1h23m54s · 2h39m53s · 3h15m44s.

- **1 de 11** huecos cae **dentro** de los 13m08s (12m29s, 08:21:17→08:33:46).
- **1 de 11** queda al borde por 81 s (14m29s).
- **Mediana 1h07m51s**. Los bursts existen, pero el ritmo normal está **muy por
  encima** de la ventana.

Conclusión honesta: el riesgo es **real pero no constante** — y **ya se
materializó una vez hoy**:

| Run | Push | Qué pasó |
|---|---|---|
| `45802050777` | 10:30:58 | `Generate API reference (rustdoc)` **cancelled** a las **10:45:46**, tras **6m01s** corriendo (arrancó 10:39:45). El run entero murió a los **14m48s**, con el push de las 10:45:27 (`45804272736`) como causa |

## (A) Concurrencia — la opción elegida, y por qué

**Elegida: `cancel-in-progress: ${{ github.event_name == 'pull_request' }}`** —
no se cancela en `push`, sí en `pull_request`.

El grupo es `${{ github.workflow }}-${{ github.ref }}`, y `github.ref` significa
cosas distintas en cada evento:

| Evento | `github.ref` | ¿Cancelar pierde algo? |
|---|---|---|
| `push` | `refs/heads/develop` | **Sí.** El run que se mata es el **único** que va a reportar por ese SHA, y ese SHA **queda en la historia** de la rama |
| `pull_request` | `refs/pull/N/merge` | **No.** GitHub **reconstruye** ese ref en cada push del PR, así que el head revisado trae **su propio** run nuevo; cancelar el viejo tira trabajo de un SHA que ya nadie revisa |

Las tres opciones, con el mismo rasero:

| | (i) `false` en todo | (ii) identidad del run en el grupo | **(iii) expresión por evento — ELEGIDA** |
|---|---|---|---|
| `push`: ¿el gate de ese SHA se completa? | sí | sí | **sí** |
| `pull_request`: ¿el head revisado tiene run propio? | sí, pero **detrás de 13 min de cola** | sí | **sí, arranca ya** |
| Paralelismo | 1 en curso + 1 pendiente por ref | **sin límite** | 1 + 1 en `push`; cancelación en PRs |
| Coste de un fix-up en un PR | **espera hasta 13 min** | ninguno | ninguno |

**Por qué (iii) y no (i):** (i) resuelve el push a costa de meter la cola de 13
min en los PRs — el pre-mortem (a) de este ticket, servido tal cual. Y en un PR
esa cola **no compra nada**: cancelar el run viejo no pierde ningún context
porque el ref se reconstruye en cada push.

**Por qué (iii) y no (ii):** meter `github.sha` (o el run id) en el grupo
también protege el push, pero deja correr N runs a la vez sin límite: un burst
de 3 pushes triplica `doc-examples` (466 s) y `doctests` (270 s) para reportar
contexts de SHAs que ya nadie va a mergear.

**Techo declarado:** en `push`, GitHub mantiene como máximo **uno en curso + uno
pendiente** por grupo (`workflow-syntax §concurrency` y *Control the concurrency
of workflows and jobs*: con `queue: single`, que es el default, un nuevo
pendiente **cancela al anterior**). Un tercer push cancela el **pendiente**,
cuyo SHA ya está superado por el que lo cancela. Existe `concurrency.queue:
max` (hasta 100 pendientes) si algún día hace falta que **cada** push reciba su
run; hoy no se usa: con runs de 13 min, encolar más de uno es encolar SHAs
muertos.

## (B) No-vacuidad auditable — el resumen que mide

El step del gate no publicaba **cuánto** midió: mientras no hubiera un rojo, la
única forma de saber que midió era leer el log — y "verde" y "verde que midió"
se ven iguales desde la UI (la lección de `FIND-768`: *un verde que no sabe
cuántos eligió es un verde que no mide*).

1. **Número real del árbol producido:** `find target/doc -mindepth 2 -maxdepth 2
   -name index.html | wc -l` — los directorios de doc con `index.html`. El
   `index.html` de la raíz es un redirect y queda fuera por `-mindepth 2`.
2. **Aserción `test "$DOCS" -gt 0`:** un 0 es **rojo**, no "ok". El pre-mortem
   era un summary que siempre dice "ok" sin contar nada; esto es lo que lo
   impide.
3. **`0 warnings` con su razón escrita:** `RUSTDOCFLAGS=-D warnings` hace que el
   primer lint **aborte** el step, así que llegar al summary **es** la prueba de
   0 warnings de rustdoc.

**Alcance honesto del número (corregido en review):** son directorios con
`index.html` (targets de lib **y** de bin, per `cargo-doc(1)`), y cargo
**acumula** doc entre invocaciones, así que el `target/` que restaura el cache
puede aportar alguno. **No** es "crates que este step compiló" — y por eso el
summary dice *"Doc directories under `target/doc`"*, no "crates documented".
Para lo que sirve (que el verde no sea de 0) es exactamente lo que hace falta.
Techo: son warnings de **rustdoc**; `RUSTDOCFLAGS` no deniega los de `rustc` al
compilar las rlibs.

## YAML final

```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: ${{ github.event_name == 'pull_request' }}
```

```yaml
# job `rustdoc`, step índice 2 de 7 (donde estaba; el step NO se movió)
- name: Deny rustdoc warnings (doc gate)
  env:
    RUSTDOCFLAGS: "-D warnings"
  run: |
    cargo doc --no-deps --workspace --keep-going
    DOCS=$(find target/doc -mindepth 2 -maxdepth 2 -name index.html | wc -l)
    {
      echo "## Doc gate (rustdoc public surface)"
      echo ""
      echo "- Doc directories under \`target/doc\`: **${DOCS}** (lib + bin targets with an \`index.html\`)"
      echo "- rustdoc warnings: **0** (\`RUSTDOCFLAGS=-D warnings\`; the first one aborts this step)"
      echo "- Command: \`cargo doc --no-deps --workspace --keep-going\`"
    } >> "$GITHUB_STEP_SUMMARY"
    test "$DOCS" -gt 0
```

## Steps

| # | Step | Estado |
|---|---|---|
| 1 | Medir la ventana y la cadencia (12 runs de hoy, `gh run view`/`gh run list`) | ✅ |
| 2 | Confirmar por API que el context es required en los dos rulesets | ✅ |
| 3 | Comprobar **qué** deja una cancelación: check-runs del SHA `337ee796` | ✅ (`completed/cancelled`, no ausente) |
| 4 | `concurrency` con expresión por evento + comentario con la razón medida | ✅ |
| 5 | Summary con número del árbol de doc + aserción de no-vacuidad | ✅ |
| 6 | Contrato mecánico: `actionlint` 0 · YAML parseado · summary ejecutado en local | ✅ |
| 7 | Review P2-01 (una ronda): `changes-required` → 3 Blocking + 3 Medium + 2 Low cerrados | ✅ |
| 8 | Cierre: este task file, fila fuera del Backlog, entrada en `ci-cd.md`, commit | ✅ |

## Verificación (contrato mecánico)

| Comprobación | Comando | Resultado |
|---|---|---|
| lint de workflows | `actionlint .github/workflows/docs-gates.yml` | **exit 0** (baseline previo: 0; acepta la expresión en `cancel-in-progress`) |
| `concurrency` final | `js-yaml` sobre el fichero | `{"group":"${{ github.workflow }}-${{ github.ref }}","cancel-in-progress":"${{ github.event_name == 'pull_request' }}"}` |
| step en su sitio | `js-yaml` → `jobs.rustdoc.steps` | índice **2 de 7**; `env` del step = `{"RUSTDOCFLAGS":"-D warnings"}`; `env` del job **sin** `RUSTDOCFLAGS`; claves = `name,env,run` (sin `continue-on-error`) |
| ningún job perdido | `js-yaml` → `Object.keys(jobs).length` | **16**, y **0** `name:` duplicados |
| `pull_request` intacto | `js-yaml` → `on.pull_request` | `{"branches":["main","develop"]}` (sin `paths`, `FIND-765`) |
| el resumen **se escribe** | `bash` con el bloque `run:` **extraído del YAML** (`js-yaml`, sin retipear) y `cargo` sombreado por un no-op sobre un `target/doc` fabricado | `Doc directories: **3**` (el `index.html` de la raíz quedó fuera: `-mindepth 2`) · `GITHUB_STEP_SUMMARY` escrito · **step exit 0** |
| el resumen **no es vacuo** | mismo arnés con `target/doc` vacío | `**0**` y **exit 1** → el verde no puede ser un conteo de 0 |
| el context no cambió de nombre | `grep -n '^    name: Generate API reference (rustdoc)'` | presente (`:549`), intacto; `ci-gate.yml:58` lo sigue comparando por string |
| fila del Backlog | `git diff --numstat docs/dev/Backlog.md` | `0 6`: las **6** líneas de la sección `FIND-771` y **nada más** (verificado con `git diff -U1`; la sección de `FIND-770` intacta) |

**Lo que NO se pudo verificar, y se declara:** la prueba viva de la ventana —
**no se pueden disparar dos pushes reales** desde acá (el push es acto del
owner, Regla 7). Se verificó la **causa** (la expresión quedó puesta y el YAML
la parsea así), el **modo de fallo histórico** (`completed/cancelled` en
`337ee796`) y el efecto del cambio sobre **cada** evento (tabla de (A)), no una
cancelación futura. Tampoco se corrió `cargo doc`: carril **CI sin compilar**
(hay otro carril compilando). El `N` real de directorios de doc lo publicará el
primer run — el workspace declara **8** entradas en `members`
(`Cargo.toml:793-802`), así que el orden de magnitud es ~8-10, no ~80.

## Review (P2-01)

- **Revisor:** `vanta-review` (contexto fresco, report-only, no implementó).
  Tier **fast** por glob (`.github/workflows/**`, `docs/dev/**`), una ronda.
- **Enfoque:** ¿la directiva cambia el comportamiento que dice cambiar? ¿el
  summary mide o decora? ¿la premisa del ticket resiste la API? ¿se coló algo de
  otro carril?
- **Cómo se probó:** re-derivó `actionlint`, el parseo `js-yaml` (posición del
  step, `env` por nivel, `continue-on-error`, 16 jobs, `name:` intacto),
  `gh run view`/`gh run list`/`gh api` para los tiempos, los rulesets y los
  **check-runs del SHA `337ee796`**, y contrastó `Cargo.toml:793-802`.
- **Veredicto: `changes-required`** (3 Blocking, 3 Medium, 2 Low). Cerrados
  todos en esta misma ronda:
  - **B-1** — la fila del Backlog había quedado invertida: mi corte a EOF borró
    también la sección de `FIND-770` (un carril concurrente la reinsertó con mi
    sección delante). Rehecho **quirúrgico** por límites de sección, no por
    cola de fichero; `git diff -U1` muestra 6 líneas, todas mías.
  - **B-2** — la premisa "el context queda ausente / deadlock `Expected`" es
    **falsa**: el check-run existe y concluye `cancelled`. Reescrito el
    mecanismo en los tres sitios (arriba §Corrección de la premisa).
  - **B-3** — §Review incompleta y un `approve` pre-escrito en `ci-cd.md`.
    Ambas cosas corregidas: este veredicto es el real.
  - **M-1** — el fix era más ancho que el defecto y no evaluaba la alternativa
    barata: `cancel-in-progress` como **expresión por evento**. Adoptada
    (opción iii), con la tabla de (A) como razón.
  - **M-2** — "80 entradas en `members`": falso. Son **8** (`Cargo.toml:793-802`).
    Corregido.
  - **M-3** — "8 pushes de hoy": eran **12** (11 huecos, mediana **1h07m51s**, no
    49m21s). Corregido con la población entera.
  - **L-1** — "resto de jobs ≤ 4m03s": es **4m10s**. Corregido.
  - **L-2** — "Crates documented" sobrevendía lo que cuenta `find`. Renombrado a
    *"Doc directories under `target/doc`"*, con el techo de `cargo doc`
    acumulativo declarado; se eliminó el `-not -path 'target/doc/src/*'`, que a
    `-maxdepth 2` no podía casar nada (código muerto vendido como load-bearing).
  - **L-3** (nit) — `false` es el default documentado: se mantiene la forma
    explícita en `push`, ahora como parte de la expresión y no como redundancia.

## Deuda / pendiente

- **`FIND-764`** (misma clase en `ci-rust.yml`, **otro carril**, no se toca):
  allí la ventana es de **17m53s** y un `Tests (Linux)` cancelado deja el SHA
  sin suite. Este ticket **no** duplica ese arreglo, pero le deja el criterio:
  en `push` no se cancela; en PR sí, porque el merge ref se reconstruye.
- **`concurrency.queue: max`** — no usado; queda documentado arriba y en el
  comentario del YAML para cuando haga falta encolar bursts.
- **El conteo del summary** mide el tamaño del árbol de doc, no "crates
  compiladas": para una aserción de no-vacuidad es suficiente y es lo que se
  declara. Si algún día se quiere el número exacto por invocación, hay que usar
  `--message-format json` o comparar contra un `target/doc` limpio.
- La fila del Backlog `FIND-771` se retira; el registro queda en
  `docs/dev/avance/activo/ci-cd.md`.

## Alcance (escritura)

`.github/workflows/docs-gates.yml` · `docs/dev/tasks/FIND-771.md` ·
`docs/dev/Backlog.md` (solo la fila `FIND-771`) ·
`docs/dev/avance/activo/ci-cd.md`.

**Prohibido y respetado:** `.github/workflows/ci-rust.yml` (`FIND-770`, que
commitó `2b282eed7` en su propio carril mientras este corría) ·
`certification.yml` y `snapshot-certification.yml` (W2/W3) · `src/**`
(`FIND-758`) · `vantadb-mcp/**` (`FIND-757`) · `vantadb-python/**`
(`FIND-118`) · `docs/dev/architecture/**` (`FIND-756`) · `.githooks/**` ·
`dev-tools/**` · `docs/dev/plans/**` · `.opencode/**`.
