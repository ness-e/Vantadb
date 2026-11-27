---
title: "FIND-815 - el guard solo medía con `rust-scope=true`: un PR docs-only con `secret-scan` roto no bloqueaba"
kind: task
status: completed
description: "El guard required juzgaba una sola cosa (¿ejecutaron las suites?) y su rama legítima de skip dejaba pasar un upstream rojo en cualquier PR sin Rust. Ahora juzga también `secret-scan`, `fmt` y `clippy`, en ese orden y antes de la rama de skip, sin coste de runner para docs."
tags: [vantadb, ci, github-actions, ruleset, required-checks, test-guard]
---

# FIND-815 - el guard tenía una sola pregunta

## Contexto

`Tests Executed (guard)` (`ci-rust.yml:586`) nació con `FIND-787` para convertir en
ROJO el `skipped` que GitHub cuenta como required satisfecho. Tenía una rama
**legítima** de skip:

```bash
if [ "${SCOPE:-}" = "false" ] && [ "${SCOPE_RESULT:-}" = "success" ]; then exit 0; fi
```

que es correcta -sin cambios de Rust no hay suites que ejecutar- pero deja el borde
que el propio `FIND-787` documentó como precondición: **en un PR docs-only, las suites
no tienen por qué correr y `Secret Scan` no es required**, así que un escaneo en rojo
no bloqueaba nada. El caso real ya existía: en `43e41f055` el `Secret Scan` falló y las
tres suites quedaron `skipped`.

## Contrato

El guard juzga **dos** cosas:

1. si las suites tocaban ejecutarse y no ejecutaron ⇒ rojo (**lo que ya hacía**);
2. si un check **upstream** que alimenta a las suites falló (`secret-scan`, `fmt`,
   `clippy`) ⇒ **rojo aunque el scope sea docs-only**.

La condición de skip pasa de *"no hay Rust"* a *"no hay Rust **y** los upstream están
verdes"*.

## Impacto mapeado (Regla 0)

**Leídos completos:** `.github/workflows/ci-rust.yml` (el job `tests-guard` y los jobs
que se agregan a su `needs`: `secret-scan:204`, `fmt:234`, `clippy:258`),
`docs/dev/tasks/FIND-787.md` (§D5, §D8 y el borde del ruleset), y
`docs/dev/avance/activo/ci-cd.md` (la entrada de `FIND-807`).

**Referencias hacia dentro:** el ruleset `23692587` matchea el job **por nombre**
(`Tests Executed (guard)`): el nombre no cambia, así que el context sigue el mismo y
**no hay cambio de ruleset que aplicar**. `timeout-minutes: 5` y el `concurrency:` de
job se conservan.

**Referencias hacia fuera:** ninguna. El job no publica artifacts ni outputs que otro
job consuma; su único efecto es el check-run.

**Veredicto:** cambio contenido en un job, aditivo (`needs` gana 3 entradas, `env` gana
3, el `run:` gana un bloque al principio). El riesgo real es de **coste** (que el guard
se ponga rojo con todo verde y un PR docs-only deje de poder mergear) y de **falso
verde** (que la rama de skip siga tapando un upstream rojo). Las dos direcciones se
miden abajo, con el `run:` extraído del fichero.

## Spec - decisiones

| # | Decisión | Elegido | Alternativa descartada | Por qué |
|---|---|---|---|---|
| D1 | ¿Qué upstream juzga? | `secret-scan`, `fmt`, `clippy` | sólo `secret-scan` | la fila nombra los tres y los tres alimentan a las suites: `fmt`/`clippy` son la compuerta de compilación. Juzgar uno y no los otros deja el mismo agujero con otro nombre. |
| D2 | ¿En qué orden? | upstream **primero**, antes de la rama de skip | después del bloque de las suites | si se juzgara después, el `exit 0` de la rama de skip cortaría el script antes de mirar los upstream. Es el orden lo que cierra el agujero, no el chequeo. |
| D3 | ¿Qué cuenta como "omitido legítimo"? | `clippy: skipped` **sólo** con `SCOPE=false && SCOPE_RESULT=success`; `secret-scan`/`fmt`: nada | aceptar `skipped` en los tres | `secret-scan` (`:204`) y `fmt` (`:234`) **no llevan `if:` ni `needs:`**: se ejecutan en todo run, así que un `skipped` ahí es anómalo ⇒ fail-closed. `clippy` sí está gateado por `rust-scope` (`:270-273`): es el único omitido que tiene una decisión medida detrás. |
| D4 | ¿Un `failure` de una suite es rojo del guard? | **no** (se conserva D7 de `FIND-787`) | fallar también | ese required ya está rojo por sí mismo; el guard responde "¿se ejecutó?", no duplica el veredicto. |
| D5 | ¿Cambia el ruleset? | **no** | exigir además `Secret Scan` | con el guard ya required, su rojo bloquea todo PR donde el escaneo falle -incluido el docs-only-. Volver required al `Secret Scan` costaría runs en PRs que no tocan nada del escaneo y es innecesario: **no se propone cambio de ruleset**. |

## Verificación — las dos direcciones, con el `run:` real

