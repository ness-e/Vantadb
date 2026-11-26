---
title: "FIND-766 - other_heavy nunca ejecutó un test: datasets a un job propio y el schedule mide develop"
kind: task
status: completed
description: "other_heavy: 0 success / 39 failure en 39 runs programados y en todos el step de tests queda skipped, porque el gate TBH-01 de datasets corre antes y porque el schedule mide main. Se separa el dataset en un job propio, los tests sin dataset (96 cli_tests + server + mcp) corren con listed==executed, y el schedule hace checkout de develop con aserción de árbol."
tags: [vantadb, ci, certification, nextest, datasets, schedule]
---

# FIND-766 — una fila roja 30 días que nunca ejecutó nada

> **Dependencias:** `FIND-752` ✅ (la auditoría que contó los runs) · `FIND-768` ✅ (el patrón
> `listed == executed`, `3554aa3db`) · `FIND-664` (el fallo de datasets que originó `TBH-01`).
> **Carril:** CI — sólo YAML + docs. Ningún `.rs` tocado, ningún `cargo` grande ejecutado
> (hay un carril Rust en review y 6,8 GB de disco).

## Contexto

`certification.yml` job `other-heavy` era **un solo step** que empezaba pidiendo el dataset
de benchmark. Si el dataset no estaba, el step moría ahí y los tests que venían después
quedaban `skipped` — que en el resumen del run no se distingue de "no había nada que correr".

**Re-medido al cerrar (H-02), no heredado de la fila:**

```bash
gh run list --workflow certification.yml --created ">=2026-10-27" --limit 200 \
  --json databaseId,event,conclusion
# 99 runs: 59 pull_request, 1 push, 39 schedule
# schedule: 37 failure + 2 cancelled, 0 success   (la fila decía 36 failure / 3 skipped)
```

Los tres últimos runs programados, con los steps del job:

```bash
gh api repos/{owner}/{repo}/actions/runs/<id>/jobs \
  --jq '.jobs[] | select(.name|test("other_heavy")) | [.steps[] | {n:.name,c:.conclusion}]'
# 45800950632 / 45598671538 / 45388973797 — idénticos:
#   failure  Verify required benchmark datasets (TBH-01)
#   skipped  Download benchmark datasets
#   skipped  Run remaining heavy tests
```

**El nombre del step que falla es la prueba de la segunda capa.** Ese step — `Verify required
benchmark datasets (TBH-01)`, el gate de 5 datasets que corría **antes** de la descarga — ya
**no existe** en `develop`: `FIND-328` lo reemplazó por `Verify required benchmark dataset
(TBH-01, GloVe-100)` después de la descarga. Que los runs sigan ejecutando el nombre viejo
demuestra que el `schedule` corre el YAML de **`main`**, no el de `develop` (552 commits por
delante). O sea: el fix del dataset existía y el lane no lo estaba mirando. Es `FIND-741`
(verificar contra un artefacto stale) con un calendario encima.

Consecuencia contada por `FIND-752`: **98 tests** (96 `cli_tests` + 2 benchmarks) con un lane
nominal que no los ejecutó nunca. Y no eran sólo 98: los otros 12 targets del mismo step
(server, mcp, ceros de certificación) tampoco corrían, porque el `skipped` era del step
entero.

## Contrato

1. **(a)** `cli_tests` (96 tests) y todo lo demás que **no** necesita dataset corre en un job
   que no depende del dataset: si falta GloVe, esos tests igual ejecutan.
2. **(b)** el rojo por dataset es un job **distinto** y con nombre propio: "falló la descarga"
   y "falló un test" dejan de verse igual en el resumen.
3. **(c)** el `schedule` mide **`develop`** (el cron no puede apuntar a una rama, así que lo
   hace el checkout) y una aserción de árbol lo verifica en cada run.
4. **No-vacuidad:** el job de tests cuenta la población con `nextest list` **antes** de correr
   y la compara con el JUnit de la fase; `listed != executed` es rojo (`FIND-768`).
