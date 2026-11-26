---
title: "MEM-73: e2e del ciclo completo L0→L3 desde el binario shipped"
kind: task
status: completed
description: "Un proceso real (`vantadb-server --mcp`) demuestra el ciclo entero —turno → L0 → cola L1 drenada por el scheduler → L1/L2/L3 (persona) → `memory_recall` sin `memory_put`— con el runner real del binario y un modelo scripted declarado como tal"
---

# MEM-73: e2e del ciclo completo L0→L3 desde el binario shipped

## Metadata
- **Plan file:** `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` (§9.b, última de la ola MCP/MEM)
- **Tipo:** test e2e (Rust, `vantadb-mcp/tests/`) + lane explícita en `dev-tools/`
- **Estado:** ✅ COMPLETED
- **Creado:** 2026-11-25

## Techo y contrato

Toda la ola está cerrada (`MCP-42/43/44/47/48`, `MEM-71/72`, `WIRE-19`), pero cada
capa se probó por separado:

| Capa | Probada por | Cómo |
|------|-------------|------|
| L0 captura + 1 L1 por turno | `MCP-44` | test in-process (`src/threads.rs::memory_feed_tests`) |
| Drenaje de la cola | `MCP-43` | test in-process con tick del scheduler |
| Consolidación L1→L3 | `MEM-71` | `run_pass` con runner scripted |
| Degradado visible | `MEM-72` | `capabilities.ingest` |

Lo que **MEM-73** añade es la **composición en un proceso real**: el binario que se
publica, arrancado con `--mcp`, hablando JSON-RPC por stdio, sin tocar APIs internas.

### Contrato mecánico

El comando que **prueba MEM-73** es el carril ignorado (el e2e no corre en el gate
por diseño, ver §Spec decisión 3):

```
cargo nextest run --profile audit -p vantadb-mcp -E 'test(e2e_l0)' \
  --run-ignored ignored-only --build-jobs 2
# o, con build + gate de SHA:
pwsh dev-tools/e2e-memory-cycle.ps1
```

Gate de regresión del mismo filtro que la ola usa en CI (corre las capas, **no**
el e2e — el e2e queda *skipped* por `#[ignore]`):

```
cargo nextest run --profile audit -p vantadb-mcp -E 'test(e2e) or test(cycle) or test(l0)' --build-jobs 2
```

### Qué NO cubre (declarado, no escondido)

- **Durabilidad entre restarts** (`FIND-745`, sin backfill L0→cola tras restart): fuera
  de contrato. El e2e corre en una vida de proceso.
- **Calidad del modelo**: no hay ollama local ni `VANTADB_OPENAI_API_KEY` en esta
  máquina. El **runner es real** (`ConcreteRunner::Ollama` construido por el binario
  desde `VANTADB_INGEST_*`, HTTP real de `StandaloneLlmRunner`); el **modelo** es un
  endpoint scripted. Se prueba la composición, no la calidad.
- **Recall vectorial**: sin proveedor de embeddings, `memory_recall` cae a keyword
  (los WARN `embedding unavailable — keyword fallback` de la corrida lo dicen). El
  ciclo se prueba en keyword.

## Hallazgos de componer (por qué valen más que el e2e)

1. **L3 no necesita 50 memorias.** La premisa del brief (`PERSONA_TRIGGER_EVERY_N = 50`)
   no aplica al ciclo real: el trigger P2 *cold start* (`scenes_processed > 0 &&
   !previously_generated && has_scene_blocks`, `core/persona/persona_trigger.rs:71-78`)
   gana después del **primer** L2. `docs/api/VANTA_MEMORY.md:334-336` ya lo documenta
   ("plus the cold start / recovery / first-scene triggers"), o sea que no hay drift de
   docs. → ni 50 turnos a mano ni bajar la cadencia: basta un turno que rinda ≥5
   memorias.
2. **El umbral real del ciclo es 5, no 50.** `MemoryTaskHandler::auto_l2_threshold = 5`
   (`services/pipeline_worker.rs:397`, TEC-18) es lo que dispara `L1 → L2`; sin 5
   memorias en **una** corrida de L1 no hay L2 y por lo tanto tampoco L3. Es una
   condición de *una pasada*, no acumulativa entre turnos.
3. **Ruido ERROR garantizado en cada L2.** `extract_and_apply_bitemporal_triplets`
   (MEM0-04, llamado desde `run_l2_inner` con `let _ =`) emite `ERROR add_edge ... Node
   not found: <id>` por cada arista `derived_from` de cada tripleta: los ids de los
   registros L1 no existen como nodos del grafo. No rompe nada (el resultado se
   descarta), pero **todo L2 ensucia el log con ERROR** — y en `--mcp` ese log va al
   stderr del cliente. Fuera del scope de MEM-73 (código de `vanta-memory`, no del
   test): queda como fila `FIND-*` para el lead.
4. **El scheduler arranca antes del primer tick.** `scheduler_running: true` es cierto
   desde `start()` (nota explícita en `memory_scheduler.rs:130-134`); `last_pass`
   distingue "no tickeó todavía" de "tick con runner". El e2e asserta ambos, en ese
   orden.
