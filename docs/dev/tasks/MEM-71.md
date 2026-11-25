---
title: "MEM-71: `run_bridge_pass` deja de anular el ciclo L3"
kind: task
status: in-progress
description: "El pass documentado para hosts del bridge (`conversation_hook`) fijaba `trigger_every_n = usize::MAX` y no cableaba la cola: un host que siguiera el patrón nunca llegaba a L2/L3. Decisión: el pass del bridge ES `scheduler::run_pass` (una sola implementación, persona a `PERSONA_TRIGGER_EVERY_N`), con la cadencia expuesta y fijada por test"
---

# MEM-71: `run_bridge_pass` deja de anular el ciclo L3

## Metadata
- **Plan file:** `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` (§9.b, ola MCP/MEM)
- **Fuente:** Backlog `MEM-71` — `vanta-memory/src/services/conversation_hook.rs:122-128` · `services/scheduler.rs:58`
- **Esfuerzo:** 🟡 2-4h
- **Tipo:** Rust (bug-fix de wiring + fix de API pública de `vanta-memory`)
- **Estado:** ⏳ IN PROGRESS
- **Creado:** 2026-11-25
- **Dependencias:** `MEM-72` ✅ (superficie `ingest::observability`) · `FIND-290` abierta y **adyacente, no tocada aquí**
- **SDP (v3, phase=BUILD):** `campaign-executor` · `progreso` · `ponytail` · `source-driven-development` · `doubt-driven-development` · `incremental-implementation` · `test-driven-development` · `context-engineering` · `documentation-skill` (Regla 3, doc tocada)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers de `run_bridge_pass` | **Cero productivos.** `vantadb-server` (WIRE-16) usa `scheduler::run_pass` desde `vantadb-server/src/scheduler.rs`; los únicos callers son `vanta-memory/tests/conversation_hook.rs:137,172,194,204` |
| Callers de `PERSONA_TRIGGER_EVERY_N` | `services/scheduler.rs::run_pass` (único); pasa a `pub` (aditivo) para que tests/hosts referencien la cadencia sin duplicar `50` |
| Callees de lo editado | `PipelineWorker::{run_once,reclaim_stale}`, `TimerScanner::run_once`, `MemoryTaskHandler::new`/`with_queue`, `evaluate_persona_trigger` |
| Implicaciones | `run_bridge_pass` pasa a devolver `PassStats` (antes `RunStats`): gana `timers_fired` + `reclaimed`, y **ejecuta** esos dos pasos. Es un cambio de firma en un crate aún no publicado (`DIST-01`: `release-plz` en hold). El comportamiento resultante es el de `run_pass`, ya en producción en `vantadb-server`. Sin migración de datos; la cola es in-process (ADR-0054). Sin locks nuevos. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos o en la zona editada):** `vanta-memory/src/services/conversation_hook.rs` (107L, completo), `vanta-memory/src/services/scheduler.rs` (L1-145), `vanta-memory/src/ingest/observability.rs` (325L, completo), `vanta-memory/src/core/persona/persona_trigger.rs` (L1-116), `vanta-memory/src/services/pipeline_worker.rs` (L56-73, L210-350, L361-560, L700-750), `vanta-memory/src/utils/checkpoint.rs` (L277-296), `vanta-memory/tests/conversation_hook.rs` (180L, completo), `vanta-memory/tests/scheduler.rs` (L1-150), `vanta-memory/tests/e2e_flow.rs` (L36-225), `docs/api/VANTA_MEMORY.md` (L315-384), `docs/dev/architecture/adr/ADR-0054-scheduler-host-vantadb-server.md` (L1-120), `.opencode/rules/core-engine.md`, `vanta-memory/Cargo.toml` (features).
- **Referencias hacia dentro de `run_bridge_pass`:** `tests/conversation_hook.rs:137,172,194,204`; `docs/api/VANTA_MEMORY.md:331-332`; `ADR-0054:31,92` (declara que "se generaliza al pass del driver"). Grep repo-wide (`--glob *.rs`) no encuentra ningún caller fuera de tests.
- **Referencias salientes:** `PipelineWorker::run_once` (batch 8), `MemoryTaskHandler::new` + `with_queue`, `L1ExtractorConfig/L1DedupConfig::default`, `TimerScanner::run_once`.
- **Veredicto impacto:** BAJO-MEDIO. 2 funciones + 1 `const` → `pub` + 2 tests + 1 párrafo de doc. Pre-mortem: (a) "arreglarlo" sin saber si era intencional → resuelto abajo con evidencia de historial y ADR; (b) axiomas basura por cadencia demasiado baja → la cadencia NO se baja: se unifica en `PERSONA_TRIGGER_EVERY_N = 50` (la que ya corre en producción en el server) y el test la fija; (c) duplicar `FIND-290` (reflexión sin promotion) → fuera de scope, ni se toca.

