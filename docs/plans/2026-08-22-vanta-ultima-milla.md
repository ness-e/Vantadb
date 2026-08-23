# Plan de Ejecución: Vanta Última Milla — integración producto end-to-end

> **Inicio:** 2026-08-22
> **Estado:** ⏳ EN PROGRESO (3/10 tareas)
> **Fuente:** auditoría de integración final (`docs/reviews/2026-08-22-auditoria-integracion-final.md`) + decisiones del usuario (2026-08-22)
> **Predecesores:** P27+P29+P30+P31+P32 ✅ (54 tareas) — roadmap TDAM 100% + bindings
> **Modo:** waves por dependencias. Sin release durante la campaña (decisión usuario).

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 10 |
| 🟡 DEFER | 0 |
| ❌ SKIP | 0 |
| 🔴 BLOQUEADO | 0 |

**Objetivo:** cerrar los 3 huecos críticos 🔴 y las brechas medias que impiden que el producto funcione como sistema end-to-end: write-back conectado, executor agéntico de tools en el proxy (O2 interceptor stream), wiki con fachada productiva, desktop con acceso al pipeline, skills HTTP, conversation/add→L1, tiktoken opt-in, Langfuse hook y parser claude-code.

**Decisiones fijadas (no re-debatir):**
- **D46 (H2 diseño):** executor = **interceptor de stream O2** — parsear SSE acumulando deltas; si el response contiene tool_use `vanta_memory_capture/search` → ejecutar server-side (capture: fire-and-forget vía WriteBack; search: recall síncrono) → sintetizar tool_result → re-request upstream con resultado anexado → loop hasta sin memory-tools o máx 3 iteraciones → streamear SOLO el response final al cliente. Requests con nuestras tools anunciadas pierden streaming incremental durante el loop interno (trade-off aceptado).
- **D47:** capture tool ejecuta vía WriteBack::track (H1) — un solo camino para writes L0.
- **D48:** límite duro de 3 iteraciones del loop agéntico + timeout 600s heredado.
- **Principios vigentes:** P4 · P7 · D34 auth obligatoria · D29 inyección system-prompt-only (los tools SON la excepción dinámica que O2 habilita correctamente) · verify mecánico del lead con `--all-targets` por tarea · SARL completo.

Status: ⬆️ uphill = 0 (todas las decisiones cerradas) · ⬇️ downhill = ~30 steps

---

## Tasks

### Task 1: MEM-50 — Wire WriteBack::track al request path (H1)
- **Appetite:** max ½d
- **Esfuerzo:** 🟢 | **Prioridad:** 🔴
- **Archivos clave:** `vanta-proxy/src/handlers/{openai,anthropic,responses}.rs` (editar), `writeback.rs` (API si falta)
- **Verificación real:** ✅ AUDITORÍA — WriteBack construido+flusheado pero cero llamadas track() (auditoría H1)
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vanta-proxy --all-targets` pasa; tests D19: request completado → track() encola el turno L0 → visible en pending queue → flush lo persiste; fallo de enqueue NO rompe el forward"
- **Risk Register:** 🟢×🟠 extraer user-text del request puede necesitar MEM-57 parcial → implementar extracción mínima inline, refinar con Task 8
- **Cynefin:** 🟦 obvio
- **Estado:** ✅ COMPLETED
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-50.md`
- **Notas:** Ruta: vanta-worker.

### Task 2: MEM-51 — H2/O2 Interceptor de stream con loop agéntico de memory-tools
- **Appetite:** max 3d
- **Esfuerzo:** 🔴 | **Prioridad:** 🔴 (diseño O2 elegido por usuario)
- **Archivos clave:** `vanta-proxy/src/{memory_tools,sse_intercept}.rs` (crear), handlers (integrar)
- **Verificación real:** ✅ AUDITORÍA H2 — tools anunciadas sin ejecutor; D46/D47/D48 fijan el diseño
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vanta-proxy --all-targets` pasa; tests D19 con upstream mockeado que scripta tool_use: (a) tool_use vanta_memory_capture → ejecutado server-side vía WriteBack → tool_result sintetizado → re-request con resultado → loop ≤3 iter → response final streameado; (b) vanta_memory_search igual con recall síncrono; (c) request SIN memory-tools → passthrough byte-identical (cero overhead); (d) máx iteraciones alcanzado → corta loop y streamea último response; (e) streaming del response final intacto"
- **Pre-mortem:** (1) parsing SSE incremental frágil → acumular deltas a mensaje completo antes de detectar tool_use (solo se intercepta cuando HAY tool_use nuestro; resto pasa directo); (2) loop infinito → cap duro D48; (3) cliente espera más en turns con tools → trade-off documentado
- **Stop conditions:** appetite excedido → entregar captura-only (search queda como mem-command ⬛)
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟠×🔴 | parser SSE rompe streams normales | test (c) passthrough byte-identical como gate permanente | cada run |
  | 🟡×🔴 | loop infinito consume cuota | cap 3 + timeout global | test d |
  | 🟡×🟡 | tool_result sintético confunde al modelo | formato estándar OpenAI/Anthropic tool_result | revisión prompt |
- **Cynefin:** 🟧 complejo — probe-sense-respond, steps cortos
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 5 steps
- **Estado:** ✅ COMPLETED
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-51.md`
- **Notas:** Ruta: vanta-worker. DEPENDE de Task 1 (WriteBack::track para capture).

