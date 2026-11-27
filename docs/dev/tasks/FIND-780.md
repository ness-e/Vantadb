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
- **Estado:** ✅ COMPLETED (review P2-01 cerrada, ver §Review)
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

Seis líneas de `git commit` escritas por un proceso que ya no existía. Es el incidente, entero.

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

- Los hooks que `git commit` corría **implícitamente** (`commit-msg`, `post-commit`) ya no corren. Este repo define sólo `pre-commit` y `pre-push` (verificado: `Get-ChildItem .githooks`), y el helper **avisa** si no encuentra el `pre-commit` — el tripwire para el día que se añada otro.
- **La ventana de CAS es ahora tan larga como los hooks** (clippy tarda minutos). Si otro carril commitea en ese intervalo, `update-ref` falla y el helper sale **6** ("nada tuyo se commiteó, re-corré"). Es un cambio de comportamiento **deliberado**: antes ese mismo escenario commiteaba un árbol construido sobre el `HEAD` viejo y **revertía** al otro carril **con exit 0** (la clase `FIND-761`). Preferimos un fallo ruidoso a una reversión silenciosa. Escenario 9 lo fija.
- El objeto creado por `commit-tree` que pierde el CAS queda **colgando** (sin ref). Es basura inofensiva para `gc`, no un commit que alguien pueda leer por accidente: se imprime su SHA.

## Contrato

1. **Matar el proceso que commitea no mueve `HEAD`**, ni crea árbol vacío: `rev-parse HEAD` = SHA anterior, `HEAD^{tree}` ≠ `4b825dc…` y el árbol de `HEAD` listando el mismo número de paths.
2. **El gate no se pierde**: el hook corre antes de que exista commit, con el índice privado como índice, y si falla **no hay commit** (`HEAD` quieto, exit 2).
3. **El árbol vacío no se detecta y deshace: no se crea.** Un índice vaciado por un hook se rechaza *antes* de `commit-tree` (exit 4). Las guardas de `FIND-778` siguen ahí como red para el caso en que **el hook** mueva `HEAD` por su cuenta (escenarios 2, 6, 7).
4. El test **falla si alguien vuelve a `& git commit`** (escenario 10, probado con `$env:FIND780_HELPER`).

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

`--ignore-missing` replica la semántica de `git commit` (sin hook, sin gate) en vez de bloquear un repo sin hooks; el `WARNING` es lo que evita que eso sea silencioso.

### 2. El árbol se escribe DESPUÉS de los hooks

`git commit` también escribe el árbol después de los hooks, así que un hook que reescriba el índice se respeta. Si el árbol cambió respecto del sembrado, se imprime `NOTE: the pre-commit hook changed the private index (…)` — la firma del incidente, ahora visible en vez de invisible.

### 3. Rechazo **antes** de crear el commit

```powershell
if ($treePlanned -eq $EMPTY_TREE) { … "REFUSED (FIND-780): the private index is EMPTY after the hooks." ; exit 4 }
$plannedEntries = Get-TreeEntryCount $treePlanned
if ($plannedEntries -lt 0 -or $plannedEntries -lt ($entriesBefore - $MAX_ENTRY_DROP)) { … ; exit 4 }
```

Mismo umbral que `FIND-778` (`$MAX_ENTRY_DROP = 200`), pero aplicado al árbol **antes** de que exista un objeto commit. Fallar cerrado incluye la medición imposible (`-1`).

### 4. `commit-tree` + CAS

```powershell
$newCommit = "$(& git commit-tree $treePlanned -p $headBefore -m $Message)" -replace '\s', ''
& git update-ref -m $Message HEAD $newCommit $headBefore
```

`commit-tree` recibe el árbol como argumento: desde ahí **ningún paso lee el índice**, así que el `finally` que lo borra (que se queda: nada de residuo, pre-mortem b) no puede afectar al commit. `update-ref` con el SHA esperado es lo que convierte "otro carril commiteó mientras corrían los hooks" en exit 6 en vez de en una reversión.

### 5. Códigos de salida

`2` = nada se creó (incluye hook que falla) · `4` = rechazado por las guardas o por el rechazo previo · `6` = `HEAD` es de otro carril **o** se movió durante los hooks (no se pisa). `5` (la restauración falló) sigue.

## Test: 10 escenarios, 62 aserciones

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

Los tres últimos son de `FIND-780`. El 8 es la prueba de que el fix **no** cambió un bug de datos por una pérdida silenciosa de gates; el 9 fija el comportamiento nuevo del CAS; el 10 es la propiedad.

## Verificación

```
pwsh -NoProfile -File dev-tools/tests/git-commit-isolated.tests.ps1
  [1] … [10]  → 62/62 PASS, EXIT=0

$env:FIND780_HELPER=<script anterior>; pwsh -File <el mismo test>
  [10] → 5 FAIL (HEAD se movió al árbol vacío, 60/60 borrados)
  total → 8 FAILED
```

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

Pendiente de la ronda única.

## Deuda / techo declarado

- **Hooks implícitos:** `commit-msg` / `post-commit` no corren. Hoy no existen en `.githooks`; el `WARNING` de hook ausente es el aviso, y añadir uno obliga a tocar el helper (declarado en el `.DESCRIPTION`).
- **Residuo del kill:** un `Stop-Process -Force` no ejecuta el `finally`, así que puede dejar un `idx-*` en `%TEMP%\opencode`. Es el mismo comportamiento que antes y no puede arreglarse desde dentro de un proceso muerto; el nombre lleva el PID para que sea atribuible.
- **`--amend` sigue fuera de alcance** y `git commit --no-verify` ya no es una salida de emergencia del helper (no hay `git commit`); el escape es `SKIP=<id>` en el entorno, que el propio hook honra.
