---
title: "MCP-47: plugin de OpenCode que postea el turno y ensambla el contexto recall-first"
kind: task
status: completed
description: "Plugin v2 de OpenCode: en `session.idle` postea el intercambio cerrado a `thread_send` (contenido por stdin, credenciales redactadas) y en el hook `context` inyecta `memory_recall` solo si hay hits; avisa una vez por sesión cuando `capabilities.ingest.is_broken()`."
---

# MCP-47: plugin de OpenCode que postea el turno y ensambla el contexto

## Metadata
- **Plan file:** `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` (§9.b, ola MCP/MEM)
- **Fuente:** Backlog `MCP-47` — opción A del análisis: el agente no captura nada; hoy hay que llamar `memory_put` a mano para que la memoria exista
- **Esfuerzo:** 🟡 2-3d
- **Tipo:** TypeScript (harness: `.opencode/plugins/`, repo Git separado) — carril C, no compila Rust
- **Estado:** ✅ COMPLETED
- **Creado:** 2026-11-25

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | El loader de OpenCode v2 (`.opencode/plugins/*.ts` se descubre solo, `https://opencode.ai/docs/plugins/`); `ctx.session.hook("context")` corre en **cada request de modelo** (incluidas continuaciones por tool); `ctx.event.subscribe({types:["session.idle"]})` una vez por sesión |
| Callees | `vanta-cli mcp-call` (WIRE-10, one-shot) → `thread_send` (MCP-44: captura L0 + encola L1), `thread_create`/`thread_list` (thread estable por sesión), `memory_recall` (recall-first), `capabilities` (MEM-72 `ingest`) |
| Implicaciones | Es el plugin que corre en **cada sesión de agente**. Tres fallos son graves: (a) romper la sesión → todo camino es `try/catch` y el transporte tiene timeout; (b) inyectar memoria irrelevante → `recalled` vacío **no** inyecta nada; (c) postear secretos → `redactSecrets` en la frontera + solo partes `text` (nunca tool-calls/results) + contenido por stdin, jamás argv. Sin migración de datos. Sin cambios en `vanta-memory/**`, `vantadb-mcp/src/**` ni `src/**`. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos o en la zona editada):** `.opencode/plugins/ponytail.ts` (205L — el único plugin v2 del repo, patrón de `id` + `setup` + `ctx.session.hook`), `.opencode/skills/vantadb-mcp/assets/hooks/opencode/vantadb-memory.js` (207L — el template v1 equivalente; su `session.idle` postea **un marcador estático**, no el turno: ese es el hueco que cierra esta tarea), `.opencode/skills/vantadb-mcp/assets/hooks/README.md` (mapa de hooks + WIRE-10), `docs/api/MCP.md` §Threads (L580-636: `thread_send`, frontera proxy/scheduler, `capabilities.ingest` L459-489, `memory_recall` L349), `docs/api/VANTA_MEMORY.md` §`ingest::observability` (L368-396: `is_broken`), `src/cli_handlers/mcp_call.rs` (`{{placeholder}}` desde stdin, cap 1 MiB, `--args` debe ser objeto), `.opencode/node_modules/@opencode/plugin@2.0.15` + `@opencode/schema` (`SessionHooks`: **no existe** hook `idle`/`message`; `session.idle` es un **evento**), `docs/dev/tasks/MCP-44.md` (contrato de `thread_send`).
- **Referencias hacia dentro:** `.opencode/opencode.jsonc` no lista el plugin (el loader descubre `.opencode/plugins/` automáticamente); ningún otro archivo importa estos módulos nuevos.
- **Referencias entrantes de lo editado:** `docs/api/MCP.md` (nueva subsección §OpenCode capture plugin) y el propio self-check; el template v1 de `skills/vantadb-mcp/assets/hooks/opencode/` queda intacto y sigue siendo válido para runtimes v1.
- **Veredicto impacto:** MEDIO-ALTO. Un archivo nuevo que corre en cada sesión + un gate ejecutable. Pre-mortem: (a) **inyectar irrelevante** → gate `recalled: [] ⇒ nada` con un test que falla si se invierte; (b) **romper la sesión** → `setup` y cada hook envueltos, transporte con timeout de 5 s y sin `throw` hacia OpenCode — test `a dead transport never breaks the session`; (c) **postear secretos** → 8 patrones de credencial + solo partes `text` + stdin — test `credentials are redacted before they are posted`; (d) **prometer consolidación** → el aviso dice literalmente "captures but does not consolidate ... the queue does not drain", atado a `isIngestBroken()` (MEM-72), y hay un test que prohíbe la frase "consolidated your memory".

