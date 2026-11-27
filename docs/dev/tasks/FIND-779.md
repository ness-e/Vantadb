---
title: "FIND-779: las cinco aristas del gate de contrato del harness (2 falsos positivos, 3 cegueras)"
kind: task
status: active
description: "check-harness-contract.mjs falla abierto en 5 aristas medidas: repo-integrity marca un `git rm --cached` masivo como inversión (FP); HAS_MEASURE se evalúa sobre la fila entera, así que «no medido» pasa como medido; un pipe escapado desalinea la celda juzgada; un header sin palabra de prosa se salta en silencio; y Problema/Detalle no tenía fixture. Cada arista con fixture que muerde."
---

# FIND-779: las cinco aristas del gate de contrato del harness

## Metadata

- **Plan file:** `docs/dev/plans/2026-11-26-batch-parciales-y-nuevas.md` (W13)
- **Fuente:** `docs/dev/Backlog.md` fila `FIND-779` (Media, 2026-11-26), salida de las rondas 4 y 5 del review de cierre de `H-02-H-07`
- **Origen:** `docs/dev/reviews/2026-11-26-harness-hardening.md` §B (contrato del gate)
- **Tipo:** JS (dev-tools, gate del harness) + fixtures
- **Zona:** `dev-tools/check-harness-contract.mjs` (exclusiva de esta tarea)
- **Estado:** ⏳ IN PROGRESS

## Síntoma medido (las cinco aristas, re-medidas antes de tocar nada)

```
node dev-tools/check-harness-contract.mjs --self-test   →  38/38 fixtures passed, exit 0
node dev-tools/check-harness-contract.mjs                →  7 checks, exit 0
```

**Falsos positivos**

- **(a)** `repo-integrity` (`:504-544`) señala "HEAD/worktree inversion" cuando el último commit
  borra ≥50 paths que **siguen en disco**. Un `git rm --cached` masivo hace exactamente eso: el
  commit borra los paths del árbol y deja los ficheros donde están. El docstring se declara
  `false-positive-safe` (`:504`) y no lo es.
- **(b)** `HAS_MEASURE` (`:123`, usado en `:194`) se evalúa sobre `text`, la **fila entera**, antes
  de saber qué columna se juzga. `no medido` matchea `medido\b` → la fila pasa **sin medición**.
  Una celda no juzgada con la palabra `medido` también absuelve a la celda del claim.

**Cegueras (el gate queda en verde sin haber mirado)**

- **(c)** `claimViolations` parte por `|` (`:206`) y `governingHeader` también (`:152`). Un pipe
  escapado (`a\|b.rs`) crea una celda de más: los índices se desalinean y se juzga la celda
  equivocada (verde, no "no mirado" declarado).
- **(d)** La rama fail-closed (`:212-221`) exige que **alguna** columna matchee `PROSE_HINT`
  (`hallazgo|finding|problema|brecha|tarea|descripcion|detalle|nota|impacto|accion`). Un header
  renombrado a `Observación/Comentario` deja `proseCols` vacío → `claimed` vacío → `continue`:
  vuelve a ser un skip, que es justo lo que la ronda 4 cerró.
- **(e)** `--self-test` no tiene fixture de la rama `synHeader` (`Problema/Detalle`): la salida
  "sigue en exit 1" no es reproducible desde el repo, y cualquier edición futura de `PROSE_HINT`
  puede perder esa rama sin que nada falle.

**Contrato del backlog (repetido aquí porque es el criterio de cierre):** cada arista con su
fixture **que muerda**, y el gate **sin ningún caso donde devuelva exit 0 con un claim presente**.

## Contrato

1. `HAS_MEASURE` se evalúa sobre **las celdas juzgadas** (`Hallazgo`/`Descripción`), nunca sobre
   la fila, y la **negación niega**: `no medido` / `sin medir` / `sin Medición:` no son medición.
   Un `Medición:` **sin valor** tampoco (la clase `FIND-785`: un campo que se cumple solo).
   Misma clase que `FIND-794` (un filtro que matchea 0 sale verde).