5. Mecánico: `actionlint` 0 · YAML parseado · `bash -n` de los cuatro bloques `run:` · bloque
   `run:` real ejecutado con `cargo` falso en 3 escenarios (población llena, selector que
   estrecha, población 0).

## Impacto mapeado (Regla 0)

| Archivo | Leído | Referencias hacia dentro | Referencias entrantes | Impacto |
|---|---|---|---|---|
| `.github/workflows/certification.yml` | completo (1057 L) | `.config/nextest.toml` (`default-filter` excluye 13 de estos binarios), `Cargo.toml` (`[[test]]` de 11 de ellos), `tests/benchmark_datasets.rs:18` | `docs/dev/operations/CI_POLICY.md` §2c, `docs/dev/avance/activo/ci-cd.md`, `nightly.yml:5` (comentario) | **se edita**: el job `other-heavy` (296-378) pasa a dos jobs (`other-heavy` sin dataset + `benchmark-datasets`), con conteo y aserción de árbol. Sin cambios en triggers, permisos, `concurrency` ni los otros 20 jobs |
| `docs/dev/operations/CI_POLICY.md` | §2c completo (642-691) | — | 26 ficheros | **sólo las 3 filas de `other_heavy`/`cli_tests` en §2c** (el ticket prohíbe el resto) |
| `docs/dev/Backlog.md` | sección 810-815 | — | — | **se retira la fila `FIND-766`**; su sección tiene una sola fila (verificado), así que se va la sección |
| `docs/dev/avance/activo/ci-cd.md` | entradas de la ola (FIND-752/764/768/771) | — | — | **se añade una entrada** al final |
| `.config/nextest.toml` | completo (108 L) | — | `ci-rust.yml`, `dev-tools/**` | **sólo lectura** — prohibido; la exclusión de `default-filter` es correcta, lo que estaba mal era la lane. Por eso el job usa `--ignore-default-filter` |
| `Cargo.toml` | 753-760 (`[profile.ci]`), bloques `[[test]]` | — | — | **sólo lectura** — decide `--cargo-profile release` (el perf que ya tenían estos tests) frente a `ci` |

Fuera de alcance (prohibido por el ticket y respetado): `ci-rust.yml`, `docs-gates.yml`,
`snapshot-certification.yml`, `.config/nextest.toml`, `src/**`, `vanta-memory/**`,
`vantadb-mcp/**`, `.opencode/**`, `dev-tools/**`, `docs/dev/plans/**`.

## Spec — decisiones