## Contrato

```
node .opencode/plugins/vanta-memory-capture.selftest.mjs
node --test .opencode/plugins/vanta-memory-capture.selftest.mjs
```

Ambas deben pasar 13/13, y **el self-check debe fallar si se invierte cualquiera de las dos ramas** (verificado por mutación, §Verification). No hay `tsconfig` en `.opencode/` ni `tsc` instalado: `npx tsc --noEmit -p .opencode` **no es ejecutable** aquí (ver §Deuda).

## Spec (SDD)

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Cómo se llama al MCP desde el plugin | A) `vanta-cli mcp-call` one-shot / B) sesión MCP stdio propia (proceso hijo vivo: fuga si el plugin recarga) / C) HTTP directo al writer (rompe la elección de writer de MCP-48) | ✅ A — WIRE-10 ya es el camino del repo (`hooks/README.md:37`); no abre un segundo escritor |
| 2 | Por dónde viaja el texto del turno | A) `--args` en argv / B) stdin con `{{turn}}` | ✅ B — `mcp_call.rs:52-108` resuelve placeholders desde stdin; argv es legible por otros procesos y esto es texto de usuario |
| 3 | Qué evento cierra el intercambio | A) `session.idle` (evento) / B) hook `message.*` / C) hook `tool.execute.before` (template v1) | ✅ A — evidencia: `SessionHooks` de `@opencode/plugin@2.0.15` **no** tiene `idle` ni `message`; `session.idle` existe como evento (`schema/session-status-event.d.ts:128`) con `data.sessionID`. C no existe en v2 y además posteaba antes de que existiera la respuesta |
| 4 | De dónde sale el contenido del turno | A) leer `ctx.session.context({sessionID})` en el idle / B) el payload del evento | ✅ A — el evento trae solo `sessionID`; el transcript se lee una vez por turno cerrado |
| 5 | Qué se postea | A) user + assistant (solo partes `text`) / B) todo el historial visible / C) solo el último mensaje | ✅ A — `thread_send` captura por rol (`docs/api/MCP.md:587`); B duplicaría turnos ya capturados (dedupe por `messageID`), C perdería la pregunta |
| 6 | Dedupe | A) por `messageID` del assistant (memoria del proceso) / B) sin dedupe | ✅ A — el idle puede repetirse por turno y por reconexión; el post duplicado infla la cola |
| 7 | Thread por sesión | A) título determinístico `vanta-opencode-idle-<sessionID>` resuelto con `thread_list` / B) un thread nuevo por proceso | ✅ A — patrón del template v1 (FIND-169): sobrevive al reinicio del plugin sin crear uno por carga |
| 8 | Recall: cuándo se inyecta | A) solo si `recalled` no está vacío / B) inyectar siempre un header ("no tengo memoria de esto") / C) inyectar cualquier `prepend_context` no vacío | ✅ A — B mete ruido en cada turno; C se tragaría un header sin hits. `recalled` es la señal de hits; `prepend_context` se usa **verbatim** cuando hay hits |
| 9 | Frecuencia del recall | A) uno por turno de usuario (cacheado por texto) / B) uno por request de modelo | ✅ A — el hook `context` corre también en continuaciones por tool; B multiplica llamadas por el mismo texto |
| 10 | Dónde vive la memoria del proceso | A) `Map` por `sessionID` en el closure de `setup` / B) `ctx.storage` (persistente) | ✅ A — todo lo cacheado (recall, threads, avisos) es reconstruible; persistirlo crearía estado viejo que sobrevive al cambio de DB |
| 11 | Aviso de pipeline roto | A) push a `system` una vez por sesión + `console.warn` / B) en cada request | ✅ A — un banner por request es ruido que el agente aprende a ignorar |
| 12 | Qué pasa sin `VANTADB_DB_PATH` | A) no-op inerte con log / B) adivinar `<repo>/.vanta/memory` | ✅ A — B crearía un store fantasma; un plugin instalado no debe inventar rutas |
| 13 | `capabilities` sin bloque `ingest` (medido: un DB fresco no lo trae) | A) silencio (superficie sin instrumentar) / B) tratarlo como `runner: none` y avisar | ✅ A — avisar de un pipeline que este proceso no reporta sería inventar el diagnóstico; `is_broken()` exige evidencia (`runner: none` + cola) |
| 14 | Presupuesto de timeout del transporte (medido, no supuesto) | A) 5 s para todo / B) dos presupuestos según el camino / C) sin timeout | ✅ B — medido en la máquina de referencia con el binario release: `capabilities` 1,6-2,5 s, `memory_recall` 7,0-8,5 s, `thread_send` >5 s (cada `mcp-call` arranca un server nuevo: storage + perfil de hardware + sesión ONNX). Con 5 s el recall **caducaba en cada turno** (feature muerta y silenciosa). Lectura 12 s (dentro de un request de modelo), escritura 15 s (en `session.idle`, nadie espera) |
| 15 | Qué pasa si el MCP está enfermo | A) reintentar en cada turno / B) breaker: 2 fallos consecutivos de lectura ⇒ recall apagado por sesión | ✅ B — A castiga cada turno con 12 s de espera; B paga como mucho dos veces. Los writes siguen reintentando (fuera del camino crítico) |

