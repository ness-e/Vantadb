---
title: "FIND-780: el commit se crea con plumbing (`commit-tree` + `update-ref`), no con `git commit`"
kind: task
status: completed
description: "Los tres commits de árbol vacío los creó `git commit` leyendo un índice privado que el `finally` del helper acababa de borrar cuando el pipe cortó el proceso. Se cambia la creación del commit a `write-tree` + `commit-tree` + `update-ref` (CAS) y el gate se corre explícitamente con `git hook run pre-commit`, de modo que matar el proceso deja HEAD intacto y no crea el árbol vacío."
---

# FIND-780: el commit se crea con plumbing

## Metadata
- **Fuente:** `docs/dev/Backlog.md` §`FIND-780` (fila retirada al cerrar)
- **Esfuerzo:** 🟠 2h
- **Prioridad:** 🔴 Alta
- **Tipo:** PowerShell (`dev-tools/`) + test + regla
- **Creado:** 2026-11-26
- **Estado:** ✅ COMPLETED (review P2-01: 2 rondas, APPROVE en la 2ª — ver §Review)
- **Causa de:** el Incidente ya era mío: el pipe lo puse yo. El fix estructural lo recomendó el review de cierre de `FIND-778`; esta tarea lo ejecuta.

## La causa raíz, reproducida (no razonada)

`FIND-778` blindó el helper para validar el resultado: cuatro guardas que corren **después** del commit y **dentro** del proceso. Funcionaron en su primer caso real —reportaron `git commit failed; HEAD did not move` y no crearon commit—. Pero **el kill ocurre antes**:

```
& git commit                 (:189)  ← el commit se crea aquí
  → el pipe se corta (`... | Select-Object -First N`)
     → PowerShell desenrolla el script: corre el `finally` (:196)
        → borra el índice privado mientras el HIJO `git commit` sigue vivo
           → git lee un índice que ya no está = VACÍO
              → commit con el árbol vacío
                 → las guardas nunca corren: el proceso que las corre ya no existe
```

Tres incidentes medidos: `72643d9df`, `290df23d0`, `5766654ee`.

**Lo que el diagnóstico de la fila no decía y el test midió:** en Windows, matar el padre **no mata al hijo**. El `git commit` sobrevive, termina su trabajo y commitea. El kill no tiene que ser ordenado: basta con matar el proceso que commitea.

### El test lo reproduce

Escenario 10 del test: el hook vacía el índice (`git read-tree --empty`, el payload del incidente), escribe un centinela y duerme; el test mata el helper con `Stop-Process -Id $proc.Id -Force`. Contra el script **anterior** (vía `$env:FIND780_HELPER`):

```
[10] kill the committing process mid-flight: HEAD must not move
  PASS  the commit reached the hook (kill lands mid-flight)
    | [master 711a687] killtest
    |  60 files changed, 60 deletions(-)
    |  delete mode 100644 docs/file0.txt
    … (60 paths)
  FAIL  HEAD did not move (kill left the previous SHA: 68a86eb2)
  FAIL  HEAD tree is NOT the empty tree (4b825dc...)
  FAIL  HEAD tree is unchanged
  FAIL  no commit was created at all
  FAIL  HEAD still lists 60 paths
FAILED: 8 assertion(s).
```

Seis líneas de `git commit` escritas por un proceso que ya no existía. Es el incidente, entero. (Corrida con los 10 escenarios de la primera versión; con los 15 actuales son **13 FAILED** — ver §Verificación.)

Contra el script **corregido**:

```
[10] kill the committing process mid-flight: HEAD must not move
  PASS  the commit reached the hook (kill lands mid-flight)
  PASS  HEAD did not move (kill left the previous SHA: 0b903923)
  PASS  HEAD tree is NOT the empty tree (4b825dc...)
  PASS  HEAD tree is unchanged
  PASS  no commit was created at all
  PASS  HEAD still lists 60 paths
  PASS  worktree intact (60 files)
```

## La opción elegida y su trade-off

**Opción (c): correr el hook REAL del repo, y sólo después el plumbing.**

