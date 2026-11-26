---
title: "Harness hardening — auditoría adversarial del sistema de ejecución de tareas y planes"
kind: review
status: active
description: "Auditoría de .opencode/ tras una sesión de 166 commits: 15 hallazgos (6 críticos) con ruta:línea, 9 propuestas de blindaje con contrato verificable y 4 correcciones a la evidencia de partida. Causa raíz común: el formato real del plan no es el que el parser lee, y ninguna premisa numérica se re-mide al cerrar."
tags: [harness, audit, pipeline, campaign-mcp, git-concurrency, reviews]
---

# Harness hardening — auditoría adversarial del sistema de ejecución de tareas y planes

**Veredicto: 🔴 CAMBIOS REQUERIDOS.**

El harness no se rompió por falta de reglas. Tiene 57 KB de `AGENTS.md`, 12 reglas
normativas, gates HITL, un MCP de 2 359 líneas y un hook de pre-push de 374 líneas.
Se rompió porque **tres de sus contratos más cargados son ficción ejecutable**:

1. El parser del task-system **no lee el formato de plan que el harness produce**
   (medido: 0 tareas parseadas de 35). Todo el camino `campaign_update_task_state`
   / `campaign_get_next_task` es un no-op silencioso.
2. El cierre de tarea **no re-mide ninguna afirmación**, así que una premisa falsa
   entra al plan, se hereda por copia entre filas y se cierra "verificada".
3. El commit concurrente **sigue siendo un `git add && git commit` desnudo en el
   prompt canónico**, aunque existen 28 lecciones acumuladas y un helper que
   funciona y que ningún prompt referencia.

## Alcance y método

- Leído completo: `commands/pipeline.md`, `commands/harness.md`,
  `prompts/plan.md`, `prompts/task.md`, `prompts/pipeline-run.md`,
  `prompts/pipeline-full.md`, `prompts/findings.md`,
  `prompts/question-gates.md`, `mcp/parsers.mjs`,
  `mcp/campaign-server.mjs` (zonas citadas), `dev-tools/git-commit-isolated.ps1`,
  `.githooks/pre-push`, `dev-tools/fmt-range.ps1` (cabecera).
- Medido con comando (no inferido): parseo real de los 3 planes vivos, conteo de
  commits de la sesión, referencias cruzadas al helper de commit, distribución de
  `**Estado:**` en 111 task files, cruce Backlog ↔ task files, conteo de
  `.grep` sobre prompts.
- **No medido** (declarado como tal, Regla 11): 17 m 28 s de
  `Blocking waiting for file lock`; "0 bytes" de disco ×2; "8 intentos" de
  `index.lock`; "16 de ~20 defectos cazados por review fresco"; los tamaños de
  `incremental` de ~7 GB por carril. No hay log retenido de esos eventos.

## Correcciones a la evidencia de partida

Auditar el harness obliga a auditar la evidencia con la que se lo acusa. Cuatro
precisiones, dos de ellas a favor del harness:

| # | Evidencia recibida | Estado tras verificación |
|---|---|---|
| E-1 | "149 commits en la sesión" | **Confirmada, no exagerada.** `git log --since="2026-11-25" --until="2026-11-27"` → **166** commits. |
| E-2 | "(c) `pre-push` medía el working tree en vez del rango" | **Ya arreglado, no es deuda viva.** `.githooks/pre-push:2` ("Juzga EL RANGO QUE VIAJA"), `:105-140` (diff del rango, fail-closed si no hay base), `:151-182` (fmt solo de los `.rs` del rango) y `dev-tools/fmt-range.ps1:44-60` (lee el toolchain del job `fmt` de `ci-rust.yml`). El eje 3 está implementado; **no hay que tocarlo**. |
| E-3 | "(3) `campaign_get_next_task` → *Ambiguous active plan* cuando había 2 planes; hoy unificado a 1" | **La unificación fue cosmética.** Hoy hay **3** planes con mtime < 24 h y `findPlanFile` ordena por mtime e **ignora el frontmatter `status:`** (`campaign-server.mjs:32-49`). Solo 1 de los 3 dice `status: active`. El próximo `touch` de un plan `superseded` reabre la ambigüedad. |
| E-4 | "(1) los estados de los planes mienten: un plan marcaba 11 ✅" | **Subestimado.** Los dos planes vivos tienen **0** marcas de estado (`✅ COMPLETED` = 0, `⬜ PENDING` = 0) porque **la tabla del plan no tiene columna de estado**. No es que el estado esté mal: no existe. |

---

# A. Auditoría por modo de fallo

## A.1 — Premisas falsas en las filas (11, cuatro del orquestador)

**Dónde vive la causa.** `prompts/plan.md:21-76` (§Paso 0 — Verificación de Realidad)
es el único punto del harness que dice "verificá la tarea contra el código". Pero
lo que verifica es **existencia, no verdad**:

- `plan.md:50-58` — la pregunta es "¿existe el archivo/símbolo?", "¿el
  comportamiento ya está implementado?". Ninguna de las 4 preguntas admite un
  **número** como objeto de verificación.
- `plan.md:67` — cláusula de escape: *"No se puede verificar sin investigación
  → registra en `Notas` y aplica DO si impacto justifica"*. Un claim cuantitativo
  ("92 unwraps en prod", "~21 sitios", "ningún job required") es exactamente
  "no verificable sin investigación" y por esta línea **entra igual**.
- `plan.md:69-74` y la plantilla `plan.md:272` piden la evidencia **en prosa**
  (`"Verificación real: ✅ CÓDIGO-REAL — símbolo X existe"`). Prosa no es
  re-ejecutable: nadie puede volver a correrla para ver si sigue siendo cierta.
- `prompts/findings.md:31` — el template de fila `FIND-*` **no tiene campo de
  medición**. Un hallazgo nace como afirmación de texto y así se copia.

**Prueba de que el mecanismo correcto existe pero no está cableado.**
`docs/dev/tasks/FIND-152.md:4` dice literalmente *"El ticket medía 43+49 `unwrap`
como 'contexto prod'. **Medido: los 92 son test-only, 0 en producción**"*. La
medición apareció **en el task file, al ejecutar**, no en la fila del plan. Entre
uno y otro pasaron un triage y un plan que aceptaron el número sin medirlo.

**Y el defecto se reproduce dentro del propio harness.**
`commands/harness.md:71-74` sigue afirmando el baseline de 2026-09-24:
*"semgrep 1448 unwrap (`~39 en contexto prod`…)"*. La fila que nació de ese
baseline (`FIND-152`) probó que son **0 en prod**. El documento que audita el
harness conserva el número refutado.

**Mecanismo de propagación (el que hay que cortar).** Una cifra sin fuente se
copia de una fila a la siguiente y **nadie la re-mide al cerrar**. No hay, en
ningún prompt, un paso de "repetí la medición que originó esta fila".