2. `repo-integrity` distingue `git rm --cached` de un borrado real **por el índice** (un untrack
   deja los paths sin trackear; el incidente los deja trackeados). Lo que **no** puede distinguir
   lo **dice**: el docstring pierde `false-positive-safe` y gana el techo declarado, y el caso
   ambiguo (≥50 paths en disco y sin trackear) se **imprime** como nota, nunca se calla.
3. (c) filas y headers se parten por **pipes no escapados**; (d) una tabla de datos con columna
   `ID` y **ninguna** columna reconocible es una violación declarada ("cannot judge"), no un skip;
   (e) fixture de `Problema/Detalle` que muerde.
4. Los **38 fixtures** actuales siguen verdes, más uno por arista. Cada fixture nuevo se prueba
   contra el binario de `HEAD` (`git show HEAD:dev-tools/check-harness-contract.mjs`): los que
   discriminan dan `exit 0` en el viejo y `exit 1` en el nuevo; los que solo son guardia (para
   probar que el fix no apagó el check) se declaran como tales.

## Blast Radius

| Dirección | Módulos / símbolos |
|-----------|--------------------|
| Callers | `/harness` (`.opencode/commands/harness.md:13`, `agents/vanta-harness.md:73,91`), `pipeline-run.md:252` (check `wave-quotas`), `findings.md:50` (`--rows`, gate al crear fila), `plan.md:192` |
| Símbolos exportados | `claimViolations` (único consumidor: `checkMedicion` + `--rows`), `governingHeader`, `claimCellIndices`, `isForbiddenCommitLine`, `loteViolations`, `parsePlanRows`, `waveConflicts`, `rustLaneCensus`, `authorityViolations`, `isActivePlan` |
| Nuevo export | `hasMeasure` (puro, lo usan `claimViolations` y `--self-test`), `repoIntegrityViolations` (para que el fixture de (a) sea posible sin abrir un repo git de verdad como `ROOT`) |
| Propiedad ajena | `dev-tools/git-commit-isolated.ps1` y `dev-tools/check-closure.ps1` (**cerrados hoy por `FIND-780`**), `src/**`, `vantadb-mcp/**`, `vanta-memory/**`, `.opencode/**` (`FIND-774`), `.github/**`, `docs/dev/plans/**` → **no se tocan** |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `dev-tools/check-harness-contract.mjs` (1001 L) ·
  `.opencode/task-system/prompts/findings.md` (plantilla de la fila + "la Medición va en la celda
  de la Descripción", `:42-51`) · `docs/dev/Backlog.md` (fila `FIND-779` + cabeceras de las tres
  tablas) · `docs/dev/tasks/FIND-775.md` (formato de task file y precedente del Backlog
  reconstruido) · `docs/dev/avance/activo/operaciones.md` (dos últimas entradas, formato).
- **Referencias hacia dentro:** `check-harness-contract.mjs` no es importado por nadie; se invoca
  como proceso (`node ... [--self-test|--rows|--plan|--root|--ref]`). Los únicos contratos que
  expone son el **exit code** y el prefijo de línea (`ok`/`FAIL`/`ERROR`).
- **Referencias hacia fuera:** git (`rev-parse`, `show --diff-filter=D`, `ls-files`, `diff`),
  `fs` (lectura de Backlog/plan/prompts).
- **Veredicto impacto:** medio-bajo. Un solo fichero, sin consumidores por import, cuyo contrato
  observable son exit codes. El riesgo real es de **falso negativo** (apagar el gate al
  "arreglarlo"), y por eso cada cambio va con su fixture y con la medición contra `HEAD`.

## Spec (decisiones)

