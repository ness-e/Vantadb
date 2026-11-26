---
title: "FIND-749: el plugin de captura lee ingest.health en vez de recomputar el predicado"
kind: task
status: in-progress
description: "El plugin de OpenCode deja de derivar isBroken de runner+queue.depth y consume health/drain_path del core, distinguiendo broken de unknown y tratando queue:null como ausencia de lectura"
---

# FIND-749: el plugin de captura lee `ingest.health` en vez de recomputar el predicado

## Metadata

- **Contrato upstream:** [ADR-0065](../architecture/adr/ADR-0065-ingest-queue-live-probe-health.md)
  § D3 (tabla de estados) y § D8 (qué responde cada proceso del wire)
- **Dependencias cerradas:** `FIND-748` (`dfb51d807`, en push) · `ADR-0065` enmendado
  (`559c622ef`) · `MCP-47` (`6196c7be0` + `2dcfcc41e`)
- **Tipo:** corrección de contrato en el cliente (`fix`)
- **Repo:** `.opencode/` es un repo Git **separado** (base `2296ccd`); el task file
  vive en VantaDB y es lo único que se commitea acá
- **Carril:** C (TypeScript/Node, sin `cargo` — hay un push corriendo `verify.ps1`)
- **Review:** `vanta-review` (P2-01), contexto fresco

## El defecto, en una frase

El core publica desde `FIND-748` un veredicto (`health: unknown|idle|draining|broken`,
más `drain_path`) y `ADR-0065` fija que el cliente **lee**, no recomputa; el plugin
sigue calculando `runner === "none" && queue.depth > 0`, que es falso en el caso que
el semáforo existe para mostrar — un loop que arrancó y murió dejando trabajo
encolado — y convierte `queue: null` en `0` con `?? 0`.

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `.opencode/plugins/vanta-memory-capture.ts`
  (645 L), `.opencode/plugins/vanta-memory-capture.selftest.mjs` (438 L),
  `docs/dev/architecture/adr/ADR-0065-ingest-queue-live-probe-health.md` (391 L),
  `vanta-memory/src/ingest/observability.rs:340-450` (derivación + `notice`),
  `docs/api/MCP.md:450-540,660-740`, `docs/dev/tasks/FIND-748.md`
- **Referencias hacia dentro (lo que el plugin importa):** `node:child_process`,
  `node:path`; el loader v2 de OpenCode resuelve el plugin sin
  `@opencode/plugin` (verificado en `ponytail.ts` y en `MCP.md:686-698`)
- **Referencias entrantes (quién nombra los archivos):** `docs/api/MCP.md:689,698`
  (contrato publicado del plugin), `docs/dev/tasks/MCP-47.md`,
  `docs/dev/tasks/FIND-748.md:38-40,380-387`, `ADR-0065:152-153,346`,
  `docs/dev/avance/activo/vanta-memory.md`, `docs/dev/Backlog.md`,
  `.opencode/task-system/memory/lessons.md`; dentro de `.opencode/`, sólo el
  propio self-test importa `ingestHealth`/`ingestNotice`/`queueDepth`
- **Veredicto impacto:** **bajo en el wire, alto en el blast radius.** El plugin
  corre en cada sesión de agente: un aviso que no dispara cuando el loop murió es
  el bug; un aviso que dispara de más entrena a ignorar el canal. La forma del
  payload no cambia (sólo se consume `health`/`drain_path`, ya publicados), así
  que ningún writer se rompe — y la ausencia de `health` se degrada sin sesión
  rota (fail-open). MCP.md `:698` afirma que el plugin «still recomputes that
  predicate», frase que este cambio vuelve falsa: se corrige en el mismo commit.

## Decisión: ¿fallback al predicado viejo, o silencio sin `health`?

**Elegido: sin `health` ⇒ sin aviso, con una línea de debug por proceso.**