| # | Decisión | Alternativas rechazadas |
|---|---|---|
| 1 | **Dos jobs**: `other-heavy` (tests) y `benchmark-datasets` (GloVe + su consumidor) | **(a)** Dos steps en el mismo job: el resumen sigue mostrando un job rojo que contiene 120+ tests, y no se sabe si corrieron — es el defecto que `FIND-752` midió. **(b)** Job de dataset como `needs:` del de tests: lo vuelve `skipped` (que GitHub reporta como **success**) justo cuando falta el dataset: el pre-mortem servido |
| 2 | `benchmark_internal` va al job de **tests**, no al de datasets | Parecía dataset-bound y no lo es: genera su propio corpus sintético de 10k (`tests/benchmark_internal.rs:40`, cero referencias a `data/benchmark`). Dejarlo del lado del dataset habría mantenido 1 test rehén de una descarga que no usa |
| 3 | Población = los targets seleccionados por `--test`, sin `-E` (`coverage=full`) | Un `-E` estrecho encogería **los dos** lados y la comparación seguiría verde: la aserción decorativa que `FIND-768` §Spec-2 ya descartó |
| 4 | **`cli_tests` tiene su propia fase** (`01-vantadb-cli-tests`), no va agregado con los otros 11 | Review H1: con una sola cifra por fase y 12 binarios dentro, vaciar `cli_tests` dejaba `listed == executed` con los otros 11 y la fase seguía **verde** — la selección parcial de `FIND-738`, una granularidad más abajo. Una cifra por fase sólo prueba algo si la fase es **un** binario (o el conjunto que se quiere medir) |
| 5 | El job de tests usa **nextest**, no `cargo test` | `cargo test` no produce JUnit y su `-- --list` no comparte formato con el conteo de ejecutados. nextest es el runner que ya usa el repo para las lanes certificadas (`audit`), y su JUnit es el patrón probado en `FIND-768`. `--cargo-profile release` conserva el perfil que estos tests ya tenían (`cargo test --release`) en vez de cambiarlos a `ci` |
| 6 | El perfil se **redeclara en el job** (`--config-file target/nextest/other-heavy/nextest.toml`), no se hereda de `.config/nextest.toml` | Review H2: el perfil `audit` del repo trae `slow-timeout = 60s terminate-after 3` — **mata** el test a los 180 s — y `leak-timeout = 500ms result = fail`. Ninguna de las dos se midió nunca en estos 11 binarios, que jamás corrieron bajo nextest: un falso rojo por timeout en la lane que existe para quitar rojos falsos. El fichero del repo es **prohibido** por el ticket, así que el perfil se declara en el step: 600 s (un cuelgue, no un runner lento) y sin `leak-timeout` (`result = "warn"` no existe; el enum es `fail`\|`pass`). `fail-fast = false` y el JUnit se conservan; ruta verificada: `target/nextest/audit/junit.xml` |
| 7 | `ref: ${{ github.event_name == 'schedule' && 'develop' || github.ref }}` + step de aserción | El cron de GitHub **sólo** dispara sobre la rama default: no hay forma de que el `schedule` "corra sobre develop" sin que el checkout lo haga. Y un checkout con expresión puede estar mal en silencio, así que el step falla si en `schedule` el árbol no es `develop`. Alternativa "correr los dos y distinguir en el nombre": duplica 98 tests de carga semanal para reportar dos veces el mismo árbol viejo |
| 8 | Sin `needs: ci-gate` en los dos jobs nuevos | `needs:` convierte el job en `skipped` — reportado como success — cuando `ci-gate` está en rojo, y este es el **único** lane de estos binarios. Misma decisión y mismo razonamiento que `security-audit` (`FIND-767`) |
| 9 | Conteo sólo en el job de tests; en `benchmark-datasets`, `--no-tests fail` | Ese job corre 1 test con `--test` directo y sin `-E`: la selección no puede encogerse sin vaciarse. Duplicar la función de 40 líneas para contar `1 == 1` no compra nada. El perfil declarado (decisión 6) sí se repite ahí, por el mismo motivo |
| 10 | `structured_api_v2` va en una fase `full`, **sin** selector que excluya su `#[ignore]` | Review L1: nextest ya deja fuera los `#[ignore]` de **los dos** lados — `nextest list` no los lista y el JUnit no los emite (`report-skipped = none`) —, verificado con un test ignorado en el proyecto de prueba (lista 1 de 2, 1 `<testcase>`). El selector `not test(ollama_integration)` que se había puesto era ruido: debilitaba la guarda del único test restante. El `#[ignore]` de `tests/api/structured_api_v2.rs:60` (Ollama externo) simplemente no está en la población |
| 11 | Techo declarado: **sólo estos dos jobs** hacen checkout de `develop` | El resto de `certification.yml` (chaos, mutation, benches, analyze) sigue midiendo `main`. Cambiarlos es otra decisión con otro blast radius: `analyze` **pushea** un commit de baseline, y desde un checkout de `develop` ese push iría a `develop` (Regla 7: nada directo a `develop` sin PR). Va como deuda abajo, no como cambio silencioso |

## Pasos

| # | Step | Estado |
|---|---|---|
| 1 | Medir la premisa con `gh` (39 runs programados, 0 success) y confirmar el `skipped` de los dos steps | ✅ |
| 2 | Separar `benchmark-datasets` (GloVe + `benchmark_datasets`) de `other-heavy` (tests sin dataset) | ✅ |
| 3 | `listed == executed` por fase (5 fases) con el patrón de `FIND-768` | ✅ |
| 4 | `ref: develop` en `schedule` + aserción de árbol en los dos jobs | ✅ |
| 5 | Verificación mecánica: `actionlint`, YAML, `bash -n`, harness A/B/C con `cargo` falso sobre el bloque real | ✅ |
| 6 | Cierre de docs: `CI_POLICY` §2c, Backlog, `ci-cd.md` | ✅ |