| # | Decisión | Alternativa descartada | Por qué |
|---|----------|------------------------|---------|
| 1 | La medición se busca en las **celdas juzgadas** (`cells.some(hasMeasure)`), y `hasMeasure` exige valor tras `Medición:` y rechaza la negación | seguir evaluando la fila, con una regex que excluya `no medido` | la fila entera absuelve siempre: cualquier columna no juzgada puede decir `medido`. Y negar solo la frase deja intacto el resto del agujero |
| 2 | `repo-integrity` clasifica por el **índice** (`git ls-files`): trackeado + en disco = inversión; sin trackear = untrack consistente | usar `git status` o el `git show` a secas | `git status --porcelain` es la misma información con más parseo; `git show` es lo que ya se lee y no distingue nada |
| 3 | Umbral ≥50 se mantiene **solo** para la violación; por debajo se imprime nota | bajar el umbral | el umbral existe porque un borrado en reparación (index re-añadido) es benigno; cambiarlo sin medir es ruido nuevo. La nota deja de ser silencio |
| 4 | El caso ambiguo (≥50 en disco y sin trackear) se imprime como `note` **siempre**, no bajo `--verbose` | callarlo para "no ensuciar" | es la lección de la tarea: un check que no puede distinguir debe **decirlo**. La nota es exactamente `git rm --cached` o un borrado cuyo índice se limpió después — el gate no puede probar intención |
| 5 | `splitCells` (pipes no escapados) se usa en filas **y** headers del camino de claims | arreglar solo la fila | desalinear un lado es exactamente el bug: el índice de la fila y el del header tienen que salir del mismo splitter |
| 6 | Tabla de datos (columna `ID`) sin columna reconocible → **violación con mensaje propio** ("cannot judge") | ampliar `PROSE_HINT` con `observacion|comentario` | una lista de palabras es un parche eterno: el siguiente rename vuelve a pasar. Y el mensaje propio permite que el fixture muerda contra la mutación de `PROSE_HINT` (si no, la rama (d) enmascara a la (e)) |
| 7 | `repoIntegrityViolations(root, ref)` se exporta | inyectar un runner de git falso | el fixture abre un repo git desechable real (`git init` + commits) y juzga el **efecto**, no un mock. Un mock del runner probaría que sé escribir el mock |

## Steps

- [x] 1. Medición del antes: `--self-test` 38/38 exit 0 · gate 7 checks exit 0 · las 5 aristas reproducidas contra el binario de `HEAD`.
- [x] 2. RED: fixtures 8N-1..5 escritos y medidos contra `git show HEAD:dev-tools/check-harness-contract.mjs` (exit 0 en el viejo donde discrimina).
- [x] 3. GREEN (b): `hasMeasure` sobre las celdas juzgadas + negación que niega.
- [x] 4. GREEN (c): `splitCells` en filas y headers.
- [x] 5. GREEN (d): tabla con `ID` sin columna reconocible → violación "cannot judge".
- [x] 6. GREEN (a): `repoIntegrityViolations` clasifica por índice, nota el ambiguo, docstring sin `false-positive-safe`.
- [x] 7. VERIFY: `--self-test` (38 → 48) · gate completo · cada fixture nuevo contra `HEAD` y contra el nuevo · 8 mutaciones.
- [x] 8. Review P2-01 ronda 1 **REQUEST CHANGES** → F1 (merge ciego) + F2 (fixture enmascarada) + F4/F5/F8/N1/N2 corregidos, cada uno con su fixture; tabla de mutaciones regenerada (`--self-test` 48 → 54).
- [ ] 9. Review P2-01 ronda 2 (artefacto pinneado por sha256) + `ocr-review.ps1`.
- [ ] 10. Cierre: fila fuera del Backlog (solo mi línea), entrada en `docs/dev/avance/activo/operaciones.md`, commit local con el helper.

## Verify (medido)

```
node dev-tools/check-harness-contract.mjs --self-test   →  38/38 antes  ·  62/62 después (exit 0)
node dev-tools/check-harness-contract.mjs                →  7 checks, exit 0 (antes y después)
node dev-tools/check-harness-contract.mjs --rows         →  exit 0 (camino que corre al crear una fila)
node dev-tools/check-harness-contract.mjs --plan <plan>  →  exit 0 (consumidor de pipeline-run)
```

**Cada fixture nuevo contra el binario de `HEAD`** (`git show HEAD:dev-tools/check-harness-contract.mjs`,
byte-idéntico al de la corrida: `sha256 c342eb76…`, 38 381 B), mismo input por los dos lados (sonda
desechable: repo git propio para (a), `--rows` sobre un `Backlog.md` de juguete para el resto):

