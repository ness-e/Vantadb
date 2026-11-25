---
title: "FIND-726 — el pre-commit stashea el árbol compartido y pierde trabajo ajeno"
kind: task
status: completed
description: "`pre-commit run` revierte a índice todo fichero tracked y luego reaplica el diff; en un checkout con varios agentes eso borró trabajo ajeno 3 veces. El hook ahora pasa `--files` y no toca el working tree."
---

# FIND-726 — el `pre-commit` no debe stashear un árbol compartido

> **Plan:** `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` §6.c
> **Ruta:** `vanta-worker` · **Carril:** C · **Appetite:** 1-2h
> **Dependencias:** `FIND-718` ✅ (`fc08c4b35`, reparó el hook y `core.hooksPath`)
> **Paralelo (disjunto):** `FIND-722` (Rust), `FIND-233` (workflow de bench)

## Contrato

Un commit **nunca** puede alterar el contenido de un fichero tracked que otro
proceso esté editando. Si el gate no puede garantizarlo, **falla ruidoso**; no
"arregla" el árbol.

```
# 1) fichero tracked con cambios unstaged
# 2) correr el hook
# 3) git hash-object <file> IDÉNTICO antes y después
```

Más el caso normal: árbol limpio sigue commiteando en verde.

## Evidencia (recogida hoy, no re-derivada)

| ID | Qué pasó |
|---|---|
| `FIND-720` | ediciones revertidas dos veces |
| `FIND-723` | sus 10 archivos desaparecieron ~2 min; su primer commit terminó con contenido ajeno |
| `FIND-721` | sus 3 archivos quedaron stasheados **sin restaurar**, `git diff` vacío |

## Causa raíz

`pre-commit run` revierte cada fichero tracked a su contenido de índice y
después reaplica el diff unstaged. Es una decisión explícita del upstream:

```python
# pre_commit/commands/run.py (verificado en v4.6.2 y en main)
stash = not args.all_files and not args.files
```

```python
# pre_commit/staged_files_only.py::_unstaged_changes_cleared
cmd_output_b(*_CHECKOUT_CMD, ...)   # git checkout -- .   <-- aquí muere el trabajo ajeno
yield
_git_apply(patch_filename)          # replay del diff capturado antes
```

Correcto para un desarrollador con un checkout. **Incorrecto** para un checkout
compartido por 2-5 agentes: cualquier escritura que caiga dentro de la ventana
la pisa el replay, y un run interrumpido deja el árbol revertido sin stash del
que recuperar (el parche queda en `~/.cache/pre-commit/patch*`, a mano).

Doc oficial: <https://pre-commit.com/#pre-commit> — *"pre-commit only runs on the
staged contents of files by temporarily stashing the unstaged changes while
running hooks"*. No existe `--no-stash` en `pre-commit run` (sólo en la CLI
`hook-impl` de versiones antiguas); la única palanca documentada sobre `stash`
es `--files` / `--all-files`.

## Opciones evaluadas (medidas, no asumidas)

| Opción | Veredicto |
|---|---|
| **(a)** invocar `pre-commit` con `--files <staged>` (= `--no-stash` efectivo) | **Elegida.** Mismo set de ficheros que deriva `pre-commit` (`git diff --staged --diff-filter=ACMRTUXB`), `stash` pasa a `False`, el hook no revierte ni reaplica el árbol |
| **(b)** detectar WIP tracked y abortar | **Rechazada.** En este checkout el WIP ajeno es el estado normal (medido hoy: `M src/executor.rs` de `FIND-722` mientras corría esta tarea). Abortar bloquearía *todos* los commits y empujaría a `--no-verify`, que es cero gate |
| **(c)** ambas | Reducida a (a) + **aviso** con la lista de ficheros que se dejan intactos y con los *partially staged*. El aviso explica un rojo de gate causado por WIP ajeno; no bloquea |
| **(d)** materializar el índice en un worktree temporal y correr ahí | **Descartada** (P2-01, F6): recuperaría semántica de contenido-index de verdad, pero añade `write-tree` + `checkout-index` + ciclo de vida de un worktree por commit. Es la salida si algún día el aviso de *partially staged* no basta |