| | Gate que queda | Kill deja HEAD |
|---|---|---|
| **(a)** `pre-commit run --config … --files <rutas>` | **sólo los gates del YAML.** Se pierden: `DOC-04` (junk/binarios), el guard de `unmerged`, el guard de config sin stagear, **`verify_staged.ps1`** y **actionlint** — porque están en el shim `.githooks/pre-commit`, no en el YAML | intacto |
| **(b)** `git commit` con el índice sembrado y verificado + `commit-tree` como recuperación | completo — pero el camino normal **sigue siendo el que crea el árbol vacío**: la verificación es otra guarda dentro del proceso que muere | se mueve (el bug) |
| **(c) elegida** `git hook run pre-commit` + `commit-tree` + `update-ref` | **completo, y por construcción**: es el mismo binario/script que ejecutaría git, con `core.hooksPath` resuelto y `GIT_INDEX_FILE` ya apuntando al índice privado (su `git diff --staged` ve exactamente los paths del commit — escenario 8 lo prueba) | intacto |

**Por qué (c) y no (a):** la lista de gates del shim son **cinco gates más** que el YAML. Reimplementarlos en el helper sería una copia que se desincroniza el día que el shim gane uno — exactamente el fallo que la pre-mortem (a) describe. `git hook run` no reimplementa nada: invoca el mismo entry point.

**Trade-off, declarado:**

- Los hooks que `git commit` corría **implícitamente** — `prepare-commit-msg`, `commit-msg`, `post-commit` — ya no corren. Este repo define sólo `pre-commit` y `pre-push` (verificado: `Get-ChildItem .githooks`), así que hoy no se pierde ningún gate; y el helper **avisa** si aparece cualquiera de los tres (P2-01 M2: antes el aviso sólo miraba `pre-commit` y por tanto no podía detectar el día que se añadiera otro). Además `commit-tree` no escribe `.git/COMMIT_EDITMSG`, no honra `commit.gpgsign`, no limpia el `-m` como git y deja el reflog con el asunto crudo en vez del prefijo `commit: ` — ninguna de esas cosas la usa este repo, y está declarado en el `.DESCRIPTION`.
- **La ventana del CAS es ahora tan larga como los hooks** (clippy tarda minutos). Si otro carril commitea en ese intervalo, `update-ref` falla. **Esto no es una garantía nueva:** `git commit` en git 2.51 ya se niega en ese caso (`fatal: cannot lock ref 'HEAD': is at … but expected …`, medido por el revisor con el helper anterior, que también salía 6). Lo que cambia es que ahora es **explícito**, no depende de un detalle interno de git, y el mensaje distingue las dos causas (otro carril movió `HEAD` vs. el ref no se pudo bloquear). Escenario 9 lo fija.
- El objeto creado por `commit-tree` que pierde el CAS queda **colgando** (sin ref). Es basura inofensiva para `gc`, no un commit que alguien pueda leer por accidente: se imprime su SHA.

## Contrato

1. **Matar el proceso que commitea no mueve `HEAD`**, ni crea árbol vacío: `rev-parse HEAD` = SHA anterior, `HEAD^{tree}` ≠ `4b825dc…` y el árbol de `HEAD` listando el mismo número de paths.
2. **El gate no se pierde**: el hook corre antes de que exista commit, con el índice privado como índice, y si falla **no hay commit** (`HEAD` quieto, exit 2).
3. **El árbol vacío no se detecta y deshace: no se crea.** Un índice vaciado por un hook se rechaza *antes* de `commit-tree` (exit 4). Las guardas de `FIND-778` siguen ahí como red para el caso en que **el hook** mueva `HEAD` por su cuenta — y esa red **debe correr también cuando el hook pasa el gate**: por eso los rechazos previos **no** hacen `exit` dentro del `try` (P2-01 HIGH-1: `exit` termina el script, así que la política de restauración no corría y un commit de árbol vacío creado por un hook que salía 0 se quedaba en `HEAD`). Escenarios 12 y 13.
4. El test **falla si alguien vuelve a `& git commit`**: escenario 10 (dinámico, con `$env:FIND780_HELPER`) y escenario 11 (estático: el script no contiene ninguna invocación `& git commit`), que cubre el caso híbrido que el kill no ve.

