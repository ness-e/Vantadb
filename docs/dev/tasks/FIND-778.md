---
title: "FIND-778: `git-commit-isolated.ps1` valida el commit resultante y limpia su índice privado en todos los caminos"
kind: task
status: completed
description: "Dos commits con árbol vacío (`4b825dc…`, 2365 paths) creados por el helper mientras imprimía `git commit failed (exit 1)`; el mecanismo no se reprodujo, así que el fix es defensivo y falla cerrado: guardas de resultado en los caminos de éxito y de error, restauración de la ref por compare-and-swap, limpieza del índice privado en `finally` y un test que fabrica el árbol vacío."
---

# FIND-778: el helper valida el commit, no el exit code

## Metadata
- **Fuente:** `docs/dev/Backlog.md` §`FIND-778` (fila retirada al cerrar)
- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🔴 Crítica
- **Tipo:** PowerShell (`dev-tools/`) + test
- **Creado:** 2026-11-26
- **Estado:** ✅ COMPLETED (review P2-01 cerrada, ver §Review)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps

## El incidente, medido

```
72643d9df  tree = 4b825dc…  (árbol vacío canónico)   2365 paths borrados, todos en disco
290df23d0  tree = 4b825dc…  idéntico
```

El helper imprimió `git commit failed (exit 1)` y **no ejecutó la verificación por contenido**: en el código, `throw` en el `catch` saltaba la sección de verificación. El worktree quedó intacto y la historia se reparó con `commit-tree` + `update-ref` (sin `--amend` ni `reset`). **El mecanismo no se reprodujo**: el índice privado listaba el fichero (`git diff --cached`) y el commit salió vacío igual.

Eso deja el diagnóstico en una clase de fallo, no en una causa: `git commit` puede crear un commit —incluso uno catastrófico— y devolver un código distinto de cero. El defecto de diseño no era el commit malo, era **tratar el exit code como si fuera el resultado**.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | Todos los carriles: el helper es el camino de commit sancionado (`AGENTS.md` §Política de git, `docs/dev/workflow/RULES.md` §11). No hay ningún import: se invoca como proceso. |
| Callees (medidos, no modificados) | `git read-tree` · `git add` · `git write-tree` · `git commit` · `git rev-parse` · `git ls-tree` · `git diff-tree` · `git update-ref` · `git reset` · `git symbolic-ref` |
| Implicaciones | Ninguna sobre producto: no se toca `src/**` ni `.githooks/**`. El script gana guardas y dos códigos de salida nuevos (4 y 6). Un commit legítimo —incluido un `git rm`— sigue funcionando; lo demuestran los escenarios 4 y 5 del test. |

## Contrato

Un commit cuyo árbol sea el vacío canónico, que borre ≥50 paths **que siguen en el worktree**, que pierda más de 200 entradas respecto de `HEAD`, o que no contenga **byte a byte** un path que el script mismo puso en el índice privado, **no puede quedar en `HEAD`**, y el script **no puede salir 0**. En los cuatro casos: ref restaurada al SHA previo, SHA del commit malo impreso y conservado, salida ≠ 0. El camino de fallo valida igual que el de éxito. El índice privado se borra siempre.

**Identidad por contenido, no por mensaje.** Si el commit en `HEAD` no es demostrablemente de este carril (mi `git commit` falló y otro carril commiteó en la misma ventana), no se restaura —revertirlo destruiría trabajo ajeno—; se reporta y se sale 6. La prueba de identidad es el árbol que sembré o los blobs de los paths nombrados, **nunca el asunto**: dos carriles usando el mismo mensaje convencional colisionan. **Excepción única:** el árbol vacío se restaura siempre, aunque el commit no sea mío, porque borra el repositorio entero y no hay trabajo de nadie que preservar.

## Lo que cambió (cuatro mejoras, en orden de importancia)

### 1. Validar el resultado antes de reportar — en los dos caminos (`:202-306`)