El `run:` **no se copia a mano**: un extractor de Python lo saca del fichero (bloque
`run: |` del step, dedentado) y se ejecuta con bash 5.3.9 y `env -i`, con **sólo** las
variables que GitHub le daría al step (`needs.*.result`, `needs.rust-scope.outputs.rust`,
`github.actor`). 60 líneas extraídas, 42 no vacías no-comentario, sha256 del bloque
`58e3c81089337aa8…`.

| # | Cara | Entradas | Esperado | Obtenido |
|---|---|---|---|---|
| i | suites ejecutan, upstream verde | `SCOPE=true`, los 3 `success` upstream, 3 tests `success` | 0 | **0** |
| ii | tres suites `skipped` (el caso `43e41f055`) | `SCOPE=true`, upstream verde, 3 `skipped` | 1 | **1** |
| iii | **docs-only, todo verde** | `SCOPE=false`, upstream `success`+`clippy=skipped`, 3 `skipped` | **0** (sin coste) | **0** |
| iv | **docs-only + `secret-scan` ROJO** | `SCOPE=false`, `SECRET_SCAN=failure`, `clippy=skipped` | **1** | **1** |
| v | docs-only + `fmt` ROJO | `SCOPE=false`, `FMT=failure` | 1 | **1** |
| vi | docs-only + `secret-scan` `skipped` (anómalo) | `SCOPE=false`, `SECRET_SCAN=skipped` | 1 | **1** |
| vii | scope medido + `clippy` `skipped` (anómalo) | `SCOPE=true`, `CLIPPY=skipped` | 1 | **1** |
| viii | `rust-scope` roto (fail-open) | `SCOPE=false`, `SCOPE_RESULT=failure` | 1 | **1** |
| ix | una suite roja, upstream verde | `LINUX=failure`, resto `success` | 0 | **0** |
| x | PR de dependabot | 3 `skipped`, `ACTOR=dependabot[bot]` | 1 | **1** |
| xi | sin ninguna variable | — | 1 (nunca verde) | **1** |
| xii | upstream rojo **y** suites ejecutadas | `SECRET_SCAN=failure`, 3 `success` | 1 | **1** |
| xiii | upstream `cancelled`, docs-only | `SECRET_SCAN=cancelled` | 1 | **1** |

`=== 13 pass / 0 fail ===`

La cara **iii** es la que protege el coste: un PR docs-only con todo verde sale **0**,
así que no se ejecuta ninguna suite y el guard sigue costando lo mismo que antes (un
job de 5 min de `ubuntu-latest` que ya existía). La cara **iv** es el agujero de la
fila, ahora cerrado.

### `actionlint` — con y sin shellcheck, cronometrado y con tope

`FIND-785` midió que un `run:` que gana una sentencia puede colgar `actionlint`. Se
midió con **tope duro de 90 s** (`Start-Process` + `WaitForExit`, kill si expira):

| Comando | Resultado |
|---|---|
| `actionlint .github/workflows/ci-rust.yml` | **exit 0 — 1,42 s** |
| `actionlint -shellcheck= .github/workflows/ci-rust.yml` | **exit 0 — 0,14 s** |

Sin cuelgue. El bloque nuevo son 3 sentencias top-level (un `for`, un `if`, un `case`)
más el `if` de la rama de skip que ya existía: la lógica se escribió **compuesta**
(`for` + `case`), que es lo que el lote pide. `shellcheck -s bash -S style` sobre el
script extraído: **exit 0**. (`-S style` sin `-s bash` da un único SC2148 por no haber
shebang, artefacto de extraer un bloque suelto.)

### Lo que **no** se verifica acá

- **Que GitHub evalúe el ruleset así.** No se aplicó ningún cambio de ruleset (no hace
  falta: el nombre del context no cambia). La prueba de campo es un PR docs-only con el
  `Secret Scan` en rojo: debe quedar en `Tests Executed (guard): failure`.
- **El orden de `needs` en un run real.** El harness stubea `needs.*`; que GitHub
  espere a `secret-scan` y a `clippy` antes de arrancar el guard es la semántica
  documentada de `needs` (y `if: always()` ya garantiza que corra aunque fallen).

## Deuda

- **El coste de pared sube con `clippy`.** El guard ya esperaba a las 3 suites (hasta
  90 min); agregar `clippy` (671 s medidos en el run de referencia de `FIND-770`) y
  `secret-scan` (10 min de timeout) no cambia el orden de magnitud, pero en un run de
  PR el guard ahora no puede reportar antes que `clippy` esté.
- **`Tests (embeddings features)` sigue fuera** del contrato del guard (no es required),
  igual que en `FIND-787`.
- **El guard sigue sin distinguir `cancelled` de `skipped` en el mensaje** (nit
  declarado en `FIND-787`): el `needs` global dice "did not execute", que ya es correcto.

## Review

**P2-01 - `vanta-review`, contexto fresco, una ronda:** ver el bloque `RESULTADO` del
lote y la sección de review de `docs/dev/avance/activo/ci-cd.md`.

## NOTICED BUT NOT TOUCHING

- `ci-gate.yml:96` sigue tratando `skipped` como verde (política `EST-03`, otro blast
  radius): la misma semántica una capa más arriba, fuera de esta fila.
- `.github/workflows/certification.yml`, `release-verify.yml`,
  `snapshot-certification.yml`: zona prohibida de este lote (otro carril).