| Fixture | Arista | `HEAD` | nuevo | ¿discrimina? |
|---|---|---|---|---|
| `8N-1a` `git rm --cached` ×60 (en disco, sin trackear) | (a) FP | exit **1** — `deleted 60 paths that are STILL in the working tree` | exit **0** — `ok` + `note … consistent with git rm --cached` | sí (el FP desaparece) |
| `8N-1b` índice obsoleto que borra 60 **trackeados** | (a) TP | 1 | 1 | guardia: el fix no apaga el check |
| `8N-1c` los 60 + `café.md` | (a) quoting | 1, cuenta **60** de 61 | 0, cuenta **61** | sí |
| `8N-1d` **merge commit** que borra 60 trackeados | (a) merge (F1) | 1 | 1 | guardia: muerde la regresión `diff-tree` (0 paths) |
| `8N-1e` `.git/index` corrupto | (a) índice ilegible (F5) | 1 | 1 **+ `note could not read the index`** | guardia: la nota es nueva |
| `8N-1f` exactamente **50** invertidos | (a) umbral (O3) | 1 | 1 | guardia: muerde `>=` → `>` |
| `8N-1g` **49** invertidos | (a) umbral (O3) | 0, **en silencio** | 0 + `note … inversion in repair` | el exit no cambia: lo que cambia es que deja de callarse |
| `8N-2a` celda del claim: `**92** — no medido` | (b) | **0** | **1** | sí |
| `8N-2b` `medido` solo en `Origen` | (b) | 0 | 1 | sí |
| `8N-2c` `**92** — Medición:` sin valor | (b) | 0 | 1 | sí |
| `8N-2d` `no fue medido` / `sin ser medido` | (b) F8 | 0 | 1 | sí |
| `8N-2e` prosa negada **antes** + `Medición:` real | (b) F8 (FP) | 0 | 0 | guardia: el campo legítimo no se pierde |
| `8N-2f` `Medición: n/a`, `tbd`, `N/A` | (b) F8/O2/R3-6.4 | 0 | 1 | sí |
| `8N-2g` `no está medido` | (b) F8a | 0 | 1 | sí |
| `8N-2h` `sin Medición: 92` / `sin Medición: ver nota` | (b) F8b | 0 | 1 | sí |
| `8N-2i` `Medición: no medido` (valor negado) | (b) O4 | 0 | 1 | sí |
| `8N-2j` `Medición: <cmd> · no medido` (prosa **después**) | (b) O1 (FP) | 0 | 0 | guardia: el valor no se contamina con la cola |
| `8N-2k` `sin  Medición: 92` (doble espacio) | (b) R3-1 | 0 | 1 | sí |
| `8N-2l` `sin embargo, medido en prod` | (b) R3-4 (FP) | 0 | 0 | guardia: la negación no se come la coma |
| `8N-3` pipe escapado en la **fila** | (c) | 0 | 1 | sí |
| `8N-3b` pipe escapado en el **header** | (c) F4 | 0 | 1 | sí |
| `8N-4` header `Observación/Comentario` | (d) | 0 | 1 (`cannot judge`) | sí |
| `8N-5a` `Problema/Detalle` + claim | (e) | 1 | 1 (`non-canonical header`) | **no** — cobertura pura: `HEAD` ya acertaba y nada lo guardaba |
| `8N-5b` `Problema/Detalle` + `Medición:` solo en `Evidencia` | (e)+(b) | 0 | 1 (`non-canonical header`) | sí |

**Mutaciones (21; una fixture que sobrevive al borrado de su protección no es una fixture):**