**Trade-off documentado:** con `--files` los hooks ven el **working tree**, no el
índice. Los hooks con `pass_filenames` (ruff, prettier, markdownlint, junk)
reciben los mismos **nombres** pero leen contenido del working tree: un fichero
*partially staged* se valida sobre una versión y se commitea otra, y eso ahora se
nombra en pantalla. Los whole-tree (`cargo fmt --check`, `cargo clippy`, clippy
por miembro de `verify_staged`) **sí** pueden fallar por un fichero que otro
agente está editando, e incluso reportar un falso "files were modified by this
hook" si ese agente escribe durante el run. Ruidoso, nunca destructivo. Bypass
explícito: `SKIP=cargo-fmt,cargo-clippy git commit ...`.

## Impacto mapeado (Regla 0)

**Leídos completos antes de editar**

| Archivo | Qué se leyó | Por qué |
|---|---|---|
| `.githooks/pre-commit` (104 L) | hook completo, bloque yaml + `verify_staged` + actionlint | es el artefacto que se modifica |
| `.githooks/pre-commit-config.yaml` (85 L) | `stages`, `pass_filenames`, `files` de los 7 hooks | decide qué hooks ven el árbol y cuáles sólo el índice |
| `pre_commit/commands/run.py`, `staged_files_only.py`, `hook_impl.py` (v4.6.2 y main) | línea `stash =`, `_unstaged_changes_cleared`, guards de merge/config | fuente de verdad de la semántica |
| `dev-tools/verify_staged.ps1` (80 L) | `filesIn()` usa `git diff --cached` | confirma que el gate post-yaml es index-scoped |
| `docs/dev/references/troubleshooting.md` § git hooks | sección creada por `FIND-718` | sección donde va el síntoma |

**Referencias hacia dentro**

- El hook lo invoca **git** en cada commit (`core.hooksPath=.githooks`), no hay
  callers en código.
- `$@` que el hook reenviaba a `pre-commit run`: el stage `pre-commit` de git no
  pasa argumentos → se elimina al insertar `--files` al final del comando.
- `verify_staged.ps1` corre **después** del bloque yaml y lee el índice: no
  cambia con `--files`, pero su clippy por miembro compila el working tree.

**Veredicto:** blast radius = todos los commits del repo, pero el cambio es
aditivo (parámetros del mismo comando) y reversible con un solo revert.

## Steps

1. ✅ RED — reproducir el stash con el hook de `HEAD` en un clon descartable.
2. ✅ GREEN — `--files` + guards (unmerged, config unstaged) + aviso de WIP.
3. ✅ Verify — contrato de hash, caso limpio en verde, gate sigue mordiendo.
4. ✅ Troubleshooting § git hooks.
5. ✅ Review P2-01 (`vanta-review`, changes-required) → F1/F2/F9 corregidos y re-verificados; commit local.

## Verificación (contrato)

Clon descartable (`git clone` de `develop` en `%TEMP%\opencode\find726-clone`),
`core.hooksPath=.githooks`, `pre-commit` visible vía shim en `PATH` (sin el shim
el hook cae a su fallback y el stash nunca corre).

**RED — hook de `HEAD`** (`c8ac579e373f24853bb9ccde4f714c7443fae986`):

```
[WARNING] Unstaged files detected.
[INFO] Stashing unstaged files to C:\Users\Eros\.cache\pre-commit\patch1795618875-27320.
[INFO] Restored changes from C:\Users\Eros\.cache\pre-commit\patch1795618875-27320.
```

y con una escritura concurrente (agente 2 escribe 900 ms dentro de la ventana):