## Contrato

```
cargo nextest run --profile audit -p vanta-memory -E 'test(bridge) or test(persona) or test(pass)' --build-jobs 2
```

La mitad `test(bridge)` vive tras `feature = "http-server"` (`tests/conversation_hook.rs`), así que el contrato se corre **también** con `--features http-server` para que ese test exista y ejecute:

```
cargo nextest run --profile audit -p vanta-memory --features http-server -E 'test(bridge) or test(persona) or test(pass)' --build-jobs 2
```

Más un test que demuestre el comportamiento elegido: (1) con el patrón del hook, el pass llega a L2/L3 y escribe persona; (2) la cadencia es `PERSONA_TRIGGER_EVERY_N` (=50) — no dispara en `N-1` y dispara en `N`, con prioridad `P4MemoryCount`.

## Hallazgo de investigación (¿intencional o descuido?)

| Evidencia | Qué dice | Lectura |
|---|---|---|
| `git log -S 'usize::MAX' -- vanta-memory/` → `49ea072ec` (MEM-55) | El doc comment de MEM-55 dice: "a conversation-add pass extracts memories, it does not regenerate personas" | El `usize::MAX` fue **deliberado en su momento**: el alcance de MEM-55 era L1 |
| `git log --oneline -- vanta-memory/src/services/conversation_hook.rs` → `06f8f84f8` | Un único commit toca el archivo desde su creación | Nadie revisó el patrón después |
| `ADR-0054:92` (aceptado 2026-10-01, **posterior**) | "`run_bridge_pass` **se generaliza al pass del driver**" | La decisión de arquitectura ya había dejado al bridge pass como paso intermedio a fusionar con `run_pass` |
| `WIRE-16` / `docs/api/VANTA_MEMORY.md:352` + `vantadb-server/src/scheduler.rs` | El host productivo cablea el bridge y llama **`run_pass`** | La generalización ocurrió; `run_bridge_pass` quedó como API huérfana que el doc seguía recomendando |

**Conclusión:** intencional como alcance de MEM-55, **superado por ADR-0054 y huérfano desde WIRE-16**. No es un diseño "el bridge no debe tocar persona": es un resto del paso intermedio. Por eso no se cierra documentando la desactivación (opción c): se cierra **eliminando la divergencia**.

Además del `usize::MAX`, el bug real es más grande: el handler se construía **sin `with_queue`**, así que el cascade L1→L2 nunca ocurría. Aun con persona habilitada, un host del patrón documentado no llegaba nunca a L3 porque **nunca encolaba L2**. Un único `usize::MAX` no explica el ciclo muerto; la falta de wiring sí.