### Task 3: MEM-52 — Fachada productiva de ingest wiki (H3)
- **Appetite:** max ½d
- **Esfuerzo:** 🟢 | **Prioridad:** 🔴
- **Archivos clave:** `src/cli_server.rs` (ruta POST /wiki/{id}/ingest) o `vantadb-mcp/src/wiki.rs` (tool wiki_ingest) — elegir según dónde esté el consumidor natural
- **Verificación real:** ✅ AUDITORÍA H3 — WikiStore+worker listos, cero fachada
- **Gate Result:** ✅ DO
- **Contrato:** "tests D19: POST/tool dispara worker::run async → estado pending→processing→ready consultable por run_id (MEM-31) → páginas disponibles para wiki_read"
- **Risk Register:** 🟢×🟡 ingest largo bloquea handler → disparar en thread/spawn, retornar run_id inmediato
- **Cynefin:** 🟦 obvio
- **Estado:** ✅ COMPLETED
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-52.md`
- **Notas:** Ruta: vanta-worker.

### Task 4: MEM-54 — Skills CRUD en server HTTP (H5)
- **Appetite:** max ½d
- **Esfuerzo:** 🟢 | **Prioridad:** 🟠
- **Archivos clave:** `src/cli_server.rs` (rutas POST/PUT/DELETE skills)
- **Verificación real:** ✅ AUDITORÍA H5 — solo GET listing existe
- **Gate Result:** ✅ DO
- **Contrato:** "tests D19: create/update/patch/delete vía HTTP con expected_version optimistic lock (patrón MEM-06) + owner check 404 sin filtrar"
- **Risk Register:** 🟢×🟢 auth ya existe (MEM-05) — reusar middleware
- **Cynefin:** 🟦 obvio
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-54.md`
- **Notas:** Ruta: vanta-worker. Leer `.opencode/rules/server-mcp.md`.

### Task 5: BND-03 — tiktoken feature-gate `precise-tokens` (enmienda D21)
- **Appetite:** max ½d
- **Esfuerzo:** 🟢 | **Prioridad:** 🟡
- **Archivos clave:** `vanta-memory/Cargo.toml` (feature), `context_engine/token_estimator.rs` (rama tiktoken)
- **Verificación real:** ✅ ADR-029 enmienda — decisión del autor tras walkthrough
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vanta-memory` default pasa (chars/3 intacto); con `--features precise-tokens`: estimate usa tiktoken-rs, tests comparan contra valores conocidos de cl100k; CJK/código ahora precisos"
- **Pre-mortem:** peso binario solo en builds con feature; verificar WASM build sin feature sigue liviano
- **Risk Register:** 🟡×🟡 tiktoken-rs version drift vs API OpenAI → pin versión + test golden
- **Cynefin:** 🟦 obvio
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/BND-03.md`
- **Notas:** Ruta: vanta-worker. Independiente.

### Task 6: MEM-57 — Parser claude-code (classify + extract user text)
- **Appetite:** max ½d
- **Esfuerzo:** 🟢 | **Prioridad:** 🟠
- **Archivos clave:** `vanta-proxy/src/session/claude_code.rs` (crear)
- **Verificación real:** ✅ DESCARTES RE-EVAL — ~2 funciones necesarias para tráfico CC real; D26 no sustituye parsing de contenido
- **Gate Result:** ✅ DO
- **Contrato:** "tests D19 port de TDAM agent-adapters/claude-code.ts: classifyCcRequest (main/fork/sidequery vía cache_control marker) + extractLastUserText (salta system-reminder blocks)"
- **Risk Register:** 🟡×🟡 formato CC cambia entre versiones → tests con fixtures reales capturadas; techo documentado
- **Cynefin:** 🟦 obvio
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-57.md`
- **Notas:** Ruta: vanta-worker. Integra con Task 1 (extracción de user-text para write-back) y Task 2 (routing forks).
- **Notas deps:** complementa Task 1 — ideal misma wave.

### Task 7: MEM-55 — conversation/add dispara extracción L1 (H6)
- **Appetite:** max 1d
- **Esfuerzo:** 🟡 | **Prioridad:** 🟠
- **Archivos clave:** `src/cli_server.rs:2611-2624` (editar), wiring pipeline_worker
- **Verificación real:** ✅ AUDITORÍA H6 — guarda threads sin disparar L1
- **Gate Result:** ✅ DO
- **Contrato:** "tests D19: POST /conversation/add → thread guardado → tarea de extracción encolada (worker MEM-16 o spawn) → memories aparecen en l1/<session>; fallo de extracción NO falla el HTTP response (P4)"
- **Risk Register:** 🟡×🟠 acoplamiento server→vanta-memory → server ya depende de core; exponer trigger vía trait/facade aditiva en vanta-memory
- **Cynefin:** 🟨 complicado
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-55.md`
- **Notas:** Ruta: vanta-worker. Requiere runner LLM configurado para extracción real (fallback P4: skip documentado).

