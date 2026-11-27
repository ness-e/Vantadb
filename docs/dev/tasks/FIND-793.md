---
title: "FIND-793: la verificación leía el blob por la consola, y un acento fallaba con el commit ya hecho"
kind: task
status: completed
description: "`-ExpectPresent`/`-ExpectAbsent` leían `HEAD:<path>` con `git show` por la tubería de PowerShell: el códec de consola volvía mojibake un `é` UTF-8 y el check fallaba DESPUÉS de crear el commit. Ahora el blob se lee con `git cat-file` + UTF-8 de .NET, ambos lados se normalizan a NFC, y una diferencia que solo son acentos sale como ENCODING (exit 8, no reintentar)."
---

# FIND-793: leer el blob, no la consola

## Metadata

- **Fuente:** `docs/dev/Backlog.md` §`FIND-793` (fila retirada al cerrar)
- **Esfuerzo:** 🟢 1-2h → real ~2h (una pieza compartida con `FIND-792/795/796`)
- **Prioridad:** 🟠 Alta
- **Tipo:** bug-fix · dev-tools (`dev-tools/git-commit-isolated.ps1`)
- **Creado:** 2026-11-26 · **Cerrado:** 2026-11-27
- **Estado:** ✅ implementado, verificado contra el helper de `HEAD` y commiteado

## Impacto mapeado (Regla 0)

- **Leído completo:** `dev-tools/git-commit-isolated.ps1` (512 líneas al abrir),
  `dev-tools/tests/git-commit-isolated.tests.ps1` (456 líneas al abrir, 15
  escenarios), `docs/dev/workflow/RULES.md` §11, la fila `FIND-793` del Backlog y
  las entradas de `docs/dev/avance/activo/operaciones.md` de `FIND-778`/`FIND-780`.
- **Referencias hacia dentro (quién usa el helper):** humanos y agentes por la
  política de `RULES.md` §11 y `AGENTS.md` §Flujo mínimo; `dev-tools/check-harness-contract.mjs`
  lo nombra en su regex `COMMIT_OK` pero **no lo invoca** (y está fuera de alcance).
  Ningún hook ni workflow lo llama: no hay contrato de proceso que romper.
- **Referencias hacia fuera:** `git` (rev-parse/cat-file/hash-object/write-tree/
  commit-tree/update-ref/hook run), `$env:TEMP` para el índice privado y los
  volcados de blob. **Ninguna se modificó.**
- **Veredicto:** el cambio es interno al helper y a su test; el único consumidor
  con contrato escrito es `RULES.md` §11, que se actualiza en el mismo commit.

## Diagnóstico (medido, no razonado)

Al cerrar `FIND-775`, el helper reportó:

```
NOT COMMITTED: '60/60 despu??s' is missing from HEAD:docs/dev/tasks/FIND-775.md
FAILED: 1 content expectation(s) unmet at <commit>
```

mientras `git show HEAD:docs/dev/tasks/FIND-775.md` **contenía** `60/60 después`
y el commit tenía exactamente los 2 ficheros esperados. El `é` viajó mojibakeado
desde el llamador y el helper lo comparó **contra el texto que la consola le
había entregado**, no contra el blob.

Segundo caso del mismo día, peor porque corrompe al escribir: una fila del
Backlog restaurada con `git show <ref>:<path>` a través de PowerShell salió con
**1519** chars donde la genuina tenía **1428** (CP437 partió el emoji de
severidad en tres caracteres y perdió la `ó`), 20 secuencias mojibake y **175**
`??` en vez de 177. El número que se reportó como prueba ("1428 chars") era la
longitud del mojibake. Otro carril (`bf3671d81`) había declarado la misma trampa
minutos antes: 514/516 líneas mangleadas.

**El defecto no es el acento: es el canal.** Ninguna verificación puede leer
texto acentuado de la consola de PowerShell; y "la expectativa falta" (defecto de
contenido) y "la expectativa se corrompió" (defecto del llamador) salían con el
mismo código, así que un orquestador que reintenta produce un **commit
duplicado** y uno que aborta deja el trabajo hecho pero reportado como fallido.

## Lo que se implementó