## Verificación

**El workflow no se puede disparar desde acá** (`workflow_dispatch` corre sobre la rama
pusheada y el push es acto del owner, Regla 7). Lo que sí se ejecutó:

| Check | Resultado |
|---|---|
| `actionlint .github/workflows/certification.yml` | ✅ exit **0** (baseline 0) — ver el incidente de abajo, no fue gratis |
| YAML parseado (`js-yaml`) | ✅ 23 jobs · `schedule` = 3 crons · **0** `name:` duplicados · `other-heavy` (5 steps) y `benchmark-datasets` (7 steps) **sin** `needs` · ambos `checkout` con la expresión de `develop` |
| `bash -n` de los 4 bloques `run:` extraídos del YAML | ✅ exit 0 en `assert`, `phases`, `dataset_run`, `tbh01` |
| **Escenario A** — población llena, ejecutados completos (nextest **real**, JUnit real) | ✅ 5 fases `listed=2 executed=2` (`01-vantadb-cli-tests`, `02-vantadb-dataset-free`, `03-vantadb-lib-hnsw`, `04-server`, `05-mcp`), driver exit **0** |
| **Escenario B** — población 2, selector estrecha la ejecución a 1 | ✅ fase **roja**: `01-vantadb-cli-tests: population is 2 test(s) but 1 executed.` driver exit **1** |
| **Escenario C** — población 0 con selector lleno | ✅ fase **roja**: `nextest list exited 0 selecting 0 test(s) with a full selector.` driver exit **1** |
| **Escenario D** — el step del dataset, nextest real | ✅ exit **0**: `2 tests run: 2 passed` con el `--config-file` declarado |
| **Aserción de árbol** — `schedule` sobre `main` | ✅ **roja**, exit 1, con el mensaje de `FIND-741` |
| **Aserción de árbol** — `schedule` sobre `develop` · `push` sobre `main` | ✅ verdes, exit 0 en los dos |

**Incidente de `actionlint` (bloqueó el `pre-commit` de todos los carriles; declarado y cerrado).** El
hook `.githooks/pre-commit` llamaba `actionlint` **sin argumentos**, y así actionlint lee
**stdin**: todos los "actionlint ok" del día eran un gate que no linteaba nada. Corregido por el
owner a `actionlint .github/workflows/*.yml`, el fichero de esta tarea lo hizo **crashear**:
`exit -1`, sin mensaje, con `Found 0 parse errors` en `-verbose` (o sea: no era YAML).
Aislado por bisección sobre un workflow mínimo:

| Paso | Resultado |
|---|---|
| `actionlint -shellcheck=` (shellcheck desactivado) | ✅ exit 0 → el crasher es el **shellcheck embebido**, no el parser YAML |
| Cuerpo del script, 136 líneas **con** los `#` de prosa | 💥 crash |
| El mismo cuerpo, 101 líneas, **sin** los `#` | ✅ exit 0 |
| Los mismos `#` movidos a comentarios YAML **encima** del step | ✅ exit 0, y el código es idéntico |

Arreglo: la prosa del step vive ahora en comentarios YAML y el bloque `run:` lleva **0 líneas de
comentario** (verificado: `phases.sh` 0, `dataset_run.sh` 0, `assert.sh` 0, `tbh01.sh` 0). Ninguna
aserción se tocó: A/B/C/D corren sobre el bloque real extraído del YAML.

El harness A no es una maqueta: el `cargo` falso delega en **cargo-nextest 0.9.133** (el
instalado) sobre un proyecto de prueba, y el conteo sale del JUnit real en
`target/nextest/audit/junit.xml`. Eso también fijó dos cosas que estaban a ojo:

- el formato de `nextest list` es `binary::test_name` (una línea por test) — la regex de
  `FIND-768` matchea una vez por línea, verificado, no asumido;