## A.2 — Árbol e índice de git compartidos (5 variantes del mismo fallo)

**Dónde vive la causa: en el prompt canónico de cierre.**

`prompts/pipeline-full.md:172` es la instrucción de commit de todo el sistema:

> `git add <solo los archivos tocados en esta tarea> && git commit -m "feat: <ID> — <name>"`

Ese `git add` + `git commit` desnudos sobre `.git/index` **son** las 5 variantes
del incidente. `git add <paths>` escribe en el índice compartido; `git commit` sin
pathspec commitea **lo que el índice tenga** en ese instante. Con dos carriles,
eso es una carrera, y está documentado 28 veces:

- `task-system/memory/lessons.md:143, 165, 167, 175, 366, 411, 421, 480, 482, 509, 511, 621, 625, 635, 661, 664, 674` (17 entradas)
- `memory/archive/lessons-archive-2026-09-24.md:327, 329, 337, 357, 388, 414, 416, 447, 476, 498` (10)
- `memory/archive/lessons-archive-2026-09-05.md:13` (1)

Cada entrada propone una mitigación distinta — `git commit -- <paths>`
(`:511`), `git diff --cached --stat` antes del commit (`:337`), `hash-object` vs
`rev-parse HEAD:path` (`:674`), worktree por instancia (`:329`) — y **ninguna
llegó a `pipeline-full.md`**.

**Lo que sí funcionó no está cableado.** `dev-tools/git-commit-isolated.ps1`
(155 líneas: índice privado vía `GIT_INDEX_FILE` + `read-tree` + `add` +
verificación **por contenido** post-commit, y falla cerrado ante una expectativa
malformada, `:133`). Medido con `Select-String -List` sobre el repo completo:

| Ámbito | Referencias al helper |
|---|---|
| `.opencode/task-system/prompts`, `.opencode/agents`, `.opencode/commands` | **0** |
| `.opencode/` completo | **1** — `memory/lessons.md:678` |
| Host (`docs/dev/**`, `dev-tools/**`) | **10** — incluye `docs/dev/workflow/RULES.md` |

El matiz importa y agrava el diagnóstico: el helper **está bien documentado**.
`docs/dev/workflow/RULES.md:252-276` lo explica completo, con su comando de uso y
su techo declarado:

> *"El índice privado **reduce la ventana** de carrera a `add`+`commit`
> (milisegundos) y **detecta** una reversión; **no la previene**. La prevención
> necesita un lock de wave, que no existe."*

Es decir: hay un runbook correcto, con límites honestos, en un fichero que
**ningún prompt del harness menciona** (0 hits de `workflow/RULES` en
`prompts/`, `agents/` y `commands/`). El `git add && git commit` de
`pipeline-full.md:172` contradice al runbook del propio repo y nada detecta la
contradicción. Es el patrón exacto que este informe audita: **conocimiento
correcto, consumidor no cableado, cero detector de drift.**

**Contradicción de autoridad (agrava todo lo anterior).** Cuatro documentos
discrepan sobre **quién** commitea:

| Fuente | Qué dice |
|---|---|
| `AGENTS.md:313` | "**Solo vanta-lead** toca git mutating (`git commit`, `git push`…)" |
| `agents/vanta-worker.md:4-7` | *"Never use me for commits/releases (**the lead commits**)"* |
| `agents/vanta-worker.md:17` | `shell: allow` — sin enforcement; el "git solo lectura" es un **comentario**, no un permiso |
| `prompts/pipeline-full.md:172` | el sub-agente ejecutor **hace el commit** |
| `AGENTS.md:641` | "`pipeline-full.md` §Cierre produce commit local" |

El resultado es el peor de los escenarios: la norma dice "solo el lead", el
prompt dice "commiteá vos", el permiso dice "podés", y **166 commits en 24 h
demuestran que gana el prompt**. Ningún gate puede proteger un índice compartido
si no hay un único committer determinista.

Además `AGENTS.md:195` ("Flujo típico: `git add .` → …") enseña el antipatrón
exacto que `lessons.md:366` describe como causa de un incidente.

**Residual del helper.** `git-commit-isolated.ps1:126-153` verifica **solo HEAD**.
`lessons.md:674` demostró que eso no alcanza: el *worktree* puede quedar con una
copia stale **después** del commit (13 líneas perdidas en `Backlog.md` con HEAD
correcto). El chequeo correcto es `git hash-object <file>` vs
`git rev-parse HEAD:<file>`.

## A.3 — Campaign MCP / task system roto

**Dónde vive la causa: el formato del plan y el parser divergieron, y nada valida el contrato.**

Medición decisiva (ejecutada con `parseTasks` real sobre los planes reales):

| Plan | Filas de tabla | `parseTasks()` | Marcas de estado |
|---|---|---|---|
| `2026-11-26-batch-parciales-y-nuevas.md` | **35** | **0** | 0 |
| `2026-11-23-release-v0.10.0-gates.md` | **8** | **0** | 0 |
| `backlog-release-v0.10.0.md` | 0 | 0 | 0 |

- `mcp/parsers.mjs:25` — `content.split(/\n(?=### Task \d+)/)`: el parser **solo**
  entiende encabezados `### Task N:`.
- `mcp/parsers.mjs:32` — el ID sale del encabezado.
- `mcp/parsers.mjs:135` — `findTaskById` exige la misma forma `### Task …`.
- `mcp/parsers.mjs:11-21` — `extractState` busca `- **Estado:**` y clasifica por
  emoji; cualquier otra forma cae a `⬜ PENDING`.
- `campaign-server.mjs:986` — el no-op se reporta como
  `"Task … not found or no changes needed"`. Es exactamente el `Task not found`
  de los ~21 task files de la sesión.
- `campaign-server.mjs:32-49` — `findPlanFile` ordena por **mtime** e ignora el
  frontmatter `status:`; con ≥2 planes < 24 h lanza `Ambiguous active plan`
  (`:44-47`).

**Por qué nadie lo vio.** `mcp/parsers.test.mjs` prueba **solo fixtures
sintéticos** con la forma `### Task N:` (`:18, :27, :37`). Ningún test abre un
plan real. El drift es invisible por construcción.

**Por qué el drift es previsible.** La plantilla canónica
(`prompts/plan.md:245-298`) sigue escribiendo `### Task 1: ID — Descripción`,
mientras los planes reales —confrontados con `documentation-skill` y su
frontmatter— son **tablas** `| # | ID | Wave | Carril | Ficheros / zona exclusiva |`.
Dos formatos, un parser, cero gate.

**Consecuencia estructural (esto es lo importante).** Como la tabla del plan **no
tiene columna de estado**, el plan nunca fue una fuente de estado: el estado real
vive en `Backlog.md`, en los task files y en los commits. El harness finge que el
plan es stateful y el MCP finge persistirlo. Ambos fingen hacia el vacío.

## A.4 — Paralelización: cupos y cuellos