## Steps

| # | Step | Estado |
|---|------|--------|
| 1 | RED: self-check `node:test` con las 3 promesas (posteo, no-inyección, inyección) — falla por módulo inexistente | ✅ |
| 2 | GREEN: `vanta-memory-capture.ts` — transporte `mcp-call` por stdin, `redactSecrets`, `recallBlock`, `isIngestBroken`/`ingestNotice`, `lastTurn`, captura en `session.idle`, ensamblado en `context` | ✅ |
| 3 | Mutaciones: invertir la rama de recall y la de captura → el self-check falla en ambas (y se revierte) | ✅ |
| 4 | Docs: `docs/api/MCP.md` §OpenCode capture plugin (contrato, config, qué no promete) | ✅ |
| 5 | Review P2-01 por agente distinto + commit local | ✅ |

## Review P2-01 (ronda 1 — `vanta-review`, veredicto `changes-required`, 17 hallazgos)

Todos los hallazgos se evaluaron contra el artefacto; 15 dieron cambio, 2 quedaron como trade-off documentado.

| # | Sev | Hallazgo | Disposición |
|---|-----|----------|-------------|
| 1 | CRIT | El **query de recall** viajaba en argv (`--args`), no por stdin: el turno del usuario quedaba en la línea de comandos | ✅ `query: "{{query}}"` + stdin, igual que el turno. Test nuevo que falla si el texto aparece en `args` |
| 2 | CRIT | `subscribe({types})` **no existe** en `@opencode/client@2.0.15` (solo `{signal,onActivity}`): el pump recibía *todos* los eventos y `await`eaba dentro del loop → un subscriber atrasado es descartado a los 4096 eventos | ✅ El filtro pasa a ser un check propio en el loop y la captura ya **no se await** dentro del stream (guard por sesión con `inFlight`) |
| 3 | HIGH | `sent === 0` ⇒ `forceNew` re-postea el turno en un **thread nuevo** (nuevo `session_id` server-side ⇒ L0 no puede dedupear) | ✅ Se elimina el retry que forkea: se invalida el id cacheado y se re-resuelve en el turno siguiente. Test: un envío fallido no crea segundo thread ni re-postea |
| 4 | HIGH | El bound era por intento, no por hook | ⚠️ Parcial: ya existía `break` en timeout (el ladder no multiplica por 4); se bajó el scan de threads de 10 a 3 páginas. `thread_list` serializa el historial completo, así que el tope importa |
| 5 | HIGH | El semáforo es **estructuralmente inalcanzable** con transporte one-shot (`ingest` es process-global y el proceso muere) y el task file afirmaba lo contrario | ✅ Medido en el E2E (`ingest: null`, `recalled: []`); corregida la afirmación del task file y documentado el borde en `docs/api/MCP.md` |
| 6 | HIGH | El aviso recomendaba `VANTADB_INGEST_PROVIDER`, que limpia `degraded` y **apaga el aviso sin drenar nada** — reintroduce la ceguera de MEM-72 | ✅ Texto reescrito: nombra los dos huecos por separado (runner vs scheduler) y no presenta la env var como solución |
| 7 | MED | El callback de `context` no tenía `try/catch` (el `try` de `setup` no cubre un callback diferido) | ✅ Cuerpo completo guardado; un fallo del hook no puede romper el request |
| 8 | MED | El gate estaba mockeado justo en las costuras que fallan: quitar `redactSecrets` dejaba 13/13 verdes | ✅ Fixture con una key real + asserts sobre `hookInput`; mutación C (quitar la redacción) ahora **falla** |
| 9 | MED | Redacción parcial: key sin `END` se posteaba entera; `password: hunter2` pasaba; `password: "correct horse battery"` dejaba residuo | ✅ Patrón de key sin terminar (hasta línea en blanco o fin), valor mínimo 6 (o quoted ≥4), cola citada cubierta. Tests de los tres casos |
| 10 | MED | Dos env knobs para una DB (`VANTADB_DB_PATH` vs `VANTADB_STORAGE_PATH`) sin cross-check | ✅ Aviso único si difieren: capturaría en un store que nadie lee |
| 11 | MED | Orden de resolución del binario contradice la topología (writer global 0.9.0 vs `target/`) | ⚠️ Trade-off: invertir a PATH-first rompería todo si el global no tiene `mcp-call` (0.9.0). Se mantiene el orden y se **anuncia el binario que responde** (una vez por proceso) para que la deriva sea visible |
| 12 | MED | `slice(0,8000)` parte pares surrogados ⇒ `serde_json` rechaza el stdin y el turno se pierde | ✅ `truncate()` en frontera de code point + test con emoji; el comentario ya no finge que 8000 es un límite de wire (es un tope de producto) |
| 13 | MED | Caché de recall por texto: nunca se invalida | ⚠️ Trade-off aceptado (idempotente por texto; un acierto repetido es un acierto). El caso "cachea `null` de un blip" ya estaba cerrado. Marcado con `ponytail:` |
| 14 | MED | `posted` se marcaba con `sent > 0` ⇒ captura a medias nunca reintentada; la clave no era la documentada | ✅ `posted` solo si se enviaron **todas** las entradas; clave documentada como assistant id → user id → texto |
| 15 | LOW | `findThreadByTitle` sin cota real; threads sin TTL | ⚠️ Páginas 10→3. TTL no se pone: la memoria sobrevive al purge del thread de todos modos y el TTL cambiaría la retención (decisión ajena) |
| 16 | LOW | `recallBlock` inyecta cuando `recalled` está *ausente* (no solo no-vacío) | ⚠️ Trade-off: ausente ≠ "sin hits" (respuestas de gateway que solo traen `prepend_context`). `recalled: []` sí es explícito y gana siempre. Documentado |
| 17 | LOW | TDZ de `timer` en `runCli`: un `finish` síncrono dejaría la promesa sin resolver | ✅ `let timer` declarado antes de `finish` |