| Opción | Qué gana | Qué cuesta |
|---|---|---|
| Fallback al predicado viejo | una alarma **verdadera pero incompleta** para writers anteriores a `FIND-748` (`degraded ∧ depth > 0` *es* `broken` bajo la precedencia de D3) | dos definiciones de «roto» en el camino caliente, una de ellas ciega al loop muerto; el plugin no puede afirmar que lee el veredicto del core; el pre-mortem (a) se cumple — el fix no arregla nada en la práctica |
| Empujar una línea «no hay lectura» para el payload sin `health` | un writer viejo con backlog no se lee como «está bien» | un banner por sesión, para siempre, por un hecho de despliegue que el plugin ya delata mejor: `transport: <bin>`, una vez por proceso. El semáforo de memoria se convertiría en detector de binario stale |
| **Sin `health` ⇒ silencio + debug (elegido)** | una sola definición de «roto», la del core | ese caso no se avisa |

Razón corregida tras el review (ronda 1, «Alternativas»): la razón **no** es
«el predicado viejo miente» —en el subconjunto legacy es cierto— sino que
**`ADR-0065` D3 deja una sola fuente de verdad**, y un segundo predicado,
aunque acierte a veces, es una segunda respuesta que puede discrepar de la
primera y que nadie va a recordar mantener. El `broken` que el core no puede
expresar en ese payload (loop muerto) es justamente el caso que motivó el
ticket, así que conservarlo da la falsa sensación de estar cubierto.

Si `health` falta, el payload es anterior a `ADR-0065`: se emite entonces
`log("… writer older than ADR-0065 … the semaphore stays silent …")`, una vez
por proceso. El probe fallido **no** se cachea (ronda 1, N1): un timeout no
puede silenciar el semáforo para todo el proceso.

## Contrato mecánico

Reglas que el plugin implementa (`ingestNotice`):

- `health === "broken"` → avisa; el texto incluye el `drain_path` **leído del
  payload** (no un literal) y distingue **runner sin configurar**
  (`degraded`/`runner: none`) de **loop muerto** (`scheduler_ever_started: true`)
  y de **loop nunca arrancado**.
- `health === "unknown"` → **no** avisa de roto; una línea dice que no hay lectura
  y que no es un veredicto.
- `health === "draining"` / `"idle"` → silencio.
- `queue === null` → `queueDepth()` devuelve `null`, nunca `0`; el texto no
  inventa un conteo.
- Una vez por sesión (`warned`), no por turno; el probe fallido **no** se cachea.

```bash
node .opencode/plugins/vanta-memory-capture.selftest.mjs
node --test .opencode/plugins/vanta-memory-capture.selftest.mjs
node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs
```

**Mutación de control (obligatoria):** reinsertar el predicado viejo
(`runner === "none" && depth > 0`) **como puerta de `ingestNotice`**, que es donde
vive la decisión:

```js
const oldIngest = ingestOf(capabilitiesOrIngest);
const oldDepth = Number(oldIngest?.queue?.depth ?? 0);
if (!(oldIngest && oldIngest.runner === "none" && Number.isFinite(oldDepth) && oldDepth > 0)) return null;
```

El caso (a) —`health: "broken"` + `drain_path: "none"` + runner sano— tiene que
caer en rojo. Mutar un helper que nadie llama no prueba nada: por eso
`isIngestBroken` **no existe** (ver § Review, B2).

## Steps

1. ✅ RED: casos (a)-(d) en el self-test, con la fórmula vieja en su sitio → rojo
2. ✅ GREEN: `ingestHealth` / `queueDepth` + `ingestNotice` por `health`;
   fixtures del self-test migradas al payload nuevo
3. ✅ VERIFY: self-test (28 tests) + mutación de control pegada + docs gates
4. ✅ Docs: `docs/api/MCP.md:698` deja de afirmar que el plugin recomputa
5. ✅ `vanta-review` ronda 1 `changes-required` → aplicado; ronda 2 `approve`
   (§ Review). Commit local de cierre.

## Alcance cerrado

| Archivo | Repo | Cambio |
|---|---|---|
| `.opencode/plugins/vanta-memory-capture.ts` | `.opencode` (base `2296ccd`) | la lógica |
| `.opencode/plugins/vanta-memory-capture.selftest.mjs` | `.opencode` (base `2296ccd`) | el contrato mecánico |
| `docs/dev/tasks/FIND-749.md` | VantaDB | este registro |
| `docs/api/MCP.md` | VantaDB | `:698` (qué lee el plugin) y `:751` (la gate son 28 tests) |
| `docs/index.md`, `llms.txt` | VantaDB | regenerados por `gen-index.mjs --write` |