5. **La trampa de FIND-741 es real y se vio en vivo.** Con el binario anterior
   (`commit e8df96232` vs HEAD `4af909977`) el e2e **falla en el pre-flight** con el
   mensaje que nombra ambos SHAs. Después de rebuildear (`commit 4af909977`) pasa. Un
   e2e sin ese gate habría corrido contra el contrato viejo.
6. **`capabilities.ingest.queue.depth` es una muestra del último pass, no un gauge
   vivo (en el camino sano).** `note_pass`/`note_skipped_pass` guardan el snapshot *al
   final* de cada pass (`ingest/observability.rs:184-198`). Justo después de
   `thread_send` la superficie sigue reportando la muestra anterior (0) aunque haya un
   L1 encolado; con el default de 5000 ms eso significa hasta 5 s de "depth 0" con
   trabajo pendiente. El campo se documenta honestamente ("as of the last pass"), pero
   un agente que lea `queue.depth` como "no hay nada que drenar" se equivoca. No rompe
   el ciclo (el efecto lo prueba), y arreglarlo es decisión de `vanta-arch`
   (¿probe vivo en `capabilities`, o dejarlo como snapshot?). Fila `FIND-*`
   candidata para el lead. **Corolario:** el "assert 1→0" sugerido en el review
   P2-01 no es implementable por esta vía (la superficie devuelve 0 hasta el
   siguiente pass); el drenaje se prueba por efecto, no por transición observada.
   **Precisión (re-review):** no es uniforme — en el camino **degradado** el loop sí
   sondea la cola en vivo (`note_skipped_pass` recibe `probe.queue_depth()`,
   `services/scheduler.rs:356-370`; ídem `threads.rs::publish_backlog_if_degraded`).
   O sea: sano = snapshot cadence-bound; degradado = probe vivo al tick.
7. **`memory_recall` decora el `content`.** El hit vuelve como `"<memoria>
   (activity time: <RFC3339>)"` y `prepend_context` agrega `[<type>|<scene>] `.
   Cualquier assert de igualdad exacta contra la memoria extraída falla por diseño
   (lo descubrió este test al endurecerlo); `starts_with(<memoria extraída>)` sí
   distingue extraído de turno crudo. Vale para futuros tests de recall.

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos o en la zona citada):**
  `vantadb-mcp/src/memory_scheduler.rs` (393L), `vantadb-mcp/src/threads.rs` (633L),
  `vanta-memory/src/services/scheduler.rs` (`run_pass`, `spawn_memory_scheduler*`),
  `vanta-memory/src/ingest/observability.rs` (L68-236), `vanta-memory/src/ingest/runner_config.rs` (L1-303),
  `vanta-memory/src/adapters/standalone/llm_runner.rs` (251L),
  `vanta-memory/src/services/pipeline_worker.rs` (L230-349, L500-739),
  `vanta-memory/src/core/persona/{persona_trigger.rs, persona_generator.rs}` (L1-400),
  `vanta-memory/src/core/record/{l1_extractor.rs, l1_dedup.rs}`,
  `vanta-memory/src/core/scene/scene_extractor.rs` (L380-520),
  `vanta-memory/src/core/abstractions/llm_runner.rs` (L60-239),
  `vantadb-mcp/src/{config.rs, scenes.rs, handlers/tools.rs (memory_recall/memory_get)}`,
  `vantadb-server/src/{main.rs, build.rs}`, `vantadb-mcp/tests/{thread_memory_pipeline.rs, capabilities_ingest.rs, demo_ai_ides.rs}`,
  `docs/api/VANTA_MEMORY.md` (§scheduler, L325-352).
- **Referencias hacia dentro de lo tocado:** el test nuevo sólo consume superficie pública
  (`vantadb_mcp::{handle_tools_call…}` no: usa el **binario**; del crate sólo tipos de
  `vanta-memory` para nada — el test no linkea símbolos nuevos). `dev-tools/e2e-memory-cycle.ps1`
  es entrada manual; nadie lo importa.
- **Referencias entrantes / blast radius de producción:** **ninguna**. No se edita código
  de producción: `vantadb-mcp/src/**` y `vanta-memory/src/**` quedan intactos (no apareció
  bug real que justificara tocarlos).
- **Veredicto:** BAJO. Dos archivos nuevos (`tests/`, `dev-tools/`), cero cambios en
  `src/**` del core, cero cambios de contrato.