## Verification

| Comando | Resultado |
|---|---|
| `node .opencode/plugins/vanta-memory-capture.selftest.mjs` | ✅ **15/15 pass** (`fail 0`) — tras los fixes del review (13 antes) |
| `node --test .opencode/plugins/vanta-memory-capture.selftest.mjs` | ✅ 15/15 pass — mismo archivo, runner del repo |
| Mutación A (recall vacío devuelve un texto en vez de `null`) | ✅ **fallan 3** tests; revertido |
| Mutación B (la captura no lee el transcript ⇒ no postea) | ✅ **fallan 3** tests; revertido |
| Mutación C (quitar `redactSecrets` del posteo — el gate viejo no la veía) | ✅ **falla 1** test (`credentials are redacted before they are posted`); revertido |
| Smoke real del transporte (`./target/release/vanta-cli.exe --db <tmp> mcp-call --tool capabilities --args '{}'`) | ✅ `mcp-call` devuelve `{"content":[{"text":"<json>"}]}` (sin `structuredContent`) y `parseResult` lo desempaqueta; el payload real de un DB fresco **no trae bloque `ingest`** → `isIngestBroken()` = `false` (superficie sin instrumentar ≠ pipeline roto). Ambos shapes (envelope y `ingest` pelado) dan el resultado esperado. Los logs INFO del binario van a stderr y no contaminan el parse. DB temporal borrada |
| **E2E real** (plugin completo contra `vanta-cli` + DB temporal, `session.idle` inyectado) | ✅ `thread_list` → `thread_create` → **2× `thread_send`** → `thread_get` devuelve `role:"user"` con el texto del turno y `role:"assistant"` con la respuesta, en el thread `vanta-opencode-idle-ses_live`; el evento distinto de `session.idle` se ignora; un solo thread (sin fork). **El token de prueba (`ghp_…`) quedó como `token=[REDACTED:credential]` en lo almacenado**: la redacción se prueba contra el binario, no solo contra el mock. DB temporal borrada |
| **L0 capture (MCP-44) — NO verificable con los binarios locales** | ⚠️ Los binarios locales son **anteriores** a `2fb20c728`: `thread_send` responde `{"ok":true}` **sin bloque `memory`** y `memory_list {"namespace":"l0/<tid>"}` devuelve `records: []`, tanto en `target/release` (16:13) como en `target/debug` (17:53) — el fuente (`vantadb-mcp/src/threads.rs:236-239`) sí emite `memory:{captured,queued}`. La mitad transporte del plugin queda probada; la mitad servidor es de MCP-44 y no se toca acá (fuera de scope). El plugin solo usa `ok === true` para contar envíos, así que funciona con ambos shapes |
| `memory_recall` tras capturar | ✅ responde en plazo con `recalled: []` / `prepend_context: ""` (L1 vacío: no hay scheduler, MCP-43). `capabilities.ingest` = `null` en el proceso efímero del `mcp-call` |
| Latencia medida (release, spawn en frío por llamada) | `capabilities` 1,6-2,5 s · `memory_recall` 7,0-8,5 s · `thread_send` >5 s (con 5 s fallaba siempre) — origen de los dos presupuestos del §Spec fila 14 |
| `npx tsc --noEmit -p .opencode` | ⛔ **no ejecutable**: no existe `.opencode/tsconfig.json` y `tsc` no está instalado (ni local ni en PATH). `node` sí parsea el `.ts` (type-stripping) — sintaxis verificada, tipos no |
| `node scripts/docs/check-links.mjs` | ✅ **exit 0** — 0 broken markdown (el único restante es un `[[wikilink]]` presupuestado en `docs/dev/avance/historial/snapshot-2026-08-03.md`, intacto en el worktree y anterior a esta tarea) |
| `node scripts/docs/check-docs.mjs` | ✅ **exit 0** — sin violaciones de esquema; `MCP-47.md` aparece en la lista informativa de *orphans* (igual que `MCP-44.md`: los task files no reciben enlaces entrantes) |
| `node scripts/docs/gen-index.mjs --check` | ⚠️ **rojo por trabajo ajeno, no regenerado a propósito**: el índice ya estaba stale antes de esta tarea (`MCP-44` figura `_(in-progress)_` estando cerrado, y el primer mismatch es `FIND-98` vs `FIND-735`, un task file de **otra sesión** que además está sin trackear). Regenerarlo hornearía filas de `FIND-735`/`MEM-71` —un archivo *staged* por otra sesión, ausente en el tree de mi commit— dejando el índice apuntando a un archivo inexistente. Lo cierra el orquestador en el batch de la wave (§3.c del plan) |