No tocados y **declarados ajenos**: `docs/dev/plans/…gates.budget.json` (lo escribe
el task-system), `web/**` (worktree ajeno), y el resto del WIP de `.opencode/`
(~1030 entradas de otra sesión). **No** se commitea nada dentro de `.opencode/`:
sus dos archivos quedan `??` y el Gate H (`/harness`) queda pendiente de quien
commitee ese repo.

## Implementación

`isIngestBroken` se **borró** (ver § Review, B2): era el último domicilio del
predicado viejo y no tenía ningún caller de producción — sólo el self-test, que
así lo mantenía vivo. La decisión vive en un solo lugar: `ingestNotice`.

| Archivo | Qué entró |
|---|---|
| `.opencode/plugins/vanta-memory-capture.ts` | `ingestHealth` (lee `health` con lista blanca de los 4 estados; un valor desconocido es «sin lectura», no «writer viejo»), `queueDepth` (`null` es ausencia de lectura, nunca `0`), `drainPathOf` (el texto imprime el `drain_path` recibido), `ingestNotice` con tres ramas (broken / unknown de una línea / idle·draining mudos), `whyNothingDrains` separando runner-sin-configurar, loop-que-arrancó-y-murió y loop-nunca-arrancado; `ingestOf` acepta el objeto `ingest` con sólo `health`; el probe **no cachea** un fallo y marca la sesión sólo cuando hubo respuesta; diagnóstico separado para «campo ausente» vs «estado que no conozco» |
| `.opencode/plugins/vanta-memory-capture.selftest.mjs` | casos (a)-(d), fidelidad de `drain_path`, unidades `ingestHealth`/`queueDepth`, y tres de wiring: payload sin `health`, estado desconocido, probe fallido que se reintenta |
| `docs/api/MCP.md` | `:698` (el plugin lee `health`, no recomputa) y `:751` (la gate mecánica son 28 tests, no 19 assertions) |
| `docs/index.md`, `llms.txt` | regenerados: el delta contra HEAD es **la fila nueva + 3 contadores** (877→878, 740→741, 247→248). No hay drift previo que este cambio haya cerrado: el que `FIND-748` declaró ya no existía en HEAD |

## Verificación

```text
node .opencode/plugins/vanta-memory-capture.selftest.mjs
  → ℹ tests 28 · pass 28 · fail 0
node --test .opencode/plugins/vanta-memory-capture.selftest.mjs
  → ℹ tests 28 · pass 28 · fail 0
node scripts/docs/check-links.mjs      → broken markdown links: 0
node scripts/docs/check-docs.mjs       → GATING: all clear
node scripts/docs/gen-index.mjs --check → exit 0
npx markdownlint-cli2 --config .github/markdownlint-cli2.yaml docs/api/MCP.md
  → 0 issues
```

`node --test .opencode/plugins/` (la forma de directorio) recolecta **0 tests**:
el runner sólo busca `*.test.*`/`test-*`, y el archivo del plugin no matchea.
La gate real es la ruta explícita del archivo; la forma de directorio no mide
y no se usa como verificación.

### Mutación de control (el test mide)