```
BEFORE:  l1 / l2-AGENT1-unstaged / l3
WROTE agent-2 during hook window
AFTER:   l1 / l2-AGENT2-concurrent / l3
AGENT1_PRESENT: False        <-- el trabajo del agente 1 se perdió
AGENT2_PRESENT: True
```

El log dice `Restored changes` igual. Ese es el fallo silencioso de `FIND-720/721/723`.

**GREEN — hook nuevo** (`12574455dfe0881ac9d296221704b001f8ee5718`):

| Caso | Comando | Resultado |
|---|---|---|
| hash del fichero con edición unstaged | `sh .githooks/pre-commit` | `H0_BEFORE: 275725b78fec46d83f2a944d0fa5e0b32541d89c` · `H1_AFTER : 275725b78fec46d83f2a944d0fa5e0b32541d89c` · **HASH_IDENTICAL: True** · `STASH_LINES: 0` |
| escritura concurrente | idem + `Add-Content` a los 900 ms | `A_PRESENT: True   B_PRESENT: True` |
| árbol limpio (caso normal) | `git commit` real, hook disparado por git | `COMMIT_RC: 0` → `[develop 069c70496]` |
| el gate sigue mordiendo | `git commit` con `junk.tar.gz` staged | `ERROR [DOC-04]: Prohibited junk/binary files` · `COMMIT_RC: 1` |
| **F1 (P2-01)** sólo un borrado staged + WIP ajeno | `sh .githooks/pre-commit` | `STAGED_FILTERED(psh): []` · `"nothing to check (no staged file with content)"` · **`STASH_LINES: 0`** · hash idéntico |
| **F2 (P2-01)** fichero partially staged | idem | aviso `partially staged — hooks validated the WORKING-TREE version` + `STASH_LINES: 0` + hash idéntico |

Los dos últimos se corrieron con el hook `ac7f6e81` (post-review) **commiteado en
el clon**, porque `git checkout -- .` del arnés revertía el fichero del hook al
de `HEAD` y las dos primeras pasadas midieron sin querer el hook viejo.

**Producción — el árbol real compartido.** El commit de esta tarea se hizo sobre
`develop` con **15 ficheros tracked de otros agentes** con ediciones unstaged
(`src/executor.rs`, `src/llm.rs`, `vantadb-python/*`, `vantadb-ts/*`,
`vantadb-wasm/src/lib.rs`, `docs/api/*`), con snapshot previo en un parche
(45 KB):

```
FOREIGN_WIP_FILES: 15
COMMIT_RC: 0   ->  [develop 5d293d759] fix(hooks): never stash a shared worktree (FIND-726)
SAME 5cb949612452 docs/api/BINDINGS_NAMESPACES.md
SAME 8316a6cd2bd6 docs/api/EMBEDDINGS.md
... (15/15 SAME) ...
DRIFT_COUNT: 0
```

`git ls-tree -r HEAD .githooks` → `100755 blob ac7f6e81 .githooks/pre-commit`
(exec bit intacto).

## Review P2-01

- **Revisor:** `vanta-review`, contexto fresco (no participó de la implementación).
- **Enfoque:** fidelidad del set de ficheros contra `pre_commit/git.py::get_staged_files`
  (v4.6.2), `pass_filenames` de los 7 hooks, `$@` eliminado, los dos guards, el
  borde `xargs`/lista vacía, veracidad de la doc y falso verde. Reproducción **sólo**
  en el clon descartable, nunca en el checkout compartido.
- **Cómo se probó:** `pre-commit --version` (4.6.2), `xargs --version`, `printf "" |
  xargs -0 echo` para el caso lista vacía, `git diff HEAD --stat` contra el diff
  declarado, y re-ejecución del caso "sólo borrados" con el hook nuevo.
- **Veredicto:** **changes-required** → corregido en el mismo commit.