## Invariantes de dominio (handoff — MUST)

- **Recall-first:** `recalled: []` ⇒ **no se inyecta nada**. "No tengo memoria de esto" es una respuesta válida; rellenar el hueco con invento es peor que el hueco.
- **Fail-open:** ningún camino del plugin puede lanzar hacia OpenCode. `setup`, el callback de `context` y el pump del stream tienen guardas propias; el transporte tiene dos presupuestos de timeout (lectura 12 s, escritura 15 s) y un breaker que apaga el recall tras 2 fallos de lectura.
- **Nunca argv:** el texto del turno **y** el query de recall viajan por stdin (`{{turn}}` / `{{query}}`). Si se cambia el transporte, esta propiedad no se negocia.
- **Solo partes `text`:** tool-calls y tool-results no se postean (llevan cuerpos de archivo y credenciales).
- **Sin fork de historial:** un envío fallido invalida el thread cacheado; nunca crea un segundo thread para el mismo turno (un `thread_id` nuevo es un `session_id` nuevo y L0 no puede dedupear).
- **Honestidad del pipeline:** mientras `MCP-43` no exista no hay scheduler en `--mcp`; el plugin **no** promete consolidación y el aviso separa "falta runner" de "falta scheduler".

## Lo que el plugin NO promete (borde honesto)