Sobre una **copia** del plugin en `%TEMP%\opencode\find749-mutation\` (el
archivo del repo nunca se mutó), se reinsertó el predicado viejo como primera
sentencia de `ingestNotice`, que es donde vive la decisión:

```js
// MUTATION (control): the pre-FIND-749 predicate.
const oldIngest = ingestOf(capabilitiesOrIngest);
const oldDepth = Number(oldIngest?.queue?.depth ?? 0);
if (!(oldIngest && oldIngest.runner === "none" && Number.isFinite(oldDepth) && oldDepth > 0)) return null;
```

```text
not ok 4 - ingestNotice is driven by health in both wiring shapes and stays mute without it
not ok 5 - (a) broken + drain_path none warns, and names the dead loop
not ok 6 - the notice prints the drain_path from the payload, not a constant
not ok 7 - (b) unknown is not reported as broken, and says there is no reading
not ok 8 - (b, wiring) an unknown pipeline injects no broken alarm
not ok 9 - (c) a null queue is not depth 0
not ok 10 - ingestNotice stays silent for idle and draining
not ok 15 - (d) broken warns once per session, not once per turn
not ok 19 - a failed capabilities probe is retried on the next session
# tests 28 · # pass 19 · # fail 9
```

```text
error: 'a dead loop with queued work must warn — silence here IS the FIND-749 bug'
code: 'ERR_ASSERTION'
expected: true
```

Restaurada la copia, la corrida vuelve a 28/28 en verde.

## Review

### Ronda 1 — `changes-required`

- **Revisor:** `vanta-review` (sub-agente, contexto fresco; no participó de la
  implementación). Enfoque adversarial sobre los tres archivos contra
  `ADR-0065` § D3/D5/D8, `vanta-memory/src/ingest/observability.rs` y
  `vantadb-mcp/src/server.rs:1057-1065`.
- **Cómo se probó:** leyó y **re-ejecutó** el self-test y la mutación en copia;
  levantó dos sondas propias en `%TEMP%` para medir los hallazgos (mutar
  `isIngestBroken`, y `capabilities` fallido en 3 sesiones).
- **Veredicto:** `changes-required` — 3 bloqueantes + 9 no bloqueantes. La lógica
  central la dio por buena («el caso (a) **sí** mide, 6 rojo, `not ok 5`»).

#### Bloqueantes aplicados

| # | Hallazgo | Corrección |
|---|---|---|
| B1 | El task file pre-marcaba `✅ Review P2-01 + commit local` sin que existieran ni la sección `## Review` ni el commit — el implementador escribiéndose el veredicto | Paso 5 a ⏳, esta sección existe, y el commit se hace después |
| B2 | La mutación que el contrato prescribía (mutar `isIngestBroken`) era un **no-op**: `ingestNotice` no llamaba al helper, así que el caso (a) seguía verde. `isIngestBroken` no tenía caller de producción; sólo el self-test, que lo mantenía vivo | `isIngestBroken` **borrado**; el contrato prescribe ahora la mutación sobre la **decisión** (`ingestNotice`), que es la que se corrió (9 rojo, incluido `not ok 5`) |
| B3 | El aviso de `broken` decía «`--mcp` once MCP-43 lands» — falso: `server.rs:1057-1065` arranca el loop antes del primer request — y se contradecía 2 líneas después («the `--mcp` writer … host the loop»). La misma frase estaba en el header del archivo | Texto y header corregidos: «a `--mcp` writer and the HTTP server both host one» |

#### No bloqueantes aplicados

- **N1** — un `capabilities` fallido quedaba cacheado y silenciaba el semáforo
  para todo el proceso, y `warned.add` iba **antes** del `await`. Ahora sólo se
  cachea una respuesta exitosa y la sesión se marca después de recibirla. Test
  nuevo: `a failed capabilities probe is retried on the next session`.
- **N2** — el párrafo de `unknown` (~60 palabras) es un banner perpetuo en
  configuraciones legítimas (writer read-only, `VANTADB_SCHEDULER_INTERVAL_MS=0`)
  y su consejo era circular. Reducido a una línea que dice la ausencia y que no
  es un veredicto.
- **N3** — un estado futuro de `health` (`IngestHealth` es `#[non_exhaustive]`)
  se diagnosticaba como «writer viejo». Ahora «campo ausente» y «estado que no
  conozco» son dos mensajes distintos, ambos fail-open. Test nuevo.
- **N4** — la rama `broken` escribía `drain_path: none` como literal. Ahora
  imprime el `drain_path` del payload (`drainPathOf`). Test de fidelidad nuevo.
- **N5** — faltaba el wiring del payload **sin** `health`, justo la decisión que
  defiende la sección anterior. Test nuevo: `a payload without health is silent
  and says why once per process`.
- **N6** — el task file afirmaba que la regeneración del índice «cerró el drift
  previo»: el diff sólo agrega una fila y 3 contadores (el drift de `FIND-748`
  ya no existía en HEAD). Corregido.
