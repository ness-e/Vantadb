---
title: "FIND-792: un commit de Backlog se llevaba la retirada de fila de otro carril, y el helper no podía verlo"
kind: task
status: completed
description: "`-Paths docs/dev/Backlog.md` commitea la copia ENTERA del fichero, así que la retirada de fila de otro carril (código sin commitear) viajaba dentro: el backlog decía 'cerrada' sobre un fichero que no está en la historia (medido 3 veces). El helper aborta ahora (exit 9) si una fila retirada no está declarada en una expectativa, salvo -AllowForeignRows."
---

# FIND-792: filas ajenas en un fichero compartido

## Metadata

- **Fuente:** `docs/dev/Backlog.md` §`FIND-792` (fila retirada al cerrar)
- **Esfuerzo:** 🟡 2-4h → real ~3h (incluye el diseño compartido de las cuatro filas)
- **Prioridad:** 🟠 Alta
- **Tipo:** bug-fix · dev-tools (`dev-tools/git-commit-isolated.ps1`)
- **Creado:** 2026-11-26 · **Cerrado:** 2026-11-27
- **Estado:** ✅ implementado, verificado y commiteado

## Impacto mapeado (Regla 0)

- **Leído completo:** `dev-tools/git-commit-isolated.ps1` (el bloque del índice
  privado, `git add -- @Paths`, y la verificación final), `dev-tools/tests/git-commit-isolated.tests.ps1`,
  `docs/dev/workflow/RULES.md` §11, las filas `FIND-792/793/795/796` del Backlog,
  y `docs/dev/avance/activo/operaciones.md` §`FIND-753`/`FIND-778`/`FIND-780`.
- **Referencias hacia dentro:** `RULES.md` §11 (norma), `docs/dev/plans/2026-11-26-batch-parciales-y-nuevas.md:67`
  (forma de invocación), `.opencode/task-system/prompts/pipeline-full.md` §Cierre.
  Ninguna se rompe: la forma documentada (`-ExpectAbsent '<path>::| \`<ID>\` |'`)
  es exactamente la declaración que el guard nuevo exige.
- **Referencias hacia fuera:** `git diff`/`rev-parse`/`cat-file`, y `$Paths`.
- **Veredicto:** el guard es aditivo y específico de `docs/dev/Backlog.md`. Un
  commit normal de código no lo toca (pre-mortem d: no se bloquea el repo).

## Diagnóstico (medido tres veces)

Con **cuatro carriles vivos** escribiendo el mismo Backlog:

| Commit | Se llevó | Estado medido |
|---|---|---|
| `6e4119c8d` | `FIND-775` | fila en `HEAD`, código en el worktree |
| `d15564b06` | `FIND-788` | `.github/workflows/certification.yml` seguía `M` |
| `a476e190c` | `FIND-774` | árbol de `.opencode/` sin commitear |

`FIND-788` aparecía **0 veces en `HEAD`** y **0 en el árbol**. **El helper no
podía detectarlo por construcción:** su verificación (`-ExpectAbsent`) comprueba
que **su** fila desapareció — y desapareció. No hay forma de saber, mirando solo
su fragmento, que en el mismo fichero viajaba la retirada de otro. Es la cara
opuesta del incidente del árbol vacío: antes el commit se llevaba **trabajo
ajeno** (recuperable desde el worktree); ahora se lleva **cierres ajenos**, y el
backlog miente en la dirección peligrosa — dice hecho lo que no está en la
historia. Si ese carril muere, queda una fila cerrada sin fix y la fuente de
verdad declarada (frontmatter + git) coincide consigo misma.

## El diseño: una pieza, no cuatro parches

Las cuatro filas (`FIND-792/793/795/796`) no son cuatro defectos independientes:
son **un** problema de verificación, y se arreglan en una sola pieza porque
comparten el mismo camino. La pieza tiene tres capas y cada fila cae en una:

```
                    ┌─ PARSEO ──────────────────────────────┐
   -Expect* ───────►│ array = 1 spec por elemento            │  FIND-795
                    │ comas: partir solo si TODAS son specs  │
                    └──────────────┬─────────────────────────┘
                                   ▼
                    ┌─ MEDICIÓN ────────────────────────────┐
                    │ blob del commit (cat-file + UTF-8)     │  FIND-793
                    │ NFC en ambos lados                     │
                    │ 0 / 1 / >1 coincidencias · línea       │
                    └──────────────┬─────────────────────────┘
                                   ▼
                    ┌─ DIAGNÓSTICO ─────────────────────────┐
                    │ (i) ausente        → 1                 │  FIND-796
                    │ (ii) ambiguo       → 7  (commit OK)    │
                    │ (iii) path ausente → dicho tal cual    │
                    │ encoding/transporte→ 8  (commit OK)    │
                    └───────────────────────────────────────┘
                    ┌─ ALCANCE DEL COMMIT ──────────────────┐
                    │ filas de Backlog retiradas vs          │  FIND-792
                    │ declaradas → abortar 9 (nada creado)   │
                    └───────────────────────────────────────┘
```