La validación vive **fuera** del `try/catch`, así que corre después de éxito y de error con un solo cuerpo de código. Compara el `HEAD` actual contra el `HEAD` previo capturado antes del commit (`:155-157`).

| Guarda | Línea | Umbral | Mensaje |
|---|---|---|---|
| árbol vacío canónico | `:229-231` | `$EMPTY_TREE` = `4b825dc642cb6eb9a060e54bf8d69288fbee4904` (`:117`) | `CATASTROPHIC: … canonical EMPTY tree` |
| borra paths que siguen en disco | `:234-246` | `$MAX_DELETED_BUT_PRESENT = 50` (`:127`) | `CATASTROPHIC: … deletes N path(s) that STILL EXIST in the worktree` |
| menos entradas que `HEAD - N` | `:248-255` | `$MAX_ENTRY_DROP = 200` (`:123`) | `SUSPICIOUS: … N tree entries, HEAD had M` |
| lo que sembré no está **idéntico** en el commit | `:257-272` | 1 path | `CATASTROPHIC: … absent or have DIFFERENT content` |

**Por qué N = 200.** Medido sobre los últimos 300 commits de este repo: el mayor descenso **legítimo** de entradas es **43** (`960e338d0`: 2375 → 2332, poda de docs), y los dos commits vacíos bajaron **2365**. 200 deja 4,6× de margen sobre el mayor borrado legítimo y queda un orden de magnitud por debajo del daño. **Techo conocido:** un borrado deliberado de más de 200 ficheros trackeados también se rechaza (y si el commit vacía el repositorio, aborta antes en el guardián del índice, `:182`); para ese caso, `git commit` normal.

El umbral de 50 paths borrados usa el criterio del contrato: **solo cuenta los que siguen existiendo en el worktree**, porque un `git rm` real los quita del disco y eso es coherente; borrar 50 ficheros que están ahí no lo es.

La cuarta guarda no estaba en el pedido y es la más ajustada al incidente: el script conoce el árbol que sembró (`git write-tree` sobre el índice privado, `:186`, tomado **antes** del commit) y exige que cada path nombrado siga presente **con el mismo blob**. La presencia sola no prueba nada —un blob viejo del mismo path también existe—; por eso la comparación es de contenido (review C1).

**Restauración (`:289-306`).** Si alguna guarda dispara y `HEAD` se movió:

```powershell
git update-ref HEAD $headBefore $headAfter
```

Es un compare-and-swap: si otro carril movió `HEAD` en la ventana, el update se **niega** (no lo pisa) y el script sale **5** pidiendo la reparación a mano. Si el update funciona, `HEAD` vuelve al SHA previo y el script sale **4**. El commit malo **no se borra ni se enmienda**: se imprime su SHA completo y se deja el `git show --stat` como pista. Nunca se reescribe historia.

**De quién es el commit en `HEAD` (`:274`, `:282-287`, `:321-329`).** Caso límite que el propio diseño abre: mi `git commit` falla y **otro carril commitea en la misma ventana**. Sin discriminador, la restauración revertiría **su** commit. La identidad se prueba por contenido (árbol sembrado, o todos los paths nombrados con el mismo blob); si no coincide, el commit es ajeno: no se toca, se reporta con su SHA y se sale **6**. Esa comprobación **también gobierna el camino de éxito**: con `git commit` ≠ 0 y un `HEAD` que no es demostrablemente mío, el script **nunca imprime `OK`** (review C1 — antes imprimía `OK` y salía 0). Excepción: el árbol vacío se restaura siempre (review H1).

### 2. El camino de fallo también verifica (`:189-190`, `:202`)

`git commit` ya no lanza: se guarda `$commitExit` y se sigue. Después de la validación, si `HEAD` no se movió, se reporta el error original (exit 2). Si `HEAD` **sí** se movió y el commit es demostrablemente nuestro, el commit vale y el script lo dice en voz alta (`NOTE: git commit reported exit N … provably ours`, `:334`) en lugar de fingir que no pasó nada. Y si `git commit` sale 0 sin mover `HEAD`, eso también es una violación (`:213-215`).

