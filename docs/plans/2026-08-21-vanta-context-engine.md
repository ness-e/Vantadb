# Plan de Ejecución: Vanta Context Engine (F5) — offload + compresión + MMD

> **Campaign ID:** (asignado por MCP al primer update_task_state)
> **Inicio:** 2026-08-21
> **Estado:** ⏳ EN PROGRESO (0/9 tareas)
> **Fuente:** `docs/Backlog.md` filas MEM-22..24, 37..42 + `docs/research/tdam/05-offload.md` + `SYNTHESIS.md` §2.2/§3 + auditoría post-P27 (2026-08-21) + decisiones del usuario (2026-08-21)
> **Predecesor:** `docs/plans/archive/2026-08-18-vanta-memory.md` (P27, F1-F4 ✅ 24/24 — crate vanta-memory con L0/L1/L2/L3/recall/offload-cursor/gateway, suite 364/364)
> **Modo:** waves por dependencias — Wave 0 (fundaciones independientes) → Wave 1 (MEM-22 núcleo) → Wave 2 (consumidores de MEM-22) → Wave 3 (gate docs).

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 9 |
| 🟡 DEFER | 10 (MEM-25..27 F6-proxy, MEM-28..33 F7-wiki, MEM-36 SDK sub-clientes → campañas propias) |
| ❌ SKIP | 0 |
| 🔴 BLOQUEADO | 0 |

**Objetivo:** la killer feature de contexto (research 05): `assemble(msgs, ratio) → {messages, report}` con compresión local LLM-free mild/aggressive/emergency, MMD Mermaid como memoria de tarea persistente, recall cross-sesión híbrido, provenance log y GC de artefactos — cerrando con el gate docs/ADR pre-release.

**Decisiones fijadas por este plan (no re-debatir en DISCOVERY):**
- **D21:** token estimator = `chars/3` configurable, SIN tiktoken (ponytail rung 3; calibración drift solo si benchmarks post-MEM-22 lo justifican — TDAM compact calibra factor 0.5–3.0 con drift >15%, research 05 §3).
- **D22:** recall_scope = `session|agent|team`, **default `agent`** (investigación MEM-40 2026-08-21: TDAM es global-scope de facto — auto-recall.ts:538/596/505 nunca pasa IsolationFilter; intención documentada en profile_sync.rs:22-23 "memory accumulates across sessions"; el híbrido evita su fuga inter-agente).
- **D23 (uphill de MEM-24):** formato MMD = Mermaid literal (TDAM) vs contrato META reutilizado (propuesta SYNTHESIS §2.2) — decidir en DISCOVERY de MEM-24 con evidencia; default lean: META.
- **Vía única:** engine local en `vanta-memory` (research 05 §6: NO copiar dual plugin/servidor). L1/L1.5/L2 LLM opcionales vía trait `LlmRunner` existente; compresión local 100% LLM-free.
- **Principios heredados P27 (vigentes):** P2 persistencia vía SDK VantaDB · P4 LLM opcional (fallo → degrada sin perder datos) · P7 prompts reescritos en inglés · sanitización namespace `[A-Za-z0-9._/-]` ≤128B / keys ≤512 sin NUL · D19 tests dedicados por tarea · sin unwrap/expect en producción · sin deps nuevas.

Status: ⬆️ uphill = 2 (formato MMD D23; performance de list_namespaces con muchas sesiones en MEM-40) · ⬇️ downhill = ~30 steps estimados

---

## Tasks