- Que la memoria **aparezca**: `thread_send` encola (`memory.queued: true`), y sin scheduler nadie drena la cola. Lo que sí garantiza: el turno quedó en L0 (persistente, medido con `thread_get` contra el binario release). Lo que **no** garantiza: que `capabilities.ingest` lo muestre — ese snapshot es del proceso que responde, y cada `mcp-call` es un proceso nuevo con cola vacía.
- Que el recall traiga algo la primera vez: L1 no existe hasta que corra un pass. `recalled` vacío ⇒ silencio, por diseño. **Medido**: tras capturar un turno real, `memory_recall` devuelve `recalled: []` — exactamente lo que el contrato del plan ("el contexto vuelve con recall no vacío") **no** puede cumplir hasta que `MCP-43` exista.
- Que el aviso de pipeline roto **se vea con este transporte**: `capabilities.ingest` describe *el proceso que responde*, y cada `mcp-call` es un proceso nuevo con cola vacía (`ingest: null` medido). El predicado y el texto son correctos; para que el aviso dispare hace falta leer la superficie desde un proceso que sostenga el backlog (sesión MCP viva, o `--mcp` cuando `MCP-43` le dé scheduler).
- Coste de latencia: el recall ocurre **dentro** del request de modelo. Con los 7-8,5 s medidos por `memory_recall`, el primer request de cada turno distinto paga esa espera (el resto del turno reusa el bloque cacheado). Es FIND-706 (sesión ONNX por proceso), no algo que este plugin pueda arreglar sin un transporte de sesión larga.
- Backfill tras reinicio: la cola es in-memory (ADR-0054). Los L0 sobreviven; el enqueue no.

## Deuda

- **Typecheck:** `.opencode/` no tiene `tsconfig` y `tsc` no está instalado; el `.ts` solo se valida por parsing y por el self-check. Añadir `tsconfig` + `tsc` a `.opencode/` (y un gate) es un cambio de harness, no de esta tarea.
- **`MODULE_TYPELESS_PACKAGE_JSON`:** al importar el `.ts` desde el self-check, `node` avisa y reparsea como ESM porque `.opencode/package.json` no declara `"type": "module"`. Cosmético (el loader de OpenCode no usa Node puro), pero es un warning por corrida.
- **Sin token budget propio:** el bloque inyectado respeta lo que `memory_recall` ya recorta (`byte_count`/`truncated`), pero el plugin no impone un techo adicional.
- **Transporte one-shot vs. latencia:** un `mcp-call` por operación paga arranque de server + ONNX en cada llamada. La alternativa (sesión MCP viva del plugin) no se tomó por el riesgo de fuga de procesos hijo; si FIND-706 no baja el arranque, esa decisión hay que revisarla con medición.
- **Caché por sesión sin cota** (`recalled`/`posted`): crece un entry por turno. Marcado con `ponytail:` en el código; añadir LRU si una sesión puede durar horas.
- **Binarios locales stale respecto a MCP-44:** el E2E no pudo observar `memory:{captured,queued}` ni una fila en `l0/<thread_id>` porque `target/{release,debug}/vanta-cli.exe` son anteriores a `2fb20c728`. Rebuild necesario para cerrar la mitad servidor del DoD; **no** se hizo acá (carril C, sin Rust, para no competir por el lock de `target/`). Candidato a `FIND-*` del orquestador: un binario local que no refleja `develop` hace que la verificación de cualquier tarea MCP-* mienta por defecto.
- **Gate H (`/harness`) pendiente:** la fila del Backlog lo exige por tocar `.opencode/`; esta tarea no lo corrió (el harness es repo separado y el cambio es un plugin nuevo, sin tocar agentes/comandos/skills). Queda para el cierre de la wave.