## Lo que cambió en `dev-tools/git-commit-isolated.ps1`

### 1. El gate, explícito y primero (`git hook run pre-commit`)

```powershell
$hookDir = "$(& git rev-parse --git-path hooks)".Trim()
$hookPresent = [bool]($hookDir -and (Test-Path -LiteralPath (Join-Path $hookDir "pre-commit")))
if (-not $hookPresent) { Write-Host "WARNING: no pre-commit hook …" }
& git hook run --ignore-missing pre-commit
$hookExit = $LASTEXITCODE
if ($hookExit -ne 0) { throw "pre-commit hook FAILED (exit $hookExit): commit blocked, nothing was created" }
```

`--ignore-missing` replica la semántica de `git commit` (sin hook, sin gate) en vez de bloquear un repo sin hooks; el `WARNING` es lo que evita que eso sea silencioso. Un segundo aviso nombra `prepare-commit-msg`/`commit-msg`/`post-commit` **si existen** en el directorio de hooks resuelto: es el único tripwire honesto para el día que alguien añada uno (P2-01 M2 — mirar sólo `pre-commit` no podía detectarlo).

### 2. El árbol se escribe DESPUÉS de los hooks

`git commit` también escribe el árbol después de los hooks, así que un hook que reescriba el índice se respeta. Si el árbol cambió respecto del sembrado, se imprime `NOTE: the pre-commit hook changed the private index (…)` — la firma del incidente, ahora visible en vez de invisible.

### 3. Rechazo **antes** de crear el commit, reportado **por** el bloque de validación

```powershell
if ($treePlanned -eq $EMPTY_TREE) {
    $preCommitRefusal = "CATASTROPHIC: … EMPTY tree ($EMPTY_TREE) …"
    throw "REFUSED (FIND-780): the private index is EMPTY after the hooks; no commit was created"
}
$plannedEntries = Get-TreeEntryCount $treePlanned
if ($plannedEntries -lt 0 -or $plannedEntries -lt ($entriesBefore - $MAX_ENTRY_DROP)) { … throw … }
```

Mismo umbral que `FIND-778` (`$MAX_ENTRY_DROP = 200`), pero aplicado al árbol **antes** de que exista un objeto commit. Fallar cerrado incluye la medición imposible (`-1`).

**Y el rechazo NO hace `exit` dentro del `try`** — es el bug que encontró el review (P2-01 HIGH-1): `exit` termina el script, así que el bloque de validación `FIND-778` no corría, y un hook que **pasa el gate** y mueve `HEAD` a un commit de árbol vacío lo dejaba ahí mientras el mensaje decía "HEAD did not move". Ahora el rechazo se guarda en `$preCommitRefusal` y se reporta donde vive la política: `-not $moved` → bloque `REFUSED` + `exit 4`; `$moved` → las guardas de `FIND-778` deciden (restaurar si es árbol vacío, conservar si es de otro carril). Escenarios 12 y 13 fijan las dos variantes.

### 4. `commit-tree` + CAS

```powershell
$newCommit = "$(& git commit-tree $treePlanned -p $headBefore -m $Message)" -replace '\s', ''
& git update-ref -m $Message HEAD $newCommit $headBefore
```

`commit-tree` recibe el árbol como argumento: desde ahí **ningún paso lee el índice**, así que el `finally` que lo borra (que se queda: nada de residuo, pre-mortem b) no puede afectar al commit. Si `update-ref` falla, se vuelve a leer `HEAD`: si cambió, el mensaje dice que otro carril lo movió; si no, dice que el ref no se pudo bloquear (un lock ajeno, I/O) — antes ambas causas se reportaban como "otro carril" (nit del review).

### 5. Códigos de salida

`2` = nada se creó (incluye hook que falla) · `4` = rechazado por las guardas o por el rechazo previo · `6` = `HEAD` es de otro carril **o** se movió durante los hooks (no se pisa). `5` (la restauración falló) sigue.

## Test: 15 escenarios, 82 aserciones