### Task 1: MEM-23 — Token estimator + emergency truncate + report types
- **Appetite:** max 1d
- **Esfuerzo:** 🟢 | **Prioridad:** 🔴 (fundación de MEM-22)
- **Archivos clave:** `vanta-memory/src/context_engine/token_estimator.rs` (crear), `vanta-memory/src/context_engine/types.rs` (crear), `vanta-memory/src/context_engine/mod.rs` (crear)
- **Verificación real:** ✅ CÓDIGO-REAL — `context_engine/` NO existe (Test-Path False); TDAM refs verificadas: `fast-token-estimate.ts` (274L), emergency trunca ~2000 chars (`llm-input-l3.ts:968,:121`), report `{messages, report}` (`compaction-handler.ts:254`)
- **Gate Justificación:** fundación sin dependencias; gap real (no existe nada del context engine); D21 decide estimator chars/3 sin deps
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vanta-memory` pasa; tests D19: estimate_tokens determinista (casos conocido vacío/ascii/unicode), truncado respeta pares tool_call/tool_result (nunca los parte), CompactionReport serde roundtrip"
- **Pre-mortem:** (1) estimator chars/3 subestima CJK → documentar techo, D21 lo acepta; (2) truncar pares tool_call rompe wire OpenAI/Anthropic → guard adjustForToolCallPair desde el día 1 (TDAM mmd-injector.ts:231)
- **Stop conditions:** appetite 1d excedido sin estimator+truncate+report green → ⬛ CANCELADO y partir en 2 tareas
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟡×🟠 | chars/3 impreciso para código (tokens reales ≠ prosa) | configurable via Config; calibración diferida post-MEM-22 | drift >15% medido |
  | 🟢×🔴 | partir tool_call pair corrompe historial | test dedicado de par intacto | primer run del test |
- **Cynefin:** 🟦 obvio — algoritmos conocidos, port directo
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 3 steps
- **DoD task:** check+nextest+fmt+clippy exit 0 · task file sync · recitation
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-23.md`
- **Branch:** | **Commit:**
- **Iteraciones:** | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |
- **Notas:** Ruta del sub-agente: vanta-worker. TDAM refs: `offload-client/token-estimator.ts` (196L, o200k_base — NO portar tiktoken), `offload/hooks/llm-input-l3.ts:755,848,968` (emergency), `offload_server/compact/compressor.ts` resolveLevel.

