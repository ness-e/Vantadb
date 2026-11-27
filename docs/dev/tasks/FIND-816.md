---
title: "FIND-816 - docs-gates.yml filtraba el push por 21 patrones: el 17,8 % del árbol quedaba fuera y sus 4 required ausentes"
kind: task
status: completed
description: "El filtro de `on.push.paths` movido DENTRO del workflow, a un job `docs-scope` que resuelve el rango del push con `event.before` (el idioma de FIND-807), y los 3 jobs caros gateados por él. Los 4 contexts required de este fichero reportan siempre."
tags: [vantadb, ci, github-actions, docs-gates, paths-filter, required-checks]
---

# FIND-816 - tres workflows, la misma clase (`FIND-797`, `FIND-807`, este)

## Contexto

`docs-gates.yml` filtraba el `push` por **21 patrones** (con `docs/**` primero) y su
`:64` afirmaba lo contrario de lo que su `:51` declaraba: *"`push` conserva el filtro:
ahi no hay required check que satisfacer"*. Hay cuatro.

**Lo medido, re-medido hoy** con `git ls-files` y los 21 patrones aplicados con la
semántica de GitHub (`*` no cruza `/`, `**` sí):

```text
tracked files: 2415
outside the 21 patterns: 430 (17.8 %)
     81  integrations/
     61  vanta-proxy/
     52  dev-tools/
     38  examples/
     27  benches/
     26  skills/
     25  evals/
     21  benchmarks/
     16  assets/
     12  fuzz/
     11  .github/     (raíz del directorio: dependabot.yml, etc.)
      8  embeddings/
      ... y sueltos: Justfile, deny.toml, build.rs, LICENSE, .gitleaks.toml
```

La fila decía **426 de 2351 (18 %)**; el árbol creció y el número con él. Misma
proporción, misma conclusión: **los 4 contexts required que salen de este fichero
(`Internal links resolve`, `Frontmatter schema, kind agreement, orphans`, `Generated
indexes are current`, `Generate API reference (rustdoc)`) quedaban ausentes** en
cualquier push cuyo rango sólo tocara esos 430 ficheros - y un context required ausente
es `Pending` para siempre (`FIND-765`), tapado hoy por el bypass de admin del ruleset.

## Contrato

La **misma solución que `FIND-807`** (`1c6cee680`, ya probada en producción): el filtro
se resuelve **dentro del job**, no en `on.push`, con `github.event.before`. Y un
chequeo que compare los 21 patrones contra los ficheros que los gates realmente juzgan.

## Impacto mapeado (Regla 0)

**Leídos completos:** `.github/workflows/docs-gates.yml` (16 jobs), el job `rust-scope`
de `ci-rust.yml` (`:99-202`, de donde sale el idioma), `docs/dev/tasks/FIND-787.md`
(§D3, §D6 y el borde del ruleset) y `docs/dev/avance/activo/ci-cd.md` (la entrada de
`FIND-807`, que ya declaraba esta fila como `NOTICED BUT NOT TOUCHING`).

**Referencias hacia dentro:**

| Consumidor | Efecto |
|---|---|
| ruleset `23692587` (`develop`) | matchea **por nombre**; ningún job cambia de nombre ⇒ los 4 contexts siguen siendo los mismos |
| ruleset `16844590` (`main`) | igual (6 contexts de este fichero) |
| `ci-gate.yml` (`REQUIRED`) | espeja el ruleset de `main` por nombre; un job **nuevo** (`docs-scope`) no lo rompe: sólo audita nombres listados |
| `docs/dev/operations/CI_POLICY.md` | ya estaba stale por `FIND-807` (declarado allí) y sigue fuera del write-scope de este carril |

**Referencias hacia afuera:** ninguna. `docs-scope` no publica artifacts ni consume
outputs ajenos; los 3 jobs caros sólo ganan `needs: [docs-scope]`.

**Veredicto:** aditivo + un filtro reubicado. El riesgo no es el YAML: es (a) que un job
required deje de reportar (lo contrario de lo que se arregla) y (b) que el coste de
runner se dispare. Los dos se miden abajo.

## Spec - decisiones