- **N7** — faltaba alcance cerrado: agregado, con `budget.json` / `web/` / WIP de
  `.opencode/` declarados ajenos.
- **N9** — se quitó una aserción que no podía fallar.

#### No aplicados (declarados)

- **N8** — `ADR-0065:152-153` cita el guard `queue?.depth ?? 0` del plugin, que ya
  no existe. El ADR es **inmutable** una vez aceptado y suyo es el cuerpo:
  queda para `vanta-arch` como una línea de puntero («now reads `health`»).
  Editar la tabla de medición de un ADR aceptado para seguir un archivo de otro
  repo es exactamente lo que la regla de inmutabilidad prohíbe.
- **Gate H (`/harness`)** — la norma lo exige **antes de commitear** en
  `.opencode/`. Esta tarea no commitea ese repo (sus dos archivos quedan `??`), así
  que el gate sigue debiendo hacerse en el commit del harness.

### Ronda 2 — `approve`

- **Revisor:** el mismo, alcance acotado a los 3 bloqueantes + 9 no bloqueantes.
- **Cómo se probó:** re-ejecutó el self-test (28/28 exit 0), la mutación sobre los
  archivos nuevos en copia (9 rojo, `# fail 9`, `not ok 5`), y los tres gates de
  docs (0 / all clear / exit 0). `grep` de `isIngestBroken` = 0 hits.
- **Veredicto:** **`approve`**. B1/B2/B3 cerrados; N1/N3/N4/N5/N6/N7/N9 cerrados;
  N8 (ADR inmutable) aceptado como deuda declarada. Cuatro nits sin bloqueo:
  dos corregidos acá (`:45` nombraba el símbolo borrado; comentario duplicado del
  probe), uno es texto para el orquestador (la fila de Backlog dice `✅` porque
  se pega **después** del commit) y uno queda sin ID (el colateral de
  `MCP.md:735` necesita una fila `FIND-*` que asigna el lead — no se edita
  `Backlog.md` desde acá).
- **Residuo aceptado, declarado:** la glosa de `unknown` describe «cannot see a
  queue» (la causa `queue: null`, que en el wire no se produce) en vez de la
  alcanzable (writer read-only / loop deshabilitado, que sí ve su `{0,0}` pero no
  evidencia hospedaje). El texto sigue siendo cierto de lo que el plugin sabe —no
  vio ninguna cola— y el fixture usa la forma wire-imposible porque es la que el
  contrato de la tarea pidió ejercer (caso c).

### Reglas transversales

- **Regla 0:** los tres archivos leídos completos antes de editar, más
  `ADR-0065`, `observability.rs:340-450`, `MCP.md:450-540,660-763` y
  `FIND-748.md`; referencias entrantes mapeadas con `rg` sobre ambos repos.
- **Scope discipline:** no se tocó el core (`vanta-memory/`, `vantadb-mcp/`),
  ni `docs/dev/plans/**`, ni `Backlog.md`, ni nada dentro de `.opencode/` fuera
  de los dos archivos del plugin.
- **Colateral detectado y NO tocado:** `docs/api/MCP.md:735` sigue diciendo
  «Until an in-process scheduler exists (`MCP-43`), a queued turn waits in L0 and
  the queue does not drain», que contradice a `:668-677` del mismo archivo (el
  writer `--mcp` sí hospeda el loop). Es la misma clase de frase que B3, fuera
  del alcance de esta tarea → fila `FIND-*` para el carril de docs.

## Fila de Backlog (para que el orquestador la cierre; no editada aquí)

> `FIND-749` | 🟡 Media | **El plugin de captura decide `isBroken` con la fórmula
> vieja y no consume `ingest.health`** | `.opencode/plugins/vanta-memory-capture.ts`
> | 🟢 30-60 min | ✅ Completado | Lee `health`/`drain_path`; `unknown` no se
> reporta como roto y `queue: null` no se lee como `0`; self-test con los 4 casos
> nuevos + mutación de control. | Origen: `FIND-748` (2026-11-25) | Ver:
> `ADR-0065`, `MCP-47`