Por qué una pieza y no cuatro arreglos sueltos:

1. **El parcheo independiente deja agujeros entre parches.** Arreglar la coma sin
   arreglar la lectura del blob habría seguido verificando texto de consola;
   arreglar el blob sin arreglar la coma habría seguido verificando un fragmento.
   La medición es la misma para los cuatro: cuántas coincidencias hay en el blob
   y en qué línea.
2. **Los códigos de salida tienen que ser coherentes entre sí** (`1` contenido ·
   `7` ambiguo · `8` encoding · `9` ajeno): un orquestador decide con uno solo.
   Cuatro parches producirían cuatro escalas incompatibles.
3. **El guard de filas ajenas necesita la misma declaración que la verificación.**
   "Qué filas son mías" se deriva de las expectativas: sin un parseo fiable
   (`FIND-795`) el guard contaría IDs inventados por una coma.
4. **Un solo punto de lectura.** Todo pasa por `Get-BlobText`; ningún otro camino
   puede reintroducir una lectura por consola sin que se note.

## Lo que se implementó

- `Get-BlobText(Rev, Path)` — oid con `git rev-parse`, volcado con
  `git cat-file blob` **redirigido a fichero** y decodificado UTF-8 por .NET.
- `Get-BacklogRowIds(Text)` — filas `| \`ID\` |` por regex `(?m)^\|\s*\`(?<id>[A-Za-z][A-Za-z0-9]*-\d+)\`\s*\|`.
- `Get-DeclaredIds(Specs)` — IDs nombrados por cualquier expectativa: esa es la
  declaración "esta fila es mía".
- Guard **antes** de `commit-tree` (nada que restaurar): lee el blob de `HEAD` y
  el del árbol preparado, resta IDs y aborta si alguna fila retirada no está
  declarada. `-AllowForeignRows` es la salida explícita para una limpieza
  deliberada de varias filas.
- Desvío del título/exit code del bloque de rechazo existente (que ya sabía
  restaurar `HEAD` si un hook lo movió): `4` para los guards de `FIND-778`,
  `9` para filas ajenas. **No se duplicó el camino de restauración.**

**Precisión (pre-mortem d):** el guard se evalúa si el **conjunto staged del índice privado** contiene `docs/dev/Backlog.md` — nunca sobre los argumentos: la ronda 1 del review midió que `-Paths docs/dev` stagea el Backlog sin deletrearlo y se saltaba el guard entero, con `OK` al final — **y** el árbol preparado retira filas de ese fichero. Una fila cuyo ID sigue apareciendo en el blob nuevo está **reformateada o movida**, no retirada: `| `FIND-102` (blocked) |` ya no acusa a nadie. Un commit de código, un `git rm` o un Backlog que solo **añade** filas no se tocan.

## Verificación

- Escenario 19 (nuevo): Backlog con `FIND-101`/`FIND-102`, copia de trabajo sin
  ninguna de las dos, `-ExpectAbsent` solo de `FIND-101` →
  **exit 9**, `FOREIGN HUNK`, nombra **`FIND-102`**, `HEAD` no se movió, **cero**
  commits creados, `FIND-102` sigue en `HEAD:docs/dev/Backlog.md`.
- Contra el helper de `HEAD`: **exit 0**, el commit se lleva las dos retiradas →
  **6 FAILED** en ese escenario.
- Escenario 19b (precisión): retirar solo `FIND-101` y declararla → **exit 0**,
  `HEAD` avanza, `ok absent`. Verifica también la forma documentada
  (`docs/dev/Backlog.md::| \`FIND-101\` |`).
- Suite completa: **131/131 PASS** (helper nuevo) · **33 FAILED** (helper de
  `HEAD`, todos en 16-20; 1-15 verdes).
- Demostración en repositorio real (worktree temporal, no el checkout vivo):
  ver §Demostración.

## Demostración con el repo real

1. **Commit que PASA, con acento en la expectativa.** El cierre de esta tarea se
   commiteó con el helper nuevo, con `-ExpectPresent` construida desde code points
   (`[char]0x00E9`) para que el transporte no pudiera tocarla, y comparada contra
   el blob del commit. Salida: `ok present` + `OK: <sha> verified against the
   commit's blobs`. Ver `docs/dev/avance/activo/operaciones.md` §cierre.
2. **Commit abortado por fila ajena.** Ocurrió en el repo real, sin fabricarlo:
   con la copia de trabajo tal como estaba (otro carril había movido la fila
   `FIND-787`), un commit de `docs/dev/Backlog.md` que declaraba las cuatro filas
   propias abortó con **exit 9**, `FOREIGN HUNK`, `Undeclared: [FIND-787]`,
   `HEAD did not move: it is still 997c41c62…` y **cero** commits creados. El
   escenario 19 lo reproduce en un repo desechable. La retirada de las cuatro
   filas se cerró después con un **blob derivado de `HEAD`** (camino manual de
   §11): 24 borrados, 0 adiciones, `FIND-787` intacta — `20a629d5b`.

## Review P2-01 (una ronda, contexto fresco, para las cuatro filas)