| # | Decisión | Elegido | Alternativa descartada | Por qué |
|---|---|---|---|---|
| D1 | Dónde vive el filtro | job `docs-scope` con `event.before`/`base.sha` | quitar el filtro y correr los 16 jobs en todo push | 967 s de runner por push (medidos en `45771790202`), 84 % en 3 jobs. `FIND-807` ya eligió este idioma y está probado en producción: reusarlo, no inventarlo. |
| D2 | Qué se gatea | los **3 caros** (`doc-examples` 466 s, `doctests` 270 s, `rustdoc` 73 s) | gatear los 16 | los 13 baratos suman ≤4m10s y **son los que reportan los 4 required**: gatearlos convertiría cuatro required en `skipped` sin ahorrar nada significativo. Es exactamente lo que el comentario `:59-63` de este fichero ya prescribía ("el filtrado fino va DENTRO de esos tres"). |
| D3 | Fail-closed o fail-open | **fail-open** (base irresoluble / fetch / diff / regex rota ⇒ `docs=true`) | fail-closed | misma política que `rust-scope` (`FIND-725/770/807`): un falso positivo cuesta minutos de runner, un falso negativo pierde un gate en silencio. |
| D4 | Forma del `if:` | `... && (needs.docs-scope.outputs.docs == 'true' \|\| needs.docs-scope.result != 'success')` | `needs.docs-scope.outputs.docs == 'true'` a secas | si `docs-scope` falla, la salida está vacía y el job se omitiría: fail-**open** en la dirección peligrosa. El `\|\| result != 'success'` es el patrón ya usado por `clippy` en `ci-rust.yml`. |
| D5 | `rustdoc` es required: ¿se puede gatear? | **sí** | no gatearlo | `skipped` cuenta como 'Success' para el ruleset (`FIND-787`), y la alternativa peor es **no reportar** (`Pending`, `FIND-765`). Gateado sólo se omite cuando el rango no toca ni `src/` ni `Cargo.*` ni ningún crate: nada que rustdoc pueda leer. Declarado, no silencioso. |
| D6 | El "chequeo que la fila pide" | el job **imprime** los ficheros del rango que caen fuera de los 21 patrones, en cada run | una lista de exclusiones escrita a mano | una lista manual es el anti-patrón que este lote cierra. El print convierte la cifra de un comentario en evidencia de cada run: si el 17,8 % crece, o si aparece ahí algo que un gate sí lea, se ve. |

## Verificación — las dos direcciones, **después del review P2-01**

### El review tumbó dos cosas de esta fila, y las dos están cerradas

| # | Hallazgo | Qué se hizo |
|---|---|---|
| **H-1** | **`rustdoc` (REQUIRED) quedaba ciego para `vanta-proxy/`.** Es member del workspace (`Cargo.toml` §members), `cargo doc --no-deps --workspace` lo documenta, y **no casaba ninguno de los 21 patrones**: un PR que sólo lo tocara daba `docs=false` ⇒ `Generate API reference (rustdoc)` = `skipped` = **Success**. Regresión, además: antes de este cambio `pull_request` no llevaba `paths:`, así que rustdoc corría en todo PR. | (a) `vanta-proxy/` entra en `PATTERNS`; (b) el "chequeo" dejó de ser un **print** y pasó a ser una **reconciliación ejecutable**: un step propio recorre todo `*/Cargo.toml` y falla si un crate del árbol no casa ningún patrón. La lista de crates **no se escribe a mano**: se deriva, y la única excepción es un crate que declara su propio `[workspace]` (hoy `fuzz/`), porque entonces no es member y rustdoc no lo lee. |
| **M-2** | `rustdoc` es required y conservaba `if: github.actor != 'dependabot[bot]'`: todo PR de Dependabot lo reportaba `skipped` = Success, justo la semántica de `FIND-787` que este commit hace *cargable*. | se le **quitó** el guard de actor, con el mismo razonamiento que este fichero ya aplica a otro required (`:351-354`, FIND-472). Coste declarado: rustdoc es el job más caro del fichero (~1m30 s caliente, 13m05 s frío) y un bump de dependencias es exactamente lo que puede romper un `cargo doc -D warnings`. |
| **L-2** | La rama de skip del guard (`SCOPE=false && SCOPE_RESULT=success`) sale 0 **antes** de mirar las tres suites. | **declarado, no cambiado**: con `docs=false` las suites están fuera de alcance por decisión medida, y un run cancelado no gatea el head nuevo. Cambiar ese orden fue el bug que `FIND-815` cerró por el otro lado (upstream sí se mira antes). |
| **M-1** | `Generated indexes are current` (REQUIRED) está **rojo en develop** y esta fila añade 5 documentos que no están en `docs/index.md`. | **declarado**: `node scripts/docs/gen-index.mjs --check` → exit 1 (y el rojo que nombra ni siquiera es de estos ficheros: es una fila `EMB-10`). No se arregla aquí porque `--write` indexaría ficheros de otros carriles (la clase de `FIND-792`) y `docs/index.md` es **zona prohibida** de este lote. |