| Defecto | Medida |
|---|---|
| Lectura por consola | `Get-BlobText` resuelve el oid (`git rev-parse <rev>:<path>`) y vuelca el objeto con `git cat-file blob` **redirigido a fichero por el SO** (`Start-Process -RedirectStandardOutput`), decodificado con `[Text.Encoding]::UTF8` de .NET. Ni un byte pasa por la tubería de PowerShell. |
| NFC vs NFD | `ConvertTo-Nfc` en ambos lados antes de comparar (`String.Normalize(FormC)`). |
| Sin distinguir transporte de contenido | Cadena de diagnóstico: (1) match exacto NFC; (2) proyección sin diacríticos (`Get-AsciiSkeleton`) — si aparece, `ENCODING` con `fichero:línea`; (3) reparación Latin-1→UTF-8 (`Repair-DoubleEncoding`) — si aparece, `ENCODING` con el texto ya decodificado; (4) si nada aparece, `NOT COMMITTED` (contenido). |
| Exit code ambiguo | `8` = EXPECTATION UNTRUSTWORTHY (el commit **existe**, no reintentar) vs `1` = contenido. |
| Escritura mojibakeada | El helper no escribe ficheros; el techo se documenta en §11 y en la fila de `FIND-792` (las escrituras van con `UTF8Encoding($false)` + conteo de mojibake, fuera del helper). |

`Get-AsciiSkeleton` **dobla** los acentos (`NFD` + descarte de `NonSpacingMark`)
en vez de borrar los no-ASCII: `después` y `despues` comparten `despues`, que es
la señal; borrar no-ASCII daría `despus` y no detectaría nada. Si el fragmento es
puramente ASCII y el blob acentuado, el helper **no adivina**: reporta contenido
y añade una línea que dice por qué no puede diagnosticar transporte (un fragmento
ASCII sin el acento es una cadena distinta, no una corrupción medible).

## Verificación

- Suite completa: `pwsh -NoProfile -File dev-tools/tests/git-commit-isolated.tests.ps1`
  → **131/131 assertions PASS**, exit 0 (20 escenarios).
- Escenario 17 (nuevo), contra el helper **nuevo**: NFD vs NFC → exit 0 y
  `ok present`; expectativa doble-codificada → **exit 8**, `ENCODING`,
  `do NOT re-run`, y **no** imprime `NOT COMMITTED`; fragmento realmente ausente
  → exit 1 + `NOT COMMITTED` (el control que prueba que las clases no se comieron
  el diagnóstico real).
- Escenario 17 contra el helper de `HEAD` (`$env:FIND780_HELPER=<git show HEAD:…>`):
  **8 FAILED** (`exit 0: … got 1`, `exit 8 … got 1`, `names the encoding class`,
  `do NOT re-run`, `does NOT report missing content`, `reports it present`).
  Los escenarios 1-15 siguen verdes.
- Commit real con acento en la expectativa: `c7689bcce` imprimió
  `ok present: docs/dev/tasks/FIND-793.md:44 :: 60/60 después` (la expectativa se
  construyó desde code points; el texto se comparó contra el blob). La corrida
  completa salió `exit 7` por una SEGUNDA expectativa demasiado ancha — ver
  `operaciones.md`, §"El propio cierre, medido".

## Review P2-01

Tier **Fast**: los paths del diff son `dev-tools/**` y `docs/dev/**` (regla
mecánica de `pipeline-full.md` §Cierre). Veredicto del reviewer independiente y
hallazgos aplicados: ver `docs/dev/tasks/FIND-792.md` §Review (una sola ronda
para las cuatro filas, porque es un solo cambio).

## Deuda / techo declarado

- Un fragmento **puramente ASCII** que debería llevar acento no se clasifica como
  encoding: es una cadena distinta y el helper no puede saber qué quiso escribir
  el llamador. Imprime la pista y falla como contenido (medido, no inventado).
- El helper no controla cómo el **llamador** construye el argumento (`-File`
  puede unir un array; un pipe de texto puede manglear). La forma segura está en
  `RULES.md` §11: array, y `git hash-object`/`git cat-file` para el contenido.