Si el commit no se creó y `HEAD` se movió a un commit de otro carril, el script sale 6 con `NOTHING OF YOURS WAS COMMITTED` (`:321-329`): el código antiguo salía 2 en ese caso y era correcto.

### 3. Índice privado: limpieza en todos los caminos y negativa a commitear uno que no sembré

- Se **aborta** si antes del commit el índice privado no existe, está vacío o lista 0 ficheros (`:180-182`). Un `git commit` con un índice inexistente o vacío crea exactamente el commit vacío del incidente: no se puede permitir.
- La limpieza está en el `finally` (`:193-200`), que cubre el camino de error.
- `Remove-PrivateIndex` (`:139-147`) reintenta 10×150 ms: en Windows un handle abierto justo después de que git termina hace fallar un `Remove-Item` único **en silencio** — es la explicación más probable de los 39 `idx-*` de ~250 KB que quedaron en `%TEMP%\opencode`. Si aun así no puede borrarlo, lo dice y da el comando.
- El nombre del índice incluye el PID (`:153`), así que dos carriles vivos no pueden compartir índice y un residuo es atribuible.

### 4. Test que fabrica el árbol vacío (`dev-tools/tests/git-commit-isolated.tests.ps1`)

Siete escenarios, cada uno en un repo git desechable bajo `%TEMP%`, con `TEMP` redirigido para que el índice privado caiga dentro del propio test (el checkout real no se toca):

| # | Escenario | Fabricación | Espera |
|---|---|---|---|
| 1 | éxito con árbol vacío | hook `pre-commit` hace `git read-tree --empty` y `exit 0` | exit **4**, `REFUSED`, `HEAD` restaurado, worktree de 60 ficheros intacto, 0 residuos |
| 2 | **fallo con `HEAD` movido** | hook vacía el índice, crea el commit con `commit-tree`, mueve `HEAD` y `exit 1` | exit **4**, `HEAD` restaurado, SHA malo nombrado y **conservado** |
| 3 | hook aborta antes del commit | hook `exit 1` | exit **2**, `HEAD` quieto, 0 residuos |
| 4 | camino feliz | sin hook, `-ExpectPresent` | exit **0**, commit nuevo, contenido verificado |
| 5 | borrado legítimo | `git rm` de un fichero | exit **0** — la guarda no bloquea un borrado real |
| 6 | commit **válido** de otro carril | hook commitea otro fichero en su propio índice y `exit 1` | exit **6**, commit ajeno intacto y **sin `OK`** (review C1) |
| 7 | árbol vacío de otro carril | hook crea el commit vacío con otro mensaje | exit **4**, restaurado **igualmente** (review H1) |

El escenario 2 es el que importa: reproduce la forma medida (commit creado **y** exit ≠ 0) sin depender de que el mecanismo original se reproduzca.

## Decisión colateral (declarada)

El refresco del índice compartido pasó de `git add -- @Paths` a `git reset -q -- @Paths` (`:347`). No es cosmético: al commitear un **borrado**, `git add` no puede expresarlo (el path no está ni en el worktree ni en el índice → `exit 128`) y el helper salía **3** tras un `git rm` legítimo, con el índice compartido desincronizado. `git reset -- <paths>` copia las entradas de `HEAD` (nuestro commit) para esos paths y es correcto para modificación, alta y baja. Lo encontró el escenario 5 del test, no una suposición. La review lo verificó además para altas y en pathspec acotado (no toca lo stageado por otros carriles).

## Verificación

```
pwsh -NoProfile -File dev-tools/tests/git-commit-isolated.tests.ps1
  [1] .. [7]  → 37/37 PASS, EXIT=0
```

Salida del test del árbol vacío (escenario 1, literal):