## Spec (SDD)

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Qué hacer con el pass del bridge | A) aceptar la cadencia como parámetro (deja **dos** passes: el del bridge sigue sin timers/reclaim/cascade) / B) delegar en `run_pass` (una sola implementación; el bridge hereda timers + reclaim + cascade + persona) / C) mantener desactivado + `persona_disabled_by_design` en `observability` (hace visible una decisión ya superada) | ✅ **B** — ADR-0054:92 ya decidió la fusión; A conserva el pass inferior y su doc, C documenta como diseño algo que el ADR dio por temporal. Coste de B: 1 línea de cuerpo |
| 2 | Firma de retorno | A) `PassStats` (honesto: el pass hace timers + reclaim) / B) seguir devolviendo `RunStats` con `.worker` (oculta `reclaimed`/`timers_fired`) | ✅ **A** — cero callers productivos, crate no publicado (`DIST-01`); B mentiría sobre lo que el pass hace |
| 3 | Cadencia | A) nueva constante propia del bridge (dos números que divergen) / B) exponer `PERSONA_TRIGGER_EVERY_N` (`pub const`) y usarla en ambos / C) bajarla para "que se vea" | ✅ **B** — mismo número que corre en producción; C es el pre-mortem (b): axiomas basura por regenerar sin material nuevo |
| 4 | ¿Campo nuevo en `observability` (MEM-72)? | A) añadir `persona_disabled_by_design` / B) no añadir | ✅ **B** — con B de la fila 1 la persona **sí** se dispara; un flag que siempre dice `false` es deuda. `last_pass` ya publica `processed/timers_fired`, que ahora incluyen el ciclo completo |
| 5 | Alias o borrado de `run_bridge_pass` | A) borrarla (1 caller productivo cero, pero es API pública documentada en `VANTA_MEMORY.md` y `ADR-0054`) / B) alias de 1 línea que delega en `run_pass` | ✅ **B** — conserva el nombre que usan los hosts del hook y el doc pasa a ser verdad; borrar deja el párrafo de la doc sin sujeto |

## Steps

| # | Step | Estado |
|---|------|--------|
| 1 | Task file con Spec + Regla 0 + contrato | ✅ |
| 2 | RED: test del patrón del hook que exige L2/L3 (falla con el handler sin `with_queue`) | ✅ |
| 3 | GREEN: `run_bridge_pass` delega en `run_pass`; `PERSONA_TRIGGER_EVERY_N` → `pub` | ✅ |
| 4 | Test que fija la cadencia (N-1 no, N sí, prioridad P4) | ✅ |
| 5 | Doc: `docs/api/VANTA_MEMORY.md` § conversation_hook | ✅ |
| 6 | Verify contrato (con y sin `--features http-server`) + fmt + clippy scoped | ✅ |
| 7 | Review P2-01 (`vanta-review`) + commit local | ✅ |

## Verification

| Comando | Resultado |
|---|---|
| RED `cargo nextest ... --features http-server -E 'test(bridge_pass_reaches_l3)'` (pre-fix) | ❌ `panicked at conversation_hook.rs:146: the bridge pass must reach L3 and write the persona` — falla por la razón correcta (el pass no llega a L2/L3) |
| `cargo nextest run --profile audit -p vanta-memory -E 'test(bridge) or test(persona) or test(pass)' --build-jobs 2` | ✅ 61/61 (688 skipped) |
| `cargo nextest run --profile audit -p vanta-memory --features http-server -E 'test(bridge) or test(persona) or test(pass)' --build-jobs 2` | ✅ 64/64 (694 skipped) — incluye los tests del bridge |
| Mutación de control (P2-01 Required-1): `PERSONA_TRIGGER_EVERY_N` → `usize::MAX` en `run_pass` | ✅ el test dispara: `left == right failed: at PERSONA_TRIGGER_EVERY_N memories the pass must regenerate the persona` (0 passed, 1 failed); fuente revertida (`git diff --stat` = 5+/1-) |
| `cargo clippy -p vanta-memory --all-targets --features http-server -- -D warnings` | ✅ limpio |
| `cargo fmt -p vanta-memory -- --check` | ✅ limpio |
| `node scripts/docs/check-links.mjs` | ✅ exit 0 (0 markdown rotos; 1 wikilink histórico dentro de presupuesto) |
| `node scripts/docs/check-docs.mjs` | ✅ exit 0 |
| `pwsh dev-tools/ocr-review.ps1 -Format json` + `ocr delegate rule` sobre los 2 `.rs` | ✅ aplicado (grupo Rust; sin Critical/High propios) |

## Resultado

`run_bridge_pass` dejó de ser un pass reducido: es `scheduler::run_pass` (una
implementación, un comportamiento). El patrón que documenta el módulo —cablear
`HttpCaptureBridge` y llamar `run_bridge_pass`— ahora ejecuta el ciclo completo
(timers + una tanda del worker + reclaim), y la cascada L1 → L2 → L3 se encola
por la misma cola con la persona a `PERSONA_TRIGGER_EVERY_N = 50`.

