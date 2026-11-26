---
title: "FIND-716: 4 jobs de bench reusan el concurrency group del workflow y mueren con cero steps"
kind: task
status: in-progress
description: "Borrar el bloque concurrency job-level redundante en light-benchmarks, high-density, analyze y critcmp-regression: el group ya lo retiene el run, así que el job espera un slot que su propio run no suelta"
id: FIND-716
---

# FIND-716: 4 jobs de bench reusan el concurrency group del workflow y mueren con cero steps

## Metadata

- **Plan file:** [2026-11-23-release-v0.10.0-gates.md](../plans/2026-11-23-release-v0.10.0-gates.md) — §6.b
- **Fuente:** fila `FIND-716` de `docs/dev/Backlog.md` (derivada del triage `FIND-328`, commit `3f7fdd3fa`)
- **Esfuerzo:** 🟢 15 min de edición · el costo de lo que reactiva no es de 15 min (§Deuda)
- **Prioridad:** 🟢 Baja (no gatea PRs; retrasa la certificación de benches)
- **Tipo:** CI/CD / DevOps — sin código de producto
- **Carril:** C
- **Dependencias:** `FIND-328` ✅ (`3f7fdd3fa`, el triage que lo midió). En paralelo: `FIND-723` (Rust, `cargo`), `FIND-724`/`FIND-721` cerrando
- **Creado:** 2026-11-26

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | Nadie: `certification.yml` es el tier pesado (schedule 02:00/dom 03:00/lun 06:00 + `workflow_dispatch`). **No es check requerido** de ningún ruleset → un error aquí retrasa la certificación, no bloquea merges |
| Callees | El runner (`ubuntu-latest`), `cargo bench` de 10 benches, `actions/upload-artifact` (`light-bench-results`, `density-bench-results`, `critcmp-baseline`), y `analyze` como consumidor de los artifacts de los dos lanes |
| Implicaciones | Ninguna sobre código. El riesgo real es de **costo**, no de corrección: arreglarlo reactiva dos lanes que llevan meses sin ejecutarse (§Deuda) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos o en la extensión necesaria):** `.github/workflows/certification.yml` (líneas 1-100 header + `on:` + workflow-level `concurrency:`, 165-214 `failpoint-tests`, 405-534 los tres jobs del bench family, 690-759 `critcmp-regression`, 855-884 `fuzz-pr` — es decir, los 6 bloques `concurrency:` del fichero), `docs/dev/operations/CI_POLICY.md` (§FIND-328 triage 415-450, §`bench-informational` 660-698), `.opencode/rules/release-ci.md` (42L), `docs/dev/workflow/RULES.md` (grep `concurrency`), `docs/dev/tasks/FIND-714.md` (formato canónico), `docs/dev/Backlog.md` fila `FIND-716`.
- **Referencias hacia dentro:** `certification.yml` es referenciado por `docs/dev/operations/CI_POLICY.md` (407-450) y por `docs/user/operations/BENCHMARKS.md` (el nightly que citan varios baselines). `analyze` consume `light-bench-results` y `density-bench-results` por nombre de artifact; el nombre no cambia.
- **Referencias hacia fuera:** `./.github/actions/rust-setup`, `actions/upload-artifact@cf430e03…` (pin v7.0.2), `ci-gate.yml` (reusable).
- **Veredicto de impacto:** **bajo en corrección, alto en costo**. El cambio es sustraer 4 bloques de 3 líneas (+ sus comentarios) y no toca ni el `concurrency:` de workflow, ni los dos job-level correctos, ni steps, ni `needs:`, ni `if:`.

## Diagnóstico

**Síntoma.** `Light benchmarks` y `High density` → `conclusion=failure`, `steps: []`, `log not found`. Nunca arrancan.

Medido de nuevo para esta tarea (el triage `FIND-328` lo había medido; se reconfirma, no se hereda):