- el JUnit vive en `target/nextest/<profile>/junit.xml` **también** con `--cargo-profile
  release` (verificado: `target/nextest/audit/junit.xml`), que es lo que la lane lee;
- el filtro de la fase 02 (`binary(structured_api_v2) and not test(ollama_integration)`)
  lista **1** de los 2 tests del binario y corre verde: la exclusión del `#[ignore]` que pide
  un Ollama externo está probada, no supuesta.

**Lo que la lane va a publicar en su primer run** (no se puede saber sin dispararla): las 5
líneas de `listed-vs-executed.txt`. Los conteos de fuente —`#[test]` por fichero— dan **283**
tests seleccionados entre los dos jobs (282 en el de tests + 1 en el de dataset), **282**
ejecutables tras el único `#[ignore]`. De esos, **98** son los del contrato de la fila (96
`cli_tests` + `benchmark_internal` + `benchmark_datasets`), que antes corrían **0**. Los otros
185 (`server` 48, `mcp_tests` 123, certificación/otros 13, lib 1) estaban en el mismo `skipped` y
tampoco corrían. Ninguno de esos números es autoritativo: lo emite el job.

## Review

**P2-01 — `vanta-review`, contexto fresco, una ronda → `changes-required`: 2 Blocking + 2 High +
3 Medium + 2 Low, todos cerrados o declarados.** Revisó el blob anterior a esta ronda y reprodujo
lo suyo por su cuenta (proyecto de prueba con nextest real, `gh`, `actionlint`).

| # | Sev | Hallazgo del revisor | Estado |
|---|---|---|---|
| B1 | **Blocking** | La fase de la lib seleccionaba **cero** tests: el path real es `index::core::tests::…`, el selector decía `index::tests::…`. `listed=0` → rojo **todas** las semanas y las fases siguientes nunca corrían. El `cargo test --lib index::tests::…` viejo lo escondía (filtro substring, 0 tests, exit 0) | ✅ **confirmado y arreglado**: `src/index/core.rs:3` (`mod tests`) bajo `pub(crate) mod core` (`src/index/mod.rs:5`), test en `core.rs:152`; el filtro viejo aparece **0 veces** en el fichero. Selector → `index::core::tests::concurrent_insert_preserves_hnsw_invariants` |
| H1 | **High** | La fase 01 contaba **agregado sobre 11 binarios**: vaciar `cli_tests` (los 96 del contrato) seguía dejando `listed == executed` con los otros 10 — la selección parcial de `FIND-738` en la granularidad equivocada | ✅ `cli_tests` tiene fase propia (`01-vantadb-cli-tests`): una cifra por fase sólo prueba algo si la fase es **un** binario |
| H2 | **High** | `--profile audit` arrastra `slow-timeout = 60s terminate-after 3` (test **matado** a los 180 s) y `leak-timeout 500ms result=fail` a 11 binarios que nunca corrieron bajo nextest. Falso rojo por timeout en la lane que quita rojos falsos; no estaba declarado | ✅ perfil re-declarado en el job con `--config-file` (600 s, sin leak-timeout) porque `.config/nextest.toml` es **prohibido**; `result = "warn"` no existe (enum `fail`\|`pass`), omitirlo queda como deuda. Verificado: el JUnit sigue en `target/nextest/audit/junit.xml` |
| M1 | Medium | §2c decía "runs 96 of 96" / "0 after FIND-766" para una lane que **nunca** había corrido, y la frase añadida ("the scheduled checkout now takes `develop`") no estaba acotada a los dos jobs | ✅ filas → "**wired to run** … **no run yet**"; frase acotada y con la deuda del resto del workflow + la aclaración de que el cron sigue ejecutando `main` hasta el merge |
| M2 | Medium | "El verde real lo publica el primer cron de domingo" es **falso**: un `schedule` ejecuta el YAML de la rama default, así que hasta el merge el cron corre la versión de un solo job | ✅ corregido en `ci-cd.md`: sólo un `workflow_dispatch` sobre `develop` lo ejercita antes |
| M3 | Medium | Docs/evidencia stale frente al artefacto (4 fases, `structured_api_v2` dentro de la fase 01) | ✅ §Verificación re-escrita contra el fichero final: 5 fases, A/B/C/D re-corridos |
| L1 | Low | La fase `scoped` de `structured_api_v2` partía de una premisa falsa: nextest ya excluye los `#[ignore]` de `list` **y** del JUnit, así que debilitaba la guarda del único test restante | ✅ eliminada: el binario vuelve a una fase `full`; verificado que `list` da 1 de 2 y el JUnit 1 `<testcase>` |
| L2 | Low | 45 min para el step del dataset contra el "~60 min cold compile" del comentario hermano | ✅ 90 min de step / 105 de job |