**La lección de H-1, que vale para el resto del lote:** un *print* no es un chequeo. Lo que
cierra la clase es un gate que pueda **fallar**, y este ahora puede: se probó creando un
crate falso (`zz-fake-crate/Cargo.toml`) → **exit 1** nombrando el crate.

### El cuelgue de `actionlint`... dentro de un comentario

Tres versiones de la reconciliación colgaron `actionlint` **con shellcheck** (>120 s,
matadas; sin shellcheck, 0,2-0,7 s — el modo en que `FIND-785` lo midió). Causa aislada
por bisección, con tope duro en cada medida:

| Variante | `actionlint` (con shellcheck) |
|---|---|
| bloque nuevo con un `if ... continue` dentro del bucle | **cuelga >120 s** |
| mismo código, con sustitución de patrón sobre el manifest | **cuelga >120 s** |
| mismo código, **comentarios sin sintaxis de shell** | **0,85 s** |
| mismo código otra vez, comentario que *cita* esa sintaxis | **cuelga >120 s** |
| **solución final**: bucle en un **step propio** + `PATTERNS` en `env:` del job | **1,35 s** / 0,13 s sin shellcheck |

O sea: no es sólo "una sentencia nueva" - **el texto del comentario también lo dispara**, y
la salida robusta es mover la lógica a su propio `run:` y dejar el comentario en prosa.
Los 20 workflows juntos: **4,81 s**, exit 0.

### Las caras, contra el `run:` extraído del fichero

El `run:` no se copia: un extractor lo saca del YAML. `PATTERNS` ya no vive dentro del
script (está en `env:` del job), así que el harness lo extrae del YAML y lo pasa como
variable de entorno, igual que GitHub. `git` es falso (sin red, sin repo) y `GITHUB_OUTPUT`
va a un temporal.

| # | Cara | Esperado | Obtenido |
|---|---|---|---|
| a | push, sólo `docs/` | true | **true** |
| b | push, sólo `dev-tools/` (territorio del 15,3 %) | false | **false** |
| c | push con `src/` | true | **true** |
| d | primer push (`before` nula) | true | **true** |
| e | `schedule` / `workflow_dispatch` | true | **true** |
| f | PR con `docs/` | true | **true** |
| g | `git fetch` falla | true (fail-open) | **true** |
| h | `git diff` falla | true (fail-open) | **true** |
| i | **`src-tauri/` no debe casar `src/`** | false | **false** |
| j | `.github/` raíz (no `workflows`/`actions`) | false | **false** |
| k | `.github/workflows/` | true | **true** |
| l | `Justfile` | false | **false** |
| m | mezcla dentro+fuera | true | **true** |
| n | rango vacío | false | **false** |
| o | `vantadb-python/tests/…` (vía `vantadb-*/`) | true | **true** |
| **p** | **`vanta-proxy/` SOLO — el agujero de H-1** | true | **true** |
| **q** | `fuzz/` (crate fuera del workspace) | false | **false** |
| **r1** | reconciliación sobre el árbol **real** | exit 0 | **exit 0** |
| **r2** | reconciliación con un crate falso fuera de los patrones | exit **1** | **exit 1** |

`=== 19 pass / 0 fail ===`

```text
r2 → ::error::crate(s) that rustdoc documents and no pattern covers: zz-fake-crate - add them to PATTERNS (FIND-816)
```

### `actionlint` — cronometrado, con tope duro

| Comando | Resultado |
|---|---|
| `actionlint .github/workflows/docs-gates.yml` | **exit 0 — 1,35 s** |
| `actionlint -shellcheck= .github/workflows/docs-gates.yml` | **exit 0 — 0,13 s** |
| `actionlint .github/workflows/*.yml` (20 ficheros, como el hook) | **exit 0 — 4,81 s** |

`shellcheck -s bash -S style` sobre los dos bloques extraídos: **exit 0**.

### La dirección que la fila pide, literalmente

> un push que sólo toque un fichero fuera de los patrones **publica** los 4 contexts

Con `on.push.paths` retirado, el workflow **arranca** en todo push a `main`/`develop`: los
13 jobs no gateados corren y reportan. La cara `b` prueba que `docs-scope` decide que los 3
caros no tienen nada que juzgar; los 4 required son de los 13 y **reportan siempre**.

### Lo que **no** se verifica acá

- **Que GitHub evalúe el ruleset así.** No se tocó ningún ruleset: los nombres de los 4
  contexts no cambian y `docs-scope` es un job nuevo que nadie exige.