```
gh api repos/ness-e/Vantadb/actions/runs/45388973797/jobs --paginate
  failure  0  Light benchmarks
  failure  0  High density
  skipped  0  Analyze
  skipped  0  critcmp regression (candidate)

gh api repos/ness-e/Vantadb/actions/runs/45225387323/jobs --paginate
  failure  0  Light benchmarks
  failure  0  High density
  skipped  0  Analyze
  skipped  0  critcmp regression (candidate)
```

Y el detalle que fija el mecanismo (run `45388973797`): los dos lanes no esperaron nada, murieron **en cola**:

```
name             start                  end                    runner  conclusion
stress_protocol  2026-11-24T11:57:56Z   2026-11-24T13:02:25Z   GitHub Actions 1000024560  success
Light benchmarks 2026-11-24T11:57:54Z   2026-11-24T11:57:54Z   none    failure
High density     2026-11-24T11:57:54Z   2026-11-24T11:57:54Z   none    failure
```

`started_at == completed_at` y `runner_name = none`: cero segundos, ningún runner asignado. No es un timeout ni un fallo de steps, es una cancelación en el momento de encolar.

**Causa.** Los cuatro jobs declaran un `concurrency:` job-level con la **misma expresión** que el `concurrency:` de workflow:

```yaml
# workflow-level (L79-81) — correcto y se queda
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: false

# job-level (pre-fix: L421/L494/L529/L702) — redundante y suicida
    concurrency:
      group: ${{ github.workflow }}-${{ github.ref }}
      cancel-in-progress: false
```

Semántica (docs oficiales, verificadas 2026-11-26): «there can be at most one running job **or workflow** in a concurrency group at any time. When a concurrent job or workflow is queued, if another job or workflow using the same concurrency group in the repository is in progress, the queued job or workflow will be `pending`. **By default, any existing `pending` job or workflow in the same concurrency group will be canceled and the new queued job or workflow will take its place**» — y `queue: single` es el default, explícito: «At most one job or workflow run can be `pending` in the concurrency group. When a new job or workflow run is queued, any existing `pending` job or workflow run in the same group is canceled and replaced».

Trasladado a este fichero: el run ya ocupa el group, y el job pide un slot que **su propio run retiene** y que no va a soltar hasta terminar. GitHub no lo encola: lo falla al instante (lo que se ve: `failure`, 0 steps, `runner: none`, 8 s después del start del run y **65 min antes** de que el run termine). Los cuatro comparten además el group entre sí, así que `queue: single` (default) no tendría dónde formar una cola tampoco: cada entrada nueva reemplaza a la pendiente anterior. Los otros dos (`analyze`, `critcmp-regression`) tienen el mismo bloque, pero su `if:` los gatea fuera en la mayoría de los runs, así que el defecto solo se ve el día que su gate abre (dispatch, o `enable_critcmp`) — por eso aparecen `skipped` en la muestra de arriba.

**Grupo de control (lo que hace falsable el diagnóstico).** En el **mismo run** (`45388973797`, 24 jobs): los 4 rotos mueren sin steps; 16 jobs ejecutan steps con el **mismo** `concurrency:` de workflow y **sin** bloque job-level — `stress_protocol` (1 h 04 min), `memory_concurrency`, `storage_persistence`, `text_index_recovery`, `failpoint_injections`… — de los cuales 14 son `success` y los 2 restantes (`mutation_test`, `other_heavy`) son rojos **pre-existentes** ya triados en `FIND-328`; los otros 4 quedaron `skipped` por su propio `if:`. La única variable que separa a los que obtienen runner de los que no es el bloque job-level duplicado.

**Alcance temporal de la evidencia (sha `c634e532`).** Los runs citados son de ese `head_sha`, donde estos jobs tenían `if: (schedule || workflow_dispatch)` y `light-benchmarks` `timeout-minutes: 60` — no los `if: schedule == '0 2 * * *'` / `240` de hoy. Validan la colisión de group (que no depende del cron), no el comportamiento de los gates actuales.

**Patrón correcto, ya presente en el mismo fichero.** `failpoint-tests` (`certification.yml:187`) usa `group: failpoints-${{ github.ref }}` y `fuzz-pr` (`:863`) usa `group: ${{ github.workflow }}-${{ github.ref }}-${{ github.job }}` — un group **distinto** del de workflow: ahí el job no compite consigo mismo y la semántica de `cancel` sí hace lo que dice. Se copia ese criterio (group propio o ninguno), no se inventa.

