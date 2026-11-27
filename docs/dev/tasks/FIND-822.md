---
title: "FIND-822: check-closure lee un rewrite de fila como retirada"
kind: task
status: in-progress
description: "El detector de retiradas de `check-closure` lee toda línea `-` del diff como fila retirada, así que una fila reescrita in-place (o un rewrite del fichero) pide entrada de avance aunque siga en el Backlog: 5 falsos positivos. Fix: restar los IDs que aún poseen fila en el fichero que se commitea. Una retirada real sin avance sigue roja (`FIND-820`)."
---

# FIND-822: `Get-RemovedBacklogIds` lee cualquier rewrite como retirada

## Metadata
- **Fuente:** `docs/dev/Backlog.md` (fila `FIND-822`, línea 987)
- **Origen:** lote 2 del barrido de marcas (`FIND-818`, 2026-11-27) · **Ver:** `FIND-818`, `FIND-792` (por qué el detector existe)
- **Esfuerzo:** 30-60min · **Prioridad:** 🟡 Media · **Tipo:** bug-fix de gate (`dev-tools/`, sin Rust)
- **Creado / cerrado:** 2026-11-27
- **Estado:** ⏳ IN PROGRESS

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Tocado | `dev-tools/check-closure.ps1` (`Get-RemovedBacklogIds` + 2 fixtures e2e) · este task file · `docs/dev/Backlog.md` (solo la fila `FIND-822`) · `docs/dev/avance/activo/operaciones.md` |
| Inmutable | `dev-tools/git-commit-isolated.ps1` (instrumento de todos) · `dev-tools/check-harness-contract.mjs` · `.github/**` · `src/**` (`FIND-800` compilando) · `vanta-memory/**` · `vantadb-mcp/**` · `.opencode/**` · `docs/dev/plans/**` · `docs/index.md` |
| Fuera de alcance | las retiradas reales ajenas (`FIND-820`, sin avance): el gate debe seguirlas marcando, no taparlas |

## Premisa (medida, no re-derivada)

Reproducción mínima en repo temporal (`docs/dev/Backlog.md` con una fila, commit, y **solo el texto de la celda reescrito** — la fila sigue ahí):

```
$ git -C $tmp diff -U0 HEAD -- docs/dev/Backlog.md
-| `FIND-901` | fila viva, marcador original |
+| `FIND-901` | fila viva, marcador REESCRITO |

$ pwsh dev-tools/check-closure.ps1 -Diff -Base HEAD -Root $tmp
  FAIL  FIND-901 : row removed from docs/dev/Backlog.md with NO entry in docs/dev/avance/
EXIT_BEFORE=1
```

Causa: `git diff` representa una línea **editada** como `-vieja` + `+nueva`. `Get-RemovedBacklogIds` (`check-closure.ps1:148-159`) se queda con las `-` y **no resta** los IDs que sobreviven, así que un rewrite in-place se lee como cierre.

Medición en el árbol real (`-Audit`, base `origin/develop`), cruzando "¿el ID aún posee fila en el Backlog?" contra las violaciones:

| ID reportado como retirado | ¿Posee fila hoy? | ¿Mención en `avance/`? | Veredicto |
|---|---|---|---|
| `MEMG-05` · `EGO-13` · `MGR-05` · `H3-05` · `H3-08` | **sí** | 0 | **falso positivo** (rewrite del lote `FIND-818`) |
| `FIND-820` | **no** | 0 | retirada **real** sin avance → debe seguir rojo |
| `FIND-819` | no | 2 | retirada real con avance → verde (control) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `dev-tools/check-closure.ps1` (408 líneas)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** ninguno — el script sólo usa `git` y `[System.IO.File]`; no importa módulos del repo.
- **Archivos que referencian a los editados (referencias entrantes):** `git grep -l check-closure` → `dev-tools/check-harness-contract.mjs:23` (sólo un comentario), `docs/dev/tasks/H-02-H-07.md`, `docs/dev/tasks/FIND-818.md`, `docs/dev/tasks/FIND-779.md`, `docs/dev/tasks/FIND-792.md`, `docs/dev/tasks/FIND-777.md`, `docs/dev/reviews/2026-11-26-harness-hardening.md`, `docs/dev/avance/activo/operaciones.md`, `docs/dev/Backlog.md`. Ninguno invoca la función por nombre ni depende de su firma: **el script es el contrato** (CLI + exit codes 0/1/2).
- **Veredicto impacto:** **bajo**. Cambia una función interna (`Get-RemovedBacklogIds`) y su call site único (`:381`); la interfaz pública del script (`-Diff/-Audit/-All/-SelfTest/-Base/-Root/-Paths` + exit codes) no se toca. Riesgo real: aflojar el gate (por eso el contrato exige probar **las dos direcciones**).

