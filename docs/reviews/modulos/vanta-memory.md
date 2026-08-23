# Deep Module Review — `vanta-memory`

> **Fecha:** 2026-08-22 · **Revisor:** ox-alpha (segunda opinión, contexto fresco — P2-01)
> **Alcance:** ~90 archivos del crate. Leídos en profundidad: lib, core (abstractions/types+llm_runner, conversation/l0_recorder, record/l1_writer+l1_reader+l1_dedup, hooks/auto_capture+auto_recall, persona/persona_generator, state, memory_generation_log, skill/conversation_add), services/pipeline_worker, context_engine (engine, compressor, mmd*, token_estimator, types), offload (state_manager, reclaimer, storage, types, hooks/after_tool_call, local_llm/parsers), ingest (mod, worker, merge, auto_sync, callback, prompts), gateway/knowledge_handlers, seed (mod, input) + bin/vanta-seed, utils (local_backend, pipeline_manager, stateful_pipeline_manager, checkpoint, managed_timer, sanitize, text_utils, timer_scanner, pipeline_factory), adapters. `core/prompts/*` y prompts de skill: muestreados (no lectura completa, según instrucción).
> **Evidencia de compilación/tests (ejecutada en esta sesión):**
> - `cargo check -p vanta-memory --all-features` → ✅
> - `cargo test -p vanta-memory` → ✅ **23 suites, 0 fallos** (296 unitarios + 173 de integración ≈ 469 tests).

---

## 1. Veredicto ejecutivo

**Score: 8.5 / 10**

El mejor crate del proyecto desde el punto de vista de disciplina de ingeniería. Un principio rector (**P4: "el LLM es opcional; la pipeline nunca bloquea y nunca pierde datos"**) está aplicado de forma consistente y verificable en cada capa — L0 idempotente con cursor persistente, L1 dedup que degrada a `store-all` ante cualquier fallo del runner, L2/L3 que escriben nada antes que escribir algo corrupto, ingest LLM-free que marca skips sin morir. Los namespaces son sanitizados en cada frontera, los cursores hacen todo replay-safe, y hay ~469 tests pasando con cobertura real de contratos (no humo). Los hallazgos son menores: un gap de atomicidad delete+put en merges L1, tres managers de pipeline con solapamiento, y superficie expuesta hoy casi nula fuera de la API Rust.

---

## 2. Superficie expuesta hoy (verificada por código)

| Superficie | Estado | Evidencia |
|---|---|---|
| Crate library API (host-neutral) | ✅ módulos públicos: `core`, `utils`, `services`, `adapters`, `offload`, `context_engine`, `gateway`, `seed`, `ingest` | `lib.rs:25-49` |
| Binario `vanta-seed` | ✅ import seed JSON a store fjall o in-memory; requiere feature `fjall` para persistir | `bin/vanta-seed.rs` |
| HTTP / transporte propio | ❌ **ninguno** — por diseño ("gateway handlers sin transporte") | `gateway/mod.rs` |
| MCP tools scene_read/list/query | ⚠️ handlers puros implementados (`scene_read`/`scene_list`/`scene_query` en `gateway/knowledge_handlers.rs`) pero **sin ningún wrapper MCP ni registro**: grep en `src/` del core → 0 referencias. Hoy solo consumibles como funciones Rust | verificado con rg |
| Wiki store (F7) | ✅ **sí existe** en el core (`vantadb::wiki::WikiStore`, state machine pending→processing→ready/failed); el ingest de este crate lo consume vía SDK público | `ingest/worker.rs`, `src/wiki/mod.rs` |
| Endpoints HTTP del proxy que exponen memoria | ✅ solo indirectamente: proxy inyecta persona/scenas y ejecuta capture/search server-side | crate `vanta-proxy` |

**Conclusión de superficie:** hoy el crate se consume de dos formas — (1) embebido por `vanta-proxy` (inyección + tools L0/L1) y (2) como librería Rust para hosts. Los "12 tools MCP query-only" descritos en F7 no están cableados a ningún transporte; existe el 100% de la lógica handler pero 0% del transporte.

---

## 3. Arquitectura y patrones

### El patrón dominante: P4 (degradación total)
Cada módulo documenta y aplica la misma disciplina:

| Capa | Sin LLM / con error | Evidencia |
|---|---|---|
| L0 recorder | LLM-free por construcción; cursor persistente hace replay idempotente | `l0_recorder.rs:145-212` |
| L1 dedup | sin candidatos → all `store`; runner falla → all `store`; parse tolerante falla → `store` | `l1_dedup.rs:103-150` |
| L1 writer | embed hook falla → guarda sin vector (warn, nunca bloquea) | `l1_writer.rs:64-75` |
| L3 persona | output vacío/oversized → `success:false` y **escribe NADA** (nunca corrompe persona existente) | `persona_generator.rs:21-24` |
| Ingest | sin runner → fuentes skipped, commit determinista, build sigue | `worker.rs:196-217` |
| Offload state | payload corrupto → default + warn (catch-and-default documentado) | `state_manager.rs:48-60` |
| Reclaimer | timestamp imparsable → skip, nunca guess; sin cursor → reclaim nothing | `reclaimer.rs:99-116` |

Esto no es decorativo: los tests ejercitan cada rama de degradación (p. ej. `corrupt_state_payload_falls_back_to_default`, `cursor_update_preserves_other_state_fields`).

### Contratos y wire format
Tipos de dominio con serde snake_case que coinciden exactamente con el wire contract de los prompts LLM (`"type"`, `record_id`, `target_ids` — testeado roundtrip en `abstractions::tests`). Un solo tipo de error por capa (`L0Error`, `L1Error`, `OffloadError`, `RecallError`, `SeedError`, `IngestError`) envolviendo `VantaError`.

### Sanitización de fronteras
Todo namespace/key que toca el store pasa por `sanitize_component`/`sanitize_key` (charset `[A-Za-z0-9._/-]`, límites de bytes respetando char boundaries UTF-8). Consistente en l0/l1/offload/context/skills/persona/seed. La suite de sanitize (774 líneas, regex-free deliberado) cubre strip de tags de feedback-loop, base64, timestamps, detección de prompt injection (subset documentado de 16 patrones TDAM).

---

## 4. Flujos end-to-end

### Mensaje → L0 → L1 → L2 → L3 (+ contexto)

```
host → AutoCaptureHook.capture(session, msgs)
   → filtro roles → sanitize_text → strip fenced code
   → L0Recorder.record_turn:
        read_cursor(l0_cursor/<s>) → filter ts > cursor
        → dedup in-batch por key sanitizada
        → put l0/<s>/t{ts}_{i} → write_cursor(max_ts)
[trigger] MemoryPipelineManager.notify_conversation
   → warm-up 1→2→4…→every_n (capture_atomic: buffer+contador+tarea/timer atómicos bajo un lock)
   → LocalStateBackend.enqueue_task(L1, priority)
PipelineWorker.run_once:
   acquire_lock(pipeline_lock:<session>, TTL 60s)
   ├─ L1: read L0 messages → extract_l1_segments (LLM) → run_l1_dedup
   │      (recall top-k keyword⊕cosine RRF → 1 llamada batch → decisions)
   │      → apply_dedup_batch (put/delete por decisión) → checkpoint counters
   ├─ L2: read_session_records → extract_scenes_with_llm → scene_index upsert (META heat)
   ├─ L3: evaluate_persona_trigger(P1..P4) → generate_persona(first/incremental)
   │      → escape_xml_tags(boundaries) → append scene navigation → persist
   └─ post-L3: assemble_with_recall (compress→MMD→recall, budget compartido)
              → put context/<s>/__assembled
   release_lock (SIEMPRE antes de actuar sobre el outcome — anti-deadlock correcto)
```

Verificado contra `services/pipeline_worker.rs` + `e2e_flow.rs` (6 tests). El orden L3→context-assembly está afirmado por test e2e.

### Detalles finos que están bien resueltos
- **Cursor L0 en namespace separado** (`l0_cursor/<s>`): jamás aparece en `read_messages`; doble defensa con el check `__cursor`.
- **Ids deterministas** `m_{now_ms}_{idx}` + `TURN_SEQ`-style desambiguación; merge conserva `created_at` más antiguo, version = max(targets)+1, timestamps unión ordenada.
- **D38 dual-pool recall**: records con vector rankean por coseno, legacy por overlap, fusión RRF — un record sin vector jamás desaparece del pool.
- **MEM-48**: scores de compresión derivados de prioridad REAL de memorias L1 vinculadas por `source_message_ids` (join precomputado O(records+links)); fallback best-effort al heurístico de roles con warn.
- **Reclaimer**: solo borra estrictamente pre-cursor; el target del cursor siempre sobrevive (el cursor no puede colgar en datos GC-eados); `< 3 días` desactiva todo.

---

## 5. Lógica sospechosa (hallazgos)

**🟡 M-1 — Merge/update L1 no es atómico (delete targets → put merged).**
`l1_writer.rs:142-148`: primero borra todos los targets, después escribe el record fusionado. Si el proceso muere entre ambas operaciones, las memorias originales se pierden sin que exista la fusionada. Mitigación parcial: VantaDB puede tener journaling propio, pero el contrato del writer no lo garantiza. Fix barato: escribir el merged **primero**, luego borrar targets (deja duplicados transitorios en el peor caso — infinitamente mejor que pérdida), o un tombstone de 2 fases. Recomendado antes de confiar el crate a memoria de producción valiosa.

