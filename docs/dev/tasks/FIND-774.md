---
title: "FIND-774: el selector de plan lee `status:`, no mtime, y el store deja de mentir"
kind: task
status: completed
description: "El selector elegía por mtime e ignoraba `status:`: dos planes `superseded` tocados por git tumbaban la elección del único `active`. El escritor además decía `Task not found` para IDs que sí existían como task file. Se declara el plan como store único (sin registry), el selector lee `status:` y el escritor nombra el task file."
---

# FIND-774: el selector de plan lee `status:`, no mtime, y el store deja de mentir

## Metadata

- **Plan file:** [2026-11-26-batch-parciales-y-nuevas.md](../plans/2026-11-26-batch-parciales-y-nuevas.md)
- **Fuente:** fila `FIND-774` de `docs/dev/Backlog.md`
- **Esfuerzo:** 🟢 2-4h
- **Prioridad:** 🟠 Alta
- **Tipo:** tooling del task-system (`.opencode/task-system/mcp/**`) — sin código de producto
- **Creado:** 2026-11-26
- **Depende de:** `FIND-775` (parser de tablas + `taskFound`, `5ed0c02`) — no se re-arregla, se verifica

## Problema (medido, no derivado)

1. **Asimetría lector/escritor.** `updateTaskStateCore` busca `### Task N:` / fila en
   el **plan file** y nunca abre `docs/dev/tasks/`; los lectores sí lo consultan
   (`campaign-server.mjs:436-438` en `validate_scope`, `:1193`, `:1492`). Un ID que
   existe sólo como task file recibía `Task not found`, indistinguible de un typo:
   ~21 falsos negativos en la sesión del 2026-11-26.
2. **`findPlanFile` ordenaba por `mtimeMs`** y tiraba `Ambiguous active plan` si había
   >1 `.md` tocado en 24 h. **No leía `status:`**, así que marcar `superseded` no tenía
   efecto observable.
3. **Escala del falso negativo, medida:** de **126** task files en `docs/dev/tasks/`,
   **108** no tienen fila en el plan activo (`2026-11-26-batch-parciales-y-nuevas.md`,
   50 tareas parseadas). El `Task not found` del escritor era estructuralmente falso
   para el **86 %** de los IDs, no un borde: los 21 de la sesión eran la muestra de lo
   que se tocó, no el total.

## Reproducción (antes del fix, repo real)

```
node -e "import('./.opencode/task-system/mcp/campaign-server.mjs').then(m=>console.log(m.findPlanFile(process.cwd())))"
→ THROWS: Ambiguous active plan: 3 planes modificados en las últimas 24h
  (2026-11-26-batch-parciales-y-nuevas.md, backlog-release-v0.10.0.md, 2026-11-23-release-v0.10.0-gates.md)
```

Los tres están en la carpeta; **uno** es `status: active` y dos son `status: superseded`.
Los dos muertos tenían mtime reciente porque `git checkout`/`git pull` los reescribió —
el criterio primario era, literalmente, el reloj del sistema de ficheros.

Después: `findPlanFile → docs/dev/plans/2026-11-26-batch-parciales-y-nuevas.md`.

## Decisión: store declarado, NO registry

**El plan file es el único store de estado. `docs/dev/tasks/<ID>.md` es la spec de la
tarea, no un segundo store. Ficheros + git son la fuente de verdad.**

| | Coste |
|---|---|
| **Registry (rechazado)** | Un índice más que cada lector y cada escritor tienen que consultar; se desincroniza en cuanto alguien edita el plan a mano, y su fallo es **silencioso** (responde con el estado viejo). Es exactamente la enfermedad que produjo los 21 `Task not found`: dos fuentes, cada una consultando la mitad. Construirlo no elimina la asimetría — la mueve a un tercer sitio. |
| **Store declarado (elegido)** | No hay caché que consultar, así que no hay caché que se quede vieja. El precio es que el escritor **no puede** registrar estado de una tarea que no tiene fila en el plan: lo que se arregla no es "escribir igual", es **dejar de decir que no existe** y nombrar el fichero. |

Consecuencia de contrato: *"Task not found" es imposible cuando el task file existe* —
el mensaje pasa a `has a task file (…/FIND-774.md) but no task block/row in <plan>- nothing
written. The plan file is the only state store; docs/dev/tasks/ holds the spec.`

## Cambios

| Fichero | Cambio |
|---|---|
| `parsers.mjs` | `isInactivePlan` exportada + `isSelectablePlan`: un solo juicio de "qué plan puede recibir writes" (`status: active` o sin frontmatter), compartido por selector y parser. |
| `campaign-server.mjs` | `findPlanFile` lee el frontmatter de cada plan; **`status:` es criterio primario, el mtime sólo desempata** dentro de los selectables. Sin selectables → `null`. >1 selectable tocado en 24 h → error que **nombra los ficheros y su status** y dice cómo resolverlo. |
| `campaign-server.mjs` | `findTaskFile` (raíz + `complete/` + `closed/`) y el early-return de `updateTaskStateCore` distingue "no existe" de "existe como task file". `updated:false`/`taskFound:false` intactos (contrato FIND-775). |
| `plan-store.test.mjs` | 10 tests nuevos (3 propiedades del contrato + regresiones). |

## Verificación

```
node --test .opencode/task-system/mcp/*.test.mjs
antes:  tests 62 / pass 62 / fail 0
después: tests 72 / pass 72 / fail 0   (+10)
```

Propiedades cubiertas, una por test:

- `superseded` con **mtime más reciente** → ignorado (`FIND-774/1`, `1b`).
- dos `active` recientes → error que **nombra ambos** y sugiere `status: superseded` (`2`, `2b`).
- plan sin frontmatter = activo (el formato bloque histórico no cambia) (`3b`); sin
  selectables → `null` (`3`); un `active` viejo sigue siendo el elegido (`3c`).
- task file presente sin fila en el plan → `taskFileFound:true`, warning que **nombra el
  fichero**, plan **byte-idéntico** (`4`, `4b`); ni fila ni fichero → sigue diciendo
  `not found` (`4c`).

## Coste medido (por qué no hay caché)

`findPlanFile` ahora lee el contenido de cada plan en vez de sólo `stat`: **0.56 ms**
promedio sobre los 3 planes reales (104 KB) — se llama una vez por tool call, no en un
hot loop. No se añade caché de status: una caché es el criterio-por-mtime con otro
nombre, y su fallo también sería silencioso.

## Deuda / fuera de alcance

- `validate_scope` (`campaign-server.mjs:435-442`) todavía arma seis rutas, tres de ellas
  al árbol muerto `.opencode/skills/campaign-executor/tasks/`. `existsSync` las salta:
  no rompe, pero es ruido. `NOTICED BUT NOT TOUCHING`.
- La retirada de la fila `FIND-774` quedó en el working tree **sin commitear**:
  `docs/dev/Backlog.md` arrastraba una edición ajena en vuelo (fila `FIND-793`) y
  commitear el fichero se la llevaba (FIND-792). Queda para el carril que commitee el
  Backlog.