**Lo que el revisor verificó como correcto y no cambió:** la expresión de `ref:` en los cuatro
triggers, `--ignore-default-filter` válido en `nextest list` 0.9.133, la ruta del JUnit con
`--cargo-profile release`, la regex (un match por línea, sin doble conteo), `PIPESTATUS` de 2,
`--no-tests fail` → exit 4, que ningún test seleccionado toca `data/benchmark`, `actionlint` 0, 23
jobs, 0 nombres duplicados y que ningún path prohibido quedó tocado.

**Hallazgo propio durante esta ronda (no del revisor, y el más caro):** el `run:` de 136 líneas
hacía crashear al shellcheck embebido de `actionlint` (`exit -1`, sin mensaje) y bloqueaba el
`pre-commit` de **todos** los carriles. Aislado y arreglado en §Verificación; la prosa del step
vive ahora en YAML y el script no lleva un solo `#`.

## Deuda

- **`leak-timeout` queda sin declarar** en el perfil del job (review H2): el enum de nextest es
  `fail|pass`, no existe "avisar y seguir", y la detección de leaks nunca se midió en estos
  binarios. Si el primer run aparece con un leak, la decisión es ponerlo en `pass` y anotar el
  hallazgo, no editar `.config/nextest.toml` (fuera de alcance).
- **El `slow-timeout` del job es 600 s, no el del repo (180 s).** Si algún día un test de esta
  lane tarda entre 180 y 600 s, el perfil del repo lo habría matado y este no: la diferencia está
  declarada acá justamente para que nadie la lea como un descuido.
- **El `run:` no puede llevar comentarios mientras `actionlint` use ese shellcheck.** Es una
  restricción del gate, no del workflow: quien vuelva a poner prosa dentro de un `run:` de este
  fichero puede volver a bloquear el `pre-commit` del repo entero. Medido: 136 líneas con `#` →
  crash; 101 sin → exit 0.
- **El resto de `certification.yml` sigue midiendo `main` en `schedule`** (chaos, mutation,
  benches, analyze): 20 jobs con la misma clase de defecto que (c). No se tocan acá por blast
  radius (`analyze` pushea baseline; desde `develop` ese push iría a `develop`, Regla 7). El
  arreglo es el mismo `ref:` + la misma aserción, job por job.
- **`benchmark_internal` no tiene a nadie más que lo corra**, y ahora sí corre: 1 test de 10k
  inserts en release. Si su latencia crece, el sitio es este job.
- **La `needs: ci-gate` sigue en los otros 9 jobs de certificación.** Cada uno de ellos se vuelve
  `skipped`-como-success cuando `main` está rojo. `other-heavy` y `security-audit` ya no.
- **`snapshot-certification.yml` lleva la misma regex y la misma ruta de JUnit que acá se
  verificaron**, pero ese fichero es de otro carril (prohibido por el contrato): la verificación de
  este task file es la primera evidencia real de que su `run_phase` funciona con nextest 0.9.133.

## NOTICED BUT NOT TOUCHING

- `nightly.yml:5` documenta `other_heavy` como "deliberately heavy, weekly" — sigue siendo
  cierto, pero ahora son **dos** jobs (`other_heavy` tests + `benchmark_datasets`).
- `CI_POLICY.md` §2c cita `run 45598671538` como "last real result" de estas filas: hoy ese
  run está a 3 runs de distancia del último (45800950632), con el mismo desenlace.
