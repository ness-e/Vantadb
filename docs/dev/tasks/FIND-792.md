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

**Precisión (pre-mortem d):** el guard solo se evalúa si `docs/dev/Backlog.md`
está en `-Paths` **y** el árbol preparado cambia filas de ese fichero. Un commit
de código, un `git rm` legítimo o un Backlog que solo **añade** filas no se
tocan. Escenario 19b lo prueba: retirar la fila propia sigue commiteando.

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
- Suite completa: **120/120 PASS** (helper nuevo) · **27 FAILED** (helper de
  `HEAD`, todos en 16-20; 1-15 verdes).
- Demostración en repositorio real (worktree temporal, no el checkout vivo):
  ver §Demostración.

## Demostración con el repo real

1. **Commit que PASA, con acento en la expectativa.** El cierre de esta tarea se
   commiteó con el helper nuevo, con `-ExpectPresent` construida desde code points
   (`[char]0x00E9`) para que el transporte no pudiera tocarla, y comparada contra
   el blob del commit. Salida: `ok present` + `OK: <sha> verified against the
   commit's blobs`. Ver `docs/dev/avance/activo/operaciones.md` §cierre.
2. **Commit abortado por fila ajena.** En una copia temporal del repo se retiró
   una fila de Backlog que no pertenecía al commit y se invocó el helper con la
   fila propia declarada: **exit 9**, `FOREIGN HUNK`, la fila ajena nombrada,
   `HEAD` sin mover y **cero** commits creados. Salida transcripta en la entrada
   de `operaciones.md`.

## Review P2-01 (una ronda, para las cuatro filas)

- **Tier:** Fast (globs `dev-tools/**` y `docs/dev/**`; ningún path adversarial).
- **Gate mecánico:** `dev-tools/git-commit-isolated.tests.ps1` **120/120** +
  `pwsh dev-tools/ocr-review.ps1` sobre el diff + checks de docs. `dev-tools/verify.ps1`
  completo **no** se corrió: sus pasos son `cargo fmt/clippy/nextest` y esta
  sesión tiene prohibido compilar Rust (disco ~7,7 GB); ningún path de este diff
  es Rust, así que esos pasos no podían cambiar el resultado. **Declarado, no
  omitido en silencio.**
- **Reviewer distinto del autor:** `vanta-review` (contexto fresco) sobre el diff
  de las cuatro filas + el test. Veredicto y hallazgos aplicados: §Review de
  `docs/dev/tasks/FIND-796.md` si la ronda cambia algo; si no, **approve** con el
  gate mecánico como evidencia.

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