| Hallazgo | Sev. | Disposición |
|---|---|---|
| **F1** `xargs` con lista vacía (sólo borrados staged) reintroducía `stash=True`, y el hook decía "(no stash)" | Critical | **Corregido.** La guarda usa la lista **filtrada** (`--diff-filter=ACMRTUXB`, la misma que alimenta `--files`) y `xargs -0 -r` como cinturón. Verificado: `STASH_LINES: 0`. |
| **F2** "verdict unchanged" falso: con `--files` el *contenido* validado es el working tree | Required | **Corregido.** Aviso `partially staged — hooks validated the WORKING-TREE version` nombrando los ficheros; doc y task file corregidos. |
| **F3** task file decía `status: completed` sin `## Review` | Required | **Corregido.** Esta sección + steps sincronizados en el commit. |
| **F4** "cannot write into the tree at all" era absoluto | Optional | **Corregido** en la cabecera: los formatters (`ruff-format`, `prettier --write`) escriben in-place por diseño. |
| **F5** falso rojo "files were modified" si otro agente escribe durante el run | Optional | **Documentado** en cabecera + trade-off: ruidoso, no destructivo. |
| **F6** alternativa: materializar el índice en un worktree temporal | Optional | **Evaluada y descartada** con su fila en la tabla de opciones (opción (d)). |
| **F7** "no re-ejecutes el hook, revertiría el árbol" (ya es falso) + ruta `~/.cache` en Windows | Nit | **Corregido** en troubleshooting: ruta que imprimió el log + `git apply --check`. |
| **F8** el snippet esperaba `gates...` y el hook imprime `gates (no stash)...` | Nit | **Corregido.** |
| **F9** `if ! git diff --quiet` bloqueaba con cualquier rc ≠ 0 | Nit | **Corregido**: sólo rc 1 bloquea (como upstream); otro rc → warning. |
| **F10** listas >128 KiB parten en N runs; nombre que empieza por `-` | Nit | **Aceptado**: ruidoso, no destructivo. Sin cambio. |

Bug adicional propio detectado al implementar F2: `while … && printf` dentro de
una sustitución de comandos devolvía rc=1 cuando el último fichero no era
*partially staged*, y `set -e` **abortaba el hook**. Reescrito con `if`.


## Invariantes

- El bit `100755` de `.githooks/pre-commit` no se pierde (`git ls-tree -r HEAD .githooks`).
- El **set** de ficheros que ven los hooks es byte-idéntico al que derivaría
  `pre-commit` solo. El **contenido** validado es el working tree: un fichero
  *partially staged* se avisa en pantalla (no se aborta).
- El hook nunca **revierte ni reaplica** el working tree, y nunca stashea: si no
  puede verificar, falla. (Los formatters `ruff-format` / `prettier --write`
  escriben in-place por diseño, como antes.)

## Deuda / hallazgos laterales

- El parche del stash interrumpido queda en el `pre-commit` store
  (`C:\Users\<u>\.cache\pre-commit\patch*` en Windows, `~/.cache/...` en POSIX):
  recuperable a mano con la ruta que imprime el log. Documentado en troubleshooting.
- **NOTICED BUT NOT TOUCHING:** `node scripts/docs/gen-index.mjs --check` falla
  **en `HEAD` sin mis cambios** (verificado en el clon descartable, que no tiene
  `FIND-726.md`). Drift preexistente de `docs/index.md` + un `web/` untracked
  compartido; regenerarlo hornearía el WIP de `FIND-706/722` en el mismo commit.
  Fuera del scope de escritura declarado (`docs/index.md` no está en la lista).
- **NOTICED BUT NOT TOUCHING:** `.githooks/pre-commit:26` usa
  `command -v pre-commit`; en este Git for Windows el `sh` de git **no** resuelve
  `pre-commit.exe` sin extensión en `command -v` (medido), así que un commit
  lanzado desde bash cae al fallback y los gates yaml no corren. Arreglarlo
  activaría gates hoy dormidos en todo el repo: decisión del owner, no efecto
  lateral de este fix.