**Dónde vive la causa.** `pipeline-run.md:239` y `commands/pipeline.md:196`:

> `MAX_CONCURRENT = min(3, tareas_en_wave)  # 3 por RAM en Windows`

Un solo número, justificado por **un** recurso (RAM), y el recurso medido **no fue
RAM**: fue el **índice de git** (A.2) y, en segundo orden, el artefacto compartido
de `cargo` (`Blocking waiting for file lock on artifact directory`, no medido por
mí) y el disco (0 bytes, no medido por mí).

El plan **ya declara** la exclusividad que falta:

```
| # | ID | Wave | Carril | Ficheros / zona exclusiva | Prerequisito declarado |
```

y el plan del 2026-11-26 incluso documenta las restricciones a mano, en prosa
(`2026-11-26-batch-parciales-y-nuevas.md`, bloque "Restricciones operativas
aprendidas": 2 carriles Rust, 1 carril por fichero de workflow, 1 carril en
`src/**`). **Ningún prompt consume esa columna:**

- `pipeline-run.md:106-108` (DETALLE OBLIGATORIO, bloque 2) pide "ARCHIVOS:
  clave" — no la zona exclusiva.
- `pipeline-run.md:231-232` detecta conflictos **ad hoc** con
  `codegraph_codegraph_explore` ("si dos tareas tocan archivos diferentes"), es
  decir, re-deriva por tarea lo que el plan ya dice por fila.

Es el patrón del bloque 1 otra vez: el dato está en el artefacto y no en el
consumidor.

## A.5 — Reviews adversariales en contexto fresco (el mejor mecanismo)

**Funciona. No rediseñar.** `task.md:137-170` (Fase 5), `pipeline-full.md:163-171`
(review risk-tiered por globs) y el ACCEPT mecanizado HARD-07
(`config/state-tools.mjs` → `campaign-server.mjs:941`, `reviewBlocked` real,
probado en `review-gate.test.mjs:91-99`). Es el único gate de la sesión con
evidencia de efectividad (16/~20 defectos reales), y funciona **porque** el
revisor no comparte contexto con el autor.

**Único hueco.** El alcance del review es *enfoque / cómo se probó / OCR /
tiering* (`task.md:149-164`). **La premisa no es objeto de review.** Por eso
`FIND-118` ("remover alias legacy" cuando el alias *es* la implementación) pasó
un review adversarial sin problema: se revisó la calidad del trabajo, no si el
trabajo era el correcto.

## A.6 — Autoridad del usuario vs del orquestador

**Dónde vive la causa: el prompt le da al orquestador autoridad de descarte sobre un lote que el usuario definió, y lo empuja a usarla.**

- `plan.md:118-123` (Reglas del gate): "Esfuerzo >> impacto → DEFER o SKIP",
  "Cosmético sin queja de usuario → DEFER".
- `plan.md:125-135` (Checklist Shape Up): "¿Es AHORA?… Un problema real pero sin
  urgencia es DEFER". Es una instrucción **a diferir**, no a preguntar.
- `question-gates.md:38` — las opciones que se le ofrecen al usuario para una
  tarea 🔴 incluyen `DEFER / SKIP` como si fueran equivalentes a "ajustar alcance".
- `question-gates.md:129-134` (non-stop/run): "D no-crítico no pregunta", "C
  auto+log". En `/pipeline run` el descarte ocurre **sin** la pregunta.
- `question-gates.md:42` exige confirmar el set DEFER/SKIP con el usuario, pero
  es una pregunta blanda, no una precondición mecánica.

**Prueba de que la cláusula falta en la plantilla y no en las personas.** El plan
del 2026-11-26 tuvo que escribirla a mano:

> **Regla del lote:** **todas las tareas se ejecutan y se quedan en este plan.**
> No hay `SKIP` ni `DEFER` por criterio del orquestador.

Eso es un campo de plantilla pidiendo existir. Un humano lo escribió una vez; el
próximo plan lo va a olvidar.

## A.7 — Cierre de tarea y deuda de bookkeeping

**Dónde vive la causa: el cierre tiene cinco artefactos, tres dueños y ningún verificador.**

Artefactos que un cierre correcto debe dejar consistentes: (1) fila fuera del
Backlog, (2) entrada en `docs/dev/avance/<dominio>.md`, (3) estado en el plan,
(4) `**Estado:**` en el task file, (5) commit local verificado.

- `pipeline-full.md:172-181` ordena: verify → commit → learnings →
  `update_task_state` → `skill progreso`.
- `progreso/SKILL.md:149-151` — *"Standalone: **no commit** — esperar
  instrucción"*; *"Desde campaign-executor: el executor maneja commits"*. El
  dueño del paso 1-2 delega explícitamente el paso 5 en otro actor.
- `progreso/SKILL.md:133` — *"Task files (`tasks/<ID>.md`) no se archivan —
  **quedan STALE** tras archivar el [plan]"*. **La deuda está documentada como
  comportamiento esperado.**

**Medición del resultado** (111 task files en `docs/dev/tasks/`):

| Hecho | Número |
|---|---|
| Task files con campo `**Estado:**` | **52 / 111** (59 sin campo) |
| Valores distintos de estado observados | 4 formas **no canónicas**: `Completado (2026-11-25)`, `RESUELTO — …`, `IN PROGRESS` (sin emoji), `❓ DISEÑO LISTO PARA IMPLEMENTACION` |
| Task files en estado no terminal | **4** (`FIND-585`, `FIND-587`, `FIND-640`, `MEMG-27`) |
| Filas ya retiradas del Backlog con task file aún `IN PROGRESS` | **3** (`FIND-721`, `FIND-723`, `FIND-726`; `FIND-738` sigue además en Backlog) |

Y el bug del `Estado` libre no es cosmético: `parsers.mjs:16-21` clasificaría
`"Completado (2026-11-25)"` y `"RESUELTO — …"` como **`⬜ PENDING`**. Un task
cerrado se reporta como pendiente.

---

# B. Plan de blindaje priorizado

Criterio de orden: **(impacto ÷ esfuerzo) × reutilización**. Cada propuesta trae
un contrato que **falla hoy**, porque un gate que no puede fallar no es un gate.

## H-01 🔴 — El parser lee el formato real del plan, y el estado es un token obligatorio

- **Previene:** el bloque entero 3 (A.3): `Task not found` ×21, `Ambiguous active
  plan`, y "los estados del plan mienten" (que hoy es peor: no existen).
- **Dónde:** `mcp/parsers.mjs:23-47` (aceptar filas de tabla además de
  `### Task N:`, sin borrar la forma vieja), `mcp/parsers.mjs:131-139`
  (`findTaskById` por ID de tabla), `campaign-server.mjs:32-49`
  (`findPlanFile`: filtrar por `status:` del frontmatter y usar mtime solo como
  desempate), `prompts/plan.md:245-298` (la plantilla adopta la tabla real + una
  columna `Estado`).