```
[1] success path, empty tree (hook empties the private index)
    | REFUSED (FIND-778): the commit on HEAD is not the commit you asked for.
    |   - CATASTROPHIC: commit 51d74b5a… has the canonical EMPTY tree (4b825dc642cb6eb9a060e54bf8d69288fbee4904) - it deletes every tracked path
    |   - CATASTROPHIC: commit 51d74b5a… deletes 60 path(s) that STILL EXIST in the worktree (first: docs/file0.txt)
    |   - CATASTROPHIC: 1 staged path(s) are absent or have DIFFERENT content in commit 51d74b5a… (first: docs/file0.txt)
    |   restoring master to 68ad9d5b… (compare-and-swap, never clobbers a concurrent update)
    |   restored: HEAD is 68ad9d5 (tree c6cd7e9)
    |   the refused commit object is KEPT for the record (never amended, never deleted): 51d74b5a…
  PASS  exits non-zero (got 4)
  PASS  exit code is 4 = refused by result guards
  PASS  HEAD tree is NOT the empty tree
  PASS  HEAD restored to the previous SHA
  PASS  worktree intact (60 files)
  PASS  no private index left behind
```

Escenario 2 (commit creado, `git commit` exit ≠ 0 — el camino que hoy no se verificaba):

```
[2] failure path (commit exists, git commit exit != 0)
    | REFUSED (FIND-778): the commit on HEAD is not the commit you asked for.
    |   - CATASTROPHIC: commit 0b4fc515… has the canonical EMPTY tree (4b825dc…)
    |   restoring master to 1813509… (compare-and-swap…)
  PASS  exit code is 4 (got 4)
  PASS  hook did create a commit (0b4fc515…)
  PASS  HEAD restored to the previous SHA
  PASS  refused commit object kept for the record
```

Escenarios 6 y 7 (identidad y excepción del árbol vacío):

```
[6] another lane's valid commit: refused, not reverted, no OK
  PASS  exit code is 6 = nothing of yours, peer commit kept (got 6)
  PASS  the other lane's commit was NOT reverted
  PASS  never prints OK
[7] another lane's EMPTY-tree commit: restored anyway
  PASS  exit code is 4 = refused and restored (got 4)
  PASS  HEAD restored despite the commit not being ours
  PASS  output explains the empty-tree exception
```

Repros adversariales del revisor (`%TEMP%\opencode\f778-*.ps1`) re-ejecutados sobre el script corregido: (C) commit válido de otro carril → **exit 6**, `OUR_CHANGE_IN_HEAD=False`, sin `OK`; (A) mismo asunto + árbol vacío → **exit 4** restaurado por la excepción, no por el asunto; (E) `Get-TreeEntryCount` del árbol vacío → **0**; HEAD **detached** → restaurado, sigue detached.

Repositorio real, tras el commit de cierre (el helper se usó a sí mismo):

```
git rev-parse "HEAD^{tree}"                              → ≠ 4b825dc…
git ls-tree -r --name-only HEAD | Measure-Object -Line   → 2368
```

**Sobre el 2365 pedido.** La línea base al abrir la tarea era 2365 (`bf7b8db42`). Al cerrar, otro carril había commiteado `docs/dev/tasks/FIND-757.md` (+1 → 2366, `ee5b78251`) y este commit añade `docs/dev/tasks/FIND-778.md` y `dev-tools/tests/git-commit-isolated.tests.ps1` (+2). 2365 + 1 + 2 = **2368**, sin un solo borrado: el commit de este carril se verifica con `git diff --name-status HEAD~1..HEAD` (solo `A`/`M`). Un cero aquí sería el incidente.