### Task 2: MEM-40 — Recall scope híbrido (session|agent|team, default agent)
- **Appetite:** max 1d
- **Esfuerzo:** 🟡 | **Prioridad:** 🟠
- **Archivos clave:** `vanta-memory/src/core/hooks/auto_recall.rs` (editar), `vanta-memory/src/core/config o types` (flag), `vanta-memory/tests/recall.rs` (extender)
- **Verificación real:** ✅ CÓDIGO-REAL — `read_session_records(db, params.session_key)` en auto_recall.rs:169 (scope session-only confirmado); `search_multi`/`search_all` existen en `src/sdk/search/multi.rs:20,76`; records L1 ya llevan team_id/agent_id (l1_reader.rs:152-155); ⚠️ search_multi sin covering tests (codegraph)
- **Gate Justificación:** investigación completa (ses_fdd473cf2ffeN1mI6nhXfxAbCF) con recomendación C-híbrido validada contra fuente TDAM; costo bajo sin migración de datos
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vanta-memory` pasa; tests D19: (a) scope=session replica comportamiento actual; (b) scope=agent encuentra memories de otra sesión del mismo agent_id; (c) scope=team filtra por team_id; (d) aislamiento: agent A NO ve memories del agent B en scope=agent; (e) test de search_multi (primer covering test)"
- **Pre-mortem:** (1) listar namespaces `l1/*` con miles de sesiones es lento → medir con benchmark simple antes de mergear; (2) contaminación inter-agente si team_id/agent_id no se escribieron en records viejos → fallback: records sin metadata solo visibles en scope=session
- **Stop conditions:** si list_namespaces resulta O(n) inviable con >1000 sesiones → degradar a índice de sesiones por agente (re-planear approach)
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟡×🔴 | fuga cross-agente (privacidad multi-tenant) | test de aislamiento (d) como gate; filtro por metadata obligatorio | primer fallo del test d |
  | 🟡×🟡 | performance list_namespaces | benchmark simple en DISCOVERY; índice si >umbral | benchmark lento |
  | 🟢×🟡 | records legacy sin team_id/agent_id | visibilidad solo session-scope para esos records | diseño |
- **Cynefin:** 🟨 complicado — diseño ya resuelto por investigación; ejecución analizable
- **Uphill/Downhill:** ⬆️ 0 (resuelto por investigación) · ⬇️ 4 steps
- **DoD task:** estándar + test de aislamiento verde
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-40.md`
- **Branch:** | **Commit:**
- **Iteraciones:** | — | — | — | — |
- **Notas:** Ruta: vanta-worker. Diseño D22: flag `recall_scope` en config del crate; implementación = enumerar namespaces `l1/*` cuyo scope matchee + `search_multi` + post-filtro por metadata del hit. Default `agent`. TDAM evidencia: auto-recall.ts:538/596/505 sin IsolationFilter; isolation.ts:159-171 define los 6 ejes.

### Task 3: MEM-41 — Generation-log provenance (L1/L2/L3 consultable)
- **Appetite:** max 1d
- **Esfuerzo:** 🟢 | **Prioridad:** 🟡
- **Archivos clave:** `vanta-memory/src/core/memory_generation_log/mod.rs` (crear), `store.rs` (crear), hooks de escritura en l1_writer/scene_extractor/persona_generator (editar, aditivo)
- **Verificación real:** ✅ CÓDIGO-REAL — módulo NO existe; TDAM ref: `core/memory-generation-log/{store,best-effort,types}.ts` (277L total); decisión usuario 2026-08-21: implementar en F5
- **Gate Justificación:** decisión explícita del usuario; módulo chico (189L TDAM); complementa MEM-34 (métricas sí, provenance no)
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vanta-memory` pasa; tests D19: (a) cada generación L1/L2/L3 exitosa registra entry {layer, status, anchor_id, session, ts}; (b) fallo LLM registra status=failed (best-effort: nunca bloquea el pipeline — Principio 4); (c) consulta por session/layer devuelve ordenado por ts"
- **Pre-mortem:** (1) logging dentro del hot path de writes agrega latencia → best-effort fire-and-forget como TDAM best-effort.ts; (2) crecimiento ilimitado del log → TTL o cap por sesión (reusar patrón cursor)
- **Stop conditions:** si el wiring toca >5 archivos existentes → reducir a API standalone sin hooks automáticos (integración manual)
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟡×🟡 | log falla → rompe pipeline (viola P4) | best-effort: error silencioso + tracing warn | test de fallo de store |
  | 🟢×🟡 | crecimiento sin límite | cap N entries por sesión (keep-recent) | diseño |
- **Cynefin:** 🟦 obvio
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 3 steps
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-41.md`
- **Branch:** | **Commit:**
- **Iteraciones:** | — | — | — | — |
- **Notas:** Ruta: vanta-worker. Namespace sugerido: `genlog/<session>` (sanitizado). TDAM store.ts:172L es la referencia de schema.

### Task 4: MEM-39 — Seed/import CLI (skills/persona iniciales)
- **Appetite:** max 1d
- **Esfuerzo:** 🟡 | **Prioridad:** 🟡
- **Archivos clave:** `vanta-memory/src/seed/mod.rs` (crear), `input.rs` (crear), `src/cli.rs` o subcomando (editar)
- **Verificación real:** ✅ CÓDIGO-REAL — módulo seed NO existe; TDAM ref: `core/seed/{input,seed-runtime,types}.ts` (924L); hueco de triage confirmado por auditoría (research 09:37,50 lo marcó referencia sin agendar); decisión usuario: diferir a F5 = esta campaña
- **Gate Justificación:** útil para onboarding/tests con datos reales; decisión del usuario lo agenda acá
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vanta-memory` pasa; CLI/subcomando importa un JSON/YAML de seed (skills + persona inicial) a namespaces sanitizados; idempotente por content-hash (re-import no duplica — patrón MEM-06/MEM-17); tests D19 con archivo temporal"
- **Pre-mortem:** (1) formato de input inventado sin paridad TDAM → leer input.ts primero y portar el schema mínimo; (2) CLI toca crate core (src/cli.rs) → mantener el parser en vanta-memory y solo el glue en cli
- **Stop conditions:** si el schema TDAM resulta acoplado a Mongo/OpenClaw → diseñar schema propio mínimo y documentar desviación
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟡×🟢 | schema TDAM no portable | schema propio mínimo documentado | DISCOVERY |
  | 🟢×🟡 | doble import duplica skills | content-hash idempotencia (patrón existente) | test replay |
- **Cynefin:** 🟦 obvio
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 3 steps
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-39.md`
- **Branch:** | **Commit:**
- **Iteraciones:** | — | — | — | — |
- **Notas:** Ruta: vanta-worker. Ponytail: importar SOLO skills+persona (los otros seeds de TDAM son host-specific).

### Task 5: MEM-22 — Context Engine: assemble + cascada mild/aggressive
- **Appetite:** max 3d
- **Esfuerzo:** 🔴 | **Prioridad:** 🔴 (killer feature F5)
- **Archivos clave:** `vanta-memory/src/context_engine/engine.rs` (crear), `compressor.rs` (crear), `vanta-memory/tests/context_engine.rs` (crear)
- **Verificación real:** ✅ CÓDIGO-REAL — engine NO existe; depende de Task 1 (estimator/report); TDAM refs verificadas: cascade MIN=10/INITIAL=7/FLOOR=1 (`llm-input-l3.ts:113-115`), compressByScoreCascade (:402), skip si summaryLength > originalLength (:530-538), aggressive one-shot con fingerprint role+200chars (`index.ts:121-129`, re-aplicado :1484-1520), boundary `_lastAggressiveBoundary` (`state-manager.ts:96-101`), ratio<0.5 skip (`context-engine.ts:477`)
- **Gate Justificación:** killer contexto del SYNTHESIS §3 (F5 🔥); dependencia MEM-23 satisfecha en Wave 1
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vanta-memory` pasa; tests D19: (a) assemble ratio<0.5 → skip sin tocar mensajes; (b) mild cascade conserva los top-score hasta bajar del presupuesto, nunca parte pares tool_call; (c) summary más largo que original se revierte (guard TDAM); (d) aggressive one-shot baja bajo umbral y el fingerprint del boundary hace idempotente la re-aplicación; (e) report expone modo/msgs conservados/tokens antes-después; (f) 100% LLM-free (sin runner)"
- **Pre-mortem:** (1) cascade score mal calibrado comprime lo importante → portar scoring exacto de llm-input-l3.ts:402 antes de innovar; (2) fingerprint frágil ante edits de 1 char → documentar semántica TDAM (role+200chars) y respetarla; (3) interacción con cursor MEM-20 (mensajes ≤ cursor no se recomprimen) → integrar OffloadStateManager en el assemble
- **Stop conditions:** appetite 3d excedido → ⬛ CANCELADO, entregar mild-only (aggressive a tarea propia)
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟠×🔴 | compresión pierde detalle crítico (trade-off 05 §7) | refs re-leen a demanda; documentar trade-off en docs del módulo | diseño |
  | 🟡×🔴 | regresión en prompt-cache (prepend inestable) | compression solo toca historial viejo, prepend/append intactos | test de estabilidad |
  | 🟡×🟠 | interacción cursor MEM-20 mal manejada | test específico cursor+compress | wave 2 |
  | 🟢×🟠 | estimator D21 impreciso distorsiona ratios | ratios configurables; calibración opcional | benchmarks |
- **Cynefin:** 🟨 complicado — algoritmos TDAM conocidos y verificados; port analizable paso a paso
- **Top 3 riesgos:** (1) pérdida de detalle sin refs, (2) rotura prompt-cache, (3) cursor interaction
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 6 steps
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-22.md`
- **Branch:** | **Commit:**
- **Iteraciones:** | — | — | — | — |
- **Notas:** Ruta: vanta-worker. DEPENDE de Task 1 (MEM-23). Firma objetivo: `assemble(msgs, ratio) -> {messages, report}` (adaptada, no literal — research 05 §6). LLM-free total en esta tarea (L1/L1.5 LLM-driven es post-MEM-24 si aplica).

### Task 6: MEM-24 — MMD como memoria de tarea persistente
- **Appetite:** max 1d
- **Esfuerzo:** 🟡 | **Prioridad:** 🟠
- **Archivos clave:** `vanta-memory/src/context_engine/mmd.rs` (crear), `mmd_injector.rs` (crear), persistencia namespace `mmd/<session>`
- **Verificación real:** ✅ CÓDIGO-REAL — mmd.rs NO existe; TDAM refs: ACTIVE siempre / HISTORY solo post-aggressive (`mmd-injector.ts:28-31`), marker `_mmdContextMessage` (:20), dedup fingerprint `${len}:${slice(0,64)}` (:372-374), presupuesto 4000 chars (`l2-prompt.ts:7`); ⬆️ D23: formato Mermaid literal vs contrato META — decidir en DISCOVERY
- **Gate Justificación:** mitad del swap compresión↔re-inyección (la otra mitad es MEM-22); propuesta SYNTHESIS de reusar META contract reduce trabajo
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vanta-memory` pasa; tests D19: (a) MMD activo se inyecta en assemble post-aggressive; (b) dedup por fingerprint no re-inyecta el mismo MMD; (c) pares tool_call nunca partidos al insertar el marker; (d) persistencia sobrevive reopen (namespace mmd/<session>)"
- **Pre-mortem:** (1) D23 sin decidir bloquea DISCOVERY → default lean META (D23) salvo evidencia fuerte por Mermaid literal; (2) inyección MMD rompe budget del recall → coordinar budget con MEM-18 (prepend)
- **Stop conditions:** DISCOVERY no puede decidir D23 en <2 iteraciones → escalar al usuario con ambas opciones bosquejadas
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟡×🟠 | D23 mal elegida → reescritura | META default; Mermaid solo si hay caller que lo renderice | DISCOVERY |
  | 🟡×🟡 | MMD infla el prompt | presupuesto 4000 chars TDAM + budget compartido con recall | test budget |
- **Cynefin:** 🟨 complicado — una decisión de diseño abierta (D23), resto mecánico
- **Top 3 riesgos:** formato, budget, dedup
- **Uphill/Downhill:** ⬆️ 1 (D23) · ⬇️ 4 steps
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-24.md`
- **Branch:** | **Commit:**
- **Iteraciones:** | — | — | — | — |
- **Notas:** Ruta: vanta-worker. DEPENDE de Task 5 (MEM-22). Generación del contenido MMD: LLM opcional vía LlmRunner (prompt inglés P7); LLM-free fallback = summary determinístico del compressed block.

### Task 7: MEM-37 — Integración offload↔recall (budget + cursor compartidos)
- **Appetite:** max 1d
- **Esfuerzo:** 🟢 | **Prioridad:** 🟠
- **Archivos clave:** `vanta-memory/src/core/hooks/auto_recall.rs` (editar), `vanta-memory/src/context_engine/engine.rs` (editar), tests (extender)
- **Verificación real:** ✅ CÓDIGO-REAL — ambos módulos existen (auto_recall.rs, engine.rs de Task 5); la fila dice "ajuste post-bootstrap — puede fusionarse en MEM-22"; auditoría e2e_flow.rs ya encadena el flujo base
- **Gate Justificación:** cierra el círculo compresión→re-inyección respetando budget y cursor; barato sobre piezas existentes
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vanta-memory` pasa; tests D19: (a) tras compresión aggressive, las memories re-inyectadas por recall respetan budget total (recall + MMD + memories ≤ budget); (b) mensajes ≤ cursor MEM-20 no se recomprimen ni duplican; (c) e2e_flow extendido: compress → recall en el mismo flujo"
- **Pre-mortem:** (1) doble budget (recall vs compression) pelean → un solo budget coordinator en assemble; (2) scope cruzado con MEM-40 (si recall_scope=agent, ¿la compresión también?) → no: compresión es per-sesión por definición
- **Stop conditions:** si requiere reescribir assemble → fusionar en MEM-22 y cancelar esta tarea (⬛, la fila misma lo anticipa)
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟡×🟡 | budgets duales inconsistentes | budget coordinator único | diseño |
  | 🟢×🟢 | solapamiento con MEM-22 | fusionar si el diff supera ~100 líneas | DISCOVERY |
- **Cynefin:** 🟦 obvio
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 2 steps
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-37.md`
- **Branch:** | **Commit:**
- **Iteraciones:** | — | — | — | — |
- **Notas:** Ruta: vanta-worker. DEPENDE de Tasks 5 (+2 si scope interactúa).

### Task 8: MEM-42 — Reclaimer GC de artefactos offload
- **Appetite:** max 1d
- **Esfuerzo:** 🟢 | **Prioridad:** 🟢
- **Archivos clave:** `vanta-memory/src/offload/reclaimer.rs` (crear), tests
- **Verificación real:** ✅ CÓDIGO-REAL — reclaimer.rs NO existe; OffloadStateManager existe (state_manager.rs:35-87); TDAM ref: `reclaimer.ts` (416L): 5 pasos por mtime, retentionDays < 3 desactiva (:75-78), ciclo 5min inicial + 24h
- **Gate Justificación:** decisión usuario post-MEM-22; TTL cubre expiración básica pero no GC por antigüedad de entradas comprimidas
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vanta-memory` pasa; tests D19: (a) entradas offload más viejas que retention_days se eliminan; (b) retention_days < 3 desactiva el reclaimer (paridad TDAM); (c) el cursor lastOffloadedToolCallId nunca apunta a entradas GC-eadas (consistencia); (d) GC LLM-free y seguro ante crash (idempotente)"
- **Pre-mortem:** (1) GC borra entradas que L1 aún va a procesar → solo GC-ear entradas ya consumidas (post-cursor); (2) timestamps por mtime no existen en record store → usar updated_at del record
- **Stop conditions:** si requiere cambios de schema en OffloadEntry → deferir a post-F5 con ADR
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟡×🔴 | GC borra data aún referenciada | solo post-cursor + test de consistencia cursor | test c |
  | 🟢×🟡 | GC no idempotente tras crash | operación delete-by-key naturalmente idempotente | diseño |
- **Cynefin:** 🟦 obvio
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 3 steps
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-42.md`
- **Branch:** | **Commit:**
- **Iteraciones:** | — | — | — | — |
- **Notas:** Ruta: vanta-worker. DEPENDE de Task 5 (artefactos existen). Trigger del GC: manual/API en esta iteración (el timer periódico es de orquestación MEM-16 si aplica después).

### Task 9: MEM-38 — Docs + ADR gate de cierre F5
- **Appetite:** max 1d
- **Esfuerzo:** 🟡 | **Prioridad:** 🟠 (gate pre-release)
- **Archivos clave:** `docs/architecture/adr/` (NNN_vanta_memory_context_engine.md), `docs/api/` (módulos nuevos), ROADMAP
- **Verificación real:** ✅ REAL — deuda arrastrada de MEM-12 (sync docs/api entity::scene → MEM-38) + P27 dejó ADR del crate pendiente; validate-docs-coverage.ps1 disponible
- **Gate Justificación:** Regla 3 (docs sync same PR acumuladas de F4+F5) + gate declarado "antes de release F5" en el plan P27
- **Gate Result:** ✅ DO
- **Contrato:** "`pwsh scripts/validate-docs-coverage.ps1` → 0 gaps; ADR del crate vanta-memory (LLM-driven, trait sync, trade-offs heat/compresión/MMD, decisiones D21-D23) publicado; docs/api de context_engine + generation_log + seed + cambios scene"
- **Pre-mortem:** (1) ADR escrito por IA viola forcing function (Regla 5: el humano articula) → IA aporta evidencia/borrador técnico, el autor humano edita y aprueba; (2) alcance docs inflándose → solo superficies públicas nuevas
- **Stop conditions:** gaps de cobertura no cerrables en appetite → listar deuda docs explícita en el ADR y cerrar
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟡×🟡 | ADR sin revisión humana | PR review del autor antes de merge | fin de tarea |
  | 🟢×🟡 | superficie doc grande (4 tareas nuevas) | solo APIs públicas; internals a docstrings | alcance |
- **Cynefin:** 🟦 obvio
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 3 steps
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-38.md`
- **Branch:** | **Commit:**
- **Iteraciones:** | — | — | — | — |
- **Notas:** Ruta: vanta-docs (documentación) con verify del lead. Incluye deuda docs/api de MEM-12 (entity::scene).

---

## DEFERIDOS a campañas posteriores (no en este plan)

| IDs | Campaña futura | Motivo |
|---|---|---|
| MEM-25..27 | F6 vanta-proxy | binario independiente; alta adopción pero no bloquea F5 |
| MEM-28..33 | F7 wiki/knowledge | usa graphrag existente; medio valor |
| MEM-36 | bindings SDK | sub-clientes TS/Python; backward-compat 100% requerido |

## Checkpoints

| # | Después de | Verificación |
|---|---|---|
| CP1 | Tasks 1+5 (estimator + engine) | suite completa + test manual: assemble comprime y reporta; commit + recitation |
| CP2 | Tasks 6+7+8 (MMD + recall-integration + GC) | e2e_flow extendido: capture→extract→compress→inject→recall; commit + recitation |
| CP3 | Task 9 | validate-docs-coverage 0 gaps + ADR publicado + release-readiness F5 |

## Lecciones aplicadas de P27 (obligatorias para la ejecución)

1. **Verify mecánico del lead tras CADA sub-agente** (atrapó corrupción en 11/12 tareas de P27) — no confiar en RESULTADO sin worktree check.
2. **SARL RESUME primera respuesta** ante resultado vacío (recuperó 6 tareas en P27 sin perder trabajo).
3. **El server MCP corrompe el header del plan file** en cada update_task_state — corregir formato tras cada cierre (o postmortem del server antes de arrancar).
4. **Sub-agentes vanta-worker: 50% primer intento en P27** — prompts con contexto de estado explícito + lista de archivos existentes reducen detenciones en silencio.
5. **Task file MEM-02 faltante** — toda tarea crea su task file en DISCOVERY, sin excepciones.

---

=== RECITATION ===
Campaign ID: (pendiente MCP)
Objetivo activo: F5 Context Engine — compresión LLM-free + MMD + recall híbrido + GC + gate docs
Estado: pending ⏳
Última acción: plan creado 2026-08-21 desde Backlog + research 05 + auditoría P27 + decisiones usuario
Resultado: —
Próxima acción: `/pipeline run docs/plans/2026-08-21-vanta-context-engine.md` (Wave 0: Tasks 1-4 en paralelo posible)
Contrato: por tarea — cargo check/nextest/fmt/clippy -p vanta-memory exit 0 + tests D19
Próxima tarea si completa: Task 1 (MEM-23)
=== END RECITATION ===