- **Contrato verificable** (hoy falla en los dos frentes):
  ```bash
  node --test .opencode/task-system/mcp/          # nuevo test con archivo real
  ```
  El test abre el plan activo de `docs/dev/plans/` y afirma
  `parseTasks(c).length === <filas de tabla>` (hoy: `0 !== 35`). Segundo test:
  directorio temporal con dos planes frescos, uno `status: superseded` →
  `findPlanFile` debe devolver el `active` (hoy lanza `Ambiguous`). Tercero:
  toda fila debe portar un token de la lista de 4 estados.
- **Esfuerzo:** 1.5-2 h. **Riesgo:** el parser es compartido con
  `findInProgressTasks` (`campaign-server.mjs:77-97`) y con
  `review-gate.test.mjs`; hay que mantener la forma `### Task N:` (los planes
  viejos no se migran) y correr los 3 suites del directorio.

## H-02 🔴 — Verificación de premisas: campo `Medición:` + re-medición al cierre

- **Previene:** las 11 premisas falsas (A.1): `FIND-98` (79/87 vs 84==84),
  `FIND-740` (1 lint → 87), `FIND-152` (92 "en prod" → 0), `FIND-755/761/763/719`
  ("ningún job required" → 3 lo eran), `FIND-765` (`web/` no trackeado),
  `FIND-769` ("~21 sitios" → 1), `FIND-771`, `FIND-756`.
- **Dónde:** `prompts/plan.md:69-76` (la evidencia deja de ser prosa) +
  `prompts/plan.md:272` (columna obligatoria cuando la fila trae un número o un
  universal) + `prompts/findings.md:31` (mismo campo en el ticket `FIND-*`) +
  `prompts/pipeline-full.md:144-153` (§Cierre: re-ejecutar la medición antes de
  marcar COMPLETED) + campo nuevo en el bloque `RESULTADO`
  (`pipeline-full.md:279-288`).
- **Diseño mínimo (una regla, no dos):** toda fila cuya prosa matchee
  `\d|\bningún\b|\btodos\b|\bsolo\b|\bnunca\b` **debe** portar
  `Medición: <comando> → <salida>`. Al cerrar, la tarea **re-ejecuta** esa
  medición; si el número cambió → `⚠️ PREMISA INVALIDADA` + fila `FIND-*` +
  el cierre no es COMPLETO. Sin esta parte, el campo es otra declaración de fe.
- **Contrato verificable:**
  ```bash
  node dev-tools/check-claims.mjs docs/dev/plans/$(ls -t docs/dev/plans/*.md | head -1)
  ```
  exit 1 listando `ID → claim sin Medición`. Sobre el plan actual **debe fallar**
  (hay filas con "87", "~21 sitios", "3 required"), así que se engancha
  *diff-scoped* como `dev-tools/floor-guard.ps1:20-26` (baseline de filas
  heredadas, igual que `.lurkr-baseline.json`).
- **Esfuerzo:** 45 min la regla + 30 min el enganche en §Cierre.
  **Riesgo:** un agente puede escribir un `Medición:` inventado. Mitigación
  barata y suficiente: el reviewer re-corre **una** medición por cada 5 tareas
  (ya hay checkpoint cada 5 en `pipeline-run.md:201-203`); un comando
  re-ejecutable convierte una mentira en un fallo detectable, que es todo lo que
  un gate puede prometer (mismo estándar que Regla 11).

## H-03 🔴 — Un solo camino de commit, y un gate que lo vigila

- **Previene:** las 5 variantes del índice compartido (A.2): copia stale horneada
  ×5 (una vez 11 líneas, exit 0), `.git/index.lock` ×8, WIP ajeno bloqueando el
  push de todos, reinserción de una fila cerrada, fichero vaciado sin commitear.