Gates de documentación: `check-docs` **0** (el primer intento dio **1** con razón: `description` de 434 caracteres contra el máximo de 400; corregido) · `check-links` **0** · `gen-index --check` **1, rojo esperado y no regenerado aquí**: `docs/index.md` es un fichero compartido y está fuera del alcance de esta tarea. Ya estaba rojo antes: `FIND-764.md` y `FIND-768.md` existían sin fila en el índice (0 filas cada uno), y este cierre añade la de `FIND-778`. Lo regenera el orquestador con `node scripts/docs/gen-index.mjs --write`.

## Review P2-01

`vanta-review`, una ronda. Veredicto **CHANGES-REQUIRED** (0 triviales, 1 crítico, 1 alto, 2 medios, 2 bajos), sobre los ficheros congelados por SHA-256. La premisa se re-verificó en vivo (los dos árboles vacíos, el `throw` que saltaba la verificación, y el umbral 43/2365 re-medido de forma independiente) y pasaron como correctos: validación fuera del `try/catch`, `finally` con reintento (0 residuos en 7 escenarios y 4 repros), guardianes del índice, CAS + exit 5, HEAD detached, commit malo conservado, `git reset` para alta/modificación/borrado, y aislamiento real del test.

| # | Hallazgo | Desenlace |
|---|---|---|
| **C1** | Falso éxito: si mi `git commit` falla y `HEAD` avanza a un commit **válido** ajeno, `$isOurs` no se consultaba en el camino de éxito → imprimía `OK` y salía 0 (el helper antiguo salía 2, correcto): regresión del fix. La guarda 4 era de **presencia**, no de contenido, y por eso no lo detectaba. | Corregido: guarda 4 compara **blobs** (`:257-272`); `$isOurs` gobierna también el camino de éxito (`:321-329`); escenario 6 del test reproduce el caso y exige exit 6 sin `OK`. Repro del revisor: `EXIT=6 OUR_CHANGE_IN_HEAD=False`. |
| **H1** | El escenario 6 dejaba el **árbol vacío** en `HEAD`, contra el contrato, y el doc afirmaba que eso ya no podía pasar. | Corregido: el árbol vacío se restaura **siempre**, aunque el commit no sea nuestro (`:282-287`); escenario 7 lo fija. El exit 6 queda solo para commits ajenos **no** catastróficos. |
| **M1** | El OR por asunto podía revertir el commit de otro carril con el mismo mensaje convencional. | Corregido: se eliminó la comparación por asunto; identidad solo por árbol/blobs (`:274`). |
| **M2** | El doc afirmaba que “un `git rm` masivo sigue commiteando”, falso por encima de 200 entradas. | Corregido: techo declarado en `.DESCRIPTION` y en esta ficha; el comportamiento se mantiene porque el contrato lo exige. |
| **L1** | Fallos de medición fallaban **abierto** (`ls-tree`/`diff-tree` silenciados). | Corregido: ambas rutas añaden una razón y se rechaza el commit (`:234-255`). |
| **L2** | Cifras y citas de línea desactualizadas (34 vs 37 aserciones, 5 vs 7 escenarios, un solo código nuevo). | Corregido en esta versión; el conteo real lo imprime el propio test. |

Queda como techo declarado, no como deuda silenciosa: el mecanismo original no se reproduce (las guardas fallan cerrado sobre el resultado, no sobre la causa) y un borrado legítimo de más de 200 ficheros requiere `git commit` normal.

## Deuda / techo declarado

- **El mecanismo original sigue sin reproducirse.** Las guardas fallan cerrado sobre el resultado, no sobre la causa. Si el índice privado desaparece entre `write-tree` y `commit`, el commit vacío **se sigue creando**; lo que ya no puede pasar es que quede en `HEAD` o que se reporte como éxito.
- **Carrera residual:** entre el `rev-parse` posterior al commit y la restauración, otro carril puede mover `HEAD`. El compare-and-swap lo detecta (sale 5 pidiendo reparación manual) en lugar de pisarlo.
- **Los 39 `idx-*` históricos en `%TEMP%\opencode`** no los borra este fix retroactivamente: el script solo limpia el suyo. Se listan al cierre para que el owner decida.