El bug era más grande que el `usize::MAX`: el handler se construía **sin
`with_queue`**, así que el cascade L1→L2 nunca ocurría y L3 era inalcanzable
aunque la persona estuviera habilitada. El test nuevo lo prueba de punta a
punta: un turno capturado por el bridge, **un** `run_bridge_pass`, 5 memorias
extraídas (cruzan el umbral L2 = 5), escena, persona; `stats.worker.processed ==
3` y cola drenada.

El `usize::MAX` **no** era un diseño "el bridge no toca persona": fue el alcance
de MEM-55 (solo L1) y quedó superado por ADR-0054:92 ("`run_bridge_pass` se
generaliza al pass del driver"), generalización que WIRE-16 ya ejecutó en
`vantadb-server`. Por eso se cierra eliminando la divergencia, no documentando
la desactivación.

## Cierre — texto para `progreso` (lo aplica el orquestador)

`docs/dev/Backlog.md` y `docs/dev/avance/**` están fuera de scope de esta tarea: se devuelve el texto, no se editan.

**1. Backlog:** eliminar la fila `MEM-71` (`docs/dev/Backlog.md:701`).

**2. Entrada para `docs/dev/avance/activo/vanta-memory.md`** (formato de la entrada de `MEM-72`):

```markdown
### El pass del bridge anulaba el ciclo L3 (`MEM-71`)

- **Fecha:** 2026-11-25
- **Objetivo:** `run_bridge_pass` —el pass que documenta `services::conversation_hook` para hosts del `/conversation/add`— construía un `MemoryTaskHandler` con `trigger_every_n = usize::MAX` **y sin `with_queue`**, así que un host que siguiera el patrón documentado se quedaba en L1: L2 nunca se encolaba y L3 era inalcanzable, con la decisión enterrada en un `usize::MAX` que nadie podía observar.
- **Resultado:** el pass del bridge **es** `scheduler::run_pass` (una implementación, un comportamiento): timers vencidos, una tanda del worker cuya cascada L1 → L2 → L3 se encola por la misma cola, y reclaim de claims vencidos; la persona L3 se regenera a `PERSONA_TRIGGER_EVERY_N = 50` (ahora `pub`, para que hosts y tests referencien el número en vez de duplicarlo). ¿Intencional? No como diseño: fue el alcance de `MEM-55` (solo L1) y quedó superado por `ADR-0054:92` ("`run_bridge_pass` se generaliza al pass del driver"), generalización que `WIRE-16` ya ejecutó en `vantadb-server` — `run_bridge_pass` tenía **cero** callers productivos.
- **El bug era mayor que el `usize::MAX`:** sin `with_queue` el cascade L1→L2 no ocurría nunca, así que habilitar persona no habría bastado. El test lo prueba de punta a punta: turno capturado por el bridge → **un** `run_bridge_pass` → 5 memorias (cruzan el umbral L2 = 5) → escena → persona, `stats.worker.processed == 3`, cola drenada. La cadencia se fija en dos capas: la función pura (`N-1` no dispara, `N` dispara con `P4MemoryCount`) y el **cableado** — el checkpoint sembrado en 49 memorias no regenera con `run_pass`, y a 50 resetea el contador; con `usize::MAX` ese segundo test falla (probado por mutación).
- **Verificación:** `nextest --profile audit -p vanta-memory -E 'test(bridge) or test(persona) or test(pass)'` **61/61** · ídem con `--features http-server` **64/64** (los tests del bridge viven tras ese feature) · `clippy -p vanta-memory --all-targets --features http-server -- -D warnings` limpio · `cargo fmt -p vanta-memory -- --check` limpio · `check-links`/`check-docs` exit 0 · markdownlint (config CI) 0 issues · OCR review aplicado · review P2-01 adversarial `vanta-review` (changes-required → corregido → **approve**).
- **Commits:** `<hash>` (código + tests + docs + task file)
- **Techo documentado:** `run_bridge_pass` queda como alias de una línea (borrarlo dejaría el párrafo de `VANTA_MEMORY.md` y `ADR-0054:31` sin sujeto); un host que quiera un pass "solo worker" se construye con `PipelineWorker` + `MemoryTaskHandler`, no reintroduciendo un segundo pass público. No se añade campo a `ingest::observability`: con el pass unificado la persona ya no está desactivada, así que `persona_disabled_by_design` sería siempre `false`.
```

## Review (P2-01, adversarial — `vanta-review`, contexto fresco)

Veredicto inicial `changes-required` (0 Critical · 2 Required · 1 Optional · 4 Nits). Corregidos en esta ronda:

| Hallazgo | Fix |
|---|---|
| **[Required]** El cableado de `PERSONA_TRIGGER_EVERY_N` en `run_pass` no lo cubría ningún test: volver a `usize::MAX` dejaba la suite verde (el e2e del bridge escribe persona por **P2 cold start**, y el otro test ejercita la función pura) | Test `run_pass_regenerates_persona_exactly_at_the_fifty_memory_cadence`: siembra el checkpoint (`mark_persona_generated` + 49 memorias), encola un L3, corre **un** `run_pass` y exige que a 50 el contador se resetee (P4 disparó). Probado por mutación: con `usize::MAX` el test falla; revertido |
| **[Required]** Cross-ref circular (`conversation_hook` ↔ `scheduler` apuntándose a un patrón que este cambio borró) y falta la advertencia single-writer en la doc host-facing | Ambos docs apuntan a construir `PipelineWorker` + `MemoryTaskHandler` (forma en `tests/e2e_flow.rs`); módulo y función de `conversation_hook` avisan "una sola pasada por proceso: si ya corres `spawn_memory_scheduler`, no uses esto — mismo owner `worker-{pid}`" |
| [Optional] `processed == 3` exacto puede romperse con un paso de cascada futuro | Se mantiene el exacto: es el canario anti-duplicado (un doble procesamiento daría 4+) y hoy es determinista |
| [Nits] refs de línea stale en `VANTA_MEMORY.md` y en este task file; "sin `unwrap`" ambiguo; cadencia leída como exclusiva | Refs actualizadas a líneas reales; invariante aclarado a **código productivo**; la doc nombra también cold start / recovery / first scene |

**Re-verdicto (mismo reviewer, sobre el estado final): `approve`** — 0 Critical / 0 Required / 0 Optional. Verificó `Required-1` por mutación propia (`PERSONA_TRIGGER_EVERY_N` → `usize::MAX` en `scheduler.rs:119` → `left: 50, right: 0` en el test de cableado; revertido, hash de archivo idéntico), confirmó que P4 es el único trigger posible con el checkpoint sembrado (`scenes_processed == 0`, sin scene blocks) y que `memories_since_last_persona` solo se resetea tras una generación exitosa (`pipeline_worker.rs:745-748` → `checkpoint.rs:177`). Nits residuales (conteos del bloque de handoff, refs `:51-53`, "shape" incompleto de `e2e_flow.rs`) corregidos antes del commit.

## Invariantes de dominio (handoff — MUST)

- **Principio 4:** ningún pass puede perder L0 ni quemar tareas; el cascade L1→L2 solo se encola con `stored_count >= auto_l2_threshold` (5) y L2→L3 siempre; L3 sin trigger es `Ok` silencioso.
- **Single writer (ADR-0054):** el pass corre en el proceso dueño de la DB. MCP-48 ya garantiza que el proxy re-elige writer; este cambio no encola en procesos ajenos.
- **Una sola cadencia:** `PERSONA_TRIGGER_EVERY_N = 50` sigue siendo el único número; no se introduce un segundo.
- **Sin `unwrap`/`expect` en código productivo** (los tests usan `#![allow(clippy::expect_used, clippy::unwrap_used)]` con invariantes documentados).
- **Sin dependencias nuevas.**

## Deuda

- `run_bridge_pass` queda como alias: si en el futuro un host quiere un pass "solo worker" (sin timers/reclaim) se construye con `PipelineWorker` + `MemoryTaskHandler` como ya se documenta en `scheduler.rs:48-56`; no se reintroduce como API pública.
- El flanco de `FIND-290` (reflexión sin `promotion` ni scheduler) sigue abierto: el ciclo L0→L3 ahora llega a L3 por ambos paths, pero la reflexión no promueve. Verificado que este cambio no lo toca.