- **Dónde:** `prompts/pipeline-full.md:172` (la instrucción de commit pasa a ser
  el helper), `AGENTS.md:195` (`git add .` fuera), `AGENTS.md:313` +
  `agents/vanta-worker.md:4-7,17` (resolver la autoría del commit: **una** regla,
  no tres), `pipeline-run.md:158-166` (CHECKLIST PRE-SPAWN gana el ítem "el
  committer es único en esta wave").
- **Contrato verificable:**
  ```bash
  rg -n "git add .*&& git commit|git commit -m|git add -p" .opencode/task-system/prompts .opencode/agents .opencode/commands
  ```
  **debe devolver 0 líneas.** Medido hoy: **2** — `pipeline-full.md:172` y
  `iter-loop-tools.md:293`. (Nótese el detalle: el patrón angosto
  `git add .*&& git commit|git commit -m` da **1**, no 2 — `iter-loop-tools.md:293`
  redacta el commit como `git add -p + git commit` y se escapa. El gate tiene que
  matchear el commit desnudo, no una redacción particular; si no, nace
  fail-open.) Se agrega como check 6 de `vanta-harness §3`
  (`.opencode/agents/vanta-harness.md`) para que `/harness` lo ejecute.
  Contrato del helper ya existente: `git-commit-isolated.ps1` sale **1** cuando
  una expectativa de contenido no se cumple (`:149-153`) y **2** cuando falla el
  staging (`:94-101`) — probado en la sesión.
- **Esfuerzo:** 30-40 min (3 ediciones de prompt + 1 grep en el agente + 1
  reemplazo de ejemplo en `AGENTS.md`). **Riesgo:** el helper no sirve para
  merge commits ni `--amend` (documentado en su cabecera `:27-30`) → el texto
  debe decir explícitamente "si estás mergeando, no uses esto". **Techo
  declarado, no ocultarlo:** `docs/dev/workflow/RULES.md:276` ya lo dice — el
  índice privado *reduce* la ventana y *detecta* la reversión, **no la previene**;
  la prevención necesitaría un lock de wave que no existe. H-03 debe citar ese
  techo en el prompt para que nadie lo lea como una garantía.
  **Ampliación opcional (P2, +15 líneas):** comparar `git hash-object <file>` con
  `git rev-parse HEAD:<file>` para cubrir el residual de `lessons.md:674`, y
  agregar a `vanta-harness §3` un check de que `docs/dev/workflow/RULES.md` sigue
  siendo la fuente única que los prompts citan (hoy: 0 citas).

## H-04 🟠 — Cierre atómico: la consistencia de estado se verifica, no se confía

- **Previene:** A.7 — 4 task files no terminales, 3 filas retiradas con task file
  `IN PROGRESS`, 59 task files sin campo `Estado`.
- **Dónde:** `prompts/pipeline-full.md:172-181` (orden del cierre y
  responsabilidad única), `prompts/task.md:186` (el token de `Estado` es
  cerrado, no prosa), y el skill `progreso`
  (`~/.agents/skills/progreso/SKILL.md:133,149-151` — **fuera de `.opencode/`,
  lo aplica el lead**).
- **Contrato verificable:**
  ```bash
  pwsh dev-tools/check-closure.ps1 -Diff
  ```
  Falla si: (a) existe un task file con `Estado` no terminal cuyo ID **no** está
  en `docs/dev/Backlog.md` (hoy: `FIND-721`, `FIND-723`, `FIND-726`); (b) un
  `**Estado:**` no pertenece al set de 4 tokens (hoy: 4 formas no canónicas); (c) un ID
  cerrado en el diff no tiene entrada en `docs/dev/avance/`.
  **Diff-scoped** para no exigir arreglar 59 archivos heredados de golpe.
- **Esfuerzo:** 1 h. **Riesgo:** el gate puede volverse ruidoso si se corre sobre
  todo el árbol; `-Diff` (patrón `floor-guard.ps1`) lo evita.

## H-05 🟠 — "Lo que el usuario listó no se difiere" como campo de plantilla

- **Previene:** el `DEFER`/`SKIP` revertido por el usuario sobre un lote que él
  había definido (A.6).
- **Dónde:** `prompts/plan.md:245-252` (bloque de cabecera del plan file) +
  `question-gates.md:32-42` (Gate P reconoce el modo lote).
- **Diseño:** la cabecera del plan gana una línea obligatoria
  `> **Regla del lote:** <todas se ejecutan | triage libre>`. Si dice "todas",
  el triage **solo** puede clasificar `✅ DO` o un *prerequisito declarado*
  (credencial, autorización, dependencia externa) — nunca `DEFER`/`SKIP` sin una
  respuesta literal del usuario registrada.
- **Contrato verificable:** `node dev-tools/check-claims.mjs` (mismo script de
  H-02, segunda regla): si el plan declara "todas se ejecutan" y alguna fila
  tiene `Gate Result: 🟡 DEFER` o `❌ SKIP` sin `Pregunta:` con respuesta → exit 1.
- **Esfuerzo:** 20 min. **Riesgo:** casi nulo; es codificar lo que un humano ya
  escribió a mano en el plan del 2026-11-26.

## H-06 🟡 — Cupos por recurso en vez de un número mágico

- **Previene:** los dos carriles Rust con ~7 GB de `incremental` y disco a 0
  bytes; dos agentes en el mismo `.github/workflows/*.yml`; y el cuello real
  (índice de git) tratado como si fuera CPU.
- **Dónde:** `pipeline-run.md:239` y `commands/pipeline.md:196` (reemplazar
  `MAX_CONCURRENT = min(3, n)`), `pipeline-run.md:106-108` (el bloque 2 del
  DETALLE OBLIGATORIO pasa a incluir la **zona exclusiva** de la fila del plan),
  `pipeline-run.md:228-239` (el DAG consume la columna en vez de re-derivarla con
  codegraph).
- **Diseño:** la columna `Ficheros / zona exclusiva` que el plan **ya tiene** se
  interpreta como recurso exclusivo. Reglas: `≤2` carriles `rust-compiler`,
  `≤1` por path de workflow, `≤1` escritor de `src/**` en barrido, `≤1`
  committer por ventana (esto último lo resuelve H-03, no un cupo).
- **Contrato verificable:**
  ```bash
  node dev-tools/check-wave-conflicts.mjs docs/dev/plans/$(ls -t … | head -1)
  ```
  exit 1 si dos filas de la **misma** wave declaran el mismo recurso exclusivo.
  Función pura sobre la tabla → un test con el plan real como fixture.
- **Esfuerzo:** 2-3 h. **Riesgo:** es el eje con **peor ROI por hora**: cualquier
  scheduler en prompts lo puede ignorar el orquestador humano. Por eso se
  propone **solo el chequeo de conflictos de wave** (20 líneas), **no** un
  scheduler. El cupo de compilador y disco se documenta en prosa en la cabecera
  del plan (donde ya funciona) y se deja de fingir que un `MAX_CONCURRENT`
  uniforme lo cubre.
- **Detección del cuello real (lo que pedía el eje):** el síntoma que distingue
  "es el índice" de "es la CPU" es que el fallo aparece **en el commit o en el
  push**, no en el build (`Unable to create .git/index.lock`, revert de una fila
  ajena, `pre-push` bloqueado por WIP ajeno). Un contador de reintentos de commit
  por wave (una línea en el checklist de `pipeline-run.md:201-203`) lo hace
  visible sin instrumentación nueva.

## H-07 🟡 — El review también revisa la premisa

- **Previene:** `FIND-118` ("remover alias legacy" cuando el alias *es* la
  implementación) pasando un review adversarial impecable.
- **Dónde:** `prompts/task.md:149-164` (alcance del review: agregar ítem 0
  "**la premisa** — ¿la afirmación de la fila se sostiene contra el código? citá
  la medición") + `pipeline-full.md:279-288` (`RESULTADO` gana
  `PREMISA: <confirmada|refutada + evidencia>`).
- **Contrato verificable:** el orquestador rechaza `✅ COMPLETO` sin el campo
  `PREMISA:` (mismo parseo que ya hace con `GATES_EVALUADOS`,
  `pipeline-run.md:169-172`). Gate de una línea sobre un bloque que ya se parsea.
- **Esfuerzo:** 20 min. **Riesgo:** bajo — reutiliza el mecanismo de mayor
  efectividad medido (16/~20), sin agregar un actor nuevo.

## H-08 🔵 — Sincronizar el número refutado del propio harness

- **Previene:** que el documento de auditoría siga afirmando un baseline que su
  propia fila cerró (`commands/harness.md:71-74`: "~39 en contexto prod" vs
  `docs/dev/tasks/FIND-152.md:4`: "0 en producción").
- **Dónde:** `commands/harness.md:71-74`.
- **Contrato verificable:** el mismo `check-claims.mjs` de H-02 aplicado a
  `commands/harness.md` (un número con `~` y sin `Medición:` al lado → exit 1).
- **Esfuerzo:** 2 min de edición. **Riesgo:** nulo.

## H-09 🚫 — Canal directo entre sub-agentes: **no**

Argumento en contra, con la evidencia de esta sesión:

1. **El canal ya existe y es el correcto.** `RESULTADO` + `BLOQUEO:`
   (`pipeline-full.md:279-288`, `question-gates.md:26`) es un contrato tipado,
   parseable y auditable en el task file. Un canal directo sería un segundo
   transporte del mismo mensaje.
2. **El harness no sufre de falta de comunicación: sufre de tres fuentes de
   verdad para el estado** (A.7). Un canal directo agrega una **cuarta**.
3. **El mejor mecanismo medido funciona porque no hay canal.** El review P2-01
   caza defectos precisamente porque el revisor **no** habló con el autor. Darles
   un bus directo erosiona la propiedad que lo hace útil.
4. **Lo único que falta es declaración de propiedad**, y ya está en el plan
   (columna `Ficheros / zona exclusiva`) — el trabajo es que el consumidor la
   lea (H-06), no inventar mensajería.

Si en el futuro hace falta handoff asíncrono entre carriles, el buzón de fichero
(`docs/pipeline-state.json`, ya previsto en `pipeline.md:219-225`) es suficiente y
respeta la regla: **estado en ficheros versionados, nunca en memoria de proceso**.

## H-10 — Eje 3 (`pre-push` / gates locales vs CI): **ya está hecho, no tocar**

`pre-push:2,105-140,151-182` juzga el rango y falla cerrado; `fmt-range.ps1:44-60`
toma el instrumento (toolchain) del job `fmt` de `ci-rust.yml` **en vez de
hardcodearlo**, y degrada a ADVISORY nombrando el motivo (`:84`) en vez de
bloquear por "no tengo nightly". `dev-tools/gate-common.ps1:1-8` ya declara ser
la fuente única de features entre `verify.ps1`, `verify_changed.ps1` y
`audit-all.ps1`, y `SKIP=<gate>` **nombra** el gate saltado
(`pre-push:61-63`) y se ignora bajo `CI` (`:46-49`). La regla "un gate que se
salta en silencio no es un gate" está implementada.

**Único residuo, sin acción:** `AGENTS.md:523-545` re-enuncia el gate en prosa y
puede desincronizarse del hook. No vale un gate propio; se corrige cuando se
toque esa sección.

## Tabla de esfuerzo / impacto

| ID | Previene | Esfuerzo | Riesgo de la mejora | Bloquea a |
|---|---|---|---|---|
| **H-01** | A.3 completo | 2 h | medio (parser compartido) | H-02, H-05, H-06 |
| **H-02** | A.1 (11 premisas) | 1.25 h | bajo (baseline heredado) | H-08 |
| **H-03** | A.2 (5 variantes) | 40 min | bajo (merge/amend excluidos) | — |
| **H-04** | A.7 | 1 h | bajo (diff-scoped) | — |
| **H-05** | A.6 | 20 min | nulo | H-01 (mismo script) |
| **H-06** | A.4 | 2-3 h | alto (ignorable) | — |
| **H-07** | A.5 (hueco) | 20 min | bajo | — |
| **H-08** | número refutado | 2 min | nulo | — |
| **H-09** | — | 0 | — | **no hacer** |

---

# C. Lo que NO hay que tocar

Cuatro mecanismos funcionaron y rediseñarlos sería el error caro:

1. **Review P2-01 en contexto fresco** (`task.md:137-170`,
   `pipeline-full.md:163-171`) con ACCEPT mecanizado (`campaign-server.mjs:941`,
   `review-gate.test.mjs:91-99`). Es el gate con mejor relación defectos/coste de
   la sesión. **Solo se le agrega el ítem de premisa (H-07); no se rediseña.**
2. **`dev-tools/git-commit-isolated.ps1`** — índice privado + verificación por
   contenido + fallo cerrado ante expectativa malformada (`:133`). Es la
   solución correcta al problema correcto. **Se cablea (H-03), no se reescribe.**
3. **Advisory no bloqueante para lo que no es propiedad del commit**
   (`pre-push:203-225` frescura del CLI, `fmt-range.ps1:84` toolchain ausente,
   `pre-push:196-201` residuos de test). Distinguir "esto es del commit" de "esto
   es de tu máquina" es lo que evita que la gente use `--no-verify`.
4. **Verificación por contenido en vez de exit code** (`git-commit-isolated.ps1:126-153`,
   `pre-push:120-132` fail-closed, `:107-109` strip de CR). El diagnóstico
   central de la sesión — "the exit code proves nothing" — está bien interiorizado.

Y uno que merece respeto sin cambios: **`findings.md:11-16`** (todo hallazgo nace
como fila en el Backlog en el momento del discovery, no al cierre). La regla es
correcta; lo que falta es que la fila **nazca medida** (H-02).

---

---

# D. Estado de aplicación (2026-11-26, `vanta-harness`)

Los seis blindajes contratados (`H-02`..`H-07`) se aplicaron en el orden de
dependencia `H-03 → H-05 → H-07 → H-02 → H-04 → H-06`. Detalle y evidencia
`ruta:línea` por ítem: [H-02-H-07](../tasks/H-02-H-07.md).

| ID | Estado | Detector que lo sostiene (falla si el contrato se rompe) |
|---|---|---|
| H-03 | ✅ aplicado | `check-harness-contract.mjs` → `commit-path` (0 commits desnudos; ≥1 cita a `workflow/RULES.md` §11) |
| H-05 | ✅ aplicado | `check-harness-contract.mjs` → `plan-lote` (DEFER/SKIP sobre lote del usuario sin firma → exit 1) |
| H-07 | ✅ aplicado | `check-harness-contract.mjs` → `premise` (scope + `PREMISA:` + rechazo del orquestador) |
| H-02 | ✅ aplicado | `check-harness-contract.mjs` → `medicion` (markers + filas añadidas con claim sin `Medición:`) |
| H-04 | ✅ aplicado | `check-closure.ps1` (`-Diff` gate, `-All` auditoría, `-SelfTest`) |
| H-06 | ✅ aplicado | `check-harness-contract.mjs` → `wave-quotas` (sin `MAX_CONCURRENT=min(3`, conflictos de zona por wave) |

**Verificación (comandos y salida real, no auto-reporte)**

```
$ node dev-tools/check-harness-contract.mjs
ok commit-path (H-03) · ok plan-lote (H-05) · ok medicion (H-02) · ok premise (H-07) · ok wave-quotas (H-06)
harness contract holds (7 checks)                                   # exit 0
$ node dev-tools/check-harness-contract.mjs --self-test   → 31/31 fixtures (cada regla puede fallar)
$ node dev-tools/check-harness-contract.mjs --rows        → exit 0 (claims de filas nuevas)
$ pwsh dev-tools/check-closure.ps1 -SelfTest              → 13/13 (9 regex + 4 end-to-end en repo temporal)
$ pwsh dev-tools/check-closure.ps1 -Diff -Paths <mis rutas> → CLOSURE OK  # exit 0
$ pwsh dev-tools/check-closure.ps1 -All                   → 2 violaciones vivas: FIND-587 y H-02-H-07 (este task file, por el review pendiente)
```

**Corrección a un auto-reporte de la ronda 1:** se informó "`-All` → 1", y son
**2** (`FIND-587` **y** `H-02-H-07`). Además `-Diff` estaba verde *sobre ese
huérfano* porque no veía ficheros untracked — el defecto M3 del review, ya
corregido. `-Diff` sin scope es hoy la **auditoría del árbol** (sale el WIP de
otros carriles); el gate de un commit se corre con `-Paths`.

Antes de la aplicación, el mismo gate de contrato daba **13 violaciones** (una
por contrato ausente, más las citas a `workflow/RULES.md`, que eran **0** en
`prompts/`). Los dos gates quedaron cableados como destroyers de drift en
`agents/vanta-harness.md` §3, con la regla de que un exit ≠ 0 es 🔴 y no admite
un veredicto ✅.

**Fuera de alcance, declarado:** `H-01`/`FIND-774` (parser del task-system,
`task-system/mcp/**`) no se tocó — dos agentes cierran tareas con esa máquina;
cambiarla a mitad de wave es interferencia por comportamiento. El review P2-01
de esta aplicación queda a cargo del orquestador: la sesión que la aplicó no
tiene tool `task` (`subagent: deny`) y no puede spawnear `vanta-review`; por eso
el task file queda `⏳ IN PROGRESS` en lugar de `✅ COMPLETED` sin veredicto
(HARD-07 aplicado al propio cierre).

## D.1 — Ronda 2 del review P2-01 (3 Blocking, 6 Medium, 5 Low)

El review refutó la premisa de esta entrega con la razón correcta: **tres
detectores no miraban donde vive el defecto**. Corregido y probado
reintroduciendo cada violación con los scripts **sin modificar** (`--root` /
`-Root` sobre repos temporales): **8/8 sondas**.

| ID | Defecto real | Fix |
|---|---|---|
| **B1** | `H-02` juzgaba la **línea entera**: `#`, ID, `2h`, `P2`, `W10` son dígitos → toda fila nueva era "claim" y quedaba bloqueada | se juzgan **celdas de prosa** (`isProse` descarta metadatos, paths, fechas, tokens); spec alineada en `plan.md`/`findings.md` (Medición en la celda del claim) |
| **B2** | `git commit` desnudo en `.opencode/AGENTS.md:8` y el gate **no escaneaba** `AGENTS.md` (el primer flujo que lee todo agente) | `AGENTS.md` entra al scan + línea 8 pasa al helper |
| **B3** | `activePlans()` exigía `status: active` y la plantilla no emitía frontmatter → un plan creado por la plantilla no se escaneaba | **status ausente = activo** (sólo `superseded|archived|done|completed|closed` excluyen), top-level only, y la plantilla emite frontmatter |
| **M1** | `git diff` sin `HEAD` → gate ciego a cambios staged | `git diff -U0 HEAD` |
| **M2** | `H-02` sólo corría en `/harness`; una fila commiteada en una tarea normal nunca entraba al diff | modo `--rows` + `findings.md` lo exige al crear la fila |
| **M3** | `check-closure -Diff` no veía untracked y su `$Base` real era `HEAD` | default `origin/develop` (fallback `HEAD~1`, resuelto y reportado) + unión de `git ls-files --others` |
| **M4** | `Contains($id)` enmascaraba IDs prefijo (`FIND-90` oculto por `FIND-901`) | `Test-IdPresent` con límite de palabra |
| **M5** | `F4` **no estaba resuelto**: 6 sitios seguían diciendo que commitea el lead | 6 ediciones de una línea; `rg` residual → **0**. **F4 / FIND-776 cerrado** |
| **M6** | auto-reporte: `-All` da **2**, no 1 | corregido arriba |
| **L1–L5** | self-test sólo de regex; `/` final; firma a 3 líneas; §11 no verificado; código muerto | fixtures end-to-end en repo temporal (4); `overlaps` normaliza; ventana 5; `RULES.md` debe tener §11; línea muerta fuera |

**Estado:** el task file sigue `⏳ IN PROGRESS` — la ronda 2 del review la lanza
el orquestador. No hay auto-aprobación.

## D.2 — Ronda 3 del review (alcance cerrado: 4 ítems)

La ronda 2 refutó B1 y abrió B4. La causa de fondo fue la misma que este
informe describe: **se validó contra un fixture que no reproduce la forma real**
(`2h` pelado, cuando la spec escribe `🟡 1d`).

| ID | Defecto | Fix |
|---|---|---|
| **B1** | `isProse` clasificaba por lista de exclusión + "≥2 tokens ⇒ prosa": `🟡 1d` y `Origen: … commit 9d5f1e7` eran "claims" (bloqueaba toda fila), y `**92**` de un token era invisible (fail-open) | se juzga **por índice del header**: sólo `Hallazgo`/`Descripción`; el resto de columnas no se juzga nunca. En esas dos columnas, cualquier dígito/universal es claim → `**92**` ya no es invisible. Fixture = fila **literal** de `findings.md:31` leída del fichero |
| **B4** | `-Diff` comparaba contra `origin/develop` (35 commits atrás, push del owner) → 10 violaciones ajenas, rojo para todos; y `-Paths` fuera del diff saltaba la regla (b) en silencio | opción (i): `-Diff` = gate del commit contra `HEAD`; **`-Audit`** = auditoría ancha explícita (`origin/develop` + censo). Guard de (b) eliminado, corre siempre. Docstring y `.PARAMETER` corregidos; `/harness` cableado a la forma correcta |
| **M5** | `vanta-lead.md:16` ("único que prepara+ejecuta commit") y `:20` ("git mutating (commit/…)") contradecían `:7-9` y `AGENTS.md:313`. El `rg` de la ronda 2 no cubría esa forma → el cierre de FIND-776 fue prematuro | líneas alineadas **y contrato implementado**: check `commit-authority` (statement canónico en `AGENTS.md` + 7 formas contradictorias) = 6º check del gate. **FIND-776 cerrado ahora con detector** |
| **Nit** | `Contains()` en `-Paths`: `-Paths doc` matcheaba `docs/…` | segmento exacto + path inexistente = exit 2 |
| **Incidente** | durante la ronda 3, el helper creó `72643d9df` con el **árbol vacío** (2.365 paths borrados) y reportó "git commit failed"; el worktree quedó intacto (inversión HEAD↔disco) | reparado en `19f759980` por contenido (`commit-tree` del árbol de `1ed92d39e` + los 2 blobs nuevos, `update-ref`; sin `--amend` ni reset). Guard nuevo: check **`repo-integrity`** falla con árbol vacío o con un commit que borra ≥50 paths que siguen en disco. El mecanismo **no se reprodujo** → fila propuesta |

**Sondas: 12/12** con los scripts sin modificar (`--root`/`-Root`): las 8 de la
ronda 2, la fila literal de `findings.md:31`, `**92**` como claim, `-Paths` con
typo, la contradicción de autoría re-introducida y `repo-integrity` contra el
commit del incidente (`--ref 72643d9df` → exit 1; `HEAD` → exit 0). **Fixtures:**
31/31 (contrato) y 9/9 + 4/4 (cierre).

**Conteos reales:** `-Diff` (gate) → **1** (`FIND-758.md` de otro carril, sin
`Estado`); `-Diff -Paths <mis rutas>` → **0**; `-Audit` → **10** (9 ajenos en
vuelo + este task file); `-All` → **2** (`FIND-587` + este task file).

**Lo que queda abierto se registra como fila** (no se itera más sobre los seis),
con el texto listo en `H-02-H-07.md` §"Ronda 3": (1) `FIND-587` con
`⏳ IN PROGRESS` y sin fila en Backlog ni plan activo — `check-closure.ps1 -All`
= 1 violación; (2) el **incidente `72643d9df`** (commit con árbol vacío, 2.365
paths, reportado como fallo por el helper, mecanismo no reproducido) — con el
guard `repo-integrity` ya cableado como detector.

## D.3 — Ronda 4 (cierre de alcance: 3 ítems)

| ID | Defecto | Fix |
|---|---|---|
| **B1** 🔴 | `claimViolations` saltaba la fila cuando el header no tenía `Hallazgo`/`Descripción`: un header `Finding/Description` desactivaba el gate **en silencio** (fail-open) | (a) `CANON_HEADER` obligatorio en `Backlog.md` y `findings.md` (exit 1 con fichero + primera línea de tabla); (b) **fail-closed por fila añadida** al Backlog bajo una tabla sin columna de claim (tablas de waves de planes exentas) |
| **REQ** 🟠 | el fixture "literal" estaba hardcodeado (`FIND-999`, `2h`) — la misma clase de error que refutó B1 en la ronda 2 | el `--self-test` **lee** `findings.md` y usa su fila y su header reales; si el fichero cambia de forma, el fixture falla |
| **REQ** 🟠 | los conteos publicados no se reproducían: `Test-IdPresent` contaba **menciones**, así que `H-02-H-07` "tenía fila" por aparecer en la fila de `FIND-778` | `Get-BacklogOwnIds` (primera celda) + `Get-PlanOwnIds` (columna `ID`); números corregidos abajo |

**Conteos corregidos, medidos (no redondeados):**

```
pwsh dev-tools/check-closure.ps1 -All    → 3   (FIND-585, FIND-587, H-02-H-07)
pwsh dev-tools/check-closure.ps1 -Audit  → 10  (9 task files ajenos sin Estado + H-02-H-07)
pwsh dev-tools/check-closure.ps1 -Diff   → 1   (FIND-758.md, untracked de otro carril)
node dev-tools/check-harness-contract.mjs --self-test → 35/35
pwsh dev-tools/check-closure.ps1 -SelfTest            → 9/9 + 4/4
```

El delta contra lo publicado (2 y 10) y contra lo medido por el review (1 y 9):
ambos contaban menciones. Al anclar a fila propia aparece **`FIND-585`**, que
estaba oculto por una mención en otra fila. Vale el número que da el comando hoy.

**Sondas de B1: 8/8** — literal→0, hallazgo92→1, desc92→1, desc92pct→1,
medicionOk→0, `enHeader`→**1**, `synHeader`→**1**, fila añadida bajo tabla sin
claim→**1**.

**Abierto, como filas propuestas (no se arregla acá):** falso positivo de
`repo-integrity` con `git rm --cached`; `HAS_MEASURE` matcheando `no medido`;
split por `|` con pipe escapado; y la discrepancia de cifras 11/11 vs 12/12 entre
documentos. Texto completo en `H-02-H-07.md` §"Ronda 4".

---

# RESULTADO

**Veredicto:** 🔴 CAMBIOS REQUERIDOS — 15 hallazgos, 6 críticos.

**Conteo por severidad**

| Severidad | N | Hallazgos |
|---|---|---|
| 🔴 Crítico | **6** | F1 plan ✕ parser (0/35 medido) · F2 `findPlanFile` ignora `status:` · F3 commit desnudo en el prompt canónico pese a 28 lecciones y a un runbook (`workflow/RULES.md:252-276`) que ningún prompt cita · F4 autoría del commit contradictoria en 4 documentos · F5 sin campo de medición ni re-medición al cierre · F6 el plan no tiene columna de estado |
| 🟠 Alto | **5** | F7 `Estado` libre (59/111 sin campo, 4 formas no canónicas medidas, "Completado" leído como PENDING) · F8 tres fuentes de verdad (3 huérfanos + 4 no terminales medidos) · F9 `MAX_CONCURRENT` ciego al recurso real · F10 el orquestador puede diferir un lote del usuario · F11 `harness.md:71-74` conserva un número refutado por su propia fila |
| 🟡 Medio | **3** | F12 `progreso` declina commitear y declara task files STALE por diseño · F13 el review no revisa la premisa · F14 el helper verifica HEAD pero no el worktree |
| 🔵 Bajo | **1** | F15 el template de `FIND-*` no tiene campo de medición |

**Los 3 blindajes de mayor impacto, en orden**

1. **H-01 — El parser lee el formato real del plan + `status:` + estado
   obligatorio.** Es el multiplicador de todo lo demás: con `get_next_task`
   devolviendo 0 tareas, el claim atómico, el gate HARD-07 y el budget están
   **apagados en silencio**. 2 h, y el `Task not found` de ~21 task files deja de
   ser un misterio.
2. **H-03 — Un solo camino de commit (`git-commit-isolated`) + grep que lo
   vigila.** 28 lecciones en tres meses demuestran que la prosa no cambia el
   comportamiento; un `rg` que debe devolver 0 líneas sí. 40 min, y las 5
   variantes del índice compartido se vuelven imposibles de reintroducir sin que
   `/harness` lo diga.
3. **H-02 + H-04 — Medir al abrir y verificar al cerrar.** Son el mismo defecto
   (nadie re-mide) visto desde dos extremos: la fila entra con un número sin
   fuente y sale con un estado que nadie reconcilió. Un script
   (`check-claims.mjs`) y un script (`check-closure.ps1`), ambos diff-scoped.
   2.25 h total.

**Aplicable hoy en menos de una hora**

| # | Acción | Tiempo |
|---|---|---|
| 1 | Reemplazar `prompts/pipeline-full.md:172` por el helper + agregar el check `rg` a `vanta-harness §3` | 30 min |
| 2 | `commands/harness.md:71-74`: corregir "~39 en contexto prod" → "0 en prod (FIND-152)" | 2 min |
| 3 | `prompts/plan.md:245-252`: campo `Regla del lote` + columna `Medición:` en la plantilla | 15 min |
| 4 | `campaign-server.mjs:32-49`: filtrar por `status: active` antes de aplicar el guard de 24 h | 15 min |
| **Total** | | **~62 min** |

Si hay que elegir **una sola** para hoy: la #1. Es la única de las cuatro que
evita **pérdida de trabajo** (una copia stale horneada en un commit), y las otras
tres evitan errores que al menos dejan rastro en `git diff`.

**Lo que este informe no prueba:** que los 4 arreglos de una hora reduzcan la
tasa de premisas falsas. Eso requiere medir el próximo lote contra el baseline
de hoy (11 filas falsas / ~55 cerradas ≈ 20 %). Sin esa comparación, H-02 es una
hipótesis razonable, no un resultado.