| # | Escenario | Espera |
|---|---|---|
| 1 | árbol vacío fabricado por hook | exit **4**, `REFUSED`, `HEAD` quieto |
| 2 | commit creado por el hook + exit 1 | exit **4**, restaurado, SHA conservado |
| 3 | hook aborta | exit **2**, mensaje `pre-commit hook FAILED` |
| 4 | camino feliz | exit **0**, contenido verificado |
| 5 | `git rm` legítimo | exit **0** |
| 6 | commit válido de otro carril | exit **6**, sin revertir, sin `OK` |
| 7 | árbol vacío de otro carril | exit **4**, restaurado igualmente |
| **8** | **el gate corre y ve el índice privado** | `.hookran` (efecto del propio hook), `git diff --staged` = sólo `docs/file0.txt`, exit **0** |
| **9** | **`HEAD` se mueve durante los hooks** | exit **6**, commit ajeno intacto, mi edición sigue en el worktree |
| **10** | **kill del proceso que commitea** | `HEAD` quieto, árbol ≠ vacío, 0 commits creados |
| **11** | **estático: el helper no invoca `git commit`** | `commit-tree` + `update-ref` presentes, ninguna línea `& git commit` |
| **12** | **hook commitea el árbol vacío, índice intacto, exit 0** | exit **4**, `HEAD` restaurado |
| **13** | **hook vacía el índice Y commitea el árbol vacío, exit 0** | exit **4**, `HEAD` restaurado, `REFUSED` |
| **14** | **hooks de commit que no corren se nombran** | `WARNING: … does NOT run: commit-msg`, commit igual creado |
| **15** | **lock del ref tomado** | exit **2**, `HEAD` quieto, mensaje dice "could not lock HEAD" |

Los ocho últimos son de `FIND-780`. El 8 es la prueba de que el fix **no** cambió un bug de datos por una pérdida silenciosa de gates; el 9 fija el CAS; el 10 es la propiedad; el 11 cierra el caso híbrido que el kill no ve; **12 y 13 son el HIGH-1 del review** — un hook que **pasa** el gate y mueve `HEAD`, que es justo el camino que un `exit` dentro del `try` saltaba; 14 y 15 cubren los dos caminos que la ronda 2 sólo había verificado a mano (aviso de hooks saltados y fallo de `update-ref` sin `HEAD` movido).

## Verificación

```
pwsh -NoProfile -File dev-tools/tests/git-commit-isolated.tests.ps1
  [1] … [15]  → 82/82 PASS, EXIT=0

$env:FIND780_HELPER=<script anterior be58bdd2e>; pwsh -File <el mismo test>
  [10] → 5 FAIL (HEAD se movió al árbol vacío, 60/60 borrados)
  [3] [8] [9] [15] → 1 FAIL cada uno · [11] [14] → 2 FAIL cada uno
  total → 13 FAILED
```

**Contraste que importa:** los escenarios 12 y 13 **pasan** contra el helper anterior (`be58bdd2e`) — su `git commit` sí restauraba el árbol vacío creado por un hook. Es decir: el HIGH-1 del review fue una **regresión introducida por la primera versión de este fix**, no una carencia previa. El test la cazó porque se escribió después de la refactorización, no antes.

Commit normal con el helper **y los hooks a la vista** (repositorio real, este cierre) — pendiente de la salida literal del `pre-commit`.

## Regla `RULES.md` §11

La regla del `Must not` que prohibía pipear el helper **no se borró**: pasó a **"(historia, FIND-780)"**, con el mecanismo y los tres SHAs, porque sigue siendo la historia de por qué el helper tiene la forma que tiene y la costumbre (capturar en variable) sigue siendo correcta. Lo que cambió es **qué** protege:

| Antes | Ahora |
|---|---|
| "no pipees el helper" era **la** defensa | la defensa es **estructural**: no hay `git commit` que leer un índice borrado |
| `Must not` de prosa (la única barrera) | `Must` de forma (`write-tree` + `commit-tree` + `update-ref`, gate explícito) |
| — | `Must not` nuevo: volver a `git commit` o no correr el gate |

El bloque de ejemplo manual de §11 (`git commit -m …` sobre índice privado) llevaba a repetir la ventana; se le puso una nota que apunta al plumbing y al helper.