- **Tier:** Fast (globs `dev-tools/**` y `docs/dev/**`; ningún path adversarial).
- **Gate mecánico:** `dev-tools/git-commit-isolated.tests.ps1` **131/131** +
  `node scripts/docs/check-docs.mjs` (GATING all clear) + `node scripts/docs/check-links.mjs`
  (0 broken) + `pwsh dev-tools/ocr-review.ps1` (spec generado; sin Critical/High).
  `dev-tools/verify.ps1` completo **no** se corrió: sus pasos son `cargo fmt/clippy/nextest`
  y esta sesión tiene prohibido compilar Rust (disco ~7,7 GB); ningún path de este
  diff es Rust, así que esos pasos no podían cambiar el resultado. **Declarado.**
- **Veredicto de la ronda 1: `changes-required`** — 1 blocker, 2 high, 1 medium,
  1 low. Los cinco se aplicaron con su medición y cada uno dejó un test que **falla
  contra el helper revisado** (**11 FAILED** en `18b`/`19c`/`19d` + `17d`):

| # | Sev | Hallazgo medido | Arreglo |
|---|-----|-----------------|---------|
| 1 | **blocker** | El guard se disparaba con `$Paths -contains $BACKLOG_PATH`: `-Paths docs/dev` (o `docs\dev\Backlog.md`) **stagea** el Backlog sin deletrearlo → exit 0 y el commit se llevaba la fila ajena, con `OK` al final | key sobre el **conjunto staged** del índice privado (`git diff --cached --name-only`), nunca sobre los argumentos |
| 2 | high | El test "pre-existente" comparaba por **número de línea**: borrar una línea desplaza las demás y el helper imprimía `this commit CHANGED that line` sobre una línea que no tocó (la clase `FIND-796` otra vez) | preguntar si el **fragmento** está en el blob del padre, no si lo está en el mismo índice |
| 3 | high | El guard reconocía solo la forma exacta de fila: un **reformateo** (`` | `FIND-102` (blocked) | ``) contaba como retirada y acusaba a otro carril de una fila que nadie retiró | descartar el ID si sigue **apareciendo en cualquier sitio** del blob nuevo (reformateo o *move*) |
| 4 | medium | La clase `ENCODING` afirmaba "the expectation's bytes were corrupted on the command line" — causa **no medida**: un blob con cualquier otro acento en cualquier línea convertía un fallo de contenido (1) en 8 | el mensaje del esqueleto ya no elige causa: enuncia las **dos** hipótesis (transporte / otra grafía) y nombra la línea; la rama del *doble-encoding* sí afirma, porque reproduce el round-trip |
| 5 | low | `RULES.md` §11 listaba `0,1,2,4,6,7,8,9` (faltaban 3 y 5) y la precedencia declarada en `Set-FailClass` no era la del código | 3 y 5 en la regla; precedencia por **rango** (1 < 7 < 8), no "last wins" |

- **Lo que el reviewer confirmó midiendo, no leyendo:** `FIND-792` (`6e4119c8d`
  retiró la fila `FIND-775` y el fix de `FIND-775` no está en ese commit);
  `FIND-793` (`[Console]::OutputEncoding` = **ibm437** en esta máquina: el mismo
  blob da 161140 chars por tubería y 163811 por fichero, y `después` está en el
  blob y no en el pipe); `FIND-795/796` (el helper de `HEAD` partía por comas e
  imprimía `REVERTED … did not carry your edit` sobre contenido presente).
- **Lo que también objetó y no se cambió (con razón declarada):** `-ExpectBlob`
  no declara IDs de fila, así que un commit de Backlog "pinned por blob" aborta
  con exit 9 si retira su propia fila. Es deliberado: hashear la copia de trabajo
  arrastra la escritura ajena, así que el blob **no** prueba qué filas son tuyas;
  la declaración sigue siendo `-ExpectAbsent '<path>::| `ID` |'`. Queda escrito
  en la cabecera de `-ExpectBlob`.
- **Segunda ronda:** no se pidió (la tarea pedía una). El gate de cierre es el
  mecánico + los **11 FAILED** contra el helper revisado, que es la misma prueba
  que la ronda 1 exigía.
## Deuda / techo declarado

- El guard cuenta **retiradas** de filas de Backlog, no cualquier hunk ajeno: una
  fila ajena **modificada** (mismo ID) no se detecta. El contrato de la fila pedía
  exactamente el conteo de filas retiradas, y ampliarlo a "cualquier hunk no
  declarado" bloquearía commits normales.
- El ID se reconoce con forma `PREFIJO-\d+` (la taxonomía del repo). Una fila con
  un ID sin dígitos (`FIND-A`) no se cuenta — y no existe en el Backlog.
- La comprobación cruzada "toda fila ausente de `HEAD` tiene su arreglo en `HEAD`"
  que la fila sugería como alternativa **no** se implementó: es un gate de estado
  del backlog, no del commit, y su sitio natural es `dev-tools/check-closure.ps1`
  (prohibido en esta tarea). Queda como fila aparte si el owner la quiere.