- **El coste real del run.** Se cita el medido en `45771790202` (967 s, 84 % en los 3
  gateados) porque este carril no puede correr un push real.

## Deuda

- **`CI_POLICY.md` queda más stale**: ya decía "`on.push` con sus 24 intactos" para
  `ci-rust` (deuda declarada por `FIND-807`); ahora tampoco `docs-gates` filtra el push.
  Fuera del write-scope de este carril.
- **Los 369 ficheros de fuera (15,3 %) siguen sin gate de docs.** El cambio no los mete en
  ningún gate: hace que su cambio **no apague** los required, imprime la lista en cada run
  y **falla** si un crate del árbol queda fuera de los patrones (la reconciliación). Si
  alguno pasa a ser material de docs, el sitio es `PATTERNS` y la reconciliación lo va a
  exigir.
- **M-1 (del review, declarado):** `Generated indexes are current` es un required que está
  **rojo en develop** y esta fila agranda la deriva con 5 task files nuevos.
  `node scripts/docs/gen-index.mjs --check` → **exit 1**, y el rojo que nombra ni siquiera
  es de estos ficheros (es una fila `EMB-10`). No se arregla acá porque `--write` indexaría
  ficheros de otros carriles (clase `FIND-792`) y `docs/index.md` es **zona prohibida** del
  lote. Lo resuelve el carril del índice.
- **`rustdoc` gateado por `docs-scope`** es una decisión declarada (D5): un required que se
  omite con una medición detrás. Si el owner prefiere que corra siempre, se le quita el
  `needs` y cuesta 73 s más por run docs-only. Ya no se omite en PRs de Dependabot (M-2).
- **Sin compilar Rust** (dos carriles vivos): nada de este cambio lo necesita.

## Review

**P2-01 - `vanta-review`, contexto fresco, una ronda. Veredicto: `changes-required` (2 High,
2 Medium, 3 Low, 2 Info).** Reprodujo las premisas por su cuenta (gitleaks 8.24.3 con el
config viejo y el nuevo, el harness sobre el `run:` extraído, el regex de los patrones sobre
`git ls-files`, `gh api` del ruleset y de los check-runs, `actionlint` con tope: 20
workflows en 23,89 s). Cerró confirmando que los 6 commits están correctamente acotados y
sin tocar zonas prohibidas.

| # | Hallazgo | Estado |
|---|---|---|
| H-1 | `rustdoc` (required) ciego para `vanta-proxy/`: member del workspace, documentado por `cargo doc --workspace`, fuera de los 21 patrones | **cerrado**: patrón añadido + reconciliación ejecutable con 2 caras (r1 verde, r2 roja) |
| H-2 | `FIND-787` registra una "eliminación" falsa (había 3 required en failure, no 1) | **cerrado en su propio commit**: ver `docs/dev/tasks/FIND-787.md` § "Corrección del review" |
| M-1 | `Generated indexes are current` rojo y esta fila lo agranda, sin declararlo | **declarado** arriba (zona prohibida + clase `FIND-792`) |
| M-2 | `rustdoc` required seguía con el guard de Dependabot | **cerrado**: guard de actor eliminado, coste declarado |
| L-1 | `MSRV`, `Experimental Crates Check` y `Miri` siguen siendo skippables por actor | **declarado** en `FIND-815.md`: el guard no los juzga y el coste de quitarlo es del owner |
| L-2 | La rama de skip no mira las tres suites | **declarado**: con `docs=false` están fuera de alcance por decisión medida |
| L-3 | El hook de `.opencode` era fail-open si faltaba `node` | **cerrado**: ahora bloquea (commit en `.opencode`) |
| I-1 | `FIND-810` dejó de estar bloqueado a mitad del review | correcto: el bloqueo fue real hasta `f5ac57998` |
| I-2 | La verificación manual de los commits con exit 7/1 fue suficiente | confirmado contra los blobs |

**Y el hallazgo que el review no buscaba:** el cuelgue de `actionlint` **también lo dispara
el texto de un comentario** dentro de un `run:` (bisección en la tabla de arriba). Es un
dato nuevo para `FIND-785`.

## NOTICED BUT NOT TOUCHING

- `.github/workflows/{certification,release-verify,snapshot-certification}.yml` son zona
  prohibida de este lote (otro carril).
- `docs-schema`/`docs-links`/`docs-index` no se gatean: son required y baratos, y su
  valor es justamente reportar siempre.
- El `schedule` semanal (`cron '23 7 * * 1'`) sigue igual: `docs-scope` devuelve `true`
  en eventos que no son push/PR, así que el barrido no pierde nada.