## Review P2-01

`vanta-review`, **dos rondas** (la segunda por bloqueante en la primera). Ficheros congelados por SHA; todas las premisas re-ejecutadas, no razonadas.

**Ronda 1 sobre `a31f8eb8a`** → 🔴 **REQUEST CHANGES** (0 críticos, 1 alto, 3 medios, 3 bajos, nits). Premisa confirmada: contra `be58bdd2e` la suite da fallos y el escenario 10 deja `HEAD` en `4b825dc…`; contra la versión nueva, verde.

| # | Hallazgo | Desenlace |
|---|---|---|
| **HIGH-1** | Los dos rechazos nuevos hacían `exit` **dentro** del `try`; `exit` termina el script, así que el bloque de validación de `FIND-778` **no corría**. Un hook que **pasa** el gate y mueve `HEAD` a un commit de árbol vacío quedaba en `HEAD` mientras el mensaje decía "HEAD did not move" (variantes A y B del revisor; reproducido por mí: `exit 4`, `EMPTY ON HEAD = True`). | **Corregido.** El rechazo se guarda en `$preCommitRefusal` + `throw`; el bloque decide: `-not $moved` → `REFUSED` + exit 4; `$moved` → política existente (restaura vacío, conserva ajeno). Escenarios 12/13. `HEAD` restaurado, `HEAD^{tree}` ≠ vacío. |
| **MEDIUM-1** | La afirmación "antes revertía con exit 0" era **falsa**: git 2.51 ya hace su propio CAS dentro de `git commit` y el helper anterior también salía 6. | **Corregido** en esta ficha, `operaciones.md` y el `.DESCRIPTION`: "misma garantía, ahora explícita y con la causa nombrada". Escenario 9 se queda como fijación, no como prueba de una diferencia. |
| **MEDIUM-2** | "El aviso es el tripwire" era falso: sólo miraba `pre-commit`, así que no podía detectar el día que se añadiera otro hook de commit. | **Corregido**: el helper busca `prepare-commit-msg`/`commit-msg`/`post-commit` y los **nombra** si existen; documentado; escenario 14. |
| **MEDIUM-3** | `status: completed` con `## Review P2-01` vacío. | **Corregido**: `in-review` durante la ronda 2; `completed` con esta tabla. |
| **LOW-1** | El escenario 10 no ve una regresión **híbrida** (`git hook run` puesto *y* `git commit` de vuelta): el kill cae antes de que exista el hijo. | **Corregido**: escenario 11 estático. El revisor lo hizo morder: inyectó `& git commit -m "x"` → 74 PASS / **1 FAIL**, justo la aserción esperada. |
| **LOW-2** | `git commit` corre los hooks **antes** de decidir "nothing to commit"; el helper decide antes. | **Declarado** como techo: un hook que stagee ficheros no puede completar un commit que el helper se niega a empezar. El shim actual nunca escribe el índice. |
| **LOW-3** | `$env:FIND780_HELPER` se usaba sin resolver: un valor relativo se resolvía contra el repo desechable. | **Corregido** con `Resolve-Path`. |
| nit | Cualquier fallo de `update-ref` se reportaba como "otro carril commiteó". | **Corregido**: se relee `HEAD` y el mensaje distingue "another lane moved HEAD" de "could not lock HEAD". Escenario 15 lo fija (lock real en `refs/heads/<rama>.lock`). |
| nit | `commit-tree` ≠ `git commit`: sin `COMMIT_EDITMSG`, sin `commit.gpgsign`, sin limpieza del `-m`, reflog sin el prefijo `commit: `. | **Declarado** en el `.DESCRIPTION` y en §Deuda. |

**Ronda 2 sobre el árbol de trabajo** → ✅ **APPROVE** (sin bloqueante). Los 10 hallazgos de la ronda 1 **CLOSED** uno a uno, con evidencia. Dejó dos LOW no bloqueantes, corregidos en este mismo commit: **LOW-A** una cifra vieja ("8 en la corrida") en `operaciones.md`, y **LOW-B** el bloque `-not $moved` afirmaba "HEAD did not move: it is still <sha>" cuando un hook **borra** la ref y vacía el índice (`$moved` es false también si `$headAfter` es `$null`); ahora dice "HEAD is no longer resolvable". Sus dos verificaciones manuales opcionales (aviso de hooks saltados, lock de ref) quedaron como escenarios 14 y 15.

