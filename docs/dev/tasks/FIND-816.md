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

## Verificación — las dos direcciones

### `docs-scope`: 15 caras contra el `run:` extraído del fichero

El `run:` **no se copia**: un extractor lo saca del YAML (bloque `run: |` del step,
dedentado → 66 líneas, 37 no vacías no-comentario) y corre en bash 5.3.9 con `env -i`,
un `git` **falso** en `PATH` (sin red, sin repo) y `GITHUB_OUTPUT` a un fichero temporal.

| # | Cara | Entradas | Esperado | Obtenido |
|---|---|---|---|---|
| a | push, sólo `docs/` | `docs/api/MCP.md` | `docs=true` | **true** |
| b | push, sólo `dev-tools/` (territorio del 17,8 %) | `dev-tools/verify.ps1` | `docs=false` | **false** |
| c | push con `src/` | `src/lib.rs` | true | **true** |
| d | primer push de una rama (`before` nula) | `-` ×40 | true | **true** |
| e | `schedule` / `workflow_dispatch` | — | true | **true** |
| f | PR con `docs/` | `docs/index.md` | true | **true** |
| g | `git fetch` falla | — | true (fail-open) | **true** |
| h | `git diff` falla | — | true (fail-open) | **true** |
| i | **`src-tauri/` no debe casar `src/`** | `src-tauri/main.rs` | false | **false** |
| j | `.github/` raíz (no `workflows`/`actions`) | `.github/dependabot.yml` | false | **false** |
| k | `.github/workflows/` | `.github/workflows/ci-rust.yml` | true | **true** |
| l | `Justfile` (fuera) | `Justfile` | false | **false** |
| m | mezcla dentro+fuera | `dev-tools/a` + `docs/b.md` | true | **true** |
| n | rango vacío | — | false | **false** |
| o | `vantadb-python/tests/…` (vía `vantadb-*/`) | `test_sdk.py` | true | **true** |

`=== 15 pass / 0 fail ===`

Y el print de cobertura, en la cara `b` (la que más importa: un push que hoy **no
producía run**):

```text
--- files changed in aaaa..HEAD ---
dev-tools/verify.ps1
Justfile
--- outside the 21 patterns (no docs gate here reads them) ---
dev-tools/verify.ps1
Justfile
GITHUB_OUTPUT: docs=false
```

### La dirección que la fila pide, literalmente

> un push que sólo toque un fichero fuera de los 21 patrones **publica** los 4 contexts

Con `on.push.paths` retirado, el workflow **arranca** en todo push a `main`/`develop`: los
13 jobs no gateados corren y reportan. `docs-scope` decide (cara `b`) que los 3 caros no
tienen nada que juzgar y los omite; los 4 required son de los 13 y **reportan siempre**.

### `actionlint` — cronometrado, con tope duro

`FIND-785` midió que un `run:` que gana una sentencia puede colgar `actionlint` (33 h
bloqueado, 0,8 s de CPU). Medido con `Start-Process` + `WaitForExit` y **kill** al tope:

| Comando | Resultado |
|---|---|
| `actionlint .github/workflows/docs-gates.yml` | **exit 0 — 0,60 s** |
| `actionlint -shellcheck= .github/workflows/docs-gates.yml` | **exit 0 — 0,17 s** |
| `actionlint .github/workflows/*.yml` (20 ficheros, como el hook) | **exit 0 — 6,53 s** |

Sin cuelgue. La lógica nueva se escribió **compuesta** (`case`) como el lote pide, y el
primer intento sirvió para algo: actionlint **sí** cazó un defecto real antes del
commit — `property "docs" is not defined in object type {}` en las 3 líneas del `if:`,
porque el job no declaraba `outputs:`. Se agregó `outputs.docs: ${{ steps.scope.outputs.docs }}`.
`shellcheck -s bash -S style` sobre el bloque extraído: **exit 0**.

### Lo que **no** se verifica acá

- **Que GitHub evalúe el ruleset así.** No se tocó ningún ruleset: los nombres de los 4
  contexts no cambian y `docs-scope` es un job nuevo que nadie exige. La prueba de campo
  es un push que sólo toque, por ejemplo, `dev-tools/**`: debe mostrar los 4 contexts
  reportados y los 3 caros `skipped`.
- **El coste real del run.** Se cita el medido en `45771790202` (967 s, 84 % en los 3
  gateados) porque este carril no puede correr un push real. Lo que **sí** está medido
  aquí es qué jobs se gatean y con qué condición.

## Deuda

- **`CI_POLICY.md` queda más stale**: ya decía "`on.push` con sus 24 intactos" para
  `ci-rust` (deuda declarada por `FIND-807`); ahora tampoco `docs-gates` filtra el push.
  Fuera del write-scope de este carril.
- **Los 430 ficheros de fuera siguen sin gate de docs.** El cambio no los mete en ningún
  gate: hace que su cambio **no apague** los required, e imprime la lista en cada run
  para que la decisión de extender los patrones sea con evidencia. Si alguno pasa a ser
  material de docs, el sitio es `PATTERNS`, y el print lo va a mostrar.
- **`rustdoc` gateado por `docs-scope`** es una decisión declarada (D5): un required que
  se omite con una medición detrás. Si el owner prefiere que corra siempre, se le quita
  el `needs` y cuesta 73 s más por run docs-only.
- **Sin compilar Rust** (dos carriles vivos): nada de este cambio lo necesita.

## Review

**P2-01 - `vanta-review`, contexto fresco, una ronda:** ver el `RESULTADO` del lote y la
sección correspondiente de `docs/dev/avance/activo/ci-cd.md`.

## NOTICED BUT NOT TOUCHING

- `.github/workflows/{certification,release-verify,snapshot-certification}.yml` son zona
  prohibida de este lote (otro carril).
- `docs-schema`/`docs-links`/`docs-index` no se gatean: son required y baratos, y su
  valor es justamente reportar siempre.
- El `schedule` semanal (`cron '23 7 * * 1'`) sigue igual: `docs-scope` devuelve `true`
  en eventos que no son push/PR, así que el barrido no pierde nada.