## Spec

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|---------------------|----------|
| 1 | ¿Qué define "retirada"? | (a) la línea no está en el diff — es lo que hay hoy y confunde edición con retirada · (b) **el ID no posee fila en el fichero que se commitea** · (c) `git show HEAD:` vs worktree | (b) | ✅ (b): es el invariante que la fila nombra ("fila ausente ⟺ entrada de avance") y reusa `Get-BacklogOwnIds`, que ya existe para la regla (a) |
| 2 | ¿Contra qué fichero se compara la supervivencia? | (a) worktree (`$backlogText`, ya leído en `:303`) · (b) `git show HEAD:` | (a) | ✅ (a): el gate mide *lo que vas a commitear*; con `-Audit` (base `origin/develop`) `HEAD` sería el árbol equivocado |
| 3 | ¿Cómo se prueba que no se aflojó? | (a) un caso e2e de rewrite que debe **callar** y dejar intacto el de retirada real que debe **gritar** · (b) revisión manual | (a) | ✅ (a): `-SelfTest` ya corre reglas end-to-end en repo temporal; +2 fixtures |
| 4 | ¿Tocar `-Audit`? | (a) no, ambas rutas comparten `Get-RemovedBacklogIds` · (b) código separado por modo | (a) | ✅ (a): un solo arreglo, y se verifica con los exit codes de `-Diff` **y** `-Audit` |

## Steps

1. ✅ Reproducir el falso positivo (repo temporal, exit 1).
2. ✅ Arreglar `Get-RemovedBacklogIds` (restar las filas que sobreviven).
3. ✅ Añadir 2 fixtures e2e: rewrite in-place y rewrite por reordenamiento (deben **pasar**); el de retirada real sigue fallando (ya existía).
4. ✅ Re-verificar `-SelfTest`, `-Diff`, `-Audit` + `check-harness-contract --self-test`.
5. ⬜ **Retirar la fila `FIND-822` del Backlog** — bloqueado: el fichero compartido lo está reescribiendo el carril de `FIND-818` **en vivo**; medido que mi retirada fue **revertida sin aviso** ~90s después (mtime 14:39:25, la fila volvió). Retirarla ahora obliga a una de dos: commitear el WIP ajeno (12 filas) o dejar a ese carril con `exit 9` de su guard (`FIND-792`). Se retira en una ventana quieta, en **su propio commit** (§11: blob derivado de `HEAD`, `0` adiciones / `1` borrado).
6. ⬜ `node scripts/docs/gen-index.mjs --write` — fuera de alcance por contrato (absorbe los `docs/**` pendientes de otros carriles; la deriva ya existía en `HEAD`).

## Contrato de verificación

```
pwsh dev-tools/check-closure.ps1 -SelfTest                     → 9/9 + 7/7, exit 0
pwsh dev-tools/check-closure.ps1 -Diff                         → sigue rojo SOLO por artefactos ajenos (task file de otro carril)
pwsh dev-tools/check-closure.ps1 -Audit                        → sigue rojo SOLO por lo ajeno (2 task files + 1 retirada real sin avance)
node dev-tools/check-harness-contract.mjs --self-test          → 89/89
```

**Invariante que no se puede romper:** una fila ausente del Backlog commiteado (p. ej. `FIND-820`) **debe** seguir pidiendo entrada de avance. Un arreglo que silencie también los verdaderos es peor que el bug (`FIND-792`).

### Medido (antes → después)

| Comando | Antes | Después |
|---|---|---|
| `-SelfTest` | 9/9 + 5/5, exit 0 | 9/9 + **7/7**, exit 0 |
| `-SelfTest` con la resta revertida (fixtures nuevas) | — | **5/7**, exit 1 → las 2 fixtures nuevas son RED sin el arreglo |
| `-Diff` | exit 0 (árbol limpio) | **exit 1, 1 violación — todas ajenas** (`FIND-800.md`, untracked de otro carril, sin `Estado` detectable) |
| `-Audit` | **7** violaciones | **3** — las 5 falsas positivas desaparecidas; las 3 restantes son de otros carriles (dos task files ajenos sin `Estado` detectable + una retirada real sin avance) |
| repo temporal: fila reescrita | exit 1 (falso positivo) | exit 0 |
| repo temporal: fila **retirada de verdad** | exit 1 | **exit 1** (sin cambio, `FIND-902`) |
| `check-harness-contract.mjs --self-test` | 89/89 | 89/89 (no tocado) |

## Deuda declarada

- **`docs/index.md` desactualizado:** `node scripts/docs/gen-index.mjs --check` falla con **8 hunks y +2 documentos** (904 → 906): uno es **este** task file y el otro un task file **ajeno** todavía sin commitear; el resto es deriva de descripciones y de alineación de otros carriles (ya fallaba antes de esta tarea). `gen-index --write` está **fuera del alcance** por contrato: regenerarlo absorbe los `docs/**` pendientes de otros carriles y hornea una copia stale — la misma clase que `FIND-813`. Lo regenera quien cierre el árbol limpio.
- **El filtro compara contra el worktree, no contra `git show HEAD:`.** Correcto para `-Diff` (mide lo que se va a commitear) y para `-Audit` (base `origin/develop`), pero una fila retirada y reintroducida en el mismo commit no se distingue de una fila viva.
- **`Test-AvanceHasId` es una mención textual** en `avance/`: nombrar una fila ajena retirada la absolvería. Por eso la entrada de avance de esta tarea no enumera IDs ajenos (los cinco falsos positivos viven en este task file, que no se escanea).