**Nota de numeración.** El Backlog y `CI_POLICY.md` citan `416, 489, 524, 697`; **pre-fix** los bloques estaban en `421, 494, 529, 702` (+5 por la edición de `FIND-328`) y **post-fix ya no existen**. Las líneas de este documento son las del fichero **después** del cambio; se cumple el contenido, no el número histórico.

## Contrato

```
actionlint .github/workflows/certification.yml      # exit 0
rg -n '^ {4}concurrency:' .github/workflows/certification.yml
```

Esperado tras el fix: **2** líneas job-level (`failpoint-tests`, `fuzz-pr`) y **1** workflow-level (`^concurrency:`), y **0** en los 4 jobs afectados.

```
rg -n -B1 '^ {4}concurrency:' .github/workflows/certification.yml   # lista de los que se quedan
```

### Los que conservan `concurrency:` job-level (verificado post-fix)

| Job | Línea | Group | Por qué se queda |
|-----|-------|-------|------------------|
| `failpoint-tests` | `187` | `failpoints-${{ github.ref }}` | Group **distinto** al del workflow: serializa la lane consigo misma (dos runs de la misma ref no inyectan failpoints a la vez) sin competir con el run que lo hospeda. `cancel-in-progress: true` aquí sí significa algo: el run nuevo reemplaza al viejo. |
| `fuzz-pr` | `863` | `${{ github.workflow }}-${{ github.ref }}-${{ github.job }}` | Añade `${{ github.job }}` → único por job, así que el `cancel-in-progress: true` cancela el fuzz de la misma ref al pushear (deseado: es el único pesado en PRs) sin colisionar con el group de workflow ni con los otros jobs de la matrix. |

Los dos son el patrón correcto **porque el group no es el del workflow**. El criterio que se copió: *si un job necesita su propio gate de concurrencia, el group debe ser distinto del workflow-level; si no, no lleva bloque.*

Workflow-level (único que se conserva a nivel raíz): `^concurrency:` → L79, `group: ${{ github.workflow }}-${{ github.ref }}`, `cancel-in-progress: false`.

```powershell
# local, sin red
actionlint --version    # 1.7.12
```

## Steps

| # | Step | Estado |
|---|------|--------|
| 1 | Borrar el `concurrency:` job-level de `light-benchmarks`, `high-density`, `analyze`, `critcmp-regression` (y sus comentarios `# Restaura proteccion…`, que quedarían mintiendo) | ✅ |
| 2 | Reescribir el comentario de cabecera (bloque de deltas) para que describa el estado real, y dejar **una** nota autoritativa en el `concurrency:` de workflow (single source of truth, no 4 copias) | ✅ |
| 3 | Verify mecánico: `actionlint` + conteo de `concurrency:` + lista de los que conservan bloque job-level con su motivo | ✅ |
| 4 | `docs/dev/tasks/FIND-716.md` con §Deuda (minutos, cadencia, qué NO es gratis) | ✅ |
| 5 | Review P2-01 (tier Fast: path `.github/workflows/**`) + commit local | ✅ |

## Investigación (digest, fuentes verificadas)