```
hasMeasure: la negación deja de negar              → 3 rojas: 8N-2a, 8N-2d, 8N-2g
hasMeasure: la negación gana al campo (orden F8)   → 2 rojas: 8N-2e, 8N-2j
hasMeasure: acepta placeholders                    → 1 roja: 8N-2f
negación: ventana \w+ en vez de \S+ (F8a)          → 1 roja: 8N-2g
campo: sin lookbehind "sin " (F8b)                 → 1 roja: 8N-2h
campo: sin negación del valor (O4)                 → 1 roja: 8N-2i
campo: valor sin tope en "·" (O1)                  → 1 roja: 8N-2j
placeholders reducidos a n/a (O2)                  → 1 roja: 8N-2f
fila: split naíf por "|"                           → 1 roja: 8N-3
header: split naíf por "|"                         → 1 roja: 8N-3b
PROSE_HINT pierde "problema"                       → 2 rojas: 8N-5a, 8N-5b
(d) vuelve a "continue"                            → 1 roja: 8N-4
repo: todo trackeado (isTracked => true)           → 2 rojas: 8N-1a, 8N-1c
repo: nada trackeado (isTracked => false)          → 5 rojas: 8N-1b, d, e, f, g
repo: índice ilegible = índice vacío               → 1 roja: 8N-1e
repo: diff-tree sin -m (regresión F1)              → 2 rojas: 8N-1d, 8N-1e
umbral >= 50 → > 50 (O3)                           → 1 roja: 8N-1f
sin la nota "in repair" (O3)                       → 1 roja: 8N-1g
lookbehind \bsin\s (un espacio) (R3-1)             → 1 roja: 8N-2k
ventana \S+ que come la coma (R3-4)                → 1 roja: 8N-2l
MEASURE_PLACEHOLDER sin /i (R3-6.4)                → 1 roja: 8N-2f
```

Ninguna protección queda sin fixture: las cuatro mutaciones que sobrevivieron a la ronda 1/2 del
review (O1, O2, O3, O4) y los dos reabiertos de F8 se cerraron con fixture propia, y la tabla es
reproducible con `--self-test --root <repo>` sobre la copia mutada. Medido de paso: con el índice
corrupto `git diff-tree` **falla** y `git show` no — otra razón por la que el diff se lee con `git show`.

## Deuda declarada

- **Ventana del positivo verdadero:** la clasificación por índice vale **mientras el índice siga
  trackeando** los paths. Si alguien limpia el índice después del commit (medido: `git read-tree HEAD`
  tras un commit stale-index), el gate da `0 violaciones + 1 nota` — declarado y no silencioso
  (P2-01 F6), pero la ventana existe y no se puede estrechar desde el commit solo.
- **Umbral de la nota:** la nota de "en disco y sin trackear" aparece a partir de ≥50 paths; un
  `git rm --cached` de 10 es legítimo y frecuente, y avisar de cada uno convertiría el gate en ruido.
  El techo está en el docstring (P2-01 F5-ii).
- **La (d) es deliberadamente gruesa:** una tabla con columna `ID` y sin columna reconocible **falla
  entera** hasta que su header se renombre. No hay forma barata de adivinar qué columna es la del
  claim — adivinarla *fue* el bug.
- **`parsePlanRows` parte por `|`** con el mismo defecto de (c) en la tabla de waves de los planes
  (P2-01 F9): sin arista medida, no se toca acá; queda como residual de la clase.
- **`hasMeasure` sigue siendo un contrato de texto** (techo del header): cubre `no medido` / `no fue
  medido` / `no está medido` / `sin ser medido` / `sin Medición:` (con uno o más espacios) /
  placeholders **exactos** (`n/a`, `N/A`, `tbd`, `pendiente`, `sin datos`, `-`), no toda negación
  imaginable. Medido y declarado (P2-01 R3-2/R3-3): `sin la Medición: 92` y `no Medición: 92` siguen
  midiendo, y `Medición: pendiente de medir` (placeholder **con cola**) también — el `^…$` exige el
  valor exacto. La palabra suelta `medido` (sin campo) sigue absolviendo a una celda, comportamiento
  heredado sin caso vivo: las filas del Backlog que dicen "medido" traen su `Medición:` en la celda.
- **`.opencode/agents/vanta-harness.md:92`** sigue publicando "fixtures (31 + 13)": con 54 el número
  ya estaba viejo **antes** de esta tarea y ese árbol está en vuelo (`FIND-774`) — clase `FIND-781`.

## Review (P2-01 — tier fast: `dev-tools/**` + `docs/dev/**`)

**Ronda 1** (`vanta-review`, agente distinto, 2026-11-26): **REQUEST CHANGES** — premisa confirmada
por re-medición propia (reprodujo las cinco aristas contra `HEAD` con input idéntico: 8N-1a 1→0,
8N-1b 1→1, 8N-2a/2b/2c 0→1, 8N-3 0→1, 8N-4 0→1, 8N-5a/5b 1/0→1/1). Findings y disposición:

| # | Finding | Estado |
|---|---------|--------|
| **F1** | `git diff-tree` sin `-m` **no emite diff para un merge**: el check se volvía ciego (medido: `show` 61 paths vs `diff-tree` 0) — peor que antes del cambio, y es la forma de commit que produce un carril malo. | ✅ Vuelto a `git show -z --diff-filter=D` (conserva merge **y** quoting) + fixture `8N-1d` (merge con 60 trackeados borrados) que muere con `diff-tree`. Bonus medido: con el índice corrupto `diff-tree` falla y `show` no. |
| **F2** | `8N-5b` assertaba `.length === 1` y pasaba **por la rama (d)** cuando `PROSE_HINT` perdía sus palabras; dos filas de la tabla de mutaciones no reproducían. | ✅ `8N-5b` asserta `/non-canonical header/`; tabla regenerada con las 11 mutaciones reales. |
| **F3** | El artefacto cambió durante el review (47→48 fixtures, `show`→`diff-tree`): un gate de segunda opinión necesita revisión pinneada. | ✅ Aceptado como defecto de proceso: la ronda 2 va pinneada por **sha256 `FC2114C8…8863`** y el fichero no se toca mientras corre. |
| **F4** | El lado **header** de `splitCells` no tenía fixture: revertir `governingHeader` dejaba todo verde. | ✅ `8N-3b` (header con pipe escapado); la mutación ahora la mata. |
| **F5** | (i) borrar la nota de "índice ilegible" no rompía nada; (ii) un ambiguo <50 es mudo. | ✅ (i) `8N-1e` (índice corrupto: violación + nota). (ii) declarado en §Deuda: el umbral de la nota es el mismo ≥50, y bajarlo convierte un `git rm --cached` de 10 en ruido. |
| **F6** | Incidente con el índice reseteado después: sale `0 + nota`. | ✅ Declarado en §Deuda (ventana del positivo verdadero); el reviewer no lo bloquea. |
| **F8** | `hasMeasure` era una lista de frases: `no fue medido` / `no está medido` / `sin ser medido` pasaban, `Medición: n/a` también, y `no medido · Medición: rg -> 92` daba **falso positivo**. | ✅ La negación admite hasta dos palabras intermedias, el valor del campo se juzga aparte (un `Medición:` real **gana** sobre la prosa negada) y un valor placeholder no mide. Fixtures `8N-2d/2e/2f`. |
| **F9** | `parsePlanRows` sigue partiendo por `\|` (tabla de waves), declarado en el avance pero no en el task file. | ✅ Declarado en §Deuda (sin arista medida). |
| N1 | El footer de `--rows` decía "without Medición:" incluso para `cannot judge`. | ✅ Nombra ambos casos (exit code y prefijo `FAIL` sin cambios). |
| N2 | `basename` importado sin uso (pre-existente). | ✅ Eliminado. |

**Ronda 2** (mismo reviewer, artefacto pinneado por `sha256 FC2114C8…8863`, verificado al inicio y al
final): **REQUEST CHANGES** acotado a `hasMeasure` — los cuatro bloqueantes de la ronda 1 (F1, F2, F4,
F5-i) **CLOSED** con sus propias mutaciones, más tres mutaciones supervivientes que el autor no había
pinneado (O2/O3/O4) y dos reabiertos de F8:

| # | Finding | Estado |
|---|---------|--------|
| **F8a** | `no está medido` seguía midiendo: la ventana era `(?:\w+\s+){0,2}` y `\w` no consume `está`. | ✅ `(?:\S+\s+){0,2}` + fixture `8N-2g` (mata la mutación `\S+ → \w+`). |
| **F8b** | Regresión mía: con el campo primero, `sin Medición: 92` **medía** (solo se negaba el valor). | ✅ Lookbehind `(?<!sin\s)` en `MEASURE_FIELD` + `8N-2h` (`sin Medición: 92` y `… ver nota`). |
| **O1** | Orden inverso seguía dando falso positivo (`Medición: … · no medido`). | ✅ El valor corta en separador (`[^|·;]*`) + `8N-2j`. |
| **O2** | Quitar `tbd\|pendiente\|sin datos\|[-–—]` de los placeholders dejaba todo verde. | ✅ `8N-2f` asserta `n/a` **y** `tbd`. |
| **O3** | El umbral `>= 50` no tenía fixture de borde (`>` dejaba todo verde). | ✅ `8N-1f` (exactamente 50 → violación) y `8N-1g` (49 → nota "in repair"), con `staleIndexRepo(n)` paramétrico. |
| **O4** | Quitar `!MEASURE_NEGATED.test(value)` dejaba todo verde: `Medición: no medido` no tenía fixture. | ✅ `8N-2i`. |
| nit | El comentario enumeraba `no está medido` como cubierto cuando no lo estaba. | ✅ Tras F8a el comentario es cierto (y cita la medición). |