## Spec (decisiones)

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Test unitario vs proceso real | A) extender `memory_feed_tests` in-process (ya cubre L0→L1; no prueba el binario) / B) spawn del binario shipped | **B** — la ola ya tiene A; lo que falta es la composición en el artefacto |
| 2 | Runner real vs scripted | A) exigir `VANTADB_OPENAI_API_KEY`/ollama (no hay en la máquina → el e2e nunca corre) / B) runner real del binario + endpoint HTTP scripted | **B** — el camino env→`ConcreteRunner`→HTTP→parseo es el real; sólo el modelo es un doble (declarado) |
| 3 | Gate por defecto vs `#[ignore]` | A) gate (2.5s, pero arranca procesos y un LLM local) / B) `#[ignore]` + lane `dev-tools/` | **B** — FIND-727/739 son exactamente un e2e frágil envenenando el gate |
| 4 | 50 memorias para L3 | A) 50 turnos a mano / B) bajar `PERSONA_TRIGGER_EVERY_N` / C) un turno con 5 memorias (cold start) | **C** — evidencia: P2 cold start gana; 50 es la cadencia de *regeneración*, no la de la primera |
| 5 | Verificación contra el binario | A) confiar en `target/` / B) `--version` + SHA vs HEAD | **B** (FIND-741) — demostrado: el binario stale falla y dice por qué |

## Verificación

```
cargo nextest run --profile audit -p vantadb-mcp -E 'test(e2e_l0)' --run-ignored ignored-only --build-jobs 2
  → PASS  (5 corridas: ciclo 1.39–1.67 s, wall 3.5–4.2 s)
  → stderr: "MEM-73 e2e: L0→L3 in 1.4303061s (pass ran, queue drained, scene=1, persona=true)"
binario previo (commit e8df96232) → FAIL en pre-flight "stale binary … HEAD is 4af909977"
binario rebuildeado (commit 4af909977) → PASS
`pwsh dev-tools/e2e-memory-cycle.ps1` → build → version gate → PASS
`pwsh scripts/validate-docs-coverage.ps1` → 0 gaps; clippy `-D warnings` + fmt `--check` ✅
gate de regresión `-E 'test(e2e) or test(cycle) or test(l0)'` → 2 passed
  (`wiki_roundtrip_e2e::e2e_ingest_then_wiki_tools_roundtrip` +
  `threads::memory_feed_tests::thread_send_captures_l0_enqueues_one_l1_task_and_recall_returns_the_turn`;
  el e2e de MEM-73 queda *skipped* por `#[ignore]`, por diseño — corre en el carril)
```

El ciclo NO se asserta contra reloj (sería `FIND-739`: una aserción que mide el
reloj y mata una corrida cuyo trabajo ya terminó). Se asserta el **efecto** dentro
de un budget de 20 s y se loguea el elapsed. El "≤5 s" del contrato se cumple con
holgura (1.39–1.67 s con tick de 1500 ms; 0.2 s con tick de 100 ms), pero es un SLO
observado, no un gate.

## Review (GATE — agente distinto, P2-01)

- **Revisor:** `vanta-review` (contexto fresco, sin autoría sobre el cambio).
- **Enfoque:** cinco ejes de ataque sobre el falso-verde — (a) ¿puede pasar sin que
  el scheduler drene?, (b) ¿contra un binario stale?, (c) ¿puede `memory_recall`
  satisfacerse con L0 o con el thread?, (d) ¿el `last_pass.skipped:false` enmascara
  un fallo?, (e) ¿el LLM scripted enmascara algo / acopla a un umbral frágil?,
  (f) fugas/hangs del proceso hijo.
- **Cómo se probó:** lectura completa del test + traza de la cadena
  (`threads.rs` → `memory_scheduler.rs` → `services/scheduler.rs` → `pipeline_worker.rs`
  → `persona_trigger.rs` → `auto_recall.rs`), y **ejecución independiente** del
  binario de test dos veces (1.64 s / 1.40 s) contra el server que declara
  `commit 4af909977`.
- **Veredicto:** 🔴 REQUEST CHANGES → **3 required aplicados** en esta revisión:
  1. Este bloque de Review faltaba con `status: completed` → agregado.
  2. `assert!(elapsed < 5s)` era `FIND-739` renacido → eliminado; el budget pasó a
     20 s y el elapsed se loguea (el contrato es el efecto).
  3. El "Contrato mecánico" nombraba un comando que **no corría** el e2e → el carril
     ignorado es ahora el contrato y el filtro de CI quedó como gate de regresión.
- Opcionales aplicados: `starts_with` en vez de `contains` (descubrió el finding 7),
  y el fake ahora **paniquea** ante un system prompt desconocido en vez de responder
  un payload L1 a cualquier etapa nueva. No aplicado (con razón documentada, finding
  6): el assert "cola 1→0" — la superficie no es un gauge vivo.
- **Resultado tras los cambios:** e2e PASS ×5, clippy/fmt verdes.
- **Re-review (contexto fresco):** ✅ **approve** — 3/3 required verificados contra
  fuente y por ejecución independiente (1.6357 s); opcionales: 2 aplicados, 1 no
  aplicado con razón (finding 6). Precisión aportada por el revisor al finding 6:
  la superficie es **snapshot en el camino sano** (cadencia = tick) y **probe vivo en
  el camino degradado** (`note_skipped_pass` sí sondea la cola); el corolario para
  este test (camino sano) no cambia.

## Artefactos

- `vantadb-mcp/tests/l0_l3_cycle_e2e.rs` — el e2e (ignorado) + preflight `--version`.
- `dev-tools/e2e-memory-cycle.ps1` — carril explícito (build → version gate → e2e).