### Task 8: MEM-53 — Desktop IPC commands para pipeline (H4)
- **Appetite:** max 1d
- **Espec:** comandos Tauri IPC: memory_capture, memory_recall, persona_get, scenes_list/current, skills_list, wiki_status — exponiendo vanta-memory desde src-tauri hacia la UI. UI mínima opcional (los comandos son el entregable; vistas vienen después).
- **Gate Result:** ✅ DO
- **Contrato:** "comandos invocables desde frontend con roundtrip a DB embebida; tests Rust de cada command"
- **Cynefin:** 🟦 obvio
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-53.md`
- **Notas:** Ruta: vanta-worker. DEPENDE de nada — independiente. desktop/src-tauri compila standalone.

### Task 9: MEM-58 — Consolidación UI ↔ context engine real
- **Appetite:** max 1d
- **Espec:** lente CONSOLIDAR del desktop llama assemble_with_recall (vía IPC commands de Task 8 o facade propia) en vez de heurística client-side D16a. Fallback client-side preservado si backend no disponible.
- **Gate Result:** ✅ DO
- **Contrato:** "test e2e Tauri: consolidar con backend disponible usa pipeline real (report con modo/tokens); sin backend → fallback heurístico actual"
- **Cynefin:** 🟨 complicado
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-58.md`
- **Notas:** Ruta: vanta-worker. DEPENDE de Tasks 1 (engine) y 8 (IPC).

### Task 10: MEM-56 — Hook Langfuse/OTLP sobre ReportHook
- **Appetite:** max 1d
- **Espec:** crate o módulo opcional que convierte TurnReport → spans OTLP hacia Langfuse/OTel endpoint configurable. Off by default. Descarte prematuro re-evaluado (valor medio-alto, esfuerzo S).
- **Gate Result:** ✅ DO
- **Contrato:** "tests D19 con collector mockeado: turno → spans emitidos; disabled default; fallo de red nunca bloquea proxy (P4)"
- **Cynefin:** 🟦 obvio
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-56.md`
- **Notas:** Ruta: vanta-worker. DEPENDE de Task 2 (TurnReport por turno del loop). OJO: puede requerir dep OTLP — evaluar feature-gate estricto o formato OTLP-JSON manual sin SDK (ponytail).

---

## Waves

| Wave | Tasks | Nota |
|---|---|---|
| W0 | 1 (H1), 3 (H3), 4 (H5), 5 (BND-03), 6 (MEM-57) | independientes |
| W1 | 2 (H2/O2 — deps W0: write-back + parser), 7 (H6) | núcleo agéntico |
| W2 | 8 (H4 desktop), 9 (consolidación), 10 (Langfuse) | cierre |

## Checkpoints
CP1 tras W0: write-back encola + wiki ingesta + skills CRUD + tiktoken gate green
CP2 tras W1: loop agéntico O2 completo con upstream mockeado + conversation/add→L1
CP3 tras W2: desktop expone pipeline + consolidación real + spans opcionales
Cierre: skill progreso + informe + decisión release (sigue deferida)

## Lecciones aplicadas
Verify del lead SIEMPRE `--all-targets` (lección MEM-48) · SARL con feedback exacto · decisiones cerradas upfront · header del plan se corrige tras cada MCP update · sub-agentes con lista explícita de archivos existentes.

---

=== RECITATION ===
Campaign ID: (pendiente MCP)
Objetivo activo: MEM-52 fachada productiva de ingest wiki (P33 Task 3, H3)
Estado: pending ⏳
Última acción: Implementado split worker begin/execute + fachada MCP wiki_ingest/wiki_ingest_status (begin sync → std::thread con execute → run_id inmediato; registro global por run_id MEM-31); test D19 3/3; verify mecánico completo verde
Resultado: OK
Próxima acción: Lead: commit feat(MEM-52) y cerrar Task 4 (MEM-54 skills CRUD HTTP)
Contrato: por tarea — cargo check/nextest/fmt/clippy --all-targets del crate tocado exit 0 + tests D19
Próxima tarea si completa: 4
=== END RECITATION ===