**🟡 M-2 — Race teórica en cursor L0 concurrente.**
`record_turn` hace read-cursor → filter → puts → write-cursor sin lock propio. Dos capturas concurrentes de la misma sesión pueden interleave (doble insert con keys distintas derivadas de índice de batch). En la práctica el worker serializa por sesión vía TTL lock, y el único otro productor (proxy) escribe a namespace distinto (`proxy-turns`). Riesgo residual solo si un host usa `AutoCaptureHook` multi-hilo sobre una sesión. Documentar la restricción de single-writer-per-session en el doc-comment sería suficiente.

**🟡 M-3 — Tres orquestadores de pipeline con solapamiento.**
`MemoryPipelineManager` (utils/pipeline_manager.rs), `StatefulPipelineManager` (stateful_pipeline_manager.rs) y `pipeline_factory` conviven; además `services::conversation_hook` es otra puerta de entrada. Cada uno tiene su razón histórica (MEM-16 vs MEM-55), pero un lector nuevo no sabe cuál usar. Consolidar o marcar uno como canónico en `utils/mod.rs`.

**🟢 N-1 — Comparación RFC3339 lexicográfica para detectar escenas cambiadas** (`updated > generated_at`): correcto SOLO porque ambos lados usan `epoch_ms_to_rfc3339` de ancho fijo. Frágil si alguien introduce otra fuente RFC3339 con offset distinto. Ya documentado en doc-comments; mantener vigilancia.

**🟢 N-2 — Worker corta el pase completo al encontrar tarea locked/retry** (`run_once` hace `break`, no `continue`): conservador y anti-starvation-documentado, pero reduce throughput del batch a 1 tarea por pase en escenarios contenciosos. Decisión consciente; revisar si el worker alguna vez corre caliente.

**🟢 N-3 — `LocalStateBackend` es volátil**: buffers, timers y cola de tareas mueren con el proceso (los datos L0-L3 persisten; solo se pierden triggers pendientes). Coherente con single-process scope, pero significa que un shutdown pierde conversaciones bufferizadas aún no capturadas si el host no flusheó. Vale un doc-comment explícito para hosts.

---

## 6. Completudes vs plan P27 F7 (segunda iteración)

Verificado contra `docs/plans/archive/2026-08-18-vanta-memory.md` (tabla DEFER/SKIP) y `2026-08-21-vanta-proxy-knowledge.md`:

| Ítem | Estado plan | Estado código | ¿Falta legítima? |
|---|---|---|---|
| Pipeline L0→L3 + triggers + skills + recall (F4) | DO | ✅ completo + tests | — |
| Context engine + offload + MMD (F5) | DO | ✅ budget compartido MEM-37/48 | — |
| Wiki ingest (MEM-30) + progress tracker (MEM-31) | DO | ✅ serial merge, STRUCTURAL_FILES protegidos, late-packet guard | — |
| Seed/import idempotente (MEM-39) + binario | DO | ✅ content-hash idempotente | — |
| Billing/quota | **DEFER** explícito ("server mode", plan P27) | ❌ ausente | ✅ DEFER legítimo |
| SDK sub-clientes (MEM-36) | **DEFER** a campaña bindings | ❌ ausente | ✅ DEFER legítimo |
| Prompts Kenty en chino | SKIP (reescritos a inglés) | ✅ prompts en inglés | ✅ SKIP ejecutado |
| Callback S2S destino (MEM-31 callback) | SKIP/DEFER (callback.rs existe como tracker local; destino remoto diferido) | ⚠️ parcial | ✅ documentado en plan |
| Transporte MCP para scenes/wiki (parte narrativa de F7) | Plan describe "12 tools MCP query-only" | ⚠️ handlers listos, transporte ausente | 🟡 **único gap real vs expectativa** — la lógica existe, falta el wiring (probablemente perteneciente a Studio/campaña aparte) |

No se detectó ningún ítem DO del plan sin implementar. Las ausencias grandes (billing, métricas Prometheus, sub-clientes SDK) están todas declaradas como DEFER/SKIP con motivo — no deben reportarse como faltas.

---

## 7. Tests

**~469 tests, todos verdes** (ejecutados en esta sesión). Distribución: 296 unitarios embebidos + 20 archivos de integración cubriendo cada capa: e2e L0→L3→assembly (6), ingest completo con state machine (15), l1_dedup con fakes (16), recall dual-pool (17), scenes/tools/navigation, persona, seed idempotencia, generation_log, pipeline_manager, llm_runner contract. Calidad alta: tests de contrato (roundtrips wire), tests de degradación explícitos, FakeClock elimina sleeps en timers, polling en vez de sleeps fijos donde hay async real.

