---
title: "FIND-796: el helper afirmaba 'the commit did not carry your edit' sin haberlo medido, y en el caso medido era falso"
kind: task
status: completed
description: "Un fallo de expectativa concluía `REVERTED: the commit did not carry your edit`, una causa que el script nunca midió y que en el caso medido era falsa (el blob del commit tenía el contenido). Ahora hay tres estados medidos: ausente (exit 1), AMBIGUOUS con `fichero:línea` de cada coincidencia (exit 7), y 'commit o path no está'. La frase inventada ya no existe."
---

# FIND-796: no afirmar una causa que no se midió

## Metadata

- **Fuente:** `docs/dev/Backlog.md` §`FIND-796` (fila retirada al cerrar)
- **Esfuerzo:** 🟢 1-2h → real ~2h (la diagnosis es la parte cara)
- **Prioridad:** 🟠 Alta
- **Tipo:** bug-fix · dev-tools
- **Creado:** 2026-11-26 · **Cerrado:** 2026-11-27
- **Estado:** ✅ implementado, verificado y commiteado

## Impacto mapeado (Regla 0)

- **Leído completo:** `dev-tools/git-commit-isolated.ps1` (bloque de verificación
  y bloque de restauración de `FIND-778`), `dev-tools/tests/git-commit-isolated.tests.ps1`,
  `docs/dev/workflow/RULES.md` §11, la fila `FIND-796`.
- **Referencias hacia dentro:** los cierres de tarea leen ese mensaje para decidir
  si rehacen trabajo (`docs/dev/tasks/FIND-789.md` lo registra como incidente).
- **Referencias hacia fuera:** ninguna; solo `git cat-file`/`rev-parse`.
- **Veredicto:** el mensaje es el contrato de la herramienta. Cambiarlo a un
  diagnóstico medido es el objetivo, no un efecto colateral.

## Diagnóstico (medido)

Al cerrar `FIND-789` se declaró `-ExpectAbsent 'docs/dev/tasks/FIND-789.md::status: active'`.
La cadena apareció en la **prosa** del propio fichero (explicando el estado
anterior), no en el frontmatter. Resultado:

```
REVERTED: 'status: active' is STILL in HEAD:docs/dev/tasks/FIND-789.md — the commit did not carry your edit.
FAILED: 1 content expectation(s) unmet
```

mientras `git show <commit>:docs/dev/tasks/FIND-789.md` tenía `status: completed`
y el árbol estaba intacto (**2379** ficheros). El fallo de la expectativa era
real; el diagnóstico era inventado. Y en la peor dirección: quien lea "el commit
no llevó tu edición" rehace el trabajo — o usa `--amend`, prohibido — sobre un
commit que está bien.

## Los tres estados (y el cuarto que no existe)

| Estado | Cuándo | Salida |
|---|---|---|
| (i) contenido ausente | 0 coincidencias en el blob del commit | `NOT COMMITTED: '<frag>' is not in the blob of <path> at <sha>` + `measured: read N chars from git cat-file (NFC-normalised on both sides), 0 occurrences` · **exit 1** |
| (ii) ambiguo / no atribuible | la coincidencia aparece **>1 vez** → `AMBIGUOUS EXPECTATION` con cada `fichero:línea` y su texto; o aparece **1 vez** en una línea **byte-idéntica al padre** → `PRESENT` + `the matched line is byte-identical in the parent commit, so this commit did not write it` | **exit 7** — el commit **existe y no está mal**; no reintentar |
| (ii-b) contenido que el commit escribió | 1 coincidencia en una línea que **difiere** del padre, con `-ExpectAbsent` | `PRESENT` + `this commit CHANGED that line` · **exit 1** (fallo de contenido medido) |
| (iii) commit/path ausente | `git rev-parse <sha>:<path>` falla | `NOT IN COMMIT: '<path>' does not exist in <sha>, so '<frag>' cannot be judged` · **exit 1** |

La frase `REVERTED: … the commit did not carry your edit` **se eliminó**. En su
lugar, cada línea impresa nombra la medición que la sostiene: coincidencias
contadas sobre el blob del commit, comparación literal de la línea contra el
blob del padre, o ausencia del path.

**Por qué "solo en la prosa" no se adjudica:** el helper no puede saber qué línea
quiso decir el operador; lo que sí puede es **nombrarla** y decir si el commit la
escribió o no. Atribuir "es prosa" sería exactamente el error que esta fila
corrige. El caso medido de `FIND-796` cae en (ii) porque la línea es idéntica al
padre: el mensaje dice que el commit no la escribió y enumera las dos causas
consistentes, sin elegir por el operador.

## Verificación

- Escenario 18 (nuevo): fichero con `status: completed` en la línea 1 y en la 3 →
  helper nuevo: **exit 7**, `AMBIGUOUS EXPECTATION`, `docs/file0.txt:1`,
  `docs/file0.txt:3`, y **no** contiene `did not carry your edit`.
- Contra el helper de `HEAD`: **exit 0** (el `-match` encontraba la primera) →
  **5 FAILED** en ese escenario. Es el caso "el gate pasaba sin decir cuál de las
  dos coincidencias verificó".
- Escenario 7 del banco nuevo (ausente sobre línea pre-existente, medido a mano
  durante el desarrollo): exit 7 con las dos hipótesis y `no reversion is
  claimed here`.
- Suite completa: **131/131 PASS** con el helper nuevo; **33 FAILED** con el de
  `HEAD` (todos en 16-20).

## Review P2-01

Tier **Fast**. Una ronda para las cuatro filas: `docs/dev/tasks/FIND-792.md` §Review.

## Deuda / techo declarado

- "Ambigua" se decide por **número de coincidencias** y por **identidad de la
  línea con el padre**, no por un análisis de prosa. Es deliberado: una
  heurística de prosa sería otra causa inventada. Un `-ExpectAbsent` que coincide
  una vez en una línea que el commit **sí** cambió se reporta como contenido
  (exit 1), que es la lectura literal y medible.
- Los exit 7/8 significan "el commit existe y no está mal": el orquestador no
  debe reintentar. Eso queda escrito en la cabecera del helper y en `RULES.md` §11.