**Veredicto final de la ronda 2: los findings reabiertos y los opcionales quedaron corregidos con
fixture propia; se pidió explícitamente que O2/O3/O4 no quedaran como deuda sin declarar.**

**Ronda 3** (pinneada por `sha256 3163DE89…6490`): **APPROVE** — contrato cumplido punto por punto,
`--self-test` 60/60, 12 mutaciones (7 del autor reproducidas exactas), y los reabiertos F8a/F8b/O1–O4
**CLOSED**. Dejó seis residuales no bloqueantes; los dos que eran **fallos abiertos de la negación** se
aplicaron antes del commit en vez de declararse:

| # | Residual | Estado |
|---|----------|--------|
| **R3-1** | `sin  Medición:` con **doble espacio** seguía midiendo (el lookbehind era `sin\s`, un espacio literal). | ✅ Lookbehind `\bsin\s+` + `8N-2k`; la mutación a `\bsin\s` la mata. |
| **R3-4** | La ventana `\S+` **se comía la coma**: `sin embargo, medido` daba violación (falso positivo nuevo, introducido por F8a). | ✅ Ventana `[^\s,;·()]+` + `8N-2l` (`sin embargo, medido en prod` → 0); `\S+` la mata. |
| **R3-6.4** | Caído el `/i`, `Medición: N/A` **contaba como medición** (la única superviviente en la dirección peligrosa). | ✅ `8N-2f` asserta `N/A` además de `n/a` y `tbd`; quitar el `/i` la mata. |
| R3-2/R3-3/R3-5 | Variantes de frase (`sin la Medición:`, `pendiente de medir`, `Medición: ; cmd`). | Deuda declarada: el techo es contrato de texto, y los placeholders se cubren **exactos**. |

`--self-test` **60 → 62**, gate 7/7, `--rows` y `--plan` exit 0, `sha256` final
`2720A024…FC8E`.

**Ronda 4** (acotada a R3-1/R3-4/R3-6.4, pinneada por `sha256 2720A024…FC8E`, verificado al inicio y al
final): **APPROVE** — `--self-test` **62/62**, gate 7/7, los diez repro de la ronda dan el valor
esperado (`sin  Medición: 92` → 1, `sin embargo, medido` → 0, `Medición: N/A` → 1, `MEDICIÓN: 92` → 0,
`Medición: 0` → 0, tab tras `sin` → 1) y el lookbehind nuevo **no** abre falsos positivos
(`sin embargo, Medición: …` → 0, `asín Medición:` → 0, `sin` dentro del valor → 0). Autoriza el commit
sobre ese pin; si el fichero cambia, el dictamen no vale.

**Delta declarado entre el pin aprobado y los bytes commiteados** (hay que poder auditarlo, no taparlo):
el primer intento de commit lo abortó **el propio `pre-commit` del repo** — su hook `prettier`
(`.githooks/pre-commit-config.yaml:73`, `files: '\.(mjs|…)$'`) reescribió **una** expresión de
`hasMeasure` (`return <expr>;` → `return (` + 3 líneas + `);`), +2/−1 líneas y **cero** cambio de
semántica. El commit se rehizo sobre los bytes ya formateados y **toda la batería se re-midió sobre
ellos**: `--self-test` 62/62, 7 checks, `--rows` y `--plan` exit 0, `prettier --check` limpio.
`sha256` commiteado: `058F18E7…BE55`. Es una desviación de proceso frente a F3 (el artefacto cambió
después del pin) resuelta por **re-medición y declaración**, no por confianza.