Huecos:
1. No hay test de crash entre el `delete` y el `put` del merge L1 (M-1) — difícil de testear tal cual el diseño actual; cambiaría si se adopta el reorder propuesto.
2. No hay test multi-hilo de `AutoCaptureHook` sobre una misma sesión (M-2).
3. `tests/conversation_hook.rs` corre 0 tests (archivo presente sin casos activos bajo default features) — muerto sin feature `http-server`; eliminar o documentar.

---

## 8. Ponytail-audit (¿sobre-ingeniería?)

Es un port fiel de TDAM y eso trae algo de equipaje:

- `yagni:` triple manager de pipeline (`pipeline_manager` + `stateful_pipeline_manager` + `pipeline_factory`) — consolidar a uno canónico. [`utils/`]
- `delete:` `tests/conversation_hook.rs` sin tests activos en build default. [`tests/`]
- `shrink:` `utils/sanitize.rs` es 50% re-export consolidation (`#[allow(unused_imports)]` incluidos) — cuando todos los callers migren, colapsar a imports directos. [`utils/sanitize.rs:17-31`]
- `yagni:` `IngestConfig::global_llm_concurrency` clamp 1..=20 que regula... un worker serial (límite trivialmente honrado, auto-documentado en `mod.rs:6-12`). Mantener solo si el extraction concurrente está planeado; sino es config muerta. [`ingest/mod.rs:88-96`]

Fuera de eso: **lean**. Las abstracciones tienen ≥2 implementaciones reales (LlmRunner: standalone/openclaw/mock; Clock: system/fake), los features gate dependencias caras correctamente (tiktoken, reqwest, fjall opt-in), y los techos conocidos están marcados con `ponytail:`/documented-ceiling comments. Net posible: ~-300 líneas, 0 deps.

---

## 9. Hallazgos consolidados

| # | Sev | Hallazgo | Evidencia |
|---|---|---|---|
| M-1 | 🟡 | Merge/update L1: delete-targets antes de put-merged sin atomicidad → ventana de pérdida en crash | `core/record/l1_writer.rs:142-148` |
| M-2 | 🟡 | Cursor L0 sin locking propio: single-writer-per-session implícito pero no documentado en el hook público | `core/conversation/l0_recorder.rs:145-212` |
| M-3 | 🟡 | Tres managers de pipeline solapados sin señalización de cuál es canónico | `utils/{pipeline_manager,stateful_pipeline_manager,pipeline_factory}.rs` |
| N-1 | 🟢 | Detección de cambios por comparación RFC3339 lexicográfica — válido solo con formato fixed-width actual | `persona_generator.rs:12-13` |
| N-2 | 🟡 | Transporte MCP para scene/knowledge handlers no existe (handlers sí) — gap vs narrativa F7 | `gateway/knowledge_handlers.rs` + grep core: 0 wiring |
| N-4 | 🟢 | `LocalStateBackend` volátil: buffers/timers/cola se pierden en shutdown sin flush del host | `utils/local_backend.rs` |

## 10. Score y desglose

| Eje | Puntaje | Comentario |
|---|---|---|
| Correctez | 9 | Idempotencia por cursores, degradación P4 sistemática, guards anti-corrupción |
| Seguridad | 8 | Sanitización de fronteras consistente, escape de boundaries XML en persona, detección de injection (no cableada a gates — paridad TDAM) |
| Arquitectura | 9 | Layering limpio, host-neutral, features bien gated, grafo acíclico |
| Readability | 8.5 | Doc-comments trazan cada decisión a TDAM/MEM-nn; triple manager confunde |
| Performance | 8 | Full-scan recall documentado con upgrade path (OK a escala sesión); joins precomputados |
| Tests | 9.5 | ~469 verdes, cobertura de degradación y contratos excepcional |
| **Global** | **8.5** | |

---

## Trazabilidad Backlog

Derivado a la fase **P32** de `docs/Backlog.md` (2026-08-23):

| Hallazgo | Tarea |
|---|---|
| M-1 — Merge/update L1 no atómico (delete targets antes de put merged) | **MOD-32** |
| M-3 — Tres managers de pipeline solapados sin señalizar el canónico | **MOD-33** |
| N-2 — Handlers scene/knowledge completos sin transporte MCP (gap vs narrativa F7) | **MOD-34** |
| M-2, N-1, N-4 — nits (cursor L0 single-writer sin documentar, RFC3339 lexicográfica frágil, `LocalStateBackend` volátil) | **MOD-35** |