**Salida literal del gate en el commit real (`a31f8eb8a`, primera versión — el camino de producción, no un repo desechable):**

```
staged in the private index:
  M	dev-tools/git-commit-isolated.ps1
  M	dev-tools/tests/git-commit-isolated.tests.ps1
  M	docs/dev/Backlog.md
  M	docs/dev/avance/activo/operaciones.md
  A	docs/dev/tasks/FIND-780.md
  M	docs/dev/workflow/RULES.md
pre-commit hook passed.
shared index refreshed for: …
ok absent: docs/dev/Backlog.md :: FIND-780
ok present: dev-tools/git-commit-isolated.ps1 :: commit-tree
ok present: docs/dev/workflow/RULES.md :: git commit-tree
OK: a31f8eb8a verified by content (3 expectation(s)).

pre-commit: [FIND-726] unstaged changes in tracked files — left untouched:
  docs/dev/plans/2026-11-23-release-v0.10.0-gates.budget.json
pre-commit: running .githooks/pre-commit-config.yaml gates (no stash)...
cargo fmt --check....................................(no files to check)Skipped
cargo clippy (subset rápido).........................(no files to check)Skipped
markdownlint-cli2........................................................Passed
pre-commit: gates passed.
ALL 0 PASS
pre-commit: staged-area gate passed.
pre-commit: running actionlint...
pre-commit: actionlint ok.
```

(`cargo fmt`/`clippy` salen `Skipped` porque el commit no toca `.rs`; `markdownlint`, `verify_staged` y `actionlint` sí midieron. El otro carril tenía WIP en el `budget.json` y el hook lo dejó intacto.)

## Deuda / techo declarado

- **Hooks implícitos:** `prepare-commit-msg`, `commit-msg` y `post-commit` no corren. Hoy no existen en `.githooks` (sólo `pre-commit` y `pre-push`); el helper **avisa si aparecen** y añadir uno obliga a cablearlo en el helper (declarado en el `.DESCRIPTION`). El aviso no puede hacer más que eso: no hay forma de ejecutar un hook que no se ejecuta.
- **Residuo del kill:** un `Stop-Process -Force` no ejecuta el `finally`, así que puede dejar un `idx-*` en `%TEMP%\opencode`. Es el mismo comportamiento que antes y no puede arreglarse desde dentro de un proceso muerto; el nombre lleva el PID para que sea atribuible.
- **Lock de ref residual:** un kill *dentro* de `git update-ref` puede dejar `.git/HEAD.lock` (o `refs/heads/<rama>.lock`). No hay pérdida de datos — la ref queda en el SHA viejo, que es el contrato — pero el siguiente `update-ref` falla hasta borrar el lock. Ventana de un renombrado; la misma exposición que tenía `git commit`, que también escribe refs.
- **"Nothing to commit" se decide ANTES de los hooks** (`git diff --cached` vacío → aborta). `git commit` corre `pre-commit` primero y re-chequea después, así que un hook que **stagee** ficheros generados puede completar un commit que este helper se niega a empezar. El shim de este repo nunca escribe el índice (verificado: `verify_staged.ps1` no tiene `git add/reset/restore/stash`), así que hoy no aplica; queda declarado (P2-01 LOW-2).
- **`commit-tree` no es `git commit`**: no escribe `.git/COMMIT_EDITMSG`, no honra `commit.gpgsign`, no limpia el mensaje `-m` y deja el reflog con el asunto crudo (sin el prefijo `commit: `). Nada de esto lo usa el repo (`commit.gpgsign=false`, sin política de firma); declarado en el `.DESCRIPTION`.
- **`--amend` sigue fuera de alcance** y `git commit --no-verify` ya no es una salida de emergencia del helper (no hay `git commit`); el escape es `SKIP=<id>` en el entorno, que el propio hook honra.