- **Semántica job-level + `queue`.** [Control the concurrency of workflows and jobs](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-workflow-concurrency) (verificado 2026-11-26): un solo job **o** workflow por group; el pendiente por defecto es cancelado y reemplazado por el nuevo; `queue: single` es el default y `queue: max` (hasta 100 pendientes) es la única forma de encolar más de uno; el group es *case insensitive*.
- **`cancel-in-progress: false` no protege de esto.** La opción solo impide cancelar lo que **está corriendo**; no cambia la regla de que una entrada `pending` es reemplazada. Por eso el bloque redundante es peor que no tener nada: `cancel-in-progress: false` da la falsa sensación de "protección" mientras el job pendiente muere igual.
- **Facturación.** [GitHub Actions billing](https://docs.github.com/en/billing/concepts/product-billing/github-actions) (verificado 2026-11-26): «The use of standard GitHub-hosted runners is free: In public repositories». `ness-e/Vantadb` es público (`gh api repos/ness-e/Vantadb --jq .visibility` → `public`), así que los minutos de `ubuntu-latest` **no se facturan**. El costo real es de capacidad/ventana, no de dólares — se declara en §Deuda sin disfrazarlo de gratis.
- **La clase está confinada a este fichero.** Barrido de `.github/workflows/` por el par `concurrency:` + `group: ${{ github.workflow }}-${{ github.ref }}` a nivel job: `nightly.yml:113` y `certification.yml:863` (fuzz-pr) **añaden** `-${{ github.job }}`, o sea son el patrón correcto; el resto de los workflows usa el group solo a nivel workflow. No hay un segundo `FIND-*` que abrir por esta clase.

## Deuda — el costo de arreglar esto (decisión del owner, declarada)

Al quitar los bloques, los lanes **vuelven a ejecutarse**. Los números, con su fuente:

| Job | Cap (`timeout-minutes`) | Cadencia | Por semana (techo) |
|-----|-------------------------|----------|--------------------|
| `light-benchmarks` | 240 min (`certification.yml:431`) | cron `0 2 * * *` → diario | 7 × 240 = **1,680 min** |
| `high-density` | 360 min (`certification.yml:501`) | cron `0 2 * * *` → diario | 7 × 360 = **2,520 min** |
| `analyze` | 10 min (`certification.yml:533`) | cron `0 2 * * *` → diario | 7 × 10 = **70 min** |
| `critcmp-regression` | 60 min (`certification.yml:703`) | **solo** `workflow_dispatch` con `enable_critcmp: true` | 0 en el régimen normal; 60 min por dispatch |

- **Techo semanal: 4,270 min ≈ 71 h** de runner. Los dos lanes de bench solos son **10 h por noche** (cap `4 h + 6 h`, `240 + 360` min) ⇒ **70 h/semana**; `analyze` añade 1 h 10 min.
- **No hay duración medida**: desde la consolidación (`a4c290914`) estos dos lanes **nunca** han ejecutado un step, así que el número honesto es el **cap**, no un tiempo observado. El primer run verde del owner reemplaza esta tabla por la duración real (Regla 9: medido, no estimado).
- **Cadencia — corrección a la fila del plan.** El plan dice «reactiva 2h y 6h de CI» y el prompt de la tarea dice «ejecuciones semanales». Dos correcciones: el cap real de `light-benchmarks` es **4 h** (`240`, no 2 h — la "2 h" venía del comentario pre-consolidación que esta tarea borró) y el cron del bench family es **diario** (`0 2 * * *`, `certification.yml:53`), no semanal. Por eso el número semanal es 7 × 10 h, no 1 × 8 h.
- **Qué NO es gratis:** los minutos no se facturan (repo público), pero (a) dos jobs de 4 h y 6 h ocupan 2 slots de la capacidad concurrente de la organización cada noche, (b) los artifacts `light-bench-results` / `density-bench-results` (incluye `target/criterion/`) se retienen y son la entrada de `analyze`, y (c) el fallo silencioso de esos artifacts es exactamente lo que hoy nadie mira: el riesgo de reactivarlos es que aparezca una regresión real que hasta ahora estaba enmascarada por el "cero steps".
- **Alternativa más barata que el owner puede elegir** (no aplicada): dejar los bloques borrados **y** bajar `high-density` a semanal, o retirar el job con decisión en `CI_POLICY.md` + `# CATEGORY`. Esta tarea no toma esa decisión: solo la deja visible.

## Lo que esta tarea NO hace

- **No** toca el `concurrency:` de workflow (`:79-81`): es el que da la protección real (`cancel-in-progress: false` a nivel run).
- **No** toca `failpoint-tests` (`:187`) ni `fuzz-pr` (`:863`): son el patrón correcto.
- **No** toca `needs:`, `if:`, timeouts, steps ni artifacts: el diff es sustracción pura.
- **No** cambia la cadencia del cron ni retira jobs (§Deuda: decisión del owner).
- **No** escribe `Backlog.md`, `plans/**` ni `avance/**` (fuera de alcance).
- **No** hace push (política de git: el push es acto explícito del owner).

## Review (P2-01)

- **Reviewer:** `vanta-review` (contexto fresco, sesión `ses_defb0138cffefJbJv613UM6Nxm`; sin acceso a la sesión que implementó).
- **Tier:** Fast (paths `.github/workflows/**` + `docs/dev/tasks/**`; ningún glob adversarial de `prompts/task.md` Fase 5). Aun así se pidió review de agente distinto, no solo el verify mecánico.
- **Veredicto inicial:** 🔴 `changes-required` — 0 Critical · **2 Required** · 4 Optional · 1 Nit. El reviewer dejó por escrito que el cambio de CI "es **correcto y no necesita rework**" y que "con R1 y R2 corregidos —dos ediciones de una línea, sin re-ejecutar CI ni tocar el YAML funcional— mi dictamen pasa a `approve`". Ambas están corregidas y verificadas abajo.
- **R1 (corregido):** §Deuda se contradecía — `4.270 min ≈ 71 h` frente a `8 h/noche = 56 h/semana`, y la "2 h" no tenía fuente (venía del comentario pre-consolidación que esta tarea borra). Ahora: cap real **`240 + 360 = 600 min = 10 h`/noche** ⇒ **70 h/semana**, más `analyze` (1 h 10) = 71 h. La línea de cadencia declara las dos correcciones al plan (4 h, no 2 h; diario, no semanal).
- **R2 (corregido):** el mecanismo escrito era "queda `pending` y se cancela al terminar el run". La medición lo refuta: `started_at == completed_at = 11:57:54Z`, run iniciado `11:57:46` y activo hasta `13:02:25` → murieron **8 s** después del start, **65 min antes** de que el run terminara, sin entrada pendiente. Reescrito en el task file y en la nota del workflow como **"no lo encola: lo falla al instante"**. La regla de `queue: single` sigue citada porque explica por qué los 4 tampoco pueden encolarse entre sí, no como causa del `failure`.
- **O1 (aplicado):** `certification.yml:20` decía "grupo estandar **cancelable**"; es falso (`cancel-in-progress: false`, stale desde `3f7fdd3fa`) → "serializado, no cancelable".
- **O2 (aplicado):** el grupo de control afirmaba "20 jobs más, todos `success` con 7-13 steps". Falso con su propio comando: son 24 jobs = 4 rotos + 4 `skipped` por su `if:` + **16 que obtuvieron runner**, de los cuales 14 `success` y 2 rojos pre-existentes (`mutation_test`, `other_heavy`, ya triados en `FIND-328`). Reescrito con los números reales; el argumento sobrevive.
- **O3 (aplicado):** añadido el `head_sha c634e532` de los runs citados y la advertencia de que ahí los `if:`/`timeout` eran otros: validan la colisión de group, no los gates actuales. El corolario sobre `CI_POLICY.md:439` queda como deuda (`queda_pendiente`), fuera del alcance de escritura de esta tarea.
- **O4 (aplicado):** re-ancladas todas las líneas a la revisión **post-fix** (`:431`, `:501`, `:703`, `:53`, `:79-81`, `:187`, `:863`) y marcadas como `pre-fix` las que ya no existen.
- **N1 (aplicado):** el Step 5 se marca al cerrar, no antes.
- **Lo que el reviewer validó del enfoque:** sustracción pura (−19/+16, sin tocar `if:`/`needs:`/timeouts/artifacts), una sola nota autoritativa en vez de 4 copias, y la §Deuda declara el costo en vez de esconderlo tras "repo público ⇒ gratis".
- **Sin re-review (una ronda, por contrato de la tarea):** las correcciones son de prosa y de un comentario; el YAML funcional no cambió respecto de lo aprobado y `actionlint` sigue en exit 0. Se registra la condición del reviewer y su cumplimiento, no una segunda aprobación literal.