## Context Save Point

- **Hecho y commiteado:** `dev-tools/check-closure.ps1` (la resta de supervivientes + 2 fixtures e2e), este task file y la entrada de `docs/dev/avance/activo/operaciones.md`.
- **Verificado:** `-SelfTest` 9/9 + 7/7 exit 0; con la guarda revertida **5/7 exit 1** (fixtures RED); rewrite → exit 0 y retirada real → exit 1 en el mismo repo; `check-harness-contract --self-test` 89/89.
- **Pendiente (1 comando, cuando el Backlog esté quieto):** construir el blob como `HEAD` − mi fila, `git diff --numstat` debe dar **`0 1 docs/dev/Backlog.md`**, y commitearlo solo (§11). Después, `node scripts/docs/gen-index.mjs --write`.
- **No hacer:** commitear `docs/dev/Backlog.md` desde el worktree — en este momento contiene 12 filas reescritas por otro carril y **cero** contenido de esta tarea.

## Review

### P2-01 ronda 1 (contexto fresco, `vanta-review`) — **changes-required**, aplicado

Veredicto: `changes-required` con **1 HIGH, 1 MEDIUM, 2 LOW**. Lo que confirmó como sano: el detector **no** pierde ninguna retirada real (IDs únicos ⇒ ninguna fila absuelve a otra; fichero borrado ⇒ `$Present` vacío ⇒ todas las `-` se marcan; fila movida de sección ⇒ correctamente silenciada; `-Paths` nunca alcanza la regla (b)); orden de definición seguro (`Get-BacklogOwnIds` :320 → `$backlogIds` :348 → call :408) y las fixtures son **true-RED** (`5/7` con la guarda revertida).

| # | Sev | Hallazgo | Acción |
|---|-----|----------|--------|
| 1 | HIGH | `docs/dev/Backlog.md` no lleva **nada** mío (12 pares `-`/`+` de otro carril, 0 de `FIND-822`) | **Se saca del commit.** El otro carril lo reescribió en vivo y **revirtió mi retirada de fila** (medido: mtime 90s después de mi escritura, fila de vuelta). Retirarla aquí = llevarme su WIP (clase `FIND-753`) o bloquear su commit (`exit 9` de su guard). Queda como step pendiente. |
| 2 | MEDIUM | "`-Audit` 7 → 2" ya no reproduce | Re-medido y corregido a **7 → 3** con su composición (un task file ajeno nuevo, sin commitear, apareció durante el cierre). |
| 3 | LOW | `[object]$Present`: un `string` suelto haría `.Contains` = **substring** y se comería una retirada | Param **tipado** `[HashSet[string]]`: un caller equivocado falla ruidoso en vez de absolver. Re-verificado 7/7 + las dos direcciones. |
| 4 | LOW | El comentario decía "file being committed" y lee el **worktree** | Corregido a "worktree file" (+ por qué es el par correcto de `-Diff` y de `-Audit`). |

### OCR delegation (`dev-tools/ocr-review.ps1`, rule group 1 = PowerShell)

`ocr delegate rule <paths>` asignó **rule group 1 (PowerShell Review Principles)** a `dev-tools/check-closure.ps1`. Auditado contra sus seis ejes:

| Eje | Veredicto |
|---|---|
| **Inyección / construcción de comandos** | n/a: no hay `Invoke-Expression`, ni `[scriptblock]::Create`, ni `pwsh -Command`; el único proceso es `git diff` con argumentos fijos. `$Present` sale de filas del propio fichero, no de input externo. |
| **Parámetros, tipos, validación** | `[object]$Present` es deliberado: el call site pasa un `HashSet[string]` y forzarlo a `[HashSet]` haría que un `$null` fallara con una excepción de binding en vez de degradar. El guard `$Present -and $Present.Contains($id)` cubre los tres estados de PowerShell: `$null` → falso, `HashSet` **vacío** → falso (colección vacía es falsy) → *todas* las `-` se marcan como retiradas, que es lo correcto si el Backlog quedó sin filas; `HashSet` con filas → resta los supervivientes. Verificado a mano por los dos caminos. |
| **Estado de procesos nativos** | se mantiene el `$LASTEXITCODE -ne 0 → return @()` preexistente; mi cambio no añade procesos. |
| **Semántica de pipeline** | `return $ids` (List) se desenrolla igual que antes y el call site ya lo envuelve en `@()`: la forma del retorno no cambia. |
| **Encoding** | sin cambios: la función sólo lee stdout de `git`, no escribe ficheros. |
| **Tests** | el fixture no puede pasar por la rama equivocada por construcción: `(b2)`/`(b3)` esperan exit 0 y `(c)` espera exit 1 en el **mismo** repo temporal, así que un fixture que "no detecta nada" rompe `(c)`. |


(pendiente)
